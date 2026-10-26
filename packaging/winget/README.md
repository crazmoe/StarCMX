# winget

Manifest für das [Windows Package Manager Community Repository](https://github.com/microsoft/winget-pkgs),
Paket-ID `crazmoe.StarCMX`. Danach installiert man StarCMX mit

```
winget install crazmoe.StarCMX
```

`manifests/` hat denselben Aufbau wie in winget-pkgs; der Ordner der aktuellen Version wird dort
1:1 eingereicht.

## Erste Einreichung (einmalig, von Hand)

1. `microsoft/winget-pkgs` auf GitHub forken (Konto `crazmoe`).
2. Im Fork einen Branch anlegen und den Ordner
   `manifests/c/crazmoe/StarCMX/1.0.1/` von hier an dieselbe Stelle kopieren
   (über „Add file › Upload files“ im Browser oder per git).
3. PR gegen `microsoft/winget-pkgs` öffnen, die Vorlage ausfüllen und die Microsoft-CLA
   bestätigen, wenn der Bot danach fragt.
4. Die Prüfung von Microsoft läuft automatisch (Installation in einer Test-VM, Virenscan) und
   dauert meist ein bis drei Tage. Rückfragen kommen als Kommentar im PR.

Alternativ unter Windows: `winget install wingetcreate`, dann
`wingetcreate submit --token <token> packaging\winget\manifests\c\crazmoe\StarCMX\1.0.1`.

Vor dem Einreichen lokal prüfen (Windows):

```
winget validate --manifest packaging\winget\manifests\c\crazmoe\StarCMX\1.0.1
winget install --manifest packaging\winget\manifests\c\crazmoe\StarCMX\1.0.1
```

## Updates (automatisch)

Der Job `winget` in `.github/workflows/ci.yml` meldet jedes neue Release über
[winget-releaser](https://github.com/vedantmgoyal9/winget-releaser) an winget: Er übernimmt die
Angaben der letzten Version, setzt Version, URL und Hash neu und öffnet den PR über den Fork.
Voraussetzungen:

- Der erste Eintrag ist in winget-pkgs aufgenommen.
- Der Fork `crazmoe/winget-pkgs` existiert.
- Repo-Secret `WINGET_TOKEN`: ein *classic* Personal Access Token mit dem Recht `public_repo`
  (Settings › Developer settings › Personal access tokens › Tokens (classic)).

Ohne Secret, oder solange `crazmoe.StarCMX` in winget-pkgs noch nicht aufgenommen ist, überspringt
die CI den Schritt. Ein Release aus dieser Zeit lässt sich nach der Aufnahme nachmelden, indem man
den Job `winget` seines Laufs neu startet. Die Manifeste hier müssen dafür nicht nachgeführt
werden; sie dienen nur der ersten Einreichung.

## Hinweise

- `ProductCode` ist `StarCMX`: Der Tauri-NSIS-Installer trägt sich unter
  `HKCU\…\Uninstall\StarCMX` ein. Installiert wird pro Benutzer (`Scope: user`), still mit `/S`.
- Das Repo und seine Releases müssen öffentlich sein, sonst kann winget den Installer nicht laden.
- Der Installer ist nicht signiert; winget-pkgs nimmt auch unsignierte Installer an.
