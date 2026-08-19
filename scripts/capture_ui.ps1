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
    [switch]$UpdateCatalogThumbnail,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -WindowWidth $WindowWidth -WindowHeight $WindowHeight -OutputDir $OutputDir -SkipBuild:$SkipBuild

if ($UpdateCatalogThumbnail) {
    $mainMenuCapture = Join-Path $gameDir (Join-Path $OutputDir "ui_mainmenu.png")
    if (-not (Test-Path -LiteralPath $mainMenuCapture)) {
        throw "Catalog update requested, but mainmenu was not captured: $mainMenuCapture"
    }
    Copy-Item -LiteralPath $mainMenuCapture -Destination (Join-Path $gameDir "catalog_thumbnail.png") -Force
    Write-Host "Catalog thumbnail refreshed from the title-screen capture."
}
