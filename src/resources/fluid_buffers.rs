use bevy::{prelude::*, render::extract_resource::ExtractResource};

#[derive(Resource, Default, Clone, ExtractResource)]
pub struct FluidBuffer {
    pub velocities: Vec<Vec2>,
    pub positions: Vec<Vec2>,
    pub region_indices: Vec<usize>,
    pub densities: Vec<(f32, f32)>,
    pub predicted_positions: Vec<Vec2>,

    pub spatial_indices: Vec<usize>,
    pub spatial_offsets: Vec<usize>,
    pub spatial_keys: Vec<usize>,

    pub sort_target_positions: Vec<Vec2>,
    pub sort_target_velocities: Vec<Vec2>,
    pub sort_target_predicted_positions: Vec<Vec2>,

    pub colors: Vec<LinearRgba>,
}

#[derive(Component)]
pub struct ParticleIndex(pub usize);

impl FluidBuffer {
    pub fn new(num_particles: usize) -> Self {
        Self {
            velocities: vec![Vec2::ZERO; num_particles],
            positions: vec![Vec2::ZERO; num_particles],
            region_indices: vec![0; num_particles],
            densities: vec![(0.0, 0.0); num_particles],
            predicted_positions: vec![Vec2::ZERO; num_particles],

            spatial_indices: vec![0; num_particles],
            spatial_offsets: vec![0; num_particles],
            spatial_keys: vec![0; num_particles],

            sort_target_positions: vec![Vec2::ZERO; num_particles],
            sort_target_velocities: vec![Vec2::ZERO; num_particles],
            sort_target_predicted_positions: vec![Vec2::ZERO; num_particles],

            colors: vec![default(); num_particles],
        }
    }

    pub fn initialise_spatial_buffers(&mut self, size: usize) {
        self.spatial_indices = vec![0; size];
        self.spatial_offsets = vec![0; size];
        self.spatial_keys = vec![0; size];
        self.sort_target_positions = vec![Vec2::ZERO; size];
        self.sort_target_velocities = vec![Vec2::ZERO; size];
        self.sort_target_predicted_positions = vec![Vec2::ZERO; size];
        self.colors = vec![default(); size];
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.densities.clear();
        self.predicted_positions.clear();
        self.region_indices.clear();
    }
}
