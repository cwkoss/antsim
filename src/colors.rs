use bevy::prelude::*;

/// Shared color configuration for consistent rendering across simulation and video
#[derive(Resource, Clone)]
pub struct ColorConfig {
    // Pheromone colors
    pub food_pheromone: Color,
    pub nest_pheromone: Color,
    pub alarm_pheromone: Color,
    
    // Entity colors
    pub nest: Color,
    pub food_source: Color,
    pub ant_exploring: Color,
    pub ant_carrying_food: Color,
    pub ant_collecting: Color,
    
    // UI colors
    pub text: Color,
    pub debug_selection: Color,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            // Pheromone colors - matching simulation render
            food_pheromone: Color::srgb(0.0, 1.0, 0.0),  // Green
            nest_pheromone: Color::srgb(0.0, 0.0, 1.0),  // Blue
            alarm_pheromone: Color::srgb(1.0, 0.0, 1.0), // Magenta
            
            // Entity colors - matching simulation render
            nest: Color::srgb(1.0, 1.0, 0.0),           // Yellow
            food_source: Color::srgb(0.0, 1.0, 0.0),    // Green
            ant_exploring: Color::srgb(1.0, 0.0, 0.0),  // Red
            ant_carrying_food: Color::srgb(1.0, 0.5, 0.0), // Orange
            ant_collecting: Color::srgb(1.0, 1.0, 0.0),    // Yellow
            
            // UI colors
            text: Color::WHITE,
            debug_selection: Color::srgb(1.0, 0.0, 1.0), // Pink/magenta
        }
    }
}

impl ColorConfig {
    /// Get pheromone color as RGB bytes for video rendering
    pub fn food_pheromone_rgb(&self) -> (u8, u8, u8) {
        let [r, g, b, _] = self.food_pheromone.to_srgba().to_u8_array();
        (r, g, b)
    }
    
    pub fn nest_pheromone_rgb(&self) -> (u8, u8, u8) {
        let [r, g, b, _] = self.nest_pheromone.to_srgba().to_u8_array();
        (r, g, b)
    }
    
    pub fn alarm_pheromone_rgb(&self) -> (u8, u8, u8) {
        let [r, g, b, _] = self.alarm_pheromone.to_srgba().to_u8_array();
        (r, g, b)
    }
    
    pub fn nest_rgb(&self) -> (u8, u8, u8) {
        let [r, g, b, _] = self.nest.to_srgba().to_u8_array();
        (r, g, b)
    }
    
    pub fn food_source_rgb(&self) -> (u8, u8, u8) {
        let [r, g, b, _] = self.food_source.to_srgba().to_u8_array();
        (r, g, b)
    }
    
    pub fn ant_exploring_rgb(&self) -> (u8, u8, u8) {
        let [r, g, b, _] = self.ant_exploring.to_srgba().to_u8_array();
        (r, g, b)
    }
    
    pub fn ant_carrying_food_rgb(&self) -> (u8, u8, u8) {
        let [r, g, b, _] = self.ant_carrying_food.to_srgba().to_u8_array();
        (r, g, b)
    }
    
    pub fn ant_collecting_rgb(&self) -> (u8, u8, u8) {
        let [r, g, b, _] = self.ant_collecting.to_srgba().to_u8_array();
        (r, g, b)
    }
}