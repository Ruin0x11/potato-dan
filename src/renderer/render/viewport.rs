use cgmath;
use glium;

#[derive(Debug)]
pub struct Viewport {
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub scale: f32,
    pub camera: (f32, f32, f32),
}

pub type RendererSubarea = ([[f32; 4]; 4], glium::Rect);

pub const FACTOR: f32 = 32.0;

impl Viewport {
    pub fn width(&self) -> u32 {
        self.size.0
    }

    pub fn height(&self) -> u32 {
        self.size.1
    }

    pub fn main_window(&self, pixels_per_unit: u32) -> RendererSubarea {
        let (w, h) = self.scaled_size();
        self.make_subarea((0, 0, w, h), pixels_per_unit)
    }

    pub fn scaled_size(&self) -> (u32, u32) {
        ((self.size.0 as f32 * self.scale) as u32, (self.size.1 as f32 * self.scale) as u32)
    }

    pub fn min_point(&self, camera: (f32, f32), pixels_per_unit: u32) -> (f32, f32) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let zoom = 1.0;
        let pixels_per_unit = pixels_per_unit as f32;

        let effective_width = w / (zoom * pixels_per_unit);
        let effective_height = h / (zoom * pixels_per_unit);
        let half_width = effective_width / 2.0;
        let half_height = effective_height / 2.0;

        (camera.0 - half_width, camera.1 - half_height)
    }

    pub fn renderable_area(&self) -> (i32, i32) {
        (self.width() as i32, self.height() as i32)
    }

    pub fn camera(&self, camera: (f32, f32)) -> (i32, i32) {
        let camera: (f32, f32) = camera.into();
        let camera = ((camera.0 * FACTOR) as i32, (camera.1 * FACTOR) as i32);
        (camera.0 - (self.width() as i32 / 2), camera.1 - (self.width() as i32 / 2))
    }

    fn make_subarea(&self, area: (u32, u32, u32, u32), pixels_per_unit: u32) -> RendererSubarea {
        (self.camera_projection(pixels_per_unit), self.scissor(area))
    }

    pub fn camera_projection(&self, pixels_per_unit: u32) -> [[f32; 4]; 4] {
        self.make_projection_matrix(pixels_per_unit, self.camera)
    }

    pub fn ui_projection(&self) -> [[f32; 4]; 4] {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);

        let left = 0.0;
        let right = w;
        let bottom = h;
        let top = 0.0;

        cgmath::ortho(left, right, bottom, top, -1.0, 1.0).into()
    }

    pub fn make_projection_matrix(&self, pixels_per_unit: u32, offset: (f32, f32, f32)) -> [[f32; 4]; 4] {
        let (x, y) = (offset.0, offset.2);
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let zoom = 1.0;
        let pixels_per_unit = pixels_per_unit as f32;

        let effective_width = w / (zoom * pixels_per_unit);
        let effective_height = h / (zoom * pixels_per_unit);
        let half_width = effective_width / 2.0;
        let half_height = effective_height / 2.0;

        //cgmath::ortho(-half_width, half_width, -half_height, half_height, -100.0, 100.0).into()
        cgmath::ortho(x - half_width, x + half_width, y + half_height, y - half_height, -100.0, 100.0).into()
    }

    fn scissor(&self, area: (u32, u32, u32, u32)) -> glium::Rect {
        let (ax, ay, aw, ah) = area;
        let (_, h) = self.scaled_size();
        let conv = |i| (i as f32 * self.scale) as u32;

        glium::Rect { left:   conv(ax),
                      bottom: conv(ay) + conv(h - ah),
                      width:  conv(aw - ax),
                      height: conv(ah) - conv(ay * 2),
        }
    }
}
