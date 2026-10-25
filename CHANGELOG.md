# Änderungen

## Unveröffentlicht

### Zwei Clients am selben Konto
Liefen StarCMX und StarCLX gleichzeitig am selben Konto, nahm Annehmen an einem Client den Anruf
auch am anderen ab. StarCLX nahm ohne das Recht „Autoprovisionierung“ das App-Telefon der
Windows-App, das auch StarCMX hat; das ist dort behoben.

- Ein Anruf, der an einem anderen eigenen Telefon angenommen wurde, steht im Call-Manager als
  „An einem anderen Telefon angenommen“, ohne Stummschalter. Das Headset geht dafür nicht mehr
  ins Gespräch, und seine Taste legt dieses Gespräch nicht auf
- Unter Linux (Entwicklung) ist der Ersatz für das App-Telefon wie bei StarCLX das der Mac-App

## 0.1.0 – 2026-10-23

Erste Testversion für Windows, auf dem Stand von StarCLX 1.6.3.

- Installer (NSIS, pro Benutzer) und CI auf Windows.
- Audio über WASAPI, Softphone mit baresip (`wasapi`), G.722 und G.711, SRTP; Serverzertifikate
  prüft es gegen den Zertifikatsspeicher von Windows.
- Anmeldung im eigenen Fenster (WebView2) mit bestätigten Zertifikaten; Sitzung in der
  Windows-Anmeldeinformationsverwaltung. `starface-app://` bleibt der STARFACE App.
- Headset-Tasten (Jabra, Poly, EPOS) und Kuando Busylight über HID.
- Systemweite Tastenkürzel, Autostart, Abwesend bei Inaktivität, Bildschirmschoner oder Sperre,
  `tel:`/`callto:`/`sip:`-Links und Eintrag unter „Standard-Apps“.
- Vorgaben aus `%ProgramData%\StarCMX\starcmxrc`.
- Noch nicht: mehrere Anlagen gleichzeitig, Play/Pause von Bluetooth-Headsets, signierter
  Installer.
