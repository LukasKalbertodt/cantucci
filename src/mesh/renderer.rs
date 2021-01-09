use glium::backend::Facade;
use glium::Program;

use errors::*;
use shape::Shape;
use util::gl;

/// Stores resources that needs to be loaded only once.
///
/// This type works hand in hand with `View`: the view contains all object
/// specific data (like vertex and index buffers), while this renderer stores
/// all things that need to be created only once (like the OpenGL program or
/// textures).
pub struct Renderer {
    program: Program,
}

impl Renderer {
    /// Creates all global resources (currently the OpenGL program only).
    pub fn new<F: Facade>(facade: &F, shape: &dyn Shape) -> Result<Self> {
        let program = gl::load_program_with_shape(facade, "iso-surface", shape)
            .chain_err(|| "loading program for shape renderer failed")?;

        Ok(Renderer {
            program: program,
        })

    }

    pub fn program(&self) -> &Program {
        &self.program
    }
}
