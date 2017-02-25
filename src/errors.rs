use glium;
use std::io;

error_chain! {
    foreign_links {
        Io(io::Error);

        // glium stuff
        GliumCreation(glium::GliumCreationError<glium::glutin::CreationError>);
        GliumProgramCreation(glium::ProgramCreationError);
        VertexBufferCreation(glium::vertex::BufferCreationError);
        IndexBufferCreation(glium::index::BufferCreationError);
        DrawError(glium::DrawError);
    }
}
