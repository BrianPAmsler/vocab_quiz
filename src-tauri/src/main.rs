#![cfg_attr(debug_assertions, allow(dead_code))]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod words;
mod xml;
mod constants;
mod program;
mod tools;
mod error;

use std::{fs::create_dir_all};

use program::{Application, DictID};
use tauri::{Manager};
use words::Word;

static mut APP: Option<Application> = None;

fn get_app() -> &'static Application {
    // This is only unsafe if used by multiple threads (WHICH SHOULD'NT EVER HAPPEN ANYWAY)
    unsafe {APP.as_ref().unwrap()}
}

fn get_app_mut() -> &'static mut Application {
    // This is only unsafe if used by multiple threads (WHICH SHOULD'NT EVER HAPPEN ANYWAY)
    unsafe {APP.as_mut().unwrap()}
}

fn init_app(app: Application) {
    // This is only unsafe if used by multiple threads (WHICH SHOULD'NT EVER HAPPEN ANYWAY)
    unsafe {APP = Some(app)};
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn get_dict_list<'a>() -> Box<[DictID]> {
    let app = get_app();
    
    app.get_dict_list()
}

#[tauri::command]
fn set_dict(dict: DictID) {
    let app = get_app_mut();

    app.set_dict(dict);
}

#[tauri::command]
fn pick_next_word() {
    let app = get_app_mut();

    app.pick_next_word();
}

#[tauri::command]
fn get_current_word() -> Option<Word> {
    let app = get_app_mut();

    app.get_current_word()
}

const USER_PATH: &'static str = "test/users";
const DICT_PATH: &'static str = "test/dicts";

fn main() {
    create_dir_all(USER_PATH).unwrap();
    create_dir_all(DICT_PATH).unwrap();

    let mut app = Application::new(USER_PATH, DICT_PATH).unwrap();
    app.load(None).unwrap();
    init_app(app);

    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![get_dict_list, set_dict, pick_next_word, get_current_word])
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            main_window.set_decorations(false).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
