<#
.SYNOPSIS
    Headless screenshot harness for Nanite Swarm.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (NANITE_SWARM_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Scenes map to Game::begin_capture_scene: "mainmenu" seeds the
    main menu, "research" seeds the research view, anything else (default
    "gameplay") jumps straight into the planetary/playing view.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("mainmenu", "gameplay", "research"),
    [int]$Frames = 150,
    [int]$WindowWidth = 0,
    [int]$WindowHeight = 0,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -WindowWidth $WindowWidth -WindowHeight $WindowHeight -OutputDir $OutputDir -SkipBuild:$SkipBuild
