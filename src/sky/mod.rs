use crate::prelude::*;

/// How far the sky is from the camera. This whole environment is drawn first
/// and doesn't use any depth test. The whole environment is invariant to
/// camera position (or "moves with the camera"), so the camera can't ever
/// leave the environment. So this distance can be choosen pretty arbitrary as
/// long as it's between the near and far plane.
const SKY_DISTANCE: f32 = 10.0;

mod dome;
// mod sun;

pub use self::dome::Dome;
// pub use self::sun::Sun;

pub struct Sky {
    // sun: Sun,
    dome: Dome,
}

impl Sky {
    // Creates a all components of the new environment.
    pub fn new(device: &wgpu::Device, out_format: wgpu::TextureFormat) -> Result<Self> {
        Ok(Sky {
            // sun: Sun::new(facade)?,
            dome: Dome::new(device, out_format)?,
        })
    }

    // // Updates all components of the environment.
    // pub fn update(&mut self, delta: f32) {
    //     self.sun.update(delta);
    // }

    // pub fn sun(&self) -> &Sun {
    //     &self.sun
    // }

    pub fn dome(&self) -> &Dome {
        &self.dome
    }
}
