use crossterm::{cursor, QueueableCommand};
use std::io::Write;
use std::time::Instant;

#[derive(PartialEq)]
pub enum TimerStatus {
    NotStart,
    Start,
    Pause,
    Resume,
    Stop,
}
pub struct Timer {
    pub start_time: Instant,
    pub pause_duration: u64,
    pub pause_time: Instant,
    pub last: Instant,
    pub is_running: bool,
}
impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
impl Timer {
    pub fn new() -> Timer {
        Timer {
            start_time: Instant::now(),
            pause_duration: 0,
            pause_time: Instant::now(),
            last: Instant::now(),
            is_running: false,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Instant::now();
        self.pause_duration = 0;
        self.is_running = true;
    }

    pub fn pause(&mut self) {
        self.pause_time = Instant::now();
        self.is_running = false;
    }

    pub fn resume(&mut self) {
        self.pause_duration += self.pause_time.elapsed().as_secs();
        self.is_running = true;
    }
    pub fn stop(&mut self) {
        self.is_running = false;
        self.pause_duration = 0;
        self.start_time = Instant::now();
        self.pause_time = Instant::now();
        self.last = Instant::now();
    }
    pub fn get_elapsed(&self) -> u64 {
        if self.is_running {
            self.start_time.elapsed().as_secs() - self.pause_duration
        } else {
            self.pause_time.duration_since(self.start_time).as_secs() - self.pause_duration
        }
    }
    // 更新耗时显示
    pub fn update_time_consuming(&self, x: u16, y: u16) {
        let mut stdout = std::io::stdout();
        stdout.queue(cursor::MoveTo(x + 10, y)).unwrap();
        print!("{}s", self.get_elapsed());
        stdout.flush().expect("Failed to flush output");
    }
}
