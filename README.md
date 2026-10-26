# StarCMX

**English** · [Deutsch](README.de.md)

StarCMX is an experimental, free Windows client for STARFACE phone systems. It is the Windows
port of [StarCLX](https://github.com/crazmoe/StarCLX) and shares its code: same features, same
interfaces (OneHub gRPC, SIP/TLS with SRTP, XMPP chat), STARFACE 10 and later.

Not affiliated with STARFACE GmbH. "STARFACE" is used for descriptive purposes only. StarCMX is
not the official STARFACE App for Windows and does not replace it.

> **Status:** version 1.0. Audio, headsets and Busylight are written against the Windows
> APIs, but are only tested on real hardware by hand. Please report what works and what doesn't.

## Features

Everything StarCLX offers (softphone, desk phone control, search and address book, call list,
chat, voicemail, recordings, conferences, door intercoms, function keys, reachability,
Busylight, headset buttons), plus on Windows:

- **Audio** via WASAPI: device list with the Windows names, ringtone and test tone, microphone
  level; the softphone uses baresip's `wasapi` module. Without a chosen device the
  *default communication device* is used.
- **Codecs**: G.722 (wideband), G.711, with SRTP.
- **Sign-in** in a window of its own (WebView2); self-signed certificates confirmed in StarCMX
  are accepted there, too. The session is stored in the Windows Credential Manager.
- **Headset buttons** over USB HID telephony and **Kuando Busylight**. Tested on Windows with
  Jabra; Poly, Yealink and EPOS use the same standard but are untested.
- **Keyboard shortcuts** system-wide (dial selected number, dial from clipboard, answer,
  hang up, toggle view).
- **Autostart**, away status on inactivity, screen saver or locked screen, `tel:`/`callto:`/`sip:`
  links (StarCMX also appears under *Settings › Apps › Default apps*).

- **Media keys** of the keyboard: during a call, play/pause answers or hangs up (StarCMX catches
  them while a call rings or runs). Buttons of Bluetooth headsets are not supported on Windows.

- **Several systems at once** ("Open another system" in the tray), as on Linux: one StarCMX per
  system, coordinated over named pipes.

Not (yet) on Windows: code signing of the installer.

## Installation

Under [Releases](../../releases) there is an installer `StarCMX_<version>_x64-setup.exe`
(per user, no admin rights needed). Development builds are stored as artifact
`starcmx-<version>-windows-x64` under **Actions**.

The installer is not signed yet, so SmartScreen asks once ("More info" › "Run anyway").
WebView2 is part of Windows 10/11; if it is missing, the installer downloads it.

### Defaults for all users (rollout)

Admins put system-wide settings in `%ProgramData%\StarCMX\starcmxrc` (usually
`C:\ProgramData\StarCMX\starcmxrc`), in the same format as on Linux. Template with all
settings: [`packaging/vorlagen/etc/xdg/starcmxrc`](packaging/vorlagen/etc/xdg/starcmxrc)
(comments in German). `key=value` is an initial value, `key[$i]=value` locks the setting.

### Door cameras with RTSP

RTSP cameras need `ffmpeg.exe` in the `PATH` (e.g. `winget install Gyan.FFmpeg`). Motion JPEG
and still images work without it.

## Login and the `windows-app` client ID

StarCMX signs in with the OAuth client `windows-app` and the return address
`starface-app://login`. The client ID only says that a Windows client is signing in; the
system numbers each client with a four-digit ID of its own. StarCMX does **not** register
`starface-app://` in Windows: its own login window catches the return address itself, so the
official STARFACE App keeps that link.

## Building

Only on Windows (MSVC). Needed: Rust (stable), Node.js 22, CMake, Visual Studio Build Tools
and OpenSSL from vcpkg:

```powershell
git clone --recurse-submodules https://github.com/crazmoe/StarCMX
vcpkg install openssl:x64-windows-static-md
$env:OPENSSL_DIR = "$env:VCPKG_INSTALLATION_ROOT\installed\x64-windows-static-md"
cd apps\desktop; npm ci; npx tauri build --bundles nsis
```

CI (`.github/workflows/ci.yml`) does the same on `windows-2022` and runs format, clippy and the
tests first.

### Protocol definitions

StarCMX does not carry its own copy of the `.proto` files. `crates/sf-proto` reads them from the
StarCLX repository, which is a git submodule under `upstream/` (pinned to a commit).

### Keeping in sync with StarCLX

The code is laid out like StarCLX. Where Windows needs something else, there is a variant with
the same interface next to the Linux one (`shortcuts_win.rs`, `media_win.rs`,
`flatpak_none.rs`, `sf-audio/src/wasapi.rs`, `sf-headset/src/hid.rs`), so changes from StarCLX
usually apply directly. `instances.rs` is shared; its D-Bus parts are Linux-only, and on Windows
`instances_pipe.rs` takes their place. Four UI state modules are called `*-state.svelte.ts` here (e.g.
`chat-state.svelte.ts`), because Windows does not tell `chat.svelte.ts` and `Chat.svelte` apart:

```sh
git -C upstream fetch && git -C upstream checkout <commit>
git -C upstream diff <old>..<new> -- apps crates \
  | sed 's/StarCLX/StarCMX/g; s/starclx/starcmx/g' | git apply --reject
```

## License

[MIT](LICENSE)
