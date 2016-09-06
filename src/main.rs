#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate glium;
#[macro_use] extern crate log;
extern crate cgmath;
extern crate core;
extern crate env_logger;
extern crate term_painter;


mod app;
mod camera;
mod control;
mod errors;
mod event;
mod mesh;
mod util;


fn main() {
    use app::App;
    use std::cmp::min;
    use term_painter::Color::*;
    use term_painter::ToStyle;

    // Init logger implementation
    env_logger::init().expect("failed to initialize logger");

    // Create whole app and run it, if it succeeds
    let res = App::init().and_then(|mut app| app.run());

    // Pretty print error chain
    if let Err(error_chain) = res {
        println!("Something went wrong ☹ ! Here is the backtrace:");
        for (i, e) in error_chain.iter().enumerate() {
            println!(
                "{: >2$} {}",
                Yellow.paint(if i == 0 { "→" } else { "⤷" }),
                Red.paint(e),
                2 * min(i, 7) + 1,
            );
        }
    }
}
