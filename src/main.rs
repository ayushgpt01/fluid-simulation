use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
    text::FontSmoothing,
    window::PrimaryWindow,
};
use chacha20::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

const THICKNESS: f32 = 2.0;
const NUMBER_OF_PARTICLES: usize = 1200;
const PIXELS_PER_METER: f32 = 50.0;
const GRAVITY: f32 = 9.81;
const COLLISION_DAMPENING: f32 = 0.95;

#[derive(Component, Default)]
struct Velocity(Vec3);

#[derive(Component, Default)]
struct Position(Vec3);

#[derive(Bundle, Default)]
struct Particle {
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    velocity: Velocity,
    position: Position,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    let screen_width = window.width();
    let screen_height = window.height();

    let mesh = meshes.add(Circle::new(THICKNESS));

    commands.spawn(Camera2d);
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467713);

    let cols = (NUMBER_OF_PARTICLES as f32).sqrt().ceil() as f32;
    let rows = (NUMBER_OF_PARTICLES as f32 / cols).ceil() as f32;

    let cell_width = screen_width / cols;
    let cell_height = screen_height / rows;

    let diameter = THICKNESS * 2.0;
    let max_jitter_x = (cell_width - diameter).max(0.0) / 2.0;
    let max_jitter_y = (cell_height - diameter).max(0.0) / 2.0;

    let mut spawned = 0;

    for row in 0..rows as usize {
        for col in 0..cols as usize {
            if spawned >= NUMBER_OF_PARTICLES {
                break;
            }

            // Calculate the exact center of the current cell.
            // Bevy's 2D origin (0,0) is in the middle of the screen.
            let base_x = -screen_width / 2.0 + (col as f32 * cell_width) + (cell_width / 2.0);
            let base_y = -screen_height / 2.0 + (row as f32 * cell_height) + (cell_height / 2.0);

            // Apply random jitter within safe bounds
            let jitter_x = rng.random_range(-max_jitter_x..=max_jitter_x);
            let jitter_y = rng.random_range(-max_jitter_y..=max_jitter_y);

            let position = vec3(base_x + jitter_x, base_y + jitter_y, 0.0);
            let color = Color::hsl(
                360. * spawned as f32 / NUMBER_OF_PARTICLES as f32,
                0.95,
                0.7,
            );

            commands.spawn((
                Particle {
                    mesh: Mesh2d(mesh.clone()),
                    material: MeshMaterial2d(materials.add(color)),
                    velocity: Velocity(Vec3::ZERO),
                    position: Position(vec3(
                        position.x / PIXELS_PER_METER,
                        position.y / PIXELS_PER_METER,
                        position.z / PIXELS_PER_METER,
                    )),
                },
                Transform::from_translation(position),
            ));

            spawned += 1;
        }
    }
}

fn sync_transform(mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in &mut query {
        transform.translation.x = position.0.x * PIXELS_PER_METER;
        transform.translation.y = position.0.y * PIXELS_PER_METER;
        transform.translation.z = 0.0;
    }
}

fn apply_forces(time: Res<Time<Fixed>>, mut query: Query<&mut Velocity>) {
    let dt = time.delta_secs();

    for mut velocity in &mut query {
        velocity.0.y -= GRAVITY * dt;
    }
}

fn update_positions(time: Res<Time<Fixed>>, mut query: Query<(&Velocity, &mut Position)>) {
    let dt = time.delta_secs();

    for (velocity, mut position) in &mut query {
        position.0 += velocity.0 * dt;
    }
}

fn resolve_collisions(
    mut query: Query<(&mut Velocity, &mut Position)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    let screen_width = window.width() / PIXELS_PER_METER;
    let screen_height = window.height() / PIXELS_PER_METER;

    let radius = THICKNESS / PIXELS_PER_METER;
    let half_screen_width = screen_width / 2.0;
    let half_screen_height = screen_height / 2.0;

    for (mut velocity, mut position) in &mut query {
        let x = position.0.x;
        let y = position.0.y;

        if x < (radius - half_screen_width) {
            position.0.x = radius - half_screen_width;
            velocity.0.x *= -1.0 * COLLISION_DAMPENING;
        } else if x > half_screen_width - radius {
            position.0.x = half_screen_width - radius;
            velocity.0.x *= -1.0 * COLLISION_DAMPENING;
        }

        if y < (radius - half_screen_height) {
            position.0.y = radius - half_screen_height;
            velocity.0.y *= -1.0 * COLLISION_DAMPENING;
        } else if y > half_screen_height - radius {
            position.0.y = half_screen_height - radius;
            velocity.0.y *= -1.0 * COLLISION_DAMPENING;
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: FontSize::Px(20.0),
                        font: default(),
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    text_color: Color::srgb(0.0, 1.0, 0.0),
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: false,
                        min_fps: 30.0,
                        target_fps: 144.0,
                    },
                },
            },
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (apply_forces, update_positions, resolve_collisions).chain(),
        )
        .add_systems(Update, sync_transform)
        .run();
}
