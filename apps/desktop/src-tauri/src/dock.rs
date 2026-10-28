//! Schnellwahl am Bildschirmrand andocken (nur Windows).
//!
//! Angedockt ist das Schnellwahl-Fenster randlos, immer im Vordergrund und
//! so hoch wie der Arbeitsbereich. „Immer sichtbar“ meldet es bei Windows
//! als AppBar an, damit andere Fenster wie bei der Taskleiste Platz machen.
//! „Einfahren“ lässt nur einen schmalen Streifen am Rand stehen; berührt
//! die Maus ihn, fährt die Oberfläche das Fenster über `quick_dock_expand`
//! wieder aus.

use crate::settings::Prefs;
use std::sync::Mutex;
use tauri::AppHandle;

/// Gewählter Rand ("left"/"right", leer = nicht angedockt) und Einfahren
static STATE: Mutex<(String, bool)> = Mutex::new((String::new(), false));

/// Gewählter Bildschirm (Name wie von Tauri gemeldet; leer = Hauptbildschirm)
static MONITOR: Mutex<String> = Mutex::new(String::new());

/// Bildschirm zum Andocken: der gewählte, sonst der Hauptbildschirm
fn pick_monitor(w: &tauri::WebviewWindow) -> Option<tauri::Monitor> {
    let name = MONITOR.lock().unwrap().clone();
    if !name.is_empty()
        && let Some(m) = w
            .available_monitors()
            .unwrap_or_default()
            .into_iter()
            .find(|m| m.name().is_some_and(|n| *n == name))
    {
        return Some(m);
    }
    w.primary_monitor()
        .ok()
        .flatten()
        .or_else(|| w.current_monitor().ok().flatten())
}

/// Angedockt?
pub fn docked() -> bool {
    !STATE.lock().unwrap().0.is_empty()
}

/// Einstellungen übernehmen (beim Start und nach dem Speichern)
pub fn apply(app: &AppHandle, prefs: &Prefs) {
    let edge = match prefs.quick_dock.as_str() {
        "left" | "right" => prefs.quick_dock.clone(),
        _ => String::new(),
    };
    let next = (edge, prefs.quick_dock_autohide);
    *MONITOR.lock().unwrap() = prefs.quick_dock_monitor.clone();
    let changed = {
        let mut s = STATE.lock().unwrap();
        let changed = *s != next;
        *s = next;
        changed
    };
    if docked() {
        crate::show_quick_window(app);
    } else if changed {
        imp::undock(app);
    }
}

/// Nach dem Anlegen oder Zeigen des Schnellwahl-Fensters: an den Rand legen
pub fn place(app: &AppHandle) {
    let (edge, autohide) = STATE.lock().unwrap().clone();
    if !edge.is_empty() {
        imp::dock(app, &edge, autohide, !autohide);
    }
}

/// Eingefahrene Leiste aus- oder wieder einfahren
#[tauri::command]
pub fn quick_dock_expand(app: AppHandle, open: bool) {
    let (edge, autohide) = STATE.lock().unwrap().clone();
    if !edge.is_empty() && autohide {
        imp::dock(&app, &edge, true, open);
    }
}

#[cfg(windows)]
mod imp {
    use std::sync::atomic::{AtomicBool, Ordering};
    use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};
    use windows::Win32::Foundation::{HWND, POINT, RECT};
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    use windows::Win32::UI::Shell::{
        ABE_LEFT, ABE_RIGHT, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, APPBARDATA,
        SHAppBarMessage,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    /// Breite der ausgefahrenen Leiste (logische Pixel)
    const WIDTH: f64 = 300.0;
    /// Breite des Streifens im eingefahrenen Zustand (physische Pixel)
    const STRIP: u32 = 6;

    /// Fenster ist als AppBar angemeldet
    static APPBAR: AtomicBool = AtomicBool::new(false);
    /// Lage der Leiste zuletzt (links, oben, rechts, unten) und ob ausgefahren
    static LAST: std::sync::Mutex<(RECT, bool)> = std::sync::Mutex::new((
        RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        false,
    ));
    static WATCHING: AtomicBool = AtomicBool::new(false);

    /// Mausposition am Rand selbst prüfen: Die Webansicht im schmalen
    /// Streifen meldet das Überfahren nicht zuverlässig.
    fn watch(app: &AppHandle) {
        if WATCHING.swap(true, Ordering::SeqCst) {
            return;
        }
        let app = app.clone();
        std::thread::spawn(move || {
            let mut outside_since: Option<std::time::Instant> = None;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(120));
                let (edge, autohide) = super::STATE.lock().unwrap().clone();
                if edge.is_empty() || !autohide {
                    outside_since = None;
                    continue;
                }
                let mut pt = POINT::default();
                if unsafe { GetCursorPos(&mut pt) }.is_err() {
                    continue;
                }
                let (rc, open) = *LAST.lock().unwrap();
                let inside =
                    pt.x >= rc.left && pt.x < rc.right && pt.y >= rc.top && pt.y < rc.bottom;
                if !open {
                    if inside {
                        dock(&app, &edge, true, true);
                    }
                    continue;
                }
                // Nicht einfahren, solange die Maustaste gedrückt ist (Gespräch ziehen)
                let pressed = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } as u16 & 0x8000 != 0;
                if inside || pressed {
                    outside_since = None;
                } else if outside_since
                    .get_or_insert_with(std::time::Instant::now)
                    .elapsed()
                    > std::time::Duration::from_millis(700)
                {
                    outside_since = None;
                    dock(&app, &edge, true, false);
                }
            }
        });
    }

    fn appbar(hwnd: HWND, edge: u32, rc: RECT, msg: u32) -> RECT {
        let mut abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            uEdge: edge,
            rc,
            ..Default::default()
        };
        unsafe { SHAppBarMessage(msg, &mut abd) };
        abd.rc
    }

    fn remove_appbar(hwnd: HWND) {
        if APPBAR.swap(false, Ordering::SeqCst) {
            appbar(hwnd, 0, RECT::default(), ABM_REMOVE);
        }
    }

    /// `reserve`: als AppBar Platz freihalten; `open`: ausgefahren
    pub fn dock(app: &AppHandle, edge: &str, autohide: bool, open: bool) {
        let Some(w) = app.get_webview_window("quick") else {
            return;
        };
        let Ok(raw) = w.hwnd() else {
            return;
        };
        let hwnd = HWND(raw.0 as _);
        let Some(mon) = super::pick_monitor(&w) else {
            return;
        };
        let left = edge == "left";
        let width = (WIDTH * mon.scale_factor()).round() as u32;
        let width = if open { width } else { STRIP };
        let _ = w.set_decorations(false);
        // Ohne Schatten und runde Ecken liegt die Leiste bündig am Rand.
        let _ = w.set_shadow(false);
        let _ = w.set_skip_taskbar(true);
        let _ = w.set_always_on_top(true);
        let _ = w.set_resizable(false);
        let _ = w.set_min_size(None::<PhysicalSize<u32>>);

        let area = mon.work_area();
        let mut rc = RECT {
            left: area.position.x,
            top: area.position.y,
            right: area.position.x + area.size.width as i32,
            bottom: area.position.y + area.size.height as i32,
        };
        if !autohide {
            // Als AppBar anmelden; Windows rückt den Arbeitsbereich zurecht.
            let ab_edge = if left { ABE_LEFT } else { ABE_RIGHT };
            if !APPBAR.swap(true, Ordering::SeqCst) {
                appbar(hwnd, ab_edge, rc, ABM_NEW);
            }
            let m = mon.position();
            let s = mon.size();
            rc = RECT {
                left: m.x,
                top: m.y,
                right: m.x + s.width as i32,
                bottom: m.y + s.height as i32,
            };
            rc = appbar(hwnd, ab_edge, rc, ABM_QUERYPOS);
            if left {
                rc.right = rc.left + width as i32;
            } else {
                rc.left = rc.right - width as i32;
            }
            rc = appbar(hwnd, ab_edge, rc, ABM_SETPOS);
        } else {
            remove_appbar(hwnd);
            // Waagrecht vom Bildschirmrand aus: Der Arbeitsbereich enthält
            // den eben freigegebenen AppBar-Platz unter Umständen noch.
            let m = mon.position();
            let s = mon.size();
            if left {
                rc.left = m.x;
                rc.right = m.x + width as i32;
            } else {
                rc.right = m.x + s.width as i32;
                rc.left = rc.right - width as i32;
            }
        }
        let _ = w.set_size(PhysicalSize::new(
            (rc.right - rc.left) as u32,
            (rc.bottom - rc.top) as u32,
        ));
        let _ = w.set_position(PhysicalPosition::new(rc.left, rc.top));
        *LAST.lock().unwrap() = (rc, open);
        if autohide {
            watch(app);
        }
        let _ = w.show();
    }

    pub fn undock(app: &AppHandle) {
        let Some(w) = app.get_webview_window("quick") else {
            return;
        };
        if let Ok(raw) = w.hwnd() {
            remove_appbar(HWND(raw.0 as _));
        }
        let _ = w.set_always_on_top(false);
        let _ = w.set_skip_taskbar(false);
        let _ = w.set_decorations(true);
        let _ = w.set_shadow(true);
        let _ = w.set_resizable(true);
        let _ = w.set_min_size(Some(tauri::LogicalSize::new(260.0, 240.0)));
        let _ = w.set_size(tauri::LogicalSize::new(340.0, 480.0));
        let _ = w.center();
        let _ = w.hide();
    }

    /// Beim Beenden den reservierten Platz freigeben
    pub fn release(app: &AppHandle) {
        if let Some(Ok(raw)) = app.get_webview_window("quick").map(|w| w.hwnd()) {
            remove_appbar(HWND(raw.0 as _));
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use tauri::AppHandle;
    pub fn dock(_: &AppHandle, _: &str, _: bool, _: bool) {}
    pub fn undock(_: &AppHandle) {}
    pub fn release(_: &AppHandle) {}
}

/// Beim Beenden aufrufen
pub fn release(app: &AppHandle) {
    imp::release(app);
}
