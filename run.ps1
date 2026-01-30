# run.ps1 - Build frontend then start the backend server.
# Usage: .\run.ps1
# The backend serves the built frontend at http://localhost:3000

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Build frontend
Write-Host "=== Building frontend ===" -ForegroundColor Cyan
Set-Location "$ScriptDir\frontend"
npm run build
if ($LASTEXITCODE -ne 0) { Write-Host "Frontend build failed!" -ForegroundColor Red; exit 1 }

# Run backend (serves frontend static files from frontend/dist)
Write-Host ""
Write-Host "=== Starting backend ===" -ForegroundColor Cyan
Write-Host "  http://localhost:3000" -ForegroundColor Green
Write-Host ""
Set-Location "$ScriptDir\backend"
cargo run
