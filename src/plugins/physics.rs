use bevy::prelude::*;
use rayon::prelude::*;

use crate::{
    PhysicsBackend, SimulationState,
    plugins::{
        CELL_OFFSETS, PhysicsGPUPlugin, density_derivative, density_kernel, get_cell, hash_cell,
        key_from_hash, near_density_derivative, near_density_kernel, near_pressure_from_density,
        pressure_from_density, spawner::SpawnerSettings, viscosity_kernel,
    },
    resources::{FluidBuffer, FluidSettings, GradientLookup},
};

pub struct Physics;

impl Plugin for Physics {
    fn build(&self, app: &mut App) {
        app.init_state::<PhysicsBackend>()
            .add_systems(
                FixedUpdate,
                (
                    external_forces,
                    run_spatial_hash,
                    update_density,
                    apply_pressure,
                    apply_viscosity,
                    update_positions,
                    resolve_collisions,
                    compute_particle_colors,
                )
                    .chain()
                    .run_if(in_state(SimulationState::Running))
                    .run_if(in_state(PhysicsBackend::CPU)),
            )
            .add_plugins(PhysicsGPUPlugin);
    }
}

const PREDICTION_FACTOR: f32 = 1.0 / 120.0;

fn external_forces(
    settings: Res<FluidSettings>,
    time: Res<Time<Fixed>>,
    mut buffer: ResMut<FluidBuffer>,
) {
    let dt = time.delta_secs();

    let FluidBuffer {
        positions,
        velocities,
        predicted_positions,
        ..
    } = &mut *buffer;

    velocities
        .par_iter_mut()
        .zip(positions)
        .zip(predicted_positions)
        .for_each(|((velocity, position), predicted_position)| {
            velocity.y -= settings.gravity * dt;
            *predicted_position = *position + *velocity * PREDICTION_FACTOR;
        });
}

fn run_spatial_hash(settings: Res<FluidSettings>, mut buffer: ResMut<FluidBuffer>) {
    let FluidBuffer {
        predicted_positions,
        spatial_offsets,
        spatial_keys,
        spatial_indices,
        ..
    } = &mut *buffer;
    let number_of_particles = settings.number_of_particles as usize;

    let mut pairs: Vec<(usize, usize)> = vec![(0, 0); number_of_particles];

    pairs.par_iter_mut().enumerate().for_each(|(index, pair)| {
        let cell = get_cell(&predicted_positions[index], settings.smoothing_radius);
        let hash = hash_cell(cell);
        let key = key_from_hash(hash, number_of_particles);
        *pair = (key, index);
    });

    pairs.sort_unstable_by_key(|(key, _)| *key);

    for (i, (key, index)) in pairs.iter().enumerate() {
        spatial_keys[i] = *key;
        spatial_indices[i] = *index;
    }

    spatial_offsets.fill(number_of_particles);

    for i in 0..number_of_particles {
        let key = spatial_keys[i];
        let prev_key = if i == 0 {
            usize::MAX
        } else {
            spatial_keys[i - 1]
        };

        if key != prev_key {
            spatial_offsets[key] = i;
        }
    }
}

fn update_density(settings: Res<FluidSettings>, mut buffer: ResMut<FluidBuffer>) {
    let FluidBuffer {
        densities,
        predicted_positions,
        spatial_offsets,
        spatial_keys,
        spatial_indices,
        ..
    } = &mut *buffer;

    let number_of_particles = settings.number_of_particles as usize;

    densities
        .par_iter_mut()
        .enumerate()
        .for_each(|(index, densities)| {
            let pos = &predicted_positions[index];
            let origin_cell = get_cell(&predicted_positions[index], settings.smoothing_radius);
            let mut density = 0.0;
            let mut near_density = 0.0;
            let sqr_radius = settings.smoothing_radius * settings.smoothing_radius;

            for i in 0..9 {
                let hash = hash_cell(origin_cell + CELL_OFFSETS[i]);
                let key = key_from_hash(hash, number_of_particles);
                let mut curr_index = spatial_offsets[key];

                while curr_index < number_of_particles {
                    let sorted_slot = curr_index;
                    curr_index += 1;
                    let neighbour_key = spatial_keys[sorted_slot];
                    if neighbour_key != key {
                        break;
                    }

                    let neighbour_index = spatial_indices[sorted_slot];
                    let neighbour_pos = predicted_positions[neighbour_index];
                    let offset_to_neighbour = neighbour_pos - pos;
                    let distance_to_neighbour = offset_to_neighbour.dot(offset_to_neighbour);

                    if distance_to_neighbour > sqr_radius {
                        continue;
                    }

                    let distance = distance_to_neighbour.sqrt();
                    density += density_kernel(distance, settings.smoothing_radius);
                    near_density += near_density_kernel(distance, settings.smoothing_radius);
                }
            }

            *densities = (density, near_density);
        });
}

fn apply_pressure(
    time: Res<Time<Fixed>>,
    settings: Res<FluidSettings>,
    mut buffer: ResMut<FluidBuffer>,
) {
    let dt = time.delta_secs();

    let FluidBuffer {
        velocities,
        densities,
        predicted_positions,
        spatial_offsets,
        spatial_keys,
        spatial_indices,
        ..
    } = &mut *buffer;

    let number_of_particles = settings.number_of_particles as usize;

    velocities
        .par_iter_mut()
        .enumerate()
        .for_each(|(index, velocity)| {
            let (density, near_density) = densities[index];
            let pressure = pressure_from_density(
                density,
                settings.target_density,
                settings.pressure_multiplier,
            );
            let near_pressure =
                near_pressure_from_density(near_density, settings.near_pressure_multiplier);

            let pos = &predicted_positions[index];
            let origin_cell = get_cell(&predicted_positions[index], settings.smoothing_radius);
            let sqr_radius = settings.smoothing_radius * settings.smoothing_radius;

            let mut pressure_force: Vec2 = Vec2::ZERO;

            for i in 0..9 {
                let hash = hash_cell(origin_cell + CELL_OFFSETS[i]);
                let key = key_from_hash(hash, number_of_particles);
                let mut curr_index = spatial_offsets[key];

                while curr_index < number_of_particles {
                    let sorted_slot = curr_index;
                    curr_index += 1;
                    let neighbour_key = spatial_keys[sorted_slot];
                    if neighbour_key != key {
                        break;
                    }

                    let neighbour_index = spatial_indices[sorted_slot];
                    if neighbour_index == index {
                        continue;
                    }

                    let neighbour_pos = predicted_positions[neighbour_index];
                    let offset_to_neighbour = neighbour_pos - pos;
                    let distance_to_neighbour = offset_to_neighbour.dot(offset_to_neighbour);

                    if distance_to_neighbour > sqr_radius {
                        continue;
                    }

                    let distance = distance_to_neighbour.sqrt();
                    let direction_to_neighbour = if distance > 0.0001 {
                        offset_to_neighbour / distance
                    } else {
                        Vec2::Y
                    };

                    let (neighbour_density, neighbour_near_density) = densities[neighbour_index];
                    let neighbour_pressure = pressure_from_density(
                        neighbour_density,
                        settings.target_density,
                        settings.pressure_multiplier,
                    );
                    let neighbour_near_pressure = near_pressure_from_density(
                        neighbour_near_density,
                        settings.near_pressure_multiplier,
                    );

                    let shared_pressure = (pressure + neighbour_pressure) * 0.5;
                    let shared_near_pressure = (near_pressure + neighbour_near_pressure) * 0.5;

                    pressure_force += direction_to_neighbour
                        * density_derivative(distance, settings.smoothing_radius)
                        * shared_pressure
                        / neighbour_density;
                    pressure_force += direction_to_neighbour
                        * near_density_derivative(distance, settings.smoothing_radius)
                        * shared_near_pressure
                        / neighbour_near_density;
                }
            }

            let acceleration = pressure_force / density;
            *velocity += acceleration * dt;
        });
}

fn apply_viscosity(
    time: Res<Time<Fixed>>,
    settings: Res<FluidSettings>,
    mut buffer: ResMut<FluidBuffer>,
) {
    let dt = time.delta_secs();

    let FluidBuffer {
        velocities,
        predicted_positions,
        spatial_offsets,
        spatial_keys,
        spatial_indices,
        ..
    } = &mut *buffer;

    let velocities_snapshot = velocities.clone();
    let number_of_particles = settings.number_of_particles as usize;

    velocities
        .par_iter_mut()
        .enumerate()
        .for_each(|(index, velocity)| {
            let pos = &predicted_positions[index];
            let origin_cell = get_cell(&predicted_positions[index], settings.smoothing_radius);
            let sqr_radius = settings.smoothing_radius * settings.smoothing_radius;

            let mut viscosity_force = Vec2::ZERO;

            for i in 0..9 {
                let hash = hash_cell(origin_cell + CELL_OFFSETS[i]);
                let key = key_from_hash(hash, number_of_particles);
                let mut curr_index = spatial_offsets[key];

                while curr_index < number_of_particles {
                    let sorted_slot = curr_index;
                    curr_index += 1;

                    let neighbour_key = spatial_keys[sorted_slot];
                    if neighbour_key != key {
                        break;
                    }

                    let neighbour_index = spatial_indices[sorted_slot];
                    if neighbour_index == index {
                        continue;
                    }

                    let neighbour_pos = predicted_positions[neighbour_index];
                    let offset_to_neighbour = neighbour_pos - pos;
                    let distance_to_neighbour = offset_to_neighbour.dot(offset_to_neighbour);

                    if distance_to_neighbour > sqr_radius {
                        continue;
                    }

                    let distance = distance_to_neighbour.sqrt();
                    let neighbour_velocity = velocities_snapshot[neighbour_index];
                    viscosity_force += (neighbour_velocity - *velocity)
                        * viscosity_kernel(distance, settings.smoothing_radius);
                }
            }

            *velocity += viscosity_force * settings.viscosity_strength * dt;
        });
}

fn update_positions(time: Res<Time<Fixed>>, mut buffer: ResMut<FluidBuffer>) {
    let dt = time.delta_secs();

    let FluidBuffer {
        positions,
        velocities,
        ..
    } = &mut *buffer;

    positions
        .par_iter_mut()
        .zip(velocities)
        .for_each(|(position, velocity)| {
            *position += *velocity * dt;
        });
}

fn compute_particle_colors(
    settings: Res<FluidSettings>,
    lookup: Res<GradientLookup>,
    mut buffer: ResMut<FluidBuffer>,
) {
    let max_speed = settings.max_speed;

    let FluidBuffer {
        colors, velocities, ..
    } = &mut *buffer;

    colors
        .par_iter_mut()
        .zip(velocities)
        .for_each(|(color, velocity)| {
            let speed = velocity.length();
            let t = (speed / max_speed).clamp(0.0, 1.0);

            let table_index = (t * 255.0) as usize;
            *color = lookup.table[table_index];
        });
}

fn resolve_collisions(
    settings: Res<FluidSettings>,
    spawner: Res<SpawnerSettings>,
    mut buffer: ResMut<FluidBuffer>,
) {
    let ppm = settings.pixels_per_meter;
    let radius = settings.thickness / ppm;

    let FluidBuffer {
        positions,
        velocities,
        region_indices,
        ..
    } = &mut *buffer;

    let regions = &spawner.spawn_regions;

    positions
        .par_iter_mut()
        .zip(velocities)
        .zip(region_indices)
        .for_each(|((position, velocity), region_index)| {
            let region = &regions[*region_index];

            let screen_width = region.size.x / ppm;
            let screen_height = region.size.y / ppm;
            let center = region.position / ppm;

            let half_w = screen_width / 2.0;
            let half_h = screen_height / 2.0;

            let min_x = center.x - half_w + radius;
            let max_x = center.x + half_w - radius;
            let min_y = center.y - half_h + radius;
            let max_y = center.y + half_h - radius;

            if position.x < min_x {
                position.x = min_x;
                velocity.x *= -1.0 * settings.collision_dampening;
            } else if position.x > max_x {
                position.x = max_x;
                velocity.x *= -1.0 * settings.collision_dampening;
            }

            if position.y < min_y {
                position.y = min_y;
                velocity.y *= -1.0 * settings.collision_dampening;
            } else if position.y > max_y {
                position.y = max_y;
                velocity.y *= -1.0 * settings.collision_dampening;
            }

            // let max_speed = settings.max_speed;
            // if velocity.length_squared() > max_speed * max_speed {
            //     *velocity = velocity.normalize() * max_speed;
            // }
        });
}
