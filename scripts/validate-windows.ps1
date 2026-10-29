#requires -Version 5.1
[CmdletBinding()]
param(
    [switch]$Consent,
    [string]$OutputPath = 'reports/validation/windows-fixtures.json'
)

$ErrorActionPreference = 'Stop'
if (-not $Consent) { throw 'Explicit -Consent is required for controlled fixtures and current-session metadata.' }
if ($env:OS -ne 'Windows_NT') { throw 'Windows fixture validation requires Windows.' }
$validationRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$validationReports = [IO.Path]::GetFullPath((Join-Path $validationRoot 'reports'))
$validationOutput = if ([IO.Path]::IsPathRooted($OutputPath)) { [IO.Path]::GetFullPath($OutputPath) } else { [IO.Path]::GetFullPath((Join-Path $validationRoot $OutputPath)) }
if (-not $validationOutput.StartsWith($validationReports + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetExtension($validationOutput) -ne '.json') {
    throw 'Validation output must be a JSON file below this repository reports directory.'
}
# Refuse report-directory reparse points; never redirect evidence writes through them.
$validationDirectory = [IO.DirectoryInfo]([IO.Path]::GetDirectoryName($validationOutput))
while ($validationDirectory -and $validationDirectory.FullName.StartsWith($validationReports, [StringComparison]::OrdinalIgnoreCase)) {
    if ($validationDirectory.Exists -and ($validationDirectory.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Report reparse points are not allowed.' }
    $validationDirectory = $validationDirectory.Parent
}
$validationChecks = New-Object 'System.Collections.Generic.List[object]'
$validationStarted = [DateTime]::UtcNow
$validationPlatform = $null
$validationRevision = $null
$validationDirty = $true
$validationRust = $null
$validationCargo = $null
$validationFatal = $false
$validationPhase = 'setup'

function Invoke-ValidationProgram {
    param([string]$Executable, [string[]]$ProgramArguments)
    # All arguments are fixed tokens selected below, never user-controlled shell text.
    $programInfo = New-Object Diagnostics.ProcessStartInfo
    $programInfo.FileName = $Executable
    $programInfo.Arguments = [string]::Join(' ', $ProgramArguments)
    $programInfo.WorkingDirectory = $validationRoot
    $programInfo.UseShellExecute = $false
    $programInfo.CreateNoWindow = $true
    $programInfo.RedirectStandardOutput = $true
    $programInfo.RedirectStandardError = $true
    $programInfo.EnvironmentVariables['CARGO_TARGET_DIR'] = Join-Path $validationRoot 'target'
    $program = New-Object Diagnostics.Process
    $program.StartInfo = $programInfo
    try {
        [void]$program.Start()
        $programOut = $program.StandardOutput.ReadToEndAsync()
        $programErr = $program.StandardError.ReadToEndAsync()
        if (-not $program.WaitForExit(180000)) {
            # Timeout cleanup targets ONLY this harness-owned verification tree,
            # never an observed participant process. No collector uses this API.
            $cleanup = New-Object Diagnostics.ProcessStartInfo
            $cleanup.FileName = Join-Path ([Environment]::GetFolderPath('System')) 'taskkill.exe'
            $cleanup.Arguments = '/PID ' + $program.Id + ' /T /F'
            $cleanup.UseShellExecute = $false
            $cleanup.CreateNoWindow = $true
            $cleanup.RedirectStandardOutput = $true
            $cleanup.RedirectStandardError = $true
            $cleanupProcess = [Diagnostics.Process]::Start($cleanup)
            [void]$cleanupProcess.WaitForExit(10000)
            $cleanupProcess.Dispose()
            return [pscustomobject]@{ Code = 124; Out = ''; Err = ''; TimedOut = $true }
        }
        return [pscustomobject]@{ Code = $program.ExitCode; Out = $programOut.GetAwaiter().GetResult(); Err = $programErr.GetAwaiter().GetResult(); TimedOut = $false }
    } finally { $program.Dispose() }
}

function Invoke-ValidationCheck {
    param([string]$Name, [string]$Executable, [string[]]$ProgramArguments)
    $script:validationPhase = $Name
    $checkWatch = [Diagnostics.Stopwatch]::StartNew()
    $checkResult = Invoke-ValidationProgram $Executable $ProgramArguments
    $summaries = [regex]::Matches(($checkResult.Out + "`n" + $checkResult.Err), 'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;')
    $passed = 0; $failed = 0; $ignored = 0
    foreach ($summary in $summaries) { $passed += [int]$summary.Groups[1].Value; $failed += [int]$summary.Groups[2].Value; $ignored += [int]$summary.Groups[3].Value }
    $record = [pscustomobject]@{ name = $Name; status = $(if ($checkResult.Code -eq 0) { 'PASS' } else { 'FAIL' }); exit_code = $checkResult.Code; duration_ms = $checkWatch.ElapsedMilliseconds; passed = $passed; failed = $failed; ignored = $ignored; timed_out = $checkResult.TimedOut }
    $validationChecks.Add($record)
    Write-Host ($Name + ': ' + $record.status + ' (exit ' + $record.exit_code + ')')
    # Raw stdout/stderr is never copied into an evidence artifact or printed.
    return $checkResult
}

try {
    $cargoProgram = (Get-Command cargo -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
    $rustProgram = (Get-Command rustc -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
    $gitProgram = (Get-Command git -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
    $rustResult = Invoke-ValidationProgram $rustProgram @('--version')
    $cargoResult = Invoke-ValidationProgram $cargoProgram @('--version')
    if ($rustResult.Out.Trim() -match '^rustc [0-9.]+ \([a-f0-9]+ [0-9-]+\)$') { $validationRust = $rustResult.Out.Trim() }
    if ($cargoResult.Out.Trim() -match '^cargo [0-9.]+ \([a-f0-9]+ [0-9-]+\)$') { $validationCargo = $cargoResult.Out.Trim() }
    $validationPhase = 'source-git-head'
    $revision = Invoke-ValidationProgram $gitProgram @('rev-parse', 'HEAD')
    if ($revision.Out.Trim() -match '^[a-f0-9]{40}$') { $validationRevision = $revision.Out.Trim() }
    $dirty = Invoke-ValidationProgram $gitProgram @('status', '--porcelain', '--untracked-files=normal')
    $validationDirty = $dirty.Code -ne 0 -or -not [string]::IsNullOrWhiteSpace($dirty.Out)
    [void](Invoke-ValidationCheck 'format' $cargoProgram @('fmt', '--all', '--', '--check'))
    [void](Invoke-ValidationCheck 'clippy' $cargoProgram @('clippy', '--locked', '--offline', '--all-targets', '--', '-D', 'warnings'))
    [void](Invoke-ValidationCheck 'build' $cargoProgram @('build', '--locked', '--offline'))
    [void](Invoke-ValidationCheck 'tests' $cargoProgram @('test', '--locked', '--offline'))
    [void](Invoke-ValidationCheck 'm22-synthetic-example' $cargoProgram @('run', '--locked', '--offline', '--example', 'diagnostic'))
    $exampleJson = Invoke-ValidationCheck 'm25-synthetic-json' $cargoProgram @('run', '--locked', '--offline', '--example', 'report_fixture', '--', '--json')
    $exampleText = Invoke-ValidationCheck 'm25-synthetic-text' $cargoProgram @('run', '--locked', '--offline', '--example', 'report_fixture')
    if ($exampleJson.Code -eq 0 -and $exampleText.Code -eq 0) {
        $expectedJson = [IO.File]::ReadAllText((Join-Path $validationRoot 'docs/examples/diagnostic.synthetic.json')).Replace("`r`n", "`n")
        $expectedText = [IO.File]::ReadAllText((Join-Path $validationRoot 'docs/examples/diagnostic.synthetic.txt')).Replace("`r`n", "`n")
        $matches = $exampleJson.Out.Replace("`r`n", "`n") -eq $expectedJson -and $exampleText.Out.Replace("`r`n", "`n") -eq $expectedText
        $validationChecks.Add([pscustomobject]@{ name = 'published-example-agreement'; status = $(if ($matches) { 'PASS' } else { 'FAIL' }); exit_code = $(if ($matches) { 0 } else { 1 }) })
    }
    $runtime = Invoke-ValidationProgram (Join-Path $validationRoot 'target/debug/anti-ai-guard.exe') @('processes', '--consent', '--json')
    $runtimeValid = $runtime.Code -in @(0, 3)
    if ($runtimeValid) {
        $runtimeReport = $runtime.Out | ConvertFrom-Json
        $runtimeValid = $runtimeReport.schema_version -eq 1 -and $runtimeReport.eligibility -eq 'NotIssued'
        # Only minimized platform metadata is retained; findings/PIDs/counts are discarded.
        $validationPlatform = $runtimeReport.platform
    }
    $validationChecks.Add([pscustomobject]@{ name = 'runtime-json-contract'; status = $(if ($runtimeValid) { 'PASS' } else { 'FAIL' }); exit_code = $runtime.Code })
} catch {
    $validationFatal = $true
    $validationChecks.Add([pscustomobject]@{ name = 'harness-setup-or-execution'; status = 'FAIL'; exit_code = 1 })
    $innerType = if ($_.Exception.InnerException) { $_.Exception.InnerException.GetType().FullName } else { 'none' }
    Write-Host ('Validation failed in ' + $validationPhase + ' at script line ' + $_.InvocationInfo.ScriptLineNumber + ' (' + $_.Exception.GetType().FullName + ', inner ' + $innerType + '); raw error details omitted.')
}

$validationPassed = -not $validationFatal -and @($validationChecks | Where-Object { $_.status -ne 'PASS' }).Count -eq 0
$validationRecord = [ordered]@{
    schema_version = 1
    suite = 'synthetic-windows-fixtures'
    recorded_at_utc = [DateTime]::UtcNow.ToString('o')
    git_head = $validationRevision
    working_tree_dirty = $validationDirty
    git_head_identifies_exact_source = -not $validationDirty
    toolchain = @{ rustc = $validationRust; cargo = $validationCargo }
    platform = $validationPlatform
    fixture_status = $(if ($validationPassed) { 'PASS' } else { 'FAIL' })
    checks = @($validationChecks.ToArray())
    duration_ms = [int64]([DateTime]::UtcNow - $validationStarted).TotalMilliseconds
    real_tool_desktop_cases = 'NOT RUN'
    desktop_matrix_validated = $false
    m2_exit_gate = 'INCOMPLETE'
    blockers = @('Windows 10 pinned-tool desktop lab not executed', 'Windows 11 full pinned-tool desktop lab not executed', 'Trusted strong identity catalog unavailable', 'Trusted boot identity unavailable', 'Required lab/variant scope review still open')
}
[void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($validationOutput))
[IO.File]::WriteAllText($validationOutput, ($validationRecord | ConvertTo-Json -Depth 10) + "`n", (New-Object Text.UTF8Encoding($false)))
Write-Host ('Fixture validation: ' + $validationRecord.fixture_status + '; M2 desktop acceptance: INCOMPLETE.')
Write-Host ('Sanitized evidence: ' + ($validationRecord | ConvertTo-Json -Depth 10 -Compress))
if ($validationPassed) { exit 0 } else { exit 1 }
