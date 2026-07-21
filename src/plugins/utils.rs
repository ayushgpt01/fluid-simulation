use std::f32::consts::PI;

use bevy::math::{IVec2, Vec2, ivec2, ops::floor};

pub const CELL_OFFSETS: [IVec2; 9] = [
    ivec2(-1, 1),
    ivec2(0, 1),
    ivec2(1, 1),
    ivec2(-1, 0),
    ivec2(0, 0),
    ivec2(1, 0),
    ivec2(-1, -1),
    ivec2(0, -1),
    ivec2(1, -1),
];

pub fn get_cell(position: &Vec2, radius: f32) -> IVec2 {
    ivec2(
        floor(position.x / radius) as i32,
        floor(position.y / radius) as i32,
    )
}

const HASH_K1: i32 = 15823;
const HASH_K2: i32 = 9737333;

pub fn hash_cell(cell: IVec2) -> i32 {
    let x = cell.x as i32;
    let y = cell.y as i32;

    x.wrapping_mul(HASH_K1)
        .wrapping_add(y.wrapping_mul(HASH_K2))
}

pub fn key_from_hash(hash: i32, table_size: usize) -> usize {
    (hash as usize) % table_size
}

fn spiky_kernel_pow_2(distance: f32, radius: f32) -> f32 {
    if distance < radius {
        let v = radius - distance;
        let scaling_factor = 6.0 / (PI * radius.powf(4.0));
        v * v * scaling_factor
    } else {
        0.0
    }
}

fn spiky_kernel_pow_3(distance: f32, radius: f32) -> f32 {
    if distance < radius {
        let v = radius - distance;
        let scaling_factor = 10.0 / (PI * radius.powf(5.0));
        v * v * v * scaling_factor
    } else {
        0.0
    }
}

fn derivative_spiky_pow_2(distance: f32, radius: f32) -> f32 {
    if distance <= radius {
        let v = radius - distance;
        let scaling_factor = 12.0 / (radius.powf(4.0) * PI);
        -v * scaling_factor
    } else {
        0.0
    }
}

fn derivative_spiky_pow_3(distance: f32, radius: f32) -> f32 {
    if distance <= radius {
        let v = radius - distance;
        let scaling_factor = 30.0 / (radius.powf(5.0) * PI);
        -v * v * scaling_factor
    } else {
        0.0
    }
}

fn smoothing_kernel_poly_6(distance: f32, radius: f32) -> f32 {
    if distance < radius {
        let v = radius * radius - distance * distance;
        let scaling_factor = 4.0 / (PI * radius.powf(8.0));
        v * v * v * scaling_factor
    } else {
        0.0
    }
}

pub fn density_kernel(distance: f32, radius: f32) -> f32 {
    spiky_kernel_pow_2(distance, radius)
}

pub fn near_density_kernel(distance: f32, radius: f32) -> f32 {
    spiky_kernel_pow_3(distance, radius)
}

pub fn viscosity_kernel(distance: f32, radius: f32) -> f32 {
    smoothing_kernel_poly_6(distance, radius)
}

pub fn density_derivative(distance: f32, radius: f32) -> f32 {
    derivative_spiky_pow_2(distance, radius)
}

pub fn near_density_derivative(distance: f32, radius: f32) -> f32 {
    derivative_spiky_pow_3(distance, radius)
}

pub fn pressure_from_density(density: f32, target_density: f32, pressure_multiplier: f32) -> f32 {
    (density - target_density) * pressure_multiplier
}

pub fn near_pressure_from_density(near_density: f32, near_pressure_multiplier: f32) -> f32 {
    near_density * near_pressure_multiplier
}
