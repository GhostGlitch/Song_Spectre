use std::thread;
use std::time::Duration;
pub mod debug;


pub fn slp (secs: f32) {
    thread::sleep(Duration::from_secs_f32(secs));
} 