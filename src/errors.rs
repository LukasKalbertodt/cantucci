use camera;
use std::error::Error as StdError;
use glium;

error_chain! {
    foreign_links {
        glium::GliumCreationError<glium::glutin::CreationError>, GliumCreation;
    }
}
