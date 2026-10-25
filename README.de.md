# StarCMX

[English](README.md) · **Deutsch**

StarCMX ist ein experimenteller, freier Windows-Client für STARFACE-Telefonanlagen. Er ist die
Windows-Portierung von [StarCLX](https://github.com/crazmoe/StarCLX) und teilt dessen Code:
dieselben Funktionen, dieselben Schnittstellen (OneHub gRPC, SIP/TLS mit SRTP, XMPP-Chat),
STARFACE 10 und neuer.

Kein Angebot der STARFACE GmbH. „STARFACE“ wird nur beschreibend verwendet. StarCMX ist nicht die
offizielle STARFACE App für Windows und ersetzt sie nicht.

> **Stand:** frühe Testversion. Audio, Headsets und Busylight sind gegen die Windows-APIs
> geschrieben, aber nur von Hand an echter Hardware getestet. Rückmeldungen, was geht und was
> nicht, sind willkommen.

## Funktionen

Alles aus StarCLX (Softphone, Tischtelefon-Steuerung, Suche und Adressbuch, Anrufliste, Chat,
Voicemail, Mitschnitte, Konferenzen, Türsprechstellen, Funktionstasten, Erreichbarkeit,
Busylight, Headset-Tasten), dazu unter Windows:

- **Audio** über WASAPI: Geräteliste mit den Windows-Namen, Klingel- und Testton, Mikrofonpegel;
  das Softphone nutzt das `wasapi`-Modul von baresip. Ohne gewähltes Gerät gilt das
  *Standardgerät für Kommunikation*.
- **Codecs**: G.722 (Breitband), G.711, mit SRTP.
- **Anmeldung** im eigenen Fenster (WebView2); in StarCMX bestätigte selbstsignierte Zertifikate
  gelten auch dort. Die Sitzung liegt in der Windows-Anmeldeinformationsverwaltung.
- **Headset-Tasten** von Jabra, Poly und EPOS (USB-HID-Telefonie) und **Kuando Busylight**.
- **Tastenkürzel** systemweit (markierte Nummer wählen, aus der Zwischenablage wählen, annehmen,
  auflegen, Ansicht umschalten).
- **Autostart**, Abwesend bei Inaktivität, Bildschirmschoner oder gesperrtem Bildschirm,
  `tel:`/`callto:`/`sip:`-Links (StarCMX steht auch unter *Einstellungen › Apps › Standard-Apps*).

- **Medientasten** der Tastatur: Im Anruf nimmt Play/Pause an oder legt auf (StarCMX fängt sie
  ab, solange ein Anruf klingelt oder läuft). Tasten von Bluetooth-Headsets werden unter Windows
  nicht unterstützt.

- **Mehrere Anlagen gleichzeitig** („Weitere Anlage öffnen“ im Tray) wie unter Linux: je Anlage ein
  StarCMX, abgestimmt über Named Pipes.

Unter Windows (noch) nicht: signierter Installer.

## Installation

Unter [Releases](../../releases) gibt es den Installer `StarCMX_<version>_x64-setup.exe` (pro
Benutzer, ohne Administratorrechte). Entwicklungsstände liegen als Artefakt
`starcmx-<version>-windows-x64` unter **Actions**.

Der Installer ist noch nicht signiert; SmartScreen fragt deshalb einmal nach („Weitere
Informationen“ › „Trotzdem ausführen“). WebView2 gehört zu Windows 10/11; fehlt es, lädt der
Installer es nach.

### Vorgaben für alle Benutzer (Rollout)

Admins legen systemweite Einstellungen in `%ProgramData%\StarCMX\starcmxrc` ab (meist
`C:\ProgramData\StarCMX\starcmxrc`), im selben Format wie unter Linux. Vorlage mit allen
Einstellungen: [`packaging/vorlagen/etc/xdg/starcmxrc`](packaging/vorlagen/etc/xdg/starcmxrc).
`schluessel=wert` ist ein Startwert, `schluessel[$i]=wert` sperrt die Einstellung.

### Türkameras mit RTSP

RTSP-Kameras brauchen `ffmpeg.exe` im `PATH` (etwa `winget install Gyan.FFmpeg`). Motion JPEG und
Standbilder gehen ohne.

## Anmeldung und die Client-ID `windows-app`

StarCMX meldet sich mit dem OAuth-Client `windows-app` und der Rücksprungadresse
`starface-app://login` an. Die Client-ID sagt nur, dass sich ein Windows-Client anmeldet; die
Anlage nummeriert jeden Client mit einer eigenen vierstelligen Kennung. `starface-app://`
registriert StarCMX **nicht** in Windows: Das eigene Anmeldefenster fängt die Rücksprungadresse
selbst ab, der Link bleibt der offiziellen STARFACE App.

## Bauen

Nur unter Windows (MSVC). Nötig sind Rust (stable), Node.js 22, CMake, die Visual Studio Build
Tools und OpenSSL aus vcpkg:

```powershell
git clone --recurse-submodules https://github.com/crazmoe/StarCMX
vcpkg install openssl:x64-windows-static-md
$env:OPENSSL_DIR = "$env:VCPKG_INSTALLATION_ROOT\installed\x64-windows-static-md"
cd apps\desktop; npm ci; npx tauri build --bundles nsis
```

Die CI (`.github/workflows/ci.yml`) macht dasselbe auf `windows-2022` und prüft vorher
Formatierung, Clippy und Tests.

### Protokolldefinitionen

StarCMX hat keine eigene Kopie der `.proto`-Dateien. `crates/sf-proto` liest sie aus dem
StarCLX-Repo, das als Git-Submodul unter `upstream/` eingebunden ist (fest auf einen Commit).

### Abgleich mit StarCLX

Der Code ist aufgebaut wie in StarCLX. Wo Windows etwas anderes braucht, liegt neben der
Linux-Variante eine mit derselben Schnittstelle (`shortcuts_win.rs`, `media_win.rs`,
`flatpak_none.rs`, `sf-audio/src/wasapi.rs`, `sf-headset/src/hid.rs`). Änderungen aus StarCLX
lassen sich deshalb meist direkt übernehmen. `instances.rs` ist gemeinsam; seine D-Bus-Teile gelten
nur unter Linux, unter Windows tritt `instances_pipe.rs` an ihre Stelle. Vier Zustandsmodule der Oberfläche heissen hier
`*-state.svelte.ts` (etwa `chat-state.svelte.ts`), weil Windows `chat.svelte.ts` und
`Chat.svelte` nicht auseinanderhält:

```sh
git -C upstream fetch && git -C upstream checkout <commit>
git -C upstream diff <alt>..<neu> -- apps crates \
  | sed 's/StarCLX/StarCMX/g; s/starclx/starcmx/g' | git apply --reject
```

## Lizenz

[MIT](LICENSE)
