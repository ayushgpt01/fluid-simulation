use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct World {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new() -> World {
        World {
            x: 50.0,
            y: 50.0,
            vx: 0.0,
            vy: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let gravity = 500.0;

        // apply gravity
        self.vy += gravity * dt;

        // update position
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // simple floor collision
        if self.y > 300.0 {
            self.y = 300.0;
            self.vy *= -0.6; // bounce
        }
    }

    pub fn apply_force(&mut self, fx: f32, fy: f32) {
        self.vx += fx;
        self.vy += fy;
    }

    pub fn x(&self) -> f32 { self.x }
    pub fn y(&self) -> f32 { self.y }
}

