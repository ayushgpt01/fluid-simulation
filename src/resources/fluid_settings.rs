use bevy::{
    ecs::{reflect::ReflectResource, resource::Resource},
    reflect::Reflect,
    render::{extract_resource::ExtractResource, render_resource::ShaderType},
};
use bevy_inspector_egui::{InspectorOptions, inspector_options::ReflectInspectorOptions};

pub const DEFAULT_REGION_HEIGHT: f32 = 600.0;
pub const DEFAULT_REGION_WIDTH: f32 = 900.0;

#[derive(Reflect, Resource, Clone, Copy, InspectorOptions, ExtractResource, ShaderType)]
#[reflect(Resource, InspectorOptions)]
pub struct FluidSettings {
    pub thickness: f32,
    pub number_of_particles: u32,
    pub pixels_per_meter: f32,
    pub gravity: f32,
    pub collision_dampening: f32,
    pub region_height: f32,
    pub region_width: f32,
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub near_pressure_multiplier: f32,
    pub viscosity_strength: f32,
    pub max_speed: f32,
    pub delta_time: f32,
}

impl Default for FluidSettings {
    fn default() -> Self {
        Self {
            thickness: 2.0,
            pixels_per_meter: 50.0,
            max_speed: 15.0,

            number_of_particles: 2000,
            gravity: 9.81,
            collision_dampening: 0.5,

            region_height: DEFAULT_REGION_HEIGHT,
            region_width: DEFAULT_REGION_WIDTH,

            smoothing_radius: 0.45,
            target_density: 4.5,

            pressure_multiplier: 40.0,
            near_pressure_multiplier: 18.0,

            viscosity_strength: 0.15,

            delta_time: 0.0,
        }
    }
}

impl FluidSettings {
    pub fn new() -> Self {
        Self {
            thickness: 2.0,
            pixels_per_meter: 50.0,
            max_speed: 15.0,

            number_of_particles: 2000,
            gravity: 9.81,
            collision_dampening: 0.5,

            region_height: DEFAULT_REGION_HEIGHT,
            region_width: DEFAULT_REGION_WIDTH,

            smoothing_radius: 0.45,
            target_density: 4.5,

            pressure_multiplier: 40.0,
            near_pressure_multiplier: 18.0,

            viscosity_strength: 0.15,

            delta_time: 0.0,
        }
    }
}
