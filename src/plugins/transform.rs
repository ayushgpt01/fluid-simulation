use bevy::prelude::*;

use crate::resources::{FluidBuffer, FluidSettings, ParticleIndex};

pub struct TransformCoordinates;

impl Plugin for TransformCoordinates {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sync_transform, update_colors));
    }
}

fn sync_transform(
    settings: Res<FluidSettings>,
    buffer: Res<FluidBuffer>,
    mut query: Query<(&ParticleIndex, &mut Transform)>,
) {
    query
        .par_iter_mut()
        .for_each(|(particle_index, mut transform)| {
            if let Some(position) = buffer.positions.get(particle_index.0) {
                transform.translation.x = position.x * settings.pixels_per_meter;
                transform.translation.y = position.y * settings.pixels_per_meter;
                transform.translation.z = 0.0;
            }
        });
}

fn update_colors(settings: Res<FluidSettings>, buffer: Res<FluidBuffer>, mut gizmos: Gizmos) {
    let radius = settings.thickness;

    for (pos, color) in buffer.positions.iter().zip(&buffer.colors) {
        gizmos.circle_2d(pos * settings.pixels_per_meter, radius, *color);
    }
}
