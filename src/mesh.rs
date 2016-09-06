use glium::{self, DepthTest, VertexBuffer, IndexBuffer, Program, Surface, DrawParameters};
use glium::backend::Facade;
use glium::index::PrimitiveType;
use camera::Camera;
use to_arr::ToArr;
use core::math::*;

pub struct FractalMesh {
    buffer: Vec<(VertexBuffer<Vertex>, IndexBuffer<u32>)>,
    program: Program,
}

impl FractalMesh {
    pub fn new<F: Facade>(facade: &F) -> Self {
        // Create and fill vertex buffer
        let raw_vbuf =  [
            Vertex { position: [-1.0, -1.0,  0.0], color: [1.0, 0.0, 0.0] },
            Vertex { position: [ 0.0,  1.0,  0.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0,  0.0,  0.0], color: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.0,  1.0,  0.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0,  0.0,  0.0], color: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.0,  0.0, -1.0], color: [1.0, 1.0, 0.0] },
        ];
        let vbuf = VertexBuffer::new(facade, &raw_vbuf).unwrap();

        // Create and fill index buffer
        let raw_ibuf = [0, 1, 2, 3, 4, 5];
        let ibuf = IndexBuffer::new(
            facade,
            PrimitiveType::TrianglesList,
            // PrimitiveType::Points,
            &raw_ibuf
        ).unwrap();

        // Create program
        let vertex_shader_src = r#"
            #version 400
            uniform dmat4 view_matrix;
            uniform dmat4 proj_matrix;

            out float z;
            out vec3 ocolor;

            in vec3 position;
            in vec3 color;
            void main() {
                z = position.z;
                ocolor = color;

                gl_Position = vec4(
                    proj_matrix *
                    view_matrix *
                    vec4(position, 1.0)
                );
            }
        "#;

        let fragment_shader_src = r#"
            #version 140
            out vec4 color;
            in vec3 ocolor;
            in float z;
            void main() {
                color = vec4(ocolor, 1.0);
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

    pub fn draw<S: Surface>(&self, surface: &mut S, camera: &Camera) {
        let uniforms = uniform! {
            view_matrix: camera.view_transform().to_arr(),
            proj_matrix: camera.proj_transform().to_arr(),
        };

        // println!("----------");
        // println!("view: {:?}", camera.view_transform());
        println!("proj: {:?}", camera.proj_transform());
        // println!("{:?}", camera.view_transform() * Vector4::new(-0.5,  -0.5, 0.0, 1.0));

        let points = [
            Vector4::new(-1.0, -1.0,  0.0, 1.0),
            Vector4::new( 0.0,  1.0,  0.0, 1.0),
            Vector4::new( 1.0,  0.0,  0.0, 1.0),
            Vector4::new( 0.0,  1.0,  0.0, 1.0),
            Vector4::new( 1.0,  0.0,  0.0, 1.0),
            Vector4::new( 0.0,  0.0, -1.0, 1.0),
        ];

        for &p in &points {
            println!("{:?} ===> {:?}", p, camera.view_transform() * p);
        }


        let params = DrawParameters {
            point_size: Some(20.0),
            depth: glium::Depth {
                write: true,
                test: DepthTest::IfLess,
                .. Default::default()
            },
            backface_culling: ::glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
            .. DrawParameters::default()
        };

        for &(ref vbuf, ref ibuf) in &self.buffer {
            surface.draw(
                vbuf,
                ibuf,
                &self.program,
                &uniforms,
                &params,
            );

        }
    }
}


#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, color);
