use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    ecs::error::error,
    input::common_conditions::input_just_pressed,
    prelude::*,
    text::FontSmoothing,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};

use crate::{
    plugins::{Physics, Spawner, TransformCoordinates},
    resources::{FluidBuffer, FluidSettings, GradientLookup},
};

mod plugins;
mod resources;

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
        let fluid_settings = FluidSettings::new();
        let fluid_buffer = FluidBuffer::new(fluid_settings.number_of_particles);

        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(fluid_settings)
            .insert_resource(fluid_buffer)
            .insert_resource(GradientLookup::default())
            .add_plugins(EguiPlugin::default())
            .add_plugins(ResourceInspectorPlugin::<FluidSettings>::new())
            .add_plugins((Spawner, Physics, TransformCoordinates))
            .add_systems(
                Update,
                handle_pause.run_if(input_just_pressed(KeyCode::Space)),
            );
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
