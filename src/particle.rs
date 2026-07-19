use bevy::{
    ecs::{bundle::Bundle, component::Component},
    math::{Vec2, Vec3},
    mesh::Mesh2d,
    sprite_render::{ColorMaterial, MeshMaterial2d},
};

#[derive(Component, Default, Clone, Copy)]
pub struct Velocity(pub Vec3);

#[derive(Component, Default)]
pub struct Position(pub Vec3);

#[derive(Component, Default)]
pub struct RegionBoundingBox {
    pub center: Vec2,
    pub size: Vec2,
}

#[derive(Bundle, Default)]
pub struct Particle {
    pubmesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    velocity: Velocity,
    position: Position,
    region: RegionBoundingBox,
}

impl Particle {
    pub fn new(
        pubmesh: Mesh2d,
        material: MeshMaterial2d<ColorMaterial>,
        velocity: Velocity,
        position: Position,
        region: RegionBoundingBox,
    ) -> Self {
        Self {
            pubmesh,
            material,
            velocity,
            position,
            region,
        }
    }
}
