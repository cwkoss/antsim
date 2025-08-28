use bevy::prelude::*;
use bevy::window::{WindowCloseRequested, PrimaryWindow};
use rand::{Rng, random};
use crate::components::*;
use crate::config::*;
use crate::pheromones::*;
use crate::colors::*;

/// Movement behavior types for unified speed management
#[derive(Debug, Clone, Copy)]
pub enum MovementType {
    /// Ant carrying food returning to nest
    CarryingFood,
    /// Ant following pheromone trail to food
    FollowingTrail,
    /// Ant exploring randomly for food
    Exploring,
    /// Ant recovering from stuck state
    StuckRecovery,
    /// Legacy movement (to be phased out)
    Legacy,
}

/// Unified function to set ant velocity based on movement type and direction
fn set_ant_velocity(velocity: &mut Velocity, direction: f32, movement_type: MovementType) {
    let speed = match movement_type {
        MovementType::CarryingFood => 60.0,    // Steady speed when returning to nest
        MovementType::FollowingTrail => 65.0,  // Slightly faster when following pheromone trails
        MovementType::Exploring => 50.0,       // Slower when randomly exploring
        MovementType::StuckRecovery => 60.0,   // Moderate speed when recovering from stuck
        MovementType::Legacy => 85.0,          // Old speed for remaining legacy code
    };
    
    velocity.x = direction.cos() * speed;
    velocity.y = direction.sin() * speed;
}

/// Unified function to set ant velocity using a direction vector
fn set_ant_velocity_from_vector(velocity: &mut Velocity, direction_vec: Vec2, movement_type: MovementType) {
    let speed = match movement_type {
        MovementType::CarryingFood => 60.0,
        MovementType::FollowingTrail => 65.0,
        MovementType::Exploring => 50.0,
        MovementType::StuckRecovery => 60.0,
        MovementType::Legacy => 90.0,  // Legacy vector-based movement
    };
    
    if direction_vec.length() > 0.0 {
        let normalized = direction_vec.normalize();
        velocity.x = normalized.x * speed;
        velocity.y = normalized.y * speed;
    }
}

pub fn sensing_system(
    mut ants: Query<(Entity, &Transform, &mut AntState, &mut Velocity, Option<&DebugAnt>)>,
    rocks: Query<(&Transform, &Rock), Without<AntState>>,
    mut pheromone_grid: Option<ResMut<PheromoneGrid>>,
    config: Res<SimConfig>,
    time: Res<Time>,
) {
    if let Some(mut grid) = pheromone_grid {
        // CYCLE 17: Pre-collect all ant positions and success data for formation flying
        let ant_positions: Vec<(Entity, Vec2, bool, u32)> = ants.iter()
            .map(|(entity, transform, ant, _, _)| {
                (entity, transform.translation.truncate(), ant.carrying_food, ant.successful_deliveries)
            })
            .collect();
        
        for (entity, transform, mut ant, mut velocity, debug_ant) in ants.iter_mut() {
            let pos = transform.translation;
            let delta_time = time.delta_seconds();
            
            // Update timers
            ant.sensing_timer -= delta_time;
            ant.startup_timer -= delta_time;
            
            // Update diagnostic timers
            ant.time_since_progress += delta_time;
            ant.trail_following_time += delta_time;
            
            // Calculate world edge proximity for edge-wandering detection
            let world_half_size = 500.0; // Assuming 1000x1000 world
            let x_edge_dist = world_half_size - pos.x.abs();
            let y_edge_dist = world_half_size - pos.y.abs();
            ant.world_edge_proximity = x_edge_dist.min(y_edge_dist);
            ant.is_edge_wanderer = ant.world_edge_proximity < 50.0 && ant.time_since_progress > 10.0;
            
            // Don't process ants that are collecting food or still in startup
            if ant.food_collection_timer > 0.0 || ant.startup_timer > 0.0 {
                continue;
            }
            
            // For carrying food: use nest pheromone following with smart obstacle avoidance
            if ant.carrying_food {
                ant.sensing_timer = 0.2; // CYCLE 14: Ultra-fast sensing for food-carrying ants
                
                // SIMPLIFIED NEST PHEROMONE FOLLOWING: Focus on stronger detection and faster following
                if ant.sensing_timer <= 0.0 {
                    let mut max_nest_pheromone = 0.0;
                    let mut best_pheromone_direction = ant.current_direction;
                    let mut found_nest_trail = false;
                    
                    // Enhanced 12-direction sampling with better range
                    for i in 0..12 {
                        let angle = (i as f32) * std::f32::consts::TAU / 12.0;
                        let sample_x = pos.x + angle.cos() * 20.0; // Increased range
                        let sample_y = pos.y + angle.sin() * 20.0;
                        
                        let nest_strength = grid.sample_directional(sample_x, sample_y, angle, 8.0, PheromoneType::Nest);
                        
                        // Lower threshold and momentum bonus for better trail detection
                        if nest_strength > 0.05 { // Much lower threshold
                            // Momentum bonus: prefer directions closer to current heading
                            let angle_diff = (angle - ant.current_direction).abs();
                            let angle_diff_normalized = if angle_diff > std::f32::consts::PI {
                                std::f32::consts::TAU - angle_diff
                            } else {
                                angle_diff
                            };
                            let momentum_bonus = (1.0 - angle_diff_normalized / std::f32::consts::PI) * 0.2;
                            
                            let effective_strength = nest_strength + momentum_bonus;
                            
                            if effective_strength > max_nest_pheromone {
                                max_nest_pheromone = effective_strength;
                                best_pheromone_direction = angle;
                                found_nest_trail = true;
                            }
                        }
                    }
                    
                    // If we found a good nest trail, follow it (with rock avoidance and loop detection)
                    if found_nest_trail {
                        // CYCLE 19: Loop detection - if following trails but not making progress, occasionally break away
                        let should_break_from_trail = ant.time_since_progress > 12.0 && 
                                                     ant.behavior_state == AntBehaviorState::Following &&
                                                     (time.elapsed_seconds() * ant.successful_deliveries as f32 + 1.0).sin() > 0.7; // Occasional break-away
                        
                        if !should_break_from_trail {
                            // Check if the pheromone direction is safe from rocks
                            let test_pos = Vec2::new(pos.x, pos.y) + Vec2::new(best_pheromone_direction.cos(), best_pheromone_direction.sin()) * 40.0;
                            let mut pheromone_path_safe = true;
                            
                            for (rock_transform, rock) in rocks.iter() {
                                let rock_pos = Vec2::new(rock_transform.translation.x, rock_transform.translation.y);
                                let distance_to_rock = test_pos.distance(rock_pos);
                                
                                if distance_to_rock < rock.radius + 30.0 {
                                    pheromone_path_safe = false;
                                    break;
                                }
                            }
                            
                            if pheromone_path_safe {
                                // SIMPLIFIED: Smooth but decisive nest trail following
                                let direction_change = best_pheromone_direction - ant.current_direction;
                                let smooth_direction_change = if direction_change.abs() > std::f32::consts::PI {
                                    if direction_change > 0.0 { direction_change - std::f32::consts::TAU } else { direction_change + std::f32::consts::TAU }
                                } else { direction_change };
                                
                                // More aggressive turning for nest trails - we want to get home quickly
                                ant.current_direction += smooth_direction_change * 0.4;
                                
                                set_ant_velocity(&mut velocity, ant.current_direction, MovementType::FollowingTrail);
                                ant.behavior_state = AntBehaviorState::Following;
                                
                                // Faster sensing for nest trails - frequent course corrections
                                ant.sensing_timer = 0.1;
                                continue; // Skip the pathfinding logic below
                            } else {
                                // ENHANCED NEST-SEEKING: No safe pheromone trail found, use intelligent nest-seeking
                                let distance_to_nest = Vec2::new(pos.x, pos.y).length();
                                
                                if distance_to_nest < 100.0 {
                                    // CLOSE TO NEST: Direct approach with obstacle avoidance
                                    let to_nest = (Vec2::ZERO - Vec2::new(pos.x, pos.y)).normalize();
                                    let direct_nest_angle = to_nest.y.atan2(to_nest.x);
                                    
                                    // Check if direct path to nest is safe
                                    let mut direct_path_safe = true;
                                    let test_distance = distance_to_nest.min(40.0);
                                    let test_pos = Vec2::new(pos.x, pos.y) + to_nest * test_distance;
                                    
                                    for (rock_transform, rock) in rocks.iter() {
                                        let rock_pos = Vec2::new(rock_transform.translation.x, rock_transform.translation.y);
                                        if test_pos.distance(rock_pos) < rock.radius + 25.0 {
                                            direct_path_safe = false;
                                            break;
                                        }
                                    }
                                    
                                    if direct_path_safe {
                                        // Direct path to nest is safe - go straight home!
                                        ant.current_direction = direct_nest_angle;
                                        set_ant_velocity(&mut velocity, direct_nest_angle, MovementType::CarryingFood);
                                        ant.behavior_state = AntBehaviorState::Exploring;
                                        ant.sensing_timer = 0.05; // Very frequent sensing near nest
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    
                    // ADAPTIVE SENSING: Adjust sensing frequency based on distance to nest
                    let distance_to_nest = Vec2::new(pos.x, pos.y).length();
                    ant.sensing_timer = if distance_to_nest < 150.0 {
                        0.08 // Very frequent sensing when close to nest
                    } else if distance_to_nest < 300.0 {
                        0.12 // Frequent sensing at medium distance
                    } else {
                        0.15 // Standard sensing when far from nest
                    };
                }
                
                // Emergency behaviors for stuck ants
                if pos.x.abs() > 470.0 || pos.y.abs() > 470.0 {
                    // Force move toward world center
                    let to_center = Vec2::ZERO - Vec2::new(pos.x, pos.y);
                    let escape_direction = to_center.normalize();
                    ant.current_direction = escape_direction.y.atan2(escape_direction.x);
                    set_ant_velocity_from_vector(&mut velocity, escape_direction, MovementType::CarryingFood);
                    ant.behavior_state = AntBehaviorState::Exploring;
                } else {
                    // Check if ant is stuck on/near a rock
                    let mut on_rock = false;
                    let mut nearest_rock_direction = 0.0f32;
                    let mut min_distance = f32::INFINITY;
                    
                    for (rock_transform, rock) in rocks.iter() {
                        let rock_pos = Vec2::new(rock_transform.translation.x, rock_transform.translation.y);
                        let distance = Vec2::new(pos.x, pos.y).distance(rock_pos);
                        
                        if distance < rock.radius + 25.0 { // Expanded sensing range
                            on_rock = true;
                            if distance < min_distance {
                                min_distance = distance;
                                // Direction AWAY from rock
                                let away_from_rock = Vec2::new(pos.x, pos.y) - rock_pos;
                                nearest_rock_direction = away_from_rock.normalize().y.atan2(away_from_rock.normalize().x);
                            }
                        }
                    }
                    
                    if on_rock && (ant.stuck_timer > 0.6 || min_distance < 35.0) { // CYCLE 13: Even faster reaction
                        // CYCLE 15: Cooperative rock mapping - deposit warning pheromones
                        let grid_pos = Vec2::new(pos.x, pos.y);
                        if let Some(grid_idx) = grid.world_to_grid(grid_pos.x, grid_pos.y) {
                            grid.alarm[grid_idx] += 2.0; // Strong warning signal for rock proximity
                        }
                        
                        // CYCLE 9: Smart rock avoidance - curve toward nest while avoiding rock
                        let to_nest = (Vec2::ZERO - Vec2::new(pos.x, pos.y)).normalize();
                        let away_from_rock = Vec2::new(nearest_rock_direction.cos(), nearest_rock_direction.sin());
                        
                        // Blend away-from-rock with toward-nest for intelligent avoidance
                        let smart_direction = (away_from_rock * 0.6 + to_nest * 0.4).normalize();
                        ant.current_direction = smart_direction.y.atan2(smart_direction.x);
                        set_ant_velocity(&mut velocity, ant.current_direction, MovementType::CarryingFood);
                        ant.behavior_state = AntBehaviorState::Exploring;
                        ant.sensing_timer = 0.1; // Quick re-sense
                    } else if ant.sensing_timer <= 0.0 {
                        let mut best_direction = ant.current_direction;
                        
                        // CYCLE 17: Find nearby successful leaders from pre-collected data
                        let current_pos = Vec2::new(pos.x, pos.y);
                        let nearby_leaders: Vec<(Vec2, u32)> = ant_positions.iter()
                            .filter_map(|(other_entity, other_pos, carrying_food, successful_deliveries)| {
                                if *other_entity == entity || !carrying_food || *successful_deliveries == 0 {
                                    None
                                } else {
                                    let distance = current_pos.distance(*other_pos);
                                    if distance < 30.0 {
                                        Some((*other_pos, *successful_deliveries))
                                    } else {
                                        None
                                    }
                                }
                            })
                            .collect();
                        
                        // ENHANCED NEST-SEEKING: Intelligent nest-oriented pathfinding
                        let nest_pos = Vec2::ZERO;
                        let to_nest = (nest_pos - Vec2::new(pos.x, pos.y)).normalize();
                        let ideal_direction = to_nest.y.atan2(to_nest.x);
                        let distance_to_nest = Vec2::new(pos.x, pos.y).length();
                        
                        // ENHANCED PATHFINDING: Distance-aware nest-seeking with improved scoring
                        let mut found_safe_path = false;
                        let mut best_score = f32::NEG_INFINITY;
                        
                        // Adaptive direction testing based on distance to nest
                        let num_directions = if distance_to_nest < 200.0 { 16 } else { 12 };
                        let max_deviation = if distance_to_nest < 150.0 {
                            std::f32::consts::PI / 3.0 // Wider search when close to nest
                        } else {
                            std::f32::consts::PI / 6.0 // Narrower focus when far
                        };
                        
                        for i in 0..num_directions {
                            let deviation = (i as f32 - (num_directions - 1) as f32 / 2.0) * max_deviation / (num_directions as f32 / 2.0);
                            let test_angle = ideal_direction + deviation;
                            
                            // Adaptive lookahead: shorter when close to nest
                            let test_distance = if distance_to_nest < 100.0 {
                                20.0 // Short lookahead near nest
                            } else {
                                40.0 // Normal lookahead far from nest
                            };
                            
                            let test_pos = Vec2::new(
                                pos.x + test_angle.cos() * test_distance,
                                pos.y + test_angle.sin() * test_distance
                            );
                            
                            let mut path_score = 0.0;
                            let mut is_safe = true;
                            
                            // Check world boundaries
                            if test_pos.x.abs() > 475.0 || test_pos.y.abs() > 475.0 {
                                is_safe = false;
                            }
                            
                            if is_safe {
                                // ENHANCED SCORING: Distance-aware nest-seeking optimization
                                let path_direction = Vec2::new(test_angle.cos(), test_angle.sin());
                                let nest_alignment = to_nest.dot(path_direction);
                                
                                // Distance-based nest alignment scoring
                                let alignment_multiplier = if distance_to_nest < 150.0 {
                                    150.0 // Very strong nest bias when close
                                } else if distance_to_nest < 300.0 {
                                    120.0 // Strong nest bias at medium distance
                                } else {
                                    100.0 // Standard nest bias when far
                                };
                                path_score += nest_alignment * alignment_multiplier;
                                
                                // Adaptive momentum bonus
                                let current_direction_vec = Vec2::new(ant.current_direction.cos(), ant.current_direction.sin());
                                let momentum_alignment = current_direction_vec.dot(path_direction);
                                let momentum_bonus = if distance_to_nest < 100.0 {
                                    15.0 // Reduced momentum near nest for better maneuvering
                                } else {
                                    25.0 // Standard momentum far from nest
                                };
                                path_score += momentum_alignment * momentum_bonus;
                                
                                // Progress bonus: reward paths that make clear progress toward nest
                                let progress_bonus = if distance_to_nest > 200.0 {
                                    let future_nest_distance = test_pos.length();
                                    let distance_improvement = distance_to_nest - future_nest_distance;
                                    distance_improvement * 2.0 // Bonus for making progress toward nest
                                } else {
                                    0.0 // Don't worry about progress when close
                                };
                                path_score += progress_bonus;
                                
                                // CYCLE 15: Cooperative rock avoidance using alarm pheromones
                                if let Some(grid_idx) = grid.world_to_grid(test_pos.x, test_pos.y) {
                                    let alarm_strength = grid.alarm[grid_idx];
                                    path_score -= alarm_strength * 40.0; // Heavy penalty for alarm areas
                                }
                                
                                // Enhanced rock avoidance scoring
                                let mut min_rock_clearance = f32::INFINITY;
                                for (rock_transform, rock) in rocks.iter() {
                                    let rock_pos = Vec2::new(rock_transform.translation.x, rock_transform.translation.y);
                                    let distance_to_rock = test_pos.distance(rock_pos);
                                    let safety_buffer = rock.radius + 35.0; // Larger safety margin
                                    
                                    if distance_to_rock < safety_buffer {
                                        is_safe = false;
                                        break;
                                    } else {
                                        min_rock_clearance = min_rock_clearance.min(distance_to_rock - safety_buffer);
                                    }
                                }
                                
                                if is_safe {
                                    // Reward good rock clearance (exponential benefit for safer paths)
                                    path_score += (min_rock_clearance.min(60.0) * 1.5).powi(2) / 100.0;
                                    
                                    // CYCLE 17: Adaptive formation flying using pre-collected leader data
                                    for (leader_pos, leader_deliveries) in &nearby_leaders {
                                        let to_leader = (*leader_pos - test_pos).normalize_or_zero();
                                        let path_to_leader_alignment = path_direction.dot(to_leader);
                                        
                                        // Bonus for following in the direction of successful ants (convoy effect)
                                        let leadership_bonus = path_to_leader_alignment * (*leader_deliveries as f32).min(3.0) * 8.0;
                                        path_score += leadership_bonus;
                                    }
                                    
                                    if path_score > best_score {
                                        best_score = path_score;
                                        best_direction = test_angle;
                                        found_safe_path = true;
                                    }
                                }
                            }
                        }
                        
                        if !found_safe_path {
                            // Emergency: just try to move away from current position
                            best_direction = ant.current_direction + 1.57; // Turn 90 degrees
                        }
                        
                        ant.current_direction = best_direction;
                        set_ant_velocity(&mut velocity, best_direction, MovementType::CarryingFood);
                        
                        // CYCLE 14: Keep ultra-fast sensing frequency for responsiveness
                        ant.sensing_timer = 0.2;
                        
                        ant.behavior_state = if found_safe_path { AntBehaviorState::Following } else { AntBehaviorState::Exploring };
                    }
                }
            } else {
                // For exploring ants: follow FOOD pheromones (trails left by successful ants who found food)
                
                // ANTI-CLUSTERING: Near-nest exclusion zone - use simple radial exploration instead of pheromone following
                let distance_from_nest = Vec2::new(pos.x, pos.y).length();
                if distance_from_nest < 100.0 {
                    // Near nest: use radial exploration to get away from crowded nest area
                    let outward_direction = Vec2::new(pos.x, pos.y).normalize();
                    ant.current_direction = outward_direction.y.atan2(outward_direction.x);
                    set_ant_velocity(&mut velocity, ant.current_direction, MovementType::Exploring);
                    ant.sensing_timer = 1.5; // Long commitment to outward movement
                    ant.behavior_state = AntBehaviorState::Exploring;
                    continue;
                }
                
                let pheromone_readings = grid.sample_all_directions(pos.x, pos.y, PheromoneType::Food);
                let mut best_direction = ant.current_direction;
                let mut max_pheromone = 0.0;
                let mut found_trail = false;
                
                // CYCLE 22: Collective swarm intelligence integration
                let swarm_context = analyze_local_swarm_intelligence(pos.x, pos.y, &ant, entity, &ant_positions, time.elapsed_seconds());
                
                // DIAGNOSTIC ANALYSIS: Update ant-centric state tracking
                let current_pheromone = pheromone_readings[0]; // Center position
                ant.can_see_trail = current_pheromone > config.detection_threshold;
                
                // Calculate distance to nearest significant pheromone concentration
                let mut min_trail_distance = f32::INFINITY;
                for i in 1..9 { // Skip center (index 0)
                    let sample_distance = 15.0; // Distance for directional sampling
                    let angle = ((i - 1) as f32) * std::f32::consts::TAU / 8.0;
                    let sample_x = pos.x + angle.cos() * sample_distance;
                    let sample_y = pos.y + angle.sin() * sample_distance;
                    let sample_strength = grid.sample_directional(sample_x, sample_y, angle, 5.0, PheromoneType::Food);
                    
                    if sample_strength > config.detection_threshold {
                        min_trail_distance = min_trail_distance.min(sample_distance);
                    }
                }
                ant.distance_from_trail = min_trail_distance;
                
                // Update trail contact timing
                if ant.can_see_trail {
                    ant.last_trail_contact_time = time.elapsed_seconds();
                    ant.trail_following_time = 0.0; // Reset - starting new trail section
                } 
                
                // Calculate pheromone gradient strength for behavior analysis
                let max_reading = pheromone_readings.iter().skip(1).copied().fold(0.0f32, f32::max);
                let min_reading = pheromone_readings.iter().skip(1).copied().fold(f32::INFINITY, f32::min);
                ant.trail_gradient_strength = max_reading - min_reading.min(max_reading);
                
                // Advanced gradient analysis with predictive lookahead
                
                for (i, &pheromone_strength) in pheromone_readings.iter().enumerate() {
                    if pheromone_strength > 0.15 {
                        let angle = (i as f32) * std::f32::consts::TAU / 8.0;
                        
                        // Calculate momentum bonus for maintaining direction
                        let angle_diff = (angle - ant.current_direction).abs();
                        let angle_diff_normalized = if angle_diff > std::f32::consts::PI { 
                            std::f32::consts::TAU - angle_diff 
                        } else { 
                            angle_diff 
                        };
                        let momentum_bonus = (1.0 - angle_diff_normalized / std::f32::consts::PI) * 1.2; // Increased momentum for better trail commitment
                        
                        // Additional persistence bonus if ant has been following trails successfully
                        let persistence_bonus = if ant.behavior_state == AntBehaviorState::Following {
                            0.25 // Enhanced commitment to successful trail following
                        } else {
                            0.0
                        };
                        
                        // TRAIL DIRECTION DETECTION: Compare pheromone strength ahead vs behind to determine trail direction
                        let search_distance = 20.0;
                        let ahead_x = pos.x + angle.cos() * search_distance;
                        let ahead_y = pos.y + angle.sin() * search_distance;
                        let ahead_pheromone = grid.sample_directional(ahead_x, ahead_y, angle, 5.0, PheromoneType::Food);
                        
                        let behind_x = pos.x - angle.cos() * search_distance;
                        let behind_y = pos.y - angle.sin() * search_distance;
                        let behind_pheromone = grid.sample_directional(behind_x, behind_y, angle, 5.0, PheromoneType::Food);
                        
                        // Trail direction bonus: stronger reward for following toward stronger pheromone (toward food)
                        let trail_direction_bonus = if ahead_pheromone > behind_pheromone + 0.05 {
                            0.8 // Strong bonus for following trail toward food
                        } else if behind_pheromone > ahead_pheromone + 0.05 {
                            -0.4 // Penalty for going away from food
                        } else {
                            0.2 // Neutral bonus for unclear direction
                        };
                        
                        // CYCLE 20: Dynamic trail width adaptation for highway detection
                        let perp_angle_1 = angle + std::f32::consts::PI / 2.0;
                        let perp_angle_2 = angle - std::f32::consts::PI / 2.0;
                        
                        // Multi-layer sampling: near (core trail), mid (established width), far (highway detection)
                        let near_left = grid.sample_directional(pos.x, pos.y, perp_angle_1, 5.0, PheromoneType::Food);
                        let near_right = grid.sample_directional(pos.x, pos.y, perp_angle_2, 5.0, PheromoneType::Food);
                        let mid_left = grid.sample_directional(pos.x, pos.y, perp_angle_1, 10.0, PheromoneType::Food);
                        let mid_right = grid.sample_directional(pos.x, pos.y, perp_angle_2, 10.0, PheromoneType::Food);
                        let far_left = grid.sample_directional(pos.x, pos.y, perp_angle_1, 18.0, PheromoneType::Food);
                        let far_right = grid.sample_directional(pos.x, pos.y, perp_angle_2, 18.0, PheromoneType::Food);
                        
                        // Detect highway patterns based on pheromone distribution width
                        let core_strength = (near_left + near_right) / 2.0;
                        let mid_strength = (mid_left + mid_right) / 2.0;
                        let far_strength = (far_left + far_right) / 2.0;
                        
                        let trail_width_factor = if core_strength > 0.3 && mid_strength > 0.2 && far_strength > 0.1 {
                            1.5 // SUPERHIGHWAY - massive bonus for ultra-wide established trails
                        } else if core_strength > 0.2 && mid_strength > 0.15 {
                            1.35 // HIGHWAY - major bonus for well-established wide trails  
                        } else if core_strength > 0.15 && (mid_strength > 0.1 || far_strength > 0.08) {
                            1.2 // WIDE TRAIL - good bonus for expanding trails
                        } else if near_left > 0.1 || near_right > 0.1 {
                            1.1 // STANDARD TRAIL - small bonus
                        } else {
                            1.0 // NARROW TRAIL - no bonus
                        };
                        
                        // CYCLE 21: Traffic flow optimization - lane assignment based on highway patterns
                        let left_side_strength = (mid_left + near_left) / 2.0;
                        let right_side_strength = (mid_right + near_right) / 2.0;
                        
                        // Detect if we're on a highway (wide trail with traffic)
                        let is_highway = core_strength > 0.25 && mid_strength > 0.2;
                        let total_side_strength = left_side_strength + right_side_strength;
                        
                        let centering_bonus = if is_highway && total_side_strength > 0.3 {
                            // HIGHWAY TRAFFIC RULES: Food-seeking ants prefer right side, food-carrying prefer left side
                            let traffic_direction = Vec2::new(angle.cos(), angle.sin());
                            let to_nest = (Vec2::ZERO - Vec2::new(pos.x, pos.y)).normalize();
                            let toward_nest = traffic_direction.dot(to_nest) > 0.3; // Are we generally heading toward nest?
                            
                            if !ant.carrying_food {
                                // Food-seeking ants: prefer right side of highway (away from nest traffic)
                                if !toward_nest && right_side_strength > left_side_strength {
                                    0.4 // Strong bonus for following outbound lane
                                } else if toward_nest && left_side_strength > right_side_strength {
                                    -0.2 // Small penalty for going against traffic flow
                                } else {
                                    0.1 // Default small bonus
                                }
                            } else {
                                // Food-carrying ants: prefer left side of highway (toward nest)
                                if toward_nest && left_side_strength > right_side_strength {
                                    0.5 // Very strong bonus for following inbound lane
                                } else if !toward_nest && right_side_strength > left_side_strength {
                                    -0.1 // Small penalty for going wrong way
                                } else {
                                    0.2 // Default bonus
                                }
                            }
                        } else {
                            // Standard trail centering for non-highways
                            if left_side_strength > right_side_strength + 0.1 {
                                if (angle - ant.current_direction).abs() < std::f32::consts::PI / 4.0 { 0.3 } else { 0.0 }
                            } else if right_side_strength > left_side_strength + 0.1 {
                                if (angle - ant.current_direction).abs() < std::f32::consts::PI / 4.0 { 0.3 } else { 0.0 }
                            } else {
                                0.0 // Already centered on trail
                            }
                        };
                        
                        // SIMPLIFIED gradient system - reduce complexity to prevent oscillation
                        let immediate_gradient = pheromone_strength - current_pheromone;
                        
                        let gradient_bonus = if immediate_gradient > 0.08 { // CYCLE 10: More sensitive gradient detection
                            // Clear improvement - moving toward stronger pheromone
                            0.45 // CYCLE 10: Slightly stronger gradient bonus
                        } else if immediate_gradient < -0.08 {
                            // Clear decline - moving away from strong pheromone
                            -0.35 // CYCLE 10: Stronger avoidance of declining trails
                        } else {
                            // Marginal differences - neutral to reduce micro-oscillation
                            0.0
                        };
                        
                        // Smart momentum-gradient hybrid: reduce momentum when gradient is very strong
                        let hybrid_momentum = if immediate_gradient > 0.15 {
                            momentum_bonus * 0.7 // Back to Generation 43 successful value
                        } else {
                            momentum_bonus
                        };
                        
                        // CYCLE 15: Cooperative rock avoidance - check for alarm pheromones
                        let sample_x = pos.x + angle.cos() * 15.0;
                        let sample_y = pos.y + angle.sin() * 15.0;
                        let alarm_penalty = if let Some(grid_idx) = grid.world_to_grid(sample_x, sample_y) {
                            grid.alarm[grid_idx] * -20.0 // Penalty for moving toward rock warning areas
                        } else {
                            0.0
                        };
                        
                        // CYCLE 22: Add collective intelligence bonus to trail following
                        let collective_intelligence_bonus = calculate_collective_intelligence_bonus(
                            angle, pheromone_strength, &swarm_context, ant.current_direction
                        );
                        
                        // Emergency swarm dispersion if too many ants are failing in this area
                        let dispersion_penalty = if swarm_context.local_failure_rate > 0.6 && swarm_context.ant_density > 5 {
                            if (angle - swarm_context.average_failed_direction).abs() > 1.5 {
                                0.8 // Strong bonus for moving away from failed attempts
                            } else {
                                -0.5 // Penalty for following failed paths
                            }
                        } else {
                            0.0
                        };
                        
                        let effective_strength = pheromone_strength * trail_width_factor + hybrid_momentum + gradient_bonus + persistence_bonus + trail_direction_bonus + centering_bonus + alarm_penalty + collective_intelligence_bonus + dispersion_penalty;
                        
                        if effective_strength > max_pheromone {
                            max_pheromone = effective_strength;
                            best_direction = angle;
                            found_trail = true;
                        }
                    }
                }
                
                // CYCLE 21: Advanced congestion management with highway awareness
                let swarming_penalty = if ant.is_swarming && ant.nearby_ant_count >= 4 {
                    // Detect if we're in highway congestion vs regular swarming
                    let highway_congestion = max_pheromone > 1.0 && ant.nearby_ant_count >= 6; // High pheromone + crowding = highway jam
                    
                    let penalty_factor = if highway_congestion {
                        // Highway congestion - more aggressive intervention
                        (ant.nearby_ant_count as f32 * 0.2).min(0.6) 
                    } else {
                        // Regular swarming - gentler intervention
                        (ant.nearby_ant_count as f32 * 0.12).min(0.4)
                    };
                    
                    max_pheromone *= 1.0 - penalty_factor;
                    
                    // Gentle deviation to maintain trail efficiency
                    let random_deviation = (rand::random::<f32>() - 0.5) * 0.3;
                    best_direction += random_deviation;
                    
                    penalty_factor
                } else if ant.is_swarming && ant.trail_following_time > 3.0 {
                    // Only light intervention for persistent swarming
                    let penalty_factor = 0.15;
                    max_pheromone *= 1.0 - penalty_factor;
                    
                    let random_deviation = (rand::random::<f32>() - 0.5) * 0.2;
                    best_direction += random_deviation;
                    
                    penalty_factor
                } else {
                    0.0
                };
                
                if found_trail && max_pheromone > 0.2 {
                    // CYCLE 19: Loop detection for food-seeking ants
                    let should_break_from_trail = ant.time_since_progress > 15.0 && 
                                                 ant.behavior_state == AntBehaviorState::Following &&
                                                 (time.elapsed_seconds() * (ant.successful_deliveries + 1) as f32).sin() > 0.8; // Rarer break-away for exploring ants
                    
                    if !should_break_from_trail {
                        // Smooth direction change for trail following
                        ant.behavior_state = AntBehaviorState::Following;
                    let angle_diff = best_direction - ant.current_direction;
                    let smooth_angle_change = if angle_diff.abs() > std::f32::consts::PI {
                        if angle_diff > 0.0 { angle_diff - std::f32::consts::TAU } else { angle_diff + std::f32::consts::TAU }
                    } else { angle_diff };
                    
                    // CYCLE 3: Revert to balanced path following
                    ant.current_direction += smooth_angle_change * 0.22; // Revert to successful setting
                    set_ant_velocity(&mut velocity, ant.current_direction, MovementType::FollowingTrail);
                    
                    // Back to Generation 51 successful sensing intervals
                    let trail_strength_factor = (max_pheromone - 0.2).max(0.0) / 0.8;
                    ant.sensing_timer = if max_pheromone > 0.4 {
                        1.0 + trail_strength_factor * 0.5 // 1.0-1.5s for strong trails - much more commitment
                    } else {
                        0.8 + trail_strength_factor * 0.4 // 0.8-1.2s for weak trails - less frequent sensing
                    };
                    } // End of !should_break_from_trail condition
                } else {
                    // CYCLE 22: Collective exploration when no trails detected
                    if swarm_context.should_use_collective_exploration {
                        // Coordinate exploration with nearby ants to avoid redundant searching
                        ant.current_direction = swarm_context.suggested_exploration_direction;
                        set_ant_velocity(&mut velocity, ant.current_direction, MovementType::Exploring);
                        ant.behavior_state = AntBehaviorState::Exploring;
                        ant.sensing_timer = 0.8; // Moderate sensing for coordinated exploration
                        ant.time_since_progress = 0.0;
                        continue;
                    }
                    
                    // No trail found - random exploration
                    ant.behavior_state = AntBehaviorState::Exploring;
                    
                    // ENHANCED EDGE-WANDERER RECOVERY: Aggressive center-seeking behavior
                    if ant.is_edge_wanderer || (ant.world_edge_proximity < 100.0 && ant.time_since_progress > 8.0) {
                        let center = Vec2::ZERO;
                        let to_center = center - Vec2::new(pos.x, pos.y);
                        let center_direction = to_center.normalize();
                        
                        // Stronger center bias for distant ants
                        let distance_from_center = to_center.length();
                        let urgency_factor = (distance_from_center / 400.0).min(1.0);
                        
                        // Mix center direction with some randomness based on urgency
                        let random_component = (rand::random::<f32>() - 0.5) * (0.8 - urgency_factor * 0.4);
                        ant.current_direction = center_direction.y.atan2(center_direction.x) + random_component;
                        
                        set_ant_velocity(&mut velocity, ant.current_direction, MovementType::Exploring);
                        ant.sensing_timer = 0.3; // Very frequent sensing for recovery
                        
                        // Reset progress timer on intervention  
                        ant.time_since_progress = 0.0;
                    } else if ant.sensing_timer <= 0.0 {
                        // Adaptive exploration: more aggressive as search time increases
                        let search_time = if ant.last_goal_achievement_time > 0.0 {
                            time.elapsed_seconds() - ant.last_goal_achievement_time
                        } else {
                            // Time since startup ended (when ant became active)
                            (time.elapsed_seconds() - 1.0).max(0.0) // Startup was 1.0s
                        };
                        
                        // CYCLE 3: Smarter exploration with spiral search pattern for lost ants
                        let exploration_factor = (search_time / 60.0).min(1.0);
                        
                        if ant.time_since_progress > 10.0 {
                            // CYCLE 5: Earlier and more optimized spiral search
                            let lost_duration = ant.time_since_progress - 10.0;
                            let spiral_angle = lost_duration * 1.0; // Even faster spiral
                            ant.current_direction += spiral_angle.sin() * 0.45; // Slightly more aggressive
                            
                            // Very frequent sensing for rapid trail discovery
                            ant.sensing_timer = ant.sensing_timer.min(0.3); // CYCLE 14: Faster trail discovery
                        } else {
                            // Normal exploration
                            let base_angle = 1.2;
                            let max_angle = 2.2;
                            let angle_range = base_angle + (max_angle - base_angle) * exploration_factor;
                            
                            let angle_change = (rand::random::<f32>() - 0.5) * angle_range;
                            ant.current_direction += angle_change;
                        }
                        set_ant_velocity(&mut velocity, ant.current_direction, MovementType::Exploring);
                        
                        // Back to Generation 51 successful exploration sensing
                        let base_sensing = 0.6;
                        let min_sensing = 0.3;
                        let sensing_time = base_sensing - (base_sensing - min_sensing) * exploration_factor;
                        ant.sensing_timer = sensing_time + rand::random::<f32>() * 0.2;
                    }
                }
            }
            
            // Basic stuck detection
            let current_pos = Vec2::new(pos.x, pos.y);
            let distance_moved = current_pos.distance(ant.last_position);
            
            if distance_moved < 5.0 {
                ant.stuck_timer += delta_time;
                if ant.stuck_timer > 2.0 {
                    // Randomize direction when stuck
                    ant.current_direction = rand::random::<f32>() * std::f32::consts::TAU;
                    set_ant_velocity(&mut velocity, ant.current_direction, MovementType::StuckRecovery);
                    ant.stuck_timer = 0.0;
                    ant.behavior_state = AntBehaviorState::Exploring;
                }
            } else {
                ant.stuck_timer = 0.0;
            }
            ant.last_position = current_pos;
        }
        
        // Debug logging for debug ants
        static mut LAST_DEBUG_LOG: f32 = 0.0;
        let current_time = time.elapsed_seconds();
        
        unsafe {
            if current_time - LAST_DEBUG_LOG > 2.0 {
                LAST_DEBUG_LOG = current_time;
                
                for (entity, transform, ant, velocity, debug_ant) in ants.iter() {
                    if let Some(debug_marker) = debug_ant {
                        let pos = transform.translation;
                        
                        // Calculate distance to nest and nearest food
                        let dist_to_nest = Vec2::new(pos.x, pos.y).length();
                        
                        // Get pheromone readings at current position
                        let pheromone_readings = grid.sample_all_directions(pos.x, pos.y, PheromoneType::Food);
                        let current_pheromone = pheromone_readings[0];
                        let max_pheromone = pheromone_readings.iter().fold(0.0f32, |a, &b| a.max(b));
                        
                        // Time since last goal achievement
                        let time_since_goal = if ant.last_goal_achievement_time > 0.0 {
                            current_time - ant.last_goal_achievement_time
                        } else {
                            // Time since startup ended (when ant became active)
                            (current_time - 1.0).max(0.0) // Startup was 1.0s
                        };
                        
                        println!("🐜 DEBUG ANT #{} @ T={:.1}s | Pos=({:.0},{:.0}) DistToNest={:.0} | State={:?} | Carrying={} | TimeSinceGoal={:.1}s", 
                            debug_marker.ant_id, current_time, pos.x, pos.y, dist_to_nest, ant.behavior_state, ant.carrying_food, time_since_goal);
                        
                        println!("   📡 Pheromones: Current={:.3} Max={:.3} | Direction={:.2}rad | Vel=({:.1},{:.1}) | SensingTimer={:.2}s", 
                            current_pheromone, max_pheromone, ant.current_direction, velocity.x, velocity.y, ant.sensing_timer);
                        
                        if ant.stuck_timer > 1.0 {
                            println!("   ⚠️ STUCK for {:.1}s | Last movement distance: {:.1}", ant.stuck_timer, 
                                Vec2::new(pos.x, pos.y).distance(ant.last_position));
                        }
                        
                        if ant.behavior_state == AntBehaviorState::Following {
                            // Look ahead for predictive analysis
                            let lookahead_x = pos.x + ant.current_direction.cos() * 15.0;
                            let lookahead_y = pos.y + ant.current_direction.sin() * 15.0;
                            let lookahead_pheromone = grid.sample_directional(lookahead_x, lookahead_y, ant.current_direction, 3.0, PheromoneType::Food);
                            let immediate_gradient = max_pheromone - current_pheromone;
                            let predictive_gradient = lookahead_pheromone - max_pheromone;
                            
                            println!("   🔮 Trail Analysis: ImmediateGrad={:.3} PredictiveGrad={:.3} LookaheadPheromone={:.3}", 
                                immediate_gradient, predictive_gradient, lookahead_pheromone);
                        }
                        
                        println!("   📊 Stats: Deliveries={} Attempts={} HasFoundFood={} | ConsecutiveGoodTrail={:.1}s", 
                            ant.successful_deliveries, ant.delivery_attempts, ant.has_found_food, ant.consecutive_good_trail_time);
                        
                        println!();
                    }
                }
            }
        }
    }
}

// New system to detect ant swarming and proximity issues
pub fn ant_proximity_analysis_system(
    mut ants: Query<(Entity, &Transform, &mut AntState)>,
    time: Res<Time>,
) {
    let mut ant_positions: Vec<(Entity, Vec2)> = Vec::new();
    
    // First pass: collect positions
    for (entity, transform, _) in ants.iter() {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);
        ant_positions.push((entity, pos));
    }
    
    // Second pass: analyze proximity and update states
    for (entity, transform, mut ant_state) in ants.iter_mut() {
        let current_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let mut nearby_count = 0;
        let proximity_threshold = 25.0;
        
        for (other_entity, other_pos) in &ant_positions {
            if *other_entity != entity {
                let distance = current_pos.distance(*other_pos);
                if distance < proximity_threshold {
                    nearby_count += 1;
                }
            }
        }
        
        ant_state.nearby_ant_count = nearby_count;
        ant_state.is_swarming = nearby_count >= 3 && ant_state.trail_following_time > 2.0;
        
        // Update exploration efficiency
        let current_time = time.elapsed_seconds();
        let time_delta = current_time - ant_state.current_goal_start_time;
        if time_delta > 0.0 {
            let distance_from_start = current_pos.distance(ant_state.last_position);
            ant_state.exploration_efficiency = distance_from_start / time_delta.max(0.1);
        }
    }
}

// Comprehensive behavior analysis and logging system
pub fn behavior_analysis_system(
    ants: Query<(Entity, &Transform, &AntState, Option<&DebugAnt>)>,
    time: Res<Time>,
    performance_tracker: Res<PerformanceTracker>,
) {
    let current_time = time.elapsed_seconds();
    
    // Analysis counters
    let mut total_ants = 0;
    let mut ants_with_trails = 0;
    let mut swarming_ants = 0;
    let mut edge_wanderers = 0;
    let mut stuck_ants = 0;
    let mut efficient_ants = 0;
    let mut total_time_since_progress = 0.0;
    let mut total_exploration_efficiency = 0.0;
    
    for (entity, transform, ant, debug_ant) in ants.iter() {
        total_ants += 1;
        
        if ant.can_see_trail { ants_with_trails += 1; }
        if ant.is_swarming { swarming_ants += 1; }
        if ant.is_edge_wanderer { edge_wanderers += 1; }
        if ant.stuck_timer > 3.0 { stuck_ants += 1; }
        if ant.exploration_efficiency > 10.0 { efficient_ants += 1; }
        
        total_time_since_progress += ant.time_since_progress;
        total_exploration_efficiency += ant.exploration_efficiency;
        
        // Detailed logging for debug ant
        if let Some(debug) = debug_ant {
            if current_time.fract() < 0.1 { // Log roughly once per second
                let pos = transform.translation;
                println!("\n🐜 DEBUG ANT #{} ANALYSIS at {:.1}s:", debug.ant_id, current_time);
                println!("   📍 Position: ({:.1}, {:.1}) | WorldEdgeProximity: {:.1}", pos.x, pos.y, ant.world_edge_proximity);
                println!("   👁️ CanSeeTrail: {} | DistanceFromTrail: {:.1} | GradientStrength: {:.3}", 
                    ant.can_see_trail, ant.distance_from_trail, ant.trail_gradient_strength);
                println!("   🚶 TimeSinceProgress: {:.1}s | ExplorationEfficiency: {:.2}", 
                    ant.time_since_progress, ant.exploration_efficiency);
                println!("   👥 NearbyAnts: {} | IsSwarming: {} | IsEdgeWanderer: {}", 
                    ant.nearby_ant_count, ant.is_swarming, ant.is_edge_wanderer);
                println!("   🛤️ TrailFollowingTime: {:.1}s | LastTrailContact: {:.1}s ago", 
                    ant.trail_following_time, current_time - ant.last_trail_contact_time);
                println!("   🎯 CarryingFood: {} | BehaviorState: {:?}", ant.carrying_food, ant.behavior_state);
            }
        }
    }
    
    // Aggregate analysis logging every 5 seconds
    if current_time % 5.0 < 0.1 {
        let avg_time_since_progress = total_time_since_progress / total_ants as f32;
        let avg_exploration_efficiency = total_exploration_efficiency / total_ants as f32;
        let trail_visibility_rate = (ants_with_trails as f32 / total_ants as f32) * 100.0;
        let swarming_rate = (swarming_ants as f32 / total_ants as f32) * 100.0;
        let edge_wanderer_rate = (edge_wanderers as f32 / total_ants as f32) * 100.0;
        
        println!("\n📊 BEHAVIOR ANALYSIS REPORT at {:.1}s:", current_time);
        println!("   Total Ants: {} | AvgTimeSinceProgress: {:.1}s | AvgExplorationEfficiency: {:.2}", 
            total_ants, avg_time_since_progress, avg_exploration_efficiency);
        println!("   TrailVisibilityRate: {:.1}% ({}/{}) | SwarmingRate: {:.1}% ({}/{})", 
            trail_visibility_rate, ants_with_trails, total_ants, swarming_rate, swarming_ants, total_ants);
        println!("   EdgeWandererRate: {:.1}% ({}/{}) | StuckAnts: {}", 
            edge_wanderer_rate, edge_wanderers, total_ants, stuck_ants);
        println!("   EfficientAnts: {} | CurrentAvgGoalTime: {:.1}s | Deliveries: {}", 
            efficient_ants, performance_tracker.average_time_since_goal, performance_tracker.successful_deliveries);
        println!();
    }
}

pub fn movement_system(
    mut ants: Query<(&mut Transform, &Velocity, &AntState)>,
    rocks: Query<(&Transform, &Rock), Without<AntState>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    
    for (mut ant_transform, velocity, _ant_state) in ants.iter_mut() {
        // Calculate proposed new position
        let new_x = ant_transform.translation.x + velocity.x * delta_time;
        let new_y = ant_transform.translation.y + velocity.y * delta_time;
        let new_position = Vec2::new(new_x, new_y);
        
        // Check for collision with rocks
        let mut collision_detected = false;
        
        for (rock_transform, rock) in rocks.iter() {
            let rock_pos = Vec2::new(rock_transform.translation.x, rock_transform.translation.y);
            let distance = new_position.distance(rock_pos);
            let ant_radius = 6.0; // Half the ant size (12x12)
            
            if distance < rock.radius + ant_radius {
                collision_detected = true;
                break;
            }
        }
        
        // If no collision detected, apply the movement
        if !collision_detected {
            ant_transform.translation.x = new_x;
            ant_transform.translation.y = new_y;
        }
        // If collision detected, ant stays at current position (blocked by rock)
        
        // Keep ants within world bounds
        let bound = 480.0;
        if ant_transform.translation.x > bound {
            ant_transform.translation.x = bound;
        } else if ant_transform.translation.x < -bound {
            ant_transform.translation.x = -bound;
        }
        
        if ant_transform.translation.y > bound {
            ant_transform.translation.y = bound;
        } else if ant_transform.translation.y < -bound {
            ant_transform.translation.y = -bound;
        }
    }
}

pub fn pheromone_deposit_system(
    ants: Query<(&Transform, &AntState)>,
    mut pheromone_grid: Option<ResMut<PheromoneGrid>>,
    config: Res<SimConfig>,
    time: Res<Time>,
) {
    if let Some(ref mut grid) = pheromone_grid {
        for (transform, ant) in ants.iter() {
            let current_pos = transform.translation;
            let last_pos = Vec3::new(ant.last_position.x, ant.last_position.y, 0.0);
            
            // Calculate distance moved this frame
            let movement_distance = current_pos.distance(last_pos);
            
            // Deposit pheromones along the path if ant moved significantly
            if movement_distance > 0.5 {
                // Number of deposits based on distance moved (ensure continuous trail)
                let num_deposits = (movement_distance / 0.8).ceil() as i32;
                
                for i in 0..=num_deposits {
                    let t = if num_deposits > 0 { i as f32 / num_deposits as f32 } else { 0.0 };
                    let deposit_pos = last_pos.lerp(current_pos, t);
                    
                    if ant.carrying_food {
                        // CYCLE 16: Enhanced trail quality based on ant success and efficiency
                        let decay_factor = (-ant.distance_from_food * 0.01).exp(); // Balanced distance decay rate
                        
                        // Quality multiplier based on ant success history
                        let success_factor = if ant.successful_deliveries > 0 {
                            1.0 + (ant.successful_deliveries as f32 * 0.1).min(0.5) // Up to 50% bonus for experienced ants
                        } else {
                            0.8 // Slight penalty for unproven ants
                        };
                        
                        // Efficiency bonus for ants making progress vs stuck ants
                        let efficiency_factor = if ant.time_since_progress < 5.0 {
                            1.2 // 20% bonus for ants making good progress
                        } else if ant.time_since_progress > 15.0 {
                            0.6 // 40% penalty for stuck ants
                        } else {
                            1.0
                        };
                        
                        // Speed bonus for fast-moving ants (better path quality)
                        let speed_factor = (movement_distance / 0.8).min(1.5); // Up to 50% bonus for fast ants
                        
                        let base_deposit_amount = config.lay_rate_food * config.food_quality_weight * decay_factor * success_factor * efficiency_factor * speed_factor;
                        
                        // CYCLE 20: Collaborative trail widening - check for nearby trail activity
                        let current_pheromone = grid.sample_directional(deposit_pos.x, deposit_pos.y, 0.0, 3.0, PheromoneType::Food);
                        let traffic_factor = if current_pheromone > 2.0 {
                            // High traffic area - widen the trail by depositing in adjacent areas
                            1.3
                        } else if current_pheromone > 1.0 {
                            // Moderate traffic - slight widening
                            1.15
                        } else {
                            1.0 // New trail - normal deposit
                        };
                        
                        let deposit_amount = base_deposit_amount * traffic_factor;
                        
                        // Primary deposit
                        grid.deposit(
                            deposit_pos.x, 
                            deposit_pos.y, 
                            PheromoneType::Food, 
                            deposit_amount / (num_deposits + 1) as f32
                        );
                        
                        // CYCLE 21: Lane-specific highway formation with traffic flow awareness
                        if current_pheromone > 1.5 && ant.successful_deliveries > 1 {
                            let movement_direction_3d = (current_pos - last_pos).normalize();
                            let movement_direction = Vec2::new(movement_direction_3d.x, movement_direction_3d.y);
                            let perp_angle = movement_direction.y.atan2(movement_direction.x) + std::f32::consts::PI / 2.0;
                            
                            // Determine which lane this ant should reinforce
                            let to_nest = (Vec2::ZERO - Vec2::new(deposit_pos.x, deposit_pos.y)).normalize();
                            let toward_nest = movement_direction.dot(to_nest) > 0.1;
                            
                            let side_deposit = deposit_amount * 0.35; // Increased side deposit for lane definition
                            
                            if toward_nest {
                                // Food-carrying ant heading toward nest - strengthen left lane (inbound)
                                let lane_offset = 3.5; // Closer to center for priority lane
                                grid.deposit(
                                    deposit_pos.x - perp_angle.cos() * lane_offset,
                                    deposit_pos.y - perp_angle.sin() * lane_offset,
                                    PheromoneType::Food,
                                    side_deposit * 1.2 / (num_deposits + 1) as f32 // 20% bonus for inbound lane
                                );
                                
                                // Light deposit on right lane for highway definition
                                grid.deposit(
                                    deposit_pos.x + perp_angle.cos() * 6.0,
                                    deposit_pos.y + perp_angle.sin() * 6.0,
                                    PheromoneType::Food,
                                    side_deposit * 0.4 / (num_deposits + 1) as f32
                                );
                            } else {
                                // Food-seeking ant heading away from nest - strengthen right lane (outbound)
                                let lane_offset = 5.5; // Further from center
                                grid.deposit(
                                    deposit_pos.x + perp_angle.cos() * lane_offset,
                                    deposit_pos.y + perp_angle.sin() * lane_offset,
                                    PheromoneType::Food,
                                    side_deposit / (num_deposits + 1) as f32
                                );
                                
                                // Light deposit on left lane for highway definition
                                grid.deposit(
                                    deposit_pos.x - perp_angle.cos() * 4.0,
                                    deposit_pos.y - perp_angle.sin() * 4.0,
                                    PheromoneType::Food,
                                    side_deposit * 0.6 / (num_deposits + 1) as f32
                                );
                            }
                        }
                        
                        // NEST PHEROMONE FIX: Food-carrying ants should ALSO deposit strong nest pheromones!
                        // This creates proven successful return paths for other food carriers to follow
                        let distance_to_nest = Vec2::new(deposit_pos.x, deposit_pos.y).length();
                        let nest_proximity_bonus = if distance_to_nest < 150.0 {
                            2.0 // Very strong bonus when approaching nest
                        } else if distance_to_nest < 300.0 {
                            1.5 // Strong bonus for mid-range
                        } else {
                            1.0 // Standard rate for distant areas
                        };
                        
                        // Success-based multiplier: experienced ants lay stronger nest trails
                        let success_multiplier = 1.0 + (ant.successful_deliveries as f32 * 0.3).min(1.5);
                        
                        // Progress bonus: ants making good progress lay stronger trails
                        let progress_bonus = if ant.time_since_progress < 5.0 {
                            1.3 // Bonus for ants making good progress toward nest
                        } else {
                            0.8 // Reduced strength for struggling ants
                        };
                        
                        let nest_deposit_amount = config.lay_rate_nest * nest_proximity_bonus * success_multiplier * progress_bonus;
                        
                        // Deposit strong nest pheromones along the successful return path
                        grid.deposit(
                            deposit_pos.x,
                            deposit_pos.y,
                            PheromoneType::Nest,
                            nest_deposit_amount / (num_deposits + 1) as f32
                        );
                        
                    } else {
                        // NEST PHEROMONE FIX: Exploring ants should deposit minimal/no nest pheromones
                        // Only successful food-carriers should create nest trails!
                        
                        // Very weak nest pheromone from experienced exploring ants only
                        if ant.has_found_food && ant.successful_deliveries > 0 {
                            let time_since_nest = time.elapsed_seconds() - ant.current_goal_start_time;
                            let time_decay = (-time_since_nest * 0.2).exp(); // Faster decay
                            let weak_deposit = config.lay_rate_nest * 0.1 * time_decay; // Much weaker
                            
                            grid.deposit(
                                deposit_pos.x,
                                deposit_pos.y,
                                PheromoneType::Nest,
                                weak_deposit / (num_deposits + 1) as f32
                            );
                        }
                        // Most exploring ants deposit NO nest pheromones
                    }
                }
            } else {
                // For very small movements, just deposit at current position
                if ant.carrying_food {
                    // Food pheromone deposition
                    let decay_factor = (-ant.distance_from_food * 0.005).exp();
                    let food_deposit_amount = config.lay_rate_food * config.food_quality_weight * decay_factor;
                    grid.deposit(current_pos.x, current_pos.y, PheromoneType::Food, food_deposit_amount);
                    
                    // NEST PHEROMONE FIX: Food-carrying ants ALSO deposit nest pheromones for small movements
                    let distance_to_nest = Vec2::new(current_pos.x, current_pos.y).length();
                    let nest_proximity_bonus = if distance_to_nest < 150.0 {
                        2.0 // Very strong bonus when approaching nest
                    } else if distance_to_nest < 300.0 {
                        1.5 // Strong bonus for mid-range
                    } else {
                        1.0
                    };
                    
                    let success_multiplier = 1.0 + (ant.successful_deliveries as f32 * 0.3).min(1.5);
                    let progress_bonus = if ant.time_since_progress < 5.0 { 1.3 } else { 0.8 };
                    let nest_deposit_amount = config.lay_rate_nest * nest_proximity_bonus * success_multiplier * progress_bonus;
                    
                    grid.deposit(current_pos.x, current_pos.y, PheromoneType::Nest, nest_deposit_amount);
                } else {
                    // NEST PHEROMONE FIX: Exploring ants deposit very little nest pheromone for small movements
                    // Only experienced exploring ants deposit weak nest pheromones
                    if ant.has_found_food && ant.successful_deliveries > 0 {
                        let time_since_nest = time.elapsed_seconds() - ant.current_goal_start_time;
                        let time_decay = (-time_since_nest * 0.2).exp();
                        let weak_deposit = config.lay_rate_nest * 0.1 * time_decay; // Much weaker
                        
                        grid.deposit(current_pos.x, current_pos.y, PheromoneType::Nest, weak_deposit);
                    }
                    // Most exploring ants deposit NO nest pheromones for small movements
                }
            }
        }
    }
}

pub fn pheromone_update_system(
    mut pheromone_grid: Option<ResMut<PheromoneGrid>>,
    food_sources: Query<&Transform, With<FoodSource>>,
    config: Res<SimConfig>,
) {
    if let Some(ref mut grid) = pheromone_grid {
        // FOOD SCENT: Food sources naturally emit pheromones in smooth circular gradient
        for food_transform in food_sources.iter() {
            let food_pos = food_transform.translation;
            
            let max_radius = 60.0;
            let max_strength = 8.0; // Strong natural scent at center
            
            // Create smooth circular gradient using polar coordinates
            // More emission points closer to center for natural concentration
            for ring in 0..=12 { // 13 concentric rings from center to edge
                let ring_radius = (ring as f32 / 12.0) * max_radius;
                let points_in_ring = if ring == 0 { 
                    1 // Center point
                } else { 
                    (ring * 6).max(8) // More points in outer rings for coverage
                };
                
                for point in 0..points_in_ring {
                    let angle = (point as f32 / points_in_ring as f32) * std::f32::consts::TAU;
                    
                    let emit_x = food_pos.x + ring_radius * angle.cos();
                    let emit_y = food_pos.y + ring_radius * angle.sin();
                    
                    // Smooth falloff: strongest at center, weaker at edges
                    let distance_factor = ring_radius / max_radius;
                    let falloff_factor = (1.0 - distance_factor * distance_factor).max(0.0);
                    let strength = max_strength * falloff_factor;
                    
                    if strength > 0.2 {
                        grid.deposit(emit_x, emit_y, PheromoneType::Food, strength * 0.025);
                    }
                }
            }
        }
        
        let evap_rates = (config.evap_food, config.evap_nest, config.evap_alarm);
        let diff_rates = (config.diff_food, config.diff_nest, config.diff_alarm);
        
        grid.update(evap_rates, diff_rates);
    }
}

pub fn food_collection_system(
    mut ants: Query<(Entity, &Transform, &mut AntState, &mut Velocity, Option<&DebugAnt>)>,
    mut food_sources: Query<(&Transform, &mut FoodSource)>,
    nests: Query<&Transform, (With<Nest>, Without<AntState>)>,
    mut performance_tracker: ResMut<PerformanceTracker>,
    time: Res<Time>,
) {
    let nest_pos = if let Ok(nest_transform) = nests.get_single() {
        nest_transform.translation
    } else {
        Vec3::ZERO
    };
    
    for (entity, ant_transform, mut ant, mut velocity, debug_ant) in ants.iter_mut() {
        let ant_pos = ant_transform.translation;
        
        if !ant.carrying_food && ant.food_collection_timer <= 0.0 {
            // Look for food sources
            for (food_transform, food) in food_sources.iter() {
                let food_pos = food_transform.translation;
                let distance = ant_pos.distance(food_pos);
                
                if distance < 25.0 && food.amount > 0.0 { // Restored to original pickup distance
                    // Start collecting food
                    ant.food_collection_timer = 0.3;
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                    break;
                }
            }
        } else if ant.food_collection_timer > 0.0 {
            // Currently collecting food
            ant.food_collection_timer -= time.delta_seconds();
            velocity.x = 0.0;
            velocity.y = 0.0;
            
            if ant.food_collection_timer <= 0.0 {
                // Look for nearby food to take
                for (food_transform, mut food) in food_sources.iter_mut() {
                    let food_pos = food_transform.translation;
                    let distance = ant_pos.distance(food_pos);
                    
                    if distance < 25.0 && food.amount > 0.0 { // Restored to original pickup distance
                        let take_amount = 1.0;
                        food.amount -= take_amount;
                        ant.carrying_food = true;
                        ant.food_pickup_time = time.elapsed_seconds();
                        ant.has_found_food = true;
                        ant.food_carry_start_time = time.elapsed_seconds();
                        ant.last_goal_achievement_time = time.elapsed_seconds();
                        ant.time_since_progress = 0.0; // Reset progress timer on food pickup
                        performance_tracker.total_food_collected += take_amount;
                        
                        // Debug logging for food pickup
                        if let Some(debug_marker) = debug_ant {
                            let search_time = (time.elapsed_seconds() - 1.0).max(0.0); // Time since 1.0s startup ended
                            println!("🎯 DEBUG ANT #{} FOUND FOOD! @ T={:.1}s | Pos=({:.0},{:.0}) | SearchTime={:.1}s | FoodLeft={:.1}", 
                                debug_marker.ant_id, time.elapsed_seconds(), ant_pos.x, ant_pos.y, search_time, food.amount);
                        }
                        
                        // Head toward nest
                        let direction = nest_pos - ant_pos;
                        let direction_2d = Vec2::new(direction.x, direction.y);
                        set_ant_velocity_from_vector(&mut velocity, direction_2d, MovementType::Legacy);
                        break;
                    }
                }
            }
        } else if ant.carrying_food {
            // Look for nest to drop off food
            let distance = ant_pos.distance(nest_pos);
            
            if distance < 15.0 { // Much smaller radius - ants must actually reach the nest
                // Successful delivery
                ant.carrying_food = false;
                ant.delivery_attempts += 1;
                ant.successful_deliveries += 1;
                ant.last_goal_achievement_time = time.elapsed_seconds();
                ant.time_since_progress = 0.0; // Reset progress timer on successful delivery
                
                // Track delivery metrics
                let delivery_time = time.elapsed_seconds() - ant.food_pickup_time;
                let return_time = time.elapsed_seconds() - ant.food_carry_start_time;
                performance_tracker.delivery_times.push(delivery_time);
                performance_tracker.return_times.push(return_time);
                performance_tracker.successful_deliveries += 1;
                performance_tracker.last_delivery_time = time.elapsed_seconds();
                
                // Update averages
                let total_time: f32 = performance_tracker.delivery_times.iter().sum();
                performance_tracker.average_delivery_time = total_time / performance_tracker.delivery_times.len() as f32;
                
                let total_return_time: f32 = performance_tracker.return_times.iter().sum();
                performance_tracker.average_return_time = total_return_time / performance_tracker.return_times.len() as f32;
                
                // Debug logging for food delivery
                if let Some(debug_marker) = debug_ant {
                    println!("✅ DEBUG ANT #{} DELIVERED FOOD! @ T={:.1}s | TotalDeliveries={} | ReturnTime={:.1}s", 
                        debug_marker.ant_id, time.elapsed_seconds(), ant.successful_deliveries, return_time);
                }
                
                
                // Start exploring again
                ant.behavior_state = AntBehaviorState::Exploring;
                ant.sensing_timer = 0.2; // CYCLE 14: Ultra-fast exploration sensing
                ant.current_direction = rand::random::<f32>() * std::f32::consts::TAU;
                set_ant_velocity(&mut velocity, ant.current_direction, MovementType::Legacy);
            }
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
    
    performance_tracker.time_since_goal_samples.clear();
    
    for ant in ants.iter() {
        if ant.stuck_timer > 3.0 {
            stuck_count += 1;
        }
        
        if ant.direction_changes > 5 && ant.stuck_timer > 1.0 {
            oscillating_count += 1;
        }
        
        if !ant.has_found_food && ant.startup_timer <= 0.0 && runtime > 45.0 {
            lost_count += 1;
        }
        
        if ant.carrying_food && ant.food_carry_start_time > 0.0 && 
           runtime - ant.food_carry_start_time > 30.0 {
            lost_food_carriers_count += 1;
        }
        
        let time_since_goal = if ant.last_goal_achievement_time > 0.0 {
            runtime - ant.last_goal_achievement_time
        } else {
            // Time since startup ended (1.0s)
            (runtime - 1.0).max(0.0)
        };
        
        if ant.startup_timer <= 0.0 {
            performance_tracker.time_since_goal_samples.push(time_since_goal);
        }
    }
    
    performance_tracker.stuck_ants_count = stuck_count;
    performance_tracker.oscillating_ants_count = oscillating_count;
    performance_tracker.lost_ants_count = lost_count;
    performance_tracker.lost_food_carriers_count = lost_food_carriers_count;
    
    performance_tracker.average_time_since_goal = if !performance_tracker.time_since_goal_samples.is_empty() {
        performance_tracker.time_since_goal_samples.iter().sum::<f32>() / performance_tracker.time_since_goal_samples.len() as f32
    } else {
        0.0
    };
    
    if performance_tracker.simulation_start_time == 0.0 {
        performance_tracker.simulation_start_time = time.elapsed_seconds();
    }
    
    // Auto-exit conditions
    if oscillating_count >= 20 {
        println!("\n🚨 AUTO-EXIT: Too many oscillating ants ({})", oscillating_count);
        exit_writer.send(AppExit::Success);
    }
    
    // Removed "too many lost food carriers" exit condition to allow more time for pathfinding
    
    if time.elapsed_seconds() > 90.0 {
        println!("\n🎉 SUCCESS: 90 seconds completed with {:.1}s avg goal time!", performance_tracker.average_time_since_goal);
        exit_writer.send(AppExit::Success);
    }
}

// Visual and UI systems remain unchanged
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
            
            let range = config.world_size as f32 * 0.4;
            let mut x = (rand::random::<f32>() - 0.5) * range;
            let mut y = (rand::random::<f32>() - 0.5) * range;
            
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
                    color: Color::srgb(1.0, 1.0, 0.0),
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
                        color: Color::srgb(1.0, 0.0, 0.0),
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
                    hysteresis_threshold: 0.0005,
                    consecutive_good_trail_time: 0.0,
                    food_pickup_time: 0.0,
                    delivery_attempts: 0,
                    successful_deliveries: 0,
                    startup_timer: 2.0 + (i as f32) * 0.1, // Staggered startup: 2.0-5.5s range
                    has_found_food: false,
                    food_carry_start_time: 0.0,
                    last_goal_achievement_time: 0.0,
                    current_goal_start_time: 0.0,
                    
                    // Initialize diagnostic fields
                    can_see_trail: false,
                    distance_from_trail: f32::INFINITY,
                    trail_following_time: 0.0,
                    last_trail_contact_time: 0.0,
                    is_swarming: false,
                    nearby_ant_count: 0,
                    time_since_progress: 0.0,
                    exploration_efficiency: 0.0,
                    is_edge_wanderer: false,
                    world_edge_proximity: 0.0,
                    trail_gradient_strength: 0.0,
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
                let angle = rand::random::<f32>() * std::f32::consts::TAU;
                let distance = 80.0 + rand::random::<f32>() * 120.0;
                (angle.cos() * distance, angle.sin() * distance)
            } else {
                let range = (config.world_size as f32) * 0.3;
                ((rand::random::<f32>() - 0.5) * range, (rand::random::<f32>() - 0.5) * range)
            };
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.0, 1.0, 0.0),
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
        
        camera_transform.translation += camera_move * 0.016;
        
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

pub fn update_pheromone_visualization(
    mut pheromone_sprites: Query<(&mut Sprite, &mut Transform), With<PheromoneVisualization>>,
    pheromone_grid: Option<Res<PheromoneGrid>>,
    color_config: Res<ColorConfig>,
) {
    if let Some(grid) = pheromone_grid {
        for (mut sprite, mut transform) in pheromone_sprites.iter_mut() {
            let world_x = transform.translation.x;
            let world_y = transform.translation.y;
            
            if let Some(idx) = grid.world_to_grid(world_x, world_y) {
                let food_strength = grid.food_trail[idx];
                let nest_strength = grid.nest_trail[idx];
                let max_strength = food_strength.max(nest_strength);
                
                if max_strength > 0.01 {
                    if food_strength > nest_strength {
                        // Logarithmic scaling: green = log(food_pheromone)^1.3 * 20, clamped to [0,255]
                        let log_intensity = food_strength.ln().powf(1.3) * 20.0;
                        let green_value = (log_intensity / 255.0).clamp(0.0, 1.0);
                        let base_color = color_config.food_pheromone;
                        sprite.color = Color::srgba(
                            base_color.to_srgba().red,
                            green_value, // Use calculated logarithmic green intensity
                            base_color.to_srgba().blue,
                            green_value // Use same value for alpha to show intensity
                        );
                        transform.translation.z = -9.0;
                    } else {
                        // Same logarithmic scaling for nest pheromone (blue)
                        let log_intensity = nest_strength.ln().powf(1.3) * 20.0;
                        let blue_value = (log_intensity / 255.0).clamp(0.0, 1.0);
                        let base_color = color_config.nest_pheromone;
                        sprite.color = Color::srgba(
                            base_color.to_srgba().red,
                            base_color.to_srgba().green,
                            blue_value, // Use calculated logarithmic blue intensity
                            blue_value // Use same value for alpha to show intensity
                        );
                        transform.translation.z = -10.0;
                    }
                } else {
                    sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                    transform.translation.z = -10.0;
                }
            } else {
                sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
            }
        }
    }
}

pub fn setup_debug_ui(mut commands: Commands, color_config: Res<ColorConfig>) {
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

    commands.spawn((
        TextBundle::from_section(
            "Performance Metrics",
            TextStyle {
                font_size: 18.0,
                color: Color::srgb(0.0, 1.0, 0.0),
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
    
    debug_info.hovered_entity = None;
    debug_info.entity_info = String::new();
    
    // Check for hovered ants
    for (entity, transform, ant_state, velocity) in ant_query.iter() {
        let distance = cursor_pos.distance(transform.translation.truncate());
        if distance < 15.0 {
            debug_info.hovered_entity = Some(entity);
            debug_info.entity_info = format!(
                "=== BASIC ANT ===\nEntity: {:?}\nPos: ({:.1}, {:.1})\nBehavior: {:?}\nCarrying Food: {}\nDirection: {:.1}°\nVelocity: ({:.2}, {:.2})\nSensing Timer: {:.2}\nStuck Timer: {:.2}",
                entity,
                transform.translation.x, transform.translation.y,
                ant_state.behavior_state,
                ant_state.carrying_food,
                ant_state.current_direction.to_degrees(),
                velocity.x, velocity.y,
                ant_state.sensing_timer,
                ant_state.stuck_timer
            );
            break;
        }
    }
    
    if debug_info.hovered_entity.is_none() {
        for (entity, transform, nest) in nest_query.iter() {
            let distance = cursor_pos.distance(transform.translation.truncate());
            if distance < 50.0 {
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
    
    if debug_info.hovered_entity.is_none() {
        for (entity, transform, food) in food_query.iter() {
            let distance = cursor_pos.distance(transform.translation.truncate());
            if distance < 20.0 {
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
    if let Ok(mut text) = pheromone_text_query.get_single_mut() {
        text.sections[0].value = debug_info.pheromone_info.clone();
    }
    
    if let Ok(mut text) = entity_text_query.get_single_mut() {
        text.sections[0].value = debug_info.entity_info.clone();
    }
    
    if let Ok(mut text) = performance_text_query.get_single_mut() {
        text.sections[0].value = format!(
            "🎯 PERFORMANCE METRICS 🎯\n\n⏰ Avg Time Since Goal: {:.1}s\n\n✅ Successful Deliveries: {}\n❌ Failed Attempts: {}\n📦 Total Food Collected: {:.1}\n⏱️ Avg Delivery Time: {:.1}s\n🏠 Avg Return Time: {:.1}s\n\n🚫 Stuck Ants: {}\n🔄 Oscillating Ants: {}\n🔍 Lost Ants: {}\n🍯 Lost Food Carriers: {},",
            performance_tracker.average_time_since_goal,
            performance_tracker.successful_deliveries,
            performance_tracker.failed_attempts,
            performance_tracker.total_food_collected,
            performance_tracker.average_delivery_time,
            performance_tracker.average_return_time,
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
        let mut ant_clicked = false;
        
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<SelectedAnt>();
        }
        
        for entity in ant_query.iter() {
            if debug_info.hovered_entity == Some(entity) {
                commands.entity(entity).insert(SelectedAnt);
                debug_info.selected_entity = Some(entity);
                ant_clicked = true;
                break;
            }
        }
        
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
            if debug_info.entity_info.is_empty() {
                debug_info.entity_info = format!(
                    "=== BASIC ANT ===\nEntity: {:?}\nPos: ({:.1}, {:.1})\nBehavior: {:?}\nCarrying Food: {}\nDirection: {:.1}°\nVelocity: ({:.2}, {:.2})\nSensing Timer: {:.2}\nStuck Timer: {:.2}",
                    entity,
                    transform.translation.x, transform.translation.y,
                    ant_state.behavior_state,
                    ant_state.carrying_food,
                    ant_state.current_direction.to_degrees(),
                    velocity.x, velocity.y,
                    ant_state.sensing_timer,
                    ant_state.stuck_timer
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
    for outline_entity in existing_outlines.iter() {
        commands.entity(outline_entity).despawn();
    }
    
    for (_ant_entity, transform) in selected_ants.iter() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: color_config.debug_selection,
                    custom_size: Some(Vec2::new(14.0, 14.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z - 0.1
                ),
                ..default()
            },
            crate::components::AntOutline,
        ));
    }
}

// CYCLE 22: Collective swarm intelligence structures and functions
#[derive(Clone)]
struct SwarmContext {
    ant_density: u32,
    local_failure_rate: f32,
    average_failed_direction: f32,
    should_use_collective_exploration: bool,
    suggested_exploration_direction: f32,
    exploration_pressure: f32,
    least_explored_direction: f32,
    collective_confidence: f32,
    successful_ant_directions: Vec<f32>,
}

// CYCLE 22: Analyze local swarm intelligence to make collective decisions
fn analyze_local_swarm_intelligence(
    x: f32, y: f32,
    ant: &AntState,
    entity: Entity,
    ant_positions: &[(Entity, Vec2, bool, u32)],
    current_time: f32
) -> SwarmContext {
    let current_pos = Vec2::new(x, y);
    let mut context = SwarmContext {
        ant_density: 0,
        local_failure_rate: 0.0,
        average_failed_direction: ant.current_direction,
        should_use_collective_exploration: false,
        suggested_exploration_direction: ant.current_direction,
        exploration_pressure: 0.0,
        least_explored_direction: 0.0,
        collective_confidence: 0.5,
        successful_ant_directions: Vec::new(),
    };
    
    let mut nearby_ants = 0;
    let mut struggling_ants = 0;
    let mut successful_ants = 0;
    let mut failed_directions = Vec::new();
    let mut successful_directions = Vec::new();
    let mut exploration_directions = Vec::new();
    
    // Analyze nearby ants within 60 unit radius for collective intelligence
    for (other_entity, other_pos, carrying_food, successful_deliveries) in ant_positions.iter() {
        if *other_entity == entity { continue; }
        
        let distance = current_pos.distance(*other_pos);
        if distance > 60.0 { continue; }
        
        nearby_ants += 1;
        
        // Classify ant performance based on success metrics
        if *successful_deliveries > 0 {
            successful_ants += 1;
            // Record directions of successful ants for collective following
            let direction_to_successful = (*other_pos - current_pos).normalize();
            successful_directions.push(direction_to_successful.y.atan2(direction_to_successful.x));
        } else {
            struggling_ants += 1;
            // Record directions away from struggling ants
            let direction_from_struggling = (current_pos - *other_pos).normalize();
            failed_directions.push(direction_from_struggling.y.atan2(direction_from_struggling.x));
        }
        
        // Track exploration patterns
        if !carrying_food {
            let exploration_dir = (*other_pos - current_pos).normalize();
            exploration_directions.push(exploration_dir.y.atan2(exploration_dir.x));
        }
    }
    
    context.ant_density = nearby_ants;
    
    // Calculate local failure rate and collective confidence
    if nearby_ants > 0 {
        context.local_failure_rate = struggling_ants as f32 / nearby_ants as f32;
        context.collective_confidence = successful_ants as f32 / nearby_ants as f32;
        
        // Calculate average direction of failed attempts
        if !failed_directions.is_empty() {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            for &dir in &failed_directions {
                sum_x += dir.cos();
                sum_y += dir.sin();
            }
            context.average_failed_direction = sum_y.atan2(sum_x);
        }
        
        // Store successful directions for collective intelligence
        context.successful_ant_directions = successful_directions;
    }
    
    // Determine if collective exploration should be used
    let exploration_threshold = if ant.time_since_progress > 8.0 {
        0.3 // Lower threshold for struggling ants
    } else {
        0.6 // Higher threshold for doing-well ants
    };
    
    context.should_use_collective_exploration = context.local_failure_rate > exploration_threshold && 
                                               nearby_ants >= 3 &&
                                               !context.successful_ant_directions.is_empty();
    
    if context.should_use_collective_exploration {
        // Calculate least explored direction using directional analysis
        let mut direction_coverage = [0u32; 8]; // 8 compass directions
        
        for &dir in &exploration_directions {
            let compass_index = ((dir + std::f32::consts::PI) / (std::f32::consts::TAU / 8.0)) as usize % 8;
            direction_coverage[compass_index] += 1;
        }
        
        // Find direction with least exploration
        let min_coverage = direction_coverage.iter().min().unwrap_or(&0);
        let least_explored_index = direction_coverage.iter().position(|&x| x == *min_coverage).unwrap_or(0);
        context.least_explored_direction = (least_explored_index as f32 * std::f32::consts::TAU / 8.0) - std::f32::consts::PI;
        
        // Suggest exploration direction with some randomization to avoid clustering
        let base_exploration = context.least_explored_direction;
        let randomization = (rand::random::<f32>() - 0.5) * 0.8;
        context.suggested_exploration_direction = base_exploration + randomization;
        
        context.exploration_pressure = context.local_failure_rate;
    }
    
    context
}

// CYCLE 22: Calculate collective intelligence bonus for trail following
fn calculate_collective_intelligence_bonus(
    angle: f32,
    pheromone_strength: f32,
    swarm_context: &SwarmContext,
    current_direction: f32
) -> f32 {
    let mut bonus = 0.0;
    
    // High collective confidence: follow the wisdom of successful nearby ants
    if swarm_context.collective_confidence > 0.6 && !swarm_context.successful_ant_directions.is_empty() {
        // Calculate alignment with successful ant directions
        let mut best_alignment = -1.0;
        for &successful_dir in &swarm_context.successful_ant_directions {
            let alignment = (angle.cos() * successful_dir.cos() + angle.sin() * successful_dir.sin());
            if alignment > best_alignment {
                best_alignment = alignment;
            }
        }
        
        if best_alignment > 0.5 {
            bonus += 0.7 * swarm_context.collective_confidence; // Strong bonus for following successful ants
        }
    }
    
    // Low collective confidence: encourage exploration away from crowded areas
    else if swarm_context.collective_confidence < 0.3 && swarm_context.ant_density > 4 {
        // Bonus for moving away from average failed direction
        let angle_diff = (angle - swarm_context.average_failed_direction).abs();
        if angle_diff > std::f32::consts::PI / 2.0 {
            bonus += 0.5; // Reward exploring opposite directions from failures
        }
    }
    
    // Distributed exploration coordination
    if swarm_context.exploration_pressure > 0.5 {
        let exploration_alignment = (angle.cos() * swarm_context.least_explored_direction.cos() + 
                                   angle.sin() * swarm_context.least_explored_direction.sin());
        if exploration_alignment > 0.4 {
            bonus += 0.6 * swarm_context.exploration_pressure; // Bonus for systematic exploration
        }
    }
    
    bonus
}

// ENHANCED NEST PHEROMONE FOLLOWING: Advanced trail detection structure
#[derive(Clone)]
struct NestTrailResult {
    found_trail: bool,
    direction: f32,
    strength: f32,
    confidence: f32,
    gradient_quality: f32,
}

// ENHANCED NEST PHEROMONE FOLLOWING: Advanced gradient-based nest trail detection
fn find_best_nest_trail_direction(
    x: f32, y: f32,
    grid: &PheromoneGrid,
    current_direction: f32,
    successful_deliveries: u32
) -> NestTrailResult {
    let mut result = NestTrailResult {
        found_trail: false,
        direction: current_direction,
        strength: 0.0,
        confidence: 0.0,
        gradient_quality: 0.0,
    };
    
    // Enhanced sampling: 16 directions for higher precision
    let num_directions = 16;
    let mut best_score = 0.0;
    let mut direction_scores = Vec::new();
    
    // Sample nest pheromones in multiple ranges for gradient analysis
    for i in 0..num_directions {
        let angle = (i as f32) * std::f32::consts::TAU / num_directions as f32;
        
        // Multi-range sampling for gradient detection
        let near_sample_x = x + angle.cos() * 12.0;
        let near_sample_y = y + angle.sin() * 12.0;
        let mid_sample_x = x + angle.cos() * 20.0;
        let mid_sample_y = y + angle.sin() * 20.0;
        let far_sample_x = x + angle.cos() * 30.0;
        let far_sample_y = y + angle.sin() * 30.0;
        
        let near_strength = grid.sample_directional(near_sample_x, near_sample_y, angle, 6.0, PheromoneType::Nest);
        let mid_strength = grid.sample_directional(mid_sample_x, mid_sample_y, angle, 6.0, PheromoneType::Nest);
        let far_strength = grid.sample_directional(far_sample_x, far_sample_y, angle, 6.0, PheromoneType::Nest);
        
        // Calculate gradient strength (positive = getting stronger toward nest)
        let gradient = (near_strength - far_strength) * 2.0; // Amplify gradient signal
        let average_strength = (near_strength + mid_strength + far_strength) / 3.0;
        
        // Momentum bonus: prefer directions close to current direction
        let angle_diff = (angle - current_direction).abs();
        let angle_diff_normalized = if angle_diff > std::f32::consts::PI {
            std::f32::consts::TAU - angle_diff
        } else {
            angle_diff
        };
        let momentum_bonus = (1.0 - angle_diff_normalized / std::f32::consts::PI) * 0.3;
        
        // Experience bonus: experienced ants can follow weaker trails
        let experience_bonus = if successful_deliveries > 2 {
            0.2 // Experienced ants get bonus for subtle trails
        } else {
            0.0
        };
        
        // Combined score: gradient + average strength + momentum + experience
        let trail_score = gradient * 2.0 + average_strength * 1.5 + momentum_bonus + experience_bonus;
        
        direction_scores.push((angle, trail_score, average_strength, gradient.abs()));
        
        if trail_score > best_score && average_strength > 0.2 { // Higher threshold for nest trails
            best_score = trail_score;
            result.direction = angle;
            result.strength = average_strength;
            result.gradient_quality = gradient.abs();
            result.found_trail = true;
        }
    }
    
    if result.found_trail {
        // Calculate confidence based on how much better this direction is vs alternatives
        let second_best_score = direction_scores.iter()
            .map(|(_, score, _, _)| *score)
            .filter(|&score| score < best_score)
            .fold(0.0f32, f32::max);
        
        let score_advantage = best_score - second_best_score;
        result.confidence = (score_advantage / (best_score + 0.1)).min(1.0);
        
        // Boost confidence for strong gradients
        if result.gradient_quality > 0.5 {
            result.confidence = (result.confidence + 0.3).min(1.0);
        }
        
        // Distance to nest boost: closer to nest = higher confidence in nest trails
        let distance_to_nest = Vec2::new(x, y).length();
        if distance_to_nest < 200.0 {
            result.confidence = (result.confidence + 0.2).min(1.0);
        }
    }
    
    result
}