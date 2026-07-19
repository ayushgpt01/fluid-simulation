use bevy::{
    ecs::{reflect::ReflectResource, resource::Resource},
    reflect::Reflect,
};
use bevy_inspector_egui::{InspectorOptions, inspector_options::ReflectInspectorOptions};

pub const DEFAULT_REGION_HEIGHT: f32 = 600.0;
pub const DEFAULT_REGION_WIDTH: f32 = 600.0;

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct FluidSettings {
    pub thickness: f32,
    pub number_of_particles: usize,
    pub pixels_per_meter: f32,
    pub gravity: f32,
    pub collision_dampening: f32,
    pub region_height: f32,
    pub region_width: f32,
}

impl FluidSettings {
    pub fn new() -> Self {
        Self {
            thickness: 2.0,
            number_of_particles: 100,
            pixels_per_meter: 50.0,
            gravity: 9.81,
            collision_dampening: 0.95,
            region_height: DEFAULT_REGION_HEIGHT,
            region_width: DEFAULT_REGION_WIDTH,
        }
    }
}
