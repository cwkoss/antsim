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
            
            evap_food: 0.0002,     // GENERATION 79: Revert to successful Generation 79 base settings
            evap_nest: 0.0005,      // Back to Generation 54 successful value
            evap_alarm: 0.01,
            diff_food: 0.15,        // GENERATION 79: Revert to successful Generation 79 base
            diff_nest: 0.05,        // Back to Generation 54 successful value
            diff_alarm: 0.2,
            
            base_exploration_noise: 0.02,    // GENERATION 79: Revert to successful Generation 79 base
            follow_gain: 3.5,       // GENERATION 79: Revert to successful Generation 79 base
            lay_rate_food: 42.0,    // CYCLE 5: Slightly increased trail deposition
            lay_rate_nest: 25.0,    // NEST PHEROMONE FIX: Strong nest trails from successful food carriers
            food_quality_weight: 1.0,
            detection_threshold: 0.0008,  // CYCLE 3: Revert to Gen 79 base
            saturation_limit: 10.0,    // GENERATION 75: Revert to optimal saturation level
        }
    }
}