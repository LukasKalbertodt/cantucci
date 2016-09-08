use camera::{Camera, Projection};
use glium::glutin::Event;
use event::{EventHandler, EventResponse};
use super::CamControl;
use core::math::*;

/// This describes the maximum speed (per seconds) in which the camera can fly
/// around
const MAX_MOVE_SPEED: f64 = 1.0;

/// This describes how slowly the maximum speed is reached. Precisely, it's
/// the time (in seconds) it takes to accelerate from speed 'x' to speed
/// '(MAX_MOVE_SPEED + x) / 2'.
const MOVE_DELAY: f64 = 0.05;

pub struct Fly {
    cam: Camera,

    forward_speed: f64,
    forward_accel: f64,
    left_speed: f64,
    left_accel: f64,
    up_speed: f64,
    up_accel: f64,
}

impl Fly {
    pub fn new(cam: Camera) -> Self {
        Fly {
            cam: cam,
            forward_speed: 0.0,
            forward_accel: 0.0,
            left_speed: 0.0,
            left_accel: 0.0,
            up_speed: 0.0,
            up_accel: 0.0,
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

    fn update(&mut self, delta: f64) {
        fn update_speed(speed: &mut f64, accel: f64, delta: f64) {
            *speed = lerp(
                *speed,
                accel * MAX_MOVE_SPEED,
                (1.0 - 2.0f64.powf(-delta / MOVE_DELAY)),
            );
        }

        update_speed(&mut self.forward_speed, self.forward_accel, delta);
        update_speed(&mut self.left_speed, self.left_accel, delta);
        update_speed(&mut self.up_speed, self.up_accel, delta);

        let up_vec = Vector3::new(0.0, 0.0, 1.0);
        let left_vec = -self.cam.direction().cross(up_vec);
        self.cam.position += delta * (
            self.cam.direction() * self.forward_speed +
            left_vec * self.left_speed +
            up_vec * self.up_speed
        );
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
                // Update accelerations
                (Pressed, Vkc::W) | (Released, Vkc::S) if self.forward_accel <= 0.0
                    => self.forward_accel += 1.0,
                (Pressed, Vkc::S) | (Released, Vkc::W) if self.forward_accel >= 0.0
                    => self.forward_accel -= 1.0,

                (Pressed, Vkc::A) | (Released, Vkc::D) if self.left_accel <= 0.0
                    => self.left_accel += 1.0,
                (Pressed, Vkc::D) | (Released, Vkc::A) if self.left_accel >= 0.0
                    => self.left_accel -= 1.0,

                (Pressed, Vkc::Space) | (Released, Vkc::LControl) if self.up_accel <= 0.0
                    => self.up_accel += 1.0,
                (Pressed, Vkc::LControl) | (Released, Vkc::Space) if self.up_accel >= 0.0
                    => self.up_accel -= 1.0,

                _ => return EventResponse::NotHandled,
            }

            EventResponse::Break
        } else {
            EventResponse::NotHandled
        }
    }
}
