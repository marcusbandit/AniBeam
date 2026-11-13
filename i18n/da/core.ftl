# da translation of en/core.ftl. An ID missing here is shown in English.

## Names that stay the same in every language. Messages use them; none is shown alone.

## The tailnet host

tailnet-person-no-name = Intet navn i Tailscale

## Counts

count-episodes =
    { $count ->
        [one] { $count } afsnit
       *[other] { $count } afsnit
    }
count-shows =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serier
    }

## Actions every screen uses

action-cancel = Annuller
action-close = Luk
action-try-again = Prøv igen
action-remove = Fjern
action-delete = Slet
action-open = Åbn
action-save = Gem
action-apply = Anvend
action-clear = Ryd
action-refresh = Opdater
action-play = Afspil
action-connect = Forbind
action-start = Start
action-back = Tilbage
action-see-all = Se alle
action-open-series = Åbn serie
action-open-settings = Åbn indstillinger

## The lists a tracker keeps

status-watching = Ser nu
status-completed = Færdig
status-paused = På pause
status-on-hold = På pause
status-dropped = Droppet
status-planning = Planlagt
status-rewatching = Genser

## Episodes

episode-number = Afsnit { $number }

## AniList's words for a show: its format, its airing status, how two shows relate

anilist-format-tv = TV
anilist-format-tv-short = TV kort
anilist-format-film = Film
anilist-format-special = Special
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Musik
anilist-status-finished = Afsluttet
anilist-status-airing = Sendes
anilist-status-not-yet-aired = Ikke sendt endnu
anilist-status-cancelled = Aflyst
anilist-status-hiatus = På pause
anilist-relation-adaptation = Tilpasning
anilist-relation-prequel = Prequel
anilist-relation-sequel = Sequel
anilist-relation-parent = Hovedhistorie
anilist-relation-side-story = Sidehistorie
anilist-relation-character = Figur
anilist-relation-summary = Resumé
anilist-relation-alternative = Alternativ
anilist-relation-spin-off = Spin-off
anilist-relation-other = Andet
anilist-relation-source = Kilde
anilist-relation-compilation = Samling
anilist-relation-contains = Indeholder

## The activity log: a problem whose place had no words of its own

event-problem-library = Noget gik galt med dit bibliotek. Udviklerindstillinger har detaljerne.
event-problem-metadata = Noget gik galt med serieinfo. Udviklerindstillinger har detaljerne.
event-problem-trackers = Noget gik galt med dine lister. Udviklerindstillinger har detaljerne.
event-problem-franchise = Noget gik galt med relaterede serier. Udviklerindstillinger har detaljerne.
event-problem-playback = Noget gik galt med afspilningen. Udviklerindstillinger har detaljerne.
event-problem-store = Noget gik galt, da dine data skulle gemmes. Udviklerindstillinger har detaljerne.
event-problem-system = Noget gik galt med AniBeam. Udviklerindstillinger har detaljerne.
event-problem-downloads = Noget gik galt med downloads. Udviklerindstillinger har detaljerne.

## Errors (contract::CoreError)

core-error-not-found-source = Kunne ikke finde mappe { $id }.
core-error-not-found-series = Kunne ikke finde serie { $id }.
core-error-not-found-file = Kunne ikke finde fil { $id }.
core-error-not-found-session = Kunne ikke finde afspilning { $id }.
core-error-not-found-job = Kunne ikke finde opgave { $id }.
core-error-invalid = Ugyldig { $field }: { $message }.
core-error-invalid-ended = Ugyldig { $field }: { $message }
core-error-unavailable = Denne mappe er ikke tilgængelig: { $path }
core-error-not-connected = { $tracker } er ikke forbundet.
core-error-refused-hidden = Det kan ikke lade sig gøre: serien er skjult.
core-error-refused-no-match = Det kan ikke lade sig gøre: serien er ikke matchet endnu; match den under Serieinfo.
core-error-refused-not-newer = Det kan ikke lade sig gøre: din liste er allerede forbi dette afsnit.
core-error-refused-extra = Det kan ikke lade sig gøre: dette er en ekstra.
core-error-refused-unmatched = Det kan ikke lade sig gøre: filen er ikke matchet med et afsnit.
core-error-refused-on-disk = Det kan ikke lade sig gøre: filen findes kun på disken.
core-error-provider = { $provider } svarede ikke: { $message }
core-error-io = Kunne ikke læse eller skrive en fil: { $message }
core-error-storage = Kunne ikke gemme dit bibliotek: { $message }
core-error-keyring = Kunne ikke nå systemets adgangskodelager: { $message }
core-error-unsupported = { $what } understøttes ikke endnu.
core-error-version = Denne eksport er fra en nyere AniBeam (format { $found }). Opdater AniBeam for at importere den.
core-error-internal = AniBeam ramte en intern fejl: { $message }
core-error-service = { $service }: { $message }

## core/src/repos/mod.rs

core-repos-paste-address = indsæt adressen på et repo
core-repos-must-be-https = adressen skal være https
core-repos-unreachable = kunne ikke nå repoet: { $message }
core-repos-not-found = intet på den adresse (et privat GitHub-repo svarer også 404)
core-repos-http-status = repoet svarede HTTP { $status }
core-repos-unreadable = kunne ikke læse { $path }: { $error }
core-repos-no-such-endpoint = intet installeret endpoint med det id
core-repos-key-changed = repoet er nu signeret med en anden nøgle end den, du stolede på. Fjern det og tilføj det igen for at godkende den nye nøgle.
core-repos-stale = repoet sendte et ældre indeks (serienummer { $serial } efter { $kept }); beholder det nyere
core-repos-key-not-shown = repoets nøgle er ikke den, du fik vist. Tilføj det igen og sammenlign fingeraftrykket.
core-repos-repo-added = Tilføjede en liste over downloadkilder.
core-repos-repo-removed = Fjernede en liste over downloadkilder.
core-repos-no-field = { $name } har intet felt { $key }
core-repos-needs-field = { $name } mangler sin { $label }
core-repos-host-only = { $name }: { $label } er kun en vært eller vært:port, som 192.0.2.10:9696
core-repos-not-added = det repo er ikke tilføjet
core-repos-not-offered = { $repo } tilbyder ikke { $id }
core-repos-already-installed = { $name } er allerede installeret
core-repos-taken-by-you = et endpoint ved navn { $id } er allerede installeret (tilføjet af dig); fjern det først
core-repos-taken-by-repo = et endpoint ved navn { $id } er allerede installeret (fra et andet repo); fjern det først
core-repos-source-added = Tilføjede en downloadkilde.
core-repos-source-updated = Opdaterede en downloadkilde.
core-repos-source-removed = Fjernede en downloadkilde.
core-repos-source-changed = Ændrede en downloadkilde.
core-repos-give-name = giv den et navn
core-repos-put-query = sæt {"{"}query{"}"} i adressen, hvor søgningen skal stå
core-repos-name-taken = et endpoint ved navn { $id } er allerede installeret
core-repos-only-by-hand = kun en kilde, der er tilføjet manuelt, kan redigeres
core-repos-keep-key = behold {"{"}{ $key }{"}"} i adressen og skift dens værdi med Skift dens { $label }, eller fjern siden og tilføj den igen uden
core-repos-note-kind = anirss kan ikke søge i typen { $kind }, så computeren udelader den; telefonen søger i den
core-repos-note-by-hand = anirss har sin egen { $id }, skrevet manuelt; computeren søger i den
core-repos-note-theirs = Denne anirss kører du selv, og computeren ændrer aldrig dens konfiguration: tilføj { $id } til den anirss' egen konfiguration, så computeren søger i den
core-repos-note-sandbox = Denne kørsel skriver aldrig i anirss' konfiguration, så computeren søger ikke i { $id } her
core-repos-note-old = anirss { $version } kan ikke tage imod endpoints fra { -app-name }; opdater den til { $min } eller nyere, så computeren søger i dette

## core/src/repos/index.rs

core-repos-key-not-ed25519 = indeksets nøgle er ikke en ed25519-nøgle
core-repos-key-not-base64 = indeksets nøgle er ikke 32 bytes base64
core-repos-key-invalid = indeksets nøgle er ikke en gyldig ed25519-nøgle
core-repos-too-large = filen er for stor til at være et repo
core-repos-not-a-repo = dette er ikke et { -app-name }-endpointrepo
core-repos-format-version = repoet er formatversion { $found }; denne { -app-name } læser version { $reads }. Opdater { -app-name }.
core-repos-payload-not-base64 = repoets indhold er ikke base64
core-repos-signature-malformed = repoets signatur er ugyldig
core-repos-index-unreadable = repoets indeks kan ikke læses: { $error }
core-repos-signature-mismatch = repoets signatur passer ikke til indholdet
core-repos-check-scheme = { $id }: adressen skal starte med https://
core-repos-check-scheme-http = { $id }: adressen skal starte med http:// eller https://
core-repos-check-auth-key = { $id }: ugyldig auth-nøgle { $key }
core-repos-check-host-torznab = { $id }: kun en torznab-adresse må angive sin vært som {"{"}{ $key }{"}"}
core-repos-check-host-placeholder = { $id }: en pladsholder i værten

## core/src/playback/tracks.rs

core-playback-track-number = Spor { $number }
core-playback-track-file = { $label } (fil)
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = Engelsk
core-playback-lang-japanese = Japansk
core-playback-lang-spanish = Spansk
core-playback-lang-portuguese = Portugisisk
core-playback-lang-french = Fransk
core-playback-lang-german = Tysk
core-playback-lang-italian = Italiensk
core-playback-lang-russian = Russisk
core-playback-lang-chinese = Kinesisk
core-playback-lang-korean = Koreansk
core-playback-lang-arabic = Arabisk
core-playback-lang-hindi = Hindi
core-playback-lang-indonesian = Indonesisk
core-playback-lang-malay = Malajisk
core-playback-lang-thai = Thai
core-playback-lang-vietnamese = Vietnamesisk
core-playback-lang-polish = Polsk
core-playback-lang-turkish = Tyrkisk
core-playback-lang-dutch = Hollandsk
core-playback-lang-swedish = Svensk
core-playback-lang-norwegian = Norsk
core-playback-lang-danish = Dansk
core-playback-lang-finnish = Finsk
core-playback-lang-hebrew = Hebraisk
core-playback-lang-ukrainian = Ukrainsk
core-playback-lang-czech = Tjekkisk
core-playback-lang-hungarian = Ungarsk
core-playback-lang-greek = Græsk
core-playback-lang-romanian = Rumænsk
core-playback-lang-filipino = Filippinsk
core-playback-region-brazil = Brasilien
core-playback-region-portugal = Portugal
core-playback-region-latin-america = Latinamerika
core-playback-region-spain = Spanien
core-playback-region-simplified = Forenklet
core-playback-region-traditional = Traditionelt

## contract/src/enums.rs

core-jobs-scan = Tjekker dine mapper
    .stopped = Tjek af dine mapper stoppede med et problem.
core-jobs-auto-match = Matcher serier
    .stopped = Matchning af serier stoppede med et problem.
core-jobs-search = Søger efter en serie
    .stopped = Søgningen efter en serie stoppede med et problem.
core-jobs-resolve-link = Læser et link
    .stopped = Læsning af et link stoppede med et problem.
core-jobs-apply-match = Matcher en serie
    .stopped = Matchning af en serie stoppede med et problem.
core-jobs-refresh = Opdaterer en serie
    .stopped = Opdatering af en serie stoppede med et problem.
core-jobs-refresh-all = Opdaterer serieinfo
    .stopped = Opdatering af serieinfo stoppede med et problem.
core-jobs-refresh-airing = Tjekker, hvad der sendes
    .stopped = Tjek af, hvad der sendes, stoppede med et problem.
core-jobs-refresh-imdb = Opdaterer bedømmelser
    .stopped = Opdatering af bedømmelser stoppede med et problem.
core-jobs-clear-images = Rydder billeder
    .stopped = Rydning af billeder stoppede med et problem.
core-jobs-fill-images = Henter billeder
    .stopped = Hentning af billeder stoppede med et problem.
core-jobs-connect-tracker = Forbinder din liste
    .stopped = Forbindelsen til din liste stoppede med et problem.
core-jobs-mark = Markerer et afsnit som set
    .stopped = Markering af et afsnit som set stoppede med et problem.
core-jobs-set-progress = Sætter din fremgang
    .stopped = Ændring af din fremgang stoppede med et problem.
core-jobs-score = Gemmer din karakter
    .stopped = Gemning af din karakter stoppede med et problem.
core-jobs-refresh-progress = Læser din fremgang på listen
    .stopped = Læsning af din fremgang stoppede med et problem.
core-jobs-refresh-watching = Læser din Ser nu-liste
    .stopped = Læsning af din Ser nu-liste stoppede med et problem.
core-jobs-refresh-airing-now = Tjekker denne sæson
    .stopped = Tjek af denne sæson stoppede med et problem.
core-jobs-refresh-recommended = Finder serier til dig
    .stopped = Søgningen efter serier til dig stoppede med et problem.
core-jobs-read-show-page = Læser en serie
    .stopped = Læsning af en serie stoppede med et problem.
core-jobs-on-hold = Flytter inaktive serier til På pause
    .stopped = Flytning af inaktive serier til På pause stoppede med et problem.
core-jobs-set-status = Ændrer en listestatus
    .stopped = Ændring af en listestatus stoppede med et problem.
core-jobs-move-lists = Flytter din liste til din tracker
    .stopped = Flytning af din liste til din tracker stoppede med et problem.
core-jobs-notes = Gemmer dine noter
    .stopped = Gemning af dine noter stoppede med et problem.
core-jobs-crawl = Finder relaterede serier
    .stopped = Søgningen efter relaterede serier stoppede med et problem.
core-jobs-skip-windows = Finder intro og outro
    .stopped = Søgningen efter intro og outro stoppede med et problem.
core-jobs-export = Gemmer dit bibliotek i en fil
    .stopped = Gemning af dit bibliotek i en fil stoppede med et problem.
core-jobs-import = Indlæser dit bibliotek fra en fil
    .stopped = Indlæsning af dit bibliotek fra en fil stoppede med et problem.
core-jobs-subscriptions = Tjekker dine abonnementer
    .stopped = Tjek af dine abonnementer stoppede med et problem.
core-jobs-connect-downloads = Forbinder downloadprogrammet
    .stopped = Forbindelsen til downloadprogrammet stoppede med et problem.
core-jobs-search-releases = Leder efter afsnit
    .stopped = Søgningen efter afsnit stoppede med et problem.
core-jobs-refine-releases = Indsnævrer afsnit
    .stopped = Indsnævringen af afsnit stoppede med et problem.
core-jobs-subscribe = Abonnerer på en serie
    .stopped = Abonnementet på en serie stoppede med et problem.
core-jobs-download = Tilføjer downloads
    .stopped = Tilføjelse af downloads stoppede med et problem.
core-jobs-remove-subscription = Fjerner et abonnement
    .stopped = Fjernelse af et abonnement stoppede med et problem.
core-jobs-control-torrents = Ændrer dine downloads
    .stopped = Ændring af dine downloads stoppede med et problem.
core-jobs-test-torrent-client = Tester downloadprogrammet
    .stopped = Test af downloadprogrammet stoppede med et problem.
core-jobs-remove-show = Fjerner en serie
    .stopped = Fjernelse af en serie stoppede med et problem.
core-jobs-repair-subscription = Reparerer et abonnement
    .stopped = Reparation af et abonnement stoppede med et problem.
core-jobs-check-endpoints = Tjekker dine downloadkilder
    .stopped = Tjek af dine downloadkilder stoppede med et problem.
core-jobs-start-endpoint = Starter en downloadkilde
    .stopped = Start af en downloadkilde stoppede med et problem.
core-jobs-schedule-agent = Spørger din kodeagent om en sendeplan
    .stopped = Spørgsmålet til din kodeagent om en sendeplan stoppede med et problem.

## core/src/airing_now.rs

core-airing-now-now = Sendes nu
core-airing-now-winter = Vinter { $year }
core-airing-now-spring = Forår { $year }
core-airing-now-summer = Sommer { $year }
core-airing-now-fall = Efterår { $year }
core-airing-now-planned = Føjede { $title } til din Planlagt-liste på { $site }.
core-airing-now-planned-both = Føjede { $title } til din Planlagt-liste på { $site } og { $other }.
core-airing-now-already-on =
    { $title } er allerede på din { $site }-liste som { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }.
core-airing-now-plan-failed = Kunne ikke føje { $title } til din Planlagt-liste. Prøv igen senere.

## core/src/aired.rs

core-aired-episode-out = Afsnit { $number } af { $title } er udkommet.

## core/src/cast/mod.rs

core-cast-connecting = Caster til { $device }.
core-cast-ended = Færdig med at caste til { $device }.
core-cast-stopped = Stoppede cast til { $device }.
core-cast-failed = Cast til { $device } mislykkedes. Tjek, at den er på samme netværk.
core-cast-not-built = Cast er ikke med i denne version
core-cast-serve-failed = kunne ikke levere afsnittet: { $error }
core-cast-device-gone = Den enhed er ikke længere på netværket
core-cast-not-a-playlist = ikke en playlisteadresse
core-cast-nothing = Intet castes
core-cast-still-starting = Enheden starter stadig
core-cast-no-route = denne enhed har ingen rute til tv'et
core-cast-no-control = tv'et gav ingen kontroladresse
core-cast-not-handed = den anden computer gav ikke afsnittet videre: { $error }
core-cast-empty-playlist = den anden computer sendte en tom playliste
core-cast-refused-both = { $error }; den afviste også filen, som den er ({ $first })
core-cast-chromecast-stopped = Chromecasten stoppede
core-cast-hevc = { $error }. Dette afsnit er HEVC, som kun nyere Chromecasts (med Google TV, eller Ultra) kan afkode

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = kunne ikke nå Chromecasten: { $error }
core-cast-chromecast-broke = forbindelsen til Chromecasten blev afbrudt: { $error }
core-cast-chromecast-no-player = Chromecasten startede ikke sin medieafspiller
core-cast-chromecast-refused-player = Chromecasten ville ikke starte sin medieafspiller ({ $reason })
core-cast-no-reason = ingen grund
core-cast-chromecast-cannot-play = Chromecasten kunne ikke afspille dette ({ $kind })
core-cast-chromecast-error = Chromecasten stoppede med en fejl under afspilningen

## core/src/cast/dlna.rs

core-cast-tv-no-answer = tv'et svarede ikke: { $error }
core-cast-tv-refused = tv'et afviste { $action }
core-cast-tv-refused-code = tv'et afviste { $action } ({ $code })

## core/src/cast/media.rs

core-cast-no-video = Denne fil har ingen video.
core-cast-hi10p = Dette afsnit er 10-bit H.264 (Hi10P), som intet tv eller Chromecast kan afkode. Det skal omkodes for at kunne castes, og det er ikke bygget endnu.
core-cast-video-codec = Dette afsnits video ({ $codec }) kan ikke castes.
core-cast-audio-codec = Dette afsnits lyd ({ $codec }) kan ikke castes.

## core/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Begyndte at downloade et afsnit af { $folder }.
       *[other] Begyndte at downloade { $count } afsnit af { $folder }.
    }
core-phone-subscribed = Abonnerer nu på { $name }.
core-phone-rule-stopped = { $name } tjekkes ikke for nye afsnit: dens kilde er væk eller skal have sin nøgle igen.
core-phone-new-episode = Et nyt afsnit af { $name } downloades.
core-phone-name-taken = { $name }: en anden kilde har det navn
core-phone-nothing-to-download = intet at downloade
core-phone-over-cap = det ville tage { -app-name } over lagergrænsen ({ $used } af { $cap } GB brugt eller på vej); slet afsnit eller hæv grænsen
core-phone-empty-search = søgningen er tom
core-phone-no-results = ingen resultater for { $query }
core-phone-own-engine = en egen torrentmotor

## core/src/phone/check.rs

core-phone-check-connected = forbundet
core-phone-check-refused = afvist: { $error }
core-phone-check-dropped = intet svar (droppet)
core-phone-check-finding = { $label }: finder adressen
core-phone-check-failed = mislykkedes: { $error }
core-phone-check-no-answer = intet svar
core-phone-check-private = { $label }: finder den privat
core-phone-check-skipped-address = sprunget over, en adresse
core-phone-check-skipped-home = sprunget over, navn på hjemmenetværket
core-phone-check-no-record = ingen offentlig adresse, overladt til dette netværks DNS
core-phone-check-no-resolver = ingen DNS-server svarede
core-phone-check-reaching = { $label }: når { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } bytes
core-phone-check-setup = opsætning

## core/src/playback/session.rs

core-playback-not-saved = Din fremgang i { $folder } blev ikke gemt. Match serien først.
core-playback-still-downloading = afsnit { $number } downloades stadig, { $done } % færdigt

## core/src/playback/skip.rs

core-playback-bad-duration = en varighed er et positivt antal sekunder

## core/src/agent.rs

core-agent-no-answer = agenten gav intet svar
core-agent-searching = Søger på nettet efter { $query }
core-agent-searching-schedule = Søger på nettet efter sendeplanen
core-agent-reading = Læser { $url }
core-agent-reading-page = Læser en side
core-agent-using = Bruger { $tool }
core-agent-none = en kodeagent på denne computer
core-agent-ended = { $name } sluttede uden et svar
core-agent-too-long = { $name } brugte for lang tid

## core/src/core.rs

core-core-guest-trackers = trackerkontiene på den computer tilhører dens ejer, så denne enhed markerer intet dér
core-core-guest-not-interested = Ikke interesseret på den computer er dens ejers liste, så denne enhed ændrer intet dér
core-core-guest-recommended = anbefalingerne på den computer er lært af dens ejers liste
core-core-nothing-picked = intet valgt

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider } er ikke en matchkilde
core-prefs-provider-count = { $provider } optræder { $count } gange; hver kilde optræder én gang
core-prefs-outside = { $value } er uden for { $low } til { $high }
core-prefs-empty = tom

## core/src/recommended/mod.rs

core-recommended-ranking = Rangerer serier til dig

## core/src/recommended/sync.rs

core-recommended-finding-people = Finder folk med din smag
core-recommended-reading-lists = Læser deres lister
core-recommended-reading-yours = Læser dine serier
core-recommended-reading-suggestions = Læser serier at foreslå

## core/src/recommended/page.rs

core-recommended-no-anime = { -anilist } har ingen anime { $id }

## core/src/host/mod.rs

core-host-off = Fra
core-host-starting = Starter
core-host-serving = Deler på { $addresses }
core-host-no-tailscale = Tailscale er ikke forbundet
core-host-cannot-listen = Kunne ikke lytte
core-host-port-taken = Port { $port } er optaget
core-host-cannot-listen-on = Kunne ikke lytte på { $address }: { $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text } er ikke en adresse og port

## core/src/host/keys.rs

core-host-not-a-key = { $key } er ikke en enhedsnøgle

## core/src/net/limiter.rs

core-net-timed-out = fik timeout efter { $ms } ms
core-net-rate-limited = { $provider } begrænser antallet af forespørgsler

## core/src/net/mod.rs

core-net-waiting = { $host }: fik timeout for et øjeblik siden; { -app-name } venter { $seconds } s, før den spørger igen

## core/src/net/anilist.rs

core-net-no-data = ingen data

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } har ingen download til denne computer
core-update-cannot-make = Kunne ikke oprette { $path }: { $error }
core-update-cannot-mark = Kunne ikke gøre det nye AppImage kørbart: { $error }
core-update-cannot-replace = Kunne ikke erstatte { $path }: { $error }
core-update-not-here = Denne kopi opdateres ikke her
core-update-cannot-start = Kunne ikke starte downloadet: { $error }
core-update-did-not-start = Downloadet startede ikke: { $error }
core-update-cannot-write = Kunne ikke skrive { $path }: { $error }
core-update-stopped = Downloadet stoppede: { $error }
core-update-cancelled = Annulleret
core-update-ended-early = Downloadet sluttede for tidligt ({ $got } af { $size } bytes)
core-update-checksum = Downloadet passer ikke med GitHubs checksum
core-update-cannot-move = Kunne ikke flytte downloadet på plads: { $error }
core-update-cannot-open-dmg = Kunne ikke åbne dmg-filen: { $error }
core-update-cannot-read-dmg = Kunne ikke læse dmg-filen: { $error }
core-update-no-app = dmg-filen indeholder ingen app
core-update-cannot-copy = Kunne ikke kopiere den nye app ved siden af denne: { $error }

## core/src/calendar.rs

core-calendar-season-episode = { $title } - Sæson { $season } Afsnit { $number }
core-calendar-episode = { $title } - Afsnit { $number }

## core/src/paths.rs

core-paths-no-home = ingen hjemmemappe

## core/src/first_run.rs

core-first-run-not-absolute = videomappen skal være en absolut sti

## core/src/presence/mod.rs

core-presence-extra = Ekstra

## core/src/trackers/accounts.rs

core-trackers-no-client-id = Intet klient-id sat for { $tracker }.
core-trackers-mal-reconnect = Forbind MyAnimeList igen i Indstillinger.
core-trackers-mal-wants-secret = MyAnimeList vil have denne apps klienthemmelighed. Tilføj den under Trackere i Indstillinger, eller registrer appen med App Type "other".
core-trackers-token-reply-unreadable = svaret med token kunne ikke læses: { $error }
core-trackers-token-reply-no-token = svaret indeholdt ingen adgangstoken

## core/src/trackers/mod.rs

core-trackers-no-token = ingen adgangstoken gemt, forbind igen i Indstillinger

## core/src/trackers/cache.rs

core-trackers-no-user-id = kontoen har intet bruger-id

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = ingen serier valgt at flytte
core-trackers-moved-up =
    { $count ->
        [one] Flyttede { $count } serie fra denne computer til { $site }.
       *[other] Flyttede { $count } serier fra denne computer til { $site }.
    }
core-trackers-moved-up-some-refused =
    Flyttede { $moved ->
        [one] { $moved } serie
       *[other] { $moved } serier
    } til { $site }; { $site } tog ikke imod { $failed ->
        [one] { $failed } serie
       *[other] { $failed } serier
    }.
core-trackers-move-refused = { $site } tog ikke imod { $show }: { $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = Godkendelsen fik timeout, prøv igen.
core-trackers-oauth-state-mismatch = state passer ikke, mulig CSRF, afbrudt.
core-trackers-oauth-no-code = ingen kode i callback
core-trackers-oauth-no-token = ingen access_token i callback
core-trackers-oauth-waiting = Venter på, at du logger ind på { $site } i din browser.
core-trackers-oauth-connected = Forbandt { $site } som { $username }.
core-trackers-oauth-no-name = profilen kom tilbage uden et navn
core-trackers-oauth-could-not-start = Login kunne ikke starte
core-trackers-oauth-port-failed = port { $port } på denne computer kunne ikke åbnes: { $error }
core-trackers-oauth-port-busy = port { $port } på denne computer bruges af et andet program. Vent et øjeblik og prøv igen, eller luk programmet, der bruger den, hvis den bliver ved med at være optaget.
core-trackers-oauth-provider-error = { $site } returnerede en fejl: { $error }
core-trackers-page-connected = { $site } er forbundet med { $app }
core-trackers-page-close-tab = Du kan lukke denne fane.
core-trackers-page-failed-title = { $app }, login mislykkedes
core-trackers-page-failed = Login blev ikke gennemført
core-trackers-page-go-back = Gå tilbage til { $app } og prøv igen.
core-trackers-page-finishing-title = { $app }, gør stadig færdig
core-trackers-page-finishing = Gør stadig færdig
core-trackers-page-finishing-body = { $site } svarede, og AniBeam gør login færdigt. Gå tilbage til { $app }, som siger til, når det er klar.
core-trackers-page-cancelled-title = { $app }, login annulleret
core-trackers-page-cancelled = Dette login blev annulleret
core-trackers-page-cancelled-body = Du kan lukke denne fane. Start igen fra { $app }, hvis det var meningen.

## core/src/trackers/on_hold.rs

core-trackers-on-hold-moving = Flytter til På pause på { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] Flyttede { $count } serie til På pause på { $site }.
       *[other] Flyttede { $count } serier til På pause på { $site }.
    }
core-trackers-on-hold-moved-two =
    { $count ->
        [one] Flyttede { $count } serie til På pause på { $site } og { $second } på { $secondSite }.
       *[other] Flyttede { $count } serier til På pause på { $site } og { $second } på { $secondSite }.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } serie kunne ikke flyttes til På pause.
       *[other] { $failed } serier kunne ikke flyttes til På pause.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] Flyttede { $count } serie til På pause på { $site }.
       *[other] Flyttede { $count } serier til På pause på { $site }.
    } { $failed ->
        [one] { $failed } serie kunne ikke flyttes til På pause.
       *[other] { $failed } serier kunne ikke flyttes til På pause.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] Flyttede { $count } serie til På pause på { $site } og { $second } på { $secondSite }.
       *[other] Flyttede { $count } serier til På pause på { $site } og { $second } på { $secondSite }.
    } { $failed ->
        [one] { $failed } serie kunne ikke flyttes til På pause.
       *[other] { $failed } serier kunne ikke flyttes til På pause.
    }
core-trackers-series-fallback = serie { $id }
core-trackers-back-on-watching = { $name } er tilbage på Ser nu på { $site }.
core-trackers-back-on-watching-two = { $name } er tilbage på Ser nu på { $site } og { $secondSite }.
core-trackers-back-on-watching-failed = { $name } kunne ikke komme tilbage på Ser nu. Prøv igen senere.

## core/src/trackers/writes.rs

core-trackers-rate-limited = { $tracker } begrænser forespørgsler, prøv igen om et minut.
core-trackers-auth-expired = { $tracker }-login er udløbet, forbind igen i Indstillinger.
core-trackers-entry-not-found = { $tracker }-posten blev ikke fundet.
core-trackers-server-error = { $tracker }-serverfejl ({ $status }), prøv igen senere.
core-trackers-error-see-log = { $tracker }-fejl, se aktivitetsloggen.
core-trackers-episode-not-whole = { $episode } er ikke et helt afsnitsnummer over nul
core-trackers-score-out-of-range = { $score } er uden for 0 til 10
core-trackers-rewatch-not-settable = genser er ikke en status, man sætter manuelt
core-trackers-notes-too-long = noter må højst være { $max } tegn
core-trackers-nothing-picked = intet valgt
core-trackers-notes-not-on-list = Ikke på din { $tracker }-liste endnu, føj den til en liste først.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } og { -mal }
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
       *[repeating] Genser
    }
core-trackers-marked = Markerede afsnit { $episode } af { $show } på { core-trackers-sites }.
core-trackers-progress-set = Satte din fremgang i { $show } til { $progress } på { core-trackers-sites }.
core-trackers-scored = Gav { $show } karakteren { $score } på { core-trackers-sites }.
core-trackers-score-cleared = Ryddede din karakter for { $show } på { core-trackers-sites }.
core-trackers-status-set = Satte { $show } til { core-trackers-status } på { core-trackers-sites }.
core-trackers-notes-saved = Gemte dine noter om { $show } på { core-trackers-sites }.
core-trackers-notes-cleared = Ryddede dine noter om { $show } på { core-trackers-sites }.
core-trackers-update-failed = Kunne ikke opdatere { $site }. Prøv igen senere.
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
            [one] { $done } serie
           *[other] { $done } serier
        }
    }
core-trackers-written-done =
    { $action ->
        [clear] Ryddede din karakter for { core-trackers-written-who } på { core-trackers-sites }.
        [status] Satte { core-trackers-written-who } til { core-trackers-status } på { core-trackers-sites }.
       *[score] Gav { core-trackers-written-who } karakteren { $score } på { core-trackers-sites }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] { $failed } serie kunne ikke opdateres: { $reason }.
           *[other] { $failed } serier kunne ikke opdateres: { $reason }.
        }
       *[no] { $failed ->
            [one] { $failed } serie kunne ikke opdateres.
           *[other] { $failed } serier kunne ikke opdateres.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] { $failed } kunne ikke opdateres: { $reason }.
       *[no] { $failed } kunne ikke opdateres.
    }
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } serie har intet match hos en tracker.
       *[other] {" "}{ $noMatch } serier har intet match hos en tracker.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } serie er skjult, så dens synkronisering er slået fra.
       *[other] {" "}{ $hidden } serier er skjult, så deres synkronisering er slået fra.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } serie findes kun hos en tracker, der ikke er forbundet.
       *[other] {" "}{ $notConnected } serier findes kun hos en tracker, der ikke er forbundet.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } serie gemmes på denne computer uden en tracker at sende til.
       *[other] {" "}{ $kept } serier gemmes på denne computer uden en tracker at sende til.
    }

## core/src/metadata/automatch.rs

core-metadata-matched-to = Matchede { $show } med { $title }.
core-metadata-auto-matched =
    { $count ->
        [one] Matchede { $count } serie.
       *[other] Matchede { $count } serier.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } serie mangler stadig et match.
       *[other] { $count } serier mangler stadig et match.
    }
core-metadata-auto-matched-some =
    Matchede { $matched ->
        [one] { $matched } serie
       *[other] { $matched } serier
    }; { $unmatched } mangler stadig { $unmatched ->
        [one] et match
       *[other] et match
    }.
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
        [yes] Matchede { $hitFolder } med { $hitTitle }.
       *[no] { $matched ->
            [one] Matchede { $matched } serie.
           *[other] Matchede { $matched } serier.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] Intet match fundet for { $missName }.
       *[no] { $unmatched ->
            [one] Intet match fundet for { $unmatched } serie.
           *[other] Intet match fundet for { $unmatched } serier.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] Der kunne ikke søges efter { $failed } serie; prøv igen senere.
       *[other] Der kunne ikke søges efter { $failed } serier; prøv igen senere.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } serie havde allerede et match.
       *[other] { $had } serier havde allerede et match.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } serie er allerede ved at blive matchet.
       *[other] { $busy } serier er allerede ved at blive matchet.
    }

## core/src/metadata/apply.rs

core-metadata-link-unreadable = Kunne ikke læse det link. Indsæt en sideadresse fra AniList, MyAnimeList, TVmaze eller IMDb.
core-metadata-no-mal-entry = AniList har ingen post for det MyAnimeList-id.
core-metadata-no-imdb-show = TVmaze har ingen serie for den IMDb-titel.
core-metadata-search-on = søgning på { $provider }
core-metadata-query-too-short = Skriv mindst to tegn for at søge.
core-metadata-not-a-link = ikke et link
core-metadata-refresh-unsupported = opdatering af et match kun fra MAL eller TMDB
core-metadata-matched = Matchede { $show }.
core-metadata-updated = Opdaterede info om { $show }.
core-metadata-update-failed = Kunne ikke opdatere info om { $show }. Prøv igen senere.
core-metadata-progress-refresh = opdatering
core-metadata-progress-backfill = udfyldning
core-metadata-updated-many =
    { $count ->
        [one] Opdaterede info om { $count } serie.
       *[other] Opdaterede info om { $count } serier.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] Kunne ikke opdatere info om { $count } serie. Prøv igen senere.
       *[other] Kunne ikke opdatere info om { $count } serier. Prøv igen senere.
    }
core-metadata-updated-some =
    { $count ->
        [one] Opdaterede info om { $count } serie; { $failed } kunne ikke opdateres.
       *[other] Opdaterede info om { $count } serier; { $failed } kunne ikke opdateres.
    }

## core/src/metadata/fetch.rs

core-metadata-no-such-show = TVmaze har ingen serie { $id }
core-metadata-no-such-media = AniList har intet medie { $id }

## core/src/metadata/imdb.rs

core-metadata-progress-imdb-links = Wikidatas IMDb-links
core-metadata-progress-imdb-ratings = IMDb-bedømmelser

## core/src/franchise/layout.rs

core-franchise-relation-character = Fælles figurer

## core/src/images/mod.rs

core-images-progress-fetching = henter billeder

## core/src/transfer/export.rs

core-transfer-exported =
    { $count ->
        [one] Gemte { $count } serie i { $folder }.
       *[other] Gemte { $count } serier i { $folder }.
    }

## core/src/transfer/format.rs

core-transfer-not-json = ikke JSON: { $error }
core-transfer-wrong-format = ikke et { $format }-dokument

## core/src/transfer/import.rs

core-transfer-imported = Indlæste dit bibliotek fra filen.

## core/src/library/delete.rs

core-library-no-such-file = ingen fil { $id } i biblioteket
core-library-file-elsewhere-not-removed = fil { $id } hører til en anden serie, så intet blev fjernet
core-library-file-outside = { $path } er ikke en fil i en biblioteksmappe, så intet blev fjernet
core-library-episodes-removed =
    { $count ->
        [one] { $count } afsnit
       *[other] { $count } afsnit
    } af { $title } { $place ->
        [deleted] slettet
       *[trash] flyttet til papirkurven
    }.
core-library-finished-removed =
    { $count ->
        [one] { $count } færdig serie
       *[other] { $count } færdige serier
    } { $place ->
        [deleted] slettet
       *[trash] flyttet til papirkurven
    }.
core-library-watched-removed =
    { $count ->
        [one] { $count } set afsnit
       *[other] { $count } sete afsnit
    } fra { $series } serier { $place ->
        [deleted] slettet
       *[trash] flyttet til papirkurven
    }.

## core/src/library/detail.rs

core-library-fix-one-file =
    Den ene fil her læses som { $read ->
        [episode] afsnit { $number }
       *[label] { $label }
    }. Den ligner { $what ->
        [film] filmen
       *[episode] seriens eneste afsnit
    }.
core-library-fix-specials =
    { $count ->
        [one] { $count } special
       *[other] { $count } specials
    } ligner { $episodes ->
        [one] det manglende afsnit
       *[other] de manglende afsnit
    } { $numbers }.
core-library-fix-numbered-on = Afsnit { $from } ligner denne posts { $to }, nummereret videre fra en tidligere sæson.
core-library-runs-span = { $first } til { $last }
core-library-runs-list = { $init } og { $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = et afsnitsnummer går fra 0 til 10000
core-library-fix-season-range = en sæson går op til 200
core-library-fix-extra-range = en ekstras indeks går op til 10000
core-library-file-elsewhere-not-fixed = fil { $id } hører til en anden serie, så intet blev rettet
core-library-fix-undone =
    { $count ->
        [one] Læser igen { $count } fil fra { $title } ud fra navnet.
       *[other] Læser igen { $count } filer fra { $title } ud fra navnet.
    }
core-library-fixed =
    { $count ->
        [one] Rettede, hvordan { $count } fil fra { $title } læses.
       *[other] Rettede, hvordan { $count } filer fra { $title } læses.
    }

## core/src/library/reads.rs

core-library-hid-one = Skjulte { $title }.
core-library-back-one = { $title } er tilbage.
core-library-hid-many = Skjulte { $count } serier.
core-library-back-many = { $count } serier er tilbage.
core-library-episode-count-set = { $title } har nu { $count } afsnit.
core-library-episode-count-cleared = Ryddede antallet af afsnit for { $title }.

## core/src/library/schedule.rs

core-library-schedule-cleared = Ryddede datonoterne for { $title }.
core-library-schedule-saved = Gemte { $count } datonoter for { $title }.

## core/src/library/scan.rs

core-library-source-not-absolute = en kildesti skal være absolut
core-library-source-exists = allerede en kilde
core-library-source-nested = ligger inde i en eksisterende kilde
core-library-source-added = Føjede mappen { $folder } til dit bibliotek.
core-library-source-removed = Fjernede mappen { $folder } fra dit bibliotek.
core-library-source-removed-with-shows =
    { $count ->
        [one] Fjernede mappen { $folder } og dens { $count } serie fra dit bibliotek.
       *[other] Fjernede mappen { $folder } og dens { $count } serier fra dit bibliotek.
    }
core-library-series-forgotten = Fjernede { $name } fra dit bibliotek.
core-library-source-back = Mappen { $folder } er tilbage.
core-library-source-unavailable = Mappen { $folder } er ikke tilgængelig. Tjek, at dens drev er tilsluttet.
core-library-source-unreadable = Kunne ikke læse mappen { $folder }. Tjek, at dens drev er tilsluttet.
core-library-source-looks-empty = Mappen { $folder } ser tom ud, så dine serier blev ladt, som de var.
core-library-source-reconcile-failed = Kunne ikke opdatere serierne i { $folder }.
core-library-scan-done =
    Tjekkede dine mapper: { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } ny serie
           *[other] { $added } nye serier
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } serie ændret
           *[other] { $changed } serier ændret
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }{ $removed } mangler
    }.

## core/src/library/report.rs

core-library-agent-no-answer = intet svar inden for { $span }
core-library-agent-stopped-code = den stoppede med kode { $code }
core-library-agent-stopped = den stoppede
core-library-agent-answered-nothing = den svarede ingenting
core-library-span-minutes =
    { $count ->
        [one] { $count } minut
       *[other] { $count } minutter
    }
core-library-span-seconds =
    { $count ->
        [one] { $count } sekund
       *[other] { $count } sekunder
    }
core-library-report-no-fix = ingen af disse filer har en rettelse at rapportere
core-library-agent-no-triage = { $agent } skrev ingen triage: { $why }
core-transfer-progress-sources = kilder
core-transfer-progress-series = serier
core-transfer-progress-history = historik
core-transfer-progress-accounts = konti
core-transfer-progress-preferences = præferencer
core-transfer-progress-done = færdig

## core/src/downloads/anirss.rs

core-downloads-anirss-no-answer = anirss svarede ikke: { $message }
core-downloads-anirss-answered = anirss svarede { $status }: { $text }
core-downloads-no-free-port = ingen ledig port: { $error }
core-downloads-anirss-exited = anirss lukkede ({ $status })
core-downloads-anirss-exited-said = anirss lukkede ({ $status }): { $said }
core-downloads-anirss-slow-start = anirss startede ikke inden for { $seconds } sekunder
core-downloads-anirss-slow-start-said = anirss startede ikke inden for { $seconds } sekunder: { $said }
core-downloads-anirss-install-mac = Installer anirss { $version } eller nyere med: { $command }
core-downloads-anirss-install-linux = Installer anirss { $version } eller nyere: anirss-pakken fra AUR, eller { $command }
core-downloads-anirss-too-old = anirss { $version } er for gammel til AniBeam, som kræver { $minimum } eller nyere. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } kan ikke holde hvert torrentprograms login for sig, hvilket AniBeam skal bruge for at skifte program; anirss { $minimum } og nyere kan. { $update }.
core-downloads-anirss-update-mac = Opdater den med: { $command }
core-downloads-anirss-update-linux = Opdater anirss-pakken, eller kør: { $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = torrentfilen kom ikke: { $message }
core-downloads-torrent-file-answered = torrentfilen svarede { $status }

## core/src/downloads/feed.rs

core-downloads-feed-answered = feedet svarede { $status }
core-downloads-not-a-feed = ikke et feed: { $error }

## core/src/downloads/jackett.rs

core-downloads-jackett-missing = Jackett er ikke installeret: { $program } blev ikke fundet. Installer det, eller skift kommandoen i Indstillinger.
core-downloads-endpoint-start-failed = { $name } kunne ikke starte: { $why }
core-downloads-endpoint-start-timeout = { $name } startede, men svarede ikke inden for { $seconds } sekunder.
core-downloads-command-exited = kommandoen sluttede med { $status }

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirss stoppede
core-downloads-anirss-missing = { $install }. Eller peg Indstillinger på en server, der kører.
core-downloads-anirss-command-missing = { $command } blev ikke fundet. { $install }. Eller peg Indstillinger på en server, der kører.
core-downloads-anirss-start-failed = { $command } kunne ikke starte: { $error }
core-downloads-anirss-too-old-said = { $tooOld } Den sagde: { $message }
core-downloads-no-torrent-client = Ingen qBittorrent svarer på { $address }, og AniBeam har ingen egen at køre. Installer qBittorrent og slå dens Web UI til, eller installer qbittorrent-nox, som AniBeam så kører for dig
core-downloads-own-client-gone = AniBeams qBittorrent er væk: qbittorrent-nox er ikke installeret. Geninstaller AniBeam, eller peg Indstillinger på din egen qBittorrent
core-downloads-no-port = ingen port i { $address }
core-downloads-own-client-settings = kan ikke skrive qBittorrents indstillinger: { $error }
core-downloads-own-client-start-failed = { $command } kunne ikke starte: { $error }
core-downloads-own-client-slow-start = AniBeams qBittorrent startede ikke
core-downloads-own-client-slow-start-said = AniBeams qBittorrent startede ikke: { $said }
core-downloads-not-ready = Downloads er ikke klar. Tjek Downloads i Indstillinger.
core-downloads-ready = Downloads er klar.
core-downloads-helper-sources =
    { $count ->
        [one] Downloads søger nu med AniBeam selv, som ikke læser den { $count } kilde i søgehjælperens egen konfiguration. Tilføj den under Indstillinger, Downloads, eller vælg søgehjælperen dér.
       *[other] Downloads søger nu med AniBeam selv, som ikke læser de { $count } kilder i søgehjælperens egen konfiguration. Tilføj dem under Indstillinger, Downloads, eller vælg søgehjælperen dér.
    }
core-downloads-client-connected = Downloadprogrammet er forbundet.
core-downloads-client-not-connected = Downloadprogrammet er ikke forbundet. Tjek dets adresse i Indstillinger.
core-downloads-not-on-demand = { $name } er ikke et endpoint, der startes efter behov
core-downloads-source-running = Downloadkilden kører.
core-downloads-source-not-running = { $name } kører ikke. Slå den til under kilderne og bekræft for at starte den.
core-downloads-empty-query = skriv noget at søge efter
core-downloads-first-endpoint = det første endpoint
core-downloads-search-starting = Starter søgningen
core-downloads-search-starting-endpoint = Starter { $endpoint }
core-downloads-search-searching-endpoint = Søger i { $endpoint }
core-downloads-search-reading = Læser resultaterne
core-downloads-subscribed = Abonnerer nu på { $name }.
core-downloads-client-no-rule = qBittorrent har ingen regel ved navn { $name }
core-downloads-trying-search = Prøver "{ $search }"
core-downloads-nothing-selected = intet valgt at downloade
core-downloads-added =
    { $count ->
        [one] Føjede { $count } download til dine downloads.
       *[other] Føjede { $count } downloads til dine downloads.
    }
core-downloads-movie-link = et filmlink skal være et magnet- eller http(s)-link
core-downloads-added-movie = Føjede en film til dine downloads.
core-downloads-unsubscribed = Afmeldte abonnementet på { $name }.
core-downloads-library-folder = { $folder } er biblioteksmappen for { $name } med afsnit, som dette abonnement ikke downloadede: fjern abonnementet uden dets filer, eller fjern { $name } fra dens side
core-downloads-no-torrents-named = ingen torrents angivet
core-downloads-paused =
    { $count ->
        [one] Satte { $count } download på pause.
       *[other] Satte { $count } downloads på pause.
    }
core-downloads-resumed =
    { $count ->
        [one] Genoptog { $count } download.
       *[other] Genoptog { $count } downloads.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] Slettede { $count } download og filerne.
       *[other] Slettede { $count } downloads og filerne.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] Fjernede { $count } download fra din liste.
       *[other] Fjernede { $count } downloads fra din liste.
    }
core-downloads-nothing-to-remove = intet at fjerne
core-downloads-removed-show = Fjernede { $title } fra dine downloads.
core-downloads-removed-the-show = Fjernede serien fra dine downloads.
core-downloads-deleted-still-listed = Slettede filer står stadig i downloadprogrammet.
core-downloads-feed-url = et feed er en http(s)-adresse
core-downloads-rule-needs-name = en regel skal have et navn
core-downloads-no-rule = ingen regel ved navn { $name }
core-downloads-feed-no-search = feedet har ingen søgning at forkorte
core-downloads-no-repair = { $name } behøver ingen reparation.
core-downloads-repaired = Reparerede abonnementet på { $name }.
core-downloads-repair-failed = ingen kortere søgning finder noget; selv "{ $search }" kommer tomt tilbage
core-downloads-no-saved-client = intet gemt torrentprogram { $id }
core-downloads-client-switched = Downloads bruger nu et andet downloadprogram.
core-downloads-host-not-running = kører ikke på værten
core-downloads-host-late = svarede ikke i tide

## core/src/downloads/native.rs

core-downloads-native-empty-query = tom søgning
core-downloads-native-no-results = ingen resultater for { $query }
core-downloads-native-feed-taken = et feed ved navn { $name } læser allerede en anden adresse
core-downloads-native-no-items = ingen elementer angivet
core-downloads-native-bad-link = links skal være magnet:- eller http(s)-adresser, fik { $link }
core-downloads-native-no-links = ingen links angivet
core-downloads-native-no-names = ingen abonnementsnavne angivet
core-downloads-native-files-elsewhere = torrentprogrammet har sine filer på en anden maskine, så de kan kun fjernes sammen med dets torrents
core-downloads-native-unknown-feeds = ukendte feeds: { $names }
core-downloads-native-outside-folders = nægter at fjerne filer uden for downloadmapperne: { $paths }
core-downloads-refuse-home-folder = { $path } er hjemmemappen eller indeholder den, så intet blev fjernet
core-downloads-refuse-library-folder = { $path } er en biblioteksmappe eller indeholder en, så intet blev fjernet

## core/src/downloads/paths.rs

core-downloads-no-default-folder = qBittorrent har ingen standardmappe at gemme i; vælg en i qBittorrent
core-downloads-name-empty = navnet må ikke være tomt
core-downloads-name-slash = navnet må ikke indeholde / eller \
core-downloads-name-not-folder = { $name } er ikke et mappenavn

## core/src/downloads/prune.rs

core-downloads-pruned = Afmeldte { $title }, den er færdig med at blive sendt.
core-downloads-prune-failed = Kunne ikke afmelde { $title }. Tjek Downloads i Indstillinger.

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path } er ikke en almindelig absolut sti, så intet blev fjernet
core-downloads-refuse-root = { $path } er filsystemets rod, så intet blev fjernet
core-downloads-refuse-outside-library = { $path } ligger ikke i en biblioteksmappe, så intet blev fjernet
core-downloads-refuse-save-folder = { $path } er mappen, torrenten blev gemt i, ikke det den downloadede, så intet blev fjernet

## core/src/downloads/watch.rs

core-downloads-episode-missed = Afsnit { $episode } af { $title } er ikke dukket op tre dage efter, det blev sendt. Dit feed leder stadig efter det.

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = anirss-kommandoen må ikke være tom
core-downloads-settings-anirss-address = anirss-adressen skal starte med http:// eller https://
core-downloads-settings-client-address = { $client }-adressen skal starte med http:// eller https://
core-downloads-settings-bad-id = { $id } er ikke et program-id: brug bogstaver, tal, - og _
core-downloads-settings-same-id = to programmer har id'et { $id }
core-downloads-settings-saved-address = adressen på { $name } skal starte med http:// eller https://
core-downloads-settings-same-name = to programmer hedder { $name }; giv hvert sit eget navn

## core/src/downloads/qbt.rs

core-downloads-qbt-no-answer = qBittorrent svarede ikke på { $address }: { $message }
core-downloads-qbt-password-rejected = qBittorrent afviste adgangskoden; ret den i Indstillinger
core-downloads-qbt-banned = qBittorrent har blokeret denne adresse efter mislykkede login; vent, eller genstart qBittorrent
core-downloads-qbt-login-rejected = qBittorrent afviste brugernavnet eller adgangskoden
core-downloads-qbt-login-failed = qBittorrents login på { $address } svarede { $status }; prøver igen ved næste kald
core-downloads-qbt-not-web-ui = { $address } svarede { $status }, men ikke som qBittorrents Web UI. Tjek adressen i Indstillinger, og at Web UI er slået til i qBittorrent
core-downloads-qbt-not-added = qBittorrent tilføjede ikke torrents: { $said }
core-downloads-qbt-some-not-added = qBittorrent kunne ikke tilføje { $count } af de torrents: { $said }
core-downloads-qbt-answered = qBittorrent svarede { $status }: { $line }

## core/src/downloads/replace.rs

core-downloads-replace-bad-item = element { $item } er ikke en af de { $count } udgivelser
core-downloads-not-replaced = Afsnit { $episode } blev ikke erstattet.
core-downloads-replaced = Erstattede afsnit { $episode } med det nye download.

## core/src/downloads/rolling.rs

core-downloads-no-roll = intet løbende download { $id }
core-downloads-rolled = { $title }: nu på afsnit { $from } til { $to } af { $total }.
core-downloads-roll-ahead = hold mindst ét afsnit foran
core-downloads-roll-nothing = intet at hente

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = en JSON-søgekilde kommer fra et endpointrepo, hvis definition kortlægger dens felter
core-releases-endpoint-no-name = mangler `name`
core-releases-endpoint-bad-name = { $name }: brug kun bogstaver, tal, '.', '_', '-'
core-releases-endpoint-json = { $name }: { $why }
core-releases-endpoint-unknown-kind = { $name }: ukendt type { $kind } (gyldige: { $valid })
core-releases-endpoint-no-url = { $name }: mangler `url`
core-releases-endpoint-not-http = { $name }: adressen skal starte med http:// eller https://
core-releases-endpoint-needs-query = { $name }: en { $kind }-url skal indeholde pladsholderen {"{"}query{"}"}
core-releases-endpoint-no-query = { $name }: en { $kind }-url tilføjer selv søgningen, så den tager ingen {"{"}query{"}"}

## core/src/releases/fetch.rs

core-releases-indexer-error = indekseren svarede med fejl { $code }
core-releases-bad-feed = ugyldigt feed: { $error }
core-releases-feed-answered = feedet svarede { $status }

## core/src/releases/json_feed.rs

core-releases-not-feed-json = ikke feedets JSON: { $error }

## core/src/releases/json.rs

core-releases-not-json = ikke JSON: { $error }
core-releases-search-error-said = søgningen svarede med en fejl: { $message }
core-releases-search-error = søgningen svarede med en fejl
core-releases-no-results-field = svaret har ingen resultater ved { $path }
core-releases-results-not-list = svarets resultater er ikke en liste
core-releases-search-answered = søgningen svarede { $status }

## core/src/releases/native.rs

core-releases-no-endpoints = ingen endpoints endnu: tilføj et repo, eller et manuelt, under Endpoints i Mere
core-releases-no-source-named = ingen kilde ved navn { $name }
core-releases-empty-query = skriv noget at søge efter
core-releases-cannot-follow = { $name } kan ikke følges: den svarer på søgninger, men har intet feed at tjekke for nye afsnit
core-releases-endpoint-gone = { $name } er ikke tilgængelig mere: fjernet, slået fra eller mangler sin nøgle

## core/src/releases/quirks.rs

core-releases-quirks-variant = one_variant { $word } skal være ét kort ord
core-releases-quirks-pattern-count = show_feed tager 1 til { $most } sidemønstre
core-releases-quirks-pattern-long = et show_feed-sidemønster er for langt
core-releases-quirks-pattern-bad = show_feed-sidemønster: { $error }
core-releases-quirks-pattern-slug = et show_feed-sidemønster skal have en gruppe ved navn slug
core-releases-quirks-url-https = show_feed-url skal være https
core-releases-quirks-url-host = show_feed-url skal ligge på endpointets egen vært
core-releases-quirks-url-slug = show_feed-url tager {"{"}slug{"}"} én gang og ingen anden pladsholder

## core/src/releases/refinement.rs

core-releases-refine-no-tokens = op 'tokens' skal have en ikke-tom `tokens`-liste

## core/src/releases/grid.rs

core-releases-group-batch = { $group }s pakke
core-releases-episodes-phrase =
    { $count ->
        [one] Afsnit { $list }
       *[other] Afsnit { $list }
    }
core-releases-episode-range = { $from } til { $to }

## core/src/subscriptions.rs

core-subscriptions-platform = anirss på denne platform
core-subscriptions-unreadable = dens svar kunne ikke læses ({ $message })
core-subscriptions-stopped = den stoppede uden at sige hvorfor
core-subscriptions-stopped-said = den stoppede: { $said }

## Names that stay the same in every language

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = et navn er 1 til { $max } tegn uden kontroltegn
core-prefs-room-relay = relæadressen skal starte med https:// eller http://
core-room-service = Se sammen
core-room-not-found = Intet rum har den kode. Det er måske lukket.
core-room-full = Det rum er fuldt.
core-room-proto = Det relæ kører en anden version af rum. Opdater { -app-name } for at gå ind i det.
core-room-unreachable = Kunne ikke nå relæet: { $message }
core-room-refused = Relæet sagde nej: { $message }
core-room-not-in = Du er ikke i et rum.
core-room-no-name = vælg det navn, de andre i rummet ser dig som
core-room-bad-code = en rumkode er 6 bogstaver og tal
core-room-created = Startede et se sammen-rum. Koden er { $code }.
core-room-joined = Gik ind i se sammen-rummet { $code }.
core-room-left = Forlod se sammen-rummet.
core-room-member-joined = { $name } kom ind i rummet.
core-room-member-left = { $name } forlod rummet.
core-room-closed = Se sammen-rummet lukkede.
core-room-lost = Mistede forbindelsen til se sammen-rummet. Prøver igen.
core-room-back = Tilbage i se sammen-rummet.
