use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    ecs::error::error,
    input::common_conditions::input_just_pressed,
    prelude::*,
    text::FontSmoothing,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};

use crate::{config::FluidSettings, particle::*, spawner::Spawner};

mod config;
mod particle;
mod spawner;

fn main() -> AppExit {
    let mut app = App::new();

    app.set_error_handler(error);

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Window {
                title: "Fluid Simulator".to_string(),
                fit_canvas_to_parent: true,
                ..default()
            }
            .into(),
            ..default()
        }),
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
        AppPlugin,
    ));

    app.init_state::<SimulationState>();
    app.run()
}

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationState {
    #[default]
    Idle,
    Running,
    Paused,
}

struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        println!("Running the build from DemoPlugin");
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(FluidSettings::new())
            .add_plugins(EguiPlugin::default())
            .add_plugins(ResourceInspectorPlugin::<FluidSettings>::new())
            .add_plugins(Spawner)
            .add_systems(
                FixedUpdate,
                (apply_forces, update_positions, resolve_collisions)
                    .chain()
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(Update, sync_transform)
            .add_systems(
                Update,
                handle_pause.run_if(input_just_pressed(KeyCode::Space)),
            );
    }
}

fn apply_forces(
    settings: Res<FluidSettings>,
    time: Res<Time<Fixed>>,
    mut query: Query<&mut Velocity>,
) {
    let dt = time.delta_secs();

    for mut velocity in &mut query {
        velocity.0.y -= settings.gravity * dt;
    }
}

fn update_positions(time: Res<Time<Fixed>>, mut query: Query<(&Velocity, &mut Position)>) {
    let dt = time.delta_secs();

    for (velocity, mut position) in &mut query {
        position.0 += velocity.0 * dt;
    }
}

fn resolve_collisions(
    settings: Res<FluidSettings>,
    mut query: Query<(&mut Velocity, &mut Position, &RegionBoundingBox)>,
) {
    let radius = settings.thickness / settings.pixels_per_meter;

    for (mut velocity, mut position, bounds) in &mut query {
        let screen_width = bounds.size.x / settings.pixels_per_meter;
        let screen_height = bounds.size.y / settings.pixels_per_meter;
        let center = bounds.center / settings.pixels_per_meter;

        let half_screen_width = (screen_width / 2.0) + center.x;
        let half_screen_height = (screen_height / 2.0) + center.y;

        let x = position.0.x - center.x;
        let y = position.0.y - center.y;

        if x < (radius - half_screen_width) {
            position.0.x = radius - half_screen_width;
            velocity.0.x *= -1.0 * settings.collision_dampening;
        } else if x > half_screen_width - radius {
            position.0.x = half_screen_width - radius;
            velocity.0.x *= -1.0 * settings.collision_dampening;
        }

        if y < (radius - half_screen_height) {
            position.0.y = radius - half_screen_height;
            velocity.0.y *= -1.0 * settings.collision_dampening;
        } else if y > half_screen_height - radius {
            position.0.y = half_screen_height - radius;
            velocity.0.y *= -1.0 * settings.collision_dampening;
        }
    }
}

fn sync_transform(settings: Res<FluidSettings>, mut query: Query<(&Position, &mut Transform)>) {
    for (position, mut transform) in &mut query {
        transform.translation.x = position.0.x * settings.pixels_per_meter;
        transform.translation.y = position.0.y * settings.pixels_per_meter;
        transform.translation.z = 0.0;
    }
}

fn handle_pause(
    state: Res<State<SimulationState>>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    match state.get() {
        SimulationState::Idle | SimulationState::Paused => {
            next_state.set(SimulationState::Running);
        }
        SimulationState::Running => {
            next_state.set(SimulationState::Paused);
        }
    }
}
