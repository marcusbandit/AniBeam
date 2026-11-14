# nl translation of en/core.ftl. An ID missing here is shown in English.
# Wat de core tegen een persoon zegt: problemen, fouten, meldingen en de woorden die elke shell deelt.

## Namen die in elke taal hetzelfde blijven. Berichten gebruiken ze; geen wordt los getoond.

## De tailnet-host

tailnet-person-no-name = Geen naam in Tailscale

## Aantallen

count-episodes =
    { $count ->
        [one] { $count } aflevering
       *[other] { $count } afleveringen
    }
count-shows =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    }

## Acties die elk scherm gebruikt

action-cancel = Annuleren
action-close = Sluiten
action-try-again = Opnieuw
action-remove = Weghalen
action-delete = Wissen
action-open = Openen
action-save = Opslaan
action-apply = Toepassen
action-clear = Leegmaken
action-refresh = Vernieuwen
action-play = Afspelen
action-connect = Koppelen
action-start = Starten
action-back = Terug
action-see-all = Alles zien
action-open-series = Serie openen
action-open-settings = Instellingen openen

## De lijsten die een tracker bijhoudt

status-watching = Kijken
status-completed = Voltooid
status-paused = Gepauzeerd
status-on-hold = In de wacht
status-dropped = Gestopt
status-planning = Gepland
status-rewatching = Herkijken

## Afleveringen

episode-number = Aflevering { $number }

## De woorden van AniList voor een serie: formaat, uitzendstatus, hoe twee series samenhangen

anilist-format-tv = TV
anilist-format-tv-short = Korte TV
anilist-format-film = Film
anilist-format-special = Special
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Muziek
anilist-status-finished = Afgelopen
anilist-status-airing = Loopt
anilist-status-not-yet-aired = Nog niet uitgezonden
anilist-status-cancelled = Geannuleerd
anilist-status-hiatus = Gepauzeerd
anilist-relation-adaptation = Adaptatie
anilist-relation-prequel = Prequel
anilist-relation-sequel = Vervolg
anilist-relation-parent = Hoofdverhaal
anilist-relation-side-story = Zijverhaal
anilist-relation-character = Personage
anilist-relation-summary = Samenvatting
anilist-relation-alternative = Alternatief
anilist-relation-spin-off = Spin-off
anilist-relation-other = Overig
anilist-relation-source = Bron
anilist-relation-compilation = Compilatie
anilist-relation-contains = Bevat

## Het activiteitenlog: een probleem waarvan de plek geen eigen woorden had

event-problem-library = Er ging iets mis met je bibliotheek. Ontwikkelaarsopties hebben de details.
event-problem-metadata = Er ging iets mis met de info van een serie. Ontwikkelaarsopties hebben de details.
event-problem-trackers = Er ging iets mis met je lijsten. Ontwikkelaarsopties hebben de details.
event-problem-franchise = Er ging iets mis met verwante series. Ontwikkelaarsopties hebben de details.
event-problem-playback = Er ging iets mis met afspelen. Ontwikkelaarsopties hebben de details.
event-problem-store = Er ging iets mis met het opslaan van je gegevens. Ontwikkelaarsopties hebben de details.
event-problem-system = Er ging iets mis met AniBeam. Ontwikkelaarsopties hebben de details.
event-problem-downloads = Er ging iets mis met downloads. Ontwikkelaarsopties hebben de details.

## Fouten (contract::CoreError)

core-error-not-found-source = Map { $id } niet gevonden.
core-error-not-found-series = Serie { $id } niet gevonden.
core-error-not-found-file = Bestand { $id } niet gevonden.
core-error-not-found-session = Afspeelsessie { $id } niet gevonden.
core-error-not-found-job = Taak { $id } niet gevonden.
core-error-invalid = Ongeldige { $field }: { $message }.
core-error-invalid-ended = Ongeldige { $field }: { $message }
core-error-unavailable = Deze map is niet beschikbaar: { $path }
core-error-not-connected = { $tracker } is niet gekoppeld.
core-error-refused-hidden = Dat kan niet: de serie is verborgen.
core-error-refused-no-match = Dat kan niet: de serie is nog niet gekoppeld; koppel hem bij Serie-info.
core-error-refused-not-newer = Dat kan niet: je lijst is al voorbij deze aflevering.
core-error-refused-extra = Dat kan niet: dit is een extra.
core-error-refused-unmatched = Dat kan niet: het bestand is niet aan een aflevering gekoppeld.
core-error-refused-on-disk = Dat kan niet: het bestand staat alleen op schijf.
core-error-provider = { $provider } reageerde niet: { $message }
core-error-io = Kon een bestand niet lezen of schrijven: { $message }
core-error-storage = Kon je bibliotheek niet opslaan: { $message }
core-error-keyring = Kon de wachtwoordopslag van het systeem niet bereiken: { $message }
core-error-unsupported = { $what } wordt nog niet ondersteund.
core-error-version = Deze export komt van een nieuwere AniBeam (formaat { $found }). Werk AniBeam bij om hem te importeren.
core-error-internal = AniBeam liep tegen een interne fout aan: { $message }
core-error-service = { $service }: { $message }

## core/src/repos/mod.rs

core-repos-paste-address = plak het adres van een repo
core-repos-must-be-https = het adres moet https zijn
core-repos-unreachable = kon de repo niet bereiken: { $message }
core-repos-not-found = niets op dat adres (een privé GitHub-repo antwoordt ook 404)
core-repos-http-status = de repo antwoordde HTTP { $status }
core-repos-unreadable = kon { $path } niet lezen: { $error }
core-repos-no-such-endpoint = geen geïnstalleerd endpoint met dat id
core-repos-key-changed = de repo is nu ondertekend met een andere sleutel dan degene die je vertrouwde. Haal hem weg en voeg hem opnieuw toe om de nieuwe sleutel te accepteren.
core-repos-stale = de repo gaf een oudere index (serienummer { $serial } na { $kept }); de nieuwere blijft
core-repos-key-not-shown = de sleutel van de repo is niet degene die je zag. Voeg hem opnieuw toe en vergelijk de vingerafdruk.
core-repos-repo-added = Een lijst met downloadbronnen toegevoegd.
core-repos-repo-removed = Een lijst met downloadbronnen weggehaald.
core-repos-no-field = { $name } heeft geen veld { $key }
core-repos-needs-field = { $name } heeft zijn { $label } nodig
core-repos-host-only = { $name }: de { $label } is alleen een host of host:poort, zoals 192.0.2.10:9696
core-repos-not-added = die repo is niet toegevoegd
core-repos-not-offered = { $repo } biedt { $id } niet aan
core-repos-already-installed = { $name } is al geïnstalleerd
core-repos-taken-by-you = er is al een endpoint { $id } geïnstalleerd (door jou toegevoegd); haal dat eerst weg
core-repos-taken-by-repo = er is al een endpoint { $id } geïnstalleerd (uit een andere repo); haal dat eerst weg
core-repos-source-added = Een downloadbron toegevoegd.
core-repos-source-updated = Een downloadbron bijgewerkt.
core-repos-source-removed = Een downloadbron weggehaald.
core-repos-source-changed = Een downloadbron gewijzigd.
core-repos-give-name = geef het een naam
core-repos-put-query = zet {"{"}query{"}"} in het adres waar de zoekterm komt
core-repos-name-taken = er is al een endpoint { $id } geïnstalleerd
core-repos-only-by-hand = alleen een handmatig toegevoegde bron kan worden bewerkt
core-repos-keep-key = laat {"{"}{ $key }{"}"} in het adres staan en wijzig de waarde met { $label } wijzigen, of haal de site weg en voeg hem zonder opnieuw toe
core-repos-note-kind = anirss kan het soort { $kind } niet doorzoeken, dus de desktop slaat het over; de telefoon doorzoekt het wel
core-repos-note-by-hand = anirss heeft een eigen { $id }, met de hand geschreven; de desktop doorzoekt die
core-repos-note-theirs = Deze anirss draai je zelf, en de desktop wijzigt nooit de config ervan: voeg { $id } toe aan de config van die anirss zodat de desktop het doorzoekt
core-repos-note-sandbox = Deze sessie schrijft nooit in de config van anirss, dus de desktop doorzoekt { $id } hier niet
core-repos-note-old = anirss { $version } kan geen endpoints van { -app-name } aannemen; werk het bij naar { $min } of nieuwer zodat de desktop dit doorzoekt

## core/src/repos/index.rs

core-repos-key-not-ed25519 = de sleutel van de index is geen ed25519-sleutel
core-repos-key-not-base64 = de sleutel van de index is geen 32 bytes base64
core-repos-key-invalid = de sleutel van de index is geen geldige ed25519-sleutel
core-repos-too-large = het bestand is te groot voor een repo
core-repos-not-a-repo = dit is geen endpoint-repo van { -app-name }
core-repos-format-version = de repo heeft formaatversie { $found }; deze { -app-name } leest versie { $reads }. Werk { -app-name } bij.
core-repos-payload-not-base64 = de inhoud van de repo is geen base64
core-repos-signature-malformed = de handtekening van de repo is misvormd
core-repos-index-unreadable = de index van de repo is niet leesbaar: { $error }
core-repos-signature-mismatch = de handtekening van de repo past niet bij de inhoud
core-repos-check-scheme = { $id }: het adres moet beginnen met https://
core-repos-check-scheme-http = { $id }: het adres moet beginnen met http:// of https://
core-repos-check-auth-key = { $id }: ongeldige auth-sleutel { $key }
core-repos-check-host-torznab = { $id }: alleen een torznab-adres mag zijn host als {"{"}{ $key }{"}"} noemen
core-repos-check-host-placeholder = { $id }: een placeholder in de host

## core/src/playback/tracks.rs

core-playback-track-number = Spoor { $number }
core-playback-track-file = { $label } (bestand)
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = Engels
core-playback-lang-japanese = Japans
core-playback-lang-spanish = Spaans
core-playback-lang-portuguese = Portugees
core-playback-lang-french = Frans
core-playback-lang-german = Duits
core-playback-lang-italian = Italiaans
core-playback-lang-russian = Russisch
core-playback-lang-chinese = Chinees
core-playback-lang-korean = Koreaans
core-playback-lang-arabic = Arabisch
core-playback-lang-hindi = Hindi
core-playback-lang-indonesian = Indonesisch
core-playback-lang-malay = Maleis
core-playback-lang-thai = Thai
core-playback-lang-vietnamese = Vietnamees
core-playback-lang-polish = Pools
core-playback-lang-turkish = Turks
core-playback-lang-dutch = Nederlands
core-playback-lang-swedish = Zweeds
core-playback-lang-norwegian = Noors
core-playback-lang-danish = Deens
core-playback-lang-finnish = Fins
core-playback-lang-hebrew = Hebreeuws
core-playback-lang-ukrainian = Oekraïens
core-playback-lang-czech = Tsjechisch
core-playback-lang-hungarian = Hongaars
core-playback-lang-greek = Grieks
core-playback-lang-romanian = Roemeens
core-playback-lang-filipino = Filipijns
core-playback-region-brazil = Brazilië
core-playback-region-portugal = Portugal
core-playback-region-latin-america = Latijns-Amerika
core-playback-region-spain = Spanje
core-playback-region-simplified = vereenvoudigd
core-playback-region-traditional = traditioneel

## contract/src/enums.rs

core-jobs-scan = Je mappen controleren
    .stopped = Mappen controleren stopte door een probleem.
core-jobs-auto-match = Series koppelen
    .stopped = Series koppelen stopte door een probleem.
core-jobs-search = Een serie zoeken
    .stopped = Een serie zoeken stopte door een probleem.
core-jobs-resolve-link = Een link lezen
    .stopped = Een link lezen stopte door een probleem.
core-jobs-apply-match = Een serie koppelen
    .stopped = Een serie koppelen stopte door een probleem.
core-jobs-refresh = Een serie bijwerken
    .stopped = Een serie bijwerken stopte door een probleem.
core-jobs-refresh-all = Serie-info bijwerken
    .stopped = Serie-info bijwerken stopte door een probleem.
core-jobs-refresh-airing = Kijken wat er wordt uitgezonden
    .stopped = Kijken wat er wordt uitgezonden stopte door een probleem.
core-jobs-refresh-imdb = Beoordelingen bijwerken
    .stopped = Beoordelingen bijwerken stopte door een probleem.
core-jobs-clear-images = Afbeeldingen wissen
    .stopped = Afbeeldingen wissen stopte door een probleem.
core-jobs-fill-images = Afbeeldingen ophalen
    .stopped = Afbeeldingen ophalen stopte door een probleem.
core-jobs-connect-tracker = Je lijst koppelen
    .stopped = Je lijst koppelen stopte door een probleem.
core-jobs-mark = Een aflevering als gekeken markeren
    .stopped = Een aflevering als gekeken markeren stopte door een probleem.
core-jobs-set-progress = Je voortgang instellen
    .stopped = Je voortgang instellen stopte door een probleem.
core-jobs-score = Je score opslaan
    .stopped = Je score opslaan stopte door een probleem.
core-jobs-refresh-progress = Je lijstvoortgang lezen
    .stopped = Je lijstvoortgang lezen stopte door een probleem.
core-jobs-refresh-watching = Je Kijken-lijst lezen
    .stopped = Je Kijken-lijst lezen stopte door een probleem.
core-jobs-refresh-airing-now = Dit seizoen controleren
    .stopped = Dit seizoen controleren stopte door een probleem.
core-jobs-refresh-recommended = Series voor je zoeken
    .stopped = Series voor je zoeken stopte door een probleem.
core-jobs-read-show-page = Een serie lezen
    .stopped = Een serie lezen stopte door een probleem.
core-jobs-on-hold = Stille series naar In de wacht zetten
    .stopped = Stille series naar In de wacht zetten stopte door een probleem.
core-jobs-set-status = Een lijststatus wijzigen
    .stopped = Een lijststatus wijzigen stopte door een probleem.
core-jobs-move-lists = Je lijst naar je tracker verhuizen
    .stopped = Je lijst naar je tracker verhuizen stopte door een probleem.
core-jobs-notes = Je notities opslaan
    .stopped = Je notities opslaan stopte door een probleem.
core-jobs-crawl = Verwante series zoeken
    .stopped = Verwante series zoeken stopte door een probleem.
core-jobs-skip-windows = Opening en ending zoeken
    .stopped = Opening en ending zoeken stopte door een probleem.
core-jobs-export = Je bibliotheek in een bestand opslaan
    .stopped = Je bibliotheek in een bestand opslaan stopte door een probleem.
core-jobs-import = Je bibliotheek uit een bestand laden
    .stopped = Je bibliotheek uit een bestand laden stopte door een probleem.
core-jobs-subscriptions = Je abonnementen controleren
    .stopped = Je abonnementen controleren stopte door een probleem.
core-jobs-connect-downloads = De downloadapp koppelen
    .stopped = De downloadapp koppelen stopte door een probleem.
core-jobs-search-releases = Afleveringen zoeken
    .stopped = Afleveringen zoeken stopte door een probleem.
core-jobs-refine-releases = Afleveringen verfijnen
    .stopped = Afleveringen verfijnen stopte door een probleem.
core-jobs-subscribe = Abonneren op een serie
    .stopped = Abonneren op een serie stopte door een probleem.
core-jobs-download = Downloads toevoegen
    .stopped = Downloads toevoegen stopte door een probleem.
core-jobs-remove-subscription = Een abonnement weghalen
    .stopped = Een abonnement weghalen stopte door een probleem.
core-jobs-control-torrents = Je downloads wijzigen
    .stopped = Je downloads wijzigen stopte door een probleem.
core-jobs-test-torrent-client = De downloadapp testen
    .stopped = De downloadapp testen stopte door een probleem.
core-jobs-remove-show = Een serie weghalen
    .stopped = Een serie weghalen stopte door een probleem.
core-jobs-repair-subscription = Een abonnement herstellen
    .stopped = Een abonnement herstellen stopte door een probleem.
core-jobs-check-endpoints = Je downloadbronnen controleren
    .stopped = Je downloadbronnen controleren stopte door een probleem.
core-jobs-start-endpoint = Een downloadbron starten
    .stopped = Een downloadbron starten stopte door een probleem.
core-jobs-schedule-agent = Je codeeragent om een schema vragen
    .stopped = Je codeeragent om een schema vragen stopte door een probleem.

## core/src/airing_now.rs

core-airing-now-now = Nu uitgezonden
core-airing-now-winter = Winter { $year }
core-airing-now-spring = Lente { $year }
core-airing-now-summer = Zomer { $year }
core-airing-now-fall = Herfst { $year }
core-airing-now-planned = { $title } op je Gepland-lijst op { $site } gezet.
core-airing-now-planned-both = { $title } op je Gepland-lijst op { $site } en { $other } gezet.
core-airing-now-already-on =
    { $title } staat al op je { $site }-lijst als { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }.
core-airing-now-plan-failed = Kon { $title } niet op je Gepland-lijst zetten. Probeer het later opnieuw.

## core/src/aired.rs

core-aired-episode-out = Aflevering { $number } van { $title } is uit.

## core/src/cast/mod.rs

core-cast-connecting = Casten naar { $device }.
core-cast-ended = Casten naar { $device } is klaar.
core-cast-stopped = Casten naar { $device } gestopt.
core-cast-failed = Casten naar { $device } is mislukt. Controleer of het op hetzelfde netwerk zit.
core-cast-not-built = Casten zit niet in deze build
core-cast-serve-failed = kon de aflevering niet aanbieden: { $error }
core-cast-device-gone = Dat apparaat is niet meer op het netwerk
core-cast-not-a-playlist = geen afspeellijstadres
core-cast-nothing = Er wordt niets gecast
core-cast-still-starting = Het apparaat start nog
core-cast-no-route = dit apparaat heeft geen route naar de tv
core-cast-no-control = de tv gaf geen besturingsadres
core-cast-not-handed = de andere computer gaf de aflevering niet door: { $error }
core-cast-empty-playlist = de andere computer stuurde een lege afspeellijst
core-cast-refused-both = { $error }; het bestand zoals het is werd ook geweigerd ({ $first })
core-cast-chromecast-stopped = de Chromecast is gestopt
core-cast-hevc = { $error }. Deze aflevering is HEVC, wat alleen nieuwere Chromecasts (met Google TV, of Ultra) decoderen

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = kon de Chromecast niet bereiken: { $error }
core-cast-chromecast-broke = de verbinding met de Chromecast is verbroken: { $error }
core-cast-chromecast-no-player = de Chromecast startte zijn mediaspeler niet
core-cast-chromecast-refused-player = de Chromecast wilde zijn mediaspeler niet starten ({ $reason })
core-cast-no-reason = geen reden
core-cast-chromecast-cannot-play = de Chromecast kon dit niet afspelen ({ $kind })
core-cast-chromecast-error = de Chromecast stopte met een fout bij het afspelen

## core/src/cast/dlna.rs

core-cast-tv-no-answer = de tv antwoordde niet: { $error }
core-cast-tv-refused = de tv weigerde { $action }
core-cast-tv-refused-code = de tv weigerde { $action } ({ $code })

## core/src/cast/media.rs

core-cast-no-video = Dit bestand heeft geen video.
core-cast-hi10p = Deze aflevering is 10-bit H.264 (Hi10P), wat geen tv of Chromecast decodeert. Casten vraagt om transcoderen, en dat is er nog niet.
core-cast-video-codec = De video van deze aflevering ({ $codec }) kan niet worden gecast.
core-cast-audio-codec = De audio van deze aflevering ({ $codec }) kan niet worden gecast.

## core/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Downloaden van een aflevering van { $folder } gestart.
       *[other] Downloaden van { $count } afleveringen van { $folder } gestart.
    }
core-phone-subscribed = Geabonneerd op { $name }.
core-phone-rule-stopped = { $name } wordt niet gecontroleerd op nieuwe afleveringen: de bron is weg of heeft zijn sleutel opnieuw nodig.
core-phone-new-episode = Een nieuwe aflevering van { $name } wordt gedownload.
core-phone-name-taken = { $name }: een andere bron heeft die naam
core-phone-nothing-to-download = niets te downloaden
core-phone-over-cap = hiermee gaat { -app-name } over zijn opslaglimiet ({ $used } van { $cap } GB gebruikt of onderweg); wis afleveringen of verhoog de limiet
core-phone-empty-search = de zoekopdracht is leeg
core-phone-no-results = geen resultaten voor { $query }
core-phone-own-engine = een eigen torrent-engine

## core/src/phone/check.rs

core-phone-check-connected = verbonden
core-phone-check-refused = geweigerd: { $error }
core-phone-check-dropped = geen antwoord (gedropt)
core-phone-check-finding = { $label }: adres zoeken
core-phone-check-failed = mislukt: { $error }
core-phone-check-no-answer = geen antwoord
core-phone-check-private = { $label }: privé zoeken
core-phone-check-skipped-address = overgeslagen, een adres
core-phone-check-skipped-home = overgeslagen, naam in thuisnetwerk
core-phone-check-no-record = geen openbaar adres, overgelaten aan de DNS van dit netwerk
core-phone-check-no-resolver = geen resolver antwoordde
core-phone-check-reaching = { $label }: { $ip } bereiken
core-phone-check-http = HTTP { $status }, { $bytes } bytes
core-phone-check-setup = instellen

## core/src/playback/session.rs

core-playback-not-saved = Je voortgang in { $folder } is niet opgeslagen. Koppel eerst de serie.
core-playback-still-downloading = aflevering { $number } wordt nog gedownload, { $done }% klaar

## core/src/playback/skip.rs

core-playback-bad-duration = een duur is een positief aantal seconden

## core/src/agent.rs

core-agent-no-answer = de agent gaf geen antwoord
core-agent-searching = Op het web zoeken naar { $query }
core-agent-searching-schedule = Op het web zoeken naar het schema
core-agent-reading = { $url } lezen
core-agent-reading-page = Een pagina lezen
core-agent-using = { $tool } gebruiken
core-agent-none = een codeeragent op deze computer
core-agent-ended = { $name } stopte zonder antwoord
core-agent-too-long = { $name } deed er te lang over

## core/src/core.rs

core-core-guest-trackers = de trackeraccounts op die computer zijn van de eigenaar, dus dit apparaat markeert daar niets
core-core-guest-not-interested = Niet interessant op die computer is de lijst van de eigenaar, dus dit apparaat wijzigt daar niets
core-core-guest-recommended = de aanbevelingen op die computer komen uit de lijst van de eigenaar
core-core-nothing-picked = niets gekozen

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider } is geen bron voor koppelen
core-prefs-provider-count = { $provider } staat er { $count } keer in; elke bron staat er één keer in
core-prefs-outside = { $value } ligt buiten { $low } tot { $high }
core-prefs-empty = leeg

## core/src/recommended/mod.rs

core-recommended-ranking = Series voor je rangschikken

## core/src/recommended/sync.rs

core-recommended-finding-people = Mensen met jouw smaak zoeken
core-recommended-reading-lists = Hun lijsten lezen
core-recommended-reading-yours = Jouw series lezen
core-recommended-reading-suggestions = Series om aan te raden lezen

## core/src/recommended/page.rs

core-recommended-no-anime = { -anilist } heeft geen anime { $id }

## core/src/host/mod.rs

core-host-off = Uit
core-host-starting = Starten
core-host-serving = Bereikbaar op { $addresses }
core-host-no-tailscale = Tailscale is niet verbonden
core-host-cannot-listen = Kon niet luisteren
core-host-port-taken = Poort { $port } is bezet
core-host-cannot-listen-on = Kon niet luisteren op { $address }: { $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text } is geen adres met poort

## core/src/host/keys.rs

core-host-not-a-key = { $key } is geen apparaatsleutel

## core/src/net/limiter.rs

core-net-timed-out = time-out na { $ms }ms
core-net-rate-limited = { $provider } remt verzoeken af

## core/src/net/mod.rs

core-net-waiting = { $host }: zojuist een time-out; { -app-name } wacht { $seconds } s voordat het weer vraagt

## core/src/net/anilist.rs

core-net-no-data = geen gegevens

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } heeft geen download voor deze computer
core-update-cannot-make = Kon { $path } niet maken: { $error }
core-update-cannot-mark = Kon de nieuwe AppImage niet uitvoerbaar maken: { $error }
core-update-cannot-replace = Kon { $path } niet vervangen: { $error }
core-update-not-here = Deze kopie wordt hier niet bijgewerkt
core-update-cannot-start = Kon de download niet starten: { $error }
core-update-did-not-start = De download is niet gestart: { $error }
core-update-cannot-write = Kon { $path } niet schrijven: { $error }
core-update-stopped = De download is gestopt: { $error }
core-update-cancelled = Geannuleerd
core-update-ended-early = De download stopte te vroeg ({ $got } van { $size } bytes)
core-update-checksum = De download past niet bij de checksum van GitHub
core-update-cannot-move = Kon de download niet op zijn plek zetten: { $error }
core-update-cannot-open-dmg = Kon de dmg niet openen: { $error }
core-update-cannot-read-dmg = Kon de dmg niet lezen: { $error }
core-update-no-app = De dmg bevat geen app
core-update-cannot-copy = Kon de nieuwe app niet naast deze kopiëren: { $error }

## core/src/calendar.rs

core-calendar-season-episode = { $title } - Seizoen { $season } Aflevering { $number }
core-calendar-episode = { $title } - Aflevering { $number }

## core/src/paths.rs

core-paths-no-home = geen thuismap

## core/src/first_run.rs

core-first-run-not-absolute = de videomap moet een absoluut pad zijn

## core/src/presence/mod.rs

core-presence-extra = Extra

## core/src/trackers/accounts.rs

core-trackers-no-client-id = Geen client-ID ingesteld voor { $tracker }.
core-trackers-mal-reconnect = Koppel MyAnimeList opnieuw in Instellingen.
core-trackers-mal-wants-secret = MyAnimeList wil het client secret van deze app. Voeg het toe onder Trackers in Instellingen, of registreer de app met App Type "other".
core-trackers-token-reply-unreadable = onleesbaar tokenantwoord: { $error }
core-trackers-token-reply-no-token = het tokenantwoord bevatte geen access token

## core/src/trackers/mod.rs

core-trackers-no-token = geen access token bewaard, koppel opnieuw in Instellingen

## core/src/trackers/cache.rs

core-trackers-no-user-id = het account heeft geen gebruikers-ID

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = geen series gekozen om te verhuizen
core-trackers-moved-up =
    { $count ->
        [one] { $count } serie van deze computer naar { $site } verhuisd.
       *[other] { $count } series van deze computer naar { $site } verhuisd.
    }
core-trackers-moved-up-some-refused =
    { $moved ->
        [one] { $moved } serie
       *[other] { $moved } series
    } naar { $site } verhuisd; { $site } nam { $failed ->
        [one] { $failed } serie
       *[other] { $failed } series
    } niet aan.
core-trackers-move-refused = { $site } nam { $show } niet aan: { $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = Autorisatie verlopen, probeer het opnieuw.
core-trackers-oauth-state-mismatch = state komt niet overeen, mogelijk CSRF, afgebroken.
core-trackers-oauth-no-code = geen code in callback
core-trackers-oauth-no-token = geen access_token in callback
core-trackers-oauth-waiting = Wacht tot je inlogt bij { $site } in je browser.
core-trackers-oauth-connected = { $site } gekoppeld als { $username }.
core-trackers-oauth-no-name = het profiel kwam terug zonder naam
core-trackers-oauth-could-not-start = Inloggen kon niet starten
core-trackers-oauth-port-failed = poort { $port } op deze computer kon niet worden geopend: { $error }
core-trackers-oauth-port-busy = poort { $port } op deze computer wordt door een ander programma gebruikt. Wacht even en probeer het opnieuw, of sluit dat programma als de poort bezet blijft.
core-trackers-oauth-provider-error = { $site } gaf een fout: { $error }
core-trackers-page-connected = { $site } is gekoppeld aan { $app }
core-trackers-page-close-tab = Je kunt dit tabblad sluiten.
core-trackers-page-failed-title = { $app }, inloggen mislukt
core-trackers-page-failed = Het inloggen is niet afgerond
core-trackers-page-go-back = Ga terug naar { $app } en probeer het opnieuw.
core-trackers-page-finishing-title = { $app }, bijna klaar
core-trackers-page-finishing = Bijna klaar
core-trackers-page-finishing-body = { $site } heeft geantwoord en AniBeam rondt het inloggen af. Ga terug naar { $app }, dat meldt wanneer het klaar is.
core-trackers-page-cancelled-title = { $app }, inloggen geannuleerd
core-trackers-page-cancelled = Dit inloggen is geannuleerd
core-trackers-page-cancelled-body = Je kunt dit tabblad sluiten. Begin opnieuw vanuit { $app } als je dat wilde.

## core/src/trackers/on_hold.rs

core-trackers-on-hold-moving = Naar In de wacht zetten op { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] { $count } serie op { $site } naar In de wacht gezet.
       *[other] { $count } series op { $site } naar In de wacht gezet.
    }
core-trackers-on-hold-moved-two =
    { $count ->
        [one] { $count } serie op { $site } en { $second } op { $secondSite } naar In de wacht gezet.
       *[other] { $count } series op { $site } en { $second } op { $secondSite } naar In de wacht gezet.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } serie kon niet naar In de wacht.
       *[other] { $failed } series konden niet naar In de wacht.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] { $count } serie op { $site } naar In de wacht gezet.
       *[other] { $count } series op { $site } naar In de wacht gezet.
    } { $failed ->
        [one] { $failed } serie kon niet naar In de wacht.
       *[other] { $failed } series konden niet naar In de wacht.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] { $count } serie op { $site } en { $second } op { $secondSite } naar In de wacht gezet.
       *[other] { $count } series op { $site } en { $second } op { $secondSite } naar In de wacht gezet.
    } { $failed ->
        [one] { $failed } serie kon niet naar In de wacht.
       *[other] { $failed } series konden niet naar In de wacht.
    }
core-trackers-series-fallback = serie { $id }
core-trackers-back-on-watching = { $name } staat weer op Kijken op { $site }.
core-trackers-back-on-watching-two = { $name } staat weer op Kijken op { $site } en { $secondSite }.
core-trackers-back-on-watching-failed = { $name } kon niet terug naar Kijken. Probeer het later opnieuw.

## core/src/trackers/writes.rs

core-trackers-rate-limited = { $tracker } remt verzoeken af, probeer het over een minuut opnieuw.
core-trackers-auth-expired = { $tracker }-login verlopen, koppel opnieuw in Instellingen.
core-trackers-entry-not-found = { $tracker }-item niet gevonden.
core-trackers-server-error = { $tracker } serverfout ({ $status }), probeer het later opnieuw.
core-trackers-error-see-log = { $tracker } fout, zie het activiteitenlog.
core-trackers-episode-not-whole = { $episode } is geen hele aflevering boven nul
core-trackers-score-out-of-range = { $score } ligt buiten 0 tot 10
core-trackers-rewatch-not-settable = herkijken is geen status om met de hand in te stellen
core-trackers-notes-too-long = notities zijn beperkt tot { $max } tekens
core-trackers-nothing-picked = niets gekozen
core-trackers-notes-not-on-list = Nog niet op je { $tracker }-lijst, zet het eerst op een lijst.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } en { -mal }
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
       *[repeating] Herkijken
    }
core-trackers-marked = Aflevering { $episode } van { $show } gemarkeerd op { core-trackers-sites }.
core-trackers-progress-set = Je voortgang in { $show } op { core-trackers-sites } op { $progress } gezet.
core-trackers-scored = { $show } een { $score } gegeven op { core-trackers-sites }.
core-trackers-score-cleared = Je score voor { $show } op { core-trackers-sites } gewist.
core-trackers-status-set = { $show } op { core-trackers-sites } op { core-trackers-status } gezet.
core-trackers-notes-saved = Je notities bij { $show } op { core-trackers-sites } opgeslagen.
core-trackers-notes-cleared = Je notities bij { $show } op { core-trackers-sites } gewist.
core-trackers-update-failed = Kon { $site } niet bijwerken. Probeer het later opnieuw.
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
           *[other] { $done } series
        }
    }
core-trackers-written-done =
    { $action ->
        [clear] Je score voor { core-trackers-written-who } op { core-trackers-sites } gewist.
        [status] { core-trackers-written-who } op { core-trackers-sites } op { core-trackers-status } gezet.
       *[score] { core-trackers-written-who } een { $score } gegeven op { core-trackers-sites }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] { $failed } serie kon niet worden bijgewerkt: { $reason }.
           *[other] { $failed } series konden niet worden bijgewerkt: { $reason }.
        }
       *[no] { $failed ->
            [one] { $failed } serie kon niet worden bijgewerkt.
           *[other] { $failed } series konden niet worden bijgewerkt.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] { $failed } niet bijgewerkt: { $reason }.
       *[no] { $failed } niet bijgewerkt.
    }
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } serie heeft geen trackerkoppeling.
       *[other] {" "}{ $noMatch } series hebben geen trackerkoppeling.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } serie is verborgen, dus de sync staat uit.
       *[other] {" "}{ $hidden } series zijn verborgen, dus de sync staat uit.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } serie staat alleen op een tracker die niet gekoppeld is.
       *[other] {" "}{ $notConnected } series staan alleen op een tracker die niet gekoppeld is.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } serie wordt op deze computer bewaard, zonder tracker om naar te sturen.
       *[other] {" "}{ $kept } series worden op deze computer bewaard, zonder tracker om naar te sturen.
    }

## core/src/metadata/automatch.rs

core-metadata-matched-to = { $show } gekoppeld aan { $title }.
core-metadata-auto-matched =
    { $count ->
        [one] { $count } serie gekoppeld.
       *[other] { $count } series gekoppeld.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } serie moet nog worden gekoppeld.
       *[other] { $count } series moeten nog worden gekoppeld.
    }
core-metadata-auto-matched-some =
    { $matched ->
        [one] { $matched } serie
       *[other] { $matched } series
    } gekoppeld; { $unmatched } { $unmatched ->
        [one] moet
       *[other] moeten
    } nog worden gekoppeld.
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
        [yes] { $hitFolder } gekoppeld aan { $hitTitle }.
       *[no] { $matched ->
            [one] { $matched } serie gekoppeld.
           *[other] { $matched } series gekoppeld.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] Geen koppeling gevonden voor { $missName }.
       *[no] { $unmatched ->
            [one] Geen koppeling gevonden voor { $unmatched } serie.
           *[other] Geen koppeling gevonden voor { $unmatched } series.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] { $failed } serie kon niet worden gezocht; probeer het later opnieuw.
       *[other] { $failed } series konden niet worden gezocht; probeer het later opnieuw.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } serie was al gekoppeld.
       *[other] { $had } series waren al gekoppeld.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } serie wordt al gekoppeld.
       *[other] { $busy } series worden al gekoppeld.
    }

## core/src/metadata/apply.rs

core-metadata-link-unreadable = Kon die link niet lezen. Plak de URL van een pagina op AniList, MyAnimeList, TVmaze of IMDb.
core-metadata-no-mal-entry = AniList heeft geen item voor dat MyAnimeList-id.
core-metadata-no-imdb-show = TVmaze heeft geen serie voor die IMDb-titel.
core-metadata-search-on = zoeken op { $provider }
core-metadata-query-too-short = Typ minstens twee tekens om te zoeken.
core-metadata-not-a-link = geen link
core-metadata-refresh-unsupported = vernieuwen van een koppeling die alleen MAL of TMDB is
core-metadata-matched = { $show } gekoppeld.
core-metadata-updated = De info van { $show } bijgewerkt.
core-metadata-update-failed = Kon de info van { $show } niet bijwerken. Probeer het later opnieuw.
core-metadata-progress-refresh = vernieuwen
core-metadata-progress-backfill = aanvullen
core-metadata-updated-many =
    { $count ->
        [one] De info van { $count } serie bijgewerkt.
       *[other] De info van { $count } series bijgewerkt.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] Kon de info van { $count } serie niet bijwerken. Probeer het later opnieuw.
       *[other] Kon de info van { $count } series niet bijwerken. Probeer het later opnieuw.
    }
core-metadata-updated-some =
    { $count ->
        [one] De info van { $count } serie bijgewerkt; { $failed } kon niet worden bijgewerkt.
       *[other] De info van { $count } series bijgewerkt; { $failed } konden niet worden bijgewerkt.
    }

## core/src/metadata/fetch.rs

core-metadata-no-such-show = TVmaze heeft geen serie { $id }
core-metadata-no-such-media = AniList heeft geen media { $id }

## core/src/metadata/imdb.rs

core-metadata-progress-imdb-links = IMDb-links van Wikidata
core-metadata-progress-imdb-ratings = IMDb-beoordelingen

## core/src/franchise/layout.rs

core-franchise-relation-character = Gedeelde personages

## core/src/images/mod.rs

core-images-progress-fetching = afbeeldingen ophalen

## core/src/transfer/export.rs

core-transfer-exported =
    { $count ->
        [one] { $count } serie opgeslagen in { $folder }.
       *[other] { $count } series opgeslagen in { $folder }.
    }

## core/src/transfer/format.rs

core-transfer-not-json = geen JSON: { $error }
core-transfer-wrong-format = geen { $format }-document

## core/src/transfer/import.rs

core-transfer-imported = Je bibliotheek is uit het bestand geladen.

## core/src/library/delete.rs

core-library-no-such-file = geen bestand { $id } in de bibliotheek
core-library-file-elsewhere-not-removed = bestand { $id } hoort bij een andere serie, dus er is niets weggehaald
core-library-file-outside = { $path } is geen bestand in een bibliotheekmap, dus er is niets weggehaald
core-library-episodes-removed =
    { $count ->
        [one] { $count } aflevering
       *[other] { $count } afleveringen
    } van { $title } { $place ->
        [deleted] gewist
       *[trash] naar de prullenbak verplaatst
    }.
core-library-finished-removed =
    { $count ->
        [one] { $count } afgeronde serie
       *[other] { $count } afgeronde series
    } { $place ->
        [deleted] gewist
       *[trash] naar de prullenbak verplaatst
    }.
core-library-watched-removed =
    { $count ->
        [one] { $count } gekeken aflevering
       *[other] { $count } gekeken afleveringen
    } van { $series } series { $place ->
        [deleted] gewist
       *[trash] naar de prullenbak verplaatst
    }.

## core/src/library/detail.rs

core-library-fix-one-file =
    Het enige bestand hier wordt gelezen als { $read ->
        [episode] aflevering { $number }
       *[label] { $label }
    }. Het lijkt { $what ->
        [film] de film
       *[episode] de enige aflevering
    }.
core-library-fix-specials =
    { $count ->
        [one] { $count } special lijkt
       *[other] { $count } specials lijken
    } op de ontbrekende { $episodes ->
        [one] aflevering
       *[other] afleveringen
    } { $numbers }.
core-library-fix-numbered-on = Afleveringen { $from } lijken de afleveringen { $to } van dit item, doorgenummerd vanaf een eerder seizoen.
core-library-runs-span = { $first } t/m { $last }
core-library-runs-list = { $init } en { $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = een afleveringsnummer loopt van 0 tot 10000
core-library-fix-season-range = een seizoen loopt tot 200
core-library-fix-extra-range = de index van een extra loopt tot 10000
core-library-file-elsewhere-not-fixed = bestand { $id } hoort bij een andere serie, dus er is niets hersteld
core-library-fix-undone =
    { $count ->
        [one] { $count } bestand van { $title } wordt weer op naam gelezen.
       *[other] { $count } bestanden van { $title } worden weer op naam gelezen.
    }
core-library-fixed =
    { $count ->
        [one] Hersteld hoe { $count } bestand van { $title } wordt gelezen.
       *[other] Hersteld hoe { $count } bestanden van { $title } worden gelezen.
    }

## core/src/library/reads.rs

core-library-hid-one = { $title } verborgen.
core-library-back-one = { $title } is terug.
core-library-hid-many = { $count } series verborgen.
core-library-back-many = { $count } series zijn terug.
core-library-episode-count-set = { $title } heeft nu { $count } afleveringen.
core-library-episode-count-cleared = Het aantal afleveringen van { $title } gewist.

## core/src/library/schedule.rs

core-library-schedule-cleared = De schemanotities van { $title } gewist.
core-library-schedule-saved = { $count } schemanotities voor { $title } opgeslagen.

## core/src/library/scan.rs

core-library-source-not-absolute = een bronpad moet absoluut zijn
core-library-source-exists = is al een bron
core-library-source-nested = ligt binnen een bestaande bron
core-library-source-added = De map { $folder } aan je bibliotheek toegevoegd.
core-library-source-removed = De map { $folder } uit je bibliotheek weggehaald.
core-library-source-removed-with-shows =
    { $count ->
        [one] De map { $folder } en zijn { $count } serie uit je bibliotheek weggehaald.
       *[other] De map { $folder } en zijn { $count } series uit je bibliotheek weggehaald.
    }
core-library-series-forgotten = { $name } uit je bibliotheek weggehaald.
core-library-source-back = De map { $folder } is terug.
core-library-source-unavailable = De map { $folder } is niet beschikbaar. Controleer of de schijf aangesloten is.
core-library-source-unreadable = Kon de map { $folder } niet lezen. Controleer of de schijf aangesloten is.
core-library-source-looks-empty = De map { $folder } lijkt leeg, dus je series zijn gelaten zoals ze waren.
core-library-source-reconcile-failed = Kon de series in { $folder } niet bijwerken.
core-library-scan-done =
    Mappen gecontroleerd: { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } nieuwe serie
           *[other] { $added } nieuwe series
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } serie gewijzigd
           *[other] { $changed } series gewijzigd
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }{ $removed } ontbreken
    }.

## core/src/library/report.rs

core-library-agent-no-answer = geen antwoord binnen { $span }
core-library-agent-stopped-code = gestopt met code { $code }
core-library-agent-stopped = gestopt
core-library-agent-answered-nothing = gaf geen antwoord
core-library-span-minutes =
    { $count ->
        [one] { $count } minuut
       *[other] { $count } minuten
    }
core-library-span-seconds =
    { $count ->
        [one] { $count } seconde
       *[other] { $count } seconden
    }
core-library-report-no-fix = geen van deze bestanden heeft een herstel om te melden
core-library-agent-no-triage = { $agent } schreef geen triage: { $why }
core-transfer-progress-sources = bronnen
core-transfer-progress-series = series
core-transfer-progress-history = geschiedenis
core-transfer-progress-accounts = accounts
core-transfer-progress-preferences = voorkeuren
core-transfer-progress-done = klaar

## core/src/downloads/anirss.rs

core-downloads-anirss-no-answer = anirss antwoordde niet: { $message }
core-downloads-anirss-answered = anirss antwoordde { $status }: { $text }
core-downloads-no-free-port = geen vrije poort: { $error }
core-downloads-anirss-exited = anirss is gestopt ({ $status })
core-downloads-anirss-exited-said = anirss is gestopt ({ $status }): { $said }
core-downloads-anirss-slow-start = anirss startte niet binnen { $seconds } seconden
core-downloads-anirss-slow-start-said = anirss startte niet binnen { $seconds } seconden: { $said }
core-downloads-anirss-install-mac = Installeer anirss { $version } of nieuwer met: { $command }
core-downloads-anirss-install-linux = Installeer anirss { $version } of nieuwer: het anirss-pakket in de AUR, of { $command }
core-downloads-anirss-too-old = anirss { $version } is te oud voor AniBeam, dat { $minimum } of nieuwer nodig heeft. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } kan de login van elke torrentclient niet apart houden, en dat heeft AniBeam nodig om van client te wisselen; anirss { $minimum } en nieuwer kunnen dat wel. { $update }.
core-downloads-anirss-update-mac = Werk het bij met: { $command }
core-downloads-anirss-update-linux = Werk het anirss-pakket bij, of voer uit: { $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = het torrentbestand kwam niet: { $message }
core-downloads-torrent-file-answered = het torrentbestand antwoordde { $status }

## core/src/downloads/feed.rs

core-downloads-feed-answered = de feed antwoordde { $status }
core-downloads-not-a-feed = geen feed: { $error }

## core/src/downloads/jackett.rs

core-downloads-jackett-missing = Jackett is niet geïnstalleerd: { $program } is niet gevonden. Installeer het, of wijzig het commando in Instellingen.
core-downloads-endpoint-start-failed = { $name } kon niet starten: { $why }
core-downloads-endpoint-start-timeout = { $name } is gestart maar antwoordde niet binnen { $seconds } seconden.
core-downloads-command-exited = het commando stopte met { $status }

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirss is gestopt
core-downloads-anirss-missing = { $install }. Of laat Instellingen naar een draaiende server wijzen.
core-downloads-anirss-command-missing = { $command } is niet gevonden. { $install }. Of laat Instellingen naar een draaiende server wijzen.
core-downloads-anirss-start-failed = { $command } kon niet starten: { $error }
core-downloads-anirss-too-old-said = { $tooOld } Het zei: { $message }
core-downloads-no-torrent-client = Geen qBittorrent antwoordt op { $address }, en AniBeam heeft er zelf geen om te draaien. Installeer qBittorrent en zet de Web UI aan, of installeer qbittorrent-nox, dat AniBeam dan voor je draait
core-downloads-own-client-gone = De qBittorrent van AniBeam is weg: qbittorrent-nox is niet geïnstalleerd. Installeer AniBeam opnieuw, of laat Instellingen naar je eigen qBittorrent wijzen
core-downloads-no-port = geen poort in { $address }
core-downloads-own-client-settings = kan de instellingen van qBittorrent niet schrijven: { $error }
core-downloads-own-client-start-failed = { $command } kon niet starten: { $error }
core-downloads-own-client-slow-start = De qBittorrent van AniBeam is niet gestart
core-downloads-own-client-slow-start-said = De qBittorrent van AniBeam is niet gestart: { $said }
core-downloads-not-ready = Downloads zijn niet klaar. Controleer Downloads in Instellingen.
core-downloads-ready = Downloads zijn klaar.
core-downloads-helper-sources =
    { $count ->
        [one] Downloads zoeken nu met AniBeam zelf, dat de { $count } bron in de eigen config van de zoekhulp niet leest. Voeg ze toe in Instellingen, Downloads, of kies daar de zoekhulp.
       *[other] Downloads zoeken nu met AniBeam zelf, dat de { $count } bronnen in de eigen config van de zoekhulp niet leest. Voeg ze toe in Instellingen, Downloads, of kies daar de zoekhulp.
    }
core-downloads-client-connected = De downloadapp is gekoppeld.
core-downloads-client-not-connected = De downloadapp is niet gekoppeld. Controleer het adres in Instellingen.
core-downloads-not-on-demand = { $name } is geen endpoint op aanvraag
core-downloads-source-running = De downloadbron draait.
core-downloads-source-not-running = { $name } draait niet. Zet het aan bij de bronnen en bevestig om het te starten.
core-downloads-empty-query = typ iets om te zoeken
core-downloads-first-endpoint = het eerste endpoint
core-downloads-search-starting = Zoeken starten
core-downloads-search-starting-endpoint = { $endpoint } starten
core-downloads-search-searching-endpoint = { $endpoint } doorzoeken
core-downloads-search-reading = Resultaten lezen
core-downloads-subscribed = Geabonneerd op { $name }.
core-downloads-client-no-rule = qBittorrent heeft geen regel met de naam { $name }
core-downloads-trying-search = "{ $search }" proberen
core-downloads-nothing-selected = niets gekozen om te downloaden
core-downloads-added =
    { $count ->
        [one] { $count } download aan je downloads toegevoegd.
       *[other] { $count } downloads aan je downloads toegevoegd.
    }
core-downloads-movie-link = een filmlink moet een magnet of een http(s)-URL zijn
core-downloads-added-movie = Een film aan je downloads toegevoegd.
core-downloads-library-folder = { $folder } is de bibliotheekmap van { $name }, met afleveringen die dit abonnement niet downloadde: haal het abonnement weg zonder de bestanden, of haal { $name } weg vanaf de pagina ervan
core-downloads-no-torrents-named = geen torrents genoemd
core-downloads-paused =
    { $count ->
        [one] { $count } download gepauzeerd.
       *[other] { $count } downloads gepauzeerd.
    }
core-downloads-resumed =
    { $count ->
        [one] { $count } download hervat.
       *[other] { $count } downloads hervat.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] { $count } download en de bestanden gewist.
       *[other] { $count } downloads en de bestanden gewist.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] { $count } download van je lijst weggehaald.
       *[other] { $count } downloads van je lijst weggehaald.
    }
core-downloads-nothing-to-remove = niets om weg te halen
core-downloads-removed-show = { $title } uit je downloads weggehaald.
core-downloads-removed-the-show = De serie uit je downloads weggehaald.
core-downloads-deleted-still-listed = Gewiste bestanden staan nog in de downloadapp.
core-downloads-feed-url = een feed is een http(s)-URL
core-downloads-rule-needs-name = een regel heeft een naam
core-downloads-no-rule = geen regel met de naam { $name }
core-downloads-feed-no-search = de feed heeft geen zoekopdracht om in te korten
core-downloads-no-repair = { $name } hoeft niet hersteld te worden.
core-downloads-repaired = Het abonnement op { $name } hersteld.
core-downloads-repair-failed = geen kortere zoekopdracht vindt iets; zelfs "{ $search }" geeft niets
core-downloads-no-saved-client = geen bewaarde torrentclient { $id }
core-downloads-client-switched = Downloads gebruiken nu een andere downloadapp.
core-downloads-host-not-running = draait niet op de host
core-downloads-host-late = antwoordde niet op tijd

## core/src/downloads/native.rs

core-downloads-native-empty-query = lege zoekopdracht
core-downloads-native-no-results = geen resultaten voor { $query }
core-downloads-native-feed-taken = een feed met de naam { $name } leest al een ander adres
core-downloads-native-no-items = geen items gegeven
core-downloads-native-bad-link = links moeten magnet: of http(s)-URL's zijn, kreeg { $link }
core-downloads-native-no-links = geen links gegeven
core-downloads-native-no-names = geen abonnementsnamen gegeven
core-downloads-native-files-elsewhere = de torrentclient bewaart zijn bestanden op een andere machine, dus die kunnen alleen samen met de torrents weg
core-downloads-native-unknown-feeds = onbekende feed(s): { $names }
core-downloads-native-outside-folders = bestanden buiten de downloadmappen worden niet weggehaald: { $paths }
core-downloads-refuse-home-folder = { $path } is de thuismap of bevat die, dus er is niets weggehaald
core-downloads-refuse-library-folder = { $path } is een bibliotheekmap of bevat er een, dus er is niets weggehaald

## core/src/downloads/paths.rs

core-downloads-no-default-folder = qBittorrent noemt geen standaard opslagmap; stel er een in qBittorrent in
core-downloads-name-empty = de naam mag niet leeg zijn
core-downloads-name-slash = de naam mag geen / of \ bevatten
core-downloads-name-not-folder = { $name } is geen mapnaam

## core/src/downloads/prune.rs

core-downloads-pruned = Abonnement op { $title } opgezegd, de uitzending is afgelopen.
core-downloads-prune-failed = Kon het abonnement op { $title } niet opzeggen. Controleer Downloads in Instellingen.

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path } is geen gewoon absoluut pad, dus er is niets weggehaald
core-downloads-refuse-root = { $path } is de root van het bestandssysteem, dus er is niets weggehaald
core-downloads-refuse-outside-library = { $path } ligt niet in een bibliotheekmap, dus er is niets weggehaald
core-downloads-refuse-save-folder = { $path } is de map waarin de torrent is opgeslagen, niet wat hij downloadde, dus er is niets weggehaald

## core/src/downloads/watch.rs

core-downloads-episode-missed = Aflevering { $episode } van { $title } is drie dagen na uitzending nog niet opgedoken. Je feed blijft ernaar zoeken.

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = het anirss-commando mag niet leeg zijn
core-downloads-settings-anirss-address = het anirss-adres moet beginnen met http:// of https://
core-downloads-settings-client-address = het { $client }-adres moet beginnen met http:// of https://
core-downloads-settings-bad-id = { $id } is geen client-id: gebruik letters, cijfers, - en _
core-downloads-settings-same-id = twee clients hebben het id { $id }
core-downloads-settings-saved-address = het adres van { $name } moet beginnen met http:// of https://
core-downloads-settings-same-name = twee clients heten { $name }; geef elk een eigen naam

## core/src/downloads/qbt.rs

core-downloads-qbt-no-answer = qBittorrent antwoordde niet op { $address }: { $message }
core-downloads-qbt-password-rejected = qBittorrent weigerde het wachtwoord; pas het aan in Instellingen
core-downloads-qbt-banned = qBittorrent heeft dit adres geblokkeerd na mislukte logins; wacht, of herstart qBittorrent
core-downloads-qbt-login-rejected = qBittorrent weigerde de gebruikersnaam of het wachtwoord
core-downloads-qbt-login-failed = de login van qBittorrent op { $address } antwoordde { $status }; volgende keer opnieuw
core-downloads-qbt-not-web-ui = { $address } antwoordde { $status }, maar niet als de Web UI van qBittorrent. Controleer het adres in Instellingen, en of de Web UI aan staat in qBittorrent
core-downloads-qbt-not-added = qBittorrent voegde de torrents niet toe: { $said }
core-downloads-qbt-some-not-added = qBittorrent kon { $count } van de torrents niet toevoegen: { $said }
core-downloads-qbt-answered = qBittorrent antwoordde { $status }: { $line }

## core/src/downloads/replace.rs

core-downloads-replace-bad-item = item { $item } is niet een van de { $count } releases
core-downloads-not-replaced = Aflevering { $episode } is niet vervangen.
core-downloads-replaced = Aflevering { $episode } vervangen door de nieuwe download.

## core/src/downloads/rolling.rs

core-downloads-no-roll = geen doorlopende download { $id }
core-downloads-rolled = { $title }: nu op afleveringen { $from } t/m { $to } van { $total }.
core-downloads-roll-ahead = houd minstens één aflevering voorsprong
core-downloads-roll-nothing = niets op te halen

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = een JSON-zoekbron komt uit een endpoint-repo waarvan de definitie de velden koppelt
core-releases-endpoint-no-name = `name` ontbreekt
core-releases-endpoint-bad-name = { $name }: gebruik alleen letters, cijfers, '.', '_', '-'
core-releases-endpoint-json = { $name }: { $why }
core-releases-endpoint-unknown-kind = { $name }: onbekend soort { $kind } (geldig: { $valid })
core-releases-endpoint-no-url = { $name }: `url` ontbreekt
core-releases-endpoint-not-http = { $name }: het adres moet beginnen met http:// of https://
core-releases-endpoint-needs-query = { $name }: een { $kind }-url moet een placeholder {"{"}query{"}"} bevatten
core-releases-endpoint-no-query = { $name }: een { $kind }-url voegt de zoekterm zelf toe, dus hij neemt geen {"{"}query{"}"}

## core/src/releases/fetch.rs

core-releases-indexer-error = de indexer antwoordde met fout { $code }
core-releases-bad-feed = ongeldige feed: { $error }
core-releases-feed-answered = de feed antwoordde { $status }

## core/src/releases/json_feed.rs

core-releases-not-feed-json = niet de JSON van de feed: { $error }

## core/src/releases/json.rs

core-releases-not-json = geen JSON: { $error }
core-releases-search-error-said = de zoekopdracht antwoordde met een fout: { $message }
core-releases-search-error = de zoekopdracht antwoordde met een fout
core-releases-no-results-field = het antwoord heeft geen resultaten op { $path }
core-releases-results-not-list = de resultaten in het antwoord zijn geen lijst
core-releases-search-answered = de zoekopdracht antwoordde { $status }

## core/src/releases/native.rs

core-releases-no-endpoints = nog geen endpoints: voeg een repo toe, of een met de hand, onder Endpoints in Meer
core-releases-no-source-named = geen bron met de naam { $name }
core-releases-empty-query = typ iets om te zoeken
core-releases-cannot-follow = { $name } kan niet worden gevolgd: het beantwoordt zoekopdrachten, maar heeft geen feed om op nieuwe afleveringen te controleren
core-releases-endpoint-gone = { $name } is niet meer beschikbaar: weggehaald, uitgezet of de sleutel ontbreekt

## core/src/releases/quirks.rs

core-releases-quirks-variant = one_variant { $word } moet één kort woord zijn
core-releases-quirks-pattern-count = show_feed neemt 1 tot { $most } paginapatronen
core-releases-quirks-pattern-long = een paginapatroon van show_feed is te lang
core-releases-quirks-pattern-bad = paginapatroon van show_feed: { $error }
core-releases-quirks-pattern-slug = een paginapatroon van show_feed heeft een groep met de naam slug nodig
core-releases-quirks-url-https = de url van show_feed moet https zijn
core-releases-quirks-url-host = de url van show_feed moet op de eigen host van het endpoint staan
core-releases-quirks-url-slug = de url van show_feed neemt {"{"}slug{"}"} één keer en geen andere placeholder

## core/src/releases/refinement.rs

core-releases-refine-no-tokens = op 'tokens' heeft een niet-lege lijst `tokens` nodig

## core/src/releases/grid.rs

core-releases-group-batch = Batch van { $group }
core-releases-episodes-phrase =
    { $count ->
        [one] Aflevering { $list }
       *[other] Afleveringen { $list }
    }
core-releases-episode-range = { $from } t/m { $to }

## Names that stay the same in every language

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = een naam is 1 tot { $max } tekens, zonder stuurtekens
core-prefs-room-relay = het relayadres moet beginnen met https:// of http://
core-room-service = Samen kijken
core-room-not-found = Geen kamer met die code. Misschien is hij gesloten.
core-room-full = Die kamer is vol.
core-room-proto = Die relay draait een andere versie van kamers. Werk { -app-name } bij om mee te doen.
core-room-unreachable = Kon de relay niet bereiken: { $message }
core-room-refused = De relay weigerde: { $message }
core-room-not-in = Je zit niet in een kamer.
core-room-no-name = kies de naam waarmee anderen in de kamer je zien
core-room-bad-code = een kamercode is 6 letters en cijfers
core-room-created = Een kamer om samen te kijken gestart. De code is { $code }.
core-room-joined = Lid geworden van kamer { $code } om samen te kijken.
core-room-left = De kamer om samen te kijken verlaten.
core-room-member-joined = { $name } kwam de kamer in.
core-room-member-left = { $name } verliet de kamer.
core-room-closed = De kamer om samen te kijken is gesloten.
core-room-lost = Verbinding met de kamer om samen te kijken verloren. Opnieuw proberen.
core-room-back = Weer in de kamer om samen te kijken.
core-room-media-asked = De host om de aflevering gevraagd.
core-room-media-cannot = De host kan deze aflevering nu niet delen.
core-room-media-sent = { $name } de torrent van de aflevering gestuurd.
core-room-media-sent-name = { $name } de naam van de release van de aflevering gestuurd.
core-room-media-nothing = De host heeft geen kopie van de aflevering om te delen.
core-room-media-torrent = De aflevering wordt gedownload via de torrent van de host.
core-room-media-searching = Je bronnen worden doorzocht op de release van de host.
core-room-media-not-found = Geen van je bronnen heeft de release van de host.
core-room-media-no-start = De download kon niet starten: { $message }
core-room-media-client = de torrentclient nam hem niet aan
core-room-media-lost = De torrentclient is de download van de aflevering kwijt.
core-room-media-not-in-library = De aflevering is gedownload, maar staat niet in je bibliotheek. Controleer of downloads in een bibliotheekmap terechtkomen.
core-room-media-here = De aflevering staat nu in je bibliotheek.
