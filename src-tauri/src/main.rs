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

use std::{fs::{create_dir_all, File}, sync::{Mutex, MutexGuard}, path::PathBuf, io::Read};

use constants::APP_DATA_FOLDER;
use error::Error;
use program::{Application, DictID, UserID};
use tauri::{Manager, api::path::data_dir};
use words::for_frontend::Word;

static APP: Mutex<Option<Application>> = Mutex::new(None);

fn get_app() -> MutexGuard<'static, Option<Application>> {
    APP.lock().unwrap()
}

fn init_app(app: Application) {
    *APP.lock().unwrap() = Some(app);
}

#[tauri::command]
fn import_dict(filename: String) -> Result<(), String> {
    match _import_dict(filename) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)?
    }
}

fn _import_dict(filename: String) -> Result<(), Error> {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    let mut path = PathBuf::new();
    path.push(filename);

    if !path.is_file() {
        return Err("Must be file!")?;
    }

    match path.extension() {
        Some(o) => match o.to_str() {
            Some("dct") => {
                let mut to = app.get_dict_dir();
                to.push(path.file_name().unwrap());
                std::fs::copy(path, app.get_dict_dir())?;
            },
            Some("xml") => {
                let file = File::open(&path)?;
                let dict = tools::xml::parse_xml_dictionary(file, words::ObscurityMode::Linear(1f64))?;

                let mut dct_path = PathBuf::new();
                dct_path.push(app.get_dict_dir());

                let new_name = path.file_stem().unwrap().to_str().unwrap().to_owned() + ".dct";
                dct_path.push(new_name);

                let mut dct_file = File::create(dct_path)?;
                dict.save_to(&mut dct_file)?;
            },
            _ => return Err("Invalid file type!")?
        },
        None => ()
    }

    Ok(())
}

#[tauri::command]
fn reload_files() {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();
    
    app.load(None).unwrap();
}

#[tauri::command]
fn get_dict_list() -> Box<[DictID]> {
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
fn get_users() -> Box<[UserID]> {
    let mtx = get_app();
    let app = mtx.as_ref().unwrap();

    app.get_users()
}

#[tauri::command]
fn set_current_user(user: UserID) -> Result<(), String> {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    Ok(app.set_current_user(Some(user)))
}

#[tauri::command]
fn get_current_user() -> Option<UserID> {
    let mtx = get_app();
    let app = mtx.as_ref().unwrap();

    app.get_current_user()
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

    let last_user_path = {let mut t = base_path.clone(); t.push("lastuser"); t};

    let last_user = if last_user_path.exists() && last_user_path.is_file() {
        let mut file = File::open(last_user_path).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        app.get_user_id(String::from_utf8(bytes).unwrap())
    } else {
        None
    };

    app.set_current_user(last_user);
    init_app(app);
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![get_dict_list, set_dict, pick_next_word, get_current_word, get_users, create_user, reload_files, import_dict, set_current_user, get_current_user])
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            main_window.set_decorations(false).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
