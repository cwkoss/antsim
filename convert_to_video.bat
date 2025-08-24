@echo off
REM Convert PNG frame sequences to MP4 videos using FFmpeg
REM Usage: Run this script from the project root directory

echo Converting PNG sequences to MP4 videos...

for /d %%D in (simulation_videos\*_frames) do (
    echo Processing %%D...
    set "frames_dir=%%D"
    set "output_video=%%D"
    set "output_video=!output_video:_frames=.mp4!"
    
    REM Convert PNG sequence to MP4 with mobile aspect ratio (even width for H.264)
    ffmpeg\ffmpeg-2025-08-20-git-4d7c609be3-full_build\bin\ffmpeg.exe -y -framerate 30 -i "%%D\frame_%%04d.png" -c:v libx264 -pix_fmt yuv420p -s 406x720 "!output_video!" 2>nul
    
    if exist "!output_video!" (
        echo ✅ Created: !output_video!
        REM Optional: Remove frames directory after successful conversion
        REM rmdir /s /q "%%D"
    ) else (
        echo ❌ Failed to create: !output_video!
    )
)

echo.
echo Done! Check simulation_videos folder for MP4 files.
pause