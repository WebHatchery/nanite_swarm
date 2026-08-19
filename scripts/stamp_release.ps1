param(
    [string]$ProjectDir = (Split-Path $PSScriptRoot -Parent)
)

$ErrorActionPreference = "Stop"
$cargo = Get-Content -LiteralPath (Join-Path $ProjectDir "Cargo.toml") -Raw
$versionMatch = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $versionMatch.Success) {
    throw "Cargo.toml has no package version"
}
$version = $versionMatch.Groups[1].Value
$metadataPath = Join-Path $ProjectDir "game_page.json"
$metadata = Get-Content -LiteralPath $metadataPath -Raw | ConvertFrom-Json
$detail = @($metadata.details | Where-Object { $_.label -eq "Version" } | Select-Object -First 1)
if (-not $detail) {
    throw "game_page.json has no Version detail"
}
$detail.value = $version
$metadata | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $metadataPath -Encoding utf8
Write-Output "Stamped game_page.json with $version"
