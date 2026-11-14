/* Windows x64 M3.1 lab. Run the elevated mode in a disposable guest. */
#define WIN32_LEAN_AND_MEAN
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0A00
#endif
#include <windows.h>
#include <evntrace.h>
#include <evntcons.h>
#include <tdh.h>
#include <limits.h>
#include <stdio.h>
#include <wchar.h>

/* Some bundled MinGW headers omit this documented ETW filter type. */
#ifndef EVENT_FILTER_TYPE_EVENT_ID
#define EVENT_FILTER_TYPE_EVENT_ID 0x80000200UL
#endif

enum { FIXTURES = 3, MAX_RECEIPTS = 1024, CAPTURE_MS = 8000 };
static volatile LONG interrupted;
static WCHAR session_name[] = L"Anti-AI Guard M3.1 ETW delivery lab";
static const GUID provider = {
    0x22fb2cd6, 0x0e7b, 0x422b, { 0xa0, 0xc7, 0x2f, 0xad, 0x1f, 0xd0, 0xe7, 0x16 }
};

typedef struct {
    EVENT_TRACE_PROPERTIES properties;
    WCHAR logger[1025];
    WCHAR logfile[1025];
} TraceBuffer;

typedef struct {
    HANDLE handle;
    ULONG pid;
    ULONGLONG created;
    ULONG starts;
    ULONG stops;
    UCHAR start_version;
    UCHAR stop_version;
    ULONGLONG start_stamp;
    ULONGLONG stop_stamp;
} Fixture;

typedef struct {
    CRITICAL_SECTION lock;
    Fixture children[FIXTURES];
    TRACEHANDLE consumer;
    ULONGLONG deadline;
    ULONG session;
    ULONG parent;
    ULONG delivered;
    ULONG outside;
    ULONG decode_errors;
    ULONG unexpected_headers;
    ULONG source_status;
    BOOL capped;
} Trial;

static BOOL WINAPI request_stop(DWORD signal)
{
    if (signal != CTRL_C_EVENT && signal != CTRL_BREAK_EVENT)
        return FALSE;
    InterlockedExchange(&interrupted, 1);
    return TRUE;
}

static BOOL stop_requested(void)
{
    return InterlockedCompareExchange(&interrupted, 0, 0) != 0;
}

static void init_properties(TraceBuffer *buffer)
{
    ZeroMemory(buffer, sizeof(*buffer));
    buffer->properties.Wnode.BufferSize = sizeof(*buffer);
    buffer->properties.Wnode.ClientContext = 1;
    buffer->properties.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    buffer->properties.LoggerNameOffset = FIELD_OFFSET(TraceBuffer, logger);
    /* An offset of zero disables a disk trace. */
    buffer->properties.LogFileMode = EVENT_TRACE_REAL_TIME_MODE;
    buffer->properties.BufferSize = 16;
    buffer->properties.MinimumBuffers = 2;
    buffer->properties.MaximumBuffers = 16;
    buffer->properties.FlushTimer = 1;
}

static BOOL numeric_property(PEVENT_RECORD record, const WCHAR *name, void *value, ULONG size)
{
    PROPERTY_DATA_DESCRIPTOR property = { 0 };
    ULONG actual = 0;
    property.PropertyName = (ULONGLONG)(ULONG_PTR)name;
    property.ArrayIndex = ULONG_MAX;
    return TdhGetPropertySize(record, 0, NULL, 1, &property, &actual) == ERROR_SUCCESS &&
        actual == size &&
        TdhGetProperty(record, 0, NULL, 1, &property, size, value) == ERROR_SUCCESS;
}

static VOID WINAPI receive_event(PEVENT_RECORD record)
{
    Trial *trial = record->UserContext;
    const EVENT_HEADER *header = &record->EventHeader;
    ULONG pid = 0;
    ULONGLONG created = 0;
    ULONG session = 0;
    ULONG parent = 0;

    /* ETW can send synthetic headers alongside provider events. */
    if (!IsEqualGUID(&header->ProviderId, &provider))
        return;
    EnterCriticalSection(&trial->lock);
    if (trial->delivered == MAX_RECEIPTS || GetTickCount64() >= trial->deadline) {
        trial->capped = TRUE;
        goto done;
    }
    trial->delivered++;
    if (header->EventDescriptor.Id != 1 && header->EventDescriptor.Id != 2) {
        trial->unexpected_headers++;
        goto done;
    }
    if (!numeric_property(record, L"ProcessID", &pid, sizeof(pid))) {
        trial->decode_errors++;
        goto done;
    }

    Fixture *match = NULL;
    for (int i = 0; i < FIXTURES; i++) {
        if (trial->children[i].handle && trial->children[i].pid == pid) {
            match = &trial->children[i];
            break;
        }
    }
    if (!match) {
        trial->outside++;
        goto done;
    }
    if (!numeric_property(record, L"CreateTime", &created, sizeof(created)) ||
        header->TimeStamp.QuadPart <= 0) {
        trial->decode_errors++;
        goto done;
    }
    if (created != match->created) {
        trial->outside++;
        goto done;
    }
    if (header->EventDescriptor.Id == 1) {
        if (!numeric_property(record, L"SessionID", &session, sizeof(session)) ||
            !numeric_property(record, L"ParentProcessID", &parent, sizeof(parent)) ||
            parent != trial->parent || session != trial->session) {
            trial->decode_errors++;
            goto done;
        }
        match->starts++;
        match->start_version = header->EventDescriptor.Version;
        match->start_stamp = (ULONGLONG)header->TimeStamp.QuadPart;
    } else {
        /* The stop schema has no SessionID. It matches the owned handle's
           exact PID and creation FILETIME from GetProcessTimes. */
        match->stops++;
        match->stop_version = header->EventDescriptor.Version;
        match->stop_stamp = (ULONGLONG)header->TimeStamp.QuadPart;
    }
done:
    LeaveCriticalSection(&trial->lock);
}

static ULONG WINAPI receive_buffer(PEVENT_TRACE_LOGFILEW buffer)
{
    Trial *trial = buffer->Context;
    BOOL keep;
    EnterCriticalSection(&trial->lock);
    keep = !trial->capped && GetTickCount64() < trial->deadline;
    LeaveCriticalSection(&trial->lock);
    return keep;
}

static DWORD WINAPI consume(LPVOID context)
{
    Trial *trial = context;
    trial->source_status = ProcessTrace(&trial->consumer, 1, NULL, NULL);
    return 0;
}

static BOOL launch_fixture(Trial *trial, int index, const WCHAR *image)
{
    STARTUPINFOW startup = { 0 };
    PROCESS_INFORMATION process = { 0 };
    FILETIME create, exit_time, kernel, user;
    /* lpApplicationName is explicit. The child only reads its mode argument. */
    WCHAR fast_command[] = L"etw-delivery.exe --fixture-fast";
    WCHAR held_command[] = L"etw-delivery.exe --fixture-held";
    WCHAR *command = index == 0 ? held_command : fast_command;
    startup.cb = sizeof(startup);

    /* Serialize registration with the callback so a fast exit can be
       reconciled even if its start event arrives during CreateProcessW. */
    EnterCriticalSection(&trial->lock);
    BOOL ok = CreateProcessW(image, command, NULL, NULL, FALSE,
                            CREATE_NO_WINDOW, NULL, NULL, &startup, &process);
    if (ok) {
        Fixture *child = &trial->children[index];
        child->handle = process.hProcess;
        child->pid = process.dwProcessId;
        CloseHandle(process.hThread);
        ok = GetProcessTimes(child->handle, &create, &exit_time, &kernel, &user);
        if (ok)
            child->created = ((ULONGLONG)create.dwHighDateTime << 32) | create.dwLowDateTime;
    }
    LeaveCriticalSection(&trial->lock);
    return ok;
}

static ULONG enable_process_events(TRACEHANDLE controller)
{
    struct {
        BOOLEAN filter_in;
        UCHAR reserved;
        USHORT count;
        USHORT events[2];
    } filter = { TRUE, 0, 2, { 1, 2 } };
    EVENT_FILTER_DESCRIPTOR descriptor = { 0 };
    ENABLE_TRACE_PARAMETERS parameters = { 0 };
    descriptor.Ptr = (ULONGLONG)(ULONG_PTR)&filter;
    descriptor.Size = sizeof(filter);
    descriptor.Type = EVENT_FILTER_TYPE_EVENT_ID;
    parameters.Version = ENABLE_TRACE_PARAMETERS_VERSION_2;
    parameters.EnableFilterDesc = &descriptor;
    parameters.FilterDescCount = 1;
    return EnableTraceEx2(controller, &provider, EVENT_CONTROL_CODE_ENABLE_PROVIDER,
                          TRACE_LEVEL_INFORMATION, 0x10, 0, 1000, &parameters);
}

static int run_trial(void)
{
    Trial trial = { 0 };
    TraceBuffer buffer;
    EVENT_TRACE_LOGFILEW logfile = { 0 };
    TRACEHANDLE controller = 0;
    HANDLE thread = NULL;
    WCHAR image[32768];
    ULONG created = 0;
    ULONG exits = 0;
    ULONG start_status, stop_status, close_status = ERROR_SUCCESS;
    BOOL ready = FALSE;
    BOOL joined = TRUE;

    trial.parent = GetCurrentProcessId();
    trial.consumer = INVALID_PROCESSTRACE_HANDLE;
    DWORD image_length = GetModuleFileNameW(NULL, image, ARRAYSIZE(image));
    if (!ProcessIdToSessionId(trial.parent, &trial.session) ||
        image_length == 0 || image_length >= ARRAYSIZE(image))
        return 1;
    InitializeCriticalSection(&trial.lock);
    init_properties(&buffer);
    start_status = StartTraceW(&controller, session_name, &buffer.properties);
    printf("START_TRACE status=%lu\n", start_status);
    if (start_status != ERROR_SUCCESS) {
        DeleteCriticalSection(&trial.lock);
        return start_status == ERROR_ACCESS_DENIED ? 3 : 1;
    }

    trial.deadline = GetTickCount64() + CAPTURE_MS;
    logfile.LoggerName = session_name;
    logfile.ProcessTraceMode = PROCESS_TRACE_MODE_REAL_TIME | PROCESS_TRACE_MODE_EVENT_RECORD;
    logfile.EventRecordCallback = receive_event;
    logfile.BufferCallback = receive_buffer;
    logfile.Context = &trial;
    trial.consumer = OpenTraceW(&logfile);
    if (trial.consumer == INVALID_PROCESSTRACE_HANDLE) {
        ULONG open_status = GetLastError();
        printf("OPEN_TRACE status=%lu\n", open_status);
    } else {
        puts("OPEN_TRACE status=0");
        thread = CreateThread(NULL, 0, consume, &trial, 0, NULL);
        if (!thread)
            printf("CREATE_CONSUMER status=%lu\n", GetLastError());
        if (thread && !stop_requested()) {
            ULONG enable_status = enable_process_events(controller);
            printf("ENABLE_PROVIDER status=%lu\n", enable_status);
            ready = enable_status == ERROR_SUCCESS;
        }
    }
    if (ready && !stop_requested()) {
        for (int i = 0; i < FIXTURES && !stop_requested() && GetTickCount64() < trial.deadline; i++) {
            if (!launch_fixture(&trial, i, image))
                break;
            created++;
        }
        for (int i = 0; i < FIXTURES; i++) {
            HANDLE child = trial.children[i].handle;
            DWORD code;
            ULONGLONG now = GetTickCount64();
            if (stop_requested() || now >= trial.deadline)
                break;
            DWORD remaining = (DWORD)(trial.deadline - now);
            DWORD wait = remaining < 2000 ? remaining : 2000;
            if (child && WaitForSingleObject(child, wait) == WAIT_OBJECT_0 &&
                GetExitCodeProcess(child, &code) && code == 0)
                exits++;
        }
    }

    /* A successful stop flushes the remaining real-time buffers. It affects
       only the controller handle created by this invocation. */
    init_properties(&buffer);
    stop_status = ControlTraceW(controller, NULL, &buffer.properties, EVENT_TRACE_CONTROL_STOP);
    if (thread)
        joined = WaitForSingleObject(thread, 3000) == WAIT_OBJECT_0;
    if (trial.consumer != INVALID_PROCESSTRACE_HANDLE)
        close_status = CloseTrace(trial.consumer);
    if (thread && !joined)
        joined = WaitForSingleObject(thread, 2000) == WAIT_OBJECT_0;
    printf("CLEANUP stop=%lu close=%lu consumer_joined=%u\n",
           stop_status, close_status, (unsigned)joined);
    if (!joined) {
        /* Callbacks may still hold pointers to this stack frame. */
        fflush(stdout);
        ExitProcess(1);
    }
    if (thread)
        CloseHandle(thread);

    ULONG starts = 0;
    ULONG stops = 0;
    ULONG mismatches = 0;
    for (int i = 0; i < FIXTURES; i++) {
        Fixture *child = &trial.children[i];
        if (child->handle) {
            if (WaitForSingleObject(child->handle, 0) == WAIT_TIMEOUT) {
                TerminateProcess(child->handle, 1);
                WaitForSingleObject(child->handle, 1000);
            }
            CloseHandle(child->handle);
        }
        starts += child->starts;
        stops += child->stops;
        if (child->starts != 1 || child->stops != 1 || child->stop_stamp < child->start_stamp)
            mismatches++;
        printf("FIXTURE index=%d starts=%lu exits=%lu start_schema_version=%u stop_schema_version=%u\n",
               i + 1, child->starts, child->stops,
               (unsigned)child->start_version, (unsigned)child->stop_version);
    }
    printf("OBSERVATION launched=%lu exited=%lu starts=%lu stops=%lu unmatched_or_reordered=%lu "
           "outside_owned=%lu decode_errors=%lu unexpected_events=%lu capped=%u trace_status=%lu "
           "events_lost=%lu realtime_buffers_lost=%lu\n",
           created, exits, starts, stops, mismatches, trial.outside, trial.decode_errors,
           trial.unexpected_headers, (unsigned)trial.capped, trial.source_status,
           buffer.properties.EventsLost, buffer.properties.RealTimeBuffersLost);
    DeleteCriticalSection(&trial.lock);
    BOOL passed = ready && !stop_requested() && created == FIXTURES && exits == FIXTURES &&
        mismatches == 0 && !trial.capped && trial.decode_errors == 0 &&
        trial.unexpected_headers == 0 && trial.source_status == ERROR_SUCCESS &&
        stop_status == ERROR_SUCCESS &&
        (close_status == ERROR_SUCCESS || close_status == ERROR_CTX_CLOSE_PENDING) &&
        buffer.properties.EventsLost == 0 && buffer.properties.RealTimeBuffersLost == 0;
    printf("RESULT %s\n", passed ? "PASS" : "INCOMPLETE");
    return passed ? 0 : 1;
}

int wmain(int argc, WCHAR **argv)
{
    WCHAR consent[4];
    if (GetEnvironmentVariableW(L"GUARD_M3_ETW_DELIVERY_LAB", consent, ARRAYSIZE(consent)) != 3 ||
        wcscmp(consent, L"yes") != 0 || argc != 2)
        return 2;
    if (wcscmp(argv[1], L"--fixture-fast") == 0)
        return 0;
    if (wcscmp(argv[1], L"--fixture-held") == 0) {
        Sleep(650);
        return 0;
    }
    if (wcscmp(argv[1], L"--consent") != 0 || sizeof(void *) != 8)
        return 2;

    HANDLE token;
    TOKEN_ELEVATION elevation;
    DWORD size;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &token))
        return 1;
    BOOL known = GetTokenInformation(token, TokenElevation, &elevation, sizeof(elevation), &size);
    CloseHandle(token);
    if (!known)
        return 1;
    if (!elevation.TokenIsElevated) {
        puts("RESULT BLOCKED elevated_token_required");
        return 3;
    }
    if (!SetConsoleCtrlHandler(request_stop, TRUE)) {
        printf("CONSOLE_HANDLER status=%lu\n", GetLastError());
        return 1;
    }
    puts("TOKEN elevated=1");
    return run_trial();
}
