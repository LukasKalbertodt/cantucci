use camera::Camera;
use core::math::*;
use errors::*;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{DrawParameters, IndexBuffer, Program, Surface, VertexBuffer};
use util::ToArr;
use util;

#[derive(Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
}

implement_vertex!(Vertex, pos);

/// Represents a simple sky dome.
pub struct Sun {
    theta: Rad<f32>,
    phi: Rad<f32>,
    // vbuf: VertexBuffer<Vertex>,
    // ibuf: IndexBuffer<u8>,
    // program: Program,
}

impl Sun {
    pub fn new<F: Facade>(facade: &F) -> Result<Self> {
        // We represent the sky by a diamond shaped mesh.
        // const SIZE: f32 = 10.0;
        // let raw_vbuf = [
        //     Vertex { pos: [-SIZE,   0.0,   0.0] },  // -x
        //     Vertex { pos: [  0.0, -SIZE,   0.0] },  // -y
        //     Vertex { pos: [ SIZE,   0.0,   0.0] },  // +x
        //     Vertex { pos: [  0.0,  SIZE,   0.0] },  // +y
        //     Vertex { pos: [  0.0,   0.0, -SIZE] },  // -z
        //     Vertex { pos: [  0.0,   0.0,  SIZE] },  // +z
        // ];

        // let raw_ibuf = [
        //     // Top triangles
        //     0, 1, 5,  // -x -y +z
        //     1, 2, 5,  // +x -y +z
        //     2, 3, 5,  // +x +y +z
        //     3, 0, 5,  // -x +y +z

        //     // Bottom triangles
        //     0, 1, 4,  // -x -y -z
        //     1, 2, 4,  // +x -y -z
        //     2, 3, 4,  // +x +y -z
        //     3, 0, 4,  // -x +y -z
        // ];

        // let vbuf = VertexBuffer::new(facade, &raw_vbuf)?;
        // let ibuf = IndexBuffer::new(
        //     facade,
        //     PrimitiveType::TrianglesList,
        //     &raw_ibuf,
        // )?;

        // let program = util::gl::load_program(facade, "sky")?;

        Ok(Sun {
            theta: Rad(1.0),
            phi: Rad(2.0),
            // vbuf: vbuf,
            // ibuf: ibuf,
            // program: program,
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

    pub fn update(&mut self, delta: f32) {
        const DAY_LENGTH: f32 = 50.0;
        const YEAR_LENGTH: f32 = 100.0 * DAY_LENGTH;

        self.theta += Rad::full_turn() * (delta / DAY_LENGTH);
        self.phi += Rad::full_turn() * (delta / YEAR_LENGTH);
    }

    pub fn draw<S: Surface>(&self, surface: &mut S, camera: &Camera) -> Result<()> {
        // We discard the translation transformation from the view matrix. This
        // results in a "fixed" sky that moves with the camera.
        // let mut view_transform = camera.view_transform();
        // view_transform.w = Vector4::new(0.0, 0.0, 0.0, view_transform.w.w);

        // let uniforms = uniform! {
        //     view_matrix: view_transform.to_arr(),
        //     proj_matrix: camera.proj_transform().to_arr(),
        // };

        // surface.draw(
        //     &self.vbuf,
        //     &self.ibuf,
        //     &self.program,
        //     &uniforms,
        //     &DrawParameters::default(),
        // )?;

        Ok(())
    }
}
