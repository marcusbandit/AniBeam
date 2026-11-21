# de translation of en/core.ftl. An ID missing here is shown in English.

## Namen, die in jeder Sprache gleich bleiben. Sie stehen nur in der englischen Datei.

## Der Tailnet-Host

tailnet-person-no-name = Kein Name in Tailscale

## Anzahlen

count-episodes =
    { $count ->
        [one] { $count } Folge
       *[other] { $count } Folgen
    }
count-shows =
    { $count ->
        [one] { $count } Serie
       *[other] { $count } Serien
    }

## Aktionen, die jeder Bildschirm nutzt

action-cancel = Abbrechen
action-close = Schließen
action-try-again = Nochmal
action-remove = Entfernen
action-delete = Löschen
action-open = Öffnen
action-save = Speichern
action-apply = Übernehmen
action-clear = Leeren
action-refresh = Aktualisieren
action-play = Abspielen
action-connect = Verbinden
action-start = Starten
action-back = Zurück
action-see-all = Alle zeigen
action-open-series = Zur Serie
action-open-settings = Einstellungen öffnen

## Die Listen eines Trackers

status-watching = Schaue ich
status-completed = Abgeschlossen
status-paused = Pausiert
status-on-hold = Pausiert
status-dropped = Abgebrochen
status-planning = Geplant
status-rewatching = Erneut

## Folgen

episode-number = Folge { $number }

## AniLists Wörter für eine Serie: Format, Ausstrahlung, Beziehung zweier Serien

anilist-format-tv = TV
anilist-format-tv-short = TV-Kurz
anilist-format-film = Film
anilist-format-special = Special
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Musik
anilist-status-finished = Beendet
anilist-status-airing = Läuft
anilist-status-not-yet-aired = Noch nicht gelaufen
anilist-status-cancelled = Abgesetzt
anilist-status-hiatus = Pausiert
anilist-relation-adaptation = Adaption
anilist-relation-prequel = Prequel
anilist-relation-sequel = Sequel
anilist-relation-parent = Hauptserie
anilist-relation-side-story = Nebenstory
anilist-relation-character = Charakter
anilist-relation-summary = Recap
anilist-relation-alternative = Alternative
anilist-relation-spin-off = Spin-off
anilist-relation-other = Sonstiges
anilist-relation-source = Quelle
anilist-relation-compilation = Zusammenschnitt
anilist-relation-contains = Enthält

## Das Aktivitätsprotokoll: ein Problem ohne eigene Worte

event-problem-library = Mit deiner Bibliothek ist etwas schiefgegangen. Details stehen in den Entwickleroptionen.
event-problem-metadata = Mit den Serieninfos ist etwas schiefgegangen. Details stehen in den Entwickleroptionen.
event-problem-trackers = Mit deinen Listen ist etwas schiefgegangen. Details stehen in den Entwickleroptionen.
event-problem-franchise = Mit verwandten Serien ist etwas schiefgegangen. Details stehen in den Entwickleroptionen.
event-problem-playback = Bei der Wiedergabe ist etwas schiefgegangen. Details stehen in den Entwickleroptionen.
event-problem-store = Beim Speichern deiner Daten ist etwas schiefgegangen. Details stehen in den Entwickleroptionen.
event-problem-system = In Anibeam ist etwas schiefgegangen. Details stehen in den Entwickleroptionen.
event-problem-downloads = Bei den Downloads ist etwas schiefgegangen. Details stehen in den Entwickleroptionen.

## Fehler (contract::CoreError)

core-error-not-found-source = Ordner { $id } nicht gefunden.
core-error-not-found-series = Serie { $id } nicht gefunden.
core-error-not-found-file = Datei { $id } nicht gefunden.
core-error-not-found-session = Wiedergabesitzung { $id } nicht gefunden.
core-error-not-found-job = Aufgabe { $id } nicht gefunden.
core-error-invalid = { $field } ist ungültig: { $message }.
core-error-invalid-ended = { $field } ist ungültig: { $message }
core-error-unavailable = Dieser Ordner ist nicht verfügbar: { $path }
core-error-not-connected = { $tracker } ist nicht verbunden.
core-error-refused-hidden = Geht nicht: Die Serie ist ausgeblendet.
core-error-refused-no-match = Geht nicht: Die Serie ist noch nicht zugeordnet; ordne sie unter Serieninfo zu.
core-error-refused-not-newer = Geht nicht: Deine Liste ist schon weiter als diese Folge.
core-error-refused-extra = Geht nicht: Das ist ein Extra.
core-error-refused-unmatched = Geht nicht: Die Datei ist keiner Folge zugeordnet.
core-error-refused-on-disk = Geht nicht: Die Datei liegt nur auf der Platte.
# Why a list was not updated, read after "Not tracked:" or a tracker's name, on every shell's player
refusal-hidden = die Serie ist ausgeblendet
refusal-no-match = die Serie ist noch nicht zugeordnet; ordne sie unter Serieninfo zu
refusal-not-newer = deine Liste ist schon weiter als diese Folge
refusal-extra = das ist ein Extra
refusal-unmatched = die Datei ist keiner Folge zugeordnet
refusal-on-disk = die Datei liegt nur auf der Platte
core-error-provider = { $provider } hat nicht geantwortet: { $message }
core-error-io = Eine Datei konnte nicht gelesen oder geschrieben werden: { $message }
core-error-storage = Deine Bibliothek konnte nicht gespeichert werden: { $message }
core-error-keyring = Der Passwortspeicher des Systems war nicht erreichbar: { $message }
core-error-unsupported = { $what } wird noch nicht unterstützt.
core-error-version = Dieser Export stammt aus einem neueren Anibeam (Format { $found }). Aktualisiere Anibeam, um ihn zu importieren.
core-error-internal = Interner Fehler in Anibeam: { $message }
core-error-service = { $service }: { $message }

## releases/src/repos/mod.rs

core-repos-paste-address = füge die Adresse einer Quellenliste ein
core-repos-must-be-https = die Adresse muss https verwenden
core-repos-unreachable = Quellenliste nicht erreichbar: { $message }
core-repos-not-found = unter dieser Adresse wurde nichts gefunden (ein privates GitHub-Repo liefert ebenfalls 404)
core-repos-http-status = die Quellenliste lieferte HTTP { $status }
core-repos-unreadable = { $path } konnte nicht gelesen werden: { $error }
core-repos-no-such-endpoint = keine installierte Quelle mit dieser ID
core-repos-key-changed = diese Quellenliste ist jetzt mit einem anderen Schlüssel signiert als dem, dem du vertraut hast. Entferne sie und füge sie neu hinzu, um den neuen Schlüssel anzunehmen.
core-repos-stale = die Quellenliste lieferte einen älteren Index (Serial { $serial } nach { $kept }); der neuere bleibt
core-repos-key-not-shown = der Signaturschlüssel passt nicht zu dem, der dir gezeigt wurde. Füge die Quellenliste neu hinzu und vergleiche den Fingerabdruck.
core-repos-repo-added = Quellenliste hinzugefügt.
core-repos-repo-removed = Quellenliste entfernt.
core-repos-no-field = { $name } hat kein Feld { $key }
core-repos-needs-field = { $name } braucht { $label }
core-repos-host-only = { $name }: { $label } darf nur ein Host oder Host:Port sein, etwa 192.0.2.10:9696
core-repos-not-added = diese Quellenliste wurde nicht hinzugefügt
core-repos-not-offered = { $repo } bietet { $id } nicht an
core-repos-already-installed = { $name } ist schon installiert
core-repos-taken-by-you = eine Quelle namens { $id } ist schon installiert (von Hand hinzugefügt); entferne sie zuerst
core-repos-taken-by-repo = eine Quelle namens { $id } ist schon installiert (aus einer anderen Quellenliste); entferne sie zuerst
core-repos-source-added = Eine Downloadquelle hinzugefügt.
core-repos-source-updated = Eine Downloadquelle aktualisiert.
core-repos-source-removed = Eine Downloadquelle entfernt.
core-repos-source-changed = Downloadquelle bearbeitet.
core-repos-give-name = gib einen Namen ein
core-repos-put-query = setze {"{"}query{"}"} dort in die Adresse, wo der Suchbegriff hingehört
core-repos-name-taken = eine Quelle namens { $id } ist schon installiert
core-repos-only-by-hand = nur von Hand hinzugefügte Quellen lassen sich bearbeiten
core-repos-keep-key = lass {"{"}{ $key }{"}"} in der Adresse und ändere den Wert über { $label } ändern, oder entferne die Seite und füge sie ohne neu hinzu
core-repos-note-kind = anirss kann { $kind }-Quellen nicht durchsuchen, also überspringt der Desktop sie; die Handy-App kann sie durchsuchen
core-repos-note-by-hand = anirss hat ein eigenes, von Hand eingerichtetes { $id }; der Desktop durchsucht dieses
core-repos-note-theirs = Diese anirss-Instanz verwaltest du selbst, also ändert { -app-name } ihre Konfiguration nicht: Trag { $id } in die Konfiguration von anirss ein, um es am Desktop zu durchsuchen
core-repos-note-sandbox = Im Sandbox-Modus wird nicht in die Konfiguration von anirss geschrieben, also durchsucht der Desktop { $id } hier nicht
core-repos-note-old = anirss { $version } unterstützt keine Quellen von { -app-name }; aktualisiere auf { $min } oder neuer, um das am Desktop zu durchsuchen

## releases/src/repos/index.rs

core-repos-key-not-ed25519 = der Index-Schlüssel ist kein ed25519-Schlüssel
core-repos-key-not-base64 = der Index-Schlüssel sind keine 32 Byte Base64
core-repos-key-invalid = der Index-Schlüssel ist kein gültiger ed25519-Schlüssel
core-repos-too-large = die Datei ist zu groß für eine Quellenliste
core-repos-not-a-repo = das ist keine gültige Quellenliste für { -app-name }
core-repos-format-version = diese Quellenliste nutzt Formatversion { $found }, aber { -app-name } liest Version { $reads }. Aktualisiere { -app-name }.
core-repos-payload-not-base64 = die Nutzdaten der Quellenliste sind kein Base64
core-repos-signature-malformed = die Signatur der Quellenliste ist fehlerhaft
core-repos-index-unreadable = der Index der Quellenliste ist nicht lesbar: { $error }
core-repos-signature-mismatch = die Signatur der Quellenliste passt nicht zu ihrem Inhalt
core-repos-check-scheme = { $id }: die Adresse muss mit https:// beginnen
core-repos-check-scheme-http = { $id }: die Adresse muss mit http:// oder https:// beginnen
core-repos-check-auth-key = { $id }: ungültiger Auth-Schlüssel { $key }
core-repos-check-host-torznab = { $id }: nur eine torznab-Adresse darf {"{"}{ $key }{"}"} als Host verwenden
core-repos-check-host-placeholder = { $id }: im Host ist kein Platzhalter erlaubt

## playback/src/tracks.rs

core-playback-track-number = Spur { $number }
core-playback-track-file = { $label } (Datei)
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = Englisch
core-playback-lang-japanese = Japanisch
core-playback-lang-spanish = Spanisch
core-playback-lang-portuguese = Portugiesisch
core-playback-lang-french = Französisch
core-playback-lang-german = Deutsch
core-playback-lang-italian = Italienisch
core-playback-lang-russian = Russisch
core-playback-lang-chinese = Chinesisch
core-playback-lang-korean = Koreanisch
core-playback-lang-arabic = Arabisch
core-playback-lang-hindi = Hindi
core-playback-lang-indonesian = Indonesisch
core-playback-lang-malay = Malaiisch
core-playback-lang-thai = Thailändisch
core-playback-lang-vietnamese = Vietnamesisch
core-playback-lang-polish = Polnisch
core-playback-lang-turkish = Türkisch
core-playback-lang-dutch = Niederländisch
core-playback-lang-swedish = Schwedisch
core-playback-lang-norwegian = Norwegisch
core-playback-lang-danish = Dänisch
core-playback-lang-finnish = Finnisch
core-playback-lang-hebrew = Hebräisch
core-playback-lang-ukrainian = Ukrainisch
core-playback-lang-czech = Tschechisch
core-playback-lang-hungarian = Ungarisch
core-playback-lang-greek = Griechisch
core-playback-lang-romanian = Rumänisch
core-playback-lang-filipino = Filipino
core-playback-region-brazil = Brasilien
core-playback-region-portugal = Portugal
core-playback-region-latin-america = Lateinamerika
core-playback-region-spain = Spanien
core-playback-region-simplified = Vereinfacht
core-playback-region-traditional = Traditionell

## contract/src/enums.rs

core-jobs-scan = Prüfe deine Ordner
    .stopped = Prüfen deiner Ordner wegen eines Problems gestoppt.
core-jobs-auto-match = Ordne Serien zu
    .stopped = Zuordnen der Serien wegen eines Problems gestoppt.
core-jobs-search = Suche eine Serie
    .stopped = Suche nach einer Serie wegen eines Problems gestoppt.
core-jobs-resolve-link = Lese einen Link
    .stopped = Lesen eines Links wegen eines Problems gestoppt.
core-jobs-apply-match = Ordne eine Serie zu
    .stopped = Zuordnen einer Serie wegen eines Problems gestoppt.
core-jobs-refresh = Aktualisiere eine Serie
    .stopped = Aktualisieren einer Serie wegen eines Problems gestoppt.
core-jobs-refresh-all = Aktualisiere Serieninfos
    .stopped = Aktualisieren der Serieninfos wegen eines Problems gestoppt.
core-jobs-refresh-airing = Prüfe, was läuft
    .stopped = Prüfen, was läuft, wegen eines Problems gestoppt.
core-jobs-refresh-imdb = Aktualisiere Bewertungen
    .stopped = Aktualisieren der Bewertungen wegen eines Problems gestoppt.
core-jobs-clear-images = Lösche Bilder
    .stopped = Löschen der Bilder wegen eines Problems gestoppt.
core-jobs-fill-images = Lade Bilder
    .stopped = Laden der Bilder wegen eines Problems gestoppt.
core-jobs-connect-tracker = Verbinde deine Liste
    .stopped = Verbinden deiner Liste wegen eines Problems gestoppt.
core-jobs-mark = Markiere eine Folge als geschaut
    .stopped = Markieren einer Folge als geschaut wegen eines Problems gestoppt.
core-jobs-set-progress = Setze deinen Fortschritt
    .stopped = Setzen deines Fortschritts wegen eines Problems gestoppt.
core-jobs-score = Speichere deine Wertung
    .stopped = Speichern deiner Wertung wegen eines Problems gestoppt.
core-jobs-refresh-progress = Lese deinen Listenfortschritt
    .stopped = Lesen deines Listenfortschritts wegen eines Problems gestoppt.
core-jobs-refresh-watching = Lese deine Liste Schaue ich
    .stopped = Lesen deiner Liste Schaue ich wegen eines Problems gestoppt.
core-jobs-refresh-airing-now = Prüfe diese Saison
    .stopped = Prüfen dieser Saison wegen eines Problems gestoppt.
core-jobs-refresh-recommended = Suche Serien für dich
    .stopped = Suche nach Serien für dich wegen eines Problems gestoppt.
core-jobs-read-show-page = Lese eine Serie
    .stopped = Lesen einer Serie wegen eines Problems gestoppt.
core-jobs-on-hold = Setze ruhende Serien auf Pausiert
    .stopped = Setzen ruhender Serien auf Pausiert wegen eines Problems gestoppt.
core-jobs-set-status = Ändere einen Listenstatus
    .stopped = Ändern eines Listenstatus wegen eines Problems gestoppt.
core-jobs-move-lists = Verschiebe deine Liste zu deinem Tracker
    .stopped = Verschieben deiner Liste zum Tracker wegen eines Problems gestoppt.
core-jobs-notes = Speichere deine Notizen
    .stopped = Speichern deiner Notizen wegen eines Problems gestoppt.
core-jobs-crawl = Suche verwandte Serien
    .stopped = Suche nach verwandten Serien wegen eines Problems gestoppt.
core-jobs-skip-windows = Suche Intro und Abspann
    .stopped = Suche nach Intro und Abspann wegen eines Problems gestoppt.
core-jobs-export = Sichere deine Bibliothek in eine Datei
    .stopped = Sichern deiner Bibliothek wegen eines Problems gestoppt.
core-jobs-import = Lade deine Bibliothek aus einer Datei
    .stopped = Laden deiner Bibliothek aus einer Datei wegen eines Problems gestoppt.
core-jobs-subscriptions = Prüfe deine Abos
    .stopped = Prüfen deiner Abos wegen eines Problems gestoppt.
core-jobs-connect-downloads = Verbinde die Download-App
    .stopped = Verbinden der Download-App wegen eines Problems gestoppt.
core-jobs-search-releases = Suche Folgen
    .stopped = Suche nach Folgen wegen eines Problems gestoppt.
core-jobs-refine-releases = Grenze Folgen ein
    .stopped = Eingrenzen der Folgen wegen eines Problems gestoppt.
core-jobs-subscribe = Abonniere eine Serie
    .stopped = Abonnieren einer Serie wegen eines Problems gestoppt.
core-jobs-download = Füge Downloads hinzu
    .stopped = Hinzufügen der Downloads wegen eines Problems gestoppt.
core-jobs-remove-subscription = Entferne ein Abo
    .stopped = Entfernen eines Abos wegen eines Problems gestoppt.
core-jobs-control-torrents = Ändere deine Downloads
    .stopped = Ändern deiner Downloads wegen eines Problems gestoppt.
core-jobs-test-torrent-client = Teste die Download-App
    .stopped = Testen der Download-App wegen eines Problems gestoppt.
core-jobs-remove-show = Entferne eine Serie
    .stopped = Entfernen einer Serie wegen eines Problems gestoppt.
core-jobs-repair-subscription = Repariere ein Abo
    .stopped = Reparieren eines Abos wegen eines Problems gestoppt.
core-jobs-check-endpoints = Prüfe deine Downloadquellen
    .stopped = Prüfen deiner Downloadquellen wegen eines Problems gestoppt.
core-jobs-start-endpoint = Starte eine Downloadquelle
    .stopped = Starten einer Downloadquelle wegen eines Problems gestoppt.
core-jobs-schedule-agent = Frage deinen Coding-Agent nach einem Zeitplan
    .stopped = Frage an deinen Coding-Agent nach einem Zeitplan wegen eines Problems gestoppt.

## metadata/src/airing_now.rs

core-airing-now-now = Läuft jetzt
core-airing-now-winter = Winter { $year }
core-airing-now-spring = Frühling { $year }
core-airing-now-summer = Sommer { $year }
core-airing-now-fall = Herbst { $year }
core-airing-now-planned = { $title } steht jetzt auf deiner Liste Geplant bei { $site }.
core-airing-now-planned-both = { $title } steht jetzt auf deiner Liste Geplant bei { $site } und { $other }.
core-airing-now-already-on =
    { $title } steht schon auf deiner Liste bei { $site }, als { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }.
core-airing-now-plan-failed = { $title } konnte nicht auf deine Liste Geplant gesetzt werden. Versuch es später nochmal.

## metadata/src/aired.rs

core-aired-episode-out = Folge { $number } von { $title } ist da.

## playback/src/cast/mod.rs

core-cast-connecting = Sende an { $device }.
core-cast-ended = Senden an { $device } beendet.
core-cast-stopped = Senden an { $device } gestoppt.
core-cast-failed = Senden an { $device } fehlgeschlagen. Prüfe, ob es im selben Netzwerk ist.
core-cast-not-built = Senden ist in diesem Build nicht enthalten
core-cast-serve-failed = die Folge konnte nicht bereitgestellt werden: { $error }
core-cast-device-gone = Dieses Gerät ist nicht mehr im Netzwerk
core-cast-not-a-playlist = keine Playlist-Adresse
core-cast-nothing = Es wird nichts gesendet
core-cast-still-starting = Das Gerät startet noch
core-cast-no-route = dieses Gerät hat keinen Weg zum TV
core-cast-no-control = der TV hat keine Steueradresse geliefert
core-cast-not-handed = der andere Computer konnte die Folge nicht senden: { $error }
core-cast-empty-playlist = der andere Computer hat eine leere Playlist geschickt
core-cast-refused-both = { $error }; auch die Originaldatei wurde abgelehnt ({ $first })
core-cast-chromecast-stopped = der Chromecast hat gestoppt
core-cast-hevc = { $error }. Diese Folge nutzt HEVC, das nur neuere Chromecasts (mit Google TV oder Ultra) unterstützen

## playback/src/cast/castv2.rs

core-cast-chromecast-unreachable = Chromecast nicht erreichbar: { $error }
core-cast-chromecast-broke = die Verbindung zum Chromecast ist abgebrochen: { $error }
core-cast-chromecast-no-player = der Chromecast hat seinen Mediaplayer nicht gestartet
core-cast-chromecast-refused-player = der Chromecast konnte seinen Mediaplayer nicht starten ({ $reason })
core-cast-no-reason = kein Grund
core-cast-chromecast-cannot-play = der Chromecast konnte diese Datei nicht abspielen ({ $kind })
core-cast-chromecast-error = der Chromecast ist beim Abspielen dieser Datei mit einem Fehler gestoppt

## playback/src/cast/dlna.rs

core-cast-tv-no-answer = der TV hat nicht reagiert: { $error }
core-cast-tv-refused = der TV hat { $action } abgelehnt
core-cast-tv-refused-code = der TV hat { $action } abgelehnt ({ $code })

## playback/src/cast/media.rs

core-cast-no-video = Diese Datei hat kein Video.
core-cast-hi10p = Diese Folge nutzt 10-Bit-H.264 (Hi10P), das TVs und Chromecasts nicht dekodieren können. Zum Senden müsste sie umgewandelt werden, und das wird noch nicht unterstützt.
core-cast-video-codec = Das Video dieser Folge ({ $codec }) lässt sich nicht senden.
core-cast-audio-codec = Der Ton dieser Folge ({ $codec }) lässt sich nicht senden.

## downloads/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Download einer Folge von { $folder } gestartet.
       *[other] Download von { $count } Folgen von { $folder } gestartet.
    }
core-phone-download-partial = Nicht jedes Release von { $folder } konnte hinzugefügt werden: { $error }
core-phone-subscribed = { $name } abonniert.
core-phone-rule-stopped = { $name } prüft nicht auf neue Folgen: Die Quelle wurde entfernt oder braucht ihren Schlüssel neu.
core-phone-new-episode = Eine neue Folge von { $name } wird geladen.
core-phone-name-taken = { $name }: eine andere Quelle hat diesen Namen
core-phone-nothing-to-download = nichts zu laden
core-phone-over-cap = damit würde das Speicherlimit von { -app-name } überschritten ({ $used } von { $cap } GB belegt oder in der Warteschlange); lösche Folgen oder erhöhe das Limit
core-phone-empty-search = die Suche ist leer
core-phone-no-results = keine Treffer für { $query }
core-phone-own-engine = eine eigene Torrent-Engine

## downloads/src/phone/check.rs

core-phone-check-connected = verbunden
core-phone-check-refused = abgelehnt: { $error }
core-phone-check-dropped = keine Antwort (verworfen)
core-phone-check-finding = { $label }: Adresse wird aufgelöst
core-phone-check-failed = fehlgeschlagen: { $error }
core-phone-check-no-answer = keine Antwort
core-phone-check-private = { $label }: private Auflösung
core-phone-check-skipped-address = übersprungen, IP-Adresse
core-phone-check-skipped-home = übersprungen, lokaler Netzwerkname
core-phone-check-no-record = kein öffentlicher Eintrag, Rückgriff auf das DNS des Netzes
core-phone-check-no-resolver = kein DNS-Resolver hat geantwortet
core-phone-check-reaching = { $label }: verbinde mit { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } Byte
core-phone-check-setup = Aufbau

## playback/src/session.rs

core-playback-not-saved = Dein Fortschritt bei { $folder } wurde nicht gespeichert. Ordne die Serie zuerst zu.
core-playback-still-downloading = Folge { $number } wird noch geladen, { $done } % fertig

## playback/src/skip.rs

core-playback-bad-duration = die Dauer muss eine positive Zahl an Sekunden sein

## core/src/agent.rs

core-agent-no-answer = der Agent hat keine Antwort geliefert
core-agent-searching = Suche im Web nach { $query }
core-agent-searching-schedule = Suche im Web nach dem Sendeplan
core-agent-reading = Lese { $url }
core-agent-reading-page = Lese eine Seite
core-agent-using = Nutze { $tool }
core-agent-none = ein Coding-Agent auf diesem Computer
core-agent-ended = { $name } wurde ohne Antwort beendet
core-agent-too-long = { $name } hat zu lange gebraucht

## core/src/core.rs

core-core-guest-trackers = die Tracker-Konten auf diesem Computer gehören seinem Besitzer, also kann dieses Gerät sie nicht aktualisieren
core-core-guest-not-interested = Kein Interesse auf diesem Computer gehört seinem Besitzer, also kann dieses Gerät es nicht ändern
core-core-guest-recommended = die Empfehlungen auf diesem Computer stammen aus der Liste seines Besitzers
core-core-nothing-picked = nichts ausgewählt

## foundation/src/prefs.rs

core-prefs-not-a-provider = { $provider } ist kein Anbieter für Zuordnungen
core-prefs-provider-count = { $provider } steht { $count }-mal drin; jeder Anbieter darf nur einmal vorkommen
core-prefs-outside = { $value } muss zwischen { $low } und { $high } liegen
core-prefs-empty = leer

## metadata/src/recommended/mod.rs

core-recommended-ranking = Sortiere Serien für dich

## metadata/src/recommended/sync.rs

core-recommended-finding-people = Suche Leute mit deinem Geschmack
core-recommended-reading-lists = Lese ihre Listen
core-recommended-reading-yours = Lese deine Serien
core-recommended-reading-suggestions = Lese Serien für Vorschläge

## metadata/src/recommended/page.rs

core-recommended-no-anime = { -anilist } hat keinen Anime { $id }

## host/src/lib.rs

core-host-off = Aus
core-host-starting = Startet
core-host-serving = Erreichbar unter { $addresses }
core-host-no-tailscale = Tailscale ist nicht verbunden
core-host-cannot-listen = Konnte nicht lauschen
core-host-port-taken = Port { $port } ist belegt
core-host-cannot-listen-on = Konnte nicht auf { $address } lauschen: { $error }

## host/src/addr.rs

core-host-not-an-address = { $text } ist keine gültige Adresse mit Port

## host/src/keys.rs

core-host-not-a-key = { $key } ist kein Geräteschlüssel

## net/src/limiter.rs

core-net-timed-out = Zeitüberschreitung nach { $ms } ms
core-net-rate-limited = { $provider } drosselt die Anfragen

## net/src/lib.rs

core-net-waiting = { $host }: vor Kurzem Zeitüberschreitung; { -app-name } wartet { $seconds } s, bevor es es erneut versucht

## net/src/anilist.rs

core-net-no-data = keine Daten

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } hat keinen Download für diesen Computer
core-update-cannot-make = { $path } konnte nicht angelegt werden: { $error }
core-update-cannot-mark = Das neue AppImage konnte nicht ausführbar gemacht werden: { $error }
core-update-cannot-replace = { $path } konnte nicht ersetzt werden: { $error }
core-update-not-here = Diese Installation kann sich hier nicht selbst aktualisieren
core-update-cannot-start = Der Download konnte nicht starten: { $error }
core-update-did-not-start = Der Download ist nicht gestartet: { $error }
core-update-cannot-write = { $path } konnte nicht geschrieben werden: { $error }
core-update-stopped = Der Download hat gestoppt: { $error }
core-update-cancelled = Abgebrochen
core-update-ended-early = Der Download endete zu früh ({ $got } von { $size } Byte)
core-update-checksum = Der Download passt nicht zur Prüfsumme von GitHub
core-update-cannot-move = Der Download konnte nicht an seinen Platz verschoben werden: { $error }
core-update-cannot-open-dmg = Die DMG konnte nicht geöffnet werden: { $error }
core-update-cannot-read-dmg = Die DMG konnte nicht gelesen werden: { $error }
core-update-no-app = Die DMG enthält keine App
core-update-cannot-copy = Die neue App konnte nicht neben diese kopiert werden: { $error }

## metadata/src/calendar.rs

core-calendar-season-episode = { $title } - Staffel { $season } Folge { $number }
core-calendar-episode = { $title } - Folge { $number }

## foundation/src/paths.rs

core-paths-no-home = kein Home-Verzeichnis

## core/src/first_run.rs

core-first-run-not-absolute = der Videoordner muss ein absoluter Pfad sein

## playback/src/presence/mod.rs

core-presence-extra = Extra
core-presence-room = Gemeinsam schauen
core-presence-in-room = In einem Raum
core-presence-no-room = Zu dieser Discord-Einladung gibt es noch keinen Raum. Bitte sie, einen zu starten.

## trackers/src/accounts.rs

core-trackers-no-client-id = Keine Client-ID für { $tracker } gesetzt.
core-trackers-mal-reconnect = Verbinde MyAnimeList in den Einstellungen neu.
core-trackers-mal-wants-secret = MyAnimeList verlangt das Client-Secret dieser App. Trag es in den Einstellungen von Anibeam unter Konten ein, oder registriere die App mit App Type "other".
core-trackers-token-reply-unreadable = Token-Antwort nicht lesbar: { $error }
core-trackers-token-reply-no-token = die Token-Antwort enthielt kein Access-Token

## trackers/src/lib.rs

core-trackers-no-token = kein Access-Token gespeichert, verbinde in den Einstellungen neu

## trackers/src/cache.rs

core-trackers-no-user-id = die Kontoantwort enthielt keine Nutzer-ID

## trackers/src/move_up.rs

core-trackers-move-none-picked = keine Serie zum Verschieben ausgewählt
core-trackers-moved-up =
    { $count ->
        [one] { $count } Serie von diesem Computer zu { $site } verschoben.
       *[other] { $count } Serien von diesem Computer zu { $site } verschoben.
    }
core-trackers-moved-up-some-refused =
    { $moved ->
        [one] { $moved } Serie
       *[other] { $moved } Serien
    } zu { $site } verschoben; { $failed ->
        [one] { $failed } Serie konnte
       *[other] { $failed } Serien konnten
    } nicht verschoben werden.
core-trackers-move-refused = { $site } konnte { $show } nicht aktualisieren: { $error }

## trackers/src/oauth.rs

core-trackers-oauth-timed-out = Anmeldung abgelaufen, versuch es nochmal.
core-trackers-oauth-state-mismatch = State passt nicht, mögliches CSRF, abgebrochen.
core-trackers-oauth-no-code = kein Code im Callback
core-trackers-oauth-no-token = kein access_token im Callback
core-trackers-oauth-waiting = Warte, bis du dich im Browser bei { $site } anmeldest.
core-trackers-oauth-connected = { $site } als { $username } verbunden.
core-trackers-oauth-no-name = die Profilantwort enthielt keinen Nutzernamen
core-trackers-oauth-could-not-start = Anmeldung konnte nicht gestartet werden
core-trackers-oauth-port-failed = Port { $port } auf diesem Computer ließ sich nicht öffnen: { $error }
core-trackers-oauth-port-busy = Port { $port } auf diesem Computer wird von einem anderen Programm genutzt. Warte kurz und versuch es nochmal, oder schließ das Programm, falls er belegt bleibt.
core-trackers-oauth-provider-error = { $site } hat einen Fehler gemeldet: { $error }
core-trackers-page-connected = { $site } ist mit { $app } verbunden
core-trackers-page-close-tab = Du kannst diesen Tab schließen.
core-trackers-page-failed-title = { $app }, Anmeldung fehlgeschlagen
core-trackers-page-failed = Anmeldung nicht abgeschlossen
core-trackers-page-go-back = Geh zurück zu { $app } und versuch es nochmal.
core-trackers-page-finishing-title = { $app }, wird noch abgeschlossen
core-trackers-page-finishing = Wird noch abgeschlossen
core-trackers-page-finishing-body = { $site } hat geantwortet, und Anibeam schließt die Anmeldung ab. Geh zurück zu { $app }, um fortzufahren.
core-trackers-page-cancelled-title = { $app }, Anmeldung abgebrochen
core-trackers-page-cancelled = Anmeldung wurde abgebrochen
core-trackers-page-cancelled-body = Du kannst diesen Tab schließen. Starte in { $app } neu, wenn du dich noch anmelden willst.

## trackers/src/on_hold.rs

core-trackers-on-hold-moving = Setze auf Pausiert bei { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] { $count } Serie bei { $site } auf Pausiert gesetzt.
       *[other] { $count } Serien bei { $site } auf Pausiert gesetzt.
    }
core-trackers-on-hold-moved-two =
    { $count ->
        [one] { $count } Serie bei { $site } und { $second } bei { $secondSite } auf Pausiert gesetzt.
       *[other] { $count } Serien bei { $site } und { $second } bei { $secondSite } auf Pausiert gesetzt.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } Serie konnte nicht auf Pausiert gesetzt werden.
       *[other] { $failed } Serien konnten nicht auf Pausiert gesetzt werden.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] { $count } Serie bei { $site } auf Pausiert gesetzt.
       *[other] { $count } Serien bei { $site } auf Pausiert gesetzt.
    } { $failed ->
        [one] { $failed } Serie konnte nicht auf Pausiert gesetzt werden.
       *[other] { $failed } Serien konnten nicht auf Pausiert gesetzt werden.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] { $count } Serie bei { $site } und { $second } bei { $secondSite } auf Pausiert gesetzt.
       *[other] { $count } Serien bei { $site } und { $second } bei { $secondSite } auf Pausiert gesetzt.
    } { $failed ->
        [one] { $failed } Serie konnte nicht auf Pausiert gesetzt werden.
       *[other] { $failed } Serien konnten nicht auf Pausiert gesetzt werden.
    }
core-trackers-series-fallback = Serie { $id }
core-trackers-back-on-watching = { $name } steht bei { $site } wieder auf Schaue ich.
core-trackers-back-on-watching-two = { $name } steht bei { $site } und { $secondSite } wieder auf Schaue ich.
core-trackers-back-on-watching-failed = { $name } konnte nicht zurück auf Schaue ich. Versuch es später nochmal.

## trackers/src/writes.rs

core-trackers-rate-limited = { $tracker } drosselt die Anfragen, versuch es in einer Minute nochmal.
core-trackers-auth-expired = Anmeldung bei { $tracker } abgelaufen, verbinde in den Einstellungen neu.
core-trackers-entry-not-found = Eintrag bei { $tracker } nicht gefunden.
core-trackers-server-error = Serverfehler bei { $tracker } ({ $status }), versuch es später nochmal.
core-trackers-error-see-log = Fehler bei { $tracker }, siehe Aktivitätsprotokoll.
core-trackers-episode-not-whole = { $episode } muss eine ganze Zahl größer als 0 sein
core-trackers-score-out-of-range = { $score } muss zwischen 0 und 10 liegen
core-trackers-rewatch-not-settable = Erneut schauen lässt sich nicht von Hand setzen
core-trackers-notes-too-long = Notizen dürfen höchstens { $max } Zeichen haben
core-trackers-nothing-picked = nichts ausgewählt
core-trackers-notes-not-on-list = Noch nicht auf deiner Liste bei { $tracker }. Füge sie zuerst einer Liste hinzu.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } und { -mal }
        [anilist] { -anilist }
       *[mal] { -mal }
    }
core-trackers-status =
    { $status ->
        [watching] { status-watching }
        [planning] { status-planning }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[repeating] Erneut
    }
core-trackers-marked = Folge { $episode } von { $show } bei { core-trackers-sites } markiert.
core-trackers-progress-set = Fortschritt bei { $show } auf { $progress } gesetzt, bei { core-trackers-sites }.
core-trackers-scored = { $show } mit { $score } bewertet, bei { core-trackers-sites }.
core-trackers-score-cleared = Wertung für { $show } bei { core-trackers-sites } gelöscht.
core-trackers-status-set = { $show } auf { core-trackers-status } gesetzt, bei { core-trackers-sites }.
core-trackers-notes-saved = Notizen zu { $show } bei { core-trackers-sites } gespeichert.
core-trackers-notes-cleared = Notizen zu { $show } bei { core-trackers-sites } gelöscht.
core-trackers-update-failed = { $site } konnte nicht aktualisiert werden. Versuch es später nochmal.
core-trackers-written =
    { $done ->
        [0] { core-trackers-written-failed }
       *[other] { core-trackers-written-done }{ $failed ->
            [0] {""}
           *[other] {" "}{ core-trackers-written-failed-more }
        }
    }{ core-trackers-left-out }
core-trackers-written-who =
    { $named ->
        [yes] { $name }
       *[no] { $done ->
            [one] { $done } Serie
           *[other] { $done } Serien
        }
    }
core-trackers-written-done =
    { $action ->
        [clear] Wertung für { core-trackers-written-who } bei { core-trackers-sites } gelöscht.
        [status] { core-trackers-written-who } auf { core-trackers-status } gesetzt, bei { core-trackers-sites }.
       *[score] { core-trackers-written-who } mit { $score } bewertet, bei { core-trackers-sites }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] { $failed } Serie konnte nicht aktualisiert werden: { $reason }.
           *[other] { $failed } Serien konnten nicht aktualisiert werden: { $reason }.
        }
       *[no] { $failed ->
            [one] { $failed } Serie konnte nicht aktualisiert werden.
           *[other] { $failed } Serien konnten nicht aktualisiert werden.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] { $failed } nicht aktualisiert: { $reason }.
       *[no] { $failed } nicht aktualisiert.
    }
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } Serie hat keine Tracker-Zuordnung.
       *[other] {" "}{ $noMatch } Serien haben keine Tracker-Zuordnung.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } Serie ist ausgeblendet, ihr Sync ist also aus.
       *[other] {" "}{ $hidden } Serien sind ausgeblendet, ihr Sync ist also aus.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } Serie ist nur bei einem Tracker, der nicht verbunden ist.
       *[other] {" "}{ $notConnected } Serien sind nur bei einem Tracker, der nicht verbunden ist.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } Serie bleibt auf diesem Computer, ohne Tracker zum Senden.
       *[other] {" "}{ $kept } Serien bleiben auf diesem Computer, ohne Tracker zum Senden.
    }

## metadata/src/automatch.rs

core-metadata-matched-to = { $show } zugeordnet zu { $title }.
core-metadata-auto-matched =
    { $count ->
        [one] { $count } Serie zugeordnet.
       *[other] { $count } Serien zugeordnet.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } Serie braucht noch eine Zuordnung.
       *[other] { $count } Serien brauchen noch eine Zuordnung.
    }
core-metadata-auto-matched-some =
    { $matched ->
        [one] { $matched } Serie
       *[other] { $matched } Serien
    } zugeordnet; { $unmatched } { $unmatched ->
        [one] braucht
       *[other] brauchen
    } noch eine Zuordnung.
core-metadata-find =
    { $matched ->
        [0] {""}
       *[other] { core-metadata-find-matched }
    }{ $unmatched ->
        [0] {""}
       *[other] { $first ->
            [unmatched] {""}
           *[other] {" "}
        }{ core-metadata-find-unmatched }
    }{ $failed ->
        [0] {""}
       *[other] { $first ->
            [failed] {""}
           *[other] {" "}
        }{ core-metadata-find-failed }
    }{ $had ->
        [0] {""}
       *[other] { $first ->
            [had] {""}
           *[other] {" "}
        }{ core-metadata-find-had }
    }{ $busy ->
        [0] {""}
       *[other] { $first ->
            [busy] {""}
           *[other] {" "}
        }{ core-metadata-find-busy }
    }
core-metadata-find-matched =
    { $hitNamed ->
        [yes] { $hitFolder } zugeordnet zu { $hitTitle }.
       *[no] { $matched ->
            [one] { $matched } Serie zugeordnet.
           *[other] { $matched } Serien zugeordnet.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] Keine Zuordnung für { $missName } gefunden.
       *[no] { $unmatched ->
            [one] Keine Zuordnung für { $unmatched } Serie gefunden.
           *[other] Keine Zuordnung für { $unmatched } Serien gefunden.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] { $failed } Serie konnte nicht gesucht werden; versuch es später nochmal.
       *[other] { $failed } Serien konnten nicht gesucht werden; versuch es später nochmal.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } Serie hatte schon eine Zuordnung.
       *[other] { $had } Serien hatten schon eine Zuordnung.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } Serie wird schon zugeordnet.
       *[other] { $busy } Serien werden schon zugeordnet.
    }

## metadata/src/apply.rs

core-metadata-link-unreadable = Dieser Link war nicht lesbar. Füge die URL einer Seite von AniList, MyAnimeList, TVmaze oder IMDb ein.
core-metadata-no-mal-entry = AniList hat keinen Eintrag zu dieser MyAnimeList-ID.
core-metadata-no-imdb-show = TVmaze hat keine Serie zu diesem IMDb-Titel.
core-metadata-search-on = Suche bei { $provider }
core-metadata-query-too-short = Gib zum Suchen mindestens zwei Zeichen ein.
core-metadata-not-a-link = kein Link
core-metadata-refresh-unsupported = Aktualisieren einer reinen MAL- oder TMDB-Zuordnung
core-metadata-matched = { $show } zugeordnet.
core-metadata-updated = Infos zu { $show } aktualisiert.
core-metadata-update-failed = Infos zu { $show } konnten nicht aktualisiert werden. Versuch es später nochmal.
core-metadata-progress-refresh = Aktualisieren
core-metadata-progress-backfill = Nachtragen
core-metadata-updated-many =
    { $count ->
        [one] Infos zu { $count } Serie aktualisiert.
       *[other] Infos zu { $count } Serien aktualisiert.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] Infos zu { $count } Serie konnten nicht aktualisiert werden. Versuch es später nochmal.
       *[other] Infos zu { $count } Serien konnten nicht aktualisiert werden. Versuch es später nochmal.
    }
core-metadata-updated-some =
    { $count ->
        [one] Infos zu { $count } Serie aktualisiert; { $failed } konnten nicht aktualisiert werden.
       *[other] Infos zu { $count } Serien aktualisiert; { $failed } konnten nicht aktualisiert werden.
    }

## metadata/src/fetch.rs

core-metadata-no-such-show = TVmaze hat keine Serie { $id }
core-metadata-no-such-media = AniList hat kein Medium { $id }

## metadata/src/imdb.rs

core-metadata-progress-imdb-links = IMDb-Links von Wikidata
core-metadata-progress-imdb-ratings = IMDb-Bewertungen

## metadata/src/franchise/layout.rs

core-franchise-relation-character = Gemeinsame Charaktere

## net/src/images.rs

core-images-progress-fetching = Bilder werden geladen

## host/src/transfer/export.rs

core-transfer-exported =
    { $count ->
        [one] { $count } Serie in { $folder } gesichert.
       *[other] { $count } Serien in { $folder } gesichert.
    }

## host/src/transfer/format.rs

core-transfer-not-json = kein JSON: { $error }
core-transfer-wrong-format = kein { $format }-Dokument

## host/src/transfer/import.rs

core-transfer-imported = Deine Bibliothek wurde aus der Datei geladen.

## library/src/delete.rs

core-library-no-such-file = keine Datei { $id } in der Bibliothek
core-library-file-elsewhere-not-removed = Datei { $id } gehört zu einer anderen Serie, also wurde nichts entfernt
core-library-file-outside = { $path } ist keine Datei in einem Bibliotheksordner, also wurde nichts entfernt
core-library-episodes-removed =
    { $count ->
        [one] { $count } Folge
       *[other] { $count } Folgen
    } von { $title } { $place ->
        [deleted] gelöscht
       *[trash] in den Papierkorb verschoben
    }.
core-library-finished-removed =
    { $count ->
        [one] { $count } beendete Serie
       *[other] { $count } beendete Serien
    } { $place ->
        [deleted] gelöscht
       *[trash] in den Papierkorb verschoben
    }.
core-library-watched-removed =
    { $count ->
        [one] { $count } geschaute Folge
       *[other] { $count } geschaute Folgen
    } aus { $series } Serien { $place ->
        [deleted] gelöscht
       *[trash] in den Papierkorb verschoben
    }.

## library/src/detail.rs

core-library-fix-one-file =
    Die einzige Datei hier wird gelesen als { $read ->
        [episode] Folge { $number }
       *[label] { $label }
    }. Sie sieht aus wie { $what ->
        [film] der Film
       *[episode] die einzige Folge
    }.
core-library-fix-specials =
    { $count ->
        [one] { $count } Special sieht
       *[other] { $count } Specials sehen
    } aus wie { $episodes ->
        [one] die fehlende Folge
       *[other] die fehlenden Folgen
    } { $numbers }.
core-library-fix-numbered-on = Die Folgen { $from } sehen aus wie die Folgen { $to }, weitergezählt aus einer früheren Staffel.
core-library-runs-span = { $first } bis { $last }
core-library-runs-list = { $init } und { $last }

## library/src/fixes.rs

core-library-fix-episode-range = die Folgennummer muss zwischen 0 und 10000 liegen
core-library-fix-season-range = die Staffelnummer darf höchstens 200 sein
core-library-fix-extra-range = der Index eines Extras darf höchstens 10000 sein
core-library-file-elsewhere-not-fixed = Datei { $id } gehört zu einer anderen Serie, also wurde nichts korrigiert
core-library-fix-undone =
    { $count ->
        [one] { $count } Datei von { $title } wird wieder nach Namen gelesen.
       *[other] { $count } Dateien von { $title } werden wieder nach Namen gelesen.
    }
core-library-fixed =
    { $count ->
        [one] Lesart von { $count } Datei von { $title } korrigiert.
       *[other] Lesart von { $count } Dateien von { $title } korrigiert.
    }

## library/src/reads.rs

core-library-hid-one = { $title } ausgeblendet.
core-library-back-one = { $title } ist wieder da.
core-library-hid-many = { $count } Serien ausgeblendet.
core-library-back-many = { $count } Serien sind wieder da.
core-library-episode-count-set = { $title } hat jetzt { $count } Folgen.
core-library-episode-count-cleared = Folgenzahl von { $title } gelöscht.

## library/src/schedule.rs

core-library-schedule-cleared = Sendeplan-Notizen von { $title } gelöscht.
core-library-schedule-saved = { $count } Sendeplan-Notizen für { $title } gespeichert.

## library/src/scan.rs

core-library-source-not-absolute = ein Quellpfad muss absolut sein
core-library-source-exists = schon eine Quelle
core-library-source-nested = liegt in einer vorhandenen Quelle
core-library-source-added = Ordner { $folder } zur Bibliothek hinzugefügt.
core-library-source-removed = Ordner { $folder } aus der Bibliothek entfernt.
core-library-source-removed-with-shows =
    { $count ->
        [one] Ordner { $folder } und seine { $count } Serie aus der Bibliothek entfernt.
       *[other] Ordner { $folder } und seine { $count } Serien aus der Bibliothek entfernt.
    }
core-library-series-forgotten = { $name } aus der Bibliothek entfernt.
core-library-source-back = Der Ordner { $folder } ist wieder da.
core-library-source-unavailable = Der Ordner { $folder } ist nicht verfügbar. Prüfe, ob sein Laufwerk angeschlossen ist.
core-library-source-unreadable = Der Ordner { $folder } konnte nicht gelesen werden. Prüfe, ob sein Laufwerk angeschlossen ist.
core-library-source-looks-empty = Der Ordner { $folder } sieht leer aus, also bleiben deine Serien, wie sie waren.
core-library-source-reconcile-failed = Die Serien in { $folder } konnten nicht aktualisiert werden.
core-library-scan-done =
    Ordner geprüft: { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } neue Serie
           *[other] { $added } neue Serien
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } Serie geändert
           *[other] { $changed } Serien geändert
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }{ $removed } fehlen
    }.

## library/src/report.rs

core-library-agent-no-answer = keine Antwort innerhalb von { $span }
core-library-agent-stopped-code = er hat mit Code { $code } aufgehört
core-library-agent-stopped = er hat aufgehört
core-library-agent-answered-nothing = er hat nichts ausgegeben
core-library-span-minutes =
    { $count ->
        [one] { $count } Minute
       *[other] { $count } Minuten
    }
core-library-span-seconds =
    { $count ->
        [one] { $count } Sekunde
       *[other] { $count } Sekunden
    }
core-library-report-no-fix = keine dieser Dateien hat eine Korrektur zu melden
core-library-agent-no-triage = { $agent } hat keine Triage geschrieben: { $why }
core-transfer-progress-sources = Quellen
core-transfer-progress-series = Serien
core-transfer-progress-history = Verlauf
core-transfer-progress-accounts = Konten
core-transfer-progress-preferences = Einstellungen
core-transfer-progress-done = fertig

## downloads/src/anirss.rs

core-downloads-anirss-no-answer = anirss hat nicht reagiert: { $message }
core-downloads-anirss-answered = anirss lieferte HTTP { $status }: { $text }
core-downloads-no-free-port = kein freier Port: { $error }
core-downloads-anirss-exited = anirss wurde beendet ({ $status })
core-downloads-anirss-exited-said = anirss wurde beendet ({ $status }): { $said }
core-downloads-anirss-slow-start = anirss ist nicht innerhalb von { $seconds } Sekunden gestartet
core-downloads-anirss-slow-start-said = anirss ist nicht innerhalb von { $seconds } Sekunden gestartet: { $said }
core-downloads-anirss-install-mac = Installiere anirss { $version } oder neuer mit: { $command }
core-downloads-anirss-install-linux = Installiere anirss { $version } oder neuer: das Paket anirss aus dem AUR, oder { $command }
core-downloads-anirss-too-old = anirss { $version } ist zu alt für Anibeam, das { $minimum } oder neuer braucht. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } kann die Anmeldung jedes Torrent-Clients nicht getrennt halten, was Anibeam zum Wechseln der Clients braucht; anirss { $minimum } und neuer kann das. { $update }.
core-downloads-anirss-update-mac = Aktualisiere es mit: { $command }
core-downloads-anirss-update-linux = Aktualisiere das Paket anirss, oder führe aus: { $command }

## downloads/src/contents.rs

core-downloads-torrent-file-no-answer = die Torrent-Datei konnte nicht geladen werden: { $message }
core-downloads-torrent-file-answered = die Torrent-Datei lieferte HTTP { $status }

## downloads/src/feed.rs

core-downloads-feed-answered = der Feed lieferte HTTP { $status }
core-downloads-not-a-feed = kein Feed: { $error }

## downloads/src/jackett.rs

core-downloads-jackett-missing = Jackett ist nicht installiert: { $program } wurde nicht gefunden. Installiere es oder ändere den Befehl in den Einstellungen.
core-downloads-endpoint-start-failed = { $name } konnte nicht starten: { $why }
core-downloads-endpoint-start-timeout = { $name } ist gestartet, hat aber nicht innerhalb von { $seconds } Sekunden reagiert.
core-downloads-command-exited = der Befehl endete mit { $status }

## downloads/src/lib.rs

core-downloads-anirss-stopped = anirss wurde beendet
core-downloads-anirss-missing = { $install }. Du kannst in den Einstellungen auch einen laufenden Server eintragen.
core-downloads-anirss-command-missing = { $command } wurde nicht gefunden. { $install }. Du kannst in den Einstellungen auch einen laufenden Server eintragen.
core-downloads-anirss-start-failed = { $command } konnte nicht starten: { $error }
core-downloads-anirss-too-old-said = { $tooOld } Es meldete: { $message }
core-downloads-no-torrent-client = Unter { $address } reagiert kein qBittorrent, und Anibeam hat keinen eingebauten Client zur Verfügung. Installiere qBittorrent und schalte sein Web UI ein, oder installiere qbittorrent-nox, das Anibeam dann automatisch startet
core-downloads-own-client-gone = Das eingebaute qBittorrent von Anibeam fehlt (qbittorrent-nox ist nicht installiert). Installiere Anibeam neu, oder trag in den Einstellungen dein eigenes qBittorrent ein
core-downloads-no-port = kein Port in { $address }
core-downloads-own-client-settings = die Einstellungen von qBittorrent lassen sich nicht schreiben: { $error }
core-downloads-own-client-start-failed = { $command } konnte nicht starten: { $error }
core-downloads-own-client-slow-start = Das qBittorrent von Anibeam ist nicht gestartet
core-downloads-own-client-slow-start-said = Das qBittorrent von Anibeam ist nicht gestartet: { $said }
core-downloads-not-ready = Downloads sind nicht bereit. Prüfe Downloads in den Einstellungen.
core-downloads-ready = Downloads sind bereit.
core-downloads-helper-sources =
    { $count ->
        [one] Downloads suchen jetzt direkt in Anibeam, das die { $count } Quelle aus der Konfiguration des Suchhelfers nicht liest. Füge sie unter Einstellungen > Downloads hinzu, oder wechsle dort zurück zum Suchhelfer.
       *[other] Downloads suchen jetzt direkt in Anibeam, das die { $count } Quellen aus der Konfiguration des Suchhelfers nicht liest. Füge sie unter Einstellungen > Downloads hinzu, oder wechsle dort zurück zum Suchhelfer.
    }
core-downloads-client-connected = Die Download-App ist verbunden.
core-downloads-client-not-connected = Die Download-App ist nicht verbunden. Prüfe ihre Adresse in den Einstellungen.
core-downloads-not-on-demand = { $name } ist keine Quelle auf Abruf
core-downloads-source-running = Die Downloadquelle läuft.
core-downloads-source-not-running = { $name } läuft nicht. Schalte die Quelle bei deinen Quellen ein und bestätige den Start.
core-downloads-empty-query = gib etwas zum Suchen ein
core-downloads-first-endpoint = die erste Quelle
core-downloads-search-starting = Suche startet
core-downloads-search-starting-endpoint = Starte { $endpoint }
core-downloads-search-searching-endpoint = Durchsuche { $endpoint }
core-downloads-search-reading = Lese die Ergebnisse
core-downloads-subscribed = { $name } abonniert.
core-downloads-client-no-rule = qBittorrent hat keine Regel namens { $name }
core-downloads-trying-search = Probiere „{ $search }“
core-downloads-nothing-selected = nichts zum Laden gewählt
core-downloads-added =
    { $count ->
        [one] { $count } Download hinzugefügt.
       *[other] { $count } Downloads hinzugefügt.
    }
core-downloads-movie-link = ein Filmlink muss ein Magnet oder eine http(s)-URL sein
core-downloads-added-movie = Einen Film zu den Downloads hinzugefügt.
core-downloads-library-folder = { $folder } ist der Bibliotheksordner von { $name } und enthält Folgen, die dieses Abo nicht geladen hat. Entferne das Abo, ohne seine Dateien zu löschen, oder entferne { $name } auf seiner Seite
core-downloads-no-torrents-named = keine Torrents genannt
core-downloads-paused =
    { $count ->
        [one] { $count } Download pausiert.
       *[other] { $count } Downloads pausiert.
    }
core-downloads-resumed =
    { $count ->
        [one] { $count } Download fortgesetzt.
       *[other] { $count } Downloads fortgesetzt.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] { $count } Download samt Dateien gelöscht.
       *[other] { $count } Downloads samt Dateien gelöscht.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] { $count } Download aus deiner Liste entfernt.
       *[other] { $count } Downloads aus deiner Liste entfernt.
    }
core-downloads-nothing-to-remove = nichts zu entfernen
core-downloads-removed-show = { $title } aus den Downloads entfernt.
core-downloads-removed-the-show = Die Serie aus den Downloads entfernt.
core-downloads-deleted-still-listed = Gelöschte Dateien stehen noch in der Download-App.
core-downloads-feed-url = die Feed-URL muss mit http:// oder https:// beginnen
core-downloads-rule-needs-name = der Regelname darf nicht leer sein
core-downloads-no-rule = keine Regel namens { $name }
core-downloads-feed-no-search = dieser Feed hat keine Suchanfrage zum Kürzen
core-downloads-no-repair = { $name } muss nicht repariert werden.
core-downloads-repaired = Abo von { $name } repariert.
core-downloads-repair-failed = keine kürzere Suche lieferte Ergebnisse; selbst „{ $search }“ blieb leer
core-downloads-no-saved-client = kein gespeicherter Torrent-Client { $id }
core-downloads-client-switched = Downloads nutzen jetzt eine andere Download-App.
core-downloads-host-not-running = läuft nicht auf dem Host
core-downloads-host-late = hat nicht rechtzeitig reagiert

## downloads/src/native.rs

core-downloads-native-empty-query = leere Suche
core-downloads-native-no-results = keine Treffer für { $query }
core-downloads-native-feed-taken = ein Feed namens { $name } zeigt schon auf eine andere Adresse
core-downloads-native-no-items = keine Einträge angegeben
core-downloads-native-bad-link = Links müssen magnet: oder http(s)-URLs sein, bekommen: { $link }
core-downloads-native-no-links = keine Links angegeben
core-downloads-native-no-names = keine Abo-Namen angegeben
core-downloads-native-files-elsewhere = der Torrent-Client speichert seine Dateien auf einem anderen Rechner, also lassen sie sich nur zusammen mit seinen Torrents löschen
core-downloads-native-unknown-feeds = unbekannte Feeds: { $names }
core-downloads-native-outside-folders = Dateien außerhalb der Downloadordner werden nicht entfernt: { $paths }
core-downloads-refuse-home-folder = { $path } ist dein Home-Ordner oder enthält ihn, also wurde nichts entfernt
core-downloads-refuse-library-folder = { $path } ist ein Bibliotheksordner oder enthält einen, also wurde nichts entfernt

## downloads/src/paths.rs

core-downloads-no-default-folder = in qBittorrent ist kein Standard-Speicherordner festgelegt; leg einen in qBittorrent fest
# $folder is the folder given
core-downloads-save-folder-not-absolute = { $folder } ist kein vollständiger Pfad zu einem Ordner
# $folder is the folder given
core-downloads-save-folder-unusable = { $folder } lässt sich nicht für Downloads nutzen: wähle einen vorhandenen Ordner, der nicht dein Home-Ordner ist
core-downloads-name-empty = der Name darf nicht leer sein
core-downloads-name-slash = der Name darf kein / oder \ enthalten
core-downloads-name-not-folder = { $name } ist kein Ordnername

## downloads/src/prune.rs

core-downloads-pruned = Abo von { $title } beendet (fertig ausgestrahlt).
core-downloads-prune-failed = Abo von { $title } konnte nicht beendet werden. Prüfe Downloads in den Einstellungen.

## library/src/trash.rs

core-downloads-refuse-not-absolute = { $path } ist kein einfacher absoluter Pfad, also wurde nichts entfernt
core-downloads-refuse-root = { $path } ist die Wurzel des Dateisystems, also wurde nichts entfernt
core-downloads-refuse-outside-library = { $path } liegt in keinem Bibliotheksordner, also wurde nichts entfernt
core-downloads-refuse-save-folder = { $path } ist der Speicherordner des Torrents, nicht die geladenen Dateien, also wurde nichts entfernt

## downloads/src/watch.rs

core-downloads-episode-missed = Folge { $episode } von { $title } ist drei Tage nach der Ausstrahlung noch nicht aufgetaucht. Dein Feed prüft weiter.

## downloads/src/settings.rs

core-downloads-settings-command-empty = der anirss-Befehl darf nicht leer sein
core-downloads-settings-anirss-address = die anirss-Adresse muss mit http:// oder https:// beginnen
core-downloads-settings-client-address = die Adresse von { $client } muss mit http:// oder https:// beginnen
core-downloads-settings-bad-id = { $id } ist keine Client-ID: nutze Buchstaben, Ziffern, - und _
core-downloads-settings-same-id = zwei Clients haben die ID { $id }
core-downloads-settings-saved-address = die Adresse von { $name } muss mit http:// oder https:// beginnen
core-downloads-settings-same-name = zwei Clients heißen { $name }; gib jedem einen eindeutigen Namen

## downloads/src/qbt.rs

core-downloads-qbt-no-answer = qBittorrent hat unter { $address } nicht reagiert: { $message }
core-downloads-qbt-password-rejected = qBittorrent hat das Passwort abgelehnt; korrigiere es in den Einstellungen
core-downloads-qbt-banned = qBittorrent hat diese Adresse nach fehlgeschlagenen Anmeldungen gesperrt; warte oder starte qBittorrent neu
core-downloads-qbt-login-rejected = qBittorrent hat Benutzername oder Passwort abgelehnt
core-downloads-qbt-login-failed = Die Anmeldung bei qBittorrent unter { $address } lieferte HTTP { $status }; nächster Versuch bei der nächsten Anfrage
core-downloads-qbt-not-web-ui = { $address } lieferte HTTP { $status }, das sieht nicht nach dem Web UI von qBittorrent aus. Prüfe die Adresse in den Einstellungen und ob das Web UI in qBittorrent eingeschaltet ist
core-downloads-qbt-not-added = qBittorrent hat die Torrents nicht hinzugefügt: { $said }
core-downloads-qbt-some-not-added = qBittorrent konnte { $count } der Torrents nicht hinzufügen: { $said }
core-downloads-qbt-answered = qBittorrent lieferte HTTP { $status }: { $line }

## downloads/src/replace.rs

core-downloads-replace-bad-item = Eintrag { $item } ist keins der { $count } Releases
core-downloads-not-replaced = Folge { $episode } wurde nicht ersetzt.
core-downloads-replaced = Folge { $episode } durch den neuen Download ersetzt.
core-downloads-no-video-moved = { $name } enthält kein Video, Anibeam kann es also nicht abspielen. Es liegt jetzt in { $folder }.
core-downloads-no-video = { $name } enthält kein Video, Anibeam kann es also nicht abspielen.

## downloads/src/rolling.rs

core-downloads-no-roll = kein fortlaufender Download { $id }
core-downloads-rolled = { $title }: jetzt bei den Folgen { $from } bis { $to } von { $total }.
core-downloads-roll-ahead = halte mindestens eine Folge Vorsprung
core-downloads-roll-nothing = nichts zu holen

## releases/src/endpoints.rs

core-releases-json-needs-repo = eine JSON-Suchquelle muss aus einer Quellenliste kommen, deren Definition ihre Felder zuordnet
core-releases-endpoint-no-name = `name` fehlt
core-releases-endpoint-bad-name = { $name }: nur Buchstaben, Ziffern, '.', '_', '-'
core-releases-endpoint-json = { $name }: { $why }
core-releases-endpoint-unknown-kind = { $name }: unbekannte Art { $kind } (gültig: { $valid })
core-releases-endpoint-no-url = { $name }: `url` fehlt
core-releases-endpoint-not-http = { $name }: die Adresse muss mit http:// oder https:// beginnen
core-releases-endpoint-needs-query = { $name }: eine { $kind }-URL muss den Platzhalter {"{"}query{"}"} enthalten
core-releases-endpoint-no-query = { $name }: eine { $kind }-URL hängt die Suchanfrage selbst an, darf also kein {"{"}query{"}"} enthalten

## releases/src/fetch.rs

core-releases-indexer-error = der Indexer lieferte Fehler { $code }
core-releases-bad-feed = fehlerhafter Feed: { $error }
core-releases-feed-answered = der Feed lieferte HTTP { $status }

## releases/src/json_feed.rs

core-releases-not-feed-json = nicht das JSON des Feeds: { $error }

## releases/src/json.rs

core-releases-not-json = kein JSON: { $error }
core-releases-search-error-said = die Suche lieferte einen Fehler: { $message }
core-releases-search-error = die Suche lieferte einen Fehler
core-releases-no-results-field = die Antwort hat keine Ergebnisse unter { $path }
core-releases-results-not-list = die Ergebnisse der Antwort sind keine Liste
core-releases-search-answered = die Suche lieferte HTTP { $status }

## releases/src/native.rs

core-releases-no-endpoints = noch keine Downloadquellen: füge zuerst eine Quellenliste oder eine eigene Quelle hinzu
core-releases-no-source-named = keine Quelle namens { $name }
core-releases-empty-query = gib etwas zum Suchen ein
core-releases-cannot-follow = { $name } lässt sich nicht abonnieren: Es unterstützt die Suche, hat aber keinen Feed für neue Folgen
core-releases-endpoint-gone = { $name } ist nicht mehr verfügbar: entfernt, ausgeschaltet oder ohne Schlüssel

## releases/src/quirks.rs

core-releases-quirks-variant = one_variant { $word } muss ein kurzes Wort sein
core-releases-quirks-pattern-count = show_feed nimmt 1 bis { $most } Seitenmuster
core-releases-quirks-pattern-long = ein Seitenmuster von show_feed ist zu lang
core-releases-quirks-pattern-bad = Seitenmuster von show_feed: { $error }
core-releases-quirks-pattern-slug = ein Seitenmuster von show_feed braucht eine Gruppe namens slug
core-releases-quirks-url-https = die URL von show_feed muss https sein
core-releases-quirks-url-host = die URL von show_feed muss auf dem Host des Endpoints liegen
core-releases-quirks-url-slug = die URL von show_feed nimmt {"{"}slug{"}"} genau einmal und keinen anderen Platzhalter

## releases/src/refinement.rs

core-releases-refine-no-tokens = op 'tokens' braucht eine nicht leere Liste `tokens`

## releases/src/grid.rs

core-releases-group-batch = Batch von { $group }
core-releases-episodes-phrase =
    { $count ->
        [one] Folge { $list }
       *[other] Folgen { $list }
    }
core-releases-episode-range = { $from } bis { $to }

## Names that stay the same in every language

-app-name = Anibeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = der Name muss 1 bis { $max } Zeichen haben, ohne Steuerzeichen
core-prefs-room-relay = die Relay-Adresse muss mit https:// oder http:// beginnen
core-room-service = Gemeinsam schauen
core-room-not-found = Kein Raum hat diesen Code. Vielleicht wurde er geschlossen.
core-room-full = Dieser Raum ist voll.
core-room-proto = Dieser Server nutzt eine andere Version von Gemeinsam schauen. Aktualisiere { -app-name }, um beizutreten.
core-room-needs-episode = Dieser Server braucht eine Folge, bevor ein Raum startet. Starte einen im Player oder aktualisiere den Server.
core-room-unreachable = Raumserver nicht erreichbar: { $message }
core-room-refused = Der Raumserver hat die Anfrage abgelehnt: { $message }
core-room-not-in = Du bist in keinem Raum.
core-room-no-name = gib den Anzeigenamen ein, den andere im Raum sehen
core-room-bad-code = Raumcodes bestehen aus 6 Buchstaben oder Ziffern
core-room-created = Raum zum gemeinsamen Schauen gestartet. Code: { $code }.
core-room-joined = Dem Raum { $code } zum gemeinsamen Schauen beigetreten.
core-room-left = Raum zum gemeinsamen Schauen verlassen.
core-room-member-joined = { $name } ist dem Raum beigetreten.
core-room-member-left = { $name } hat den Raum verlassen.
core-room-closed = Der Raum zum gemeinsamen Schauen wurde geschlossen.
core-room-lost = Verbindung zum Raum zum gemeinsamen Schauen verloren. Verbinde neu.
core-room-back = Wieder mit dem Raum zum gemeinsamen Schauen verbunden.
core-room-media-asked = Den Host um die Folge gebeten.
core-room-media-cannot = Der Host kann diese Folge gerade nicht teilen.
core-room-media-sent = { $name } den Torrent der Folge geschickt.
core-room-media-sent-name = { $name } den Namen des Releases der Folge geschickt.
core-room-media-nothing = Der Host hat keine Kopie der Folge zum Teilen.
core-room-media-no-answer = Die App des Hosts hat nicht reagiert. Versuch es gleich noch einmal.
core-room-media-torrent = Lade die Folge über den Torrent des Hosts herunter.
core-room-media-searching = Suche in deinen Quellen nach dem Release des Hosts.
core-room-media-not-found = Keine deiner Quellen hat das Release des Hosts.
core-room-media-no-sources = Alle Quellen sind ausgeschaltet. Schalte eine ein, um zu suchen.
core-room-media-no-start = Der Download ließ sich nicht starten: { $message }
core-room-media-client = der Torrent-Client hat ihn abgelehnt
core-room-media-lost = Der Torrent-Client hat den Download der Folge verloren.
core-room-media-stalled = Der Download hängt seit zehn Minuten fest. Versuch es später noch einmal.
core-room-media-not-in-library = Die Folge ist fertig geladen, aber nicht in deiner Bibliothek aufgetaucht. Prüfe, ob Downloads in einem Bibliotheksordner landen.
core-room-media-here = Die Folge ist jetzt in deiner Bibliothek.
core-room-media-rest-missing = Der Rest der Serie war nicht zu bekommen. Nur diese Folge wird geladen.
