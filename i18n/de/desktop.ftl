# de translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = Start
nav-library = Bibliothek
nav-airing = Läuft jetzt
nav-feed = Feed
nav-watching = Schaue ich
nav-stats = Statistik
nav-metadata = Serieninfo
nav-downloads = Downloads
nav-settings = Einstellungen
nav-series = Serie
nav-player = Player
nav-show = Serie
nav-recommended = Empfohlen

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } Fehler
   *[other] { $count } Fehler
}

## ScanSummary

scan-summary-nothing = Noch nichts zu zeigen
scan-summary-found = { $count ->
    [one] { $count } Serie
   *[other] { $count } Serien
} gefunden, { $recognised } erkannt
scan-summary-found-need = { $count ->
    [one] { $count } Serie
   *[other] { $count } Serien
} gefunden, { $recognised } erkannt, { $need } brauchen dich
scan-summary-episodes = { $count ->
    [one] { $count } Folge
   *[other] { $count } Folgen
} auf der Platte.
scan-summary-episodes-watched = { $count ->
    [one] { $count } Folge
   *[other] { $count } Folgen
} auf der Platte, { $watched } schon bei { $tracker } geschaut.
scan-summary-title = Dein Ordner ist drin
scan-summary-count-found = gefunden
scan-summary-count-recognised = erkannt
scan-summary-count-need = brauchen dich
scan-summary-later = Später
scan-summary-match = Die { $count } zuordnen
scan-summary-open-library = Bibliothek öffnen

## UpdateNotice

update-notice-check-failed = AniBeam konnte nicht nach Updates suchen. Prüfe deine Internetverbindung und versuch es nochmal.
update-notice-up-to-date = AniBeam ist aktuell
update-notice-downloading = Lade AniBeam { $version }
update-notice-ready = AniBeam { $version } ist bereit
update-notice-failed = Das Update ist fehlgeschlagen
update-notice-on-aur = AniBeam { $version } ist im AUR
update-notice-out = AniBeam { $version } ist da
update-notice-out-titled = AniBeam { $version } „{ $title }“ ist da
update-notice-progress = { $percent } %  ·  { $got } von { $total } MB
update-notice-restart-hint = Jetzt neu starten, oder es öffnet sich beim nächsten Mal
update-notice-from-source = Diese Kopie ist aus dem Quellcode gebaut: pullen und neu bauen
update-notice-no-download = Hier kein Download für diesen Computer
update-notice-install-hint = Aktualisieren installiert es und startet AniBeam neu
update-notice-retry = Nochmal
update-notice-update = Aktualisieren
update-notice-restart = Neu starten
update-notice-copied = Kopiert
update-notice-copy = Kopieren
update-notice-release-page = Release-Seite
update-notice-later = Später

## WhatsNew

whats-new-title = Neuerungen
whats-new-got-it = Verstanden

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = d. MMM HH:mm
activity-drawer-stage-library = Bibliothek
activity-drawer-stage-metadata = Metadaten
activity-drawer-stage-trackers = Tracker
activity-drawer-stage-franchise = Franchise
activity-drawer-stage-playback = Wiedergabe
activity-drawer-stage-store = Speicher
activity-drawer-stage-system = System
activity-drawer-stage-downloads = Downloads
activity-drawer-level-info = Info
activity-drawer-level-warn = Warnung
activity-drawer-level-error = Fehler
activity-drawer-copied-line = Zeile kopiert
activity-drawer-title = Aktivität
activity-drawer-everything = Alles
activity-drawer-problems = Probleme
activity-drawer-copied = Kopiert
activity-drawer-copy = Kopieren
activity-drawer-clear-every = Alle Zeilen leeren
activity-drawer-copy-tip = Klicken, um die Zeile zu kopieren
activity-drawer-fold = Einklappen
activity-drawer-empty-technical = Noch nichts protokolliert.
activity-drawer-empty-problems = Keine Probleme. Alles hat geklappt.
activity-drawer-empty = Noch nichts zu melden. Was AniBeam für dich tut, erscheint hier.

## ActivityColumn

activity-column-episode-run = Folgen { $first } bis { $last }
activity-column-episode-list = Folgen { $list }
activity-column-downloads = { $count ->
    [one] { $count } Download
   *[other] { $count } Downloads
}
activity-column-batch = Batch-Download
activity-column-download = Download
activity-column-name-batch = { $title }, ein Batch
activity-column-name-episode = { $title }, Folge { $number }
activity-column-title = Aktivität
activity-column-down = runter
activity-column-up = hoch
activity-column-not-connected = { $client } ist nicht verbunden, also bewegt sich hier nichts.
activity-column-connect-again = Neu verbinden
activity-column-downloading = Lädt gerade
activity-column-fewer = Weniger
activity-column-more = Und { $count } mehr
activity-column-nothing-downloading = Es wird nichts geladen.
activity-column-finished-today = Heute fertig
activity-column-today = Heute
activity-column-today-empty = Suchen, Downloads und neue Abos erscheinen hier, sobald sie passieren.
activity-column-stopping = Stoppt
activity-column-starting = Startet
activity-column-start-tip = Download starten
activity-column-stop-tip = Download stoppen; was schon da ist, bleibt
activity-column-cancelling = Bricht ab
activity-column-cancel-all-tip = Diese Downloads abbrechen
activity-column-cancel-tip = Diesen Download abbrechen
activity-column-size-of = { $got } von { $total }
activity-column-files-missing = Dateien fehlen
activity-column-time-format = HH:mm
activity-column-left = noch { $time }
activity-column-waiting-peers = Wartet auf Peers
activity-column-episodes-done = { $count ->
    [one] { $done } von { $count } Folge fertig
   *[other] { $done } von { $count } Folgen fertig
}
activity-column-cube-skipped = Folge { $number }: übersprungen
activity-column-cube-done = Folge { $number }: fertig
activity-column-cube-progress = Folge { $number }: { $percent } %

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Bereit
frame-strip-progress = { $doing }, { $done } von { $total }
frame-strip-idle = Nichts läuft
frame-strip-time-format = HH:mm
frame-finish-sign-in = Schließ zuerst die Anmeldung bei { $tracker } ab
frame-did-not-work = Das hat nicht geklappt.
frame-could-not-open = AniBeam konnte das nicht öffnen. Kopiere es und öffne es selbst.
frame-copy = Kopieren
frame-copied = Kopiert
frame-status-refused = { $tracker } hat es nicht übernommen
frame-status-refused-why = { $tracker } hat es nicht übernommen: { $why }
frame-lists-refused = { $tracker } hat { $count ->
    [one] 1 Serie
   *[other] { $count } Serien
} nicht übernommen; siehe Aktivitätsprotokoll
frame-lists-moved = { $count ->
    [one] 1 Serie
   *[other] { $count } Serien
} zu { $tracker } verschoben
frame-notes-refused = { $tracker } hat deine Notizen nicht übernommen
frame-notes-refused-why = { $tracker } hat deine Notizen nicht übernommen: { $why }
frame-notes-saved = Notizen gespeichert
frame-connected = Mit { $tracker } verbunden als { $user }
frame-sign-in-stopped = Die Anmeldung bei { $tracker } hat vor dem Ende aufgehört.
frame-toast-hide-details = Details ausblenden
frame-toast-details = Details
frame-toast-details-tip = Zeigt wörtlich, was schiefging, zum Kopieren
frame-forward = Vor
frame-copy-magnet = Magnet kopieren
frame-copy-link = Torrent-Link kopieren
frame-magnet-copied = Magnet kopiert
frame-link-copied = Link kopiert
frame-open-release-page = Release-Seite öffnen
frame-open-folder = Ordner öffnen
frame-copy-path = Pfad kopieren
frame-path-copied = Pfad kopiert
frame-mark-unwatched = Ab hier als ungeschaut markieren
frame-mark-watched = Bis hier als geschaut markieren
frame-copy-title = Titel kopieren
frame-open-on-anilist = Auf AniList öffnen
frame-list-home-online = Ändere es bei jeder verbundenen Liste
frame-list-home-unmatched = Nur auf diesem Gerät: Kein Tracker führt diese Serie
frame-list-home-not-carried = Nur auf diesem Gerät: Dein Tracker führt diese Serie nicht
frame-list-home-offline = Nur auf diesem Gerät, bis du AniList oder MyAnimeList verbindest
frame-moving-to = Wechsel zu { $list }
frame-moved-to = Gewechselt zu { $list }
frame-plan-added = { $title } kommt auf deine Liste Geplant
frame-plan-listed = { $title } steht schon auf deiner Liste
frame-delete-watched = { $kind ->
    [film] Geschaute Filme löschen
   *[episode] Geschaute Folgen löschen
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] Den Film löschen
       *[episode] Die Folge löschen
    }
   *[other] { $kind ->
        [film] Alle { $count } Filme löschen
       *[episode] Alle { $count } Folgen löschen
    }
}
frame-series-gone = Diese Serie ist weg
frame-delete-none-watched = Keine geschauten Folgen von { $title } auf der Platte
frame-delete-none = Nichts von { $title } auf der Platte
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } geschauten Film
           *[other] { $count } geschaute Filme
        }
       *[episode] { $count ->
            [one] { $count } geschaute Folge
           *[other] { $count } geschaute Folgen
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } Film
           *[other] { $count } Filme
        }
       *[episode] { $count ->
            [one] { $count } Folge
           *[other] { $count } Folgen
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } Extra
   *[other] { $count } Extras
}
frame-delete-and = { $first } und { $second }
frame-delete-heading = { $what } von { $title } löschen?
frame-delete-base = Die Dateien und ihre Untertitel kommen in den Papierkorb, von dort holst du sie zurück. Ihre Torrents verlassen den Torrent-Client, also lassen sie sich neu laden.
frame-delete-confirm = In den Papierkorb
frame-delete-done = { $what } von { $title } in den Papierkorb verschoben
frame-delete-watched-done = { $count ->
    [one] { $count } Folge
   *[other] { $count } Folgen
}, { $size }, in den Papierkorb verschoben
frame-delete-nothing = Nichts zu löschen
frame-fix-not-drafted = Korrigiert. Das Issue konnte nicht entworfen werden
frame-fix-agent = Korrigiert. { $agent } schaut es sich an, und das Issue öffnet sich, wenn es fertig ist
frame-fix-opening = Korrigiert. Das Issue wird geöffnet
frame-fix-not-saved = Die Korrektur wurde nicht gespeichert: { $error }
frame-fix-undone = { $count ->
    [one] { $count } Datei wird
   *[other] { $count } Dateien werden
} wieder nach Namen gelesen
frame-fix-done = Lesart von { $count ->
    [one] { $count } Datei
   *[other] { $count } Dateien
} korrigiert
frame-report-failed = Das Issue konnte nicht entworfen werden: { $error }
frame-report-untriaged = Das Issue ist in deinem Browser offen, ohne Triage. { $error }
frame-report-open = Das Issue ist in deinem Browser offen, bereit zum Absenden
frame-downloads-not-read = Nicht gelesen.

## HomePage

home-title = Start
home-scan-started = Scan gestartet
home-ago-minutes = vor { $count } Min.
home-ago-hours = vor { $count } Std.
home-ago-days = vor { $count } T.
home-today = Heute
home-tomorrow = Morgen
home-yesterday = Gestern
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d. MMM
home-day-date-format = ddd, d. MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = Noch kein Datum
home-unit-days = { $count ->
    [one] Tag
   *[other] Tage
}
home-unit-hours = Std.
home-unit-minutes = Min.
home-week-late = { $count ->
    [one] { $count } Folge ist spät dran
   *[other] { $count } Folgen sind spät dran
}
home-week-waiting = { $count ->
    [one] { $count } Folge ist da zum Schauen
   *[other] { $count } Folgen sind da zum Schauen
}
home-week-coming = { $count ->
    [one] { $count } Folge in den nächsten sieben Tagen
   *[other] { $count } Folgen in den nächsten sieben Tagen
}
home-week-next = Als Nächstes: { $title } in { $time }
home-countdown-days = { $days } T. { $hours } Std.
home-countdown-hours = { $hours } Std. { $minutes } Min.
home-countdown-minutes = { $minutes } Min.
home-out-not-downloaded = Noch nicht geladen
home-out-downloading = Lädt
home-out-when = { $day } { $time }
home-next-up = Als Nächstes
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, Zeit geschätzt
home-next-when-season = Staffel { $season }, { $span }, { $time }
home-next-when-season-estimated = Staffel { $season }, { $span }, { $time }, Zeit geschätzt
home-no-date = Kein Datum
home-tile-watched = geschaut
home-tile-downloading = lädt
home-tile-out = erschienen { $ago }
home-tile-not-out = noch nicht da
home-tile-estimated = geschätzt
home-tile-season-span = S{ $season } { $span }
home-tile-to-watch = { $count } zu schauen
home-day-nothing = Nichts
home-now = Jetzt { $time }
home-card-aired = { $episode } erschienen { $ago }
home-continue = Weiterschauen
home-continue-fact = { $count } Serien zum Weiterschauen
home-on-hold = Pausiert
home-on-hold-fact = { $count } Serien zum Wiederaufnehmen
home-hero-open = { $title } öffnen
home-hero-pick-up = Da weiter, wo du warst
home-hero-up-next = Als Nächstes
home-hero-downloading = Lade { $count ->
    [one] { $count } Folge
   *[other] { $count } Folgen
}
home-hero-min-left = noch { $count } Min.
home-hero-watched = { $watched } von { $total } geschaut
home-hero-resume = Weiter ab Folge { $number }
home-hero-play = Folge { $number } abspielen
home-hero-resume-unnumbered = Folge fortsetzen
home-hero-play-unnumbered = Folge abspielen
home-downloaded = Zuletzt geladen
home-downloaded-fact = { $count } Serien mit neuen Dateien in den letzten zwei Wochen
home-week = Diese Woche
home-week-show-empty = Leere Tage zeigen
home-week-hide-empty = Leere Tage ausblenden
home-week-airing = Läuft jetzt
home-week-out = Jetzt da
home-week-coming-up = Demnächst
home-recommended = Empfohlen für dich
home-recommended-fact = Aus deiner eigenen Liste
home-recommended-scope = Empfohlen aus
home-recommended-all = Alle
home-recommended-in-library = Bibliothek
home-recommended-on-list = Liste
home-recommended-none-library = Aus deiner Bibliothek ist nichts mehr zu empfehlen. Alle findet Serien, die du noch nicht hast.
home-recommended-none-list = Von deinen Listen ist nichts mehr zu empfehlen. Alle findet Serien, die du noch nicht gelistet hast.
home-catch-up = Aufholen
home-catch-up-fact = { $count } Serien mit wartenden Folgen
home-fresh = Etwas Neues anfangen
home-fresh-fact = { $count } Serien nicht angefangen
home-genres = Genres
home-open-library = Bibliothek öffnen
home-dropped = Abgebrochen
home-dropped-fact = { $count ->
    [one] { $count } Datei noch auf der Platte
   *[other] { $count } Dateien noch auf der Platte
}
home-out-now = Jetzt da
home-out-now-fact = Neue Folgen dieser Woche, von AniList
home-popular = Beliebt diese Saison
home-add-folder-title = Deine eigenen Serien gehören auch hierher
home-add-folder-body = Füge den Ordner mit deinen Anime hinzu, und Start füllt sich mit dem, was du schaust.
home-add-folder-button = Ordner hinzufügen
home-empty-scanned-title = Noch nichts in deinen Ordnern gefunden
home-empty-title = Noch keine Serien
home-empty-scanned-body = AniBeam hat in den beobachteten Ordnern keine Serien oder Filme gefunden. Scanne sie neu, oder prüfe die Ordner in den Einstellungen.
home-empty-body = Füge den Ordner mit deinen Anime hinzu, und die Bibliothek füllt sich von selbst.
home-scan-all = Alle scannen
home-library-settings = Bibliothekseinstellungen
home-failed-title = Start wurde nicht geladen
home-failed-body = AniBeam konnte die Bibliothek gerade nicht lesen. Versuch es gleich nochmal.

## Onboarding

onboarding-language = Sprache
onboarding-step-welcome = Willkommen
onboarding-step-library = Bibliothek
onboarding-step-lists = Deine Liste
onboarding-step-downloads = Downloads
onboarding-step-ready = Fertig
onboarding-scan-reading = Lese den Ordner
onboarding-scan-found = { $count } Serien gefunden. Mach weiter, wann du willst; der Rest füllt sich von selbst.
onboarding-scan-starter-empty = Dein neuer Ordner ist noch leer. Serien, die du hineinlegst, erscheinen hier von selbst.
onboarding-scan-no-video = Keine Videodateien gefunden. Prüfe, ob es der richtige Ordner ist.
onboarding-scan-your-shows = deine Serien
onboarding-scan-looking-up = Suche { $what }
onboarding-scan-looking-up-tally = Suche { $what }, { $posters } von { $count } gefunden
onboarding-scan-posters = Lade Poster
onboarding-scan-posters-tally = Lade Poster, { $posters } von { $count } gefunden
onboarding-scan-seasons = Suche verwandte Staffeln
onboarding-scan-seasons-tally = Suche verwandte Staffeln, { $posters } von { $count } gefunden
onboarding-choose-folder = Wähle deinen Anime-Ordner
onboarding-about = Eine Minute Einrichtung. Deine Bibliothek macht sich danach selbst fertig.
onboarding-step-later = Später
onboarding-skip-setup = Einrichtung überspringen
onboarding-title-welcome = Willkommen bei AniBeam
onboarding-title-library = Wo sind deine Anime?
onboarding-title-lists = Verbinde deine Anime-Liste
onboarding-title-downloads = So funktionieren Downloads
onboarding-title-ready = Alles bereit
onboarding-title-later = { $count ->
    [1] Bereit, eine Sache für später
    [2] Bereit, zwei Sachen für später
   *[other] Bereit, drei Sachen für später
}
onboarding-body-welcome = AniBeam hält deine Anime an einem Ort. Es findet die Serien auf deiner Platte, spielt sie direkt hier ab und hält AniList oder MyAnimeList beim Schauen aktuell.
onboarding-body-library = Wähle den Ordner, in dem deine Serien liegen. Jeder Ordner darin wird eine Serie, mit Poster, Folgen und der Stelle, an der du warst. AniBeam liest den Ordner nur. Es verschiebt oder benennt nie eine Datei um.
onboarding-body-lists = Melde dich einmal an, und jede Folge, die du hier beendest, wird auf deiner Liste markiert, ohne Tippen. Die Anmeldung läuft in deinem Browser, also sieht AniBeam dein Passwort nie.
onboarding-body-downloads = Abonniere eine Serie auf der Seite Downloads, und jede neue Folge landet bei Ausstrahlung in deiner Bibliothek. Dafür braucht es zwei Dinge: einen Ort zum Suchen und einen Torrent-Client zum Laden.
onboarding-body-ready = Alles hier lässt sich später in den Einstellungen ändern, und Einstellungen, Info startet diese Einrichtung erneut.
onboarding-skip-step = Erst mal überspringen
onboarding-get-started = Los geht's
onboarding-open-app = AniBeam öffnen
onboarding-continue = Weiter
onboarding-payoff-library = Deine Bibliothek
onboarding-payoff-library-line = Poster, Folgen und was als Nächstes kommt, für jede Serie in deinen Ordnern.
onboarding-payoff-plays = Spielt direkt hier
onboarding-payoff-plays-line = Untertitel, Intro überspringen, und es merkt sich, wo du warst.
onboarding-payoff-list = Deine Liste, gesynct
onboarding-payoff-list-line = Beende eine Folge, und sie zählt bei AniList oder MyAnimeList.
onboarding-payoff-episodes = Folgen kommen an
onboarding-payoff-episodes-line = Abonniere eine Serie, und jede neue Folge landet in deiner Bibliothek.
onboarding-films-too = Ein Ordner mit Filmen geht auch: Nenn ihn Movies.
onboarding-choose-folder-button = Ordner wählen
onboarding-no-folder = Ich habe noch keinen Ordner
onboarding-starter-where = Das legt einen Ordner Anime in { $dir } an und nutzt ihn.
onboarding-add-another = Weiteren Ordner hinzufügen
onboarding-shelf-more = mehr
onboarding-preview-show = Deine Serie
onboarding-preview-finished = Folge 3 hier beendet
onboarding-preview-on-list = Auf deiner Liste: 3 geschaut
onboarding-preview-set-by = Von AniBeam gesetzt, sobald der Abspann läuft
onboarding-lists-connected = Verbunden. Was du hier schaust, zählt jetzt auf deiner Liste.
onboarding-lists-no-account = Noch kein Konto? Beide sind kostenlos, und eins reicht. Trennen kannst du jederzeit unter Einstellungen, Konten.
onboarding-search-title = Ein Ort zum Suchen
onboarding-search-ready = { $count ->
    [one] { $count } Quelle bereit zum Suchen.
   *[other] { $count } Quellen bereit zum Suchen.
}
onboarding-search-none = AniBeam bringt keine eigenen Quellen mit. Du fügst sie aus einem Quellen-Repo hinzu: ein Link, den dir jemand gibt und den du unter Einstellungen, Downloads einfügst.
onboarding-sources-after = Öffnet nach der Einrichtung
onboarding-sources-add = Repo nach der Einrichtung hinzufügen
onboarding-fetch-title = Wer lädt
onboarding-fetch-auto = AniBeam kümmert sich
onboarding-fetch-recommended = Empfohlen
onboarding-fetch-auto-line = AniBeam bringt ein eigenes qBittorrent mit und lässt es still im Hintergrund laufen. Nichts zu installieren, nichts auszufüllen.
onboarding-fetch-existing = Ich nutze schon qBittorrent
onboarding-fetch-existing-line = AniBeam verbindet sich mit deinem laufenden qBittorrent. Adresse und Anmeldung trägst du gleich in den Einstellungen ein.
onboarding-ready-library = Bibliothek
onboarding-ready-library-none = Noch kein Ordner. Er findet sich unter Einstellungen, Bibliothek, wenn du ihn brauchst.
onboarding-ready-library-series = { $count ->
    [one] { $count } Ordner
   *[other] { $count } Ordner
}, bisher { $series } Serien
onboarding-ready-library-empty = { $count ->
    [one] { $count } Ordner
   *[other] { $count } Ordner
}, noch leer
onboarding-ready-library-scanning = { $count ->
    [one] { $count } Ordner
   *[other] { $count } Ordner
}, wird gescannt
onboarding-ready-lists = Deine Liste
onboarding-ready-lists-as = { $tracker }, als { $user }
onboarding-ready-lists-none = Noch nicht verbunden. Das findet sich unter Einstellungen, Konten, wenn du es brauchst.
onboarding-ready-downloads = Downloads
onboarding-ready-downloads-line = { $sources ->
    [0] Noch keine Quellen
    [one] { $sources } Quelle
   *[other] { $sources } Quellen
}, { $client ->
    [own] dein eigenes qBittorrent.
   *[bundled] das qBittorrent von AniBeam.
} { $next ->
    [ready] Alles bereit.
    [opens] Als Nächstes öffnet Einstellungen, Downloads.
   *[both] Beides findest du unter Einstellungen, Downloads.
}
onboarding-ready-open-downloads = Einstellungen, Downloads öffnen
onboarding-ready-set-up = Jetzt einrichten

## Rail.qml

rail-home = Start
rail-library = Bibliothek
rail-airing = Läuft
rail-feed = Feed
rail-stats = Statistik
rail-show-info = Serieninfo
rail-downloads = Downloads
rail-settings = Einstellungen
rail-downloads-complete =
    { $count ->
        [one] Downloads: { $count } Abo ist vollständig
       *[other] Downloads: { $count } Abos sind vollständig
    }
rail-title-language-name = Titelsprache: { $language }. Klicken für die nächste
rail-title-language-english = Englisch
rail-title-language-tip = Titelsprache: Romaji, Englisch oder Japanisch
rail-whats-new-in = Neu in { $release }
rail-whats-new = Neuerungen
rail-settings-note = Einstellungen: { $note }

## FixSheet.qml

fix-kind-episode = Folge
fix-kind-opening = Opening
fix-kind-ending = Ending
fix-kind-preview = Vorschau
fix-kind-special = Special
fix-kind-other = Sonstiges
fix-reads-season-episode = Staffel { $season } Folge { $number }
fix-reads-extra = { $kind } { $index }
fix-apply-failed = Die Korrekturen ließen sich nicht übernehmen.
fix-title = Korrigieren, wie Dateien gelesen werden
fix-lede = { $title }. Sag, was jede Datei wirklich ist; jeder spätere Scan behält das.
fix-rescan = Ordner neu scannen
fix-rescan-started = Lese den Ordner neu
fix-filled-in = Ausgefüllt
fix-fill-in = Ausfüllen
fix-column-file = Datei
fix-column-reads-as = Gelesen als
fix-column-should-be = Sollte sein
fix-fixed = Korrigiert
fix-number-placeholder = Nr.
fix-index-placeholder = Keine
fix-keep = Korrektur behalten
fix-leave-as-read = Datei so lassen, wie sie gelesen wird
fix-read-by-name = Datei wieder nach Namen lesen
fix-report = Issue auf GitHub melden
fix-report-body = Öffnet ein ausgefülltes Issue zum Absenden. Es ist öffentlich und nennt diese Dateien.
fix-report-body-agent = Öffnet ein ausgefülltes Issue zum Absenden. Es ist öffentlich und nennt diese Dateien. { $agent } schaut es sich vorher an.
fix-close-unapplied =
    { $count ->
        [one] Schließen, ohne { $count } Korrektur zu übernehmen?
       *[other] Schließen, ohne { $count } Korrekturen zu übernehmen?
    }
fix-number-invalid = Eine rote Zahl muss zuerst korrigiert werden
fix-close-anyway = Trotzdem schließen
fix-keep-editing = Weiter bearbeiten
fix-apply-count =
    { $count ->
        [one] { $count } Korrektur übernehmen
       *[other] { $count } Korrekturen übernehmen
    }

## MatchPanel.qml

match-matched-to = Zugeordnet zu { $title }
match-applied = Zuordnung übernommen
match-found-tvmaze = Serie bei TVmaze gefunden
match-found-anilist = Bei AniList gefunden
match-found-anilist-season = Bei AniList gefunden, Staffel { $season }
match-found-mal = Bei MyAnimeList gefunden
match-sites-and = { $first } und { $next }
match-matched-on = Zugeordnet bei { $sites }
match-provider-silent = { $provider } hat nicht geantwortet
match-title-walk = Zuordnen, { $walk }
match-title = Zuordnen
match-skip = Überspringen
match-refresh-info = Infos neu laden
match-refreshing = { $title } wird aktualisiert
match-clear-confirm = Lösen: entfernt die Verknüpfung und die gesetzte Folgenzahl
match-clear = Zuordnung lösen
match-cleared = Zuordnung gelöst
match-search-placeholder = { $provider } durchsuchen
match-link-tvmaze = Link von TVmaze oder IMDb einfügen
match-link-anilist = Link von AniList oder MyAnimeList einfügen
match-look-up = Suchen
match-searching = Suche { $provider }
match-type-more = Gib mindestens zwei Buchstaben ein, um { $provider } zu durchsuchen.
match-no-results = Keine Titel bei { $provider } gefunden.

## WatchingPage.qml

watching-episode-code = F { $number }
watching-countdown-days = { $days } T. { $hours } Std.
watching-countdown-hours = { $hours } Std. { $minutes } Min.
watching-countdown-minutes = { $minutes } Min.
watching-on-disk = Auf der Platte
watching-on-anilist = Auf AniList
watching-ready-to-start = Bereit zum Start
watching-not-in-library = Nicht in der Bibliothek
watching-get-to-start = Lade Folgen zum Starten
watching-anilist-updated = AniList aktualisiert { $when }
watching-airing = Läuft
watching-next-in = Nächste: { $title } in { $wait }
watching-episodes-waiting = Folgen warten
watching-series-behind =
    { $count ->
        [one] { $count } Serie im Rückstand
       *[other] { $count } Serien im Rückstand
    }
watching-up-to-date = Auf dem Stand
watching-anilist-only = Nur auf AniList
watching-planning-summary-off = Was du auf AniList schauen willst.
watching-planning-summary =
    { $count ->
        [one] { $count } Serie geplant, { $here } auf der Platte.
       *[other] { $count } Serien geplant, { $here } auf der Platte.
    }
watching-summary-off = Was du auf AniList schaust oder erneut schaust.
watching-summary-waiting =
    { $count ->
        [one] { $count } Serie hat wartende Folgen.
       *[other] { $count } Serien haben wartende Folgen.
    }
watching-caught-up = Alles aufgeholt.
watching-start-watching = Anfangen
watching-play-next = Nächste abspielen
watching-open-on-anilist = Auf AniList öffnen
watching-refreshing = Aktualisiere
watching-connect-title = AniList ist nicht verbunden
watching-connect-body = Verbinde es, und hier stehen die Serien, die du auf AniList schaust oder wieder schaust, mit Abspielen bei vorhandenen, Countdown bei laufenden.
watching-connect-waiting = Warte auf den Browser
watching-connect-anilist = AniList verbinden
watching-load-failed-title = Die Liste Schaue ich wurde nicht geladen
watching-load-failed-body = AniList war nicht erreichbar. Die Liste kommt beim nächsten Aktualisieren zurück.
watching-planning-failed-title = Die Liste Geplant wurde nicht geladen
watching-planning-failed-body = Ihr Cache war nicht lesbar. Versuch es nochmal, oder sie kommt beim nächsten Aktualisieren zurück.
watching-planning-empty-title = Nichts auf deiner Liste Geplant
watching-planning-empty-body = Vormerken unter Läuft jetzt, oder Geplant im Listenmenü einer Serie, setzt sie hierher.
watching-airing-now = Läuft jetzt
watching-empty-title = Nichts auf deiner Liste Schaue ich
watching-empty-body = Setze eine Serie auf AniList auf Schaue ich, und sie landet hier.
watching-open-anilist = AniList öffnen

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] { $count } Serie ausgeblendet
       *[other] { $count } Serien ausgeblendet
    }
library-hidden-sync-stops =
    { $count ->
        [one] { $count } Serie ausgeblendet, ihr Tracker-Sync stoppt
       *[other] { $count } Serien ausgeblendet, ihr Tracker-Sync stoppt
    }
library-unhidden =
    { $count ->
        [one] { $count } Serie wieder da und wird gesynct
       *[other] { $count } Serien wieder da und werden gesynct
    }
library-undo = Rückgängig
library-clearing-score =
    { $count ->
        [one] Lösche die Wertung von { $count } Serie
       *[other] Lösche die Wertung von { $count } Serien
    }
library-rating =
    { $count ->
        [one] Bewerte { $count } Serie mit { $score }
       *[other] Bewerte { $count } Serien mit { $score }
    }
library-moving =
    { $count ->
        [one] Verschiebe { $count } Serie nach { $list }
       *[other] Verschiebe { $count } Serien nach { $list }
    }
library-rescanning =
    { $count ->
        [one] Scanne { $count } Serie neu
       *[other] Scanne { $count } Serien neu
    }
library-finding-matches =
    { $count ->
        [one] Suche eine Zuordnung für { $count } Serie
       *[other] Suche eine Zuordnung für { $count } Serien
    }
library-sort-last-viewed = Zuletzt geschaut
library-sort-downloaded = Zuletzt geladen
library-sort-title = Titel
library-sort-progress = Fortschritt
library-sort-average-score = Durchschnittswertung
library-sort-my-score = Meine Wertung
library-dir-a-to-z = A bis Z
library-dir-z-to-a = Z bis A
library-dir-oldest = Älteste zuerst
library-dir-newest = Neueste zuerst
library-dir-least-watched = Am wenigsten geschaut zuerst
library-dir-most-watched = Am meisten geschaut zuerst
library-dir-lowest = Niedrigste zuerst
library-dir-highest = Höchste zuerst
library-group-none = Nichts
library-group-season = Saison
library-group-type = Typ
library-group-genre = Genre
library-group-watch-state = Schaustatus
library-scan-started = Scan gestartet
library-tab-all = Alle
library-tab-series = Serien
library-tab-movies = Filme
library-tab-watched = Geschaut
library-tab-hidden = Ausgeblendet
library-count-movies =
    { $count ->
        [one] { $count } Film
       *[other] { $count } Filme
    }
library-count-watched = { $count } geschaut
library-count-hidden = { $count } ausgeblendet
library-count-series =
    { $count ->
        [one] { $count } Serie
       *[other] { $count } Serien
    }
library-title = Bibliothek
library-all-genres = Alle Genres
library-search-placeholder = Titel oder Ordner suchen
library-sort = Sortieren
library-group = Gruppieren
library-franchises-together = Franchises zusammen
library-watched-shows = Geschaute Serien
library-load-failed-title = Die Bibliothek wurde nicht geladen
library-no-matches = Keine Treffer für „{ $query }“
library-none-in-genre =
    { $what ->
        [series] Keine { $genre }-Serien hier
        [films] Keine { $genre }-Filme hier
        [watched] Keine geschauten { $genre }-Serien hier
        [hidden] Keine ausgeblendeten { $genre }-Serien hier
       *[shows] Keine { $genre }-Serien hier
    }
library-none-here =
    { $what ->
        [series] Keine Serien hier
        [films] Keine Filme hier
        [watched] Keine geschauten Serien hier
        [hidden] Keine ausgeblendeten Serien hier
       *[shows] Keine Serien hier
    }
library-load-failed-body = AniBeam konnte die Bibliothek gerade nicht lesen. Versuch es gleich nochmal.
library-no-matches-body = Prüfe die Schreibweise, oder durchsuche unter Alle die ganze Bibliothek.
library-none-in-genre-body = Nichts in dieser Ansicht hat dieses Genre. Alle Genres zeigt den Rest.
library-none-in-tab-body = Davon hat deine Bibliothek noch nichts. Alle zeigt alles darin.
library-all-filtered-body = Alles in deiner Bibliothek ist aus dieser Ansicht herausgefiltert.
library-clear-search = Suche leeren
library-show-all = Alle zeigen
library-picked = { $count } gewählt
library-pick-all = Alle
library-pick-all-tip = Jede gezeigte Karte wählen (Strg+A)
library-unhide = Einblenden
library-hide = Ausblenden
library-rate = Bewerten
library-rate-tip = Eine Wertung für jede Auswahl, bei jedem verbundenen Tracker
library-needs-tracker = Braucht einen verbundenen Tracker und eine zugeordnete Serie
library-move-to-list = Auf Liste setzen
library-rescan = Neu scannen
library-find-match = Zuordnung suchen
library-find-match-tip = Für jede Auswahl ohne Zuordnung suchen
library-all-matched = Jede Auswahl ist zugeordnet
library-empty-scanned-title = Noch nichts in deinen Ordnern gefunden
library-empty-title = Deine Bibliothek ist leer
library-empty-scanned-body = AniBeam hat in den beobachteten Ordnern keine Serien oder Filme gefunden. Scanne sie neu, oder prüfe die Ordner in den Einstellungen.
library-empty-body = Füge den Ordner mit deinen Anime hinzu, und die Bibliothek füllt sich von selbst, oder importiere einen AniBeam-Export.
library-scan-all = Alle scannen
library-add-folder = Ordner hinzufügen
library-settings = Bibliothekseinstellungen
library-import = Importieren
library-import-title = AniBeam-Export importieren
library-import-filter-export = AniBeam-Export (*.json)
library-import-filter-all = Alle Dateien (*)
library-import-started = Import gestartet

## MetadataPage.qml

metadata-title = Serieninfo
metadata-filter-all = Alle { $count }
metadata-filter-series = Serien { $count }
metadata-filter-films = Filme { $count }
metadata-filter-missing = Fehlt { $count }
metadata-scan-started = Scan gestartet
metadata-sort-added = Hinzugefügt
metadata-sort-season = Saison
metadata-sort-a-to-z = A bis Z
metadata-section-unmatched = Nicht erkannt
metadata-section-missing = Ordner nicht gefunden
metadata-section-incomplete = Fehlende Folgen
metadata-section-empty = Nicht geladen
metadata-section-airing = Läuft
metadata-section-complete = Vollständig
metadata-no-info = Noch keine Serieninfo
metadata-state-aired = { $have } von { $aired } gelaufen
metadata-state-of = { $have } von { $expected }
metadata-state-of-extra = { $have } von { $expected } +{ $extra }
metadata-detail-none-aired = Keine der { $aired } gelaufenen Folgen ist auf der Platte
metadata-detail-aired = { $have } von { $aired } gelaufenen auf der Platte
metadata-detail-aired-missing = { $have } von { $aired } gelaufenen auf der Platte, { $missing } fehlen
metadata-detail-none = Keine der { $expected } Folgen ist auf der Platte
metadata-detail-missing = { $have } von { $expected } auf der Platte, { $missing } fehlen
metadata-walk-position = { $at } von { $total }
metadata-walk-done = Das war die letzte
metadata-walk-left =
    { $count ->
        [one] Noch eine unter Zuordnung nötig
       *[other] Noch { $count } unter Zuordnung nötig
    }
metadata-countdown-days = { $days } T. { $hours } Std.
metadata-countdown-hours = { $hours } Std. { $minutes } Min.
metadata-countdown-minutes = { $minutes } Min.
metadata-next-in = Nächste in { $wait }
metadata-files-of = { $have } von { $expected }
metadata-files-on-disk = Dateien auf der Platte
metadata-forgot = { $title } vergessen
metadata-refreshed-failed = { $refreshed } aktualisiert, { $failed } nicht
metadata-refreshed =
    { $count ->
        [one] { $count } Serie aktualisiert
       *[other] { $count } Serien aktualisiert
    }
metadata-found =
    { $count ->
        [one] { $count } Serie gefunden
       *[other] { $count } Serien gefunden
    }
metadata-found-some-left =
    { $count ->
        [one] { $count } Serie gefunden, { $unmatched } noch nicht erkannt
       *[other] { $count } Serien gefunden, { $unmatched } noch nicht erkannt
    }
metadata-found-nothing = Nichts Neues gefunden, { $unmatched } noch nicht erkannt
metadata-all-recognised = Jede Serie ist erkannt
metadata-count =
    { $count ->
        [one] { $count } Serie
       *[other] { $count } Serien
    }
metadata-search-placeholder = Titel filtern
metadata-find-rest = Rest suchen
metadata-find-rest-started = Suche die Ordner, die AniBeam noch nicht kennt
metadata-find-rest-tip = Sucht jeden Ordner, den AniBeam noch nicht kennt, auf jeder Infoseite in der Reihenfolge aus den Einstellungen
metadata-refresh-all = Alle aktualisieren
metadata-refresh-all-started = Aktualisiere die Infos jeder Serie
metadata-refresh-all-tip = Lädt die Infos jeder erkannten Serie neu
metadata-crawling = Suche Franchise-Links
metadata-crawl-progress = { $done } von { $total }
metadata-load-failed-title = Serieninfo wurde nicht geladen
metadata-empty-scanned-title = Noch nichts in deinen Ordnern gefunden
metadata-no-matches = Keine Treffer für „{ $query }“
metadata-nothing-under =
    { $filter ->
        [Series] Nichts unter Serien
        [Movies] Nichts unter Filme
        [MissingFiles] Nichts unter Fehlt
       *[other] Nichts unter { $filter }
    }
metadata-load-failed-body = AniBeam konnte die Bibliothek gerade nicht lesen. Versuch es gleich nochmal.
metadata-empty-scanned-body = AniBeam hat in den beobachteten Ordnern keine Serien gefunden. Scanne sie neu, oder prüfe die Ordner in den Einstellungen.
metadata-empty-body = Deine Bibliothek ist leer. Füge den Ordner mit deinen Anime hinzu, und AniBeam scannt ihn.
metadata-no-matches-body = Kein Titel in dieser Ansicht hat diese Wörter.
metadata-nothing-under-body = Keine deiner Serien passt gerade zu diesem Filter. Alle zeigt den Rest.
metadata-scan-all = Alle scannen
metadata-add-folder = Ordner hinzufügen
metadata-clear-search = Suche leeren
metadata-show-all = Alle zeigen
metadata-refreshing = { $title } wird aktualisiert
metadata-refresh-tip = Infos dieser Serie neu laden
metadata-forget-armed = Entfernen
metadata-forget-confirm-tip = Nochmal klicken, um diese Serie und ihren Verlauf aus AniBeam zu entfernen
metadata-forget-tip = Der Ordner ist nicht mehr auf der Platte: Entferne diese Serie und ihren Verlauf aus AniBeam

## AiringPage.qml

airing-title = Läuft jetzt
airing-sort-popular = Beliebtheit
airing-sort-score = Wertung
airing-sort-trending = Trend
airing-sort-favourites = Favoriten
airing-sort-soonest = Nächste Folge
airing-sort-aired = Gelaufene Folgen
airing-sort-episodes = Folgenzahl
airing-sort-start = Startdatum
airing-sort-duration = Folgenlänge
airing-sort-title = Titel
airing-dir-most-popular = Beliebteste zuerst
airing-dir-least-popular = Am wenigsten beliebt zuerst
airing-dir-lowest = Niedrigste zuerst
airing-dir-highest = Höchste zuerst
airing-dir-least-trending = Schwächster Trend zuerst
airing-dir-most-trending = Stärkster Trend zuerst
airing-dir-fewest = Wenigste zuerst
airing-dir-most = Meiste zuerst
airing-dir-soonest = Früheste zuerst
airing-dir-latest = Späteste zuerst
airing-dir-oldest = Älteste zuerst
airing-dir-newest = Neueste zuerst
airing-dir-shortest = Kürzeste zuerst
airing-dir-longest = Längste zuerst
airing-dir-a-to-z = A bis Z
airing-dir-z-to-a = Z bis A
airing-season =
    { $season ->
        [Winter] Winter { $year }
        [Spring] Frühling { $year }
        [Summer] Sommer { $year }
        [Fall] Herbst { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = vor { $minutes } Min.
airing-ago-hours = vor { $hours } Std.
airing-ago-days = vor { $days } T.
airing-episode-out = { $episode } erschienen { $ago }
airing-no-date = Noch kein Datum
airing-in-library = In der Bibliothek
airing-out-today =
    { $count ->
        [one] { $count } Folge heute erschienen
       *[other] { $count } Folgen heute erschienen
    }
airing-dismissed-one = { $title } erscheint hier nicht mehr
airing-dismissed =
    { $count ->
        [one] { $count } Serie erscheint hier nicht mehr
       *[other] { $count } Serien erscheinen hier nicht mehr
    }
airing-planned-already-one = { $title } steht schon auf deiner Liste
airing-planned-already-all = Alle stehen schon auf deiner Liste
airing-planned =
    { $count ->
        [one] { $count } Serie kommt auf deine Liste Geplant
       *[other] { $count } Serien kommen auf deine Liste Geplant
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } Serie kommt auf deine Liste Geplant, { $already } standen schon drauf
       *[other] { $count } Serien kommen auf deine Liste Geplant, { $already } standen schon drauf
    }
airing-play-next = Nächste abspielen
airing-plan-to-watch = Vormerken
airing-get-more-episodes = Mehr Folgen laden
airing-get-episodes = Folgen laden
airing-open-on-anilist = Auf AniList öffnen
airing-not-interested = Kein Interesse
airing-not-interested-count = { $count } ohne Interesse
airing-not-interested-tip = Serien, die du als uninteressant markiert hast, in den Einstellungen
airing-refreshing = Aktualisiere
airing-pick-season = Andere Saison wählen
airing-filter-all = Alles
airing-filter-lists = Auf meinen Listen
airing-filter-library = In meiner Bibliothek
airing-hide-lists = Meine Listen ausblenden
airing-hide-lists-tip = Lässt die Serien auf deinen Listen bei AniList oder MAL weg
airing-hide-library = Meine Bibliothek ausblenden
airing-hide-library-tip = Lässt die Serien weg, von denen du Folgen hast
airing-out-now = Jetzt da
airing-out-last-day =
    { $count ->
        [one] { $count } Folge in den letzten 24 Stunden
       *[other] { $count } Folgen in den letzten 24 Stunden
    }
airing-out-fewer = Weniger
airing-out-all = Alle { $count }
airing-sort = Sortieren
airing-load-failed-title = AniList war nicht erreichbar
airing-load-failed-body = Die laufenden Serien kommen von AniList. Sie kommen beim nächsten Aktualisieren zurück.
airing-reading-title = Lese, was gerade läuft
airing-nothing-now = Gerade läuft nichts
airing-nothing-for = Nichts läuft in { $season }
airing-nothing-read = Noch nichts gelesen
airing-reading-body = Jede laufende Serie, von AniList. Beim ersten Mal dauert das ein paar Sekunden.
airing-nothing-body = AniList listet für diese Saison noch keine Serien. Aktualisieren liest sie neu, oder wähle eine andere Saison.
airing-nothing-read-body = Aktualisieren liest jede laufende Serie von AniList.
airing-hides-empty-title = Deine Filter lassen hier nichts übrig
airing-lists-empty-title = Nichts hier steht auf deinen Listen
airing-library-empty-title = Nichts hier ist in deiner Bibliothek
airing-hides-empty-body = Jede Serie hier steht auf deinen Listen oder in deiner Bibliothek, und die hast du ausgeblendet.
airing-lists-empty-body = Vormerken auf einer Karte setzt sie auf deine Liste.
airing-library-empty-body = Folgen laden auf einer Karte lädt sie in deine Bibliothek.
airing-show-everything = Alles zeigen
airing-picked = { $count } gewählt
airing-undo = Rückgängig

## EpisodeRow.qml

episode-row-date-time-format = ddd, d. MMM, HH:mm
episode-row-date-upcoming-format = ddd, d. MMM
episode-row-date-format = d. MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Klicken, um ab hier als ungeschaut zu markieren
episode-row-mark-watched-tip = { $state }. Klicken, um bis hier als geschaut zu markieren
episode-row-next-up = Als Nächstes
episode-row-versions =
    { $count ->
        [one] { $count } Version
       *[other] { $count } Versionen
    }
episode-row-versions-tip =
    { $count ->
        [one] Dieselbe Folge aus { $count } Release. Klicken, um zu wählen, welche läuft
       *[other] Dieselbe Folge aus { $count } Releases. Klicken, um zu wählen, welche läuft
    }
episode-row-downloading = Lädt { $percent } %
episode-row-get = Folge laden
episode-row-predicted = Geschätzt
episode-row-extra = Extra
episode-row-resume = Weiter bei { $time }

## EpisodeCubes.qml

episode-cubes-one = Eine Folge
episode-cubes-range =
    { $count ->
        [one] { $count } Folge, { $first } bis { $last }
       *[other] { $count } Folgen, { $first } bis { $last }
    }
episode-cubes-tip = Folge { $number }: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } Extra
       *[other] { $count } Extras
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } nicht zugeordnete Datei
       *[other] { $count } nicht zugeordnete Dateien
    }
episodes-pane-on-disk = Auf der Platte
episodes-pane-not-on-disk = Nicht auf der Platte
episodes-pane-season = S{ $season }
episodes-pane-get-tip = Folge { $number } laden
episodes-pane-none-on-disk = Keine Folgen auf der Platte
episodes-pane-change-match = Zuordnung ändern
episodes-pane-refreshing = { $title } wird aktualisiert

## ShowDetail.qml

show-detail-not-in-library = Dieser Ordner ist noch nicht in der Bibliothek. Sobald ein Scan ihn findet, erscheinen hier Poster und Folgen.
show-detail-film-on-disk = Der Film ist auf der Platte.
show-detail-film-not-on-disk = Der Film ist noch nicht auf der Platte.
show-detail-complete-line = Zu Ende gelaufen, und alle { $total } Folgen sind da. Es kommt nichts mehr, also hat das Abo nichts mehr zu tun.
show-detail-finished = Zu Ende gelaufen.
show-detail-airing = Läuft.
show-detail-have-of-total = { $have } von { $total } Folgen hier.
show-detail-have =
    { $count ->
        [one] { $count } Folge hier.
       *[other] { $count } Folgen hier.
    }
show-detail-next-predicted =
    { $count ->
        [one] { $span } wird in { $wait } erwartet.
       *[other] { $span } werden in { $wait } erwartet.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span } läuft in { $wait }.
       *[other] { $span } laufen in { $wait }.
    }
show-detail-missing =
    { $count ->
        [one] Folge { $list } ist nicht da.
       *[other] Folgen { $list } sind nicht da.
    }
show-detail-missing-more = Folgen { $list } und { $more } weitere sind nicht da.
show-detail-downloaded = Geladen
show-detail-downloading = Lädt { $percent } %
show-detail-taken-adding = Von der Regel genommen, wird hinzugefügt
show-detail-taken = Von der Regel genommen
show-detail-left-out = Von der Regel ausgelassen
show-detail-subscribed-reading = Abonniert. qBittorrent liest den Feed.
show-detail-taken-moving = Die Regel nimmt { $taken } von { $releases ->
        [one] { $releases } Release
       *[other] { $releases } Releases
    } im Feed; { $moving ->
        [one] { $moving } lädt
       *[other] { $moving } laden
    } unten.
show-detail-taken-held = Die Regel nimmt { $taken } von { $releases ->
        [one] { $releases } Release
       *[other] { $releases } Releases
    } im Feed; { $torrents ->
        [one] { $torrents } Torrent ist
       *[other] { $torrents } Torrents sind
    } im Client.
show-detail-taken-adding-line = Die Regel nimmt { $taken } von { $releases ->
        [one] { $releases } Release
       *[other] { $releases } Releases
    } im Feed; qBittorrent fügt { $taken ->
        [one] es hinzu.
       *[other] sie hinzu.
    }
show-detail-feed-holds-nothing = Der Feed enthält noch nichts. Passende Releases werden geladen, sobald sie darin erscheinen.
show-detail-nothing-fits = Nichts im Feed passt bisher zur Regel. Passende Releases werden geladen, sobald sie erscheinen.
show-detail-complete = Vollständig
show-detail-open-in-library = In der Bibliothek öffnen
show-detail-feed-finds-nothing = Dieser Feed findet nichts
show-detail-feed-empty-query = Er fragt { $source } nach „{ $query }“, und das bleibt leer, also kann nichts Neues die Regel erreichen. Eine kürzere Suche liefert meist etwas; die weggelassenen Wörter können stattdessen in der Regel bleiben.
show-detail-feed-empty = Er fragt { $source }, und das bleibt leer, also kann nichts Neues die Regel erreichen. Eine kürzere Suche liefert meist etwas; die weggelassenen Wörter können stattdessen in der Regel bleiben.
show-detail-feed-empty-earlier = qBittorrent zeigt noch, was eine frühere Version dieses Feeds gefunden hat.
show-detail-repairing = Repariere
show-detail-repair = Reparieren
show-detail-unsubscribe = Abo beenden
show-detail-remove-torrents =
    { $count ->
        [one] Torrent entfernen
       *[other] { $count } Torrents entfernen
    }
show-detail-searches = Sucht
show-detail-quoted = „{ $words }“
show-detail-whole-feed = den ganzen Feed
show-detail-from = Von
show-detail-rule = Regel
show-detail-rule-unfollowable = Kann keine neuen Folgen holen: Die Quelle beantwortet Suchen, hat aber keinen Feed
show-detail-rule-active = Aktiv
show-detail-keeps = Behält
show-detail-keeps-value = Releases mit „{ $words }“
show-detail-drops = Verwirft
show-detail-drops-value = Releases mit { $words }
show-detail-or = { $first } oder { $second }
show-detail-last-took = Zuletzt genommen
show-detail-saves-to = Speichert in
show-detail-folder = Ordner
show-detail-in-the-feed = Im Feed
show-detail-feed-reading = Lese den Feed.
show-detail-feed-offers = Was der Feed jetzt anbietet, neueste zuerst.
show-detail-feed-is-empty = Der Feed ist leer.
show-detail-read-again = Neu lesen
show-detail-seeders =
    { $count ->
        [one] { $count } Seeder
       *[other] { $count } Seeder
    }
show-detail-download = Laden
show-detail-torrents = Torrents
show-detail-torrents-moving =
    { $count ->
        [one] { $count } lädt gerade; die Balken füllen sich dabei.
       *[other] { $count } laden gerade; die Balken füllen sich dabei.
    }
show-detail-torrents-held = Was der Client für diesen Ordner hält.
show-detail-torrents-waiting = Warte, bis qBittorrent hinzufügt, was die Regel nimmt.
show-detail-torrents-none = Der Client hält nichts für diesen Ordner.

## SeriesPage.qml

series-trackers-both = { $first } und { $second }
series-count-cleared = Folgenzahl gelöscht
series-count-set = Auf { $count } Folgen gesetzt
series-held-one-undated = Folge { $first } hat noch kein Datum
series-held-range-undated = Folgen { $first } bis { $last } haben noch kein Datum
series-held-one-due = Folge { $first } erwartet: { $when }
series-held-range-due = Folgen { $first } bis { $last } erwartet: { $when }
series-held-month-format = MMMM yyyy
series-fix-schedule = Folgen und Termine
series-fix-files = Dateien
series-fix-rescan-folder = Ordner neu scannen
series-fix-match = Zuordnen
series-find-match = Zuordnung suchen
series-rescan-show = Serie neu scannen
series-open-in-show-info = In Serieninfo öffnen
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } Film in den Papierkorb verschoben
           *[other] { $count } Filme in den Papierkorb verschoben
        }
        [episode] { $count ->
            [one] { $count } Folge in den Papierkorb verschoben
           *[other] { $count } Folgen in den Papierkorb verschoben
        }
       *[file] { $count ->
            [one] { $count } Datei in den Papierkorb verschoben
           *[other] { $count } Dateien in den Papierkorb verschoben
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } Film gewählt.
           *[other] { $count } Filme gewählt.
        }
        [episode] { $count ->
            [one] { $count } Folge gewählt.
           *[other] { $count } Folgen gewählt.
        }
       *[file] { $count ->
            [one] { $count } Datei gewählt.
           *[other] { $count } Dateien gewählt.
        }
    }
series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } Film in den Papierkorb
           *[other] { $count } Filme in den Papierkorb
        }
        [episode] { $count ->
            [one] { $count } Folge in den Papierkorb
           *[other] { $count } Folgen in den Papierkorb
        }
       *[file] { $count ->
            [one] { $count } Datei in den Papierkorb
           *[other] { $count } Dateien in den Papierkorb
        }
    }
series-delete-click = Klicke auf die Dateien, die gelöscht werden sollen.
series-delete-where = Sie kommen in den Papierkorb, von dort holst du sie zurück. Was du geschaut hast, bleibt erfasst.
series-delete-move = In den Papierkorb
series-pick-none = Keine wählen
series-pick-all = Alle wählen
series-rescan-started = Neuer Scan gestartet
series-rated = Bewertet mit { $score }
series-rated-cleared = Wertung gelöscht
series-score-failed = Wertung fehlgeschlagen
series-still-downloading = { $episode } wird noch geladen
series-format-series = Serie
series-tracker-refused = { $tracker } hat es nicht übernommen
series-tracker-refused-why = { $tracker } hat es nicht übernommen: { $why }
series-watched-up-to = Geschaut bis { $episode }
series-unwatched-from = Ungeschaut ab { $episode }
series-undo = Rückgängig
series-play-downloading = Lade Folge { $number }
series-play-downloading-unnumbered = Lade Folge
series-play-continue = Weiterschauen
series-play-again = Nochmal schauen
series-play-continue-episode = Weiter mit Folge { $number }
series-play-again-from = Nochmal ab Folge { $number }
series-play-episode = Folge { $number } abspielen
series-page-not-tracked = Nicht erfasst
series-not-tracked-on-disk =
    { $count ->
        [one] Nicht erfasst, { $count } Folge auf der Platte
       *[other] Nicht erfasst, { $count } Folgen auf der Platte
    }
series-watched = { $count } geschaut
series-watched-of = { $watched } von { $total } geschaut
series-watched-of-estimate = { $watched } von { $total }+ geschaut
series-facts-episodes-estimate =
    { $count ->
        [one] { $count } Folge+
       *[other] { $count } Folgen+
    }
series-facts-each = je { $runtime }
series-facts-on-disk = { $count } auf der Platte
series-next = Nächste: { $episode }
series-episodes-downloading =
    { $count ->
        [one] { $count } Folge lädt
       *[other] { $count } Folgen laden
    }
series-all-watched = Alle geschaut
series-no-poster = Kein Poster
series-next-season-span = S{ $season } { $span }
series-next-predicted = { $span } erwartet am { $date }, in { $wait }
series-next-date-format = ddd, d. MMM
series-next-airs =
    { $count ->
        [one] { $span } läuft in { $wait }
       *[other] { $span } laufen in { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] Lade { $count } Folge
       *[other] Lade { $count } Folgen
    }
series-count-season-ends = Staffel { $season } endet mit Folge { $number }
series-count-ends = Endet mit Folge { $number }
series-count-gave-way = Du hast { $count } Folgen gesetzt; das gilt nicht mehr
series-count-set-by-you = { $line }, von dir gesetzt
series-change = Ändern
series-get-episodes = Folgen laden
series-your-score = Deine Wertung { $score }
series-rate = Bewerten
series-page-average-score = Durchschnitt { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = { $votes } Stimmen
series-add-to-list = Zur Liste
series-page-rewatched =
    { $count ->
        [one] { $count }-mal erneut geschaut
       *[other] { $count }-mal erneut geschaut
    }
series-your-notes = Deine Notizen
series-add-notes = Notiz anlegen
series-notes-tip = Notizen zu deinem Listeneintrag, gespeichert bei { $sites }
series-unhide = Einblenden
series-hide = Ausblenden
series-hidden-toast = { $title } ist hier ausgeblendet, und AniBeam synct es nicht mehr mit AniList und MyAnimeList
series-unhidden-toast = { $title } ist wieder da und wird gesynct
series-hide-tip-tracked = Auf jeder Seite hier ausgeblendet, und AniBeam synct es nicht mehr mit AniList und MyAnimeList
series-hide-tip = Aus jeder Liste ausgeblendet
series-fixes = Korrektur
series-fixes-tip = Dateien, Folgenzahl oder Zuordnung korrigieren, oder neu scannen
series-quick-fix = Schnellkorrektur
series-film = Film
series-episodes = Folgen
series-extras = Openings, Endings und mehr
series-extras-openings = Openings
series-extras-endings = Endings
series-extras-previews = Vorschauen und Trailer
series-extras-specials = Specials
series-extras-other = Sonstiges
series-extras-group = { $kind }, { $count }
series-extra-files = Zusätzliche Dateien
series-extra-files-warning =
    { $count ->
        [one] Eine Datei ist mehr als die { $total } erwarteten Folgen: falsch benannt, doppelt oder ein Special. Benenne sie um oder entferne sie.
       *[other] { $count } Dateien sind mehr als die { $total } erwarteten Folgen: falsch benannt, doppelt oder Specials. Benenne sie um oder entferne sie.
    }
series-synopsis = Handlung
series-less = Weniger
series-more = Mehr
series-match = Zuordnen
series-not-matched = Noch nicht zugeordnet, also kein Poster, keine Handlung und kein Tracking. Suche die Serieninfo auf AniList oder TVmaze.
series-naming-title = Namen, die passen
series-naming-body = Der Ordner ist der Titel der Serie, so wie AniList ihn schreibt, mit der Staffel dahinter, wenn es nicht die erste ist. Jede Datei trägt die Folgennummer.
series-fix-how-files-read = Lesart der Dateien korrigieren
series-characters = Charaktere
series-related = Verwandt
series-related-filling = Wird noch von AniList nachgeladen.
series-tags = Tags
series-hide-spoilers = Spoiler ausblenden
series-spoilers = Spoiler
series-all-tags = Alle Tags
series-tag = { $name }, { $rank } %
series-recommendations = Empfehlungen
series-relation-owned = In deiner Bibliothek
series-relation-unreleased = Noch nicht erschienen
series-relation-on-anilist = Auf AniList
series-score-saved = Wertung gespeichert
series-score-cleared = Wertung gelöscht

## GetShow.qml

get-show-the-client = der Torrent-Client
get-show-group-batch = Batch von { $group }
get-show-run = { $first } bis { $last }
get-show-episodes =
    { $count ->
        [one] Folge { $list }
       *[other] Folgen { $list }
    }
get-show-episodes-inline =
    { $count ->
        [one] Folge { $list }
       *[other] Folgen { $list }
    }
get-show-sending-to = Sende an { $client }
get-show-go =
    { $count ->
        [one] { $count } Folge laden
       *[other] { $count } Folgen laden
    }
get-show-go-replace =
    { $count ->
        [one] { $count } Folge laden und ersetzen
       *[other] { $count } Folgen laden und ersetzen
    }
get-show-go-subscribe =
    { $count ->
        [one] { $count } Folge laden und abonnieren
       *[other] { $count } Folgen laden und abonnieren
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] { $count } Folge laden, ersetzen und abonnieren
       *[other] { $count } Folgen laden, ersetzen und abonnieren
    }
get-show-subscribe = Neue Folgen abonnieren
get-show-nothing-new = Nichts Neues zu laden
get-show-pick-episode = Wähle eine Folge
get-show-replace-off = Aus. Die Auswahl für { $episodes } bleibt draußen, weil die Bibliothek sie hat. Schalte es ein, um sie zu tauschen.
get-show-replace-on = Deine Dateien für { $episodes } kommen in den Papierkorb, sobald die neuen fertig sind.
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } lädt gerade: Dieser Download wird abgebrochen, und was er geschrieben hat, kommt in den Papierkorb.
       *[other] { $episodes } laden gerade: Dieser Download wird abgebrochen, und was er geschrieben hat, kommt in den Papierkorb.
    }
get-show-kind-on-demand = Startet, wenn eine Suche es braucht
get-show-kind-rss-search = RSS-Suche
get-show-kind-rss-feed = RSS-Feed
get-show-kind-json-search = JSON-Suche
get-show-kind-indexer-search = Indexer-Suche
get-show-ready = Bereit
get-show-running = Läuft
get-show-not-running = Läuft nicht
get-show-not-running-start = Läuft nicht. Klicke oben auf die Quelle, um sie zu starten.
get-show-switched-off = Ausgeschaltet
get-show-click-to-see = Klicken, um zu sehen, was sie für diese Serie hat
get-show-searching = Sucht
get-show-searching-source = Durchsuche { $name }
get-show-searching-elapsed = { $label }, { $seconds } s
get-show-releases =
    { $count ->
        [one] { $count } Release
       *[other] { $count } Releases
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } Release, { $cover } von { $total } Folgen abgedeckt
       *[other] { $count } Releases, { $cover } von { $total } Folgen abgedeckt
    }
get-show-on-your-list = Auf deiner Liste
get-show-plan-to-watch = Vormerken
get-show-on-your-list-tip = Sie steht auf deiner Liste
get-show-plan-to-watch-tip = Auf deine Liste Geplant setzen
get-show-open-series-tip = Die Serienseite: Details, Folgen und Abspielen
get-show-next-in = Folge { $number } in { $wait }
get-show-airing = Läuft
get-show-finished = Beendet
get-show-total-episodes = { $count } Folgen
get-show-downloading-episodes =
    { $count ->
        [one] Lade { $count } Folge
       *[other] Lade { $count } Folgen
    }
get-show-downloading-torrents =
    { $count ->
        [one] Lade { $count } Torrent
       *[other] Lade { $count } Torrents
    }
get-show-in-library = In deiner Bibliothek
get-show-not-in-library = Nicht in deiner Bibliothek
get-show-downloading = Lädt
get-show-stop-all = Alle stoppen
get-show-start-all = Alle starten
get-show-stop-all-tip = Jeden Download dieser Serie stoppen; was sie haben, bleibt
get-show-start-all-tip = Jeden Download dieser Serie wieder starten
get-show-cancel-all = Alle abbrechen
get-show-cancel-all-tip = Jeden Download dieser Serie abbrechen und das Geschriebene in den Papierkorb verschieben
get-show-sources = Verfügbare Quellen
get-show-check-again = Neu prüfen
get-show-check-again-tip = Fragt jeden Anbieter auf Abruf, ob er läuft. Startet nichts.
get-show-search-source = { $name } durchsuchen
get-show-release-list = Jedes Release als Liste
get-show-your-library = Deine Bibliothek
get-show-library-on-disk = { $have } von { $total } auf der Platte
get-show-library-downloading = { $count } laden
get-show-library-nothing = Noch nichts
get-show-row-batch = { $group }, Batch
get-show-batch-counting = zähle die Folgen, ein Torrent
get-show-batch-unlisted = Folgen nicht gelistet, die ganze Serie nach Titel, ein Torrent
get-show-batch-episodes =
    { $count ->
        [one] { $count } Folge, ein Torrent
       *[other] { $count } Folgen, ein Torrent
    }
get-show-nothing-new-picked = Nichts Neues gewählt
get-show-nothing-picked = Nichts gewählt
get-show-pick-provider = Wähle einen Anbieter
get-show-plan-line = { $episodes } von { $groups } über { $source }.
get-show-and = { $first } und { $second }
get-show-library-has =
    { $count ->
        [one] Die Bibliothek hat { $episodes }. Schalte unten Vorhandene Folgen ersetzen ein, um sie zu tauschen.
       *[other] Die Bibliothek hat { $episodes }. Schalte unten Vorhandene Folgen ersetzen ein, um sie zu tauschen.
    }
get-show-together = zusammen
get-show-torrents-unit =
    { $count ->
        [one] Torrent
       *[other] Torrents
    }
get-show-replace = Vorhandene Folgen ersetzen
get-show-sequential = Der Reihe nach laden
get-show-sequential-on = An: Jeder neue Torrent holt seine Teile von vorn, also werden die ersten Folgen zuerst fertig und du kannst schauen, während der Rest kommt. Insgesamt etwas langsamer.
get-show-sequential-off = Aus: Der Client holt die Teile in der Reihenfolge, die der Schwarm anbietet.
get-show-rolling = Die nächsten bereithalten
get-show-rolling-on = An: { $count } Folgen laden jetzt, und jede, die du beendest, startet die nächste.
get-show-rolling-on-delete = An: { $count } Folgen laden jetzt, und jede, die du beendest, startet die nächste. Geschaute Folgen verlassen die Platte.
get-show-rolling-off = Aus: Jede gewählte Folge lädt sofort.
get-show-follow-off = Aus: Neue Folgen erscheinen hier als Lücken, und nichts lädt von selbst.
get-show-follow-no-feed = { $source } hat keinen Feed zum Abonnieren, also können neue Folgen von dort nicht von selbst laden.
get-show-follow-pick-group = Wähle zuerst eine Gruppe: Ein Abo folgt einer Gruppe und Qualität.
get-show-follow-batch-only = { $group } hat hier nur den Batch, also gibt es keinen Feed einzelner Folgen zum Abonnieren.
get-show-follow-on = An: ein Feed für { $feed } über { $source }. Jede neue Folge lädt, sobald sie erscheint.
get-show-confirm =
    { $count ->
        [one] { $episodes } lädt gerade. Ersetzen bricht diesen Download ab und verschiebt das Geschriebene in den Papierkorb.
       *[other] { $episodes } laden gerade. Ersetzen bricht diesen Download ab und verschiebt das Geschriebene in den Papierkorb.
    }
get-show-confirm-go = Abbrechen und ersetzen
get-show-state-error = Fehler
get-show-state-files-missing = Dateien fehlen
get-show-state-stopped = Gestoppt
get-show-state-waiting-peers = Wartet auf Peers
get-show-state-metadata = Lade Metadaten
get-show-state-queued = In der Warteschlange
get-show-state-checking = Prüfe Dateien
get-show-state-allocating = Reserviere Platz
get-show-state-moving = Verschiebe Dateien
get-show-state-waiting = Wartet
get-show-a-batch = Ein Batch
get-show-no-group = Gruppe nicht im Namen
get-show-likely = wohl { $source }
get-show-tracker = Tracker { $tracker }
get-show-not-known = Noch nicht bekannt
get-show-bytes-of = { $done } von { $size }
get-show-speed-down = { $speed }/s runter
get-show-time-left = noch { $wait }
get-show-stopping = Stoppt
get-show-starting = Startet
get-show-stop = Stoppen
get-show-start-tip = Download wieder starten
get-show-stop-tip = Download stoppen; was schon da ist, bleibt
get-show-cancelling = Bricht ab
get-show-cancel-tip = Diesen Download abbrechen und das Geschriebene in den Papierkorb verschieben
get-show-episode-order = In Folgenreihenfolge laden
get-show-episode-order-on = An: Die Teile kommen vom Anfang jeder Datei, also werden die ersten Folgen zuerst fertig.
get-show-cube-skipped = Folge { $number }: übersprungen
get-show-cube-done = Folge { $number }: fertig
get-show-cube-percent = Folge { $number }: { $percent } %
get-show-fact-torrent = Torrent
get-show-fact-release = Release
get-show-fact-from = Von
get-show-fact-saving-to = Speichert in
get-show-span-counting = Zähle
get-show-span-unknown = { $runs }?
get-show-span-episodes = Folgen { $runs }
get-show-cube-no-release = Folge { $number }: kein Release von { $group }
get-show-seeders = { $count } Seeder
get-show-cube-on-disk = schon auf der Platte, klicken zum Ersetzen
get-show-cube-downloading = lädt gerade, klicken zum Ersetzen

## TorrentRow

torrent-row-state-downloading = Lädt
torrent-row-state-stalled = Hängt
torrent-row-state-metadata = Lade Metadaten
torrent-row-state-queued = Wartet
torrent-row-state-stopped = Gestoppt
torrent-row-state-done = Fertig
torrent-row-state-seeding = Seedet
torrent-row-state-checking = Prüft
torrent-row-state-moving = Verschiebt
torrent-row-state-error = Fehler
torrent-row-state-files-missing = Dateien fehlen
torrent-row-state-making-room = Schafft Platz
torrent-row-state-finding-peers = Sucht Peers
torrent-row-state-waiting = Wartet
torrent-row-done-of-size = { $done } von { $size }
torrent-row-speed-down = { $speed }/s runter
torrent-row-speed-up = { $speed }/s hoch
torrent-row-time-left = noch { $time }
torrent-row-seeders =
    { $count ->
        [one] { $count } Seeder
       *[other] { $count } Seeder
    }
torrent-row-rolling = rollend, Folgen { $next } bis { $through } von { $total }
torrent-row-stop = Stoppen

## SourceRow

source-row-unavailable-fix = Schließ das Laufwerk an, dann Neu scannen. Wenn der Ordner umgezogen ist, entferne ihn und füge den neuen Ort hinzu.
source-row-counts = { $count } Serien
source-row-counts-movies =
    { $movies ->
        [one] { $count } Serien, { $movies } Filmordner
       *[other] { $count } Serien, { $movies } Filmordner
    }
source-row-unavailable = Nicht verfügbar
    .tip = AniBeam erreicht diesen Ordner nicht. Liegt er auf einem Laufwerk, das nicht angeschlossen oder eingehängt ist, schließ es an, und AniBeam findet ihn wieder. Ist er umgezogen, entferne ihn hier und füge den neuen Ort hinzu.
source-row-rescan = Neu scannen
source-row-remove-question = { $name } entfernen, mit { $count } Serien und ihrem Verlauf?

## StartSourceSheet

start-source-title = { $service } für { $name } starten?
start-source-busy = Starte { $service }. { $name } schaltet sich ein, sobald es antwortet; das kann bis zu einer Minute dauern.
start-source-body = { $service } ist der Dienst, der { $name } für AniBeam durchsucht. Beim Start läuft er im Hintergrund auf diesem Computer, und er läuft weiter, wenn AniBeam geschlossen wird.
start-source-starting = Startet

## SearchProgress

search-progress-why-helper = AniBeam startet seinen Suchhelfer; das dauert ein, zwei Sekunden.
search-progress-why-starting = { $endpoint } läuft über einen Helfer, der startet, wenn eine Suche ihn braucht. Die erste Suche wartet auf ihn, bis zu { $seconds } Sekunden; die nächsten sparen sich das.
search-progress-why-reading = Teile die Releases in Gruppen auf und gleiche sie mit der Bibliothek ab.
search-progress-why-ondemand = Ein Helfer liest { $endpoint } für jede neue Suche frisch, was meist zehn bis zwanzig Sekunden dauert.
search-progress-why-slow = { $endpoint } antwortet gerade langsam.
search-progress-why-usual = Das dauert meist ein, zwei Sekunden.
search-progress-why-and-slow = { $why } Das dauert länger als sonst; brich ab, um eine andere Quelle zu probieren.
search-progress-searching = Suche
search-progress-seconds = { $seconds } s
search-progress-try = { $endpoint } probieren

## ReleaseTable

release-table-col-release = Release
release-table-col-group = Gruppe
release-table-col-res = Aufl.
release-table-col-rip = Rip
release-table-col-ep = F
release-table-col-size = Größe
release-table-col-seeders = Seeder
release-table-col-leechers = Leecher
release-table-col-downloads = Downloads
release-table-col-released = Erschienen
release-table-tick-all = Jedes Release ankreuzen
release-table-untick-all = Kein Release ankreuzen
release-table-on-disk = Auf der Platte
release-table-other-on-disk = Anderes Release auf der Platte
release-table-season-batch = Batch Staffel { $season }
release-table-batch = Batch
release-table-via = über { $source }
release-table-season-episode = S{ $season } { $episode }
release-table-date-format = d. MMMM yyyy, HH:mm
release-table-download = Laden
release-table-more = und { $count } weitere. Grenze die Liste ein oder sortiere sie, um andere zuerst zu sehen.
release-table-pick = Dieses Release wählen

## DownloadsPage

downloads-title = Downloads
downloads-builtin-client = Eingebauter Client
downloads-start-done = { $name } läuft und ist eingeschaltet
downloads-disk-film = Film
downloads-disk-watched = { $size } geschaut
downloads-disk-all-watched = alles geschaut
downloads-delete-watched-menu =
    { $count ->
        [one] Jede geschaute Folge: { $count } Folge, { $size }
       *[other] Jede geschaute Folge: { $count } Folgen, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Jede beendete Serie, die du geschaut hast: { $count } Serie, { $size }
       *[other] Jede beendete Serie, die du geschaut hast: { $count } Serien, { $size }
    }
downloads-titles-and = { $list } und { $last }
downloads-titles-more = { $list } und { $count } weitere
downloads-delete-finished-none = Keine beendete Serie ist ganz durchgeschaut
downloads-delete-watched-none = Nichts Geschautes liegt auf der Platte
downloads-delete-finished-heading =
    { $count ->
        [one] { $count } beendete Serie löschen, { $size }?
       *[other] { $count } beendete Serien löschen, { $size }?
    }
downloads-delete-finished-base = { $titles }. Nur Serien, die laut Anbieter beendet sind, mit jeder Folge geschaut; eine Serie, die noch läuft oder deren Ende geschätzt ist, bleibt. Jede ihrer Dateien kommt in den Papierkorb, auch Extras. Ihre Torrents verlassen auch { $client }, also lassen sie sich neu laden.
downloads-delete-watched-heading =
    { $count ->
        [one] { $count } geschaute Folge löschen, { $size }?
       *[other] { $count } geschaute Folgen löschen, { $size }?
    }
downloads-delete-watched-base = Jede Folge, die du geschaut hast, in jeder Serie. Noch nicht geschaute Folgen und Extras bleiben. Die Dateien und ihre Untertitel kommen in den Papierkorb, von dort holst du sie zurück. Ihre Torrents verlassen auch { $client }, also lassen sie sich neu laden.
downloads-move-to-trash = In den Papierkorb
downloads-remove-group-torrents =
    { $count ->
        [one] Torrent entfernen
       *[other] Seine { $count } Torrents entfernen
    }
downloads-run-span = { $from } bis { $to }
downloads-run-pair = { $first } und { $second }
downloads-gap-missing =
    { $count ->
        [one] Folge { $runs } fehlt
       *[other] Folgen { $runs } fehlen
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Folge { $runs } nicht geladen
       *[other] Folgen { $runs } nicht geladen
    }
downloads-gap-then-not-downloaded = { $runs } nicht geladen
downloads-trying-query = Probiere { $query }
downloads-client-settings = Einstellungen des Torrent-Clients
downloads-error-unknown = Etwas ist schiefgegangen. Versuch es nochmal.
downloads-search-all-off = Jede Quelle ist ausgeschaltet. Schalte eine ein, um zu suchen.
downloads-turn-on = { $name } einschalten
downloads-search-no-sources = Es gibt noch keine Quellen zum Suchen. Füge in den Einstellungen unter Downloads eine Quellenliste hinzu.
downloads-search-none-started = Keine Quelle konnte die Suche starten. Versuch es gleich nochmal.
downloads-auto-resolution = Eingegrenzt auf { $resolution }, die schärfste im Angebot
downloads-search-one-failed = Die Suche bei { $name } hat nicht geklappt. Versuch es nochmal oder durchsuche eine andere Quelle.
downloads-search-all-failed = Keine der Quellen konnte suchen. Versuch es gleich nochmal.
downloads-narrowing-on = Grenze ein auf { $name }
downloads-refine-zero = Dann bliebe nichts übrig, also bleibt die Liste, wie sie ist.
downloads-refine-no-match = Kein Release passt zu diesem Begriff.
downloads-refine-empty = Gib einen Begriff zum Eingrenzen ein.
downloads-refine-no-change = Das würde nichts ändern.
downloads-movie-tick-one = Kreuze ein Release an und klicke dann auf Als Film laden.
downloads-group-downloading = Lädt
downloads-group-downloading-line = Läuft gerade; Aktivität rechts zeigt die Details.
downloads-group-seeding = Seedet
downloads-group-seeding-line = Geladen und wird weiter geteilt.
downloads-group-done = Fertig
downloads-group-done-line = Geladen; der Client teilt sie nicht.
downloads-group-stopped = Gestoppt
downloads-group-stopped-line = Gestoppt, bevor sie fertig waren.
downloads-group-broken = Dateien fehlen
downloads-group-broken-line = Der Client findet ihre Dateien nicht: Sie zeigen auf einen Ordner, den es nicht gibt. Die Torrents zu entfernen räumt den Client auf.
downloads-sub-nofeed = Kein Abo möglich
downloads-sub-nofeed-line = Ihre Quelle beantwortet Suchen, hat aber keinen Feed, also kommen keine neuen Folgen. Beende das Abo und abonniere die Serie über eine Quelle mit Feed.
downloads-sub-complete = Vollständig
downloads-sub-complete-line = Jede Folge ist in der Bibliothek, und es läuft nichts mehr: Diese Abos haben nichts mehr zu tun.
downloads-sub-airing = Läuft
downloads-sub-airing-line = Neue Folgen kommen noch, die nächsten zuerst.
downloads-sub-incomplete = Unvollständig
downloads-sub-incomplete-line = Zu Ende gelaufen, aber Folgen fehlen, oder nichts sagt, wo die Serie steht.
downloads-sub-missing = Nicht in der Bibliothek
downloads-sub-missing-line = Ihr Ordner enthält nichts, was die Bibliothek gefunden hat: Es ist noch nichts angekommen, oder es speichert dort, wo die Bibliothek nicht sucht.
downloads-torrents =
    { $count ->
        [one] { $count } Torrent
       *[other] { $count } Torrents
    }
downloads-rule-film-on-disk = Film, auf der Platte
downloads-rule-all-here = Alle { $total } da
downloads-rule-next =
    { $predicted ->
        [yes] F { $episode } erwartet in { $time }
       *[no] F { $episode } in { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] F { $episode } bis { $through } erwartet in { $time }
       *[no] F { $episode } bis { $through } in { $time }
    }
downloads-rule-have-of = { $have } von { $total } da
downloads-rule-have =
    { $count ->
        [one] { $count } Folge da
       *[other] { $count } Folgen da
    }
downloads-tag-not-downloaded = Nicht geladen
downloads-tag-missing = { $count } fehlen
downloads-tag-n-not-downloaded = { $count } nicht geladen
downloads-tag-unknown = Unbekannt
downloads-speed-down = { $speed }/s runter
downloads-time-left = noch { $time }
downloads-repair-trying = Probiere kürzere Suchen
downloads-sent-to-client = { $title } an { $client } gesendet
downloads-files-torrents = was die Torrents geladen haben
downloads-files-torrents-remote = was die Torrents geladen haben, gelöscht von { $client }
downloads-files-folder-and = { $folder } und { $torrents }
downloads-unsubscribe-heading = Abo von { $title } beenden?
downloads-unsubscribe-base = Sein Feed und seine Downloadregel verlassen { $client }, also kommt nichts Neues.
downloads-unsubscribe-complete-heading = Abos aller { $count } vollständigen Serien beenden?
downloads-unsubscribe-complete-base = Ihre Feeds und Downloadregeln verlassen { $client }. Jede ihrer Folgen ist in der Bibliothek.
downloads-files-folders-and-torrents = ihre Ordner und was ihre Torrents geladen haben
downloads-one-at-a-time = Sie gehen einzeln; die Seite zeigt jeden Schritt.
downloads-subscription-stays = { $base } Das Abo bleibt.
downloads-cancel-heading = { $title } abbrechen?
downloads-cancel-group-base =
    { $count ->
        [one] Sein Download stoppt und verlässt { $client }.
       *[other] Seine { $count } Downloads stoppen und verlassen { $client }.
    }
downloads-files-so-far-it = was er bisher geladen hat
downloads-files-so-far-they = was sie bisher geladen haben
downloads-cancel-download = Download abbrechen
downloads-keep-downloading = Weiter laden
downloads-remove-heading = { $title } entfernen?
downloads-remove-group-base =
    { $count ->
        [one] Sein Torrent verlässt { $client }; die Dateien bleiben, außer du kreuzt unten an.
       *[other] Seine { $count } Torrents verlassen { $client }; die Dateien bleiben, außer du kreuzt unten an.
    }
downloads-remove-detail-base =
    { $count ->
        [one] Sein { $count } Torrent verlässt { $client }; die Dateien bleiben, außer du kreuzt unten an.
       *[other] Seine { $count } Torrents verlassen { $client }; die Dateien bleiben, außer du kreuzt unten an.
    }
downloads-remove-broken-heading = Die Torrents aller { $count } Serien mit fehlenden Dateien entfernen?
downloads-remove-broken-base = Ihre { $count } Torrents verlassen { $client }. Sie zeigen auf Ordner, die es nicht gibt, also wird keine Datei angefasst; Abos bleiben.
downloads-remove-torrent-heading = Diesen Torrent entfernen?
downloads-files-it-downloaded = was er geladen hat
downloads-cancel-torrent-heading = Diesen Download abbrechen?
downloads-cancel-many-heading =
    { $count ->
        [one] { $count } Download abbrechen?
       *[other] { $count } Downloads abbrechen?
    }
downloads-cancel-many-base = Jeder Download dieser Serie stoppt und verlässt { $client }.
downloads-cancel-all = Alle abbrechen
downloads-summary-connecting = Verbinde.
downloads-summary-client-on = { $client } { $version } ist verbunden.
downloads-summary-client-off = { $client } ist nicht verbunden. Prüfe Adresse und Passwort in den Einstellungen unter Downloads.
downloads-summary-native = AniBeam durchsucht deine Quellen selbst.
downloads-summary-native-with = AniBeam durchsucht deine Quellen selbst. { $client }
downloads-summary-starting = Starte Downloads.
downloads-summary-not-running = Downloads läuft nicht.
downloads-summary-not-started = Downloads ist noch nicht gestartet.
downloads-summary-helper-ready = Der Suchhelfer { $version } ist bereit. { $client }
downloads-subscribed-as = Abonniert als { $name }.
downloads-repaired =
    { $count ->
        [one] { $name } repariert: Der Feed findet { $count } Release
       *[other] { $name } repariert: Der Feed findet { $count } Releases
    }
downloads-repaired-keeps =
    { $count ->
        [one] { $name } repariert: Der Feed findet { $count } Release, und die Regel behält „{ $term }“
       *[other] { $name } repariert: Der Feed findet { $count } Releases, und die Regel behält „{ $term }“
    }
downloads-sent-torrents =
    { $count ->
        [one] { $count } Torrent an { $client } gesendet.
       *[other] { $count } Torrents an { $client } gesendet.
    }
downloads-added-torrents =
    { $count ->
        [one] { $count } Torrent hinzugefügt.
       *[other] { $count } Torrents hinzugefügt.
    }
downloads-trashed-paths =
    { $count ->
        [one] { $count } Pfad von { $title } in den Papierkorb verschoben
       *[other] { $count } Pfade von { $title } in den Papierkorb verschoben
    }
downloads-show-gone = Diese Serie ist nicht mehr in der Bibliothek
downloads-activity-hide = Aktivität ausblenden
downloads-activity = Aktivität
downloads-moving =
    { $count ->
        [one] { $count } Download läuft
       *[other] { $count } Downloads laufen
    }
downloads-back = Zurück zu Downloads
downloads-chip-search-ready = Suche bereit
downloads-chip-search-starting = Suche startet
downloads-chip-search-off = Suche aus
downloads-chip-answers-at = { $summary } Er antwortet unter { $url }.
downloads-chip-helper-missing = Der Suchhelfer ist nicht installiert. Klicke für die Downloadeinstellungen, dort steht, wie du ihn hinzufügst
downloads-chip-helper-failed = Der Suchhelfer wurde beendet. Neu verbinden startet ihn wieder; die Downloadeinstellungen sagen, was schiefging
downloads-chip-open-settings = Downloadeinstellungen öffnen
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } aus
downloads-chip-client-not-connected = { $client } ist nicht verbunden. Verbinde neu oder prüfe seine Adresse in den Einstellungen
downloads-chip-client-remote = Ein Suchhelfer, den du selbst betreibst, nutzt den Client aus seinen eigenen Einstellungen
downloads-chip-client-switch = Klicken, um den Client zu wechseln
downloads-connecting = Verbinde
downloads-reconnect = Neu verbinden
downloads-search-placeholder = Suche eine Serie oder einen Release-Namen
downloads-search-go-tip = Release-Namen bei { $names } suchen
downloads-search-go-off = Jede Quelle ist ausgeschaltet.
downloads-suggest-airing =
    { $predicted ->
        [yes] Läuft, Folge { $episode } erwartet in { $time }
       *[no] Läuft, Folge { $episode } in { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] Läuft, Folgen { $episode } bis { $through } erwartet in { $time }
       *[no] Läuft, Folgen { $episode } bis { $through } in { $time }
    }
downloads-suggest-finished = Beendet
downloads-suggest-episodes = { $count } Folgen
downloads-suggest-in-library = In deiner Bibliothek
downloads-suggest-on-anilist = Auf AniList, nicht in deiner Bibliothek
downloads-plan-to-watch = Vormerken
downloads-suggest-release-search = Stattdessen Release-Namen nach „{ $query }“ bei { $names } suchen
downloads-suggest-release-search-none = Stattdessen Release-Namen nach „{ $query }“ bei keiner Quelle suchen
downloads-key-return = Enter
downloads-sources = Quellen
downloads-source-hint-starting = startet
downloads-source-hint-down = läuft nicht
downloads-source-tip-down = { $name } braucht einen Hilfsdienst, der nicht läuft. Klicken, um ihn zu starten.
downloads-source-tip-on = Suchen fragen { $name }. Klicken, um es auszuschalten.
downloads-source-tip-off = Ausgeschaltet. Klicken, um { $name } wieder zu fragen.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Prüfen
    .tip = Fragt, ob jede Quelle auf Abruf läuft. Startet nichts.
downloads-check-busy = Prüfe
downloads-back-to-grid = Zurück zum Raster
    .tip = Zurück zu { $title } und dem, was jeder Anbieter gefunden hat. Escape tut dasselbe.
downloads-releases-on =
    { $count ->
        [one] { $count } Release bei { $name }
       *[other] { $count } Releases bei { $name }
    }
downloads-found-on = Gefunden bei
downloads-found-searching = sucht
downloads-found-failed = fehlgeschlagen
downloads-found-cancelled = abgebrochen
downloads-releases =
    { $count ->
        [one] { $count } Release
       *[other] { $count } Releases
    }
downloads-found-retry-tip =
    { $why }
    Klicken, um es nochmal zu versuchen
downloads-facet-groups = Release-Gruppe
downloads-facet-quality = Qualität
downloads-facet-words = Wörter
downloads-filters-fewer = Weniger Filter
downloads-filters-more = Mehr Filter ({ $count })
downloads-narrow-placeholder = Eingrenzen mit Begriff, oder -Begriff zum Ausschließen
downloads-best-match = Beste Wahl
    .tip = Behält die Releases, die wie das beste aussehen: { $query }
downloads-best-match-none = Hier sticht kein Release heraus
downloads-strip-have-of = { $have } von { $total } Folgen auf der Platte
downloads-strip-have =
    { $count ->
        [one] { $count } Folge auf der Platte
       *[other] { $count } Folgen auf der Platte
    }
downloads-strip-next =
    { $predicted ->
        [yes] Folge { $episode } erwartet in { $time }
       *[no] Folge { $episode } in { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] Folgen { $episode } bis { $through } erwartet in { $time }
       *[no] Folgen { $episode } bis { $through } in { $time }
    }
downloads-strip-complete = vollständig
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = Keins dieser Releases ist für eine Folge, die du schon hast.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } Release hier ist die Datei, die du hast, markiert mit Auf der Platte
       *[other] { $count } Releases hier sind die Datei, die du hast, markiert mit Auf der Platte
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } Release ist für eine Folge, die du aus einem anderen Release hast
       *[other] { $count } Releases sind für eine Folge, die du aus einem anderen Release hast
    }
downloads-strip-both = { $first }; { $second }.
downloads-tick-missing =
    { $count ->
        [one] Die { $count } fehlende Folge ankreuzen
       *[other] Die { $count } fehlenden Folgen ankreuzen
    }
    .tip = Für jede Folge hier, die der Bibliothek fehlt, das Release mit den meisten Seedern
downloads-open-in-library = In der Bibliothek öffnen
downloads-not-this-show = Nicht diese Serie
downloads-no-releases = Keine Releases
downloads-no-releases-body = Nichts bei { $name } passt zu „{ $query }“. Eine andere Quelle oder weniger Wörter finden es vielleicht.
downloads-try-source = { $name } probieren
downloads-ticked =
    { $count ->
        [one] { $count } Release angekreuzt
       *[other] { $count } Releases angekreuzt
    }
downloads-nothing-ticked = Nichts angekreuzt
downloads-ticked-size = Zusammen { $size }.
downloads-tick-hint = Kreuze Releases in der Tabelle an, oder lade alle.
downloads-tick-all = Alle ankreuzen
downloads-working = Arbeite
downloads-download-ticked =
    { $count ->
        [one] { $count } Release laden
       *[other] { $count } Releases laden
    }
downloads-download-shown = Die { $count } gezeigten laden
downloads-download-all = Alle { $count } laden
downloads-download-movie = Als Film laden
    .tip = Ein angekreuztes Release, direkt in den Filmordner
downloads-subscribe = Abonnieren
downloads-subscribe-body = Neue Releases, die zu dieser Suche passen, laden, sobald sie erscheinen, in den hier genannten Ordner.
downloads-name-placeholder = Aus dem ersten Release übernommen
downloads-subscribe-search = Diese Suche abonnieren
downloads-subscribe-no-feed = Hier kein Feed zum Abonnieren
downloads-preview-again = Regel nochmal zeigen
downloads-preview-show = Die erzeugte Regel zeigen
downloads-preview-from = Von { $host }
downloads-preview-from-named = Von { $host } ({ $name })
downloads-preview-must = Muss { $terms } enthalten
downloads-preview-must-not = Darf { $terms } nicht enthalten
downloads-helper-changed = Downloads ist auf einen Helfer gestellt, der nicht antwortet
downloads-helper-missing = Diese Kopie von AniBeam kam ohne Downloads
downloads-helper-failed = Downloads konnte nicht starten
downloads-helper-changed-body = Suchen und Downloads laufen über ein kleines Hilfsprogramm. Die Einstellungen zeigen auf ein anderes als das von AniBeam, und es antwortet nicht. Stell zurück, dann startet es von selbst.
downloads-helper-missing-body = Sie wurde ohne das Hilfsprogramm gebaut, über das Suchen und Downloads laufen. Die Installer auf der Release-Seite haben es. Einstellungen, Downloads sagt, wie du es dieser Kopie hinzufügst.
downloads-helper-failed-body = Das Hilfsprogramm für Suchen und Downloads wurde beendet, bevor es bereit war. Ein Neustart behebt das meistens.
downloads-helper-use-own = Das von AniBeam nutzen
downloads-helper-start-again = Neu starten
downloads-helper-show-how = Zeig mir wie
downloads-helper-what-went-wrong = Was schiefging
downloads-lens-gaps-count = Lücken in deiner Bibliothek  { $count }
downloads-lens-airing-count = Läuft  { $count }
downloads-lens-following-count = Abos  { $count }
downloads-lens-downloaded-count = Geladen  { $count }
downloads-lens-gaps = Lücken in deiner Bibliothek
downloads-lens-gaps-line = Ausgestrahlte Folgen, die nicht auf der Platte sind. Öffne eine Serie, um zu sehen, wer sie hat.
downloads-lens-gaps-none = Nichts fehlt: Jede ausgestrahlte Folge jeder Serie ist auf der Platte.
downloads-lens-airing-line = Serien in deiner Bibliothek, von denen noch Folgen kommen. Öffne eine, um sie zu abonnieren.
downloads-lens-airing-none = Nichts in deiner Bibliothek läuft gerade
downloads-lens-airing-none-body = Eine Serie landet hier, sobald sie in deiner Bibliothek ist und noch Folgen kommen. Läuft jetzt listet alles, was gerade läuft.
downloads-airing-now = Läuft jetzt
downloads-lens-following = Abos
downloads-card-next =
    { $predicted ->
        [yes] Folge { $episode } erwartet in { $time }
       *[no] Folge { $episode } in { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Folgen { $episode } bis { $through } erwartet in { $time }
       *[no] Folgen { $episode } bis { $through } in { $time }
    }
downloads-rules-unread = Deine Abos konnten nicht gelesen werden.
downloads-lens-following-line = Feeds, die AniBeam beobachtet, nach den Serien, die sie füllen.
downloads-none-yet = Noch keine.
downloads-reading = Lese.
downloads-rules-error = AniBeam konnte deine Abos nicht lesen. Versuch es gleich nochmal.
downloads-removing-to-go = Entferne, noch { $count }
downloads-unsubscribe-all = Alle { $count } Abos beenden
downloads-unsubscribe = Abo beenden
downloads-no-rules = Keine Abos
downloads-no-rules-body = Suche oben und abonniere, damit eine Serie weiter kommt.
downloads-search = Suchen
downloads-lens-downloaded = Geladen
downloads-dl-line = { $size } auf der Platte. { $rest }
downloads-dl-client-off = { $client } ist nicht verbunden.
downloads-dl-unread = Die Downloadliste konnte nicht gelesen werden.
downloads-dl-in-client =
    { $count ->
        [one] { $count } Torrent, { $size } in { $client }.
       *[other] { $count } Torrents, { $size } in { $client }.
    }
downloads-usage-watched = Geschaut
downloads-usage-not-watched = Nicht geschaut
downloads-deleting = Lösche
downloads-delete-watched = Geschautes löschen
downloads-remove-all = Alle { $count } entfernen
downloads-on-disk = Auf der Platte
downloads-on-disk-line = In deiner Bibliothek ohne Torrent in { $client }, die größten zuerst. Rechtsklick auf eine, um sie zu löschen.
downloads-nothing-downloaded = Nichts geladen
downloads-nothing-downloaded-body = Was du lädst oder abonnierst, erscheint hier, eine Serie nach der anderen.
downloads-client-off-title = { $client } ist nicht verbunden
downloads-client-off-body = Prüfe, ob es läuft, und verbinde dann neu. Adresse und Passwort stehen in den Einstellungen unter Downloads.
downloads-connect-again = Neu verbinden
downloads-view-grid = Raster
downloads-view-list = Liste
downloads-chip-subscribed = Abonniert
downloads-speed-up = { $speed }/s hoch
downloads-broken-some = { $count } mit fehlenden Dateien
downloads-tag-watched = Geschaut

## PlayerDelays.qml

player-delays-title = Versatz
player-delays-subtitles = Untertitel
player-delays-audio = Audio
player-delays-subtitles-earlier = Untertitel 0,1 s früher
player-delays-subtitles-later = Untertitel 0,1 s später
player-delays-audio-earlier = Audio 0,1 s früher
player-delays-audio-later = Audio 0,1 s später
player-delays-reset = Bis zur nächsten Datei auf 0 setzen
player-delays-none = Kein Versatz

## TrackPicker.qml

track-picker-off = Aus

## RollControls.qml

roll-controls-window = { $ahead } bereit: Folgen { $next } bis { $through } von { $total }
roll-controls-window-watched = { $ahead } bereit: Folgen { $next } bis { $through } von { $total }, { $watched } geschaut
roll-controls-more = { $count } mehr laden
roll-controls-more-tip = Die nächsten Folgen jetzt laden. AniBeam hält wieder ein paar bereit, sobald du aufholst.
roll-controls-fewer-tip = Eine weniger vorhalten
roll-controls-ahead = { $count } voraus
roll-controls-further-tip = Eine mehr vorhalten
roll-controls-all = Alle laden
roll-controls-all-tip = Nicht mehr nur ein paar bereithalten, sondern jede restliche Folge laden
roll-controls-all-question = Alle { $count } restlichen Folgen laden?

## PlayerChrome.qml

player-chrome-delays = Untertitel- und Audioversatz
player-chrome-audio = Tonspur
player-chrome-subtitles = Untertitel
player-chrome-subtitles-off = Untertitel aus
player-chrome-casting-to = Sende an { $device }
player-chrome-cast = An einen TV senden
player-chrome-mark-watched = Als geschaut markieren
player-chrome-shortcuts = Tastenkürzel
player-chrome-fullscreen = Vollbild
player-chrome-exit-fullscreen = Vollbild beenden
player-chrome-stay = Bleiben
player-chrome-skip-intro = Intro überspringen
player-chrome-skip-outro = Abspann überspringen
player-chrome-playing-on = Läuft auf { $device }
player-chrome-replay = Nochmal

## PlayerPage.qml

player-title = Player
player-next = Weiter
player-extra-no-next = Ein Extra hat keine nächste Folge
player-last-episode = Das ist die letzte Folge
player-extra-no-previous = Ein Extra hat keine vorige Folge
player-first-episode = Das ist die erste Folge
player-resumed-at = Weiter bei { $time }
player-start-over = Von vorn
player-audio-track = Audio: { $track }
player-subtitles-track = Untertitel: { $track }
player-subtitles-on = Untertitel: an
player-subtitles-off = Untertitel aus
player-no-subtitles = Diese Datei hat keine Untertitel
player-delay-seconds = { $value } s
player-audio-delay = Audioversatz { $value }
player-subtitle-delay = Untertitelversatz { $value }
player-volume = Lautstärke { $volume }
player-speed = Tempo { $speed }×
player-not-while-casting = Nicht beim Senden
player-hold-fast = 2× beim Halten
player-no-chapters = Diese Datei hat keine Kapitel
player-last-chapter = Das ist das letzte Kapitel
player-chapter = Kapitel { $number }
player-chapter-titled = Kapitel { $number }: { $title }
player-screenshot-saved = Gespeichert in Pictures/AniBeam
player-skipped-intro = Intro übersprungen
player-skipped-outro = Abspann übersprungen
player-undo = Rückgängig
player-refusal-hidden = die Serie ist ausgeblendet
player-refusal-no-match = die Serie ist noch nicht zugeordnet; ordne sie unter Serieninfo zu
player-refusal-not-newer = deine Liste ist schon weiter als diese Folge
player-refusal-extra = das ist ein Extra
player-refusal-unmatched = die Datei ist keiner Folge zugeordnet
player-refusal-on-disk = die Datei liegt nur auf der Platte
player-not-tracked-because = Nicht erfasst: { $reason }.
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = { $tracker } konnte nicht aktualisiert werden
player-outcome-ok = { $tracker } ok
player-outcome-at = { $tracker } bei { $progress }
player-outcome-tracked = Erfasst
player-outcome-not-tracked = Nicht erfasst
player-outcome-could-not-update = Liste konnte nicht aktualisiert werden
player-outcome-tracked-list = Erfasst: { $list }
player-outcome-not-tracked-list = Nicht erfasst: { $list }
player-outcome-failed-list = Liste konnte nicht aktualisiert werden: { $list }
player-rated = Bewertet mit { $score }
player-score-failed = Wertung konnte nicht gespeichert werden
player-frame = Bild { $number } bei { $time }
player-audio-title = Audio
player-subtitles-title = Untertitel
player-cast-to = Senden an
player-cast-looking = Suche nach Chromecasts und TVs
player-cast-unreachable = TV nicht erreichbar
player-cast-tv = TV
player-cast-stop = Senden an { $device } beenden
player-cast-connecting = Verbinde
player-cast-loading = Lädt
player-cast-playing = Läuft
player-cast-paused = Pausiert
player-cast-finished = Beendet
player-cast-stopped = Gestoppt
player-rate-prompt = Die letzte Folge ist erfasst. Diese Serie bewerten?
player-rate = Bewerten
player-not-now = Nicht jetzt

## EndpointReposPanel.qml

endpoint-repos-panel-title = Quellenlisten
endpoint-repos-panel-helper = Eine Quellenliste ist eine signierte Liste von Suchorten, per Adresse hinzugefügt. AniBeam prüft die Signatur und zeigt den Schlüssel vor dem Speichern, dann installierst du, was du willst. Sie ergänzen die Suchquellen oben.
endpoint-repos-panel-could-not-start = AniBeam konnte das nicht starten. Versuch es gleich nochmal.
endpoint-repos-panel-needs-value = { $name } braucht einen Wert
endpoint-repos-panel-values-of = Werte von { $name }
endpoint-repos-panel-secret-note = Liegt im Geheimnisspeicher dieses Rechners, nie in der Bibliothek oder einem Export.
endpoint-repos-panel-secret-note-keep = Liegt im Geheimnisspeicher dieses Rechners, nie in der Bibliothek oder einem Export. Ein leeres Feld behält, was gespeichert ist.
endpoint-repos-panel-asks-again = { $name } Version { $version } fragt erneut
endpoint-repos-panel-update-sends-to = Die neue Version sendet das an { $host }.
endpoint-repos-panel-fields-changed = Seine Felder haben sich geändert.
endpoint-repos-panel-needs-fields = Braucht { $fields }, bevor es sucht.
endpoint-repos-panel-no-longer-offered = Von seiner Quellenliste nicht mehr angeboten.
endpoint-repos-panel-update-requests-to = Das Update sendet Anfragen an { $host }.
endpoint-repos-panel-unreadable = Diese Liste war nicht lesbar. Listen aktualisieren versucht es erneut.
endpoint-repos-panel-add-list = Quellenliste hinzufügen
endpoint-repos-panel-add-list-helper = Die Adresse ihrer Indexdatei oder ein Pfad zu einer auf diesem Rechner.
endpoint-repos-panel-checking = Prüfe
endpoint-repos-panel-add = Hinzufügen
endpoint-repos-panel-trust-question = { $name } vertrauen?
endpoint-repos-panel-fingerprint = Schlüssel-Fingerabdruck  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    { $count } Quellen, signiert. Ihre Quellen bestimmen, wohin deine Suchen gehen; vertraue einer Liste nur, wenn du ihrem Absender vertraust.
endpoint-repos-panel-preview-trusted =
    { $url }
    { $count } Quellen, signiert, schon vertraut. Ihre Quellen bestimmen, wohin deine Suchen gehen; vertraue einer Liste nur, wenn du ihrem Absender vertraust.
endpoint-repos-panel-trust = Vertrauen
endpoint-repos-panel-saving = Speichere
endpoint-repos-panel-install = Installieren
endpoint-repos-panel-update = Aktualisieren
endpoint-repos-panel-added-by-you = von dir hinzugefügt
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Auf v{ $version } aktualisieren
endpoint-repos-panel-add-value = Wert eintragen
endpoint-repos-panel-change = { $fields } ändern
endpoint-repos-panel-asks-for = Fragt nach { $fields }.
endpoint-repos-panel-by-hand = Von Hand hinzufügen
endpoint-repos-panel-by-hand-helper = Für Seiten ohne Liste: jeder Feed mit {"{"}query{"}"} statt des Suchbegriffs, auch per Proxy. Ein Schlüssel in der Adresse liegt im Passwortspeicher des Systems.
endpoint-repos-panel-name = Name
endpoint-repos-panel-refreshing = Aktualisiere
endpoint-repos-panel-refresh = Listen aktualisieren

## ScheduleSheet.qml

schedule-episodes-range = Folgen { $first } bis { $last }
schedule-day-format = ddd, d. MMM yyyy
schedule-month-format = MMMM yyyy
schedule-tile-day-format = d. MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = Irgendwann { $year }
schedule-no-date-yet = Noch kein Datum
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = Kein Datum
schedule-tile-last = Letzte
schedule-tile-out = Da
schedule-ends-at = Endet mit Folge { $number }
schedule-dated-by-site = { $count } datiert von { $site }
schedule-dated-by-record = { $count } datiert vom Eintrag
schedule-set-by-you = { $count } von dir gesetzt
schedule-guessed-weekly = { $count } wöchentlich geschätzt
schedule-nothing-set = Noch nichts gesetzt
schedule-saved = Gespeichert
schedule-cleared = Sendeplan-Notizen gelöscht
schedule-title = Folgen und Termine
schedule-season = Staffel { $number }
schedule-ask-agent = { $agent } fragen
schedule-done-pasting = Einfügen fertig
schedule-paste-list = Liste einfügen
schedule-agent-looking = { $agent } sucht die Termine. Gespeichert wird erst, wenn du auf Speichern klickst.
schedule-agent-stopped = { $agent } hat aufgehört: { $error }
schedule-stop = Stoppen
schedule-paste-placeholder = Füge eine Release-Liste, eine Tabelle oder einen Post ein
schedule-paste-helper = Was AniBeam liest, erscheint bei den Folgen unten.
schedule-agent-hint = Mit Claude Code auf diesem Computer kann AniBeam es bitten, die Termine für dich zu suchen.
schedule-show-more = Mehr Folgen zeigen
schedule-pick-hint = Klicke auf eine Folge, um sie zu datieren. Umschalt-Klick wählt eine Reihe.
schedule-picked-one = Folge { $number }:
schedule-picked = { $count ->
    [one] { $count } Folge:
   *[other] { $count } Folgen:
}
schedule-date-placeholder = 7. Jan., März 2027, 2027
schedule-set-date = Datum setzen
schedule-weekly = Wöchentlich
schedule-not-last = Nicht die letzte
schedule-last-episode = Letzte Folge
schedule-clear-date = Datum löschen

## ShowPage.qml

show-title = Serie
show-runtime-each = je { $runtime }
show-add-to-list = Zur Liste
show-moving-to = Wechsel zu { $list }
show-moved-to = Gewechselt zu { $list }
show-relation-related = Verwandt
show-no-poster = Kein Poster
show-reading-anilist = Lese AniList
show-not-in-library = Nicht in deiner Bibliothek
show-get-episodes = Folgen laden
show-trailer = Trailer
show-your-score = Deine Wertung { $score }
show-rate-it = Bewerten
show-anilist-score = AniList { $score }
show-forecast = { $percent } % für dich
show-because-you-watched = Weil du geschaut hast
show-hidden-loved = { $count ->
    [one] Ein verstecktes Tag, das du liebst, steckt dahinter
   *[other] { $count } versteckte Tags, die du liebst, stecken dahinter
}
show-you-gave-it = Du hast { $score } gegeben
show-story = Handlung
show-genres-tags = Genres und Tags
show-studios = Studios  { $studios }
show-related = Verwandt
show-recommendations = Empfehlungen
show-from-anilist = Von AniList
show-reading-rest = Lese den Rest von AniList

## ConnectSheet.qml

connect-connected-to = Mit { $name } verbunden
connect-connect-to = { $name } verbinden
connect-did-not-finish = Die Anmeldung wurde nicht abgeschlossen
connect-finish-in-browser = Im Browser abschließen
connect-signed-in-anilist = Angemeldet als { $account }. Deine Liste steht auf der Seite Schaue ich.
connect-signed-in-mal = Angemeldet als { $account }. Folgen werden dort beim Schauen markiert.
connect-setup-lead = { $name } meldet AniBeam über eine eigene App von dir an. Das dauert einmalig etwa eine Minute.
connect-page-open = Die Anmeldeseite von { $name } ist in deinem Browser offen. Erlaube AniBeam dort, dann schließt sich das hier von selbst.
connect-page-opening = Öffne die Anmeldeseite von { $name } in deinem Browser.
connect-open-watching = Schaue ich öffnen
connect-done = Fertig
connect-open-again = Anmeldeseite nochmal öffnen
connect-secret-where = Es ist die lange Zeichenfolge unter "Client Secret" auf der Seite der App unter myanimelist.net/apiconfig.
connect-client-secret = Client-Secret
connect-save-try-again = Speichern und nochmal versuchen
connect-change-client-id = Client-ID ändern
connect-step-create-anilist = Den Client auf AniList anlegen
connect-step-create-mal = Die App auf MyAnimeList anlegen
connect-step-create-anilist-body = Öffne die Entwicklereinstellungen von AniList, leg einen neuen Client an und füge diese Adresse genau so in das Feld Redirect URL ein.
connect-step-create-mal-body = Öffne die API-Seite von MyAnimeList, leg eine ID mit App Type "other" an und füge diese Adresse genau so in das Feld App Redirect URL ein.
connect-open-developer-page = Entwicklerseite öffnen
connect-copy-address = Adresse kopieren
connect-step-paste-id = Die ID der App einfügen
connect-step-paste-id-anilist = AniList nennt sie Client ID: die Zahl auf der Seite deines neuen Clients.
connect-step-paste-id-mal = MyAnimeList nennt sie Client ID: die lange Zeichenfolge auf der Seite deiner neuen App.
connect-client-id = Client-ID
connect-client-secret-web = Client-Secret (nur für Web-Apps)
connect-step-connect-body = Der Browser öffnet sich, { $name } bittet dich, dich anzumelden und AniBeam zu erlauben, und AniBeam fängt die Antwort ab.

## SettingsPage

settings-title = Einstellungen
settings-search-placeholder = Einstellungen suchen
settings-section-library = Bibliothek
settings-section-accounts = Konten
settings-section-appearance = Darstellung
settings-section-playback = Wiedergabe
settings-section-subtitles = Untertitel
settings-section-keys = Tastatur
settings-section-downloads = Downloads
settings-section-phone = Handy-App
settings-section-storage = Speicher und Backup
settings-section-help = Hilfe
settings-section-about = Info
settings-section-developer = Entwickler
settings-status-library-empty = Noch keine Ordner
settings-status-library =
    { $count ->
        [one] { $count } Serie
       *[other] { $count } Serien
    } in { $folders ->
        [one] { $folders } Ordner
       *[other] { $folders } Ordnern
    }
settings-status-library-missing =
    { $gone ->
        [one] { $gone } Ordner
       *[other] { $gone } Ordner
    } nicht gefunden, { $count ->
        [one] { $count } Serie
       *[other] { $count } Serien
    } in { $folders ->
        [one] { $folders } Ordner
       *[other] { $folders } Ordnern
    }
settings-status-accounts-anilist = Sync mit AniList
settings-status-accounts-mal = Sync mit MyAnimeList
settings-status-accounts-both = Sync mit AniList und MyAnimeList
settings-status-accounts-none = Keine Liste verbunden
settings-status-appearance =
    { $mode ->
        [system] Folgt dem System
        [light] Hell
       *[dark] Dunkel
    }, { $source ->
        [theme] Designfarben
       *[desktop] Desktopfarben
    }
settings-status-playback =
    { $skip ->
        [both] Überspringt Intros und Abspänne
        [intro] Überspringt Intros
        [outro] Überspringt Abspänne
       *[none] Spielt alles durch
    }
settings-status-playback-audio =
    { $skip ->
        [both] Überspringt Intros und Abspänne
        [intro] Überspringt Intros
        [outro] Überspringt Abspänne
       *[none] Spielt alles durch
    }, Audio { $languages }
settings-status-subtitles =
    Größe { $percent } Prozent, { $override ->
        [AsScripted] gestaltete Dateien wie gemacht
        [Force] dein Stil auf jeder Datei
       *[ScaleOnly] gestaltete Dateien behalten ihren Look
    }
settings-status-keys = Jede Taste, auf die AniBeam hört
settings-status-downloads-unread = Noch nicht gelesen
settings-status-downloads-answered = qBittorrent hat geantwortet
settings-status-downloads-missing = Suchhelfer nicht installiert
settings-status-downloads-unconnected = Noch nicht verbunden
settings-status-phone-shared = In deinem Tailnet geteilt
settings-status-phone-off = Nicht geteilt
settings-status-storage = { $size } Bilder im Cache
settings-status-storage-unread = Bilder, Export und Import
settings-status-help = Antworten auf die häufigsten Fragen
settings-status-about-update = Version { $version } ist da
settings-status-developer = Misst jede Seite
settings-status-developer-last = { $page } gebaut in { $ms } ms
settings-results =
    { $count ->
        [one] { $count } Einstellung passt
       *[other] { $count } Einstellungen passen
    }
settings-results-none = Nichts passt
settings-results-hint = Enter öffnet die erste.
settings-results-none-hint = Versuch ein einfacheres Wort: Design, Untertitel, Tracker, Passwort, Ordner.

settings-find-folders =
    .words = ordner hinzufügen scan scannen neu entfernen quelle pfad sammlung
settings-find-matching =
    .words = metadaten serieninfo anbieter anilist tvmaze reihenfolge zuordnung zuordnen
settings-find-hidden-shows =
    .words = ausgeblendet versteckt einblenden zeigen
settings-find-hidden-airing =
    .words = kein interesse uninteressant läuft wieder zeigen
settings-find-calendar =
    .words = kalender ics ical sendeplan termine abonnieren
settings-find-anime-lists =
    .words = anilist myanimelist mal anmelden verbinden login konto trennen tracker
settings-find-main-list =
    .words = fortschritt karten haupt tracker primär
settings-find-sync-now =
    .words = aktualisieren sync fortschritt listen tracker
settings-find-on-hold =
    .words = pausiert pause inaktiv unberührt tage
settings-find-language =
    .words = sprache übersetzung locale deutsch englisch
settings-find-mode =
    .words = dunkel hell system design nacht modus
settings-find-colours-from =
    .words = terminal desktop system design quelle kitty farben
settings-find-dark-theme =
    .words = farbschema farben base16 dunkel
settings-find-light-theme =
    .words = farbschema farben base16 hell
settings-find-accent =
    .words = farbe akzent hervorhebung
settings-find-text-size =
    .words = schrift text wörter größer kleiner zoom skalierung lesen barrierefreiheit
settings-find-density =
    .words = kompakt bequem abstand dichte größe
settings-find-poster-size =
    .words = karten klein mittel groß poster
settings-find-out-now =
    .words = läuft jetzt da karten liste zeilen stil banner poster
settings-find-out-now-cards =
    .words = läuft jetzt da raster spalten zeilen layout
settings-find-reduce-motion =
    .words = animation bewegung barrierefreiheit
settings-find-app-icon =
    .words = logo symbol launcher dock taskleiste fenster dunkel
settings-find-skip-intros =
    .words = opening op intro automatisch überspringen aniskip kapitel
settings-find-skip-outros =
    .words = ending ed abspann credits automatisch überspringen aniskip
settings-find-mpv-settings =
    .words = mpv mpv.conf konfiguration erweitert
settings-find-discord =
    .words = discord rich presence status profil
settings-find-next-episode =
    .words = nächste folge knopf titel autoplay ende
settings-find-subtitle-languages =
    .words = sprache spur untertitel subs
settings-find-audio-languages =
    .words = sprache tonspur audio dub synchro japanisch
settings-find-start-over =
    .words = zurücksetzen standard untertitel stil
settings-find-size =
    .words = größe skalierung untertitel
settings-find-position =
    .words = höhe rand position untertitel
settings-find-styled =
    .words = ass gestaltet überschreiben erzwingen
settings-find-font =
    .words = schrift schriftart
settings-find-text-colour =
    .words = farbe textfarbe
settings-find-outline =
    .words = umriss rand kontur
settings-find-background =
    .words = box hintergrund deckkraft
settings-find-bold =
    .words = fett gewicht
settings-find-keys-player =
    .words = tastatur tastenkürzel tasten shortcuts hotkeys
settings-find-keys-player-around =
    .words = tastatur tastenkürzel vollbild nächste vorige folge zurück
settings-find-keys-around =
    .words = tastatur tastenkürzel zurück vor rechtsklick menü vollbild
settings-find-keys-anywhere =
    .words = tastatur tastenkürzel tasten hotkeys einstellungen beenden
settings-find-torrent-client =
    .words = qbittorrent passwort benutzername adresse webui seedbox
settings-find-search-helper =
    .words = anirss rss feeds token befehl server erweitert
settings-find-next-few =
    .words = rollend folgen voraus löschen geschaut behalten platte speicherplatz
settings-find-unsubscribe =
    .words = abo beenden aufräumen feeds beendet
settings-find-last-search =
    .words = downloads suche merken
settings-find-search-sources =
    .words = endpoints feeds konfiguration anirss quellen
settings-find-source-lists =
    .words = repo repos endpoint signiert endpoints jackett prowlarr quellenliste
settings-find-share-tailscale =
    .words = handy android tailnet tailscale fernzugriff host teilen
settings-find-stream =
    .words = stream streamen abspielen senden folgen handy dateien
settings-find-devices =
    .words = vertrauen vertraut ausschalten sperren gerät tailscale
settings-find-back-up =
    .words = export backup sichern datei json
settings-find-restore =
    .words = import wiederherstellen datei json
settings-find-image-cache =
    .words = poster leeren bilder platte cache speicher
settings-find-files =
    .words = datenbank daten konfiguration cache ordner pfad
settings-find-help-folders =
    .words = hilfe ordner aufbau benennung folgen staffeln filme dateien struktur
settings-find-help-recognised =
    .words = hilfe zuordnung falsch serie nicht gefunden nicht zugeordnet erkannt
settings-find-help-anilist =
    .words = hilfe anilist myanimelist mal anmelden verbinden login tracker
settings-find-help-downloads =
    .words = hilfe downloads leer quellen nichts torrent qbittorrent
settings-find-help-data =
    .words = hilfe daten dateien backup datenbank datenschutz
settings-find-help-else =
    .words = hilfe einrichtung onboarding anleitung melden fehler problem issue
settings-find-version =
    .words = build neuerungen changelog version
settings-find-setup =
    .words = erster start anleitung onboarding einrichtung erneut
settings-find-updates =
    .words = stabil stable nightly kanal prüfen update
settings-find-developer-options =
    .words = debug messung entwickler
settings-find-ratings =
    .words = imdb credits bewertungen

## SettingsDownloadsTab

settings-downloads-title = Downloads
settings-downloads-lead = Wohin AniBeam schickt, was du lädst, und wie es die Release-Feeds durchsucht.
settings-downloads-keep-none = Eine Folge zu beenden löscht die davor.
settings-downloads-keep-line = Geschaute Folgen, die hinter der beendeten bleiben. Bei { $count } löscht das Beenden von Folge { $finishing } Folge 1.
settings-downloads-anirss-ready = Läuft, Version { $version } unter { $url }.
settings-downloads-anirss-starting = Starte den Suchhelfer.
settings-downloads-anirss-missing = Der Suchhelfer ist nicht installiert. Installiere AniBeam neu, oder trag unter Befehl dein eigenes anirss ein.
settings-downloads-anirss-failed = Der Suchhelfer wurde beendet. Nochmal startet ihn; Details zeigt, was er gemeldet hat.
settings-downloads-anirss-idle = Noch nicht gestartet. Er startet bei der ersten Suche auf der Seite Downloads.
settings-downloads-torrent-trying = Probiere { $client }.
settings-downloads-torrent-answered = { $client } { $version } hat geantwortet.
settings-downloads-torrent-failed = { $client } hat sich nicht verbunden. Prüfe Adresse und Passwort oben und teste erneut.
settings-downloads-torrent-untried = Noch nicht probiert.
settings-downloads-mode-existing = Ein qBittorrent, das du selbst betreibst, mit eingeschaltetem Web UI.
settings-downloads-mode-managed = AniBeam betreibt ein eigenes qBittorrent, das nach ~/Downloads/Anime lädt. Es läuft weiter, wenn AniBeam beendet wird, also laden Abos weiter.
settings-downloads-mode-auto = qBittorrent an der Adresse unten, sonst ein eigenes.
settings-downloads-password-saved = Für diesen Client ist hier ein Passwort gespeichert.
settings-downloads-password-anirss = Nutzt das Passwort, das der Suchhelfer gespeichert hat; speichere hier eins, um nicht davon abzuhängen.
settings-downloads-password-none = Kein Passwort gespeichert. Ein Client ohne Login hier braucht keins.
settings-downloads-mismatch = Der Suchhelfer spricht mit { $helperUrl }, AniBeam aber mit { $url }. Ein Suchhelfer, den du selbst betreibst, liest die Adresse aus seiner eigenen config.toml, also verschiebt ein Clientwechsel hier ihn nicht.
settings-downloads-torrent-title = Torrent-Client
settings-downloads-torrent-helper = Das qBittorrent, das lädt. Sein Passwort liegt im Passwortspeicher des Systems.
settings-downloads-which-client = Welcher Client
settings-downloads-pick-for-me = Für mich wählen
settings-downloads-my-own = Mein eigener
settings-downloads-built-in = Eingebaut
settings-downloads-saved-clients = Gespeicherte Clients
settings-downloads-saved-clients-helper = Zum Beispiel eine Seedbox und der auf diesem Rechner. Der Client-Chip auf der Seite Downloads wechselt auch zwischen ihnen.
settings-downloads-add-client = Hinzufügen
settings-downloads-name = Name
settings-downloads-name-placeholder = Nach seiner Adresse benannt
settings-downloads-address = Adresse
settings-downloads-address-managed-helper = Nur dieser Rechner erreicht ihn, und er braucht kein Passwort.
settings-downloads-address-helper = Die Adresse des Web UI.
settings-downloads-username = Benutzername
settings-downloads-password = Passwort
settings-downloads-secret-saved = Gespeichert
settings-downloads-secret-unset = Nicht gesetzt
settings-downloads-connection = Verbindung
settings-downloads-open-config = Konfiguration öffnen
settings-downloads-details = Details
settings-downloads-details-torrent = Das hat { $client } gemeldet, als AniBeam es probiert hat.
settings-downloads-test-trying = Probiere
settings-downloads-test-save = Speichern und testen
settings-downloads-test = Testen
settings-downloads-searching = Suche
settings-downloads-search-helper = Suchhelfer
settings-downloads-searching-helper = AniBeam durchsucht deine Quellen selbst und schickt, was du wählst, an den Torrent-Client. Der Suchhelfer wird nicht gebraucht.
settings-downloads-search-helper-helper = Der Teil von AniBeam, der sucht und abonniert, ein Programm namens anirss. AniBeam startet es für dich; ändere das nur, wenn du ein eigenes betreibst.
settings-downloads-runs-in = Suchen laufen in
settings-downloads-runs-in-native-helper = Abos gehen an den Torrent-Client, so kommen neue Folgen auch, wenn AniBeam zu ist.
settings-downloads-runs-in-helper = Dem Suchhelfer, einem eigenen Programm, das AniBeam oder du selbst startest.
settings-downloads-engine-native = AniBeam selbst
settings-downloads-how-runs = Wie er läuft
settings-downloads-how-runs-sidecar = AniBeam startet ihn beim Öffnen und stoppt ihn beim Beenden.
settings-downloads-how-runs-remote = AniBeam nutzt einen, den du schon betreibst, unter der Adresse und mit dem Schlüssel unten.
settings-downloads-runs-sidecar = AniBeam startet ihn
settings-downloads-runs-remote = Ich starte ihn
settings-downloads-command = Befehl
settings-downloads-command-helper = Ein Name im PATH oder ein voller Pfad. ~/.local/bin wird auch durchsucht.
settings-downloads-server-address-helper = Die Basis des Servers, ohne /api/v1.
settings-downloads-key = Schlüssel
settings-downloads-key-saved-helper = Ein Schlüssel ist gespeichert. Gib einen neuen ein, um ihn zu ersetzen, oder leere das Feld, um ihn zu entfernen.
settings-downloads-key-helper = Der Schlüssel, den der Suchhelfer beim Start gezeigt hat. Er lässt AniBeam herein.
settings-downloads-details-anirss = Das hat der Suchhelfer gemeldet, als er beendet wurde.
settings-downloads-starting = Startet
settings-downloads-reconnect = Neu verbinden
settings-downloads-save-first = Speichere zuerst deine Änderungen
settings-downloads-roll-title = Die nächsten bereithalten
settings-downloads-roll-helper = Eine lange Serie lädt ein paar Folgen auf einmal; jede beendete startet die nächste. Nie muss alles auf die Platte.
settings-downloads-roll-big = Für lange Serien
settings-downloads-roll-big-helper = Serien über { $count } Folgen laden von selbst ein paar auf einmal. Ihre Zeile unter Downloads kann trotzdem alles sofort holen.
settings-downloads-roll-ahead = Folgen voraus
settings-downloads-roll-ahead-helper = Wie viele nach der aktuellen geladen bleiben. 12 ist etwa eine Staffel.
settings-downloads-unit-episodes = Folgen
settings-downloads-roll-over = Lang heißt über
settings-downloads-roll-over-helper = Downloads mit mehr Folgen als das kommen in Häppchen.
settings-downloads-roll-watched = Geschaute Folgen
settings-downloads-roll-watched-delete-helper = Gelöscht, sobald du weiter bist, also hält die Platte nur, was noch zu schauen ist.
settings-downloads-roll-watched-keep-helper = Jede Datei bleibt nach dem Schauen auf der Platte.
settings-downloads-keep-all = Alle behalten
settings-downloads-keep-behind = Zurück behalten
settings-downloads-subscriptions-title = Abos und Suchen
settings-downloads-last-search = Letzte Suche behalten
settings-downloads-last-search-helper = Die Seite Downloads öffnet mit deiner letzten Suche. Aus, öffnet sie leer.
settings-downloads-prune = Abos beendeter Serien beenden
settings-downloads-prune-helper = 16 Tage nach der letzten Folge einer Serie fällt ihr Feed weg. Die Serie und ihre Dateien bleiben.
settings-downloads-sources-title = Suchquellen
settings-downloads-sources-helper = Wohin eine Suche geht, in Probierreihenfolge. Findet eine nichts, kommt die nächste.
settings-downloads-sources-empty = Noch keine Quellen eingerichtet
settings-downloads-sources-connect = Verbinde den Suchhelfer, um sie aufzulisten
settings-downloads-sources-empty-helper = Füge unten eine Liste hinzu, um Quellen zu sehen.
settings-downloads-config-file = Einstellungsdatei des Suchhelfers
settings-downloads-config-native-helper = Enthält die Speicherordner, die Tags und die Regeln für die beste Wahl, denen AniBeam selbst folgt, wenn es den Helfer startet. Seine eigenen Quellen gehören nur dem Helfer.
settings-downloads-config-helper = Enthält die Quellen, die Speicherordner, die Tags und die Regeln für die beste Wahl.
settings-downloads-unsaved = Ungespeicherte Änderungen
settings-downloads-unsaved-helper = Speichern startet den Suchhelfer mit den neuen Einstellungen neu.
settings-downloads-discard = Verwerfen

## SettingsDeveloperTab

settings-developer-title = Entwickler
settings-developer-lead = Warum eine Seite langsam ist, gemessen. Jeder Schalter gilt für die nächste Seite, die du öffnest.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds } s
settings-developer-switches = Schalter
settings-developer-load-images = Bilder laden
settings-developer-load-images-helper = Aus wird kein Poster, Banner oder Porträt gelesen oder dekodiert.
settings-developer-overlay = Overlay für Seitenladen
settings-developer-overlay-helper = Das letzte Seitenladen und der Speicher, unten rechts im Fenster.
settings-developer-read-cache = Bibliotheksabfragen behalten
settings-developer-read-cache-helper = Die Antwort auf eine Abfrage der ganzen Bibliothek bleibt, bis sich die Bibliothek ändert. Aus fragt jede Seite den Kern neu.
settings-developer-poster-pool = Poster im Hintergrund vorladen
settings-developer-poster-pool-helper = Dekodiert die 60 Poster, die du am ehesten siehst, und hält sie im Speicher.
settings-developer-options = Entwickleroptionen
settings-developer-options-helper = Schaltet diesen Tab, das Overlay und das Messen aus.
settings-developer-memory = Speicher
settings-developer-memory-helper = Was der Kernel für diesen Prozess zählt. Bilder halten dekodierte Pixel im Speicher und auf der Grafikkarte.
settings-developer-memory-unknown = Dieses System sagt nicht, wie viel Speicher die App nutzt.
settings-developer-in-use = Belegt
settings-developer-peak = Spitze
settings-developer-heap = Heap
settings-developer-files-mapped = Gemappte Dateien
settings-developer-threads = Threads
settings-developer-give-back = Speicher freigeben
settings-developer-give-back-helper = Verwirft, was der Szenengraph neu bauen kann, sammelt JavaScript-Müll und gibt freien Heap ans System zurück. Was dabei frei wird, hat die App gehalten, ohne es zu brauchen.
settings-developer-released = { $before } auf { $after }
settings-developer-free-now = Jetzt freigeben
settings-developer-images-total = Bilder seit dem Start
settings-developer-images-total-line = { $decodes } dekodiert, { $pixels } Pixel aus { $disk } auf der Platte, { $ms } ms
settings-developer-loads = Seitenladen
settings-developer-loads-helper = Gebaut ist die Wartezeit nach einem Klick: Der Qt-Thread tut nichts anderes, bis die Seite existiert. Fertig ist, wenn ihr letztes Bild ankam.
settings-developer-touring = Messe, { $at } von { $steps }
settings-developer-tour = Jede Seite messen
settings-developer-copy-report = Bericht kopieren
settings-developer-col-page = Seite
settings-developer-col-built = Gebaut
settings-developer-col-painted = Gezeichnet
settings-developer-col-settled = Fertig
settings-developer-col-calls = Aufrufe, Daten
settings-developer-col-decodes = Dekodierungen, Zeit, Pixel
settings-developer-col-items = Elemente
settings-developer-col-offscreen = Bilder außer Sicht
settings-developer-col-memory = Speicher
settings-developer-load-no-images = { $page }, keine Bilder
settings-developer-offscreen-of = { $offscreen } von { $total }
settings-developer-loads-empty = Öffne eine beliebige Seite, und sie erscheint hier.
settings-developer-load-images-off = { $page }, Bilder aus
settings-developer-load-frames = { $frames } Frames gezeichnet beim Laden, die längste Lücke zwischen zweien { $gap }.
settings-developer-load-frames-timed-out = { $frames } Frames gezeichnet beim Laden, die längste Lücke zwischen zweien { $gap }. Nach 15 s nicht mehr auf die Bilder gewartet.
settings-developer-built = Gebaut
settings-developer-settled = Fertig
settings-developer-core-data = Daten vom Kern
settings-developer-pixels = Dekodierte Pixel
settings-developer-page-items = Elemente auf der Seite
settings-developer-cached-images = Bilder schon im Speicher
settings-developer-calls = Kernaufrufe, langsamste zuerst
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  gecacht
settings-developer-call = { $name }  { $ms } ms  { $size }  Kern { $core }, zu QML { $qml }
settings-developer-decodes = Langsamste Dekodierungen
settings-developer-decodes-line = { $count } Dekodierungen nacheinander im Bild-Thread, { $span } von der ersten bis zur letzten; { $requests } angefragt, { $cached } schon dekodiert, { $blocked } blockiert
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } auf { $w }x{ $h }  { $size }
settings-developer-decodes-none = Nichts dekodiert: Jedes Bild war schon im Speicher.
settings-developer-images-were-off = Bilder waren aus.
settings-developer-scene = Woraus die Seite besteht
settings-developer-scene-line = { $items } Elemente, { $offscreen } außerhalb des Fensters; { $texts } Texte, { $shapes } Formen, { $icons } Symbole; { $images } Bilder, davon { $imagesOffscreen } außerhalb des Fensters mit { $size }
settings-developer-copy-load = Diesen Ladevorgang kopieren
settings-developer-tour-title = Jede Seite, Bilder an und aus
settings-developer-tour-helper = Der mittlere von drei Besuchen jeder Seite. Wo beide Spalten gleich sind, machen nicht die Bilder die Seite langsam.
settings-developer-col-built-on = Gebaut, Bilder an
settings-developer-col-built-off = Gebaut, Bilder aus
settings-developer-col-settled-on = Fertig, an
settings-developer-col-settled-off = Fertig, aus
settings-developer-col-decodes-time = Dekodierungen, Zeit
settings-developer-col-data = Daten
settings-developer-tour-peak = Spitzenspeicher während der Messung { $size }
settings-developer-events = Ereignisse
settings-developer-events-helper = Alles, was der Kern sagt, jede Stufe, neueste zuerst. Jedes Ereignis wird als JSON in den Qt-Thread kopiert; die Größe ist, was das Senden gekostet hat.
settings-developer-event-rate = { $kind } { $count }/min, { $size }
settings-developer-events-filter = Nach Wörtern, Art oder Stufe filtern
settings-developer-event-plain = { $message }   (einfach: { $plain })
settings-developer-copy-events = Ereignisse kopieren

## SettingsKeysTab

settings-keys-title = Tastatur
settings-keys-lead = Jede Taste, auf die AniBeam hört. Drück überall ?, um die Tasten zu sehen, die dort gehen.
settings-keys-player = Im Player
settings-keys-player-around = Im Player bewegen
settings-keys-around = Bewegen
settings-keys-anywhere = Überall

## KeyHelp

key-help-title = Tastenkürzel
key-help-group-playback = Wiedergabe
key-help-group-navigation = Navigation
key-help-group-library = Bibliothek
key-help-key-space = Leertaste / K
key-help-key-left-right = Links / Rechts
key-help-key-ctrl-right = Strg+Rechts
key-help-key-backspace = Rücktaste
key-help-key-up-down = Hoch / Runter
key-help-key-hold = Video gedrückt halten
key-help-key-page = Bild auf / ab
key-help-key-home-end = Pos1 / Ende
key-help-key-escape = Esc
key-help-key-alt-left = Alt+Links
key-help-key-alt-right = Alt+Rechts
key-help-key-shift-f10 = Umschalt+F10
key-help-key-ctrl-k = Strg+K
key-help-key-search = / oder Strg+F
key-help-key-ctrl-space = Strg+Leertaste
key-help-key-shift-space = Umschalt+Leertaste
key-help-key-ctrl-a = Strg+A
key-help-key-ctrl-l = Strg+L
key-help-key-ctrl-comma = Strg+,
key-help-key-ctrl-q = Strg+Q
key-help-play-pause = Abspielen oder Pause
key-help-seek-5 = 5 s spulen
key-help-seek-10 = 10 s spulen
key-help-skip = Intro oder Abspann überspringen, sonst 90 s
key-help-frame-step = Ein Bild zurück oder vor
key-help-speed = Langsamer oder schneller, in Schritten von 0,25
key-help-normal-speed = Normales Tempo
key-help-volume = Lautstärke um 5 lauter oder leiser
key-help-mute = Stumm
key-help-subtitles-toggle = Untertitel aus und wieder an
key-help-subtitle-delay = Untertitel 100 ms früher oder später
key-help-audio-delay = Audio 100 ms früher oder später
key-help-hold-speed = 2× Tempo beim Halten
key-help-screenshot = Screenshot in Bilder speichern
key-help-episode = Nächste oder vorige Folge
key-help-chapter = Voriges oder nächstes Kapitel
key-help-start-end = Der Anfang oder der Abspann
key-help-fullscreen = Vollbild
key-help-escape-player = Schließt, was offen ist, sonst Vollbild beenden, sonst den Player verlassen
key-help-toggle-list = Diese Liste zeigen oder ausblenden
key-help-library-search = Bibliothek und Suche
key-help-search-page = Diese Seite durchsuchen
key-help-activity = Aktivität
key-help-settings = Einstellungen
key-help-quit = Beenden
key-help-forward = Vor
key-help-escape-app = Schließt, was offen ist, sonst zurück
key-help-context-menu = Das Rechtsklickmenü
key-help-pick-card = Eine Karte wählen, um mehrere zu bearbeiten
key-help-pick-range = Jede Karte bis zu dieser wählen
key-help-pick-all = Jede gezeigte Karte wählen

## SettingsAboutTab

settings-about-title = Info
settings-about-lead = Welches AniBeam das ist, und wie es vom nächsten erfährt.
settings-about-update-failed = Das Update wurde nicht abgeschlossen. Versuch es nochmal oder hol es von der Release-Seite.
settings-about-update-ready = { $version } ist bereit zur Installation.
settings-about-update-command = { $version } ist da. Aktualisiere mit { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = { $version } ist da. Diese Kopie ist aus dem Quellcode gebaut: pullen und neu bauen.
settings-about-update-elsewhere = { $version } ist da, ohne Download für diesen Computer.
settings-about-version = Version
settings-about-build = AniBeam { $build }
settings-about-whats-new = Neuerungen
settings-about-install = { $version } installieren
settings-about-setup = Einrichtung
settings-about-setup-helper = Die Einführung neuer Installationen: Ordner, Liste und Downloads.
settings-about-run-setup = Einrichtung erneut starten
settings-about-updates = Updates
settings-about-nightly-helper = Fast täglich ein neuer Build. Neuer und weniger getestet.
settings-about-stable-helper = Eine neue Version, sobald sie getestet und veröffentlicht ist.
settings-about-stable = Stabil
settings-about-nightly = Nightly
settings-about-check-now = Jetzt prüfen
settings-about-developer = Entwickleroptionen
settings-about-developer-helper = Fügt den Bereich Entwickler hinzu: misst jede Seite, zeigt Speicher und das ganze Protokoll, schaltet Bilder ab.
settings-about-credits = Credits
settings-about-ratings = Bewertungen
settings-about-ratings-helper = Information courtesy of IMDb (https://www.imdb.com). Used with permission.
settings-about-series = Serien und Folgen
settings-about-series-helper = AniList und TVmaze. Skip-Zeiten von AniSkip.

## SettingsPlaybackTab

settings-playback-title = Wiedergabe
settings-playback-lead = Was der Player von selbst tut, damit du nur eingreifst, wenn du willst.
settings-playback-skipping = Überspringen
settings-playback-skipping-helper = Nutzt Dateikapitel, sonst online geteilte Skip-Zeiten. Rückgängig im Player stoppt das Überspringen, bis AniBeam schließt.
settings-playback-skip-intros = Intros überspringen
settings-playback-skip-outros = Abspänne überspringen
settings-playback-skip-outros-helper = Beim Abspann wird die Folge auch als geschaut markiert.
settings-playback-next-episode = Nächste Folge
settings-playback-name-next = Nächste Folge benennen
settings-playback-name-next-helper = Der Knopf Weiter am Ende einer Folge zeigt den Titel der nächsten. Aus zeigt er nur ihre Nummer.
settings-playback-languages = Sprachen
settings-playback-languages-helper = Sprachcodes, die liebste zuerst. Die erste, die eine Datei hat, gewinnt. Eine Spur, die du im Player wählst, merkt sich die Serie.
settings-playback-subtitle-languages = Untertitelsprachen
settings-playback-audio-languages = Audiosprachen
settings-playback-discord = Auf Discord zeigen, was du schaust
settings-playback-discord-check-failed = AniBeam konnte Discord gerade nicht prüfen. Versuch es gleich nochmal.
settings-playback-discord-helper = Serie, Folge und Poster erscheinen beim Abspielen in deinem Discord-Profil. Gesendet wird nichts, solange das aus oder Discord geschlossen ist.
settings-playback-discord-unavailable = Dieser Build hat keine Discord-Anwendung.
settings-playback-discord-connected = Mit Discord verbunden
settings-playback-discord-not-connected = Nicht verbunden. AniBeam verbindet sich, während eine Folge läuft und Discord offen ist
settings-playback-advanced = Erweitert
settings-playback-my-mpv = Meine mpv-Einstellungen nutzen
settings-playback-my-mpv-helper = Liest deine mpv.conf. Optionen, die nur beim Start wirken, werden übersprungen, und Skripte laden nie.
settings-playback-no-mpv-conf = Keine mpv-Einstellungsdatei gefunden. Trag deine Optionen in mpv.conf ein und schalte das dann ein.

## SliderRow

slider-row-spoken-unit = { $name }, in { $unit }
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Akzent Rot
swatches-accent-orange = Akzent Orange
swatches-accent-yellow = Akzent Gelb
swatches-accent-green = Akzent Grün
swatches-accent-cyan = Akzent Cyan
swatches-accent-blue = Akzent Blau

## LookPreview

look-preview-dark = Dunkel
look-preview-light = Hell

## SettingsAccountsTab

settings-accounts-title = Konten
settings-accounts-lead = Verbinde AniList oder MyAnimeList, und jede Folge, die du beendest, wird dort für dich markiert. Zahlen gehen nur nach oben.
settings-accounts-syncing = Synce deine Listen
settings-accounts-synced = Listen gesynct
settings-accounts-lists = Deine Anime-Listen
settings-accounts-lists-helper = Ohne Liste merkt sich AniBeam trotzdem, was du auf diesem Computer geschaut hast. Eine Liste bei AniList oder MyAnimeList bringt es auf dein Handy und in dein Profil.
settings-accounts-progress = Fortschritt
settings-accounts-main-list = Hauptliste
settings-accounts-main-list-helper = Die Karten zeigen den Fortschritt aus dieser. Die andere bekommt trotzdem jedes Update.
settings-accounts-local-lists = Listen auf diesem Computer
settings-accounts-local-lists-helper =
    { $count ->
        [one] Eine Serie hat
       *[other] { $count } Serien haben
    } Status oder Wertung nur hier. Verschiebe sie zu { $tracker }, um sie auf deinem Handy und in deinem Profil zu sehen.
settings-accounts-review-move = Prüfen und verschieben
settings-accounts-nothing-to-move = Nichts mehr zu verschieben
settings-accounts-sync-now = Jetzt syncen
settings-accounts-sync-now-helper = Liest jede verbundene Liste neu. AniBeam tut das auch beim Start.
settings-accounts-sync = Sync
settings-accounts-move-on-hold = Auf Pausiert setzen
settings-accounts-move-on-hold-helper = Eine Serie auf Schaue ich, die du eine Weile nicht angerührt hast, wird auf deinen Listen auf Pausiert gesetzt. Abspielen holt sie zurück. Eine Serie, die auf ihre nächste Folge wartet, bleibt, wo sie ist.
settings-accounts-after = Nach
settings-accounts-after-line = Tage ohne gespielte Folge
settings-accounts-days-unit = Tage

## SettingsPhoneTab

settings-phone-title = Handy-App
settings-phone-lead = AniBeam auf deinem Handy kann diese Bibliothek abspielen, zeigen, was läuft, und deinen Downloads folgen, wo immer du bist.
settings-phone-online = Online
settings-phone-offline = Offline
settings-phone-untrusted = Bekommt nichts von diesem Computer
settings-phone-sharing = Teilen
settings-phone-sharing-helper = Nur deine eigenen Geräte in Tailscale erreichen es. Auf dem Handy geschaute Folgen behalten hier ihre Stelle und ihren Fortschritt.
settings-phone-share = Über Tailscale teilen
settings-phone-stream = Folgen streamen
settings-phone-stream-on = Deine Geräte können Folgen von hier abspielen und senden.
settings-phone-stream-off = Deine Geräte sehen die Bibliothek und was läuft, können aber nicht von hier abspielen.
settings-phone-host-service = Hostdienst
settings-phone-not-running = Läuft nicht
settings-phone-running = Läuft
settings-phone-service-off-helper = Führe den Befehl einmal aus, dann erreichen deine Geräte AniBeam ab der Anmeldung, ohne offenes Fenster.
settings-phone-service-on-helper = Deine Geräte erreichen AniBeam ohne offenes Fenster. Solange dieses Fenster offen ist, teilt es anstelle des Dienstes.
settings-phone-copy-command = Befehl kopieren
settings-phone-command-copied = Befehl kopiert
settings-phone-nobody = Bekommt nichts: Tailscale nennt keine Person dafür
settings-phone-people = Personen
settings-phone-own-profile = Profil dieses Computers
settings-phone-person-own-devices =
    { $count ->
        [0] Dieser Computer
        [one] Dieser Computer und { $count } Gerät
       *[other] Dieser Computer und { $count } Geräte
    }
settings-phone-person-devices =
    { $count ->
        [one] { $count } Gerät
       *[other] { $count } Geräte
    }
settings-phone-devices = Geräte in deinem Tailnet
settings-phone-devices-helper = Jedes Gerät im Tailnet erreicht diesen Computer, solange Teilen an ist. Ausgeschaltet bekommt es nichts mehr.
settings-phone-addresses = Adressen
settings-phone-addresses-helper = Gib eine davon in die Handy-App ein, wenn sie beim ersten Mal fragt.
settings-phone-copy = Kopieren
settings-phone-address-copied = Adresse kopiert

## SettingsHelpTab

settings-help-title = Hilfe
settings-help-lead = Kurze Antworten auf häufige Fragen, je mit direktem Weg zur Lösung.
settings-help-folders = Wie sollen meine Ordner aussehen?
settings-help-folders-helper = Ein Ordner pro Serie, die Folgen darin. Benenne jede Datei mit ihrer Folgennummer wie Releases: "Frieren - 01.mkv" oder "Frieren S01E01.mkv". Eine zweite Staffel ist ein eigener Ordner oder "Season 2" in der Serie. Filme kommen in den Ordner Movies, je eine Datei.
settings-help-for-example = Zum Beispiel
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = Noch kein Ordner hinzugefügt
settings-help-reads-folders =
    { $count ->
        [one] AniBeam liest einen Ordner
       *[other] AniBeam liest { $count } Ordner
    }
settings-help-add-folder-helper = Füge den Ordner hinzu, der deine Serien enthält, nicht jede Serie einzeln.
settings-help-add-folder = Ordner hinzufügen
settings-help-open-folders = Deine Ordner öffnen
settings-help-match = Warum wurde meine Serie nicht erkannt?
settings-help-match-helper = AniBeam sucht Ordner nach Namen, erst bei AniList, dann bei TVmaze. Extra-Wörter, Tippfehler oder ein fremdsprachiger Name gehen mal daneben. Wähle die Serie unter Serieninfo; deine Wahl bleibt immer.
settings-help-all-recognised = Jede Serie ist erkannt
settings-help-unrecognised =
    { $count ->
        [one] Eine Serie ist nicht erkannt
       *[other] { $count } Serien sind nicht erkannt
    }
settings-help-open-show-info = Serieninfo öffnen
settings-help-connect = Wie verbinde ich AniList?
settings-help-connect-helper = Verbinde es, und jede beendete Folge wird auf deiner Liste markiert, damit Handy und Profil mitkommen. AniList will einmal eine kleine eigene App; AniBeam führt dich durch, in etwa einer Minute. MyAnimeList geht genauso.
settings-help-both-connected = Beide sind verbunden
settings-help-anilist-connected = AniList ist verbunden
settings-help-mal-connected = MyAnimeList ist verbunden
settings-help-none-connected = Keins verbunden
settings-help-connect-anilist = AniList verbinden
settings-help-connect-mal = MyAnimeList verbinden
settings-help-downloads = Warum ist Downloads leer?
settings-help-downloads-helper = AniBeam bringt keine Suchorte mit. Füge unter Einstellungen, Downloads eine Quellenliste hinzu und installiere daraus, was du willst. Dazu braucht es einen Torrent-Client: AniBeam startet einen eigenen oder nutzt dein qBittorrent.
settings-help-sources = Quellen und der Torrent-Client
settings-help-open-downloads = Downloadeinstellungen öffnen
settings-help-data = Wo sind meine Daten?
settings-help-data-helper = Bibliothek, Verlauf und Einstellungen bleiben auf diesem Computer. AniList- und MyAnimeList-Anmeldungen liegen im Passwortspeicher des Systems, sonst in einer Datei nur für dich. Sichere alles unter Speicher und Backup in eine Datei.
settings-help-library-history = Bibliothek und Verlauf
settings-help-show-folder = Ordner zeigen
settings-help-backup = Sichern oder wiederherstellen
settings-help-open-storage = Speicher und Backup öffnen
settings-help-more = Etwas anderes?
settings-help-more-helper = Starte die erste Einrichtung erneut, oder sag uns, was schiefging. Ein Screenshot hilft sehr.
settings-help-first-setup = Die erste Einrichtung
settings-help-first-setup-helper = Ordner, Liste und Downloads.
settings-help-run-setup = Einrichtung erneut starten
settings-help-report = Problem melden
settings-help-report-helper = Öffnet einen Fehlerbericht auf GitHub in deinem Browser, mit dieser Version schon eingetragen.

## SettingsSubtitlesTab

settings-subtitles-title = Untertitel
settings-subtitles-lead = Womit jede Folge startet. Änderungen erscheinen sofort im Bild unten.
settings-subtitles-reset-done = Untertitel sehen wieder aus wie am Anfang
settings-subtitles-size-place = Größe und Position
settings-subtitles-size = Größe
settings-subtitles-position = Position
settings-subtitles-position-helper = 100 sitzt am unteren Rand. Kleinere Zahlen heben die Zeilen im Bild an.
settings-subtitles-style = Stil
settings-subtitles-styled = Gestaltete Untertitel
settings-subtitles-as-made-helper = Gestaltete (ASS) Dateien sehen genau so aus, wie ihre Macher sie eingestellt haben.
settings-subtitles-mine-helper = Dein Stil unten gilt für jede Datei. Signs und Karaoke können kaputtgehen.
settings-subtitles-size-only-helper = Gestaltete behalten ihren Look in deiner Größe, schlichte nehmen deinen Stil.
settings-subtitles-as-made = Original
settings-subtitles-size-only = Nur Größe
settings-subtitles-mine = Eigene
settings-subtitles-font = Schrift
settings-subtitles-font-invalid = Gib einen Schriftnamen ein, etwa Noto Sans.
settings-subtitles-bold = Fett
settings-subtitles-text-colour = Textfarbe
settings-subtitles-colour-invalid = Gib eine Farbe wie #ffcc00 ein.
settings-subtitles-outline = Umriss
settings-subtitles-outline-invalid = Die Breite ist eine Zahl, 0 oder mehr.
settings-subtitles-outline-colour-invalid = Gib die Farbe wie #000000 ein.
settings-subtitles-outline-helper = Erst die Breite, dann die Farbe.
settings-subtitles-shadow = Schatten
settings-subtitles-shadow-invalid = Eine Zahl, 0 oder mehr. 0 heißt kein Schatten.
settings-subtitles-shadow-helper = Wie weit der Schatten fällt, 0 für keinen.
settings-subtitles-box = Hintergrundbox
settings-subtitles-box-helper = Dunkle Box hinterm Text. 0 lässt sie weg.
settings-subtitles-start-over = Von vorn
settings-subtitles-start-over-helper = Setzt Größe, Schrift, Farben, Umriss, Box und Position auf die Ausgangswerte zurück.
settings-subtitles-reset = Zurücksetzen

## SettingsDataTab

settings-data-title = Speicher und Backup
settings-data-lead = Bibliothek umziehen, eine Kopie sichern oder den Platz der Bilder freigeben.
settings-data-backed-up = Gesichert in { $path }
settings-data-restore-finished = Wiederherstellung fertig
settings-data-images-removed =
    { $count ->
        [one] { $count } Bild entfernt
       *[other] { $count } Bilder entfernt
    }
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] { $count } Ordner hinzugefügt
       *[other] { $count } Ordner hinzugefügt
    }
settings-data-summary-series = { $count } Serien angelegt
settings-data-summary-matches =
    { $count ->
        [one] { $count } Zuordnung übernommen
       *[other] { $count } Zuordnungen übernommen
    }
settings-data-summary-views =
    { $views ->
        [one] { $views } Ansicht
       *[other] { $views } Ansichten
    } und { $episodes ->
        [one] { $episodes } beendete Folge
       *[other] { $episodes } beendete Folgen
    } zusammengeführt
settings-data-summary-resume-points =
    { $count ->
        [one] { $count } Fortsetzungspunkt zusammengeführt
       *[other] { $count } Fortsetzungspunkte zusammengeführt
    }
settings-data-summary-accounts =
    { $count ->
        [one] { $count } Listenkonto übernommen
       *[other] { $count } Listenkonten übernommen
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } Datei übernimmt eine Korrektur von dir.
       *[other] { $count } Dateien übernehmen eine Korrektur von dir.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } Ordner wurde auf diesem Computer nicht gefunden.
       *[other] { $count } Ordner wurden auf diesem Computer nicht gefunden.
    }
settings-data-summary-left-out = Ausgelassen: { $fields }.
settings-data-backup = Backup
settings-data-back-up-library = Bibliothek sichern
settings-data-back-up-helper = Eine Datei mit Ordnern, Zuordnungen und Geschautem.
settings-data-last-written = Zuletzt geschrieben nach { $path }
settings-data-open-folder = Ordner öffnen
settings-data-back-up = Sichern
settings-data-private = Private Daten einschließen
settings-data-private-helper = Fügt Anmeldungen bei AniList und MyAnimeList, Verlauf und Einstellungen hinzu. Anmeldungen stehen im Klartext, behalte die Datei für dich.
settings-data-restore-from = Aus Backup wiederherstellen
settings-data-restore-helper = Liest ein Backup in diese Bibliothek. Was auf der Platte liegt, gilt; die Datei füllt den Rest auf.
settings-data-restore = Wiederherstellen
settings-data-image-cache = Bildcache
settings-data-image-cache-helper = Poster, Banner und Porträts liegen auf der Platte, damit die Bibliothek offline geht. Gelöschte lädt die Seite neu.
settings-data-cache-unmeasured = Der Bildcache ließ sich nicht messen
settings-data-cache-size =
    { $count ->
        [one] { $size } in { $count } Bild
       *[other] { $size } in { $count } Bildern
    }
settings-data-cache-empty = Keine Bilder im Cache
settings-data-cache-unmeasured-helper = Versuch es gleich nochmal. Die Bilder selbst sind in Ordnung.
settings-data-remove-images =
    { $count ->
        [one] { $count } Bild entfernen?
       *[other] { $count } Bilder entfernen?
    }
settings-data-files = Wo AniBeam seine Dateien ablegt
settings-data-files-database = Bibliotheksdatenbank
settings-data-files-data = Daten
settings-data-files-settings = Einstellungen
settings-data-files-cache = Cache
settings-data-backup-filter = AniBeam-Backup (*.json)
settings-data-all-files-filter = Alle Dateien (*)
settings-data-backing-up = Sichere
settings-data-restoring = Stelle wieder her

## SettingsLibraryTab

settings-library-title = Bibliothek
settings-library-lead = Die Ordner, die AniBeam beobachtet, und wie es herausfindet, was darin ist.
settings-library-not-scanned = Noch nicht gescannt
settings-library-scan-nothing-changed = Nichts geändert
settings-library-scan-added = { $count } hinzugefügt
settings-library-scan-changed = { $count } geändert
settings-library-scan-removed = { $count } entfernt
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = Anime: Serien, Filme und wie sie zusammenhängen. Was du schaust, wird bei deinen Trackern markiert.
settings-library-tvmaze-about = Anderes Fernsehen: Realserien, Cartoons, Dokus. Nichts wird bei einem Tracker markiert.
settings-library-tried-first = Zuerst probiert
settings-library-tried-second = Als Zweites probiert
settings-library-tried-third = Als Drittes probiert
settings-library-tried-fourth = Als Viertes probiert
settings-library-tried-nth = Als { $number }. probiert
settings-library-folders = Ordner
settings-library-folders-helper = Jeder Ordner in einem davon ist eine Serie. Eine Datei oben in einem Ordner Movies ist ein Film.
settings-library-rescan-started = Neuer Scan gestartet
settings-library-load-failed = Deine Ordner konnten nicht gelesen werden
settings-library-load-failed-helper = AniBeam konnte seine Ordnerliste nicht lesen. Versuch es gleich nochmal.
settings-library-no-folders = Noch keine Ordner
settings-library-no-folders-helper = Zeig AniBeam den Ordner mit deinen Anime. Es findet die Serien und Filme von selbst.
settings-library-scanned = Gescannt { $when }
settings-library-add-a-folder = Ordner hinzufügen
settings-library-scan-all = Alle scannen
settings-library-scan-started = Scan gestartet
settings-library-add-folder = Ordner hinzufügen
settings-library-matching = Zuordnung
settings-library-matching-helper = Neue Ordner werden der Reihe nach gesucht, der erste gute Treffer gilt. Eigene Zuordnungen bleiben immer.
settings-library-try-later = Später probieren
settings-library-try-first = Zuerst probieren
settings-library-check-matches = Zuordnungen prüfen
settings-library-check-matches-helper = Die Seite Serieninfo listet jede Serie mit den Infos, die AniBeam gefunden hat, und korrigiert eine falsche.
settings-library-open-show-info = Serieninfo öffnen
settings-library-hidden-shows = Ausgeblendete Serien
settings-library-show-hidden = Ausgeblendete Serien zeigen
settings-library-show-hidden-helper = Zeigt ausgeblendete Serien überall wieder, bis AniBeam schließt.
settings-library-hidden-from-airing = Aus Läuft jetzt ausgeblendet
settings-library-hidden-from-airing-helper = Serien, die du als uninteressant markiert hast. Jede bleibt aus Läuft jetzt raus, bis du sie zurückholst.
settings-library-hidden-from-airing-empty = Noch nichts. Serien, die du unter Läuft jetzt als uninteressant markierst, sammeln sich hier, damit jede zurückkommen kann.
settings-library-show-again = Wieder zeigen
settings-library-calendar = Kalender
settings-library-calendar-feed = Sendezeiten im Kalender
settings-library-calendar-feed-helper = Hält eine Kalenderdatei mit den kommenden Folgen der Serien auf deinen Listen, aktualisiert, wenn sich der Sendeplan ändert.
settings-library-calendar-file = Kalenderdatei
settings-library-calendar-file-helper = Abonniere diese Datei in deiner Kalender-App.
settings-library-copy = Kopieren
settings-library-path-copied = Pfad kopiert
settings-library-open-folder = Ordner öffnen
settings-library-notifications = Benachrichtigungen
settings-library-new-episodes = Neue Folgen
settings-library-new-episodes-helper = Eine Desktop-Benachrichtigung, wenn eine Folge einer Serie auf deiner Liste Schaue ich läuft. Klick darauf öffnet die Serie.
settings-library-folder-added = Ordner hinzugefügt, scanne

## SettingsAppearanceTab

settings-appearance-language = Sprache
settings-appearance-language-system = Wie das System ({ $language })
settings-appearance-text-smaller = Kleiner
settings-appearance-text-normal = Normal
settings-appearance-text-large = Groß
settings-appearance-text-larger = Größer
settings-appearance-text-largest = Am größten
settings-appearance-title = Darstellung
settings-appearance-lead = Änderungen gelten sofort, hier und auf jeder Seite. Sie liegen in ~/.config/anibeam/theme.toml, das du auch von Hand bearbeiten kannst.
settings-appearance-colours = Farben
settings-appearance-mode = Modus
settings-appearance-mode-dark = Dunkel
settings-appearance-mode-light = Hell
settings-appearance-mode-system = System
settings-appearance-colours-from = Farben von
settings-appearance-from-theme-helper = Das Designpaar unten, eins für dunkel und eins für hell.
settings-appearance-from-desktop-helper = Die Farben deines Terminals, sonst das Farbschema und der Akzent des Desktops.
settings-appearance-from-desktop = Desktop
settings-appearance-from-theme = Design
settings-appearance-dark-theme = Dunkles Design
settings-appearance-light-theme = Helles Design
settings-appearance-themes-helper = Designdateien, die du in ~/.config/anibeam/themes legst, erscheinen in beiden Listen.
settings-appearance-accent = Akzent
settings-appearance-accent-helper = Markiert, was ausgewählt ist, läuft oder neu ist.
settings-appearance-icon = Symbol
settings-appearance-app-icon = App-Symbol
settings-appearance-app-icon-helper = Das des Fensters, der Taskleiste und des Launchers. Dunkel tauscht Gelb und Schwarz.
settings-appearance-icon-default = Standard
settings-appearance-icon-default-name = Standard-App-Symbol
settings-appearance-icon-dark = Dunkel
settings-appearance-icon-dark-name = Dunkles App-Symbol
settings-appearance-size-motion = Größe und Bewegung
settings-appearance-text-size = Textgröße
settings-appearance-text-size-helper = Wie groß die Wörter überall in AniBeam sind, zusätzlich zur Textgröße deines Systems.
settings-appearance-density = Dichte
settings-appearance-density-helper = Der Raum um alles, von eng bis luftig.
settings-appearance-density-compact = Kompakt
settings-appearance-density-normal = Normal
settings-appearance-density-comfortable = Luftig
settings-appearance-poster-size = Postergröße
settings-appearance-poster-size-helper = Wie groß Serien in Bibliothek und Start sind.
settings-appearance-poster-small = Klein
settings-appearance-poster-medium = Mittel
settings-appearance-poster-large = Groß
settings-appearance-out-now = Jetzt da unter Läuft
settings-appearance-out-now-helper = Karten mit dem Banner jeder Serie, oder eine Liste kleiner Poster mit Titeln, drei Zeilen tief.
settings-appearance-out-now-cards = Karten
settings-appearance-out-now-list = Liste
settings-appearance-out-now-grid = Karten unter Jetzt da
settings-appearance-out-now-grid-helper = Wie viele Karten erscheinen, Spalten mal Zeilen, bevor Alle den Rest öffnet.
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Bewegung reduzieren
settings-appearance-reduce-motion-helper = Lässt den Auftakt der Bibliothek und andere reine Zierbewegung weg.

## LookPane

look-pane-sample-series = Beispielserie
look-pane-episode-code = F { $number }
look-pane-library = Bibliothek
look-pane-series-count =
    { $count ->
        [one] { $count } Serie
       *[other] { $count } Serien
    }
look-pane-search = Titel oder Ordner suchen
look-pane-search-hint = / oder Strg+K
look-pane-all = Alle
look-pane-series = Serien
look-pane-movies = Filme
look-pane-plain = Normal
look-pane-selected = Ausgewählt
look-pane-errors =
    { $count ->
        [one] { $count } Fehler
       *[other] { $count } Fehler
    }
look-pane-type-large = Groß
look-pane-type-normal = Normal
look-pane-type-small = Klein, gedimmt
look-pane-sample-switch = Beispielschalter
look-pane-button = Knopf
look-pane-scan = Scannen
look-pane-sample-status =
    { $count ->
        [one] { $count } neue Folge in { $title }
       *[other] { $count } neue Folgen in { $title }
    }

## FeedPage

feed-title = Feed
feed-aired = { $episode } gelaufen { $when }
feed-downloaded = Datei geladen { $when }
feed-countdown-days = { $days } T. { $hours } Std.
feed-countdown-hours = { $hours } Std. { $minutes } Min.
feed-countdown-minutes = { $minutes } Min.
feed-section-this-week = Diese Woche
feed-new-episodes =
    { $count ->
        [one] { $count } neue Folge
       *[other] { $count } neue Folgen
    }
feed-section-earlier = Früher
feed-latest-out = Zuletzt erschienen { $when }
feed-section-coming-soon = Demnächst
feed-this-week-count = { $count } diese Woche
feed-section-everything-else = Alles andere
feed-out-this-week = { $count } diese Woche erschienen
feed-nothing-scheduled = Noch nichts geplant.
feed-next = Nächste: { $title } in { $countdown }
feed-nothing-new = Diese Woche nichts Neues.
feed-new-this-week =
    { $count ->
        [one] { $count } neue Folge diese Woche.
       *[other] { $count } neue Folgen diese Woche.
    }
feed-sort-recent = Zuletzt erschienen
feed-sort-upcoming = Demnächst
feed-failed-title = Der Feed wurde nicht geladen
feed-empty-title = Hier ist noch nichts
feed-failed-body = AniBeam konnte den Feed gerade nicht lesen. Versuch es gleich nochmal.
feed-empty-body-folders = Der Feed füllt sich, wenn Folgen laufen und ankommen. Läuft jetzt zeigt, was diese Saison läuft.
feed-empty-body = Füge den Ordner mit deinen Anime hinzu, und der Feed füllt sich mit seinen Folgen.
feed-add-folder = Ordner hinzufügen
feed-airing-now = Läuft jetzt

## RecommendedPage

recommended-title = Empfohlen für dich
recommended-ranked-minutes = Sortiert vor { $minutes } Min.
recommended-ranked-hours = Sortiert vor { $hours } Std.
recommended-ranked-days = Sortiert vor { $days } T.
recommended-scope-name = Empfohlen aus
recommended-scope-all = Alle
recommended-scope-library = Bibliothek
recommended-scope-list = Liste
recommended-ranking = Sortiere
recommended-rank-again = Neu sortieren
recommended-no-scores-title = Bewerte erst ein paar Serien
recommended-no-list-title = Noch nichts zum Lernen
recommended-never-fitted-title = Sortiere deine Serien
recommended-nothing-left-title = Hier ist nichts mehr
recommended-no-scores-body = Empfehlungen lernen aus den Wertungen auf deiner Liste. Bewerte ein paar Serien auf AniList, dann erscheinen sie hier.
recommended-no-list-body = Verbinde AniList in den Einstellungen oder führe hier eine Liste. Wertungen prägen die Seite.
recommended-never-fitted-body = Die erste Sortierung liest AniList ein, zwei Minuten lang. Sie läuft im Hintergrund.
recommended-nothing-left-library-body = Alles Passende in deiner Bibliothek ist geschaut. Alle findet Serien, die du noch nicht hast.
recommended-nothing-left-body = Alles Passende auf deinen Listen ist geschaut. Alle findet Serien, die du noch nicht gelistet hast.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] Verschiebe 1 Serie zu { $site }
       *[other] Verschiebe { $count } Serien zu { $site }
    }
move-lists-score = Wertung { $score }
move-lists-title = Liste zu { $site } verschieben
move-lists-body =
    { $count ->
        [one] Eine Serie hat
       *[other] { $count } Serien haben
    } nur auf diesem Computer einen Status oder eine Wertung. Beim Verschieben landen sie auf deiner { $site }-Liste, wo dein Handy und dein Profil sie sehen. Bei diesen Serien wird ersetzt, was { $site } jetzt hat. Gesendet wird erst, wenn du auf Verschieben klickst.
move-lists-head-show = Serie
move-lists-head-here = Auf diesem Computer
move-lists-head-tracker = Jetzt bei { $site }
move-lists-nothing = Nichts
move-lists-not-on-list = Nicht auf deiner Liste
move-lists-later-hint = Später: Einstellungen, Konten
move-lists-not-now = Nicht jetzt
move-lists-move =
    { $count ->
        [one] 1 Serie verschieben
       *[other] { $count } Serien verschieben
    }

## StatsPage

stats-title = Statistik
stats-month-jan = Jan
stats-month-feb = Feb
stats-month-mar = Mär
stats-month-apr = Apr
stats-month-may = Mai
stats-month-jun = Jun
stats-month-jul = Jul
stats-month-aug = Aug
stats-month-sep = Sep
stats-month-oct = Okt
stats-month-nov = Nov
stats-month-dec = Dez
stats-nothing-yet = Noch nichts
stats-ranked-episodes =
    { $count ->
        [one] { $count } F.
       *[other] { $count } F.
    }
stats-year-summary = Das hast du { $year } geschaut
stats-empty-title = Noch nichts geschaut
stats-empty-body = Schau eine Folge zu Ende, dann zählt sie hier: Stunden, Genres und die Saisons, die du geschaut hast.
stats-open-library = Bibliothek
stats-hours-watched = Std. geschaut
stats-episodes-label =
    { $count ->
        [one] Folge
       *[other] Folgen
    }
stats-series-finished = Serien beendet
stats-none = Keine
stats-busiest-month = stärkster Monat
stats-by-month = Nach Monat
stats-month-hours = { $hours } Std.
stats-top-genres = Top-Genres
stats-top-studios = Top-Studios
stats-seasons-watched = Gesehene Saisons
stats-recap-title = Mein Anime-Jahr
stats-recap-hours = Stunden
stats-recap-episodes = Folgen
stats-recap-top-genre = Top-Genre
stats-save-image = Als Bild speichern
stats-save-dialog-title = Rückblick speichern
stats-save-filter-png = PNG-Bild (*.png)
stats-recap-saved = Rückblick gespeichert
stats-recap-save-failed = Rückblick konnte nicht gespeichert werden

## TrackerRow

tracker-row-connected = Verbunden als { $user }, gesynct { $when }
tracker-row-unknown-user = ?
tracker-row-never = nie
tracker-row-waiting = Warte, bis du im Browser zustimmst
tracker-row-not-connected = Nicht verbunden
tracker-row-open-again = Nochmal öffnen
tracker-row-open-again-tip = Anmeldeseite nochmal öffnen
tracker-row-disconnect = Trennen
tracker-row-disconnect-question = { $name } trennen? Deine Anmeldung wird entfernt.
tracker-row-secret-saved = Für diese App ist ein Client-Secret gespeichert.
tracker-row-remove-secret = Entfernen
tracker-row-secret-removed = Client-Secret entfernt
tracker-row-add-secret = Client-Secret hinzufügen
tracker-row-secret-where = Es ist die lange Zeichenfolge unter "Client Secret" auf der Seite der App unter myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Client-Secret
tracker-row-save-connect = Speichern und verbinden

## RemoveSheet

remove-unsubscribe-delete = Abo beenden und löschen
remove-unsubscribe-torrents = Abo beenden und Torrents entfernen
remove-unsubscribe = Abo beenden
remove-delete-files = Entfernen und Dateien löschen
remove-torrents =
    { $count ->
        [one] Torrent entfernen
       *[other] Torrents entfernen
    }
remove-also-torrents =
    { $count ->
        [one] Auch seinen Torrent entfernen
       *[other] Auch seine { $count } Torrents entfernen
    }
remove-torrents-go-with-files = Beim Löschen der Dateien gehen die Torrents mit.
remove-torrents-files-stay = Sie verlassen den Client; die Dateien bleiben.
remove-also-files = Auch die Dateien löschen
remove-files-default-where = was die Torrents geladen haben
remove-files-detail-rule = Verschiebt { $where } in den Papierkorb und entfernt die Torrents.
remove-files-detail = Verschiebt { $where } in den Papierkorb.

## NotesSheet

notes-your-tracker = deinem Tracker
notes-title = Notizen
notes-placeholder = Wo du aufgehört hast, wer es empfohlen hat, welches Release du holen sollst
notes-over-limit = { $count } Zeichen über dem Limit
notes-saved-to = Gespeichert bei { $sites }

## RecommendationCard

recommendation-card-available = Vorhanden
recommendation-card-open-library = { $title } in deiner Bibliothek öffnen
recommendation-card-open-anilist = { $title } auf AniList öffnen

## SearchField

search-field-placeholder = Titel oder Ordner suchen
search-field-hint = / oder Strg+K
search-field-clear = Suche leeren

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent } %
recommended-hover-hidden-tags =
    { $count ->
        [one] Ein verstecktes Tag, das du liebst
       *[other] { $count } versteckte Tags, die du liebst
    }
recommended-hover-because = Weil du geschaut hast

## PersonCard

person-card-unknown = Unbekannt
person-card-role-main = Haupt
person-card-role-supporting = Neben
person-card-role-background = Hintergrund
person-card-the-character = der Charakter
person-card-open = { $name } auf AniList öffnen

## StatusDot

status-dot-on-list = Auf deiner Liste: { $status }

## TrackerNudge

tracker-nudge-title = Verbinde deine Anime-Liste
tracker-nudge-body = AniBeam markiert Geschautes auf AniList oder MyAnimeList.
tracker-nudge-connect-anilist = AniList verbinden
tracker-nudge-connect-mal = MyAnimeList verbinden
tracker-nudge-not-now = Nicht jetzt

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } Prozent für dich
recommended-card-forecast = { $percent } %
recommended-card-in-library = In der Bibliothek

## InlineConfirm

inline-confirm-keep = Behalten

## Tokens

tokens-cube-watched-tracked = Geschaut, online erfasst
    .inline = geschaut, online erfasst
tokens-cube-watched-here = Hier geschaut, noch nicht erfasst
    .inline = hier geschaut, noch nicht erfasst
tokens-cube-on-disk = Auf der Platte
    .inline = auf der Platte
tokens-cube-downloading = Lädt
    .inline = lädt
tokens-cube-missing = Fehlt
    .inline = fehlt
tokens-cube-error = Fehler
    .inline = Fehler
tokens-cube-not-downloaded = Nicht geladen
    .inline = nicht geladen
tokens-cube-not-aired = Noch nicht gelaufen
    .inline = noch nicht gelaufen
tokens-cube-available = Verfügbar
    .inline = verfügbar
tokens-cube-picked = Zum Laden gewählt
    .inline = zum Laden gewählt
tokens-cube-already-on-disk = Schon auf der Platte
    .inline = schon auf der Platte
tokens-cube-no-release = Kein Release
    .inline = kein Release

## Card

card-not-interested = Kein Interesse

## PagePlaceholder

page-placeholder-coming-soon = Kommt bald

## ErrorLine

error-line-hide-details = Details ausblenden
error-line-details = Details
error-line-details-tip = Zeigt wörtlich, was schiefging, zum Kopieren
error-line-copied = Kopiert.

## Readouts (apps/linux/src/format.rs): Zeiten, Countdowns und Längen

fmt-just-now = gerade eben
fmt-now = jetzt
fmt-ago = vor { $span }
fmt-in = in { $span }
fmt-seconds-short = { $n } s
fmt-minutes-short = { $n } Min.
fmt-hours-short = { $n } Std.
fmt-days-short = { $n } T.
fmt-months-short = { $n } Mon.
fmt-years-short = { $n } J.
fmt-years-months-short = { $years } J. { $months } Mon.
fmt-countdown-days = { $d } T. { $h } Std. { $m } Min.
fmt-countdown-hours = { $h } Std. { $m } Min.
fmt-countdown-days-seconds = { $d } T. { $h } Std. { $m } Min. { $s } s
fmt-countdown-hours-seconds = { $h } Std. { $m } Min. { $s } s
fmt-countdown-minutes-seconds = { $m } Min. { $s } s
fmt-runtime-minutes = { $m } Min.
fmt-runtime-hours = { $h } Std.
fmt-runtime-hours-minutes = { $h } Std. { $m } Min.
fmt-episode = Folge
fmt-size-b = { $size } B
fmt-size-kb = { $size } KB
fmt-size-mb = { $size } MB
fmt-size-gb = { $size } GB
fmt-size-tb = { $size } TB
fmt-episode-span = Folgen { $first } bis { $through }

## apps/linux/src/problem.rs: jeder Fehler in Worten, was passiert ist und was zu tun ist

problem-report = Versuch es nochmal. Wenn es wieder passiert, melde es aus dem Aktivitätsprotokoll.
problem-try-later = Versuch es später nochmal.
problem-helper-unreachable = Der Suchhelfer hat nicht geantwortet.
problem-helper-unauthorized = Der Suchhelfer hat die Anmeldung abgelehnt.
problem-helper-not-running = Der Suchhelfer läuft nicht.
problem-helper-missing = Der Suchhelfer ist nicht installiert.
problem-helper-failed = Der Suchhelfer konnte das nicht.
problem-helper-no-login = Der Suchhelfer kann sich nicht beim Torrent-Client anmelden.
problem-helper-no-login-next = Speichere das Passwort des Torrent-Clients in den Einstellungen unter Downloads.
problem-torrent-unreachable = Der Torrent-Client hat nicht geantwortet.
problem-torrent-unauthorized = Der Torrent-Client hat die Anmeldung abgelehnt.
problem-torrent-not-running = Der Torrent-Client läuft nicht.
problem-torrent-missing = Der Torrent-Client ist nicht installiert.
problem-torrent-failed = Der Torrent-Client konnte das nicht.
problem-downloads-unreachable-next = Prüfe, ob er läuft und seine Adresse in den Einstellungen unter Downloads stimmt.
problem-downloads-unauthorized-next = Prüfe Benutzername und Passwort in den Einstellungen unter Downloads.
problem-downloads-not-running-next = Starte ihn in den Einstellungen unter Downloads.
problem-downloads-missing-next = Einstellungen, Downloads sagt, wie du ihn einrichtest.
problem-trash = { -app-name } konnte die Dateien nicht in den Papierkorb verschieben.
problem-trash-next = Prüfe, ob die Dateien dir gehören und die Platte nicht schreibgeschützt ist.
problem-source-list = Diese Quellenliste hat nicht funktioniert.
problem-source-list-next = Prüfe ihre Adresse in den Einstellungen unter Downloads, Quellenlisten.
problem-remote-unreachable = { $name } hat nicht geantwortet.
problem-remote-unreachable-next = Die Seite ist vielleicht down. Versuch es später nochmal oder durchsuche eine andere Quelle.
problem-remote-unauthorized = { $name } hat den Schlüssel abgelehnt.
problem-remote-unauthorized-next = Ändere den Schlüssel in den Einstellungen unter Downloads, Quellenlisten.
problem-remote-failed = { $name } konnte das nicht.
problem-remote-failed-next = Versuch es später nochmal oder durchsuche eine andere Quelle.
problem-provider-sign-in = { $name } hat die Anmeldung von { -app-name } abgelehnt.
problem-provider-sign-in-next = Verbinde es neu in den Einstellungen unter Konten.
problem-provider-turned-away = { $name } hat { -app-name } abgewiesen.
problem-provider-busy = { $name } ist gerade ausgelastet.
problem-provider-busy-next = Warte eine Minute und versuch es dann nochmal.
problem-provider-no-answer = { $name } hat nicht geantwortet.
problem-provider-no-answer-next = Prüfe deine Internetverbindung und versuch es nochmal.
problem-provider-trouble = { $name } hat gerade Probleme.
problem-provider-trouble-next = Versuch es in ein paar Minuten nochmal.
problem-provider-rejected = { $name } hat das nicht angenommen.
problem-not-found = { -app-name } konnte das nicht mehr finden.
problem-not-found-next = Vielleicht wurde es entfernt. Versuch es nochmal, um die Seite neu zu laden.
problem-invalid-next = Prüfe es und versuch es nochmal.
problem-unavailable = Dieser Ordner ist gerade nicht erreichbar.
problem-unavailable-next = Schließ sein Laufwerk an oder prüfe den Ordner in den Einstellungen unter Bibliothek.
problem-not-connected = { $tracker } ist nicht verbunden.
problem-not-connected-next = Verbinde es in den Einstellungen unter Konten.
problem-io = { -app-name } konnte eine Datei nicht lesen oder schreiben.
problem-io-next = Prüfe, ob die Platte Platz hat und der Ordner dir gehört, und versuch es nochmal.
problem-storage = { -app-name } konnte deine Bibliothek nicht speichern.
problem-storage-next = Prüfe, ob die Platte Platz hat, und versuch es nochmal.
problem-keyring = Der Passwortspeicher des Systems hat nicht geantwortet.
problem-keyring-next = Entsperre den Schlüsselbund deines Desktops und versuch es nochmal.
problem-unsupported = { -app-name } kann das hier noch nicht.
problem-version = Dieses Backup stammt aus einem neueren { -app-name }.
problem-version-next = Aktualisiere { -app-name } in den Einstellungen unter Info und importiere es dann erneut.
problem-internal = In { -app-name } ist etwas schiefgegangen.
problem-refused-hidden = Diese Serie ist ausgeblendet.
problem-refused-hidden-next = Blende sie zuerst auf ihrer Seite ein.
problem-refused-no-match = Diese Serie ist noch nicht zugeordnet.
problem-refused-no-match-next = Ordne sie zuerst unter Serieninfo zu.
problem-refused-not-newer = Deine Liste ist schon weiter als diese Folge.
problem-refused-extra = Diese Datei ist ein Extra, keine Folge.
problem-refused-unmatched = Diese Datei ist keiner Folge zugeordnet.
problem-refused-unmatched-next = Korrigiere es auf der Seite der Serie unter Korrektur.
problem-refused-on-disk = Diese Datei liegt nur auf diesem Computer.

## apps/linux/src/library_view.rs: die Abschnittstitel der Bibliothek und der Fakt darunter
## (Abgebrochen, Pausiert und Schaue ich nutzen status-* aus core.ftl)

library-view-airing = Läuft
library-view-no-date = Kein Datum
library-view-type-tv = TV
library-view-type-movies = Filme
library-view-type-extras = OVA, ONA und Specials
library-view-type-music = Musik
library-view-type-unknown = Unbekannt
library-view-no-genre = Nicht zugeordnet
library-view-state-caught-up = Aufgeholt
library-view-state-not-started = Nicht angefangen
library-view-next-episode = Nächste Folge in { $span }
library-view-countdown-days = { $d } T. { $h } Std.
library-view-countdown-hours = { $h } Std. { $m } Min.
library-view-countdown-minutes = { $m } Min.
library-view-in-progress = { $count } angefangen
library-view-not-started = { $count } nicht angefangen
library-view-all-watched = Alle geschaut
library-view-on-disk =
    { $count ->
        [one] { $count } Folge
       *[other] { $count } Folgen
    } auf der Platte
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } Datei
       *[other] { $count } Dateien
    } noch auf der Platte

## apps/linux/src/dbus/notify.rs: die Desktop-Benachrichtigung für eine neue Folge
## (ihr Knopf Öffnen nutzt action-open aus core.ftl)

notify-episode-out = Folge { $episode } von { $title } ist da
notify-click-open = Klicken, um sie in { -app-name } zu öffnen
notify-click-get = Klicken, um die Folgen in { -app-name } zu laden
notify-get-episodes = Folgen laden

## apps/linux/src/nowplaying.rs: die zweite Zeile eines Medien-Widgets, während eine Folge läuft

now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: warum der Bildschirm an bleibt

inhibit-reason = Spielt eine Folge ab

## apps/linux/src/bridge/door.rs: warum ein Wert, den eine Seite sendet, nicht nutzbar ist

door-not-a-choice = { $value } ist keine gültige Wahl
door-tokens-op-empty = eine tokens-Operation braucht Tokens
door-not-a-refine-op = { $value } ist keine Eingrenzungs-Operation
door-limit-too-low = muss mindestens 1 sein, bekommen: { $limit }
door-not-a-torrent-action = { $value } ist keine Torrent-Aktion

## apps/linux/src/bridge/shell.rs: der Startordner beim ersten Start und der Update-Hinweis

shell-starter-folder-failed = { $folder } konnte nicht angelegt werden: { $error }
shell-update-gone = { -app-name } { $version } wird nicht mehr angeboten

## Watch together (#846)

onboarding-step-together = Gemeinsam schauen
onboarding-title-together = Mit Freunden schauen
onboarding-body-together = Starte im Player einen Raum und schick seinen Code. Alle spielen ihre eigene Kopie der Folge ab, und Abspielen, Pause und Spulen bleiben für alle im Takt.
onboarding-ready-together = Gemeinsam schauen
onboarding-ready-together-as = Räume zeigen dich als { $name }.
onboarding-ready-together-none = Noch kein Name. Das Raumpanel fragt beim ersten Mal.
onboarding-together-you = Du
onboarding-together-code = Raumcode
onboarding-together-episode = Frieren · Folge 3
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = Dein Name in Räumen
rail-room = Gemeinsam schauen (Strg+J)
rail-room-in = Im Raum zum gemeinsamen Schauen (Strg+J)
rail-join-room = Raum beitreten
rail-in-room = Im Raum
player-chrome-watch-together = Gemeinsam schauen
player-chrome-room = Raum { $code } · { $count ->
    [one] 1 Person
   *[other] { $count } Personen
}
room-name-empty = Gib den Namen ein, den andere im Raum sehen.
room-name-bad = Dieser Name geht nicht. Nutze 1 bis 32 Zeichen.
room-error-not-found = Kein Raum hat diesen Code. Prüfe ihn mit der Person, die ihn geschickt hat; der Raum ist vielleicht geschlossen.
room-error-full = Dieser Raum ist voll. Ein Raum fasst 10 Personen.
room-error-unreachable = Raumserver nicht erreichbar. Prüfe deine Verbindung und versuch es nochmal.
room-error-refused = Der Raumserver hat abgelehnt. Versuch es in einer Minute nochmal.
room-error-proto = Dieser Raum läuft auf einer neueren Version von Räumen. Aktualisiere { -app-name }, um beizutreten.
room-error-not-in-room = Du bist nicht mehr in diesem Raum. Starte einen oder tritt wieder einem bei.
room-error-bad-code = Ein Raumcode hat 6 Buchstaben und Ziffern, wie K7QM2X.
room-error-bad-relay = Der Raumserver unter Einstellungen, Wiedergabe ist keine Webadresse. Korrigiere ihn dort und versuch es nochmal.
room-error-cannot-fetch = Der Host kann diese Folge gerade nicht teilen. Versuch es wieder, wenn er zurück im Raum ist.
room-panel-title = Gemeinsam schauen
room-panel-close = Schließen
room-panel-connected = Verbunden
room-panel-reconnecting = Verbinde neu
room-panel-name-label = Dein Name in Räumen
room-panel-name-placeholder = Dein Name
room-panel-name-start = Raum starten
room-panel-name-join = Raum beitreten
room-panel-start = Einen Raum starten
room-panel-or-join = Oder tritt einem mit Code bei
room-panel-join-label = Raumcode
room-panel-join = Beitreten
room-panel-you-are = Du erscheinst als { $name }
room-panel-change-name = Ändern
room-panel-code-label = Raumcode
room-panel-copy = Link kopieren
room-panel-link-copied = Raumlink kopiert
room-panel-watching = { $series } · { $episode }
room-panel-open-episode = Folge öffnen
room-panel-people = { $count ->
    [one] 1 Person
   *[other] { $count } Personen
}
room-panel-host = Host
room-panel-you = Du
room-panel-ready = Bereit
room-panel-opening = Öffnet
room-panel-missing = Hat sie nicht
room-panel-offset = Dein Versatz
room-panel-offset-earlier = Eine halbe Sekunde früher
room-panel-offset-later = Eine halbe Sekunde später
room-panel-seconds = { $value } s
room-panel-suggested = Die Openings schlagen { $offset } vor
room-panel-use-suggested = Übernehmen
room-panel-leave = Raum verlassen
room-panel-get-media = Medien holen
room-panel-downloading = Lädt herunter
room-panel-asking = Frage den Host
room-panel-searching = Suche
room-panel-failed = Nicht bekommen
room-panel-percent = { $percent } %
room-panel-left = noch { $time }
player-room-speed = Das Tempo bestimmt der Raum
settings-find-room-name =
    .words = gemeinsam schauen raum name freunde
settings-find-room-relay =
    .words = gemeinsam schauen raum relay server
key-help-key-ctrl-j = Strg+J
key-help-room = Gemeinsam schauen
settings-playback-together = Gemeinsam schauen
settings-playback-room-name = Dein Name beim gemeinsamen Schauen
settings-playback-room-name-placeholder = Dein Name
settings-playback-room-relay = Raumserver
settings-playback-room-relay-helper = Hier entstehen die Räume. Ändere das nur für einen Server, den du selbst betreibst.
