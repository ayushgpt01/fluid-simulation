use std::f32::consts::PI;

use macroquad::prelude::*;

// Constants
const GRAVITY: f32 = 0.0;
const PIXELS_PER_METER: f32 = 50.0;
const NUMBER_OF_PARTICLES: usize = 3200;
const SMOOTHING_RADIUS: f32 = 0.3;
const MASS: f32 = 1.0;
const TARGET_DENSITY: f32 = 1.5;
const PRESSURE_MULTIPLIER: f32 = 16.3;

// In pixels
const PARTICLE_SIZE: f32 = 4.0;

// In meters
const PARTICLE_SPACING: f32 = 0.9;
const COLLISION_DAMPENING: f32 = 0.6;

#[derive(Clone, Copy)]
struct Pair {
    index: usize,
    cell_key: usize,
}

fn create_grid_particles(positions: &mut Vec<Vec2>, velocities: &mut Vec<Vec2>) {
    let particles_per_row = (NUMBER_OF_PARTICLES as f32).sqrt();
    let particles_per_col = ((NUMBER_OF_PARTICLES as f32) - 1.0) / particles_per_row + 1.0;
    let spacing: f32 = PARTICLE_SPACING;

    let screen_w_meters = screen_width() / PIXELS_PER_METER;
    let screen_h_meters = screen_height() / PIXELS_PER_METER;
    let center_x = screen_w_meters / 2.0;
    let center_y = screen_h_meters / 2.0;

    for index in 0..NUMBER_OF_PARTICLES {
        let x: f32 = ((index as f32) % particles_per_row - particles_per_row / 2.0 + 0.5) * spacing;
        let y: f32 = ((index as f32) / particles_per_row - particles_per_col / 2.0 + 0.5) * spacing;
        positions.push(vec2(center_x + x, center_y + y));
        velocities.push(vec2(0.0, 0.0));
    }
}

#[macroquad::main("Wave Simulation")]
async fn main() -> Result<(), macroquad::Error> {
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
            for i in 0..NUMBER_OF_PARTICLES {
                velocities[i].y += GRAVITY * delta_time;
                predicted_positions[i] = positions[i] + velocities[i] * delta_time;
            }

            update_spatial_lookup(
                &mut spatial_lookup,
                &mut start_indices,
                &predicted_positions,
                SMOOTHING_RADIUS,
            );

            for i in 0..NUMBER_OF_PARTICLES {
                densities[i] = calculate_density(i, &predicted_positions);
            }

            for i in 0..NUMBER_OF_PARTICLES {
                let pressure_force = calculate_pressure_force(
                    i,
                    &predicted_positions,
                    &densities,
                    PRESSURE_MULTIPLIER,
                );
                let pressure_accelaration = pressure_force / densities[i];
                velocities[i] += pressure_accelaration * delta_time;
            }
        }

        for i in 0..NUMBER_OF_PARTICLES {
            if !is_paused {
                positions[i] += velocities[i] * delta_time;

                resolve_collisions(&mut positions[i], &mut velocities[i]);
            }

            let render_x = positions[i].x * PIXELS_PER_METER;
            let render_y = positions[i].y * PIXELS_PER_METER;

            draw_poly(render_x, render_y, 255, PARTICLE_SIZE, 0., SKYBLUE);
        }

        next_frame().await
    }
}

fn resolve_collisions(position: &mut Vec2, velocity: &mut Vec2) {
    let screen_w_meters = screen_width() / PIXELS_PER_METER;
    let screen_h_meters = screen_height() / PIXELS_PER_METER;
    let radius_meters = PARTICLE_SIZE / PIXELS_PER_METER;

    if position.x < radius_meters || position.x.abs() > screen_w_meters - radius_meters {
        position.x = clamp(position.x, radius_meters, screen_w_meters - radius_meters);
        velocity.x *= -1.0 * COLLISION_DAMPENING;
    }

    if position.y < radius_meters || position.y.abs() > screen_h_meters - radius_meters {
        position.y = clamp(position.y, radius_meters, screen_h_meters - radius_meters);
        velocity.y *= -1.0 * COLLISION_DAMPENING;
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

fn convert_density_to_pressure(density: f32, pressure_multiplier: f32) -> f32 {
    let density_error = density - TARGET_DENSITY;
    density_error * pressure_multiplier
}

fn calculate_shared_pressure(d1: f32, d2: f32, pressure_multiplier: f32) -> f32 {
    (convert_density_to_pressure(d1, pressure_multiplier)
        + convert_density_to_pressure(d2, pressure_multiplier))
        / 2.0
}

fn calculate_pressure_force(
    particle_index: usize,
    positions: &Vec<Vec2>,
    densities: &Vec<f32>,
    pressure_multiplier: f32,
) -> Vec2 {
    let mut pressure_force = vec2(0.0, 0.0);

    for i in 0..NUMBER_OF_PARTICLES {
        if particle_index == i {
            continue;
        }
        let offset = positions[i] - positions[particle_index];
        let dist = offset.length();
        let dir = if dist == 0.0 {
            vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)).normalize_or_zero()
        } else {
            offset / dist
        };
        let slope = smoothing_kernel_derivative(SMOOTHING_RADIUS, dist);
        let density = densities[i];
        let shared_pressure =
            calculate_shared_pressure(density, densities[particle_index], pressure_multiplier);
        pressure_force += shared_pressure * dir * slope * MASS / density;
    }

    pressure_force
}

fn calculate_density(position_index: usize, positions: &Vec<Vec2>) -> f32 {
    let mut density: f32 = 0.0;

    for position in positions {
        let distance = (*position - positions[position_index]).length();
        let influence = smoothing_kernel(SMOOTHING_RADIUS, distance);
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
) -> impl Iterator<Item = usize> {
    let (center_x, center_y) = position_to_cell_coord(point, SMOOTHING_RADIUS);
    let sqr_radius = SMOOTHING_RADIUS * SMOOTHING_RADIUS;

    CELL_OFFSETS.iter().flat_map(move |&(offset_x, offset_y)| {
        let key = get_key_from_hash(
            hash_cell(center_x + offset_x, center_y + offset_y),
            spatial_lookup.len(),
        );
        let cell_start_index = start_indices[key];

        // if cell_start_index == usize::MAX {
        //     return [].as_slice().iter().copied().flat_map(|_| None);
        // }

        spatial_lookup[cell_start_index..]
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
