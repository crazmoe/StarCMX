//! Ausserhalb von Linux gibt es kein Flatpak: Programme des Systems werden
//! direkt gestartet.

use std::process::Command;

pub fn app_id() -> Option<String> {
    None
}

pub fn sandboxed() -> bool {
    false
}

pub fn host_command(program: &str) -> Command {
    Command::new(program)
}
