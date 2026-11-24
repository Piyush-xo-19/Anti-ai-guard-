# Windows 11 sleep and resume

BOOT-03 passed on Windows 11 build 26300.9974 (registry release 26H2).
The scanner reported standard-user inspection and a compiled x64 target.
The checkpoint and comparison both used clean source
`5f524bc5802c7264f42d63f5d989cd89ab943728`.

The first attempt spanned an update reboot between recording the checkpoint
and entering sleep. The boot identity differed, so this comparison could
not isolate sleep. The maintainer confirmed that the update reboot preceded
sleep. The previous checkpoint was archived privately.

A fresh checkpoint on the updated build passed the same-boot control.
The maintainer then chose Sleep and woke the computer. The System log
showed Kernel-Power events 42 and 107 after checkpoint creation, with no
Kernel-General startup or shutdown events in the comparison interval.
The resume test returned `PASS` and `same_boot=true`. The checkpoint
hash matched before and after the comparison. See Microsoft's
[power event reference](https://learn.microsoft.com/en-us/troubleshoot/azure/virtual-machines/windows/hibernate-resume-troubleshooting-windows)
and [startup event reference](https://learn.microsoft.com/en-us/troubleshoot/windows-server/performance/troubleshoot-unexpected-reboots-system-event-logs).

To reproduce, record a checkpoint on the current boot and follow the
[boot transition procedure](../../validation/boot-transition-lab.md)
with the `resume` phase. A reboot between record and comparison requires
a new checkpoint and another sleep cycle.

On the same source and build, the locked default suite passed 112 tests
and doctests, with five intentional ignores. Fast Startup is unavailable
in the current configuration because hibernation is disabled. Restore
remains pending in a disposable lab.
