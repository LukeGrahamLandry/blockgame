use blockgame::State;
use blockgame::window::{App, WindowContext};

fn main() {
    println!("file:///{}", concat!(env!("OUT_DIR"), "/gen.rs"));
    env_logger::init();
    pollster::block_on(WindowContext::run(State::new));
}
