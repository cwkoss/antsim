use bevy::prelude::*;
use crate::components::*;
use crate::pheromones::*;
use crate::colors::*;
use std::fs;

#[derive(Component)]
pub struct VideoCamera;

#[derive(Resource)]
pub struct VideoRenderTarget {
    pub image: Handle<Image>,
}

pub fn setup_video_camera() {
    // Simplified setup - we'll use a different approach to capture the main camera's output
    println!("🎥 Video recording system initialized (screenshot-based capture ready)");
}

pub fn video_recording_system(
    mut video_recorder: ResMut<VideoRecorder>,
    performance_tracker: Res<PerformanceTracker>,
    generation_info: Res<GenerationInfo>,
    time: Res<Time>,
    pheromone_grid: Res<PheromoneGrid>,
    color_config: Res<ColorConfig>,
    ant_query: Query<(&Transform, &AntState), (With<AntState>, Without<Nest>)>,
    food_query: Query<&Transform, (With<FoodSource>, Without<Nest>)>,
    nest_query: Query<&Transform, With<Nest>>,
) {
    // Start recording when simulation has been running for a bit
    if !video_recorder.is_recording {
        video_recorder.is_recording = true;
        println!("📹 Started video recording for Test {}", video_recorder.test_number);
    }
    
    if video_recorder.is_recording {
        // Create visual frame with actual simulation data (capture whole simulation)
        capture_simulation_frame(&mut video_recorder, &performance_tracker, &generation_info, time.elapsed_seconds(), 
                               &pheromone_grid, &color_config, &ant_query, &food_query, &nest_query);
    }
    
    // Check if simulation is ending and should save video
    if should_save_video(&performance_tracker, &time) && video_recorder.is_recording {
        save_video_on_exit(&mut video_recorder, &performance_tracker);
        video_recorder.is_recording = false;
    }
}


fn capture_simulation_frame(
    video_recorder: &mut VideoRecorder, 
    performance_tracker: &PerformanceTracker, 
    generation_info: &GenerationInfo,
    elapsed_time: f32,
    pheromone_grid: &PheromoneGrid,
    color_config: &ColorConfig,
    ant_query: &Query<(&Transform, &AntState), (With<AntState>, Without<Nest>)>,
    food_query: &Query<&Transform, (With<FoodSource>, Without<Nest>)>,
    nest_query: &Query<&Transform, With<Nest>>,
) {
    let target_width = video_recorder.frame_width;
    let target_height = video_recorder.frame_height;
    let frame_size = (target_width * target_height * 4) as usize;
    let mut frame = vec![0u8; frame_size];
    
    // Render pheromone trails as background
    let world_size = 1000.0;
    let grid_to_screen_x = |grid_x: usize| -> u32 {
        ((grid_x as f32 / pheromone_grid.width as f32) * target_width as f32) as u32
    };
    let grid_to_screen_y = |grid_y: usize| -> u32 {
        ((grid_y as f32 / pheromone_grid.height as f32) * target_height as f32) as u32
    };
    
    // Render pheromone grid
    for grid_y in 0..pheromone_grid.height {
        for grid_x in 0..pheromone_grid.width {
            let grid_idx = grid_y * pheromone_grid.width + grid_x;
            
            // Get pheromone values
            let food_pheromone = pheromone_grid.food_trail[grid_idx].min(1.0);
            let nest_pheromone = pheromone_grid.nest_trail[grid_idx].min(1.0);
            
            // Map to screen coordinates 
            let screen_x = grid_to_screen_x(grid_x);
            let screen_y = grid_to_screen_y(grid_y);
            
            if screen_x < target_width && screen_y < target_height {
                let pixel_idx = ((screen_y * target_width + screen_x) * 4) as usize;
                
                if pixel_idx + 3 < frame.len() {
                    // Use shared color config for consistent rendering
                    let (food_r, food_g, food_b) = color_config.food_pheromone_rgb();
                    let (nest_r, nest_g, nest_b) = color_config.nest_pheromone_rgb();
                    
                    // Blend colors based on pheromone strength
                    let r = ((nest_pheromone * nest_r as f32) + (food_pheromone * food_r as f32)).min(255.0) as u8;
                    let g = ((nest_pheromone * nest_g as f32) + (food_pheromone * food_g as f32)).min(255.0) as u8;
                    let b = ((nest_pheromone * nest_b as f32) + (food_pheromone * food_b as f32)).min(255.0) as u8;
                    
                    frame[pixel_idx] = r;
                    frame[pixel_idx + 1] = g;
                    frame[pixel_idx + 2] = b;
                    frame[pixel_idx + 3] = 255; // Full opacity
                }
            }
        }
    }
    
    // World bounds for simulation (assuming 1000x1000 world)
    let world_size = 1000.0;
    let world_to_screen_x = |world_x: f32| -> i32 {
        ((world_x + world_size / 2.0) / world_size * target_width as f32) as i32
    };
    let world_to_screen_y = |world_y: f32| -> i32 {
        ((world_y + world_size / 2.0) / world_size * target_height as f32) as i32
    };
    
    // Draw nest (yellow circle)
    if let Ok(nest_transform) = nest_query.get_single() {
        let nest_x = world_to_screen_x(nest_transform.translation.x);
        let nest_y = world_to_screen_y(nest_transform.translation.y);
        
        // Draw 15x15 pixel nest
        for dy in -7..8 {
            for dx in -7..8 {
                let px = (nest_x + dx).max(0).min(target_width as i32 - 1) as u32;
                let py = (nest_y + dy).max(0).min(target_height as i32 - 1) as u32;
                let idx = ((py * target_width + px) * 4) as usize;
                
                if idx + 3 < frame.len() {
                    let (r, g, b) = color_config.nest_rgb();
                    frame[idx] = r;
                    frame[idx + 1] = g;
                    frame[idx + 2] = b;
                    frame[idx + 3] = 255;
                }
            }
        }
    }
    
    // Draw food sources (green circles)
    for food_transform in food_query.iter() {
        let food_x = world_to_screen_x(food_transform.translation.x);
        let food_y = world_to_screen_y(food_transform.translation.y);
        
        // Draw 8x8 pixel food
        for dy in -4..4 {
            for dx in -4..4 {
                let px = (food_x + dx).max(0).min(target_width as i32 - 1) as u32;
                let py = (food_y + dy).max(0).min(target_height as i32 - 1) as u32;
                let idx = ((py * target_width + px) * 4) as usize;
                
                if idx + 3 < frame.len() {
                    let (r, g, b) = color_config.food_source_rgb();
                    frame[idx] = r;
                    frame[idx + 1] = g;
                    frame[idx + 2] = b;
                    frame[idx + 3] = 255;
                }
            }
        }
    }
    
    // Draw ants with state-based colors
    for (ant_transform, ant_state) in ant_query.iter() {
        let ant_x = world_to_screen_x(ant_transform.translation.x);
        let ant_y = world_to_screen_y(ant_transform.translation.y);
        
        // Determine ant color based on state using shared config
        let (r, g, b) = if ant_state.carrying_food {
            color_config.ant_carrying_food_rgb()
        } else if ant_state.food_collection_timer > 0.0 {
            color_config.ant_collecting_rgb()
        } else {
            color_config.ant_exploring_rgb()
        };
        
        // Draw 4x4 pixel ant body (slightly larger for better visibility)
        for dy in -2..2 {
            for dx in -2..2 {
                let px = (ant_x + dx).max(0).min(target_width as i32 - 1) as u32;
                let py = (ant_y + dy).max(0).min(target_height as i32 - 1) as u32;
                let idx = ((py * target_width + px) * 4) as usize;
                
                if idx + 3 < frame.len() {
                    frame[idx] = r;
                    frame[idx + 1] = g;
                    frame[idx + 2] = b;
                    frame[idx + 3] = 255;
                }
            }
        }
        
        // Add directional indicator - a bright white pixel in the direction the ant is facing
        let direction = ant_state.current_direction;
        let indicator_distance = 3.0; // Pixels from center
        let indicator_x = ant_x + (direction.cos() * indicator_distance) as i32;
        let indicator_y = ant_y + (direction.sin() * indicator_distance) as i32;
        
        if indicator_x >= 0 && indicator_x < target_width as i32 && 
           indicator_y >= 0 && indicator_y < target_height as i32 {
            let idx = ((indicator_y as u32 * target_width + indicator_x as u32) * 4) as usize;
            if idx + 3 < frame.len() {
                frame[idx] = 255;     // Bright white indicator
                frame[idx + 1] = 255;
                frame[idx + 2] = 255;
                frame[idx + 3] = 255;
            }
        }
    }
    
    // Add comprehensive text overlay at top (first 80 pixels height)
    let text_height = 80;
    for y in 0..text_height {
        for x in 0..target_width {
            let idx = ((y * target_width + x) * 4) as usize;
            if idx + 3 < frame.len() {
                // Semi-transparent dark overlay for text background
                frame[idx] = 0;       // R
                frame[idx + 1] = 0;   // G  
                frame[idx + 2] = 0;   // B
                frame[idx + 3] = 200; // More opaque for better text readability
            }
        }
    }
    
    // Render text information (simple pixel text simulation)
    render_text_overlay(&mut frame, target_width, target_height, generation_info, performance_tracker, elapsed_time);
    
    video_recorder.frames.push(frame);
}

fn capture_placeholder_frame(video_recorder: &mut VideoRecorder) {
    // Create a dummy frame when real capture isn't available
    let frame_size = (video_recorder.frame_width * video_recorder.frame_height * 4) as usize;
    let mut frame = vec![0u8; frame_size];
    
    // Simple placeholder pattern
    for i in (0..frame_size).step_by(4) {
        frame[i] = 50;     // R
        frame[i + 1] = 50; // G  
        frame[i + 2] = 50; // B
        frame[i + 3] = 255; // A
    }
    
    video_recorder.frames.push(frame);
}

fn should_save_video(performance_tracker: &PerformanceTracker, time: &Time) -> bool {
    // Save video when:
    // 1. Auto-exit conditions are met (oscillation/lost carriers)
    // 2. Or simulation has run for 2+ minutes successfully
    time.elapsed_seconds() > 300.0 || 
    performance_tracker.oscillating_ants_count >= 20 ||
    performance_tracker.lost_food_carriers_count >= 10
}

fn save_video_on_exit(video_recorder: &mut VideoRecorder, performance_tracker: &PerformanceTracker) {
    // Create videos directory if it doesn't exist
    let videos_dir = "simulation_videos";
    if let Err(e) = fs::create_dir_all(videos_dir) {
        println!("❌ Failed to create videos directory: {}", e);
        return;
    }
    
    // Generate filename with timestamp and test info
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!(
        "{}/test_{:03}_{}_deliveries_{:.1}_return_{:.1}s.mp4",
        videos_dir,
        video_recorder.test_number,
        timestamp,
        performance_tracker.deliveries_per_minute,
        performance_tracker.average_return_time
    );
    
    println!("📹 Saving video: {}", filename);
    println!("   Changes: {}", video_recorder.changes_description);
    println!("   Frames captured: {}", video_recorder.frames.len());
    println!("   Final stats: {:.1} deliveries/min, {:.1}s return time", 
        performance_tracker.deliveries_per_minute,
        performance_tracker.average_return_time
    );
    
    // Save frames as PNG sequence that can be converted to video later
    // Each frame will be saved as PNG with mobile aspect ratio and overlays
    
    let frames_dir = filename.replace(".mp4", "_frames");
    if let Err(e) = fs::create_dir_all(&frames_dir) {
        println!("❌ Failed to create frames directory: {}", e);
        return;
    }
    
    println!("💾 Saving {} frames to: {}", video_recorder.frames.len(), frames_dir);
    
    // Save every 6th frame for 5-second video (6x speed from 30s capture)
    for (i, frame) in video_recorder.frames.iter().step_by(6).enumerate() {
        let frame_path = format!("{}/frame_{:04}.png", frames_dir, i);
        
        // Debug frame data before saving
        println!("🔍 Frame {}: {} bytes, expected {}", 
            i, 
            frame.len(), 
            video_recorder.frame_width * video_recorder.frame_height * 4
        );
        
        // Try to save as actual PNG image
        if let Err(e) = save_frame_as_png(&frame_path, frame, video_recorder.frame_width, video_recorder.frame_height) {
            println!("❌ Failed to save frame {} as PNG: {}", i, e);
            // Fallback to text file with frame info
            let fallback_path = format!("{}/frame_{:04}.txt", frames_dir, i);
            let frame_info = format!("Frame {}: {} bytes captured\nError: {}", i, frame.len(), e);
            if let Err(e2) = fs::write(&fallback_path, frame_info) {
                println!("❌ Failed to save fallback info for frame {}: {}", i, e2);
            }
        } else {
            println!("✅ Saved frame {} as PNG: {}", i, frame_path);
        }
    }
    
    // Create metadata file  
    let metadata_file = filename.replace(".mp4", "_metadata.txt");
    let metadata = format!(
        "Test #{}\nChanges: {}\nDeliveries/min: {:.2}\nReturn time: {:.1}s\nFrames: {}\nDuration: {:.1} seconds (6x speed from entire simulation)\n",
        video_recorder.test_number,
        video_recorder.changes_description,
        performance_tracker.deliveries_per_minute,
        performance_tracker.average_return_time,
        video_recorder.frames.len(),
        video_recorder.frames.len() as f32 / 6.0 / 30.0 // frames / speedup / fps
    );
    
    if let Err(e) = fs::write(&metadata_file, metadata) {
        println!("❌ Failed to write metadata: {}", e);
    } else {
        println!("✅ Video metadata saved: {}", metadata_file);
    }
    
    // Clear frames for next test
    video_recorder.frames.clear();
    video_recorder.test_number += 1;
    
    // Update changes description for next test
    video_recorder.changes_description = "Algorithm optimization iteration".to_string();
}

fn save_frame_as_png(
    path: &str,
    frame_data: &[u8],
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    use png::ColorType;
    use std::io::BufWriter;
    
    println!("🔍 PNG save: {}x{}, {} bytes, path: {}", width, height, frame_data.len(), path);
    
    // Check if frame data has the right size for RGBA
    let expected_size = (width * height * 4) as usize;
    if frame_data.len() != expected_size {
        return Err(format!(
            "Frame data size mismatch: expected {}, got {}",
            expected_size,
            frame_data.len()
        ).into());
    }

    // Create file
    println!("🔍 Creating PNG file: {}", path);
    let file = std::fs::File::create(path)?;
    let ref mut w = BufWriter::new(file);

    println!("🔍 Setting up PNG encoder...");
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    
    println!("🔍 Writing PNG header...");
    let mut writer = encoder.write_header()?;
    
    println!("🔍 Writing PNG image data...");
    writer.write_image_data(frame_data)?;
    
    println!("✅ PNG saved successfully: {}", path);
    Ok(())
}

fn render_text_overlay(
    frame: &mut [u8],
    width: u32, 
    height: u32,
    generation_info: &GenerationInfo,
    performance_tracker: &PerformanceTracker,
    elapsed_time: f32,
) {
    // Simple pixel-based text rendering - create bright colored pixels for text visibility
    // This is a basic implementation for readability
    
    // Line 1: Generation info (y = 10-15)
    let gen_text = format!("GEN {}: {:.40}...", generation_info.current_generation, generation_info.description);
    render_text_line(frame, width, &gen_text, 5, 10, [255, 255, 255]); // White text
    
    // Line 2: Performance metrics (y = 25-30)  
    let perf_text = format!("📊 {:.1}del/min {:.1}s ret {:.1}s goal", 
        performance_tracker.deliveries_per_minute,
        performance_tracker.average_return_time,
        performance_tracker.average_time_since_goal
    );
    render_text_line(frame, width, &perf_text, 5, 25, [0, 255, 255]); // Cyan text
    
    // Line 3: Time and issues (y = 40-45)
    let status_text = format!("⏱️ {:.0}s 🚨 {}stuck {}lost", 
        elapsed_time,
        performance_tracker.stuck_ants_count,
        performance_tracker.lost_ants_count
    );
    render_text_line(frame, width, &status_text, 5, 40, [255, 255, 0]); // Yellow text
    
    // Line 4: Deliveries count (y = 55-60)
    let delivery_text = format!("✅ {} deliveries total", performance_tracker.successful_deliveries);
    render_text_line(frame, width, &delivery_text, 5, 55, [0, 255, 0]); // Green text
}

fn render_text_line(frame: &mut [u8], width: u32, text: &str, x_start: u32, y_start: u32, color: [u8; 3]) {
    // Very simple character rendering - each character is 4x6 pixels with 1px spacing
    let char_width = 4;
    let char_height = 6;
    let char_spacing = 1;
    
    for (char_index, _char) in text.chars().enumerate() {
        let char_x = x_start + (char_index as u32) * (char_width + char_spacing);
        
        // Simple block character representation for visibility
        for dy in 0..char_height {
            for dx in 0..char_width {
                let px = char_x + dx;
                let py = y_start + dy;
                
                if px < width && py < 75 { // Keep within text overlay area
                    let idx = ((py * width + px) * 4) as usize;
                    if idx + 3 < frame.len() {
                        frame[idx] = color[0];     // R
                        frame[idx + 1] = color[1]; // G
                        frame[idx + 2] = color[2]; // B
                        frame[idx + 3] = 255;      // Full opacity
                    }
                }
            }
        }
    }
}