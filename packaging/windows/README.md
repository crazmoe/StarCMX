# StarCMX im Firmennetz verteilen (Windows, GPO)

Zwei Skripte:
- **StarCMX-herunterladen.ps1** läuft auf dem Server und holt automatisch die neueste Version von GitHub auf die Freigabe.
- **StarCMX-anmeldeskript.ps1** läuft bei jeder Benutzeranmeldung und installiert bzw. aktualisiert StarCMX still.

StarCMX wird pro Benutzer installiert. Darum läuft die Installation als **Anmeldeskript** (Benutzerkonfiguration), nicht als Computer-Startskript.

## 1. Freigabe anlegen
- Zum Beispiel `\\SERVER\Software\StarCMX\`
- Rechte: Domänen-Benutzer **lesen**, das Konto der geplanten Aufgabe (z. B. SYSTEM bzw. das Computerkonto des Servers) **ändern**
- In beiden Skripten oben `$Freigabe` eintragen

## 2. Herunterladen automatisch (auf dem Server)
1. `StarCMX-herunterladen.ps1` z. B. nach `C:\Skripte\` kopieren
2. Einmal von Hand testen (PowerShell als Admin):
   `powershell -ExecutionPolicy Bypass -File C:\Skripte\StarCMX-herunterladen.ps1`
   Danach liegen auf der Freigabe `StarCMX_<Version>_x64-setup.exe`, `version.txt` und `herunterladen.log`.
3. Aufgabenplanung › Einfache Aufgabe erstellen:
   - Trigger: täglich, z. B. 05:00
   - Aktion: Programm starten, Programm `powershell.exe`,
     Argumente `-ExecutionPolicy Bypass -File C:\Skripte\StarCMX-herunterladen.ps1`
   - In den Eigenschaften: „Unabhängig von der Benutzeranmeldung ausführen“
   - Der Server braucht Internetzugang zu `api.github.com`, `github.com` und `objects.githubusercontent.com` (Proxy/Firewall)

Das Skript prüft die Prüfsumme der Datei. Erst wenn alles stimmt, schreibt es die neue Version in `version.txt`. Alte Installer räumt es weg. Fehler stehen in `herunterladen.log`.

## 3. GPO für die Benutzer
Gruppenrichtlinienverwaltung › neue GPO, mit der OU der Benutzer verknüpfen:
- **Benutzerkonfiguration › Richtlinien › Windows-Einstellungen › Skripts › Anmelden › Reiter „Skripts“** (nicht „PowerShell-Skripts“, dort blockiert die Windows-Ausführungsrichtlinie das Skript meist still) › Hinzufügen:
  - Skriptname: `powershell.exe`
  - Skriptparameter: `-ExecutionPolicy Bypass -NoProfile -WindowStyle Hidden -File "\\DOMAIN\SYSVOL\…\StarCMX-anmeldeskript.ps1"`
  (Skript vorher über „Dateien anzeigen“ in den GPO-Ordner kopieren und diesen Pfad nehmen, oder das Skript auf die StarCMX-Freigabe legen)

Das Skript installiert nur, wenn StarCMX fehlt oder eine andere Version installiert ist. Vor einem Update beendet es ein laufendes StarCMX.

Falls die SmartScreen-Meldung trotzdem kommt, zusätzlich in derselben GPO:
- **Benutzerkonfiguration › Administrative Vorlagen › Windows-Komponenten › Internet Explorer › Internetsystemsteuerung › Sicherheitsseite › Liste der Site zu Zonenzuweisungen**: `\\SERVER` (bzw. `file://SERVER`) mit Wert `1` (Lokales Intranet)

## 4. Testen
Auf einem Testrechner `gpupdate /force`, ab- und wieder anmelden. StarCMX sollte ohne Rückfrage installiert sein.

Updates kommen danach von selbst: neue Version auf GitHub → am nächsten Morgen auf der Freigabe → bei der nächsten Anmeldung beim Benutzer.

## 5. Wenn nichts installiert wird: Fehlersuche
Auf dem Testrechner, angemeldet als Testbenutzer:

1. **Wurde die GPO überhaupt angewendet?**
   `gpresult /r /scope user` → die GPO muss unter „Angewendete Gruppenrichtlinienobjekte“ stehen.
   Steht sie unter „Herausgefiltert“ oder fehlt sie: GPO ist mit der falschen OU verknüpft (die *Benutzer* müssen in der OU sein, nicht die Computer) oder die Sicherheitsfilterung passt nicht.
2. **Lief das Skript?** Datei `%TEMP%\StarCMX-gpo.log` öffnen (Windows-Taste+R, `%TEMP%` eingeben).
   - Keine Datei → Skript wurde gar nicht gestartet (Punkte 3 und 4).
   - Datei vorhanden → die letzte Zeile sagt, wo es hängt (Freigabe nicht erreichbar, Installer fehlt, Exitcode …).
3. **Anmeldeskript-Verzögerung:** Windows startet Anmeldeskripte standardmässig erst **5 Minuten** nach der Anmeldung. Also nach dem Anmelden mindestens 5 Minuten warten. Abschalten per GPO:
   **Computerkonfiguration › Administrative Vorlagen › System › Gruppenrichtlinie › Verzögerung für Anmeldeskript konfigurieren** → Aktiviert, Wert `0`. (Das ist eine Computereinstellung: Sie wirkt nur in einer GPO, die mit der OU der *Computer* verknüpft ist. Dasselbe gilt für Punkt 5.)
4. **Ereignisanzeige:** Anwendungs- und Dienstprotokolle › Microsoft › Windows › GroupPolicy › Operational. Nach Einträgen mit „Skript“ bzw. Ereignis-ID 5018/7018 (Skripts) schauen, dort steht ein allfälliger Fehler.
5. **Ausführungsrichtlinie:** Steht im Protokoll gar nichts, aber die GPO ist angewendet, blockiert evtl. die PowerShell-Ausführungsrichtlinie. Test von Hand als Benutzer:
   `powershell -ExecutionPolicy Bypass -File \\DOMAIN\SYSVOL\...\StarCMX-anmeldeskript.ps1`
   Klappt das, per GPO setzen: **Computerkonfiguration › Administrative Vorlagen › Windows-Komponenten › Windows PowerShell › Skriptausführung aktivieren** → „Lokale Skripts und remote signierte Skripts zulassen“.

## 6. Anlage für die Benutzer vorgeben
StarCMX liest beim Start die Datei `C:\ProgramData\StarCMX\starcmxrc` (Vorlage: `starcmxrc` hier im Ordner, Dateiname **ohne** Endung).
- `server=https://…` füllt die Adresse der Anlage im Anmeldefenster vor.
- `server[$i]=https://…` legt sie fest, der Benutzer kann keine andere wählen.

Verteilen per GPO (mit der OU der **Computer** verknüpft):
**Computerkonfiguration › Einstellungen › Windows-Einstellungen › Dateien** › Neu › Datei:
- Aktion: Ersetzen
- Quelldatei: `\\SERVER\Software\StarCMX\starcmxrc`
- Zieldatei: `C:\ProgramData\StarCMX\starcmxrc`

Benutzername und Passwort lassen sich nicht vorgeben: Der Benutzer meldet sich einmal auf der Anmeldeseite der Anlage an. Danach bleibt er angemeldet (das Anmelde-Token liegt in der Windows-Anmeldeinformationsverwaltung und hält 90 Tage).

Beim Bearbeiten mit dem Editor als „UTF-8“ speichern, nicht „UTF-16“/„Unicode“.
