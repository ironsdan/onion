use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Rad, Vector3, Vector4};
use glam::Mat4;

pub trait Camera {
    fn mvp_mat(&self) -> Mat4;

    fn rotate_x(&mut self, degs: Deg<f32>);

    fn rotate_y(&mut self, degs: Deg<f32>);

    fn rotate_z(&mut self, degs: Deg<f32>);

    fn translate_x(&mut self, amount: f32);

    fn translate_y(&mut self, amount: f32);

    fn translate_z(&mut self, amount: f32);
}

#[allow(unused)]
/// A model of an ideal pinhole camera that follows perspective projection.
///  
/// Useful for 3D images where perspective is necessary. The struct contains methods for doing any
/// common transformation on the camera by transforming the model, view, or projection component.
///
/// Note: Follows Vulkan tradition of x: (-1, 1), y: (-1, 1), z: (0, 1) starting at the top left-front (-1,-1, 0),
/// continuing with the consistency of Vulkan the camera looks down the POSITIVE z-direction rather than the negative
/// that is the standard in OpenGL.
///
/// Note: Default values are fov: 75, aspect_ratio: 4.0/3.0, near: 5, far: 1000.
///
/// # Examples
/// ```
/// use ledge_engine::graphics::camera;
/// use cgmath::Deg;
///
/// pub fn main() {
///     let camera = camera::PerspectiveCamera::new(75, 800.0/600.0, 5, 1000);
///     camera.rotate_x(Deg(20.0));
///     camera.translate_z(100.0);
/// }
/// ```
#[derive(Debug)]
pub struct PerspectiveCamera {
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
    model: Matrix4<f32>,
    camera: Matrix4<f32>,
    // view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        let fov: f32 = 75.0;
        let aspect_ratio = 4.0 / 3.0;
        let n = 5.0;
        let f = 1000.0;

        PerspectiveCamera::new(fov, aspect_ratio, n, f)
    }
}

impl PerspectiveCamera {
    pub fn new(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let angle_rad: Rad<f32> = Deg(fov).into();
        let focal_length = 1.0 / Rad::tan(angle_rad / 2.0);

        let c0r0 = focal_length / aspect_ratio;
        let c1r1 = -focal_length;
        let c2r2 = (far) / (far - near);
        let c3r2 = -near * c2r2;

        let proj_x = Vector4::new(c0r0, 0.0, 0.0, 0.0);
        let proj_y = Vector4::new(0.0, c1r1, 0.0, 0.0);
        let proj_z = Vector4::new(0.0, 0.0, c2r2, 1.0);
        let proj_w = Vector4::new(0.0, 0.0, c3r2, 0.0);
        // let proj = Matrix4::identity();

        let proj = Matrix4::from_cols(proj_x, proj_y, proj_z, proj_w);
        let camera = Matrix4::identity();
        let model = Matrix4::identity();
        // let view = Matrix4::identity();
        // println!("m: {:?}\nv: {:?}\np: {:?}", model, view, proj);

        Self {
            fov,
            aspect_ratio,
            near,
            far,
            model,
            // view,
            camera,
            proj,
        }
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        let angle_rad: Rad<f32> = Deg(self.fov).into();
        let focal_length = 1.0 / Rad::tan(angle_rad / 2.0);

        let c0r0 = focal_length / aspect_ratio;
        let c1r1 = -focal_length;
        let c2r2 = (self.far) / (self.far - self.near);
        let c3r2 = -self.near * c2r2;

        let proj_x = Vector4::new(c0r0, 0.0, 0.0, 0.0);
        let proj_y = Vector4::new(0.0, c1r1, 0.0, 0.0);
        let proj_z = Vector4::new(0.0, 0.0, c2r2, 1.0);
        let proj_w = Vector4::new(0.0, 0.0, c3r2, 0.0);
        // let proj = Matrix4::identity();

        let proj = Matrix4::from_cols(proj_x, proj_y, proj_z, proj_w);
        self.aspect_ratio = aspect_ratio;
        self.proj = proj;
    }
}

impl Camera for PerspectiveCamera {
    fn mvp_mat(&self) -> Mat4 {
        let view = self.camera.invert().unwrap();
        let mvp = self.proj * view * self.model;
        let t: [[f32; 4]; 4] = mvp.into();
        Mat4::from_cols_array_2d(&t)
    }

    fn rotate_x(&mut self, degs: Deg<f32>) {
        let rotation = Matrix4::from_angle_x(degs);
        self.camera = self.camera * rotation;
        println!(
            "m: {:?}\nc: {:?}\np: {:?}",
            self.model, self.camera, self.proj
        );
    }

    fn rotate_y(&mut self, degs: Deg<f32>) {
        let rotation = Matrix4::from_angle_y(degs);
        self.camera = self.camera * rotation;
        println!(
            "m: {:?}\nc: {:?}\np: {:?}",
            self.model, self.camera, self.proj
        );
    }

    fn rotate_z(&mut self, degs: Deg<f32>) {
        let rotation = Matrix4::from_angle_z(degs);
        self.camera = self.camera * rotation;
        println!(
            "m: {:?}\nc: {:?}\np: {:?}",
            self.model, self.camera, self.proj
        );
    }

    fn translate_x(&mut self, amount: f32) {
        let translation = Matrix4::from_translation(Vector3::new(amount, 0.0, 0.0));
        self.camera = self.camera * translation;
        println!(
            "m: {:?}\nc: {:?}\np: {:?}",
            self.model, self.camera, self.proj
        );
    }

    fn translate_y(&mut self, amount: f32) {
        let translation = Matrix4::from_translation(Vector3::new(0.0, amount, 0.0));
        self.camera = self.camera * translation;
        println!(
            "m: {:?}\nc: {:?}\np: {:?}",
            self.model, self.camera, self.proj
        );
    }

    fn translate_z(&mut self, amount: f32) {
        let translation = Matrix4::from_translation(Vector3::new(0.0, 0.0, amount));
        self.camera = self.camera * translation;
        println!(
            "m: {:?}\nc: {:?}\np: {:?}",
            self.model, self.camera, self.proj
        );
    }
}
