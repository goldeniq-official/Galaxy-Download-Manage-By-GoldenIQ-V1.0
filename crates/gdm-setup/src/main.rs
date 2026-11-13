// SPDX-License-Identifier: RPL-1.5
//! GDM Setup: the window people see when they install GDM. The Tauri NSIS installer inside does the install
//! (`/S`: per user, the previous folder on an upgrade, the browser bridge registered by its hook), so the
//! updater, the uninstaller and WebView2's bootstrapper stay exactly what they were.
//! The page (ui/) is GDM's own design: the app's tokens and fonts, served from this exe, nothing remote.
//! Without WebView2 (very old Windows 10) Setup runs the NSIS installer with its own wizard, which installs it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    eprintln!("GDM Setup is the Windows installer; other systems use their own packages");
}

#[cfg(windows)]
fn main() {
    win::main()
}

#[cfg(windows)]
mod win {
    use std::borrow::Cow;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    use gdm_bridge::browsers::{browsers, store_key, store_url};
    use serde_json::{json, Value};
    use tao::dpi::LogicalSize;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
    use tao::platform::windows::{IconExtWindows, WindowBuilderExtWindows, WindowExtWindows};
    use tao::window::{Icon, WindowBuilder};
    use windows::core::{w, HSTRING, PCWSTR};
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HWND};
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DWM_WINDOW_CORNER_PREFERENCE};
    use windows::Win32::System::LibraryLoader::{SetDefaultDllDirectories, LOAD_LIBRARY_SEARCH_SYSTEM32};
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ};
    use windows::Win32::System::Threading::CreateMutexW;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, SW_SHOWNORMAL};
    use wry::http::{header, Request, Response};
    use wry::{NewWindowResponse, WebContext, WebViewBuilder, WebViewBuilderExtWindows};

    static PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.exe"));
    const ORIGIN: &str = "http://gdm.localhost/";
    const PRODUCT_KEY: PCWSTR = w!(r"Software\GoldenIQ\Galaxy Download Manager"); // the NSIS template's MANUPRODUCTKEY
    const UNINSTALL_KEY: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Galaxy Download Manager");

    /// served files: (path, content type, bytes)
    const FILES: &[(&str, &str, &[u8])] = &[
        ("", "text/html; charset=utf-8", include_bytes!("../ui/index.html")),
        ("setup.css", "text/css; charset=utf-8", include_bytes!("../ui/setup.css")),
        ("setup.js", "text/javascript; charset=utf-8", include_bytes!("../ui/setup.js")),
        ("logo.png", "image/png", include_bytes!("../../../public/gdm-icon-256.png")),
        ("tokens.css", "text/css; charset=utf-8", include_bytes!("../../../src/styles/tokens.css")),
        ("fonts/outfit-latin-400-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/outfit-latin-400-normal.woff2")),
        ("fonts/outfit-latin-500-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/outfit-latin-500-normal.woff2")),
        ("fonts/outfit-latin-600-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/outfit-latin-600-normal.woff2")),
        ("fonts/outfit-latin-700-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/outfit-latin-700-normal.woff2")),
        ("fonts/outfit-latin-800-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/outfit-latin-800-normal.woff2")),
        ("fonts/kantumruy-pro-khmer-400-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/kantumruy-pro-khmer-400-normal.woff2")),
        ("fonts/kantumruy-pro-khmer-500-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/kantumruy-pro-khmer-500-normal.woff2")),
        ("fonts/kantumruy-pro-khmer-600-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/kantumruy-pro-khmer-600-normal.woff2")),
        ("fonts/kantumruy-pro-khmer-700-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/kantumruy-pro-khmer-700-normal.woff2")),
        ("fonts/jetbrains-mono-latin-400-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/jetbrains-mono-latin-400-normal.woff2")),
        ("fonts/jetbrains-mono-latin-500-normal.woff2", "font/woff2", include_bytes!("../../../public/fonts/jetbrains-mono-latin-500-normal.woff2")),
    ];
    const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'none'; base-uri 'none'; form-action 'none'";

    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Ready,
        Running,
        Done,
        Failed,
    }

    enum Ev {
        Ipc(String),
        Installed(Result<PathBuf, String>),
    }

    pub fn main() {
        unsafe {
            // run from Downloads: never load a DLL planted beside Setup
            let _ = SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32);
            // one Setup at a time (two NSIS runs into one folder would fight)
            let _m = CreateMutexW(None, false, w!("Local\\GoldenIQ.GDM.Setup"));
            if GetLastError() == ERROR_ALREADY_EXISTS {
                return;
            }
            std::mem::forget(_m);
        }

        let event_loop = EventLoopBuilder::<Ev>::with_user_event().build();
        let window = WindowBuilder::new()
            .with_title("Galaxy Download Manager Setup")
            .with_inner_size(LogicalSize::new(760.0, 480.0))
            .with_resizable(false)
            .with_maximizable(false)
            .with_decorations(false)
            .with_undecorated_shadow(true)
            .with_visible(false)
            .with_window_icon(Icon::from_resource(1, None).ok())
            .with_taskbar_icon(Icon::from_resource(1, None).ok())
            .build(&event_loop)
            .expect("window");
        unsafe {
            let pref: DWM_WINDOW_CORNER_PREFERENCE = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                HWND(window.hwnd() as _),
                DWMWA_WINDOW_CORNER_PREFERENCE,
                (&pref as *const DWM_WINDOW_CORNER_PREFERENCE).cast(),
                4,
            );
        }

        // WebView2's profile in TEMP, never beside the exe (Downloads, read-only media)
        let mut ctx = WebContext::new(Some(std::env::temp_dir().join("GDM-Setup").join("webview")));
        let proxy = event_loop.create_proxy();
        let ipc = proxy.clone();
        let webview = WebViewBuilder::new_with_web_context(&mut ctx)
            .with_custom_protocol("gdm".into(), |_, req| serve(&req))
            .with_url(ORIGIN)
            .with_background_color((0x15, 0x17, 0x1c, 255))
            .with_ipc_handler(move |req: Request<String>| {
                if req.uri().to_string().starts_with(ORIGIN) && req.body().len() < 4096 {
                    let _ = ipc.send_event(Ev::Ipc(req.body().clone()));
                }
            })
            .with_navigation_handler(|url| url.starts_with(ORIGIN))
            .with_new_window_req_handler(|_, _| NewWindowResponse::Deny)
            .with_devtools(cfg!(debug_assertions))
            .with_hotkeys_zoom(false)
            .with_clipboard(false)
            .with_browser_accelerator_keys(false)
            .with_default_context_menus(false)
            .build(&window);
        let webview = match webview {
            Ok(v) => v,
            Err(_) => return classic_wizard(),
        };

        let mut state = State::Ready;
        let mut app: Option<PathBuf> = None;
        event_loop.run(move |event, _, flow| {
            *flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } if state != State::Running => *flow = ControlFlow::Exit,
                Event::UserEvent(Ev::Installed(r)) => {
                    let reply = match r {
                        Ok(exe) => {
                            app = Some(exe);
                            state = State::Done;
                            json!({ "t": "done" })
                        }
                        Err(e) => {
                            state = State::Failed;
                            json!({ "t": "failed", "error": e })
                        }
                    };
                    let _ = webview.evaluate_script(&format!("gdm({reply})"));
                }
                Event::UserEvent(Ev::Ipc(body)) => {
                    let m: Value = serde_json::from_str(&body).unwrap_or_default();
                    match (m["c"].as_str().unwrap_or(""), state) {
                        ("ready", _) => {
                            let list: Vec<Value> =
                                browsers().into_iter().filter(|b| b.2).map(|(key, name, _)| json!({ "key": key, "name": name })).collect();
                            let hello = json!({
                                "t": "hello",
                                "version": env!("CARGO_PKG_VERSION"),
                                "lang": "km", // Khmer first, as the app starts (the page switches)
                                "dir": install_dir().unwrap_or_else(default_dir),
                                "installed": install_dir().and_then(|_| reg_sz(UNINSTALL_KEY, w!("DisplayVersion"))),
                                "browsers": list,
                                "payload": !PAYLOAD.is_empty(),
                            });
                            let _ = webview.evaluate_script(&format!("gdm({hello})"));
                            window.set_visible(true);
                        }
                        ("install", State::Ready | State::Failed) if !PAYLOAD.is_empty() => {
                            state = State::Running;
                            let p: EventLoopProxy<Ev> = proxy.clone();
                            std::thread::spawn(move || {
                                let _ = p.send_event(Ev::Installed(install()));
                            });
                        }
                        ("store", _) => {
                            // only a browser this PC has, only its fixed store page
                            let b = m["b"].as_str().unwrap_or("");
                            if let Some(url) = browsers().into_iter().find(|x| x.0 == b && x.2).and_then(|x| store_url(store_key(x.0))) {
                                open(&url);
                            }
                        }
                        ("link", _) => match m["k"].as_str() {
                            Some("license") => open("https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/blob/main/LICENSE"),
                            Some("privacy") => {
                                open("https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/blob/main/PRIVACY.md")
                            }
                            Some("help") => open("https://github.com/goldeniq-official/Galaxy-Download-Manage-By-GoldenIQ-V1.0/issues"),
                            _ => {}
                        },
                        ("launch", State::Done) => {
                            if let Some(exe) = &app {
                                let _ = Command::new(exe).current_dir(exe.parent().unwrap_or(exe)).spawn();
                            }
                            *flow = ControlFlow::Exit;
                        }
                        ("min", _) => window.set_minimized(true),
                        ("drag", _) => {
                            let _ = window.drag_window();
                        }
                        ("close", s) if s != State::Running => *flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
                _ => {}
            }
        });
    }

    fn serve(req: &Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
        let path = req.uri().path().trim_start_matches('/');
        match FILES.iter().find(|f| f.0 == path) {
            Some((_, kind, body)) => {
                Response::builder().header(header::CONTENT_TYPE, *kind).header("Content-Security-Policy", CSP).body(Cow::Borrowed(*body)).unwrap()
            }
            None => Response::builder().status(404).body(Cow::Borrowed(&b""[..])).unwrap(),
        }
    }

    /// The NSIS installer into a fresh private folder of this user's TEMP, run silently, then checked.
    fn install() -> Result<PathBuf, String> {
        let code = run_payload(&["/S"])?.code();
        if code != Some(0) {
            return Err(format!("installer exit code {}", code.map_or("none".into(), |c| c.to_string())));
        }
        let app = PathBuf::from(install_dir().ok_or("GDM's folder is not registered")?).join("gdm.exe");
        if !app.is_file() {
            return Err(format!("{} is missing", app.display()));
        }
        Ok(app)
    }

    /// no WebView2: the NSIS wizard installs it (Tauri's bootstrapper) and then GDM
    /// (the wizard reports its own errors and the user's cancel; Setup only reports a wizard that never started)
    fn classic_wizard() {
        if PAYLOAD.is_empty() || run_payload(&[]).is_err() {
            unsafe {
                MessageBoxW(None, w!("GDM Setup could not start. Download it again from the GDM releases page."), w!("GDM Setup"), MB_ICONERROR)
            };
        }
    }

    /// The NSIS installer written into a new folder of this user's TEMP (create_dir fails on anything already
    /// there, so a planted folder is never used, and only a folder this run made is removed), then run.
    fn run_payload(args: &[&str]) -> Result<std::process::ExitStatus, String> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("GDM-Setup-{}-{nanos}", std::process::id()));
        std::fs::create_dir(&dir).map_err(|e| format!("temp: {e}"))?;
        let exe = dir.join("gdm-installer.exe");
        let run = (|| {
            let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(&exe).map_err(|e| format!("temp: {e}"))?;
            f.write_all(PAYLOAD).and_then(|_| f.sync_all()).map_err(|e| format!("temp: {e}"))?;
            drop(f);
            Command::new(&exe).args(args).current_dir(&dir).status().map_err(|e| format!("installer: {e}"))
        })();
        let _ = std::fs::remove_dir_all(&dir);
        run
    }

    fn install_dir() -> Option<String> {
        reg_sz(PRODUCT_KEY, PCWSTR::null())
    }

    fn reg_sz(key: PCWSTR, value: PCWSTR) -> Option<String> {
        let mut buf = [0u16; 1024];
        let mut len = (buf.len() * 2) as u32;
        let r = unsafe { RegGetValueW(HKEY_CURRENT_USER, key, value, RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr().cast()), Some(&mut len)) };
        if r.is_err() {
            return None;
        }
        let s = String::from_utf16_lossy(&buf[..(len as usize / 2).saturating_sub(1)]);
        (!s.is_empty()).then_some(s)
    }

    fn default_dir() -> String {
        let base = std::env::var("LOCALAPPDATA").unwrap_or_default();
        format!(r"{base}\Galaxy Download Manager")
    }

    fn open(url: &str) {
        unsafe { ShellExecuteW(None, w!("open"), &HSTRING::from(url), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
    }
}
