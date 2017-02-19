use camera::Camera;
use core::math::*;
use errors::*;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{DrawParameters, IndexBuffer, Program, Surface, VertexBuffer};
use util::ToArr;
use util;



mod sky;
mod sun;

pub use self::sky::Sky;
pub use self::sun::Sun;

pub struct Environment {
    sun: Sun,
    sky: Sky,
}

impl Environment {
    pub fn new<F: Facade>(facade: &F) -> Result<Self> {
        Ok(Environment {
            sun: Sun::new(facade)?,
            sky: Sky::new(facade)?,
        })
    }

    pub fn update(&mut self, delta: f32) {
        self.sun.update(delta);
    }

    pub fn sun(&self) -> &Sun {
        &self.sun
    }

    pub fn sky(&self) -> &Sky {
        &self.sky
    }
}
