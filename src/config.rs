use bevy::prelude::*;

#[derive(Resource)]
pub struct SimConfig {
    pub world_size: usize,
    pub initial_ants: usize,
    pub food_sources: usize,
    
    // Pheromone parameters
    pub evap_food: f32,
    pub evap_nest: f32,
    pub evap_alarm: f32,
    pub diff_food: f32,
    pub diff_nest: f32,
    pub diff_alarm: f32,
    
    // Ant behavior parameters  
    pub base_exploration_noise: f32,
    pub follow_gain: f32,
    pub lay_rate_food: f32,
    pub lay_rate_nest: f32,
    pub food_quality_weight: f32,
    pub detection_threshold: f32,
    pub saturation_limit: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            world_size: 1000,
            initial_ants: 50,       // Back to original 50 ants  
            food_sources: 10,       // Back to original 10 food sources - no cheating!
            
            evap_food: 0.00012,     // Slightly slower decay for longer-lasting trails
            evap_nest: 0.0005,      // REVERTED: Slower nest decay - provides essential navigation 
            evap_alarm: 0.01,
            diff_food: 0.12,        // Increased diffusion for even easier trail detection  
            diff_nest: 0.05,        // OPTIMIZATION 2: More nest diffusion
            diff_alarm: 0.2,
            
            base_exploration_noise: 0.03,    // Further reduced noise for very focused movement
            follow_gain: 2.5,    // Increased sensitivity for pheromone detection
            lay_rate_food: 40.0,    // OPTIMIZATION 3: Doubled initial pheromone deposition
            lay_rate_nest: 50.0,    // OPTIMIZATION 3: Doubled nest pheromone deposition
            food_quality_weight: 1.0,
            detection_threshold: 0.001,  // Back to Generation 37 value
            saturation_limit: 10.0,
        }
    }
}