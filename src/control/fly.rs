use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::{CursorState, Event};

use camera::{Camera, Projection};
use math::*;
use shape::Shape;
use event::{EventHandler, EventResponse};
use super::CamControl;

/// This describes the maximum speed (per seconds) in which the camera can fly
/// around
const MAX_MOVE_SPEED: f32 = 1.0;

/// This describes how slowly the maximum speed is reached. Precisely, it's
/// the time (in seconds) it takes to accelerate from speed 'x' to speed
/// '(MAX_MOVE_SPEED + x) / 2'.
const MOVE_DELAY: f32 = 0.05;

/// How much faster the move speed is when going into fast mode.
const FAST_MOVE_MULTIPLIER: f32 = 3.0;

/// Describes how much the angle of the look at vector is changed, when the
/// mouse moves one pixel. This is doubled for phi, as its range is twice as
/// big.
const TURN_PER_PIXEL: Rad<f32> = Rad(0.00025);

pub struct Fly {
    cam: Camera,

    facade: GlutinFacade,

    forward_speed: f32,
    forward_accel: f32,
    left_speed: f32,
    left_accel: f32,
    up_speed: f32,
    up_accel: f32,
    faster: bool,
}

impl Fly {
    /// Creates a new free-fly control. The facade mustn't be headless!
    pub fn new(cam: Camera, facade: &GlutinFacade) -> Self {
        assert!(facade.get_window().is_some());

        Fly {
            cam: cam,

            facade: facade.clone(),

            forward_speed: 0.0,
            forward_accel: 0.0,
            left_speed: 0.0,
            left_accel: 0.0,
            up_speed: 0.0,
            up_accel: 0.0,
            faster: false,
        }
    }

    fn set_cursor_to_center(&mut self) {
        self.facade
            .get_window()
            .and_then(|win| {
                win.get_inner_size_pixels().and_then(|(w, h)| {
                    win.set_cursor_position((w / 2) as i32, (h / 2) as i32).ok()
                })
            })
            .expect("lost window");
    }
}

impl CamControl for Fly {
    fn camera(&self) -> Camera {
        self.cam
    }

    fn projection_mut(&mut self) -> &mut Projection {
        &mut self.cam.projection
    }

    fn update(&mut self, delta: f32, shape: &dyn Shape) {
        fn update_speed(speed: &mut f32, accel: f32, delta: f32) {
            *speed = lerp(
                *speed,
                accel * MAX_MOVE_SPEED,
                1.0 - 2.0f32.powf(-delta / MOVE_DELAY),
            );
        }

        update_speed(&mut self.forward_speed, self.forward_accel, delta);
        update_speed(&mut self.left_speed, self.left_accel, delta);
        update_speed(&mut self.up_speed, self.up_accel, delta);

        let speed_multiplier = if self.faster {
            FAST_MOVE_MULTIPLIER
        } else {
            1.0
        };

        let distance_multiplier = clamp(
            2.0 * shape.min_distance_from(self.cam.position).abs(),
            0.0005,
            2.0
        );

        let up_vec = Vector3::new(0.0, 0.0, 1.0);
        let left_vec = -self.cam.direction().cross(up_vec).normalize();
        self.cam.position += distance_multiplier * speed_multiplier * delta * (
            self.cam.direction() * self.forward_speed +
            left_vec * self.left_speed +
            up_vec * self.up_speed
        );
    }

    fn as_event_handler(&mut self) -> &mut dyn EventHandler {
        self
    }

    fn match_view(&mut self, other: &Camera) {
        self.cam.position = other.position;
        self.cam.look_in(other.direction());
    }

    fn on_control_gain(&mut self) {
        self.facade
            .get_window()
            .unwrap()
            .set_cursor_state(CursorState::Hide)
            .expect("failed to set cursor state");
        self.set_cursor_to_center();
    }

    fn on_control_loss(&mut self) {
        self.facade
            .get_window()
            .unwrap()
            .set_cursor_state(CursorState::Normal)
            .expect("failed to set cursor state");
    }
}

impl EventHandler for Fly {
    fn handle_event(&mut self, e: &Event) -> EventResponse {
        match *e {
            Event::KeyboardInput(state, _, Some(key)) => {
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

                    (Pressed, Vkc::LShift) => self.faster = true,
                    (Released, Vkc::LShift) => self.faster = false,

                    _ => return EventResponse::NotHandled,
                }

                EventResponse::Break
            }
            Event::MouseMoved(x, y) => {
                // We reset the cursor to the center each time, so we have to
                // calculate the delta from the center
                let (w, h) = self.facade
                    .get_window()
                    .and_then(|w| w.get_inner_size_pixels())
                    .expect("lost window");
                let (x_center, y_center) = (w / 2, h / 2);
                let (x_diff, y_diff) = (x - (x_center as i32), y - (y_center as i32));

                let (mut theta, mut phi) = self.cam.spherical_coords();
                theta += TURN_PER_PIXEL * (y_diff as f32);
                phi += TURN_PER_PIXEL * 2.0 * (-x_diff as f32);

                self.cam.look_at_sphere(theta, phi);

                self.set_cursor_to_center();

                EventResponse::Break
            }
            _ => EventResponse::NotHandled,
        }
    }
}
