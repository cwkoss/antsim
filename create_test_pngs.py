#!/usr/bin/env python3
"""
Create test PNG files for video conversion testing
"""

from PIL import Image
import numpy as np
import os

def create_test_frames():
    # Create test frames directory
    frames_dir = "simulation_videos/test_png_frames"
    os.makedirs(frames_dir, exist_ok=True)
    
    # Mobile aspect ratio: 405x720
    width, height = 405, 720
    
    # Create 30 test frames (5 seconds at 6 fps)
    for i in range(30):
        # Create gradient pattern that changes over time
        frame = np.zeros((height, width, 4), dtype=np.uint8)
        
        # Create time-based color animation
        time_factor = i / 29.0  # 0 to 1
        
        for y in range(height):
            for x in range(width):
                # Horizontal gradient (red)
                frame[y, x, 0] = int(255 * (x / width) * (1 - time_factor))
                # Vertical gradient (green) 
                frame[y, x, 1] = int(255 * (y / height) * time_factor)
                # Blue increases over time
                frame[y, x, 2] = int(255 * time_factor)
                # Full opacity
                frame[y, x, 3] = 255
        
        # Add some moving elements (simulate ants)
        for ant in range(10):
            ant_x = int((width * 0.8) * ((i + ant * 3) % 30) / 30) + 50
            ant_y = int((height * 0.8) * ((i + ant * 2) % 30) / 30) + 50
            
            # Draw ant (small yellow square)
            for dy in range(-2, 3):
                for dx in range(-2, 3):
                    if 0 <= ant_x + dx < width and 0 <= ant_y + dy < height:
                        frame[ant_y + dy, ant_x + dx] = [255, 255, 0, 255]  # Yellow
        
        # Save as PNG
        img = Image.fromarray(frame, 'RGBA')
        img.save(f"{frames_dir}/frame_{i:04d}.png")
        print(f"Created frame_{i:04d}.png")
    
    print(f"✅ Created {30} test PNG frames in {frames_dir}")
    return frames_dir

if __name__ == "__main__":
    create_test_frames()