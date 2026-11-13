## Namen die namen blijven

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Woorden die veel pagina's delen

common-cancel = Annuleren
common-back = Terug
common-save = Opslaan
common-done = Klaar
common-close = Sluiten
common-delete = Wissen
common-retry = Opnieuw
common-copy = Kopieer
common-clear = Leegmaken
common-on = Aan
common-off = Uit
common-play = Afspelen
common-hide = Verberg
common-remove = Weghalen
common-add = Toevoegen
common-test = Test
common-reading = Lezen...
common-working = Bezig...
common-checking = Controleren...
common-all = Alles
common-none = Geen
common-episodes = { $count ->
    [one] { $count } aflevering
   *[other] { $count } afleveringen
}
common-episode-n = Aflevering { $n }
common-join-comma = { $a }, { $b }

## Taal (#793)

lang-follow-phone = Zoals de telefoon
more-language = Taal
more-language-phone = Zoals de telefoon: { $lang }
lang-name-en = Engels
lang-name-da = Deens
lang-name-de = Duits
lang-name-es = Spaans
lang-name-fr = Frans
lang-name-pt-BR = Portugees (Brazilië)
lang-name-it = Italiaans
lang-name-nl = Nederlands
lang-name-pl = Pools
lang-name-ru = Russisch
lang-name-ja = Japans
lang-name-ko = Koreaans
lang-name-zh-CN = Chinees (vereenvoudigd)

## Introductie

onb-welcome-title = AniBeam
onb-welcome-note = Je anime op deze telefoon. Zoek een serie, zie wie hem heeft, download hem en houd je lijsten bij.
onb-language-title = Taal
onb-language-note = AniBeam spreekt de taal van je telefoon, tenzij je een andere kiest. Je kunt dit later wijzigen onder Meer.
onb-lists-title = Je lijsten
onb-lists-note = Log in zodat wat je hier kijkt meetelt op AniList of MyAnimeList. Dit kan ook later onder Meer.
onb-wifi-title = Alleen Wi-Fi
onb-wifi-note = Afleveringen zijn groot. AniBeam downloadt via Wi-Fi, tenzij je anders kiest.
onb-storage-title = Opslag
onb-storage-note = Een limiet voor wat AniBeam mag downloaden. Er wordt niets gereserveerd: de ruimte blijft vrij tot afleveringen haar vullen.
onb-notify-title = Meldingen
onb-notify-note = Tijdens het downloaden toont een melding de voortgang, en een andere meldt wanneer een aflevering klaar is om te kijken. Downloads lopen hoe dan ook door.
onb-sites-title = Zoeksites
onb-sites-note = AniBeam wordt zonder zoekbronnen geleverd. Een repo is een ondertekende lijst met sites om te doorzoeken. Plak het adres van een repo, controleer de sleutel en installeer dan de sites die je wilt. Dit kan ook later onder Meer.
onb-wifi-only = Alleen via Wi-Fi
onb-wifi-on = Aan. Mobiele data vraagt eerst.
onb-wifi-off = Uit. Downloads gebruiken ook mobiele data.
onb-storage-caption = 18 GB is ongeveer 12 afleveringen. Schuif tot maximaal de hele opslag van de telefoon.
onb-use-cap = Gebruik limiet
onb-cap-on = Aan. Bij { $cap } downloadt AniBeam niets meer tot je afleveringen wist.
onb-cap-off = Uit. AniBeam downloadt tot de telefoon vol is.
onb-start = Starten
onb-continue = Verder
onb-back = Terug

## Weergave

look-colours = Kleuren
look-mode = Modus
look-dark = Donker
look-light = Licht
look-system = Systeem
look-follows-phone-dark = Volgt de instelling van de telefoon: donker.
look-follows-phone-light = Volgt de instelling van de telefoon: licht.
look-changes-at-once = Wijzigingen zie je meteen, hier en op elke pagina.
look-dark-theme = Donker thema
look-light-theme = Licht thema
look-accent = Accent
look-accent-note = Markeert wat geselecteerd is, afspeelt of nieuw is.
look-size = Grootte
look-text-size = Tekstgrootte
look-size-smaller = Kleiner
look-size-normal = Normaal
look-size-large = Groot
look-size-larger = Groter
look-size-largest = Grootst
look-text-size-note = Hoe groot de tekst overal in AniBeam is, bovenop de grootte van je telefoon.

## Het venster: tabs en de sheets boven elke pagina

app-name = AniBeam
nav-home = Start
nav-calendar = Agenda
nav-library = Bibliotheek
nav-downloads = Downloads
nav-more = Meer
app-mobile-title = Downloaden via mobiele data?
app-mobile-note = Eén aflevering is vaak 1 tot 2 GB. Met een beperkt abonnement kan dat in één avond een maand aan data kosten.
app-mobile-keep = Alleen Wi-Fi houden
app-mobile-use = Ook mobiele data
app-versions-note = Dezelfde aflevering uit { $count } releases. Kies er een om af te spelen.
app-versions-plays-now = { $release }, speelt nu
app-menu-get = Afl. ophalen
app-menu-delete = Afl. wissen

## Meer: het overzicht en per onderdeel een regel over de stand

more-settings = Instellingen
more-search = Instellingen zoeken
more-search-none = Geen resultaten
more-search-hits = { $count ->
    [one] 1 instelling
   *[other] { $count } instellingen
}
more-search-tap = Tik om te openen
more-search-hint = Probeer een eenvoudiger woord: opslag, Wi-Fi, tracker, site, computer.
more-accounts = Accounts
more-streaming = Streamen
more-subtitles = Ondertitels
more-connection = Verbinding
more-computer = Je computer
more-icon = App-icoon
more-stats = Stats
more-data = Gegevens
more-appearance = Weergave
more-activity = Activiteit
more-about = Over
more-accounts-none = Geen tracker gekoppeld
more-accounts-syncing = Synct met { $tracker }
more-accounts-syncing-both = Synct met AniList en MAL
more-dl-status-wifi = Wi-Fi, geen limiet
more-dl-status-wifi-cap = Alleen Wi-Fi, limiet { $cap }
more-dl-status-mobile = Wi-Fi en mobiele data, geen limiet
more-dl-status-mobile-cap = Wi-Fi en mobiele data, limiet { $cap }
more-stream-status-auto = Auto, 720p via mobiele data of relay
more-stream-status-original = Telkens het originele bestand
more-stream-status-converted = Omgezet naar { $quality }
more-sites-status-none = Nog geen repo
more-sites-status-on = { $sites ->
    [one] 1 site aan
   *[other] { $sites } sites aan
}
more-sites-status-on-from = { $sites ->
    [one] 1 site aan
   *[other] { $sites } sites aan
}, { $repos ->
    [one] uit 1 repo
   *[other] uit { $repos } repos
}
more-conn-status-unchecked = Nog niet getest
more-conn-status-ok = Elke stap is gelukt
more-conn-status-failed = { $total ->
    *[other] { $failed } van { $total } stappen mislukt
}
more-computer-status-none = Nog niet verbonden
more-icon-default = Standaard
more-icon-dark = Donker
more-icon-line = Lijntekening
more-subs-status = Grootte, plek, lettertype, kleuren en vak
more-data-status = Back-up, herstel, afbeeldingscache
more-activity-status = Wat AniBeam deed, nieuwste eerst
more-look-status-phone = Volgt de telefoon
more-stats-status = { $count ->
    [one] { $hours } uur gekeken in { $year }
   *[other] { $hours } uur gekeken in { $year }
}
more-stats-status-empty = Een jaar kijken

## Meer: de eigen instellingen van de onderdelen

more-accounts-note = Wat je hier kijkt werkt je lijsten bij, en voortgang van daar zie je terug bij je series.
more-storage = Opslag
more-storage-capped = Limiet { $cap }
more-storage-no-cap = Onbeperkt
more-storage-used = voor afleveringen
more-storage-cap-caption = limiet
more-storage-free = vrij op de telefoon
more-set-storage-cap = Opslaglimiet
more-storage-cap-on = Bij { $cap } downloadt er niets meer tot je afleveringen wist.
more-storage-cap-off = Uit: AniBeam downloadt tot de telefoon vol is.
more-network = Netwerk
more-network-wifi = Alleen Wi-Fi
more-network-mobile = Wi-Fi en mobiele data
more-wifi-on = Aan. Downloads wachten op Wi-Fi; mobiele data vraagt eerst.
more-set-stream-quality = Streamkwaliteit
more-stream-question = Hoe een aflevering van je computer hier afspeelt.
more-stream-auto-note = Auto speelt het bestand ongewijzigd af via Wi-Fi, en omgezet naar 720p via mobiele data of als Tailscale je computer via een relay bereikt.
more-stream-original-note = Telkens het bestand zoals het is. Grote bestanden hebben een snelle verbinding nodig.
more-stream-converted-note = Op je computer omgezet naar { $quality }. Gebruikt minder data; spoelen duurt even.
more-set-feeds = Je eigen feeds
more-feeds-note = Eerst doorzocht, voor de sites
more-set-check = Verbinding controleren
more-check-note = Test elke stap tussen deze telefoon en de zoeksites: ze vinden, ze bereiken en een beveiligde verbinding.
more-check-copy = Resultaten kopiëren
more-set-computer = AniBeam op je computer
more-computer-note = De bibliotheek ervan staat in Bibliotheek en de afleveringen spelen hier af. Een naam als mijn-computer of een adres als 100.64.0.1.
more-computer-placeholder = mijn-computer:8790
more-subs-note = Waarmee elke aflevering op deze telefoon begint. Een aflevering die van je computer streamt, begint met de instellingen van die computer.
more-icon-note = Het icoon op je startscherm en in het appmenu.
more-icon-line-note = Lijntekening neemt de kleuren van je achtergrond over, zoals je andere Material You-iconen.
more-icon-moved-note = Sommige launchers halen het icoon van het startscherm als het wijzigt. Het blijft in het appmenu; sleep het daar terug.
more-set-whats-new = Wat is nieuw
more-whats-new-note = Wat er in elke versie veranderde, nieuwste eerst.
more-set-onboarding = Opnieuw starten
more-onboarding-note = Doorloop de vragen van de eerste start nog een keer.
more-set-version = Versie

## Statusregels die Rust op de pagina's schrijft

host-synced = Gesynct { $when }.
host-last-try = Laatste poging { $when }: { $error }
host-asking = Vragen...
host-reached = { $host } bereikt, AniBeam { $version }.
host-saved = Opgeslagen. Syncen...
acct-signed-in-as = Ingelogd als { $name }
acct-signed-in = Ingelogd
acct-not-in-build = Niet beschikbaar in deze build
acct-tap-to-sign-in = Niet ingelogd. Tik om in te loggen.
acct-sign-in-failed = Inloggen mislukt: { $why }. Tik om opnieuw te proberen.
acct-sign-in-not-started = Inloggen is niet gestart: { $error }. Tik om opnieuw te proberen.
acct-still-starting = AniBeam is nog aan het starten. Probeer het zo opnieuw.
home-recently-watched = Recent gekeken
home-recently-downloaded = Recent gedownload
home-note-empty = Op deze telefoon
home-note = { $count ->
    [one] Op deze telefoon, 1 serie
   *[other] Op deze telefoon, { $count } series
}
lib-this-phone = Deze telefoon
lib-note-phone = { $count ->
    [one] Deze telefoon, 1 serie
   *[other] Deze telefoon, { $count } series
}
lib-note-remote = { $device }, gesynct { $when }
lib-empty-search = Niets gevonden voor "{ $query }".
lib-empty-phone = Nog niets op deze telefoon.
lib-empty-remote = Dat apparaat heeft niets in zijn bibliotheek.
cal-view-airing = Nu uitgezonden
cal-note-none = Geen uitzendingen deze week
cal-note = { $count ->
    [one] 1 aflevering in de komende 7 dagen
   *[other] { $count } afleveringen in de komende 7 dagen
}
airing-asking = AniList vragen
airing-asking-newest = AniList om het nieuwste vragen
airing-filter-all = Alles { $count }
airing-filter-listed = Op mijn lijsten { $count }
airing-filter-owned = Op deze telefoon { $count }
airing-this-season = Dit seizoen
airing-next-season = Volgend seizoen
airing-other-season = Overig
airing-hidden = { $count ->
    *[other] { $count } verborgen als niet interessant
}
airing-plan-needs-tracker = Later kijken vraagt om AniList of MAL. Log in onder Meer, Accounts.
airing-plan-failed = Later kijken is niet gelukt: { $error }
list-watching = Kijken
list-planning = Gepland
list-completed = Voltooid
list-paused = Gepauzeerd
list-dropped = Gestopt
list-repeating = Herkijken
more-check-all-ok = Elke stap is gelukt.
more-check-failed = { $failed } van { $total } stappen mislukt. Een screenshot van deze lijst laat zien waar.
more-check-row = { $step }  { $ms } ms
more-check-copied = Gekopieerd. Plak het in een bericht.
more-version = AniBeam voor Android { $version } (build { $build })
dl-note-waiting = Wachten op Wi-Fi. Downloads gaan vanzelf verder zodra de telefoon op Wi-Fi zit.
dl-note-nothing = Er wordt niets gedownload.
dl-state-done = Klaar
dl-state-waiting = Wacht op Wi-Fi
dl-state-paused = Gepauzeerd
dl-state-peers = Peers zoeken
dl-state-checking = Controleren
dl-state-error = Gestopt door een fout
dl-state-downloading = Downloaden
dl-amount = { $got } van { $size }
dl-speed = { $rate }/s
dl-left = nog { $span }
dl-episodes-range = Aflevering { $from } t/m { $to }
dl-batch = Een batch
disk-storage-line = { $used } gebruikt
disk-storage-line-free = { $used } gebruikt, { $free } vrij
disk-free = Deze telefoon heeft { $free } vrij.
disk-free-below-cap = Deze telefoon heeft { $free } vrij, minder dan de limiet: de telefoon loopt vol voordat AniBeam stopt.

## Zoeken in Meer: de woorden die iemand voor elke instelling kan typen

more-set-subs-size = Grootte en plek van ondertitels
more-set-subs-style = Ondertitelstijl
more-words-anilist = anilist tracker inloggen account koppelen lijst voortgang
more-words-mal = mal myanimelist tracker inloggen account koppelen lijst voortgang
more-words-storage-cap = ruimte schijf limiet gb vol opslag wissen grootte
more-words-wifi = wifi wi-fi mobiel data netwerk
more-words-notifications = melding meldingen waarschuwing voortgang toestemming toestaan
more-words-stream-quality = auto origineel 1080p 720p 480p omzetten data computer afspelen
more-words-check-repos = vernieuwen update repo endpoint
more-words-add-repo = repo endpoint vertrouwen vingerafdruk adres url
more-words-add-site = eigen endpoint feed rss torznab jackett prowlarr adres
more-words-feeds = feed rss bron verwijderen
more-words-check = netwerk dns test https beveiligd bereiken internet
more-words-computer = tailscale tailnet host adres desktop bibliotheek streamen test opslaan apparaat
more-words-whats-new = changelog wijzigingen nieuw update
more-words-onboarding = introductie eerste start opnieuw beginnen
more-words-version = build over
more-words-stats = uren afleveringen genres studio's seizoenen jaar gekeken terugblik drukste maand geschiedenis
more-words-backup = back-up exporteren bestand opslaan kopiëren verplaatsen computer json privé logins
more-words-restore = herstellen importeren bestand laden bibliotheek samenvoegen
more-words-cache = afbeeldingen posters banners wissen opslag ruimte cache
more-words-activity = log gebeurtenissen fouten problemen kopiëren wissen geschiedenis wat gebeurde
more-words-colours = weergave thema donker licht modus systeem accent kleur
more-words-text-size = weergave lettertype groter kleiner grootte tekst lezen
more-words-icon = icoon launcher startscherm logo donker lijn thema material you
more-words-subs-size = ondertitels grootte schaal positie omhoog onderkant
more-words-subs-style = ondertitels lettertype kleur omlijning schaduw vak vet ass stijl herstellen
more-words-language = taal vertaling engels nederlands

## Start

home-title = Start
home-next-to-air = Eerstvolgende
home-after-that = Daarna
home-coming-up = Binnenkort
home-nothing-week = Niets de komende week.
home-empty-title = Nog niets op deze telefoon
home-empty-body = Series die je hier downloadt, en wat ervan wordt uitgezonden, staan op Start. Zoek er een onder Downloads.

## Bibliotheek

lib-title = Bibliotheek
lib-refresh = Vernieuwen
lib-search-phone = Zoek op deze telefoon
lib-search-device = Zoek op { $device }
lib-find-in-downloads = "{ $query }" zoeken in Downloads
lib-empty-body = Afleveringen die je op deze telefoon downloadt komen hier. Zoek een serie onder Downloads.
lib-pick-show = Kies een serie om hem hier te openen.
lib-shows-caption = { $count ->
    [one] serie
   *[other] series
}
lib-on-phone = op de telefoon
lib-free = vrij
lib-hold-poster = Houd een poster ingedrukt om meer afleveringen op te halen of te wissen.
phone-nothing-on-phone = niets op telefoon
phone-on-phone = { $count } op telefoon

## Agenda

cal-title = Agenda
cal-week = Week
cal-days = Dagen
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A %-d %B
cal-today = Vandaag
cal-tomorrow = Morgen
cal-yesterday = Gisteren
cal-nothing = Niets
cal-next = Hierna
cal-get = Ophalen
cal-day-note = { $date }, { $count }
cal-next-airing = Volgende: { $title }
cal-nothing-aired-yesterday = Gisteren is niets uitgezonden.
cal-nothing-aired-that-day = Die dag is niets uitgezonden.
cal-nothing-today = Vandaag geen uitzending.
cal-nothing-tomorrow = Morgen geen uitzending.
cal-nothing-that-day = Die dag geen uitzending.
cal-empty-title = Nog geen series op deze telefoon
cal-empty-body = Afleveringen van de series die je hier bewaart staan op hun dag, twee dagen terug en een week vooruit.
phone-episodes-range = Aflevering { $from } t/m { $to }
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = Op de telefoon
phone-out-ago = Uit { $ago }

## Nu uitgezonden

airing-out-now = Nu uit
airing-last-two-days = De laatste twee dagen
airing-season-note = De series van het seizoen komen van AniList.
airing-none-lists-title = Niets dat loopt staat op je lijsten
airing-none-lists-body = Later kijken bij een serie zet hem op je lijst.
airing-none-phone-title = Niets dat loopt staat op deze telefoon
airing-none-phone-body = Afl. ophalen bij een serie downloadt hem hier.
airing-marked = { $title } is als niet interessant gemarkeerd.
airing-undo = Ongedaan
airing-plan = Later kijken
airing-get-more = Meer afl. ophalen
airing-get = Afl. ophalen
airing-not-interested = Niet interessant

## Seizoenen

season-winter = Winter
season-spring = Lente
season-summer = Zomer
season-fall = Herfst
season-named = { $season } { $year }
season-pick = Kies een seizoen

## De pagina van een serie

series-back-home = Start
series-back-library = Bibliotheek
series-back-downloads = Downloads
series-back-calendar = Agenda
series-get-episodes = Afl. ophalen
series-episodes = Afleveringen
series-delete-episodes = Afl. wissen
series-delete-tap = Tik op de afleveringen om te wissen.
series-delete-picked = { $count ->
    [one] { $count } aflevering gekozen.
   *[other] { $count } afleveringen gekozen.
}
series-delete-note = De bestanden worden van deze telefoon gewist. Wat je keek blijft bijgehouden.
series-delete-button = { $count ->
    [one] { $count } aflevering wissen
   *[other] { $count } afleveringen wissen
}
series-no-episodes = Geen afleveringen op deze telefoon en er staan er geen gepland.
series-versions = { $count ->
    [one] { $count } versie
   *[other] { $count } versies
}
series-note-next-up = Hierna
series-note-missing = Ontbreekt
series-note-not-here = Niet op deze telefoon
series-note-not-downloaded = Niet gedownload
series-note-downloading = Downloaden { $percent }%
series-note-resume-at = Hervat bij { $time }
phone-episodes-estimate = { $count ->
    [one] { $count }? aflevering
   *[other] { $count }? afleveringen
}
phone-next-airs = Aflevering { $n } over { $time }
phone-watched = Gekeken
phone-not-watched = Nog niet gekeken
phone-progress = { $watched } van { $total } gekeken, { $count } op deze telefoon
phone-on-this-phone = { $count } op deze telefoon
phone-play-film = Film afspelen
phone-resume-film = Film hervatten
phone-play-episode = Speel aflevering { $n }
phone-resume-episode = Hervat aflevering { $n }
phone-about-date = rond { $date }

## Lijststatus, score, notities en correcties van een serie

show-status-watching = Kijken
show-status-planning = Gepland
show-status-completed = Voltooid
show-status-paused = Gepauzeerd
show-status-dropped = Gestopt
show-status-repeating = Herkijken
show-status-now = { $label }, nu
show-add-to-list = Op lijst zetten
show-your-score = Jouw score { $score }
show-rate = Score
show-fixes = Herstel
show-your-notes = Je notities
show-add-notes = Notitie maken
show-home-each = Wijzig het op elke gekoppelde lijst.
show-home-no-tracker = Bewaard op deze telefoon: geen tracker heeft deze serie.
show-home-tracker-lacks = Bewaard op deze telefoon: je tracker heeft deze serie niet.
show-home-until-connected = Bewaard op deze telefoon tot je AniList of MyAnimeList koppelt.
show-sites-both = AniList en MyAnimeList
show-notes-title = Notities
show-notes-placeholder = Waar je bleef, wie het aanraadde, welke release je moet halen
show-notes-over = { $count ->
    [one] { $count } teken boven de limiet
   *[other] { $count } tekens boven de limiet
}
show-notes-saved-to = Opgeslagen bij { $sites }.
show-notes-saved-to-tracker = Opgeslagen bij je tracker.
show-notes-saving = Notities opslaan.
show-notes-clearing = Je notities worden gewist.
show-score-saving = Score opslaan.
show-score-saved = Score bewaard.
show-score-clearing = Je score wordt gewist.
show-score-cleared = Score gewist.
show-moving-to = Zetten op { $label }.
show-moved-to = Gezet op { $label }.
show-rescan-started = Scan gestart.
show-still-starting = AniBeam is nog aan het starten. Probeer het zo opnieuw.
show-fix-match = Koppeling
show-fix-find-match = Koppeling zoeken
show-fix-rescan = Opnieuw scannen

## Personages, gerelateerde series en aanbevelingen van een serie

show-characters = Personages
show-related = Verwant
show-related-all = Alle { $count } tonen
show-related-filling = Wordt nog aangevuld vanuit AniList.
show-recommendations = Aanbevelingen
show-card-owned = In je bibliotheek
show-card-unreleased = Nog niet uitgekomen
show-card-on-anilist = Op AniList
show-rec-available = Beschikbaar
show-relation-prequel = Voorloper
show-relation-sequel = Vervolg
show-relation-side-story = Zijverhaal
show-relation-parent = Hoofdserie
show-relation-spin-off = Spin-off
show-relation-adaptation = Bewerking
show-relation-alternative = Alternatief
show-relation-summary = Recap
show-relation-character = Personage
show-relation-source = Bron
show-relation-compilation = Compilatie
show-relation-contains = Bevat
show-relation-other = Overig
show-role-main = Hoofd
show-role-supporting = Bijrol
show-role-background = Achtergrond
show-format-tv = Tv
show-format-tv-short = Korte tv
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Special
show-format-film = Film
show-entry-get = Afl. ophalen
show-entry-open-anilist = Openen op AniList
show-entry-line = { $line }, { $status }

## Een verkeerde koppeling herstellen

match-title = Koppeling
match-refresh = Info vernieuwen
match-clear = Ontkoppelen
match-clear-it = Wissen
match-clear-warning = Wissen haalt de trackerkoppeling en een met de hand ingesteld aantal afleveringen weg. Tik nogmaals om te wissen.
match-tvmaze = TVmaze
match-search-anilist = Zoek op AniList
match-search-tvmaze = Zoek op TVmaze
match-paste-anilist = Plak een AniList- of MyAnimeList-link
match-paste-tvmaze = Plak een TVmaze- of IMDb-link
match-look-up = Opzoeken
match-apply = Toepassen
match-look-up-all = Zoek elke map op die AniBeam nog niet kent
match-format-music = Muziek
match-join-and = { $first } en { $second }
match-matched-on = Gekoppeld via { $sites }
match-found-tvmaze = Serie gevonden op TVmaze
match-found-anilist = Gevonden op AniList
match-found-anilist-season = Gevonden op AniList, seizoen { $season }
match-found-mal = Gevonden op MyAnimeList
match-searching = { $name } doorzoeken
match-type-more = Typ minstens twee letters om { $name } te doorzoeken.
match-none = Geen titels gevonden op { $name }.
match-looking-up = Opzoeken.
match-matched-to = Gekoppeld aan { $title }.
match-applied = Koppeling gezet.
match-applying = Koppeling toepassen.
match-refreshing = { $title } vernieuwen.
match-clearing = Koppeling wissen.
match-cleared = Koppeling gewist.
match-auto-looking = De mappen opzoeken die AniBeam nog niet kent.

## Batches (de regel onder een release in een zoekopdracht)

inspect-reading = Bestandslijst lezen
inspect-holds = { $count ->
    [one] { $count } aflevering erin: { $runs }
   *[other] { $count } afleveringen erin: { $runs }
}

## Wat is nieuw

notes-title = Wat is nieuw
notes-got-it = Begrepen

## De lijst naar een tracker verplaatsen

move-lists-label = Lijsten op deze telefoon
move-title = Verplaats je lijst naar { $site }
move-here = Op deze telefoon: { $list }
move-there = Nu op { $site }: { $list }
move-later-hint = Later: Meer, Accounts
move-not-now = Niet nu
move-nothing = Niets
move-not-on-list = Niet op je lijst
move-nothing-left = Niets meer te verplaatsen
move-side-status-score = { $status }, score { $score }
move-side-score = score { $score }
move-lead = { $count ->
    [one] Eén serie heeft een status of score alleen op deze telefoon. Verplaatsen zet hem op je { $site }-lijst, waar je computer en je profiel hem zien. Voor deze series wordt vervangen wat { $site } nu heeft. Er wordt niets verstuurd tot je op Verplaatsen drukt.
   *[other] { $count } series hebben een status of score alleen op deze telefoon. Verplaatsen zet ze op je { $site }-lijst, waar je computer en je profiel ze zien. Voor deze series wordt vervangen wat { $site } nu heeft. Er wordt niets verstuurd tot je op Verplaatsen drukt.
}
move-label = { $count ->
    [one] Verplaats { $count } serie
   *[other] Verplaats { $count } series
}
move-waiting = { $count ->
    [one] Eén serie heeft een status of score alleen hier. Verplaats hem naar { $site } om hem op je computer en je profiel te zien.
   *[other] { $count } series hebben een status of score alleen hier. Verplaats ze naar { $site } om ze op je computer en je profiel te zien.
}
move-moving = { $count ->
    [one] { $count } serie naar { $site } verplaatsen
   *[other] { $count } series naar { $site } verplaatsen
}
move-moved = { $count ->
    [one] { $count } serie naar { $site } verplaatst
   *[other] { $count } series naar { $site } verplaatst
}
move-failed = { $count ->
    [one] { $site } nam { $count } serie niet aan; zie het activiteitenlogboek
   *[other] { $site } nam { $count } series niet aan; zie het activiteitenlogboek
}

## Downloads: zoeken (zoekopdracht, series, bronnen, releases)

find-join-comma = { $a }, { $b }
find-join-and = { $a } en { $b }
find-join-sentences = { $a } { $b }

find-downloads = Downloads

find-lens-list = Je lijst
find-lens-gaps = Gaten
find-lens-airing = Lopend
find-lens-following = Gevolgd
find-lens-downloaded = Gedownload
find-lens-with-count = { $lens } { $count }
find-what-to-get = Wat te halen
find-lens-list-line = Kijken en Gepland op je lijst. Open er een om de afleveringen op te halen.
find-lens-gaps-line = Uitgezonden afleveringen die niet op deze telefoon staan. Open een serie om te zien wie ze heeft.
find-lens-airing-line = Nieuwe afleveringen die nog komen, de eerste het eerst.
find-lens-following-line = De abonnementen: nieuwe afleveringen downloaden vanzelf. Open er een om te stoppen.
find-lens-downloaded-line = Wat deze telefoon downloadde, per serie.

find-lens-gaps-empty = Geen gaten: van elke serie die je begon staan de uitgezonden afleveringen hier.
find-lens-airing-empty = Niets op deze telefoon loopt nog.
find-lens-following-empty = Nog geen abonnementen. Open een serie en zet Volgen aan.
find-lens-downloaded-empty = Nog niets gedownload.
find-list-empty = Nog niets op je lijst. Zoek hierboven een serie en voeg hem toe.
find-list-reading = Je lijst lezen...
find-list-signed-out = Log in bij AniList onder Meer, Accounts, en je lijst verschijnt hier.
find-list-failed = Je lijst kon niet worden gelezen: { $error }

find-aired = uit
find-ledger-no-feed = geen nieuwe afleveringen: de bron heeft geen feed
find-ledger-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
find-ledger-downloaded = gedownload
find-ledger-planning = Gepland
find-ledger-watching-of = Kijken, { $progress } van { $total }
find-ledger-watching-seen = Kijken, { $progress } gezien

find-gap-missing = { $count } ontbreken
find-gap-not-downloaded = { $count } niet gedownload

find-list-status-planning = Op je lijst: Gepland
find-list-status-watching = Op je lijst: Kijken
find-list-status-already = Staat al op je lijst
find-going-planning = { $title } wordt op Gepland gezet.
find-already-listed = { $title } staat al op je lijst.
find-plan-needs-tracker = Toevoegen aan je lijst vraagt om AniList of MAL. Log in onder Meer, Accounts.
find-plan-failed = Niet aan je lijst toegevoegd: { $error }
find-show-gone = Die serie staat niet meer in de bibliotheek op deze telefoon.

## Series tijdens het typen

find-shows = Series
find-suggesting = AniList vragen...
find-suggest-tap = Tik op een serie om te zien wie hem heeft. Enter doorzoekt releasenamen.
find-suggest-none = Geen serie met die naam. Enter doorzoekt in plaats daarvan releasenamen.
find-suggest-hint = Series verschijnen terwijl je typt. Enter doorzoekt releasenamen.
find-suggest-failed = AniList kon niet worden bevraagd: { $error }
find-library-line = In je bibliotheek
find-library-line-none = In je bibliotheek, geen van { $total } op deze telefoon
find-library-line-some = In je bibliotheek, { $here } van { $total } op deze telefoon

## De serieweergave

find-plan-to-watch = Later kijken
find-on-phone-count = { $count } op deze telefoon
find-airing = Lopend
find-dropped = { $count ->
    [one] { $count } release die deze serie niet noemt is weggelaten.
   *[other] { $count } releases die deze serie niet noemen zijn weggelaten.
}

find-following = Gevolgd
find-stop-following-ask = { $name } niet meer volgen?
find-unfollow-explain = Nieuwe afleveringen downloaden niet meer. Wat in de map staat kan blijven, of mee weg: elke download daar, ook afleveringen die je zelf haalde.
find-stop-following = Niet meer volgen
find-stop-keep = Stoppen, afleveringen houden
find-stop-delete = Stoppen en downloads in de map wissen
find-rule-taken = { $downloads ->
    [one] { $downloads } download
   *[other] { $downloads } downloads
}, { $files ->
    [one] { $files } bestand
   *[other] { $files } bestanden
} in de map
find-rule-paused = Gepauzeerd, dus er komen geen nieuwe afleveringen. { $taken }.
find-rule-no-feed = De bron heeft geen feed, dus er komen geen nieuwe afleveringen. { $taken }.
find-rule-active = Nieuwe afleveringen downloaden vanzelf. { $taken }.
find-unfollowed-deleted = { $name } niet meer gevolgd en de downloads in de map gewist.
find-unfollowed-kept = { $name } niet meer gevolgd. De afleveringen blijven op deze telefoon.
find-stop-failed = Stoppen is niet afgerond: { $error }

find-where = Waar te halen
find-where-pick = Kies een bron om te zien wat hij heeft.
find-where-nothing = Er wordt niets gedownload tot jij het zegt.
find-source-idle = Tik om te zien
find-source-trying = { $name } proberen
find-source-searching = Zoeken
find-source-silent = Geen antwoord
find-no-sites-title = Nog geen zoeksites
find-no-sites-body = Voeg onder Meer een repo met zoeksites toe, of een eigen feed.
find-open-more = Meer openen
find-pick-source-title = Kies een bron
find-pick-source-body = De releases ervan verschijnen hier als raster, een aflevering per blokje.
find-releases = { $count ->
    [one] { $count } release
   *[other] { $count } releases
}
find-unreachable = { $host } kon niet worden bereikt. Sommige netwerken en landen blokkeren releasesites; AniBeam komt al langs blokkades op naam, dus deze blokkeert de site zelf, of de site is offline. Een vpn komt erlangs, of probeer een andere bron.
find-the-source = De bron

find-has = Wat hij heeft
find-has-note = Tik op een rij om alles te nemen, of losse afleveringen.
find-episodes-tag = Afleveringen
find-batch = Batch { $label }
find-row-meta = { common-episodes }  { $seeds } seeds  { $size }
find-on-this-phone = Op deze telefoon
find-on-computer = Op { $name }

find-plan-no-releases = Deze bron heeft geen releases van deze serie.
find-plan-nothing-picked = Niets gekozen: tik op een rij of de blokjes.
find-plan-line = { $episodes } van { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] { $count } aflevering die je al hebt is weggelaten. Zet Vervangen aan om haar opnieuw te halen.
   *[other] Afleveringen die je al hebt zijn weggelaten. Zet Vervangen aan om ze opnieuw te halen.
}
find-plan-follow-note = Nieuwe afleveringen van { $query } downloaden vanzelf.
find-plan-on-desk = Ze downloaden op { $name }, in de bibliotheek ervan.
find-follow-new = Nieuwe afl. volgen
find-follow-new-note = Ze downloaden vanzelf zodra ze uitkomen.
find-replace = Vervang wat je hebt
find-replace-note = Downloadt de gekozen afleveringen opnieuw over die op deze telefoon.
find-go = { $mode ->
    [replace] { common-episodes } ophalen en vervangen
    [subscribe] { common-episodes } ophalen en abonneren
    [both] { common-episodes } ophalen, vervangen en abonneren
   *[plain] { common-episodes } ophalen
}
find-go-pick = Kies afleveringen
find-every-release = Elke release van deze bron

## Downloaden naar

find-download-to = Downloaden naar
find-this-phone = Deze telefoon
find-your-computer = Je computer
find-desk-asking = { $name } vragen...
find-desk-silent = { $name } antwoordt niet. Kies Deze telefoon, of probeer opnieuw als hij aan staat.
find-sending-to = Versturen naar { $name }...

## De releaselijst

find-best-fit = Beste match
find-on-disk = Op schijf
find-get = Ophalen
find-get-all = Alle { $count } ophalen
find-subscribe = Abonneren
find-back-step = Een stap terug
find-narrow-placeholder = Verfijn, of -woord om uit te sluiten
find-best-fit-searches = Beste match zoekt: { $query }
find-fact-batch = batch
find-fact-seeds = { $count } seeds
find-fact-via = via { $source }
find-searching-for = Zoeken naar { $text }...
find-search-line = { $count ->
    [0] Niets op { $source } hiervoor
    [one] { $count } release op { $source }
   *[other] { $count } releases op { $source }
}{ $fell ->
    [1] , nadat de eerste bron niets had
   *[other] { "" }
}{ $narrowed ->
    [1] , verfijnd tot { $token }
   *[other] { "" }
}.
find-refined = { find-releases }, { $delta } { $dir ->
    [down] minder
   *[up] meer
}.
find-refine-nothing-left = Dan blijft er niets over.
find-refine-no-match = Geen release heeft dat.
find-refine-empty = Typ eerst iets.
find-refine-no-change = Dat verandert niets.

## Ophalen en abonneren

find-getting = { $what } ophalen.
find-subscribed = Geabonneerd: nieuwe afleveringen downloaden vanzelf.
find-subscribed-desk = Geabonneerd op { $name }: nieuwe afleveringen downloaden daar vanzelf.
find-subscribed-as = Geabonneerd als { $name }: nieuwe afleveringen downloaden vanzelf.
find-downloading = { $count ->
    [one] { $count } release downloaden.
   *[other] { $count } releases downloaden.
}
find-not-added = Niet toegevoegd: { $error }
find-not-subscribed = Niet geabonneerd: { $error }
find-desk-refused = { $name } nam het niet aan: { $error }
find-desk-not-subscribed = Niet geabonneerd op { $name }: { $error }

## Speler

player-subtitles = Ondertitels
player-audio = Audio
player-speed = Snelheid
player-connecting = Verbinden
player-hold-speed = 2× snelheid
player-replay = Opnieuw
player-unlock = Ontgrendel
player-next-stay = Blijf
player-next-go = Volgende
player-jump-back = −10 s
player-jump-forward = +10 s
player-speed-now = Snelheid { $speed }
player-skip-intro = Sla intro over
player-skip-outro = Sla outro over
player-skipped-intro = Intro overgeslagen
player-skipped-outro = Outro overgeslagen
player-undo = Ongedaan
player-start-over = Van het begin
player-resumed-at = Hervat bij { $time }
player-picture-failed = Het beeld kon niet starten. Het geluid speelt door.
player-file-failed = Dit bestand speelde niet af: { $why }.
player-streaming-from = Streamen van { $host } in { $height }p
player-mpv-no-start = mpv is niet gestart
player-mpv-no-start-why = mpv is niet gestart: { $why }
player-android-only = de speler draait alleen op Android

## Speler: vertraging

player-delay-title = Timing
player-delay-note = Elke tik verschuift 0,1 s. Tot het volgende bestand.
player-delay-seconds = { $value } s
player-sub-delay-now = Ondertitelvertraging { $value }
player-audio-delay-now = Audiovertraging { $value }

## Speler: casten

player-cast-playing-on = Speelt af op { $device }
player-cast-stop = Stop met casten naar { $device }
player-cast-searching = Chromecasts en tv's zoeken
player-cast-to = Casten naar
player-cast-tv = Tv
player-cast-loading = Laden
player-cast-playing = Speelt af
player-cast-paused = Gepauzeerd
player-cast-finished = Afgelopen
player-cast-stopped = Gestopt

## Speler: het woord van de tracker

player-rated = Gescoord
player-score-failed = Score opslaan mislukt
player-marked-watched = Als gekeken
player-tracked = Bijgewerkt
player-not-tracked = Niet bijgewerkt
player-track-update-failed = Je lijst bijwerken mislukt
player-not-tracked-why = Niet bijgewerkt: { $reason }.
player-track-ok-at = { $tracker } bij { $progress }
player-track-ok = { $tracker } ok
player-track-note = { $tracker }: { $note }
player-track-failed = { $tracker } kon niet worden bijgewerkt
player-track-line = { $head }: { $parts }
player-refusal-hidden = de serie is verborgen
player-refusal-no-match = de serie heeft geen trackerkoppeling
player-refusal-not-newer = de tracker is al voorbij deze aflevering
player-refusal-extra = dit is een extra
player-refusal-unmatched = het bestand is niet aan een aflevering gekoppeld
player-refusal-on-disk = het bestand is alleen op schijf

## Ondertitels

subs-line-default = Grootte 1×, gewone bestanden nemen je stijl
subs-sample = Waar heb je het verstopt?
subs-size-place-title = Grootte en plek
subs-size = Grootte
subs-position = Positie
subs-position-helper = 100 zit op de onderrand. Lagere getallen tillen de regels hoger in het beeld.
subs-style-title = Stijl
subs-styled = Opgemaakte tekst
subs-styled-as-made = Opgemaakte (ASS) bestanden zien eruit zoals hun makers ze instelden.
subs-styled-mine = Je stijl hieronder geldt voor elk bestand. Bordjes en karaoke kunnen stuk gaan.
subs-styled-size-only = Opgemaakte ondertitelbestanden houden hun uiterlijk en nemen je grootte. Gewone bestanden nemen je stijl.
subs-mode-as-made = Origineel
subs-mode-size-only = Alleen maat
subs-mode-mine = Mijn
subs-font = Font
subs-font-bad = Typ een lettertypenaam, zoals Noto Sans.
subs-bold = Vet
subs-colour = Tekstkleur
subs-colour-bad = Typ een kleur als #ffcc00.
subs-outline = Omlijning
subs-outline-bad-width = De breedte is een getal, 0 of meer.
subs-outline-bad-colour = Typ de kleur als #000000.
subs-outline-helper = Eerst de breedte, dan de kleur.
subs-shadow = Schaduw
subs-shadow-bad = Een getal, 0 of meer. 0 betekent geen schaduw.
subs-shadow-helper = Hoe ver de schaduw valt, 0 voor geen.
subs-box = Achtergrondvak
subs-box-helper = Een donker vak achter de regels. 0 laat het weg.
subs-reset-title = Terugzetten
subs-reset-helper = Zet grootte, lettertype, kleuren, omlijning, vak en positie terug zoals ze waren.
subs-reset = Herstel

## Streamen

stream-quality-auto = Auto
stream-quality-original = Origineel
stream-app-starting = De app is nog aan het starten
stream-off-on-host = Streamen staat uit op { $host }. Zet daar Afleveringen streamen aan, onder Instellingen, Telefoon-app.
stream-cannot-stream = { $host } draait AniBeam { $version }, dat nog niet kan streamen. Werk het daar bij.
stream-no-session = Het andere apparaat gaf geen sessie
stream-answer-unreadable = Het antwoord was onleesbaar: { $why }
stream-answered-status = { $name } antwoordde { $status }

## Statistieken

stats-nothing-yet = Nog niets
stats-empty-title = Nog niets gekeken
stats-empty-body = Kijk een aflevering uit en hij telt hier mee: uren, genres en de seizoenen die je keek.
stats-hours-caption = uur gekeken
stats-finished-caption = series uitgekeken
stats-busiest-caption = drukste maand
stats-episodes-caption = { $count ->
    [one] aflevering
   *[other] afleveringen
}
stats-by-month = Per maand
stats-top-genres = Top genres
stats-top-studios = Top studio's
stats-seasons = Gekeken seizoenen
stats-watched-most = Meest gekeken
stats-year-line = Wat je keek in { $year }
stats-eps = { $count ->
    [one] { $count } afl.
   *[other] { $count } afl.
}
stats-month-1 = jan
stats-month-2 = feb
stats-month-3 = mrt
stats-month-4 = apr
stats-month-5 = mei
stats-month-6 = jun
stats-month-7 = jul
stats-month-8 = aug
stats-month-9 = sep
stats-month-10 = okt
stats-month-11 = nov
stats-month-12 = dec
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

## Back-up en afbeeldingscache

backup-title = Back-up
backup-export = Bibliotheek back-uppen
backup-export-helper = Eén bestand met je mappen, koppelingen en wat je keek.
backup-private = Privégegevens meenemen
backup-private-helper = Voegt AniList- en MyAnimeList-logins, kijkgeschiedenis en voorkeuren toe. Logins staan als leesbare tekst, dus houd het bestand voor jezelf.
backup-restore = Herstellen uit een back-up
backup-restore-helper = Leest een back-up in deze bibliotheek. Wat op schijf staat wint; het bestand vult de rest aan.
backup-done = Back-up gemaakt. Het bestand staat waar je het opsloeg.
backup-not-saved = Niet opgeslagen. Er is niets geschreven.
backup-starting = AniBeam is nog aan het starten. Probeer het zo opnieuw.
backup-images-title = Afbeeldingen
backup-images-body = Posters, banners en portretten blijven op de telefoon zodat de bibliotheek offline werkt. Gewiste afbeeldingen worden opnieuw gedownload als pagina's ze nodig hebben.
backup-keep = Houden
backup-removed = { $count ->
    [one] { $count } afbeelding gewist
   *[other] { $count } afbeeldingen gewist
}
backup-images-none = Cache is leeg
backup-images-unmeasured = De afbeeldingscache kon niet worden gemeten
backup-images-retry-note = Probeer het zo opnieuw. De afbeeldingen zelf zijn in orde.
backup-images-line = { $size } in { $count ->
    [one] { $count } afbeelding
   *[other] { $count } afbeeldingen
}
backup-images-question = { $count ->
    [one] { $count } afbeelding wissen?
   *[other] { $count } afbeeldingen wissen?
}

backup-sum-list = { $items }.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] { $count } map toegevoegd
   *[other] { $count } mappen toegevoegd
}
backup-sum-series = { $count } series aangemaakt
backup-sum-matches = { $count ->
    [one] { $count } koppeling toegepast
   *[other] { $count } koppelingen toegepast
}
backup-sum-views = { $views ->
    [one] { $views } kijkbeurt
   *[other] { $views } kijkbeurten
} en { $episodes ->
    [one] { $episodes } uitgekeken aflevering
   *[other] { $episodes } uitgekeken afleveringen
} samengevoegd
backup-sum-resume = { $count ->
    [one] { $count } hervatpunt samengevoegd
   *[other] { $count } hervatpunten samengevoegd
}
backup-sum-accounts = { $count ->
    [one] { $count } lijstaccount overgenomen
   *[other] { $count } lijstaccounts overgenomen
}
backup-sum-fixes = { $count ->
    [one] { $count } bestand neemt een correctie van jou over.
   *[other] { $count } bestanden nemen een correctie van jou over.
}
backup-sum-missing = { $count ->
    [one] { $count } map is niet op deze telefoon gevonden.
   *[other] { $count } mappen zijn niet op deze telefoon gevonden.
}
backup-sum-left-out = Weggelaten: { $fields }.

## Activiteit

activity-everything = Alles
activity-problems = Problemen
activity-copied = Gekopieerd
activity-clear-every-line = Elke regel wissen
activity-fold = Klap in
activity-no-problems = Geen problemen. Alles werkte.
activity-nothing-yet = Nog niets te melden. Wat AniBeam voor je doet verschijnt hier.
activity-time-today = %H:%M:%S
activity-time-other = %-d %b %H:%M

## Datums en getallen (companion format.rs)

fmt-clock = %H:%M
fmt-day-month = %-d %b
fmt-weekday-date-time = %a %-d %b %H:%M
fmt-weekday-date-year-time = %a %-d %b %Y %H:%M
fmt-stamp = %-d %b %H:%M
fmt-decimal-point = ,

fmt-today-at = Vandaag { $time }
fmt-tomorrow-at = Morgen { $time }
fmt-yesterday-at = Gisteren { $time }

fmt-span-days = { $d } d { $h } u
fmt-span-hours = { $h } u { $m } min
fmt-span-minutes = { $m } min { $s } s
fmt-span-seconds = { $s } s
fmt-in = over { $span }
fmt-full-days = { $d } d { $h } u { $m } min { $s } s
fmt-full-hours = { $h } u { $m } min { $s } s
fmt-full-minutes = { $m } min { $s } s

fmt-ago-now = zojuist
fmt-ago-minutes = { $count } min geleden
fmt-ago-hours = { $count } u geleden
fmt-ago-days = { $count } d geleden

fmt-bytes-b = { $value } B
fmt-bytes-kb = { $value } kB
fmt-bytes-mb = { $value } MB
fmt-bytes-gb = { $value } GB
fmt-bytes-tb = { $value } TB

## Praten met de computer (companion client.rs, sync.rs, address.rs)

client-not-found = { $host } is geen naam die de telefoon kan vinden. Is Tailscale verbonden?
client-timeout = { $host } antwoordde niet. Staat hij aan, en is Tailscale aan beide kanten verbonden?
client-connect-failed = Kon { $host } niet bereiken. Staat hij aan, en is Tailscale aan beide kanten verbonden?
client-refused = { $host } staat aan, maar AniBeam daar is gesloten of deelt niet. Open AniBeam erop en zet Delen via Tailscale aan onder Instellingen, Telefoon-app.
client-no-route = Geen route naar { $host }. Is Tailscale verbonden?
client-invalid-input = Kon geen verbinding maken met { $host }
client-unreachable = Kon { $host } niet bereiken: { $why }
client-cannot-find = De telefoon kan { $host } niet vinden. Is Tailscale verbonden?
client-broke-off = Het antwoord van { $host } brak af: { $why }
client-answered = { $host } antwoordde { $status } voor { $path }
client-not-anibeam-status = Iets antwoordde, maar het is geen AniBeam ({ $host } antwoordde { $status } voor { $path })
client-not-anibeam-hello = Iets antwoordde, maar het is geen AniBeam (het antwoord is niet de hello van AniBeam)
client-not-anibeam-name = Iets antwoordde, maar het is geen AniBeam (het noemt zichzelf { $name })
client-too-new = De host spreekt API { $host_api }; deze app leest { $app_api }. Werk de app bij.
client-snapshot-unreadable = De momentopname was onleesbaar: { $why }
client-probe-unreadable = De bestandsdetails waren onleesbaar: { $why }
client-peers-unreadable = De lijst met AniBeams was onleesbaar: { $why }
sync-save-failed = Kon de momentopname niet opslaan: { $why }
address-https = AniBeam op je computer antwoordt via Tailscale op gewoon http; laat https:// weg.
address-empty = Typ de naam of het adres van de host, bijvoorbeeld my-computer:8790.
address-ipv6-close = Een IPv6-adres heeft een afsluitende ] nodig.
address-ipv6-port = Zet de poort na de ], zoals [fd7a::1]:8790.
address-bad-host = "{ $text }" is geen hostnaam.
address-bad-port = "{ $port }" is geen poort.

## Seriegegevens (companion library.rs)

clib-status-airing = Lopend
clib-status-not-yet = Nog niet gestart
clib-format-tv-short = Korte tv
clib-format-movie = Film
clib-format-special = Special
clib-format-music = Muziek

## Zoeksites en repos

repos-by-hand-title = Handmatig toevoegen
repos-by-hand-note = Voor een site die geen repo heeft. Elke feed met { "{query}" } waar de zoekterm komt, ook een Jackett- of Prowlarr-feed. Een sleutel of token in het adres wordt naar de geheimenopslag van deze telefoon verplaatst.
repos-name-placeholder = Naam
repos-url-placeholder = https://...{ "{query}" }...
repos-index-placeholder = https://.../index.json
repos-add-site = Site erbij
repos-add-title = Repo toevoegen
repos-add-note = Plak het adres van de index van een repo. AniBeam controleert de handtekening en toont je de sleutel voordat iets wordt opgeslagen.
repos-add-repo = Repo erbij
repos-trust-title = Deze repo vertrouwen?
repos-fingerprint = Vingerafdruk
repos-trust-warning = De sites bepalen waar je zoekopdrachten heen gaan. Vertrouw alleen een repo van iemand die je vertrouwt.
repos-trust = Vertrouw
repos-open-link = { $url } openen
repos-saving = Opslaan...
repos-install = Installeren
repos-update = Bijwerken
repos-edit-note = Een sleutel of token in het adres wordt naar de geheimenopslag van deze telefoon verplaatst.
repos-edit-note-keys = Laat elke { "{key}" } in het adres staan; de waarde blijft op deze telefoon.
repos-change-its = { $keys } wijzigen
repos-check-repos = Repos controleren op nieuwe sites
repos-check-sites = Sites controleren
repos-add-site-label = Site handmatig toevoegen
repos-needs-before-search = Heeft { $labels } nodig voor het zoekt
repos-no-longer-offered = Wordt niet meer aangeboden door de repo
repos-update-host = De update stuurt verzoeken naar { $host }
repos-tap-change-custom = Tik om de naam of het adres te wijzigen
repos-tap-change-keys = Tik om { $labels } te wijzigen
repos-line-custom = { $host }  { $kind }  door jou toegevoegd
repos-update-to = Bijwerken naar v{ $version }
repos-add-its-value = Waarde toevoegen
repos-asks-for = Vraagt om je { $labels }
repos-preview-line = { $count ->
    [one] { $count } site, ondertekend
   *[other] { $count } sites, ondertekend
}
repos-preview-trusted = { $count ->
    [one] { $count } site, ondertekend, al vertrouwd
   *[other] { $count } sites, ondertekend, al vertrouwd
}
repos-ask-kept-empty = Alleen bewaard op deze telefoon. Een leeg veld behoudt wat is opgeslagen.
repos-ask-kept-never = Alleen bewaard op deze telefoon, nooit in de bibliotheek of een export.
repos-ask-new-host = De nieuwe versie stuurt dit naar { $host }, dus hij vraagt het opnieuw.
repos-ask-new = De nieuwe versie vraagt hier opnieuw om.
repos-type-value = Typ eerst een waarde.
repos-health-answered = Antwoord in { $ms } ms
repos-health-failed = Geen antwoord. { $step }: { $detail }

## Downloads

dl-title = Downloads
dl-wifi-only = Alleen Wi-Fi
dl-wifi-and-mobile = Wi-Fi en mobiele data
dl-joined = { $first }, { $second }
dl-batch-files = { $count ->
    [one] Batch, { $count } bestand
   *[other] Batch, { $count } bestanden
}
dl-batch-tag = Batch
dl-downloading-count = { $count ->
    [one] { $count } downloadt
   *[other] { $count } downloaden
}
dl-free-on-phone = vrij op de telefoon
dl-search-placeholder = Een serie of releasenaam
dl-sources = Bronnen
dl-downloading = Downloaden
dl-queue-note = { $count ->
    [one] { $count } aflevering
   *[other] { $count } downloads
}
dl-finished = Klaar
dl-finished-note = { $count ->
    [one] { $count } download, in je bibliotheek
   *[other] { $count } downloads, in je bibliotheek
}
dl-get = Ophalen
dl-pick = Kies

## Opslag en verborgen series

disk-on-phone = Op deze telefoon
disk-watched = Gekeken
disk-not-watched = Niet gekeken
disk-deleting = Wissen
disk-delete-watched = Gekeken wissen
disk-hidden-shows = Verborgen series
disk-hidden-helper = Weg van Start en de bibliotheek tot je ze terughaalt.
disk-unhide = Tonen
disk-delete-note = Bestanden en ondertitels worden definitief van deze telefoon gewist.
disk-every-watched = Elke gekeken aflevering
disk-every-finished = Elke uitgekeken serie die je keek
disk-episodes-size = { $count ->
    [one] { $count } aflevering, { $size }
   *[other] { $count } afleveringen, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } serie, { $size }
   *[other] { $count } series, { $size }
}
disk-list-two = { $a } en { $b }
disk-list-three = { $a }, { $b } en { $c }
disk-list-more = { $a }, { $b }, { $c } en { $count } meer
disk-delete-finished-title = { $count ->
    [one] { $count } uitgekeken serie
   *[other] { $count } uitgekeken series
} wissen, { $size }?
disk-delete-watched-title = { $count ->
    [one] { $count } gekeken aflevering
   *[other] { $count } gekeken afleveringen
} wissen, { $size }?
disk-delete-finished-body = { $names }. Alleen series waarvan de provider zegt dat ze zijn afgelopen, elke aflevering gekeken; een serie die nog loopt, of waarvan het einde een gok is, blijft. Elk bestand ervan wordt van deze telefoon gewist, extra's ook. Hun downloads verdwijnen ook, dus ze kunnen opnieuw worden gedownload.
disk-delete-watched-body = Elke aflevering die je keek, in elke serie. Nog niet gekeken afleveringen en extra's blijven. De bestanden en hun ondertitels worden definitief van deze telefoon gewist.
disk-no-finished = Geen enkele uitgekeken serie is helemaal gekeken
disk-nothing-watched = Niets gekeken staat op deze telefoon
disk-deleted = { $count ->
    [one] { $count } aflevering
   *[other] { $count } afleveringen
} gewist, { $size }
disk-delete-failed = Er is niets gewist: { $error }

## Downloadmeldingen

notify-card-head = AniBeam · nu
notify-sample-called = Frieren aflevering 7
notify-sample-called-ready = Frieren aflevering 6
notify-sample-facts = 420 MB van 1,4 GB, 3,2 MB/s, nog 5 min
notify-downloading = { $called } downloaden
notify-progress = { $percent }%, { $facts }
notify-progress-of = { $percent }%, { $done } van { $size }
notify-starting = Starten
notify-many = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
notify-called-episode = { $show } aflevering { $n }
notify-called-episodes = { $show } aflevering { $from } t/m { $to }
notify-ready = { $called } is klaar
notify-ready-many = { $called } zijn klaar
notify-ready-batch = Een batch van { $show } is klaar
notify-ready-text = Gedownload naar deze telefoon. Tik om AniBeam te openen.
notify-channel-downloads = Downloads
notify-channel-ready = Afleveringen klaar
notify-fallback-title = Downloaden
notify-row-on = Meldingen
notify-row-allow = Meldingen toestaan
notify-row-open-settings = Meldingsinstellingen openen
notify-helper-on = Aan. Downloads staan in je meldingen zolang ze lopen.
notify-helper-asks-next = Android vraagt hierna.
notify-helper-asks-next-later = Android vraagt het hierna. Je kunt later van gedachten veranderen onder Meer.
notify-helper-off = Uit. Zet daar meldingen voor AniBeam aan.
notify-question-title = Downloads in je meldingen zien?
notify-question-body = Tijdens het downloaden toont een melding de voortgang, en een andere meldt wanneer een aflevering klaar is. Ze downloaden hoe dan ook; dit bepaalt alleen of je het ziet.
notify-allow = Toestaan
notify-not-now = Niet nu
notify-pointer = Downloads staan niet in je meldingen.
notify-turn-on = Aanzetten

## Feed

feed-title = Feed
feed-recently-released = Recent uitgekomen
feed-coming-soon = Binnenkort
feed-this-week = Deze week
feed-earlier = Eerder
feed-everything-else = Al het andere
feed-countdown-days = { $d } d { $h } u
feed-countdown-hours = { $h } u { $m } min
feed-countdown-minutes = { $m } min
feed-next = Hierna: { $title } over { $countdown }
feed-nothing-scheduled = Nog niets gepland.
feed-nothing-new = Niets nieuws deze week.
feed-new-episodes = { $count ->
    [one] { $count } nieuwe aflevering deze week.
   *[other] { $count } nieuwe afleveringen deze week.
}
feed-aired = uit { $ago }
feed-downloaded = gedownload { $ago }

## Bestanden en de bestandskiezer

files-already-picking = Er wordt al een bestand gekozen.
files-took-too-long = De bestandskiezer deed er te lang over.
files-did-not-open = De bestandskiezer ging niet open: { $error }
files-none-here = Er is hier geen bestandskiezer.

## De downloads van de computer

desk-sent-adding = { $what } naar { $name } gestuurd. Hij is ze nog aan het toevoegen.
desk-sent = { $what } ophalen op { $name }.

## De bibliotheek van je computer

remote-watched = Gekeken
remote-movie = Film
remote-not-watched-yet = Nog niet gekeken
remote-watched-of = { $done } van { $total } gekeken
remote-watched-count = { $count } gekeken
remote-on-disk = { $count } op schijf
remote-airs-in-season = S{ $season } aflevering { $n } over { $countdown }
remote-airs-in = Aflevering { $n } over { $countdown }
remote-play-episode = Speel aflevering { $n }
remote-watch-again = Kijk aflevering { $n } opnieuw
remote-update-host = Werk AniBeam op die computer bij om dit hier af te spelen
remote-aired = uit
remote-missing = Ontbreekt
remote-on-computer = Op je computer
remote-out = Uit { $ago }
remote-season-episode = Seizoen { $season }, aflevering { $n }
