use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::Event;


/// Every event receiver has to return a response for each event received.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventResponse {
    /// The event was not handled at all.
    NotHandled,
    /// The event was handled but should be forwarded to other receivers, too.
    Continue,
    /// The event was handled and should *not* be forwarded to other receivers.
    Break,
    /// In response to the event, the program should terminate.
    Quit,
}

/// Ability to handle and react to to certain input events.
pub trait EventHandler {
    fn handle_event(&mut self, e: &Event) -> EventResponse;
}

impl<F: FnMut(&Event) -> EventResponse> EventHandler for F {
    fn handle_event(&mut self, e: &Event) -> EventResponse {
        self(e)
    }
}

/// Handler that handles events intended to quit the program.
pub struct QuitHandler;

impl EventHandler for QuitHandler {
    fn handle_event(&mut self, e: &Event) -> EventResponse {
        use glium::glutin::VirtualKeyCode;

        match *e {
            Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) |
                Event::Closed => EventResponse::Quit,
            _ => EventResponse::NotHandled,
        }
    }
}

// Given a list of event handlers, pull new events from the window and let
// the handlers handle those events.
pub fn poll_events_with(
    facade: &GlutinFacade,
    handlers: &mut [&mut dyn EventHandler]
) -> EventResponse {
    // We need to check if we handled at least one event
    let mut handled_at_least_one = false;

    for ev in facade.poll_events() {
        for handler in handlers.iter_mut() {
            let response = handler.handle_event(&ev);

            if response != EventResponse::NotHandled {
                handled_at_least_one = true;
            }

            match response {
                EventResponse::NotHandled | EventResponse::Continue => (),
                EventResponse::Break => break,
                EventResponse::Quit => return EventResponse::Quit,
            }
        }
    }

    if handled_at_least_one {
        EventResponse::Continue
    } else {
        EventResponse::NotHandled
    }
}
