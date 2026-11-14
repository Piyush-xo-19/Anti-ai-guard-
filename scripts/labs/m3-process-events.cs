// M3.1 Windows lab. Compile with the desktop .NET Framework C# compiler.
using System;
using System.Diagnostics;
using System.IO;
using System.Management;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Security.Principal;
using System.Text.RegularExpressions;

static class ProcessEventLab
{
    const string OptIn = "GUARD_M3_PROCESS_EVENT_LAB";

    static int Main(string[] args)
    {
        if (Environment.OSVersion.Platform != PlatformID.Win32NT ||
            Environment.GetEnvironmentVariable(OptIn) != "yes")
            return 2;

        if (args.Length != 1 || (args[0] != "--consent" && args[0] != "--etw-consent" &&
            args[0] != "--wmi-stop-consent"))
            return 2;

        using (var identity = WindowsIdentity.GetCurrent())
        using (var self = Process.GetCurrentProcess())
        {
            var principal = new WindowsPrincipal(identity);
            bool elevated = principal.IsInRole(WindowsBuiltInRole.Administrator);
            bool logUser = principal.IsInRole(new SecurityIdentifier("S-1-5-32-559"));
            bool service = identity.User.IsWellKnown(WellKnownSidType.LocalSystemSid) ||
                identity.User.IsWellKnown(WellKnownSidType.LocalServiceSid) ||
                identity.User.IsWellKnown(WellKnownSidType.NetworkServiceSid);
            Console.WriteLine(
                "TOKEN administrator_role_enabled={0} performance_log_users_role_enabled={1} service_identity={2} x64={3}",
                elevated, logUser, service, Environment.Is64BitProcess);
            if (elevated || logUser || service)
                return 2;
            if (args[0] == "--etw-consent")
                return ProbeEtwControl();

            string image = Assembly.GetExecutingAssembly().Location;
            string name = Path.GetFileName(image);
            if (!Regex.IsMatch(name, @"\Am3-probe-[0-9a-f]{32}\.exe\z"))
                return 2;
            string eventClass = args[0] == "--wmi-stop-consent"
                ? "Win32_ProcessStopTrace"
                : "Win32_ProcessStartTrace";
            return ProbeWmi(eventClass, self.Id, self.SessionId, name);
        }
    }

    // EVENT_TRACE_PROPERTIES v1 with an appended UTF-16 session name.
    [StructLayout(LayoutKind.Sequential)]
    struct WnodeHeader
    {
        public uint BufferSize, ProviderId;
        public ulong HistoricalContext;
        public long TimeStamp;
        public Guid Guid;
        public uint ClientContext, Flags;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct TraceProperties
    {
        public WnodeHeader Wnode;
        public uint BufferSize, MinimumBuffers, MaximumBuffers, MaximumFileSize;
        public uint LogFileMode, FlushTimer, EnableFlags, FlushThreshold;
        public uint NumberOfBuffers, FreeBuffers, EventsLost, BuffersWritten;
        public uint LogBuffersLost, RealTimeBuffersLost;
        public IntPtr LoggerThreadId;
        public uint LogFileNameOffset, LoggerNameOffset;
    }

    [DllImport("advapi32.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
    static extern uint StartTraceW(out ulong handle, string name, IntPtr properties);

    [DllImport("advapi32.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
    static extern uint ControlTraceW(ulong handle, string name, IntPtr properties, uint code);

    static int ProbeEtwControl()
    {
        const string Name = "Anti-AI Guard M3.1 ETW capability lab";
        const uint TracedGuid = 0x00020000;
        const uint RealTimeMode = 0x00000100;
        const uint StopTrace = 1;
        const uint AccessDenied = 5;
        var size = Marshal.SizeOf(typeof(TraceProperties));
        // ControlTrace can also return the log name. Reserve documented
        // maximum lengths for both strings even though no file is configured.
        var bytes = new byte[size + 2 * 1025 * sizeof(char)];
        var buffer = Marshal.AllocHGlobal(bytes.Length);
        try
        {
            var properties = new TraceProperties();
            properties.Wnode.BufferSize = (uint)bytes.Length;
            properties.Wnode.ClientContext = 1; // Performance counter clock.
            properties.Wnode.Flags = TracedGuid;
            properties.BufferSize = 4; // KiB.
            properties.MinimumBuffers = 2;
            properties.MaximumBuffers = 2;
            properties.LogFileMode = RealTimeMode;
            properties.LoggerNameOffset = (uint)size;
            Marshal.Copy(bytes, 0, buffer, bytes.Length);
            Marshal.StructureToPtr(properties, buffer, false);
            ulong handle;
            // This session enables zero providers and has no disk log.
            uint start = StartTraceW(out handle, Name, buffer);
            if (start != 0)
            {
                Console.WriteLine("ETW start_status={0} providers_enabled=0", start);
                return start == AccessDenied ? 3 : 1;
            }
            // Use only the handle from our successful StartTraceW call.
            uint stop = ControlTraceW(handle, null, buffer, StopTrace);
            Console.WriteLine("ETW start_status=0 stop_status={0} providers_enabled=0", stop);
            return stop == 0 ? 0 : 1;
        }
        finally
        {
            Marshal.FreeHGlobal(buffer);
        }
    }

    static int ProbeWmi(string eventClass, int parent, int session, string name)
    {
        var scope = new ManagementScope(@"\\.\root\cimv2");
        string phase = "namespace";
        try
        {
            scope.Connect();
            Console.WriteLine("WMI namespace_connected=true event_class={0}", eventClass);
            var query = new EventQuery(String.Format(
                System.Globalization.CultureInfo.InvariantCulture,
                "SELECT ProcessID, ParentProcessID, SessionID, TIME_CREATED FROM {0} " +
                "WHERE ParentProcessID = {1} AND SessionID = {2} AND ProcessName = '{3}'",
                eventClass, parent, session, name));
            var options = new EventWatcherOptions();
            options.BlockSize = 1;
            options.Timeout = TimeSpan.FromMilliseconds(400);

            // This permission probe creates no child processes. Its filter
            // selects only a direct child using this uniquely named image.
            using (var watcher = new ManagementEventWatcher(scope, query, options))
            {
                phase = "subscription";
                bool timedOut = false;
                try
                {
                    using (var unexpected = watcher.WaitForNextEvent())
                    {
                        Console.WriteLine("WMI unexpected_receipt={0}", unexpected != null);
                    }
                }
                catch (ManagementException error)
                {
                    if (error.ErrorCode != ManagementStatus.Timedout)
                        throw;
                    timedOut = true;
                }
                phase = "cleanup";
                watcher.Stop();
                Console.WriteLine("WMI stopped=true receive_timed_out={0}", timedOut);
                return timedOut ? 0 : 1;
            }
        }
        catch (ManagementException error)
        {
            Console.WriteLine("WMI_ERROR phase={0} status={1}", phase, error.ErrorCode);
            return error.ErrorCode == ManagementStatus.AccessDenied && phase != "cleanup" ? 3 : 1;
        }
        catch (UnauthorizedAccessException)
        {
            Console.WriteLine("WMI_ERROR phase={0} status=AccessDenied", phase);
            return phase == "cleanup" ? 1 : 3;
        }
    }
}
