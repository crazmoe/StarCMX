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
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::UI::Shell::{
        ABE_LEFT, ABE_RIGHT, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, APPBARDATA,
        SHAppBarMessage,
    };

    /// Breite der ausgefahrenen Leiste (logische Pixel)
    const WIDTH: f64 = 300.0;
    /// Breite des Streifens im eingefahrenen Zustand (physische Pixel)
    const STRIP: u32 = 6;

    /// Fenster ist als AppBar angemeldet
    static APPBAR: AtomicBool = AtomicBool::new(false);

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
        let Ok(Some(mon)) = w.current_monitor().or_else(|_| w.primary_monitor()) else {
            return;
        };
        let left = edge == "left";
        let width = (WIDTH * mon.scale_factor()).round() as u32;
        let width = if open { width } else { STRIP };
        let _ = w.set_decorations(false);
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
            if left {
                rc.right = rc.left + width as i32;
            } else {
                rc.left = rc.right - width as i32;
            }
        }
        let _ = w.set_size(PhysicalSize::new(
            (rc.right - rc.left) as u32,
            (rc.bottom - rc.top) as u32,
        ));
        let _ = w.set_position(PhysicalPosition::new(rc.left, rc.top));
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
