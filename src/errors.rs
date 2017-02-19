use glium;
use std::io;

error_chain! {
    foreign_links {
        io::Error, Io;

        // glium stuff
        glium::GliumCreationError<glium::glutin::CreationError>, GliumCreation;
        glium::ProgramCreationError, GliumProgramCreation;
        glium::vertex::BufferCreationError, VertexBufferCreation;
        glium::index::BufferCreationError, IndexBufferCreation;
        glium::DrawError, DrawError;
    }
}
