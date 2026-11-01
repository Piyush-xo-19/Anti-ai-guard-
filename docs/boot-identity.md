# Boot identity: fixed self-process telemetry header

Implemented as an M2 diagnostic capability on 2026-11-01; Windows 10 and restart
validation remain pending. This is device-local correlation metadata, not an
attestation, hardware ID or globally unique cross-device identifier.

The source is NtQueryInformationProcess information class 64 and the BootId field
of Microsoft's documented PROCESS_TELEMETRY_ID_INFORMATION structure:

- [Query API and compatibility warning](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess)
- [Documented header](https://learn.microsoft.com/en-us/windows/win32/devnotes/process_telemetry_id_information_type)

The adapter dynamically resolves the OS export from the already loaded ntdll,
queries only its own process, and supplies exactly the 96-byte fixed header.
It never retries with a larger buffer, requests another process's telemetry,
follows a variable offset or receives the variable payload containing paths/SIDs/
arguments. The known BUFFER_OVERFLOW behavior is an empirically tested native
capability, not a promise made by the API documentation: accept it only with a
complete known header, matching self PID/creation time and all payload offsets
zero. Any unknown status, layout, length, unavailable export or identity mismatch
leaves MissingBootIdentity in place. Raw boot tokens are never projected publicly.

The token includes a versioned provider prefix and the OS BootId. It applies only
to this Windows installation; callers must not correlate records from different
devices, restored OS images or reinstalled systems using that token alone.
No uptime/wall-clock estimate or random per-scan ID substitutes for boot identity.
A normal-access Windows 11 build 26200.9711 probe returned a 96-byte complete
header with zero variable offsets and stable adjacent-query identity. Controlled
process tests exercise integration. Synthetic tests cover changed boot tokens,
short/malformed headers, error statuses and every variable-offset field.

M2 completion still requires a real restart/Fast Startup/resume/restore test
matrix and the Windows 10 VM evidence. Reboot/restore trials are not claimed from
adjacent queries or fixture comparisons. No machine restart is initiated by tests.
