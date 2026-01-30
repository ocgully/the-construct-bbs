# dev.ps1 - Start backend and frontend dev servers together.
# Each runs in its own window. Press Enter in this window to stop both.

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Build frontend first
Write-Host "=== Building frontend ===" -ForegroundColor Cyan
Set-Location "$ScriptDir\frontend"
npm run build
if ($LASTEXITCODE -ne 0) { Write-Host "Frontend build failed!" -ForegroundColor Red; exit 1 }

# Start backend in its own window
Write-Host ""
Write-Host "=== Starting backend ===" -ForegroundColor Cyan
$backend = Start-Process -PassThru -FilePath "cmd.exe" -ArgumentList "/k","title Backend && cargo run" -WorkingDirectory "$ScriptDir\backend"

# Start frontend dev server in its own window
Write-Host "=== Starting frontend dev server ===" -ForegroundColor Cyan
$frontend = Start-Process -PassThru -FilePath "cmd.exe" -ArgumentList "/k","title Frontend && npm run dev" -WorkingDirectory "$ScriptDir\frontend"

Write-Host ""
Write-Host "================================================" -ForegroundColor Green
Write-Host "  Backend:  http://localhost:3000  (static build)" -ForegroundColor Green
Write-Host "  Frontend: http://localhost:5173  (dev + hot reload)" -ForegroundColor Green
Write-Host "" -ForegroundColor Green
Write-Host "  Press Enter here to stop both servers." -ForegroundColor Yellow
Write-Host "================================================" -ForegroundColor Green
Write-Host ""

Read-Host

Write-Host "Shutting down..." -ForegroundColor Yellow

# Kill backend window and child processes
if (!$backend.HasExited) {
    Stop-Process -Id $backend.Id -Force -ErrorAction SilentlyContinue
}
Get-Process -Name "bbs-backend" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Write-Host "  Backend stopped."

# Kill frontend window and child processes
if (!$frontend.HasExited) {
    Stop-Process -Id $frontend.Id -Force -ErrorAction SilentlyContinue
}
Get-Process -Name "node" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Write-Host "  Frontend stopped."

Write-Host "Done." -ForegroundColor Green
