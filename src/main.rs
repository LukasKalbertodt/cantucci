#![feature(array_value_iter, array_map, const_in_array_repeat_expressions)]
#![feature(or_patterns)]

#![allow(dead_code)] // TODO

#[macro_use] mod util;
mod app;
mod prelude;
mod camera;
mod control;
mod event;
mod math;
mod mesh;
mod octree;
mod shape;
mod sky;
mod wgpu;


fn main() {
    // Init logger implementation
    env_logger::init();

    // Create whole app and run it, if it succeeds
    let res = futures::executor::block_on(app::run());

    // Pretty print error chain
    if let Err(e) = res {
        eprintln!("Cantucci error: {:?}", e);
    }
}
