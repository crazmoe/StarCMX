# Änderungen

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
