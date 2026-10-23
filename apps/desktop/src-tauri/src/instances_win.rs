//! Instanzen unter Windows: vorerst genau eine.
//!
//! Ein weiterer Start (Link, Tastenkürzel, Startmenü) landet über
//! tauri-plugin-single-instance bei der laufenden App. Mehrere Anlagen
//! gleichzeitig wie unter Linux (`--profile`) gibt es hier noch nicht; die
//! Schnittstelle ist dieselbe wie in `instances.rs`, damit der Rest der App
//! nicht unterscheiden muss.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use sf_core::phone::{CallPhase, CallView};
use tauri::{AppHandle, Manager};

use crate::bus::{self, Event};
use crate::i18n::t;

/// Mehrere Anlagen gleichzeitig („Weitere Anlage öffnen“)
pub const MULTI: bool = false;

/// Angemeldete Anlage (Rechnername) und Benutzer, für Titel und Tray
static LABEL: Mutex<Option<(String, String)>> = Mutex::new(None);
/// Eigener Anrufzustand für Headset und Busylight
static REPORTS: Mutex<Reports> = Mutex::new(Reports {
    own: Report {
        ready: false,
        muted: false,
        lines: Vec::new(),
    },
    ring_order: None,
    next: 0,
});

/// Profil dieser Instanz; unter Windows immer das Standardprofil.
pub fn profile() -> &'static str {
    ""
}

/// Die einzige Instanz ist der Hauptclient.
pub fn is_main() -> bool {
    true
}

pub fn others() -> usize {
    0
}

pub fn dir(base: PathBuf) -> PathBuf {
    base
}

pub fn webview<'a, R: tauri::Runtime, M: Manager<R>>(
    _app: &AppHandle,
    builder: tauri::WebviewWindowBuilder<'a, R, M>,
) -> tauri::WebviewWindowBuilder<'a, R, M> {
    builder
}

pub fn icon(app: &AppHandle) -> Option<tauri::image::Image<'static>> {
    app.default_window_icon().map(|i| i.clone().to_owned())
}

pub fn tray_id() -> String {
    "main".to_owned()
}

pub fn set_icon(_app: &AppHandle, _window: &tauri::WebviewWindow) {}

/// Vor dem Start von Tauri; das Weiterreichen an eine laufende Instanz
/// erledigt tauri-plugin-single-instance.
pub fn init() -> bool {
    true
}

/// Nach dem Start der Oberfläche: eigene Anrufe für Headset und Busylight
/// verfolgen.
pub fn ready(app: &AppHandle) {
    bus::listen(app, "instances", |app, event| {
        let mut reports = REPORTS.lock().unwrap();
        let report = match event {
            Event::PhoneStopped => Report::default(),
            Event::PhoneReady => Report {
                ready: true,
                ..reports.own.clone()
            },
            Event::Calls { calls, muted } => Report {
                ready: true,
                muted,
                lines: calls.iter().map(ReportLine::from).collect(),
            },
            _ => return,
        };
        if reports.own == report {
            return;
        }
        reports.set(report);
        drop(reports);
        bus::publish(app, Event::Devices);
    });
    apply_titles(app);
}

/// Anrufzustand, wie ihn Headset und Busylight brauchen
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// Softphone bzw. Anrufsteuerung läuft
    pub ready: bool,
    pub muted: bool,
    pub lines: Vec<ReportLine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportLine {
    pub id: String,
    /// Klingelt bei uns
    pub ringing: bool,
    pub held: bool,
}

impl From<&CallView> for ReportLine {
    fn from(c: &CallView) -> Self {
        Self {
            id: c.id.clone(),
            ringing: c.incoming && c.phase == CallPhase::Ringing,
            held: c.phase == CallPhase::Held,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceLine {
    pub profile: String,
    pub id: String,
    pub ringing: bool,
    pub held: bool,
}

/// Was Headset und Busylight anzeigen
#[derive(Debug, Default)]
pub struct Devices {
    pub ready: bool,
    /// Klingelnde Anrufe in der Reihenfolge, in der sie kamen, davor die übrigen
    pub lines: Vec<DeviceLine>,
    /// Stummschaltung je Profil
    pub muted: BTreeMap<String, bool>,
}

struct Reports {
    own: Report,
    /// Seit wann (Zähler) ein Anruf klingelt
    ring_order: Option<HashMap<String, u64>>,
    next: u64,
}

impl Reports {
    fn set(&mut self, report: Report) {
        let order = self.ring_order.get_or_insert_with(HashMap::new);
        order.retain(|id, _| report.lines.iter().any(|l| l.ringing && l.id == *id));
        for l in report.lines.iter().filter(|l| l.ringing) {
            order.entry(l.id.clone()).or_insert_with(|| {
                self.next += 1;
                self.next
            });
        }
        self.own = report;
    }

    fn devices(&self) -> Devices {
        let order = |id: &str| {
            self.ring_order
                .as_ref()
                .and_then(|o| o.get(id))
                .copied()
                .unwrap_or(u64::MAX)
        };
        let mut lines: Vec<DeviceLine> = self
            .own
            .lines
            .iter()
            .map(|l| DeviceLine {
                profile: String::new(),
                id: l.id.clone(),
                ringing: l.ringing,
                held: l.held,
            })
            .collect();
        lines.sort_by_key(|l| if l.ringing { (1, order(&l.id)) } else { (0, 0) });
        Devices {
            ready: self.own.ready,
            lines,
            muted: BTreeMap::from([(String::new(), self.own.muted)]),
        }
    }
}

pub fn devices() -> Devices {
    REPORTS.lock().unwrap().devices()
}

/// Gibt es nur mit mehreren Instanzen.
pub fn send_headset_action(_profile: &str, _action: &crate::plugins::headset::Action) {}

pub fn forward_link(_url: &str) {}

pub fn ensure_free() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn instances_busy() -> Option<String> {
    None
}

pub fn send_headset_test() {}

pub fn send_busylight_test(_sound: String, _volume: u8) {}

/// Angemeldete Anlage für den Tooltip merken
pub fn set_label(app: &AppHandle, label: Option<(String, String)>) {
    *LABEL.lock().unwrap() = label;
    apply_titles(app);
}

pub fn install_desktop_entry(_app: &AppHandle, _host: Option<&str>) {}

/// Tooltip des Tray-Symbols
pub fn apply_titles(app: &AppHandle) {
    let tooltip = match LABEL.lock().unwrap().as_ref() {
        Some((_, name)) => format!("StarCMX: {name}"),
        None => t("StarCMX: abgemeldet").to_owned(),
    };
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_title("StarCMX");
    }
    if let Some(w) = app.get_webview_window("quick") {
        let _ = w.set_title(t("StarCMX Schnellwahl"));
    }
    if let Some(tray) = app.tray_by_id(&tray_id()) {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

pub fn quit(app: &AppHandle) {
    app.exit(0);
}

pub fn open_new(_app: &AppHandle) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(id: &str, ringing: bool) -> ReportLine {
        ReportLine {
            id: id.into(),
            ringing,
            held: false,
        }
    }

    #[test]
    fn ringing_calls_in_arrival_order_after_others() {
        let mut r = Reports {
            own: Report::default(),
            ring_order: None,
            next: 0,
        };
        let report = |lines| Report {
            ready: true,
            muted: false,
            lines,
        };
        r.set(report(vec![line("b", true)]));
        r.set(report(vec![
            line("a", true),
            line("b", true),
            line("c", false),
        ]));
        let ids: Vec<_> = r.devices().lines.into_iter().map(|l| l.id).collect();
        assert_eq!(ids, ["c", "b", "a"]);
    }
}
