$ErrorActionPreference = 'Stop'
$w64 = if (Test-Path "$PSScriptRoot\tools\w64devkit\bin") {
    "$PSScriptRoot\tools\w64devkit\bin"
} else {
    Join-Path (Split-Path $PSScriptRoot -Parent) "tools\w64devkit\bin"
}
$env:PATH = "$env:USERPROFILE\.cargo\bin;$w64;" + $env:PATH
# Keep Cargo target directory strictly ASCII so w64devkit ld.exe never fails on non-ASCII project folder names.
$targetDir = Join-Path $env:LOCALAPPDATA "StudyPlannerGeminiTarget"
$env:CARGO_TARGET_DIR = $targetDir

Push-Location $PSScriptRoot
try {
    cargo test --offline --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    cargo build --release --offline --locked
    if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
    New-Item -ItemType Directory -Path "$PSScriptRoot\dist" -Force | Out-Null
    $exeName = (-join ([char[]]@(0x041C,0x043E,0x0439,0x0020,0x0434,0x0435,0x043D,0x044C))) + '.exe'
    $destPath = Join-Path "$PSScriptRoot\dist" $exeName
    Copy-Item -LiteralPath "$targetDir\release\study-planner.exe" -Destination $destPath -Force
    Write-Host "Build completed: $destPath"
} finally { Pop-Location }
