#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env, fs,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    thread,
    time::Duration,
};
use tauri::{AppHandle, Manager, State};

const CREATE_NO_WINDOW: u32 = 0x08000000;

struct StatusLock(Mutex<()>);

fn requested_status() -> Option<String> {
    let mut args = env::args();
    while let Some(argument) = args.next() {
        if argument == "--status-host" {
            return args.next();
        }
    }
    None
}

fn status_directory(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|path| path.join("status-runners"))
        .map_err(|error| error.to_string())
}

fn safe_file_stem(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("表示名を入力してください。".into());
    }

    let mut safe = String::new();
    for character in trimmed.chars().take(60) {
        if character.is_control() || r#"<>:"/\|?*"#.contains(character) {
            safe.push('＿');
        } else {
            safe.push(character);
        }
    }

    let safe = safe.trim_end_matches([' ', '.']).to_string();
    if safe.is_empty() {
        return Err("ファイル名として使える文字を含めてください。".into());
    }

    let upper = safe.to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    if reserved.contains(&upper.as_str()) {
        Ok(format!("_{safe}"))
    } else {
        Ok(safe)
    }
}

fn stop_recorded_process(directory: &Path, preserve_pid: Option<u32>) {
    let pid_path = directory.join("active.pid");
    if let Ok(pid) = fs::read_to_string(&pid_path) {
        let pid = pid.trim();
        if pid
            .parse::<u32>()
            .is_ok_and(|pid| Some(pid) != preserve_pid)
        {
            let _ = Command::new("taskkill")
                .args(["/PID", pid, "/T", "/F"])
                .creation_flags(CREATE_NO_WINDOW)
                .status();
            thread::sleep(Duration::from_millis(300));
        }
    }

    let _ = fs::remove_file(pid_path);
    let _ = fs::remove_file(directory.join("active.name"));
}

#[tauri::command]
fn activate_status(
    app: AppHandle,
    state: State<'_, StatusLock>,
    name: String,
) -> Result<String, String> {
    let _guard = state
        .0
        .lock()
        .map_err(|_| "ステータス切り替え処理を開始できませんでした。")?;
    let display_name = name.trim().to_string();
    let file_stem = safe_file_stem(&display_name)?;
    let directory = status_directory(&app)?;

    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    stop_recorded_process(&directory, Some(std::process::id()));

    let source = env::current_exe().map_err(|error| error.to_string())?;
    let source_stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    if file_stem.eq_ignore_ascii_case(source_stem) {
        fs::write(directory.join("active.pid"), std::process::id().to_string())
            .map_err(|error| error.to_string())?;
        fs::write(directory.join("active.name"), &display_name)
            .map_err(|error| error.to_string())?;

        let window = app
            .get_webview_window("main")
            .ok_or("メインウィンドウが見つかりません。")?;
        window
            .set_title(&display_name)
            .map_err(|error| error.to_string())?;

        return Ok(display_name);
    }

    let runner = directory.join(format!("{file_stem}.exe"));
    fs::copy(&source, &runner).map_err(|error| error.to_string())?;
    let child = Command::new(&runner)
        .args(["--status-host", &display_name])
        .spawn()
        .map_err(|error| error.to_string())?;

    fs::write(directory.join("active.pid"), child.id().to_string())
        .map_err(|error| error.to_string())?;
    fs::write(directory.join("active.name"), &display_name)
        .map_err(|error| error.to_string())?;

    let window = app
        .get_webview_window("main")
        .ok_or("メインウィンドウが見つかりません。")?;
    window
        .set_title(&display_name)
        .map_err(|error| error.to_string())?;

    Ok(display_name)
}

#[tauri::command]
fn stop_status(app: AppHandle, state: State<'_, StatusLock>) -> Result<(), String> {
    let _guard = state
        .0
        .lock()
        .map_err(|_| "ステータス停止処理を開始できませんでした。")?;
    let directory = status_directory(&app)?;

    let current_process_is_active = fs::read_to_string(directory.join("active.pid"))
        .ok()
        .and_then(|pid| pid.trim().parse::<u32>().ok())
        .is_some_and(|pid| pid == std::process::id());
    if current_process_is_active {
        let _ = fs::remove_file(directory.join("active.pid"));
        let _ = fs::remove_file(directory.join("active.name"));
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            app.exit(0);
        });
        return Ok(());
    }

    stop_recorded_process(&directory, Some(std::process::id()));
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_title("たぶん作業中")
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_active_status(app: AppHandle) -> Result<Option<String>, String> {
    let path = status_directory(&app)?.join("active.name");
    match fs::read_to_string(path) {
        Ok(name) => Ok(Some(name)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

#[tauri::command]
fn minimize_launcher(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("メインウィンドウが見つかりません。")?;
    window.minimize().map_err(|error| error.to_string())
}

fn main() {
    let status = requested_status();

    tauri::Builder::default()
        .manage(StatusLock(Mutex::new(())))
        .invoke_handler(tauri::generate_handler![
            activate_status,
            stop_status,
            get_active_status,
            minimize_launcher
        ])
        .setup(move |app| {
            let window = app.get_webview_window("main").expect("main window");

            if let Some(status_name) = status.clone() {
                window.set_title(&status_name)?;
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(750));
                    let _ = window.minimize();
                });
            } else {
                let active_name = status_directory(app.handle())
                    .ok()
                    .and_then(|directory| fs::read_to_string(directory.join("active.name")).ok())
                    .filter(|name| !name.trim().is_empty());
                window.set_title(active_name.as_deref().unwrap_or("たぶん作業中"))?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
