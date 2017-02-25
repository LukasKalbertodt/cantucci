//! This module contains the definition of all environment things. This
//! includes the sky and sun, but possible more things in the future.
//!
//! The `Environment` type creates and handles all components.
//!

use glium::backend::Facade;

use errors::*;

/// How far the sky is from the camera. This whole environment is drawn first
/// and doesn't use any depth test. The whole environment is invariant to
/// camera position (or "moves with the camera"), so the camera can't ever
/// leave the environment. So this distance can be choosen pretty arbitrary as
/// long as it's between the near and far plane.
const SKY_DISTANCE: f32 = 10.0;

mod sky;
mod sun;

pub use self::sky::Sky;
pub use self::sun::Sun;

pub struct Environment {
    sun: Sun,
    sky: Sky,
}

impl Environment {
    // Creates a all components of the new environment.
    pub fn new<F: Facade>(facade: &F) -> Result<Self> {
        Ok(Environment {
            sun: Sun::new(facade)?,
            sky: Sky::new(facade)?,
        })
    }

    // Updates all components of the environment.
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
