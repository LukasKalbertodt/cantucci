use camera::{Camera, Projection};
use core::math::*;
use core::Shape;
use event::{EventHandler, EventResponse};
use glium::glutin::Event;
use glium::glutin::VirtualKeyCode;

mod fly;
mod orbit;

pub use self::fly::*;
pub use self::orbit::*;

/// Types that manage an internal camera and change its properties as reaction
/// to input events.
pub trait CamControl: EventHandler {
    /// Returns the internal camera
    fn camera(&self) -> Camera;

    /// Returns a mutable reference to the camera's projection properties. This
    /// should only be called to change the projection. To use the projection
    /// for rendering, call `camera()` instead.
    fn projection_mut(&mut self) -> &mut Projection;

    /// Is called regularly to update the internal camera. `delta` is the time
    /// in seconds since the last time this method was called.
    fn update(&mut self, _delta: f64, _shape: &Shape) {}

    /// Returns `self` as `EventHandler` trait object.
    fn as_event_handler(&mut self) -> &mut EventHandler;

    /// Adjusts the internal camera to match the given one as close as
    /// possible (it might not be completely possible). This is used for
    /// a smooth transition between two controls.
    ///
    /// *Note*: this only refers to the position and direction of the camera,
    /// not to projection parameters.
    fn match_view(&mut self, other: &Camera);

    /// Is called when the control is activated
    fn on_control_gain(&mut self) {}

    /// Is called when the control is deactivated
    fn on_control_loss(&mut self) {}
}

pub struct KeySwitcher<A, B> {
    first: A,
    second: B,
    switch_key: VirtualKeyCode,
    first_active: bool,

    /// How much the first camera influences the final camera (between 1.0
    /// and 0.0).
    amount_first: f64,
}

impl<A: CamControl, B: CamControl> KeySwitcher<A, B> {
    pub fn new(first: A, second: B, switch_key: VirtualKeyCode) -> Self {
        KeySwitcher {
            first: first,
            second: second,
            switch_key: switch_key,
            first_active: true,
            amount_first: 1.0,
        }
    }
}

impl<A: CamControl, B: CamControl> EventHandler for KeySwitcher<A, B> {
    fn handle_event(&mut self, e: &Event) -> EventResponse {
        use glium::glutin::ElementState;

        match e {
            &Event::KeyboardInput(ElementState::Pressed, _, Some(key))
                if key == self.switch_key =>
            {
                match self.first_active {
                    true => {
                        self.second.match_view(&self.first.camera());
                        self.second
                            .projection_mut()
                            .set_aspect_ratio_from(&self.first.camera().projection);
                        self.first.on_control_loss();
                        self.second.on_control_gain();
                    }
                    false => {
                        self.first.match_view(&self.second.camera());
                        self.first
                            .projection_mut()
                            .set_aspect_ratio_from(&self.second.camera().projection);
                        self.second.on_control_loss();
                        self.first.on_control_gain();
                    }
                }
                self.first_active = !self.first_active;

                EventResponse::Break
            }
            e if self.first_active => self.first.handle_event(e),
            e => self.second.handle_event(e),
        }
    }
}

impl<A: CamControl, B: CamControl> CamControl for KeySwitcher<A, B> {
    fn camera(&self) -> Camera {
        // FIXME: the lerped direction vector could be zero ...
        Camera::new(
            lerp(
                self.second.camera().position,
                self.first.camera().position,
                self.amount_first,
            ),
            lerp(
                self.second.camera().direction(),
                self.first.camera().direction(),
                self.amount_first,
            ),
            match self.first_active {
                true => self.first.camera().projection,
                false => self.second.camera().projection,
            }
        )
    }

    fn projection_mut(&mut self) -> &mut Projection {
        match self.first_active {
            true => self.first.projection_mut(),
            false => self.second.projection_mut(),
        }
    }

    fn update(&mut self, delta: f64, shape: &Shape) {
        const TRANSITION_DURATION: f64 = 0.3;

        self.amount_first += (delta / TRANSITION_DURATION) * if self.first_active {
            1.0
        } else {
            -1.0
        };
        self.amount_first = clamp(self.amount_first, 0.0, 1.0);

        match self.first_active {
            true => self.first.update(delta, shape),
            false => self.second.update(delta, shape),
        }
    }

    fn as_event_handler(&mut self) -> &mut EventHandler {
        self
    }

    fn match_view(&mut self, other: &Camera) {
        match self.first_active {
            true => self.first.match_view(other),
            false => self.second.match_view(other),
        }
    }
}
