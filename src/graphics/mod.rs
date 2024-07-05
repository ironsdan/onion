pub mod camera;
pub mod context;
pub mod cube;
pub mod pipelines;
pub mod render_pass;
pub mod shape;

#[derive(Debug, Clone, Copy)]
pub struct Color([f32; 4]);

impl From<[f32; 4]> for Color {
    fn from(a: [f32; 4]) -> Color {
        Color(a)
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> [f32; 4] {
        c.0
    }
}

impl From<[f32; 3]> for Color {
    fn from(a: [f32; 3]) -> Color {
        let b = [a[0], a[1], a[2], 1.0];
        Color(b)
    }
}

impl From<Color> for [f32; 3] {
    fn from(c: Color) -> [f32; 3] {
        [c.0[0], c.0[1], c.0[2]]
    }
}

impl Color {
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color([
            r as f32 / 255.,
            g as f32 / 255.,
            b as f32 / 255.,
            a as f32 / 255.,
        ])
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color([r as f32 / 255., g as f32 / 255., b as f32 / 255., 1.0])
    }

    pub fn black() -> Color {
        Color([0.0, 0.0, 0.0, 1.0])
    }

    pub fn grey() -> Color {
        Color([0.25, 0.25, 0.25, 1.0])
    }

    pub fn white() -> Color {
        Color([1.0, 1.0, 1.0, 1.0])
    }

    pub fn red() -> Color {
        Color([1.0, 0.05, 0.05, 1.0])
    }

    pub fn transparent() -> Color {
        Color([0.0, 0.0, 0.0, 0.0])
    }

    pub fn as_u8_arr(&self) -> [u8; 4] {
        let mut arr = [0u8; 4];
        arr[0] = (self.0[0] * 255.) as u8;
        arr[1] = (self.0[1] * 255.) as u8;
        arr[2] = (self.0[2] * 255.) as u8;
        arr[3] = (self.0[3] * 255.) as u8;
        arr
    }

    pub fn as_u8_vec(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push((self.0[0] * 255.) as u8);
        v.push((self.0[1] * 255.) as u8);
        v.push((self.0[2] * 255.) as u8);
        v.push((self.0[3] * 255.) as u8);
        v
    }
}

impl Default for Color {
    fn default() -> Color {
        Color::black()
    }
}
