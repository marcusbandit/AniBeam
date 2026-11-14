# da translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = Hjem
nav-library = Bibliotek
nav-airing = Sendes nu
nav-feed = Feed
nav-watching = Ser nu
nav-stats = Statistik
nav-metadata = Serieinfo
nav-downloads = Downloads
nav-settings = Indstillinger
nav-series = Serie
nav-player = Afspiller
nav-show = Serie
nav-recommended = Anbefalet

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } fejl
   *[other] { $count } fejl
}

## ScanSummary

scan-summary-nothing = Intet at vise endnu
scan-summary-found = Fandt { $count ->
    [one] { $count } serie
   *[other] { $count } serier
}, { $recognised } genkendt
scan-summary-found-need = Fandt { $count ->
    [one] { $count } serie
   *[other] { $count } serier
}, { $recognised } genkendt, { $need } kræver dig
scan-summary-episodes = { $count ->
    [one] { $count } afsnit
   *[other] { $count } afsnit
} på disken.
scan-summary-episodes-watched = { $count ->
    [one] { $count } afsnit
   *[other] { $count } afsnit
} på disken, { $watched } allerede set på { $tracker }.
scan-summary-title = Din mappe er tilføjet
scan-summary-count-found = fundet
scan-summary-count-recognised = genkendt
scan-summary-count-need = kræver dig
scan-summary-later = Senere
scan-summary-match = Match de { $count }
scan-summary-open-library = Åbn biblioteket

## UpdateNotice

update-notice-check-failed = AniBeam kunne ikke tjekke for opdateringer. Tjek din internetforbindelse, og prøv igen.
update-notice-up-to-date = AniBeam er opdateret
update-notice-downloading = Downloader AniBeam { $version }
update-notice-ready = AniBeam { $version } er klar
update-notice-failed = Opdateringen mislykkedes
update-notice-on-aur = AniBeam { $version } er på AUR
update-notice-out = AniBeam { $version } er udkommet
update-notice-out-titled = AniBeam { $version } “{ $title }” er udkommet
update-notice-progress = { $percent } %  ·  { $got } af { $total } MB
update-notice-restart-hint = Genstart nu, eller den åbner næste gang
update-notice-from-source = Denne kopi er bygget fra kildekoden: pull og byg igen
update-notice-no-download = Ingen download her til denne computer
update-notice-install-hint = Opdater installerer den og genstarter AniBeam
update-notice-retry = Prøv igen
update-notice-update = Opdater
update-notice-restart = Genstart
update-notice-copied = Kopieret
update-notice-copy = Kopier
update-notice-release-page = Udgivelsesside
update-notice-later = Senere

## WhatsNew

whats-new-title = Nyheder
whats-new-got-it = Forstået

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = d. MMM HH:mm
activity-drawer-stage-library = Bibliotek
activity-drawer-stage-metadata = Metadata
activity-drawer-stage-trackers = Trackere
activity-drawer-stage-franchise = Franchise
activity-drawer-stage-playback = Afspilning
activity-drawer-stage-store = Lager
activity-drawer-stage-system = System
activity-drawer-stage-downloads = Downloads
activity-drawer-level-info = Info
activity-drawer-level-warn = Advarsel
activity-drawer-level-error = Fejl
activity-drawer-copied-line = Kopierede linjen
activity-drawer-title = Aktivitet
activity-drawer-everything = Alt
activity-drawer-problems = Problemer
activity-drawer-copied = Kopieret
activity-drawer-copy = Kopier
activity-drawer-clear-every = Ryd alle linjer
activity-drawer-copy-tip = Klik for at kopiere linjen
activity-drawer-fold = Luk
activity-drawer-empty-technical = Intet logget endnu.
activity-drawer-empty-problems = Ingen problemer. Alt virkede.
activity-drawer-empty = Intet at rapportere endnu. Det, AniBeam gør for dig, vises her.

## ActivityColumn

activity-column-episode-run = Afsnit { $first } til { $last }
activity-column-episode-list = Afsnit { $list }
activity-column-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
activity-column-batch = Pakkedownload
activity-column-download = Download
activity-column-name-batch = { $title }, en pakke
activity-column-name-episode = { $title }, afsnit { $number }
activity-column-title = Aktivitet
activity-column-down = ned
activity-column-up = op
activity-column-not-connected = { $client } er ikke forbundet, så intet bevæger sig her.
activity-column-connect-again = Forbind igen
activity-column-downloading = Downloader nu
activity-column-fewer = Færre
activity-column-more = Og { $count } mere
activity-column-nothing-downloading = Intet downloades.
activity-column-finished-today = Færdige i dag
activity-column-today = I dag
activity-column-today-empty = Søgninger, downloads og nye abonnementer vises her, efterhånden som de sker.
activity-column-stopping = Stopper
activity-column-starting = Starter
activity-column-start-tip = Start download
activity-column-stop-tip = Stop download; det hentede bliver
activity-column-cancelling = Annullerer
activity-column-cancel-all-tip = Annuller disse downloads
activity-column-cancel-tip = Annuller dette download
activity-column-size-of = { $got } af { $total }
activity-column-files-missing = Filer mangler
activity-column-time-format = HH:mm
activity-column-left = { $time } tilbage
activity-column-waiting-peers = Venter på peers
activity-column-episodes-done = { $count ->
    [one] { $done } af { $count } afsnit færdigt
   *[other] { $done } af { $count } afsnit færdige
}
activity-column-cube-skipped = Afsnit { $number }: sprunget over
activity-column-cube-done = Afsnit { $number }: færdigt
activity-column-cube-progress = Afsnit { $number }: { $percent } %

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Klar
frame-strip-progress = { $doing }, { $done } af { $total }
frame-strip-idle = Intet kører
frame-strip-time-format = HH:mm
frame-finish-sign-in = Gør login på { $tracker } færdigt først
frame-did-not-work = Det virkede ikke.
frame-could-not-open = AniBeam kunne ikke åbne det. Kopier det og åbn det selv.
frame-copy = Kopier
frame-copied = Kopieret
frame-status-refused = { $tracker } tog ikke imod det
frame-status-refused-why = { $tracker } tog ikke imod det: { $why }
frame-lists-refused = { $tracker } tog ikke imod { $count ->
    [one] 1 serie
   *[other] { $count } serier
}; se aktivitetsloggen
frame-lists-moved = Flyttede { $count ->
    [one] 1 serie
   *[other] { $count } serier
} til { $tracker }
frame-notes-refused = { $tracker } tog ikke imod dine noter
frame-notes-refused-why = { $tracker } tog ikke imod dine noter: { $why }
frame-notes-saved = Noter gemt
frame-connected = Forbundet med { $tracker } som { $user }
frame-sign-in-stopped = Login på { $tracker } stoppede, før det blev færdigt.
frame-toast-hide-details = Skjul detaljer
frame-toast-details = Detaljer
frame-toast-details-tip = Vis præcis, hvad der gik galt, og kopier det
frame-forward = Frem
frame-copy-magnet = Kopier magnet
frame-copy-link = Kopier torrentlink
frame-magnet-copied = Magnet kopieret
frame-link-copied = Link kopieret
frame-open-release-page = Åbn udgivelsesside
frame-open-folder = Åbn mappe
frame-copy-path = Kopier sti
frame-path-copied = Sti kopieret
frame-mark-unwatched = Markér som uset herfra
frame-mark-watched = Markér som set indtil her
frame-copy-title = Kopier titel
frame-open-on-anilist = Åbn på AniList
frame-list-home-online = Ændr det på hver liste, der er forbundet
frame-list-home-unmatched = Gemt på denne enhed: ingen tracker har denne serie
frame-list-home-not-carried = Gemt på denne enhed: din tracker har ikke denne serie
frame-list-home-offline = Gemt på denne enhed, indtil du forbinder AniList eller MyAnimeList
frame-moving-to = Flytter til { $list }
frame-moved-to = Flyttet til { $list }
frame-plan-added = { $title } sættes på din Planlagt-liste
frame-plan-listed = { $title } er allerede på din liste
frame-delete-watched = { $kind ->
    [film] Slet sete film
   *[episode] Slet sete afsnit
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] Slet filmen
       *[episode] Slet afsnittet
    }
   *[other] { $kind ->
        [film] Slet alle { $count } film
       *[episode] Slet alle { $count } afsnit
    }
}
frame-series-gone = Den serie er væk
frame-delete-none-watched = Ingen sete afsnit af { $title } på disken
frame-delete-none = Intet af { $title } på disken
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } set film
           *[other] { $count } sete film
        }
       *[episode] { $count ->
            [one] { $count } set afsnit
           *[other] { $count } sete afsnit
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } film
           *[other] { $count } film
        }
       *[episode] { $count ->
            [one] { $count } afsnit
           *[other] { $count } afsnit
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } ekstra
   *[other] { $count } ekstra
}
frame-delete-and = { $first } og { $second }
frame-delete-heading = Slet { $what } af { $title }?
frame-delete-base = Filerne og deres undertekster flyttes til papirkurven, så du kan hente dem tilbage derfra. Deres torrents fjernes fra torrentprogrammet, så de kan downloades igen.
frame-delete-confirm = Flyt til papirkurven
frame-delete-done = { $what } af { $title } flyttet til papirkurven
frame-delete-watched-done = { $count ->
    [one] { $count } afsnit
   *[other] { $count } afsnit
}, { $size }, flyttet til papirkurven
frame-delete-nothing = Intet at slette
frame-fix-not-drafted = Rettet. Fejlrapporten kunne ikke skrives
frame-fix-agent = Rettet. { $agent } kigger på det, og fejlrapporten åbner, når den er færdig
frame-fix-opening = Rettet. Åbner fejlrapporten
frame-fix-not-saved = Rettelsen blev ikke gemt: { $error }
frame-fix-undone = { $count ->
    [one] { $count } fil
   *[other] { $count } filer
} læses ud fra navnet igen
frame-fix-done = Rettede, hvordan { $count ->
    [one] { $count } fil
   *[other] { $count } filer
} læses
frame-report-failed = Fejlrapporten kunne ikke skrives: { $error }
frame-report-untriaged = Fejlrapporten er åben i din browser, uden triage. { $error }
frame-report-open = Fejlrapporten er åben i din browser, klar til at sende
frame-downloads-not-read = Ikke læst.

## HomePage

home-title = Hjem
home-scan-started = Scanning startet
home-ago-minutes = { $count } min. siden
home-ago-hours = { $count } t siden
home-ago-days = { $count } d siden
home-today = I dag
home-tomorrow = I morgen
home-yesterday = I går
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d. MMM
home-day-date-format = ddd d. MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = Ingen dato endnu
home-unit-days = { $count ->
    [one] dag
   *[other] dage
}
home-unit-hours = t
home-unit-minutes = m
home-week-late = { $count ->
    [one] { $count } afsnit er forsinket
   *[other] { $count } afsnit er forsinkede
}
home-week-waiting = { $count ->
    [one] { $count } afsnit klar til at se
   *[other] { $count } afsnit klar til at se
}
home-week-coming = { $count ->
    [one] { $count } afsnit de næste syv dage
   *[other] { $count } afsnit de næste syv dage
}
home-week-next = Næste: { $title } om { $time }
home-countdown-days = { $days } d { $hours } t
home-countdown-hours = { $hours } t { $minutes } m
home-countdown-minutes = { $minutes } m
home-out-not-downloaded = Ikke downloadet endnu
home-out-downloading = Downloader
home-out-when = { $day } { $time }
home-next-up = Næste
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, anslået tid
home-next-when-season = Sæson { $season }, { $span }, { $time }
home-next-when-season-estimated = Sæson { $season }, { $span }, { $time }, anslået tid
home-no-date = Ingen dato
home-tile-watched = set
home-tile-downloading = downloader
home-tile-out = udkom { $ago }
home-tile-not-out = ikke udkommet
home-tile-estimated = anslået
home-tile-season-span = S{ $season } { $span }
home-tile-to-watch = { $count } at se
home-day-nothing = Intet
home-now = Nu { $time }
home-card-aired = { $episode } udkom { $ago }
home-continue = Se videre
home-continue-fact = { $count } serier at se videre i
home-on-hold = På pause
home-on-hold-fact = { $count } serier at vende tilbage til
home-hero-open = Åbn { $title }
home-hero-pick-up = Fortsæt, hvor du stoppede
home-hero-up-next = Næste
home-hero-downloading = Downloader { $count ->
    [one] { $count } afsnit
   *[other] { $count } afsnit
}
home-hero-min-left = { $count } min. tilbage
home-hero-watched = { $watched } af { $total } set
home-hero-resume = Fortsæt afsnit { $number }
home-hero-play = Afspil afsnit { $number }
home-hero-resume-unnumbered = Fortsæt afsnit
home-hero-play-unnumbered = Afspil afsnit
home-downloaded = Downloadet for nylig
home-downloaded-fact = { $count } serier med nye filer de sidste 14 dage
home-week = Denne uge
home-week-show-empty = Vis tomme dage
home-week-hide-empty = Skjul tomme dage
home-week-airing = Sendes nu
home-week-out = Udkommet nu
home-week-coming-up = Kommer snart
home-recommended = Anbefalet til dig
home-recommended-fact = Ud fra din egen liste
home-recommended-scope = Anbefalet fra
home-recommended-all = Alle
home-recommended-in-library = I biblioteket
home-recommended-on-list = På listen
home-recommended-none-library = Der er intet tilbage at anbefale i dit bibliotek. Alle finder serier, du ikke har endnu.
home-recommended-none-list = Der er intet tilbage at anbefale på dine lister. Alle finder serier, du ikke har på en liste.
home-catch-up = Indhent
home-catch-up-fact = { $count } serier med afsnit, der venter
home-fresh = Start noget nyt
home-fresh-fact = { $count } serier ikke startet
home-genres = Genrer
home-open-library = Åbn bibliotek
home-dropped = Droppet
home-dropped-fact = { $count ->
    [one] { $count } fil stadig på disken
   *[other] { $count } filer stadig på disken
}
home-out-now = Udkommet nu
home-out-now-fact = Nye afsnit denne uge, fra AniList
home-popular = Populært denne sæson
home-add-folder-title = Dine egne serier hører også til her
home-add-folder-body = Tilføj mappen med din anime, så fyldes Hjem med det, du ser.
home-add-folder-button = Tilføj en mappe
home-empty-scanned-title = Intet fundet i dine mapper endnu
home-empty-title = Ingen serier endnu
home-empty-scanned-body = AniBeam fandt ingen serier eller film i de mapper, den holder øje med. Scan dem igen, eller tjek mapperne i Indstillinger.
home-empty-body = Tilføj mappen med din anime, så fylder biblioteket sig selv.
home-scan-all = Scan alle
home-library-settings = Biblioteksindstillinger
home-failed-title = Hjem kunne ikke indlæses
home-failed-body = AniBeam kunne ikke læse biblioteket lige nu. Prøv igen om lidt.

## Onboarding

onboarding-language = Sprog
onboarding-step-welcome = Velkommen
onboarding-step-library = Dit bibliotek
onboarding-step-lists = Din liste
onboarding-step-downloads = Downloads
onboarding-step-ready = Klar
onboarding-scan-reading = Læser mappen
onboarding-scan-found = Fandt { $count } serier. Fortsæt, når du vil; resten udfyldes af sig selv.
onboarding-scan-starter-empty = Din nye mappe er tom indtil videre. Serier, du lægger i den, dukker op her af sig selv.
onboarding-scan-no-video = Der dukkede ingen videofiler op. Tjek, at det er den rigtige mappe.
onboarding-scan-your-shows = dine serier
onboarding-scan-looking-up = Slår { $what } op
onboarding-scan-looking-up-tally = Slår { $what } op, { $posters } af { $count } fundet
onboarding-scan-posters = Henter plakater
onboarding-scan-posters-tally = Henter plakater, { $posters } af { $count } fundet
onboarding-scan-seasons = Finder relaterede sæsoner
onboarding-scan-seasons-tally = Finder relaterede sæsoner, { $posters } af { $count } fundet
onboarding-choose-folder = Vælg din animemappe
onboarding-about = Et minuts opsætning. Dit bibliotek bliver færdigt af sig selv.
onboarding-step-later = Senere
onboarding-skip-setup = Spring opsætning over
onboarding-title-welcome = Velkommen til AniBeam
onboarding-title-library = Hvor er din anime?
onboarding-title-lists = Forbind din animeliste
onboarding-title-downloads = Sådan virker downloads
onboarding-title-ready = Du er klar
onboarding-title-later = { $count ->
    [1] Klar, én ting til senere
    [2] Klar, to ting til senere
   *[other] Klar, tre ting til senere
}
onboarding-body-welcome = AniBeam samler din anime ét sted. Den finder serierne på din disk, afspiller dem lige her og holder AniList eller MyAnimeList opdateret, mens du ser.
onboarding-body-library = Vælg den mappe, du har dine serier i. Hver mappe inden i den bliver en serie med plakat, afsnit og stedet, hvor du slap. AniBeam læser kun mappen. Den flytter eller omdøber aldrig en fil.
onboarding-body-lists = Log ind én gang, så bliver hvert afsnit, du ser færdigt her, markeret på din liste, uden at du skal skrive noget. Login sker i din browser, så AniBeam ser aldrig din adgangskode.
onboarding-body-downloads = Følg en serie på siden Downloads, så lander hvert nyt afsnit i dit bibliotek, når det bliver sendt. Det kræver to ting: et sted at søge og et torrentprogram til at hente.
onboarding-body-ready = Alt her kan ændres senere i Indstillinger, og Indstillinger, Om kører denne opsætning igen.
onboarding-skip-step = Spring over for nu
onboarding-get-started = Kom i gang
onboarding-open-app = Åbn AniBeam
onboarding-continue = Fortsæt
onboarding-payoff-library = Dit bibliotek
onboarding-payoff-library-line = Plakater, afsnit og hvad der kommer næste gang, for hver serie i dine mapper.
onboarding-payoff-plays = Afspiller lige her
onboarding-payoff-plays-line = Undertekster, spring intro over, og den husker, hvor du stoppede.
onboarding-payoff-list = Din liste, synkroniseret
onboarding-payoff-list-line = Se et afsnit færdigt, så tæller det på AniList eller MyAnimeList.
onboarding-payoff-episodes = Afsnit kommer af sig selv
onboarding-payoff-episodes-line = Følg en serie, så lander hvert nyt afsnit i dit bibliotek.
onboarding-films-too = En mappe med film virker også: kald den Movies.
onboarding-choose-folder-button = Vælg mappe
onboarding-no-folder = Jeg har ikke en mappe endnu
onboarding-starter-where = Det opretter en Anime-mappe i { $dir } og bruger den.
onboarding-add-another = Tilføj en mappe mere
onboarding-shelf-more = mere
onboarding-preview-show = Din serie
onboarding-preview-finished = Afsnit 3 set færdigt her
onboarding-preview-on-list = På din liste: 3 set
onboarding-preview-set-by = Sat af AniBeam, så snart rulleteksterne starter
onboarding-lists-connected = Forbundet. Det, du ser her, tæller nu på din liste.
onboarding-lists-no-account = Ingen konto endnu? Begge er gratis, og én er rigeligt. Du kan afbryde forbindelsen når som helst i Indstillinger, Konti.
onboarding-search-title = Et sted at søge
onboarding-search-ready = { $count ->
    [one] { $count } kilde klar til søgning.
   *[other] { $count } kilder klar til søgning.
}
onboarding-search-none = AniBeam kommer uden egne kilder. Du tilføjer dem fra et kilderepo: et link, som nogen deler med dig, og som du indsætter i Indstillinger, Downloads.
onboarding-sources-after = Åbner efter opsætningen
onboarding-sources-add = Tilføj et repo efter opsætningen
onboarding-fetch-title = Hvem henter
onboarding-fetch-auto = Lad AniBeam klare det
onboarding-fetch-recommended = Anbefalet
onboarding-fetch-auto-line = AniBeam har sin egen qBittorrent med og kører den stille i baggrunden. Intet at installere, intet at udfylde.
onboarding-fetch-existing = Jeg bruger allerede qBittorrent
onboarding-fetch-existing-line = AniBeam forbinder til den qBittorrent, du allerede kører. Du skriver dens adresse og login i Indstillinger bagefter.
onboarding-ready-library = Bibliotek
onboarding-ready-library-none = Ingen mappe endnu. Den findes i Indstillinger, Bibliotek, når du vil.
onboarding-ready-library-series = { $count ->
    [one] { $count } mappe
   *[other] { $count } mapper
}, { $series } serier indtil videre
onboarding-ready-library-empty = { $count ->
    [one] { $count } mappe
   *[other] { $count } mapper
}, tom indtil videre
onboarding-ready-library-scanning = { $count ->
    [one] { $count } mappe
   *[other] { $count } mapper
}, scanner
onboarding-ready-lists = Din liste
onboarding-ready-lists-as = { $tracker }, som { $user }
onboarding-ready-lists-none = Ikke forbundet endnu. Det findes i Indstillinger, Konti, når du vil.
onboarding-ready-downloads = Downloads
onboarding-ready-downloads-line = { $sources ->
    [0] Ingen kilder endnu
    [one] { $sources } kilde
   *[other] { $sources } kilder
}, { $client ->
    [own] din egen qBittorrent.
   *[bundled] AniBeams egen qBittorrent.
} { $next ->
    [ready] Alt er klar.
    [opens] Indstillinger, Downloads åbner bagefter.
   *[both] Begge findes i Indstillinger, Downloads.
}
onboarding-ready-open-downloads = Åbn Indstillinger, Downloads
onboarding-ready-set-up = Opsæt nu

## Rail.qml

rail-home = Hjem
rail-library = Bibliotek
rail-airing = Sendes
rail-feed = Feed
rail-stats = Statistik
rail-show-info = Serieinfo
rail-downloads = Downloads
rail-settings = Indstillinger
rail-downloads-complete =
    { $count ->
        [one] Downloads: { $count } abonnement er færdigt
       *[other] Downloads: { $count } abonnementer er færdige
    }
rail-title-language-name = Titelsprog: { $language }. Tryk for det næste
rail-title-language-english = Engelsk
rail-title-language-tip = Titelsprog: romaji, engelsk eller japansk
rail-whats-new-in = Nyheder i { $release }
rail-whats-new = Nyheder
rail-settings-note = Indstillinger: { $note }

## FixSheet.qml

fix-kind-episode = Afsnit
fix-kind-opening = Intro
fix-kind-ending = Outro
fix-kind-preview = Forsmag
fix-kind-special = Special
fix-kind-other = Andet
fix-reads-season-episode = Sæson { $season } afsnit { $number }
fix-reads-extra = { $kind } { $index }
fix-apply-failed = Kunne ikke anvende rettelserne.
fix-title = Ret, hvordan filer læses
fix-lede = { $title }. Sig, hvad hver fil i virkeligheden er; alle senere scanninger beholder det.
fix-rescan = Genscan mappe
fix-rescan-started = Læser mappen igen
fix-filled-in = Udfyldt
fix-fill-in = Udfyld
fix-column-file = Fil
fix-column-reads-as = Læses som
fix-column-should-be = Skal være
fix-fixed = Rettet
fix-number-placeholder = Nr.
fix-index-placeholder = Ingen
fix-keep = Behold rettelsen
fix-leave-as-read = Lad filen blive læst, som den er
fix-read-by-name = Læs filen ud fra navnet igen
fix-report = Opret en fejlrapport på GitHub
fix-report-body = Åbner en udfyldt fejlrapport, som du kan sende. Den er offentlig og nævner disse filer.
fix-report-body-agent = Åbner en udfyldt fejlrapport, som du kan sende. Den er offentlig og nævner disse filer. { $agent } kigger på den først.
fix-close-unapplied =
    { $count ->
        [one] Luk uden at anvende { $count } rettelse?
       *[other] Luk uden at anvende { $count } rettelser?
    }
fix-number-invalid = Et rødt tal skal rettes først
fix-close-anyway = Luk alligevel
fix-keep-editing = Bliv ved
fix-apply-count =
    { $count ->
        [one] Anvend { $count } rettelse
       *[other] Anvend { $count } rettelser
    }

## MatchPanel.qml

match-matched-to = Matchet med { $title }
match-applied = Match anvendt
match-found-tvmaze = Fandt serien på TVmaze
match-found-anilist = Fandt den på AniList
match-found-anilist-season = Fandt den på AniList, sæson { $season }
match-found-mal = Fandt den på MyAnimeList
match-sites-and = { $first } og { $next }
match-matched-on = Matchet på { $sites }
match-provider-silent = { $provider } svarede ikke
match-title-walk = Match, { $walk }
match-title = Match
match-skip = Spring over
match-refresh-info = Opdater info
match-refreshing = Opdaterer { $title }
match-clear-confirm = Ryd det: fjerner linket og det satte antal afsnit
match-clear = Ryd match
match-cleared = Match ryddet
match-search-placeholder = Søg på { $provider }
match-link-tvmaze = Indsæt et TVmaze- eller IMDb-link
match-link-anilist = Indsæt et AniList- eller MyAnimeList-link
match-look-up = Slå op
match-searching = Søger på { $provider }
match-type-more = Skriv mindst to bogstaver for at søge på { $provider }.
match-no-results = Ingen titler fundet på { $provider }.

## WatchingPage.qml

watching-episode-code = Afs. { $number }
watching-countdown-days = { $days } d { $hours } t
watching-countdown-hours = { $hours } t { $minutes } m
watching-countdown-minutes = { $minutes } m
watching-on-disk = På disken
watching-on-anilist = På AniList
watching-ready-to-start = Klar til start
watching-not-in-library = Ikke i biblioteket
watching-get-to-start = Hent afsnit for at starte
watching-anilist-updated = AniList opdateret { $when }
watching-airing = Sendes
watching-next-in = Næste: { $title } om { $wait }
watching-episodes-waiting = Afsnit venter
watching-series-behind =
    { $count ->
        [one] { $count } serie bagud
       *[other] { $count } serier bagud
    }
watching-up-to-date = Helt med
watching-anilist-only = Kun på AniList
watching-planning-summary-off = Det, du planlægger at se på AniList.
watching-planning-summary =
    { $count ->
        [one] { $count } serie planlagt, { $here } på disken.
       *[other] { $count } serier planlagt, { $here } på disken.
    }
watching-summary-off = Det, du ser eller genser på AniList.
watching-summary-waiting =
    { $count ->
        [one] { $count } serie har afsnit, der venter.
       *[other] { $count } serier har afsnit, der venter.
    }
watching-caught-up = Du er helt med.
watching-start-watching = Begynd at se
watching-play-next = Afspil næste
watching-open-on-anilist = Åbn på AniList
watching-refreshing = Opdaterer
watching-connect-title = AniList er ikke forbundet
watching-connect-body = Forbind den, så viser denne side de serier, du ser eller genser på AniList, med Afspil på dem, du har, og en nedtælling på dem, der sendes.
watching-connect-waiting = Venter på browseren
watching-connect-anilist = Forbind AniList
watching-load-failed-title = Ser nu-listen kunne ikke indlæses
watching-load-failed-body = AniList kunne ikke nås. Listen kommer igen ved næste opdatering.
watching-planning-failed-title = Planlagt-listen kunne ikke indlæses
watching-planning-failed-body = Dens cache kunne ikke læses. Prøv igen, ellers kommer den igen ved næste opdatering.
watching-planning-empty-title = Intet på din Planlagt-liste
watching-planning-empty-body = Planlæg under Sendes nu, eller Planlagt i en series listemenu, sætter en serie her.
watching-airing-now = Sendes nu
watching-empty-title = Intet på din Ser nu-liste
watching-empty-body = Markér en serie som Ser nu på AniList, så lander den her.
watching-open-anilist = Åbn AniList

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] { $count } serie skjult
       *[other] { $count } serier skjult
    }
library-hidden-sync-stops =
    { $count ->
        [one] { $count } serie skjult, og dens synkronisering med trackeren stopper
       *[other] { $count } serier skjult, og deres synkronisering med trackeren stopper
    }
library-unhidden =
    { $count ->
        [one] { $count } serie tilbage og synkroniseres igen
       *[other] { $count } serier tilbage og synkroniseres igen
    }
library-undo = Fortryd
library-clearing-score =
    { $count ->
        [one] Rydder karakteren for { $count } serie
       *[other] Rydder karakteren for { $count } serier
    }
library-rating =
    { $count ->
        [one] Giver { $count } serie karakteren { $score }
       *[other] Giver { $count } serier karakteren { $score }
    }
library-moving =
    { $count ->
        [one] Flytter { $count } serie til { $list }
       *[other] Flytter { $count } serier til { $list }
    }
library-rescanning =
    { $count ->
        [one] Genscanner { $count } serie
       *[other] Genscanner { $count } serier
    }
library-finding-matches =
    { $count ->
        [one] Leder efter et match til { $count } serie
       *[other] Leder efter et match til { $count } serier
    }
library-sort-last-viewed = Set for nylig
library-sort-downloaded = Downloadet for nylig
library-sort-title = Titel
library-sort-progress = Fremgang
library-sort-average-score = Gennemsnit
library-sort-my-score = Min karakter
library-dir-a-to-z = A til Å
library-dir-z-to-a = Å til A
library-dir-oldest = Ældste først
library-dir-newest = Nyeste først
library-dir-least-watched = Mindst set først
library-dir-most-watched = Mest set først
library-dir-lowest = Laveste først
library-dir-highest = Højeste først
library-group-none = Ingen
library-group-season = Sæson
library-group-type = Type
library-group-genre = Genre
library-group-watch-state = Status
library-scan-started = Scanning startet
library-tab-all = Alle
library-tab-series = Serier
library-tab-movies = Film
library-tab-watched = Set
library-tab-hidden = Skjulte
library-count-movies =
    { $count ->
        [one] { $count } film
       *[other] { $count } film
    }
library-count-watched = { $count } set
library-count-hidden = { $count } skjult
library-count-series =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serier
    }
library-title = Bibliotek
library-all-genres = Alle genrer
library-search-placeholder = Søg i titler eller mapper
library-sort = Sortér
library-group = Gruppér
library-franchises-together = Franchises samlet
library-watched-shows = Sete serier
library-load-failed-title = Biblioteket kunne ikke indlæses
library-no-matches = Ingen resultater for "{ $query }"
library-none-in-genre =
    { $what ->
        [series] Ingen { $genre }-serier her
        [films] Ingen { $genre }-film her
        [watched] Ingen sete { $genre }-serier her
        [hidden] Ingen skjulte { $genre }-serier her
       *[shows] Ingen { $genre }-serier her
    }
library-none-here =
    { $what ->
        [series] Ingen serier her
        [films] Ingen film her
        [watched] Ingen sete serier her
        [hidden] Ingen skjulte serier her
       *[shows] Ingen serier her
    }
library-load-failed-body = AniBeam kunne ikke læse biblioteket lige nu. Prøv igen om lidt.
library-no-matches-body = Tjek stavningen, eller søg i hele biblioteket fra Alle.
library-none-in-genre-body = Intet i denne visning har den genre. Alle genrer viser resten.
library-none-in-tab-body = Dit bibliotek har ingen af disse endnu. Alle viser alt i det.
library-all-filtered-body = Alt i dit bibliotek er filtreret fra i denne visning.
library-clear-search = Ryd søgning
library-show-all = Vis alle
library-picked = { $count } valgt
library-pick-all = Alle
library-pick-all-tip = Vælg alle viste kort (Ctrl+A)
library-unhide = Vis igen
library-hide = Skjul
library-rate = Bedøm
library-rate-tip = Én karakter til alle valgte, på hver forbundet tracker
library-needs-tracker = Kræver en forbundet tracker og en matchet serie
library-move-to-list = Flyt til liste
library-rescan = Genscan
library-find-match = Find et match
library-find-match-tip = Søg efter hver valgt serie uden et match
library-all-matched = Alle valgte har et match
library-empty-scanned-title = Intet fundet i dine mapper endnu
library-empty-title = Dit bibliotek er tomt
library-empty-scanned-body = AniBeam fandt ingen serier eller film i de mapper, den holder øje med. Scan dem igen, eller tjek mapperne i Indstillinger.
library-empty-body = Tilføj mappen med din anime, så fylder biblioteket sig selv, eller importer en AniBeam-eksport.
library-scan-all = Scan alle
library-add-folder = Tilføj en mappe
library-settings = Biblioteksindstillinger
library-import = Importer
library-import-title = Importer en AniBeam-eksport
library-import-filter-export = AniBeam-eksport (*.json)
library-import-filter-all = Alle filer (*)
library-import-started = Import startet

## MetadataPage.qml

metadata-title = Serieinfo
metadata-filter-all = Alle { $count }
metadata-filter-series = Serier { $count }
metadata-filter-films = Film { $count }
metadata-filter-missing = Mangler { $count }
metadata-scan-started = Scanning startet
metadata-sort-added = Tilføjet
metadata-sort-season = Sæson
metadata-sort-a-to-z = A til Å
metadata-section-unmatched = Ikke genkendt
metadata-section-missing = Mappe ikke fundet
metadata-section-incomplete = Mangler afsnit
metadata-section-empty = Ikke downloadet
metadata-section-airing = Sendes
metadata-section-complete = Komplet
metadata-no-info = Ingen serieinfo endnu
metadata-state-aired = { $have } af { $aired } sendt
metadata-state-of = { $have } af { $expected }
metadata-state-of-extra = { $have } af { $expected } +{ $extra }
metadata-detail-none-aired = Ingen af de { $aired } sendte afsnit er på disken
metadata-detail-aired = { $have } af { $aired } sendte på disken
metadata-detail-aired-missing = { $have } af { $aired } sendte på disken, { $missing } mangler
metadata-detail-none = Ingen af de { $expected } afsnit er på disken
metadata-detail-missing = { $have } af { $expected } på disken, { $missing } mangler
metadata-walk-position = { $at } af { $total }
metadata-walk-done = Det var den sidste
metadata-walk-left =
    { $count ->
        [one] Én tilbage under Mangler match
       *[other] { $count } tilbage under Mangler match
    }
metadata-countdown-days = { $days } d { $hours } t
metadata-countdown-hours = { $hours } t { $minutes } m
metadata-countdown-minutes = { $minutes } m
metadata-next-in = Næste om { $wait }
metadata-files-of = { $have } af { $expected }
metadata-files-on-disk = Filer på disken
metadata-forgot = Fjernede { $title }
metadata-refreshed-failed = Opdaterede { $refreshed }, { $failed } kunne ikke opdateres
metadata-refreshed =
    { $count ->
        [one] Opdaterede { $count } serie
       *[other] Opdaterede { $count } serier
    }
metadata-found =
    { $count ->
        [one] Fandt { $count } serie
       *[other] Fandt { $count } serier
    }
metadata-found-some-left =
    { $count ->
        [one] Fandt { $count } serie, { $unmatched } stadig ikke genkendt
       *[other] Fandt { $count } serier, { $unmatched } stadig ikke genkendt
    }
metadata-found-nothing = Fandt intet nyt, { $unmatched } stadig ikke genkendt
metadata-all-recognised = Alle serier er genkendt
metadata-count =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serier
    }
metadata-search-placeholder = Filtrer titler
metadata-find-rest = Find resten
metadata-find-rest-started = Slår de mapper op, AniBeam endnu ikke kender
metadata-find-rest-tip = Slår hver mappe op, som AniBeam endnu ikke kender, på hver infokilde i den rækkefølge, Indstillinger angiver
metadata-refresh-all = Opdater alle
metadata-refresh-all-started = Opdaterer info om alle serier
metadata-refresh-all-tip = Henter info om hver genkendt serie igen
metadata-crawling = Finder franchiselinks
metadata-crawl-progress = { $done } af { $total }
metadata-load-failed-title = Serieinfo kunne ikke indlæses
metadata-empty-scanned-title = Intet fundet i dine mapper endnu
metadata-no-matches = Ingen resultater for "{ $query }"
metadata-nothing-under =
    { $filter ->
        [Series] Intet under Serier
        [Movies] Intet under Film
        [MissingFiles] Intet under Mangler
       *[other] Intet under { $filter }
    }
metadata-load-failed-body = AniBeam kunne ikke læse biblioteket lige nu. Prøv igen om lidt.
metadata-empty-scanned-body = AniBeam fandt ingen serier i de mapper, den holder øje med. Scan dem igen, eller tjek mapperne i Indstillinger.
metadata-empty-body = Dit bibliotek er tomt. Tilføj mappen med din anime, så scanner AniBeam den.
metadata-no-matches-body = Ingen titel i denne visning har de ord.
metadata-nothing-under-body = Ingen af dine serier passer til dette filter lige nu. Alle viser resten.
metadata-scan-all = Scan alle
metadata-add-folder = Tilføj en mappe
metadata-clear-search = Ryd søgning
metadata-show-all = Vis alle
metadata-refreshing = Opdaterer { $title }
metadata-refresh-tip = Hent seriens info igen
metadata-forget-armed = Fjern den
metadata-forget-confirm-tip = Klik igen for at fjerne serien og dens visningshistorik fra AniBeam
metadata-forget-tip = Mappen findes ikke længere på disken: fjern serien og dens historik fra AniBeam

## AiringPage.qml

airing-title = Sendes nu
airing-sort-popular = Popularitet
airing-sort-score = Karakter
airing-sort-trending = Trending
airing-sort-favourites = Favoritter
airing-sort-soonest = Næste afsnit
airing-sort-aired = Sendte afsnit
airing-sort-episodes = Antal afsnit
airing-sort-start = Startdato
airing-sort-duration = Afsnitslængde
airing-sort-title = Titel
airing-dir-most-popular = Mest populære først
airing-dir-least-popular = Mindst populære først
airing-dir-lowest = Laveste først
airing-dir-highest = Højeste først
airing-dir-least-trending = Mindst trending først
airing-dir-most-trending = Mest trending først
airing-dir-fewest = Færrest først
airing-dir-most = Flest først
airing-dir-soonest = Snarest først
airing-dir-latest = Senest først
airing-dir-oldest = Ældste først
airing-dir-newest = Nyeste først
airing-dir-shortest = Korteste først
airing-dir-longest = Længste først
airing-dir-a-to-z = A til Å
airing-dir-z-to-a = Å til A
airing-season =
    { $season ->
        [Winter] Vinter { $year }
        [Spring] Forår { $year }
        [Summer] Sommer { $year }
        [Fall] Efterår { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = { $minutes } min. siden
airing-ago-hours = { $hours } t siden
airing-ago-days = { $days } d siden
airing-episode-out = { $episode } udkom { $ago }
airing-no-date = Ingen dato endnu
airing-in-library = I biblioteket
airing-out-today =
    { $count ->
        [one] { $count } afsnit udkommet i dag
       *[other] { $count } afsnit udkommet i dag
    }
airing-dismissed-one = { $title } vises ikke her igen
airing-dismissed =
    { $count ->
        [one] { $count } serie vises ikke her igen
       *[other] { $count } serier vises ikke her igen
    }
airing-planned-already-one = { $title } er allerede på din liste
airing-planned-already-all = De er allerede alle på din liste
airing-planned =
    { $count ->
        [one] { $count } serie sættes på din Planlagt-liste
       *[other] { $count } serier sættes på din Planlagt-liste
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } serie sættes på din Planlagt-liste, { $already } var der allerede
       *[other] { $count } serier sættes på din Planlagt-liste, { $already } var der allerede
    }
airing-play-next = Afspil næste
airing-plan-to-watch = Planlæg
airing-get-more-episodes = Hent flere afsnit
airing-get-episodes = Hent afsnit
airing-open-on-anilist = Åbn på AniList
airing-not-interested = Ikke interesseret
airing-not-interested-count = { $count } ikke interesseret
airing-not-interested-tip = Serier, du har markeret som uinteressante, i Indstillinger
airing-refreshing = Opdaterer
airing-pick-season = Vælg en anden sæson
airing-filter-all = Alt
airing-filter-lists = På mine lister
airing-filter-library = I mit bibliotek
airing-hide-lists = Skjul mine lister
airing-hide-lists-tip = Udelad serierne på dine AniList- eller MAL-lister
airing-hide-library = Skjul mit bibliotek
airing-hide-library-tip = Udelad de serier, du har afsnit af
airing-out-now = Udkommet nu
airing-out-last-day =
    { $count ->
        [one] { $count } afsnit det seneste døgn
       *[other] { $count } afsnit det seneste døgn
    }
airing-out-fewer = Færre
airing-out-all = Alle { $count }
airing-sort = Sortér
airing-load-failed-title = AniList kunne ikke nås
airing-load-failed-body = Serierne, der sendes, kommer fra AniList. De kommer igen ved næste opdatering.
airing-reading-title = Læser, hvad der sendes
airing-nothing-now = Intet sendes lige nu
airing-nothing-for = Intet sendes i { $season }
airing-nothing-read = Intet læst endnu
airing-reading-body = Alle serier, der sendes, fra AniList. Det tager et par sekunder første gang.
airing-nothing-body = AniList har ingen serier for denne sæson endnu. Opdater læser den igen, eller vælg en anden sæson.
airing-nothing-read-body = Opdater læser alle serier, der sendes, fra AniList.
airing-hides-empty-title = Dine filtre efterlader intet her
airing-lists-empty-title = Intet her er på dine lister
airing-library-empty-title = Intet her er i dit bibliotek
airing-hides-empty-body = Alle serier her er på dine lister eller i dit bibliotek, og dem har du skjult.
airing-lists-empty-body = Planlæg på et kort sætter serien på din liste.
airing-library-empty-body = Hent afsnit på et kort downloader serien til dit bibliotek.
airing-show-everything = Vis alt
airing-picked = { $count } valgt
airing-undo = Fortryd

## EpisodeRow.qml

episode-row-date-time-format = ddd d. MMM, HH:mm
episode-row-date-upcoming-format = ddd d. MMM
episode-row-date-format = d. MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Klik for at markere som uset herfra
episode-row-mark-watched-tip = { $state }. Klik for at markere som set indtil her
episode-row-next-up = Næste
episode-row-versions =
    { $count ->
        [one] { $count } version
       *[other] { $count } versioner
    }
episode-row-versions-tip =
    { $count ->
        [one] Samme afsnit fra { $count } udgivelse. Klik for at vælge, hvilken der afspilles
       *[other] Samme afsnit fra { $count } udgivelser. Klik for at vælge, hvilken der afspilles
    }
episode-row-downloading = Downloader { $percent } %
episode-row-get = Hent afsnit
episode-row-predicted = Forventet
episode-row-extra = Ekstra
episode-row-resume = Fortsæt ved { $time }

## EpisodeCubes.qml

episode-cubes-one = Ét afsnit
episode-cubes-range =
    { $count ->
        [one] { $count } afsnit, { $first } til { $last }
       *[other] { $count } afsnit, { $first } til { $last }
    }
episode-cubes-tip = Afsnit { $number }: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } ekstra
       *[other] { $count } ekstra
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } umatchet fil
       *[other] { $count } umatchede filer
    }
episodes-pane-on-disk = På disken
episodes-pane-not-on-disk = Ikke på disken
episodes-pane-season = S{ $season }
episodes-pane-get-tip = Hent afsnit { $number }
episodes-pane-none-on-disk = Ingen afsnit på disken
episodes-pane-change-match = Skift match
episodes-pane-refreshing = Opdaterer { $title }

## ShowDetail.qml

show-detail-not-in-library = Denne mappe er ikke i biblioteket endnu. Når en scanning finder den, vises dens plakat og afsnit her.
show-detail-film-on-disk = Filmen er på disken.
show-detail-film-not-on-disk = Filmen er ikke på disken endnu.
show-detail-complete-line = Færdig med at blive sendt, og alle { $total } afsnit er her. Der kommer ikke mere, så abonnementet har intet tilbage at gøre.
show-detail-finished = Færdig med at blive sendt.
show-detail-airing = Sendes.
show-detail-have-of-total = { $have } af { $total } afsnit er her.
show-detail-have =
    { $count ->
        [one] { $count } afsnit er her.
       *[other] { $count } afsnit er her.
    }
show-detail-next-predicted =
    { $count ->
        [one] { $span } ventes om { $wait }.
       *[other] { $span } ventes om { $wait }.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span } sendes om { $wait }.
       *[other] { $span } sendes om { $wait }.
    }
show-detail-missing =
    { $count ->
        [one] Afsnit { $list } er her ikke.
       *[other] Afsnit { $list } er her ikke.
    }
show-detail-missing-more = Afsnit { $list } og { $more } mere er her ikke.
show-detail-downloaded = Downloadet
show-detail-downloading = Downloader { $percent } %
show-detail-taken-adding = Taget af reglen, tilføjes
show-detail-taken = Taget af reglen
show-detail-left-out = Udeladt af reglen
show-detail-subscribed-reading = Abonnerer. qBittorrent læser feedet.
show-detail-taken-moving = Reglen tager { $taken } af feedets { $releases ->
        [one] { $releases } udgivelse
       *[other] { $releases } udgivelser
    }; { $moving ->
        [one] { $moving } downloades
       *[other] { $moving } downloades
    } nedenfor.
show-detail-taken-held = Reglen tager { $taken } af feedets { $releases ->
        [one] { $releases } udgivelse
       *[other] { $releases } udgivelser
    }; { $torrents ->
        [one] { $torrents } torrent er
       *[other] { $torrents } torrents er
    } i programmet.
show-detail-taken-adding-line = Reglen tager { $taken } af feedets { $releases ->
        [one] { $releases } udgivelse
       *[other] { $releases } udgivelser
    }; qBittorrent tilføjer { $taken ->
        [one] den.
       *[other] dem.
    }
show-detail-feed-holds-nothing = Feedet har intet endnu. Udgivelser, der passer, downloades, så snart de dukker op i det.
show-detail-nothing-fits = Intet i feedet passer til reglen endnu. Udgivelser, der passer, downloades, så snart de dukker op.
show-detail-complete = Komplet
show-detail-open-in-library = Åbn i biblioteket
show-detail-feed-finds-nothing = Dette feed finder intet
show-detail-feed-empty-query = Det spørger { $source } efter “{ $query }”, som kommer tomt tilbage, så intet nyt kan nå reglen. En kortere søgning giver som regel svar; de ord, den udelader, kan blive i reglen i stedet.
show-detail-feed-empty = Det spørger { $source }, som kommer tomt tilbage, så intet nyt kan nå reglen. En kortere søgning giver som regel svar; de ord, den udelader, kan blive i reglen i stedet.
show-detail-feed-empty-earlier = qBittorrent viser stadig, hvad en tidligere version af dette feed fandt.
show-detail-repairing = Reparerer
show-detail-repair = Reparer
show-detail-unsubscribe = Afmeld
show-detail-remove-torrents =
    { $count ->
        [one] Fjern torrenten
       *[other] Fjern { $count } torrents
    }
show-detail-searches = Søger
show-detail-quoted = “{ $words }”
show-detail-whole-feed = hele feedet
show-detail-from = Fra
show-detail-rule = Regel
show-detail-rule-unfollowable = Kan ikke hente nye afsnit: kilden svarer på søgninger, men har intet feed
show-detail-rule-active = Aktiv
show-detail-keeps = Beholder
show-detail-keeps-value = udgivelser med “{ $words }”
show-detail-drops = Udelader
show-detail-drops-value = udgivelser med { $words }
show-detail-or = { $first } eller { $second }
show-detail-last-took = Tog senest
show-detail-saves-to = Gemmer i
show-detail-folder = Mappe
show-detail-in-the-feed = I feedet
show-detail-feed-reading = Læser feedet.
show-detail-feed-offers = Hvad feedet tilbyder nu, nyeste først.
show-detail-feed-is-empty = Feedet er tomt.
show-detail-read-again = Læs igen
show-detail-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seedere
    }
show-detail-download = Download
show-detail-torrents = Torrents
show-detail-torrents-moving =
    { $count ->
        [one] { $count } downloades nu; bjælkerne fyldes undervejs.
       *[other] { $count } downloades nu; bjælkerne fyldes undervejs.
    }
show-detail-torrents-held = Hvad programmet har for denne mappe.
show-detail-torrents-waiting = Venter på, at qBittorrent tilføjer det, reglen tager.
show-detail-torrents-none = Programmet har intet for denne mappe.

## SeriesPage.qml

series-trackers-both = { $first } og { $second }
series-count-cleared = Antal afsnit ryddet
series-count-set = Sat til { $count } afsnit
series-held-one-undated = Afsnit { $first } har ingen dato endnu
series-held-range-undated = Afsnit { $first } til { $last } har ingen dato endnu
series-held-one-due = Afsnit { $first } ventes i { $when }
series-held-range-due = Afsnit { $first } til { $last } ventes i { $when }
series-held-month-format = MMMM yyyy
series-fix-schedule = Afsnit og datoer
series-fix-files = Filer
series-fix-rescan-folder = Genscan mappe
series-fix-match = Match
series-find-match = Find et match
series-rescan-show = Genscan serie
series-open-in-show-info = Åbn i Serieinfo
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } film flyttet til papirkurven
           *[other] { $count } film flyttet til papirkurven
        }
        [episode] { $count ->
            [one] { $count } afsnit flyttet til papirkurven
           *[other] { $count } afsnit flyttet til papirkurven
        }
       *[file] { $count ->
            [one] { $count } fil flyttet til papirkurven
           *[other] { $count } filer flyttet til papirkurven
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } film valgt.
           *[other] { $count } film valgt.
        }
        [episode] { $count ->
            [one] { $count } afsnit valgt.
           *[other] { $count } afsnit valgt.
        }
       *[file] { $count ->
            [one] { $count } fil valgt.
           *[other] { $count } filer valgt.
        }
    }
series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] Flyt { $count } film til papirkurven
           *[other] Flyt { $count } film til papirkurven
        }
        [episode] { $count ->
            [one] Flyt { $count } afsnit til papirkurven
           *[other] Flyt { $count } afsnit til papirkurven
        }
       *[file] { $count ->
            [one] Flyt { $count } fil til papirkurven
           *[other] Flyt { $count } filer til papirkurven
        }
    }
series-delete-click = Klik på de filer, der skal slettes.
series-delete-where = De flyttes til papirkurven, hvor du kan hente dem tilbage. Det, du har set, forbliver registreret.
series-delete-move = Flyt til papirkurven
series-pick-none = Vælg ingen
series-pick-all = Vælg alle
series-rescan-started = Genscanning startet
series-rated = Bedømt { $score }
series-rated-cleared = Bedømmelse ryddet
series-score-failed = Karakteren mislykkedes
series-still-downloading = { $episode } downloades stadig
series-format-series = Serie
series-tracker-refused = { $tracker } tog ikke imod det
series-tracker-refused-why = { $tracker } tog ikke imod det: { $why }
series-watched-up-to = Set til og med { $episode }
series-unwatched-from = Uset fra { $episode }
series-undo = Fortryd
series-play-downloading = Downloader afsnit { $number }
series-play-downloading-unnumbered = Downloader afsnit
series-play-continue = Fortsæt
series-play-again = Se igen
series-play-continue-episode = Fortsæt afsnit { $number }
series-play-again-from = Se igen fra afsnit { $number }
series-play-episode = Afspil afsnit { $number }
series-page-not-tracked = Ikke tracket
series-not-tracked-on-disk =
    { $count ->
        [one] Ikke tracket, { $count } afsnit på disken
       *[other] Ikke tracket, { $count } afsnit på disken
    }
series-watched = { $count } set
series-watched-of = { $watched } af { $total } set
series-watched-of-estimate = { $watched } af { $total }+ set
series-facts-episodes-estimate =
    { $count ->
        [one] { $count }+ afsnit
       *[other] { $count }+ afsnit
    }
series-facts-each = { $runtime } hver
series-facts-on-disk = { $count } på disken
series-next = Næste: { $episode }
series-episodes-downloading =
    { $count ->
        [one] { $count } afsnit downloades
       *[other] { $count } afsnit downloades
    }
series-all-watched = Alt set
series-no-poster = Ingen plakat
series-next-season-span = S{ $season } { $span }
series-next-predicted = { $span } ventes { $date }, om { $wait }
series-next-date-format = ddd d. MMM
series-next-airs =
    { $count ->
        [one] { $span } sendes om { $wait }
       *[other] { $span } sendes om { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] Downloader { $count } afsnit
       *[other] Downloader { $count } afsnit
    }
series-count-season-ends = Sæson { $season } slutter ved afsnit { $number }
series-count-ends = Slutter ved afsnit { $number }
series-count-gave-way = Du satte { $count } afsnit; det gælder ikke længere
series-count-set-by-you = { $line }, sat af dig
series-change = Ændr
series-get-episodes = Hent afsnit
series-your-score = Din karakter { $score }
series-rate = Bedøm
series-page-average-score = Gennemsnit { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = { $votes } stemmer
series-add-to-list = Føj til en liste
series-page-rewatched =
    { $count ->
        [one] Genset { $count } gang
       *[other] Genset { $count } gange
    }
series-your-notes = Dine noter
series-add-notes = Tilføj noter
series-notes-tip = Noter på din listepost, gemt på { $sites }
series-unhide = Vis igen
series-hide = Skjul
series-hidden-toast = { $title } er skjult her, og AniBeam stopper med at synkronisere den med AniList og MyAnimeList
series-unhidden-toast = { $title } er tilbage og synkroniseres igen
series-hide-tip-tracked = Skjult på alle sider her, og AniBeam stopper med at synkronisere den med AniList og MyAnimeList
series-hide-tip = Skjult på alle lister
series-fixes = Rettelser
series-fixes-tip = Ret filer, antal afsnit, matchet, eller genscan
series-quick-fix = Hurtig rettelse
series-film = Film
series-episodes = Afsnit
series-extras = Intro, outro og mere
series-extras-openings = Introer
series-extras-endings = Outroer
series-extras-previews = Forsmag og trailere
series-extras-specials = Specials
series-extras-other = Andet
series-extras-group = { $kind }, { $count }
series-extra-files = Ekstra filer
series-extra-files-warning =
    { $count ->
        [one] Én fil er ud over de { $total } forventede afsnit: forkert navngivet, dubletter eller specials. Omdøb eller fjern den.
       *[other] { $count } filer er ud over de { $total } forventede afsnit: forkert navngivet, dubletter eller specials. Omdøb eller fjern dem.
    }
series-synopsis = Handling
series-less = Mindre
series-more = Mere
series-match = Match
series-not-matched = Ikke matchet endnu, så der er ingen plakat, handling eller tracking. Find dens serieinfo på AniList eller TVmaze.
series-naming-title = Navne, der matcher
series-naming-body = Mappen er seriens titel, som AniList skriver den, med sæsonen efter, når det ikke er den første. Hver fil har afsnitsnummeret.
series-fix-how-files-read = Ret, hvordan filer læses
series-characters = Figurer
series-related = Relateret
series-related-filling = Hentes stadig fra AniList.
series-tags = Tags
series-hide-spoilers = Skjul spoilere
series-spoilers = Spoilere
series-all-tags = Alle tags
series-tag = { $name }, { $rank } %
series-recommendations = Anbefalinger
series-relation-owned = I dit bibliotek
series-relation-unreleased = Endnu ikke udgivet
series-relation-on-anilist = På AniList
series-score-saved = Karakter gemt
series-score-cleared = Karakter ryddet

## GetShow.qml

get-show-the-client = torrentprogrammet
get-show-group-batch = { $group }s pakke
get-show-run = { $first } til { $last }
get-show-episodes =
    { $count ->
        [one] Afsnit { $list }
       *[other] Afsnit { $list }
    }
get-show-episodes-inline =
    { $count ->
        [one] afsnit { $list }
       *[other] afsnit { $list }
    }
get-show-sending-to = Sender til { $client }
get-show-go =
    { $count ->
        [one] Hent { $count } afsnit
       *[other] Hent { $count } afsnit
    }
get-show-go-replace =
    { $count ->
        [one] Hent { $count } afsnit og erstat
       *[other] Hent { $count } afsnit og erstat
    }
get-show-go-subscribe =
    { $count ->
        [one] Hent { $count } afsnit og abonnér
       *[other] Hent { $count } afsnit og abonnér
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] Hent { $count } afsnit, erstat og abonnér
       *[other] Hent { $count } afsnit, erstat og abonnér
    }
get-show-subscribe = Abonnér på nye afsnit
get-show-nothing-new = Intet nyt at hente
get-show-pick-episode = Vælg et afsnit
get-show-replace-off = Fra. De valgte for { $episodes } udelades, da biblioteket har dem. Slå det til for at bytte dem.
get-show-replace-on = Dine filer til { $episodes } flyttes til papirkurven, når de nye er færdige.
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } downloades nu: det download annulleres, og det, det skrev, flyttes til papirkurven.
       *[other] { $episodes } downloades nu: det download annulleres, og det, det skrev, flyttes til papirkurven.
    }
get-show-kind-on-demand = Startes, når en søgning kræver det
get-show-kind-rss-search = RSS-søgning
get-show-kind-rss-feed = RSS-feed
get-show-kind-json-search = JSON-søgning
get-show-kind-indexer-search = Indekssøgning
get-show-ready = Klar
get-show-running = Kører
get-show-not-running = Kører ikke
get-show-not-running-start = Kører ikke. Klik på kilden ovenfor for at starte den.
get-show-switched-off = Slået fra
get-show-click-to-see = Klik for at se, hvad den har af denne serie
get-show-searching = Søger
get-show-searching-source = Søger i { $name }
get-show-searching-elapsed = { $label }, { $seconds } s
get-show-releases =
    { $count ->
        [one] { $count } udgivelse
       *[other] { $count } udgivelser
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } udgivelse, { $cover } af { $total } afsnit dækket
       *[other] { $count } udgivelser, { $cover } af { $total } afsnit dækket
    }
get-show-on-your-list = På din liste
get-show-plan-to-watch = Planlæg
get-show-on-your-list-tip = Den er på din liste
get-show-plan-to-watch-tip = Sæt den på din Planlagt-liste
get-show-open-series-tip = Seriens side: detaljer, afsnit og afspilning
get-show-next-in = Afsnit { $number } om { $wait }
get-show-airing = Sendes
get-show-finished = Afsluttet
get-show-total-episodes = { $count } afsnit
get-show-downloading-episodes =
    { $count ->
        [one] Downloader { $count } afsnit
       *[other] Downloader { $count } afsnit
    }
get-show-downloading-torrents =
    { $count ->
        [one] Downloader { $count } torrent
       *[other] Downloader { $count } torrents
    }
get-show-in-library = I dit bibliotek
get-show-not-in-library = Ikke i dit bibliotek
get-show-downloading = Downloader
get-show-stop-all = Stop alle
get-show-start-all = Start alle
get-show-stop-all-tip = Stop alle downloads af denne serie; det hentede bliver
get-show-start-all-tip = Start alle downloads af denne serie igen
get-show-cancel-all = Annuller alle
get-show-cancel-all-tip = Annuller alle downloads af denne serie og flyt det, de skrev, til papirkurven
get-show-sources = Tilgængelige kilder
get-show-check-again = Tjek igen
get-show-check-again-tip = Spørger hver kilde, der startes efter behov, om den kører. Starter intet.
get-show-search-source = Søg i { $name }
get-show-release-list = Alle udgivelser som en liste
get-show-your-library = Dit bibliotek
get-show-library-on-disk = { $have } af { $total } på disken
get-show-library-downloading = { $count } downloades
get-show-library-nothing = Intet endnu
get-show-row-batch = { $group }, pakke
get-show-batch-counting = tæller afsnit, én torrent
get-show-batch-unlisted = afsnit ikke angivet, hele serien efter titel, én torrent
get-show-batch-episodes =
    { $count ->
        [one] { $count } afsnit, én torrent
       *[other] { $count } afsnit, én torrent
    }
get-show-nothing-new-picked = Intet nyt valgt
get-show-nothing-picked = Intet valgt
get-show-pick-provider = Vælg en kilde
get-show-plan-line = { $episodes } fra { $groups } på { $source }.
get-show-and = { $first } og { $second }
get-show-library-has =
    { $count ->
        [one] Biblioteket har { $episodes }. Slå Erstat nuværende afsnit til nedenfor for at bytte det.
       *[other] Biblioteket har { $episodes }. Slå Erstat nuværende afsnit til nedenfor for at bytte dem.
    }
get-show-together = samlet
get-show-torrents-unit =
    { $count ->
        [one] torrent
       *[other] torrents
    }
get-show-replace = Erstat nuværende afsnit
get-show-sequential = Download i rækkefølge
get-show-sequential-on = Til: hver ny torrent henter sine stykker fra starten, så de første afsnit bliver færdige først, og du kan se, mens resten kommer. Lidt langsommere i alt.
get-show-sequential-off = Fra: programmet henter stykker i den rækkefølge, swarmen tilbyder dem.
get-show-rolling = Hold de næste klar
get-show-rolling-on = Til: { $count } afsnit downloades nu, og hvert afsnit, du ser færdigt, starter det næste.
get-show-rolling-on-delete = Til: { $count } afsnit downloades nu, og hvert afsnit, du ser færdigt, starter det næste. Sete afsnit fjernes fra disken.
get-show-rolling-off = Fra: alle valgte afsnit downloades på én gang.
get-show-follow-off = Fra: nye afsnit vises her som huller, og intet downloades af sig selv.
get-show-follow-no-feed = { $source } har intet feed at følge, så nye afsnit kan ikke downloades af sig selv derfra.
get-show-follow-pick-group = Vælg en gruppe først: et abonnement følger én gruppe og kvalitet.
get-show-follow-batch-only = { $group } har kun pakken her, så der er intet feed med enkelte afsnit at abonnere på.
get-show-follow-on = Til: et feed for { $feed } på { $source }. Hvert nyt afsnit downloades, når det dukker op.
get-show-confirm =
    { $count ->
        [one] { $episodes } downloades nu. Erstatter du det, annulleres det download, og det, det skrev, flyttes til papirkurven.
       *[other] { $episodes } downloades nu. Erstatter du dem, annulleres det download, og det, det skrev, flyttes til papirkurven.
    }
get-show-confirm-go = Annuller og erstat
get-show-state-error = Fejl
get-show-state-files-missing = Filer mangler
get-show-state-stopped = Stoppet
get-show-state-waiting-peers = Venter på peers
get-show-state-metadata = Henter metadata
get-show-state-queued = I kø
get-show-state-checking = Tjekker filer
get-show-state-allocating = Reserverer plads
get-show-state-moving = Flytter filer
get-show-state-waiting = Venter
get-show-a-batch = En pakke
get-show-no-group = Gruppe ikke i navnet
get-show-likely = sandsynligvis { $source }
get-show-tracker = tracker { $tracker }
get-show-not-known = Ukendt endnu
get-show-bytes-of = { $done } af { $size }
get-show-speed-down = { $speed }/s ned
get-show-time-left = { $wait } tilbage
get-show-stopping = Stopper
get-show-starting = Starter
get-show-stop = Stop
get-show-start-tip = Start download igen
get-show-stop-tip = Stop download; det hentede bliver
get-show-cancelling = Annullerer
get-show-cancel-tip = Annuller dette download og flyt det, det skrev, til papirkurven
get-show-episode-order = Download i afsnitsrækkefølge
get-show-episode-order-on = Til: stykkerne hentes fra starten af hver fil, så de første afsnit bliver færdige først.
get-show-cube-skipped = Afsnit { $number }: sprunget over
get-show-cube-done = Afsnit { $number }: færdigt
get-show-cube-percent = Afsnit { $number }: { $percent } %
get-show-fact-torrent = Torrent
get-show-fact-release = Udgivelse
get-show-fact-from = Fra
get-show-fact-saving-to = Gemmer i
get-show-span-counting = Tæller
get-show-span-unknown = { $runs }?
get-show-span-episodes = Afsnit { $runs }
get-show-cube-no-release = Afsnit { $number }: ingen udgivelse fra { $group }
get-show-seeders = { $count } seedere
get-show-cube-on-disk = allerede på disken, klik for at erstatte det
get-show-cube-downloading = downloades nu, klik for at erstatte det

## TorrentRow

torrent-row-state-downloading = Downloader
torrent-row-state-stalled = Går i stå
torrent-row-state-metadata = Henter metadata
torrent-row-state-queued = I kø
torrent-row-state-stopped = Stoppet
torrent-row-state-done = Færdig
torrent-row-state-seeding = Seeder
torrent-row-state-checking = Tjekker
torrent-row-state-moving = Flytter
torrent-row-state-error = Fejl
torrent-row-state-files-missing = Filer mangler
torrent-row-state-making-room = Gør plads
torrent-row-state-finding-peers = Finder peers
torrent-row-state-waiting = Venter
torrent-row-done-of-size = { $done } af { $size }
torrent-row-speed-down = { $speed }/s ned
torrent-row-speed-up = { $speed }/s op
torrent-row-time-left = { $time } tilbage
torrent-row-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seedere
    }
torrent-row-rolling = løbende, afsnit { $next } til { $through } af { $total }
torrent-row-stop = Stop

## SourceRow

source-row-unavailable-fix = Tilslut drevet, og vælg Genscan. Hvis mappen er flyttet, så fjern den og tilføj det nye sted.
source-row-counts = { $count } serier
source-row-counts-movies =
    { $movies ->
        [one] { $count } serier, { $movies } filmmappe
       *[other] { $count } serier, { $movies } filmmapper
    }
source-row-unavailable = Utilgængelig
    .tip = AniBeam kan ikke nå denne mappe. Ligger den på et drev, der ikke er tilsluttet eller monteret, så tilslut det, og AniBeam finder den igen. Er den flyttet, så fjern den her og tilføj det nye sted.
source-row-rescan = Genscan
source-row-remove-question = Fjern { $name }, { $count } serier og deres historik?

## StartSourceSheet

start-source-title = Start { $service } til { $name }?
start-source-busy = Starter { $service }. { $name } slås til, når den svarer; det kan tage op til et minut.
start-source-body = { $service } er tjenesten, der søger i { $name } for AniBeam. Den kører i baggrunden på denne computer og bliver ved med at køre, når AniBeam lukker.
start-source-starting = Starter

## SearchProgress

search-progress-why-helper = AniBeam starter sin søgehjælper; det tager et sekund eller to.
search-progress-why-starting = { $endpoint } kører gennem en hjælper, der starter, når en søgning har brug for den. Første søgning venter på den i op til { $seconds } sekunder; de næste springer det over.
search-progress-why-reading = Deler udgivelserne op i grupper og tjekker dem mod biblioteket.
search-progress-why-ondemand = En hjælper læser { $endpoint } forfra ved hver ny søgning, hvilket som regel tager ti til tyve sekunder.
search-progress-why-slow = { $endpoint } er langsom til at svare lige nu.
search-progress-why-usual = Det tager som regel et sekund eller to.
search-progress-why-and-slow = { $why } Det tager længere tid end normalt; annuller for at prøve en anden kilde.
search-progress-searching = Søger
search-progress-seconds = { $seconds } s
search-progress-try = Prøv { $endpoint }

## ReleaseTable

release-table-col-release = Udgivelse
release-table-col-group = Gruppe
release-table-col-res = Opl.
release-table-col-rip = Rip
release-table-col-ep = Afs.
release-table-col-size = Størrelse
release-table-col-seeders = Seedere
release-table-col-leechers = Leechere
release-table-col-downloads = Downloads
release-table-col-released = Udgivet
release-table-tick-all = Markér alle udgivelser
release-table-untick-all = Fjern markering fra alle udgivelser
release-table-on-disk = På disken
release-table-other-on-disk = Anden udgivelse på disken
release-table-season-batch = Pakke, sæson { $season }
release-table-batch = Pakke
release-table-via = via { $source }
release-table-season-episode = S{ $season } { $episode }
release-table-date-format = d. MMMM yyyy, HH:mm
release-table-download = Download
release-table-more = og { $count } mere. Indsnævr listen, eller sortér den for at se andre først.
release-table-pick = Vælg denne udgivelse

## DownloadsPage

downloads-title = Downloads
downloads-builtin-client = Indbygget program
downloads-start-done = { $name } kører og er slået til
downloads-disk-film = Film
downloads-disk-watched = { $size } set
downloads-disk-all-watched = alt set
downloads-delete-watched-menu =
    { $count ->
        [one] Alle sete afsnit: { $count } afsnit, { $size }
       *[other] Alle sete afsnit: { $count } afsnit, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Alle afsluttede serier, du har set: { $count } serie, { $size }
       *[other] Alle afsluttede serier, du har set: { $count } serier, { $size }
    }
downloads-titles-and = { $list } og { $last }
downloads-titles-more = { $list } og { $count } mere
downloads-delete-finished-none = Ingen afsluttet serie er set helt til ende
downloads-delete-watched-none = Intet set ligger på disken
downloads-delete-finished-heading =
    { $count ->
        [one] Slet { $count } afsluttet serie, { $size }?
       *[other] Slet { $count } afsluttede serier, { $size }?
    }
downloads-delete-finished-base = { $titles }. Kun serier, som kilden siger er afsluttet, med alle afsnit set; en serie, der stadig sendes, eller hvis slutning er et gæt, bliver. Alle deres filer flyttes til papirkurven, også ekstra. Deres torrents fjernes også fra { $client }, så de kan downloades igen.
downloads-delete-watched-heading =
    { $count ->
        [one] Slet { $count } set afsnit, { $size }?
       *[other] Slet { $count } sete afsnit, { $size }?
    }
downloads-delete-watched-base = Alle afsnit, du har set, i alle serier. Afsnit, du ikke har set endnu, og ekstra bliver. Filerne og deres undertekster flyttes til papirkurven, så du kan hente dem tilbage derfra. Deres torrents fjernes også fra { $client }, så de kan downloades igen.
downloads-move-to-trash = Flyt til papirkurven
downloads-remove-group-torrents =
    { $count ->
        [one] Fjern torrenten
       *[other] Fjern dens { $count } torrents
    }
downloads-run-span = { $from } til { $to }
downloads-run-pair = { $first } og { $second }
downloads-gap-missing =
    { $count ->
        [one] Afsnit { $runs } mangler
       *[other] Afsnit { $runs } mangler
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Afsnit { $runs } ikke downloadet
       *[other] Afsnit { $runs } ikke downloadet
    }
downloads-gap-then-not-downloaded = { $runs } ikke downloadet
downloads-trying-query = Prøver { $query }
downloads-client-settings = Indstillinger for torrentprogram
downloads-error-unknown = Noget gik galt. Prøv igen.
downloads-search-all-off = Alle kilder er slået fra. Slå en til for at søge.
downloads-turn-on = Slå { $name } til
downloads-search-no-sources = Der er ingen kilder at søge i endnu. Tilføj en kildeliste i Indstillinger under Downloads.
downloads-search-none-started = Ingen kilde kunne starte søgningen. Prøv igen om lidt.
downloads-auto-resolution = Indsnævret til { $resolution }, den skarpeste, der er
downloads-search-one-failed = Søgningen på { $name } virkede ikke. Prøv igen, eller søg i en anden kilde.
downloads-search-all-failed = Ingen af kilderne kunne søge. Prøv igen om lidt.
downloads-narrowing-on = Indsnævrer på { $name }
downloads-refine-zero = Det ville ikke efterlade noget, så listen bliver, som den er.
downloads-refine-no-match = Ingen udgivelse passer til det ord.
downloads-refine-empty = Skriv et ord at indsnævre med.
downloads-refine-no-change = Det ville ikke ændre noget.
downloads-movie-tick-one = Markér én udgivelse, og tryk så Download som film.
downloads-group-downloading = Downloader
downloads-group-downloading-line = I gang nu; Aktivitet til højre har detaljerne.
downloads-group-seeding = Seeder
downloads-group-seeding-line = Downloadet og deles stadig.
downloads-group-done = Færdige
downloads-group-done-line = Downloadet; programmet deler dem ikke.
downloads-group-stopped = Stoppet
downloads-group-stopped-line = Stoppet, før de blev færdige.
downloads-group-broken = Filer mangler
downloads-group-broken-line = Programmet kan ikke finde deres filer: de peger på en mappe, der ikke findes. Fjern torrents for at rydde op i programmet.
downloads-sub-nofeed = Kan ikke følges
downloads-sub-nofeed-line = Deres kilde svarer på søgninger, men har intet feed, så der kommer ingen nye afsnit. Afmeld, og følg serien på en kilde, der har et.
downloads-sub-complete = Komplet
downloads-sub-complete-line = Alle afsnit er i biblioteket, og der sendes ikke flere: disse abonnementer har intet tilbage at gøre.
downloads-sub-airing = Sendes
downloads-sub-airing-line = Nye afsnit på vej, de nærmeste først.
downloads-sub-incomplete = Ufuldstændige
downloads-sub-incomplete-line = Færdige med at blive sendt med afsnit, der ikke er her, eller intet siger, hvor serien står.
downloads-sub-missing = Ikke i biblioteket
downloads-sub-missing-line = Deres mappe indeholder intet, biblioteket har fundet: intet er kommet endnu, eller den gemmer et sted, biblioteket ikke kigger.
downloads-torrents =
    { $count ->
        [one] { $count } torrent
       *[other] { $count } torrents
    }
downloads-rule-film-on-disk = Film, på disken
downloads-rule-all-here = Alle { $total } er her
downloads-rule-next =
    { $predicted ->
        [yes] Afs. { $episode } ventes om { $time }
       *[no] Afs. { $episode } om { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] Afs. { $episode } til { $through } ventes om { $time }
       *[no] Afs. { $episode } til { $through } om { $time }
    }
downloads-rule-have-of = { $have } af { $total } her
downloads-rule-have =
    { $count ->
        [one] { $count } afsnit her
       *[other] { $count } afsnit her
    }
downloads-tag-not-downloaded = Ikke downloadet
downloads-tag-missing = { $count } mangler
downloads-tag-n-not-downloaded = { $count } ikke downloadet
downloads-tag-unknown = Ukendt
downloads-speed-down = { $speed }/s ned
downloads-time-left = { $time } tilbage
downloads-repair-trying = Prøver kortere søgninger
downloads-sent-to-client = Sendte { $title } til { $client }
downloads-files-torrents = det, torrents downloadede
downloads-files-torrents-remote = det, torrents downloadede, slettet af { $client }
downloads-files-folder-and = { $folder } og { $torrents }
downloads-unsubscribe-heading = Afmeld { $title }?
downloads-unsubscribe-base = Dens feed og downloadregel fjernes fra { $client }, så der kommer intet nyt.
downloads-unsubscribe-complete-heading = Afmeld alle { $count } komplette serier?
downloads-unsubscribe-complete-base = Deres feeds og downloadregler fjernes fra { $client }. Alle deres afsnit er i biblioteket.
downloads-files-folders-and-torrents = deres mapper og det, deres torrents downloadede
downloads-one-at-a-time = De fjernes én ad gangen; siden viser hver, efterhånden som det sker.
downloads-subscription-stays = { $base } Abonnementet bliver.
downloads-cancel-heading = Annuller { $title }?
downloads-cancel-group-base =
    { $count ->
        [one] Dens download stopper og fjernes fra { $client }.
       *[other] Dens { $count } downloads stopper og fjernes fra { $client }.
    }
downloads-files-so-far-it = det, den har downloadet indtil nu
downloads-files-so-far-they = det, de har downloadet indtil nu
downloads-cancel-download = Annuller download
downloads-keep-downloading = Bliv ved med at downloade
downloads-remove-heading = Fjern { $title }?
downloads-remove-group-base =
    { $count ->
        [one] Dens torrent fjernes fra { $client }; filerne bliver, medmindre du markerer nedenfor.
       *[other] Dens { $count } torrents fjernes fra { $client }; filerne bliver, medmindre du markerer nedenfor.
    }
downloads-remove-detail-base =
    { $count ->
        [one] Dens { $count } torrent fjernes fra { $client }; filerne bliver, medmindre du markerer nedenfor.
       *[other] Dens { $count } torrents fjernes fra { $client }; filerne bliver, medmindre du markerer nedenfor.
    }
downloads-remove-broken-heading = Fjern torrents for alle { $count } serier med manglende filer?
downloads-remove-broken-base = Deres { $count } torrents fjernes fra { $client }. De peger på mapper, der ikke findes, så ingen fil røres; abonnementer bliver.
downloads-remove-torrent-heading = Fjern denne torrent?
downloads-files-it-downloaded = det, den downloadede
downloads-cancel-torrent-heading = Annuller dette download?
downloads-cancel-many-heading =
    { $count ->
        [one] Annuller { $count } download?
       *[other] Annuller { $count } downloads?
    }
downloads-cancel-many-base = Alle downloads af denne serie stopper og fjernes fra { $client }.
downloads-cancel-all = Annuller dem alle
downloads-summary-connecting = Forbinder.
downloads-summary-client-on = { $client } { $version } er forbundet.
downloads-summary-client-off = { $client } er ikke forbundet. Tjek adressen og adgangskoden i Indstillinger under Downloads.
downloads-summary-native = AniBeam søger selv i dine kilder.
downloads-summary-native-with = AniBeam søger selv i dine kilder. { $client }
downloads-summary-starting = Starter downloads.
downloads-summary-not-running = Downloads kører ikke.
downloads-summary-not-started = Downloads er ikke startet endnu.
downloads-summary-helper-ready = Søgehjælperen { $version } er klar. { $client }
downloads-subscribed-as = Abonnerer som { $name }.
downloads-repaired =
    { $count ->
        [one] Reparerede { $name }: feedet finder { $count } udgivelse
       *[other] Reparerede { $name }: feedet finder { $count } udgivelser
    }
downloads-repaired-keeps =
    { $count ->
        [one] Reparerede { $name }: feedet finder { $count } udgivelse, og reglen beholder “{ $term }”
       *[other] Reparerede { $name }: feedet finder { $count } udgivelser, og reglen beholder “{ $term }”
    }
downloads-sent-torrents =
    { $count ->
        [one] Sendte { $count } torrent til { $client }.
       *[other] Sendte { $count } torrents til { $client }.
    }
downloads-added-torrents =
    { $count ->
        [one] Tilføjede { $count } torrent.
       *[other] Tilføjede { $count } torrents.
    }
downloads-trashed-paths =
    { $count ->
        [one] { $count } sti fra { $title } flyttet til papirkurven
       *[other] { $count } stier fra { $title } flyttet til papirkurven
    }
downloads-show-gone = Den serie er ikke længere i biblioteket
downloads-activity-hide = Skjul aktivitet
downloads-activity = Aktivitet
downloads-moving =
    { $count ->
        [one] { $count } download i gang
       *[other] { $count } downloads i gang
    }
downloads-back = Tilbage til Downloads
downloads-chip-search-ready = Søgning klar
downloads-chip-search-starting = Søgning starter
downloads-chip-search-off = Søgning fra
downloads-chip-answers-at = { $summary } Den svarer på { $url }.
downloads-chip-helper-missing = Søgehjælperen er ikke installeret. Klik for indstillingerne for Downloads, som forklarer, hvordan du tilføjer den
downloads-chip-helper-failed = Søgehjælperen stoppede. Forbind igen starter den; indstillingerne for Downloads siger, hvad der gik galt
downloads-chip-open-settings = Åbn indstillingerne for Downloads
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } fra
downloads-chip-client-not-connected = { $client } er ikke forbundet. Forbind igen, eller tjek adressen i Indstillinger
downloads-chip-client-remote = En søgehjælper, du selv kører, bruger det program, dens egne indstillinger angiver
downloads-chip-client-switch = Klik for at skifte program
downloads-connecting = Forbinder
downloads-reconnect = Forbind igen
downloads-search-placeholder = Søg efter en serie eller et udgivelsesnavn
downloads-search-go-tip = Søg i udgivelsesnavne på { $names }
downloads-search-go-off = Alle kilder er slået fra.
downloads-suggest-airing =
    { $predicted ->
        [yes] Sendes, afsnit { $episode } ventes om { $time }
       *[no] Sendes, afsnit { $episode } om { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] Sendes, afsnit { $episode } til { $through } ventes om { $time }
       *[no] Sendes, afsnit { $episode } til { $through } om { $time }
    }
downloads-suggest-finished = Afsluttet
downloads-suggest-episodes = { $count } afsnit
downloads-suggest-in-library = I dit bibliotek
downloads-suggest-on-anilist = På AniList, ikke i dit bibliotek
downloads-plan-to-watch = Planlæg
downloads-suggest-release-search = Søg i udgivelsesnavne efter “{ $query }” på { $names } i stedet
downloads-suggest-release-search-none = Søg i udgivelsesnavne efter “{ $query }” uden nogen kilde i stedet
downloads-key-return = Retur
downloads-sources = Kilder
downloads-source-hint-starting = starter
downloads-source-hint-down = kører ikke
downloads-source-tip-down = { $name } kræver en hjælpetjeneste, der ikke kører. Klik for at starte den.
downloads-source-tip-on = Søgninger spørger { $name }. Klik for at slå den fra.
downloads-source-tip-off = Slået fra. Klik for at spørge { $name } igen.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Tjek
    .tip = Spørger, om hver kilde, der startes efter behov, kører. Starter intet.
downloads-check-busy = Tjekker
downloads-back-to-grid = Tilbage til gitteret
    .tip = Tilbage til { $title } og hvad hver kilde fandt. Escape gør det samme.
downloads-releases-on =
    { $count ->
        [one] { $count } udgivelse på { $name }
       *[other] { $count } udgivelser på { $name }
    }
downloads-found-on = Fundet på
downloads-found-searching = søger
downloads-found-failed = mislykkedes
downloads-found-cancelled = annulleret
downloads-releases =
    { $count ->
        [one] { $count } udgivelse
       *[other] { $count } udgivelser
    }
downloads-found-retry-tip =
    { $why }
    Klik for at prøve igen
downloads-facet-groups = Udgivelsesgruppe
downloads-facet-quality = Kvalitet
downloads-facet-words = Ord
downloads-filters-fewer = Færre filtre
downloads-filters-more = Flere filtre ({ $count })
downloads-narrow-placeholder = Indsnævr med et ord, eller -ord for at udelade
downloads-best-match = Bedste match
    .tip = Beholder de udgivelser, der ligner den bedste: { $query }
downloads-best-match-none = Ingen udgivelse skiller sig ud her
downloads-strip-have-of = { $have } af { $total } afsnit på disken
downloads-strip-have =
    { $count ->
        [one] { $count } afsnit på disken
       *[other] { $count } afsnit på disken
    }
downloads-strip-next =
    { $predicted ->
        [yes] afsnit { $episode } ventes om { $time }
       *[no] afsnit { $episode } om { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] afsnit { $episode } til { $through } ventes om { $time }
       *[no] afsnit { $episode } til { $through } om { $time }
    }
downloads-strip-complete = komplet
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = Ingen af disse udgivelser er til et afsnit, du allerede har.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } udgivelse her er den fil, du har, markeret På disken
       *[other] { $count } udgivelser her er de filer, du har, markeret På disken
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } udgivelse er til et afsnit, du har fra en anden udgivelse
       *[other] { $count } udgivelser er til afsnit, du har fra en anden udgivelse
    }
downloads-strip-both = { $first }; { $second }.
downloads-tick-missing =
    { $count ->
        [one] Markér det { $count } manglende afsnit
       *[other] Markér de { $count } manglende afsnit
    }
    .tip = For hvert afsnit her, som biblioteket ikke har, udgivelsen med flest seedere
downloads-open-in-library = Åbn i biblioteket
downloads-not-this-show = Ikke denne serie
downloads-no-releases = Ingen udgivelser
downloads-no-releases-body = Intet på { $name } passer til “{ $query }”. En anden kilde eller færre ord finder det måske.
downloads-try-source = Prøv { $name }
downloads-ticked =
    { $count ->
        [one] { $count } udgivelse markeret
       *[other] { $count } udgivelser markeret
    }
downloads-nothing-ticked = Intet markeret
downloads-ticked-size = { $size } i alt.
downloads-tick-hint = Markér udgivelser i tabellen, eller download dem alle.
downloads-tick-all = Markér alle
downloads-working = Arbejder
downloads-download-ticked =
    { $count ->
        [one] Download { $count } udgivelse
       *[other] Download { $count } udgivelser
    }
downloads-download-shown = Download de { $count } viste
downloads-download-all = Download alle { $count }
downloads-download-movie = Download som film
    .tip = Én markeret udgivelse, direkte i filmmappen
downloads-subscribe = Abonnér
downloads-subscribe-body = Nye udgivelser, der passer til denne søgning, downloades, når de dukker op, til mappen, der står her.
downloads-name-placeholder = Taget fra den første udgivelse
downloads-subscribe-search = Abonnér på denne søgning
downloads-subscribe-no-feed = Intet feed at abonnere på her
downloads-preview-again = Vis reglen igen
downloads-preview-show = Vis reglen, det giver
downloads-preview-from = Fra { $host }
downloads-preview-from-named = Fra { $host } ({ $name })
downloads-preview-must = Skal indeholde { $terms }
downloads-preview-must-not = Må ikke indeholde { $terms }
downloads-helper-changed = Downloads er sat til en hjælper, der ikke svarer
downloads-helper-missing = Denne kopi af AniBeam kom uden downloads
downloads-helper-failed = Downloads kunne ikke starte
downloads-helper-changed-body = Søgninger og downloads kører gennem et lille hjælpeprogram. Indstillinger peger på et andet end AniBeams eget, og det svarer ikke. Skift tilbage, så starter det af sig selv.
downloads-helper-missing-body = Den er bygget uden hjælpeprogrammet, som søgninger og downloads kører gennem. Installationsfilerne på udgivelsessiden har det. Indstillinger, Downloads forklarer, hvordan du tilføjer det til denne kopi.
downloads-helper-failed-body = Hjælpeprogrammet, som søgninger og downloads kører gennem, stoppede, før det var klar. Som regel hjælper det at starte det igen.
downloads-helper-use-own = Brug AniBeams eget
downloads-helper-start-again = Start igen
downloads-helper-show-how = Vis mig hvordan
downloads-helper-what-went-wrong = Hvad gik galt
downloads-lens-gaps-count = Huller i dit bibliotek  { $count }
downloads-lens-airing-count = Sendes  { $count }
downloads-lens-following-count = Følger  { $count }
downloads-lens-downloaded-count = Downloadet  { $count }
downloads-lens-gaps = Huller i dit bibliotek
downloads-lens-gaps-line = Sendte afsnit, der ikke er på disken. Åbn en serie for at se, hvem der har dem.
downloads-lens-gaps-none = Intet mangler: alle sendte afsnit af alle serier er på disken.
downloads-lens-airing-line = Serier i dit bibliotek med afsnit på vej. Åbn en for at følge den.
downloads-lens-airing-none = Intet i dit bibliotek sendes
downloads-lens-airing-none-body = En serie havner her, når den er i dit bibliotek og har afsnit på vej. Sendes nu viser alt, der sendes.
downloads-airing-now = Sendes nu
downloads-lens-following = Følger
downloads-card-next =
    { $predicted ->
        [yes] Afsnit { $episode } ventes om { $time }
       *[no] Afsnit { $episode } om { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Afsnit { $episode } til { $through } ventes om { $time }
       *[no] Afsnit { $episode } til { $through } om { $time }
    }
downloads-rules-unread = Dine abonnementer kunne ikke læses.
downloads-lens-following-line = Feeds, AniBeam holder øje med, vist som de serier, de fylder.
downloads-none-yet = Ingen endnu.
downloads-reading = Læser.
downloads-rules-error = AniBeam kunne ikke læse dine abonnementer. Prøv igen om lidt.
downloads-removing-to-go = Fjerner, { $count } tilbage
downloads-unsubscribe-all = Afmeld alle { $count }
downloads-unsubscribe = Afmeld
downloads-no-rules = Ingen abonnementer
downloads-no-rules-body = Søg ovenfor, og abonnér for at få en serie til at blive ved med at komme.
downloads-search = Søg
downloads-lens-downloaded = Downloadet
downloads-dl-line = { $size } på disken. { $rest }
downloads-dl-client-off = { $client } er ikke forbundet.
downloads-dl-unread = Downloadlisten kunne ikke læses.
downloads-dl-in-client =
    { $count ->
        [one] { $count } torrent, { $size } i { $client }.
       *[other] { $count } torrents, { $size } i { $client }.
    }
downloads-usage-watched = Set
downloads-usage-not-watched = Ikke set
downloads-deleting = Sletter
downloads-delete-watched = Slet sete
downloads-remove-all = Fjern alle { $count }
downloads-on-disk = På disken
downloads-on-disk-line = I dit bibliotek uden en torrent i { $client }, de største først. Højreklik på en for at slette den.
downloads-nothing-downloaded = Intet downloadet
downloads-nothing-downloaded-body = Det, du downloader eller abonnerer på, vises her, en serie ad gangen.
downloads-client-off-title = { $client } er ikke forbundet
downloads-client-off-body = Tjek, at den kører, og forbind igen. Dens adresse og adgangskode står i Indstillinger under Downloads.
downloads-connect-again = Forbind igen
downloads-view-grid = Gitter
downloads-view-list = Liste
downloads-chip-subscribed = Abonnerer
downloads-speed-up = { $speed }/s op
downloads-broken-some = { $count } med manglende filer
downloads-tag-watched = Set

## PlayerDelays.qml

player-delays-title = Forsinkelse
player-delays-subtitles = Undertekster
player-delays-audio = Lyd
player-delays-subtitles-earlier = Undertekster 0,1 s tidligere
player-delays-subtitles-later = Undertekster 0,1 s senere
player-delays-audio-earlier = Lyd 0,1 s tidligere
player-delays-audio-later = Lyd 0,1 s senere
player-delays-reset = Nulstil til 0 indtil næste fil
player-delays-none = Ingen forsinkelse

## TrackPicker.qml

track-picker-off = Fra

## RollControls.qml

roll-controls-window = Holder { $ahead } klar: afsnit { $next } til { $through } af { $total }
roll-controls-window-watched = Holder { $ahead } klar: afsnit { $next } til { $through } af { $total }, { $watched } set
roll-controls-more = Hent { $count } mere
roll-controls-more-tip = Download de næste afsnit nu. AniBeam holder de næste klar igen, når du har indhentet dem.
roll-controls-fewer-tip = Hold ét færre klar
roll-controls-ahead = { $count } foran
roll-controls-further-tip = Hold ét mere klar
roll-controls-all = Hent alle
roll-controls-all-tip = Stop med at holde nogle få klar, og download alle resterende afsnit
roll-controls-all-question = Download alle { $count } resterende afsnit?

## PlayerChrome.qml

player-chrome-delays = Undertekst- og lydforsinkelse
player-chrome-audio = Lydspor
player-chrome-subtitles = Undertekster
player-chrome-subtitles-off = Undertekster fra
player-chrome-casting-to = Caster til { $device }
player-chrome-cast = Cast til et tv
player-chrome-mark-watched = Markér som set
player-chrome-shortcuts = Tastaturgenveje
player-chrome-fullscreen = Fuld skærm
player-chrome-exit-fullscreen = Afslut fuld skærm
player-chrome-stay = Bliv
player-chrome-skip-intro = Spring intro over
player-chrome-skip-outro = Spring outro over
player-chrome-playing-on = Afspilles på { $device }
player-chrome-replay = Afspil igen

## PlayerPage.qml

player-title = Afspiller
player-next = Næste
player-extra-no-next = En ekstra har intet næste afsnit
player-last-episode = Dette er det sidste afsnit
player-extra-no-previous = En ekstra har intet forrige afsnit
player-first-episode = Dette er det første afsnit
player-resumed-at = Fortsatte ved { $time }
player-start-over = Fra start
player-audio-track = Lyd: { $track }
player-subtitles-track = Undertekster: { $track }
player-subtitles-on = Undertekster: til
player-subtitles-off = Undertekster fra
player-no-subtitles = Denne fil har ingen undertekster
player-delay-seconds = { $value } s
player-audio-delay = Lydforsinkelse { $value }
player-subtitle-delay = Undertekstforsinkelse { $value }
player-volume = Lydstyrke { $volume }
player-speed = Hastighed { $speed }×
player-not-while-casting = Ikke under cast
player-hold-fast = 2× mens du holder
player-no-chapters = Denne fil har ingen kapitler
player-last-chapter = Dette er det sidste kapitel
player-chapter = Kapitel { $number }
player-chapter-titled = Kapitel { $number }: { $title }
player-screenshot-saved = Gemt i Billeder/AniBeam
player-skipped-intro = Sprang intro over
player-skipped-outro = Sprang outro over
player-undo = Fortryd
player-refusal-hidden = serien er skjult
player-refusal-no-match = serien er ikke matchet endnu; match den under Serieinfo
player-refusal-not-newer = din liste er allerede forbi dette afsnit
player-refusal-extra = dette er en ekstra
player-refusal-unmatched = filen er ikke matchet med et afsnit
player-refusal-on-disk = filen findes kun på disken
player-not-tracked-because = Ikke tracket: { $reason }.
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = { $tracker } kunne ikke opdateres
player-outcome-ok = { $tracker } ok
player-outcome-at = { $tracker } ved { $progress }
player-outcome-tracked = Tracket
player-outcome-not-tracked = Ikke tracket
player-outcome-could-not-update = Kunne ikke opdatere din liste
player-outcome-tracked-list = Tracket: { $list }
player-outcome-not-tracked-list = Ikke tracket: { $list }
player-outcome-failed-list = Kunne ikke opdatere din liste: { $list }
player-rated = Bedømt { $score }
player-score-failed = Kunne ikke gemme karakteren
player-frame = Billede { $number } ved { $time }
player-audio-title = Lyd
player-subtitles-title = Undertekster
player-cast-to = Cast til
player-cast-looking = Leder efter Chromecasts og tv
player-cast-unreachable = Kunne ikke nå tv'et
player-cast-tv = TV
player-cast-stop = Stop cast til { $device }
player-cast-connecting = Forbinder
player-cast-loading = Indlæser
player-cast-playing = Afspiller
player-cast-paused = På pause
player-cast-finished = Færdig
player-cast-stopped = Stoppet
player-rate-prompt = Det sidste afsnit er tracket. Vil du bedømme serien?
player-rate = Bedøm
player-not-now = Ikke nu

## EndpointReposPanel.qml

endpoint-repos-panel-title = Kildelister
endpoint-repos-panel-helper = En kildeliste er et signeret sæt steder at søge, tilføjet via sin adresse. AniBeam tjekker signaturen og viser dig nøglen, før noget gemmes, og så installerer du de kilder, du vil have. Installerede kilder kommer med under Søgekilder ovenfor.
endpoint-repos-panel-could-not-start = AniBeam kunne ikke starte det. Prøv igen om lidt.
endpoint-repos-panel-needs-value = { $name } skal have en værdi
endpoint-repos-panel-values-of = Værdier for { $name }
endpoint-repos-panel-secret-note = Gemmes i denne maskines hemmelige lager, aldrig i biblioteket eller en eksport.
endpoint-repos-panel-secret-note-keep = Gemmes i denne maskines hemmelige lager, aldrig i biblioteket eller en eksport. Et felt, der lades tomt, beholder det gemte.
endpoint-repos-panel-asks-again = { $name } version { $version } spørger igen
endpoint-repos-panel-update-sends-to = Den nye version sender dette til { $host }.
endpoint-repos-panel-fields-changed = Dens felter er ændret.
endpoint-repos-panel-needs-fields = Skal have sin { $fields }, før den søger.
endpoint-repos-panel-no-longer-offered = Tilbydes ikke længere af sin kildeliste.
endpoint-repos-panel-update-requests-to = Opdateringen sender forespørgsler til { $host }.
endpoint-repos-panel-unreadable = Denne liste kunne ikke læses. Opdater lister prøver igen.
endpoint-repos-panel-add-list = Tilføj en kildeliste
endpoint-repos-panel-add-list-helper = Adressen på dens indeksfil eller en sti til en på denne maskine.
endpoint-repos-panel-checking = Tjekker
endpoint-repos-panel-add = Tilføj
endpoint-repos-panel-trust-question = Stol på { $name }?
endpoint-repos-panel-fingerprint = Nøglens fingeraftryk  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    { $count } kilder, signeret. Dens kilder afgør, hvor dine søgninger går hen; stol kun på en liste fra nogen, du stoler på.
endpoint-repos-panel-preview-trusted =
    { $url }
    { $count } kilder, signeret, allerede betroet. Dens kilder afgør, hvor dine søgninger går hen; stol kun på en liste fra nogen, du stoler på.
endpoint-repos-panel-trust = Stol på
endpoint-repos-panel-saving = Gemmer
endpoint-repos-panel-install = Installer
endpoint-repos-panel-update = Opdater
endpoint-repos-panel-added-by-you = tilføjet af dig
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Opdater til v{ $version }
endpoint-repos-panel-add-value = Tilføj dens værdi
endpoint-repos-panel-change = Skift { $fields }
endpoint-repos-panel-asks-for = Beder om din { $fields }.
endpoint-repos-panel-by-hand = Tilføj en manuelt
endpoint-repos-panel-by-hand-helper = Til en side, ingen liste har: ethvert feed med {"{"}query{"}"}, hvor søgningen skal stå, også et fra en søgeproxy. En nøgle i adressen gemmes for sig, i systemets adgangskodelager, hvor der er et.
endpoint-repos-panel-name = Navn
endpoint-repos-panel-refreshing = Opdaterer
endpoint-repos-panel-refresh = Opdater lister

## ScheduleSheet.qml

schedule-episodes-range = Afsnit { $first } til { $last }
schedule-day-format = ddd d. MMM yyyy
schedule-month-format = MMMM yyyy
schedule-tile-day-format = d. MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = Engang i { $year }
schedule-no-date-yet = Ingen dato endnu
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = Ingen dato
schedule-tile-last = Sidste
schedule-tile-out = Udkom
schedule-ends-at = Slutter ved afsnit { $number }
schedule-dated-by-site = { $count } dateret af { $site }
schedule-dated-by-record = { $count } dateret af posten
schedule-set-by-you = { $count } sat af dig
schedule-guessed-weekly = { $count } gættet ugentligt
schedule-nothing-set = Intet sat endnu
schedule-saved = Gemt
schedule-cleared = Datonoter ryddet
schedule-title = Afsnit og datoer
schedule-season = Sæson { $number }
schedule-ask-agent = Spørg { $agent }
schedule-done-pasting = Færdig med at indsætte
schedule-paste-list = Indsæt en liste
schedule-agent-looking = { $agent } slår datoerne op. Intet gemmes, før du trykker Gem.
schedule-agent-stopped = { $agent } stoppede: { $error }
schedule-stop = Stop
schedule-paste-placeholder = Indsæt en udgivelsesliste, en tabel eller et opslag
schedule-paste-helper = Det, AniBeam læser, vises på afsnittene nedenfor.
schedule-agent-hint = Med Claude Code på denne computer kan AniBeam bede den om at slå datoerne op for dig.
schedule-show-more = Vis flere afsnit
schedule-pick-hint = Klik på et afsnit for at give det en dato. Shift-klik vælger en række.
schedule-picked-one = Afsnit { $number }:
schedule-picked = { $count ->
    [one] { $count } afsnit:
   *[other] { $count } afsnit:
}
schedule-date-placeholder = 7. jan., marts 2027, 2027
schedule-set-date = Sæt dato
schedule-weekly = Ugentligt
schedule-not-last = Ikke sidste
schedule-last-episode = Sidste afsnit
schedule-clear-date = Ryd dato

## ShowPage.qml

show-title = Serie
show-runtime-each = { $runtime } hver
show-add-to-list = Føj til liste
show-moving-to = Flytter til { $list }
show-moved-to = Flyttet til { $list }
show-relation-related = Relateret
show-no-poster = Ingen plakat
show-reading-anilist = Læser AniList
show-not-in-library = Ikke i dit bibliotek
show-get-episodes = Hent afsnit
show-trailer = Trailer
show-your-score = Din karakter { $score }
show-rate-it = Bedøm den
show-anilist-score = AniList { $score }
show-forecast = { $percent } % for dig
show-because-you-watched = Fordi du så
show-hidden-loved = { $count ->
    [one] Et skjult tag, du elsker, står bag
   *[other] { $count } skjulte tags, du elsker, står bag
}
show-you-gave-it = Du gav den { $score }
show-story = Handling
show-genres-tags = Genrer og tags
show-studios = Studier  { $studios }
show-related = Relateret
show-recommendations = Anbefalinger
show-from-anilist = Fra AniList
show-reading-rest = Læser resten fra AniList

## ConnectSheet.qml

connect-connected-to = Forbundet med { $name }
connect-connect-to = Forbind { $name }
connect-did-not-finish = Login blev ikke gennemført
connect-finish-in-browser = Gør det færdigt i din browser
connect-signed-in-anilist = Logget ind som { $account }. Din liste er på siden Ser nu.
connect-signed-in-mal = Logget ind som { $account }. Afsnit markeres dér, mens du ser.
connect-setup-lead = { $name } logger AniBeam ind via din egen app. Det tager cirka et minut, én gang.
connect-page-open = { $name }s loginside er åben i din browser. Godkend AniBeam dér, så lukker dette sig selv.
connect-page-opening = Åbner { $name }s loginside i din browser.
connect-open-watching = Åbn Ser nu
connect-done = Færdig
connect-open-again = Åbn loginsiden igen
connect-secret-where = Det er den lange streng under "Client Secret" på appens side på myanimelist.net/apiconfig.
connect-client-secret = Klienthemmelighed
connect-save-try-again = Gem og prøv igen
connect-change-client-id = Skift Client ID
connect-step-create-anilist = Opret klienten på AniList
connect-step-create-mal = Opret appen på MyAnimeList
connect-step-create-anilist-body = Åbn AniLists udviklerindstillinger, opret en ny klient, og indsæt denne adresse i feltet Redirect URL præcis, som den står her.
connect-step-create-mal-body = Åbn MyAnimeLists API-side, opret et ID med App Type "other", og indsæt denne adresse i feltet App Redirect URL præcis, som den står her.
connect-open-developer-page = Åbn udviklersiden
connect-copy-address = Kopier adressen
connect-step-paste-id = Indsæt appens ID
connect-step-paste-id-anilist = AniList kalder det Client ID: tallet på din nye klients side.
connect-step-paste-id-mal = MyAnimeList kalder det Client ID: den lange streng på din nye apps side.
connect-client-id = Client ID
connect-client-secret-web = Klienthemmelighed (kun til Web-apps)
connect-step-connect-body = Browseren åbner, { $name } beder dig logge ind og godkende AniBeam, og AniBeam fanger svaret.

## SettingsPage

settings-title = Indstillinger
settings-search-placeholder = Søg i indstillinger
settings-section-library = Bibliotek
settings-section-accounts = Konti
settings-section-appearance = Udseende
settings-section-playback = Afspilning
settings-section-subtitles = Undertekster
settings-section-keys = Tastatur
settings-section-downloads = Downloads
settings-section-phone = Telefon-app
settings-section-storage = Lager og backup
settings-section-help = Hjælp
settings-section-about = Om
settings-section-developer = Udvikler
settings-status-library-empty = Ingen mapper endnu
settings-status-library =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serier
    } i { $folders ->
        [one] { $folders } mappe
       *[other] { $folders } mapper
    }
settings-status-library-missing =
    { $gone ->
        [one] { $gone } mappe
       *[other] { $gone } mapper
    } ikke fundet, { $count ->
        [one] { $count } serie
       *[other] { $count } serier
    } i { $folders ->
        [one] { $folders } mappe
       *[other] { $folders } mapper
    }
settings-status-accounts-anilist = Synkroniserer med AniList
settings-status-accounts-mal = Synkroniserer med MyAnimeList
settings-status-accounts-both = Synkroniserer med AniList og MyAnimeList
settings-status-accounts-none = Ingen liste forbundet
settings-status-appearance =
    { $mode ->
        [system] Følger systemet
        [light] Lys
       *[dark] Mørk
    }, { $source ->
        [theme] temafarver
       *[desktop] skrivebordets farver
    }
settings-status-playback =
    { $skip ->
        [both] Springer intro og outro over
        [intro] Springer intro over
        [outro] Springer outro over
       *[none] Afspiller alt
    }
settings-status-playback-audio =
    { $skip ->
        [both] Springer intro og outro over
        [intro] Springer intro over
        [outro] Springer outro over
       *[none] Afspiller alt
    }, lyd { $languages }
settings-status-subtitles =
    Størrelse { $percent } procent, { $override ->
        [AsScripted] stilede filer som lavet
        [Force] din stil på alle filer
       *[ScaleOnly] stilede filer beholder deres udseende
    }
settings-status-keys = Alle taster, AniBeam reagerer på
settings-status-downloads-unread = Ikke læst endnu
settings-status-downloads-answered = qBittorrent svarede
settings-status-downloads-missing = Søgehjælper ikke installeret
settings-status-downloads-unconnected = Ikke forbundet endnu
settings-status-phone-shared = Delt på dit tailnet
settings-status-phone-off = Ikke delt
settings-status-storage = { $size } billeder i cache
settings-status-storage-unread = Billeder, eksport og import
settings-status-help = Svar på det, folk oftest spørger om
settings-status-about-update = Version { $version } er udkommet
settings-status-developer = Måler hver side
settings-status-developer-last = { $page } bygget på { $ms } ms
settings-results =
    { $count ->
        [one] { $count } indstilling passer
       *[other] { $count } indstillinger passer
    }
settings-results-none = Intet passer
settings-results-hint = Retur åbner den første.
settings-results-none-hint = Prøv et enklere ord: tema, undertekster, tracker, adgangskode, mappe.

settings-find-folders =
    .words = tilføj mappe scan genscan fjern kilde sti samling
settings-find-matching =
    .words = metadata serieinfo kilde anilist tvmaze rækkefølge match
settings-find-hidden-shows =
    .words = skjult skjulte vis
settings-find-hidden-airing =
    .words = ikke interesseret sendes vis igen
settings-find-calendar =
    .words = kalender ics ical sendes sendeplan abonnér
settings-find-anime-lists =
    .words = anilist myanimelist mal log ind forbind login konto afbryd tracker
settings-find-main-list =
    .words = fremgang kort primær tracker
settings-find-sync-now =
    .words = opdater fremgang lister tracker synkroniser
settings-find-on-hold =
    .words = på pause pause inaktiv urørt dage
settings-find-language =
    .words = sprog oversættelse locale engelsk dansk
settings-find-mode =
    .words = mørk lys system tema nat
settings-find-colours-from =
    .words = terminal skrivebord system tema kilde kitty
settings-find-dark-theme =
    .words = farve farveskema base16
settings-find-light-theme =
    .words = farve farveskema base16
settings-find-accent =
    .words = farve fremhævning accent
settings-find-text-size =
    .words = skrift tekst ord større mindre zoom skala læs tilgængelighed
settings-find-density =
    .words = kompakt luftig afstand størrelse tæthed
settings-find-poster-size =
    .words = kort lille mellem stor plakat
settings-find-out-now =
    .words = sendes udkommet nu kort liste rækker stil banner plakater
settings-find-out-now-cards =
    .words = sendes udkommet nu gitter kolonner rækker layout
settings-find-reduce-motion =
    .words = animation bevægelse tilgængelighed
settings-find-app-icon =
    .words = logo ikon launcher dock proceslinje vindue mørk
settings-find-skip-intros =
    .words = intro opening op auto spring over aniskip kapitler
settings-find-skip-outros =
    .words = outro ending ed rulletekster auto spring over aniskip
settings-find-mpv-settings =
    .words = mpv.conf konfiguration avanceret
settings-find-discord =
    .words = discord rich presence status profil
settings-find-next-episode =
    .words = næste knap titel autoplay slut
settings-find-subtitle-languages =
    .words = sprog spor undertekster tekster
settings-find-audio-languages =
    .words = sprog spor lyd dub japansk
settings-find-start-over =
    .words = nulstil standard undertekststil
settings-find-size =
    .words = skala størrelse undertekster
settings-find-position =
    .words = højde margen placering undertekster
settings-find-styled =
    .words = ass tilsidesæt tving stilede
settings-find-font =
    .words = skrift skrifttype
settings-find-text-colour =
    .words = farve tekstfarve
settings-find-outline =
    .words = kant kontur streg
settings-find-background =
    .words = boks baggrund gennemsigtighed
settings-find-bold =
    .words = fed vægt
settings-find-keys-player =
    .words = tastatur genveje taster genvejstaster
settings-find-keys-player-around =
    .words = tastatur genveje fuld skærm næste forrige afsnit tilbage
settings-find-keys-around =
    .words = tastatur genveje tilbage frem højreklik menu fuld skærm
settings-find-keys-anywhere =
    .words = tastatur genveje taster genvejstaster indstillinger afslut
settings-find-torrent-client =
    .words = qbittorrent adgangskode brugernavn adresse webui seedbox torrentprogram
settings-find-search-helper =
    .words = anirss rss feeds token kommando server avanceret søgehjælper
settings-find-next-few =
    .words = løbende afsnit foran slet sete behold diskplads
settings-find-unsubscribe =
    .words = afmeld feeds afsluttede
settings-find-last-search =
    .words = downloads søgning husk
settings-find-search-sources =
    .words = endpoints feeds konfiguration anirss kilder søgekilder
settings-find-source-lists =
    .words = repo repos endpoint signeret endpoints jackett prowlarr kildelister
settings-find-share-tailscale =
    .words = telefon android tailnet fjern vært del tailscale
settings-find-stream =
    .words = stream afspil cast afsnit telefon filer
settings-find-devices =
    .words = stol betroet slå fra bloker enhed enheder tailscale
settings-find-back-up =
    .words = eksport backup sikkerhedskopi fil json
settings-find-restore =
    .words = import gendan fil json
settings-find-image-cache =
    .words = plakater ryd billeder disk cache
settings-find-files =
    .words = database data konfiguration cache mappe sti filer
settings-find-help-folders =
    .words = hjælp mappe opbygning navngivning afsnit sæsoner film filer struktur
settings-find-help-recognised =
    .words = hjælp match forkert serie ikke fundet umatchet genkendt
settings-find-help-anilist =
    .words = hjælp anilist myanimelist mal log ind forbind login tracker
settings-find-help-downloads =
    .words = hjælp downloads tom kilder intet torrent qbittorrent
settings-find-help-data =
    .words = hjælp data filer backup database privatliv
settings-find-help-else =
    .words = hjælp opsætning introduktion gennemgang rapporter fejl problem
settings-find-version =
    .words = build nyheder ændringslog version
settings-find-setup =
    .words = første start gennemgang introduktion opsætning igen
settings-find-updates =
    .words = stabil nightly kanal tjek opdatering
settings-find-developer-options =
    .words = debug fejlfinding måling udvikler
settings-find-ratings =
    .words = imdb kreditering bedømmelser

## SettingsDownloadsTab

settings-downloads-title = Downloads
settings-downloads-lead = Hvor AniBeam sender det, du downloader, og hvordan den søger i udgivelsesfeeds.
settings-downloads-keep-none = Når du ser et afsnit færdigt, slettes det forrige.
settings-downloads-keep-line = Sete afsnit, der bliver liggende bag det seneste. Med { $count } slettes afsnit 1, når du har set afsnit { $finishing }.
settings-downloads-anirss-ready = Kører, version { $version } på { $url }.
settings-downloads-anirss-starting = Starter søgehjælperen.
settings-downloads-anirss-missing = Søgehjælperen er ikke installeret. Geninstaller AniBeam, eller angiv din egen anirss under Kommando.
settings-downloads-anirss-failed = Søgehjælperen stoppede. Prøv igen starter den; Detaljer viser, hvad den sagde.
settings-downloads-anirss-idle = Ikke startet endnu. Den starter første gang, du søger på siden Downloads.
settings-downloads-torrent-trying = Prøver { $client }.
settings-downloads-torrent-answered = { $client } { $version } svarede.
settings-downloads-torrent-failed = { $client } blev ikke forbundet. Tjek adressen og adgangskoden ovenfor, og test igen.
settings-downloads-torrent-untried = Ikke prøvet endnu.
settings-downloads-mode-existing = En qBittorrent, du selv kører, med Web UI slået til.
settings-downloads-mode-managed = AniBeam kører sin egen qBittorrent, der henter til ~/Downloads/Anime. Den kører videre, når AniBeam lukker, så abonnementer bliver ved med at downloade.
settings-downloads-mode-auto = Bruger qBittorrent på adressen nedenfor, ellers sin egen.
settings-downloads-password-saved = Der er gemt en adgangskode her til dette program.
settings-downloads-password-anirss = Bruger den adgangskode, søgehjælperen gemte; gem en her for ikke at være afhængig af den.
settings-downloads-password-none = Ingen adgangskode gemt. Et program, der springer login over for denne maskine, behøver ingen.
settings-downloads-mismatch = Søgehjælperen taler med { $helperUrl }, mens AniBeam taler med { $url }. En søgehjælper, du selv kører, læser adressen fra sin egen config.toml, så et skift af program her flytter den ikke.
settings-downloads-torrent-title = Torrentprogram
settings-downloads-torrent-helper = Den qBittorrent, der henter. Adgangskoden gemmes i systemets adgangskodelager.
settings-downloads-which-client = Hvilket program
settings-downloads-pick-for-me = Vælg for mig
settings-downloads-my-own = Mit eget
settings-downloads-built-in = Indbygget
settings-downloads-saved-clients = Gemte programmer
settings-downloads-saved-clients-helper = For eksempel en seedbox og den på denne maskine. Programchippen på siden Downloads skifter også mellem dem.
settings-downloads-add-client = Tilføj
settings-downloads-name = Navn
settings-downloads-name-placeholder = Opkaldt efter sin adresse
settings-downloads-address = Adresse
settings-downloads-address-managed-helper = Kun denne maskine kan nå den, og den kræver ingen adgangskode.
settings-downloads-address-helper = Adressen på Web UI.
settings-downloads-username = Brugernavn
settings-downloads-password = Adgangskode
settings-downloads-secret-saved = Gemt
settings-downloads-secret-unset = Ikke sat
settings-downloads-connection = Forbindelse
settings-downloads-open-config = Åbn dens konfiguration
settings-downloads-details = Detaljer
settings-downloads-details-torrent = { $client } sagde dette, da AniBeam prøvede den.
settings-downloads-test-trying = Prøver
settings-downloads-test-save = Gem og test
settings-downloads-test = Test
settings-downloads-searching = Søgning
settings-downloads-search-helper = Søgehjælper
settings-downloads-searching-helper = AniBeam søger selv i dine kilder og sender det, du vælger, til torrentprogrammet. Søgehjælperen er ikke nødvendig.
settings-downloads-search-helper-helper = Den del af AniBeam, der søger og abonnerer, et program ved navn anirss. AniBeam kører det for dig; skift kun dette, hvis du kører dit eget.
settings-downloads-runs-in = Søgninger kører i
settings-downloads-runs-in-native-helper = Abonnementer kører i torrentprogrammet, også med AniBeam lukket.
settings-downloads-runs-in-helper = Søgehjælperen, et separat program, som AniBeam kører, eller som du selv kører.
settings-downloads-engine-native = AniBeam selv
settings-downloads-how-runs = Sådan kører den
settings-downloads-how-runs-sidecar = AniBeam starter den, når den åbner, og stopper den, når den lukker.
settings-downloads-how-runs-remote = AniBeam bruger en, du allerede kører, på adressen og med nøglen nedenfor.
settings-downloads-runs-sidecar = AniBeam kører den
settings-downloads-runs-remote = Jeg kører den
settings-downloads-command = Kommando
settings-downloads-command-helper = Et navn på PATH eller en fuld sti. Der søges også i ~/.local/bin.
settings-downloads-server-address-helper = Serverens grundadresse, uden /api/v1.
settings-downloads-key = Nøgle
settings-downloads-key-saved-helper = Der er gemt en nøgle. Skriv en ny for at erstatte den, eller ryd feltet for at fjerne den.
settings-downloads-key-helper = Den nøgle, søgehjælperen viste, da den startede. Den lukker AniBeam ind.
settings-downloads-details-anirss = Søgehjælperen sagde dette, da den stoppede.
settings-downloads-starting = Starter
settings-downloads-reconnect = Forbind igen
settings-downloads-save-first = Gem dine ændringer først
settings-downloads-roll-title = Hold de næste klar
settings-downloads-roll-helper = En lang serie downloades få afsnit ad gangen, og hvert, du ser færdigt, starter det næste. Hele serien skal aldrig være på disken.
settings-downloads-roll-big = Til lange serier
settings-downloads-roll-big-helper = Alt over { $count } afsnit downloades selv nogle få ad gangen. Dens række på siden Downloads kan stadig hente alt på én gang.
settings-downloads-roll-ahead = Afsnit foran
settings-downloads-roll-ahead-helper = Hvor mange der forbliver downloadet efter det, du er nået til. 12 er cirka en sæson.
settings-downloads-unit-episodes = afsnit
settings-downloads-roll-over = Lang betyder over
settings-downloads-roll-over-helper = Downloads med flere afsnit end dette hentes nogle få ad gangen.
settings-downloads-roll-watched = Sete afsnit
settings-downloads-roll-watched-delete-helper = Slettes, når du er nået længere, så disken kun har det, der er tilbage at se.
settings-downloads-roll-watched-keep-helper = Alle filer bliver på disken, efter du har set dem.
settings-downloads-keep-all = Behold alle
settings-downloads-keep-behind = Behold bagud
settings-downloads-subscriptions-title = Abonnementer og søgninger
settings-downloads-last-search = Husk den sidste søgning
settings-downloads-last-search-helper = Downloads åbner med din seneste søgning. Slået fra åbner den tom.
settings-downloads-prune = Afmeld afsluttede serier
settings-downloads-prune-helper = 16 dage efter, at en series sidste afsnit er sendt, droppes dens feed. Serien og dens filer bliver.
settings-downloads-sources-title = Søgekilder
settings-downloads-sources-helper = Hvor en søgning går hen, i den rækkefølge de prøves. Finder én intet, prøves den næste.
settings-downloads-sources-empty = Ingen kilder sat op endnu
settings-downloads-sources-connect = Forbind søgehjælperen for at vise dem
settings-downloads-sources-empty-helper = Tilføj en kildeliste nedenfor, så vises dens kilder her.
settings-downloads-config-file = Søgehjælperens indstillingsfil
settings-downloads-config-native-helper = Indeholder gemmemapperne, tags og reglerne for bedste match, som AniBeam selv følger, når den kører hjælperen. Dens egne kilder tilhører kun hjælperen.
settings-downloads-config-helper = Indeholder kilderne, gemmemapperne, tags og reglerne for bedste match.
settings-downloads-unsaved = Ændringer ikke gemt
settings-downloads-unsaved-helper = Når du gemmer, genstarter søgehjælperen med de nye indstillinger.
settings-downloads-discard = Kassér

## SettingsDeveloperTab

settings-developer-title = Udvikler
settings-developer-lead = Hvorfor en side er langsom, målt. Hver kontakt gælder for den næste side, du åbner.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds } s
settings-developer-switches = Kontakter
settings-developer-load-images = Indlæs billeder
settings-developer-load-images-helper = Slået fra læses eller afkodes ingen plakater, bannere eller portrætter.
settings-developer-overlay = Overlay for sideindlæsning
settings-developer-overlay-helper = Den sidste sideindlæsning og hukommelsen, i vinduets nederste højre hjørne.
settings-developer-read-cache = Gem bibliotekslæsninger
settings-developer-read-cache-helper = Svaret på en læsning af hele biblioteket gemmes, indtil biblioteket ændrer sig. Slået fra spørger hver side kernen igen.
settings-developer-poster-pool = Varm plakater op i baggrunden
settings-developer-poster-pool-helper = Afkoder de 60 plakater, du mest sandsynligt ser, og holder dem i hukommelsen.
settings-developer-options = Udviklerindstillinger
settings-developer-options-helper = Slår denne fane, overlayet og målingerne fra.
settings-developer-memory = Hukommelse
settings-developer-memory-helper = Det, kernen tæller for denne proces. Billeder holder afkodede pixels i hukommelsen og på grafikkortet.
settings-developer-memory-unknown = Dette system oplyser ikke, hvor meget hukommelse appen bruger.
settings-developer-in-use = I brug
settings-developer-peak = Top
settings-developer-heap = Heap
settings-developer-files-mapped = Mappede filer
settings-developer-threads = Tråde
settings-developer-give-back = Frigiv hukommelse
settings-developer-give-back-helper = Smider det væk, scenegrafen kan genopbygge, samler JavaScript-affald og giver ledig heap tilbage til systemet. Det, der frigives, er det, appen holdt på uden at have brug for det.
settings-developer-released = { $before } til { $after }
settings-developer-free-now = Frigiv nu
settings-developer-images-total = Billeder siden start
settings-developer-images-total-line = { $decodes } afkodet, { $pixels } pixels fra { $disk } på disken, { $ms } ms
settings-developer-loads = Sideindlæsninger
settings-developer-loads-helper = Bygget er ventetiden efter et klik: Qt-tråden laver intet andet, før siden findes. Faldet til er, når dens sidste billede kom.
settings-developer-touring = Måler, { $at } af { $steps }
settings-developer-tour = Mål hver side
settings-developer-copy-report = Kopier rapport
settings-developer-col-page = Side
settings-developer-col-built = Bygget
settings-developer-col-painted = Tegnet
settings-developer-col-settled = Faldet til
settings-developer-col-calls = Kald, data
settings-developer-col-decodes = Afkodninger, tid, pixels
settings-developer-col-items = Elementer
settings-developer-col-offscreen = Billeder uden for visning
settings-developer-col-memory = Hukommelse
settings-developer-load-no-images = { $page }, ingen billeder
settings-developer-offscreen-of = { $offscreen } af { $total }
settings-developer-loads-empty = Åbn en side, så vises den her.
settings-developer-load-images-off = { $page }, billeder fra
settings-developer-load-frames = { $frames } billeder tegnet under indlæsningen, det længste mellemrum mellem to { $gap }.
settings-developer-load-frames-timed-out = { $frames } billeder tegnet under indlæsningen, det længste mellemrum mellem to { $gap }. Opgav at vente på dens billeder efter 15 s.
settings-developer-built = Bygget
settings-developer-settled = Faldet til
settings-developer-core-data = Data fra kernen
settings-developer-pixels = Pixels afkodet
settings-developer-page-items = Elementer på siden
settings-developer-cached-images = Billeder allerede i hukommelsen
settings-developer-calls = Kernekald, langsomste først
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  cachet
settings-developer-call = { $name }  { $ms } ms  { $size }  kerne { $core }, til QML { $qml }
settings-developer-decodes = Langsomste afkodninger
settings-developer-decodes-line = { $count } afkodninger efter hinanden på billedtråden, { $span } fra den første til den sidste; { $requests } bestilt, { $cached } allerede afkodet, { $blocked } blokeret
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } til { $w }x{ $h }  { $size }
settings-developer-decodes-none = Intet afkodet: alle billeder var allerede i hukommelsen.
settings-developer-images-were-off = Billeder var slået fra.
settings-developer-scene = Hvad siden består af
settings-developer-scene-line = { $items } elementer, { $offscreen } uden for vinduet; { $texts } tekster, { $shapes } former, { $icons } ikoner; { $images } billeder, heraf { $imagesOffscreen } uden for vinduet, der fylder { $size }
settings-developer-copy-load = Kopier denne indlæsning
settings-developer-tour-title = Hver side, billeder til og fra
settings-developer-tour-helper = Den midterste af tre besøg på hver side. Hvor de to kolonner er ens, er det ikke billederne, der gør siden langsom.
settings-developer-col-built-on = Bygget, billeder til
settings-developer-col-built-off = Bygget, billeder fra
settings-developer-col-settled-on = Faldet til, til
settings-developer-col-settled-off = Faldet til, fra
settings-developer-col-decodes-time = Afkodninger, tid
settings-developer-col-data = Data
settings-developer-tour-peak = Højeste hukommelsesforbrug under målingen { $size }
settings-developer-events = Hændelser
settings-developer-events-helper = Alt, kernen siger, på alle niveauer, nyeste først. Hver hændelse kopieres til Qt-tråden som JSON; størrelsen er, hvad det kostede at sende.
settings-developer-event-rate = { $kind } { $count }/min, { $size }
settings-developer-events-filter = Filtrer på ord, type eller trin
settings-developer-event-plain = { $message }   (ren: { $plain })
settings-developer-copy-events = Kopier hændelser

## SettingsKeysTab

settings-keys-title = Tastatur
settings-keys-lead = Alle taster, AniBeam reagerer på. Tryk ? hvor som helst for de taster, der virker, hvor du er.
settings-keys-player = I afspilleren
settings-keys-player-around = Rundt i afspilleren
settings-keys-around = Rundt i appen
settings-keys-anywhere = Overalt

## KeyHelp

key-help-title = Tastaturgenveje
key-help-group-playback = Afspilning
key-help-group-navigation = Navigation
key-help-group-library = Bibliotek
key-help-key-space = Mellemrum / K
key-help-key-left-right = Venstre / Højre
key-help-key-ctrl-right = Ctrl+Højre
key-help-key-backspace = Backspace
key-help-key-up-down = Op / Ned
key-help-key-hold = Hold på videoen
key-help-key-page = Page Up / Down
key-help-key-home-end = Home / End
key-help-key-escape = Escape
key-help-key-alt-left = Alt+Venstre
key-help-key-alt-right = Alt+Højre
key-help-key-shift-f10 = Shift+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / eller Ctrl+F
key-help-key-ctrl-space = Ctrl+Mellemrum
key-help-key-shift-space = Shift+Mellemrum
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = Afspil eller pause
key-help-seek-5 = Spol 5 s
key-help-seek-10 = Spol 10 s
key-help-skip = Spring intro eller outro over, ellers 90 s
key-help-frame-step = Ét billede tilbage eller frem
key-help-speed = Langsommere eller hurtigere, i trin på 0,25
key-help-normal-speed = Normal hastighed
key-help-volume = Lydstyrke op eller ned med 5
key-help-mute = Slå lyden fra
key-help-subtitles-toggle = Undertekster fra og til igen
key-help-subtitle-delay = Undertekstforsinkelse 100 ms tidligere eller senere
key-help-audio-delay = Lydforsinkelse 100 ms tidligere eller senere
key-help-hold-speed = 2× hastighed, mens du holder
key-help-screenshot = Gem et skærmbillede i Billeder
key-help-episode = Næste eller forrige afsnit
key-help-chapter = Forrige eller næste kapitel
key-help-start-end = Starten, eller outroen
key-help-fullscreen = Fuld skærm
key-help-escape-player = Luk det åbne, ellers forlad fuld skærm, ellers forlad afspilleren
key-help-toggle-list = Vis eller skjul denne liste
key-help-library-search = Bibliotek og søgning
key-help-search-page = Søg på denne side
key-help-activity = Aktivitet
key-help-settings = Indstillinger
key-help-quit = Afslut
key-help-forward = Frem
key-help-escape-app = Luk det åbne, ellers tilbage
key-help-context-menu = Højrekliksmenuen
key-help-pick-card = Vælg et kort, for at handle på flere
key-help-pick-range = Vælg alle kort op til dette
key-help-pick-all = Vælg alle viste kort

## SettingsAboutTab

settings-about-title = Om
settings-about-lead = Hvilken AniBeam dette er, og hvordan den hører om den næste.
settings-about-update-failed = Opdateringen blev ikke færdig. Prøv igen, eller hent den fra udgivelsessiden.
settings-about-update-ready = { $version } er klar til at blive installeret.
settings-about-update-command = { $version } er udkommet. Opdater den med { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = { $version } er udkommet. Denne kopi er bygget fra kildekoden: pull og byg igen.
settings-about-update-elsewhere = { $version } er udkommet, uden en download her til denne computer.
settings-about-version = Version
settings-about-build = AniBeam { $build }
settings-about-whats-new = Nyheder
settings-about-install = Installer { $version }
settings-about-setup = Opsætning
settings-about-setup-helper = Gennemgangen, en ny installation åbner med: din mappe, din liste og downloads.
settings-about-run-setup = Kør opsætning igen
settings-about-updates = Opdateringer
settings-about-nightly-helper = En ny version de fleste dage. Nyere og mindre testet.
settings-about-stable-helper = En ny version, når den er testet og udgivet.
settings-about-stable = Stabil
settings-about-nightly = Nightly
settings-about-check-now = Tjek nu
settings-about-developer = Udviklerindstillinger
settings-about-developer-helper = Tilføjer en Udvikler-sektion, der måler hver side, viser hukommelse og hele den tekniske log og kan slå billeder fra.
settings-about-credits = Kreditering
settings-about-ratings = Bedømmelser
settings-about-ratings-helper = Oplysninger stillet til rådighed af IMDb (https://www.imdb.com). Brugt med tilladelse.
settings-about-series = Serier og afsnit
settings-about-series-helper = AniList og TVmaze. Springtider fra AniSkip.

## SettingsPlaybackTab

settings-playback-title = Afspilning
settings-playback-lead = Det, afspilleren gør af sig selv, så du kun skal røre den, når du vil.
settings-playback-skipping = Spring over
settings-playback-skipping-helper = Bruger filens kapitler, ellers springtider delt af andre seere online. Fortryd i afspilleren slår spring fra, til AniBeam lukker.
settings-playback-skip-intros = Spring intro over
settings-playback-skip-outros = Spring outro over
settings-playback-skip-outros-helper = Når outroen nås, markeres afsnittet også som set.
settings-playback-next-episode = Næste afsnit
settings-playback-name-next = Vis næste afsnits navn
settings-playback-name-next-helper = Knappen Næste i slutningen af et afsnit viser det næstes titel. Slået fra viser den kun nummeret.
settings-playback-languages = Sprog
settings-playback-languages-helper = Sprogkoder, de mest ønskede først. Det første, en fil har, vinder. Et spor, du vælger i afspilleren, huskes for den serie.
settings-playback-subtitle-languages = Undertekstsprog
settings-playback-audio-languages = Lydsprog
settings-playback-discord = Vis, hvad du ser, på Discord
settings-playback-discord-check-failed = AniBeam kunne ikke tjekke Discord lige nu. Prøv igen om lidt.
settings-playback-discord-helper = Serien, afsnittet og plakaten vises på din Discord-profil, mens du afspiller. Intet sendes, mens dette er slået fra, eller Discord er lukket.
settings-playback-discord-unavailable = Denne version har ingen Discord-applikation.
settings-playback-discord-connected = Forbundet med Discord
settings-playback-discord-not-connected = Ikke forbundet. AniBeam forbinder, mens et afsnit afspilles, og Discord er åben
settings-playback-advanced = Avanceret
settings-playback-my-mpv = Brug mine mpv-indstillinger
settings-playback-my-mpv-helper = Læser din mpv.conf. Indstillinger, der kun virker ved opstart, springes over, og scripts indlæses aldrig.
settings-playback-no-mpv-conf = Ingen mpv-indstillingsfil fundet. Sæt dine indstillinger i mpv.conf, og slå så dette til.

## SliderRow

slider-row-spoken-unit = { $name }, i { $unit }
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Accent rød
swatches-accent-orange = Accent orange
swatches-accent-yellow = Accent gul
swatches-accent-green = Accent grøn
swatches-accent-cyan = Accent cyan
swatches-accent-blue = Accent blå

## LookPreview

look-preview-dark = Mørk
look-preview-light = Lys

## SettingsAccountsTab

settings-accounts-title = Konti
settings-accounts-lead = Forbind AniList eller MyAnimeList, så bliver hvert afsnit, du ser færdigt, markeret dér for dig. Tallene går kun op.
settings-accounts-syncing = Synkroniserer dine lister
settings-accounts-synced = Lister synkroniseret
settings-accounts-lists = Dine animelister
settings-accounts-lists-helper = Uden en husker AniBeam stadig, hvad du har set på denne computer. En liste på AniList eller MyAnimeList tager det med til din telefon og din profil.
settings-accounts-progress = Fremgang
settings-accounts-main-list = Hovedliste
settings-accounts-main-list-helper = Kortene viser fremgang fra denne. Den anden får stadig alle opdateringer.
settings-accounts-local-lists = Lister på denne computer
settings-accounts-local-lists-helper =
    { $count ->
        [one] Én serie har
       *[other] { $count } serier har
    } en status eller karakter kun her. Flyt dem til { $tracker } for at se dem på din telefon og din profil.
settings-accounts-review-move = Gennemse og flyt
settings-accounts-nothing-to-move = Intet tilbage at flytte
settings-accounts-sync-now = Synkroniser nu
settings-accounts-sync-now-helper = Læser alle forbundne lister igen. AniBeam gør det også, når den starter.
settings-accounts-sync = Synkroniser
settings-accounts-move-on-hold = Flyt til På pause
settings-accounts-move-on-hold-helper = En serie på Ser nu, som du ikke har rørt i et stykke tid, flyttes til På pause på dine lister. Afspiller du den, flyttes den tilbage. En serie, der venter på sit næste afsnit, bliver, hvor den er.
settings-accounts-after = Efter
settings-accounts-after-line = Dage uden at afspille et afsnit
settings-accounts-days-unit = dage

## SettingsPhoneTab

settings-phone-title = Telefon-app
settings-phone-lead = AniBeam på din telefon kan afspille dette bibliotek, se hvad der sendes og følge dine downloads, uanset hvor du er.
settings-phone-online = Online
settings-phone-offline = Offline
settings-phone-untrusted = Får intet fra denne computer
settings-phone-sharing = Deling
settings-phone-sharing-helper = Kun dine egne enheder på Tailscale kan nå den. Afsnit, du ser på telefonen, beholder deres plads og fremgang her.
settings-phone-share = Del via Tailscale
settings-phone-stream = Stream afsnit
settings-phone-stream-on = Dine enheder kan afspille og caste afsnittene på denne computer.
settings-phone-stream-off = Dine enheder ser biblioteket og hvad der sendes, men kan ikke afspille herfra.
settings-phone-host-service = Værtstjeneste
settings-phone-not-running = Kører ikke
settings-phone-running = Kører
settings-phone-service-off-helper = Kør kommandoen én gang, så når dine enheder AniBeam fra login, uden et åbent vindue.
settings-phone-service-on-helper = Dine enheder når AniBeam uden et åbent vindue. Mens dette vindue er åbent, deler det i stedet for tjenesten.
settings-phone-copy-command = Kopier kommando
settings-phone-command-copied = Kommando kopieret
settings-phone-nobody = Får intet: Tailscale angiver ingen person for den
settings-phone-people = Personer
settings-phone-own-profile = Denne computers profil
settings-phone-person-own-devices =
    { $count ->
        [0] Denne computer
        [one] Denne computer og { $count } enhed
       *[other] Denne computer og { $count } enheder
    }
settings-phone-person-devices =
    { $count ->
        [one] { $count } enhed
       *[other] { $count } enheder
    }
settings-phone-devices = Enheder på dit tailnet
settings-phone-devices-helper = Alle enheder på dit tailnet kan nå denne computer, mens deling er slået til. Slå en fra, så får den intet herfra.
settings-phone-addresses = Adresser
settings-phone-addresses-helper = Skriv en af disse i telefonappen, første gang den spørger.
settings-phone-copy = Kopier
settings-phone-address-copied = Adresse kopieret

## SettingsHelpTab

settings-help-title = Hjælp
settings-help-lead = Korte svar på det, folk oftest spørger om, hver med en direkte vej til løsningen.
settings-help-folders = Hvordan skal mine mapper se ud?
settings-help-folders-helper = Én mappe pr. serie med afsnittene i. Giv hver fil afsnitsnummeret i navnet, som udgivelser gør: "Frieren - 01.mkv" eller "Frieren S01E01.mkv". En anden sæson kan være sin egen mappe eller en "Season 2"-mappe inde i serien. Film skal i en mappe ved navn Movies, én fil hver.
settings-help-for-example = For eksempel
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = Ingen mappe tilføjet endnu
settings-help-reads-folders =
    { $count ->
        [one] AniBeam læser én mappe
       *[other] AniBeam læser { $count } mapper
    }
settings-help-add-folder-helper = Tilføj mappen, der indeholder dine serier, ikke hver serie for sig.
settings-help-add-folder = Tilføj en mappe
settings-help-open-folders = Åbn dine mapper
settings-help-match = Hvorfor blev min serie ikke genkendt?
settings-help-match-helper = AniBeam slår hver mappe op ud fra navnet, først på AniList, så på TVmaze. Ekstra ord, en stavefejl eller et navn på et andet sprog kan ramme forbi. Vælg den rigtige serie manuelt under Serieinfo; et valg, du træffer, bliver aldrig erstattet.
settings-help-all-recognised = Alle serier er genkendt
settings-help-unrecognised =
    { $count ->
        [one] Én serie er ikke genkendt
       *[other] { $count } serier er ikke genkendt
    }
settings-help-open-show-info = Åbn Serieinfo
settings-help-connect = Hvordan forbinder jeg AniList?
settings-help-connect-helper = Forbind den, så bliver hvert afsnit, du ser færdigt, markeret på din liste, og din telefon og din profil følger med. AniList beder dig lave en lille app selv én gang; AniBeam guider dig igennem, og det tager cirka et minut. MyAnimeList virker på samme måde.
settings-help-both-connected = Begge er forbundet
settings-help-anilist-connected = AniList er forbundet
settings-help-mal-connected = MyAnimeList er forbundet
settings-help-none-connected = Ingen af dem er forbundet endnu
settings-help-connect-anilist = Forbind AniList
settings-help-connect-mal = Forbind MyAnimeList
settings-help-downloads = Hvorfor er Downloads tom?
settings-help-downloads-helper = AniBeam kommer uden steder at søge. Tilføj en kildeliste i Indstillinger under Downloads, og installer de kilder, du vil have, fra den. At downloade kræver også et torrentprogram: AniBeam kan køre sit eget eller bruge en qBittorrent, du allerede har.
settings-help-sources = Kilder og torrentprogrammet
settings-help-open-downloads = Åbn indstillinger for Downloads
settings-help-data = Hvor er mine data?
settings-help-data-helper = Dit bibliotek, det du har set, og dine indstillinger bliver på denne computer. Login til AniList og MyAnimeList gemmes i systemets adgangskodelager, eller i en fil kun du kan læse, hvor der ikke er et. Tag backup af det hele til én fil under Lager og backup.
settings-help-library-history = Bibliotek og historik
settings-help-show-folder = Vis mappen
settings-help-backup = Tag backup eller gendan
settings-help-open-storage = Åbn Lager og backup
settings-help-more = Noget andet?
settings-help-more-helper = Kør den første opsætning igen, eller fortæl os, hvad der gik galt. Et skærmbillede hjælper meget.
settings-help-first-setup = Den første opsætning
settings-help-first-setup-helper = Mappe, liste og downloads, trin for trin.
settings-help-run-setup = Kør opsætning igen
settings-help-report = Meld en fejl
settings-help-report-helper = Åbner en fejlrapport på GitHub i din browser med denne version udfyldt.

## SettingsSubtitlesTab

settings-subtitles-title = Undertekster
settings-subtitles-lead = Det, hvert afsnit starter med. Ændringer vises på billedet nedenfor, mens du laver dem.
settings-subtitles-reset-done = Underteksterne ser ud, som de gjorde fra start
settings-subtitles-size-place = Størrelse og placering
settings-subtitles-size = Størrelse
settings-subtitles-position = Placering
settings-subtitles-position-helper = 100 ligger ved bunden. Lavere tal løfter linjerne op i billedet.
settings-subtitles-style = Stil
settings-subtitles-styled = Stilede undertekster
settings-subtitles-as-made-helper = Stilede (ASS) filer ser ud præcis, som deres skabere har sat dem.
settings-subtitles-mine-helper = Din stil nedenfor bruges på alle filer. Skilte og karaoke kan gå i stykker.
settings-subtitles-size-only-helper = Stilede undertekster beholder udseendet og får din størrelse. Almindelige får din stil.
settings-subtitles-as-made = Original
settings-subtitles-size-only = Kun størrelse
settings-subtitles-mine = Mine
settings-subtitles-font = Skrift
settings-subtitles-font-invalid = Skriv et skriftnavn, fx Noto Sans.
settings-subtitles-bold = Fed
settings-subtitles-text-colour = Tekstfarve
settings-subtitles-colour-invalid = Skriv en farve som #ffcc00.
settings-subtitles-outline = Kontur
settings-subtitles-outline-invalid = Bredden er et tal, 0 eller mere.
settings-subtitles-outline-colour-invalid = Skriv farven som #000000.
settings-subtitles-outline-helper = Dens bredde, så dens farve.
settings-subtitles-shadow = Skygge
settings-subtitles-shadow-invalid = Et tal, 0 eller mere. 0 betyder ingen skygge.
settings-subtitles-shadow-helper = Hvor langt skyggen falder, 0 for ingen.
settings-subtitles-box = Baggrundsboks
settings-subtitles-box-helper = En mørk boks bag linjerne. 0 udelader den.
settings-subtitles-start-over = Fra start
settings-subtitles-start-over-helper = Sætter størrelse, skrift, farver, kontur, boks og placering tilbage, som de var.
settings-subtitles-reset = Nulstil

## SettingsDataTab

settings-data-title = Lager og backup
settings-data-lead = Flyt dit bibliotek til en anden computer, gem en kopi sikkert, eller frigør pladsen, billeder fylder.
settings-data-backed-up = Backup gemt i { $path }
settings-data-restore-finished = Gendannelse færdig
settings-data-images-removed =
    { $count ->
        [one] Fjernede { $count } billede
       *[other] Fjernede { $count } billeder
    }
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] { $count } mappe tilføjet
       *[other] { $count } mapper tilføjet
    }
settings-data-summary-series = { $count } serier oprettet
settings-data-summary-matches =
    { $count ->
        [one] { $count } match anvendt
       *[other] { $count } matches anvendt
    }
settings-data-summary-views =
    { $views ->
        [one] { $views } visning
       *[other] { $views } visninger
    } og { $episodes ->
        [one] { $episodes } færdigt afsnit
       *[other] { $episodes } færdige afsnit
    } flettet
settings-data-summary-resume-points =
    { $count ->
        [one] { $count } genoptagelsespunkt flettet
       *[other] { $count } genoptagelsespunkter flettet
    }
settings-data-summary-accounts =
    { $count ->
        [one] { $count } listekonto hentet ind
       *[other] { $count } listekonti hentet ind
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } fil bruger en rettelse, du har lavet.
       *[other] { $count } filer bruger en rettelse, du har lavet.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } mappe blev ikke fundet på denne computer.
       *[other] { $count } mapper blev ikke fundet på denne computer.
    }
settings-data-summary-left-out = Udeladt: { $fields }.
settings-data-backup = Backup
settings-data-back-up-library = Tag backup af biblioteket
settings-data-back-up-helper = Én fil med dine mapper, matches og det, du har set.
settings-data-last-written = Sidst gemt i { $path }
settings-data-open-folder = Åbn mappe
settings-data-back-up = Tag backup
settings-data-private = Medtag private data
settings-data-private-helper = Tilføjer login til AniList og MyAnimeList, visningshistorik og præferencer. Login gemmes som ren tekst, så hold filen for dig selv.
settings-data-restore-from = Gendan fra en backup
settings-data-restore-helper = Læser en backup ind i dette bibliotek. Det, der ligger på disken, vinder; filen udfylder resten.
settings-data-restore = Gendan
settings-data-image-cache = Billedcache
settings-data-image-cache-helper = Plakater, bannere og portrætter gemmes på disken, så biblioteket virker offline. Ryddede billeder hentes igen efter behov.
settings-data-cache-unmeasured = Billedcachen kunne ikke måles
settings-data-cache-size =
    { $count ->
        [one] { $size } i { $count } billede
       *[other] { $size } i { $count } billeder
    }
settings-data-cache-empty = Ingen billeder i cache
settings-data-cache-unmeasured-helper = Prøv igen om lidt. Selve billederne er i orden.
settings-data-remove-images =
    { $count ->
        [one] Fjern { $count } billede?
       *[other] Fjern { $count } billeder?
    }
settings-data-files = Hvor AniBeam gemmer sine filer
settings-data-files-database = Biblioteksdatabase
settings-data-files-data = Data
settings-data-files-settings = Indstillinger
settings-data-files-cache = Cache
settings-data-backup-filter = AniBeam-backup (*.json)
settings-data-all-files-filter = Alle filer (*)
settings-data-backing-up = Tager backup
settings-data-restoring = Gendanner

## SettingsLibraryTab

settings-library-title = Bibliotek
settings-library-lead = De mapper, AniBeam overvåger, og hvordan den genkender indholdet.
settings-library-not-scanned = Ikke scannet endnu
settings-library-scan-nothing-changed = Intet ændret
settings-library-scan-added = { $count } tilføjet
settings-library-scan-changed = { $count } ændret
settings-library-scan-removed = { $count } fjernet
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = Anime: serier, film og hvordan de hænger sammen. Det, du ser, markeres hos dine trackere.
settings-library-tvmaze-about = Andet tv: live action, tegnefilm, dokumentarer. Intet markeres hos en tracker.
settings-library-tried-first = Prøves først
settings-library-tried-second = Prøves som nr. 2
settings-library-tried-third = Prøves som nr. 3
settings-library-tried-fourth = Prøves som nr. 4
settings-library-tried-nth = Prøves som nr. { $number }
settings-library-folders = Mapper
settings-library-folders-helper = Hver mappe inden i en af disse er en serie. En fil øverst i en Movies-mappe er en film.
settings-library-rescan-started = Genscanning startet
settings-library-load-failed = Dine mapper kunne ikke læses
settings-library-load-failed-helper = AniBeam kunne ikke læse sin liste over mapper. Prøv igen om lidt.
settings-library-no-folders = Ingen mapper endnu
settings-library-no-folders-helper = Peg AniBeam på mappen med din anime. Den finder selv serierne og filmene.
settings-library-scanned = Scannet { $when }
settings-library-add-a-folder = Tilføj en mappe
settings-library-scan-all = Scan alle
settings-library-scan-started = Scanning startet
settings-library-add-folder = Tilføj mappe
settings-library-matching = Matchning
settings-library-matching-helper = En ny mappe slås op i denne rækkefølge og beholder det første gode match. Et match, du sætter manuelt, bliver aldrig erstattet.
settings-library-try-later = Prøv senere
settings-library-try-first = Prøv først
settings-library-check-matches = Tjek matchene
settings-library-check-matches-helper = Siden Serieinfo viser hver serie med den info, AniBeam fandt, og retter en forkert.
settings-library-open-show-info = Åbn Serieinfo
settings-library-hidden-shows = Skjulte serier
settings-library-show-hidden = Vis skjulte serier
settings-library-show-hidden-helper = Viser de serier, du har skjult, på alle sider igen, indtil AniBeam lukker.
settings-library-hidden-from-airing = Skjult fra Sendes nu
settings-library-hidden-from-airing-helper = Serier, du har markeret som uinteressante. Hver bliver væk fra Sendes nu, indtil du henter den tilbage.
settings-library-hidden-from-airing-empty = Intet endnu. Serier, du markerer som uinteressante under Sendes nu, samles her, så de kan komme tilbage.
settings-library-show-again = Vis igen
settings-library-calendar = Kalender
settings-library-calendar-feed = Sendetider i en kalender
settings-library-calendar-feed-helper = Holder en kalenderfil med de kommende afsnit af serierne på dine lister, opdateret når sendeplanen ændrer sig.
settings-library-calendar-file = Kalenderfil
settings-library-calendar-file-helper = Abonnér på denne fil i din kalenderapp.
settings-library-copy = Kopier
settings-library-path-copied = Sti kopieret
settings-library-open-folder = Åbn mappe
settings-library-notifications = Notifikationer
settings-library-new-episodes = Nye afsnit
settings-library-new-episodes-helper = En notifikation på skrivebordet, når et afsnit af en serie på din Ser nu-liste sendes. Klik på den for at åbne serien.
settings-library-folder-added = Mappe tilføjet, scanner

## SettingsAppearanceTab

settings-appearance-language = Sprog
settings-appearance-language-system = Samme som systemet ({ $language })
settings-appearance-text-smaller = Mindre
settings-appearance-text-normal = Normal
settings-appearance-text-large = Stor
settings-appearance-text-larger = Større
settings-appearance-text-largest = Størst
settings-appearance-title = Udseende
settings-appearance-lead = Ændringer ses med det samme, her og på alle sider. De gemmes i ~/.config/anibeam/theme.toml, som du også kan redigere selv.
settings-appearance-colours = Farver
settings-appearance-mode = Tilstand
settings-appearance-mode-dark = Mørk
settings-appearance-mode-light = Lys
settings-appearance-mode-system = System
settings-appearance-colours-from = Farver fra
settings-appearance-from-theme-helper = Temaparret nedenfor, ét til mørk og ét til lys.
settings-appearance-from-desktop-helper = Din terminals farver, ellers skrivebordets farveskema og accent.
settings-appearance-from-desktop = Skrivebord
settings-appearance-from-theme = Tema
settings-appearance-dark-theme = Mørkt tema
settings-appearance-light-theme = Lyst tema
settings-appearance-themes-helper = Temafiler, du lægger i ~/.config/anibeam/themes, vises på begge lister.
settings-appearance-accent = Accent
settings-appearance-accent-helper = Markerer det valgte, det der afspilles og det nye.
settings-appearance-icon = Ikon
settings-appearance-app-icon = Appikon
settings-appearance-app-icon-helper = Vinduets, proceslinjens og launcherens. Mørk bytter det gule og det sorte.
settings-appearance-icon-default = Standard
settings-appearance-icon-default-name = Standardappikon
settings-appearance-icon-dark = Mørk
settings-appearance-icon-dark-name = Mørkt appikon
settings-appearance-size-motion = Størrelse og bevægelse
settings-appearance-text-size = Tekststørrelse
settings-appearance-text-size-helper = Hvor stor teksten er overalt i AniBeam, oven i systemets egen størrelse.
settings-appearance-density = Tæthed
settings-appearance-density-helper = Pladsen omkring alt, fra tæt til luftigt.
settings-appearance-density-compact = Kompakt
settings-appearance-density-normal = Normal
settings-appearance-density-comfortable = Luftig
settings-appearance-poster-size = Plakatstørrelse
settings-appearance-poster-size-helper = Hvor stor en serie er i biblioteket og på Hjem.
settings-appearance-poster-small = Lille
settings-appearance-poster-medium = Mellem
settings-appearance-poster-large = Stor
settings-appearance-out-now = Udkommet nu under Sendes
settings-appearance-out-now-helper = Kort med hver series banner eller en liste med små plakater og titler, tre rækker dyb.
settings-appearance-out-now-cards = Kort
settings-appearance-out-now-list = Liste
settings-appearance-out-now-grid = Kort under Udkommet nu
settings-appearance-out-now-grid-helper = Hvor mange kort der vises, kolonner gange rækker, før Alle åbner resten.
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Reducer bevægelse
settings-appearance-reduce-motion-helper = Springer bibliotekets åbningsbevægelse og anden ren pynt over.

## LookPane

look-pane-sample-series = Eksempelserie
look-pane-episode-code = Afs. { $number }
look-pane-library = Bibliotek
look-pane-series-count =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serier
    }
look-pane-search = Søg i titler eller mapper
look-pane-search-hint = / eller Ctrl+K
look-pane-all = Alle
look-pane-series = Serier
look-pane-movies = Film
look-pane-plain = Almindelig
look-pane-selected = Valgt
look-pane-errors =
    { $count ->
        [one] { $count } fejl
       *[other] { $count } fejl
    }
look-pane-type-large = Stor
look-pane-type-normal = Normal
look-pane-type-small = Lille, dæmpet
look-pane-sample-switch = Eksempelkontakt
look-pane-button = Knap
look-pane-scan = Scan
look-pane-sample-status =
    { $count ->
        [one] { $count } nyt afsnit i { $title }
       *[other] { $count } nye afsnit i { $title }
    }

## FeedPage

feed-title = Feed
feed-aired = { $episode } sendt { $when }
feed-downloaded = Fil downloadet { $when }
feed-countdown-days = { $days } d { $hours } t
feed-countdown-hours = { $hours } t { $minutes } m
feed-countdown-minutes = { $minutes } m
feed-section-this-week = Denne uge
feed-new-episodes =
    { $count ->
        [one] { $count } nyt afsnit
       *[other] { $count } nye afsnit
    }
feed-section-earlier = Tidligere
feed-latest-out = Senest udkommet { $when }
feed-section-coming-soon = Kommer snart
feed-this-week-count = { $count } denne uge
feed-section-everything-else = Alt andet
feed-out-this-week = { $count } udkommet denne uge
feed-nothing-scheduled = Intet planlagt endnu.
feed-next = Næste: { $title } om { $countdown }
feed-nothing-new = Intet nyt denne uge.
feed-new-this-week =
    { $count ->
        [one] { $count } nyt afsnit denne uge.
       *[other] { $count } nye afsnit denne uge.
    }
feed-sort-recent = Udkommet for nylig
feed-sort-upcoming = Kommer snart
feed-failed-title = Feedet kunne ikke indlæses
feed-empty-title = Intet her endnu
feed-failed-body = AniBeam kunne ikke læse feedet lige nu. Prøv igen om lidt.
feed-empty-body-folders = Feedet fyldes, efterhånden som afsnit sendes og kommer. Sendes nu viser, hvad der sendes denne sæson.
feed-empty-body = Tilføj mappen med din anime, så fyldes feedet med dens afsnit.
feed-add-folder = Tilføj en mappe
feed-airing-now = Sendes nu

## RecommendedPage

recommended-title = Anbefalet til dig
recommended-ranked-minutes = Rangeret for { $minutes } min. siden
recommended-ranked-hours = Rangeret for { $hours } t siden
recommended-ranked-days = Rangeret for { $days } d siden
recommended-scope-name = Anbefalet fra
recommended-scope-all = Alle
recommended-scope-library = I biblioteket
recommended-scope-list = På listen
recommended-ranking = Rangerer
recommended-rank-again = Ranger igen
recommended-no-scores-title = Bedøm et par serier først
recommended-no-list-title = Intet at lære af endnu
recommended-never-fitted-title = Rangerer dine serier
recommended-nothing-left-title = Intet tilbage her
recommended-no-scores-body = Anbefalinger lærer af karaktererne på din liste. Giv et par serier karakter på AniList, så vises de her.
recommended-no-list-body = Forbind AniList i Indstillinger, eller hold en liste på denne enhed, så lærer siden af de serier, du bedømmer.
recommended-never-fitted-body = Den første rangering læser AniList i et minut eller to. Den kører i baggrunden.
recommended-nothing-left-library-body = Alt i dit bibliotek, der passer, er set. Alle finder serier, du ikke har endnu.
recommended-nothing-left-body = Alt på dine lister, der passer, er set. Alle finder serier, du ikke har på en liste.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] Flytter 1 serie til { $site }
       *[other] Flytter { $count } serier til { $site }
    }
move-lists-score = karakter { $score }
move-lists-title = Flyt din liste til { $site }
move-lists-body =
    { $count ->
        [one] Én serie har
       *[other] { $count } serier har
    } en status eller karakter kun på denne computer. Flytter du dem, havner de på din { $site }-liste, hvor din telefon og din profil kan se dem. For disse serier erstattes det, { $site } har nu. Intet sendes, før du trykker Flyt.
move-lists-head-show = Serie
move-lists-head-here = På denne computer
move-lists-head-tracker = På { $site } nu
move-lists-nothing = Intet
move-lists-not-on-list = Ikke på din liste
move-lists-later-hint = Senere: Indstillinger, Konti
move-lists-not-now = Ikke nu
move-lists-move =
    { $count ->
        [one] Flyt 1 serie
       *[other] Flyt { $count } serier
    }

## StatsPage

stats-title = Statistik
stats-month-jan = Jan
stats-month-feb = Feb
stats-month-mar = Mar
stats-month-apr = Apr
stats-month-may = Maj
stats-month-jun = Jun
stats-month-jul = Jul
stats-month-aug = Aug
stats-month-sep = Sep
stats-month-oct = Okt
stats-month-nov = Nov
stats-month-dec = Dec
stats-nothing-yet = Intet endnu
stats-ranked-episodes =
    { $count ->
        [one] { $count } afs.
       *[other] { $count } afs.
    }
stats-year-summary = Hvad du så i { $year }
stats-empty-title = Intet set endnu
stats-empty-body = Se et afsnit færdigt, så tæller det her: timer, genrer og de sæsoner, du har set.
stats-open-library = Bibliotek
stats-hours-watched = timer set
stats-episodes-label =
    { $count ->
        [one] afsnit
       *[other] afsnit
    }
stats-series-finished = serier set færdige
stats-none = Ingen
stats-busiest-month = travleste måned
stats-by-month = Pr. måned
stats-month-hours = { $hours } t
stats-top-genres = Topgenrer
stats-top-studios = Topstudier
stats-seasons-watched = Sæsoner set
stats-recap-title = Mit år i anime
stats-recap-hours = timer
stats-recap-episodes = afsnit
stats-recap-top-genre = topgenre
stats-save-image = Gem som billede
stats-save-dialog-title = Gem opsummeringen
stats-save-filter-png = PNG-billede (*.png)
stats-recap-saved = Opsummering gemt
stats-recap-save-failed = Kunne ikke gemme opsummeringen

## TrackerRow

tracker-row-connected = Forbundet som { $user }, synkroniseret { $when }
tracker-row-unknown-user = ?
tracker-row-never = aldrig
tracker-row-waiting = Venter på, at du godkender i browseren
tracker-row-not-connected = Ikke forbundet
tracker-row-open-again = Åbn igen
tracker-row-open-again-tip = Åbn loginsiden igen
tracker-row-disconnect = Afbryd
tracker-row-disconnect-question = Afbryd { $name }? Dit login bliver fjernet.
tracker-row-secret-saved = Der er gemt en klienthemmelighed til denne app.
tracker-row-remove-secret = Fjern den
tracker-row-secret-removed = Klienthemmelighed fjernet
tracker-row-add-secret = Tilføj klienthemmelighed
tracker-row-secret-where = Det er den lange streng under "Client Secret" på appens side på myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Klienthemmelighed
tracker-row-save-connect = Gem og forbind

## RemoveSheet

remove-unsubscribe-delete = Afmeld og slet
remove-unsubscribe-torrents = Afmeld og fjern torrents
remove-unsubscribe = Afmeld
remove-delete-files = Fjern og slet filer
remove-torrents =
    { $count ->
        [one] Fjern torrenten
       *[other] Fjern torrents
    }
remove-also-torrents =
    { $count ->
        [one] Fjern også dens torrent
       *[other] Fjern også dens { $count } torrents
    }
remove-torrents-go-with-files = Når filerne slettes, fjernes torrents også.
remove-torrents-files-stay = De fjernes fra programmet; filerne bliver.
remove-also-files = Slet også filerne
remove-files-default-where = det, torrents downloadede
remove-files-detail-rule = Flytter { $where } til papirkurven og fjerner torrents.
remove-files-detail = Flytter { $where } til papirkurven.

## NotesSheet

notes-your-tracker = din tracker
notes-title = Noter
notes-placeholder = Hvor du slap, hvem der anbefalede den, hvilken udgivelse du skal hente
notes-over-limit = { $count } tegn over grænsen
notes-saved-to = Gemt på { $sites }

## RecommendationCard

recommendation-card-available = Tilgængelig
recommendation-card-open-library = Åbn { $title } i dit bibliotek
recommendation-card-open-anilist = Åbn { $title } på AniList

## SearchField

search-field-placeholder = Søg i titler eller mapper
search-field-hint = / eller Ctrl+K
search-field-clear = Ryd søgningen

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent } %
recommended-hover-hidden-tags =
    { $count ->
        [one] Et skjult tag, du elsker
       *[other] { $count } skjulte tags, du elsker
    }
recommended-hover-because = Fordi du så

## PersonCard

person-card-unknown = Ukendt
person-card-role-main = Hovedrolle
person-card-role-supporting = Birolle
person-card-role-background = Baggrund
person-card-the-character = figuren
person-card-open = Åbn { $name } på AniList

## StatusDot

status-dot-on-list = På din liste: { $status }

## TrackerNudge

tracker-nudge-title = Forbind din animeliste
tracker-nudge-body = AniBeam markerer afsnit på AniList eller MyAnimeList, mens du ser.
tracker-nudge-connect-anilist = Forbind AniList
tracker-nudge-connect-mal = Forbind MyAnimeList
tracker-nudge-not-now = Ikke nu

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } procent for dig
recommended-card-forecast = { $percent } %
recommended-card-in-library = I biblioteket

## InlineConfirm

inline-confirm-keep = Behold

## Tokens

tokens-cube-watched-tracked = Set, tracket online
    .inline = set, tracket online
tokens-cube-watched-here = Set her, ikke tracket endnu
    .inline = set her, ikke tracket endnu
tokens-cube-on-disk = På disken
    .inline = på disken
tokens-cube-downloading = Downloader
    .inline = downloades
tokens-cube-missing = Mangler
    .inline = mangler
tokens-cube-error = Fejl
    .inline = fejl
tokens-cube-not-downloaded = Ikke downloadet
    .inline = ikke downloadet
tokens-cube-not-aired = Ikke sendt endnu
    .inline = ikke sendt endnu
tokens-cube-available = Tilgængelig
    .inline = tilgængelig
tokens-cube-picked = Valgt til at hente
    .inline = valgt til at hente
tokens-cube-already-on-disk = Allerede på disken
    .inline = allerede på disken
tokens-cube-no-release = Ingen udgivelse
    .inline = ingen udgivelse

## Card

card-not-interested = Ikke interesseret

## PagePlaceholder

page-placeholder-coming-soon = Kommer snart

## ErrorLine

error-line-hide-details = Skjul detaljer
error-line-details = Detaljer
error-line-details-tip = Vis præcis, hvad der gik galt, og kopier det
error-line-copied = Kopieret.

## Readouts (apps/linux/src/format.rs): times, countdowns and lengths

fmt-just-now = lige nu
fmt-now = nu
fmt-ago = for { $span } siden
fmt-in = om { $span }
fmt-seconds-short = { $n } s
fmt-minutes-short = { $n } m
fmt-hours-short = { $n } t
fmt-days-short = { $n } d
fmt-months-short = { $n } md.
fmt-years-short = { $n } år
fmt-years-months-short = { $years } år { $months } md.
fmt-countdown-days = { $d } d { $h } t { $m } m
fmt-countdown-hours = { $h } t { $m } m
fmt-countdown-days-seconds = { $d } d { $h } t { $m } m { $s } s
fmt-countdown-hours-seconds = { $h } t { $m } m { $s } s
fmt-countdown-minutes-seconds = { $m } m { $s } s
fmt-runtime-minutes = { $m } min
fmt-runtime-hours = { $h } t
fmt-runtime-hours-minutes = { $h } t { $m } min
fmt-episode = Afsnit
fmt-size-b = { $size } B
fmt-size-kb = { $size } KB
fmt-size-mb = { $size } MB
fmt-size-gb = { $size } GB
fmt-size-tb = { $size } TB
fmt-episode-span = Afsnit { $first } til { $through }

## apps/linux/src/problem.rs: every call error in a person's words, what happened and what to do

problem-report = Prøv igen. Hvis det bliver ved, så meld det fra aktivitetsloggen.
problem-try-later = Prøv igen senere.
problem-helper-unreachable = Søgehjælperen svarede ikke.
problem-helper-unauthorized = Søgehjælperen afviste login.
problem-helper-not-running = Søgehjælperen kører ikke.
problem-helper-missing = Søgehjælperen er ikke installeret.
problem-helper-failed = Søgehjælperen kunne ikke gøre det.
problem-helper-no-login = Søgehjælperen kan ikke logge ind i torrentprogrammet.
problem-helper-no-login-next = Gem torrentprogrammets adgangskode i Indstillinger under Downloads.
problem-torrent-unreachable = Torrentprogrammet svarede ikke.
problem-torrent-unauthorized = Torrentprogrammet afviste login.
problem-torrent-not-running = Torrentprogrammet kører ikke.
problem-torrent-missing = Torrentprogrammet er ikke installeret.
problem-torrent-failed = Torrentprogrammet kunne ikke gøre det.
problem-downloads-unreachable-next = Tjek, at det kører, og at adressen er rigtig i Indstillinger under Downloads.
problem-downloads-unauthorized-next = Tjek brugernavn og adgangskode i Indstillinger under Downloads.
problem-downloads-not-running-next = Start det fra Indstillinger under Downloads.
problem-downloads-missing-next = Indstillinger under Downloads forklarer, hvordan det sættes op.
problem-trash = { -app-name } kunne ikke flytte filerne til papirkurven.
problem-trash-next = Tjek, at filerne er dine, og at disken ikke er skrivebeskyttet.
problem-source-list = Den kildeliste virkede ikke.
problem-source-list-next = Tjek dens adresse i Indstillinger under Downloads, Kildelister.
problem-remote-unreachable = { $name } svarede ikke.
problem-remote-unreachable-next = Siden er måske nede. Prøv igen senere, eller søg i en anden kilde.
problem-remote-unauthorized = { $name } afviste den nøgle, den fik.
problem-remote-unauthorized-next = Skift dens nøgle i Indstillinger under Downloads, Kildelister.
problem-remote-failed = { $name } kunne ikke gøre det.
problem-remote-failed-next = Prøv igen senere, eller søg i en anden kilde.
problem-provider-sign-in = { $name } afviste { -app-name }s login.
problem-provider-sign-in-next = Forbind den igen i Indstillinger under Konti.
problem-provider-turned-away = { $name } afviste { -app-name }.
problem-provider-busy = { $name } har travlt lige nu.
problem-provider-busy-next = Vent et minut, og prøv igen.
problem-provider-no-answer = { $name } svarede ikke.
problem-provider-no-answer-next = Tjek din internetforbindelse, og prøv igen.
problem-provider-trouble = { $name } har problemer lige nu.
problem-provider-trouble-next = Prøv igen om et par minutter.
problem-provider-rejected = { $name } accepterede det ikke.
problem-not-found = { -app-name } kunne ikke finde det længere.
problem-not-found-next = Det er måske blevet fjernet. Prøv igen for at indlæse siden forfra.
problem-invalid-next = Tjek det, og prøv igen.
problem-unavailable = Den mappe kan ikke nås lige nu.
problem-unavailable-next = Tilslut drevet, eller tjek mappen i Indstillinger under Bibliotek.
problem-not-connected = { $tracker } er ikke forbundet.
problem-not-connected-next = Forbind den i Indstillinger under Konti.
problem-io = { -app-name } kunne ikke læse eller skrive en fil.
problem-io-next = Tjek, at der er plads på disken, og at mappen er din, og prøv igen.
problem-storage = { -app-name } kunne ikke gemme dit bibliotek.
problem-storage-next = Tjek, at der er plads på disken, og prøv igen.
problem-keyring = Systemets adgangskodelager svarede ikke.
problem-keyring-next = Lås dit skrivebords nøglering op, og prøv igen.
problem-unsupported = { -app-name } kan ikke gøre det her endnu.
problem-version = Denne backup kommer fra en nyere { -app-name }.
problem-version-next = Opdater { -app-name } i Indstillinger under Om, og importer den igen.
problem-internal = Noget gik galt inde i { -app-name }.
problem-refused-hidden = Den serie er skjult.
problem-refused-hidden-next = Vis den igen fra dens side først.
problem-refused-no-match = Den serie er ikke matchet endnu.
problem-refused-no-match-next = Match den under Serieinfo først.
problem-refused-not-newer = Din liste er allerede forbi dette afsnit.
problem-refused-extra = Den fil er en ekstra, ikke et afsnit.
problem-refused-unmatched = Den fil er ikke knyttet til et afsnit.
problem-refused-unmatched-next = Ret det fra seriens side under Rettelser.
problem-refused-on-disk = Den fil findes kun på denne computer.

## apps/linux/src/library_view.rs: the library page's section titles and the one fact under each
## (Dropped, On hold and Watching reuse status-* from core.ftl)

library-view-airing = Sendes
library-view-no-date = Ingen dato
library-view-type-tv = TV
library-view-type-movies = Film
library-view-type-extras = OVA, ONA og specials
library-view-type-music = Musik
library-view-type-unknown = Ukendt
library-view-no-genre = Umatchet
library-view-state-caught-up = Helt med
library-view-state-not-started = Ikke startet
library-view-next-episode = Næste afsnit om { $span }
library-view-countdown-days = { $d } d { $h } t
library-view-countdown-hours = { $h } t { $m } m
library-view-countdown-minutes = { $m } m
library-view-in-progress = { $count } i gang
library-view-not-started = { $count } ikke startet
library-view-all-watched = Alt set
library-view-on-disk =
    { $count ->
        [one] { $count } afsnit
       *[other] { $count } afsnit
    } på disken
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } fil
       *[other] { $count } filer
    } stadig på disken

## apps/linux/src/dbus/notify.rs: the desktop notification for a new episode
## (its Open button reuses action-open from core.ftl)

notify-episode-out = Afsnit { $episode } af { $title } er udkommet
notify-click-open = Klik for at åbne det i { -app-name }
notify-click-get = Klik for at hente afsnittene i { -app-name }
notify-get-episodes = Hent afsnit

## apps/linux/src/nowplaying.rs: the second line a media widget shows while an episode plays

now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: why the screen stays on, as the desktop's power panel shows it

inhibit-reason = Afspiller et afsnit

## apps/linux/src/bridge/door.rs: why a value a page sent cannot be used (an error's detail line)

door-not-a-choice = { $value } er ikke et gyldigt valg
door-tokens-op-empty = en tokens-op skal have tokens
door-not-a-refine-op = { $value } er ikke en indsnævringsoperation
door-limit-too-low = skal være mindst 1, fik { $limit }
door-not-a-torrent-action = { $value } er ikke en torrenthandling

## apps/linux/src/bridge/shell.rs: the first-run starter folder and the update notice

shell-starter-folder-failed = Kunne ikke oprette { $folder }: { $error }
shell-update-gone = { -app-name } { $version } tilbydes ikke længere

## Watch together (#846)

onboarding-step-together = Se sammen
onboarding-title-together = Se med venner
onboarding-body-together = Start et rum fra afspilleren, og send koden. Alle afspiller deres egen kopi af afsnittet, og afspil, pause og spol følges ad for jer alle.
onboarding-ready-together = Se sammen
onboarding-ready-together-as = Rum viser dig som { $name }.
onboarding-ready-together-none = Intet navn endnu. Rumpanelet spørger første gang.
onboarding-together-you = Dig
onboarding-together-code = Rumkode
onboarding-together-episode = Frieren · Afsnit 3
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = Dit navn i rum
rail-room = Se sammen (Ctrl+J)
rail-room-in = I et se sammen-rum (Ctrl+J)
rail-join-room = Gå ind i rum
rail-in-room = I et rum
player-chrome-watch-together = Se sammen
player-chrome-room = Rum { $code } · { $count ->
    [one] 1 person
   *[other] { $count } personer
}
room-name-empty = Skriv det navn, de andre i rummet skal se.
room-name-bad = Det navn virker ikke. Brug 1 til 32 tegn.
room-error-not-found = Intet rum har den kode. Tjek den med den, der sendte den; rummet er måske lukket.
room-error-full = Det rum er fuldt. Et rum har plads til 10 personer.
room-error-unreachable = Kunne ikke nå rumserveren. Tjek din forbindelse, og prøv igen.
room-error-refused = Rumserveren afviste det. Prøv igen om et minut.
room-error-proto = Det rum kører på en nyere version af rum. Opdater { -app-name } for at gå ind i det.
room-error-not-in-room = Du er ikke længere i det rum. Start eller gå ind i et igen.
room-error-bad-code = En rumkode er 6 bogstaver og tal, fx K7QM2X.
room-error-bad-relay = Rumserveren i Indstillinger, Afspilning er ikke en webadresse. Ret den der, og prøv igen.
room-panel-title = Se sammen
room-panel-close = Luk
room-panel-connected = Forbundet
room-panel-reconnecting = Forbinder igen
room-panel-name-label = Dit navn i rum
room-panel-name-placeholder = Dit navn
room-panel-name-start = Start rummet
room-panel-name-join = Gå ind i rummet
room-panel-start = Start et rum
room-panel-or-join = Eller gå ind i et med dets kode
room-panel-join-label = Rumkode
room-panel-join = Gå ind
room-panel-you-are = Du vises som { $name }
room-panel-change-name = Skift
room-panel-code-label = Rumkode
room-panel-copy = Kopier link
room-panel-link-copied = Rumlink kopieret
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = Dette afsnit er ikke i dit bibliotek, så du kan ikke se med endnu.
room-panel-open-episode = Åbn afsnittet
room-panel-people = { $count ->
    [one] 1 person
   *[other] { $count } personer
}
room-panel-host = Vært
room-panel-you = Dig
room-panel-ready = Klar
room-panel-opening = Åbner
room-panel-missing = Har det ikke
room-panel-offset = Din forskydning
room-panel-offset-earlier = Et halvt sekund tidligere
room-panel-offset-later = Et halvt sekund senere
room-panel-seconds = { $value } s
room-panel-suggested = Introerne foreslår { $offset }
room-panel-use-suggested = Brug den
room-panel-leave = Forlad rummet
player-room-speed = Hastigheden følger rummet
settings-find-room-name =
    .words = se sammen rum navn venner
settings-find-room-relay =
    .words = se sammen rum relæ server
key-help-key-ctrl-j = Ctrl+J
key-help-room = Se sammen
settings-playback-together = Se sammen
settings-playback-room-name = Dit navn i se sammen-rum
settings-playback-room-name-placeholder = Dit navn
settings-playback-room-relay = Rumserver
settings-playback-room-relay-helper = Rum oprettes her. Skift den kun til en server, du selv kører.
