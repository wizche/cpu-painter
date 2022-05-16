mod cpu;
mod font;
mod drawing;

use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use crate::cpu::get_num_logical_cpus_ex_windows;
use crate::drawing::Canvas;
use crate::font::font_bitmap;

fn main() {
    let cpu_count = cpu::get_num_logical_cpus_ex_windows().unwrap();
    let size = (cpu_count as f64).sqrt() as usize;
    println!("Initializing array of {}x{}", size, size);
    let mut canvas = Canvas::new(size);
    canvas.init();
    canvas.print_string("RUSTISHARD!", 1800);
}

