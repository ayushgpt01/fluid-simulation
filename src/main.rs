use std::f32::consts::PI;

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use rayon::prelude::*;

// Constants
const MAX_SPEED: f32 = 14.0;
const MASS: f32 = 1.0;
const PIXELS_PER_METER: f32 = 50.0;
const NUMBER_OF_PARTICLES: usize = 1200;

// In pixels
const PARTICLE_SIZE: f32 = 4.0;

// In meters
const PARTICLE_SPACING: f32 = 0.9;

#[derive(Clone, Copy)]
struct Pair {
    index: usize,
    cell_key: usize,
}

fn get_screen_width() -> f32 {
    600.0
}
fn get_screen_height() -> f32 {
    600.0
}

fn create_grid_particles(positions: &mut Vec<Vec2>, velocities: &mut Vec<Vec2>) {
    let particles_per_row = (NUMBER_OF_PARTICLES as f32).sqrt();
    let particles_per_col = ((NUMBER_OF_PARTICLES as f32) - 1.0) / particles_per_row + 1.0;
    let spacing: f32 = PARTICLE_SPACING;

    let screen_w_meters = get_screen_width() / PIXELS_PER_METER;
    let screen_h_meters = get_screen_height() / PIXELS_PER_METER;
    let center_x = screen_w_meters / 2.0;
    let center_y = screen_h_meters / 2.0;

    for index in 0..NUMBER_OF_PARTICLES {
        let x: f32 = ((index as f32) % particles_per_row - particles_per_row / 2.0 + 0.5) * spacing;
        let y: f32 = ((index as f32) / particles_per_row - particles_per_col / 2.0 + 0.5) * spacing;
        positions.push(vec2(center_x + x, center_y + y));
        velocities.push(vec2(0.0, 0.0));
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Wave Simulation".to_owned(),
        window_width: get_screen_width() as i32,
        window_height: get_screen_height() as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), macroquad::Error> {
    let mut gravity: f32 = 9.8;
    let mut smoothing_radius: f32 = 0.35;
    let mut target_density: f32 = 20.0;
    let mut pressure_multiplier: f32 = 15.68;
    let mut viscosity_strength: f32 = 0.75;
    let mut collision_dampening: f32 = 0.95;

    let mut positions: Vec<Vec2> = vec![];
    let mut velocities: Vec<Vec2> = vec![];
    let mut is_paused: bool = true;

    create_grid_particles(&mut positions, &mut velocities);

    let mut densities = vec![0.0; NUMBER_OF_PARTICLES];
    let mut predicted_positions: Vec<Vec2> = vec![vec2(0.0, 0.0); NUMBER_OF_PARTICLES];
    let mut spatial_lookup = vec![
        Pair {
            cell_key: 0,
            index: 0
        };
        NUMBER_OF_PARTICLES
    ];
    let mut start_indices = vec![0; NUMBER_OF_PARTICLES];

    loop {
        clear_background(BLACK);
        let delta_time = get_frame_time();

        if is_key_pressed(KeyCode::Space) {
            is_paused = !is_paused;
        }

        if !is_paused {
            velocities
                .par_iter_mut()
                .zip(&mut predicted_positions)
                .zip(&positions)
                .for_each(|((velocity, predicted_pos), position)| {
                    velocity.y += gravity * delta_time;
                    *predicted_pos = *position + *velocity * delta_time;
                });

            update_spatial_lookup(
                &mut spatial_lookup,
                &mut start_indices,
                &predicted_positions,
                smoothing_radius,
            );

            densities
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, density)| {
                    *density = calculate_density(
                        i,
                        &predicted_positions,
                        &spatial_lookup,
                        &start_indices,
                        smoothing_radius,
                    );
                });

            let velocities_snapshot = velocities.clone();

            velocities
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, velocity)| {
                    let pressure_force = calculate_pressure_force(
                        i,
                        &predicted_positions,
                        &densities,
                        pressure_multiplier,
                        &spatial_lookup,
                        &start_indices,
                        target_density,
                        smoothing_radius,
                    );
                    let viscosity_force = calculate_viscosity_force(
                        i,
                        &predicted_positions,
                        &velocities_snapshot,
                        &spatial_lookup,
                        &start_indices,
                        smoothing_radius,
                        viscosity_strength,
                    );

                    let total_force = pressure_force + viscosity_force;
                    let acceleration = total_force / densities[i];
                    *velocity += acceleration * delta_time;
                });

            positions
                .par_iter_mut()
                .zip(&mut velocities)
                .for_each(|(position, velocity)| {
                    *position += *velocity * delta_time;

                    resolve_collisions(position, velocity, collision_dampening);
                });
        }

        for i in 0..NUMBER_OF_PARTICLES {
            let render_x = positions[i].x * PIXELS_PER_METER;
            let render_y = positions[i].y * PIXELS_PER_METER;
            let speed = velocities[i].length();
            let normalized_speed = (speed / MAX_SPEED).clamp(0.0, 1.0);

            let current_color = lerp_color(BLUE, RED, normalized_speed);
            draw_poly(render_x, render_y, 255, PARTICLE_SIZE, 0., current_color);
        }

        widgets::Window::new(hash!(), vec2(20.0, 20.0), vec2(250.0, 180.0))
            .label("Simulation Settings")
            .ui(&mut *root_ui(), |ui| {
                // Show status and a simple button to toggle pause
                let btn_label = if is_paused {
                    "Resume Simulation"
                } else {
                    "Pause Simulation"
                };
                if ui.button(None, btn_label) {
                    is_paused = !is_paused;
                }

                ui.separator();

                // Sliders to dynamically adjust values at runtime
                ui.slider(hash!("gravity"), "Gravity", -20.0..20.0, &mut gravity);
                // let mut smoothing_radius: f32 = 0.35;
                ui.slider(
                    hash!("smoothing_radius"),
                    "Smoothing Radius",
                    0.05..1.0,
                    &mut smoothing_radius,
                );
                // let mut target_density: f32 = 5.2;
                ui.slider(
                    hash!("target_density"),
                    "Target Density",
                    0.5..5.0,
                    &mut target_density,
                );
                // let mut pressure_multiplier: f32 = 15.8;
                ui.slider(
                    hash!("pressure_multiplier"),
                    "Pressure Multiplier",
                    1.0..100.0,
                    &mut pressure_multiplier,
                );
                // let mut viscosity_strength: f32 = 2.5;
                ui.slider(
                    hash!("viscosity_strength"),
                    "Viscosity Strength",
                    0.0..2.0,
                    &mut viscosity_strength,
                );
                // let mut collision_dampening: f32 = 0.95;
                ui.slider(
                    hash!("collision_dampening"),
                    "Collision Dampening",
                    0.0..1.0,
                    &mut collision_dampening,
                );
            });

        draw_fps();

        next_frame().await
    }
}

fn resolve_collisions(position: &mut Vec2, velocity: &mut Vec2, collision_dampening: f32) {
    let screen_w_meters = get_screen_width() / PIXELS_PER_METER;
    let screen_h_meters = get_screen_height() / PIXELS_PER_METER;
    let radius_meters = PARTICLE_SIZE / PIXELS_PER_METER;

    if position.x < radius_meters || position.x.abs() > screen_w_meters - radius_meters {
        position.x = clamp(position.x, radius_meters, screen_w_meters - radius_meters);
        velocity.x *= -1.0 * collision_dampening;
    }

    if position.y < radius_meters || position.y.abs() > screen_h_meters - radius_meters {
        position.y = clamp(position.y, radius_meters, screen_h_meters - radius_meters);
        velocity.y *= -1.0 * collision_dampening;
    }
}

// W(r,h) -> h is core radius
fn smoothing_kernel(radius: f32, distance: f32) -> f32 {
    if distance >= radius {
        return 0.0;
    }

    let volume = (PI * radius.powf(4.0)) / 6.0;
    (radius - distance) * (radius - distance) / volume
}

fn smoothing_kernel_derivative(radius: f32, distance: f32) -> f32 {
    if distance >= radius || distance == 0.0 {
        return 0.0;
    }

    let scale = 12.0 / (PI * radius.powf(4.0));
    (distance - radius) * scale
}

fn convert_density_to_pressure(density: f32, pressure_multiplier: f32, target_density: f32) -> f32 {
    let density_error = density - target_density;
    density_error * pressure_multiplier
}

fn calculate_shared_pressure(
    d1: f32,
    d2: f32,
    pressure_multiplier: f32,
    target_density: f32,
) -> f32 {
    (convert_density_to_pressure(d1, pressure_multiplier, target_density)
        + convert_density_to_pressure(d2, pressure_multiplier, target_density))
        / 2.0
}

fn calculate_pressure_force(
    particle_index: usize,
    positions: &Vec<Vec2>,
    densities: &Vec<f32>,
    pressure_multiplier: f32,
    spatial_lookup: &Vec<Pair>,
    start_indices: &Vec<usize>,
    target_density: f32,
    smoothing_radius: f32,
) -> Vec2 {
    let mut pressure_force = vec2(0.0, 0.0);
    let pos = positions[particle_index];

    for index in look_within_radius(
        &pos,
        spatial_lookup,
        start_indices,
        positions,
        smoothing_radius,
    ) {
        if particle_index == index {
            continue;
        }
        let offset = positions[index] - pos;
        let dist = offset.length();
        let dir = if dist == 0.0 {
            let angle = (index as f32) * 0.1;
            vec2(angle.cos(), angle.sin())
        } else {
            offset / dist
        };
        let slope = smoothing_kernel_derivative(smoothing_radius, dist);
        let density = densities[index];
        let shared_pressure = calculate_shared_pressure(
            density,
            densities[particle_index],
            pressure_multiplier,
            target_density,
        );
        pressure_force += shared_pressure * dir * slope * MASS / density;
    }

    pressure_force
}

fn calculate_density(
    position_index: usize,
    positions: &Vec<Vec2>,
    spatial_lookup: &Vec<Pair>,
    start_indices: &Vec<usize>,
    smoothing_radius: f32,
) -> f32 {
    let mut density: f32 = 0.0;
    let pos = positions[position_index];

    for index in look_within_radius(
        &pos,
        spatial_lookup,
        start_indices,
        positions,
        smoothing_radius,
    ) {
        let distance = (positions[index] - pos).length();
        let influence = smoothing_kernel(smoothing_radius, distance);
        density += MASS * influence;
    }

    density
}

fn position_to_cell_coord(point: &Vec2, radius: f32) -> (i32, i32) {
    let cell_x = (point.x / radius) as i32;
    let cell_y = (point.y / radius) as i32;
    (cell_x, cell_y)
}

fn hash_cell(cell_x: i32, cell_y: i32) -> usize {
    let x = cell_x as usize;
    let y = cell_y as usize;

    x.wrapping_mul(15823).wrapping_add(y.wrapping_mul(9737333))
}

fn get_key_from_hash(hash: usize, len: usize) -> usize {
    hash % len
}

fn update_spatial_lookup(
    spatial_lookup: &mut Vec<Pair>,
    start_indices: &mut Vec<usize>,
    positions: &Vec<Vec2>,
    radius: f32,
) {
    for i in 0..positions.len() {
        let (cell_x, cell_y) = position_to_cell_coord(&positions[i], radius);
        let cell_key = get_key_from_hash(hash_cell(cell_x, cell_y), spatial_lookup.len());
        spatial_lookup[i] = Pair { index: i, cell_key };
        start_indices[i] = usize::MAX;
    }

    spatial_lookup.sort_by_key(|p| p.cell_key);

    for i in 0..positions.len() {
        let key = spatial_lookup[i].cell_key;
        let key_prev = if i == 0 {
            usize::MAX
        } else {
            spatial_lookup[i - 1].cell_key
        };

        if key != key_prev {
            start_indices[key] = i;
        }
    }
}

fn viscosity_smoothing_kernel(distance: f32, radius: f32) -> f32 {
    let volume = PI * radius.powf(8.0) / 4.0;
    let dist = radius * radius - distance * distance;
    let value = dist.max(0.0);
    value * value * value / volume
}

fn calculate_viscosity_force(
    particle_index: usize,
    positions: &Vec<Vec2>,
    velocities: &Vec<Vec2>,
    spatial_lookup: &Vec<Pair>,
    start_indices: &Vec<usize>,
    smoothing_radius: f32,
    viscosity_strength: f32,
) -> Vec2 {
    let mut force = vec2(0.0, 0.0);
    let pos = positions[particle_index];

    for index in look_within_radius(
        &pos,
        spatial_lookup,
        start_indices,
        positions,
        smoothing_radius,
    ) {
        if particle_index == index {
            continue;
        }

        let dist = (pos - positions[index]).length();
        let influence = viscosity_smoothing_kernel(dist, smoothing_radius);
        force += (velocities[index] - velocities[particle_index]) * influence;
    }

    force * viscosity_strength
}

const CELL_OFFSETS: [(i32, i32); 9] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (0, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

fn look_within_radius(
    point: &Vec2,
    spatial_lookup: &Vec<Pair>,
    start_indices: &Vec<usize>,
    positions: &Vec<Vec2>,
    smoothing_radius: f32,
) -> impl Iterator<Item = usize> {
    let (center_x, center_y) = position_to_cell_coord(point, smoothing_radius);
    let sqr_radius = smoothing_radius * smoothing_radius;

    CELL_OFFSETS.iter().flat_map(move |&(offset_x, offset_y)| {
        let cell_x = center_x.wrapping_add(offset_x);
        let cell_y = center_y.wrapping_add(offset_y);

        let key = get_key_from_hash(hash_cell(cell_x, cell_y), spatial_lookup.len());
        let cell_start_index = start_indices[key];

        let slice = if cell_start_index == usize::MAX {
            &spatial_lookup[0..0]
        } else {
            &spatial_lookup[cell_start_index..]
        };

        slice
            .iter()
            .take_while(move |pair| pair.cell_key == key)
            .filter_map(move |pair| {
                let particle_index = pair.index;
                let sqr_dist = (positions[particle_index] - *point).length_squared();
                if sqr_dist <= sqr_radius {
                    Some(particle_index)
                } else {
                    None
                }
            })
    })
}

// Optimising Lookups - https://youtu.be/rSKMYc1CQHE?si=lIqsg9nogyTjc1um&t=1351
// first commit link - https://github.com/SebLague/Fluid-Sim/commit/f9dd346947b399de521390bdb2c5d4514c0c18c6#diff-125c6cd84a4d34af9e518d4f6c5b0e5161be9a9800e0e300575bb2a7f7826026
