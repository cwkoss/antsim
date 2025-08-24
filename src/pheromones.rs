use bevy::prelude::*;
use rayon::prelude::*;

#[derive(Resource)]
pub struct PheromoneGrid {
    pub width: usize,
    pub height: usize,
    pub food_trail: Vec<f32>,
    pub nest_trail: Vec<f32>,
    pub alarm: Vec<f32>,
    
    // Double buffer for updates
    food_trail_buffer: Vec<f32>,
    nest_trail_buffer: Vec<f32>,
    alarm_buffer: Vec<f32>,
}

impl PheromoneGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            food_trail: vec![0.0; size],
            nest_trail: vec![0.0; size],
            alarm: vec![0.0; size],
            food_trail_buffer: vec![0.0; size],
            nest_trail_buffer: vec![0.0; size],
            alarm_buffer: vec![0.0; size],
        }
    }
    
    pub fn world_to_grid(&self, x: f32, y: f32) -> Option<usize> {
        // Map world coordinates (-500 to +500) to grid coordinates (0 to 999)
        // 1:1 mapping - each world unit = one grid cell
        let world_size = 1000.0;
        
        let grid_x = (x + world_size * 0.5) as i32;
        let grid_y = (y + world_size * 0.5) as i32;
        
        if grid_x >= 0 && grid_x < self.width as i32 && grid_y >= 0 && grid_y < self.height as i32 {
            Some(grid_y as usize * self.width + grid_x as usize)
        } else {
            None
        }
    }
    
    pub fn sample_gradient(&self, x: f32, y: f32, pheromone_type: PheromoneType) -> (f32, f32, f32) {
        let data = match pheromone_type {
            PheromoneType::Food => &self.food_trail,
            PheromoneType::Nest => &self.nest_trail,
            PheromoneType::Alarm => &self.alarm,
        };
        
        if let Some(center) = self.world_to_grid(x, y) {
            // Sample in a wider radius for better gradient detection
            let sample_distance = 30.0;
            let left = self.world_to_grid(x - sample_distance, y).unwrap_or(center);
            let front = self.world_to_grid(x, y + sample_distance).unwrap_or(center);
            let right = self.world_to_grid(x + sample_distance, y).unwrap_or(center);
            
            (data[left], data[front], data[right])
        } else {
            (0.0, 0.0, 0.0)
        }
    }
    
    pub fn sample_directional(&self, x: f32, y: f32, direction: f32, distance: f32, pheromone_type: PheromoneType) -> f32 {
        let data = match pheromone_type {
            PheromoneType::Food => &self.food_trail,
            PheromoneType::Nest => &self.nest_trail,
            PheromoneType::Alarm => &self.alarm,
        };
        
        let sample_x = x + direction.cos() * distance;
        let sample_y = y + direction.sin() * distance;
        
        if let Some(idx) = self.world_to_grid(sample_x, sample_y) {
            // Sample a 3x3 area and average
            let mut total = 0.0;
            let mut count = 0;
            
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let grid_x = (sample_x + dx as f32) as i32;
                    let grid_y = (sample_y + dy as f32) as i32;
                    
                    if let Some(neighbor_idx) = self.world_to_grid(grid_x as f32, grid_y as f32) {
                        if neighbor_idx < data.len() {
                            total += data[neighbor_idx];
                            count += 1;
                        }
                    }
                }
            }
            
            if count > 0 { total / count as f32 } else { 0.0 }
        } else {
            0.0
        }
    }
    
    pub fn sample_all_directions(&self, x: f32, y: f32, pheromone_type: PheromoneType) -> [f32; 8] {
        let directions = [
            0.0,                    // North
            std::f32::consts::PI / 4.0,       // NE
            std::f32::consts::PI / 2.0,       // East
            3.0 * std::f32::consts::PI / 4.0, // SE
            std::f32::consts::PI,             // South
            5.0 * std::f32::consts::PI / 4.0, // SW
            3.0 * std::f32::consts::PI / 2.0, // West
            7.0 * std::f32::consts::PI / 4.0, // NW
        ];
        
        let sensing_distance = 25.0;
        let mut samples = [0.0; 8];
        
        for (i, &direction) in directions.iter().enumerate() {
            samples[i] = self.sample_directional(x, y, direction, sensing_distance, pheromone_type);
        }
        
        samples
    }
    
    pub fn deposit(&mut self, x: f32, y: f32, pheromone_type: PheromoneType, amount: f32) {
        if let Some(idx) = self.world_to_grid(x, y) {
            match pheromone_type {
                PheromoneType::Food => self.food_trail[idx] += amount,
                PheromoneType::Nest => self.nest_trail[idx] += amount,
                PheromoneType::Alarm => self.alarm[idx] += amount,
            }
        }
    }
    
    pub fn update(&mut self, evap_rates: (f32, f32, f32), diff_rates: (f32, f32, f32)) {
        // Evaporation - use parallel iterator directly on slices
        self.food_trail.par_iter_mut().for_each(|val| *val *= 1.0 - evap_rates.0);
        self.nest_trail.par_iter_mut().for_each(|val| *val *= 1.0 - evap_rates.1);
        self.alarm.par_iter_mut().for_each(|val| *val *= 1.0 - evap_rates.2);
        
        // Simple diffusion - copy to buffer, then average with neighbors
        self.food_trail_buffer.copy_from_slice(&self.food_trail);
        self.nest_trail_buffer.copy_from_slice(&self.nest_trail);
        self.alarm_buffer.copy_from_slice(&self.alarm);
        
        for y in 1..self.height-1 {
            for x in 1..self.width-1 {
                let idx = y * self.width + x;
                let neighbors = [
                    idx - self.width - 1, idx - self.width, idx - self.width + 1,
                    idx - 1,               idx,               idx + 1,
                    idx + self.width - 1,  idx + self.width,  idx + self.width + 1,
                ];
                
                let food_avg: f32 = neighbors.iter().map(|&i| self.food_trail_buffer[i]).sum::<f32>() / 9.0;
                let nest_avg: f32 = neighbors.iter().map(|&i| self.nest_trail_buffer[i]).sum::<f32>() / 9.0;
                let alarm_avg: f32 = neighbors.iter().map(|&i| self.alarm_buffer[i]).sum::<f32>() / 9.0;
                
                self.food_trail[idx] = self.food_trail[idx] * (1.0 - diff_rates.0) + food_avg * diff_rates.0;
                self.nest_trail[idx] = self.nest_trail[idx] * (1.0 - diff_rates.1) + nest_avg * diff_rates.1;
                self.alarm[idx] = self.alarm[idx] * (1.0 - diff_rates.2) + alarm_avg * diff_rates.2;
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum PheromoneType {
    Food,
    Nest,
    Alarm,
}