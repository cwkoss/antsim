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
            
            // STARTUP PERIOD: Basic exploration before intelligent behavior
            if ant.startup_timer > 0.0 {
                // Continue basic exploration during startup period
                if ant.sensing_timer <= 0.0 {
                    // Random direction changes during startup
                    if rand::random::<f32>() < 0.05 * delta_time { // 5% chance per second
                        let angle_change = (rand::random::<f32>() - 0.5) * 0.8;
                        ant.current_direction += angle_change;
                        velocity.x = ant.current_direction.cos() * config.ant_speed;
                        velocity.y = ant.current_direction.sin() * config.ant_speed;
                    }
                    ant.sensing_timer = 0.5; // Sense every 0.5 seconds
                }
                continue; // Skip intelligent behavior during startup
            }
            
            match ant.behavior_state {
                AntBehaviorState::Exploring => {
                    // Original exploration behavior
                    if ant.sensing_timer <= 0.0 {
                        // Random direction changes
                        if rand::random::<f32>() < 0.1 * delta_time { // 10% chance per second
                            let angle_change = (rand::random::<f32>() - 0.5) * 1.0;
                            ant.current_direction += angle_change;
                            velocity.x = ant.current_direction.cos() * config.ant_speed;
                            velocity.y = ant.current_direction.sin() * config.ant_speed;
                        }
                        ant.sensing_timer = 1.5 + rand::random::<f32>() * 0.5; // 1.5-2.0 seconds
                    }
                },
                
                AntBehaviorState::Sensing => {
                    // Simple sensing system that was working
                    ant.behavior_state = AntBehaviorState::Exploring;
                    ant.sensing_timer = 1.5 + rand::random::<f32>() * 0.5;
                },
                
                AntBehaviorState::Following => {
                    // Simple following behavior
                    ant.behavior_state = AntBehaviorState::Exploring;
                    ant.sensing_timer = 1.5 + rand::random::<f32>() * 0.5;
                },
                
                AntBehaviorState::Tracking => {
                    // Simple tracking behavior
                    ant.behavior_state = AntBehaviorState::Exploring;
                    ant.sensing_timer = 1.5 + rand::random::<f32>() * 0.5;
                }
            }
            
            // Update current direction based on velocity
            if velocity.x.abs() > 0.1 || velocity.y.abs() > 0.1 {
                ant.current_direction = velocity.y.atan2(velocity.x);
            }
        }
    }
}

// Rest of the systems remain unchanged from the original working version
pub fn movement_system(
    mut ants: Query<(&mut Transform, &Velocity, &AntState)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    
    for (mut transform, velocity, _ant_state) in ants.iter_mut() {
        transform.translation.x += velocity.x * delta_time;
        transform.translation.y += velocity.y * delta_time;
        
        // Keep ants within world bounds (with some padding)
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

// All other system functions remain the same...
// Just keep all the working systems from the original