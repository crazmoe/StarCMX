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
//! Wechsel des Anrufzustands zählt die Mediensteuerung deshalb nicht.

use std::cell::RefCell;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use sf_headset::Button;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use windows::Foundation::TypedEventHandler;
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
const QUIET: Duration = Duration::from_millis(1500);

/// Wechsel des Anrufzustands festhalten
fn note_calls() {
    let calls: Calls = crate::instances::devices()
        .lines
        .iter()
        .map(|l| (l.id.clone(), l.ringing))
        .collect();
    let mut last = LAST_CHANGE.lock().unwrap_or_else(|e| e.into_inner());
    if last.as_ref().is_none_or(|(c, _)| *c != calls) {
        *last = Some((calls, Instant::now()));
    }
}

/// Liegt der letzte Wechsel des Anrufzustands erst kurz zurück?
fn just_changed() -> bool {
    note_calls();
    LAST_CHANGE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .is_some_and(|(_, at)| at.elapsed() < QUIET)
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
    note_calls();
    let mut current = ACTIVE.lock().unwrap_or_else(|e| e.into_inner());
    if *current == active {
        return;
    }
    tracing::debug!(active, "Medientasten");
    hotkeys(app, active);
    let handle = app.clone();
    let result = app.run_on_main_thread(move || {
        if let Err(e) = transport_controls(&handle, active) {
            tracing::warn!(error = %e, active, "Mediensteuerung nicht umgestellt");
        }
    });
    if let Err(e) = result {
        tracing::warn!(error = %e, "Mediensteuerung nicht erreicht");
    }
    *current = active;
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

fn press(app: &AppHandle, button: Button) {
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
