//! Bluetooth-Kopfhörer und -Headsets unter Windows: Ihre Taste schickt (über
//! AVRCP) Play/Pause. Windows gibt das nicht als Tastendruck weiter, sondern
//! an die Mediensteuerung des Systems (System Media Transport Controls), und
//! zwar an die gerade aktive Wiedergabe. Solange ein Anruf klingelt oder
//! läuft, meldet sich StarCMX deshalb dort als laufende Wiedergabe an (wie
//! unter Linux als MPRIS-Spieler); die Taste nimmt dann an oder legt auf.
//! Ohne Anruf bleibt sie beim Musikspieler.
//!
//! Medientasten der Tastatur fängt StarCMX während des Anrufs zusätzlich
//! systemweit ab (`RegisterHotKey`). Ob die Taste zählt, entscheidet wie
//! unter Linux die Einstellung `play_pause_call` (siehe `super::media_key`).
//!
//! Wechselt Windows beim Annehmen auf das Freisprechprofil (HFP), meldet es
//! von sich aus „Pause“, ohne dass jemand gedrückt hat. Kurz nach jedem
//! Wechsel des Anrufzustands zählt die Mediensteuerung deshalb nicht, außer
//! es klingelt.
//!
//! Im Gespräch schicken manche Headsets ihre Taste nicht mehr über AVRCP.
//! Was Windows daraus als App-Befehl (`WM_APPCOMMAND`) macht, sieht StarCMX
//! über einen Shell-Hook ([`shell_hook`]).

use std::cell::RefCell;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use sf_headset::Button;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
use windows::Media::{
    MediaPlaybackStatus, MediaPlaybackType, SystemMediaTransportControls,
    SystemMediaTransportControlsButton, SystemMediaTransportControlsButtonPressedEventArgs,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::WinRT::ISystemMediaTransportControlsInterop;
use windows::core::HSTRING;

/// Medientasten sind gerade angemeldet
static ACTIVE: Mutex<bool> = Mutex::new(false);

/// Anrufzustand (Gespräch, klingelt) beim letzten Wechsel, und wann
static LAST_CHANGE: Mutex<Option<(Calls, Instant)>> = Mutex::new(None);

/// Anrufe mit „klingelt“
type Calls = Vec<(String, bool)>;

/// So lange nach einem Wechsel zählt die Mediensteuerung nicht
const QUIET: Duration = Duration::from_secs(3);

/// Wechsel des Anrufzustands festhalten; true, wenn er sich geändert hat
fn note_calls() -> bool {
    let calls: Calls = crate::instances::devices()
        .lines
        .iter()
        .map(|l| (l.id.clone(), l.ringing))
        .collect();
    let mut last = LAST_CHANGE.lock().unwrap_or_else(|e| e.into_inner());
    let changed = last.as_ref().is_none_or(|(c, _)| *c != calls);
    if changed {
        *last = Some((calls, Instant::now()));
    }
    changed
}

/// Liegt der letzte Wechsel des Anrufzustands erst kurz zurück? Klingelt
/// es, zählt die Taste immer: Annehmen soll gleich gehen.
fn just_changed() -> bool {
    note_calls();
    let last = LAST_CHANGE.lock().unwrap_or_else(|e| e.into_inner());
    last.as_ref().is_some_and(|(calls, at)| {
        !calls.iter().any(|(_, ringing)| *ringing) && at.elapsed() < QUIET
    })
}

const KEYS: [(&str, Button); 2] = [
    ("MediaPlayPause", Button::HookToggle),
    ("MediaStop", Button::HookOn),
];

thread_local! {
    /// Mediensteuerung des Hauptfensters; nur im Hauptthread benutzt
    static SMTC: RefCell<Option<SystemMediaTransportControls>> = const { RefCell::new(None) };
}

/// Medientasten an- (Anruf) oder abmelden (kein Anruf)
pub fn set(app: &AppHandle, active: bool) {
    let changed = note_calls();
    let mut current = ACTIVE.lock().unwrap_or_else(|e| e.into_inner());
    if *current == active {
        // Nach dem Annehmen gibt Windows die Tasten sonst einem anderen
        // Spieler; StarCMX meldet sich deshalb bei jedem Wechsel neu.
        if active && changed {
            update(app, active);
        }
        return;
    }
    tracing::debug!(active, "Medientasten");
    shell_hook::start(app);
    hotkeys(app, active);
    update(app, active);
    *current = active;
}

fn update(app: &AppHandle, active: bool) {
    let handle = app.clone();
    let result = app.run_on_main_thread(move || {
        if let Err(e) = transport_controls(&handle, active) {
            tracing::warn!(error = %e, active, "Mediensteuerung nicht umgestellt");
        }
    });
    if let Err(e) = result {
        tracing::warn!(error = %e, "Mediensteuerung nicht erreicht");
    }
    if active {
        std::thread::spawn(log_current_session);
    }
}

/// Welcher Spieler bekommt gerade die Medientasten? Nur zur Diagnose.
fn log_current_session() {
    let app = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .and_then(|op| op.join())
        .and_then(|manager| manager.GetCurrentSession())
        .and_then(|session| session.SourceAppUserModelId());
    match app {
        Ok(app) => tracing::debug!(%app, "Mediensteuerung: aktueller Spieler"),
        Err(e) => tracing::debug!(error = %e, "Mediensteuerung: kein aktueller Spieler"),
    }
}

fn hotkeys(app: &AppHandle, active: bool) {
    let gs = app.global_shortcut();
    for (key, button) in KEYS {
        let Ok(shortcut) = key.parse::<Shortcut>() else {
            continue;
        };
        let result = if active {
            gs.on_shortcut(shortcut, move |app, _, event| {
                if event.state == ShortcutState::Pressed {
                    tracing::debug!(key, "Medientaste");
                    press(app, button);
                }
            })
        } else {
            gs.unregister(shortcut)
        };
        if let Err(e) = result {
            // Etwa schon von einem anderen Programm belegt
            tracing::warn!(error = %e, key, active, "Medientaste nicht umgemeldet");
        }
    }
}

/// Letzter weitergegebener Tastendruck
static LAST_PRESS: Mutex<Option<Instant>> = Mutex::new(None);

/// Kommt derselbe Druck auf zwei Wegen an, zählt er nur einmal
const DOUBLE: Duration = Duration::from_millis(800);

fn press(app: &AppHandle, button: Button) {
    {
        let mut last = LAST_PRESS.lock().unwrap_or_else(|e| e.into_inner());
        if last.is_some_and(|at| at.elapsed() < DOUBLE) {
            tracing::debug!(?button, "Tastendruck doppelt, übergangen");
            return;
        }
        *last = Some(Instant::now());
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move { super::media_key(&app, button).await });
}

/// Als laufende Wiedergabe an- bzw. abmelden. Im Hauptthread.
fn transport_controls(app: &AppHandle, active: bool) -> windows::core::Result<()> {
    SMTC.with(|cell| {
        let mut cell = cell.borrow_mut();
        if cell.is_none() {
            if !active {
                return Ok(());
            }
            *cell = Some(create(app)?);
        }
        let smtc = cell.as_ref().expect("eben angelegt");
        if active {
            smtc.SetIsEnabled(true)?;
            smtc.SetPlaybackStatus(MediaPlaybackStatus::Playing)?;
            smtc.DisplayUpdater()?.Update()?;
        } else {
            smtc.SetPlaybackStatus(MediaPlaybackStatus::Closed)?;
            smtc.SetIsEnabled(false)?;
        }
        Ok(())
    })
}

fn create(app: &AppHandle) -> windows::core::Result<SystemMediaTransportControls> {
    let window = app.get_webview_window("main").ok_or_else(|| {
        windows::core::Error::new(windows::core::HRESULT(-1), "Hauptfenster fehlt")
    })?;
    let hwnd = window
        .hwnd()
        .map_err(|e| windows::core::Error::new(windows::core::HRESULT(-1), e.to_string()))?;
    let interop = windows::core::factory::<
        SystemMediaTransportControls,
        ISystemMediaTransportControlsInterop,
    >()?;
    let smtc: SystemMediaTransportControls = unsafe { interop.GetForWindow(HWND(hwnd.0 as _))? };
    smtc.SetIsPlayEnabled(true)?;
    smtc.SetIsPauseEnabled(true)?;
    smtc.SetIsStopEnabled(true)?;
    let display = smtc.DisplayUpdater()?;
    display.SetType(MediaPlaybackType::Music)?;
    let music = display.MusicProperties()?;
    music.SetTitle(&HSTRING::from(crate::i18n::t("Anruf")))?;
    music.SetArtist(&HSTRING::from("StarCMX"))?;
    display.Update()?;
    let app = app.clone();
    smtc.ButtonPressed(&TypedEventHandler::<
        SystemMediaTransportControls,
        SystemMediaTransportControlsButtonPressedEventArgs,
    >::new(move |_, args| {
        let Some(args) = args.as_ref() else {
            return Ok(());
        };
        let pressed = args.Button()?;
        tracing::debug!(button = pressed.0, "Mediensteuerung");
        let button = match pressed {
            SystemMediaTransportControlsButton::Play
            | SystemMediaTransportControlsButton::Pause => Button::HookToggle,
            SystemMediaTransportControlsButton::Stop => Button::HookOn,
            _ => return Ok(()),
        };
        if just_changed() {
            tracing::debug!(
                button = pressed.0,
                "Mediensteuerung kurz nach Anrufwechsel übergangen"
            );
            return Ok(());
        }
        press(&app, button);
        Ok(())
    }))?;
    Ok(smtc)
}

/// App-Befehle aller Fenster über einen Shell-Hook mitlesen
mod shell_hook {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering};

    use sf_headset::Button;
    use tauri::AppHandle;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, HSHELL_APPCOMMAND, MSG,
        RegisterClassW, RegisterShellHookWindow, RegisterWindowMessageW, WINDOW_EX_STYLE,
        WINDOW_STYLE, WNDCLASSW,
    };
    use windows::core::w;

    static APP: OnceLock<AppHandle> = OnceLock::new();
    /// Nachrichtennummer der Shell-Hook-Meldungen
    static SHELLHOOK: AtomicU32 = AtomicU32::new(0);

    const MEDIA_STOP: u32 = 13;
    const MEDIA_PLAY_PAUSE: u32 = 14;
    const MEDIA_PLAY: u32 = 46;
    const MEDIA_PAUSE: u32 = 47;

    /// Einmal starten, eigener Thread mit unsichtbarem Fenster
    pub fn start(app: &AppHandle) {
        if APP.set(app.clone()).is_err() {
            return;
        }
        std::thread::spawn(|| {
            if let Err(e) = run() {
                tracing::warn!(error = %e, "Shell-Hook nicht angemeldet");
            }
        });
    }

    fn run() -> windows::core::Result<()> {
        unsafe {
            let instance = GetModuleHandleW(None)?.into();
            let class = w!("StarCMXShellHook");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                hInstance: instance,
                lpszClassName: class,
                ..Default::default()
            };
            RegisterClassW(&wc);
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class,
                w!(""),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                None,
                None,
                Some(instance),
                None,
            )?;
            SHELLHOOK.store(RegisterWindowMessageW(w!("SHELLHOOK")), Ordering::Relaxed);
            RegisterShellHookWindow(hwnd).ok()?;
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }

    extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let hook = SHELLHOOK.load(Ordering::Relaxed);
        if hook != 0 && msg == hook && wparam.0 as u32 == HSHELL_APPCOMMAND {
            // GET_APPCOMMAND_LPARAM: oberes Wort ohne Geräte-Bits
            let cmd = ((lparam.0 as u32) >> 16) & 0x0FFF;
            command(cmd);
        }
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    fn command(cmd: u32) {
        let active = *super::ACTIVE.lock().unwrap_or_else(|e| e.into_inner());
        tracing::debug!(cmd, active, "App-Befehl");
        if !active {
            return;
        }
        let button = match cmd {
            MEDIA_PLAY_PAUSE | MEDIA_PLAY | MEDIA_PAUSE => Button::HookToggle,
            MEDIA_STOP => Button::HookOn,
            _ => return,
        };
        if super::just_changed() {
            tracing::debug!(cmd, "App-Befehl kurz nach Anrufwechsel übergangen");
            return;
        }
        if let Some(app) = APP.get() {
            super::press(app, button);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_parse() {
        for (key, _) in KEYS {
            assert!(key.parse::<Shortcut>().is_ok(), "{key}");
        }
    }
}
