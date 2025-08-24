# Ant Colony Simulation with Automated Video Generation
# PowerShell version for more robust error handling

Write-Host "🐜 Starting Ant Colony Simulation with Automated Video Generation" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan

# Create directories if they don't exist
$videosDir = "videos"
$debugDir = "debug"

if (!(Test-Path $videosDir)) {
    New-Item -ItemType Directory -Path $videosDir | Out-Null
    Write-Host "📁 Created videos directory" -ForegroundColor Green
}

if (!(Test-Path $debugDir)) {
    New-Item -ItemType Directory -Path $debugDir | Out-Null  
    Write-Host "📁 Created debug directory" -ForegroundColor Green
}

# Run the simulation
Write-Host "🚀 Running simulation..." -ForegroundColor Yellow
try {
    $process = Start-Process -FilePath "cargo" -ArgumentList "run" -Wait -PassThru -NoNewWindow
    if ($process.ExitCode -ne 0) {
        throw "Cargo run failed with exit code $($process.ExitCode)"
    }
    Write-Host "✅ Simulation completed successfully!" -ForegroundColor Green
} catch {
    Write-Host "❌ Simulation failed: $_" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Find the most recent frames directory
Write-Host "📹 Looking for captured frames..." -ForegroundColor Yellow
$framesDirectories = Get-ChildItem -Path "simulation_videos" -Directory -Name "*_frames" -ErrorAction SilentlyContinue | Sort-Object CreationTime -Descending

if ($framesDirectories.Count -eq 0) {
    Write-Host "❌ No frame directories found in simulation_videos/" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

$latestFrames = $framesDirectories[0]
$videoName = $latestFrames -replace "_frames$", ""

Write-Host "🎬 Found frames: $latestFrames" -ForegroundColor Green

# Convert frames to video
Write-Host "🔄 Converting frames to MP4..." -ForegroundColor Yellow
$ffmpegPath = ".\ffmpeg\ffmpeg-2025-08-20-git-4d7c609be3-full_build\bin\ffmpeg.exe"
$inputPattern = "simulation_videos\$latestFrames\frame_%04d.png"
$outputPath = "simulation_videos\$videoName.mp4"

try {
    $ffmpegProcess = Start-Process -FilePath $ffmpegPath -ArgumentList @(
        "-framerate", "30",
        "-i", $inputPattern,
        "-c:v", "libx264", 
        "-pix_fmt", "yuv420p",
        "-y", $outputPath
    ) -Wait -PassThru -NoNewWindow

    if ($ffmpegProcess.ExitCode -ne 0) {
        throw "FFmpeg failed with exit code $($ffmpegProcess.ExitCode)"
    }
    Write-Host "✅ Video created: $outputPath" -ForegroundColor Green
} catch {
    Write-Host "❌ Video conversion failed: $_" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Organize files
Write-Host "📁 Organizing files..." -ForegroundColor Yellow

# Move video to videos/ directory
$finalVideoPath = "videos\$videoName.mp4"
Move-Item -Path $outputPath -Destination $finalVideoPath -Force

# Move frames to debug/ directory  
$debugFramesPath = "debug\$latestFrames"
Move-Item -Path "simulation_videos\$latestFrames" -Destination $debugFramesPath -Force

# Move metadata if it exists
$metadataPath = "simulation_videos\$videoName" + "_metadata.txt"
if (Test-Path $metadataPath) {
    Move-Item -Path $metadataPath -Destination "debug\" -Force
}

Write-Host "🎉 Complete! Video saved to: $finalVideoPath" -ForegroundColor Green
Write-Host "🔍 Debug frames saved to: $debugFramesPath\" -ForegroundColor Green
Write-Host ""

# Update generation info for next run
Write-Host "📝 Updating generation info..." -ForegroundColor Yellow
try {
    if (Test-Path "generation_info.json") {
        $genInfo = Get-Content "generation_info.json" | ConvertFrom-Json
        $genInfo.current_generation += 1
        $genInfo.timestamp = (Get-Date).ToString("yyyy-MM-ddTHH:mm:ssZ")
        $genInfo.description = "Automated run with averageTimeSinceGoal metric and video overlay"
        $genInfo.video_filename = "$($genInfo.current_generation.ToString("0000"))_automated_run.mp4"
        
        $genInfo | ConvertTo-Json -Depth 10 | Set-Content "generation_info.json"
        Write-Host "✅ Generation incremented to $($genInfo.current_generation)" -ForegroundColor Green
    }
} catch {
    Write-Host "⚠️ Could not update generation info: $_" -ForegroundColor Yellow
}

Read-Host "Press Enter to exit"