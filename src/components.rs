use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct DebugInfo {
    pub cursor_world_pos: Vec2,
    pub hovered_entity: Option<Entity>,
    pub selected_entity: Option<Entity>,
    pub pheromone_info: String,
    pub entity_info: String,
}

#[derive(Resource)]
pub struct PerformanceTracker {
    pub successful_deliveries: u32,
    pub failed_attempts: u32,
    pub total_food_collected: f32,
    pub average_delivery_time: f32,
    pub delivery_times: Vec<f32>,
    pub simulation_start_time: f32,
    pub last_delivery_time: f32,
    pub stuck_ants_count: u32,
    pub oscillating_ants_count: u32,
    pub lost_ants_count: u32, // Ants that never found food
    pub lost_food_carriers_count: u32, // Ants carrying food but lost (can't find nest)
    pub average_return_time: f32, // Average time for food-carrying ants to return to nest
    pub return_times: Vec<f32>, // Track individual return times
    pub average_time_since_goal: f32, // NEW METRIC: Average time since each ant reached its goal
    pub time_since_goal_samples: Vec<f32>, // Individual ant time-since-goal samples for this frame
}

#[derive(Resource)]
pub struct VideoRecorder {
    pub frames: Vec<Vec<u8>>, // Store RGBA frames
    pub is_recording: bool,
    pub frame_width: u32,
    pub frame_height: u32,
    pub max_frames: usize, // 30 seconds worth at ~60fps = 1800 frames
    pub test_number: u32,
    pub changes_description: String,
    pub frame_timer: f32, // Timer for frame capture interval
    pub frame_interval: f32, // How often to capture frames (in seconds)
}

impl Default for VideoRecorder {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            is_recording: false,
            frame_width: 406,
            frame_height: 720,
            max_frames: 3600, // Extended for longer, more detailed videos
            test_number: 1,
            changes_description: "Default configuration".to_string(),
            frame_timer: 0.0,
            frame_interval: 0.2, // Capture every 0.2 seconds to get exactly 450 frames over 90s (90/450=0.2)
        }
    }
}

#[derive(Resource)]
pub struct GenerationInfo {
    pub current_generation: u32,
    pub description: String,
    pub timestamp: String,
    pub video_filename: String,
}

impl Default for GenerationInfo {
    fn default() -> Self {
        Self {
            current_generation: 1,
            description: "Initial implementation".to_string(),
            timestamp: "2025-08-24".to_string(),
            video_filename: "0001_initial.mp4".to_string(),
        }
    }
}

impl GenerationInfo {
    pub fn from_json_file() -> Self {
        use std::fs;
        
        let json_content = match fs::read_to_string("generation_info.json") {
            Ok(content) => content,
            Err(_) => return GenerationInfo::default(), // Fallback to default if file not found
        };
        
        // Simple JSON parsing for the fields we need
        let mut generation = 1;
        let mut description = "Initial implementation".to_string();
        let mut timestamp = "2025-08-24".to_string();
        let mut video_filename = "0001_initial.mp4".to_string();
        
        // Basic parsing - look for the fields we need
        for line in json_content.lines() {
            let line = line.trim();
            if line.starts_with("\"current_generation\":") {
                if let Some(value) = line.split(':').nth(1) {
                    let value = value.trim().trim_end_matches(',');
                    generation = value.parse().unwrap_or(1);
                }
            } else if line.starts_with("\"description\":") {
                if let Some(value) = line.split(':').nth(1) {
                    let value = value.trim().trim_start_matches('"').trim_end_matches("\",");
                    description = value.to_string();
                }
            } else if line.starts_with("\"video_filename\":") {
                if let Some(value) = line.split(':').nth(1) {
                    let value = value.trim().trim_start_matches('"').trim_end_matches("\",");
                    video_filename = value.to_string();
                }
            }
        }
        
        Self {
            current_generation: generation,
            description,
            timestamp,
            video_filename,
        }
    }
}

// Removed duplicate Default implementation - using the one above

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self {
            successful_deliveries: 0,
            failed_attempts: 0,
            total_food_collected: 0.0,
            average_delivery_time: 0.0,
            delivery_times: Vec::new(),
            simulation_start_time: 0.0,
            last_delivery_time: 0.0,
            stuck_ants_count: 0,
            oscillating_ants_count: 0,
            lost_ants_count: 0,
            lost_food_carriers_count: 0,
            average_return_time: 0.0,
            return_times: Vec::new(),
            average_time_since_goal: 0.0,
            time_since_goal_samples: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct PheromoneDebugText;

#[derive(Component)]
pub struct EntityDebugText;

#[derive(Component)]
pub struct PerformanceText;

#[derive(Component)]
pub struct SelectedAnt;

#[derive(Component)]
pub struct AntOutline;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AntBehaviorState {
    Exploring,  // Random walk with occasional sensing
    Sensing,    // Paused and sampling all directions
    Following,  // Moving toward strongest pheromone gradient
    Tracking,   // Continuing in current direction while monitoring
}

#[derive(Component)]
pub struct AntState {
    pub carrying_food: bool,
    pub hunger: f32,
    pub sensitivity_adapt: f32,
    pub food_collection_timer: f32, // Time spent collecting food
    pub last_pheromone_strength: f32, // Track pheromone strength from last frame
    pub distance_from_food: f32, // Track distance traveled since picking up food trail
    pub distance_from_nest: f32, // Track distance traveled from nest (for nest pheromone strength)
    pub has_exit_direction: bool, // Track if ant has already chosen an exit direction
    pub behavior_state: AntBehaviorState,
    pub sensing_timer: f32, // Time until next sensing check
    pub current_direction: f32, // Current movement direction in radians
    pub trail_strength: f32, // Strength of trail being followed
    pub momentum_timer: f32, // Time to continue in current direction
    pub last_position: Vec2, // Previous position for stuck detection
    pub stuck_timer: f32, // Time spent in same general area
    pub direction_changes: u32, // Count of recent direction changes
    pub last_sensing_result: [f32; 8], // Results from last 8-direction sensing
    pub trail_memory: [f32; 5], // Recent trail directions (rolling buffer)
    pub memory_index: usize, // Current position in trail memory buffer
    pub trail_quality: f32, // Assessment of current trail quality (consistency)
    pub hysteresis_threshold: f32, // Dynamic threshold for direction changes
    pub consecutive_good_trail_time: f32, // Time spent on consistent, strong trails
    pub food_pickup_time: f32, // When food was picked up (for delivery time tracking)
    pub delivery_attempts: u32, // Number of times this ant has attempted delivery
    pub successful_deliveries: u32, // Number of successful deliveries by this ant
    pub startup_timer: f32, // Grace period for simple behavior after spawning
    pub has_found_food: bool, // Track if ant has ever found food to identify "lost" ants
    pub food_carry_start_time: f32, // When ant picked up food (for return time tracking)
    pub last_goal_achievement_time: f32, // When ant last reached a goal (found food or delivered to nest)
    pub current_goal_start_time: f32, // When ant started pursuing current goal
    
    // New diagnostic fields for behavior analysis
    pub can_see_trail: bool, // Whether ant can detect any pheromone trail nearby
    pub distance_from_trail: f32, // Distance to nearest significant pheromone concentration
    pub trail_following_time: f32, // How long ant has been following current trail
    pub last_trail_contact_time: f32, // When ant last detected significant pheromone
    pub is_swarming: bool, // Whether ant is stuck in traffic with other ants
    pub nearby_ant_count: u32, // Number of ants within close proximity
    pub time_since_progress: f32, // Time since ant made meaningful progress toward goal
    pub exploration_efficiency: f32, // Ratio of distance covered vs time spent exploring
    pub is_edge_wanderer: bool, // Whether ant is stuck wandering world edges
    pub world_edge_proximity: f32, // Distance from nearest world edge
    pub trail_gradient_strength: f32, // Strength of pheromone gradient at current position
}

#[derive(Component)]
pub struct DebugAnt {
    pub ant_id: u32,
}

#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct FoodSource {
    pub amount: f32,
    pub max_amount: f32,
}

#[derive(Component)]
pub struct Nest {
    pub capacity: f32,
}

#[derive(Component)]
pub struct PheromoneData {
    pub food_trail: f32,
    pub nest_trail: f32,
    pub alarm: f32,
}

impl Default for PheromoneData {
    fn default() -> Self {
        Self {
            food_trail: 0.0,
            nest_trail: 0.0,
            alarm: 0.0,
        }
    }
}

#[derive(Component)]
pub struct PheromoneVisualization {
    pub grid_x: usize,
    pub grid_y: usize,
}

#[derive(Component)]
pub struct Rock {
    pub radius: f32,
}

#[derive(Resource)]
pub struct ChallengeConfig {
    pub challenge_number: u32,
}

impl Default for ChallengeConfig {
    fn default() -> Self {
        Self {
            challenge_number: 1,
        }
    }
}