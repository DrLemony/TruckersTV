//! An isolated, lazily created web browser composed into the cabin frame.
use crate::signin::{Progress, SignInWindow};
use crate::webview::{Error, OffscreenWebView, Stage};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};
use webview2_com::Microsoft::Web::WebView2::Win32::{COREWEBVIEW2_PERMISSION_STATE_DENY, ICoreWebView2_4};
use webview2_com::{
    CallDevToolsProtocolMethodCompletedHandler, CoTaskMemPWSTR, DownloadStartingEventHandler,
    NavigationCompletedEventHandler, NavigationStartingEventHandler, NewWindowRequestedEventHandler,
    PermissionRequestedEventHandler,
};
use windows::core::BOOL;
use windows::core::Interface;
use windows::core::{HSTRING, PWSTR};

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum Command {
    Navigate { url: String },
    Back,
    Forward,
    Reload,
    Stop,
    Hide,
    Close,
    Show { x: f32, y: f32, width: u32, height: u32 },
    Zoom { factor: f64 },
    Input { event: Input },
    /// A site app's sign-in, in a window on the desktop (signin.rs), on this browser's profile.
    SignIn { app: String },
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Input {
    Text { text: String },
    Key { key: String },
}

#[derive(Default, Serialize)]
struct State {
    url: String,
    title: String,
    loading: bool,
    back: bool,
    forward: bool,
    error: Option<String>,
}

/// Whether what was typed without a scheme is a site (`netflix.com/x`, `localhost:3000`, an IPv4 address) rather
/// than words to search for: no spaces, and a host with a letters-only top-level domain.
fn looks_like_address(input: &str) -> bool {
    let host = input.split(['/', '?', '#']).next().unwrap_or_default();
    let host = match host.rsplit_once(':') {
        Some((name, port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => name,
        _ => host,
    };
    !input.contains(char::is_whitespace)
        && (host.eq_ignore_ascii_case("localhost")
            || host.parse::<std::net::Ipv4Addr>().is_ok()
            || host.rsplit_once('.').is_some_and(|(name, tld)| {
                !name.is_empty() && tld.chars().count() >= 2 && tld.chars().all(char::is_alphabetic)
            }))
}

/// Only web navigation, including redirects and popups. No local files, scripts, credentials or app origins.
/// Words that aren't an address are a Google search.
pub fn address(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() || input.len() > 8192 {
        return None;
    }
    let candidate = if input.contains("://")
        || input.starts_with("about:")
        || input.starts_with("javascript:")
        || input.starts_with("data:")
        || input.starts_with("file:")
    {
        input.to_owned()
    } else if looks_like_address(input) {
        format!("https://{input}")
    } else {
        // Plain words are a search, as in a browser's own address bar.
        return url::Url::parse_with_params("https://www.google.com/search", [("q", input)]).ok().map(String::from);
    };
    let parsed = url::Url::parse(&candidate).ok()?;
    let host = parsed.host_str()?;
    if !matches!(parsed.scheme(), "http" | "https")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || host == "westplay.cabin"
        || host == "westplay.maps"
        || host == "westplay.customize"
    {
        return None;
    }
    Some(parsed.to_string())
}

pub struct Browser {
    pub view: OffscreenWebView,
    pub area: Option<(f32, f32, u32, u32)>,
    state: Rc<RefCell<State>>,
    focus: Rc<RefCell<Option<bool>>>,
    pub check_focus: bool,
    /// The page was pressed: its full screen's X shows (the cabin page under it hears nothing of it).
    pub pressed: bool,
    last_poll: Instant,
    sent: String,
    /// The sign-in window open, and whose. The page shows signed in once it's closed.
    sign_in: Option<(String, SignInWindow)>,
    /// The site apps signed in, kept in `profile`.
    signed_in: Vec<String>,
    profile: PathBuf,
    sent_sign_in: String,
}

const SIGNED_IN: &str = "signed-in.txt";

/// The site apps signed in, as `profile` keeps them (a line each). A session running out isn't seen: their
/// sign-in opens again from the companion.
pub fn signed_in(profile: &Path) -> Vec<String> {
    std::fs::read_to_string(profile.join(SIGNED_IN))
        .map(|kept| kept.lines().map(str::to_owned).collect())
        .unwrap_or_default()
}

impl Browser {
    pub fn new(stage: &Stage, profile: &Path) -> Result<Self, Error> {
        // Without the GPU (a browser's "hardware acceleration" off), protected video (Netflix's) is drawn where
        // the screen's capture sees it, not black; Chrome's own process, so the cab's other views keep theirs.
        let view = OffscreenWebView::with_arguments(stage, 800, 600, profile, "--disable-gpu")?;
        let core = view.core();
        let state = Rc::new(RefCell::new(State::default()));
        let focus = Rc::new(RefCell::new(None));
        // SAFETY: these COM objects are alive on their owning STA thread. No remote-page messaging or host objects.
        unsafe {
            let settings = core.Settings()?;
            settings.SetIsWebMessageEnabled(false)?;
            settings.SetAreHostObjectsAllowed(false)?;
            settings.SetAreDevToolsEnabled(false)?;
            let s = state.clone();
            let f = focus.clone();
            let starting = NavigationStartingEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    let mut uri = PWSTR::null();
                    args.Uri(&mut uri)?;
                    let uri = CoTaskMemPWSTR::from(uri).to_string();
                    if address(&uri).is_none() {
                        args.SetCancel(true)?;
                        return Ok(());
                    }
                    let mut s = s.borrow_mut();
                    *f.borrow_mut() = Some(false);
                    s.loading = true;
                    s.error = None;
                    s.url = uri;
                }
                Ok(())
            }));
            let mut token = 0;
            core.add_NavigationStarting(&starting, &mut token)?;
            let s = state.clone();
            let completed = NavigationCompletedEventHandler::create(Box::new(move |_, args| {
                let mut s = s.borrow_mut();
                s.loading = false;
                if let Some(args) = args {
                    let mut success = BOOL::default();
                    args.IsSuccess(&mut success)?;
                    if !success.as_bool() {
                        s.error = Some("Could not load this page. Check the address or connection and retry.".into());
                    }
                }
                Ok(())
            }));
            core.add_NavigationCompleted(&completed, &mut token)?;
            let popup = NewWindowRequestedEventHandler::create(Box::new(move |sender, args| {
                if let Some(args) = args {
                    args.SetHandled(true)?;
                    let mut uri = PWSTR::null();
                    args.Uri(&mut uri)?;
                    let uri = CoTaskMemPWSTR::from(uri).to_string();
                    if let (Some(core), Some(uri)) = (sender, address(&uri)) {
                        core.Navigate(&HSTRING::from(uri))?;
                    }
                }
                Ok(())
            }));
            core.add_NewWindowRequested(&popup, &mut token)?;
            let permission = PermissionRequestedEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY)?;
                }
                Ok(())
            }));
            core.add_PermissionRequested(&permission, &mut token)?;
            let s = state.clone();
            let download = DownloadStartingEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    args.SetCancel(true)?;
                    args.SetHandled(true)?;
                }
                s.borrow_mut().error = Some("File downloads are not available in the cabin browser.".into());
                Ok(())
            }));
            core.cast::<ICoreWebView2_4>()?.add_DownloadStarting(&download, &mut token)?;
        }
        stage.add(&view, true)?;
        view.show(false)?;
        Ok(Self {
            view,
            area: None,
            state,
            focus,
            check_focus: false,
            pressed: false,
            last_poll: Instant::now() - Duration::from_secs(1),
            sent: String::new(),
            sign_in: None,
            signed_in: signed_in(profile),
            profile: profile.to_owned(),
            sent_sign_in: String::new(),
        })
    }

    pub fn command(&mut self, command: Command, stage: &Stage) -> Result<(), Error> {
        match command {
            Command::Navigate { url } => {
                // The cabin page may be new: it's told who's signed in again.
                self.sent_sign_in.clear();
                if let Some(url) = address(&url) {
                    self.view.navigate(&url)?;
                } else {
                    self.state.borrow_mut().error = Some("Enter an HTTP or HTTPS address.".into());
                }
            }
            Command::Show { x, y, width, height } => {
                if self.area.is_none() {
                    self.sent.clear();
                }
                let (sw, sh) = stage.page_size().get();
                if !x.is_finite()
                    || !y.is_finite()
                    || x < 0.0
                    || y < 0.0
                    || width == 0
                    || height == 0
                    || x + width as f32 > sw as f32 + 1.0
                    || y + height as f32 > sh as f32 + 1.0
                {
                    return Ok(());
                }
                self.view.place(x, y, width, height)?;
                self.view.show(true)?;
                self.view.set_muted(false)?;
                self.area = Some((x, y, width, height));
            }
            Command::Hide | Command::Close => {
                self.area = None;
                self.view.show(false)?;
                self.view.set_muted(true)?;
            }
            Command::Reload => {
                self.view.reload()?;
            }
            Command::SignIn { app } => {
                if let Some((open, window)) = &self.sign_in
                    && *open == app
                {
                    window.bring_forward();
                } else if let Some(site) = crate::signin::site_of(&app) {
                    self.sign_in = Some((app, SignInWindow::open(self.view.environment(), site)?));
                }
            }
            Command::Zoom { factor } if factor.is_finite() => {
                self.view.zoom(factor.clamp(0.5, 2.0))?;
            }
            // On a small screen the keyboard can cover the entire surface. The trusted cabin still owns
            // the selected field, and detaches it when leaving the browser.
            Command::Input { event } => {
                self.input(event)?;
            }
            command => {
                // SAFETY: the control is alive on this STA; navigation commands have no pointer arguments.
                unsafe {
                    match command {
                        Command::Back => self.view.core().GoBack()?,
                        Command::Forward => self.view.core().GoForward()?,
                        Command::Stop => {
                            self.view.core().Stop()?;
                            self.state.borrow_mut().loading = false;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn input(&self, event: Input) -> windows::core::Result<()> {
        match event {
            Input::Text { text } => self
                .devtools("Input.insertText", serde_json::json!({"text": text.chars().take(8192).collect::<String>()})),
            Input::Key { key } => {
                let (name, code, modifiers) = match key.as_str() {
                    "backspace" => ("Backspace", 8, 0),
                    "wordBackspace" => ("Backspace", 8, 2),
                    "enter" => ("Enter", 13, 0),
                    "escape" => ("Escape", 27, 0),
                    "tab" => ("Tab", 9, 0),
                    "delete" => ("Delete", 46, 0),
                    "left" => ("ArrowLeft", 37, 0),
                    "right" => ("ArrowRight", 39, 0),
                    "home" => ("Home", 36, 0),
                    "end" => ("End", 35, 0),
                    "selectAll" => ("a", 65, 2),
                    _ => return Ok(()),
                };
                for kind in ["keyDown", "keyUp"] {
                    let mut args =
                        serde_json::json!({"type":kind,"key":name,"windowsVirtualKeyCode":code,"modifiers":modifiers});
                    if code == 13 && kind == "keyDown" {
                        args["text"] = "\r".into();
                    }
                    self.devtools("Input.dispatchKeyEvent", args)?;
                }
                Ok(())
            }
        }
    }

    fn devtools(&self, method: &str, args: serde_json::Value) -> windows::core::Result<()> {
        let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(|_, _| Ok(())));
        // SAFETY: fixed host-owned method names, JSON encoded arguments, and a retained completion handler.
        unsafe {
            self.view.core().CallDevToolsProtocolMethod(
                &HSTRING::from(method),
                &HSTRING::from(args.to_string()),
                &handler,
            )
        }
    }

    /// The site app whose sign-in window is open, for the companion.
    pub fn signing_in(&self) -> Option<&str> {
        self.sign_in.as_ref().map(|(app, _)| app.as_str())
    }

    /// The site apps signed in, for the companion.
    pub fn signed_in(&self) -> &[String] {
        &self.signed_in
    }

    pub fn tick(&mut self, cabin: &OffscreenWebView) {
        if let Some(editable) = self.focus.borrow_mut().take() {
            let _ = cabin.post_json(
                &serde_json::json!({"type":"browserFocus","editable":editable && self.area.is_some()}).to_string(),
            );
        }
        if std::mem::take(&mut self.pressed) {
            let _ = cabin.post_json(r#"{"type":"browserPress"}"#);
        }
        if self.last_poll.elapsed() < Duration::from_millis(200) {
            return;
        }
        self.last_poll = Instant::now();
        let progress = self.sign_in.as_ref().map(|(_, window)| window.progress());
        if matches!(progress, Some(Progress::Closed | Progress::SignedIn)) {
            if let Some((app, _)) = self.sign_in.take()
                && progress == Some(Progress::SignedIn)
                && !self.signed_in.contains(&app)
            {
                self.signed_in.push(app);
                if let Err(error) = std::fs::write(self.profile.join(SIGNED_IN), self.signed_in.join("\n")) {
                    crate::log!("[site browser] could not keep the sign-in: {error}");
                }
            }
            let _ = self.view.reload();
        }
        let message = serde_json::json!({
            "type": "browserSignIn",
            "open": self.sign_in.is_some(),
            "signedIn": self.signed_in,
        })
        .to_string();
        if message != self.sent_sign_in {
            let _ = cabin.post_json(&message);
            self.sent_sign_in = message;
        }
        if self.view.take_failure().is_some() {
            self.state.borrow_mut().error = Some("Browser stopped. Close and reopen it to try again.".into());
            self.state.borrow_mut().loading = false;
        }
        // SAFETY: native properties from a live control; strings freed by CoTaskMemPWSTR.
        unsafe {
            let core = self.view.core();
            let mut s = self.state.borrow_mut();
            let mut title = PWSTR::null();
            if core.DocumentTitle(&mut title).is_ok() {
                s.title = CoTaskMemPWSTR::from(title).to_string();
            }
            let mut uri = PWSTR::null();
            if core.Source(&mut uri).is_ok() {
                let uri = CoTaskMemPWSTR::from(uri).to_string();
                if address(&uri).is_some() {
                    s.url = uri;
                }
            }
            let mut back = BOOL::default();
            let mut forward = BOOL::default();
            let _ = core.CanGoBack(&mut back);
            let _ = core.CanGoForward(&mut forward);
            s.back = back.as_bool();
            s.forward = forward.as_bool();
        }
        if self.check_focus && self.area.is_some() {
            self.check_focus = false;
            let focus = self.focus.clone();
            // Only a boolean crosses the boundary. No field values or page-supplied commands are consumed.
            let _ = self.view.run_script("(() => { let e = document.activeElement; while (e?.shadowRoot?.activeElement) e = e.shadowRoot.activeElement; return !!e && (e.isContentEditable || e.tagName === 'IFRAME' || ((e.tagName === 'INPUT' || e.tagName === 'TEXTAREA') && !e.disabled && !e.readOnly)); })()", move |value| {
                *focus.borrow_mut() = Some(value == "true");
            });
        }
        let mut value = serde_json::to_value(&*self.state.borrow()).unwrap_or_default();
        value["type"] = "browserState".into();
        let message = value.to_string();
        if message != self.sent {
            let _ = cabin.post_json(&message);
            self.sent = message;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "renders the built browser controls; WESTPLAY_UI names the UI dist directory"]
    fn built_browser_controls() {
        use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
        // SAFETY: COM is initialized on this test's own thread before creating the view.
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap();
        }
        let ui = std::env::var("WESTPLAY_UI").expect("WESTPLAY_UI");
        let stage = Stage::new(1280, 720).unwrap();
        let view = OffscreenWebView::new(
            &stage,
            1280,
            720,
            &std::env::temp_dir().join(format!("westplay-browser-ui-{}", std::process::id())),
        )
        .unwrap();
        view.map_folder("westplay.cabin", Path::new(&ui)).unwrap();
        stage.add(&view, true).unwrap();
        // A layout preview has no continuous frame capture to advance native view transitions.
        view.add_startup_script("document.startViewTransition = undefined; window.__errors=[]; addEventListener('error',e=>__errors.push(e.message)); addEventListener('unhandledrejection',e=>__errors.push(String(e.reason)))").unwrap();
        view.navigate("https://westplay.cabin/cabin.html?demo").unwrap();
        let pump = || {
            let until = Instant::now() + Duration::from_secs(2);
            while Instant::now() < until {
                crate::pump_messages();
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        pump();
        for _ in 0..5 {
            if view.evaluate("!!document.querySelector('.tv-tile[data-app=chrome]')").unwrap() == "true" {
                break;
            }
            pump();
        }
        view.evaluate("document.querySelector('.tv-tile[data-app=chrome]').click()").unwrap();
        pump();
        view.capture_png(&std::env::temp_dir().join("browser-preview.png")).unwrap();
        println!(
            "browser preview: {}",
            view.evaluate("({errors:window.__errors,url:location.href, text:document.body.innerText.slice(0,600)})")
                .unwrap()
        );
        assert_eq!(view.evaluate("!!document.querySelector('.chrome-bar')").unwrap(), "true");
        assert_eq!(
            view.evaluate("document.querySelector('.chrome-page').getBoundingClientRect().height > 100").unwrap(),
            "true"
        );
        view.capture_png(&std::env::temp_dir().join("browser-preview.png")).unwrap();
    }
    #[test]
    fn accepts_web_addresses_only() {
        assert_eq!(address(" example.org/a?q=1 ").unwrap(), "https://example.org/a?q=1");
        assert_eq!(address("http://localhost:8080").unwrap(), "http://localhost:8080/");
        assert_eq!(address("netflix.com").unwrap(), "https://netflix.com/");
        assert_eq!(address("localhost:3000/x").unwrap(), "https://localhost:3000/x");
        assert_eq!(address("192.168.1.1").unwrap(), "https://192.168.1.1/");
        assert_eq!(address("tod.com.tr/canli-tv").unwrap(), "https://tod.com.tr/canli-tv");
        // Words are a search.
        assert_eq!(address("galatasaray maçı").unwrap(), "https://www.google.com/search?q=galatasaray+ma%C3%A7%C4%B1");
        assert_eq!(address("netflix").unwrap(), "https://www.google.com/search?q=netflix");
        assert_eq!(address("hava durumu 3.5").unwrap(), "https://www.google.com/search?q=hava+durumu+3.5");
        assert_eq!(address("v1.2").unwrap(), "https://www.google.com/search?q=v1.2");
        for value in [
            "",
            "javascript:alert(1)",
            "file:///C:/secret",
            "data:text/html,x",
            "about:blank",
            "ftp://example.org",
            "https://user:pass@example.org",
            "https://westplay.cabin/cabin.html",
        ] {
            assert!(address(value).is_none(), "{value}");
        }
    }

    /// Exercises the real native control, including mouse focus and Chromium editing in a cross-origin frame.
    #[test]
    #[ignore = "runs WebView2 against a local HTTP fixture"]
    fn native_navigation_and_input() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            sync::{
                Arc,
                atomic::{AtomicBool, Ordering},
            },
        };
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_MOUSE_EVENT_KIND_LEFT_BUTTON_DOWN, COREWEBVIEW2_MOUSE_EVENT_KIND_LEFT_BUTTON_UP,
            COREWEBVIEW2_MOUSE_EVENT_VIRTUAL_KEYS_LEFT_BUTTON, COREWEBVIEW2_MOUSE_EVENT_VIRTUAL_KEYS_NONE,
        };
        use windows::Win32::Foundation::POINT;
        use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
        // SAFETY: COM is initialized once on this test's own STA thread before creating any controls.
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap();
        }
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = server.local_addr().unwrap().port();
        server.set_nonblocking(true).unwrap();
        struct Server(Arc<AtomicBool>);
        impl Drop for Server {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }
        let stop = Arc::new(AtomicBool::new(false));
        let _server = Server(stop.clone());
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                if let Ok((mut stream, _)) = server.accept() {
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
                    let mut bytes = [0; 4096];
                    let count = stream.read(&mut bytes).unwrap_or(0);
                    let request = String::from_utf8_lossy(&bytes[..count]);
                    let page = if request.starts_with("GET /frame") {
                        "<input id=p style='position:absolute;left:10px;top:10px;width:200px;height:30px' oninput='parent.postMessage({testValue:this.value},\"*\")'>".to_string()
                    } else if request.starts_with("GET /second") {
                        "<title>Second</title><p>Second page</p>".to_string()
                    } else {
                        format!(
                            "<title>Fixture</title><script>addEventListener('message',e=>{{if(e.origin==='http://localhost:{port}')window.frameValue=e.data.testValue}})</script><input id=p type=password style='position:absolute;left:20px;top:20px;width:200px;height:30px'><iframe src='http://localhost:{port}/frame' style='position:absolute;left:20px;top:100px;width:300px;height:120px;border:0'></iframe>"
                        )
                    };
                    let body = format!("<!doctype html><meta charset=utf-8>{page}");
                    let _ = write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                } else {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        });
        let pump = |ms| {
            let end = Instant::now() + Duration::from_millis(ms);
            while Instant::now() < end {
                crate::pump_messages();
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        let profile = std::env::temp_dir().join(format!("westplay-browser-test-{}", std::process::id()));
        let stage = Stage::new(800, 600).unwrap();
        let mut browser = Browser::new(&stage, &profile).unwrap();
        browser.command(Command::Show { x: 0.0, y: 0.0, width: 800, height: 600 }, &stage).unwrap();
        browser.command(Command::Navigate { url: format!("http://127.0.0.1:{port}/") }, &stage).unwrap();
        pump(2500);
        assert_eq!(browser.view.evaluate("document.title").unwrap(), "\"Fixture\"");
        let click = |x, y| {
            let point = POINT { x, y };
            browser
                .view
                .send_mouse(
                    COREWEBVIEW2_MOUSE_EVENT_KIND_LEFT_BUTTON_DOWN,
                    COREWEBVIEW2_MOUSE_EVENT_VIRTUAL_KEYS_LEFT_BUTTON,
                    0,
                    point,
                )
                .unwrap();
            browser
                .view
                .send_mouse(
                    COREWEBVIEW2_MOUSE_EVENT_KIND_LEFT_BUTTON_UP,
                    COREWEBVIEW2_MOUSE_EVENT_VIRTUAL_KEYS_NONE,
                    0,
                    point,
                )
                .unwrap();
            pump(100);
        };
        click(50, 35);
        browser.input(Input::Text { text: "Привет".into() }).unwrap();
        pump(150);
        browser.input(Input::Key { key: "backspace".into() }).unwrap();
        pump(150);
        assert_eq!(browser.view.evaluate("document.getElementById('p').value").unwrap(), "\"Приве\"");
        // The local fixture reports its own input for this assertion; production pages have no bridge.
        click(60, 130);
        browser.input(Input::Text { text: "frame".into() }).unwrap();
        pump(150);
        assert_eq!(browser.view.evaluate("document.activeElement.tagName").unwrap(), "\"IFRAME\"");
        assert_eq!(browser.view.evaluate("window.frameValue").unwrap(), "\"frame\"");
        browser.input(Input::Key { key: "selectAll".into() }).unwrap();
        pump(100);
        browser.input(Input::Text { text: "replaced".into() }).unwrap();
        pump(100);
        assert_eq!(browser.view.evaluate("window.frameValue").unwrap(), "\"replaced\"");
        assert!(browser.view.take_page_messages().is_empty());
        browser.view.evaluate("document.cookie='browserTest=kept;path=/'").unwrap();
        browser.command(Command::Navigate { url: format!("http://127.0.0.1:{port}/second") }, &stage).unwrap();
        pump(500);
        assert_eq!(browser.view.evaluate("document.title").unwrap(), "\"Second\"");
        assert!(browser.view.evaluate("document.cookie").unwrap().contains("browserTest=kept"));
        browser.command(Command::Back, &stage).unwrap();
        pump(500);
        assert_eq!(browser.view.evaluate("document.title").unwrap(), "\"Fixture\"");
        browser.command(Command::Forward, &stage).unwrap();
        pump(500);
        assert_eq!(browser.view.evaluate("document.title").unwrap(), "\"Second\"");
        browser.command(Command::Navigate { url: "file:///C:/Windows/win.ini".into() }, &stage).unwrap();
        pump(200);
        assert_eq!(browser.view.evaluate("document.title").unwrap(), "\"Second\"");
        browser.command(Command::Zoom { factor: 1.2 }, &stage).unwrap();
        browser.command(Command::Hide, &stage).unwrap();
        assert!(browser.area.is_none());
    }
}
