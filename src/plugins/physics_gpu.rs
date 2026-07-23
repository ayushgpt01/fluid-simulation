use bevy::{
    material::descriptor::BindGroupLayoutDescriptor,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup,
        extract_resource::ExtractResourcePlugin,
        render_resource::{
            binding_types::{storage_buffer, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};
use std::borrow::Cow;

use crate::resources::{FluidBuffer, FluidSettings};

#[derive(Resource)]
struct ComputePipeline {
    bind_group_layout: BindGroupLayoutDescriptor,
    update_positions: CachedComputePipelineId,
    external_forces: CachedComputePipelineId,
}

pub struct PhysicsGPUPlugin;

impl Plugin for PhysicsGPUPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<FluidSettings>::default(),
            ExtractResourcePlugin::<FluidBuffer>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_compute_pipeline)
            .add_systems(Render, prepare_buffers)
            .add_systems(Render, run_simulation);
    }
    fn cleanup(&self, _app: &mut App) {}
}

const SHADER_ASSET_PATH: &str = "shaders/fluid_sim.wgsl";

fn init_compute_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let shader = asset_server.load(SHADER_ASSET_PATH);

    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "Fluid Buffers Layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                uniform_buffer::<FluidSettings>(false),
                storage_buffer::<Vec2>(false),
                storage_buffer::<Vec2>(false),
            ),
        ),
    );

    let external_forces = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("Update Positions Pipeline".into()),
        layout: vec![bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("external_forces")),
        ..default()
    });

    let update_positions = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("Update Positions Pipeline".into()),
        layout: vec![bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("update_positions")),
        ..default()
    });

    commands.insert_resource(ComputePipeline {
        bind_group_layout,
        update_positions,
        external_forces,
    });
}

#[derive(Resource)]
struct GPUBuffers {
    positions: Buffer,
    velocities: Buffer,
    settings_uniform: UniformBuffer<FluidSettings>,
}

fn prepare_buffers(
    time: Res<Time>,
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    pipelines: Option<Res<ComputePipeline>>,
    buffers: Option<Res<FluidBuffer>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    settings: Option<Res<FluidSettings>>,
    gpu_buffers: Option<ResMut<GPUBuffers>>,
) {
    let Some(pipelines) = pipelines else { return };
    let Some(buffers) = buffers else { return };
    let Some(settings) = settings else { return };

    if let Some(mut existing_buffers) = gpu_buffers {
        existing_buffers.settings_uniform.set(FluidSettings {
            delta_time: time.delta_secs(),
            ..settings.clone()
        });
        existing_buffers
            .settings_uniform
            .write_buffer(&render_device, &render_queue);
        return;
    }

    if pipeline_cache
        .get_compute_pipeline(pipelines.external_forces)
        .is_none()
        || pipeline_cache
            .get_compute_pipeline(pipelines.update_positions)
            .is_none()
    {
        return;
    }

    // Initialise GPUBuffers
    let positions = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Positions buffer"),
        contents: bytemuck::cast_slice(&buffers.positions),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    let velocities = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Velocities Buffer"),
        contents: bytemuck::cast_slice(&buffers.velocities),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::VERTEX,
    });

    let mut settings_uniform = UniformBuffer::from(FluidSettings {
        delta_time: time.delta_secs(),
        ..settings.clone()
    });
    settings_uniform.write_buffer(&render_device, &render_queue);

    commands.insert_resource(GPUBuffers {
        positions,
        velocities,
        settings_uniform,
    });
}

fn run_simulation(
    mut render_context: RenderContext,
    pipeline_cache: Res<PipelineCache>,
    pipelines: Option<Res<ComputePipeline>>,
    gpu_buffers: Option<Res<GPUBuffers>>,
    settings: Option<Res<FluidSettings>>,
) {
    let (Some(pipelines), Some(gpu_buffers), Some(settings)) = (pipelines, gpu_buffers, settings)
    else {
        return;
    };

    let Some(update_positions) = pipeline_cache.get_compute_pipeline(pipelines.update_positions)
    else {
        return;
    };

    let Some(external_forces) = pipeline_cache.get_compute_pipeline(pipelines.external_forces)
    else {
        return;
    };

    let bind_group = render_context.render_device().create_bind_group(
        "Fluid buffers group",
        &pipeline_cache.get_bind_group_layout(&pipelines.bind_group_layout),
        &BindGroupEntries::sequential((
            gpu_buffers.settings_uniform.into_binding(),
            gpu_buffers.positions.as_entire_buffer_binding(),
            gpu_buffers.velocities.as_entire_buffer_binding(),
        )),
    );

    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("Compute Pass"),
            ..default()
        });

    pass.set_bind_group(0, &bind_group, &[]);

    let workgroups = (settings.number_of_particles + 63) / 64;

    pass.set_pipeline(external_forces);
    pass.dispatch_workgroups(workgroups, 1, 1);

    pass.set_pipeline(update_positions);
    pass.dispatch_workgroups(workgroups, 1, 1);
}
