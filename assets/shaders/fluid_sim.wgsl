struct FluidSettings {
    thickness: f32,
    number_of_particles: u32,
    pixels_per_meter: f32,
    gravity: f32,
    collision_dampening: f32,
    region_height: f32,
    region_width: f32,
    smoothing_radius: f32,
    target_density: f32,
    pressure_multiplier: f32,
    near_pressure_multiplier: f32,
    viscosity_strength: f32,
    max_speed: f32,
    delta_time: f32,
}

@group(0) @binding(0) var<uniform> settings: FluidSettings;
@group(0) @binding(1) var<storage, read_write> positions: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> velocities: array<vec2<f32>>;

@compute @workgroup_size(64)
fn external_forces(@builtin(global_invocation_id) id: vec3<u32>) {
    let dt = settings.delta_time;
    let index = id.x;

    velocities[index].y -= settings.gravity * dt; 
}

@compute @workgroup_size(64)
fn update_positions(@builtin(global_invocation_id) id: vec3<u32>) {
    let dt = settings.delta_time;
    let index = id.x;

    positions[index] += velocities[index] * dt;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let particle_pos = positions[vertex_index];

    out.clip_position = vec4<f32>(particle_pos, 0.0, 1.0);
    out.color = vec4<f32>(0.2, 0.6, 1.0, 1.0);

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}