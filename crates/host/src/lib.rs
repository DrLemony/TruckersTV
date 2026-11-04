//! TruckersTV host: renders the in-cabin UI in an offscreen WebView2 and feeds it to the game plugin.
//!
//! [`run`] owns one STA thread: it pumps window messages for WebView2, keeps the capture pipeline on the
//! game's GPU, forwards telemetry from the bridge to the page, and beats the host heartbeat.

#[macro_use]
pub mod log;
mod camera;
mod capture;
mod cursor;
pub mod fitted;
mod game_settings;
mod gpu;
pub mod graphics;
mod lyrics;
mod performance;
mod play_log;
mod player;
mod pointer;
pub mod settings;
mod signin;
mod site_browser;
mod tv_view;
pub mod speakers;
mod spotify;
pub mod startup;
mod typing;
mod webview;
mod youtube_music;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::{Duration, Instant};

use westplay_shared::{BridgeMapping, JobInfo, Keybind, Telemetry, TruckInfo};
use windows::Win32::Foundation::LUID;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage};

use capture::Capture;
use gpu::{Gpu, SharedFrame};
use player::{PartSpeakers, Player, PlayerWatch};
use pointer::Pointer;
use settings::SharedSettings;
use webview::{OffscreenWebView, Stage};

pub use webview::{Error, wait_for_browsers};
/// Size the cabin page is rendered at: the size of the frames the plugin copies into the screen textures.
pub use westplay_shared::{SCREEN_HEIGHT, SCREEN_WIDTH};

/// Built-in page used when no UI URL is given: shows that frames and telemetry arrive.
pub const TEST_PAGE: &str = include_str!("test_page.html");

#[derive(Clone)]
pub enum Content {
    Url(String),
    Html(String),
    /// A built UI folder, served at `https://westplay.cabin/` through a WebView2 virtual host.
    Folder {
        root: PathBuf,
        page: String,
    },
}

/// Host health, reported about once per second.
#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub webview_version: String,
    /// The GPU frames are handed to the game on (the game's, once it runs).
    pub adapter: Option<String>,
    /// The GPU the cab is drawn on, and the game's (`None` until the game runs).
    pub cab_adapter: Option<String>,
    pub game_adapter: Option<String>,
    pub frames_per_second: u64,
    pub dropped_total: u64,
    pub game_connected: bool,
    pub screen_drawing: bool,
    /// Whether the truck the player is in is one TruckersTV shows in, as the plugin last saw (`None`: not known yet,
    /// or no game).
    pub truck_supported: Option<bool>,
    /// Whether YouTube Music is signed in, as its page last said (`None`: not known, it hasn't run yet), and
    /// whether Google's sign-in window is open. Nothing else of the account.
    pub youtube_music_signed_in: Option<bool>,
    pub youtube_music_signing_in: bool,
    /// The same of Spotify: signed in or not (`None`: not known, Spotify not run yet), its sign-in window open.
    pub spotify_signed_in: Option<bool>,
    pub spotify_signing_in: bool,
    /// The site app whose sign-in window is open, by its id, and those signed in.
    pub site_signing_in: Option<String>,
    pub site_signed_in: Vec<String>,
    pub error: Option<String>,
}

/// The companion window's requests to sign YouTube Music or Spotify in or out (or to know whether it
/// is), from any thread, for the host to carry out as the cab's apps would.
#[derive(Debug, Default)]
pub struct AccountRequests {
    pub youtube_music: AccountRequest,
    pub spotify: AccountRequest,
    /// A site app to sign in to, by its id.
    pub site: std::sync::Mutex<Option<String>>,
}

/// One service's request, the latest not carried out yet.
#[derive(Debug, Default)]
pub struct AccountRequest(AtomicU8);

/// What the companion asks of a service's sign-in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountAction {
    SignIn,
    SignOut,
    /// Run it (hidden), so whether it's signed in is known (the companion's Accounts, shown).
    Check,
}

impl AccountRequest {
    const NONE: u8 = 0;
    const SIGN_IN: u8 = 1;
    const SIGN_OUT: u8 = 2;
    const CHECK: u8 = 3;

    /// Asks for it (replacing a request not carried out yet).
    pub fn request(&self, action: AccountAction) {
        let value = match action {
            AccountAction::SignIn => Self::SIGN_IN,
            AccountAction::SignOut => Self::SIGN_OUT,
            AccountAction::Check => Self::CHECK,
        };
        self.0.store(value, Ordering::Release);
    }

    /// The latest request not carried out yet, if any.
    pub fn take(&self) -> Option<AccountAction> {
        match self.0.swap(Self::NONE, Ordering::AcqRel) {
            Self::SIGN_IN => Some(AccountAction::SignIn),
            Self::SIGN_OUT => Some(AccountAction::SignOut),
            Self::CHECK => Some(AccountAction::Check),
            _ => None,
        }
    }
}

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub content: Content,
    /// WebView2 user data (cache, cookies) lives here.
    pub data_dir: PathBuf,
    /// Signing YouTube Music or Spotify in or out (or finding out whether it is), from the companion window.
    pub accounts: Arc<AccountRequests>,
    /// Called on the host thread about once per second.
    pub on_status: Box<dyn FnMut(&Status) + Send>,
    /// The player's settings; changes are applied as they happen.
    pub settings: Arc<SharedSettings>,
    /// Called when the cabin page changes a setting itself (its Settings' equalizer), after it's applied: to
    /// keep it and tell the companion.
    pub on_settings: Box<dyn FnMut(&settings::Settings) + Send>,
}

struct Pipeline {
    gpu: Gpu,
    /// Holds the shared frame texture (and its name) alive.
    capture: Capture,
}

/// Runs until `stop` is set. Blocks the calling thread, which becomes the WebView2 UI thread.
pub fn run(mut config: Config, stop: Arc<AtomicBool>) -> Result<(), Error> {
    startup::mark("host: started");
    // SAFETY: COM on this thread, once, before anything on it uses COM (WebView2 needs a single-threaded
    // apartment).
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
    let bridge = Arc::new(BridgeMapping::open()?);
    let host = &bridge.bridge().host;
    host.pid.store(std::process::id(), Ordering::Release);
    startup::mark("host: game bridge open");

    let stage = Stage::new(config.width, config.height)?;
    // The GPU the cab is drawn on (graphics.rs): the player's choice, the game's if it's running, or Windows'
    // high-performance one. Every WebView of this run draws on it.
    let mut adapters = graphics::adapters();
    let mut chosen_gpu = config.settings.get().graphics.gpu;
    let cab_gpu = graphics::choose(chosen_gpu.as_deref(), game_adapter(&bridge), None, &adapters);
    webview::draw_on(cab_gpu.map(graphics::browser_argument));
    if let Some(luid) = cab_gpu {
        log!("the cab is drawn on {}", graphics::name_of(luid, &adapters).unwrap_or_default());
    }
    let webview = OffscreenWebView::new(&stage, config.width, config.height, &config.data_dir.join("webview2"))?;
    stage.add(&webview, true)?;
    startup::mark("host: cabin browser (WebView2) ready");
    match &config.content {
        Content::Url(url) => webview.navigate(url)?,
        Content::Html(html) => webview.navigate_to_string(html)?,
        Content::Folder { root, page } => {
            webview.map_folder("westplay.cabin", root)?;
            webview.navigate(&format!("https://westplay.cabin/{page}"))?;
        }
    }
    startup::mark("host: cabin page requested");
    // What TruckersTV costs while the game runs, in the log each minute (performance.rs).
    let mut meter = performance::Meter::new(match &config.content {
        Content::Url(url) => performance::host_of(url).unwrap_or_default(),
        Content::Html(_) => String::new(),
        Content::Folder { .. } => "westplay.cabin".to_string(),
    });
    let mut status = Status {
        webview_version: webview.version.clone(),
        cab_adapter: cab_gpu.and_then(|luid| graphics::name_of(luid, &adapters)),
        site_signed_in: site_browser::signed_in(&config.data_dir.join("site-browser")),
        ..Default::default()
    };

    let mut pipeline: Option<Pipeline> = None;
    let mut generation_counter = 0u32;
    let mut last_check = Instant::now() - Duration::from_secs(10);
    let mut last_stats = Instant::now();
    let mut last_telemetry_post = Instant::now();
    let mut last_telemetry: Option<Telemetry> = None;
    let mut last_truck: Option<TruckInfo> = None;
    let mut last_job: Option<JobInfo> = None;
    // The game's menu gone while it's still paused (`camera::MenuGone`), and as last told to the page.
    let mut menu = camera::MenuGone::default();
    let mut menu_gone = false;
    // Whether the camera was outside the cab, as last said in the log.
    let mut was_outside = false;
    // The players' tone (the cab's Settings › Equalizer), passed on with the cab ([`settings::SoundSettings`]).
    let mut equalizer = config.settings.get().sound.playing();
    // The music's spectrum wanted by the cab (its Equalizer open): asked of the players with the cab.
    let mut spectrum = false;
    let mut pointer = Pointer::new(config.width, config.height);
    let mut site_browser: Option<site_browser::Browser> = None;
    // TV's player (tv_view.rs), while TV uses it.
    let mut tv: Option<tv_view::Tv> = None;
    // The screen's cursor, drawn into the frames the game takes (cursor.rs), and whether the page was last told the
    // host draws it (`None`: not told since it loaded).
    let mut cursor = cursor::Cursor::new(Instant::now());
    let mut cursor_told: Option<bool> = None;
    // What the player's latest save says (the game's language), kept to send again when the page reloads.
    let player_watch = PlayerWatch::start();
    // The speakers of the parts of the truck the player is in, for the trucks whose speakers come with them.
    let mut part_speakers: Option<PartSpeakers> = None;
    let mut player_messages: Vec<String> = Vec::new();
    // Whether the game's window shows (as last told the page; sent again to a reloaded page).
    let mut game_shown = true;
    // Whether the game runs (its process, the plugin's pid), as last told the page. As it closes (or crashes)
    // the web players close with it, so none plays on, unseen, with the game gone (2026-10-21).
    let mut game_running = false;
    // Whether the player's keyboard types into the page's field, as last told to the page.
    let mut typing_state: Option<westplay_shared::TypingState> = None;
    // Whether the game's camera is outside the cab (the music muffled then), and the player's camera
    // settings in the game (it turns with the steering).
    let mut camera = camera::Camera::default();
    let mut game_settings = game_settings::GameSettings::new();
    let mut spotify_log = play_log::PlayLog::new("spotify");
    let mut youtube_music_log = play_log::PlayLog::new("youtube music");
    // Spotify's web player, while the page uses it, and what it last said (sent again when the page reloads):
    // signed in or not, and what plays.
    let mut spotify: Option<spotify::Spotify> = None;
    let mut spotify_state: Option<spotify::State> = None;
    let mut spotify_signing_in = false;
    // Spotify's sign-in confirming after its window closed signed in (spotify.rs).
    let mut spotify_confirming = false;
    let mut spotify_playback: Option<String> = None;
    // YouTube Music's web player, the same way (on the cabin page's profile).
    let mut youtube_music: Option<youtube_music::YouTubeMusic> = None;
    let mut youtube_music_state: Option<youtube_music::State> = None;
    let (mut youtube_music_window_open, mut youtube_music_confirming) = (false, false);
    let mut youtube_music_playback: Option<String> = None;
    // The settings generation last applied (`None`: apply them once at startup).
    let mut applied_settings = None;
    // Each keybind's presses the page was told of (`PluginBlock::keybind_presses`).
    let mut keybind_presses =
        bridge.bridge().plugin.keybind_presses.each_ref().map(|count| count.load(Ordering::Acquire));
    let mut last_frames = 0;
    // The startup stages still to log (`startup`), checked every turn until all are reached. The game's
    // screen frame counter as found: a session before this one may have left it counting.
    let mut startup_pending = true;
    let screen_frame_before = bridge.bridge().plugin.screen_frame.load(Ordering::Relaxed);

    let mut browser_gone = false;
    let mut gpu_changed = false;
    let mut game_liveness = Liveness::default();
    while !stop.load(Ordering::Relaxed) {
        pump_messages();
        host.heartbeat.fetch_add(1, Ordering::Release);
        // Set for this turn when the game has just closed: every web player closes, as its WebView failing
        // closes it (below), the page told so.
        let mut players_closing = false;
        // A captured frame that came before its turn goes out now that it's due.
        // The cursor drawn over it where the mouse is now.
        if let Some(pipeline) = &pipeline {
            pipeline.capture.flush(cursor.look(Instant::now()), 1.0);
        }
        // The page told whether the host draws the cursor (it draws its own where the host can't).
        let drawn = pipeline.as_ref().is_some_and(|pipeline| pipeline.capture.draws_cursor());
        if cursor_told != Some(drawn) {
            cursor_told = Some(drawn);
            let _ = webview.post_json(&cursor_message(drawn));
        }
        // The page's process failed (a crash, or hung): load it again, and it gets the current state again
        // as it announces itself. The browser's: this WebView is gone, so start over (`BrowserGone`).
        match webview.take_failure() {
            Some(webview::Failure::Page) => {
                startup::mark("host: the cabin page's process failed; reloading it");
                if let Err(e) = webview.reload() {
                    status.error = Some(format!("the cabin page couldn't be reloaded: {e}"));
                    (config.on_status)(&status);
                }
            }
            Some(webview::Failure::Browser) => {
                browser_gone = true;
                break;
            }
            None => {}
        }
        if startup_pending {
            startup_pending = log_startup(&bridge, pipeline.as_ref(), screen_frame_before);
        }

        // TruckersTV's frames go to the truck's screen while the host runs (the page, open or closed, is
        // what shows there).
        host.screen_enabled.store(1, Ordering::Release);
        // The companion's YouTube Music and Spotify sign-ins or sign-outs, as the cab's would be (or only
        // running it, to know whether it's signed in).
        if let Some(action) = config.accounts.youtube_music.take()
            && let Some(player) = start_youtube_music(&mut youtube_music, &stage, &config)
        {
            match action {
                AccountAction::SignIn => drop(player.command(&youtube_music::Command::SignIn)),
                AccountAction::SignOut => drop(player.command(&youtube_music::Command::SignOut)),
                AccountAction::Check => {}
            }
        }
        if let Some(action) = config.accounts.spotify.take()
            && let Some(player) = start_spotify(&mut spotify, &stage, &config)
        {
            let _ = player.command(&match action {
                AccountAction::SignIn => spotify::Command::SignIn,
                AccountAction::SignOut => spotify::Command::SignOut,
                AccountAction::Check => spotify::Command::Start,
            });
        }

        if let Some(app) = config.accounts.site.lock().ok().and_then(|mut app| app.take())
            && let Some(browser) = start_site_browser(&mut site_browser, &stage, &config, &webview)
            && let Err(error) = browser.command(site_browser::Command::SignIn { app }, &stage)
        {
            log!("[site browser] sign-in failed: {error}");
        }

        // Checked often so the game is picked up right away when it starts or switches GPU.
        if last_check.elapsed() >= Duration::from_millis(200) {
            last_check = Instant::now();
            // The game's window showing or not (minimised, on another desktop): the page pauses its music
            // meanwhile. Without a game, nothing is away.
            let shown = is_shown(bridge.bridge().plugin.pid.load(Ordering::Relaxed));
            if shown != game_shown {
                game_shown = shown;
                let _ = webview.post_json(&game_shown_message(shown));
            }
            let running = is_running(bridge.bridge().plugin.pid.load(Ordering::Relaxed));
            if running != game_running {
                game_running = running;
                if !running {
                    players_closing = true;
                    log!("the game closed: its web players are closed");
                }
                let _ = webview.post_json(&game_message(running));
            }
            let wanted = game_adapter(&bridge);
            status.game_adapter = wanted.and_then(|luid| graphics::name_of(luid, &adapters));
            // The cab on another GPU than it's drawn on (the game's, or the player's choice): WebView2 is
            // started again on it.
            let cab_wanted = graphics::choose(chosen_gpu.as_deref(), wanted, cab_gpu, &adapters);
            if let (Some(now), Some(next)) = (cab_gpu, cab_wanted)
                && !graphics::same(now, next)
            {
                let name = graphics::name_of(next, &adapters).unwrap_or_default();
                log!("the cab moves to {name}");
                gpu_changed = true;
                break;
            }
            let needs_rebuild = match &pipeline {
                None => true,
                Some(p) => p.capture.broken() || wanted.is_some_and(|luid| !same_luid(luid, p.gpu.luid)),
            };
            if needs_rebuild {
                host.frame_generation.store(0, Ordering::Release);
                pipeline = None;
                generation_counter = generation_counter.wrapping_add(1);
                let generation = ((std::process::id() & 0x00ff_ffff) << 8) | (generation_counter & 0xff);
                match build_pipeline(&config, &stage, &bridge, wanted, generation) {
                    Ok(built) => {
                        startup::mark(if wanted.is_some() {
                            "host: capturing for the game's GPU"
                        } else {
                            "host: capturing (game not seen yet)"
                        });
                        host.frame_width.store(config.width, Ordering::Relaxed);
                        host.frame_height.store(config.height, Ordering::Relaxed);
                        host.frame_generation.store(generation, Ordering::Release);
                        status.adapter = Some(built.gpu.adapter_name.clone());
                        status.error = None;
                        pipeline = Some(built);
                    }
                    Err(e) => status.error = Some(format!("capture pipeline failed: {e}")),
                }
            }
            if last_stats.elapsed() >= Duration::from_secs(1) {
                last_stats = Instant::now();
                let plugin = &bridge.bridge().plugin;
                let frames = pipeline.as_ref().map_or(0, |p| p.capture.frames.load(Ordering::Relaxed));
                // TruckersTV's screen drawn, or the cab's mark in a truck with the TV (the TV itself isn't said there,
                // as it's drawn from outside the cab too): as the camera tells the cab drawn.
                let screen_frame = camera::drawn_frame(
                    plugin.screen_frame.load(Ordering::Relaxed),
                    plugin.cab_frame.load(Ordering::Relaxed),
                );
                let screen_age = plugin.heartbeat.load(Ordering::Relaxed).saturating_sub(screen_frame);
                status.frames_per_second = frames.saturating_sub(last_frames);
                status.dropped_total = pipeline.as_ref().map_or(0, |p| p.capture.dropped.load(Ordering::Relaxed));
                // A game that crashed leaves its pid and counters behind, frozen: alive is its heartbeat
                // moving.
                let alive = game_liveness.update(plugin.heartbeat.load(Ordering::Relaxed), Instant::now());
                status.game_connected = plugin.pid.load(Ordering::Relaxed) != 0 && alive && bridge.matches();
                // A plugin of another version (the game still running from before an update): not used.
                if !bridge.matches() && plugin.pid.load(Ordering::Relaxed) != 0 {
                    status.error = Some("the game's TruckersTV plugin is another version: restart the game".into());
                }
                status.screen_drawing = status.game_connected && screen_frame != 0 && screen_age < 120;
                status.truck_supported = match plugin.truck_supported.load(Ordering::Relaxed) {
                    _ if !status.game_connected => None,
                    westplay_shared::TRUCK_SUPPORTED => Some(true),
                    westplay_shared::TRUCK_UNSUPPORTED => Some(false),
                    _ => None,
                };
                let mut profiles = vec![(performance::Profile::Cab, webview.environment())];
                if let Some(spotify) = &spotify {
                    profiles.push((performance::Profile::Spotify, spotify.view().environment()));
                }
                let sample = status.game_connected.then(|| performance::Sample {
                    game_frames: plugin.heartbeat.load(Ordering::Relaxed),
                    in_view: status.screen_drawing,
                    sent: frames,
                    captured: pipeline.as_ref().map_or(0, |p| p.capture.arrived.load(Ordering::Relaxed)),
                    waited: status.dropped_total,
                });
                meter.second(Instant::now(), sample, &webview, &profiles);
                (config.on_status)(&status);
                last_frames = frames;
            }
        }

        if applied_settings != Some(config.settings.generation()) {
            applied_settings = Some(config.settings.generation());
            // The player may have chosen another GPU: the list again too (an external GPU plugged in).
            let graphics = config.settings.get().graphics;
            if graphics.gpu != chosen_gpu {
                chosen_gpu = graphics.gpu;
                adapters = graphics::adapters();
            }
            let interaction = config.settings.get().interaction;
            pointer.set_sensitivity(interaction.sensitivity.get());
            host.interaction_key.store(interaction.key.virtual_key(), Ordering::Release);
            let toggle = interaction.mode == settings::InteractionMode::Toggle;
            host.interaction_toggle.store(u32::from(toggle), Ordering::Release);
            for (slot, value) in host.keybinds.iter().zip(config.settings.get().keybinds.bridge_values()) {
                slot.store(value, Ordering::Release);
            }
            let _ = webview.post_json(&settings_message(&config.settings.get()));
        }
        let mouse = pointer.update(&bridge.bridge().plugin.pointer, &webview, site_browser.as_mut());
        cursor.follow(mouse, Instant::now());
        // The player's keybinds pressed in the game: the page acts on each.
        for keybind in Keybind::ALL {
            let presses = bridge.bridge().plugin.keybind_presses[keybind as usize].load(Ordering::Acquire);
            let new = presses.wrapping_sub(keybind_presses[keybind as usize]);
            keybind_presses[keybind as usize] = presses;
            // More at once than a player presses in a moment: another plugin's count (the game restarted).
            if new <= MOST_KEYBIND_PRESSES {
                for _ in 0..new {
                    let _ = webview.post_json(&keybind_message(keybind));
                }
            }
        }

        while let Ok(player) = player_watch.updates.try_recv() {
            player_messages = player_messages_for(&player);
            for message in &player_messages {
                let _ = webview.post_json(message);
            }
        }
        while let Ok(found) = player_watch.speakers.try_recv() {
            part_speakers = Some(found);
        }

        for message in webview.take_page_messages() {
            match parse_page_message(&message) {
                Some(PageMessage::Browser { command }) => {
                    if matches!(command, site_browser::Command::Close) {
                        if let Some(browser) = site_browser.take() {
                            let _ = stage.remove(&browser.view);
                        }
                        continue;
                    }
                    if !matches!(command, site_browser::Command::Hide) {
                        start_site_browser(&mut site_browser, &stage, &config, &webview);
                    }
                    if let Some(browser) = &mut site_browser {
                        if let Err(error) = browser.command(command, &stage) {
                            log!("[site browser] command failed: {error}");
                        }
                    }
                }
                Some(PageMessage::Tv { command }) => {
                    if matches!(command, tv_view::Command::Close) {
                        if let Some(closing) = tv.take() {
                            let _ = stage.remove(&closing.view);
                        }
                        continue;
                    }
                    if tv.is_none() && !matches!(command, tv_view::Command::Hide) {
                        match tv_page(&config.content)
                            .map(|page| tv_view::Tv::new(&stage, &config.data_dir.join("tv-player"), &page))
                        {
                            Some(Ok(player)) => tv = Some(player),
                            Some(Err(error)) => log!("[tv] could not start: {error}"),
                            None => log!("[tv] no player page for this cabin page"),
                        }
                    }
                    if let Some(player) = &mut tv {
                        let videos = config.settings.get().tv.videos;
                        match player.command(command, &stage, videos.as_deref()) {
                            Ok(Some(answer)) => drop(webview.post_json(&answer)),
                            Ok(None) => {}
                            Err(error) => log!("[tv] command failed: {error}"),
                        }
                    }
                }
                // The page announces itself when it (re)loads; it missed everything sent before, so send the
                // current state again (some of it is otherwise only sent when it changes).
                Some(PageMessage::Ready) => {
                    // A page loaded anew asks again if it wants it.
                    spectrum = false;
                    cursor_told = None;
                    startup::mark("page: running (its scripts loaded)");
                    last_telemetry = None;
                    last_truck = None;
                    last_job = None;
                    typing_state = None;
                    let _ = webview.post_json(&game_shown_message(game_shown));
                    let _ = webview.post_json(&game_message(game_running));
                    if spotify.is_some() {
                        let _ = webview.post_json(&spotify::state_message(spotify_state.as_ref(), spotify_signing_in));
                        if let Some(playback) = &spotify_playback {
                            let _ = webview.post_json(playback);
                        }
                    }
                    if youtube_music.is_some() {
                        let signing_in = youtube_music_window_open || youtube_music_confirming;
                        let message = youtube_music::state_message(youtube_music_state.as_ref(), signing_in);
                        let _ = webview.post_json(&message);
                        if let Some(playback) = &youtube_music_playback {
                            let _ = webview.post_json(playback);
                        }
                    }
                    let _ = webview.post_json(&settings_message(&config.settings.get()));
                    for message in player_messages.iter() {
                        let _ = webview.post_json(message);
                    }
                }
                Some(PageMessage::Startup { stage }) => startup::mark(&format!("page: {stage}")),
                Some(PageMessage::Spectrum { on }) => spectrum = on,
                Some(PageMessage::CursorLight { light }) => cursor.set_light(light, Instant::now()),
                Some(PageMessage::Equalizer { equalizer: curve, custom }) => {
                    let sound = settings::SoundSettings { equalizer: curve.clamped(), custom };
                    equalizer = sound.playing();
                    let mut settings = config.settings.get();
                    if settings.sound != sound {
                        settings.sound = sound;
                        config.settings.set(settings.clone());
                        (config.on_settings)(&settings);
                    }
                }
                Some(PageMessage::Spotify { command }) => {
                    if spotify.is_none() && matches!(command, spotify::Command::Pause) {
                        // Nothing to pause: Spotify isn't running.
                    } else if let Some(player) = start_spotify(&mut spotify, &stage, &config) {
                        let _ = player.command(&command);
                    }
                }
                Some(PageMessage::YoutubeMusic { command }) => {
                    if youtube_music.is_none() && matches!(command, youtube_music::Command::Pause) {
                        // Nothing to pause: YouTube Music isn't running.
                    } else if let Some(player) = start_youtube_music(&mut youtube_music, &stage, &config) {
                        let _ = player.command(&command);
                    }
                }
                Some(PageMessage::Typing { selection }) => {
                    bridge.bridge().typing.selection.store(selection, Ordering::Release);
                }
                None => {}
            }
        }
        // What Spotify's page says (signed in, playing) and its sign-in window, for the page; its WebView
        // failing closes it (the page opens it again).
        if let Some(player) = &spotify {
            if player.failed() || players_closing {
                if players_closing {
                    log!("[spotify] closed with the game");
                } else {
                    log!("[spotify] its browser failed: closed, the page opens it again");
                }
                if let Some(line) = spotify_log.heard(None) {
                    log!("{line}");
                }
                if let Some(failed) = spotify.take() {
                    failed.close(&stage);
                }
                spotify_state = None;
                spotify_playback = None;
                status.spotify_signed_in = None;
                status.spotify_signing_in = false;
                spotify_confirming = false;
                (config.on_status)(&status);
                let _ = webview.post_json(&spotify::state_message(None, false));
                let _ = webview.post_json(&spotify::playback_message(None));
            } else {
                let mut state_changed = false;
                for report in player.take_reports() {
                    match report {
                        spotify::Report::Spectrum(bars) => {
                            let _ = webview.post_json(&speakers::spectrum_message(&bars));
                        }
                        spotify::Report::State(state) => {
                            if status.spotify_signed_in != Some(state.signed_in) {
                                status.spotify_signed_in = Some(state.signed_in);
                                (config.on_status)(&status);
                            }
                            spotify_state = Some(state);
                            state_changed = true;
                        }
                        spotify::Report::SignInWindow(open) => {
                            spotify_signing_in = open;
                            status.spotify_signing_in = spotify_signing_in || spotify_confirming;
                            (config.on_status)(&status);
                            state_changed = true;
                        }
                        spotify::Report::Confirming(confirming) => {
                            spotify_confirming = confirming;
                            status.spotify_signing_in = spotify_signing_in || spotify_confirming;
                            (config.on_status)(&status);
                        }
                        // Answers and what plays go straight on to the page (never logged).
                        spotify::Report::Answer(answer) => {
                            let _ = webview.post_json(&spotify::answer_message(&answer));
                        }
                        spotify::Report::Timing(timing) => log!("{}", timing.line()),
                        spotify::Report::Playback(playback) => {
                            // What plays on the account's other devices (a phone) isn't this player's.
                            let here = playback.as_deref().filter(|playback| playback.here != Some(false));
                            let heard = here.map(|playback| play_log::Heard {
                                doing: if playback.playing {
                                    play_log::Doing::Playing
                                } else {
                                    play_log::Doing::Paused
                                },
                                track: playback.track.as_ref().map(|track| track.uri.as_str()),
                                position: playback.position,
                            });
                            if let Some(line) = spotify_log.heard(heard) {
                                log!("{line}");
                            }
                            let message = spotify::playback_message(playback.as_deref());
                            let _ = webview.post_json(&message);
                            spotify_playback = Some(message);
                        }
                    }
                }
                if state_changed {
                    let _ = webview.post_json(&spotify::state_message(spotify_state.as_ref(), spotify_signing_in));
                }
            }
        }

        // What YouTube Music's page says, and its sign-in window, for the page; its WebView failing closes it.
        if let Some(player) = &youtube_music {
            if player.failed() || players_closing {
                if players_closing {
                    log!("[youtube music] closed with the game");
                } else {
                    log!("[youtube music] its browser failed: closed, the page opens it again");
                }
                if let Some(line) = youtube_music_log.heard(None) {
                    log!("{line}");
                }
                if let Some(failed) = youtube_music.take() {
                    failed.close(&stage);
                }
                youtube_music_state = None;
                youtube_music_playback = None;
                (youtube_music_window_open, youtube_music_confirming) = (false, false);
                status.youtube_music_signing_in = false;
                (config.on_status)(&status);
                let _ = webview.post_json(&youtube_music::state_message(None, false));
                let _ = webview.post_json(&youtube_music::playback_message(None));
            } else {
                let mut state_changed = false;
                for report in player.take_reports() {
                    match report {
                        youtube_music::Report::Spectrum(bars) => {
                            let _ = webview.post_json(&speakers::spectrum_message(&bars));
                        }
                        youtube_music::Report::State(state) => {
                            if status.youtube_music_signed_in != Some(state.signed_in) {
                                status.youtube_music_signed_in = Some(state.signed_in);
                                (config.on_status)(&status);
                            }
                            youtube_music_state = Some(state);
                            state_changed = true;
                        }
                        youtube_music::Report::SignInWindow(open) => {
                            youtube_music_window_open = open;
                            status.youtube_music_signing_in = youtube_music_window_open || youtube_music_confirming;
                            (config.on_status)(&status);
                            state_changed = true;
                        }
                        // The cab shows it signing in until the page here says it's signed in.
                        youtube_music::Report::Confirming(confirming) => {
                            youtube_music_confirming = confirming;
                            status.youtube_music_signing_in = youtube_music_window_open || youtube_music_confirming;
                            (config.on_status)(&status);
                            state_changed = true;
                        }
                        // Answers and what plays go straight on to the page (never logged).
                        youtube_music::Report::Answer(answer) => {
                            let _ = webview.post_json(&youtube_music::answer_message(&answer));
                        }
                        // Times only (never what played): for the log.
                        youtube_music::Report::Timing(timing) => log!("{}", timing.line()),
                        youtube_music::Report::Playback(playback) => {
                            let heard = playback.as_deref().map(|playback| play_log::Heard {
                                doing: play_log::doing(playback.loading, playback.playing),
                                track: Some(playback.track.video.as_str()),
                                position: playback.position,
                            });
                            if let Some(line) = youtube_music_log.heard(heard) {
                                log!("{line}");
                            }
                            let message = youtube_music::playback_message(playback.as_deref());
                            let _ = webview.post_json(&message);
                            youtube_music_playback = Some(message);
                        }
                    }
                }
                if state_changed {
                    let signing_in = youtube_music_window_open || youtube_music_confirming;
                    let message = youtube_music::state_message(youtube_music_state.as_ref(), signing_in);
                    let _ = webview.post_json(&message);
                }
            }
        }

        if let Some(browser) = &mut site_browser {
            browser.tick(&webview);
            status.site_signing_in = browser.signing_in().map(str::to_owned);
            status.site_signed_in = browser.signed_in().to_vec();
        }
        if let Some(player) = &mut tv {
            for message in player.take_messages() {
                let _ = webview.post_json(&message);
            }
        }

        // What the player typed on their own keyboard, and whether it types (the plugin decides: the truck
        // standing still, a field selected then).
        let typing_block = &bridge.bridge().typing;
        let typed: Vec<u32> = std::iter::from_fn(|| typing_block.keys.pop()).collect();
        if let Some(message) = typing::typed_message(&typed, typing::clipboard_text) {
            let _ = webview.post_json(&message);
        }
        let state = westplay_shared::TypingState::from_code(typing_block.state.load(Ordering::Acquire));
        if typing_state != Some(state) {
            typing_state = Some(state);
            let _ = webview.post_json(&typing::state_message(state));
        }

        // YouTube Music's and Spotify's sound from the truck's own speakers, heard from the
        // driver's head (as last told, every 50 ms); muffled with the camera outside the cab, seen on every pass (as
        // quick as the game's cut).
        if let (Some(truck), Some(telemetry)) = (&last_truck, &last_telemetry) {
            let plugin = &bridge.bridge().plugin;
            // The cab drawn: TruckersTV's screen, or the cab's mark (the TV's left out as the player looks away).
            let cab_drawn = camera::drawn_frame(
                plugin.screen_frame.load(Ordering::Acquire),
                plugin.cab_frame.load(Ordering::Acquire),
            );
            let game_frame = plugin.heartbeat.load(Ordering::Acquire);
            let (electrics, paused) = (telemetry.electric_enabled, telemetry.paused);
            let outside = camera.outside(truck.id.as_str(), cab_drawn, game_frame, electrics, paused);
            if outside != was_outside {
                was_outside = outside;
                log!("camera: {} the cab", if outside { "outside" } else { "in" });
            }
            // The game's menu gone while it's still paused: the music back as its still picture shows.
            let gone = menu.update(paused, camera::drawn(cab_drawn, game_frame), outside);
            if gone != menu_gone {
                menu_gone = gone;
                if gone {
                    log!("the game's menu has gone, the game about to run again");
                }
                let _ = webview.post_json(&serde_json::json!({ "type": "menuGone", "gone": gone }).to_string());
            }
            let distance = camera::Camera::distance(telemetry, outside).unwrap_or(0.0) as f32;
            let turn = game_settings.camera().steering_turn(telemetry.steering);
            let head = speakers::Head::of(telemetry).turned(turn);
            // The truck's speakers: its parts' for a truck whose speakers come with them (none until they're
            // found), else its own.
            let truck_speakers = match &part_speakers {
                Some(found) if found.truck == *truck => found.speakers.as_deref(),
                _ if speakers::by_parts(truck.id.as_str()) => None,
                _ => speakers::in_truck(truck.id.as_str()),
            };
            if let Some(player) = &youtube_music {
                let _ = player.cab(truck_speakers, head, outside, distance, equalizer, spectrum);
            }
            if let Some(player) = &spotify {
                let _ = player.cab(truck_speakers, head, outside, distance, equalizer, spectrum);
            }
        }

        if last_telemetry_post.elapsed() >= Duration::from_millis(50) {
            last_telemetry_post = Instant::now();
            let plugin = &bridge.bridge().plugin;
            // A write the game never finished (it crashed in the middle) reads as nothing: nothing to send. A
            // game that closed leaves its last telemetry behind: not sent (a page reloading would power on).
            if game_running
                && let Some(telemetry) = plugin.telemetry.read()
                && last_telemetry != Some(telemetry)
            {
                last_telemetry = Some(telemetry);
                let message = serde_json::json!({ "type": "telemetry", "data": telemetry });
                let _ = webview.post_json(&message.to_string());
            }
            if let Some(truck) = plugin.truck.read()
                && last_truck != Some(truck)
            {
                if truck != TruckInfo::default() {
                    startup::mark("host: truck sent to the page");
                }
                last_truck = Some(truck);
                player_watch.truck(truck);
                let message = serde_json::json!({ "type": "truck", "data": truck });
                let _ = webview.post_json(&message.to_string());
            }
            if let Some(job) = plugin.job.read()
                && last_job != Some(job)
            {
                last_job = Some(job);
                let message = serde_json::json!({ "type": "job", "data": job });
                let _ = webview.post_json(&message.to_string());
            }
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    host.frame_generation.store(0, Ordering::Release);
    host.screen_enabled.store(0, Ordering::Release);
    host.pid.store(0, Ordering::Release);
    drop(pipeline);
    if browser_gone {
        return Err(Box::new(BrowserGone));
    }
    if gpu_changed {
        return Err(Box::new(graphics::GpuChanged));
    }
    Ok(())
}

/// Whether the game's plugin is alive, judged by its heartbeat (its frame count) having moved within
/// [`GAME_TIMEOUT`]. The first value seen proves nothing: a game that's gone may have left it.
#[derive(Default)]
struct Liveness {
    last: Option<(u64, Instant)>,
}

/// A game whose heartbeat stands still this long is taken to be gone (crashed, or stopped responding).
const GAME_TIMEOUT: Duration = Duration::from_secs(3);

impl Liveness {
    fn update(&mut self, heartbeat: u64, now: Instant) -> bool {
        match self.last {
            Some((seen, _)) if seen == heartbeat => {}
            Some(_) => self.last = Some((heartbeat, now)),
            None => self.last = Some((heartbeat, now.checked_sub(GAME_TIMEOUT).unwrap_or(now))),
        }
        self.last.is_some_and(|(_, moved)| now.duration_since(moved) < GAME_TIMEOUT)
    }
}

/// Being the one TruckersTV running in this Windows session: two would both drive the game's bridge (taking
/// turns at its typed-key queue, which has room for one reader, and flipping the game between their frames).
/// Held for as long as the app runs; a TruckersTV that's killed lets go with its process.
pub struct Instance(windows::Win32::Foundation::HANDLE);

// SAFETY: a handle to a named mutex, only ever closed (once, as it goes), from whichever thread.
unsafe impl Send for Instance {}

impl Instance {
    /// Claims being the one TruckersTV, or `None` if another is running.
    pub fn claim() -> Option<Self> {
        Self::claim_named(windows::core::w!(r"Local\TruckersTV.Instance"))
    }

    fn claim_named(name: windows::core::PCWSTR) -> Option<Self> {
        use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError};
        // SAFETY: a plain call: the name is a static string, the mutex is the process's own.
        let handle = unsafe { windows::Win32::System::Threading::CreateMutexW(None, false, name) }.ok()?;
        // SAFETY: reads this thread's last error, right after the call it's for.
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            // SAFETY: `handle` was just opened, and nothing else holds it.
            unsafe {
                let _ = CloseHandle(handle);
            }
            return None;
        }
        Some(Self(handle))
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // SAFETY: the mutex is this `Instance`'s own, closed once, as it goes.
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

/// Tells the player TruckersTV is already running (a second one was started), in a message box.
pub fn tell_already_running() {
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK, MessageBoxW};
    // SAFETY: a message box with static strings, no owner window.
    unsafe {
        MessageBoxW(
            None,
            windows::core::w!("TruckersTV is already running. Find it in the taskbar or the notification area."),
            windows::core::w!("TruckersTV"),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

/// [`run`] stopped because WebView2's browser process exited, taking the cabin page's WebView with it: run it
/// again, on a new thread (the thread's composition, set up for the WebView, can't be set up twice).
#[derive(Debug)]
pub struct BrowserGone;

impl std::fmt::Display for BrowserGone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the cabin browser (WebView2) stopped")
    }
}

impl std::error::Error for BrowserGone {}

/// Logs the startup stages the host sees by watching: the game connecting, the first frame captured, the
/// game first drawing one on the truck's screen (its frame counter moving on from `screen_frame_before`).
/// Whether any are still to come.
fn log_startup(bridge: &BridgeMapping, pipeline: Option<&Pipeline>, screen_frame_before: u64) -> bool {
    let plugin = &bridge.bridge().plugin;
    let stages = [
        (plugin.pid.load(Ordering::Relaxed) != 0, "game: connected"),
        (pipeline.is_some_and(|p| p.capture.frames.load(Ordering::Relaxed) > 0), "host: first frame captured"),
        (
            plugin.screen_frame.load(Ordering::Relaxed) != screen_frame_before,
            "game: drawing TruckersTV on the truck's screen",
        ),
    ];
    for (reached, stage) in stages {
        if reached {
            startup::mark(stage);
        }
    }
    !stages.iter().all(|(reached, _)| *reached)
}

/// Whether the process `game` (its pid; 0: no game) has a window showing: shown, not minimised, not cloaked
/// (on another virtual desktop). In front or not: Lossless Scaling's window is in front of a game it scales,
/// and a window over the game leaves it showing (owner 2026-10-28). Without a game, it counts as showing
/// (nothing to be away from).
fn is_shown(game: u32) -> bool {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, IsIconic, IsWindowVisible};
    use windows::core::BOOL;

    struct Looking {
        game: u32,
        shown: bool,
    }
    unsafe extern "system" fn each(window: HWND, looking: LPARAM) -> BOOL {
        // SAFETY: `looking` is the `Looking` below, alive through `EnumWindows`; the window is only asked about.
        unsafe {
            let looking = &mut *(looking.0 as *mut Looking);
            let mut owner = 0;
            GetWindowThreadProcessId(window, Some(&mut owner));
            if owner != looking.game || !IsWindowVisible(window).as_bool() || IsIconic(window).as_bool() {
                return BOOL(1);
            }
            let mut cloaked = 0u32;
            let read = DwmGetWindowAttribute(window, DWMWA_CLOAKED, (&raw mut cloaked).cast(), size_of::<u32>() as u32);
            looking.shown = read.is_err() || cloaked == 0;
            // Found one showing: done.
            BOOL(i32::from(!looking.shown))
        }
    }

    if game == 0 {
        return true;
    }
    let mut looking = Looking { game, shown: false };
    // SAFETY: `each` only reads the windows and writes `looking`, which outlives the call. Stopping early makes
    // it return an error: what was found stands.
    let _ = unsafe { EnumWindows(Some(each), LPARAM(&raw mut looking as isize)) };
    looking.shown
}

/// Whether the process `game` (its pid; 0: no game) is running: the plugin clears its pid as the game closes,
/// and a game that crashed leaves it behind, its process gone.
fn is_running(game: u32) -> bool {
    use windows::Win32::Foundation::{CloseHandle, WAIT_TIMEOUT};
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject};
    if game == 0 {
        return false;
    }
    // SAFETY: a handle opened here, waited on without waiting (signalled: the process has ended), and closed.
    unsafe {
        let Ok(process) = OpenProcess(PROCESS_SYNCHRONIZE, false, game) else { return false };
        let running = WaitForSingleObject(process, 0) == WAIT_TIMEOUT;
        let _ = CloseHandle(process);
        running
    }
}

/// `{"type":"cursor","host":…}`: whether the host draws the screen's cursor (`cursor`), or the page draws its own.
fn cursor_message(host: bool) -> String {
    serde_json::json!({ "type": "cursor", "host": host }).to_string()
}

/// `{"type":"game","running":…}`: whether the game runs. As it closes, the page powers off (as with the
/// truck's electrics off) and forgets the game's last telemetry, so the next game starts it afresh.
fn game_message(running: bool) -> String {
    serde_json::json!({ "type": "game", "running": running }).to_string()
}

/// The most presses of a keybind the page is told of at once.
const MOST_KEYBIND_PRESSES: u32 = 10;

/// `{"type":"keybind","action":"volumeUp"|"volumeDown"|"playPause"}`: the player pressed a keybind in the game.
fn keybind_message(keybind: Keybind) -> String {
    let action = match keybind {
        Keybind::VolumeUp => "volumeUp",
        Keybind::VolumeDown => "volumeDown",
        Keybind::PlayPause => "playPause",
        // The plugin's own (the camera), never counted for the app.
        Keybind::Zoom => "zoom",
    };
    serde_json::json!({ "type": "keybind", "action": action }).to_string()
}

/// `{"type":"gameShown","shown":…}`: whether the game's window shows (`is_shown`).
fn game_shown_message(shown: bool) -> String {
    serde_json::json!({ "type": "gameShown", "shown": shown }).to_string()
}

/// The site browser, started if it isn't yet.
fn start_site_browser<'a>(
    browser: &'a mut Option<site_browser::Browser>,
    stage: &Stage,
    config: &Config,
    webview: &OffscreenWebView,
) -> Option<&'a mut site_browser::Browser> {
    if browser.is_none() {
        match site_browser::Browser::new(stage, &config.data_dir.join("site-browser")) {
            Ok(started) => *browser = Some(started),
            Err(error) => {
                log!("[site browser] could not start: {error}");
                let _ = webview.post_json(r#"{"type":"browserState","url":"","title":"","loading":false,"back":false,"forward":false,"error":"Could not start the browser. Close and reopen it to retry."}"#);
            }
        }
    }
    browser.as_mut()
}

/// YouTube Music's web player, started as the page first shows it (on the cabin page's profile:
/// they share the sign-in), or `None` if it couldn't start.
fn start_youtube_music<'a>(
    youtube_music: &'a mut Option<youtube_music::YouTubeMusic>,
    stage: &Stage,
    config: &Config,
) -> Option<&'a youtube_music::YouTubeMusic> {
    if youtube_music.is_none() {
        let user_data = config.data_dir.join("webview2");
        match youtube_music::YouTubeMusic::start(stage, config.width, config.height, &user_data) {
            Ok(started) => {
                log!("[youtube music] opened");
                *youtube_music = Some(started)
            }
            Err(e) => startup::mark(&format!("host: YouTube Music couldn't start: {e}")),
        }
    }
    youtube_music.as_ref()
}

/// Spotify's web player, started as the page first shows it (on a WebView2 profile of its own: signing out
/// of Spotify forgets nothing else), or `None` if it couldn't start.
fn start_spotify<'a>(
    spotify: &'a mut Option<spotify::Spotify>,
    stage: &Stage,
    config: &Config,
) -> Option<&'a spotify::Spotify> {
    if spotify.is_none() {
        let user_data = config.data_dir.join("webview2-spotify");
        match spotify::Spotify::start(stage, config.width, config.height, &user_data) {
            Ok(started) => {
                log!("[spotify] opened");
                *spotify = Some(started)
            }
            Err(e) => startup::mark(&format!("host: Spotify couldn't start: {e}")),
        }
    }
    spotify.as_ref()
}

fn build_pipeline(
    config: &Config,
    stage: &Stage,
    bridge: &Arc<BridgeMapping>,
    luid: Option<LUID>,
    generation: u32,
) -> Result<Pipeline, Error> {
    let gpu = Gpu::new(luid)?;
    let shared = Arc::new(SharedFrame::new(&gpu, config.width, config.height, generation)?);
    let capture = Capture::start(&gpu, &stage.visual()?, shared, bridge.clone())?;
    Ok(Pipeline { gpu, capture })
}

/// The GPU the game renders on, once the plugin has published it.
fn game_adapter(bridge: &BridgeMapping) -> Option<LUID> {
    let plugin = &bridge.bridge().plugin;
    if plugin.pid.load(Ordering::Acquire) == 0 {
        return None;
    }
    let luid = LUID {
        LowPart: plugin.adapter_luid_low.load(Ordering::Relaxed),
        HighPart: plugin.adapter_luid_high.load(Ordering::Relaxed),
    };
    (luid.LowPart != 0 || luid.HighPart != 0).then_some(luid)
}

/// `{"type":"settings","equalizer":…}`: the settings the cabin page applies itself.
fn settings_message(settings: &settings::Settings) -> String {
    serde_json::json!({
        "type": "settings",
        "equalizer": settings.sound.equalizer.clamped(),
        "equalizerCustom": settings.sound.custom,
    })
    .to_string()
}

/// Where TV's player page is: beside the cabin page (its UI folder), or on the same dev server.
fn tv_page(content: &Content) -> Option<tv_view::TvPage> {
    match content {
        Content::Folder { root, .. } => Some(tv_view::TvPage::Folder(root.clone())),
        Content::Url(url) => {
            let mut page = url::Url::parse(url).ok()?;
            page.set_path("/tv-player.html");
            page.set_query(None);
            Some(tv_view::TvPage::Url(page.to_string()))
        }
        Content::Html(_) => None,
    }
}

/// Messages from the cabin page (`PageMessage` in ui/src/shared/host.ts).
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum PageMessage {
    Browser { command: site_browser::Command },
    Tv { command: tv_view::Command },
    /// Posted when the page (re)loads.
    Ready,
    /// A startup stage the page reached (the boot screens shown), for the startup log (`startup`).
    Startup { stage: String },
    /// The players' tone, set in the cab's Settings (Equalizer): the player's curve (bass, mid and treble, dB
    /// each), and whether it plays (Custom) or the speakers as tuned (Recommended).
    Equalizer {
        #[serde(flatten)]
        equalizer: speakers::Equalizer,
        #[serde(default)]
        custom: bool,
    },
    /// The music's spectrum wanted (the cab's Equalizer open) or not: the players measure it, and it's passed
    /// on to the page as they say it.
    Spectrum { on: bool },
    /// Whether what's under the cursor is light (it's drawn dark there: `cursor`).
    CursorLight { light: bool },
    /// A command for Spotify's web player (`spotify`): its sign-in, a question, its controls.
    Spotify { command: spotify::Command },
    /// A command for YouTube Music's web player (`youtubeMusic`): its sign-in, a question, its controls.
    YoutubeMusic { command: youtube_music::Command },
    /// The text field selected on the page, a number that changes with every selection (0: none), for the
    /// player's own keyboard to type into (`typing`).
    Typing { selection: u32 },
}

fn parse_page_message(message: &str) -> Option<PageMessage> {
    serde_json::from_str(message).ok()
}

/// What the player's latest save says, for the page: `{"type":"gameLanguage","language":"tr_tr"}`.
fn player_messages_for(player: &Player) -> Vec<String> {
    vec![serde_json::json!({ "type": "gameLanguage", "language": player.language }).to_string()]
}

fn same_luid(a: LUID, b: LUID) -> bool {
    a.LowPart == b.LowPart && a.HighPart == b.HighPart
}

fn pump_messages() {
    // SAFETY: this thread's own messages, each passed on as Windows hands it; `msg` is a local.
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PageMessage, parse_page_message};

    #[test]
    fn the_game_shows_while_its_window_does_in_front_or_not() {
        use windows::Win32::System::Threading::GetCurrentProcessId;
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, SW_HIDE, SW_MINIMIZE, SW_SHOWNOACTIVATE, ShowWindow, WINDOW_EX_STYLE,
            WS_OVERLAPPEDWINDOW,
        };
        use windows::core::w;
        // SAFETY: a plain query.
        let this = unsafe { GetCurrentProcessId() };
        // No game: nothing to be away from.
        assert!(super::is_shown(0));
        // No window of its own yet.
        assert!(!super::is_shown(this));
        // SAFETY: a window of this test's own, shown, minimised, hidden and destroyed here.
        unsafe {
            let window = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("game"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                200,
                100,
                None,
                None,
                None,
                None,
            )
            .unwrap();
            // Shown without being brought to the front: it shows.
            let _ = ShowWindow(window, SW_SHOWNOACTIVATE);
            assert!(super::is_shown(this));
            let _ = ShowWindow(window, SW_MINIMIZE);
            assert!(!super::is_shown(this));
            let _ = ShowWindow(window, SW_HIDE);
            assert!(!super::is_shown(this));
            let _ = DestroyWindow(window);
        }
    }

    #[test]
    fn reads_the_pages_messages() {
        assert_eq!(parse_page_message(r#"{"type":"ready"}"#), Some(PageMessage::Ready));
        // The PC's own music apps aren't followed (removed 2026-10-17): their controls are no message.
        assert_eq!(parse_page_message(r#"{"type":"media","source":"Music.exe","action":"playPause"}"#), None);
        assert_eq!(
            parse_page_message(r#"{"type":"startup","stage":"power splash"}"#),
            Some(PageMessage::Startup { stage: "power splash".into() })
        );
        assert_eq!(
            parse_page_message(r#"{"type":"equalizer","bass":-3,"mid":0,"treble":2,"custom":true}"#),
            Some(PageMessage::Equalizer {
                equalizer: crate::speakers::Equalizer { bass: -3, mid: 0, treble: 2 },
                custom: true
            })
        );
        // A band not said is flat; one past a byte is no message (one past the range is kept to it as it's set).
        assert_eq!(
            parse_page_message(r#"{"type":"equalizer","bass":4}"#),
            Some(PageMessage::Equalizer {
                equalizer: crate::speakers::Equalizer { bass: 4, mid: 0, treble: 0 },
                custom: false
            })
        );
        assert_eq!(parse_page_message(r#"{"type":"equalizer","bass":400}"#), None);
        assert_eq!(parse_page_message(r#"{"type":"spectrum","on":true}"#), Some(PageMessage::Spectrum { on: true }));
        assert_eq!(
            parse_page_message(r#"{"type":"cursorLight","light":true}"#),
            Some(PageMessage::CursorLight { light: true })
        );
        assert_eq!(
            parse_page_message(r#"{"type":"typing","selection":3}"#),
            Some(PageMessage::Typing { selection: 3 })
        );
        assert_eq!(parse_page_message(r#"{"type":"other"}"#), None);
        assert_eq!(parse_page_message("not json"), None);
    }

    #[test]
    fn only_one_westplay_claims_the_session() {
        // A name of the test's own (a TruckersTV may be running on this PC).
        let name = windows::core::w!(r"Local\TruckersTV.Instance.Test");
        let first = super::Instance::claim_named(name).expect("the first claims it");
        assert!(super::Instance::claim_named(name).is_none(), "a second must not");
        drop(first);
        assert!(super::Instance::claim_named(name).is_some(), "once let go, it can be claimed again");
    }

    #[test]
    fn a_game_runs_while_its_process_does() {
        use super::is_running;
        assert!(!is_running(0));
        assert!(is_running(std::process::id()));
        // A game that crashed: its pid left behind, its process ended.
        let mut ended = std::process::Command::new("cmd").args(["/c", "exit"]).spawn().unwrap();
        let pid = ended.id();
        ended.wait().unwrap();
        assert!(!is_running(pid));
    }

    #[test]
    fn a_game_is_alive_while_its_heartbeat_moves() {
        use std::time::{Duration, Instant};
        let start = Instant::now() + Duration::from_secs(60);
        let mut liveness = super::Liveness::default();
        // A heartbeat already there proves nothing (a crashed game leaves it behind).
        assert!(!liveness.update(500, start));
        assert!(!liveness.update(500, start + Duration::from_secs(1)));
        // Moving: alive.
        assert!(liveness.update(560, start + Duration::from_secs(2)));
        assert!(liveness.update(560, start + Duration::from_secs(4)));
        // Standing still past the timeout (the game crashed): gone.
        assert!(!liveness.update(560, start + Duration::from_secs(2) + super::GAME_TIMEOUT));
        // Moving again (a new game): alive.
        assert!(liveness.update(3, start + Duration::from_secs(20)));
    }

    #[test]
    fn passes_on_the_companions_sign_ins_once() {
        use super::{AccountAction, AccountRequests};
        let requests = AccountRequests::default();
        assert_eq!(requests.youtube_music.take(), None);
        requests.youtube_music.request(AccountAction::SignOut);
        requests.youtube_music.request(AccountAction::SignIn);
        requests.spotify.request(AccountAction::Check);
        assert_eq!(requests.youtube_music.take(), Some(AccountAction::SignIn));
        assert_eq!(requests.youtube_music.take(), None);
        assert_eq!(requests.spotify.take(), Some(AccountAction::Check));
        assert_eq!(requests.spotify.take(), None);
        // The companion hears whether YouTube Music is signed in, and nothing more of the account.
        let status = super::Status { youtube_music_signed_in: Some(true), youtube_music_signing_in: false, ..Default::default() };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!((json["youtubeMusicSignedIn"].as_bool(), json["youtubeMusicSigningIn"].as_bool()), (Some(true), Some(false)));
        let status = super::Status { spotify_signed_in: Some(false), spotify_signing_in: true, ..Default::default() };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!((json["spotifySignedIn"].as_bool(), json["spotifySigningIn"].as_bool()), (Some(false), Some(true)));
    }

}
