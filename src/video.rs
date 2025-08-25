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
        
        // Debug: Print frame count periodically
        if video_recorder.frames.len() % 60 == 0 {
            println!("📹 Captured {} frames at {:.1}s", video_recorder.frames.len(), time.elapsed_seconds());
        }
    }
    
    // Check if simulation is ending and should save video
    if should_save_video(&performance_tracker, &time) && video_recorder.is_recording {
        save_video_on_exit(&mut video_recorder, &performance_tracker, &generation_info);
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
        
        // Add enhanced directional indicator - a 2x2 bright white square in the direction the ant is facing
        let direction = ant_state.current_direction;
        let indicator_distance = 4.0; // Pixels from center, increased for better visibility
        let indicator_x = ant_x + (direction.cos() * indicator_distance) as i32;
        let indicator_y = ant_y + (direction.sin() * indicator_distance) as i32;
        
        // Draw a 2x2 pixel indicator for better visibility
        for dy in -1..1 {
            for dx in -1..1 {
                let px = indicator_x + dx;
                let py = indicator_y + dy;
                
                if px >= 0 && px < target_width as i32 && 
                   py >= 0 && py < target_height as i32 {
                    let idx = ((py as u32 * target_width + px as u32) * 4) as usize;
                    if idx + 3 < frame.len() {
                        frame[idx] = 255;     // Bright white indicator
                        frame[idx + 1] = 255;
                        frame[idx + 2] = 255;
                        frame[idx + 3] = 255;
                    }
                }
            }
        }
    }
    
    // Add comprehensive text overlay at top (first 85 pixels height to accommodate 5 lines)
    let text_height = 85;
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
    // TEMPORARY DEBUG: Only save after 90 seconds - disable all early exit conditions
    let elapsed = time.elapsed_seconds();
    let time_condition = elapsed > 90.0;
    
    if time_condition {
        println!("🎬 Video save triggered: 90 seconds elapsed ({:.1}s), frames: {}", elapsed, performance_tracker.successful_deliveries); // Use any field for frame count debug
    }
    
    // Print periodic status to debug what's happening
    if elapsed > 0.0 && (elapsed as u32) % 10 == 0 && elapsed.fract() < 0.1 {
        println!("📊 Status at {:.0}s: oscillating={}, lost_carriers={}", elapsed, performance_tracker.oscillating_ants_count, performance_tracker.lost_food_carriers_count);
    }
    
    time_condition // Only save after 90 seconds
}

fn save_video_on_exit(video_recorder: &mut VideoRecorder, performance_tracker: &PerformanceTracker, generation_info: &GenerationInfo) {
    // Create videos directory if it doesn't exist
    let videos_dir = "simulation_videos";
    if let Err(e) = fs::create_dir_all(videos_dir) {
        println!("❌ Failed to create videos directory: {}", e);
        return;
    }
    
    // Generate filename following convention: ####_description.mp4
    let filename = format!(
        "{}/{:04}_{}.mp4",
        videos_dir,
        generation_info.current_generation,
        generation_info.description.replace(" ", "_").to_lowercase()
    );
    
    println!("📹 Saving video: {}", filename);
    println!("   Changes: {}", video_recorder.changes_description);
    println!("   Frames captured: {}", video_recorder.frames.len());
    println!("   Final stats: {:.1}s avg goal time, {:.1}s return time", 
        performance_tracker.average_time_since_goal,
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
        
        // Save as PNG image
        let _ = save_frame_as_png(&frame_path, frame, video_recorder.frame_width, video_recorder.frame_height);
    }
    
    // Create metadata file  
    let metadata_file = filename.replace(".mp4", "_metadata.txt");
    let metadata = format!(
        "Generation {}\nChanges: {}\nAvg Goal Time: {:.1}s\nReturn time: {:.1}s\nFrames: {}\nDuration: {:.1} seconds (6x speed from entire simulation)\n",
        generation_info.current_generation,
        video_recorder.changes_description,
        performance_tracker.average_time_since_goal,
        performance_tracker.average_return_time,
        video_recorder.frames.len(),
        video_recorder.frames.len() as f32 / 6.0 / 30.0 // frames / speedup / fps
    );
    
    if let Err(e) = fs::write(&metadata_file, metadata) {
        println!("❌ Failed to write metadata: {}", e);
    } else {
        println!("✅ Video metadata saved: {}", metadata_file);
    }
    
    // Update generation_info.json with current performance metrics
    update_generation_info(generation_info, performance_tracker);
    
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

    let file = std::fs::File::create(path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    
    let mut writer = encoder.write_header()?;
    writer.write_image_data(frame_data)?;
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
    let gen_text = format!("GEN {}: {}", generation_info.current_generation, generation_info.description);
    render_text_line(frame, width, &gen_text, 5, 10, [255, 255, 255]); // White text
    
    // Line 2: Primary metric - Average Time Since Goal
    let perf_text = format!("AvgGoalTime: {:.1}s | {:.1}s return", 
        performance_tracker.average_time_since_goal,
        performance_tracker.average_return_time
    );
    render_text_line(frame, width, &perf_text, 5, 25, [0, 255, 255]); // Cyan text
    
    // Line 3: Time and issues (y = 40-45) - Split into two lines to prevent overflow
    let time_text = format!("T: {:.0}s elapsed", elapsed_time);
    render_text_line(frame, width, &time_text, 5, 40, [255, 255, 0]); // Yellow text
    
    // Line 4: Issues status (y = 55-60)
    let issues_text = format!("Issues: {}stuck {}lost", 
        performance_tracker.stuck_ants_count,
        performance_tracker.lost_ants_count
    );
    render_text_line(frame, width, &issues_text, 5, 55, [255, 100, 0]); // Orange text
    
    // Line 5: Deliveries count (y = 70-75) - Move down to accommodate split lines
    let delivery_text = format!("D: {} deliveries total", performance_tracker.successful_deliveries);
    render_text_line(frame, width, &delivery_text, 5, 70, [0, 255, 0]); // Green text
}

fn render_text_line(frame: &mut [u8], width: u32, text: &str, x_start: u32, y_start: u32, color: [u8; 3]) {
    // Better character rendering with actual readable patterns
    let char_width = 6;
    let char_height = 8;
    let char_spacing = 1;
    
    for (char_index, ch) in text.chars().enumerate() {
        let char_x = x_start + (char_index as u32) * (char_width + char_spacing);
        
        // Get the bitmap pattern for this character
        let pattern = get_char_pattern(ch);
        
        // Render the character based on its bitmap pattern
        for (dy, row) in pattern.iter().enumerate() {
            for dx in 0..char_width {
                let px = char_x + dx;
                let py = y_start + dy as u32;
                
                if px < width && py < 85 { // Keep within expanded text overlay area
                    let idx = ((py * width + px) * 4) as usize;
                    if idx + 3 < frame.len() {
                        // Check if this pixel should be lit based on the bitmap
                        let bit_index = char_width - 1 - dx; // Reverse for correct orientation
                        let pixel_on = (row & (1 << bit_index)) != 0;
                        
                        if pixel_on {
                            frame[idx] = color[0];     // R
                            frame[idx + 1] = color[1]; // G
                            frame[idx + 2] = color[2]; // B
                            frame[idx + 3] = 255;      // Full opacity
                        }
                        // Don't modify transparent pixels (leave background as is)
                    }
                }
            }
        }
    }
}

fn get_char_pattern(ch: char) -> [u8; 8] {
    // 6x8 bitmap patterns for common characters (each u8 represents a row)
    match ch {
        'G' => [0b011110, 0b100001, 0b100000, 0b100111, 0b100001, 0b100001, 0b011110, 0b000000],
        'E' => [0b111111, 0b100000, 0b100000, 0b111100, 0b100000, 0b100000, 0b111111, 0b000000],
        'N' => [0b100001, 0b110001, 0b101001, 0b100101, 0b100011, 0b100001, 0b100001, 0b000000],
        'R' => [0b111110, 0b100001, 0b100001, 0b111110, 0b100010, 0b100001, 0b100001, 0b000000],
        'A' => [0b011110, 0b100001, 0b100001, 0b111111, 0b100001, 0b100001, 0b100001, 0b000000],
        'T' => [0b111111, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b000000],
        'I' => [0b111111, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b111111, 0b000000],
        'O' => [0b011110, 0b100001, 0b100001, 0b100001, 0b100001, 0b100001, 0b011110, 0b000000],
        'S' => [0b011111, 0b100000, 0b100000, 0b011110, 0b000001, 0b000001, 0b111110, 0b000000],
        'L' => [0b100000, 0b100000, 0b100000, 0b100000, 0b100000, 0b100000, 0b111111, 0b000000],
        'C' => [0b011110, 0b100001, 0b100000, 0b100000, 0b100000, 0b100001, 0b011110, 0b000000],
        'D' => [0b111110, 0b100001, 0b100001, 0b100001, 0b100001, 0b100001, 0b111110, 0b000000],
        'M' => [0b100001, 0b110011, 0b101101, 0b100001, 0b100001, 0b100001, 0b100001, 0b000000],
        'P' => [0b111110, 0b100001, 0b100001, 0b111110, 0b100000, 0b100000, 0b100000, 0b000000],
        'U' => [0b100001, 0b100001, 0b100001, 0b100001, 0b100001, 0b100001, 0b011110, 0b000000],
        'V' => [0b100001, 0b100001, 0b100001, 0b100001, 0b010010, 0b001100, 0b001100, 0b000000],
        'H' => [0b100001, 0b100001, 0b100001, 0b111111, 0b100001, 0b100001, 0b100001, 0b000000],
        'F' => [0b111111, 0b100000, 0b100000, 0b111100, 0b100000, 0b100000, 0b100000, 0b000000],
        'K' => [0b100001, 0b100010, 0b100100, 0b111000, 0b100100, 0b100010, 0b100001, 0b000000],
        'Y' => [0b100001, 0b100001, 0b010010, 0b001100, 0b001000, 0b001000, 0b001000, 0b000000],
        'B' => [0b111110, 0b100001, 0b100001, 0b111110, 0b100001, 0b100001, 0b111110, 0b000000],
        'W' => [0b100001, 0b100001, 0b100001, 0b100001, 0b101101, 0b110011, 0b100001, 0b000000],
        'X' => [0b100001, 0b010010, 0b001100, 0b001100, 0b010010, 0b100001, 0b100001, 0b000000],
        'Z' => [0b111111, 0b000001, 0b000010, 0b001100, 0b010000, 0b100000, 0b111111, 0b000000],
        'J' => [0b000001, 0b000001, 0b000001, 0b000001, 0b100001, 0b100001, 0b011110, 0b000000],
        'Q' => [0b011110, 0b100001, 0b100001, 0b101001, 0b100010, 0b011110, 0b000001, 0b000000],
        // Lowercase letters
        'a' => [0b000000, 0b000000, 0b011110, 0b000001, 0b011111, 0b100001, 0b011111, 0b000000],
        'b' => [0b100000, 0b100000, 0b111110, 0b100001, 0b100001, 0b100001, 0b111110, 0b000000],
        'c' => [0b000000, 0b000000, 0b011110, 0b100000, 0b100000, 0b100000, 0b011110, 0b000000],
        'd' => [0b000001, 0b000001, 0b011111, 0b100001, 0b100001, 0b100001, 0b011111, 0b000000],
        'e' => [0b000000, 0b000000, 0b011110, 0b100001, 0b111110, 0b100000, 0b011110, 0b000000],
        'f' => [0b001111, 0b010000, 0b010000, 0b111110, 0b010000, 0b010000, 0b010000, 0b000000],
        'g' => [0b000000, 0b000000, 0b011111, 0b100001, 0b011111, 0b000001, 0b011110, 0b000000],
        'h' => [0b100000, 0b100000, 0b111110, 0b100001, 0b100001, 0b100001, 0b100001, 0b000000],
        'i' => [0b001000, 0b000000, 0b011000, 0b001000, 0b001000, 0b001000, 0b011100, 0b000000],
        'j' => [0b000010, 0b000000, 0b000110, 0b000010, 0b000010, 0b100010, 0b011100, 0b000000],
        'k' => [0b100000, 0b100000, 0b100010, 0b100100, 0b111000, 0b100100, 0b100010, 0b000000],
        'l' => [0b011000, 0b001000, 0b001000, 0b001000, 0b001000, 0b001000, 0b011100, 0b000000],
        'm' => [0b000000, 0b000000, 0b110110, 0b101101, 0b101101, 0b101101, 0b101101, 0b000000],
        'n' => [0b000000, 0b000000, 0b111110, 0b100001, 0b100001, 0b100001, 0b100001, 0b000000],
        'o' => [0b000000, 0b000000, 0b011110, 0b100001, 0b100001, 0b100001, 0b011110, 0b000000],
        'p' => [0b000000, 0b000000, 0b111110, 0b100001, 0b111110, 0b100000, 0b100000, 0b000000],
        'q' => [0b000000, 0b000000, 0b011111, 0b100001, 0b011111, 0b000001, 0b000001, 0b000000],
        'r' => [0b000000, 0b000000, 0b101110, 0b110000, 0b100000, 0b100000, 0b100000, 0b000000],
        's' => [0b000000, 0b000000, 0b011110, 0b100000, 0b011110, 0b000001, 0b111110, 0b000000],
        't' => [0b010000, 0b010000, 0b111110, 0b010000, 0b010000, 0b010000, 0b001110, 0b000000],
        'u' => [0b000000, 0b000000, 0b100001, 0b100001, 0b100001, 0b100001, 0b011111, 0b000000],
        'v' => [0b000000, 0b000000, 0b100001, 0b100001, 0b010010, 0b010010, 0b001100, 0b000000],
        'w' => [0b000000, 0b000000, 0b100001, 0b101101, 0b101101, 0b101101, 0b010010, 0b000000],
        'x' => [0b000000, 0b000000, 0b100001, 0b010010, 0b001100, 0b010010, 0b100001, 0b000000],
        'y' => [0b000000, 0b000000, 0b100001, 0b100001, 0b011111, 0b000001, 0b011110, 0b000000],
        'z' => [0b000000, 0b000000, 0b111111, 0b000010, 0b001100, 0b100000, 0b111111, 0b000000],
        // Emoji replacements with simple patterns  
        '📊' => [0b111111, 0b001001, 0b010101, 0b100001, 0b100001, 0b111111, 0b000000, 0b000000], // Chart bars
        '⏱' => [0b011110, 0b100001, 0b100101, 0b100101, 0b100001, 0b011110, 0b000000, 0b000000], // Clock
        '️' => [0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000], // Variation selector (invisible)
        '🚨' => [0b001100, 0b010010, 0b100001, 0b111111, 0b111111, 0b111111, 0b000000, 0b000000], // Warning
        '📈' => [0b000001, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000, 0b111111, 0b000000], // Graph up
        ' ' => [0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b000000],
        ':' => [0b000000, 0b011000, 0b011000, 0b000000, 0b011000, 0b011000, 0b000000, 0b000000],
        '.' => [0b000000, 0b000000, 0b000000, 0b000000, 0b000000, 0b011000, 0b011000, 0b000000],
        '/' => [0b000001, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000, 0b000000, 0b000000],
        '0' => [0b011110, 0b100001, 0b100011, 0b100101, 0b101001, 0b100001, 0b011110, 0b000000],
        '1' => [0b001000, 0b011000, 0b001000, 0b001000, 0b001000, 0b001000, 0b011100, 0b000000],
        '2' => [0b011110, 0b100001, 0b000001, 0b011110, 0b100000, 0b100000, 0b111111, 0b000000],
        '3' => [0b011110, 0b100001, 0b000001, 0b011110, 0b000001, 0b100001, 0b011110, 0b000000],
        '4' => [0b100001, 0b100001, 0b100001, 0b111111, 0b000001, 0b000001, 0b000001, 0b000000],
        '5' => [0b111111, 0b100000, 0b100000, 0b111110, 0b000001, 0b000001, 0b111110, 0b000000],
        '6' => [0b011110, 0b100000, 0b100000, 0b111110, 0b100001, 0b100001, 0b011110, 0b000000],
        '7' => [0b111111, 0b000001, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000, 0b000000],
        '8' => [0b011110, 0b100001, 0b100001, 0b011110, 0b100001, 0b100001, 0b011110, 0b000000],
        '9' => [0b011110, 0b100001, 0b100001, 0b011111, 0b000001, 0b000001, 0b011110, 0b000000],
        // Default for unknown characters - show a small box
        _ => [0b111111, 0b100001, 0b100001, 0b100001, 0b100001, 0b100001, 0b111111, 0b000000],
    }
}

fn update_generation_info(generation_info: &GenerationInfo, performance_tracker: &PerformanceTracker) {
    let updated_json = serde_json::json!({
        "current_generation": generation_info.current_generation,
        "description": generation_info.description,
        "timestamp": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "video_filename": format!("{:04}_{}.mp4", generation_info.current_generation, generation_info.description.replace(" ", "_").to_lowercase()),
        "performance_metrics": {
            "average_time_since_goal_seconds": performance_tracker.average_time_since_goal,
            "average_return_time_seconds": performance_tracker.average_return_time,
            "successful_deliveries": performance_tracker.successful_deliveries,
            "simulation_duration_seconds": 90,
            "total_food_collected": performance_tracker.total_food_collected
        }
    });
    
    if let Ok(json_string) = serde_json::to_string_pretty(&updated_json) {
        if let Err(e) = fs::write("generation_info.json", json_string) {
            println!("❌ Failed to update generation_info.json: {}", e);
        } else {
            println!("✅ Updated generation_info.json with current performance metrics");
        }
    }
}