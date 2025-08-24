use png::ColorType;
use std::fs::{self, File};
use std::io::BufWriter;

pub fn create_test_png_frames() -> Result<(), Box<dyn std::error::Error>> {
    // Create test frames directory
    let frames_dir = "simulation_videos/test_png_frames";
    fs::create_dir_all(frames_dir)?;
    
    // Mobile aspect ratio: 405x720
    let width = 405u32;
    let height = 720u32;
    let frame_size = (width * height * 4) as usize;
    
    // Create 30 test frames (5 seconds worth)
    for i in 0..30 {
        let mut frame_data = vec![0u8; frame_size];
        let time_factor = i as f32 / 29.0; // 0.0 to 1.0
        
        // Create animated pattern
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                
                // Horizontal gradient (red) fading over time
                frame_data[idx] = ((255.0 * (x as f32 / width as f32) * (1.0 - time_factor)) as u8);
                // Vertical gradient (green) appearing over time
                frame_data[idx + 1] = ((255.0 * (y as f32 / height as f32) * time_factor) as u8);
                // Blue increases over time
                frame_data[idx + 2] = ((255.0 * time_factor) as u8);
                // Full opacity
                frame_data[idx + 3] = 255;
            }
        }
        
        // Add moving elements (simulate ants)
        for ant in 0..10 {
            let ant_x = ((width as f32 * 0.8) * ((i + ant * 3) % 30) as f32 / 30.0) as u32 + 50;
            let ant_y = ((height as f32 * 0.8) * ((i + ant * 2) % 30) as f32 / 30.0) as u32 + 50;
            
            // Draw ant (small yellow square)
            for dy in -2i32..=2i32 {
                for dx in -2i32..=2i32 {
                    let px = ant_x as i32 + dx;
                    let py = ant_y as i32 + dy;
                    
                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        let idx = ((py as u32 * width + px as u32) * 4) as usize;
                        if idx + 3 < frame_data.len() {
                            frame_data[idx] = 255;     // R - Yellow
                            frame_data[idx + 1] = 255; // G - Yellow
                            frame_data[idx + 2] = 0;   // B - Yellow
                            frame_data[idx + 3] = 255; // A - Full opacity
                        }
                    }
                }
            }
        }
        
        // Save as PNG
        let frame_path = format!("{}/frame_{:04}.png", frames_dir, i);
        let file = File::create(&frame_path)?;
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, width, height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&frame_data)?;
        
        println!("Created {}", frame_path);
    }
    
    println!("✅ Created 30 test PNG frames in {}", frames_dir);
    Ok(())
}