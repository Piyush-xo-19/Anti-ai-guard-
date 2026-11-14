#Requires -Version 5.1
param([switch]$Consent)

$ErrorActionPreference = 'Stop'
if (-not $Consent) {
    Write-Error 'Pass -Consent to run the Windows event source lab.'
}
if (-not [Environment]::Is64BitOperatingSystem) {
    Write-Error 'The lab compiler and native layout require Windows x64.'
}

$repo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
$target = Join-Path $repo 'target'
if (-not (Test-Path -LiteralPath $target -PathType Container)) {
    New-Item -Path $target -ItemType Directory | Out-Null
}
$ancestor = Get-Item -LiteralPath $target
while ($ancestor) {
    if ($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw 'The lab output path must have ordinary directory ancestors.'
    }
    $ancestor = $ancestor.Parent
}
$outputDirectory = Join-Path $target ('m3-1-' + [Guid]::NewGuid().ToString('N'))
New-Item -Path $outputDirectory -ItemType Directory | Out-Null
$exe = Join-Path $outputDirectory ('m3-probe-' + [Guid]::NewGuid().ToString('N') + '.exe')
$compiler = Join-Path $env:SystemRoot 'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
$management = Join-Path (Split-Path $compiler) 'System.Management.dll'
$source = Join-Path $PSScriptRoot 'm3-process-events.cs'
& $compiler /nologo /warnaserror+ /platform:x64 ("/reference:" + $management) ("/out:" + $exe) $source
if ($LASTEXITCODE -ne 0) {
    throw 'The event lab did not compile.'
}

function Invoke-Probe([string]$Mode, [bool]$OptIn) {
    $info = New-Object Diagnostics.ProcessStartInfo
    $info.FileName = $exe
    $info.Arguments = $Mode
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    if ($OptIn) {
        $info.EnvironmentVariables['GUARD_M3_PROCESS_EVENT_LAB'] = 'yes'
    } else {
        $info.EnvironmentVariables.Remove('GUARD_M3_PROCESS_EVENT_LAB')
    }
    $process = [Diagnostics.Process]::Start($info)
    try {
        $output = $process.StandardOutput.ReadToEndAsync()
        if (-not $process.WaitForExit(12000)) {
            $process.Kill()
            if (-not $process.WaitForExit(3000)) {
                throw 'The owned capability probe did not stop.'
            }
            throw 'The capability probe timed out.'
        }
        if (-not $output.Wait(1000)) {
            throw 'The capability probe output did not close.'
        }
        [pscustomobject]@{ Mode = $Mode; ExitCode = $process.ExitCode; Output = $output.Result }
    } finally {
        $process.Dispose()
    }
}

$modes = @('--consent', '--wmi-stop-consent', '--etw-consent')
foreach ($mode in $modes) {
    $refused = Invoke-Probe $mode $false
    if ($refused.ExitCode -ne 2 -or $refused.Output) {
        throw 'The probe failed to enforce its environment opt-in.'
    }
}
$refused = Invoke-Probe '' $true
if ($refused.ExitCode -ne 2 -or $refused.Output) {
    throw 'The probe failed to enforce its command-line opt-in.'
}
'Consent controls passed'

foreach ($mode in $modes) {
    $outcome = Invoke-Probe $mode $true
    $outcome.Output.TrimEnd()
    "Mode=$mode Exit=$($outcome.ExitCode)"
    if ($outcome.ExitCode -notin 0, 3) {
        throw 'The probe returned an unexpected result.'
    }
}
$version = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
"WindowsRelease=$($version.DisplayVersion) FullBuild=$($version.CurrentBuildNumber).$($version.UBR)"
