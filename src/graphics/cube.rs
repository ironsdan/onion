use glam::{Mat4, Vec3, Vec4};

use super::vertex::Vertex;
use rand::Rng;

const TRIANGLE_LIST_UNIT_CUBE: [Vec3; 36] = [
    // Start Left
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    // End Left
    // Start Back
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(-1.0, -1.0, -1.0),
    // End Back
    // Start Bottom
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(-1.0, -1.0, -1.0),
    // End Bottom
    // Start Front
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    // End Front
    // Start Right
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    // End Right
    // Start Top
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    // End Top
];

pub struct Cube {
    model: Mat4,
}

impl Cube {
    pub fn new() -> Self {
        let model = Mat4::IDENTITY;
        Cube { model }
    }

    pub fn translate_x(&mut self, amount: f32) {
        let translation = Mat4::from_translation(Vec3::new(amount, 0.0, 0.0));
        self.model = self.model * translation;
    }

    pub fn translate_y(&mut self, amount: f32) {
        let translation = Mat4::from_translation(Vec3::new(0.0, amount, 0.0));
        self.model = self.model * translation;
    }

    pub fn translate_z(&mut self, amount: f32) {
        let translation = Mat4::from_translation(Vec3::new(0.0, 0.0, amount));
        self.model = self.model * translation;
    }

    pub fn scale(&mut self, amount: f32) {
        let scale = Mat4::from_scale(Vec3::new(amount, amount, amount));
        self.model = self.model * scale;
    }

    pub fn to_vec(&self) -> Vec<Vertex> {
        let mut rng = rand::thread_rng();
        let mut v = Vec::new();
        let mut i = 0;
        let mut color: [f32; 3] = [
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        ];
        for t in TRIANGLE_LIST_UNIT_CUBE {
            let f = self.model * Vec4::new(t.x, t.y, t.z, 1.0);
            v.push(Vertex {
                position: [f.x, f.y, f.z],
                color: color,
                ..Default::default()
            });
            i += 1;
            if i % 6 == 0 {
                i = 0;
                color = [
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                ]
            }
        }

        return v;
    }
}
