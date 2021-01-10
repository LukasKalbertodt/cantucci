use winit::event::{KeyboardInput, VirtualKeyCode, WindowEvent};


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
    fn handle_event(&mut self, e: &WindowEvent) -> EventResponse;
}

impl<F: FnMut(&WindowEvent) -> EventResponse> EventHandler for F {
    fn handle_event(&mut self, e: &WindowEvent) -> EventResponse {
        self(e)
    }
}

/// Handler that handles events intended to quit the program.
pub struct QuitHandler;

impl EventHandler for QuitHandler {
    fn handle_event(&mut self, e: &WindowEvent) -> EventResponse {
        let should_quit = matches!(e,
            WindowEvent::CloseRequested
                | WindowEvent::Destroyed
                | WindowEvent::KeyboardInput { input: KeyboardInput {
                virtual_keycode: Some(VirtualKeyCode::Escape),
                ..
            }, ..}
        );

        if should_quit {
            EventResponse::Quit
        } else {
            EventResponse::NotHandled
        }
    }
}

// Given a list of event handlers, pull new events from the window and let
// the handlers handle those events.
pub(crate) fn handle_with(
    event: &WindowEvent,
    handlers: &mut [&mut dyn EventHandler]
) -> EventResponse {
    // We need to check if we handled at least one event
    let mut handled_at_least_one = false;

    for handler in handlers.iter_mut() {
        let response = handler.handle_event(&event);

        if response != EventResponse::NotHandled {
            handled_at_least_one = true;
        }

        match response {
            EventResponse::NotHandled | EventResponse::Continue => (),
            EventResponse::Break => break,
            EventResponse::Quit => return EventResponse::Quit,
        }
    }

    if handled_at_least_one {
        EventResponse::Continue
    } else {
        EventResponse::NotHandled
    }
}
