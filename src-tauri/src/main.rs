#![cfg_attr(debug_assertions, allow(dead_code))]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod words;
mod constants;
mod program;
mod tools;
mod error;

use std::{fs::create_dir_all, sync::{Mutex, MutexGuard}};

use constants::APP_DATA_FOLDER;
use program::{Application, DictID};
use tauri::{Manager, api::path::data_dir};
use words::for_frontend::Word;

static APP: Mutex<Option<Application>> = Mutex::new(None);

fn get_app() -> MutexGuard<'static, Option<Application>> {
    APP.lock().unwrap()
}

fn init_app(app: Application) {
    *APP.lock().unwrap() = Some(app);
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn get_dict_list<'a>() -> Box<[DictID]> {
    let mtx = get_app();
    let app = mtx.as_ref().unwrap();
    
    app.get_dict_list()
}

#[tauri::command]
fn set_dict(dict: DictID) {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    app.set_dict(dict);
}

#[tauri::command]
fn pick_next_word() {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    app.pick_next_word();
}

#[tauri::command]
fn get_current_word() -> Option<Word> {
    let mtx = get_app();
    let app = mtx.as_ref().unwrap();

    app.get_current_word()
}

#[tauri::command]
fn get_users() -> Box<[String]> {
    let mtx = get_app();
    let app = mtx.as_ref().unwrap();

    app.get_users()
}

#[tauri::command]
fn create_user(name: String) -> Result<(), String> {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    Ok(app.create_user(name)?)
}

fn main() {
    let mut base_path = data_dir().unwrap();
    base_path.push(APP_DATA_FOLDER);

    let mut user_path = base_path.clone();
    user_path.push("users");

    let mut dict_path = base_path.clone();
    dict_path.push("dicts");

    create_dir_all(&user_path).unwrap();
    create_dir_all(&dict_path).unwrap();

    let mut app = Application::new(user_path, dict_path).unwrap();
    app.load(None).unwrap();
    init_app(app);

    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![get_dict_list, set_dict, pick_next_word, get_current_word, get_users, create_user])
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            main_window.set_decorations(false).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
