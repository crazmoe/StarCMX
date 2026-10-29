# Änderungen

## 1.7.0 – 2026-10-29

### Warteschlangen
- Die Reiter „Angenommen“ und „Verpasst“ lassen sich auf **Heute** oder die letzten **7 Tage**
  begrenzen; die Wahl bleibt gespeichert
- Die Agenten stehen jetzt oben, direkt unter der Statistik und vor den Reitern

## 1.6.1 – 2026-10-29

### Warteschlangen
- Der Verlauf kommt jetzt wie im STARFACE-Windows-Client aus der Rufliste der Anlage: Reiter
  **Warteschlange**, **Angenommen** (mit „angenommen von“ und Dauer) und **Verpasst** (mit Rückruf).
  Er enthält damit auch Anrufe, während das Programm nicht lief, und die Notizen und
  Rückruf-Markierungen aus der Rufliste
- Der selbst mitgeschriebene Verlauf aus der Vorversion entfällt

## 1.6.0 – 2026-10-29

### Warteschlangen
- Neuer **Verlauf** je Warteschlange: welcher Anrufer von wem angenommen wurde (mit Wartezeit und
  Gesprächsdauer) und welche Anrufer nicht angenommen wurden (mit Wartezeit und bei wem es
  geklingelt hat). Filter Alle/Angenommen/Verpasst, Rückruf per Knopf, Verlauf leeren
- Die Anlage liefert keinen Verlauf. Das Programm schreibt ihn selbst mit, solange es läuft, und
  merkt sich die letzten 500 Anrufe. Ob ein Anrufer aufgelegt hat oder die Wartezeit abgelaufen
  ist, meldet die Anlage nicht; beides erscheint als „nicht angenommen“

## 1.5.1 – 2026-10-28

### Schnellwahl am Bildschirmrand
- Wird ein Bildschirm an- oder abgesteckt (z. B. Laptop ab- oder angedockt), wechselt die Leiste
  von selbst an den Hauptbildschirm und wieder zurück an den gewählten Bildschirm

## 1.5.0 – 2026-10-28

### Schnellwahl am Bildschirmrand
- Bei mehreren Bildschirmen lässt sich wählen, an welchem die Schnellwahl andockt
  (Einstellungen → Personalisierung → Bildschirm)

## 1.4.1 – 2026-10-28

### Verbindung
- Nach Standby oder Ruhezustand verbindet StarCMX wieder zuverlässig, auch das Wählen geht
  wieder. Windows meldet das Aufwachen jetzt direkt; bisher erkannte StarCMX es nicht, weil die
  Zeitmessung von Windows im Ruhezustand weiterläuft
- Nach dem Aufwachen wartet StarCMX, bis die Anlage antwortet (WLAN, VPN), baut dann eine neue
  Verbindung auf, erneuert die Anmeldung und startet das Softphone neu. Läuft ein Gespräch, wird
  nur die SIP-Verbindung erneuert
- Bricht die Verbindung zur Anlage ab, baut StarCMX sie sofort neu auf

## 1.4.0 – 2026-10-27

### Schnellwahl am Bildschirmrand
- Neu unter Einstellungen → Personalisierung → **Schnellwahl am Bildschirmrand**: Die Schnellwahl
  dockt links oder rechts an, randlos, immer im Vordergrund und so hoch wie der Bildschirm. Andere
  Fenster machen ihr Platz wie bei der Taskleiste
- Wahlweise **einfahren**: Nur ein schmaler Streifen bleibt am Rand; berührt die Maus ihn, fährt
  die Leiste aus und danach wieder ein
- Angedockt fehlt „Schnellwahl“ im Menü des Infobereichs
- Doppelklick auf das Symbol im Infobereich öffnet das Programmfenster; das Menü kommt mit rechts

## 1.3.0 – 2026-10-27

### Schnellwahl
- Das Schnellwahl-Fenster zeigt neben den Besetztlampenfeldern auch die **Direktwahltasten**
- Laufende Gespräche erscheinen oben im Schnellwahl-Fenster. Eine Gesprächskarte lässt sich auf
  ein Besetztlampenfeld oder eine Direktwahltaste ziehen: Das startet die Rückfrage, Auflegen
  vermittelt. Direktwahltasten sind jetzt auch im Hauptfenster Vermittlungsziel

## 1.2.0 – 2026-10-27

### Nextcloud
- Neu unter Einstellungen → Personalisierung → **Nextcloud**: Die Verbindung läuft über die
  Anmeldung im Browser (auch mit Zwei-Faktor). StarCMX bekommt nur ein App-Passwort und legt es
  im Schlüsselbund ab. „Trennen“ widerruft es. Im Instanzenbetrieb hat jede Anlage ihre eigene
  Nextcloud (oder keine)
- **Kontakte:** Die Nextcloud-Adressbücher erscheinen im Adressbuch als weitere Ordner und in
  der Suche. Kontakte darin lassen sich anlegen, bearbeiten und löschen, wobei unbekannte Angaben
  (Fotos, weitere Nummern) erhalten bleiben. Abgleich beim Start, alle 15 Minuten und auf Knopfdruck. Welche
  Adressbücher mitmachen, lässt sich einzeln wählen
- Liefert die Anlage keinen Namen für einen Anrufer, ergänzt StarCMX ihn aus den
  Nextcloud-Kontakten (abschaltbar), auch in der Rufliste
- **Teilen:** Aufzeichnungen (Wolken-Knopf) und Anrufnotizen („Anruf weitergeben“ →
  „Nextcloud-Link …“) werden in Nextcloud abgelegt, dazu entsteht ein Link zum Kopieren. Ordner,
  Ablauf des Links (Standard 7 Tage) und Passwortschutz sind einstellbar
- Der Ordner „Gesprächsnotizen“ ist für spätere Zusammenfassungen von Gesprächen vorbereitet

## 1.1.0 – 2026-10-27

### Arbeitsbereich
- Mehrere Ansichten bei „Frei anordnen“: Links steht eine Leiste mit den Ansichten, ein Klick
  zeigt deren Kacheln. Jede Ansicht hat ihre eigene Anordnung, ein Modul darf in mehreren
  Ansichten vorkommen. Im Bearbeiten-Modus legt **+** eine neue, leere Ansicht an
- Im Bearbeiten-Modus (Stift oben) öffnet ein Klick auf die gewählte Ansicht Name, Symbol und
  Löschen; Ziehen ändert die Reihenfolge. „Abbrechen“ nimmt alles zurück
- An der Ansicht steht die Zahl ungelesener Chats, neuer Voicemails usw. ihrer Kacheln
- Beim Start kommt die zuletzt gewählte Ansicht. Die bisherige Anordnung wird zur „Übersicht“.
  Im Instanzenbetrieb hat jede Anlage ihre eigenen Ansichten

## 1.0.3 – 2026-10-27

### Audio
- Gespeicherte Geräte, die gerade nicht verbunden sind, zeigen in den Audio-Einstellungen ihren
  Namen statt der Windows-Geräte-ID (`{0.0.0.00000000}.{…}`). Den Namen liefert Windows, auch
  für ausgesteckte oder deaktivierte Geräte; sonst gilt der zuletzt gesehene

## 1.0.2 – 2026-10-27

### Rufliste
- Neuer Schalter „Eigene“: blendet Gruppenanrufe aus, die verpasst oder von jemand anderem
  angenommen wurden. Er lässt sich mit den Filtern kombinieren, etwa „Eigene“ + „Verpasst“, und
  bleibt nach dem Neustart eingestellt
- Neuer Filter „Angenommen“ für angenommene eingehende Anrufe

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
