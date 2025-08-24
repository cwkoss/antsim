use bevy::prelude::*;

mod components;
mod systems;
mod pheromones;
mod config;
mod video;
mod png_test;
mod create_test_frames;
mod colors;

use components::*;
use systems::*;
use config::*;
use pheromones::*;
use video::*;
use png_test::*;
use create_test_frames::*;
use colors::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::window::WindowPlugin {
            primary_window: Some(bevy::window::Window {
                title: "Ant Simulation".into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        }))
        .insert_resource(SimConfig::default())
        .insert_resource(PheromoneGrid::new(1000, 1000)) // 1:1 with world grid
        .insert_resource(DebugInfo::default())
        .insert_resource(PerformanceTracker::default())
        .insert_resource(VideoRecorder::default())
        .insert_resource(ColorConfig::default())
        .insert_resource(GenerationInfo::from_json_file())
        .add_systems(Startup, (setup, setup_pheromone_visualization, setup_debug_ui, setup_video_camera))
        .add_systems(
            Update,
            (
                sensing_system,
                movement_system,
                pheromone_deposit_system,
                pheromone_update_system,
                food_collection_system,
                ant_visual_system,
                food_visual_system,
                update_pheromone_visualization,
                exit_system,
                exit_event_listener,
                window_close_system,
                restart_system,
                camera_control_system,
                cursor_tracking_system,
                hover_detection_system,
                ant_selection_system,
                selected_ant_display_system,
                selected_ant_outline_system,
                performance_analysis_system,
                update_debug_ui,
            ).chain()
        )
        .add_systems(Update, video_recording_system)
        .run();
}

fn setup(mut commands: Commands, config: Res<SimConfig>, color_config: Res<ColorConfig>) {
    commands.spawn(Camera2dBundle::default());
    
    // Add debug text to verify rendering
    commands.spawn(TextBundle::from_section(
        "Ant Simulation\nRed: Exploring  Yellow: Collecting  Orange: Carrying\nWASD: Move  Wheel: Zoom  R: Restart  ESC: Exit",
        TextStyle {
            font_size: 24.0,
            color: color_config.text,
            ..default()
        },
    ).with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        left: Val::Px(10.0),
        ..default()
    }));
    
    // Spawn nest at center
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: color_config.nest,
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 5.0),
            ..default()
        },
        Nest { capacity: 10000.0 },
    ));
    
    // Spawn initial ants around nest
    for i in 0..config.initial_ants {
        let angle = (i as f32) * std::f32::consts::TAU / config.initial_ants as f32;
        let x = angle.cos() * 50.0;
        let y = angle.sin() * 50.0;
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: color_config.ant_exploring,
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
                sensing_timer: rand::random::<f32>() * 2.0, // Random initial sensing delay
                current_direction: angle,
                trail_strength: 0.0,
                momentum_timer: 0.0,
                last_position: Vec2::new(x, y),
                stuck_timer: 0.0,
                direction_changes: 0,
                last_sensing_result: [0.0; 8],
                trail_memory: [angle; 5], // Initialize with current direction
                memory_index: 0,
                trail_quality: 0.0,
                hysteresis_threshold: config.detection_threshold,
                consecutive_good_trail_time: 0.0,
                food_pickup_time: 0.0,
                delivery_attempts: 0,
                successful_deliveries: 0,
                startup_timer: 5.0, // OPTIMIZATION 4: Further reduced to 5s for even faster food seeking
                has_found_food: false, // Track if ant has ever found food
                food_carry_start_time: 0.0, // When ant picked up food
                last_goal_achievement_time: 0.0, // Initialize as never achieved a goal
                current_goal_start_time: 0.0, // Will be set when startup timer expires
            },
            Velocity {
                x: (rand::random::<f32>() * 2.0 - 1.0) * 1.5,
                y: (rand::random::<f32>() * 2.0 - 1.0) * 1.5,
            },
        ));
    }
    
    // CHALLENGE MODE: All food sources FAR from nest (minimum 1/3 world size away)
    for _i in 0..config.food_sources {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        // Minimum distance = 1/3 world size = 333 units from nest
        // Maximum distance = 1/2 world size = 500 units from nest  
        let distance = 333.0 + rand::random::<f32>() * 167.0; // 333-500 units away
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;
        
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
            FoodSource { amount: 100.0, max_amount: 100.0 }, // Back to original food amount
        ));
    }
}