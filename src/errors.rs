use glium;
use std::io;

error_chain! {
    foreign_links {
        io::Error, Io;

        // glium stuff
        glium::GliumCreationError<glium::glutin::CreationError>, GliumCreation;
        glium::ProgramCreationError, GliumProgramCreation;
    }
}
