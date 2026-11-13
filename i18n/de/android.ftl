## Names that stay names

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Words many pages share. Use these before adding a page's own copy.

common-cancel = Abbrechen
common-back = Zurück
common-save = Speichern
common-done = Fertig
common-close = Schließen
common-delete = Löschen
common-retry = Nochmal
common-copy = Kopieren
common-clear = Leeren
common-on = An
common-off = Aus
common-play = Abspielen
common-hide = Ausblenden
common-remove = Entfernen
common-add = Hinzufügen
common-test = Testen
common-reading = Lese...
common-working = Läuft...
common-checking = Prüfe...
common-all = Alle
common-none = Keine
common-episodes = { $count ->
    [one] { $count } Folge
   *[other] { $count } Folgen
}
common-episode-n = Folge { $n }
common-join-comma = { $a }, { $b }

## Language (#793)

lang-follow-phone = Wie das Handy
more-language = Sprache
more-language-phone = Wie das Handy: { $lang }
lang-name-en = Englisch
lang-name-da = Dänisch
lang-name-de = Deutsch
lang-name-es = Spanisch
lang-name-fr = Französisch
lang-name-pt-BR = Portugiesisch (Brasilien)
lang-name-it = Italienisch
lang-name-nl = Niederländisch
lang-name-pl = Polnisch
lang-name-ru = Russisch
lang-name-ja = Japanisch
lang-name-ko = Koreanisch
lang-name-zh-CN = Chinesisch (vereinfacht)

## Onboarding

onb-welcome-title = AniBeam
onb-welcome-note = Deine Anime auf diesem Handy. Finde eine Serie, sieh, wer sie hat, lade sie und halte deine Listen aktuell.
onb-language-title = Sprache
onb-language-note = AniBeam spricht die Sprache deines Handys, solange du keine andere wählst. Du kannst das später unter Mehr ändern.
onb-lists-title = Deine Listen
onb-lists-note = Melde dich an, damit zählt, was du hier schaust, bei AniList oder MyAnimeList. Das geht auch später unter Mehr.
onb-wifi-title = Nur WLAN
onb-wifi-note = Folgen sind groß. AniBeam lädt nur im WLAN, außer du sagst etwas anderes.
onb-storage-title = Speicher
onb-storage-note = Ein Limit dafür, wie viel AniBeam laden darf. Es reserviert nichts: Der Platz bleibt frei, bis Folgen ihn füllen.
onb-notify-title = Benachrichtigungen
onb-notify-note = Während Folgen laden, zeigt eine Benachrichtigung den Fortschritt, eine weitere sagt, wann eine Folge bereit ist. Geladen wird auch ohne.
onb-sites-title = Suchseiten
onb-sites-note = AniBeam bringt nichts zum Suchen mit. Ein Repo ist eine signierte Liste von Seiten zum Durchsuchen. Füge die Adresse eines Repos ein, prüfe den Schlüssel und installiere dann die Seiten, die du willst. Das geht auch später unter Mehr.
onb-wifi-only = Nur im WLAN
onb-wifi-on = An. Mobile Daten fragen vorher.
onb-wifi-off = Aus. Geladen wird auch über mobile Daten.
onb-storage-caption = 18 GB sind etwa 12 Folgen. Zieh den Regler bis zum gesamten Speicher des Handys.
onb-use-cap = Limit nutzen
onb-cap-on = An. Bei { $cap } lädt AniBeam nichts mehr, bis du Folgen löschst.
onb-cap-off = Aus. AniBeam lädt, bis das Handy voll ist.
onb-start = Start
onb-continue = Weiter
onb-back = Zurück

## Appearance

look-colours = Farben
look-mode = Modus
look-dark = Dunkel
look-light = Hell
look-system = System
look-follows-phone-dark = Folgt der Einstellung des Handys: dunkel.
look-follows-phone-light = Folgt der Einstellung des Handys: hell.
look-changes-at-once = Änderungen gelten sofort, hier und auf jeder Seite.
look-dark-theme = Dunkles Design
look-light-theme = Helles Design
look-accent = Akzent
look-accent-note = Markiert, was ausgewählt ist, läuft oder neu ist.
look-size = Größe
look-text-size = Textgröße
look-size-smaller = Kleiner
look-size-normal = Normal
look-size-large = Groß
look-size-larger = Größer
look-size-largest = Am größten
look-text-size-note = Wie groß die Wörter überall in AniBeam sind, zusätzlich zur Textgröße deines Handys.

## The window: tabs and the sheets over every page

app-name = AniBeam
nav-home = Start
nav-calendar = Kalender
nav-library = Bibliothek
nav-downloads = Downloads
nav-more = Mehr
app-mobile-title = Über mobile Daten laden?
app-mobile-note = Eine Folge hat oft 1 bis 2 GB. Mit Datenlimit kann das an einem Abend ein ganzes Monatsvolumen verbrauchen.
app-mobile-keep = Nur WLAN behalten
app-mobile-use = Auch mobile Daten nutzen
app-versions-note = Dieselbe Folge aus { $count } Releases. Wähle eins zum Abspielen.
app-versions-plays-now = { $release }, läuft gerade
app-menu-get = Folgen laden
app-menu-delete = Folgen löschen

## More: the index and each section's line of how it stands

more-settings = Einstellungen
more-search = Einstellungen suchen
more-search-none = Keine Treffer
more-search-hits = { $count ->
    [one] 1 Einstellung
   *[other] { $count } Einstellungen
}
more-search-tap = Zum Öffnen antippen
more-search-hint = Versuch ein einfacheres Wort: Speicher, WLAN, Tracker, Seite, Computer.
more-accounts = Konten
more-streaming = Streaming
more-subtitles = Untertitel
more-connection = Verbindung
more-computer = Dein Computer
more-icon = App-Symbol
more-stats = Statistik
more-data = Daten
more-appearance = Darstellung
more-activity = Aktivität
more-about = Info
more-accounts-none = Kein Tracker verbunden
more-accounts-syncing = Sync mit { $tracker }
more-accounts-syncing-both = Sync mit AniList und MAL
more-dl-status-wifi = Nur WLAN, kein Limit
more-dl-status-wifi-cap = Nur WLAN, Limit { $cap }
more-dl-status-mobile = WLAN und mobile Daten, kein Limit
more-dl-status-mobile-cap = WLAN und mobile Daten, Limit { $cap }
more-stream-status-auto = Auto, 720p bei mobilen Daten oder Relay
more-stream-status-original = Immer die Originaldatei
more-stream-status-converted = Umgewandelt in { $quality }
more-sites-status-none = Noch kein Repo
more-sites-status-on = { $sites ->
    [one] 1 Seite an
   *[other] { $sites } Seiten an
}
more-sites-status-on-from = { $sites ->
    [one] 1 Seite an
   *[other] { $sites } Seiten an
}, { $repos ->
    [one] von 1 Repo
   *[other] von { $repos } Repos
}
more-conn-status-unchecked = Noch nicht geprüft
more-conn-status-ok = Alle Schritte geklappt
more-conn-status-failed = { $total ->
    *[other] { $failed } von { $total } Schritten fehlgeschlagen
}
more-computer-status-none = Noch nicht verbunden
more-icon-default = Standard
more-icon-dark = Dunkel
more-icon-line = Linien
more-subs-status = Größe, Position, Schrift, Farben, Box
more-data-status = Sichern, Wiederherstellen, Bildcache
more-activity-status = Was AniBeam getan hat, neueste zuerst
more-look-status-phone = Wie das Handy
more-stats-status = { $count ->
    [one] { $hours } Stunde { $year } geschaut
   *[other] { $hours } Stunden { $year } geschaut
}
more-stats-status-empty = Ein Jahr Anime

## More: the sections' own settings

more-accounts-note = Was du hier schaust, markiert deine Listen, und ihr Fortschritt erscheint bei deinen Serien.
more-storage = Speicher
more-storage-capped = Limit { $cap }
more-storage-no-cap = Kein Limit
more-storage-used = von Folgen belegt
more-storage-cap-caption = Limit
more-storage-free = frei auf dem Handy
more-set-storage-cap = Speicherlimit
more-storage-cap-on = Bei { $cap } wird nichts mehr geladen, bis du Folgen löschst.
more-storage-cap-off = Aus: AniBeam lädt, bis das Handy voll ist.
more-network = Netzwerk
more-network-wifi = Nur WLAN
more-network-mobile = WLAN und mobile Daten
more-wifi-on = An. Downloads warten auf WLAN; mobile Daten fragen vorher.
more-set-stream-quality = Streamqualität
more-stream-question = Wie eine Folge von deinem Computer hier abgespielt wird.
more-stream-auto-note = Auto spielt die Datei im WLAN unverändert ab, bei mobilen Daten oder wenn Tailscale deinen Computer über ein Relay erreicht umgewandelt in 720p.
more-stream-original-note = Die Datei unverändert, immer. Große Dateien brauchen eine schnelle Verbindung.
more-stream-converted-note = Auf deinem Computer in { $quality } umgewandelt. Spart Daten; Spulen dauert einen Moment.
more-set-feeds = Eigene Feeds
more-feeds-note = Wird vor den Seiten durchsucht
more-set-check = Verbindung prüfen
more-check-note = Testet jeden Schritt zwischen diesem Handy und den Suchseiten: finden, erreichen und eine sichere Verbindung.
more-check-copy = Ergebnisse kopieren
more-set-computer = AniBeam auf deinem Computer
more-computer-note = Seine Bibliothek erscheint unter Bibliothek, seine Folgen laufen hier. Ein Name wie my-computer oder eine Adresse wie 100.64.0.1.
more-computer-placeholder = my-computer:8790
more-subs-note = Womit jede Folge auf diesem Handy startet. Eine, die von deinem Computer gestreamt wird, startet mit den Einstellungen dieses Computers.
more-icon-note = Das Symbol auf dem Startbildschirm und in der App-Übersicht.
more-icon-line-note = Linien übernimmt die Farben deines Hintergrundbilds, wie deine anderen Material-You-Symbole.
more-icon-moved-note = Manche Launcher nehmen das Symbol vom Startbildschirm, wenn es sich ändert. In der App-Übersicht bleibt es; zieh es von dort zurück.
more-set-whats-new = Neuerungen
more-whats-new-note = Was sich in jeder Version geändert hat, neueste zuerst.
more-set-onboarding = Neu einrichten
more-onboarding-note = Die Fragen vom ersten Start noch einmal durchgehen.
more-set-version = Version

## Status lines Rust writes on the pages

host-synced = Gesynct { $when }.
host-last-try = Letzter Versuch { $when }: { $error }
host-asking = Frage nach...
host-reached = { $host } erreicht, AniBeam { $version }.
host-saved = Gespeichert. Sync...
acct-signed-in-as = Angemeldet als { $name }
acct-signed-in = Angemeldet
acct-not-in-build = In dieser Version nicht verfügbar
acct-tap-to-sign-in = Nicht angemeldet. Zum Anmelden tippen.
acct-sign-in-failed = Anmeldung fehlgeschlagen: { $why }. Tippe zum Wiederholen.
acct-sign-in-not-started = Anmeldung nicht gestartet: { $error }. Tippe zum Wiederholen.
acct-still-starting = AniBeam startet noch. Versuch es gleich nochmal.
home-recently-watched = Zuletzt geschaut
home-recently-downloaded = Zuletzt geladen
home-note-empty = Auf diesem Handy
home-note = { $count ->
    [one] Auf diesem Handy, 1 Serie
   *[other] Auf diesem Handy, { $count } Serien
}
lib-this-phone = Dieses Handy
lib-note-phone = { $count ->
    [one] Dieses Handy, 1 Serie
   *[other] Dieses Handy, { $count } Serien
}
lib-note-remote = { $device }, gesynct { $when }
lib-empty-search = Keine Treffer für „{ $query }“.
lib-empty-phone = Noch nichts auf diesem Handy.
lib-empty-remote = Dieses Gerät hat nichts in seiner Bibliothek.
cal-view-airing = Läuft jetzt
cal-note-none = Diese Woche läuft nichts
cal-note = { $count ->
    [one] 1 Folge in den nächsten 7 Tagen
   *[other] { $count } Folgen in den nächsten 7 Tagen
}
airing-asking = Frage AniList
airing-asking-newest = AniList nach den neuesten fragen
airing-filter-all = Alles { $count }
airing-filter-listed = Meine Listen { $count }
airing-filter-owned = Auf dem Handy { $count }
airing-this-season = Diese Saison
airing-next-season = Nächste Saison
airing-other-season = Andere
airing-hidden = { $count ->
    *[other] { $count } als uninteressant ausgeblendet
}
airing-plan-needs-tracker = Vormerken braucht AniList oder MAL. Melde dich unter Mehr, Konten an.
airing-plan-failed = Vormerken hat nicht geklappt: { $error }
list-watching = Schaue ich
list-planning = Geplant
list-completed = Abgeschlossen
list-paused = Pausiert
list-dropped = Abgebrochen
list-repeating = Erneut
more-check-all-ok = Alle Schritte geklappt.
more-check-failed = { $failed } von { $total } Schritten fehlgeschlagen. Ein Screenshot dieser Liste zeigt, wo.
more-check-row = { $step }  { $ms } ms
more-check-copied = Kopiert. Füg es in eine Nachricht ein.
more-version = AniBeam für Android { $version } (Build { $build })
dl-note-waiting = Warte auf WLAN. Downloads laufen von selbst weiter, sobald das Handy im WLAN ist.
dl-note-nothing = Nichts wird geladen.
dl-state-done = Fertig
dl-state-waiting = Wartet auf WLAN
dl-state-paused = Pausiert
dl-state-peers = Sucht Peers
dl-state-checking = Prüft
dl-state-error = Durch Fehler gestoppt
dl-state-downloading = Lädt
dl-amount = { $got } von { $size }
dl-speed = { $rate }/s
dl-left = noch { $span }
dl-episodes-range = Folgen { $from } bis { $to }
dl-batch = Ein Batch
disk-storage-line = { $used } belegt
disk-storage-line-free = { $used } belegt, { $free } frei
disk-free = Auf diesem Handy sind { $free } frei.
disk-free-below-cap = Auf diesem Handy sind { $free } frei, weniger als das Limit: Das Handy ist voll, bevor AniBeam stoppt.

## More's search: the words a person might type for each setting

more-set-subs-size = Untertitelgröße und -position
more-set-subs-style = Untertitelstil
more-words-anilist = anilist tracker anmelden login konto verbinden liste fortschritt
more-words-mal = mal myanimelist tracker anmelden login konto verbinden liste fortschritt
more-words-storage-cap = platz speicher limit gb voll löschen größe
more-words-wifi = wlan wifi wi-fi mobile daten mobilfunk netzwerk
more-words-notifications = benachrichtigung hinweis mitteilung fortschritt berechtigung erlauben
more-words-stream-quality = auto original 1080p 720p 480p umwandeln daten computer abspielen
more-words-check-repos = aktualisieren prüfen repo endpoint
more-words-add-repo = repo endpoint vertrauen fingerabdruck adresse url
more-words-add-site = eigene endpoint feed rss torznab jackett prowlarr adresse
more-words-feeds = feed rss quelle entfernen
more-words-check = netzwerk dns test https sicher erreichen internet
more-words-computer = tailscale tailnet host adresse desktop bibliothek stream test speichern gerät
more-words-whats-new = changelog patchnotes neuerungen änderungen update
more-words-onboarding = einrichtung erster start neu anfangen
more-words-version = build info
more-words-stats = stunden folgen genres studios saisons jahr geschaut rückblick monat verlauf
more-words-backup = backup sicherung export datei speichern kopieren umziehen computer json privat anmeldungen
more-words-restore = wiederherstellen import datei laden bibliothek zusammenführen
more-words-cache = bilder poster banner leeren speicher platz cache
more-words-activity = protokoll log ereignisse fehler probleme kopieren leeren verlauf was passiert
more-words-colours = darstellung design dunkel hell modus system akzent farbe
more-words-text-size = darstellung schrift größer kleiner größe wörter lesen
more-words-icon = symbol icon launcher startbildschirm logo dunkel linien themed material you
more-words-subs-size = untertitel größe skalierung position anheben unten
more-words-subs-style = untertitel schrift farbe umriss schatten box fett ass gestaltet zurücksetzen
more-words-language = sprache übersetzung deutsch englisch locale

## Home

home-title = Start
home-next-to-air = Als Nächstes
home-after-that = Danach
home-coming-up = Demnächst
home-nothing-week = Nichts in der nächsten Woche.
home-empty-title = Noch nichts auf diesem Handy
home-empty-body = Serien, die du hier lädst, und was dafür läuft, erscheinen unter Start. Finde eine unter Downloads.

## Library

lib-title = Bibliothek
lib-refresh = Neu laden
lib-search-phone = Handy durchsuchen
lib-search-device = { $device } durchsuchen
lib-find-in-downloads = „{ $query }“ in Downloads suchen
lib-empty-body = Folgen, die du auf diesem Handy lädst, landen hier. Finde eine Serie unter Downloads.
lib-pick-show = Wähle eine Serie, um sie hier zu öffnen.
lib-shows-caption = { $count ->
    [one] Serie
   *[other] Serien
}
lib-on-phone = auf dem Handy
lib-free = frei
lib-hold-poster = Halte ein Poster gedrückt, um mehr Folgen zu laden oder sie zu löschen.
phone-nothing-on-phone = nichts auf dem Handy
phone-on-phone = { $count } auf dem Handy

## Calendar

cal-title = Kalender
cal-week = Woche
cal-days = Tage
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A, %-d. %B
cal-today = Heute
cal-tomorrow = Morgen
cal-yesterday = Gestern
cal-nothing = Nichts
cal-next = Nächste
cal-get = Laden
cal-day-note = { $date }, { $count }
cal-next-airing = Nächste Folge: { $title }
cal-nothing-aired-yesterday = Gestern lief nichts.
cal-nothing-aired-that-day = An dem Tag lief nichts.
cal-nothing-today = Heute läuft nichts.
cal-nothing-tomorrow = Morgen läuft nichts.
cal-nothing-that-day = An dem Tag läuft nichts.
cal-empty-title = Noch keine Serien auf diesem Handy
cal-empty-body = Folgen der Serien, die du hier behältst, erscheinen an ihrem Tag, zwei Tage zurück und eine Woche voraus.
phone-episodes-range = Folgen { $from } bis { $to }
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = Auf dem Handy
phone-out-ago = Erschienen { $ago }

## Airing now

airing-out-now = Jetzt neu
airing-last-two-days = Die letzten zwei Tage
airing-season-note = Die Serien der Saison kommen von AniList.
airing-none-lists-title = Nichts Laufendes steht auf deinen Listen
airing-none-lists-body = Vormerken bei einer Serie setzt sie auf deine Liste.
airing-none-phone-title = Nichts Laufendes ist auf diesem Handy
airing-none-phone-body = Folgen laden bei einer Serie lädt sie hierher.
airing-marked = { $title } ist als uninteressant markiert.
airing-undo = Rückgängig
airing-plan = Vormerken
airing-get-more = Mehr Folgen laden
airing-get = Folgen laden
airing-not-interested = Kein Interesse

## Seasons

season-winter = Winter
season-spring = Frühling
season-summer = Sommer
season-fall = Herbst
season-named = { $season } { $year }
season-pick = Saison wählen

## A show's page

series-back-home = Start
series-back-library = Bibliothek
series-back-downloads = Downloads
series-back-calendar = Kalender
series-get-episodes = Folgen laden
series-episodes = Folgen
series-delete-episodes = Folgen löschen
series-delete-tap = Tippe auf die Folgen, die gelöscht werden sollen.
series-delete-picked = { $count ->
    [one] { $count } Folge gewählt.
   *[other] { $count } Folgen gewählt.
}
series-delete-note = Ihre Dateien werden von diesem Handy gelöscht. Was du geschaut hast, bleibt erfasst.
series-delete-button = { $count ->
    [one] { $count } Folge löschen
   *[other] { $count } Folgen löschen
}
series-no-episodes = Keine Folgen auf diesem Handy und keine geplant.
series-versions = { $count ->
    [one] { $count } Version
   *[other] { $count } Versionen
}
series-note-next-up = Nächste
series-note-missing = Fehlt
series-note-not-here = Nicht auf dem Handy
series-note-not-downloaded = Nicht geladen
series-note-downloading = Lädt { $percent } %
series-note-resume-at = Weiter bei { $time }
phone-episodes-estimate = { $count ->
    [one] { $count }? Folge
   *[other] { $count }? Folgen
}
phone-next-airs = Folge { $n } läuft in { $time }
phone-watched = Geschaut
phone-not-watched = Noch nicht geschaut
phone-progress = { $watched } von { $total } geschaut, { $count } auf dem Handy
phone-on-this-phone = { $count } auf diesem Handy
phone-play-film = Film abspielen
phone-resume-film = Film fortsetzen
phone-play-episode = Folge { $n } abspielen
phone-resume-episode = Folge { $n } fortsetzen
phone-about-date = ca. { $date }

## A show's list status, score, notes and fixes

show-status-watching = Schaue ich
show-status-planning = Geplant
show-status-completed = Abgeschlossen
show-status-paused = Pausiert
show-status-dropped = Abgebrochen
show-status-repeating = Erneut
show-status-now = { $label }, jetzt
show-add-to-list = Zur Liste
show-your-score = Deine Wertung { $score }
show-rate = Bewerten
show-fixes = Korrektur
show-your-notes = Deine Notizen
show-add-notes = Notiz anlegen
show-home-each = Ändere es bei jeder verbundenen Liste.
show-home-no-tracker = Nur auf diesem Handy gespeichert: Kein Tracker führt diese Serie.
show-home-tracker-lacks = Nur auf diesem Handy gespeichert: Dein Tracker führt diese Serie nicht.
show-home-until-connected = Nur auf diesem Handy gespeichert, bis du AniList oder MyAnimeList verbindest.
show-sites-both = AniList und MyAnimeList
show-notes-title = Notizen
show-notes-placeholder = Wo du aufgehört hast, wer es empfohlen hat, welches Release du holen sollst
show-notes-over = { $count ->
    [one] { $count } Zeichen über dem Limit
   *[other] { $count } Zeichen über dem Limit
}
show-notes-saved-to = Gespeichert bei { $sites }.
show-notes-saved-to-tracker = Bei deinem Tracker gespeichert.
show-notes-saving = Notiz wird gespeichert.
show-notes-clearing = Notiz wird gelöscht.
show-score-saving = Wertung wird gespeichert.
show-score-saved = Wertung gespeichert.
show-score-clearing = Wertung wird gelöscht.
show-score-cleared = Wertung gelöscht.
show-moving-to = Wechsel zu { $label }.
show-moved-to = Gewechselt zu { $label }.
show-rescan-started = Neuer Scan gestartet.
show-still-starting = AniBeam startet noch. Versuch es gleich nochmal.
show-fix-match = Zuordnen
show-fix-find-match = Zuordnung suchen
show-fix-rescan = Neu scannen

## A show's characters, related shows and recommendations

show-characters = Charaktere
show-related = Verwandt
show-related-all = Alle { $count } zeigen
show-related-filling = Wird noch von AniList nachgeladen.
show-recommendations = Empfehlungen
show-card-owned = In deiner Bibliothek
show-card-unreleased = Noch nicht erschienen
show-card-on-anilist = Auf AniList
show-rec-available = Vorhanden
show-relation-prequel = Prequel
show-relation-sequel = Sequel
show-relation-side-story = Nebenstory
show-relation-parent = Hauptserie
show-relation-spin-off = Spin-off
show-relation-adaptation = Adaption
show-relation-alternative = Alternative
show-relation-summary = Recap
show-relation-character = Charakter
show-relation-source = Quelle
show-relation-compilation = Zusammenschnitt
show-relation-contains = Enthält
show-relation-other = Sonstiges
show-role-main = Haupt
show-role-supporting = Neben
show-role-background = Hintergrund
show-format-tv = TV
show-format-tv-short = TV-Kurz
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Special
show-format-film = Film
show-entry-get = Folgen laden
show-entry-open-anilist = Auf AniList öffnen
show-entry-line = { $line }, { $status }

## Fix a wrong match

match-title = Zuordnen
match-refresh = Infos neu laden
match-clear = Zuordnung lösen
match-clear-it = Lösen
match-clear-warning = Das Lösen entfernt die Tracker-Verknüpfung und eine von Hand gesetzte Folgenzahl. Tippe nochmal zum Lösen.
match-tvmaze = TVmaze
match-search-anilist = AniList suchen
match-search-tvmaze = TVmaze suchen
match-paste-anilist = Link von AniList oder MyAnimeList einfügen
match-paste-tvmaze = Link von TVmaze oder IMDb einfügen
match-look-up = Suchen
match-apply = Übernehmen
match-look-up-all = Alle Ordner suchen, die AniBeam noch nicht kennt
match-format-music = Musik
match-join-and = { $first } und { $second }
match-matched-on = Zugeordnet bei { $sites }
match-found-tvmaze = Serie bei TVmaze gefunden
match-found-anilist = Bei AniList gefunden
match-found-anilist-season = Bei AniList gefunden, Staffel { $season }
match-found-mal = Bei MyAnimeList gefunden
match-searching = Suche { $name }
match-type-more = Gib mindestens zwei Buchstaben ein, um { $name } zu durchsuchen.
match-none = Keine Titel bei { $name } gefunden.
match-looking-up = Wird gesucht.
match-matched-to = Zugeordnet zu { $title }.
match-applied = Zuordnung übernommen.
match-applying = Zuordnung wird übernommen.
match-refreshing = { $title } wird aktualisiert.
match-clearing = Zuordnung wird gelöst.
match-cleared = Zuordnung gelöst.
match-auto-looking = Ordner, die AniBeam noch nicht kennt, werden gesucht.

## Batches (the line under a release in a search)

inspect-reading = Dateiliste wird gelesen
inspect-holds = { $count ->
    [one] { $count } Folge darin: { $runs }
   *[other] { $count } Folgen darin: { $runs }
}

## What's new

notes-title = Neuerungen
notes-got-it = Verstanden

## Move the list to a tracker

move-lists-label = Listen auf diesem Handy
move-title = Liste zu { $site } verschieben
move-here = Auf diesem Handy: { $list }
move-there = Jetzt bei { $site }: { $list }
move-later-hint = Später: Mehr, Konten
move-not-now = Nicht jetzt
move-nothing = Nichts
move-not-on-list = Nicht gelistet
move-nothing-left = Nichts mehr zu verschieben
move-side-status-score = { $status }, Wertung { $score }
move-side-score = Wertung { $score }
move-lead = { $count ->
    [one] Eine Serie hat nur auf diesem Handy einen Status oder eine Wertung. Beim Verschieben landen sie auf deiner { $site }-Liste, wo dein Computer und dein Profil sie sehen. Bei diesen Serien wird ersetzt, was { $site } jetzt hat. Gesendet wird erst, wenn du auf Verschieben tippst.
   *[other] { $count } Serien haben nur auf diesem Handy einen Status oder eine Wertung. Beim Verschieben landen sie auf deiner { $site }-Liste, wo dein Computer und dein Profil sie sehen. Bei diesen Serien wird ersetzt, was { $site } jetzt hat. Gesendet wird erst, wenn du auf Verschieben tippst.
}
move-label = { $count ->
    [one] { $count } Serie verschieben
   *[other] { $count } Serien verschieben
}
move-waiting = { $count ->
    [one] Eine Serie hat Status oder Wertung nur hier. Verschiebe sie zu { $site }, um sie auf deinem Computer und in deinem Profil zu sehen.
   *[other] { $count } Serien haben Status oder Wertung nur hier. Verschiebe sie zu { $site }, um sie auf deinem Computer und in deinem Profil zu sehen.
}
move-moving = { $count ->
    [one] { $count } Serie wird zu { $site } verschoben
   *[other] { $count } Serien werden zu { $site } verschoben
}
move-moved = { $count ->
    [one] { $count } Serie zu { $site } verschoben
   *[other] { $count } Serien zu { $site } verschoben
}
move-failed = { $count ->
    [one] { $site } hat { $count } Serie nicht übernommen; siehe Aktivitätsprotokoll
   *[other] { $site } hat { $count } Serien nicht übernommen; siehe Aktivitätsprotokoll
}

## Downloads: find (search, shows, sources, releases)

find-join-comma = { $a }, { $b }
find-join-and = { $a } und { $b }
find-join-sentences = { $a } { $b }

find-downloads = Downloads

find-lens-list = Deine Liste
find-lens-gaps = Lücken
find-lens-airing = Läuft
find-lens-following = Abos
find-lens-downloaded = Geladen
find-lens-with-count = { $lens } { $count }
find-what-to-get = Was laden
find-lens-list-line = Schaue ich und Geplant auf deiner Liste. Öffne eine Serie, um ihre Folgen zu laden.
find-lens-gaps-line = Ausgestrahlte Folgen, die nicht auf diesem Handy sind. Öffne eine Serie, um zu sehen, wer sie hat.
find-lens-airing-line = Neue Folgen, die noch kommen, die nächsten zuerst.
find-lens-following-line = Die Abos: Neue Folgen werden von selbst geladen. Öffne eins, um es zu beenden.
find-lens-downloaded-line = Was dieses Handy geladen hat, eine Serie nach der anderen.

find-lens-gaps-empty = Keine Lücken: Jede begonnene Serie hat ihre ausgestrahlten Folgen hier.
find-lens-airing-empty = Nichts auf diesem Handy läuft gerade.
find-lens-following-empty = Noch keine Abos. Öffne eine Serie und schalte Abonnieren ein.
find-lens-downloaded-empty = Noch nichts geladen.
find-list-empty = Noch nichts auf deiner Liste. Suche oben eine Serie und füge sie hinzu.
find-list-reading = Liste wird gelesen...
find-list-signed-out = Melde dich unter Mehr, Konten bei AniList an, dann erscheint deine Liste hier.
find-list-failed = Deine Liste konnte nicht gelesen werden: { $error }

find-aired = gelaufen
find-ledger-no-feed = keine neuen Folgen: Die Quelle hat keinen Feed
find-ledger-downloads = { $count ->
    [one] { $count } Download
   *[other] { $count } Downloads
}
find-ledger-downloaded = geladen
find-ledger-planning = Geplant
find-ledger-watching-of = Schaue ich, { $progress } von { $total }
find-ledger-watching-seen = Schaue ich, { $progress } gesehen

find-gap-missing = { $count } fehlen
find-gap-not-downloaded = { $count } nicht geladen

find-list-status-planning = Auf deiner Liste: Geplant
find-list-status-watching = Auf deiner Liste: Schaue ich
find-list-status-already = Schon auf deiner Liste
find-going-planning = { $title } wird auf Geplant gesetzt.
find-already-listed = { $title } ist schon auf deiner Liste.
find-plan-needs-tracker = Zur Liste hinzufügen braucht AniList oder MAL. Melde dich unter Mehr, Konten an.
find-plan-failed = Nicht zur Liste hinzugefügt: { $error }
find-show-gone = Diese Serie ist nicht mehr in der Bibliothek auf diesem Handy.

## Shows as you type

find-shows = Serien
find-suggesting = Frage AniList...
find-suggest-tap = Tippe auf eine, um zu sehen, wer sie hat. Enter sucht Release-Namen.
find-suggest-none = Keine Serie mit diesem Namen. Enter sucht stattdessen Release-Namen.
find-suggest-hint = Serien erscheinen beim Tippen. Enter sucht Release-Namen.
find-suggest-failed = AniList konnte nicht gefragt werden: { $error }
find-library-line = In deiner Bibliothek
find-library-line-none = In deiner Bibliothek, keine von { $total } auf diesem Handy
find-library-line-some = In deiner Bibliothek, { $here } von { $total } auf diesem Handy

## The show view

find-plan-to-watch = Vormerken
find-on-phone-count = { $count } auf diesem Handy
find-airing = Läuft
find-dropped = { $count ->
    [one] { $count } Release, das diese Serie nicht nennt, bleibt draußen.
   *[other] { $count } Releases, die diese Serie nicht nennen, bleiben draußen.
}

find-following = Abo
find-stop-following-ask = Abo von { $name } beenden?
find-unfollow-explain = Neue Folgen werden nicht mehr geladen. Was in seinem Ordner liegt, kann bleiben oder mitgehen: jeder Download dort, auch von Hand geholte Folgen.
find-stop-following = Abo beenden
find-stop-keep = Beenden, Folgen behalten
find-stop-delete = Beenden und Downloads seines Ordners löschen
find-rule-taken = { $downloads ->
    [one] { $downloads } Download
   *[other] { $downloads } Downloads
}, { $files ->
    [one] { $files } Datei
   *[other] { $files } Dateien
} in seinem Ordner
find-rule-paused = Es ist pausiert, also kommen keine neuen Folgen. { $taken }.
find-rule-no-feed = Seine Quelle hat keinen Feed, also kommen keine neuen Folgen. { $taken }.
find-rule-active = Neue Folgen werden von selbst geladen. { $taken }.
find-unfollowed-deleted = Abo von { $name } beendet und die Downloads in seinem Ordner gelöscht.
find-unfollowed-kept = Abo von { $name } beendet. Seine Folgen bleiben auf diesem Handy.
find-stop-failed = Beenden ist nicht durchgelaufen: { $error }

find-where = Woher laden
find-where-pick = Wähle eine Quelle, um zu sehen, was sie hat.
find-where-nothing = Nichts wird geladen, bis du es sagst.
find-source-idle = Antippen
find-source-trying = Probiere { $name }
find-source-searching = Sucht
find-source-silent = Antwortet nicht
find-no-sites-title = Noch keine Suchseiten
find-no-sites-body = Füge unter Mehr ein Repo mit Suchseiten oder einen eigenen Feed hinzu.
find-open-more = Mehr öffnen
find-pick-source-title = Quelle wählen
find-pick-source-body = Ihre Releases erscheinen hier als Raster, eine Folge pro Würfel.
find-releases = { $count ->
    [one] { $count } Release
   *[other] { $count } Releases
}
find-unreachable = { $host } war nicht erreichbar. Manche Netze und Länder sperren Release-Seiten; AniBeam umgeht Sperren nach Namen schon, also sperrt dieses Netz die Seite selbst, oder die Seite ist down. Ein VPN hilft, oder probiere eine andere Quelle.
find-the-source = Die Quelle

find-has = Was sie hat
find-has-note = Tippe auf eine Zeile, um alles zu nehmen, oder auf einzelne Folgen.
find-episodes-tag = Folgen
find-batch = Batch { $label }
find-row-meta = { common-episodes }  { $seeds } Seeds  { $size }
find-on-this-phone = Auf diesem Handy
find-on-computer = Auf { $name }

find-plan-no-releases = Diese Quelle hat keine Releases dieser Serie.
find-plan-nothing-picked = Nichts gewählt: Tippe auf eine Zeile oder ihre Würfel.
find-plan-line = { $episodes } von { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] { $count } Folge, die du schon hast, bleibt draußen. Schalte Ersetzen ein, um sie nochmal zu laden.
   *[other] Folgen, die du schon hast, bleiben draußen. Schalte Ersetzen ein, um sie nochmal zu laden.
}
find-plan-follow-note = Neue Folgen von { $query } werden von selbst geladen.
find-plan-on-desk = Sie werden auf { $name } in dessen Bibliothek geladen.
find-follow-new = Neue Folgen abonnieren
find-follow-new-note = Sie werden von selbst geladen, sobald sie laufen.
find-replace = Vorhandenes ersetzen
find-replace-note = Lädt die gewählten Folgen nochmal über die auf diesem Handy.
find-go = { $mode ->
    [replace] { common-episodes } laden und ersetzen
    [subscribe] { common-episodes } laden und abonnieren
    [both] { common-episodes } laden, ersetzen und abonnieren
   *[plain] { common-episodes } laden
}
find-go-pick = Folgen zum Laden wählen
find-every-release = Jedes Release dieser Quelle

## Download to

find-download-to = Laden auf
find-this-phone = Dieses Handy
find-your-computer = Dein Computer
find-desk-asking = { $name } wird gefragt...
find-desk-silent = { $name } antwortet nicht. Wähle Dieses Handy oder versuch es nochmal, wenn er an ist.
find-sending-to = Wird an { $name } gesendet...

## The release list

find-best-fit = Beste Wahl
find-on-disk = Auf Platte
find-get = Laden
find-get-all = Alle { $count } laden
find-subscribe = Abonnieren
find-back-step = Zurück
find-narrow-placeholder = Eingrenzen, oder -Wort zum Ausschließen
find-best-fit-searches = Beste Wahl sucht: { $query }
find-fact-batch = Batch
find-fact-seeds = { $count } Seeds
find-fact-via = über { $source }
find-searching-for = { $text } wird gesucht...
find-search-line = { $count ->
    [0] Nichts bei { $source } dafür
    [one] { $count } Release bei { $source }
   *[other] { $count } Releases bei { $source }
}{ $fell ->
    [1] , nachdem die erste Quelle nichts hatte
   *[other] { "" }
}{ $narrowed ->
    [1] , eingegrenzt auf { $token }
   *[other] { "" }
}.
find-refined = { find-releases }, { $delta } { $dir ->
    [down] weniger
   *[up] mehr
}.
find-refine-nothing-left = Dann bleibt nichts übrig.
find-refine-no-match = Kein Release hat das.
find-refine-empty = Gib zuerst etwas ein.
find-refine-no-change = Das ändert nichts.

## Getting and subscribing

find-getting = Lade { $what }.
find-subscribed = Abonniert: Neue Folgen werden von selbst geladen.
find-subscribed-desk = Auf { $name } abonniert: Neue Folgen werden dort von selbst geladen.
find-subscribed-as = Abonniert als { $name }: Neue Folgen werden von selbst geladen.
find-downloading = { $count ->
    [one] { $count } Release wird geladen.
   *[other] { $count } Releases werden geladen.
}
find-not-added = Nicht hinzugefügt: { $error }
find-not-subscribed = Nicht abonniert: { $error }
find-desk-refused = { $name } hat es nicht angenommen: { $error }
find-desk-not-subscribed = Nicht auf { $name } abonniert: { $error }

## Player

player-subtitles = Untertitel
player-audio = Audio
player-speed = Tempo
player-connecting = Verbinde
player-hold-speed = 2× Tempo
player-replay = Nochmal
player-unlock = Entsperren
player-next-stay = Bleiben
player-next-go = Weiter
player-jump-back = −10 s
player-jump-forward = +10 s
player-speed-now = Tempo { $speed }
player-skip-intro = Intro überspringen
player-skip-outro = Abspann überspringen
player-skipped-intro = Intro übersprungen
player-skipped-outro = Abspann übersprungen
player-undo = Rückgängig
player-start-over = Von vorn
player-resumed-at = Weiter bei { $time }
player-picture-failed = Das Bild konnte nicht starten. Der Ton läuft weiter.
player-file-failed = Diese Datei lief nicht: { $why }.
player-streaming-from = Stream von { $host } in { $height }p
player-mpv-no-start = mpv startete nicht
player-mpv-no-start-why = mpv ist nicht gestartet: { $why }
player-android-only = der Player läuft nur auf Android

## Player: delays

player-delay-title = Versatz
player-delay-note = Jeder Tipp verschiebt um 0,1 s. Bis zur nächsten Datei.
player-delay-seconds = { $value } s
player-sub-delay-now = Untertitelversatz { $value }
player-audio-delay-now = Audioversatz { $value }

## Player: cast

player-cast-playing-on = Läuft auf { $device }
player-cast-stop = Wiedergabe auf { $device } beenden
player-cast-searching = Suche nach Chromecasts und TVs
player-cast-to = Senden an
player-cast-tv = TV
player-cast-loading = Lädt
player-cast-playing = Läuft
player-cast-paused = Pausiert
player-cast-finished = Beendet
player-cast-stopped = Gestoppt

## Player: the tracker's word

player-rated = Bewertet
player-score-failed = Wertung nicht gespeichert
player-marked-watched = Als geschaut
player-tracked = Erfasst
player-not-tracked = Nicht erfasst
player-track-update-failed = Liste konnte nicht aktualisiert werden
player-not-tracked-why = Nicht erfasst: { $reason }.
player-track-ok-at = { $tracker } bei { $progress }
player-track-ok = { $tracker } ok
player-track-note = { $tracker }: { $note }
player-track-failed = { $tracker } konnte nicht aktualisiert werden
player-track-line = { $head }: { $parts }
player-refusal-hidden = Serie ausgeblendet
player-refusal-no-match = die Serie hat keine Tracker-Zuordnung
player-refusal-not-newer = der Tracker ist schon weiter als diese Folge
player-refusal-extra = das ist ein Extra
player-refusal-unmatched = die Datei ist keiner Folge zugeordnet
player-refusal-on-disk = Datei nur auf Platte

## Subtitles

subs-line-default = Größe 1×, einfache Dateien nehmen deinen Stil
subs-sample = Wo hast du es versteckt?
subs-size-place-title = Größe und Position
subs-size = Größe
subs-position = Position
subs-position-helper = 100 sitzt am unteren Rand. Kleinere Zahlen heben die Zeilen im Bild an.
subs-style-title = Stil
subs-styled = Gestaltete Untertitel
subs-styled-as-made = Gestaltete (ASS) Dateien sehen genau so aus, wie ihre Macher sie eingestellt haben.
subs-styled-mine = Dein Stil unten gilt für jede Datei. Signs und Karaoke können kaputtgehen.
subs-styled-size-only = Gestaltete Untertiteldateien behalten ihren Look und nehmen deine Größe. Einfache Dateien nehmen deinen Stil.
subs-mode-as-made = Original
subs-mode-size-only = Nur Größe
subs-mode-mine = Eigene
subs-font = Schrift
subs-font-bad = Gib einen Schriftnamen ein, etwa Noto Sans.
subs-bold = Fett
subs-colour = Textfarbe
subs-colour-bad = Gib eine Farbe wie #ffcc00 ein.
subs-outline = Umriss
subs-outline-bad-width = Die Breite ist eine Zahl, 0 oder mehr.
subs-outline-bad-colour = Gib die Farbe wie #000000 ein.
subs-outline-helper = Erst die Breite, dann die Farbe.
subs-shadow = Schatten
subs-shadow-bad = Eine Zahl, 0 oder mehr. 0 heißt kein Schatten.
subs-shadow-helper = Wie weit der Schatten fällt, 0 für keinen.
subs-box = Hintergrundbox
subs-box-helper = Eine dunkle Box hinter den Zeilen. 0 lässt sie weg.
subs-reset-title = Von vorn
subs-reset-helper = Setzt Größe, Schrift, Farben, Umriss, Box und Position auf die Ausgangswerte zurück.
subs-reset = Zurücksetzen

## Streaming

stream-quality-auto = Auto
stream-quality-original = Original
stream-app-starting = Die App startet noch
stream-off-on-host = Streaming ist auf { $host } aus. Schalte dort Stream episodes unter Settings, Phone app ein.
stream-cannot-stream = { $host } nutzt AniBeam { $version }, das noch nicht streamen kann. Aktualisiere es dort.
stream-no-session = Das andere Gerät hat keine Sitzung geliefert
stream-answer-unreadable = Die Antwort war nicht lesbar: { $why }
stream-answered-status = { $name } antwortete { $status }

## Stats

stats-nothing-yet = Noch nichts
stats-empty-title = Noch nichts geschaut
stats-empty-body = Schau eine Folge zu Ende, dann zählt sie hier: Stunden, Genres und die Saisons, die du geschaut hast.
stats-hours-caption = Std. geschaut
stats-finished-caption = Serien beendet
stats-busiest-caption = stärkster Monat
stats-episodes-caption = { $count ->
    [one] Folge
   *[other] Folgen
}
stats-by-month = Nach Monat
stats-top-genres = Top-Genres
stats-top-studios = Top-Studios
stats-seasons = Gesehene Saisons
stats-watched-most = Meist geschaut
stats-year-line = Das hast du { $year } geschaut
stats-eps = { $count ->
    [one] { $count } F
   *[other] { $count } F
}
stats-month-1 = Jan
stats-month-2 = Feb
stats-month-3 = Mär
stats-month-4 = Apr
stats-month-5 = Mai
stats-month-6 = Jun
stats-month-7 = Jul
stats-month-8 = Aug
stats-month-9 = Sep
stats-month-10 = Okt
stats-month-11 = Nov
stats-month-12 = Dez
stats-month-letter-1 = J
stats-month-letter-2 = F
stats-month-letter-3 = M
stats-month-letter-4 = A
stats-month-letter-5 = M
stats-month-letter-6 = J
stats-month-letter-7 = J
stats-month-letter-8 = A
stats-month-letter-9 = S
stats-month-letter-10 = O
stats-month-letter-11 = N
stats-month-letter-12 = D

## Backup and image cache

backup-title = Backup
backup-export = Bibliothek sichern
backup-export-helper = Eine Datei mit deinen Ordnern, Zuordnungen und dem, was du geschaut hast.
backup-private = Private Daten einschließen
backup-private-helper = Fügt Anmeldungen bei AniList und MyAnimeList, Verlauf und Einstellungen hinzu. Anmeldungen stehen im Klartext, behalte die Datei für dich.
backup-restore = Aus Backup wiederherstellen
backup-restore-helper = Liest ein Backup in diese Bibliothek. Was auf der Platte liegt, gilt; die Datei füllt den Rest auf.
backup-done = Gesichert. Die Datei liegt dort, wo du sie gespeichert hast.
backup-not-saved = Nicht gespeichert. Es wurde nichts geschrieben.
backup-starting = AniBeam startet noch. Versuch es gleich nochmal.
backup-images-title = Bildcache
backup-images-body = Poster, Banner und Porträts bleiben auf dem Handy, damit die Bibliothek offline läuft. Geleerte Bilder werden neu geladen, wenn Seiten sie brauchen.
backup-keep = Behalten
backup-removed = { $count ->
    [one] { $count } Bild entfernt
   *[other] { $count } Bilder entfernt
}
backup-images-none = Keine Bilder im Cache
backup-images-unmeasured = Der Bildcache ließ sich nicht messen
backup-images-retry-note = Versuch es gleich nochmal. Den Bildern selbst fehlt nichts.
backup-images-line = { $size } in { $count ->
    [one] { $count } Bild
   *[other] { $count } Bildern
}
backup-images-question = { $count ->
    [one] { $count } Bild entfernen?
   *[other] { $count } Bilder entfernen?
}

backup-sum-list = { $items }.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] { $count } Ordner hinzugefügt
   *[other] { $count } Ordner hinzugefügt
}
backup-sum-series = { $count ->
    [one] { $count } Serie erstellt
   *[other] { $count } Serien erstellt
}
backup-sum-matches = { $count ->
    [one] { $count } Zuordnung übernommen
   *[other] { $count } Zuordnungen übernommen
}
backup-sum-views = { $views ->
    [one] { $views } Wiedergabe
   *[other] { $views } Wiedergaben
} und { $episodes ->
    [one] { $episodes } fertige Folge
   *[other] { $episodes } fertige Folgen
} zusammengeführt
backup-sum-resume = { $count ->
    [one] { $count } Fortsetzpunkt zusammengeführt
   *[other] { $count } Fortsetzpunkte zusammengeführt
}
backup-sum-accounts = { $count ->
    [one] { $count } Listenkonto übernommen
   *[other] { $count } Listenkonten übernommen
}
backup-sum-fixes = { $count ->
    [one] { $count } Datei übernimmt eine Korrektur von dir.
   *[other] { $count } Dateien übernehmen eine Korrektur von dir.
}
backup-sum-missing = { $count ->
    [one] { $count } Ordner wurde auf diesem Handy nicht gefunden.
   *[other] { $count } Ordner wurden auf diesem Handy nicht gefunden.
}
backup-sum-left-out = Ausgelassen: { $fields }.

## Activity

activity-everything = Alles
activity-problems = Probleme
activity-copied = Kopiert
activity-clear-every-line = Alles leeren
activity-fold = Einklappen
activity-no-problems = Keine Probleme. Alles lief.
activity-nothing-yet = Noch nichts zu melden. Was AniBeam für dich tut, erscheint hier.
activity-time-today = %H:%M:%S
activity-time-other = %-d. %b %H:%M

## Dates and numbers (companion format.rs)

fmt-clock = %H:%M
fmt-day-month = %-d. %b
fmt-weekday-date-time = %a, %-d. %b %H:%M
fmt-weekday-date-year-time = %a, %-d. %b %Y %H:%M
fmt-stamp = %-d. %b %H:%M
fmt-decimal-point = ,

fmt-today-at = Heute, { $time }
fmt-tomorrow-at = Morgen, { $time }
fmt-yesterday-at = Gestern, { $time }

fmt-span-days = { $d } T { $h } Std.
fmt-span-hours = { $h } Std. { $m } Min.
fmt-span-minutes = { $m } Min. { $s } s
fmt-span-seconds = { $s } s
fmt-in = in { $span }
fmt-full-days = { $d } T { $h } Std. { $m } Min. { $s } s
fmt-full-hours = { $h } Std. { $m } Min. { $s } s
fmt-full-minutes = { $m } Min. { $s } s

fmt-ago-now = gerade eben
fmt-ago-minutes = vor { $count } Min.
fmt-ago-hours = vor { $count } Std.
fmt-ago-days = vor { $count } T

fmt-bytes-b = { $value } B
fmt-bytes-kb = { $value } KB
fmt-bytes-mb = { $value } MB
fmt-bytes-gb = { $value } GB
fmt-bytes-tb = { $value } TB

## Talking to the computer (companion client.rs, sync.rs, address.rs)

client-not-found = { $host } ist kein Name, den das Handy finden kann. Ist Tailscale verbunden?
client-timeout = { $host } antwortet nicht. Ist er an, und ist Tailscale auf beiden verbunden?
client-connect-failed = { $host } war nicht erreichbar. Ist er an, und ist Tailscale auf beiden verbunden?
client-refused = { $host } ist an, aber AniBeam dort ist geschlossen oder gibt nichts frei. Öffne AniBeam dort und schalte Share over Tailscale unter Settings, Phone app ein.
client-no-route = Kein Weg zu { $host }. Ist Tailscale verbunden?
client-invalid-input = Verbindung zu { $host } nicht möglich
client-unreachable = { $host } nicht erreichbar: { $why }
client-cannot-find = Das Handy findet { $host } nicht. Ist Tailscale verbunden?
client-broke-off = Die Antwort von { $host } brach ab: { $why }
client-answered = { $host } antwortete { $status } für { $path }
client-not-anibeam-status = Etwas hat geantwortet, aber nicht AniBeam ({ $host } antwortete { $status } für { $path })
client-not-anibeam-hello = Etwas hat geantwortet, aber nicht AniBeam (die Antwort ist nicht das Hallo von AniBeam)
client-not-anibeam-name = Etwas hat geantwortet, aber nicht AniBeam (es nennt sich { $name })
client-too-new = Der Host spricht API { $host_api }; diese App liest { $app_api }. Aktualisiere die App.
client-snapshot-unreadable = Der Snapshot war nicht lesbar: { $why }
client-probe-unreadable = Die Details der Datei waren nicht lesbar: { $why }
client-peers-unreadable = Die Liste der AniBeams war nicht lesbar: { $why }
sync-save-failed = Snapshot konnte nicht gespeichert werden: { $why }
address-https = Das AniBeam deines Computers antwortet über Tailscale auf einfachem http; lass das https:// weg.
address-empty = Gib Namen oder Adresse des Hosts ein, zum Beispiel my-computer:8790.
address-ipv6-close = Eine IPv6-Adresse braucht das schließende ].
address-ipv6-port = Setze den Port hinter das ], etwa [fd7a::1]:8790.
address-bad-host = „{ $text }“ ist kein Hostname.
address-bad-port = „{ $port }“ ist kein Port.

## Series facts (companion library.rs)

clib-status-airing = Läuft
clib-status-not-yet = Kommt noch
clib-format-tv-short = TV-Kurz
clib-format-movie = Film
clib-format-special = Special
clib-format-music = Musik

## Search sites and repos

repos-by-hand-title = Von Hand hinzufügen
repos-by-hand-note = Für eine Seite, die kein Repo führt. Jeder Feed mit { "{query}" } dort, wo die Suche hin soll, auch Jackett- oder Prowlarr-Feeds. Ein Schlüssel oder Token in der Adresse wird in den Geheimspeicher dieses Handys verschoben.
repos-name-placeholder = Name
repos-url-placeholder = https://...{ "{query}" }...
repos-index-placeholder = https://.../index.json
repos-add-site = Hinzufügen
repos-add-title = Repo hinzufügen
repos-add-note = Füge die Adresse des Repo-Index ein. AniBeam prüft die Signatur und zeigt dir den Schlüssel, bevor etwas gespeichert wird.
repos-add-repo = Hinzufügen
repos-trust-title = Diesem Repo vertrauen?
repos-fingerprint = Fingerabdruck
repos-trust-warning = Seine Seiten bestimmen, wohin deine Suchen gehen. Vertraue nur Repos von Leuten, denen du vertraust.
repos-trust = Vertrauen
repos-open-link = { $url } öffnen
repos-saving = Speichert...
repos-install = Installieren
repos-update = Update
repos-edit-note = Ein Schlüssel oder Token in der Adresse wird in den Geheimspeicher dieses Handys verschoben.
repos-edit-note-keys = Lass jedes { "{key}" } in der Adresse stehen; sein Wert bleibt auf diesem Handy.
repos-change-its = { $keys } ändern
repos-check-repos = Repos auf neue Seiten prüfen
repos-check-sites = Seiten prüfen
repos-add-site-label = Seite von Hand hinzufügen
repos-needs-before-search = Braucht { $labels }, bevor es sucht
repos-no-longer-offered = Wird vom Repo nicht mehr angeboten
repos-update-host = Das Update sendet Anfragen an { $host }
repos-tap-change-custom = Tippen, um Name oder Adresse zu ändern
repos-tap-change-keys = Tippen, um { $labels } zu ändern
repos-line-custom = { $host }  { $kind }  von dir hinzugefügt
repos-update-to = Update auf v{ $version }
repos-add-its-value = Wert hinzufügen
repos-asks-for = Fragt nach { $labels }
repos-preview-line = { $count ->
    [one] { $count } Seite, signiert
   *[other] { $count } Seiten, signiert
}
repos-preview-trusted = { $count ->
    [one] { $count } Seite, signiert, schon vertraut
   *[other] { $count } Seiten, signiert, schon vertraut
}
repos-ask-kept-empty = Nur auf diesem Handy gespeichert. Ein leeres Feld behält den gespeicherten Wert.
repos-ask-kept-never = Nur auf diesem Handy gespeichert, nie in der Bibliothek oder einem Export.
repos-ask-new-host = Die neue Version sendet das an { $host }, darum fragt sie erneut.
repos-ask-new = Die neue Version fragt erneut danach.
repos-type-value = Gib zuerst einen Wert ein.
repos-health-answered = Antwort in { $ms } ms
repos-health-failed = Keine Antwort. { $step }: { $detail }

## Downloads

dl-title = Downloads
dl-wifi-only = Nur WLAN
dl-wifi-and-mobile = WLAN und mobile Daten
dl-joined = { $first }, { $second }
dl-batch-files = { $count ->
    [one] Batch, { $count } Datei
   *[other] Batch, { $count } Dateien
}
dl-batch-tag = Batch
dl-downloading-count = { $count ->
    [one] { $count } lädt
   *[other] { $count } laden
}
dl-free-on-phone = frei auf dem Handy
dl-search-placeholder = Eine Serie oder ein Release-Name
dl-sources = Quellen
dl-downloading = Lädt
dl-queue-note = { $count ->
    [one] { $count } Folge
   *[other] { $count } Downloads
}
dl-finished = Fertig
dl-finished-note = { $count ->
    [one] { $count } Download, in deiner Bibliothek
   *[other] { $count } Downloads, in deiner Bibliothek
}
dl-get = Laden
dl-pick = Wählen

## Storage and hidden shows

disk-on-phone = Auf diesem Handy
disk-watched = Geschaut
disk-not-watched = Nicht geschaut
disk-deleting = Löscht
disk-delete-watched = Geschaute löschen
disk-hidden-shows = Ausgeblendet
disk-hidden-helper = Weg von Start und Bibliothek, bis du sie zurückholst.
disk-unhide = Einblenden
disk-delete-note = Dateien und Untertitel werden endgültig von diesem Handy gelöscht.
disk-every-watched = Jede geschaute Folge
disk-every-finished = Jede fertige Serie, die du geschaut hast
disk-episodes-size = { $count ->
    [one] { $count } Folge, { $size }
   *[other] { $count } Folgen, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } Serie, { $size }
   *[other] { $count } Serien, { $size }
}
disk-list-two = { $a } und { $b }
disk-list-three = { $a }, { $b } und { $c }
disk-list-more = { $a }, { $b }, { $c } und { $count } weitere
disk-delete-finished-title = { $count ->
    [one] { $count } fertige Serie
   *[other] { $count } fertige Serien
} löschen, { $size }?
disk-delete-watched-title = { $count ->
    [one] { $count } geschaute Folge
   *[other] { $count } geschaute Folgen
} löschen, { $size }?
disk-delete-finished-body = { $names }. Nur Serien, die der Anbieter als beendet führt und von denen jede Folge geschaut ist; eine Serie, die noch läuft oder deren Ende geschätzt ist, bleibt. Jede Datei davon wird von diesem Handy gelöscht, Extras auch. Ihre Downloads verschwinden ebenfalls, sie lassen sich also neu laden.
disk-delete-watched-body = Jede Folge, die du geschaut hast, in jeder Serie. Noch nicht geschaute Folgen und Extras bleiben. Die Dateien und ihre Untertitel werden endgültig von diesem Handy gelöscht.
disk-no-finished = Keine beendete Serie ist ganz geschaut
disk-nothing-watched = Nichts Geschautes ist auf diesem Handy
disk-deleted = { $count ->
    [one] { $count } Folge
   *[other] { $count } Folgen
} gelöscht, { $size }
disk-delete-failed = Nichts wurde gelöscht: { $error }

## Download notifications

notify-card-head = AniBeam · jetzt
notify-sample-called = Frieren Folge 7
notify-sample-called-ready = Frieren Folge 6
notify-sample-facts = 420 MB von 1,4 GB, 3,2 MB/s, noch 5 Min.
notify-downloading = Lädt { $called }
notify-progress = { $percent } %, { $facts }
notify-progress-of = { $percent } %, { $done } von { $size }
notify-starting = Startet
notify-many = { $count ->
    [one] { $count } Download
   *[other] { $count } Downloads
}
notify-called-episode = { $show } Folge { $n }
notify-called-episodes = { $show } Folgen { $from } bis { $to }
notify-ready = { $called } ist bereit
notify-ready-many = { $called } sind bereit
notify-ready-batch = Ein Batch von { $show } ist bereit
notify-ready-text = Auf dieses Handy geladen. Tippe, um AniBeam zu öffnen.
notify-channel-downloads = Downloads
notify-channel-ready = Folgen bereit
notify-fallback-title = Lädt
notify-row-on = Benachrichtigungen
notify-row-allow = Benachrichtigungen erlauben
notify-row-open-settings = Benachrichtigungseinstellungen öffnen
notify-helper-on = An. Downloads erscheinen in deinen Benachrichtigungen, solange sie laufen.
notify-helper-asks-next = Android fragt gleich.
notify-helper-asks-next-later = Android fragt als Nächstes. Du kannst es später unter Mehr ändern.
notify-helper-off = Aus. Schalte dort Benachrichtigungen für AniBeam ein.
notify-question-title = Downloads in den Benachrichtigungen sehen?
notify-question-body = Während Folgen laden, zeigt eine Benachrichtigung den Fortschritt, eine weitere sagt, wann eine bereit ist. Geladen wird so oder so; das hier entscheidet nur, ob du es siehst.
notify-allow = Erlauben
notify-not-now = Nicht jetzt
notify-pointer = Downloads erscheinen nicht in deinen Benachrichtigungen.
notify-turn-on = Einschalten

## Feed

feed-title = Feed
feed-recently-released = Kürzlich erschienen
feed-coming-soon = Bald
feed-this-week = Diese Woche
feed-earlier = Früher
feed-everything-else = Alles andere
feed-countdown-days = { $d } T { $h } Std.
feed-countdown-hours = { $h } Std. { $m } Min.
feed-countdown-minutes = { $m } Min.
feed-next = Als Nächstes: { $title } in { $countdown }
feed-nothing-scheduled = Noch nichts geplant.
feed-nothing-new = Diese Woche nichts Neues.
feed-new-episodes = { $count ->
    [one] { $count } neue Folge diese Woche.
   *[other] { $count } neue Folgen diese Woche.
}
feed-aired = lief { $ago }
feed-downloaded = geladen { $ago }

## Files and the file picker

files-already-picking = Es wird schon eine Datei gewählt.
files-took-too-long = Die Dateiauswahl hat zu lange gebraucht.
files-did-not-open = Die Dateiauswahl ließ sich nicht öffnen: { $error }
files-none-here = Hier gibt es keine Dateiauswahl.

## The computer's downloads

desk-sent-adding = { $what } an { $name } gesendet. Es wird noch hinzugefügt.
desk-sent = Lade { $what } auf { $name }.

## Your computer's library

remote-watched = Geschaut
remote-movie = Film
remote-not-watched-yet = Noch nicht geschaut
remote-watched-of = { $done } von { $total } geschaut
remote-watched-count = { $count } geschaut
remote-on-disk = { $count } auf Platte
remote-airs-in-season = S{ $season } Folge { $n } läuft in { $countdown }
remote-airs-in = Folge { $n } läuft in { $countdown }
remote-play-episode = Folge { $n } abspielen
remote-watch-again = Folge { $n } nochmal schauen
remote-update-host = Aktualisiere AniBeam auf diesem Computer, um das hier abzuspielen
remote-aired = gelaufen
remote-missing = Fehlt
remote-on-computer = Auf deinem Computer
remote-out = Erschienen { $ago }
remote-season-episode = Staffel { $season }, Folge { $n }
