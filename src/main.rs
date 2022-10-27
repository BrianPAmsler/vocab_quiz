#![cfg_attr(debug_assertions, allow(dead_code))]
mod words;
mod xml;
mod constants;
mod program;
mod tools;
mod error;

use program::Application; 

fn main() {
    let app = Application::new("misc/users", "misc/dicts"); 
}
