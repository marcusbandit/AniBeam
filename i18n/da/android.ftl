## Navne, der forbliver navne

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Ord, mange sider deler

common-cancel = Annuller
common-back = Tilbage
common-save = Gem
common-done = Færdig
common-close = Luk
common-delete = Slet
common-retry = Prøv igen
common-copy = Kopier
common-clear = Ryd
common-on = Til
common-off = Fra
common-play = Afspil
common-hide = Skjul
common-remove = Fjern
common-add = Tilføj
common-test = Test
common-reading = Læser...
common-working = Arbejder...
common-checking = Tjekker...
common-all = Alle
common-none = Ingen
common-episodes = { $count ->
    [one] { $count } afsnit
   *[other] { $count } afsnit
}
common-episode-n = Afsnit { $n }
common-join-comma = { $a }, { $b }

## Sprog (#793)

lang-follow-phone = Samme som telefonen
more-language = Sprog
more-language-phone = Samme som telefonen: { $lang }
lang-name-en = Engelsk
lang-name-da = Dansk
lang-name-de = Tysk
lang-name-es = Spansk
lang-name-fr = Fransk
lang-name-pt-BR = Portugisisk (Brasilien)
lang-name-it = Italiensk
lang-name-nl = Hollandsk
lang-name-pl = Polsk
lang-name-ru = Russisk
lang-name-ja = Japansk
lang-name-ko = Koreansk
lang-name-zh-CN = Kinesisk (forenklet)

## Introduktion

onb-welcome-title = AniBeam
onb-welcome-note = Din anime på denne telefon. Find en serie, se hvem der har den, download den, og hold dine lister ajour.
onb-language-title = Sprog
onb-language-note = AniBeam bruger telefonens sprog, medmindre du vælger et andet. Du kan ændre det senere under Mere.
onb-lists-title = Dine lister
onb-lists-note = Log ind, så det du ser her tæller på AniList eller MyAnimeList. Du kan gøre det senere under Mere.
onb-wifi-title = Kun Wi-Fi
onb-wifi-note = Afsnit fylder meget. AniBeam downloader kun på Wi-Fi, medmindre du siger andet.
onb-storage-title = Lager
onb-storage-note = En grænse for, hvor meget AniBeam må downloade. Den reserverer ingenting: pladsen er fri, indtil afsnit fylder den.
onb-notify-title = Notifikationer
onb-notify-note = Mens afsnit downloades, viser en notifikation, hvor langt de er, og en anden fortæller, når et er klar til at se. Downloads kører uanset.
onb-sites-title = Søgesider
onb-sites-note = AniBeam kommer uden noget at søge i. Et repo er en signeret liste over sider, der kan søges i. Indsæt et repos adresse, tjek dets nøgle, og installer de sider, du vil have. Du kan gøre det senere under Mere.
onb-computer-title = Din computer
onb-computer-note = AniBeam på en computer kan dele sit bibliotek med denne telefon over Tailscale: dens serier vises under Bibliotek, og deres afsnit afspilles her. Skriv dens Tailscale-navn og port, og tryk Test. Ingen computer? Lad feltet stå tomt og start; Mere har det senere.
onb-wifi-only = Kun på Wi-Fi
onb-wifi-on = Til. Mobildata spørger først.
onb-wifi-off = Fra. Downloads bruger også mobildata.
onb-storage-caption = 18 GB er cirka 12 afsnit. Træk det hvorhen som helst op til telefonens fulde lager.
onb-use-cap = Brug grænsen
onb-cap-on = Til. Ved { $cap } downloader AniBeam ikke mere, før du sletter afsnit.
onb-cap-off = Fra. AniBeam downloader, indtil telefonen er fuld.
onb-start = Start
onb-continue = Fortsæt
onb-back = Tilbage

## Udseende

look-colours = Farver
look-mode = Tilstand
look-dark = Mørk
look-light = Lys
look-system = System
look-follows-phone-dark = Følger telefonens egen indstilling: mørk.
look-follows-phone-light = Følger telefonens egen indstilling: lys.
look-changes-at-once = Ændringer ses med det samme, her og på alle sider.
look-dark-theme = Mørkt tema
look-light-theme = Lyst tema
look-accent = Accent
look-accent-note = Markerer det valgte, det der afspilles og det nye.
look-size = Størrelse
look-text-size = Tekststørrelse
look-size-smaller = Mindre
look-size-normal = Normal
look-size-large = Stor
look-size-larger = Større
look-size-largest = Størst
look-text-size-note = Hvor stor teksten er overalt i AniBeam, oven i telefonens egen størrelse.

## Vinduet: faner og ark over alle sider

app-title = AniBeam
phone-nav-home = Hjem
nav-calendar = Kalender
phone-nav-library = Bibliotek
phone-nav-downloads = Downloads
nav-more = Mere
app-mobile-title = Download over mobildata?
app-mobile-note = Ét afsnit fylder ofte 1 til 2 GB. Med et begrænset abonnement kan det æde en måneds data på en aften.
app-mobile-keep = Behold kun Wi-Fi
app-mobile-use = Brug også mobildata
app-versions-note = Samme afsnit fra { $count } udgivelser. Vælg den, der skal afspilles.
app-versions-plays-now = { $release }, afspilles nu
app-menu-get = Hent afsnit
app-menu-delete = Slet afsnit

## Mere: oversigten og hvert afsnits status

more-settings = Indstillinger
more-search = Søg i indstillinger
more-search-none = Ingen resultater
more-search-hits = { $count ->
    [one] 1 indstilling
   *[other] { $count } indstillinger
}
more-search-tap = Tryk for at gå dertil
more-search-hint = Prøv et enklere ord: lager, Wi-Fi, tracker, side, computer.
more-accounts = Konti
more-streaming = Streaming
more-subtitles = Undertekster
more-connection = Forbindelse
more-computer = Din computer
more-icon = Appikon
more-stats = Statistik
more-data = Data
more-appearance = Udseende
more-activity = Aktivitet
more-about = Om
more-accounts-none = Ingen tracker forbundet
more-accounts-syncing = Synkroniserer med { $tracker }
more-accounts-syncing-both = Synkroniserer med AniList og MAL
more-dl-status-wifi = Kun Wi-Fi, ingen grænse
more-dl-status-wifi-cap = Kun Wi-Fi, grænse på { $cap }
more-dl-status-mobile = Wi-Fi og mobildata, ingen grænse
more-dl-status-mobile-cap = Wi-Fi og mobildata, grænse på { $cap }
more-stream-status-auto = Auto, 720p på mobildata eller relay
more-stream-status-original = Originalfilen, hver gang
more-stream-status-converted = Konverteret til { $quality }
more-sites-status-none = Intet repo
more-sites-status-on = { $sites ->
    [one] 1 side til
   *[other] { $sites } sider til
}
more-sites-status-on-from = { $sites ->
    [one] 1 side til
   *[other] { $sites } sider til
}, { $repos ->
    [one] fra 1 repo
   *[other] fra { $repos } repos
}
more-conn-status-unchecked = Ikke tjekket endnu
more-conn-status-ok = Alle trin gik igennem
more-conn-status-failed = { $total ->
    *[other] { $failed } af { $total } trin fejlede
}
more-computer-status-none = Ikke forbundet endnu
more-icon-default = Standard
more-icon-dark = Mørk
more-icon-line = Stregtegning
more-subs-status = Størrelse, placering, skrift, farver og boks
more-data-status = Backup, gendannelse, billedcache
more-activity-status = Hvad AniBeam gjorde, nyeste først
more-look-status-phone = Følger telefonen
more-stats-status = { $count ->
    [one] { $hours } time set i { $year }
   *[other] { $hours } timer set i { $year }
}
more-stats-status-empty = Et år med anime

## Mere: afsnittenes egne indstillinger

more-accounts-note = Det du ser her opdaterer dine lister, og fremgang derfra vises på dine serier.
more-storage = Lager
more-storage-capped = Grænse på { $cap }
more-storage-no-cap = Ubegrænset
more-storage-used = brugt af afsnit
more-storage-cap-caption = grænse
more-storage-free = ledigt på telefonen
more-set-storage-cap = Lagergrænse
more-storage-cap-on = Ved { $cap } downloades der ikke mere, før du sletter afsnit.
more-storage-cap-off = Fra: AniBeam downloader, indtil telefonen er fuld.
more-network = Netværk
more-network-wifi = Kun Wi-Fi
more-network-mobile = Wi-Fi og mobildata
more-wifi-on = Til. Downloads venter på Wi-Fi; mobildata spørger først.
more-set-stream-quality = Streamingkvalitet
more-stream-question = Hvordan et afsnit fra din computer afspilles her.
more-stream-auto-note = Auto afspiller filen, som den er, på Wi-Fi, og konverteret til 720p på mobildata, eller når Tailscale når din computer via et relay.
more-stream-original-note = Filen, som den er, hver gang. Store filer kræver en hurtig forbindelse.
more-stream-converted-note = Konverteres på din computer til { $quality }. Bruger mindre data; spoling tager et øjeblik.
more-set-feeds = Dine egne feeds
more-feeds-note = Søges før siderne
more-set-check = Tjek forbindelsen
more-check-note = Tester hvert trin mellem telefonen og søgesiderne: at finde dem, at nå dem og en sikker forbindelse.
more-check-copy = Kopier resultaterne
more-set-computer = AniBeam på din computer
more-computer-note = Dens bibliotek vises under Bibliotek, og dens afsnit afspilles her. Et navn som my-computer eller en adresse som 100.64.0.1.
more-computer-placeholder = my-computer:8790
more-subs-note = Det hvert afsnit på denne telefon starter med. Et afsnit streamet fra din computer starter med computerens indstillinger.
more-icon-note = Ikonet på din startskærm og i appoversigten.
more-icon-line-note = Stregtegning låner din baggrunds farver, som dine andre Material You-ikoner.
more-icon-moved-note = Nogle launchere fjerner ikonet fra startskærmen, når det skifter. Det bliver i appoversigten; træk det tilbage derfra.
more-set-whats-new = Nyheder
more-whats-new-note = Hvad der ændrede sig i hver version, nyeste først.
more-set-onboarding = Opsæt igen
more-onboarding-note = Gennemgå spørgsmålene fra første start en gang til.
more-set-version = Version

## Statuslinjer, Rust skriver på siderne

host-synced = Synket { $when }.
host-last-try = Sidste forsøg { $when }: { $error }
host-asking = Spørger...
host-reached = Fik forbindelse til { $host }, AniBeam { $version }.
host-saved = Gemt. Synkroniserer...
host-none = Ingen computer. Telefonen holder sig til sit eget bibliotek.
acct-signed-in-as = Logget ind som { $name }
acct-signed-in = Logget ind
acct-not-in-build = Ikke tilgængelig i denne version
acct-tap-to-sign-in = Ikke logget ind. Tryk for at logge ind.
acct-sign-in-failed = Login mislykkedes: { $why }. Tryk for at prøve igen.
acct-sign-in-not-started = Login startede ikke: { $error }. Tryk for at prøve igen.
acct-still-starting = AniBeam starter stadig. Prøv igen om lidt.
home-recently-watched = Set for nylig
home-recently-downloaded = Downloadet for nylig
home-note-empty = På denne telefon
home-note = { $count ->
    [one] På denne telefon, 1 serie
   *[other] På denne telefon, { $count } serier
}
lib-this-phone = Denne telefon
lib-note-phone = { $count ->
    [one] Denne telefon, 1 serie
   *[other] Denne telefon, { $count } serier
}
lib-note-remote = { $device }, synkroniseret { $when }
lib-empty-search = Ingen resultater for “{ $query }”.
lib-empty-phone = Intet på denne telefon endnu.
lib-empty-remote = Den enhed har intet i sit bibliotek.
cal-view-airing = Sendes nu
cal-note-none = Intet sendes denne uge
cal-note = { $count ->
    [one] 1 afsnit de næste 7 dage
   *[other] { $count } afsnit de næste 7 dage
}
airing-asking = Spørger AniList
airing-asking-newest = Spørger AniList om det nyeste
phone-airing-filter-all = Alt { $count }
airing-filter-listed = På mine lister { $count }
airing-filter-owned = På telefonen { $count }
airing-this-season = Denne sæson
airing-next-season = Næste sæson
airing-other-season = Andre
airing-hidden = { $count ->
    *[other] { $count } skjult som uinteressante
}
airing-plan-needs-tracker = Planlæg kræver AniList eller MAL. Log ind under Mere, Konti.
airing-plan-failed = Planlægningen gik ikke igennem: { $error }
list-watching = Ser nu
list-planning = Planlagt
list-completed = Færdig
list-paused = På pause
list-dropped = Droppet
list-repeating = Genser
more-check-all-ok = Alle trin gik igennem.
more-check-failed = { $failed } af { $total } trin fejlede. Et skærmbillede af listen viser hvilke.
more-check-row = { $step }  { $ms } ms
more-check-copied = Kopieret. Indsæt det i en besked.
more-version = AniBeam til Android { $version } (build { $build })
dl-note-waiting = Venter på Wi-Fi. Downloads fortsætter af sig selv, når telefonen er på Wi-Fi.
dl-note-nothing = Intet downloades.
dl-state-done = Færdig
dl-state-waiting = Venter på Wi-Fi
dl-state-paused = På pause
dl-state-peers = Leder efter peers
dl-state-checking = Tjekker
dl-state-error = Stoppet af en fejl
dl-state-downloading = Downloader
dl-amount = { $got } af { $size }
dl-speed = { $rate }/s
dl-left = { $span } tilbage
dl-episodes-range = Afsnit { $from } til { $to }
dl-batch = En pakke
disk-storage-line = { $used } brugt
disk-storage-line-free = { $used } brugt, { $free } ledigt
disk-free = Denne telefon har { $free } ledigt.
disk-free-below-cap = Denne telefon har { $free } ledigt, mindre end grænsen: telefonen bliver fuld, før AniBeam stopper.

## Søgning i Mere: ord man kunne skrive for hver indstilling

more-set-subs-size = Størrelse, placering
more-set-subs-style = Undertekststil
more-words-anilist = anilist tracker log ind login konto forbind liste fremgang
more-words-mal = mal myanimelist tracker log ind login konto forbind liste fremgang
more-words-storage-cap = plads disk grænse gb fuld lager slet størrelse
more-words-wifi = wifi wi-fi mobil mobildata netværk
more-words-notifications = notifikation besked advarsel fremgang tilladelse underretning tillad
more-words-stream-quality = auto original 1080p 720p 480p konverter data computer afspil
more-words-check-repos = opdater opdatering repo endpoint
more-words-add-repo = repo endpoint stol fingeraftryk adresse url
more-words-add-site = egen endpoint feed rss torznab jackett prowlarr adresse
more-words-feeds = feed rss kilde fjern
more-words-check = netværk dns test https sikker nå internet
more-words-computer = tailscale tailnet vært adresse desktop bibliotek stream test gem enhed
more-words-whats-new = ændringslog nyheder ændringer opdatering
more-words-onboarding = introduktion første start forfra
more-words-version = build om
more-words-stats = timer afsnit genrer studier sæsoner år set opsummering travleste måned historik
more-words-backup = sikkerhedskopi backup eksport fil gem kopi flyt computer json privat logins
more-words-restore = gendan import fil indlæs bibliotek flet
more-words-cache = billeder plakater bannere ryd lager plads cache
more-words-activity = log hændelser fejl problemer kopier ryd historik hvad skete
more-words-colours = udseende tema mørk lys tilstand system accent farve
more-words-text-size = udseende skrift større mindre størrelse tekst læs
more-words-icon = ikon launcher startskærm logo mørk streg tema material you
more-words-subs-size = undertekster tekster størrelse skala placering position løft bund
more-words-subs-style = undertekster tekster skrift farve kontur skygge boks fed ass stil nulstil
more-words-language = sprog oversættelse engelsk dansk locale

## Hjem

phone-home-title = Hjem
home-next-to-air = Sendes næste
home-after-that = Derefter
home-coming-up = Kommer snart
home-nothing-week = Intet den næste uge.
phone-home-empty-title = Intet på denne telefon endnu
phone-home-empty-body = Serier, du downloader her, og hvad der sendes af dem, vises under Hjem. Find en under Downloads.

## Bibliotek

lib-title = Bibliotek
lib-refresh = Opdater
lib-search-phone = Søg på denne telefon
lib-search-device = Søg i { $device }
lib-find-in-downloads = Find “{ $query }” i Downloads
lib-empty-body = Afsnit, du downloader på denne telefon, lander her. Find en serie under Downloads.
lib-pick-show = Vælg en serie for at åbne den her.
lib-shows-caption = { $count ->
    [one] serie
   *[other] serier
}
lib-on-phone = på telefonen
lib-free = ledigt
lib-hold-poster = Hold på en plakat for at hente flere afsnit eller slette dem.
phone-nothing-on-phone = intet på telefon
phone-on-phone = { $count } på telefon

## Kalender

cal-title = Kalender
cal-week = Uge
cal-days = Dage
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A %-d. %B
cal-today = I dag
cal-tomorrow = I morgen
cal-yesterday = I går
cal-nothing = Intet
cal-next = Næste
cal-get = Hent
cal-day-note = { $date }, { $count }
cal-next-airing = Næste afsnit: { $title }
cal-nothing-aired-yesterday = Intet blev sendt i går.
cal-nothing-aired-that-day = Intet blev sendt den dag.
cal-nothing-today = Intet sendes i dag.
cal-nothing-tomorrow = Intet sendes i morgen.
cal-nothing-that-day = Intet sendes den dag.
cal-empty-title = Ingen serier på denne telefon endnu
cal-empty-body = Afsnit af de serier, du har her, vises på deres dag, to dage tilbage og en uge frem.
phone-episodes-range = Afsnit { $from } til { $to }
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = På telefonen
phone-out-ago = Udkom { $ago }

## Sendes nu

phone-airing-out-now = Udkommet nu
airing-last-two-days = De sidste to dage
airing-season-note = Sæsonens serier kommer fra AniList.
airing-none-lists-title = Intet, der sendes, er på dine lister
airing-none-lists-body = Planlæg på en serie sætter den på din liste.
airing-none-phone-title = Intet, der sendes, er på telefonen
airing-none-phone-body = Hent afsnit på en serie downloader den her.
airing-marked = { $title } er markeret som uinteressant.
phone-airing-undo = Fortryd
airing-plan = Planlæg
airing-get-more = Hent flere afsnit
airing-get = Hent afsnit
phone-airing-not-interested = Ikke interesseret

## Sæsoner

season-winter = Vinter
season-spring = Forår
season-summer = Sommer
season-fall = Efterår
season-named = { $season } { $year }
season-pick = Vælg en sæson

## En series side

series-back-home = Hjem
series-back-library = Bibliotek
series-back-downloads = Downloads
series-back-calendar = Kalender
phone-series-get-episodes = Hent afsnit
phone-series-episodes = Afsnit
series-delete-episodes = Slet afsnit
series-delete-tap = Tryk på de afsnit, der skal slettes.
phone-series-delete-picked = { $count ->
    [one] { $count } afsnit valgt.
   *[other] { $count } afsnit valgt.
}
series-delete-note = Deres filer slettes fra denne telefon. Det du har set, bliver registreret.
series-delete-button = { $count ->
    [one] Slet { $count } afsnit
   *[other] Slet { $count } afsnit
}
series-no-episodes = Ingen afsnit på denne telefon og ingen planlagt.
series-versions = { $count ->
    [one] { $count } version
   *[other] { $count } versioner
}
series-note-next-up = Næste
series-note-missing = Mangler
series-note-not-here = Ikke på telefonen
series-note-not-downloaded = Ikke downloadet
series-note-downloading = Downloader { $percent } %
series-note-resume-at = Fortsæt ved { $time }
phone-episodes-estimate = { $count ->
    [one] { $count }? afsnit
   *[other] { $count }? afsnit
}
phone-next-airs = Afsnit { $n } sendes om { $time }
phone-watched = Set
phone-not-watched = Ikke set endnu
phone-progress = { $watched } af { $total } set, { $count } på telefonen
phone-on-this-phone = { $count } på telefonen
phone-play-film = Afspil filmen
phone-resume-film = Fortsæt filmen
phone-play-episode = Afspil afsnit { $n }
phone-resume-episode = Fortsæt afsnit { $n }
phone-about-date = omkring { $date }

## En series listestatus, karakter, noter og rettelser

show-status-watching = Ser nu
show-status-planning = Planlagt
show-status-completed = Færdig
show-status-paused = På pause
show-status-dropped = Droppet
show-status-repeating = Genser
show-status-now = { $label }, nu
phone-show-add-to-list = Føj til liste
phone-show-your-score = Din karakter { $score }
show-rate = Bedøm
show-fixes = Rettelser
show-your-notes = Dine noter
show-add-notes = Tilføj noter
show-home-each = Ændr det på hver liste, der er forbundet.
show-home-no-tracker = Gemt på denne telefon: ingen tracker har denne serie.
show-home-tracker-lacks = Gemt på denne telefon: din tracker har ikke denne serie.
show-home-until-connected = Gemt på denne telefon, indtil du forbinder AniList eller MyAnimeList.
show-sites-both = AniList og MyAnimeList
show-notes-title = Noter
show-notes-placeholder = Hvor du slap, hvem der anbefalede den, hvilken udgivelse du skal hente
show-notes-over = { $count ->
    [one] { $count } tegn over grænsen
   *[other] { $count } tegn over grænsen
}
show-notes-saved-to = Gemt på { $sites }.
show-notes-saved-to-tracker = Gemt på din tracker.
show-notes-saving = Gemmer dine noter.
show-notes-clearing = Rydder dine noter.
show-score-saving = Gemmer din karakter.
show-score-saved = Karakter gemt.
show-score-clearing = Rydder din karakter.
show-score-cleared = Karakter ryddet.
phone-show-moving-to = Flytter til { $label }.
phone-show-moved-to = Flyttet til { $label }.
show-rescan-started = Genscanning startet.
show-still-starting = AniBeam starter stadig. Prøv igen om lidt.
show-fix-match = Match
show-fix-find-match = Find et match
show-fix-rescan = Genscan mappe

## En series figurer, relaterede serier og anbefalinger

show-characters = Figurer
phone-show-related = Relateret
show-related-all = Vis alle { $count }
show-related-filling = Hentes stadig fra AniList.
phone-show-recommendations = Anbefalinger
show-card-owned = I dit bibliotek
show-card-unreleased = Endnu ikke udgivet
show-card-on-anilist = På AniList
show-rec-available = Tilgængelig
show-relation-prequel = Prequel
show-relation-sequel = Sequel
show-relation-side-story = Sidehistorie
show-relation-parent = Overordnet
show-relation-spin-off = Spin-off
show-relation-adaptation = Tilpasning
show-relation-alternative = Alternativ
show-relation-summary = Resumé
show-relation-character = Figur
show-relation-source = Kilde
show-relation-compilation = Samling
show-relation-contains = Indeholder
show-relation-other = Andet
show-role-main = Hoved
show-role-supporting = Birolle
show-role-background = Baggrund
show-format-tv = TV
show-format-tv-short = TV kort
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Special
show-format-film = Film
show-entry-get = Hent afsnit
show-entry-open-anilist = Åbn på AniList
show-entry-line = { $line }, { $status }

## Ret et forkert match

phone-match-title = Match
match-refresh = Opdater info
phone-match-clear = Ryd match
match-clear-it = Ryd det
match-clear-warning = Rydning fjerner trackerlinket og et manuelt sat afsnitsantal. Tryk igen for at rydde.
match-tvmaze = TVmaze
match-search-anilist = Søg på AniList
match-search-tvmaze = Søg på TVmaze
match-paste-anilist = Indsæt et AniList- eller MyAnimeList-link
match-paste-tvmaze = Indsæt et TVmaze- eller IMDb-link
phone-match-look-up = Slå op
match-apply = Anvend
match-look-up-all = Slå op for hver mappe, AniBeam endnu ikke kender
match-format-music = Musik
phone-match-matched-on = Matchet på { $sites }
phone-match-found-tvmaze = Fandt serien på TVmaze
phone-match-found-anilist = Fandt den på AniList
phone-match-found-anilist-season = Fandt den på AniList, sæson { $season }
phone-match-found-mal = Fandt den på MyAnimeList
phone-match-searching = Søger på { $name }
phone-match-type-more = Skriv mindst to bogstaver for at søge på { $name }.
match-none = Ingen titler fundet på { $name }.
match-looking-up = Slår op.
phone-match-matched-to = Matchet med { $title }.
phone-match-applied = Match anvendt.
match-applying = Anvender matchet.
phone-match-refreshing = Opdaterer { $title }.
match-clearing = Rydder matchet.
phone-match-cleared = Match ryddet.
match-auto-looking = Slår de mapper op, AniBeam endnu ikke kender.

## Pakker (linjen under en udgivelse i en søgning)

inspect-reading = Læser dens filliste
inspect-holds = { $count ->
    [one] { $count } afsnit i den: { $runs }
   *[other] { $count } afsnit i den: { $runs }
}

## Nyheder

phone-notes-title = Nyheder
notes-got-it = Forstået

## Flyt listen til en tracker

move-lists-label = Lister på denne telefon
move-title = Flyt din liste til { $site }
move-here = På denne telefon: { $list }
move-there = På { $site } nu: { $list }
move-later-hint = Senere: Mere, Konti
move-not-now = Ikke nu
move-nothing = Intet
move-not-on-list = Ikke på din liste
move-nothing-left = Intet tilbage at flytte
move-side-status-score = { $status }, karakter { $score }
move-side-score = karakter { $score }
move-lead = { $count ->
    [one] Én serie har en status eller karakter kun på denne telefon. Flytter du dem, havner de på din { $site }-liste, hvor din computer og din profil kan se dem. For disse serier erstattes det, { $site } har nu. Intet sendes, før du trykker Flyt.
   *[other] { $count } serier har en status eller karakter kun på denne telefon. Flytter du dem, havner de på din { $site }-liste, hvor din computer og din profil kan se dem. For disse serier erstattes det, { $site } har nu. Intet sendes, før du trykker Flyt.
}
move-label = { $count ->
    [one] Flyt { $count } serie
   *[other] Flyt { $count } serier
}
move-waiting = { $count ->
    [one] Én serie har en status eller karakter kun her. Flyt dem til { $site } for at se dem på din computer og din profil.
   *[other] { $count } serier har en status eller karakter kun her. Flyt dem til { $site } for at se dem på din computer og din profil.
}
move-moving = { $count ->
    [one] Flytter { $count } serie til { $site }
   *[other] Flytter { $count } serier til { $site }
}
move-moved = { $count ->
    [one] Flyttede { $count } serie til { $site }
   *[other] Flyttede { $count } serier til { $site }
}
move-failed = { $count ->
    [one] { $site } tog ikke imod { $count } serie; se aktivitetsloggen
   *[other] { $site } tog ikke imod { $count } serier; se aktivitetsloggen
}

## Downloads: find (søgning, serier, kilder, udgivelser)

common-join-and = { $a } og { $b }
find-join-sentences = { $a } { $b }

find-downloads = Downloads

find-lens-list = Din liste
find-lens-gaps = Huller
find-lens-airing = Sendes
find-lens-following = Følger
find-lens-downloaded = Hentet
find-lens-with-count = { $lens } { $count }
find-what-to-get = At hente
find-lens-list-line = Ser nu og Planlagt på din liste. Åbn en for at hente dens afsnit.
find-lens-gaps-line = Sendte afsnit, der ikke er på denne telefon. Åbn en serie for at se, hvem der har dem.
find-lens-airing-line = Nye afsnit, der stadig kommer, de nærmeste først.
find-lens-following-line = Abonnementerne: nye afsnit downloades af sig selv. Åbn et for at stoppe det.
find-lens-downloaded-line = Hvad denne telefon har downloadet, en serie ad gangen.

find-lens-gaps-empty = Ingen huller: hver serie, du er begyndt på, har sine sendte afsnit her.
find-lens-airing-empty = Intet på denne telefon sendes lige nu.
find-lens-following-empty = Ingen abonnementer endnu. Åbn en serie og slå Følg til.
find-lens-downloaded-empty = Intet downloadet endnu.
find-list-empty = Intet på din liste endnu. Søg efter en serie ovenfor og tilføj den.
find-list-reading = Læser din liste...
find-list-signed-out = Log ind på AniList under Mere, Konti, så vises din liste her.
find-list-failed = Din liste kunne ikke læses: { $error }

find-aired = sendt
find-ledger-no-feed = ingen nye afsnit: kilden har intet feed
find-ledger-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
find-ledger-downloaded = downloadet
find-ledger-planning = Planlagt
find-ledger-watching-of = Ser nu, { $progress } af { $total }
find-ledger-watching-seen = Ser nu, { $progress } set

find-gap-missing = { $count } mangler
find-gap-not-downloaded = { $count } ikke downloadet

find-list-status-planning = På din liste: Planlagt
find-list-status-watching = På din liste: Ser nu
find-list-status-already = Allerede på din liste
find-going-planning = { $title } sættes på Planlagt.
find-already-listed = { $title } er allerede på din liste.
find-plan-needs-tracker = At føje til din liste kræver AniList eller MAL. Log ind under Mere, Konti.
find-plan-failed = Ikke føjet til din liste: { $error }
find-show-gone = Den serie er ikke længere i biblioteket på denne telefon.

## Serier mens du skriver

find-shows = Serier
find-suggesting = Spørger AniList...
find-suggest-tap = Tryk på en for at se, hvem der har den. Retur søger i udgivelsesnavne.
find-suggest-none = Ingen serie med det navn. Retur søger i udgivelsesnavne i stedet.
find-suggest-hint = Serier dukker op, mens du skriver. Retur søger i udgivelsesnavne.
find-suggest-failed = AniList kunne ikke spørges: { $error }
find-library-line = I dit bibliotek
find-library-line-none = I dit bibliotek, ingen af { $total } på denne telefon
find-library-line-some = I dit bibliotek, { $here } af { $total } på denne telefon

## Seriens visning

find-plan-to-watch = Planlæg
find-on-phone-count = { $count } på denne telefon
find-airing = Sendes
find-dropped = { $count ->
    [one] { $count } udgivelse, der ikke nævner denne serie, er udeladt.
   *[other] { $count } udgivelser, der ikke nævner denne serie, er udeladt.
}

find-following = Følger
find-stop-following-ask = Stop med at følge { $name }?
find-unfollow-explain = Nye afsnit downloades ikke mere. Det, der ligger i mappen, kan blive eller følge med: alle downloads dér, også afsnit du hentede selv.
find-stop-following = Stop med at følge
find-stop-keep = Stop, behold afsnittene
find-stop-delete = Stop og slet mappens downloads
find-rule-taken = { $downloads ->
    [one] { $downloads } download
   *[other] { $downloads } downloads
}, { $files ->
    [one] { $files } fil
   *[other] { $files } filer
} i mappen
find-rule-paused = Det er sat på pause, så der kommer ingen nye afsnit. { $taken }.
find-rule-no-feed = Kilden har intet feed, så der kommer ingen nye afsnit. { $taken }.
find-rule-active = Nye afsnit downloades af sig selv. { $taken }.
find-unfollowed-deleted = Stoppede med at følge { $name } og slettede downloads i dens mappe.
find-unfollowed-kept = Stoppede med at følge { $name }. Dens afsnit bliver på denne telefon.
find-stop-failed = Stoppet blev ikke færdigt: { $error }

find-where = Hvor den fås
find-where-pick = Vælg en kilde for at se, hvad den har.
find-where-nothing = Intet downloades, før du siger til.
find-source-idle = Spørg
find-source-trying = Prøver { $name }
find-source-searching = Søger
find-source-silent = Svarer ikke
find-no-sites-title = Ingen søgesider endnu
find-no-sites-body = Tilføj et repo med søgesider, eller et eget feed, under Mere.
find-open-more = Åbn Mere
find-pick-source-title = Vælg en kilde
find-pick-source-body = Dens udgivelser vises her som et gitter, en terning pr. afsnit.
find-releases = { $count ->
    [one] { $count } udgivelse
   *[other] { $count } udgivelser
}
find-unreachable = { $host } kunne ikke nås. Nogle netværk og lande blokerer udgivelsessider; AniBeam kommer allerede uden om blokeringer på navne, så her blokeres selve siden, eller siden er nede. Et VPN kommer uden om det, eller prøv en anden kilde.
find-the-source = Kilden

find-has = Hvad den har
find-has-note = Tryk på en række for at tage det hele, eller enkelte afsnit.
find-episodes-tag = Afsnit
find-batch = Pakke { $label }
find-row-meta = { common-episodes }  { $seeds } seedere  { $size }
find-on-this-phone = På denne telefon
find-on-computer = På { $name }

find-plan-no-releases = Denne kilde har ingen udgivelser af denne serie.
find-plan-nothing-picked = Intet valgt: tryk på en række eller dens terninger.
find-plan-line = { $episodes } fra { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] { $count } afsnit, du allerede har, er udeladt. Slå Erstat til for at hente det igen.
   *[other] Afsnit, du allerede har, er udeladt. Slå Erstat til for at hente dem igen.
}
find-plan-follow-note = Nye afsnit fra { $query } downloades af sig selv.
find-plan-on-desk = De downloades på { $name }, ind i dens bibliotek.
find-follow-new = Følg nye afsnit
find-follow-new-note = De downloades af sig selv, efterhånden som de sendes.
find-replace = Erstat det, du har
find-replace-note = Downloader de valgte afsnit igen oven på dem på denne telefon.
find-go = { $mode ->
    [replace] Hent { common-episodes } og erstat
    [subscribe] Hent { common-episodes } og abonnér
    [both] Hent { common-episodes }, erstat og abonnér
   *[plain] Hent { common-episodes }
}
find-go-pick = Vælg afsnit at hente
find-every-release = Alle udgivelser fra denne kilde

## Download til

find-download-to = Download til
find-this-phone = Denne telefon
find-your-computer = Din computer
find-desk-asking = Spørger { $name }...
find-desk-silent = { $name } svarer ikke. Vælg Denne telefon, eller prøv igen, når den er tændt.
find-sending-to = Sender til { $name }...

## Udgivelseslisten

find-best-fit = Bedste match
find-on-disk = På disk
find-get = Hent
find-get-all = Hent alle { $count }
find-subscribe = Abonnér
find-back-step = Et trin tilbage
find-narrow-placeholder = Indsnævr, eller -ord for at udelade
find-best-fit-searches = Bedste match søger: { $query }
find-fact-batch = pakke
find-fact-seeds = { $count } seedere
find-fact-via = via { $source }
find-searching-for = Søger efter { $text }...
find-search-line = { $count ->
    [0] Intet på { $source } for det
    [one] { $count } udgivelse på { $source }
   *[other] { $count } udgivelser på { $source }
}{ $fell ->
    [1] , efter at den første kilde intet havde
   *[other] { "" }
}{ $narrowed ->
    [1] , indsnævret til { $token }
   *[other] { "" }
}.
find-refined = { find-releases }, { $dir ->
    [down] ned
   *[up] op
} { $delta }.
find-refine-nothing-left = Så er der intet tilbage.
find-refine-no-match = Ingen udgivelse har det.
find-refine-empty = Skriv noget først.
find-refine-no-change = Det ændrer intet.

## At hente og abonnere

find-getting = Henter { $what }.
find-subscribed = Abonneret: nye afsnit downloades af sig selv.
find-subscribed-desk = Abonneret på { $name }: nye afsnit downloades dér af sig selv.
find-subscribed-as = Abonneret som { $name }: nye afsnit downloades af sig selv.
find-downloading = { $count ->
    [one] Downloader { $count } udgivelse.
   *[other] Downloader { $count } udgivelser.
}
find-not-added = Ikke tilføjet: { $error }
find-not-subscribed = Ikke abonneret: { $error }
find-desk-refused = { $name } tog ikke imod det: { $error }
find-desk-not-subscribed = Ikke abonneret på { $name }: { $error }

## Afspiller

player-subtitles = Undertekster
player-audio = Lyd
phone-player-speed = Hastighed
player-connecting = Forbinder
player-hold-speed = 2× hastighed
player-replay = Gentag
player-unlock = Lås op
player-next-stay = Bliv
player-next-go = Næste
player-jump-back = −10 s
player-jump-forward = +10 s
player-speed-now = Hastighed { $speed }
player-skip-intro = Spring intro over
player-skip-outro = Spring outro over
phone-player-skipped-intro = Sprang intro over
phone-player-skipped-outro = Sprang outro over
phone-player-undo = Fortryd
phone-player-start-over = Fra start
phone-player-resumed-at = Fortsatte ved { $time }
player-picture-failed = Billedet kunne ikke starte. Lyden afspilles videre.
player-file-failed = Denne fil kunne ikke afspilles: { $why }.
player-streaming-from = Streamer fra { $host } i { $height }p
player-mpv-no-start = mpv startede ikke
player-mpv-no-start-why = mpv startede ikke: { $why }
player-android-only = afspilleren kører kun på Android

## Afspiller: forsinkelser

player-delay-title = Forsinkelse
player-delay-note = Hvert tryk flytter den 0,1 s. Indtil næste fil.
phone-player-delay-seconds = { $value } s
player-sub-delay-now = Undertekstforsinkelse { $value }
player-audio-delay-now = Lydforsinkelse { $value }

## Afspiller: cast

player-cast-playing-on = Afspilles på { $device }
phone-player-cast-stop = Stop cast til { $device }
player-cast-searching = Leder efter Chromecasts og tv
phone-player-cast-to = Cast til
phone-player-cast-tv = TV
phone-player-cast-loading = Indlæser
phone-player-cast-playing = Afspiller
phone-player-cast-paused = På pause
phone-player-cast-finished = Færdig
phone-player-cast-stopped = Stoppet

## Afspiller: trackerens ord

phone-player-rated = Bedømt
phone-player-score-failed = Kunne ikke gemme karakteren
player-marked-watched = Markeret som set
player-tracked = Synket
player-not-tracked = Ikke synket
player-track-update-failed = Kunne ikke opdatere din liste
player-not-tracked-why = Ikke synket: { $reason }.
player-track-ok-at = { $tracker } ved { $progress }
player-track-ok = { $tracker } ok
player-track-note = { $tracker }: { $note }
player-track-failed = { $tracker } kunne ikke opdateres
player-track-line = { $head }: { $parts }
phone-player-refusal-hidden = serien er skjult
phone-player-refusal-no-match = serien har intet match hos trackeren
phone-player-refusal-not-newer = trackeren er allerede forbi dette afsnit
phone-player-refusal-extra = det er en ekstra
phone-player-refusal-unmatched = filen er ikke matchet med et afsnit
phone-player-refusal-on-disk = filen findes kun på disken

## Undertekster

subs-line-default = Størrelse 1×, almindelige filer bruger din stil
subs-sample = Hvor gemte du den?
subs-size-place-title = Størrelse, sted
subs-size = Størrelse
subs-position = Placering
subs-position-helper = 100 ligger ved bunden. Lavere tal løfter linjerne op i billedet.
subs-style-title = Stil
subs-styled = Stilede undertekster
subs-styled-as-made = Stilede (ASS) filer ser ud præcis, som deres skabere har sat dem.
subs-styled-mine = Din stil nedenfor bruges på alle filer. Skilte og karaoke kan gå i stykker.
subs-styled-size-only = Stilede undertekstfiler beholder deres udseende og bruger din størrelse. Almindelige filer bruger din stil.
subs-mode-as-made = Original
subs-mode-size-only = Størrelse
subs-mode-mine = Mine
subs-font = Skrift
subs-font-bad = Skriv et skriftnavn, fx Noto Sans.
subs-bold = Fed
subs-colour = Tekstfarve
subs-colour-bad = Skriv en farve som #ffcc00.
subs-outline = Kontur
subs-outline-bad-width = Bredden er et tal, 0 eller mere.
subs-outline-bad-colour = Skriv farven som #000000.
subs-outline-helper = Dens bredde, så dens farve.
subs-shadow = Skygge
subs-shadow-bad = Et tal, 0 eller mere. 0 betyder ingen skygge.
subs-shadow-helper = Hvor langt skyggen falder, 0 for ingen.
subs-box = Baggrundsboks
subs-box-helper = En mørk boks bag linjerne. 0 udelader den.
subs-reset-title = Fra start
subs-reset-helper = Sætter størrelse, skrift, farver, kontur, boks og placering tilbage, som de var.
subs-reset = Nulstil

## Streaming

stream-quality-auto = Auto
stream-quality-original = Original
stream-app-starting = Appen starter stadig
stream-no-computer = Ingen computer endnu. Tilføj din under Mere.
stream-off-on-host = Streaming er slået fra på { $host }. Slå Stream afsnit til dér under Indstillinger, Telefon-app.
stream-cannot-stream = { $host } kører AniBeam { $version }, som ikke kan streame endnu. Opdater den dér.
stream-no-session = Den anden enhed gav ingen session
stream-answer-unreadable = Svaret kunne ikke læses: { $why }
stream-answered-status = { $name } svarede { $status }

## Statistik

phone-stats-nothing-yet = Intet endnu
phone-stats-empty-title = Intet set endnu
phone-stats-empty-body = Se et afsnit færdigt, så tæller det her: timer, genrer og de sæsoner, du har set.
stats-hours-caption = timer set
stats-finished-caption = serier set færdige
stats-busiest-caption = travleste måned
stats-episodes-caption = { $count ->
    [one] afsnit
   *[other] afsnit
}
phone-stats-by-month = Pr. måned
phone-stats-top-genres = Topgenrer
phone-stats-top-studios = Topstudier
stats-seasons = Sæsoner set
stats-watched-most = Mest set
stats-year-line = Hvad du så i { $year }
stats-eps = { $count ->
    [one] { $count } afs.
   *[other] { $count } afs.
}
stats-month-1 = Jan
stats-month-2 = Feb
stats-month-3 = Mar
stats-month-4 = Apr
stats-month-5 = Maj
stats-month-6 = Jun
stats-month-7 = Jul
stats-month-8 = Aug
stats-month-9 = Sep
stats-month-10 = Okt
stats-month-11 = Nov
stats-month-12 = Dec
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

## Backup og billedcache

backup-title = Backup
backup-export = Backup af biblioteket
backup-export-helper = Én fil med dine mapper, matches og det, du har set.
backup-private = Medtag private data
backup-private-helper = Tilføjer login til AniList og MyAnimeList, visningshistorik og præferencer. Login gemmes som ren tekst, så hold filen for dig selv.
backup-restore = Gendan fra en backup
backup-restore-helper = Læser en backup ind i dette bibliotek. Det, der ligger på disken, vinder; filen udfylder resten.
backup-done = Backup lavet. Filen ligger, hvor du gemte den.
backup-not-saved = Ikke gemt. Intet blev skrevet.
backup-starting = AniBeam starter stadig. Prøv igen om lidt.
backup-images-title = Billedcache
backup-images-body = Plakater, bannere og portrætter gemmes på telefonen, så biblioteket virker offline. Ryddede billeder downloades igen, når sider har brug for dem.
backup-keep = Behold
backup-removed = { $count ->
    [one] Fjernede { $count } billede
   *[other] Fjernede { $count } billeder
}
backup-images-none = Ingen billeder
backup-images-unmeasured = Billedcachen kunne ikke måles
backup-images-retry-note = Prøv igen om lidt. Selve billederne er i orden.
backup-images-line = { $size } i { $count ->
    [one] { $count } billede
   *[other] { $count } billeder
}
backup-images-question = { $count ->
    [one] Fjern { $count } billede?
   *[other] Fjern { $count } billeder?
}

backup-sum-list = { $items }.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] { $count } mappe tilføjet
   *[other] { $count } mapper tilføjet
}
backup-sum-series = { $count ->
    [one] { $count } serie oprettet
   *[other] { $count } serier oprettet
}
backup-sum-matches = { $count ->
    [one] { $count } match anvendt
   *[other] { $count } matches anvendt
}
backup-sum-views = { $views ->
    [one] { $views } visning
   *[other] { $views } visninger
} og { $episodes ->
    [one] { $episodes } færdigt afsnit
   *[other] { $episodes } færdige afsnit
} flettet
backup-sum-resume = { $count ->
    [one] { $count } genoptagelsespunkt flettet
   *[other] { $count } genoptagelsespunkter flettet
}
backup-sum-accounts = { $count ->
    [one] { $count } listekonto hentet ind
   *[other] { $count } listekonti hentet ind
}
backup-sum-fixes = { $count ->
    [one] { $count } fil tager en rettelse, du lavede.
   *[other] { $count } filer tager en rettelse, du lavede.
}
backup-sum-missing = { $count ->
    [one] { $count } mappe blev ikke fundet på denne telefon.
   *[other] { $count } mapper blev ikke fundet på denne telefon.
}
backup-sum-left-out = Udeladt: { $fields }.

## Aktivitet

activity-everything = Alt
activity-problems = Problemer
activity-copied = Kopieret
activity-clear-every-line = Ryd alle linjer
activity-fold = Luk
activity-no-problems = Ingen problemer. Alt virkede.
activity-nothing-yet = Intet at rapportere endnu. Det, AniBeam gør for dig, vises her.
activity-time-today = %H:%M:%S
activity-time-other = %-d. %b %H:%M

## Datoer og tal (companion format.rs)

fmt-clock = %H:%M
fmt-day-month = %-d. %b
fmt-weekday-date-time = %a %-d. %b %H:%M
fmt-weekday-date-year-time = %a %-d. %b %Y %H:%M
fmt-stamp = %-d. %b %H:%M
fmt-decimal-point = ,

fmt-today-at = I dag { $time }
fmt-tomorrow-at = I morgen { $time }
fmt-yesterday-at = I går { $time }

fmt-span-days = { $d } d { $h } t
fmt-span-hours = { $h } t { $m } m
fmt-span-minutes = { $m } m { $s } s
fmt-span-seconds = { $s } s
phone-fmt-in = om { $span }
fmt-full-days = { $d } d { $h } t { $m } m { $s } s
fmt-full-hours = { $h } t { $m } m { $s } s
fmt-full-minutes = { $m } m { $s } s

fmt-ago-now = lige nu
fmt-ago-minutes = { $count } min. siden
fmt-ago-hours = { $count } t siden
fmt-ago-days = { $count } d siden

fmt-bytes-b = { $value } B
fmt-bytes-kb = { $value } KB
fmt-bytes-mb = { $value } MB
fmt-bytes-gb = { $value } GB
fmt-bytes-tb = { $value } TB

## Samtale med computeren (companion client.rs, sync.rs, address.rs)

client-not-found = { $host } er ikke et navn, telefonen kan finde. Er Tailscale forbundet?
client-timeout = { $host } svarede ikke. Er den tændt, og er Tailscale forbundet på begge?
client-connect-failed = Kunne ikke nå { $host }. Er den tændt, og er Tailscale forbundet på begge?
client-refused = { $host } er tændt, men AniBeam dér er lukket eller deler ikke. Åbn AniBeam på den, og slå Del via Tailscale til under Indstillinger, Telefon-app.
client-no-route = Ingen rute til { $host }. Er Tailscale forbundet?
client-invalid-input = Kunne ikke oprette forbindelse til { $host }
client-unreachable = Kunne ikke nå { $host }: { $why }
client-cannot-find = Telefonen kan ikke finde { $host }. Er Tailscale forbundet?
client-broke-off = Svaret fra { $host } blev afbrudt: { $why }
client-answered = { $host } svarede { $status } på { $path }
client-not-anibeam-status = Noget svarede, men det er ikke AniBeam ({ $host } svarede { $status } på { $path })
client-not-anibeam-hello = Noget svarede, men det er ikke AniBeam (dets svar er ikke AniBeams hello)
client-not-anibeam-name = Noget svarede, men det er ikke AniBeam (det kalder sig { $name })
client-too-new = Værten taler API { $host_api }; denne app læser { $app_api }. Opdater appen.
client-snapshot-unreadable = Øjebliksbilledet kunne ikke læses: { $why }
client-probe-unreadable = Filens detaljer kunne ikke læses: { $why }
client-peers-unreadable = Listen over AniBeams kunne ikke læses: { $why }
sync-save-failed = Kunne ikke gemme øjebliksbilledet: { $why }
address-https = AniBeam på din computer svarer på almindelig http over Tailscale; fjern https://.
address-empty = Skriv værtens navn eller adresse, for eksempel my-computer:8790.
address-ipv6-close = En IPv6-adresse skal have sin afsluttende ].
address-ipv6-port = Sæt porten efter ], som [fd7a::1]:8790.
address-bad-host = “{ $text }” er ikke et værtsnavn.
address-bad-port = “{ $port }” er ikke en port.

## Seriefakta (companion library.rs)

clib-status-airing = Sendes
clib-status-not-yet = Ikke sendt endnu
clib-format-tv-short = TV kort
clib-format-movie = Film
clib-format-special = Special
clib-format-music = Musik

## Søgesider og repos

repos-by-hand-title = Tilføj en manuelt
repos-by-hand-note = Til en side, intet repo har. Ethvert feed med { "{query}" } der, hvor søgningen skal stå, også Jackett- eller Prowlarr-feeds. En nøgle eller token i adressen flyttes til denne telefons hemmelige lager.
repos-name-placeholder = Navn
repos-url-placeholder = https://...{ "{query}" }...
repos-index-placeholder = https://.../index.json
repos-add-site = Tilføj side
repos-add-title = Tilføj et repo
repos-add-note = Indsæt adressen på et repos indeks. AniBeam tjekker dets signatur og viser dig dets nøgle, før noget gemmes.
repos-add-repo = Tilføj repo
repos-trust-title = Stol på dette repo?
repos-fingerprint = Nøglens fingeraftryk
repos-trust-warning = Dets sider afgør, hvor dine søgninger går hen. Stol kun på et repo fra nogen, du stoler på.
repos-trust = Stol på
repos-open-link = Åbn { $url }
repos-saving = Gemmer...
repos-install = Installer
repos-update = Opdater
repos-edit-note = En nøgle eller token skrevet i adressen flyttes til denne telefons hemmelige lager.
repos-edit-note-keys = Lad hver { "{key}" } stå i adressen, som den er; dens værdi gemmes på denne telefon.
repos-change-its = Skift dens { $keys }
repos-check-repos = Tjek repos for nye sider
repos-check-sites = Tjek siderne
repos-add-site-label = Tilføj en side manuelt
repos-needs-before-search = Skal have sin { $labels }, før den søger
repos-no-longer-offered = Tilbydes ikke længere af sit repo
repos-update-host = Opdateringen sender forespørgsler til { $host }
repos-tap-change-custom = Tryk for at ændre dens navn eller adresse
repos-tap-change-keys = Tryk for at ændre dens { $labels }
repos-line-custom = { $host }  { $kind }  tilføjet af dig
repos-update-to = Opdater til v{ $version }
repos-add-its-value = Tilføj dens værdi
repos-asks-for = Beder om din { $labels }
repos-preview-line = { $count ->
    [one] { $count } side, signeret
   *[other] { $count } sider, signeret
}
repos-preview-trusted = { $count ->
    [one] { $count } side, signeret, allerede betroet
   *[other] { $count } sider, signeret, allerede betroet
}
repos-ask-kept-empty = Gemmes kun på denne telefon. Et felt, der lades tomt, beholder det gemte.
repos-ask-kept-never = Gemmes kun på denne telefon, aldrig i biblioteket eller en eksport.
repos-ask-new-host = Den nye version sender dette til { $host }, så den spørger igen.
repos-ask-new = Den nye version beder om disse igen.
repos-type-value = Skriv en værdi først.
repos-health-answered = Svarede på { $ms } ms
repos-health-failed = Svarede ikke. { $step }: { $detail }

## Downloads

dl-title = Downloads
dl-wifi-only = Kun Wi-Fi
dl-wifi-and-mobile = Wi-Fi og mobildata
dl-joined = { $first }, { $second }
dl-batch-files = { $count ->
    [one] Pakke, { $count } fil
   *[other] Pakke, { $count } filer
}
dl-batch-tag = Pakke
dl-downloading-count = { $count ->
    [one] { $count } downloader
   *[other] { $count } downloader
}
dl-free-on-phone = ledigt på telefonen
dl-search-placeholder = En serie eller et udgivelsesnavn
dl-sources = Kilder
dl-downloading = Downloader
dl-queue-note = { $count ->
    [one] { $count } afsnit
   *[other] { $count } downloads
}
dl-finished = Færdige
dl-finished-note = { $count ->
    [one] { $count } download, i dit bibliotek
   *[other] { $count } downloads, i dit bibliotek
}
dl-get = Hent
dl-pick = Vælg

## Lager og skjulte serier

disk-on-phone = På denne telefon
disk-watched = Set
disk-not-watched = Ikke set
disk-deleting = Sletter
disk-delete-watched = Slet sete
disk-hidden-shows = Skjulte serier
disk-hidden-helper = Væk fra Hjem og biblioteket, til du henter dem tilbage.
disk-unhide = Vis igen
disk-delete-note = Filer og undertekster slettes fra denne telefon for altid.
disk-every-watched = Alle sete afsnit
disk-every-finished = Alle færdige serier, du har set
disk-episodes-size = { $count ->
    [one] { $count } afsnit, { $size }
   *[other] { $count } afsnit, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } serie, { $size }
   *[other] { $count } serier, { $size }
}
disk-list-two = { $a } og { $b }
disk-list-three = { $a }, { $b } og { $c }
disk-list-more = { $a }, { $b }, { $c } og { $count } til
disk-delete-finished-title = Slet { $count ->
    [one] { $count } færdig serie
   *[other] { $count } færdige serier
}, { $size }?
disk-delete-watched-title = Slet { $count ->
    [one] { $count } set afsnit
   *[other] { $count } sete afsnit
}, { $size }?
disk-delete-finished-body = { $names }. Kun serier, som udbyderen siger er afsluttet, med alle afsnit set; en serie, der stadig sendes, eller hvis slutning er et gæt, bliver. Hver fil fra dem slettes fra denne telefon, også ekstraer. Deres downloads forsvinder også, så de kan downloades igen.
disk-delete-watched-body = Hvert afsnit, du har set, i hver serie. Afsnit, du ikke har set endnu, og ekstraer bliver. Filerne og deres undertekster slettes fra denne telefon for altid.
disk-no-finished = Ingen afsluttet serie er set helt til ende
disk-nothing-watched = Intet set ligger på denne telefon
disk-deleted = Slettede { $count ->
    [one] { $count } afsnit
   *[other] { $count } afsnit
}, { $size }
disk-delete-failed = Intet blev slettet: { $error }

## Download-notifikationer

notify-card-head = AniBeam · nu
notify-sample-called = Frieren afsnit 7
notify-sample-called-ready = Frieren afsnit 6
notify-sample-facts = 420 MB af 1,4 GB, 3,2 MB/s, 5 m tilbage
notify-downloading = Downloader { $called }
notify-progress = { $percent } %, { $facts }
notify-progress-of = { $percent } %, { $done } af { $size }
notify-starting = Starter
notify-many = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
notify-called-episode = { $show } afsnit { $n }
notify-called-episodes = { $show } afsnit { $from } til { $to }
notify-ready = { $called } er klar
notify-ready-many = { $called } er klar
notify-ready-batch = En pakke af { $show } er klar
notify-ready-text = Downloadet til denne telefon. Tryk for at åbne AniBeam.
notify-channel-downloads = Downloads
notify-channel-ready = Afsnit klar
notify-fallback-title = Downloader
notify-row-on = Notifikationer
notify-row-allow = Tillad notifikationer
notify-row-open-settings = Åbn notifikationsindstillinger
notify-helper-on = Til. Downloads vises i dine notifikationer, mens de kører.
notify-helper-asks-next = Android spørger nu.
notify-helper-asks-next-later = Android spørger nu. Du kan skifte mening senere under Mere.
notify-helper-off = Fra. Slå notifikationer til for AniBeam dér.
notify-question-title = Se downloads i dine notifikationer?
notify-question-body = Mens afsnit downloades, viser en notifikation, hvor langt de er, og en anden fortæller, når et er klar. De downloades uanset; det afgør kun, om du ser det.
notify-allow = Tillad
notify-not-now = Ikke nu
notify-pointer = Downloads vises ikke i dine notifikationer.
notify-turn-on = Slå til

## Feed

phone-feed-title = Feed
feed-recently-released = Nyligt udkommet
feed-coming-soon = Kommer snart
feed-this-week = Denne uge
feed-earlier = Tidligere
feed-everything-else = Alt andet
phone-feed-countdown-days = { $d } d { $h } t
phone-feed-countdown-hours = { $h } t { $m } m
phone-feed-countdown-minutes = { $m } m
phone-feed-next = Næste: { $title } om { $countdown }
phone-feed-nothing-scheduled = Intet planlagt endnu.
phone-feed-nothing-new = Intet nyt denne uge.
phone-feed-new-episodes = { $count ->
    [one] { $count } nyt afsnit denne uge.
   *[other] { $count } nye afsnit denne uge.
}
phone-feed-aired = sendt { $ago }
phone-feed-downloaded = downloadet { $ago }

## Filer og filvælgeren

files-already-picking = Der vælges allerede en fil.
files-took-too-long = Filvælgeren tog for lang tid.
files-did-not-open = Filvælgeren åbnede ikke: { $error }
files-none-here = Der er ingen filvælger her.

## Computerens downloads

desk-sent-adding = Sendte { $what } til { $name }. Den tilføjer dem stadig.
desk-sent = Henter { $what } på { $name }.

## Din computers bibliotek

remote-watched = Set
remote-movie = Film
remote-not-watched-yet = Ikke set endnu
remote-watched-of = { $done } af { $total } set
remote-watched-count = { $count } set
remote-on-disk = { $count } på disk
remote-airs-in-season = S{ $season } afsnit { $n } sendes om { $countdown }
remote-airs-in = Afsnit { $n } sendes om { $countdown }
remote-play-episode = Afspil afsnit { $n }
remote-watch-again = Se afsnit { $n } igen
remote-update-host = Opdater AniBeam på den computer for at afspille dette her
remote-aired = sendt
remote-missing = Mangler
remote-on-computer = På din computer
remote-out = Udkom { $ago }
remote-season-episode = Sæson { $season }, afsnit { $n }

## Rating-ord i søgning
more-words-ratings = imdb kreditter score bedømmelser karakter

## Format
show-format-series = Serie

## Afsnit og datoer
dates-title = Afsnit og datoer
dates-season-line = { $title } · Sæson { $season }
dates-paste-open = Indsæt en liste
dates-paste-done = Færdig
dates-paste-hint = Indsæt en udgivelsesliste, en tabel eller et opslag
dates-paste-note = Det AniBeam læser, vises på afsnittene nedenfor.
dates-tap-hint = Tryk på et afsnit for at give det en dato.
dates-run = Vælg en række
dates-field-hint = 7. jan., marts 2027, 2027
dates-set = Sæt dato
dates-no-date = Ingen dato
dates-weekly = Ugentligt
dates-last = Sidste afsnit
dates-not-last = Ikke sidste
dates-clear = Ryd dato
dates-save = Gem
dates-saving = Gemmer
dates-cancel = Annuller
dates-change = Ændre
dates-picked-one = Afsnit { $episode }:
dates-picked-many =
    { $count ->
        [one] { $count } afsnit:
       *[other] { $count } afsnit:
    }
dates-saved = Gemt.
dates-cleared = Datonoter ryddet.
dates-starting = AniBeam starter stadig. Prøv igen om lidt.
dates-tile-out = Udkom
dates-tile-last = Sidste
dates-tile-none = Ingen dato
dates-tile-guess = ≈ { $date }
dates-summary-none = Intet sat endnu
dates-summary-ends = Slutter ved afsnit { $episode }
dates-summary-record =
    { $count ->
        [one] { $count } dateret af { $site }
       *[other] { $count } dateret af { $site }
    }
dates-summary-record-unnamed =
    { $count ->
        [one] { $count } dateret af kilden
       *[other] { $count } dateret af kilden
    }
dates-summary-set =
    { $count ->
        [one] { $count } sat af dig
       *[other] { $count } sat af dig
    }
dates-summary-guessed =
    { $count ->
        [one] { $count } gættet ugentligt
       *[other] { $count } gættet ugentligt
    }
dates-told-one = Afsnit { $episode }
dates-told-many = Afsnit { $first } til { $last }
dates-told-undated = { $episodes } har ingen dato endnu
dates-told-due = { $episodes } forventes om { $when }
dates-told-ends = Slutter ved afsnit { $episode }
dates-told-season-ends = Sæson { $season } slutter ved afsnit { $episode }
dates-told-by-you = { $told }, sat af dig
dates-told-gave-way =
    { $count ->
        [one] Du satte { $count } afsnit; det gælder ikke længere
       *[other] Du satte { $count } afsnit; de gælder ikke længere
    }

## Seriens fakta
series-runtime =
    { $hours ->
        [0] { $minutes } min
       *[other] { $minutes ->
            [0] { $hours } t
           *[other] { $hours } t { $minutes } min
        }
    }
series-runtime-each = { $runtime } hver
series-status-airing = Sendes
series-status-finished = Afsluttet
series-status-upcoming = Kommer
series-status-cancelled = Aflyst
series-status-hiatus = På pause
series-average-score = Gennemsnit { $score }
series-imdb = IMDb { $score }
series-imdb-votes = IMDb { $score } fra { $votes } stemmer
series-rewatched =
    { $count ->
        [one] Set igen { $count } gang
       *[other] Set igen { $count } gange
    }

## Om, bedømmelser
about-ratings = Bedømmelser
about-ratings-credit = Oplysninger stillet til rådighed af IMDb (https://www.imdb.com). Brugt med tilladelse.

## Markér afsnit på seriesiden
mark-menu-watched = Markér som set indtil her
mark-menu-unwatched = Markér som uset herfra
mark-menu-cancel = Annuller
mark-watched-up-to = Set til og med afsnit { $episode }
mark-unwatched-from = Uset fra afsnit { $episode }
mark-not-connected = { $reason } Log ind under Mere, Konti.
series-not-tracked = Ikke tracket
series-not-tracked-on-phone =
    { $count ->
        [one] Ikke tracket, { $count } afsnit på denne telefon
       *[other] Ikke tracket, { $count } afsnit på denne telefon
    }

## Afspiller
player-speed-not-while-casting = Ikke under cast

## Downloads
downloads-other-release-on-disk = Anden udgivelse på disk

## Month patterns

dates-month-short = %b %Y
dates-month-long = %B %Y
