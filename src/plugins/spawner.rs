use std::f32::consts::PI;

use bevy::{
    math::ops::{ceil, sqrt},
    prelude::*,
};
use chacha20::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

use crate::{
    SimulationState,
    resources::{
        DEFAULT_REGION_HEIGHT, DEFAULT_REGION_WIDTH, FluidBuffer, FluidSettings, ParticleIndex,
    },
};

#[derive(Clone)]
pub struct SpawnRegion {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
}

pub struct Spawner;

#[derive(Resource)]
pub struct SpawnerSettings {
    pub spawn_density: f32,
    pub initial_velocity: Vec2,
    pub jitter: f32,
    pub spawn_regions: Vec<SpawnRegion>,
    pub show_spawn_gizmo: bool,
}

impl SpawnerSettings {
    pub fn default() -> Self {
        Self {
            spawn_density: 0.005,
            jitter: 2.0,
            initial_velocity: Vec2::ZERO,
            spawn_regions: vec![SpawnRegion {
                position: Vec2::new(10.0, 0.0),
                size: Vec2::new(DEFAULT_REGION_WIDTH, DEFAULT_REGION_HEIGHT),
                color: Color::srgb(0.0, 1.0, 0.0),
            }],
            show_spawn_gizmo: true,
        }
    }
}

impl Plugin for Spawner {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnerSettings::default())
            .add_systems(
                Update,
                draw_spawn_gizmos.run_if(in_state(SimulationState::Idle)),
            )
            .add_systems(Startup, spawn_particles);
    }
}

fn draw_spawn_gizmos(
    mut gizmos: Gizmos,
    settings: Res<SpawnerSettings>,
    fluid_settings: Res<FluidSettings>,
) {
    if !settings.show_spawn_gizmo {
        return;
    }

    for region in &settings.spawn_regions {
        gizmos.rect_2d(
            region.position,
            region.size + (2.0 * fluid_settings.thickness),
            region.color,
        );
    }
}

fn calculate_spawn_points_per_axis(size: Vec2, spawn_density: f32) -> IVec2 {
    let area = size.x * size.y;
    let target_total = ceil(area * spawn_density) as i32;
    let perimeter = size.x + size.y;
    let aspect_ratio = size / perimeter;
    let m = sqrt(target_total as f32 / (aspect_ratio.x * aspect_ratio.y));
    let nx = ceil(aspect_ratio.x * m) as i32;
    let ny = ceil(aspect_ratio.y * m) as i32;

    ivec2(nx, ny)
}

fn spawn_in_region(region: &SpawnRegion, spawn_density: f32) -> Vec<Vec2> {
    let center = region.position;
    let size = region.size;
    let mut points = vec![];
    let num_per_axis = calculate_spawn_points_per_axis(size, spawn_density);

    for y in 0..num_per_axis.y {
        for x in 0..num_per_axis.x {
            let tx = if num_per_axis.x > 1 {
                (x as f32) / (num_per_axis.x - 1) as f32
            } else {
                0.5
            };

            let ty = if num_per_axis.y > 1 {
                (y as f32) / (num_per_axis.y - 1) as f32
            } else {
                0.5
            };

            let px = (tx - 0.5) * size.x + center.x;
            let py = (ty - 0.5) * size.y + center.y;
            points.push(vec2(px, py));
        }
    }

    points
}

fn spawn_particles(
    mut commands: Commands,
    fluid_settings: Res<FluidSettings>,
    settings: Res<SpawnerSettings>,
    mut buffer: ResMut<FluidBuffer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    let mesh = meshes.add(Circle::new(fluid_settings.thickness));
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467713);

    buffer.clear();

    for (region_idx, region) in settings.spawn_regions.iter().enumerate() {
        let points = spawn_in_region(region, settings.spawn_density);

        for point in points {
            let random_element: f32 = rng.random();
            let angle = random_element * PI * 2.0;
            let dir = Vec2::from_angle(angle);
            let jitter_random: f32 = rng.random();
            let jitter = dir * settings.jitter * (jitter_random * 0.5);
            let position: Vec2 = (point + jitter) / fluid_settings.pixels_per_meter;

            buffer.positions.push(position);
            buffer.velocities.push(settings.initial_velocity);
            buffer.densities.push((0.0, 0.0));
            buffer.predicted_positions.push(position);
            buffer.region_indices.push(region_idx);
            let current_index = buffer.positions.len() - 1;

            commands.spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
                Transform::from_translation(position.extend(0.0)),
                ParticleIndex(current_index),
            ));
        }
    }

    let total_spawned = buffer.positions.len();
    buffer.initialise_spatial_buffers(total_spawned);
}
