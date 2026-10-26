# Änderungen

## 1.0.1 – 2026-10-26

### Adressbuch
- Die Detailansicht eines Kontakts zeigt neben den Rufnummern auch Nachname, Vorname, Strasse,
  PLZ, Ort, Land, E-Mail und Webseite, soweit die Anlage sie gespeichert hat. E-Mail und Webseite
  lassen sich anklicken

### Oberfläche
- Die Funktionstasten nutzen die ganze Fensterbreite: Die Spalten richten sich nur noch nach dem
  verfügbaren Platz. Die eingestellte Spaltenzahl gilt weiter für den Editor in den Einstellungen

## 1.0.0 – 2026-10-26

Erste stabile Version für Windows.

### Ton und Headsets
- Bluetooth-Headsets (HFP): Windows meldet das Mikrofon beim Umschalten auf den Gesprächsmodus
  oft kurz als nicht bereit. StarCMX versucht es jetzt bis zu 20 Sekunden lang erneut, statt
  das Gespräch ohne Mikrofon zu führen.
- Die in StarCMX gewählten Geräte gelten bei jedem Gespräch neu, auch wenn das Headset erst nach
  dem Start verbunden wurde, und unabhängig vom Windows-Standardgerät. Neue Gespräche beginnen
  gleich auf diesen Geräten.
- Headset-Tasten über HID: mit Jabra getestet; Poly, Yealink und EPOS nutzen denselben Standard,
  sind aber ungetestet. Tasten von Bluetooth-Headsets ohne HID (z. B. JBL) werden unter Windows
  nicht unterstützt; Medientasten der Tastatur gehen weiter.
- Meldungen von baresip stehen im Protokoll.

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
