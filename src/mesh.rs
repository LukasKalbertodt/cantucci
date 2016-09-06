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
        fn is_in_set(mut p: Point3f) -> bool {
            ::std::mem::swap(&mut p.y, &mut p.z);
            let mut z = p;
            let mut r = 0.0;

            const MAX_ITERS: u32 = 10;
            const BAILOUT: f64 = 2.5;
            const POWER: f64 = 8.0;

            for i in 0..MAX_ITERS {

                r = z.to_vec().magnitude();
                if r > BAILOUT {
                    return false;
                }

                // convert to polar coordinates
                let theta = (z.z / r).acos();
                let phi = f64::atan2(z.y, z.x);

                // scale and rotate the point
                let zr = r.powf(POWER);
                let theta = theta * POWER;
                let phi = phi * POWER;

                // convert back to cartesian coordinates
                z = zr * Point3f::new(
                    theta.sin() * phi.cos(),
                    phi.sin() * theta.sin(),
                    theta.cos()
                );
                z += p.to_vec();
            }

            true
        }

        let mut raw_vbuf = Vec::new();
        const RES: i32 = 50;
        for x in -RES..RES {
            for y in -RES..RES {
                for z in -RES..RES {
                    let p = Point3::new(
                        (x as f64) / (RES as f64),
                        (y as f64) / (RES as f64),
                        (z as f64) / (RES as f64)
                    );
                    let m = (p.to_vec().magnitude() as f32).powf(8.0);
                    if is_in_set(p) {
                        raw_vbuf.push(Vertex {
                            position: [p.x as f32, p.y as f32, p.z as f32],
                            color: [m, m, m],
                        });
                    }
                }
            }
        }

        // Create and fill vertex buffer
        // let raw_vbuf =  [
        //     Vertex { position: [-1.0, -1.0,  0.0], color: [1.0, 0.0, 0.0] },
        //     Vertex { position: [ 0.0,  1.0,  0.0], color: [0.0, 1.0, 0.0] },
        //     Vertex { position: [ 1.0,  0.0,  0.0], color: [0.0, 0.0, 1.0] },
        //     Vertex { position: [ 0.0,  1.0,  0.0], color: [0.0, 1.0, 0.0] },
        //     Vertex { position: [ 1.0,  0.0,  0.0], color: [0.0, 0.0, 1.0] },
        //     Vertex { position: [ 0.0,  0.0, -1.0], color: [1.0, 1.0, 0.0] },
        // ];
        let vbuf = VertexBuffer::new(facade, &raw_vbuf).unwrap();
        println!("{:?}", vbuf.len());

        // Create and fill index buffer
        // let raw_ibuf = [0, 1, 2, 3, 4, 5];
        let raw_ibuf: Vec<_> = (0..raw_vbuf.len() as u32).collect();
        let ibuf = IndexBuffer::new(
            facade,
            // PrimitiveType::TrianglesList,
            PrimitiveType::Points,
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
        // println!("proj: {:?}", camera.proj_transform());
        // println!("{:?}", camera.view_transform() * Vector4::new(-0.5,  -0.5, 0.0, 1.0));



        let params = DrawParameters {
            point_size: Some(2.0),
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
