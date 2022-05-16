use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Instant;
use std::{thread, time::Duration};
use crate::{cpu, font};
use crate::font_bitmap;

pub struct Canvas {
    size: usize,
    bitmap: Arc<Mutex<Vec<Vec<u8>>>>,
    threads: Vec<JoinHandle<()>>
}

impl Canvas {
    /// Returns the normalized value (0-100)
    fn normalize(val: u8) -> u8 {
        ((val as f64 / u8::MAX as f64) * 100.0) as u8
    }

    fn get_pixel_value_by_id(id: usize, bitmap: MutexGuard<Vec<Vec<u8>>>) -> u8 {
        let width = bitmap.len();
        let row = id / width;
        let col = id - (row * width);
        return bitmap[row][col];
    }

    fn spawn_worker(id: usize, mutex: Arc<Mutex<Vec<Vec<u8>>>>) -> JoinHandle<()> {
        thread::spawn(move || {
            println!("Spawning thread for {}, ", id);
            cpu::set_thread_affinity(id);

            loop {
                let now = Instant::now();
                let val = Canvas::get_pixel_value_by_id(id, mutex.lock().unwrap());
                let val_norm = Canvas::normalize(val);
                let sleep_ms = 100 - val_norm;
                while now.elapsed() < Duration::from_millis((val_norm) as u64) {}
                thread::sleep(Duration::from_millis(sleep_ms as u64));
            }
        })
    }

    pub fn new(size: usize) -> Canvas {
        Canvas {
            size,
            bitmap: Arc::new(Mutex::new(vec![vec![u8::MAX; size]; size])),
            threads: vec![]
        }
    }

    pub fn draw_pixel(&mut self, x: usize , y: usize, val: u8){
        self.bitmap.lock().unwrap()[x][y] = val;
    }

    pub fn print_string(&mut self, string: &str, interval_ms: u64) {
        for c in string.chars() {
            self.print_char(c);
            thread::sleep(Duration::from_millis(interval_ms));
        }
    }
}

impl Canvas {

    pub fn init(&mut self){
        for id in 0..self.size*self.size {
            self.threads.push(Canvas::spawn_worker(id,
                                                   self.bitmap.clone()));
        }

        let step = (u8::MAX as f64 / (self.size * self.size) as f64) as f64;
        let mut idx = 0;
        for r in 0..self.size {
            for c in 0..self.size {
                let val = (step * idx as f64) as u8;
                println!("Bitmap {};{} value {} (max {}, idx {}, step {})", r, c, val, u8::MAX, idx, step);
                self.bitmap.lock().unwrap()[r][c] = val;
                idx += 1;
            }
        }
    }

    fn print_char(&mut self, c: char) {
        let size = self.size;
        let val = c as usize;
        println!("Printing '{}' value '{}'", c, val);
        let chr_bitmap = font_bitmap[val];
        let mut bitmap = vec![vec![0u8; size]; size];
        let mut row_count = 0;
        for b in chr_bitmap {
            let binary_str = format!("{:010b}", b).chars().rev().collect::<String>();
            let binary_str = &binary_str[..10];
            println!("{}", binary_str);
            let row = binary_str.split("");
            let mut col_count = 0;
            for r in row.filter(|c| *c == "1" || *c == "0") {
                bitmap[row_count][col_count] = if r == "1" { u8::MAX } else { 0u8 };
                col_count += 1;
            }
            row_count += 1;
        }
        self.print_bitmap(bitmap);
    }

    fn print_bitmap(&mut self, myvec: Vec<Vec<u8>>){
        for (ridx, row) in myvec.iter().enumerate() {
            for (cidx, y) in row.iter().enumerate() {
                self.draw_pixel(ridx, cidx, *y);
            }
        }
    }
}