#Requires -Version 5.1
[CmdletBinding(SupportsShouldProcess = $true)]
Param(
    [Parameter()][Alias('h')][switch]$Help,
    [Parameter()][switch]$Build,
    [Parameter()][switch]$NoBuild,
    [Parameter()][switch]$Launch,
    [Parameter()][ValidateSet("stable", "preview", "nightly", "dev")]
    [string]$Channel = "stable",
    [Parameter()][Alias('a')][string]$Architecture,
    [Parameter()][string]$InstallDir,
    [Parameter()][switch]$NoShortcut,
    [Parameter()][switch]$Force,
    [Parameter()][int]$Jobs
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

. "$PSScriptRoot/lib/windows-env.ps1"

function Show-DeployWindowsHelp {
    Write-Output @"
Usage: deploy-windows.ps1 [-NoBuild] [-Launch] [-Channel <channel>] [-Architecture <arch>] [-InstallDir <path>] [-Force] [-WhatIf] [-Help]

Build Synth Code and install it into its own per-user folder
(%LOCALAPPDATA%\Programs\SynthCode by default), side by side with any
official Zed install. Official Zed's folder, processes and winget package are
not touched.

Options:
  -Build               Build before deploying (default).
  -NoBuild             Deploy the last release artifacts without building.
  -Launch              Start synth-code.exe after a successful deploy.
  -Channel             Release channel compiled into the binaries (default: stable).
  -Architecture, -a    x86_64 or aarch64 (default: host architecture).
  -InstallDir          Destination (default: %LOCALAPPDATA%\Programs\SynthCode).
  -NoShortcut          Do not create the "Synth Code" Start menu shortcut.
  -Force               Deploy even into a non-empty folder without synth-code.exe.
  -Jobs                Cargo parallel jobs (default: 2). Raise this on machines
                       with plenty of RAM; thin LTO can crash rustc otherwise.
  -WhatIf              Print actions without stopping processes or writing files.
  -Help, -h            Show this help message.

Examples:
  script/deploy-windows.ps1
  script/deploy-windows.ps1 -NoBuild
  script/deploy-windows.ps1 -Launch
  script/deploy-windows.ps1 -WhatIf
"@
}

if ($Help) {
    Show-DeployWindowsHelp
    exit 0
}

if ($Build -and $NoBuild) {
    throw "Use only one of -Build and -NoBuild."
}

$shouldBuild = -not $NoBuild
$hostArch = Get-ZedWindowsArch
if (-not $Architecture) {
    $Architecture = $hostArch
}
if ($Architecture -notin @("x86_64", "aarch64")) {
    throw "Unsupported architecture: $Architecture (expected x86_64 or aarch64)"
}

$triple = Get-ZedWindowsTargetTriple -Arch $Architecture
$crossCompile = $Architecture -ne $hostArch
$repoRoot = Get-ZedRepoRoot
if (-not $InstallDir) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\SynthCode"
}
$InstallDir = [System.IO.Path]::GetFullPath($InstallDir)
$zedDest = Join-Path $InstallDir "synth-code.exe"
$cliDest = Join-Path $InstallDir "bin\scode.exe"
$helperDest = Join-Path $InstallDir "tools\auto_update_helper.exe"

function Get-ZedReleaseOutDir {
    $hostOut = Join-Path $repoRoot "target\release"
    $tripleOut = Join-Path $repoRoot "target\$triple\release"
    if ($crossCompile) {
        return $tripleOut
    }
    if (Test-Path -LiteralPath (Join-Path $hostOut "synth-code.exe")) {
        return $hostOut
    }
    if (Test-Path -LiteralPath (Join-Path $tripleOut "synth-code.exe")) {
        return $tripleOut
    }
    $hostOut
}

function Test-ZedInstallLayout {
    param([string]$Dir)

    if (-not (Test-Path -LiteralPath $Dir)) {
        return $true
    }
    if (Test-Path -LiteralPath (Join-Path $Dir "synth-code.exe")) {
        return $true
    }
    @(Get-ChildItem -LiteralPath $Dir -Force).Count -eq 0
}

function Test-ZedFileUnlocked {
    param([Parameter(Mandatory = $true)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return $true
    }
    try {
        $stream = [System.IO.File]::Open(
            $Path,
            [System.IO.FileMode]::Open,
            [System.IO.FileAccess]::ReadWrite,
            [System.IO.FileShare]::None
        )
        $stream.Close()
        $true
    }
    catch {
        $false
    }
}

function Stop-ZedProcesses {
    # Only stop processes started from this install, so official Zed keeps running.
    $prefix = $InstallDir.TrimEnd("\") + "\"
    $procs = @(Get-Process -ErrorAction SilentlyContinue | Where-Object {
            $_.Path -and $_.Path.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)
        })
    if ($procs.Count -eq 0) {
        Write-Host "No running Synth Code process."
        return
    }
    $ids = ($procs | ForEach-Object { $_.Id }) -join ", "
    if (-not $PSCmdlet.ShouldProcess("Synth Code processes ($ids)", "Stop")) {
        return
    }
    Write-Host "Stopping Synth Code processes: $ids"
    $procs | Stop-Process -Force
}

function Wait-ZedInstallUnlocked {
    param(
        [string[]]$Paths,
        [int]$TimeoutSeconds = 30
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $locked = @($Paths | Where-Object { -not (Test-ZedFileUnlocked -Path $_) })
        if ($locked.Count -eq 0) {
            return
        }
        Start-Sleep -Milliseconds 400
    }
    $stillLocked = @($Paths | Where-Object { -not (Test-ZedFileUnlocked -Path $_) })
    if ($stillLocked.Count -gt 0) {
        throw "Timed out waiting for files to unlock: $($stillLocked -join ', ')"
    }
}

function Copy-ZedFileWithRetry {
    param(
        [Parameter(Mandatory = $true)][string]$Source,
        [Parameter(Mandatory = $true)][string]$Destination,
        [int]$Attempts = 8
    )

    $destDir = Split-Path $Destination -Parent
    if (-not (Test-Path -LiteralPath $destDir)) {
        if ($PSCmdlet.ShouldProcess($destDir, "Create directory")) {
            New-Item -ItemType Directory -Path $destDir -Force | Out-Null
        }
    }

    if (-not $PSCmdlet.ShouldProcess($Destination, "Copy $Source")) {
        return
    }

    if (-not (Test-Path -LiteralPath $Source)) {
        throw "Source file not found: $Source"
    }

    $lastError = $null
    for ($i = 1; $i -le $Attempts; $i++) {
        try {
            Copy-Item -LiteralPath $Source -Destination $Destination -Force
            return
        }
        catch {
            $lastError = $_
            Write-Host "Copy attempt $i failed: $($_.Exception.Message)"
            Start-Sleep -Milliseconds (300 * $i)
        }
    }
    throw "Failed to copy $Source to $Destination : $($lastError.Exception.Message)"
}

function Copy-ZedRuntimeFiles {
    # A fresh folder lacks the ConPTY / AGS runtime files that the installer bundles.
    # Copy them from the official install (read-only there) when they are missing here.
    $official = Join-Path $env:LOCALAPPDATA "Programs\Zed"
    foreach ($relative in @("conpty.dll", "amd_ags_x64.dll", "x64\OpenConsole.exe", "arm64\OpenConsole.exe")) {
        $source = Join-Path $official $relative
        $destination = Join-Path $InstallDir $relative
        if ((Test-Path -LiteralPath $source) -and -not (Test-Path -LiteralPath $destination)) {
            Copy-ZedFileWithRetry -Source $source -Destination $destination
        }
    }
}

function New-ZedStartMenuShortcut {
    $programs = [Environment]::GetFolderPath("Programs")
    $link = Join-Path $programs "Synth Code.lnk"
    if (Test-Path -LiteralPath $link) {
        Write-Host "Start menu shortcut already exists at $link"
        return
    }
    if (-not $PSCmdlet.ShouldProcess($link, "Create shortcut to $zedDest")) {
        return
    }
    Write-Host "Creating Start menu shortcut $link"
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($link)
    $shortcut.TargetPath = $zedDest
    $shortcut.WorkingDirectory = $InstallDir
    $shortcut.Save()
}

function Invoke-ZedDeployBuild {
    if ($crossCompile) {
        throw "Cross-compiling deploy from $hostArch to $Architecture is not supported. Build on the target machine."
    }

    Set-Location $repoRoot
    Initialize-ZedCargoPath
    Write-Host "==> env"
    Enter-ZedVsDevShell -Architecture $Architecture -HostArchitecture $hostArch

    $cmake = Find-ZedCmake
    if ($cmake) {
        $cmakeDir = Split-Path $cmake -Parent
        if ($env:Path -notlike "*$cmakeDir*") {
            $env:Path = "$cmakeDir;$env:Path"
        }
    }

    $env:ZED_RELEASE_CHANNEL = $Channel
    $env:RELEASE_CHANNEL = $Channel

    if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
        throw "rustup is not on PATH. Run script/windows.ps1 first."
    }
    rustup target add $triple
    if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
        throw "rustup target add $triple failed with exit code $LASTEXITCODE"
    }

    $jobCount = if ($Jobs -gt 0) { $Jobs } else { 2 }
    $buildArgs = @(
        "build", "--release", "--jobs", "$jobCount",
        "--package", "zed",
        "--package", "cli",
        "--package", "auto_update_helper"
    )
    Write-Host "Running: cargo $($buildArgs -join ' ') (channel $Channel)"
    cargo @buildArgs
    if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }
}

Set-Location $repoRoot
Initialize-ZedCargoPath

if (-not $Force -and -not (Test-ZedInstallLayout -Dir $InstallDir)) {
    throw "$InstallDir is not empty and has no synth-code.exe. Pass -Force to deploy anyway, or -InstallDir to choose another folder."
}

Write-Host "Deploy"
Write-Host "  channel:     $Channel"
Write-Host "  architecture: $Architecture"
Write-Host "  install dir: $InstallDir"
Write-Host "  build:       $shouldBuild"
Write-Host "  jobs:        $(if ($Jobs -gt 0) { $Jobs } else { 2 })"
Write-Host "  shortcut:    $(-not $NoShortcut)"
Write-Host "  launch:      $Launch"

$whatIf = [bool]$WhatIfPreference

if ($shouldBuild) {
    if ($whatIf) {
        Write-Host "Would build release packages zed, cli, auto_update_helper (channel $Channel)"
    }
    else {
        Invoke-ZedDeployBuild
    }
}

$outDir = Get-ZedReleaseOutDir
$zedSrc = Join-Path $outDir "synth-code.exe"
$cliSrc = Join-Path $outDir "cli.exe"
$helperSrc = Join-Path $outDir "auto_update_helper.exe"

$missing = @()
foreach ($pair in @(
        @{ Name = "synth-code.exe"; Path = $zedSrc },
        @{ Name = "cli.exe"; Path = $cliSrc },
        @{ Name = "auto_update_helper.exe"; Path = $helperSrc }
    )) {
    if (-not (Test-Path -LiteralPath $pair.Path)) {
        $missing += "$($pair.Name) ($($pair.Path))"
    }
}
if ($missing.Count -gt 0) {
    $message = "Missing $($missing -join ', '). Run without -NoBuild, or build those packages first."
    if ($whatIf) {
        Write-Host $message
    }
    else {
        throw $message
    }
}

Write-Host "Artifacts:"
Write-Host "  $zedSrc -> $zedDest"
Write-Host "  $cliSrc -> $cliDest"
Write-Host "  $helperSrc -> $helperDest"

Stop-ZedProcesses
if (-not $whatIf) {
    Wait-ZedInstallUnlocked -Paths @($zedDest, $cliDest, $helperDest)
}

Copy-ZedFileWithRetry -Source $zedSrc -Destination $zedDest
Copy-ZedFileWithRetry -Source $cliSrc -Destination $cliDest
Copy-ZedFileWithRetry -Source $helperSrc -Destination $helperDest
Copy-ZedRuntimeFiles

if (-not $NoShortcut) {
    New-ZedStartMenuShortcut
}

if (-not $whatIf -and (Test-Path -LiteralPath $zedDest)) {
    $info = (Get-Item -LiteralPath $zedDest).VersionInfo
    Write-Host "Installed $($info.ProductName) $($info.ProductVersion)"
}

if ($Launch) {
    if ($PSCmdlet.ShouldProcess($zedDest, "Start")) {
        Start-Process -FilePath $zedDest
    }
    else {
        Write-Host "Would launch $zedDest"
    }
}

Write-Host "Windows deploy succeeded."
exit 0
