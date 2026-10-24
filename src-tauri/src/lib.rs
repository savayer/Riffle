#[cfg(target_os = "macos")]
mod context_menu;
mod media;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

/// File the app was launched with. On macOS a Finder open arrives as
/// `RunEvent::Opened`, possibly before the frontend has loaded, so it is kept here.
struct InitialFile(Mutex<Option<PathBuf>>);

#[derive(Serialize)]
struct MediaList {
    files: Vec<String>,
    index: usize,
}

#[tauri::command]
fn get_initial_file(state: State<InitialFile>) -> Option<String> {
    state
        .0
        .lock()
        .unwrap()
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
async fn list_media(app: AppHandle, path: String) -> Result<MediaList, String> {
    // canonical paths only: the asset scope is matched against them (/tmp -> /private/tmp)
    let current = Path::new(&path).canonicalize().map_err(|e| e.to_string())?;
    // never widen the asset scope for non-media paths
    if !media::is_media(&current) {
        return Err("unsupported file type".into());
    }
    let dir = current.parent().ok_or("file has no parent folder")?;
    let scope = app.asset_protocol_scope();
    // the file itself: a dot-file never matches the folder's `dir/*` pattern
    scope.allow_file(&current).map_err(|e| e.to_string())?;
    scope
        .allow_directory(dir, false)
        .map_err(|e| e.to_string())?;
    let (files, index) = media::list_media(dir, &current).map_err(|e| e.to_string())?;
    Ok(MediaList {
        files: files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect(),
        index,
    })
}

/// Selects the file in a Finder window (`open -R`).
fn reveal(path: &str) -> Result<(), String> {
    let status = std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("open -R failed: {status}"))
    }
}

#[tauri::command]
async fn reveal_in_finder(path: String) -> Result<(), String> {
    reveal(&path)
}

/// The file on screen, for the native context menu's "Show in Finder".
#[tauri::command]
fn set_current_file(path: Option<String>) {
    #[cfg(target_os = "macos")]
    context_menu::set_current(path);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(InitialFile(Mutex::new(None)))
        .setup(|app| {
            // `tauri dev -- -- <file>` / `open -a Riffle --args <file>`
            let arg = std::env::args()
                .skip(1)
                .find(|a| !a.starts_with('-') && Path::new(a).is_file());
            // setup runs on Ready, i.e. after a cold-start RunEvent::Opened: don't overwrite it
            let state = app.state::<InitialFile>();
            let mut initial = state.0.lock().unwrap();
            if initial.is_none() {
                *initial = arg.map(PathBuf::from);
            }
            #[cfg(target_os = "macos")]
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.with_webview(|wv| context_menu::install(wv.inner()));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_initial_file,
            list_media,
            reveal_in_finder,
            set_current_file
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            if let tauri::RunEvent::Opened { urls } = event {
                use tauri::Emitter;
                // to_file_path() decodes %20, Cyrillic etc.; url.path() would not
                let Some(path) = urls.iter().find_map(|u| u.to_file_path().ok()) else {
                    return;
                };
                *app.state::<InitialFile>().0.lock().unwrap() = Some(path.clone());
                let _ = app.emit("open-file", path.to_string_lossy());
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.unminimize();
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        });
}
