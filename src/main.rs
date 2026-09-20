#![feature(default_field_values)]
#![feature(iter_collect_into)]

use roboscope_ipc::{Publisher, Subscriber, snapshot::{ControllerInput}, display::{DisplayFrame, DisplayInput}};
use winit::event_loop::EventLoop;

use crate::app::App;

pub mod app;
pub mod physics;

pub type DispInputPubType = Publisher<DisplayInput>;
pub type FrameSubType = Subscriber<DisplayFrame>;
pub type ContPubType = Publisher<ControllerInput>;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}