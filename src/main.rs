//#![windows_subsystem = "windows"]
#![allow(unused)]
// #![allow(unused)]
mod app;
mod misc;
mod platform;
mod support;

use app::{application::ApplicationBuilder, event::UserEvent};
use glutin::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event();
    ApplicationBuilder::new()
        .with_name("Polaroid")
        .build(&event_loop)
        .expect("failed to create application")
        .run(event_loop);
}
