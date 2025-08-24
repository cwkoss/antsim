#!/bin/bash
# Convert PNG frame sequences to MP4 videos using FFmpeg
# Usage: Run this script from the project root directory

echo "Converting PNG sequences to MP4 videos..."

for frames_dir in simulation_videos/*_frames/; do
    if [ -d "$frames_dir" ]; then
        echo "Processing $frames_dir..."
        
        # Generate output video filename
        output_video="${frames_dir%_frames/}.mp4"
        
        # Convert PNG sequence to MP4 with mobile aspect ratio (even width for H.264)
        ./ffmpeg/ffmpeg-2025-08-20-git-4d7c609be3-full_build/bin/ffmpeg.exe -y -framerate 30 -i "${frames_dir}frame_%04d.png" \
               -c:v libx264 -pix_fmt yuv420p -s 406x720 \
               "$output_video" 2>/dev/null
        
        if [ -f "$output_video" ]; then
            echo "✅ Created: $output_video"
            # Optional: Remove frames directory after successful conversion
            # rm -rf "$frames_dir"
        else
            echo "❌ Failed to create: $output_video"
        fi
    fi
done

echo
echo "Done! Check simulation_videos folder for MP4 files."