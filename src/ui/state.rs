pub struct UiState {
    pub paused: bool,
    pub time_scale: f32,

    pub show_trails: bool,
    pub show_octree: bool,
    pub show_velocity_vectors: bool,

    pub theta: f32,
}

impl UiState {
    pub fn new() -> Self {
        Self {  
            // Simulation
            paused: false,
            time_scale: 1.0,
            theta: 1.0,
            
            // Visualization
            show_trails: true,
            show_octree: false,
            show_velocity_vectors: false,
        }
    }
}
