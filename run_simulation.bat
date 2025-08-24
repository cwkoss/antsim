@echo off
echo 🐜 Starting Ant Colony Simulation with Automated Video Generation
echo ================================================================

REM Create videos and debug directories if they don't exist
if not exist "videos" mkdir videos
if not exist "debug" mkdir debug

REM Run the simulation
echo 🚀 Running simulation...
cargo run
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Simulation failed with error %ERRORLEVEL%
    pause
    exit /b %ERRORLEVEL%
)

echo ✅ Simulation completed successfully!

REM Find the most recent frames directory
echo 📹 Looking for captured frames...
for /f "delims=" %%i in ('dir /b /od simulation_videos\*_frames 2^>nul') do set LATEST_FRAMES=%%i
if "%LATEST_FRAMES%" == "" (
    echo ❌ No frame directories found in simulation_videos/
    pause
    exit /b 1
)

echo 🎬 Found frames: %LATEST_FRAMES%

REM Extract the base filename (remove _frames suffix)
set VIDEO_NAME=%LATEST_FRAMES:_frames=%

REM Convert frames to video
echo 🔄 Converting frames to MP4...
.\ffmpeg\ffmpeg-2025-08-20-git-4d7c609be3-full_build\bin\ffmpeg.exe -framerate 30 -i "simulation_videos\%LATEST_FRAMES%\frame_%%04d.png" -c:v libx264 -pix_fmt yuv420p -y "simulation_videos\%VIDEO_NAME%.mp4"

if %ERRORLEVEL% NEQ 0 (
    echo ❌ Video conversion failed with error %ERRORLEVEL%
    pause
    exit /b %ERRORLEVEL%
)

echo ✅ Video created: simulation_videos\%VIDEO_NAME%.mp4

REM Move video to videos/ directory and frames to debug/
echo 📁 Organizing files...
move "simulation_videos\%VIDEO_NAME%.mp4" "videos\" >nul
move "simulation_videos\%LATEST_FRAMES%" "debug\" >nul
if exist "simulation_videos\%VIDEO_NAME%_metadata.txt" (
    move "simulation_videos\%VIDEO_NAME%_metadata.txt" "debug\" >nul
)

echo 🎉 Complete! Video saved to: videos\%VIDEO_NAME%.mp4
echo 🔍 Debug frames saved to: debug\%LATEST_FRAMES%\
echo.
pause