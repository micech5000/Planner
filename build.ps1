$ErrorActionPreference = 'Stop'
$env:PATH = "$env:USERPROFILE\.cargo\bin;$PSScriptRoot\tools\w64devkit\bin;" + $env:PATH
Push-Location $PSScriptRoot
try {
    cargo test --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
    New-Item -ItemType Directory -Path "$PSScriptRoot\dist" -Force | Out-Null
    Copy-Item -LiteralPath "$PSScriptRoot\target\release\study-planner.exe" -Destination "$PSScriptRoot\dist\Мой день.exe" -Force
} finally { Pop-Location }
