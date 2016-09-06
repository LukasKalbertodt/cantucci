use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::{Event, VirtualKeyCode};


/// Every event receiver has to return a response for each event received.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum EventResponse {
    /// The event was not handled at all
    NotHandled,
    /// The event was handled but should be forwarded to other receivers, too
    Continue,
    /// The event was handled and should *not* be forwarded to other receivers
    Break,
    /// In response to the event, the program should terminate
    Quit,
}

/// Ability to handle and reacto to certain input events.
pub trait EventHandler {
    fn handle_event(&mut self, e: &Event) -> EventResponse;
}

impl<F> EventHandler for F where F: FnMut(&Event) -> EventResponse {
    fn handle_event(&mut self, e: &Event) -> EventResponse {
        self(e)
    }
}

/// Handler that handles events intended to quit the program.
pub struct CloseHandler;

impl EventHandler for CloseHandler {
    fn handle_event(&mut self, e: &Event) -> EventResponse {
        match *e {
            Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) |
                Event::Closed => EventResponse::Quit,
            _ => EventResponse::NotHandled,
        }
    }
}

pub fn poll_events_with(facade: GlutinFacade, mut handlers: Vec<&mut EventHandler>)
    -> EventResponse
{
    let mut handled_one  = false;
    for ev in facade.poll_events() {
        for i in 0..handlers.len() {
            let response = handlers[i].handle_event(&ev);

            if response != EventResponse::NotHandled {
                handled_one  = true;
            }

            match response {
                EventResponse::NotHandled | EventResponse::Continue => (),
                EventResponse::Break => break,
                EventResponse::Quit => return EventResponse::Quit,
            }
        }
    }

    if handled_one {
        EventResponse::Continue
    } else {
        EventResponse::NotHandled
    }
}
