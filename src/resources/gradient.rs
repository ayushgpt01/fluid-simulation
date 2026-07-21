use bevy::prelude::*;

#[derive(Resource)]
pub struct GradientLookup {
    pub table: [LinearRgba; 256],
}

impl Default for GradientLookup {
    fn default() -> Self {
        let mut table = [LinearRgba::BLACK; 256];
        for i in 0..256 {
            let t = i as f32 / 255.0;
            table[i] = evaluate_gradient(t);
        }

        Self { table }
    }
}

fn lerp_color(a: LinearRgba, b: LinearRgba, t: f32) -> LinearRgba {
    LinearRgba::new(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        1.0,
    )
}

fn evaluate_gradient(t: f32) -> LinearRgba {
    let t = t.clamp(0.0, 1.0);

    let c0 = LinearRgba::new(0.0, 0.1, 0.8, 1.0); // Dark Blue
    let c1 = LinearRgba::new(0.0, 0.8, 1.0, 1.0); // Cyan
    let c2 = LinearRgba::new(0.0, 0.9, 0.2, 1.0); // Green
    let c3 = LinearRgba::new(1.0, 0.9, 0.0, 1.0); // Yellow
    let c4 = LinearRgba::new(1.0, 0.1, 0.0, 1.0); // Red

    if t < 0.25 {
        let local_t = t / 0.25;
        lerp_color(c0, c1, local_t)
    } else if t < 0.50 {
        let local_t = (t - 0.25) / 0.25;
        lerp_color(c1, c2, local_t)
    } else if t < 0.75 {
        let local_t = (t - 0.50) / 0.25;
        lerp_color(c2, c3, local_t)
    } else {
        let local_t = (t - 0.75) / 0.25;
        lerp_color(c3, c4, local_t)
    }
}
