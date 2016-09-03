use glium::{VertexBuffer, IndexBuffer, Program, Surface, DrawParameters};
use glium::backend::Facade;
use glium::index::PrimitiveType;

pub struct FractalMesh {
    buffer: Vec<(VertexBuffer<Vertex>, IndexBuffer<u32>)>,
    program: Program,
}

impl FractalMesh {
    pub fn new<F: Facade>(facade: &F) -> Self {
        // Create and fill vertex buffer
        let raw_vbuf =  [
            Vertex { position: [-0.5,  -0.5, 0.0] },
            Vertex { position: [ 0.0,   0.5, 0.0] },
            Vertex { position: [ 0.5, -0.25, 0.0] },
        ];
        let vbuf = VertexBuffer::new(facade, &raw_vbuf).unwrap();

        // Create and fill index buffer
        let raw_ibuf = [0, 1, 2];
        let ibuf = IndexBuffer::new(
            facade,
            PrimitiveType::TrianglesList,
            &raw_ibuf
        ).unwrap();

        // Create program
        let vertex_shader_src = r#"
            #version 140
            in vec3 position;
            void main() {
                gl_Position = vec4(position, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140
            out vec4 color;
            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        let program = Program::from_source(
            facade,
            vertex_shader_src,
            fragment_shader_src,
        None).unwrap();

        FractalMesh {
            buffer: vec![(vbuf, ibuf)],
            program: program,
        }
    }

    pub fn draw<S: Surface>(&self, surface: &mut S) {
        for &(ref vbuf, ref ibuf) in &self.buffer {
            surface.draw(
                vbuf,
                ibuf,
                &self.program,
                &uniform!{},
                &DrawParameters::default(),
            );

        }
    }
}


#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);
