//! TruckersTV app: the companion window (Tauri) plus the in-cabin UI host on its own thread.

// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod customize;
mod setup;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use setup::SetupStore;
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, WebviewWindow};
use westplay_host::settings::{Settings, SettingsFile, SharedSettings};
use westplay_host::{AccountAction, AccountRequests, BrowserGone, Config, Content, SCREEN_HEIGHT, SCREEN_WIDTH, Status};

struct Host {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

/// The companion's sign-ins and sign-outs, for the host.
struct Accounts(Arc<AccountRequests>);

/// The player's settings: kept on disk, applied live by the host thread, changed from the companion window.
struct SettingsStore {
    file: SettingsFile,
    shared: Arc<SharedSettings>,
}

/// The companion window starts hidden and shows once its page has drawn its first screen
/// (`companion_ready`), so it never shows blank while the page loads; after this long it shows anyway.
const SHOW_ANYWAY: Duration = Duration::from_secs(8);

/// The companion's page has drawn its first screen: the window shows.
#[tauri::command]
fn companion_ready(window: WebviewWindow) {
    if !window.is_visible().unwrap_or(false) {
        westplay_host::startup::mark("app: window shown");
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Opens a site app's sign-in in a window on the desktop.
#[tauri::command]
fn site_sign_in(app: String, accounts: State<Accounts>) {
    if let Ok(mut asked) = accounts.0.site.lock() {
        *asked = Some(app);
    }
}

/// Opens Google's sign-in for YouTube Music in a window on the desktop (the status then says when it's done).
#[tauri::command]
fn youtube_music_sign_in(accounts: State<Accounts>) {
    accounts.0.youtube_music.request(AccountAction::SignIn);
}

/// Signs YouTube Music out.
#[tauri::command]
fn youtube_music_sign_out(accounts: State<Accounts>) {
    accounts.0.youtube_music.request(AccountAction::SignOut);
}

/// Runs YouTube Music (hidden), so the status says whether it's signed in.
#[tauri::command]
fn youtube_music_check(accounts: State<Accounts>) {
    accounts.0.youtube_music.request(AccountAction::Check);
}

/// Opens Spotify's sign-in in a window on the desktop (the status then says when it's done).
#[tauri::command]
fn spotify_sign_in(accounts: State<Accounts>) {
    accounts.0.spotify.request(AccountAction::SignIn);
}

/// Signs Spotify out.
#[tauri::command]
fn spotify_sign_out(accounts: State<Accounts>) {
    accounts.0.spotify.request(AccountAction::SignOut);
}

/// Runs Spotify's web player (hidden), so the status says whether it's signed in.
#[tauri::command]
fn spotify_check(accounts: State<Accounts>) {
    accounts.0.spotify.request(AccountAction::Check);
}

/// A keyboard key's name for the player's keyboard, by its virtual-key code (the keybinds' labels); `None`
/// for a mouse button or a key without one.
#[tauri::command]
fn key_name(key: u32) -> Option<String> {
    westplay_host::settings::KeyCode::name(key)
}

#[tauri::command]
fn get_settings(store: State<SettingsStore>) -> Settings {
    store.shared.get()
}

/// Saves new settings and applies them. Values are corrected on the way in (see `westplay_host::settings`);
/// the settings as saved are returned and announced to every window (`settings-changed`).
#[tauri::command]
fn update_settings(settings: Settings, store: State<SettingsStore>, app: AppHandle) -> Result<Settings, String> {
    store.file.save(&settings).map_err(|e| format!("Couldn't save the settings: {e}"))?;
    store.shared.set(settings.clone());
    let _ = app.emit("settings-changed", &settings);
    Ok(settings)
}

fn main() {
    // Started as administrator only to copy the plugin into the game (setup asked for it): that, and done.
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = westplay_setup::plugin::run_as_administrator_copy(&args) {
        std::process::exit(code);
    }
    // One TruckersTV at a time: a second only says so (held until the app ends). Before the log starts: a second
    // making the log anew would empty the running one's.
    let Some(_instance) = westplay_host::Instance::claim() else {
        westplay_host::tell_already_running();
        return;
    };
    westplay_host::log::init();
    westplay_host::startup::mark("app: started");
    let accounts = Arc::new(AccountRequests::default());
    let app = tauri::Builder::default()
        .manage(Accounts(accounts.clone()))
        .invoke_handler(tauri::generate_handler![
            companion_ready,
            site_sign_in,
            youtube_music_sign_in,
            youtube_music_sign_out,
            youtube_music_check,
            spotify_sign_in,
            spotify_sign_out,
            spotify_check,
            customize::pick_tv_videos,
            get_settings,
            update_settings,
            key_name,
            setup::setup_state,
            setup::game_running,
            setup::choose_game_folder,
            setup::check_game,
            setup::install_into_game,
            setup::finish_setup,
            setup::uninstall_from_game,
            setup::remove_data,
            setup::leave_pc
        ])
        .setup(move |app| {
            westplay_host::startup::mark("app: window set up");
            // A page that never says it's ready (it failed to load) still shows, rather than nothing at all.
            if let Some(window) = app.get_webview_window("main") {
                std::thread::spawn(move || {
                    std::thread::sleep(SHOW_ANYWAY);
                    if !window.is_visible().unwrap_or(true) {
                        westplay_host::startup::mark("app: window shown without its page ready");
                        let _ = window.show();
                    }
                });
            }
            let config_dir = app.path().app_config_dir()?;
            let file = SettingsFile::in_dir(&config_dir);
            let shared = Arc::new(SharedSettings::new(file.load()));
            let setup = SetupStore::load(&config_dir, app.handle());
            setup::watch_installed_mods(&setup);
            let host = start_host(app.handle(), accounts.clone(), shared.clone(), config_dir.clone())?;
            app.manage(SettingsStore { file, shared });
            app.manage(setup);
            app.manage(std::sync::Mutex::new(host));
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build the TruckersTV app");

    app.run(|app, event| {
        if let RunEvent::Exit = event {
            stop_host(app);
        }
    });
}

/// Stops the host, so its WebView2 shuts down cleanly with the app (it quitting, or an update's installer
/// about to replace it). Once: a stopped host has nothing left to stop.
fn stop_host(app: &AppHandle) {
    if let Some(host) = app.try_state::<std::sync::Mutex<Host>>() {
        let mut host = host.lock().unwrap_or_else(|e| e.into_inner());
        host.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = host.thread.take() {
            let _ = thread.join();
        }
    }
}

/// How a run of the host ended.
enum Ended {
    Stopped,
    BrowserGone,
    /// To draw the cab on another GPU (westplay_host::graphics).
    GpuChanged,
    Failed(String),
}

/// The host is started again after its browser fails at most this often within the window, after a pause.
const MAX_RESTARTS: usize = 5;
const RESTART_WINDOW: Duration = Duration::from_secs(60);
const RESTART_DELAY: Duration = Duration::from_secs(1);
/// How long WebView2's browsers are waited for to go before the host starts on another GPU.
const BROWSERS_GOING: Duration = Duration::from_secs(10);

fn start_host(
    app: &AppHandle,
    accounts: Arc<AccountRequests>,
    settings: Arc<SharedSettings>,
    config_dir: PathBuf,
) -> Result<Host, Box<dyn std::error::Error>> {
    let content = cabin_content(app);
    let data_dir = app.path().app_local_data_dir()?;
    let stop = Arc::new(AtomicBool::new(false));

    let events = app.clone();
    let thread_stop = stop.clone();
    // The host runs on a thread of its own, started again (on a new thread) if WebView2's browser process
    // exits under it (westplay_host::BrowserGone): the cabin page comes back rather than staying blank. Not
    // endlessly: a browser failing again and again is reported instead.
    let thread = std::thread::Builder::new().name("westplay-host-supervisor".into()).spawn(move || {
        let mut restarts: Vec<Instant> = Vec::new();
        loop {
            let (content, data_dir) = (content.clone(), data_dir.clone());
            let (accounts, settings, stop) = (accounts.clone(), settings.clone(), thread_stop.clone());
            let status_events = events.clone();
            let (settings_events, settings_dir) = (events.clone(), config_dir.clone());
            let run = std::thread::Builder::new().name("westplay-host".into()).spawn(move || {
                let config = Config {
                    width: SCREEN_WIDTH,
                    height: SCREEN_HEIGHT,
                    content,
                    data_dir,
                    accounts,
                    settings,
                    on_status: Box::new(move |status| {
                        let _ = status_events.emit("host-status", status);
                    }),
                    // The cab's own Settings changed one (its equalizer): kept, and the companion told.
                    on_settings: Box::new(move |settings| {
                        if let Err(error) = SettingsFile::in_dir(&settings_dir).save(settings) {
                            westplay_host::log!("[settings] couldn't save the cab's change: {error}");
                        }
                        let _ = settings_events.emit("settings-changed", settings);
                    }),
                };
                match westplay_host::run(config, stop) {
                    Ok(()) => Ended::Stopped,
                    Err(e) if e.is::<BrowserGone>() => Ended::BrowserGone,
                    Err(e) if e.is::<westplay_host::graphics::GpuChanged>() => Ended::GpuChanged,
                    Err(e) => Ended::Failed(e.to_string()),
                }
            });
            let ended = match run.map(|thread| thread.join()) {
                Ok(Ok(ended)) => ended,
                Ok(Err(_)) => Ended::Failed("it crashed".into()),
                Err(e) => Ended::Failed(e.to_string()),
            };
            let error = match ended {
                Ended::Stopped => return,
                Ended::BrowserGone | Ended::GpuChanged if thread_stop.load(Ordering::Relaxed) => return,
                // Started again on the other GPU once the browsers have gone (one still running would refuse
                // the new arguments). Not counted as a failure.
                Ended::GpuChanged => {
                    westplay_host::wait_for_browsers(BROWSERS_GOING);
                    continue;
                }
                Ended::BrowserGone => {
                    let now = Instant::now();
                    restarts.retain(|&at| now.duration_since(at) < RESTART_WINDOW);
                    restarts.push(now);
                    if restarts.len() <= MAX_RESTARTS {
                        std::thread::sleep(RESTART_DELAY);
                        continue;
                    }
                    BrowserGone.to_string()
                }
                Ended::Failed(error) => error,
            };
            let status = Status { error: Some(format!("TruckersTV engine stopped: {error}")), ..Default::default() };
            let _ = events.emit("host-status", &status);
            return;
        }
    })?;
    Ok(Host { stop, thread: Some(thread) })
}

/// The cabin page: from the Vite dev server while developing, from the bundled UI otherwise.
fn cabin_content(app: &AppHandle) -> Content {
    if cfg!(debug_assertions)
        && let Some(dev_url) = &app.config().build.dev_url
    {
        return Content::Url(format!("{}cabin.html", dev_url.as_str()));
    }
    let bundled =
        app.path().resource_dir().map(|dir| dir.join("ui")).ok().filter(|dir| dir.join("cabin.html").exists());
    // Unbundled release builds (`cargo build --release`) use the UI build next to the sources.
    let root = bundled.unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/dist"));
    Content::Folder { root, page: "cabin.html".into() }
}
