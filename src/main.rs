#![cfg_attr(debug_assertions, allow(dead_code))]
mod words;
mod xml;
mod constants;
mod program;
mod tools;
mod error;

use program::{Application, ProgressTracker, Progress};

struct TestTracker;

impl ProgressTracker for TestTracker {
    fn update_progress(&self, progress: f32) {
        println!("Progress {:.2}%", progress * 100.0);
    }
}

fn main() {
    let mut app = Application::new("misc/users", "misc/dicts").unwrap();

    let mut progress = Progress::new(0);
    progress.set_tracker(TestTracker);
    app.load(progress).unwrap();
}
