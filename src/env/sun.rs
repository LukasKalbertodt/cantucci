use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{
    Blend,
    BlendingFunction,
    LinearBlendingFactor,
    DrawParameters,
    IndexBuffer,
    Program,
    Surface,
    VertexBuffer
};

use camera::Camera;
use core::math::*;
use errors::*;
use super::SKY_DISTANCE;
use util::{self, ToArr};

/// The size of the sun relative to the whole sky box. Note that this value is
/// far from realistic. But a realistic sun size would seem too small as humans
/// always incorrectly believe the sun is huge in the sky. The angular diameter
/// of the sun in our sky is 0.5 degrees.
const SUN_SIZE: f32 = 0.1;

/// Represents a sun floating in the sky.
pub struct Sun {
    theta: Rad<f32>,
    phi: Rad<f32>,
    vbuf: VertexBuffer<Vertex>,
    ibuf: IndexBuffer<u8>,
    program: Program,
}

impl Sun {
    /// Creates all resources to draw the sun in the sky and to keep track of
    /// the current sun position.
    pub fn new<F: Facade>(facade: &F) -> Result<Self> {
        // The sun is rendered as a simple square. The shader will paint it in
        // a way to make it look like a gloomy circle.
        let size = SKY_DISTANCE * SUN_SIZE;
        let raw_vbuf = [
            Vertex { pos: [-size, -size, SKY_DISTANCE], unit_pos: [ -1.0, -1.0] },
            Vertex { pos: [-size,  size, SKY_DISTANCE], unit_pos: [ -1.0,  1.0] },
            Vertex { pos: [ size, -size, SKY_DISTANCE], unit_pos: [  1.0, -1.0] },
            Vertex { pos: [ size,  size, SKY_DISTANCE], unit_pos: [  1.0,  1.0] },
        ];
        let raw_ibuf = [
            0, 1, 2,
            1, 2, 3,
        ];

        let vbuf = VertexBuffer::new(facade, &raw_vbuf)?;
        let ibuf = IndexBuffer::new(facade, PrimitiveType::TrianglesList, &raw_ibuf)?;
        let program = util::gl::load_program(facade, "sun")?;

        Ok(Sun {
            // This means the sun starts slightly above the horizon.
            theta: Rad(4.2),
            // This is just a meaningless value which right now makes the sun
            // appear on the screen at the very beginning.
            phi: Rad(3.8),
            vbuf: vbuf,
            ibuf: ibuf,
            program: program,
        })
    }

    /// Returns the direction of light coming from the sun (from the sun to
    /// the scene). The vector is normalized.
    pub fn light_dir(&self) -> Vector3<f32> {
        -Vector3::new(
            self.theta.sin() * self.phi.cos(),
            self.phi.sin() * self.theta.sin(),
            self.theta.cos(),
        ).normalize()
    }

    /// Updates the position of the sun according to the time that has passed.
    pub fn update(&mut self, delta: f32) {
        const DAY_LENGTH: f32 = 50.0;
        const YEAR_LENGTH: f32 = 50.0 * DAY_LENGTH;

        self.theta += Rad::full_turn() * (delta / DAY_LENGTH);
        self.phi += Rad::full_turn() * (delta / YEAR_LENGTH);
    }

    pub fn draw<S: Surface>(&self, surface: &mut S, camera: &Camera) -> Result<()> {
        // We discard the translation transformation from the view matrix. This
        // results in a "fixed" sky that moves with the camera.
        let mut view_transform = camera.view_transform();
        view_transform.w = Vector4::new(0.0, 0.0, 0.0, view_transform.w.w);

        // The sun needs to be put at the correct spot in the sky.
        let world_transform = Matrix4::from_angle_z(self.phi)
            * Matrix4::from_angle_y(self.theta);

        let uniforms = uniform! {
            world_matrix: world_transform.to_arr(),
            view_matrix: view_transform.to_arr(),
            proj_matrix: camera.proj_transform().to_arr(),
            day_theta: self.theta.0,
        };

        // We want to make the sun's edges soft, so we need alpha blending.
        let params = DrawParameters {
            blend: Blend {
                color: BlendingFunction::Addition {
                    source: LinearBlendingFactor::SourceAlpha,
                    destination: LinearBlendingFactor::OneMinusSourceAlpha,
                },
                alpha: BlendingFunction::Max,
                ..Default::default()
            },
            ..Default::default()
        };

        surface.draw(
            &self.vbuf,
            &self.ibuf,
            &self.program,
            &uniforms,
            &params,
        )?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
    unit_pos: [f32; 2],
}

implement_vertex!(Vertex, pos, unit_pos);
