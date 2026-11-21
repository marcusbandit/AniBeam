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
event-problem-system = Er ging iets mis met Anibeam. Ontwikkelaarsopties hebben de details.
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
# Why a list was not updated, read after "Not tracked:" or a tracker's name, on every shell's player
refusal-hidden = de serie is verborgen
refusal-no-match = de serie is nog niet gekoppeld; koppel hem bij Serie-info
refusal-not-newer = je lijst is al voorbij deze aflevering
refusal-extra = dit is een extra
refusal-unmatched = het bestand is niet aan een aflevering gekoppeld
refusal-on-disk = het bestand staat alleen op schijf
core-error-provider = { $provider } reageerde niet: { $message }
core-error-io = Kon een bestand niet lezen of schrijven: { $message }
core-error-storage = Kon je bibliotheek niet opslaan: { $message }
core-error-keyring = Kon de wachtwoordopslag van het systeem niet bereiken: { $message }
core-error-unsupported = { $what } wordt nog niet ondersteund.
core-error-version = Deze export komt van een nieuwere Anibeam (formaat { $found }). Werk Anibeam bij om hem te importeren.
core-error-internal = Anibeam liep tegen een interne fout aan: { $message }
core-error-service = { $service }: { $message }

## releases/src/repos/mod.rs

core-repos-paste-address = plak het adres van een bronlijst
core-repos-must-be-https = het adres moet https gebruiken
core-repos-unreachable = kon de bronlijst niet bereiken: { $message }
core-repos-not-found = niets gevonden op dat adres (een privé GitHub-repo geeft ook 404)
core-repos-http-status = de bronlijst gaf HTTP { $status }
core-repos-unreadable = kon { $path } niet lezen: { $error }
core-repos-no-such-endpoint = geen geïnstalleerde bron met dat ID
core-repos-key-changed = deze bronlijst is nu ondertekend met een andere sleutel dan degene die je vertrouwde. Verwijder hem en voeg hem opnieuw toe om de nieuwe sleutel te accepteren.
core-repos-stale = de bronlijst gaf een oudere index (serienummer { $serial } na { $kept }); de nieuwere blijft
core-repos-key-not-shown = de ondertekeningssleutel komt niet overeen met de sleutel die je zag. Voeg de bronlijst opnieuw toe en vergelijk de vingerafdruk.
core-repos-repo-added = Bronlijst toegevoegd.
core-repos-repo-removed = Bronlijst verwijderd.
core-repos-no-field = { $name } heeft geen veld { $key }
core-repos-needs-field = { $name } vereist zijn { $label }
core-repos-host-only = { $name }: { $label } moet alleen een host of host:poort zijn, zoals 192.0.2.10:9696
core-repos-not-added = die bronlijst is niet toegevoegd
core-repos-not-offered = { $repo } biedt { $id } niet aan
core-repos-already-installed = { $name } is al geïnstalleerd
core-repos-taken-by-you = er is al een bron { $id } geïnstalleerd (handmatig toegevoegd); verwijder die eerst
core-repos-taken-by-repo = er is al een bron { $id } geïnstalleerd (uit een andere bronlijst); verwijder die eerst
core-repos-source-added = Een downloadbron toegevoegd.
core-repos-source-updated = Een downloadbron bijgewerkt.
core-repos-source-removed = Een downloadbron weggehaald.
core-repos-source-changed = Een downloadbron bewerkt.
core-repos-give-name = vul een naam in
core-repos-put-query = zet {"{"}query{"}"} in het adres waar de zoekterm komt
core-repos-name-taken = er is al een bron { $id } geïnstalleerd
core-repos-only-by-hand = alleen handmatig toegevoegde bronnen kunnen worden bewerkt
core-repos-keep-key = laat {"{"}{ $key }{"}"} in het adres staan en wijzig de waarde met { $label } wijzigen, of verwijder de site en voeg hem zonder opnieuw toe
core-repos-note-kind = anirss kan geen { $kind }-bronnen doorzoeken, dus de desktop slaat het over; de telefoonapp kan het wel doorzoeken
core-repos-note-by-hand = anirss heeft een eigen, handmatig ingestelde { $id }; de desktop doorzoekt die
core-repos-note-theirs = Je beheert deze anirss zelf, dus { -app-name } wijzigt de config niet: voeg { $id } toe aan de config van anirss om het op de desktop te doorzoeken
core-repos-note-sandbox = In sandboxmodus wordt niet in de config van anirss geschreven, dus de desktop doorzoekt { $id } hier niet
core-repos-note-old = anirss { $version } ondersteunt geen bronnen van { -app-name }; werk bij naar { $min } of nieuwer om dit op de desktop te doorzoeken

## releases/src/repos/index.rs

core-repos-key-not-ed25519 = de indexsleutel is geen ed25519-sleutel
core-repos-key-not-base64 = de indexsleutel is geen 32 bytes base64
core-repos-key-invalid = de indexsleutel is geen geldige ed25519-sleutel
core-repos-too-large = het bestand is te groot voor een bronlijst
core-repos-not-a-repo = dit is geen geldige bronlijst van { -app-name }
core-repos-format-version = deze bronlijst gebruikt formaatversie { $found }, maar { -app-name } leest versie { $reads }. Werk { -app-name } bij.
core-repos-payload-not-base64 = de inhoud van de bronlijst is geen base64
core-repos-signature-malformed = de handtekening van de bronlijst is misvormd
core-repos-index-unreadable = kon de index van de bronlijst niet lezen: { $error }
core-repos-signature-mismatch = de handtekening van de bronlijst past niet bij de inhoud
core-repos-check-scheme = { $id }: het adres moet beginnen met https://
core-repos-check-scheme-http = { $id }: het adres moet beginnen met http:// of https://
core-repos-check-auth-key = { $id }: ongeldige auth-sleutel { $key }
core-repos-check-host-torznab = { $id }: alleen een torznab-adres mag {"{"}{ $key }{"}"} als host gebruiken
core-repos-check-host-placeholder = { $id }: placeholder niet toegestaan in de host

## playback/src/tracks.rs

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

## metadata/src/airing_now.rs

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

## metadata/src/aired.rs

core-aired-episode-out = Aflevering { $number } van { $title } is uit.

## playback/src/cast/mod.rs

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
core-cast-not-handed = de andere computer kon de aflevering niet sturen: { $error }
core-cast-empty-playlist = de andere computer stuurde een lege afspeellijst
core-cast-refused-both = { $error }; het originele bestand werd ook geweigerd ({ $first })
core-cast-chromecast-stopped = de Chromecast is gestopt
core-cast-hevc = { $error }. Deze aflevering gebruikt HEVC, wat alleen nieuwere Chromecasts (met Google TV of Ultra) ondersteunen

## playback/src/cast/castv2.rs

core-cast-chromecast-unreachable = kon de Chromecast niet bereiken: { $error }
core-cast-chromecast-broke = de verbinding met de Chromecast is weggevallen: { $error }
core-cast-chromecast-no-player = de Chromecast startte zijn mediaspeler niet
core-cast-chromecast-refused-player = de Chromecast kon zijn mediaspeler niet starten ({ $reason })
core-cast-no-reason = geen reden
core-cast-chromecast-cannot-play = de Chromecast kon dit bestand niet afspelen ({ $kind })
core-cast-chromecast-error = de Chromecast stopte met een fout bij het afspelen van dit bestand

## playback/src/cast/dlna.rs

core-cast-tv-no-answer = de tv reageerde niet: { $error }
core-cast-tv-refused = de tv weigerde { $action }
core-cast-tv-refused-code = de tv weigerde { $action } ({ $code })

## playback/src/cast/media.rs

core-cast-no-video = Dit bestand heeft geen video.
core-cast-hi10p = Deze aflevering gebruikt 10-bit H.264 (Hi10P), wat tv's en Chromecasts niet kunnen decoderen. Casten vereist transcoderen, en dat wordt nog niet ondersteund.
core-cast-video-codec = De video van deze aflevering ({ $codec }) kan niet worden gecast.
core-cast-audio-codec = De audio van deze aflevering ({ $codec }) kan niet worden gecast.

## downloads/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Downloaden van een aflevering van { $folder } gestart.
       *[other] Downloaden van { $count } afleveringen van { $folder } gestart.
    }
core-phone-download-partial = Niet elke release van { $folder } kon worden toegevoegd: { $error }
core-phone-subscribed = Geabonneerd op { $name }.
core-phone-rule-stopped = { $name } controleert niet op nieuwe afleveringen: de bron is verwijderd of heeft zijn sleutel opnieuw nodig.
core-phone-new-episode = Een nieuwe aflevering van { $name } wordt gedownload.
core-phone-name-taken = { $name }: een andere bron heeft die naam
core-phone-nothing-to-download = niets te downloaden
core-phone-over-cap = hiermee gaat { -app-name } over zijn opslaglimiet ({ $used } van { $cap } GB gebruikt of in de wachtrij); wis afleveringen of verhoog de limiet
core-phone-empty-search = de zoekopdracht is leeg
core-phone-no-results = geen resultaten voor { $query }
core-phone-own-engine = een eigen torrent-engine

## downloads/src/phone/check.rs

core-phone-check-connected = verbonden
core-phone-check-refused = geweigerd: { $error }
core-phone-check-dropped = geen reactie (gedropt)
core-phone-check-finding = { $label }: adres opzoeken
core-phone-check-failed = mislukt: { $error }
core-phone-check-no-answer = geen reactie
core-phone-check-private = { $label }: privé opzoeken
core-phone-check-skipped-address = overgeslagen, IP-adres
core-phone-check-skipped-home = overgeslagen, lokale netwerknaam
core-phone-check-no-record = geen openbaar record, terugval op de DNS van het netwerk
core-phone-check-no-resolver = geen DNS-resolver reageerde
core-phone-check-reaching = { $label }: verbinden met { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } bytes
core-phone-check-setup = instellen

## playback/src/session.rs

core-playback-not-saved = Je voortgang in { $folder } is niet opgeslagen. Koppel eerst de serie.
core-playback-still-downloading = aflevering { $number } wordt nog gedownload, { $done }% klaar

## playback/src/skip.rs

core-playback-bad-duration = de duur moet een positief aantal seconden zijn

## core/src/agent.rs

core-agent-no-answer = de agent gaf geen antwoord
core-agent-searching = Op het web zoeken naar { $query }
core-agent-searching-schedule = Op het web zoeken naar het schema
core-agent-reading = { $url } lezen
core-agent-reading-page = Een pagina lezen
core-agent-using = { $tool } gebruiken
core-agent-none = een codeeragent op deze computer
core-agent-ended = { $name } is klaar zonder antwoord
core-agent-too-long = { $name } deed er te lang over

## core/src/core.rs

core-core-guest-trackers = de trackeraccounts op die computer zijn van de eigenaar, dus dit apparaat kan ze niet bijwerken
core-core-guest-not-interested = Niet interessant op die computer is van de eigenaar, dus dit apparaat kan het niet wijzigen
core-core-guest-recommended = aanbevelingen op die computer zijn geleerd uit de lijst van de eigenaar
core-core-nothing-picked = niets geselecteerd

## foundation/src/prefs.rs

core-prefs-not-a-provider = { $provider } is geen bron voor koppelen
core-prefs-provider-count = { $provider } staat er { $count } keer in; elke bron mag er maar één keer in staan
core-prefs-outside = { $value } moet tussen { $low } en { $high } liggen
core-prefs-empty = leeg

## metadata/src/recommended/mod.rs

core-recommended-ranking = Series voor je rangschikken

## metadata/src/recommended/sync.rs

core-recommended-finding-people = Mensen met jouw smaak zoeken
core-recommended-reading-lists = Hun lijsten lezen
core-recommended-reading-yours = Jouw series lezen
core-recommended-reading-suggestions = Series om aan te raden lezen

## metadata/src/recommended/page.rs

core-recommended-no-anime = { -anilist } heeft geen anime { $id }

## host/src/lib.rs

core-host-off = Uit
core-host-starting = Starten
core-host-serving = Bereikbaar op { $addresses }
core-host-no-tailscale = Tailscale is niet verbonden
core-host-cannot-listen = Kon niet luisteren
core-host-port-taken = Poort { $port } is bezet
core-host-cannot-listen-on = Kon niet luisteren op { $address }: { $error }

## host/src/addr.rs

core-host-not-an-address = { $text } is geen geldig adres met poort

## host/src/keys.rs

core-host-not-a-key = { $key } is geen apparaatsleutel

## net/src/limiter.rs

core-net-timed-out = time-out na { $ms }ms
core-net-rate-limited = { $provider } remt verzoeken af

## net/src/lib.rs

core-net-waiting = { $host }: kortgeleden een time-out; { -app-name } wacht { $seconds } s voordat het opnieuw probeert

## net/src/anilist.rs

core-net-no-data = geen gegevens

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } heeft geen download voor deze computer
core-update-cannot-make = Kon { $path } niet aanmaken: { $error }
core-update-cannot-mark = Kon de nieuwe AppImage niet uitvoerbaar maken: { $error }
core-update-cannot-replace = Kon { $path } niet vervangen: { $error }
core-update-not-here = Deze installatie kan zichzelf hier niet bijwerken
core-update-cannot-start = Kon de download niet starten: { $error }
core-update-did-not-start = De download is niet gestart: { $error }
core-update-cannot-write = Kon { $path } niet schrijven: { $error }
core-update-stopped = De download is gestopt: { $error }
core-update-cancelled = Geannuleerd
core-update-ended-early = De download stopte te vroeg ({ $got } van { $size } bytes)
core-update-checksum = De download past niet bij de checksum van GitHub
core-update-cannot-move = Kon de download niet op zijn plek zetten: { $error }
core-update-cannot-open-dmg = Kon de DMG niet openen: { $error }
core-update-cannot-read-dmg = Kon de DMG niet lezen: { $error }
core-update-no-app = De DMG bevat geen app
core-update-cannot-copy = Kon de nieuwe app niet naast deze kopiëren: { $error }

## metadata/src/calendar.rs

core-calendar-season-episode = { $title } - Seizoen { $season } Aflevering { $number }
core-calendar-episode = { $title } - Aflevering { $number }

## foundation/src/paths.rs

core-paths-no-home = geen thuismap

## core/src/first_run.rs

core-first-run-not-absolute = de videomap moet een absoluut pad zijn

## playback/src/presence/mod.rs

core-presence-extra = Extra
core-presence-room = Samen kijken
core-presence-in-room = In een kamer
core-presence-no-room = Die Discord-uitnodiging heeft nog geen kamer. Vraag of ze er een starten.

## trackers/src/accounts.rs

core-trackers-no-client-id = Geen client-ID ingesteld voor { $tracker }.
core-trackers-mal-reconnect = Koppel MyAnimeList opnieuw in Instellingen.
core-trackers-mal-wants-secret = MyAnimeList vereist het client secret van deze app. Voeg het toe onder Accounts in de instellingen van Anibeam, of registreer de app met App Type "other".
core-trackers-token-reply-unreadable = kon het tokenantwoord niet lezen: { $error }
core-trackers-token-reply-no-token = het tokenantwoord bevatte geen access token

## trackers/src/lib.rs

core-trackers-no-token = geen access token bewaard, koppel opnieuw in Instellingen

## trackers/src/cache.rs

core-trackers-no-user-id = het accountantwoord bevatte geen gebruikers-ID

## trackers/src/move_up.rs

core-trackers-move-none-picked = geen series geselecteerd om te verhuizen
core-trackers-moved-up =
    { $count ->
        [one] { $count } serie van deze computer naar { $site } verhuisd.
       *[other] { $count } series van deze computer naar { $site } verhuisd.
    }
core-trackers-moved-up-some-refused =
    { $moved ->
        [one] { $moved } serie
       *[other] { $moved } series
    } naar { $site } verhuisd; { $failed ->
        [one] { $failed } serie
       *[other] { $failed } series
    } kon niet worden verhuisd.
core-trackers-move-refused = { $site } kon { $show } niet bijwerken: { $error }

## trackers/src/oauth.rs

core-trackers-oauth-timed-out = Autorisatie verlopen, probeer het opnieuw.
core-trackers-oauth-state-mismatch = state komt niet overeen, mogelijk CSRF, afgebroken.
core-trackers-oauth-no-code = geen code in callback
core-trackers-oauth-no-token = geen access_token in callback
core-trackers-oauth-waiting = Wacht tot je inlogt bij { $site } in je browser.
core-trackers-oauth-connected = { $site } gekoppeld als { $username }.
core-trackers-oauth-no-name = het profielantwoord bevatte geen gebruikersnaam
core-trackers-oauth-could-not-start = Kon het inloggen niet starten
core-trackers-oauth-port-failed = poort { $port } op deze computer kon niet worden geopend: { $error }
core-trackers-oauth-port-busy = poort { $port } op deze computer wordt door een ander programma gebruikt. Wacht even en probeer het opnieuw, of sluit dat programma als de poort bezet blijft.
core-trackers-oauth-provider-error = { $site } gaf een fout: { $error }
core-trackers-page-connected = { $site } is gekoppeld aan { $app }
core-trackers-page-close-tab = Je kunt dit tabblad sluiten.
core-trackers-page-failed-title = { $app }, inloggen mislukt
core-trackers-page-failed = Inloggen is niet afgerond
core-trackers-page-go-back = Ga terug naar { $app } en probeer het opnieuw.
core-trackers-page-finishing-title = { $app }, bijna klaar
core-trackers-page-finishing = Bijna klaar
core-trackers-page-finishing-body = { $site } heeft gereageerd en Anibeam rondt het inloggen af. Ga terug naar { $app } om verder te gaan.
core-trackers-page-cancelled-title = { $app }, inloggen geannuleerd
core-trackers-page-cancelled = Inloggen is geannuleerd
core-trackers-page-cancelled-body = Je kunt dit tabblad sluiten. Begin opnieuw vanuit { $app } als je toch wilt inloggen.

## trackers/src/on_hold.rs

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

## trackers/src/writes.rs

core-trackers-rate-limited = { $tracker } remt verzoeken af, probeer het over een minuut opnieuw.
core-trackers-auth-expired = { $tracker }-login verlopen, koppel opnieuw in Instellingen.
core-trackers-entry-not-found = { $tracker }-item niet gevonden.
core-trackers-server-error = { $tracker } serverfout ({ $status }), probeer het later opnieuw.
core-trackers-error-see-log = { $tracker } fout, zie het activiteitenlog.
core-trackers-episode-not-whole = { $episode } moet een heel getal groter dan 0 zijn
core-trackers-score-out-of-range = { $score } moet tussen 0 en 10 liggen
core-trackers-rewatch-not-settable = herkijken kan niet handmatig worden ingesteld
core-trackers-notes-too-long = notities mogen niet langer zijn dan { $max } tekens
core-trackers-nothing-picked = niets geselecteerd
core-trackers-notes-not-on-list = Nog niet op je { $tracker }-lijst. Zet het eerst op een lijst.
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

## metadata/src/automatch.rs

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

## metadata/src/apply.rs

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

## metadata/src/fetch.rs

core-metadata-no-such-show = TVmaze heeft geen serie { $id }
core-metadata-no-such-media = AniList heeft geen media { $id }

## metadata/src/imdb.rs

core-metadata-progress-imdb-links = IMDb-links van Wikidata
core-metadata-progress-imdb-ratings = IMDb-beoordelingen

## metadata/src/franchise/layout.rs

core-franchise-relation-character = Gedeelde personages

## net/src/images.rs

core-images-progress-fetching = afbeeldingen ophalen

## host/src/transfer/export.rs

core-transfer-exported =
    { $count ->
        [one] { $count } serie opgeslagen in { $folder }.
       *[other] { $count } series opgeslagen in { $folder }.
    }

## host/src/transfer/format.rs

core-transfer-not-json = geen JSON: { $error }
core-transfer-wrong-format = geen { $format }-document

## host/src/transfer/import.rs

core-transfer-imported = Je bibliotheek is uit het bestand geladen.

## library/src/delete.rs

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

## library/src/detail.rs

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
core-library-fix-numbered-on = Afleveringen { $from } lijken afleveringen { $to }, doorgenummerd vanaf een eerder seizoen.
core-library-runs-span = { $first } t/m { $last }
core-library-runs-list = { $init } en { $last }

## library/src/fixes.rs

core-library-fix-episode-range = het afleveringsnummer moet tussen 0 en 10000 liggen
core-library-fix-season-range = het seizoensnummer moet 200 of lager zijn
core-library-fix-extra-range = de index van een extra moet 10000 of lager zijn
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

## library/src/reads.rs

core-library-hid-one = { $title } verborgen.
core-library-back-one = { $title } is terug.
core-library-hid-many = { $count } series verborgen.
core-library-back-many = { $count } series zijn terug.
core-library-episode-count-set = { $title } heeft nu { $count } afleveringen.
core-library-episode-count-cleared = Het aantal afleveringen van { $title } gewist.

## library/src/schedule.rs

core-library-schedule-cleared = De schemanotities van { $title } gewist.
core-library-schedule-saved = { $count } schemanotities voor { $title } opgeslagen.

## library/src/scan.rs

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

## library/src/report.rs

core-library-agent-no-answer = geen antwoord binnen { $span }
core-library-agent-stopped-code = gestopt met code { $code }
core-library-agent-stopped = gestopt
core-library-agent-answered-nothing = gaf geen uitvoer
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

## downloads/src/anirss.rs

core-downloads-anirss-no-answer = anirss reageerde niet: { $message }
core-downloads-anirss-answered = anirss gaf HTTP { $status }: { $text }
core-downloads-no-free-port = geen vrije poort: { $error }
core-downloads-anirss-exited = anirss is gestopt ({ $status })
core-downloads-anirss-exited-said = anirss is gestopt ({ $status }): { $said }
core-downloads-anirss-slow-start = anirss startte niet binnen { $seconds } seconden
core-downloads-anirss-slow-start-said = anirss startte niet binnen { $seconds } seconden: { $said }
core-downloads-anirss-install-mac = Installeer anirss { $version } of nieuwer met: { $command }
core-downloads-anirss-install-linux = Installeer anirss { $version } of nieuwer: het anirss-pakket in de AUR, of { $command }
core-downloads-anirss-too-old = anirss { $version } is te oud voor Anibeam, dat { $minimum } of nieuwer nodig heeft. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } kan de login van elke torrentclient niet apart houden, en dat heeft Anibeam nodig om van client te wisselen; anirss { $minimum } en nieuwer kunnen dat wel. { $update }.
core-downloads-anirss-update-mac = Werk het bij met: { $command }
core-downloads-anirss-update-linux = Werk het anirss-pakket bij, of voer uit: { $command }

## downloads/src/contents.rs

core-downloads-torrent-file-no-answer = kon het torrentbestand niet downloaden: { $message }
core-downloads-torrent-file-answered = het torrentbestand gaf HTTP { $status }

## downloads/src/feed.rs

core-downloads-feed-answered = de feed gaf HTTP { $status }
core-downloads-not-a-feed = geen feed: { $error }

## downloads/src/jackett.rs

core-downloads-jackett-missing = Jackett is niet geïnstalleerd: { $program } is niet gevonden. Installeer het, of wijzig het commando in Instellingen.
core-downloads-endpoint-start-failed = { $name } kon niet starten: { $why }
core-downloads-endpoint-start-timeout = { $name } is gestart maar reageerde niet binnen { $seconds } seconden.
core-downloads-command-exited = het commando stopte met { $status }

## downloads/src/lib.rs

core-downloads-anirss-stopped = anirss is gestopt
core-downloads-anirss-missing = { $install }. Je kunt Instellingen ook naar een draaiende server laten wijzen.
core-downloads-anirss-command-missing = { $command } is niet gevonden. { $install }. Je kunt Instellingen ook naar een draaiende server laten wijzen.
core-downloads-anirss-start-failed = { $command } kon niet starten: { $error }
core-downloads-anirss-too-old-said = { $tooOld } Het zei: { $message }
core-downloads-no-torrent-client = Geen qBittorrent reageerde op { $address }, en Anibeam heeft geen ingebouwde client beschikbaar. Installeer qBittorrent en zet de Web UI aan, of installeer qbittorrent-nox, dat Anibeam dan automatisch draait
core-downloads-own-client-gone = De ingebouwde qBittorrent van Anibeam ontbreekt (qbittorrent-nox is niet geïnstalleerd). Installeer Anibeam opnieuw, of laat Instellingen naar je eigen qBittorrent wijzen
core-downloads-no-port = geen poort in { $address }
core-downloads-own-client-settings = kan de instellingen van qBittorrent niet schrijven: { $error }
core-downloads-own-client-start-failed = { $command } kon niet starten: { $error }
core-downloads-own-client-slow-start = De qBittorrent van Anibeam is niet gestart
core-downloads-own-client-slow-start-said = De qBittorrent van Anibeam is niet gestart: { $said }
core-downloads-not-ready = Downloads zijn niet klaar. Controleer Downloads in Instellingen.
core-downloads-ready = Downloads zijn klaar.
core-downloads-helper-sources =
    { $count ->
        [one] Downloads zoeken nu direct in Anibeam, dat de { $count } bron in de config van de zoekhulp niet leest. Voeg hem toe in Instellingen > Downloads, of schakel daar terug naar de zoekhulp.
       *[other] Downloads zoeken nu direct in Anibeam, dat de { $count } bronnen in de config van de zoekhulp niet leest. Voeg ze toe in Instellingen > Downloads, of schakel daar terug naar de zoekhulp.
    }
core-downloads-client-connected = De downloadapp is gekoppeld.
core-downloads-client-not-connected = De downloadapp is niet gekoppeld. Controleer het adres in Instellingen.
core-downloads-not-on-demand = { $name } is geen bron op aanvraag
core-downloads-source-running = De downloadbron draait.
core-downloads-source-not-running = { $name } draait niet. Zet het aan bij je bronnen en bevestig om het te starten.
core-downloads-empty-query = typ iets om te zoeken
core-downloads-first-endpoint = de eerste bron
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
core-downloads-library-folder = { $folder } is de bibliotheekmap van { $name } en bevat afleveringen die dit abonnement niet downloadde. Verwijder het abonnement zonder de bestanden te wissen, of verwijder { $name } vanaf de pagina ervan
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
core-downloads-feed-url = de feed-URL moet beginnen met http:// of https://
core-downloads-rule-needs-name = de naam van een regel mag niet leeg zijn
core-downloads-no-rule = geen regel met de naam { $name }
core-downloads-feed-no-search = deze feed heeft geen zoekopdracht om in te korten
core-downloads-no-repair = { $name } hoeft niet hersteld te worden.
core-downloads-repaired = Het abonnement op { $name } hersteld.
core-downloads-repair-failed = geen kortere zoekopdracht gaf resultaten; zelfs "{ $search }" gaf niets
core-downloads-no-saved-client = geen bewaarde torrentclient { $id }
core-downloads-client-switched = Downloads gebruiken nu een andere downloadapp.
core-downloads-host-not-running = draait niet op de host
core-downloads-host-late = reageerde niet op tijd

## downloads/src/native.rs

core-downloads-native-empty-query = lege zoekopdracht
core-downloads-native-no-results = geen resultaten voor { $query }
core-downloads-native-feed-taken = een feed met de naam { $name } wijst al naar een ander adres
core-downloads-native-no-items = geen items gegeven
core-downloads-native-bad-link = links moeten magnet: of http(s)-URL's zijn, kreeg { $link }
core-downloads-native-no-links = geen links gegeven
core-downloads-native-no-names = geen abonnementsnamen gegeven
core-downloads-native-files-elsewhere = de torrentclient bewaart zijn bestanden op een andere machine, dus die kunnen alleen samen met de torrents worden gewist
core-downloads-native-unknown-feeds = onbekende feed(s): { $names }
core-downloads-native-outside-folders = bestanden buiten de downloadmappen worden niet weggehaald: { $paths }
core-downloads-refuse-home-folder = { $path } is of bevat je thuismap, dus er is niets verwijderd
core-downloads-refuse-library-folder = { $path } is of bevat een bibliotheekmap, dus er is niets verwijderd

## downloads/src/paths.rs

core-downloads-no-default-folder = qBittorrent heeft geen standaard opslagmap; stel er een in qBittorrent in
# $folder is the folder given
core-downloads-save-folder-not-absolute = { $folder } is geen volledig pad naar een map
# $folder is the folder given
core-downloads-save-folder-unusable = { $folder } kan niet voor downloads worden gebruikt: kies een bestaande map die niet je thuismap is
core-downloads-name-empty = de naam mag niet leeg zijn
core-downloads-name-slash = de naam mag geen / of \ bevatten
core-downloads-name-not-folder = { $name } is geen mapnaam

## downloads/src/prune.rs

core-downloads-pruned = Abonnement op { $title } opgezegd (uitzending afgelopen).
core-downloads-prune-failed = Kon het abonnement op { $title } niet opzeggen. Controleer Downloads in Instellingen.

## library/src/trash.rs

core-downloads-refuse-not-absolute = { $path } is geen gewoon absoluut pad, dus er is niets weggehaald
core-downloads-refuse-root = { $path } is de root van het bestandssysteem, dus er is niets weggehaald
core-downloads-refuse-outside-library = { $path } ligt niet in een bibliotheekmap, dus er is niets weggehaald
core-downloads-refuse-save-folder = { $path } is de opslagmap van de torrent, niet de gedownloade bestanden, dus er is niets verwijderd

## downloads/src/watch.rs

core-downloads-episode-missed = Aflevering { $episode } van { $title } is drie dagen na uitzending nog steeds niet verschenen. Je feed blijft ernaar zoeken.

## downloads/src/settings.rs

core-downloads-settings-command-empty = het anirss-commando mag niet leeg zijn
core-downloads-settings-anirss-address = het anirss-adres moet beginnen met http:// of https://
core-downloads-settings-client-address = het { $client }-adres moet beginnen met http:// of https://
core-downloads-settings-bad-id = { $id } is geen client-ID: gebruik letters, cijfers, - en _
core-downloads-settings-same-id = twee clients hebben het ID { $id }
core-downloads-settings-saved-address = het adres van { $name } moet beginnen met http:// of https://
core-downloads-settings-same-name = twee clients heten { $name }; geef elk een unieke naam

## downloads/src/qbt.rs

core-downloads-qbt-no-answer = qBittorrent reageerde niet op { $address }: { $message }
core-downloads-qbt-password-rejected = qBittorrent weigerde het wachtwoord; pas het aan in Instellingen
core-downloads-qbt-banned = qBittorrent heeft dit adres geblokkeerd na mislukte logins; wacht, of herstart qBittorrent
core-downloads-qbt-login-rejected = qBittorrent weigerde de gebruikersnaam of het wachtwoord
core-downloads-qbt-login-failed = de login van qBittorrent op { $address } gaf HTTP { $status }; bij het volgende verzoek opnieuw
core-downloads-qbt-not-web-ui = { $address } gaf HTTP { $status }, en dat lijkt niet op de Web UI van qBittorrent. Controleer het adres in Instellingen en zorg dat de Web UI aan staat in qBittorrent
core-downloads-qbt-not-added = qBittorrent voegde de torrents niet toe: { $said }
core-downloads-qbt-some-not-added = qBittorrent kon { $count } van de torrents niet toevoegen: { $said }
core-downloads-qbt-answered = qBittorrent gaf HTTP { $status }: { $line }

## downloads/src/replace.rs

core-downloads-replace-bad-item = item { $item } is niet een van de { $count } releases
core-downloads-not-replaced = Aflevering { $episode } is niet vervangen.
core-downloads-replaced = Aflevering { $episode } vervangen door de nieuwe download.
core-downloads-no-video-moved = { $name } bevat geen video, dus Anibeam kan het niet afspelen. Het staat nu in { $folder }.
core-downloads-no-video = { $name } bevat geen video, dus Anibeam kan het niet afspelen.

## downloads/src/rolling.rs

core-downloads-no-roll = geen doorlopende download { $id }
core-downloads-rolled = { $title }: nu op afleveringen { $from } t/m { $to } van { $total }.
core-downloads-roll-ahead = houd minstens één aflevering voorsprong
core-downloads-roll-nothing = niets op te halen

## releases/src/endpoints.rs

core-releases-json-needs-repo = een JSON-zoekbron moet uit een bronlijst komen waarvan de definitie de velden koppelt
core-releases-endpoint-no-name = `name` ontbreekt
core-releases-endpoint-bad-name = { $name }: gebruik alleen letters, cijfers, '.', '_', '-'
core-releases-endpoint-json = { $name }: { $why }
core-releases-endpoint-unknown-kind = { $name }: onbekend soort { $kind } (geldig: { $valid })
core-releases-endpoint-no-url = { $name }: `url` ontbreekt
core-releases-endpoint-not-http = { $name }: het adres moet beginnen met http:// of https://
core-releases-endpoint-needs-query = { $name }: een { $kind }-URL moet een placeholder {"{"}query{"}"} bevatten
core-releases-endpoint-no-query = { $name }: een { $kind }-URL voegt de zoekterm zelf toe, dus hij mag geen {"{"}query{"}"} bevatten

## releases/src/fetch.rs

core-releases-indexer-error = de indexer gaf fout { $code }
core-releases-bad-feed = ongeldige feed: { $error }
core-releases-feed-answered = de feed gaf HTTP { $status }

## releases/src/json_feed.rs

core-releases-not-feed-json = niet de JSON van de feed: { $error }

## releases/src/json.rs

core-releases-not-json = geen JSON: { $error }
core-releases-search-error-said = de zoekopdracht gaf een fout: { $message }
core-releases-search-error = de zoekopdracht gaf een fout
core-releases-no-results-field = het antwoord heeft geen resultaten op { $path }
core-releases-results-not-list = de resultaten in het antwoord zijn geen lijst
core-releases-search-answered = de zoekopdracht gaf HTTP { $status }

## releases/src/native.rs

core-releases-no-endpoints = nog geen downloadbronnen: voeg eerst een bronlijst of een eigen bron toe
core-releases-no-source-named = geen bron met de naam { $name }
core-releases-empty-query = typ iets om te zoeken
core-releases-cannot-follow = { $name } kan niet worden gevolgd: het ondersteunt zoeken, maar heeft geen feed voor nieuwe afleveringen
core-releases-endpoint-gone = { $name } is niet meer beschikbaar: verwijderd, uitgezet of de sleutel ontbreekt

## releases/src/quirks.rs

core-releases-quirks-variant = one_variant { $word } moet één kort woord zijn
core-releases-quirks-pattern-count = show_feed neemt 1 tot { $most } paginapatronen
core-releases-quirks-pattern-long = een paginapatroon van show_feed is te lang
core-releases-quirks-pattern-bad = paginapatroon van show_feed: { $error }
core-releases-quirks-pattern-slug = een paginapatroon van show_feed heeft een groep met de naam slug nodig
core-releases-quirks-url-https = de url van show_feed moet https zijn
core-releases-quirks-url-host = de url van show_feed moet op de eigen host van het endpoint staan
core-releases-quirks-url-slug = de url van show_feed neemt {"{"}slug{"}"} één keer en geen andere placeholder

## releases/src/refinement.rs

core-releases-refine-no-tokens = op 'tokens' heeft een niet-lege lijst `tokens` nodig

## releases/src/grid.rs

core-releases-group-batch = Batch van { $group }
core-releases-episodes-phrase =
    { $count ->
        [one] Aflevering { $list }
       *[other] Afleveringen { $list }
    }
core-releases-episode-range = { $from } t/m { $to }

## Names that stay the same in every language

-app-name = Anibeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = de naam moet 1 tot { $max } tekens zijn, zonder stuurtekens
core-prefs-room-relay = het relayadres moet beginnen met https:// of http://
core-room-service = Samen kijken
core-room-not-found = Geen kamer met die code. Misschien is hij gesloten.
core-room-full = Die kamer is vol.
core-room-proto = Die server gebruikt een andere versie van Samen kijken. Werk { -app-name } bij om mee te doen.
core-room-needs-episode = Die server vereist een aflevering voordat er een kamer start. Start er een vanuit de speler, of werk de server bij.
core-room-unreachable = Kon de kamerserver niet bereiken: { $message }
core-room-refused = De kamerserver weigerde het verzoek: { $message }
core-room-not-in = Je zit niet in een kamer.
core-room-no-name = vul de naam in die anderen in de kamer zien
core-room-bad-code = een kamercode bestaat uit 6 letters of cijfers
core-room-created = Een kamer voor Samen kijken gestart. Code: { $code }.
core-room-joined = Lid geworden van kamer { $code } voor Samen kijken.
core-room-left = De kamer voor Samen kijken verlaten.
core-room-member-joined = { $name } kwam de kamer in.
core-room-member-left = { $name } verliet de kamer.
core-room-closed = De kamer voor Samen kijken is gesloten.
core-room-lost = Verbinding met de kamer voor Samen kijken verloren. Opnieuw verbinden.
core-room-back = Weer verbonden met de kamer voor Samen kijken.
core-room-media-asked = De host om de aflevering gevraagd.
core-room-media-cannot = De host kan deze aflevering nu niet delen.
core-room-media-sent = { $name } de torrent van de aflevering gestuurd.
core-room-media-sent-name = { $name } de naam van de release van de aflevering gestuurd.
core-room-media-nothing = De host heeft geen kopie van de aflevering om te delen.
core-room-media-no-answer = De app van de host reageerde niet. Probeer het zo opnieuw.
core-room-media-torrent = De aflevering wordt gedownload via de torrent van de host.
core-room-media-searching = Je bronnen worden doorzocht op de release van de host.
core-room-media-not-found = Geen van je bronnen heeft de release van de host.
core-room-media-no-sources = Alle bronnen staan uit. Zet er een aan om te zoeken.
core-room-media-no-start = De download kon niet starten: { $message }
core-room-media-client = de torrentclient weigerde hem
core-room-media-lost = De torrentclient is de download van de aflevering kwijt.
core-room-media-stalled = De download staat al tien minuten stil. Probeer het later opnieuw.
core-room-media-not-in-library = De aflevering is gedownload, maar verscheen niet in je bibliotheek. Controleer of downloads in een bibliotheekmap terechtkomen.
core-room-media-here = De aflevering staat nu in je bibliotheek.
core-room-media-rest-missing = De rest van de serie kon niet worden opgehaald. Alleen deze aflevering wordt gedownload.
