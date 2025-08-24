Write-Host "Testing Ant Colony Simulation Pipeline" -ForegroundColor Cyan

# Test that files exist
$genInfo = Test-Path 'generation_info.json'
$genHistory = Test-Path 'generation_history.json'

Write-Host "Generation info system: $genInfo" -ForegroundColor Green
Write-Host "History tracking system: $genHistory" -ForegroundColor Green

# Quick compilation test
Write-Host "Testing compilation..." -ForegroundColor Yellow
cargo check | Out-Null
if ($LASTEXITCODE -eq 0) {
    Write-Host "Compilation successful!" -ForegroundColor Green
} else {
    Write-Host "Compilation failed" -ForegroundColor Red
}

Write-Host "Basic automation tests complete!" -ForegroundColor Green