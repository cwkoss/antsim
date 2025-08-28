#!/bin/bash

# Ant Colony Simulation with Automated Video Generation
# Bash version for cross-platform compatibility

echo "🐜 Starting Ant Colony Simulation with Automated Video Generation"
echo "================================================================"
echo ""

# Get generation description and optional challenge from command line arguments
if [ $# -eq 0 ]; then
    echo "❌ Usage: $0 \"<generation description>\" [challenge_number]"
    echo "   Example: $0 \"Path persistence optimization with stronger momentum\""
    echo "   Example: $0 \"Pathfinding around rocks\" 2"
    read -p "Press Enter to exit"
    exit 1
fi

GENERATION_DESCRIPTION="$1"
CHALLENGE_NUMBER="${2:-1}"  # Default to challenge 1 if not specified
echo "📋 This generation will be labeled: '$GENERATION_DESCRIPTION'"
echo "🎯 Challenge: $CHALLENGE_NUMBER"
echo ""

# Update generation description in generation_info.json before running simulation
if [ -f "generation_info.json" ]; then
    echo "📝 Updating generation description..."
    sed -i "s/\"description\": \"[^\"]*\"/\"description\": \"$GENERATION_DESCRIPTION\"/" generation_info.json
    echo "✅ Description updated in generation_info.json"
else
    echo "⚠️ generation_info.json not found - description will use simulation default"
fi
echo ""

# Create directories if they don't exist
if [ ! -d "videos" ]; then
    mkdir -p videos
    echo "📁 Created videos directory"
fi

if [ ! -d "debug" ]; then
    mkdir -p debug
    echo "📁 Created debug directory"
fi

# Run the simulation
echo "🚀 Running simulation..."
if ! cargo run -- --challenge "$CHALLENGE_NUMBER"; then
    echo "❌ Simulation failed"
    read -p "Press Enter to exit"
    exit 1
fi
echo "✅ Simulation completed successfully!"

# Find the most recent frames directory (by modification time)
echo "📹 Looking for captured frames..."
LATEST_FRAMES=$(find simulation_videos -name "*_frames" -type d -printf "%T@ %f\n" 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2)
if [ -z "$LATEST_FRAMES" ]; then
    # Fallback for systems without -printf support
    LATEST_FRAMES=$(ls -t simulation_videos/*_frames 2>/dev/null | head -1 | xargs basename 2>/dev/null)
fi

if [ -z "$LATEST_FRAMES" ]; then
    echo "❌ No frame directories found in simulation_videos/"
    read -p "Press Enter to exit"
    exit 1
fi

VIDEO_NAME=${LATEST_FRAMES%_frames}

echo "🎬 Found frames: $LATEST_FRAMES"

# Convert frames to video - fix Windows path issues in Git Bash
echo "🔄 Converting frames to MP4..."
cd simulation_videos
FFMPEG_PATH="../ffmpeg/ffmpeg-2025-08-20-git-4d7c609be3-full_build/bin/ffmpeg.exe"

if ! "$FFMPEG_PATH" -framerate 30 -i "${LATEST_FRAMES}/frame_%04d.png" -c:v libx264 -pix_fmt yuv420p -y "${VIDEO_NAME}.mp4"; then
    echo "❌ Video conversion failed"
    cd ..
    read -p "Press Enter to exit"
    exit 1
fi
cd ..
echo "✅ Video created: simulation_videos/${VIDEO_NAME}.mp4"

# Organize files
echo "📁 Organizing files..."

# Move video to videos/ directory
FINAL_VIDEO_PATH="videos/$VIDEO_NAME.mp4"
mv "simulation_videos/${VIDEO_NAME}.mp4" "$FINAL_VIDEO_PATH"

# Move frames to debug/ directory
DEBUG_FRAMES_PATH="debug/$LATEST_FRAMES"
mv "simulation_videos/$LATEST_FRAMES" "$DEBUG_FRAMES_PATH"

# Move metadata if it exists
METADATA_PATH="simulation_videos/${VIDEO_NAME}_metadata.txt"
if [ -f "$METADATA_PATH" ]; then
    mv "$METADATA_PATH" "debug/"
fi

echo "🎉 Complete! Video saved to: $FINAL_VIDEO_PATH"
echo "🔍 Debug frames saved to: $DEBUG_FRAMES_PATH/"
echo ""

# Update generation info for next run
echo "📝 Updating generation info..."
if [ -f "generation_info.json" ]; then
    # Basic increment - could be enhanced with jq for proper JSON parsing
    CURRENT_GEN=$(grep -o '"current_generation": *[0-9]*' generation_info.json | grep -o '[0-9]*')
    NEW_GEN=$((CURRENT_GEN + 1))
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    # Simple sed replacement (not as robust as PowerShell version)
    sed -i "s/\"current_generation\": $CURRENT_GEN/\"current_generation\": $NEW_GEN/" generation_info.json
    sed -i "s/\"timestamp\": \"[^\"]*\"/\"timestamp\": \"$TIMESTAMP\"/" generation_info.json
    # Description was already updated before simulation
    
    echo "✅ Generation incremented to $NEW_GEN"
else
    echo "⚠️ Could not find generation_info.json"
fi

read -p "Press Enter to exit"