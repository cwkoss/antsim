use bevy::prelude::*;
use bevy::window::{WindowCloseRequested, PrimaryWindow};
use rand::Rng;
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
            ant.startup_timer -= delta_time;
            
            // Don't process ants that are collecting food or still in startup
            if ant.food_collection_timer > 0.0 || ant.startup_timer > 0.0 {
                continue;
            }
            
            // For carrying food: use simple nest-seeking behavior to avoid pheromone confusion
            if ant.carrying_food {
                // Simple direct navigation to nest when carrying food
                let nest_pos = Vec2::ZERO; // Nest is at origin
                let to_nest = nest_pos - Vec2::new(pos.x, pos.y);
                let distance_to_nest = to_nest.length();
                
                if distance_to_nest > 10.0 {
                    let nest_direction = to_nest.normalize();
                    ant.current_direction = nest_direction.y.atan2(nest_direction.x);
                    set_ant_velocity_from_vector(&mut velocity, nest_direction, MovementType::CarryingFood);
                    ant.behavior_state = AntBehaviorState::Following;
                }
                ant.sensing_timer = 0.3; // Faster sensing for food-carrying ants returning to nest
            } else {
                // For exploring ants: follow FOOD pheromones (trails left by successful ants who found food)
                let pheromone_readings = grid.sample_all_directions(pos.x, pos.y, PheromoneType::Food);
                let mut best_direction = ant.current_direction;
                let mut max_pheromone = 0.0;
                let mut found_trail = false;
                
                // Look for directional pheromone gradients instead of just strength
                let current_pheromone = grid.sample_all_directions(pos.x, pos.y, PheromoneType::Food)[0]; // Sample at current position
                
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
                        let momentum_bonus = (1.0 - angle_diff_normalized / std::f32::consts::PI) * 0.8; // Strong momentum to reduce path bouncing
                        
                        // Additional persistence bonus if ant has been following trails successfully
                        let persistence_bonus = if ant.behavior_state == AntBehaviorState::Following {
                            0.2 // Extra bias to continue following if already on a trail
                        } else {
                            0.0
                        };
                        
                        // Add gradient bonus for food pheromones - follow trails that lead toward food
                        let gradient_bonus = if pheromone_strength > current_pheromone + 0.05 {
                            0.3 // Bonus for following stronger food pheromone (toward food sources)
                        } else if pheromone_strength < current_pheromone - 0.05 {
                            -0.2 // Strict penalty - abandon declining trails
                        } else {
                            0.0 // Neutral if pheromone strength is similar
                        };
                        
                        let effective_strength = pheromone_strength + momentum_bonus + gradient_bonus + persistence_bonus;
                        
                        if effective_strength > max_pheromone {
                            max_pheromone = effective_strength;
                            best_direction = angle;
                            found_trail = true;
                        }
                    }
                }
                
                if found_trail && max_pheromone > 0.2 {
                    // Smooth direction change for trail following
                    ant.behavior_state = AntBehaviorState::Following;
                    let angle_diff = best_direction - ant.current_direction;
                    let smooth_angle_change = if angle_diff.abs() > std::f32::consts::PI {
                        if angle_diff > 0.0 { angle_diff - std::f32::consts::TAU } else { angle_diff + std::f32::consts::TAU }
                    } else { angle_diff };
                    
                    // More gradual direction adjustment to stick to paths better
                    ant.current_direction += smooth_angle_change * 0.25; // Reduced from 0.4 for path stability
                    set_ant_velocity(&mut velocity, ant.current_direction, MovementType::FollowingTrail);
                    
                    // Longer sensing intervals on strong trails to maintain momentum
                    let trail_strength_factor = (max_pheromone - 0.2).max(0.0) / 0.8; // Normalize 0.2-1.0 to 0.0-1.0
                    ant.sensing_timer = if max_pheromone > 0.4 {
                        0.5 + trail_strength_factor * 0.3 // 0.5-0.8s for strong trails - maintain momentum
                    } else {
                        0.2 + trail_strength_factor * 0.1 // 0.2-0.3s for weak trails - stay responsive
                    };
                } else {
                    // No trail found - random exploration
                    ant.behavior_state = AntBehaviorState::Exploring;
                    
                    if ant.sensing_timer <= 0.0 {
                        // Adaptive exploration: more aggressive as search time increases
                        let search_time = if ant.last_goal_achievement_time > 0.0 {
                            time.elapsed_seconds() - ant.last_goal_achievement_time
                        } else {
                            time.elapsed_seconds() - ant.startup_timer.max(0.0)
                        };
                        
                        // Increase exploration aggressiveness with search time (up to 60s max)
                        let exploration_factor = (search_time / 60.0).min(1.0); // 0.0 to 1.0 over 60 seconds
                        let base_angle = 1.2;
                        let max_angle = 2.2;
                        let angle_range = base_angle + (max_angle - base_angle) * exploration_factor;
                        
                        let angle_change = (rand::random::<f32>() - 0.5) * angle_range;
                        ant.current_direction += angle_change;
                        set_ant_velocity(&mut velocity, ant.current_direction, MovementType::Exploring);
                        
                        // Faster sensing for longer-searching ants
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
    }
}

pub fn movement_system(
    mut ants: Query<(&mut Transform, &Velocity, &AntState)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    
    for (mut transform, velocity, _ant_state) in ants.iter_mut() {
        transform.translation.x += velocity.x * delta_time;
        transform.translation.y += velocity.y * delta_time;
        
        // Keep ants within world bounds
        let bound = 480.0;
        if transform.translation.x > bound {
            transform.translation.x = bound;
        } else if transform.translation.x < -bound {
            transform.translation.x = -bound;
        }
        
        if transform.translation.y > bound {
            transform.translation.y = bound;
        } else if transform.translation.y < -bound {
            transform.translation.y = -bound;
        }
    }
}

pub fn pheromone_deposit_system(
    ants: Query<(&Transform, &AntState)>,
    mut pheromone_grid: Option<ResMut<PheromoneGrid>>,
    config: Res<SimConfig>,
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
                        // Lay food trail when returning to nest
                        let decay_factor = (-ant.distance_from_food * 0.01).exp(); // Balanced distance decay rate
                        let deposit_amount = config.lay_rate_food * config.food_quality_weight * decay_factor;
                        
                        grid.deposit(
                            deposit_pos.x, 
                            deposit_pos.y, 
                            PheromoneType::Food, 
                            deposit_amount / (num_deposits + 1) as f32 // Distribute amount across deposits
                        );
                        
                    } else {
                        // Lay nest trail when exploring
                        let decay_factor = (-ant.distance_from_nest * 0.003).exp();
                        let deposit_amount = config.lay_rate_nest * decay_factor;
                        
                        grid.deposit(
                            deposit_pos.x, 
                            deposit_pos.y, 
                            PheromoneType::Nest, 
                            deposit_amount / (num_deposits + 1) as f32 // Distribute amount across deposits
                        );
                    }
                }
            } else {
                // For very small movements, just deposit at current position
                if ant.carrying_food {
                    let decay_factor = (-ant.distance_from_food * 0.005).exp();
                    let deposit_amount = config.lay_rate_food * config.food_quality_weight * decay_factor;
                    
                    grid.deposit(current_pos.x, current_pos.y, PheromoneType::Food, deposit_amount);
                } else {
                    let decay_factor = (-ant.distance_from_nest * 0.003).exp();
                    let deposit_amount = config.lay_rate_nest * decay_factor;
                    
                    grid.deposit(current_pos.x, current_pos.y, PheromoneType::Nest, deposit_amount);
                }
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
    let nest_pos = if let Ok(nest_transform) = nests.get_single() {
        nest_transform.translation
    } else {
        Vec3::ZERO
    };
    
    for (ant_transform, mut ant, mut velocity) in ants.iter_mut() {
        let ant_pos = ant_transform.translation;
        
        if !ant.carrying_food && ant.food_collection_timer <= 0.0 {
            // Look for food sources
            for (food_transform, food) in food_sources.iter() {
                let food_pos = food_transform.translation;
                let distance = ant_pos.distance(food_pos);
                
                if distance < 25.0 && food.amount > 0.0 {
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
                    
                    if distance < 25.0 && food.amount > 0.0 {
                        let take_amount = 1.0;
                        food.amount -= take_amount;
                        ant.carrying_food = true;
                        ant.food_pickup_time = time.elapsed_seconds();
                        ant.has_found_food = true;
                        ant.food_carry_start_time = time.elapsed_seconds();
                        ant.last_goal_achievement_time = time.elapsed_seconds();
                        performance_tracker.total_food_collected += take_amount;
                        
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
            
            if distance < 40.0 {
                // Successful delivery
                ant.carrying_food = false;
                ant.delivery_attempts += 1;
                ant.successful_deliveries += 1;
                ant.last_goal_achievement_time = time.elapsed_seconds();
                
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
                
                
                // Start exploring again
                ant.behavior_state = AntBehaviorState::Exploring;
                ant.sensing_timer = 0.3;
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
            (runtime - ant.startup_timer).max(0.0)
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
    
    if lost_food_carriers_count >= 10 {
        println!("\n🚨 AUTO-EXIT: Too many lost food carriers ({})", lost_food_carriers_count);
        exit_writer.send(AppExit::Success);
    }
    
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
                        let intensity = (food_strength.ln() / 3.0).clamp(0.0, 1.0);
                        let base_color = color_config.food_pheromone;
                        sprite.color = Color::srgba(
                            base_color.to_srgba().red,
                            base_color.to_srgba().green,
                            base_color.to_srgba().blue,
                            intensity
                        );
                        transform.translation.z = -9.0;
                    } else {
                        let intensity = (nest_strength.ln() / 3.0).clamp(0.0, 1.0);
                        let base_color = color_config.nest_pheromone;
                        sprite.color = Color::srgba(
                            base_color.to_srgba().red,
                            base_color.to_srgba().green,
                            base_color.to_srgba().blue,
                            intensity
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