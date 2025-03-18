#![cfg_attr(debug_assertions, allow(dead_code))]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod constants;
mod error;
mod program;
mod tools;
mod words;

use std::{
    fs::{create_dir_all, File},
    io::Read,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use constants::APP_DATA_FOLDER;
use error::Error;
use program::{Application, DictID, UserID};
use tauri::Manager;
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
        Err(e) => Err(e)?,
    }
}

fn _import_dict(filename: String) -> Result<(), Error> {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    let mut path = PathBuf::new();
    path.push(filename);

    #[cfg(mobile)]
    {
        // convert the file uri to just a path by removing the content://.../root part
        path = path.into_iter().skip(3).collect();
    }

    println!("{}", path.to_str().unwrap());
    
    if !path.is_file() {
        return Err("Must be file!")?;
    }

    match path.extension() {
        Some(o) => match o.to_str() {
            Some("dct") => {
                let mut to = app.get_dict_dir();
                to.push(path.file_name().unwrap());
                std::fs::copy(path, to)?;
            }
            Some("xml") => {
                let file = File::open(&path)?;
                let dict =
                    tools::xml::parse_xml_dictionary(file, words::ObscurityMode::Linear(1f64))?;

                let mut dct_path = PathBuf::new();
                dct_path.push(app.get_dict_dir());

                let new_name = path.file_stem().unwrap().to_str().unwrap().to_owned() + ".dct";
                dct_path.push(new_name);

                let mut dct_file = File::create(dct_path)?;
                dict.save_to(&mut dct_file)?;
            }
            Some(other) => return Err(format!("Invalid file type: {}!", other))?,
            None => return Err("Nothing??")?
        },
        None => return Err("No extension")?,
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
fn get_pool_size(dict: DictID) -> usize {
    let mtx = get_app();
    let app = mtx.as_ref().unwrap();

    app.get_pool_size(dict)
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
fn start_practice_session() -> bool {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    let r = app.start_practice_session();

    if !r {
        let words = app.get_active_words();

        app.set_active_words(words + 20);

        return app.start_practice_session();
    }

    true
}

#[tauri::command]
fn get_remaining_words() -> usize {
    let mtx = get_app();
    let app = mtx.as_ref().unwrap();

    app.get_session_len()
}

#[tauri::command]
fn practice_current_word(result: bool) {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    app.practice_current_word(result);
}

#[tauri::command]
fn conclude_session() {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    app.conclude_session();
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
fn save_current_user() -> Result<(), String> {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    app.save_current_user()?;

    Ok(())
}

#[tauri::command]
fn create_user(name: String) -> Result<(), String> {
    let mut mtx = get_app();
    let app = mtx.as_mut().unwrap();

    Ok(app.create_user(name)?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init());

    
    #[cfg(not(mobile))]
    let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .invoke_handler(tauri::generate_handler![
            get_dict_list,
            set_dict,
            pick_next_word,
            get_current_word,
            get_users,
            create_user,
            reload_files,
            import_dict,
            set_current_user,
            get_current_user,
            start_practice_session,
            practice_current_word,
            get_remaining_words,
            conclude_session,
            save_current_user,
            get_pool_size
        ])
        .setup(|app| {
            println!("setup");
            let mut base_path = app.path().data_dir().unwrap();
            base_path.push(APP_DATA_FOLDER);
        
            let mut user_path = base_path.clone();
            user_path.push("users");
        
            let mut dict_path = base_path.clone();
            dict_path.push("dicts");
        
            create_dir_all(&user_path).unwrap();
            create_dir_all(&dict_path).unwrap();

            println!("{}", base_path.to_str().unwrap());
        
            let mut appl = Application::new(user_path, dict_path, app.handle().to_owned()).unwrap();
            appl.load(None).unwrap();
        
            let last_user_path = {
                let mut t = base_path.clone();
                t.push("lastuser");
                t
            };
        
            let last_user = if last_user_path.exists() && last_user_path.is_file() {
                let mut file = File::open(last_user_path).unwrap();
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).unwrap();
        
                appl.get_user_id(String::from_utf8(bytes).unwrap())
            } else {
                None
            };
        
            appl.set_current_user(last_user);

            init_app(appl);

            let main_window = app.get_webview_window("main").unwrap();
            #[cfg(not(mobile))]
            {
                println!("not mobile");
                main_window.set_decorations(false).unwrap();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
