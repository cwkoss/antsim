use bevy::prelude::*;
use bevy::window::{WindowCloseRequested, PrimaryWindow};
use rand::Rng;
use crate::components::*;
use crate::config::*;
use crate::pheromones::*;
use crate::colors::*;

pub fn sensing_system(
    mut ants: Query<(&Transform, &mut AntState, &mut Velocity)>,
    pheromone_grid: Option<Res<PheromoneGrid>>,
    config: Res<SimConfig>,
    time: Res<Time>,
) {
    if let Some(grid) = pheromone_grid {
        for (transform, mut ant, mut velocity) in ants.iter_mut() {
            let pos = transform.translation;
            let delta_time = time.delta_seconds();
            
            // Update timers
            ant.sensing_timer -= delta_time;
            ant.momentum_timer -= delta_time;
            ant.startup_timer -= delta_time;
            
            // Don't process ants that are collecting food
            if ant.food_collection_timer > 0.0 {
                continue;
            }
            
            // STARTUP GRACE PERIOD: Simple exploration for newly spawned ants
            if ant.startup_timer > 0.0 {
                // Simple, smooth exploration without sensing complexity
                apply_stable_exploration(&mut velocity, &config, ant.current_direction);
                continue; // Skip all the complex behavior
            }
            
            // Determine pheromone type to follow
            let pheromone_type = if ant.carrying_food {
                PheromoneType::Nest
            } else {
                PheromoneType::Food
            };
            
            match ant.behavior_state {
                AntBehaviorState::Exploring => {
                    // SIMPLIFIED: Just do stable exploration, no complex sensing
                    apply_stable_exploration(&mut velocity, &config, ant.current_direction);
                },
                
                AntBehaviorState::Sensing => {
                    // SIMPLIFIED: Skip complex sensing, just do stable exploration
                    ant.behavior_state = AntBehaviorState::Exploring;
                    apply_stable_exploration(&mut velocity, &config, ant.current_direction);
                    /*
                    if ant.sensing_timer <= 0.0 {
                        // SUPER INTELLIGENT SENSING: Use all 5 advanced techniques
                        let samples = grid.sample_all_directions(pos.x, pos.y, pheromone_type);
                        ant.last_sensing_result = samples;
                        
                        // 1. GRADIENT-BASED DIRECTION: Calculate weighted direction instead of strongest
                        let (gradient_direction, max_strength, calculated_quality) = 
                            calculate_gradient_direction(&samples, ant.current_direction, &ant.trail_memory);
                        
                        // 2. TRAIL QUALITY ASSESSMENT: Evaluate trail consistency and strength
                        ant.trail_quality = assess_trail_quality(&samples, ant.trail_quality);
                        
                        // 3. HYSTERESIS SYSTEM: Dynamic thresholds based on current state
                        let mut effective_threshold = ant.hysteresis_threshold;
                        if ant.behavior_state == AntBehaviorState::Following {
                            // Higher threshold to stay on current trail (stickiness)
                            effective_threshold *= 0.6; // 40% easier to continue following
                        } else {
                            // Standard threshold for new trails
                            effective_threshold = config.detection_threshold;
                        }
                        
                        // Check direction change magnitude for oscillation detection
                        let direction_change = (gradient_direction - ant.current_direction).abs().min(
                            2.0 * std::f32::consts::PI - (gradient_direction - ant.current_direction).abs()
                        );
                        
                        if direction_change > std::f32::consts::PI / 2.0 { // > 90 degree change (more lenient)
                            ant.direction_changes += 1;
                        }
                        
                        // Update dynamic threshold based on trail quality and recent behavior
                        ant.hysteresis_threshold = if ant.trail_quality > 0.7 {
                            config.detection_threshold * 0.8 // Lower threshold for high quality trails
                        } else {
                            config.detection_threshold * 1.2 // Higher threshold for poor quality trails
                        };
                        
                        ant.trail_strength = max_strength;
                        ant.last_pheromone_strength = max_strength;
                        
                        // Decision logic with quality and hysteresis considerations (more lenient)
                        let should_follow = max_strength > effective_threshold && 
                                          ant.trail_quality > 0.1 && // TEST 1: Lower quality requirement
                                          calculated_quality > 0.05; // TEST 1: Much lower immediate sensing quality requirement
                        
                        if should_follow {
                            // Check anti-oscillation measures
                            if ant.direction_changes > 8 && ant.stuck_timer > 5.0 { // TEST 1: More lenient anti-oscillation
                                // Force long exploration to break pathological oscillation
                                ant.behavior_state = AntBehaviorState::Exploring;
                                ant.sensing_timer = 5.0 + rand::random::<f32>() * 3.0;
                                ant.direction_changes = 0;
                                ant.consecutive_good_trail_time = 0.0;
                                apply_random_walk(&mut velocity, &config);
                            } else {
                                // 4. TRAIL MEMORY: Update memory with new direction
                                let current_index = ant.memory_index;
                                ant.trail_memory[current_index] = gradient_direction;
                                ant.memory_index = (current_index + 1) % ant.trail_memory.len();
                                
                                // Transition to following high-quality trail
                                ant.behavior_state = AntBehaviorState::Following;
                                ant.current_direction = gradient_direction;
                                
                                // Momentum based on trail quality (better trails = longer commitment)
                                ant.momentum_timer = 1.0 + ant.trail_quality * 3.0;
                                ant.consecutive_good_trail_time += delta_time;
                                
                                let speed = if ant.carrying_food { 85.0 } else { 110.0 };
                                velocity.x = gradient_direction.cos() * speed;
                                velocity.y = gradient_direction.sin() * speed;
                            }
                        } else {
                            // No suitable trail found
                            ant.behavior_state = AntBehaviorState::Exploring;
                            ant.sensing_timer = 1.5 + rand::random::<f32>() * 2.0;
                            ant.consecutive_good_trail_time = 0.0;
                            apply_random_walk(&mut velocity, &config);
                        }
                    }
                    */
                },
                
                AntBehaviorState::Following => {
                    // 5. PREDICTIVE SENSING: Look ahead in current direction instead of full 8-direction scan
                    let ahead_distance = 20.0;
                    let current_strength = grid.sample_directional(pos.x, pos.y, ant.current_direction, ahead_distance, pheromone_type);
                    
                    // Sample slightly left and right of current direction for course correction
                    let left_direction = ant.current_direction - std::f32::consts::PI / 8.0; // 22.5 degrees left
                    let right_direction = ant.current_direction + std::f32::consts::PI / 8.0; // 22.5 degrees right
                    let left_strength = grid.sample_directional(pos.x, pos.y, left_direction, ahead_distance, pheromone_type);
                    let right_strength = grid.sample_directional(pos.x, pos.y, right_direction, ahead_distance, pheromone_type);
                    
                    // Update trail quality based on predictive sensing
                    let predictive_samples = [0.0, 0.0, right_strength, 0.0, current_strength, 0.0, left_strength, 0.0];
                    ant.trail_quality = assess_trail_quality(&predictive_samples, ant.trail_quality);
                    
                    // Hysteresis: Trail must weaken significantly to abandon (stickiness)
                    let abandon_threshold = ant.trail_strength * 0.4; // More tolerant than before
                    let quality_threshold = 0.2; // Minimum acceptable quality while following
                    
                    if current_strength < abandon_threshold || 
                       ant.trail_quality < quality_threshold || 
                       ant.momentum_timer <= 0.0 {
                        // Trail degraded enough to warrant re-sensing
                        ant.behavior_state = AntBehaviorState::Sensing;
                        ant.sensing_timer = 0.2; // Quick re-sense
                        ant.consecutive_good_trail_time = 0.0;
                    } else {
                        // Continue following with potential course correction
                        let mut best_direction = ant.current_direction;
                        let mut best_strength = current_strength;
                        
                        // Minor course corrections based on predictive sensing
                        if left_strength > best_strength * 1.15 { // 15% better to left
                            best_direction = left_direction;
                            best_strength = left_strength;
                        } else if right_strength > best_strength * 1.15 { // 15% better to right
                            best_direction = right_direction;
                            best_strength = right_strength;
                        }
                        
                        // Update trail memory with refined direction
                        if best_direction != ant.current_direction {
                            let current_index = ant.memory_index;
                            ant.trail_memory[current_index] = best_direction;
                            ant.memory_index = (current_index + 1) % ant.trail_memory.len();
                        }
                        
                        ant.current_direction = best_direction;
                        ant.trail_strength = best_strength;
                        ant.consecutive_good_trail_time += delta_time;
                        
                        // Speed based on trail quality and confidence
                        let base_speed = if ant.carrying_food { 85.0 } else { 115.0 };
                        let quality_multiplier = 0.8 + (ant.trail_quality * 0.4); // 0.8 to 1.2x speed
                        let speed = base_speed * quality_multiplier;
                        
                        velocity.x = ant.current_direction.cos() * speed;
                        velocity.y = ant.current_direction.sin() * speed;
                    }
                },
                
                AntBehaviorState::Tracking => {
                    // Monitor gradient while moving
                    let samples = grid.sample_all_directions(pos.x, pos.y, pheromone_type);
                    let (best_direction, best_strength, _quality) = calculate_gradient_direction(&samples, ant.current_direction, &ant.trail_memory);
                    
                    if best_strength > ant.trail_strength * 1.5 {
                        // Much stronger trail found
                        ant.behavior_state = AntBehaviorState::Following;
                        ant.current_direction = best_direction;
                        ant.trail_strength = best_strength;
                        ant.momentum_timer = 2.0;
                    } else if ant.momentum_timer <= 0.0 {
                        ant.behavior_state = AntBehaviorState::Sensing;
                        ant.sensing_timer = 0.1;
                    }
                }
            }
            
            // Update current direction based on velocity
            if velocity.x.abs() > 0.1 || velocity.y.abs() > 0.1 {
                ant.current_direction = velocity.y.atan2(velocity.x);
            }
            
            // Stuck detection and performance tracking
            let current_pos = Vec2::new(pos.x, pos.y);
            let distance_moved = current_pos.distance(ant.last_position);
            
            if distance_moved < 5.0 { // Haven't moved much (tighter threshold)
                ant.stuck_timer += delta_time;
            } else {
                ant.stuck_timer = 0.0;
                ant.direction_changes = (ant.direction_changes.saturating_sub(1)).max(0); // Decay direction changes over time
            }
            ant.last_position = current_pos;
            
            // Adapt sensitivity to avoid saturation
            if ant.trail_strength > 0.1 {
                ant.sensitivity_adapt = (ant.sensitivity_adapt * 0.99 + ant.trail_strength * 0.01).min(1.0);
            } else {
                ant.sensitivity_adapt = (ant.sensitivity_adapt * 0.99 + 0.01).max(0.1);
            }
        }
    }
}

fn calculate_gradient_direction(samples: &[f32; 8], current_direction: f32, trail_memory: &[f32; 5]) -> (f32, f32, f32) {
    let directions = [
        0.0,                          // North
        std::f32::consts::PI / 4.0,           // NE  
        std::f32::consts::PI / 2.0,           // East
        3.0 * std::f32::consts::PI / 4.0,     // SE
        std::f32::consts::PI,                 // South
        5.0 * std::f32::consts::PI / 4.0,     // SW
        3.0 * std::f32::consts::PI / 2.0,     // West
        7.0 * std::f32::consts::PI / 4.0,     // NW
    ];
    
    // Calculate weighted average direction based on pheromone strengths
    let mut weighted_x = 0.0;
    let mut weighted_y = 0.0;
    let mut total_weight = 0.0;
    let mut max_strength: f32 = 0.0;
    
    for (i, &strength) in samples.iter().enumerate() {
        if strength > 0.001 { // Only consider meaningful strengths
            let direction = directions[i];
            let weight = strength.powf(2.0); // Square for more decisive weighting
            
            weighted_x += direction.cos() * weight;
            weighted_y += direction.sin() * weight;
            total_weight += weight;
            max_strength = max_strength.max(strength);
        }
    }
    
    if total_weight > 0.001 {
        // Normalize weighted direction vector to avoid bias
        let gradient_length = (weighted_x * weighted_x + weighted_y * weighted_y).sqrt();
        if gradient_length > 0.001 {
            weighted_x /= gradient_length;
            weighted_y /= gradient_length;
        }
        
        let gradient_direction = weighted_y.atan2(weighted_x);
        
        // Apply trail memory influence (prefer consistency)
        let memory_influence = calculate_memory_direction(trail_memory);
        let memory_weight = 0.2; // Reduced memory influence to 20%
        
        // More careful vector averaging to avoid bias
        let gradient_vec_x = gradient_direction.cos();
        let gradient_vec_y = gradient_direction.sin();
        let memory_vec_x = memory_influence.cos();
        let memory_vec_y = memory_influence.sin();
        
        let final_x = gradient_vec_x * (1.0 - memory_weight) + memory_vec_x * memory_weight;
        let final_y = gradient_vec_y * (1.0 - memory_weight) + memory_vec_y * memory_weight;
        
        // Fix rightward bias: ensure proper atan2 usage and normalize
        let final_length = (final_x * final_x + final_y * final_y).sqrt();
        let final_direction = if final_length > 0.001 {
            final_y.atan2(final_x) // Correct atan2 usage: atan2(y, x)
        } else {
            current_direction
        };
        
        // Calculate trail quality (consistency across samples)
        let variance = calculate_direction_variance(samples, &directions);
        let quality = (1.0 / (1.0 + variance * 3.0)).clamp(0.0, 1.0); // Less harsh quality penalty
        
        (final_direction, max_strength, quality)
    } else {
        // No significant pheromone detected - continue in current direction or random
        (current_direction, 0.0, 0.0)
    }
}

fn calculate_memory_direction(trail_memory: &[f32; 5]) -> f32 {
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    
    for &direction in trail_memory {
        sum_x += direction.cos();
        sum_y += direction.sin();
    }
    
    sum_y.atan2(sum_x)
}

fn calculate_direction_variance(samples: &[f32; 8], directions: &[f32; 8]) -> f32 {
    let mut weighted_directions = Vec::new();
    
    for (i, &strength) in samples.iter().enumerate() {
        if strength > 0.001 {
            for _ in 0..(strength * 100.0) as usize {
                weighted_directions.push(directions[i]);
            }
        }
    }
    
    if weighted_directions.len() < 2 {
        return 0.0;
    }
    
    let mean = weighted_directions.iter().sum::<f32>() / weighted_directions.len() as f32;
    let variance: f32 = weighted_directions
        .iter()
        .map(|&x| {
            let diff = (x - mean + std::f32::consts::PI) % (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
            diff * diff
        })
        .sum::<f32>() / weighted_directions.len() as f32;
    
    variance
}

fn update_trail_memory(trail_memory: &mut [f32; 5], memory_index: &mut usize, new_direction: f32) {
    trail_memory[*memory_index] = new_direction;
    *memory_index = (*memory_index + 1) % trail_memory.len();
}

fn assess_trail_quality(samples: &[f32; 8], previous_quality: f32) -> f32 {
    let max_strength: f32 = samples.iter().fold(0.0f32, |a, &b| a.max(b));
    let total_strength: f32 = samples.iter().sum();
    let average_strength = total_strength / 8.0;
    
    // Quality factors:
    // 1. Strength consistency (low variance is good)
    // 2. Overall strength level
    // 3. Presence of clear gradient (not uniform)
    
    let consistency = if max_strength > 0.001 {
        1.0 - (total_strength - max_strength * 8.0).abs() / (max_strength * 8.0)
    } else {
        0.0
    };
    
    let strength_factor = (max_strength * 2.0).clamp(0.0, 1.0);
    let gradient_clarity = if total_strength > 0.001 {
        max_strength / average_strength
    } else {
        1.0
    };
    
    let new_quality = (consistency * 0.4 + strength_factor * 0.4 + (gradient_clarity / 8.0) * 0.2).clamp(0.0, 1.0);
    
    // Smooth quality changes
    previous_quality * 0.7 + new_quality * 0.3
}

fn apply_random_walk(velocity: &mut Velocity, config: &SimConfig) {
    // Generate truly random angle to avoid directional bias
    let random_angle = rand::random::<f32>() * std::f32::consts::TAU;
    let noise_strength = config.base_exploration_noise * 150.0;
    
    // Apply noise in random direction
    velocity.x += random_angle.cos() * noise_strength;
    velocity.y += random_angle.sin() * noise_strength;
    
    let speed_limit = 120.0;
    let current_speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
    if current_speed > speed_limit {
        velocity.x = (velocity.x / current_speed) * speed_limit;
        velocity.y = (velocity.y / current_speed) * speed_limit;
    }
}

fn apply_stable_exploration(velocity: &mut Velocity, _config: &SimConfig, current_direction: f32) {
    // VERY gentle exploration with minimal jitter
    let current_speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
    
    if current_speed < 30.0 {
        // If moving too slow, give it a direction based on current direction with slight randomness
        let slight_variation = (rand::random::<f32>() - 0.5) * 0.1; // ±0.05 radians = ±3 degrees
        let new_direction = current_direction + slight_variation;
        velocity.x = new_direction.cos() * 70.0;
        velocity.y = new_direction.sin() * 70.0;
    } else {
        // VERY small course corrections - like a gentle drift
        let tiny_adjustment = (rand::random::<f32>() - 0.5) * 0.05; // ±0.025 radians = ±1.4 degrees  
        let new_direction = current_direction + tiny_adjustment;
        
        // Very gradual adjustment toward new direction
        let target_x = new_direction.cos() * 70.0;
        let target_y = new_direction.sin() * 70.0;
        
        // 95% current direction, 5% adjustment - very smooth
        velocity.x = velocity.x * 0.95 + target_x * 0.05;
        velocity.y = velocity.y * 0.95 + target_y * 0.05;
    }
    
    // Maintain consistent speed
    let final_speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
    if final_speed > 0.1 {
        let target_speed = 70.0;
        velocity.x = (velocity.x / final_speed) * target_speed;
        velocity.y = (velocity.y / final_speed) * target_speed;
    }
}

pub fn movement_system(
    mut ants: Query<(&mut Transform, &mut Velocity, &mut AntState)>,
    pheromone_grid: Option<Res<PheromoneGrid>>,
    nests: Query<&Transform, (With<Nest>, Without<AntState>)>,
    config: Res<SimConfig>,
) {
    // Get nest position for direct homing
    let nest_pos = if let Ok(nest_transform) = nests.get_single() {
        nest_transform.translation
    } else {
        Vec3::ZERO
    };
    
    if let Some(grid) = pheromone_grid {
        for (mut transform, mut velocity, mut ant) in ants.iter_mut() {
            // Don't move ants that are collecting food
            if ant.food_collection_timer > 0.0 {
                continue;
            }
            let pos = transform.translation;
            
            // Determine which pheromone to follow
            let pheromone_type = if ant.carrying_food {
                PheromoneType::Nest
            } else {
                PheromoneType::Food
            };
            
            let (left, front, right) = grid.sample_gradient(pos.x, pos.y, pheromone_type);
            
            // Apply sensitivity adaptation
            let adapted_left = (left - ant.sensitivity_adapt).max(0.0).min(config.saturation_limit);
            let adapted_front = (front - ant.sensitivity_adapt).max(0.0).min(config.saturation_limit);
            let adapted_right = (right - ant.sensitivity_adapt).max(0.0).min(config.saturation_limit);
            
            let max_pheromone = adapted_left.max(adapted_front).max(adapted_right);
            
            // Movement decision
            let mut rng = rand::thread_rng();
            // Very high threshold for food-carrying ants to prioritize direct homing
            let detection_threshold = if ant.carrying_food {
                config.detection_threshold * 100.0 // 100x higher threshold = almost ignore pheromones
            } else {
                config.detection_threshold
            };
            
            let direction = if max_pheromone > detection_threshold {
                // Check if transitioning from exploration to trail following for faster turning
                let was_exploring = ant.last_pheromone_strength <= detection_threshold;
                let turn_strength = if was_exploring && !ant.carrying_food {
                    2.0 // Much stronger turn when first detecting trail
                } else {
                    0.8 // Normal trail following turn strength
                };
                
                // Follow gradient deterministically - turn toward strongest pheromone
                if adapted_left > adapted_front && adapted_left > adapted_right {
                    -turn_strength // Turn left toward strongest pheromone
                } else if adapted_right > adapted_front && adapted_right > adapted_left {
                    turn_strength // Turn right toward strongest pheromone
                } else {
                    // Front is strongest (or tied) - go straight with small random component
                    (rng.gen::<f32>() - 0.5) * 0.2
                }
            } else if ant.carrying_food {
                // CRITICAL FIX: Ultra-reliable nest homing for food-carrying ants
                let to_nest = nest_pos - pos;
                let distance_to_nest = to_nest.length();
                
                // Always home to nest regardless of distance - never get lost!
                if distance_to_nest > 5.0 { // Any distance from nest center
                    let current_angle = velocity.y.atan2(velocity.x);
                    let desired_angle = to_nest.y.atan2(to_nest.x);
                    let angle_diff = (desired_angle - current_angle + std::f32::consts::PI) % (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
                    
                    // EMERGENCY: Maximum possible homing strength to fix lost carriers
                    let homing_strength = if distance_to_nest > 400.0 {
                        8.0 // Emergency ultra-strong homing when very far
                    } else if distance_to_nest > 300.0 {
                        7.0 // Ultra-strong homing when far
                    } else if distance_to_nest > 200.0 {
                        6.0 // Very strong homing
                    } else if distance_to_nest > 100.0 {
                        5.0 // Strong homing
                    } else {
                        4.0 // Still strong when close
                    };
                    
                    // Clamp the angle difference to prevent extreme turns
                    let clamped_angle_diff = angle_diff.clamp(-1.0, 1.0);
                    clamped_angle_diff * homing_strength
                } else {
                    // Very close to nest - minimal correction
                    0.1
                }
            } else {
                // Random exploration for exploring ants
                rng.gen::<f32>() * 2.0 - 1.0
            };
            
            // Update velocity
            let current_angle = velocity.y.atan2(velocity.x);
            
            // Use less noise based on ant state and pheromone detection
            let exploration_factor = if ant.carrying_food {
                config.base_exploration_noise * 0.01 // MINIMAL random turning when carrying food
            } else if max_pheromone > detection_threshold {
                config.base_exploration_noise * 0.1 // Much less noise when following pheromone trail
            } else {
                config.base_exploration_noise // Full exploration when no pheromones detected
            };
            
            // CRITICAL: Special handling for food-carrying ants - bypass exploration factor
            if ant.carrying_food {
                // Direct angle calculation for nest homing - ignore exploration factor completely
                let to_nest = nest_pos - pos;
                if to_nest.length() > 5.0 {
                    let nest_angle = to_nest.y.atan2(to_nest.x);
                    let speed = 2.5; // Even faster for urgent nest return
                    velocity.x = nest_angle.cos() * speed;
                    velocity.y = nest_angle.sin() * speed;
                } else {
                    // Very close to nest - slow down
                    velocity.x *= 0.5;
                    velocity.y *= 0.5;
                }
            } else {
                // Normal movement for non-food-carrying ants
                let new_angle = current_angle + direction * exploration_factor;
                let speed = 2.0; // OPTIMIZATION 1: Increased from 1.5 to 2.0 for faster movement
                
                velocity.x = new_angle.cos() * speed;
                velocity.y = new_angle.sin() * speed;
            }
            
            // Update position
            let old_pos = Vec2::new(transform.translation.x, transform.translation.y);
            transform.translation.x += velocity.x;
            transform.translation.y += velocity.y;
            
            // BOUNDARY HANDLING: Bounce off edges to prevent getting stuck
            let world_boundary = (config.world_size as f32) * 0.5; // -500 to +500
            let mut bounced = false;
            
            if transform.translation.x < -world_boundary {
                transform.translation.x = -world_boundary;
                velocity.x = velocity.x.abs(); // Bounce right
                bounced = true;
            } else if transform.translation.x > world_boundary {
                transform.translation.x = world_boundary;
                velocity.x = -velocity.x.abs(); // Bounce left
                bounced = true;
            }
            
            if transform.translation.y < -world_boundary {
                transform.translation.y = -world_boundary;
                velocity.y = velocity.y.abs(); // Bounce up
                bounced = true;
            } else if transform.translation.y > world_boundary {
                transform.translation.y = world_boundary;
                velocity.y = -velocity.y.abs(); // Bounce down
                bounced = true;
            }
            
            // If bounced, update current direction to match new velocity
            if bounced {
                ant.current_direction = velocity.y.atan2(velocity.x);
            }
            
            let new_pos = Vec2::new(transform.translation.x, transform.translation.y);
            
            // Track distance traveled for pheromone strength calculations
            let travel_distance = old_pos.distance(new_pos);
            
            if ant.carrying_food {
                ant.distance_from_food += travel_distance;
            } else {
                ant.distance_from_food = 0.0; // Reset when not carrying food
                ant.distance_from_nest += travel_distance; // Track distance from nest when exploring
            }
            
            // Reset nest distance when near nest (within 50 units)
            let distance_to_nest = new_pos.distance(Vec2::new(nest_pos.x, nest_pos.y));
            if distance_to_nest < 50.0 {
                ant.distance_from_nest = 0.0;
                
                // If ant is not carrying food, near nest, and hasn't chosen exit direction yet
                if !ant.carrying_food && !ant.has_exit_direction {
                    let mut best_direction = Vec2::ZERO;
                    let mut max_food_strength = 0.0;
                    
                    // Sample in a wider radius around the nest for food pheromones
                    let sample_radius = 80.0;
                    let sample_points = 16; // Check 16 directions
                    
                    for i in 0..sample_points {
                        let angle = (i as f32) * std::f32::consts::TAU / sample_points as f32;
                        let sample_x = new_pos.x + angle.cos() * sample_radius;
                        let sample_y = new_pos.y + angle.sin() * sample_radius;
                        
                        if let Some(idx) = grid.world_to_grid(sample_x, sample_y) {
                            let food_strength = grid.food_trail[idx];
                            if food_strength > max_food_strength {
                                max_food_strength = food_strength;
                                best_direction = Vec2::new(angle.cos(), angle.sin());
                            }
                        }
                    }
                    
                    // If significant food pheromone found, orient toward it
                    if max_food_strength > config.detection_threshold {
                        let desired_angle = best_direction.y.atan2(best_direction.x);
                        velocity.x = desired_angle.cos() * 1.5;
                        velocity.y = desired_angle.sin() * 1.5;
                    }
                    
                    // Mark that this ant has chosen its exit direction
                    ant.has_exit_direction = true;
                }
            } else {
                // Reset exit direction flag when ant gets far from nest
                if distance_to_nest > 80.0 && !ant.carrying_food {
                    ant.has_exit_direction = false;
                }
            }
            
            // Bounce off world edges with 180° turn
            let half_world = config.world_size as f32 * 0.5;
            
            if transform.translation.x <= -half_world || transform.translation.x >= half_world {
                velocity.x = -velocity.x; // Reverse X direction
            }
            if transform.translation.y <= -half_world || transform.translation.y >= half_world {
                velocity.y = -velocity.y; // Reverse Y direction 
            }
            
            // Keep within bounds after bouncing
            transform.translation.x = transform.translation.x.clamp(-half_world, half_world);
            transform.translation.y = transform.translation.y.clamp(-half_world, half_world);
        }
    }
}

pub fn performance_analysis_system(
    ants: Query<&AntState>,
    mut performance_tracker: ResMut<PerformanceTracker>,
    mut exit_writer: EventWriter<bevy::app::AppExit>,
    time: Res<Time>,
) {
    let mut stuck_count = 0;
    let mut oscillating_count = 0;
    let mut lost_count = 0;
    let mut lost_food_carriers_count = 0;
    let runtime = time.elapsed_seconds();
    
    for ant in ants.iter() {
        // Count stuck ants (not moving for >3 seconds)
        if ant.stuck_timer > 3.0 {
            stuck_count += 1;
        }
        
        // Count oscillating ants (>5 direction changes and stuck)
        if ant.direction_changes > 5 && ant.stuck_timer > 1.0 {
            oscillating_count += 1;
        }
        
        // Count lost ants (haven't found food after grace period + 30 seconds)
        if !ant.has_found_food && ant.startup_timer <= 0.0 && runtime > 45.0 {
            lost_count += 1;
        }
        
        // Count lost food carriers (carrying food for >30 seconds without delivering)
        if ant.carrying_food && ant.food_carry_start_time > 0.0 && 
           runtime - ant.food_carry_start_time > 30.0 {
            lost_food_carriers_count += 1;
        }
    }
    
    performance_tracker.stuck_ants_count = stuck_count;
    performance_tracker.oscillating_ants_count = oscillating_count;
    performance_tracker.lost_ants_count = lost_count;
    performance_tracker.lost_food_carriers_count = lost_food_carriers_count;
    
    // Initialize simulation start time on first run
    if performance_tracker.simulation_start_time == 0.0 {
        performance_tracker.simulation_start_time = time.elapsed_seconds();
    }
    
    // AUTO-EXIT when oscillating ants hits 20 for optimization testing
    if oscillating_count >= 20 {
        println!("\n🚨 AUTO-EXIT: Too many oscillating ants ({}) - optimization needed!", oscillating_count);
        println!("📊 Final Stats:");
        println!("   Deliveries: {}", performance_tracker.successful_deliveries);
        println!("   Deliveries/min: {:.2}", performance_tracker.deliveries_per_minute);
        println!("   Avg delivery time: {:.1}s", performance_tracker.average_delivery_time);
        println!("   Avg return time: {:.1}s", performance_tracker.average_return_time);
        println!("   Stuck ants: {}", performance_tracker.stuck_ants_count);
        println!("   Lost ants: {}", performance_tracker.lost_ants_count);
        println!("   Lost food carriers: {}", performance_tracker.lost_food_carriers_count);
        println!("   Runtime: {:.1}s", time.elapsed_seconds());
        exit_writer.send(AppExit::Success);
    }
    
    // AUTO-EXIT when too many food carriers get lost 
    if lost_food_carriers_count >= 10 {
        println!("\n🚨 AUTO-EXIT: Too many lost food carriers ({}) - nest homing broken!", lost_food_carriers_count);
        println!("📊 Final Stats:");
        println!("   Deliveries: {}", performance_tracker.successful_deliveries);
        println!("   Deliveries/min: {:.2}", performance_tracker.deliveries_per_minute);
        println!("   Avg delivery time: {:.1}s", performance_tracker.average_delivery_time);
        println!("   Avg return time: {:.1}s", performance_tracker.average_return_time);
        println!("   Stuck ants: {}", performance_tracker.stuck_ants_count);
        println!("   Lost ants: {}", performance_tracker.lost_ants_count);
        println!("   Lost food carriers: {}", performance_tracker.lost_food_carriers_count);
        println!("   Runtime: {:.1}s", time.elapsed_seconds());
        exit_writer.send(AppExit::Success);
    }
    
    // Also auto-exit after 5 minutes if we achieve the goal
    if time.elapsed_seconds() > 300.0 && performance_tracker.deliveries_per_minute >= 10.0 {
        println!("\n🎉 SUCCESS: 5 minutes completed with {:.2} deliveries/min!", performance_tracker.deliveries_per_minute);
        exit_writer.send(AppExit::Success);
    }
}

pub fn pheromone_deposit_system(
    ants: Query<(&Transform, &AntState)>,
    mut pheromone_grid: Option<ResMut<PheromoneGrid>>,
    config: Res<SimConfig>,
) {
    if let Some(ref mut grid) = pheromone_grid {
        for (transform, ant) in ants.iter() {
            let pos = transform.translation;
            
            if ant.carrying_food {
                // Lay food trail when returning to nest - decreases with distance from food
                let decay_factor = (-ant.distance_from_food * 0.005).exp(); // Exponential decay
                let deposit_amount = config.lay_rate_food * config.food_quality_weight * decay_factor;
                
                grid.deposit(
                    pos.x, 
                    pos.y, 
                    PheromoneType::Food, 
                    deposit_amount
                );
                
            } else {
                // Lay nest trail when exploring - decreases with distance from nest
                let decay_factor = (-ant.distance_from_nest * 0.003).exp(); // Exponential decay
                let deposit_amount = config.lay_rate_nest * decay_factor;
                
                grid.deposit(
                    pos.x, 
                    pos.y, 
                    PheromoneType::Nest, 
                    deposit_amount
                );
            }
        }
    }
}

pub fn pheromone_update_system(
    mut pheromone_grid: Option<ResMut<PheromoneGrid>>,
    config: Res<SimConfig>,
) {
    if let Some(ref mut grid) = pheromone_grid {
        let evap_rates = (config.evap_food, config.evap_nest, config.evap_alarm);
        let diff_rates = (config.diff_food, config.diff_nest, config.diff_alarm);
        
        grid.update(evap_rates, diff_rates);
    }
}

pub fn food_collection_system(
    mut ants: Query<(&Transform, &mut AntState, &mut Velocity)>,
    mut food_sources: Query<(&Transform, &mut FoodSource)>,
    nests: Query<&Transform, (With<Nest>, Without<AntState>)>,
    mut performance_tracker: ResMut<PerformanceTracker>,
    time: Res<Time>,
) {
    // Find nest position first
    let nest_pos = if let Ok(nest_transform) = nests.get_single() {
        nest_transform.translation
    } else {
        Vec3::ZERO // Default to origin if no nest found
    };
    
    for (ant_transform, mut ant, mut velocity) in ants.iter_mut() {
        let ant_pos = ant_transform.translation;
        
        if !ant.carrying_food && ant.food_collection_timer <= 0.0 {
            // Look for food sources
            for (food_transform, food) in food_sources.iter() {
                let food_pos = food_transform.translation;
                let distance = ant_pos.distance(food_pos);
                
                if distance < 25.0 && food.amount > 0.0 {
                    // Start collecting food - stop moving
                    ant.food_collection_timer = 0.3; // OPTIMIZATION 4: Faster food collection
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                    break;
                }
            }
        } else if ant.food_collection_timer > 0.0 {
            // Currently collecting food - stay still
            ant.food_collection_timer -= 0.016; // Rough delta time
            velocity.x = 0.0;
            velocity.y = 0.0;
            
            // Check if collection is complete
            if ant.food_collection_timer <= 0.0 {
                // Look for nearby food to actually take
                for (food_transform, mut food) in food_sources.iter_mut() {
                    let food_pos = food_transform.translation;
                    let distance = ant_pos.distance(food_pos);
                    
                    if distance < 25.0 && food.amount > 0.0 {
                        let take_amount = 1.0; // Take exactly 1 bite
                        food.amount -= take_amount;
                        ant.carrying_food = true;
                        ant.food_pickup_time = time.elapsed_seconds(); // Track pickup time
                        ant.has_found_food = true; // Mark that this ant has found food
                        ant.food_carry_start_time = time.elapsed_seconds(); // Track when started carrying for return time
                        performance_tracker.total_food_collected += take_amount;
                        
                        // Orient toward nest immediately after picking up food
                        let direction = nest_pos - ant_pos;
                        let distance_to_nest = direction.length();
                        if distance_to_nest > 0.0 {
                            let normalized_direction = direction / distance_to_nest;
                            velocity.x = normalized_direction.x * 2.0; // OPTIMIZATION 1: Faster nest homing
                            velocity.y = normalized_direction.y * 2.0;
                        }
                        break;
                    }
                }
            }
        } else if ant.carrying_food {
            // Look for nest to drop off food
            let distance = ant_pos.distance(nest_pos);
            
            if distance < 40.0 { // Match nest visual radius
                // SUCCESSFUL DELIVERY - Track performance!
                ant.carrying_food = false;
                ant.delivery_attempts += 1;
                ant.successful_deliveries += 1;
                
                // Track delivery time (total cycle time)
                let delivery_time = time.elapsed_seconds() - ant.food_pickup_time;
                performance_tracker.delivery_times.push(delivery_time);
                
                // Track return time (just the carrying phase)
                let return_time = time.elapsed_seconds() - ant.food_carry_start_time;
                performance_tracker.return_times.push(return_time);
                
                performance_tracker.successful_deliveries += 1;
                performance_tracker.last_delivery_time = time.elapsed_seconds();
                
                // Update average delivery time
                let total_time: f32 = performance_tracker.delivery_times.iter().sum();
                performance_tracker.average_delivery_time = total_time / performance_tracker.delivery_times.len() as f32;
                
                // Update average return time
                let total_return_time: f32 = performance_tracker.return_times.iter().sum();
                performance_tracker.average_return_time = total_return_time / performance_tracker.return_times.len() as f32;
                
                // Calculate deliveries per minute
                let elapsed_minutes = time.elapsed_seconds() / 60.0;
                performance_tracker.deliveries_per_minute = performance_tracker.successful_deliveries as f32 / elapsed_minutes.max(0.1);
                
                // Give random direction after dropping off food
                let angle = rand::random::<f32>() * std::f32::consts::TAU;
                velocity.x = angle.cos() * 1.5;
                velocity.y = angle.sin() * 1.5;
            }
        }
    }
}

pub fn ant_visual_system(
    mut ants: Query<(&AntState, &mut Sprite), (With<AntState>, Without<PheromoneVisualization>)>,
    color_config: Res<ColorConfig>,
) {
    for (ant, mut sprite) in ants.iter_mut() {
        if ant.carrying_food {
            sprite.color = color_config.ant_carrying_food;
        } else if ant.food_collection_timer > 0.0 {
            sprite.color = color_config.ant_collecting;
        } else {
            sprite.color = color_config.ant_exploring;
        }
    }
}

pub fn food_visual_system(
    mut food_sources: Query<(Entity, &FoodSource, &mut Sprite, &Transform), (With<FoodSource>, Without<PheromoneVisualization>)>,
    mut commands: Commands,
    config: Res<SimConfig>,
    color_config: Res<ColorConfig>,
) {
    for (entity, food, mut sprite, _transform) in food_sources.iter_mut() {
        if food.amount > 0.0 {
            // Make food dimmer as it's consumed (30% to 100% brightness)
            let intensity = (food.amount / food.max_amount).clamp(0.3, 1.0);
            let base_color = color_config.food_source;
            sprite.color = Color::srgba(
                base_color.to_srgba().red,
                base_color.to_srgba().green * intensity,
                base_color.to_srgba().blue,
                base_color.to_srgba().alpha
            );
        } else {
            // Despawn depleted food and spawn new one
            commands.entity(entity).despawn();
            
            // Spawn replacement food at random location
            let range = config.world_size as f32 * 0.4;
            let mut x = (rand::random::<f32>() - 0.5) * range;
            let mut y = (rand::random::<f32>() - 0.5) * range;
            
            // Ensure minimum distance of 150 units from nest (0,0)
            let dist_from_nest = (x * x + y * y).sqrt();
            if dist_from_nest < 150.0 {
                let scale = 150.0 / dist_from_nest;
                x *= scale;
                y *= scale;
            }
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: color_config.food_source,
                        custom_size: Some(Vec2::new(30.0, 30.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, y, 2.0),
                    ..default()
                },
                FoodSource { amount: 100.0, max_amount: 100.0 },
            ));
        }
    }
}

pub fn exit_system(
    input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if input.just_pressed(KeyCode::Escape) {
        println!("Exit reason: User pressed ESC key");
        exit.send(AppExit::Success);
    }
}

pub fn exit_event_listener(
    mut exit_events: EventReader<AppExit>,
) {
    for exit_event in exit_events.read() {
        match exit_event {
            AppExit::Success => println!("Application exiting successfully"),
            AppExit::Error(code) => println!("Application exiting with error code: {}", code),
        }
    }
}

pub fn window_close_system(
    mut close_events: EventReader<WindowCloseRequested>,
    mut exit: EventWriter<AppExit>,
) {
    for _event in close_events.read() {
        println!("Exit reason: User closed window via X button");
        exit.send(AppExit::Success);
    }
}


pub fn restart_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    ants: Query<Entity, With<AntState>>,
    food_sources: Query<Entity, With<FoodSource>>,
    nests: Query<Entity, With<Nest>>,
    pheromone_vis: Query<Entity, With<PheromoneVisualization>>,
    config: Res<SimConfig>,
    mut pheromone_grid: Option<ResMut<PheromoneGrid>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        // Clear existing entities
        for entity in ants.iter() {
            commands.entity(entity).despawn();
        }
        for entity in food_sources.iter() {
            commands.entity(entity).despawn();
        }
        for entity in nests.iter() {
            commands.entity(entity).despawn();
        }
        for entity in pheromone_vis.iter() {
            commands.entity(entity).despawn();
        }
        
        // Reset pheromone grid
        if let Some(ref mut grid) = pheromone_grid {
            **grid = PheromoneGrid::new(1000, 1000);
        }
        
        // Respawn nest at center
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 1.0, 0.0), // Bright yellow for nest
                    custom_size: Some(Vec2::new(80.0, 80.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 5.0),
                ..default()
            },
            Nest { capacity: 10000.0 },
        ));
        
        // Respawn ants around nest
        for i in 0..config.initial_ants {
            let angle = (i as f32) * std::f32::consts::TAU / config.initial_ants as f32;
            let x = angle.cos() * 50.0;
            let y = angle.sin() * 50.0;
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(1.0, 0.0, 0.0), // Bright red for ants
                        custom_size: Some(Vec2::new(12.0, 12.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, y, 6.0),
                    ..default()
                },
                AntState {
                    carrying_food: false,
                    hunger: 0.0,
                    sensitivity_adapt: 1.0,
                    food_collection_timer: 0.0,
                    last_pheromone_strength: 0.0,
                    distance_from_food: 0.0,
                    distance_from_nest: 0.0,
                    has_exit_direction: false,
                    behavior_state: AntBehaviorState::Exploring,
                    sensing_timer: rand::random::<f32>() * 2.0,
                    current_direction: rand::random::<f32>() * std::f32::consts::TAU,
                    trail_strength: 0.0,
                    momentum_timer: 0.0,
                    last_position: Vec2::new(x, y),
                    stuck_timer: 0.0,
                    direction_changes: 0,
                    last_sensing_result: [0.0; 8],
                    trail_memory: [rand::random::<f32>() * std::f32::consts::TAU; 5],
                    memory_index: 0,
                    trail_quality: 0.0,
                    hysteresis_threshold: 0.0005, // Default detection threshold
                    consecutive_good_trail_time: 0.0,
                    food_pickup_time: 0.0,
                    delivery_attempts: 0,
                    successful_deliveries: 0,
                    startup_timer: 5.0, // OPTIMIZATION 4: Further reduced to 5s for even faster food seeking
                    has_found_food: false, // Track if ant has ever found food
                    food_carry_start_time: 0.0, // When ant picked up food
                },
                Velocity {
                    x: (rand::random::<f32>() * 2.0 - 1.0) * 1.5,
                    y: (rand::random::<f32>() * 2.0 - 1.0) * 1.5,
                },
            ));
        }
        
        // Respawn food sources
        for i in 0..config.food_sources {
            let (x, y) = if i < config.food_sources / 2 {
                // Close food sources (within 200 units of nest)
                let angle = rand::random::<f32>() * std::f32::consts::TAU;
                let distance = 80.0 + rand::random::<f32>() * 120.0; // 80-200 units away
                (angle.cos() * distance, angle.sin() * distance)
            } else {
                // Far food sources (scattered across world)
                let range = (config.world_size as f32) * 0.3; // Smaller range than before
                ((rand::random::<f32>() - 0.5) * range, (rand::random::<f32>() - 0.5) * range)
            };
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.0, 1.0, 0.0), // Bright green for food
                        custom_size: Some(Vec2::new(30.0, 30.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, y, 2.0),
                    ..default()
                },
                FoodSource { amount: 100.0, max_amount: 100.0 },
            ));
        }
        
        // Recreate pheromone visualization
        let grid_size = 200;
        let cell_size = 5.0;
        
        for x in 0..grid_size {
            for y in 0..grid_size {
                let world_x = (x as f32 - grid_size as f32 / 2.0) * cell_size;
                let world_y = (y as f32 - grid_size as f32 / 2.0) * cell_size;
                
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                            custom_size: Some(Vec2::new(cell_size, cell_size)),
                            ..default()
                        },
                        transform: Transform::from_xyz(world_x, world_y, -10.0),
                        ..default()
                    },
                    PheromoneVisualization {
                        grid_x: x,
                        grid_y: y,
                    },
                ));
            }
        }
    }
}

pub fn camera_control_system(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    input: Res<ButtonInput<KeyCode>>,
    _mouse_input: Res<ButtonInput<MouseButton>>,
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    _cursor_moved_events: EventReader<bevy::window::CursorMoved>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        let mut camera_move = Vec3::ZERO;
        let camera_speed = 200.0;
        
        // WASD movement
        if input.pressed(KeyCode::KeyW) {
            camera_move.y += camera_speed;
        }
        if input.pressed(KeyCode::KeyS) {
            camera_move.y -= camera_speed;
        }
        if input.pressed(KeyCode::KeyA) {
            camera_move.x -= camera_speed;
        }
        if input.pressed(KeyCode::KeyD) {
            camera_move.x += camera_speed;
        }
        
        camera_transform.translation += camera_move * 0.016; // rough delta time
        
        // Mouse wheel zoom
        for event in scroll_events.read() {
            let zoom_factor = if event.y > 0.0 { 0.9 } else { 1.1 };
            camera_transform.scale *= zoom_factor;
            camera_transform.scale = camera_transform.scale.clamp(Vec3::splat(0.1), Vec3::splat(5.0));
        }
    }
}

pub fn setup_pheromone_visualization(
    mut commands: Commands,
    _config: Res<SimConfig>,
) {
    let grid_size = 200; // Higher resolution for 1000x1000 world
    let cell_size = 5.0; // Each cell covers 5x5 world units
    
    for x in 0..grid_size {
        for y in 0..grid_size {
            let world_x = (x as f32 - grid_size as f32 / 2.0) * cell_size;
            let world_y = (y as f32 - grid_size as f32 / 2.0) * cell_size;
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Transparent initially
                        custom_size: Some(Vec2::new(cell_size, cell_size)),
                        ..default()
                    },
                    transform: Transform::from_xyz(world_x, world_y, -10.0), // Far behind everything
                    ..default()
                },
                PheromoneVisualization {
                    grid_x: x,
                    grid_y: y,
                },
            ));
        }
    }
}

pub fn update_pheromone_visualization(
    mut pheromone_sprites: Query<(&mut Sprite, &mut Transform), With<PheromoneVisualization>>,
    pheromone_grid: Option<Res<PheromoneGrid>>,
    color_config: Res<ColorConfig>,
) {
    if let Some(grid) = pheromone_grid {
        for (mut sprite, mut transform) in pheromone_sprites.iter_mut() {
            // Use the exact world position of the visualization sprite
            let world_x = transform.translation.x;
            let world_y = transform.translation.y;
            
            // Sample the pheromone grid directly at this position
            if let Some(idx) = grid.world_to_grid(world_x, world_y) {
                let food_strength = grid.food_trail[idx];
                let nest_strength = grid.nest_trail[idx];
                let max_strength = food_strength.max(nest_strength);
                
                // Debug: Disabled to reduce console spam during optimization tests
                // if idx % 50000 == 0 && max_strength > 5.0 {
                //     println!("Pheromone at {},{}: food={:.2}, nest={:.2}", world_x, world_y, food_strength, nest_strength);
                // }
                
                if max_strength > 0.01 {
                    if food_strength > nest_strength {
                        // Food trails - use more aggressive logarithmic scaling
                        let intensity = (food_strength.ln() / 3.0).clamp(0.0, 1.0);
                        let base_color = color_config.food_pheromone;
                        sprite.color = Color::srgba(
                            base_color.to_srgba().red,
                            base_color.to_srgba().green,
                            base_color.to_srgba().blue,
                            intensity
                        );
                        transform.translation.z = -9.0; // Food pheromones above nest pheromones
                    } else {
                        // Nest trails - use more aggressive logarithmic scaling
                        let intensity = (nest_strength.ln() / 3.0).clamp(0.0, 1.0);
                        let base_color = color_config.nest_pheromone;
                        sprite.color = Color::srgba(
                            base_color.to_srgba().red,
                            base_color.to_srgba().green,
                            base_color.to_srgba().blue,
                            intensity
                        );
                        transform.translation.z = -10.0; // Nest pheromones below food pheromones
                    }
                } else {
                    sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                    transform.translation.z = -10.0; // Default Z-level
                }
            } else {
                sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
            }
        }
    }
}

pub fn setup_debug_ui(mut commands: Commands, color_config: Res<ColorConfig>) {
    // Pheromone debug text in lower left
    commands.spawn((
        TextBundle::from_section(
            "Pheromone Info",
            TextStyle {
                font_size: 16.0,
                color: color_config.text,
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        PheromoneDebugText,
    ));

    // Entity debug text on right side
    commands.spawn((
        TextBundle::from_section(
            "Entity Info",
            TextStyle {
                font_size: 16.0,
                color: color_config.text,
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            right: Val::Px(10.0),
            max_width: Val::Px(300.0),
            ..default()
        }),
        EntityDebugText,
    ));

    // Performance metrics in top right
    commands.spawn((
        TextBundle::from_section(
            "Performance Metrics",
            TextStyle {
                font_size: 18.0,
                color: Color::srgb(0.0, 1.0, 0.0), // Bright green
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            max_width: Val::Px(400.0),
            ..default()
        }),
        PerformanceText,
    ));
}

pub fn cursor_tracking_system(
    mut debug_info: ResMut<DebugInfo>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let (Ok(window), Ok((camera, camera_transform))) = (window_query.get_single(), camera_query.get_single()) {
        if let Some(cursor_position) = window.cursor_position() {
            // Convert screen coordinates to world coordinates
            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                debug_info.cursor_world_pos = world_pos;
            }
        }
    }
}

pub fn hover_detection_system(
    mut debug_info: ResMut<DebugInfo>,
    pheromone_grid: Option<Res<PheromoneGrid>>,
    ant_query: Query<(Entity, &Transform, &AntState, &Velocity), With<AntState>>,
    nest_query: Query<(Entity, &Transform, &Nest), With<Nest>>,
    food_query: Query<(Entity, &Transform, &FoodSource), With<FoodSource>>,
) {
    let cursor_pos = debug_info.cursor_world_pos;
    
    // Update pheromone info
    if let Some(grid) = pheromone_grid.as_deref() {
        if let Some(idx) = grid.world_to_grid(cursor_pos.x, cursor_pos.y) {
            let food_strength = grid.food_trail[idx];
            let nest_strength = grid.nest_trail[idx];
            let alarm_strength = grid.alarm[idx];
            
            debug_info.pheromone_info = format!(
                "Pos: ({:.1}, {:.1})\nFood: {:.3}\nNest: {:.3}\nAlarm: {:.3}",
                cursor_pos.x, cursor_pos.y, food_strength, nest_strength, alarm_strength
            );
        }
    }
    
    // Reset hovered entity
    debug_info.hovered_entity = None;
    debug_info.entity_info = String::new();
    
    // Check for hovered ants
    for (entity, transform, ant_state, velocity) in ant_query.iter() {
        let distance = cursor_pos.distance(transform.translation.truncate());
        if distance < 15.0 { // Within 15 units of ant
            debug_info.hovered_entity = Some(entity);
            debug_info.entity_info = format!(
                "=== SUPER INTELLIGENT ANT ===\nEntity: {:?}\nPos: ({:.1}, {:.1})\nBehavior: {:?}\nCarrying Food: {}\n\n--- INTELLIGENCE SYSTEMS ---\nTrail Quality: {:.3}\nHysteresis Threshold: {:.4}\nConsecutive Good Trail: {:.2}s\nMemory Direction: {:.1}°\n\n--- SENSING DATA ---\nLast Pheromone: {:.3}\nTrail Strength: {:.3}\nSensing Timer: {:.2}\nMomentum Timer: {:.2}\nDirection: {:.2}°\nStuck Timer: {:.2}s\nDir Changes: {}\nDistance Moved: {:.1}\n\n--- 8-DIRECTION SCAN ---\n  N:{:.3} NE:{:.3} E:{:.3} SE:{:.3}\n  S:{:.3} SW:{:.3} W:{:.3} NW:{:.3}\n\n--- LEGACY DATA ---\nHunger: {:.3}\nSensitivity: {:.3}\nCollection Timer: {:.3}\nDistance from Food: {:.1}\nDistance from Nest: {:.1}\nHas Exit Direction: {}\nVelocity: ({:.2}, {:.2})",
                entity,
                transform.translation.x, transform.translation.y,
                ant_state.behavior_state,
                ant_state.carrying_food,
                ant_state.trail_quality,
                ant_state.hysteresis_threshold,
                ant_state.consecutive_good_trail_time,
                calculate_memory_direction(&ant_state.trail_memory).to_degrees(),
                ant_state.last_pheromone_strength,
                ant_state.trail_strength,
                ant_state.sensing_timer,
                ant_state.momentum_timer,
                ant_state.current_direction.to_degrees(),
                ant_state.stuck_timer,
                ant_state.direction_changes,
                transform.translation.truncate().distance(ant_state.last_position),
                ant_state.last_sensing_result[0], ant_state.last_sensing_result[1], ant_state.last_sensing_result[2], ant_state.last_sensing_result[3],
                ant_state.last_sensing_result[4], ant_state.last_sensing_result[5], ant_state.last_sensing_result[6], ant_state.last_sensing_result[7],
                ant_state.hunger,
                ant_state.sensitivity_adapt,
                ant_state.food_collection_timer,
                ant_state.distance_from_food,
                ant_state.distance_from_nest,
                ant_state.has_exit_direction,
                velocity.x, velocity.y
            );
            break;
        }
    }
    
    // Check for hovered nest (if no ant was hovered)
    if debug_info.hovered_entity.is_none() {
        for (entity, transform, nest) in nest_query.iter() {
            let distance = cursor_pos.distance(transform.translation.truncate());
            if distance < 50.0 { // Within 50 units of nest center
                debug_info.hovered_entity = Some(entity);
                debug_info.entity_info = format!(
                    "=== NEST ===\nEntity: {:?}\nPos: ({:.1}, {:.1})\nCapacity: {:.1}",
                    entity,
                    transform.translation.x, transform.translation.y,
                    nest.capacity
                );
                break;
            }
        }
    }
    
    // Check for hovered food (if no ant or nest was hovered)
    if debug_info.hovered_entity.is_none() {
        for (entity, transform, food) in food_query.iter() {
            let distance = cursor_pos.distance(transform.translation.truncate());
            if distance < 20.0 { // Within 20 units of food
                debug_info.hovered_entity = Some(entity);
                debug_info.entity_info = format!(
                    "=== FOOD ===\nEntity: {:?}\nPos: ({:.1}, {:.1})\nAmount: {:.1}\nMax Amount: {:.1}\nRemaining: {:.1}%",
                    entity,
                    transform.translation.x, transform.translation.y,
                    food.amount,
                    food.max_amount,
                    (food.amount / food.max_amount) * 100.0
                );
                break;
            }
        }
    }
}

pub fn update_debug_ui(
    debug_info: Res<DebugInfo>,
    performance_tracker: Res<PerformanceTracker>,
    mut pheromone_text_query: Query<&mut Text, (With<PheromoneDebugText>, Without<EntityDebugText>, Without<PerformanceText>)>,
    mut entity_text_query: Query<&mut Text, (With<EntityDebugText>, Without<PheromoneDebugText>, Without<PerformanceText>)>,
    mut performance_text_query: Query<&mut Text, (With<PerformanceText>, Without<PheromoneDebugText>, Without<EntityDebugText>)>,
) {
    // Update pheromone debug text
    if let Ok(mut text) = pheromone_text_query.get_single_mut() {
        text.sections[0].value = debug_info.pheromone_info.clone();
    }
    
    // Update entity debug text
    if let Ok(mut text) = entity_text_query.get_single_mut() {
        text.sections[0].value = debug_info.entity_info.clone();
    }
    
    // Update performance metrics
    if let Ok(mut text) = performance_text_query.get_single_mut() {
        text.sections[0].value = format!(
            "🎯 PERFORMANCE METRICS 🎯\n\n✅ Successful Deliveries: {}\n❌ Failed Attempts: {}\n📦 Total Food Collected: {:.1}\n⏱️ Avg Delivery Time: {:.1}s\n🏠 Avg Return Time: {:.1}s\n📈 Deliveries/Min: {:.2}\n\n🚫 Stuck Ants: {}\n🔄 Oscillating Ants: {}\n🔍 Lost Ants: {}\n🍯 Lost Food Carriers: {}\n\n🔧 OPTIMIZATION TARGET:\nMaximize: Deliveries/Min\nMinimize: Return Time + Lost Carriers",
            performance_tracker.successful_deliveries,
            performance_tracker.failed_attempts,
            performance_tracker.total_food_collected,
            performance_tracker.average_delivery_time,
            performance_tracker.average_return_time,
            performance_tracker.deliveries_per_minute,
            performance_tracker.stuck_ants_count,
            performance_tracker.oscillating_ants_count,
            performance_tracker.lost_ants_count,
            performance_tracker.lost_food_carriers_count,
        );
    }
}

pub fn ant_selection_system(
    mut debug_info: ResMut<DebugInfo>,
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    ant_query: Query<Entity, With<AntState>>,
    selected_query: Query<Entity, With<SelectedAnt>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let _cursor_pos = debug_info.cursor_world_pos;
        let mut ant_clicked = false;
        
        // Remove selection from all ants
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<SelectedAnt>();
        }
        
        // Check if we clicked on an ant
        for entity in ant_query.iter() {
            // Use the hovered entity if available (we're clicking on what we're hovering)
            if debug_info.hovered_entity == Some(entity) {
                commands.entity(entity).insert(SelectedAnt);
                debug_info.selected_entity = Some(entity);
                ant_clicked = true;
                break;
            }
        }
        
        // If we didn't click on an ant, deselect
        if !ant_clicked {
            debug_info.selected_entity = None;
        }
    }
}

pub fn selected_ant_display_system(
    mut debug_info: ResMut<DebugInfo>,
    ant_query: Query<(Entity, &Transform, &AntState, &Velocity), With<AntState>>,
) {
    if let Some(selected_entity) = debug_info.selected_entity {
        if let Ok((entity, transform, ant_state, velocity)) = ant_query.get(selected_entity) {
            // Only update if we're not showing hover info (hover takes priority)
            if debug_info.entity_info.is_empty() {
                debug_info.entity_info = format!(
                    "=== SUPER INTELLIGENT ANT ===\nEntity: {:?}\nPos: ({:.1}, {:.1})\nBehavior: {:?}\nCarrying Food: {}\n\n--- INTELLIGENCE SYSTEMS ---\nTrail Quality: {:.3}\nHysteresis Threshold: {:.4}\nConsecutive Good Trail: {:.2}s\nMemory Direction: {:.1}°\n\n--- SENSING DATA ---\nLast Pheromone: {:.3}\nTrail Strength: {:.3}\nSensing Timer: {:.2}\nMomentum Timer: {:.2}\nDirection: {:.2}°\nStuck Timer: {:.2}s\nDir Changes: {}\nDistance Moved: {:.1}\n\n--- 8-DIRECTION SCAN ---\n  N:{:.3} NE:{:.3} E:{:.3} SE:{:.3}\n  S:{:.3} SW:{:.3} W:{:.3} NW:{:.3}\n\n--- LEGACY DATA ---\nHunger: {:.3}\nSensitivity: {:.3}\nCollection Timer: {:.3}\nDistance from Food: {:.1}\nDistance from Nest: {:.1}\nHas Exit Direction: {}\nVelocity: ({:.2}, {:.2})",
                    entity,
                    transform.translation.x, transform.translation.y,
                    ant_state.behavior_state,
                    ant_state.carrying_food,
                    ant_state.trail_quality,
                    ant_state.hysteresis_threshold,
                    ant_state.consecutive_good_trail_time,
                    calculate_memory_direction(&ant_state.trail_memory).to_degrees(),
                    ant_state.last_pheromone_strength,
                    ant_state.trail_strength,
                    ant_state.sensing_timer,
                    ant_state.momentum_timer,
                    ant_state.current_direction.to_degrees(),
                    ant_state.stuck_timer,
                    ant_state.direction_changes,
                    transform.translation.truncate().distance(ant_state.last_position),
                    ant_state.last_sensing_result[0], ant_state.last_sensing_result[1], ant_state.last_sensing_result[2], ant_state.last_sensing_result[3],
                    ant_state.last_sensing_result[4], ant_state.last_sensing_result[5], ant_state.last_sensing_result[6], ant_state.last_sensing_result[7],
                    ant_state.hunger,
                    ant_state.sensitivity_adapt,
                    ant_state.food_collection_timer,
                    ant_state.distance_from_food,
                    ant_state.distance_from_nest,
                    ant_state.has_exit_direction,
                    velocity.x, velocity.y
                );
            }
        }
    }
}

pub fn selected_ant_outline_system(
    mut commands: Commands,
    selected_ants: Query<(Entity, &Transform), (With<AntState>, With<SelectedAnt>)>,
    existing_outlines: Query<Entity, With<crate::components::AntOutline>>,
    color_config: Res<ColorConfig>,
) {
    // Remove existing outlines
    for outline_entity in existing_outlines.iter() {
        commands.entity(outline_entity).despawn();
    }
    
    // Add outlines for selected ants
    for (_ant_entity, transform) in selected_ants.iter() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: color_config.debug_selection,
                    custom_size: Some(Vec2::new(14.0, 14.0)), // Slightly larger than ant (12x12)
                    ..default()
                },
                transform: Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z - 0.1 // Just behind the ant
                ),
                ..default()
            },
            crate::components::AntOutline,
        ));
    }
}