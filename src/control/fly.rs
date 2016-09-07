use camera::{Camera, Projection};
use glium::glutin::Event;
use event::{EventHandler, EventResponse};
use super::CamControl;

pub struct Fly {
    cam: Camera,
}

impl Fly {
    pub fn new(cam: Camera) -> Self {
        Fly {
            cam: cam,
        }
    }
}

impl CamControl for Fly {
    fn camera(&self) -> Camera {
        self.cam
    }

    fn projection_mut(&mut self) -> &mut Projection {
        &mut self.cam.projection
    }

    fn update(&mut self, _delta: f64) {
    }

    fn as_event_handler(&mut self) -> &mut EventHandler {
        self
    }

    fn match_view(&mut self, other: &Camera) {
        self.cam.position = other.position;
        self.cam.look_in(other.direction());
    }
}

impl EventHandler for Fly {
    fn handle_event(&mut self, e: &Event) -> EventResponse {
        // We are only interested in keyboard input ...
        if let Event::KeyboardInput(state, _, Some(key)) = *e {
            use glium::glutin::ElementState::*;
            use glium::glutin::VirtualKeyCode as Vkc;

            match (state, key) {
                // Update accelerations for turning
                (Pressed, Vkc::Up) => self.cam.position.z += 0.1,

                _ => return EventResponse::NotHandled,
            }

            EventResponse::Break
        } else {
            EventResponse::NotHandled
        }
    }
}
