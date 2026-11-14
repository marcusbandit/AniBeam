# nl translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = Start
nav-library = Bibliotheek
nav-airing = Nu uitgezonden
nav-feed = Feed
nav-watching = Kijken
nav-stats = Stats
nav-metadata = Serie-info
nav-downloads = Downloads
nav-settings = Instellingen
nav-series = Serie
nav-player = Speler
nav-show = Serie
nav-recommended = Aanbevolen

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } fout
   *[other] { $count } fouten
}

## ScanSummary

scan-summary-nothing = Nog niets te tonen
scan-summary-found = { $count ->
    [one] { $count } serie
   *[other] { $count } series
} gevonden, { $recognised } herkend
scan-summary-found-need = { $count ->
    [one] { $count } serie
   *[other] { $count } series
} gevonden, { $recognised } herkend, { $need } hebben jou nodig
scan-summary-episodes = { $count ->
    [one] { $count } aflevering
   *[other] { $count } afleveringen
} op schijf.
scan-summary-episodes-watched = { $count ->
    [one] { $count } aflevering
   *[other] { $count } afleveringen
} op schijf, { $watched } al gekeken op { $tracker }.
scan-summary-title = Je map is binnen
scan-summary-count-found = gevonden
scan-summary-count-recognised = herkend
scan-summary-count-need = jij nodig
scan-summary-later = Later
scan-summary-match = Koppel de { $count }
scan-summary-open-library = Bibliotheek openen

## UpdateNotice

update-notice-check-failed = AniBeam kon niet op updates controleren. Controleer je internetverbinding en probeer het opnieuw.
update-notice-up-to-date = AniBeam is bijgewerkt
update-notice-downloading = AniBeam { $version } downloaden
update-notice-ready = AniBeam { $version } staat klaar
update-notice-failed = De update is mislukt
update-notice-on-aur = AniBeam { $version } staat in de AUR
update-notice-out = AniBeam { $version } is uit
update-notice-out-titled = AniBeam { $version } “{ $title }” is uit
update-notice-progress = { $percent }%  ·  { $got } van { $total } MB
update-notice-restart-hint = Herstart nu, of hij opent de volgende keer
update-notice-from-source = Deze kopie is uit de broncode gebouwd: pull en bouw opnieuw
update-notice-no-download = Geen download voor deze computer
update-notice-install-hint = Bijwerken installeert hem en herstart AniBeam
update-notice-retry = Opnieuw
update-notice-update = Bijwerken
update-notice-restart = Herstarten
update-notice-copied = Gekopieerd
update-notice-copy = Kopieer
update-notice-release-page = Releasepagina
update-notice-later = Later

## WhatsNew

whats-new-title = Wat is nieuw
whats-new-got-it = Begrepen

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = d MMM HH:mm
activity-drawer-stage-library = Bibliotheek
activity-drawer-stage-metadata = Metadata
activity-drawer-stage-trackers = Trackers
activity-drawer-stage-franchise = Franchise
activity-drawer-stage-playback = Afspelen
activity-drawer-stage-store = Opslag
activity-drawer-stage-system = Systeem
activity-drawer-stage-downloads = Downloads
activity-drawer-level-info = Info
activity-drawer-level-warn = Waarschuwing
activity-drawer-level-error = Fout
activity-drawer-copied-line = Regel gekopieerd
activity-drawer-title = Activiteit
activity-drawer-everything = Alles
activity-drawer-problems = Problemen
activity-drawer-copied = Gekopieerd
activity-drawer-copy = Kopieer
activity-drawer-clear-every = Alle regels wissen
activity-drawer-copy-tip = Klik om de regel te kopiëren
activity-drawer-fold = Inklappen
activity-drawer-empty-technical = Nog niets gelogd.
activity-drawer-empty-problems = Geen problemen. Alles werkte.
activity-drawer-empty = Nog niets te melden. Wat AniBeam voor je doet verschijnt hier.

## ActivityColumn

activity-column-episode-run = Afleveringen { $first } t/m { $last }
activity-column-episode-list = Afleveringen { $list }
activity-column-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
activity-column-batch = Batchdownload
activity-column-download = Download
activity-column-name-batch = { $title }, een batch
activity-column-name-episode = { $title }, aflevering { $number }
activity-column-title = Activiteit
activity-column-down = omlaag
activity-column-up = omhoog
activity-column-not-connected = { $client } is niet gekoppeld, dus hier beweegt niets.
activity-column-connect-again = Opnieuw koppelen
activity-column-downloading = Nu aan het downloaden
activity-column-fewer = Minder
activity-column-more = En nog { $count }
activity-column-nothing-downloading = Er wordt niets gedownload.
activity-column-finished-today = Vandaag klaar
activity-column-today = Vandaag
activity-column-today-empty = Zoekopdrachten, downloads en nieuwe abonnementen verschijnen hier zodra ze gebeuren.
activity-column-stopping = Stoppen
activity-column-starting = Starten
activity-column-start-tip = Downloaden starten
activity-column-stop-tip = Downloaden stoppen; wat er al is blijft
activity-column-cancelling = Annuleren
activity-column-cancel-all-tip = Deze downloads annuleren
activity-column-cancel-tip = Deze download annuleren
activity-column-size-of = { $got } van { $total }
activity-column-files-missing = Bestanden ontbreken
activity-column-time-format = HH:mm
activity-column-left = nog { $time }
activity-column-waiting-peers = Wachten op peers
activity-column-episodes-done = { $count ->
    [one] { $done } van { $count } aflevering klaar
   *[other] { $done } van { $count } afleveringen klaar
}
activity-column-cube-skipped = Aflevering { $number }: overgeslagen
activity-column-cube-done = Aflevering { $number }: klaar
activity-column-cube-progress = Aflevering { $number }: { $percent }%

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Klaar
frame-strip-progress = { $doing }, { $done } van { $total }
frame-strip-idle = Er loopt niets
frame-strip-time-format = HH:mm
frame-finish-sign-in = Rond eerst het inloggen bij { $tracker } af
frame-did-not-work = Dat werkte niet.
frame-could-not-open = AniBeam kon dat niet openen. Kopieer het en open het zelf.
frame-copy = Kopieer
frame-copied = Gekopieerd
frame-status-refused = { $tracker } nam het niet aan
frame-status-refused-why = { $tracker } nam het niet aan: { $why }
frame-lists-refused = { $tracker } nam { $count ->
    [one] 1 serie
   *[other] { $count } series
} niet aan; zie het activiteitenlog
frame-lists-moved = { $count ->
    [one] 1 serie
   *[other] { $count } series
} naar { $tracker } verhuisd
frame-notes-refused = { $tracker } nam je notities niet aan
frame-notes-refused-why = { $tracker } nam je notities niet aan: { $why }
frame-notes-saved = Notities opgeslagen
frame-connected = Gekoppeld met { $tracker } als { $user }
frame-sign-in-stopped = Het inloggen bij { $tracker } stopte voordat het klaar was.
frame-toast-hide-details = Details verbergen
frame-toast-details = Details
frame-toast-details-tip = Toon letterlijk wat er misging, en kopieer het
frame-forward = Vooruit
frame-copy-magnet = Magnet kopiëren
frame-copy-link = Torrentlink kopiëren
frame-magnet-copied = Magnet gekopieerd
frame-link-copied = Link gekopieerd
frame-open-release-page = Releasepagina openen
frame-open-folder = Map openen
frame-copy-path = Pad kopiëren
frame-path-copied = Pad gekopieerd
frame-mark-unwatched = Vanaf hier als ongekeken markeren
frame-mark-watched = Tot hier als gekeken markeren
frame-copy-title = Titel kopiëren
frame-open-on-anilist = Openen op AniList
frame-list-home-online = Wijzig het op elke gekoppelde lijst
frame-list-home-unmatched = Bewaard op dit apparaat: geen tracker heeft deze serie
frame-list-home-not-carried = Bewaard op dit apparaat: je tracker heeft deze serie niet
frame-list-home-offline = Bewaard op dit apparaat tot je AniList of MyAnimeList koppelt
frame-moving-to = Naar { $list } zetten
frame-moved-to = Naar { $list } gezet
frame-plan-added = { $title } komt op je Gepland-lijst
frame-plan-listed = { $title } staat al op je lijst
frame-delete-watched = { $kind ->
    [film] Gekeken films wissen
   *[episode] Gekeken afleveringen wissen
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] De film wissen
       *[episode] De aflevering wissen
    }
   *[other] { $kind ->
        [film] Alle { $count } films wissen
       *[episode] Alle { $count } afleveringen wissen
    }
}
frame-series-gone = Die serie is weg
frame-delete-none-watched = Geen gekeken afleveringen van { $title } op schijf
frame-delete-none = Niets van { $title } op schijf
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } gekeken film
           *[other] { $count } gekeken films
        }
       *[episode] { $count ->
            [one] { $count } gekeken aflevering
           *[other] { $count } gekeken afleveringen
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } film
           *[other] { $count } films
        }
       *[episode] { $count ->
            [one] { $count } aflevering
           *[other] { $count } afleveringen
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } extra
   *[other] { $count } extra's
}
frame-delete-and = { $first } en { $second }
frame-delete-heading = { $what } van { $title } wissen?
frame-delete-base = De bestanden en hun ondertitels gaan naar de prullenbak, dus je kunt ze daar terughalen. Hun torrents verdwijnen uit de torrentclient, zodat ze opnieuw gedownload kunnen worden.
frame-delete-confirm = Naar prullenbak
frame-delete-done = { $what } van { $title } naar de prullenbak verplaatst
frame-delete-watched-done = { $count ->
    [one] { $count } aflevering
   *[other] { $count } afleveringen
}, { $size }, naar de prullenbak verplaatst
frame-delete-nothing = Niets te wissen
frame-fix-not-drafted = Hersteld. Het issue kon niet worden opgesteld
frame-fix-agent = Hersteld. { $agent } kijkt ernaar, en het issue opent als het klaar is
frame-fix-opening = Hersteld. Het issue wordt geopend
frame-fix-not-saved = Het herstel is niet opgeslagen: { $error }
frame-fix-undone = { $count ->
    [one] { $count } bestand wordt
   *[other] { $count } bestanden worden
} weer op naam gelezen
frame-fix-done = Hersteld hoe { $count ->
    [one] { $count } bestand wordt
   *[other] { $count } bestanden worden
} gelezen
frame-report-failed = Het issue kon niet worden opgesteld: { $error }
frame-report-untriaged = Het issue staat open in je browser, zonder triage. { $error }
frame-report-open = Het issue staat open in je browser, klaar om te versturen
frame-downloads-not-read = Niet gelezen.

## HomePage

home-title = Start
home-scan-started = Scan gestart
home-ago-minutes = { $count } min geleden
home-ago-hours = { $count } u geleden
home-ago-days = { $count } d geleden
home-today = Vandaag
home-tomorrow = Morgen
home-yesterday = Gisteren
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d MMM
home-day-date-format = ddd d MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = Nog geen datum
home-unit-days = { $count ->
    [one] dag
   *[other] dagen
}
home-unit-hours = u
home-unit-minutes = m
home-week-late = { $count ->
    [one] { $count } aflevering is laat
   *[other] { $count } afleveringen zijn laat
}
home-week-waiting = { $count ->
    [one] { $count } aflevering uit om te kijken
   *[other] { $count } afleveringen uit om te kijken
}
home-week-coming = { $count ->
    [one] { $count } aflevering in de komende zeven dagen
   *[other] { $count } afleveringen in de komende zeven dagen
}
home-week-next = Hierna: { $title } over { $time }
home-countdown-days = { $days }d { $hours }u
home-countdown-hours = { $hours }u { $minutes }m
home-countdown-minutes = { $minutes }m
home-out-not-downloaded = Nog niet gedownload
home-out-downloading = Downloaden
home-out-when = { $day } { $time }
home-next-up = Hierna
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, tijd geschat
home-next-when-season = Seizoen { $season }, { $span }, { $time }
home-next-when-season-estimated = Seizoen { $season }, { $span }, { $time }, tijd geschat
home-no-date = Geen datum
home-tile-watched = gekeken
home-tile-downloading = downloaden
home-tile-out = uit { $ago }
home-tile-not-out = nog niet uit
home-tile-estimated = geschat
home-tile-season-span = S{ $season } { $span }
home-tile-to-watch = { $count } te kijken
home-day-nothing = Niets
home-now = Nu { $time }
home-card-aired = { $episode } uit { $ago }
home-continue = Verder kijken
home-continue-fact = { $count } series om mee verder te gaan
home-on-hold = In de wacht
home-on-hold-fact = { $count } series om naar terug te keren
home-hero-open = { $title } openen
home-hero-pick-up = Ga verder waar je stopte
home-hero-up-next = Hierna
home-hero-downloading = { $count ->
    [one] { $count } aflevering
   *[other] { $count } afleveringen
} downloaden
home-hero-min-left = nog { $count } min
home-hero-watched = { $watched } van { $total } gekeken
home-hero-resume = Hervat aflevering { $number }
home-hero-play = Speel aflevering { $number }
home-hero-resume-unnumbered = Aflevering hervatten
home-hero-play-unnumbered = Aflevering afspelen
home-downloaded = Recent gedownload
home-downloaded-fact = { $count } series met nieuwe bestanden deze twee weken
home-week = Deze week
home-week-show-empty = Lege dagen tonen
home-week-hide-empty = Lege dagen verbergen
home-week-airing = Nu uitgezonden
home-week-out = Nu uit
home-week-coming-up = Binnenkort
home-recommended = Aanbevolen voor jou
home-recommended-fact = Uit je eigen lijst
home-recommended-scope = Aanbevolen uit
home-recommended-all = Alles
home-recommended-in-library = In bibliotheek
home-recommended-on-list = Op lijst
home-recommended-none-library = Niets in je bibliotheek valt nog aan te raden. Alles vindt series die je nog niet hebt.
home-recommended-none-list = Niets op je lijsten valt nog aan te raden. Alles vindt series die niet op je lijsten staan.
home-catch-up = Bijkijken
home-catch-up-fact = { $count } series met afleveringen die wachten
home-fresh = Begin iets nieuws
home-fresh-fact = { $count } series niet begonnen
home-genres = Genres
home-open-library = Bibliotheek openen
home-dropped = Gestopt
home-dropped-fact = { $count ->
    [one] { $count } bestand nog op schijf
   *[other] { $count } bestanden nog op schijf
}
home-out-now = Nu uit
home-out-now-fact = Nieuwe afleveringen deze week, van AniList
home-popular = Populair dit seizoen
home-add-folder-title = Je eigen series komen hier ook
home-add-folder-body = Voeg de map toe waar je anime staat en Start vult zich met wat je kijkt.
home-add-folder-button = Map toevoegen
home-empty-scanned-title = Nog niets gevonden in je mappen
home-empty-title = Nog geen series
home-empty-scanned-body = AniBeam vond geen series of films in de mappen die het bewaakt. Scan ze opnieuw, of controleer de mappen in Instellingen.
home-empty-body = Voeg de map toe waar je anime staat en de bibliotheek vult zichzelf.
home-scan-all = Alles scannen
home-library-settings = Bibliotheekinstellingen
home-failed-title = Start is niet geladen
home-failed-body = AniBeam kon de bibliotheek nu niet lezen. Probeer het zo opnieuw.

## Onboarding

onboarding-language = Taal
onboarding-step-welcome = Welkom
onboarding-step-library = Je bibliotheek
onboarding-step-lists = Je lijst
onboarding-step-downloads = Downloads
onboarding-step-ready = Klaar
onboarding-scan-reading = De map lezen
onboarding-scan-found = { $count } series gevonden. Ga verder wanneer je wilt; de rest vult zichzelf aan.
onboarding-scan-starter-empty = Je nieuwe map is voorlopig leeg. Series die je erin zet verschijnen hier vanzelf.
onboarding-scan-no-video = Geen videobestanden gevonden. Controleer of het de juiste map is.
onboarding-scan-your-shows = je series
onboarding-scan-looking-up = { $what } opzoeken
onboarding-scan-looking-up-tally = { $what } opzoeken, { $posters } van { $count } gevonden
onboarding-scan-posters = Posters ophalen
onboarding-scan-posters-tally = Posters ophalen, { $posters } van { $count } gevonden
onboarding-scan-seasons = Verwante seizoenen zoeken
onboarding-scan-seasons-tally = Verwante seizoenen zoeken, { $posters } van { $count } gevonden
onboarding-choose-folder = Kies je animemap
onboarding-about = Een minuut instellen. Je bibliotheek maakt zichzelf af.
onboarding-step-later = Later
onboarding-skip-setup = Overslaan
onboarding-title-welcome = Welkom bij AniBeam
onboarding-title-library = Waar staat je anime?
onboarding-title-lists = Koppel je animelijst
onboarding-title-downloads = Zo werken downloads
onboarding-title-ready = Je bent klaar
onboarding-title-later = { $count ->
    [1] Klaar, één ding voor later
    [2] Klaar, twee dingen voor later
   *[other] Klaar, drie dingen voor later
}
onboarding-body-welcome = AniBeam houdt je anime op één plek. Het vindt de series op je schijf, speelt ze hier af en houdt AniList of MyAnimeList bij terwijl je kijkt.
onboarding-body-library = Kies de map waarin je je series bewaart. Elke map daarin wordt een serie, met poster, afleveringen en waar je bleef. AniBeam leest de map alleen. Het verplaatst of hernoemt nooit een bestand.
onboarding-body-lists = Log één keer in, en elke aflevering die je hier afkijkt wordt op je lijst gemarkeerd, zonder iets te typen. Het inloggen gebeurt in je browser, dus AniBeam ziet je wachtwoord nooit.
onboarding-body-downloads = Volg een serie op de pagina Downloads en elke nieuwe aflevering komt in je bibliotheek zodra hij uitkomt. Daarvoor zijn twee dingen nodig: een plek om te zoeken en een torrentclient om op te halen.
onboarding-body-ready = Alles hier kun je later wijzigen in Instellingen, en Instellingen, Over start deze introductie opnieuw.
onboarding-skip-step = Nu overslaan
onboarding-get-started = Aan de slag
onboarding-open-app = AniBeam openen
onboarding-continue = Verder
onboarding-payoff-library = Je bibliotheek
onboarding-payoff-library-line = Posters, afleveringen en wat er hierna komt, voor elke serie in je mappen.
onboarding-payoff-plays = Speelt hier af
onboarding-payoff-plays-line = Ondertitels, intro overslaan, en het onthoudt waar je stopte.
onboarding-payoff-list = Je lijst, gesynct
onboarding-payoff-list-line = Kijk een aflevering af en hij telt mee op AniList of MyAnimeList.
onboarding-payoff-episodes = Afleveringen komen binnen
onboarding-payoff-episodes-line = Volg een serie en elke nieuwe aflevering komt in je bibliotheek.
onboarding-films-too = Een map met films werkt ook: noem hem Movies.
onboarding-choose-folder-button = Map kiezen
onboarding-no-folder = Ik heb nog geen map
onboarding-starter-where = Dat maakt een map Anime in { $dir } en gebruikt die.
onboarding-add-another = Nog een map toevoegen
onboarding-shelf-more = meer
onboarding-preview-show = Je serie
onboarding-preview-finished = Aflevering 3 hier afgekeken
onboarding-preview-on-list = Op je lijst: 3 gekeken
onboarding-preview-set-by = Door AniBeam ingesteld zodra de aftiteling begint
onboarding-lists-connected = Gekoppeld. Wat je hier kijkt telt nu mee op je lijst.
onboarding-lists-no-account = Nog geen account? Beide zijn gratis, en één is genoeg. Je kunt altijd ontkoppelen in Instellingen, Accounts.
onboarding-search-title = Een plek om te zoeken
onboarding-search-ready = { $count ->
    [one] { $count } bron klaar om te doorzoeken.
   *[other] { $count } bronnen klaar om te doorzoeken.
}
onboarding-search-none = AniBeam heeft geen eigen bronnen. Je voegt ze toe uit een bronrepo: een link die iemand met je deelt, en die je plakt in Instellingen, Downloads.
onboarding-sources-after = Opent na de introductie
onboarding-sources-add = Repo toevoegen na introductie
onboarding-fetch-title = Wie haalt op
onboarding-fetch-auto = Laat AniBeam het regelen
onboarding-fetch-recommended = Aanbevolen
onboarding-fetch-auto-line = AniBeam brengt zijn eigen qBittorrent mee en draait die stil op de achtergrond. Niets te installeren, niets in te vullen.
onboarding-fetch-existing = Ik gebruik al qBittorrent
onboarding-fetch-existing-line = AniBeam koppelt met de qBittorrent die je al draait. Je vult daarna het adres en de login in bij Instellingen.
onboarding-ready-library = Bibliotheek
onboarding-ready-library-none = Nog geen map. Die staat in Instellingen, Bibliotheek wanneer je wilt.
onboarding-ready-library-series = { $count ->
    [one] { $count } map
   *[other] { $count } mappen
}, { $series } series tot nu toe
onboarding-ready-library-empty = { $count ->
    [one] { $count } map
   *[other] { $count } mappen
}, voorlopig leeg
onboarding-ready-library-scanning = { $count ->
    [one] { $count } map
   *[other] { $count } mappen
}, scannen
onboarding-ready-lists = Je lijst
onboarding-ready-lists-as = { $tracker }, als { $user }
onboarding-ready-lists-none = Nog niet gekoppeld. Dat staat in Instellingen, Accounts wanneer je wilt.
onboarding-ready-downloads = Downloads
onboarding-ready-downloads-line = { $sources ->
    [0] Nog geen bronnen
    [one] { $sources } bron
   *[other] { $sources } bronnen
}, { $client ->
    [own] je eigen qBittorrent.
   *[bundled] de eigen qBittorrent van AniBeam.
} { $next ->
    [ready] Alles klaar.
    [opens] Instellingen, Downloads opent hierna.
   *[both] Beide staan in Instellingen, Downloads.
}
onboarding-ready-open-downloads = Instellingen, Downloads openen
onboarding-ready-set-up = Nu instellen

## Rail.qml

rail-home = Start
rail-library = Bibliotheek
rail-airing = Uitzending
rail-feed = Feed
rail-stats = Stats
rail-show-info = Serie-info
rail-downloads = Downloads
rail-settings = Instellingen
rail-downloads-complete =
    { $count ->
        [one] Downloads: { $count } abonnement is compleet
       *[other] Downloads: { $count } abonnementen zijn compleet
    }
rail-title-language-name = Titeltaal: { $language }. Druk voor de volgende
rail-title-language-english = Engels
rail-title-language-tip = Titeltaal: romaji, Engels of Japans
rail-whats-new-in = Wat is nieuw in { $release }
rail-whats-new = Wat is nieuw
rail-settings-note = Instellingen: { $note }

## FixSheet.qml

fix-kind-episode = Aflevering
fix-kind-opening = Opening
fix-kind-ending = Ending
fix-kind-preview = Preview
fix-kind-special = Special
fix-kind-other = Overig
fix-reads-season-episode = Seizoen { $season } aflevering { $number }
fix-reads-extra = { $kind } { $index }
fix-apply-failed = Kon de correcties niet toepassen.
fix-title = Corrigeer hoe bestanden worden gelezen
fix-lede = { $title }. Zeg wat elk bestand echt is; elke scan hierna houdt dat aan.
fix-rescan = Map opnieuw scannen
fix-rescan-started = De map wordt opnieuw gelezen
fix-filled-in = Ingevuld
fix-fill-in = Invullen
fix-column-file = Bestand
fix-column-reads-as = Gelezen als
fix-column-should-be = Moet zijn
fix-fixed = Gecorrigeerd
fix-number-placeholder = Nr.
fix-index-placeholder = Geen
fix-keep = Correctie houden
fix-leave-as-read = Laat dit bestand zoals het gelezen wordt
fix-read-by-name = Lees dit bestand weer op naam
fix-report = Issue melden op GitHub
fix-report-body = Opent een ingevuld issue dat jij verstuurt. Het is openbaar en noemt deze bestanden.
fix-report-body-agent = Opent een ingevuld issue dat jij verstuurt. Het is openbaar en noemt deze bestanden. { $agent } kijkt er eerst naar.
fix-close-unapplied =
    { $count ->
        [one] Sluiten zonder { $count } correctie toe te passen?
       *[other] Sluiten zonder { $count } correcties toe te passen?
    }
fix-number-invalid = Een rood getal moet eerst worden gecorrigeerd
fix-close-anyway = Toch sluiten
fix-keep-editing = Verder bewerken
fix-apply-count =
    { $count ->
        [one] { $count } correctie toepassen
       *[other] { $count } correcties toepassen
    }

## MatchPanel.qml

match-matched-to = Gekoppeld aan { $title }
match-applied = Koppeling toegepast
match-found-tvmaze = Serie gevonden op TVmaze
match-found-anilist = Gevonden op AniList
match-found-anilist-season = Gevonden op AniList, seizoen { $season }
match-found-mal = Gevonden op MyAnimeList
match-sites-and = { $first } en { $next }
match-matched-on = Gekoppeld op { $sites }
match-provider-silent = { $provider } antwoordde niet
match-title-walk = Koppelen, { $walk }
match-title = Koppelen
match-skip = Overslaan
match-refresh-info = Info vernieuwen
match-refreshing = { $title } vernieuwen
match-clear-confirm = Wissen: verwijdert de koppeling en het ingestelde aantal afleveringen
match-clear = Koppeling wissen
match-cleared = Koppeling gewist
match-search-placeholder = Zoek op { $provider }
match-link-tvmaze = Plak een TVmaze- of IMDb-link
match-link-anilist = Plak een AniList- of MyAnimeList-link
match-look-up = Opzoeken
match-searching = { $provider } doorzoeken
match-type-more = Typ minstens twee letters om op { $provider } te zoeken.
match-no-results = Geen titels gevonden op { $provider }.

## WatchingPage.qml

watching-episode-code = Afl. { $number }
watching-countdown-days = { $days }d { $hours }u
watching-countdown-hours = { $hours }u { $minutes }m
watching-countdown-minutes = { $minutes }m
watching-on-disk = Op schijf
watching-on-anilist = Op AniList
watching-ready-to-start = Klaar om te beginnen
watching-not-in-library = Niet in de bibliotheek
watching-get-to-start = Haal afleveringen op om te beginnen
watching-anilist-updated = AniList bijgewerkt { $when }
watching-airing = Loopt
watching-next-in = Volgende: { $title } over { $wait }
watching-episodes-waiting = Afleveringen wachten
watching-series-behind =
    { $count ->
        [one] { $count } serie achter
       *[other] { $count } series achter
    }
watching-up-to-date = Bij
watching-anilist-only = Alleen op AniList
watching-planning-summary-off = Wat je van plan bent te kijken op AniList.
watching-planning-summary =
    { $count ->
        [one] { $count } serie gepland, { $here } op schijf.
       *[other] { $count } series gepland, { $here } op schijf.
    }
watching-summary-off = Wat je kijkt of herkijkt op AniList.
watching-summary-waiting =
    { $count ->
        [one] { $count } serie heeft afleveringen die wachten.
       *[other] { $count } series hebben afleveringen die wachten.
    }
watching-caught-up = Helemaal bij.
watching-start-watching = Begin met kijken
watching-play-next = Volgende afspelen
watching-open-on-anilist = Openen op AniList
watching-refreshing = Vernieuwen
watching-connect-title = AniList is niet gekoppeld
watching-connect-body = Koppel het en deze pagina toont de series die je op AniList kijkt of herkijkt, met Afspelen bij de series die je hebt en een aftelling bij de series die lopen.
watching-connect-waiting = Wachten op de browser
watching-connect-anilist = AniList koppelen
watching-load-failed-title = De kijklijst is niet geladen
watching-load-failed-body = AniList was niet bereikbaar. De lijst komt terug bij de volgende vernieuwing.
watching-planning-failed-title = De planlijst is niet geladen
watching-planning-failed-body = De cache kon niet worden gelezen. Probeer het opnieuw, of hij komt terug bij de volgende vernieuwing.
watching-planning-empty-title = Niets op je planlijst
watching-planning-empty-body = Later kijken bij Nu uitgezonden, of Gepland in het lijstmenu van een serie, zet een serie hier.
watching-airing-now = Nu uitgezonden
watching-empty-title = Niets op je kijklijst
watching-empty-body = Zet een serie op Kijken op AniList en hij komt hier.
watching-open-anilist = AniList openen

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] { $count } serie verborgen
       *[other] { $count } series verborgen
    }
library-hidden-sync-stops =
    { $count ->
        [one] { $count } serie verborgen, en de trackersync stopt
       *[other] { $count } series verborgen, en de trackersync stopt
    }
library-unhidden =
    { $count ->
        [one] { $count } serie terug, en synct weer
       *[other] { $count } series terug, en syncen weer
    }
library-undo = Ongedaan maken
library-clearing-score =
    { $count ->
        [one] De score van { $count } serie wissen
       *[other] De score van { $count } series wissen
    }
library-rating =
    { $count ->
        [one] { $count } serie een { $score } geven
       *[other] { $count } series een { $score } geven
    }
library-moving =
    { $count ->
        [one] { $count } serie naar { $list } zetten
       *[other] { $count } series naar { $list } zetten
    }
library-rescanning =
    { $count ->
        [one] { $count } serie opnieuw scannen
       *[other] { $count } series opnieuw scannen
    }
library-finding-matches =
    { $count ->
        [one] Een koppeling zoeken voor { $count } serie
       *[other] Een koppeling zoeken voor { $count } series
    }
library-sort-last-viewed = Recent gekeken
library-sort-downloaded = Recent gedownload
library-sort-title = Titel
library-sort-progress = Voortgang
library-sort-average-score = Gemiddelde score
library-sort-my-score = Mijn score
library-dir-a-to-z = A tot Z
library-dir-z-to-a = Z tot A
library-dir-oldest = Oudste eerst
library-dir-newest = Nieuwste eerst
library-dir-least-watched = Minst gekeken eerst
library-dir-most-watched = Meest gekeken eerst
library-dir-lowest = Laagste eerst
library-dir-highest = Hoogste eerst
library-group-none = Niets
library-group-season = Seizoen
library-group-type = Type
library-group-genre = Genre
library-group-watch-state = Kijkstatus
library-scan-started = Scan gestart
library-tab-all = Alles
library-tab-series = Series
library-tab-movies = Films
library-tab-watched = Gekeken
library-tab-hidden = Verborgen
library-count-movies =
    { $count ->
        [one] { $count } film
       *[other] { $count } films
    }
library-count-watched = { $count } gekeken
library-count-hidden = { $count } verborgen
library-count-series =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    }
library-title = Bibliotheek
library-all-genres = Alle genres
library-search-placeholder = Zoek titels of mappen
library-sort = Sorteren
library-group = Groeperen
library-franchises-together = Franchises samen
library-watched-shows = Gekeken series
library-load-failed-title = De bibliotheek is niet geladen
library-no-matches = Geen resultaten voor "{ $query }"
library-none-in-genre =
    { $what ->
        [series] Geen series in { $genre } hier
        [films] Geen films in { $genre } hier
        [watched] Geen gekeken series in { $genre } hier
        [hidden] Geen verborgen series in { $genre } hier
       *[shows] Geen series in { $genre } hier
    }
library-none-here =
    { $what ->
        [series] Geen series hier
        [films] Geen films hier
        [watched] Geen gekeken series hier
        [hidden] Geen verborgen series hier
       *[shows] Geen series hier
    }
library-load-failed-body = AniBeam kon de bibliotheek nu niet lezen. Probeer het zo opnieuw.
library-no-matches-body = Controleer de spelling, of doorzoek de hele bibliotheek vanuit Alles.
library-none-in-genre-body = Niets in deze weergave heeft dat genre. Alle genres toont de rest.
library-none-in-tab-body = Je bibliotheek heeft hier nog niets van. Alles toont alles wat erin staat.
library-all-filtered-body = Alles in je bibliotheek is uit deze weergave gefilterd.
library-clear-search = Zoekopdracht wissen
library-show-all = Alles tonen
library-picked = { $count } gekozen
library-pick-all = Alles
library-pick-all-tip = Kies elke getoonde kaart (Ctrl+A)
library-unhide = Tonen
library-hide = Verbergen
library-rate = Score
library-rate-tip = Eén score voor elke keuze, op elke gekoppelde tracker
library-needs-tracker = Vraagt om een gekoppelde tracker en een gekoppelde serie
library-move-to-list = Naar lijst
library-rescan = Opnieuw scannen
library-find-match = Koppeling zoeken
library-find-match-tip = Zoek voor elke keuze zonder koppeling
library-all-matched = Elke keuze is gekoppeld
library-empty-scanned-title = Nog niets gevonden in je mappen
library-empty-title = Je bibliotheek is leeg
library-empty-scanned-body = AniBeam vond geen series of films in de mappen die het bewaakt. Scan ze opnieuw, of controleer de mappen in Instellingen.
library-empty-body = Voeg de map toe waar je anime staat en de bibliotheek vult zichzelf, of importeer een AniBeam-export.
library-scan-all = Alles scannen
library-add-folder = Map toevoegen
library-settings = Bibliotheekinstellingen
library-import = Importeren
library-import-title = Een AniBeam-export importeren
library-import-filter-export = AniBeam-export (*.json)
library-import-filter-all = Alle bestanden (*)
library-import-started = Import gestart

## MetadataPage.qml

metadata-title = Serie-info
metadata-filter-all = Alles { $count }
metadata-filter-series = Series { $count }
metadata-filter-films = Films { $count }
metadata-filter-missing = Ontbreekt { $count }
metadata-scan-started = Scan gestart
metadata-sort-added = Toegevoegd
metadata-sort-season = Seizoen
metadata-sort-a-to-z = A tot Z
metadata-section-unmatched = Niet herkend
metadata-section-missing = Map niet gevonden
metadata-section-incomplete = Onvolledig
metadata-section-empty = Niet gedownload
metadata-section-airing = Loopt
metadata-section-complete = Compleet
metadata-no-info = Nog geen serie-info
metadata-state-aired = { $have } van { $aired } uit
metadata-state-of = { $have } van { $expected }
metadata-state-of-extra = { $have } van { $expected } +{ $extra }
metadata-detail-none-aired = Geen van de { $aired } uitgezonden afleveringen staat op schijf
metadata-detail-aired = { $have } van { $aired } uitgezonden op schijf
metadata-detail-aired-missing = { $have } van { $aired } uitgezonden op schijf, { $missing } ontbreken
metadata-detail-none = Geen van de { $expected } afleveringen staat op schijf
metadata-detail-missing = { $have } van { $expected } op schijf, { $missing } ontbreken
metadata-walk-position = { $at } van { $total }
metadata-walk-done = Dat was de laatste
metadata-walk-left =
    { $count ->
        [one] Nog één onder Moet gekoppeld
       *[other] Nog { $count } onder Moet gekoppeld
    }
metadata-countdown-days = { $days }d { $hours }u
metadata-countdown-hours = { $hours }u { $minutes }m
metadata-countdown-minutes = { $minutes }m
metadata-next-in = Volgende over { $wait }
metadata-files-of = { $have } van { $expected }
metadata-files-on-disk = Bestanden op schijf
metadata-forgot = { $title } vergeten
metadata-refreshed-failed = { $refreshed } vernieuwd, { $failed } konden niet worden vernieuwd
metadata-refreshed =
    { $count ->
        [one] { $count } serie vernieuwd
       *[other] { $count } series vernieuwd
    }
metadata-found =
    { $count ->
        [one] { $count } serie gevonden
       *[other] { $count } series gevonden
    }
metadata-found-some-left =
    { $count ->
        [one] { $count } serie gevonden, { $unmatched } nog niet herkend
       *[other] { $count } series gevonden, { $unmatched } nog niet herkend
    }
metadata-found-nothing = Niets nieuws gevonden, { $unmatched } nog niet herkend
metadata-all-recognised = Elke serie is herkend
metadata-count =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    }
metadata-search-placeholder = Titels filteren
metadata-find-rest = Zoek de rest
metadata-find-rest-started = De mappen opzoeken die AniBeam nog niet kent
metadata-find-rest-tip = Zoekt elke map op die AniBeam nog niet kent, op elke site voor serie-info in de volgorde die Instellingen aangeeft
metadata-refresh-all = Alles vernieuwen
metadata-refresh-all-started = De info van elke serie bijwerken
metadata-refresh-all-tip = Haalt de info van elke herkende serie opnieuw op
metadata-crawling = Franchiselinks zoeken
metadata-crawl-progress = { $done } van { $total }
metadata-load-failed-title = Serie-info is niet geladen
metadata-empty-scanned-title = Nog niets gevonden in je mappen
metadata-no-matches = Geen resultaten voor "{ $query }"
metadata-nothing-under =
    { $filter ->
        [Series] Niets onder Series
        [Movies] Niets onder Films
        [MissingFiles] Niets onder Ontbreekt
       *[other] Niets onder { $filter }
    }
metadata-load-failed-body = AniBeam kon de bibliotheek nu niet lezen. Probeer het zo opnieuw.
metadata-empty-scanned-body = AniBeam vond geen series in de mappen die het bewaakt. Scan ze opnieuw, of controleer de mappen in Instellingen.
metadata-empty-body = Je bibliotheek is leeg. Voeg de map toe waar je anime staat en AniBeam scant hem.
metadata-no-matches-body = Geen titel in deze weergave bevat die woorden.
metadata-nothing-under-body = Geen van je series past nu bij dit filter. Alles toont de rest.
metadata-scan-all = Alles scannen
metadata-add-folder = Map toevoegen
metadata-clear-search = Zoekopdracht wissen
metadata-show-all = Alles tonen
metadata-refreshing = { $title } vernieuwen
metadata-refresh-tip = Haal de info van deze serie opnieuw op
metadata-forget-armed = Weghalen
metadata-forget-confirm-tip = Klik nogmaals om deze serie en de kijkgeschiedenis uit AniBeam te halen
metadata-forget-tip = De map staat niet meer op de schijf: haal deze serie en de geschiedenis uit AniBeam

## AiringPage.qml

airing-title = Nu uitgezonden
airing-sort-popular = Populariteit
airing-sort-score = Score
airing-sort-trending = Trending
airing-sort-favourites = Favorieten
airing-sort-soonest = Volgende aflevering
airing-sort-aired = Uitgezonden afleveringen
airing-sort-episodes = Aantal afleveringen
airing-sort-start = Startdatum
airing-sort-duration = Afleveringsduur
airing-sort-title = Titel
airing-dir-most-popular = Populairste eerst
airing-dir-least-popular = Minst populair eerst
airing-dir-lowest = Laagste eerst
airing-dir-highest = Hoogste eerst
airing-dir-least-trending = Minst trending eerst
airing-dir-most-trending = Meest trending eerst
airing-dir-fewest = Minste eerst
airing-dir-most = Meeste eerst
airing-dir-soonest = Eerstvolgende eerst
airing-dir-latest = Laatste eerst
airing-dir-oldest = Oudste eerst
airing-dir-newest = Nieuwste eerst
airing-dir-shortest = Kortste eerst
airing-dir-longest = Langste eerst
airing-dir-a-to-z = A tot Z
airing-dir-z-to-a = Z tot A
airing-season =
    { $season ->
        [Winter] Winter { $year }
        [Spring] Lente { $year }
        [Summer] Zomer { $year }
        [Fall] Herfst { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = { $minutes } min geleden
airing-ago-hours = { $hours } u geleden
airing-ago-days = { $days } d geleden
airing-episode-out = { $episode } uit { $ago }
airing-no-date = Nog geen datum
airing-in-library = In bibliotheek
airing-out-today =
    { $count ->
        [one] { $count } aflevering vandaag uit
       *[other] { $count } afleveringen vandaag uit
    }
airing-dismissed-one = { $title } verschijnt hier niet meer
airing-dismissed =
    { $count ->
        [one] { $count } serie verschijnt hier niet meer
       *[other] { $count } series verschijnen hier niet meer
    }
airing-planned-already-one = { $title } staat al op je lijst
airing-planned-already-all = Ze staan al allemaal op je lijst
airing-planned =
    { $count ->
        [one] { $count } serie komt op je Gepland-lijst
       *[other] { $count } series komen op je Gepland-lijst
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } serie komt op je Gepland-lijst, { $already } stonden er al op
       *[other] { $count } series komen op je Gepland-lijst, { $already } stonden er al op
    }
airing-play-next = Volgende afspelen
airing-plan-to-watch = Later kijken
airing-get-more-episodes = Meer afleveringen ophalen
airing-get-episodes = Afleveringen ophalen
airing-open-on-anilist = Openen op AniList
airing-not-interested = Niet interessant
airing-not-interested-count = { $count } niet interessant
airing-not-interested-tip = Series die je als niet interessant hebt gemarkeerd, in Instellingen
airing-refreshing = Vernieuwen
airing-pick-season = Kies een ander seizoen
airing-filter-all = Alles
airing-filter-lists = Op mijn lijsten
airing-filter-library = In mijn bibliotheek
airing-hide-lists = Mijn lijsten verbergen
airing-hide-lists-tip = Laat de series op je AniList- of MAL-lijsten weg
airing-hide-library = Mijn bibliotheek verbergen
airing-hide-library-tip = Laat de series weg waarvan je afleveringen hebt
airing-out-now = Nu uit
airing-out-last-day =
    { $count ->
        [one] { $count } aflevering in de laatste 24 uur
       *[other] { $count } afleveringen in de laatste 24 uur
    }
airing-out-fewer = Minder
airing-out-all = Alle { $count }
airing-sort = Sorteren
airing-load-failed-title = AniList was niet bereikbaar
airing-load-failed-body = De series die lopen komen van AniList. Ze komen terug bij de volgende vernieuwing.
airing-reading-title = Lezen wat er loopt
airing-nothing-now = Er loopt nu niets
airing-nothing-for = Niets op tv in { $season }
airing-nothing-read = Nog niets gelezen
airing-reading-body = Elke serie die loopt, van AniList. De eerste keer duurt dit een paar seconden.
airing-nothing-body = AniList heeft nog geen series voor dit seizoen. Vernieuwen leest het opnieuw, of kies een ander seizoen.
airing-nothing-read-body = Vernieuwen leest elke lopende serie van AniList.
airing-hides-empty-title = Door je filters blijft hier niets over
airing-lists-empty-title = Niets hier staat op je lijsten
airing-library-empty-title = Niets hier staat in je bibliotheek
airing-hides-empty-body = Elke serie hier staat op je lijsten of in je bibliotheek, en die heb je verborgen.
airing-lists-empty-body = Later kijken op een kaart zet hem op je lijst.
airing-library-empty-body = Afleveringen ophalen op een kaart downloadt hem in je bibliotheek.
airing-show-everything = Alles tonen
airing-picked = { $count } gekozen
airing-undo = Ongedaan maken

## EpisodeRow.qml

episode-row-date-time-format = ddd d MMM, HH:mm
episode-row-date-upcoming-format = ddd d MMM
episode-row-date-format = d MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Klik om vanaf hier als ongekeken te markeren
episode-row-mark-watched-tip = { $state }. Klik om tot hier als gekeken te markeren
episode-row-next-up = Hierna
episode-row-versions =
    { $count ->
        [one] { $count } versie
       *[other] { $count } versies
    }
episode-row-versions-tip =
    { $count ->
        [one] Dezelfde aflevering uit { $count } release. Klik om te kiezen welke speelt
       *[other] Dezelfde aflevering uit { $count } releases. Klik om te kiezen welke speelt
    }
episode-row-downloading = Downloaden { $percent }%
episode-row-get = Aflevering ophalen
episode-row-predicted = Voorspeld
episode-row-extra = Extra
episode-row-resume = Hervat bij { $time }

## EpisodeCubes.qml

episode-cubes-one = Eén aflevering
episode-cubes-range =
    { $count ->
        [one] { $count } aflevering, { $first } t/m { $last }
       *[other] { $count } afleveringen, { $first } t/m { $last }
    }
episode-cubes-tip = Aflevering { $number }: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } extra
       *[other] { $count } extra's
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } ongekoppeld bestand
       *[other] { $count } ongekoppelde bestanden
    }
episodes-pane-on-disk = Op schijf
episodes-pane-not-on-disk = Niet op schijf
episodes-pane-season = S{ $season }
episodes-pane-get-tip = Aflevering { $number } ophalen
episodes-pane-none-on-disk = Geen afleveringen op schijf
episodes-pane-change-match = Koppeling wijzigen
episodes-pane-refreshing = { $title } vernieuwen

## ShowDetail.qml

show-detail-not-in-library = Deze map staat nog niet in de bibliotheek. Zodra een scan hem vindt, verschijnen hier de poster en afleveringen.
show-detail-film-on-disk = De film staat op schijf.
show-detail-film-not-on-disk = De film staat nog niet op schijf.
show-detail-complete-line = Uitzending afgelopen, en alle { $total } afleveringen zijn hier. Er komt niets meer bij, dus het abonnement heeft niets meer te doen.
show-detail-finished = Uitzending afgelopen.
show-detail-airing = Loopt.
show-detail-have-of-total = { $have } van { $total } afleveringen hier.
show-detail-have =
    { $count ->
        [one] { $count } aflevering hier.
       *[other] { $count } afleveringen hier.
    }
show-detail-next-predicted =
    { $count ->
        [one] { $span } wordt verwacht over { $wait }.
       *[other] { $span } worden verwacht over { $wait }.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span } komt uit over { $wait }.
       *[other] { $span } komen uit over { $wait }.
    }
show-detail-missing =
    { $count ->
        [one] Aflevering { $list } is er niet.
       *[other] Afleveringen { $list } zijn er niet.
    }
show-detail-missing-more = Afleveringen { $list } en nog { $more } zijn er niet.
show-detail-downloaded = Gedownload
show-detail-downloading = Downloaden { $percent }%
show-detail-taken-adding = Genomen door de regel, toevoegen
show-detail-taken = Genomen door de regel
show-detail-left-out = Weggelaten door de regel
show-detail-subscribed-reading = Geabonneerd. qBittorrent leest de feed.
show-detail-taken-moving = De regel neemt { $taken } van de { $releases ->
        [one] { $releases } release
       *[other] { $releases } releases
    } in de feed; { $moving ->
        [one] { $moving } wordt
       *[other] { $moving } worden
    } hieronder gedownload.
show-detail-taken-held = De regel neemt { $taken } van de { $releases ->
        [one] { $releases } release
       *[other] { $releases } releases
    } in de feed; { $torrents ->
        [one] { $torrents } torrent staat
       *[other] { $torrents } torrents staan
    } in de client.
show-detail-taken-adding-line = De regel neemt { $taken } van de { $releases ->
        [one] { $releases } release
       *[other] { $releases } releases
    } in de feed; qBittorrent voegt { $taken ->
        [one] hem toe.
       *[other] ze toe.
    }
show-detail-feed-holds-nothing = De feed bevat nog niets. Releases die passen worden gedownload zodra ze erin verschijnen.
show-detail-nothing-fits = Nog niets in de feed past bij de regel. Releases die passen worden gedownload zodra ze verschijnen.
show-detail-complete = Compleet
show-detail-open-in-library = Openen in bibliotheek
show-detail-feed-finds-nothing = Deze feed vindt niets
show-detail-feed-empty-query = Hij vraagt { $source } om “{ $query }”, en dat geeft niets, dus niets nieuws bereikt de regel. Een kortere zoekopdracht geeft meestal wel antwoord; de woorden die wegvallen kunnen in de regel blijven.
show-detail-feed-empty = Hij vraagt { $source }, en dat geeft niets, dus niets nieuws bereikt de regel. Een kortere zoekopdracht geeft meestal wel antwoord; de woorden die wegvallen kunnen in de regel blijven.
show-detail-feed-empty-earlier = qBittorrent toont nog wat een eerdere versie van deze feed vond.
show-detail-repairing = Herstellen
show-detail-repair = Herstel
show-detail-unsubscribe = Opzeggen
show-detail-remove-torrents =
    { $count ->
        [one] De torrent weghalen
       *[other] { $count } torrents weghalen
    }
show-detail-searches = Zoekt
show-detail-quoted = “{ $words }”
show-detail-whole-feed = de hele feed
show-detail-from = Van
show-detail-rule = Regel
show-detail-rule-unfollowable = Kan geen nieuwe afleveringen ophalen: de bron beantwoordt zoekopdrachten maar heeft geen feed
show-detail-rule-active = Actief
show-detail-keeps = Houdt
show-detail-keeps-value = releases met “{ $words }”
show-detail-drops = Laat weg
show-detail-drops-value = releases met { $words }
show-detail-or = { $first } of { $second }
show-detail-last-took = Laatst genomen
show-detail-saves-to = Opslaan in
show-detail-folder = Map
show-detail-in-the-feed = In de feed
show-detail-feed-reading = De feed lezen.
show-detail-feed-offers = Wat de feed nu aanbiedt, nieuwste eerst.
show-detail-feed-is-empty = De feed is leeg.
show-detail-read-again = Opnieuw lezen
show-detail-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
show-detail-download = Download
show-detail-torrents = Torrents
show-detail-torrents-moving =
    { $count ->
        [one] { $count } wordt nu gedownload; de balken vullen zich gaandeweg.
       *[other] { $count } worden nu gedownload; de balken vullen zich gaandeweg.
    }
show-detail-torrents-held = Wat de client voor deze map heeft.
show-detail-torrents-waiting = Wachten tot qBittorrent toevoegt wat de regel neemt.
show-detail-torrents-none = De client heeft niets voor deze map.

## SeriesPage.qml

series-trackers-both = { $first } en { $second }
series-count-cleared = Aantal afleveringen gewist
series-count-set = Ingesteld op { $count } afleveringen
series-held-one-undated = Aflevering { $first } heeft nog geen datum
series-held-range-undated = Afleveringen { $first } t/m { $last } hebben nog geen datum
series-held-one-due = Aflevering { $first } verwacht in { $when }
series-held-range-due = Afleveringen { $first } t/m { $last } verwacht in { $when }
series-held-month-format = MMMM yyyy
series-fix-schedule = Afleveringen en data
series-fix-files = Bestanden
series-fix-rescan-folder = Map opnieuw scannen
series-fix-match = Koppelen
series-find-match = Koppeling zoeken
series-rescan-show = Serie opnieuw scannen
series-open-in-show-info = Openen in Serie-info
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } film naar de prullenbak verplaatst
           *[other] { $count } films naar de prullenbak verplaatst
        }
        [episode] { $count ->
            [one] { $count } aflevering naar de prullenbak verplaatst
           *[other] { $count } afleveringen naar de prullenbak verplaatst
        }
       *[file] { $count ->
            [one] { $count } bestand naar de prullenbak verplaatst
           *[other] { $count } bestanden naar de prullenbak verplaatst
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } film gekozen.
           *[other] { $count } films gekozen.
        }
        [episode] { $count ->
            [one] { $count } aflevering gekozen.
           *[other] { $count } afleveringen gekozen.
        }
       *[file] { $count ->
            [one] { $count } bestand gekozen.
           *[other] { $count } bestanden gekozen.
        }
    }
series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } film naar de prullenbak
           *[other] { $count } films naar de prullenbak
        }
        [episode] { $count ->
            [one] { $count } aflevering naar de prullenbak
           *[other] { $count } afleveringen naar de prullenbak
        }
       *[file] { $count ->
            [one] { $count } bestand naar de prullenbak
           *[other] { $count } bestanden naar de prullenbak
        }
    }
series-delete-click = Klik op de bestanden om te wissen.
series-delete-where = Ze gaan naar de prullenbak, waar je ze terug kunt halen. Wat je keek blijft bijgehouden.
series-delete-move = Naar de prullenbak
series-pick-none = Niets kiezen
series-pick-all = Alles kiezen
series-rescan-started = Opnieuw scannen gestart
series-rated = Score { $score }
series-rated-cleared = Score gewist
series-score-failed = Score mislukt
series-still-downloading = { $episode } wordt nog gedownload
series-format-series = Serie
series-tracker-refused = { $tracker } nam het niet aan
series-tracker-refused-why = { $tracker } nam het niet aan: { $why }
series-watched-up-to = Gekeken tot { $episode }
series-unwatched-from = Ongekeken vanaf { $episode }
series-undo = Ongedaan maken
series-play-downloading = Aflevering { $number } downloaden
series-play-downloading-unnumbered = Aflevering downloaden
series-play-continue = Verder kijken
series-play-again = Opnieuw kijken
series-play-continue-episode = Verder met aflevering { $number }
series-play-again-from = Opnieuw kijken vanaf aflevering { $number }
series-play-episode = Speel aflevering { $number }
series-page-not-tracked = Niet bijgehouden
series-not-tracked-on-disk =
    { $count ->
        [one] Niet bijgehouden, { $count } aflevering op schijf
       *[other] Niet bijgehouden, { $count } afleveringen op schijf
    }
series-watched = { $count } gekeken
series-watched-of = { $watched } van { $total } gekeken
series-watched-of-estimate = { $watched } van { $total }+ gekeken
series-facts-episodes-estimate =
    { $count ->
        [one] { $count } aflevering+
       *[other] { $count } afleveringen+
    }
series-facts-each = { $runtime } per stuk
series-facts-on-disk = { $count } op schijf
series-next = Volgende: { $episode }
series-episodes-downloading =
    { $count ->
        [one] { $count } aflevering wordt gedownload
       *[other] { $count } afleveringen worden gedownload
    }
series-all-watched = Alles gekeken
series-no-poster = Geen poster
series-next-season-span = S{ $season } { $span }
series-next-predicted = { $span } verwacht op { $date }, over { $wait }
series-next-date-format = ddd d MMM
series-next-airs =
    { $count ->
        [one] { $span } komt uit over { $wait }
       *[other] { $span } komen uit over { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] { $count } aflevering downloaden
       *[other] { $count } afleveringen downloaden
    }
series-count-season-ends = Seizoen { $season } eindigt bij aflevering { $number }
series-count-ends = Eindigt bij aflevering { $number }
series-count-gave-way = Je stelde { $count } afleveringen in; dat geldt niet meer
series-count-set-by-you = { $line }, door jou ingesteld
series-change = Wijzigen
series-get-episodes = Afleveringen ophalen
series-your-score = Jouw score { $score }
series-rate = Score
series-page-average-score = Gemiddelde score { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = { $votes } stemmen
series-add-to-list = Op een lijst zetten
series-page-rewatched =
    { $count ->
        [one] { $count } keer herkeken
       *[other] { $count } keer herkeken
    }
series-your-notes = Je notities
series-add-notes = Notitie maken
series-notes-tip = Notities bij je lijstitem, bewaard op { $sites }
series-unhide = Tonen
series-hide = Verbergen
series-hidden-toast = { $title } is hier verborgen, en AniBeam synct hem niet meer met AniList en MyAnimeList
series-unhidden-toast = { $title } is terug, en synct weer
series-hide-tip-tracked = Verborgen op elke pagina hier, en AniBeam synct hem niet meer met AniList en MyAnimeList
series-hide-tip = Verborgen in elke lijst
series-fixes = Correcties
series-fixes-tip = Corrigeer bestanden, het aantal afleveringen, de koppeling, of scan opnieuw
series-quick-fix = Snel corrigeren
series-film = Film
series-episodes = Afleveringen
series-extras = Openings, endings en meer
series-extras-openings = Openings
series-extras-endings = Endings
series-extras-previews = Previews en trailers
series-extras-specials = Specials
series-extras-other = Overig
series-extras-group = { $kind }, { $count }
series-extra-files = Extra bestanden
series-extra-files-warning =
    { $count ->
        [one] Eén bestand gaat boven de { $total } verwachte afleveringen uit: verkeerd benoemd, dubbel of een special. Hernoem het of haal het weg.
       *[other] { $count } bestanden gaan boven de { $total } verwachte afleveringen uit: verkeerd benoemd, dubbel of specials. Hernoem ze of haal ze weg.
    }
series-synopsis = Samenvatting
series-less = Minder
series-more = Meer
series-match = Koppelen
series-not-matched = Nog niet gekoppeld, dus geen poster, samenvatting of bijhouden. Zoek de serie-info op AniList of TVmaze.
series-naming-title = Namen die passen
series-naming-body = De map is de titel van de serie, zoals AniList hem schrijft, met het seizoen erachter als het niet het eerste is. Elk bestand bevat het afleveringsnummer.
series-fix-how-files-read = Corrigeer hoe bestanden lezen
series-characters = Personages
series-related = Verwant
series-related-filling = Wordt nog aangevuld vanuit AniList.
series-tags = Tags
series-hide-spoilers = Spoilers verbergen
series-spoilers = Spoilers
series-all-tags = Alle tags
series-tag = { $name }, { $rank }%
series-recommendations = Aanbevelingen
series-relation-owned = In je bibliotheek
series-relation-unreleased = Nog niet uit
series-relation-on-anilist = Op AniList
series-score-saved = Score opgeslagen
series-score-cleared = Score gewist

## GetShow.qml

get-show-the-client = de torrentclient
get-show-group-batch = Batch van { $group }
get-show-run = { $first } t/m { $last }
get-show-episodes =
    { $count ->
        [one] Aflevering { $list }
       *[other] Afleveringen { $list }
    }
get-show-episodes-inline =
    { $count ->
        [one] aflevering { $list }
       *[other] afleveringen { $list }
    }
get-show-sending-to = Versturen naar { $client }
get-show-go =
    { $count ->
        [one] { $count } aflevering ophalen
       *[other] { $count } afleveringen ophalen
    }
get-show-go-replace =
    { $count ->
        [one] { $count } aflevering ophalen en vervangen
       *[other] { $count } afleveringen ophalen en vervangen
    }
get-show-go-subscribe =
    { $count ->
        [one] { $count } aflevering ophalen en abonneren
       *[other] { $count } afleveringen ophalen en abonneren
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] { $count } aflevering ophalen, vervangen en abonneren
       *[other] { $count } afleveringen ophalen, vervangen en abonneren
    }
get-show-subscribe = Abonneren op nieuwe afleveringen
get-show-nothing-new = Niets nieuws op te halen
get-show-pick-episode = Kies een aflevering
get-show-replace-off = Uit. De keuzes voor { $episodes } worden weggelaten, omdat de bibliotheek ze heeft. Zet dit aan om ze te vervangen.
get-show-replace-on = De bestanden die je hebt voor { $episodes } gaan naar de prullenbak zodra de nieuwe klaar zijn.
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } wordt nu gedownload: die download wordt geannuleerd en wat hij schreef gaat naar de prullenbak.
       *[other] { $episodes } worden nu gedownload: die download wordt geannuleerd en wat hij schreef gaat naar de prullenbak.
    }
get-show-kind-on-demand = Gestart als een zoekopdracht het nodig heeft
get-show-kind-rss-search = RSS-zoeken
get-show-kind-rss-feed = RSS-feed
get-show-kind-json-search = JSON-zoeken
get-show-kind-indexer-search = Indexer-zoeken
get-show-ready = Klaar
get-show-running = Draait
get-show-not-running = Draait niet
get-show-not-running-start = Draait niet. Klik hierboven op de bron om hem te starten.
get-show-switched-off = Uitgezet
get-show-click-to-see = Klik om te zien wat het voor deze serie heeft
get-show-searching = Zoeken
get-show-searching-source = { $name } doorzoeken
get-show-searching-elapsed = { $label }, { $seconds } s
get-show-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } releases
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } release, { $cover } van { $total } afleveringen gedekt
       *[other] { $count } releases, { $cover } van { $total } afleveringen gedekt
    }
get-show-on-your-list = Op je lijst
get-show-plan-to-watch = Later kijken
get-show-on-your-list-tip = Het staat op je lijst
get-show-plan-to-watch-tip = Zet het op je Gepland-lijst
get-show-open-series-tip = De seriepagina: details, afleveringen en afspelen
get-show-next-in = Aflevering { $number } over { $wait }
get-show-airing = Loopt
get-show-finished = Afgelopen
get-show-total-episodes = { $count } afleveringen
get-show-downloading-episodes =
    { $count ->
        [one] { $count } aflevering downloaden
       *[other] { $count } afleveringen downloaden
    }
get-show-downloading-torrents =
    { $count ->
        [one] { $count } torrent downloaden
       *[other] { $count } torrents downloaden
    }
get-show-in-library = In je bibliotheek
get-show-not-in-library = Niet in je bibliotheek
get-show-downloading = Downloaden
get-show-stop-all = Alles stoppen
get-show-start-all = Alles starten
get-show-stop-all-tip = Stop elke download van deze serie; wat ze al hebben blijft
get-show-start-all-tip = Start elke download van deze serie opnieuw
get-show-cancel-all = Alles annuleren
get-show-cancel-all-tip = Annuleer elke download van deze serie en verplaats wat ze schreven naar de prullenbak
get-show-sources = Beschikbare bronnen
get-show-check-again = Opnieuw controleren
get-show-check-again-tip = Vraagt elke bron op aanvraag of hij draait. Start niets.
get-show-search-source = Zoek op { $name }
get-show-release-list = Elke release als lijst
get-show-your-library = Je bibliotheek
get-show-library-on-disk = { $have } van { $total } op schijf
get-show-library-downloading = { $count } downloaden
get-show-library-nothing = Nog niets
get-show-row-batch = { $group }, batch
get-show-batch-counting = afleveringen tellen, één torrent
get-show-batch-unlisted = afleveringen niet vermeld, de hele serie op titel, één torrent
get-show-batch-episodes =
    { $count ->
        [one] { $count } aflevering, één torrent
       *[other] { $count } afleveringen, één torrent
    }
get-show-nothing-new-picked = Niets nieuws gekozen
get-show-nothing-picked = Niets gekozen
get-show-pick-provider = Kies een bron
get-show-plan-line = { $episodes } van { $groups } op { $source }.
get-show-and = { $first } en { $second }
get-show-library-has =
    { $count ->
        [one] De bibliotheek heeft { $episodes }. Zet hieronder Huidige afleveringen vervangen aan om hem te vervangen.
       *[other] De bibliotheek heeft { $episodes }. Zet hieronder Huidige afleveringen vervangen aan om ze te vervangen.
    }
get-show-together = samen
get-show-torrents-unit =
    { $count ->
        [one] torrent
       *[other] torrents
    }
get-show-replace = Huidige afleveringen vervangen
get-show-sequential = Op volgorde downloaden
get-show-sequential-on = Aan: elke nieuwe torrent haalt zijn stukken vanaf het begin, dus de eerste afleveringen zijn eerst klaar en je kunt kijken terwijl de rest komt. In totaal iets trager.
get-show-sequential-off = Uit: de client haalt stukken in de volgorde die de swarm aanbiedt.
get-show-rolling = Houd de volgende paar klaar
get-show-rolling-on = Aan: { $count } afleveringen downloaden nu, en elke aflevering die je afkijkt start de volgende.
get-show-rolling-on-delete = Aan: { $count } afleveringen downloaden nu, en elke aflevering die je afkijkt start de volgende. Gekeken afleveringen gaan van de schijf.
get-show-rolling-off = Uit: elke gekozen aflevering downloadt tegelijk.
get-show-follow-off = Uit: nieuwe afleveringen verschijnen hier als gaten, en er downloadt niets vanzelf.
get-show-follow-no-feed = { $source } heeft geen feed om te volgen, dus nieuwe afleveringen kunnen daar niet vanzelf van downloaden.
get-show-follow-pick-group = Kies eerst een groep: een abonnement volgt één groep en kwaliteit.
get-show-follow-batch-only = { $group } heeft hier alleen de batch, dus er is geen feed met losse afleveringen om op te abonneren.
get-show-follow-on = Aan: een feed voor { $feed } op { $source }. Elke nieuwe aflevering downloadt zodra hij verschijnt.
get-show-confirm =
    { $count ->
        [one] { $episodes } wordt nu gedownload. Vervangen annuleert die download en verplaatst wat hij schreef naar de prullenbak.
       *[other] { $episodes } worden nu gedownload. Vervangen annuleert die download en verplaatst wat hij schreef naar de prullenbak.
    }
get-show-confirm-go = Annuleren en vervangen
get-show-state-error = Fout
get-show-state-files-missing = Bestanden ontbreken
get-show-state-stopped = Gestopt
get-show-state-waiting-peers = Wachten op peers
get-show-state-metadata = Metadata ophalen
get-show-state-queued = In wachtrij
get-show-state-checking = Bestanden controleren
get-show-state-allocating = Ruimte reserveren
get-show-state-moving = Bestanden verplaatsen
get-show-state-waiting = Wachten
get-show-a-batch = Een batch
get-show-no-group = Groep niet in de naam
get-show-likely = waarschijnlijk { $source }
get-show-tracker = tracker { $tracker }
get-show-not-known = Nog niet bekend
get-show-bytes-of = { $done } van { $size }
get-show-speed-down = { $speed }/s omlaag
get-show-time-left = nog { $wait }
get-show-stopping = Stoppen
get-show-starting = Starten
get-show-stop = Stop
get-show-start-tip = Opnieuw beginnen met downloaden
get-show-stop-tip = Downloaden stoppen; wat er al is blijft
get-show-cancelling = Annuleren
get-show-cancel-tip = Annuleer deze download en verplaats wat hij schreef naar de prullenbak
get-show-episode-order = Op afleveringsvolgorde downloaden
get-show-episode-order-on = Aan: stukken komen vanaf het begin van elk bestand, dus de eerste afleveringen zijn eerst klaar.
get-show-cube-skipped = Aflevering { $number }: overgeslagen
get-show-cube-done = Aflevering { $number }: klaar
get-show-cube-percent = Aflevering { $number }: { $percent }%
get-show-fact-torrent = Torrent
get-show-fact-release = Release
get-show-fact-from = Van
get-show-fact-saving-to = Opslaan in
get-show-span-counting = Tellen
get-show-span-unknown = { $runs }?
get-show-span-episodes = Afleveringen { $runs }
get-show-cube-no-release = Aflevering { $number }: geen release van { $group }
get-show-seeders = { $count } seeders
get-show-cube-on-disk = al op schijf, klik om te vervangen
get-show-cube-downloading = wordt nu gedownload, klik om te vervangen

## TorrentRow

torrent-row-state-downloading = Downloaden
torrent-row-state-stalled = Vastgelopen
torrent-row-state-metadata = Metadata ophalen
torrent-row-state-queued = In wachtrij
torrent-row-state-stopped = Gestopt
torrent-row-state-done = Klaar
torrent-row-state-seeding = Seeden
torrent-row-state-checking = Controleren
torrent-row-state-moving = Verplaatsen
torrent-row-state-error = Fout
torrent-row-state-files-missing = Bestanden ontbreken
torrent-row-state-making-room = Ruimte maken
torrent-row-state-finding-peers = Peers zoeken
torrent-row-state-waiting = Wachten
torrent-row-done-of-size = { $done } van { $size }
torrent-row-speed-down = { $speed }/s omlaag
torrent-row-speed-up = { $speed }/s omhoog
torrent-row-time-left = nog { $time }
torrent-row-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
torrent-row-rolling = doorlopend, afleveringen { $next } t/m { $through } van { $total }
torrent-row-stop = Stop

## SourceRow

source-row-unavailable-fix = Sluit de schijf aan en scan opnieuw. Is de map verplaatst, haal hem dan weg en voeg de nieuwe plek toe.
source-row-counts = { $count } series
source-row-counts-movies =
    { $movies ->
        [one] { $count } series, { $movies } filmmap
       *[other] { $count } series, { $movies } filmmappen
    }
source-row-unavailable = Niet beschikbaar
    .tip = AniBeam kan deze map niet bereiken. Staat hij op een schijf die niet is aangesloten of gekoppeld, sluit die dan aan en AniBeam pakt hem weer op. Is hij verplaatst, haal hem hier weg en voeg de nieuwe plek toe.
source-row-rescan = Opnieuw scannen
source-row-remove-question = { $name }, { $count } series en hun geschiedenis weghalen?

## StartSourceSheet

start-source-title = { $service } starten voor { $name }?
start-source-busy = { $service } starten. { $name } gaat aan zodra het antwoordt; dit kan tot een minuut duren.
start-source-body = { $service } is de dienst die { $name } doorzoekt voor AniBeam. Starten draait het op de achtergrond op deze computer, en het blijft draaien nadat AniBeam sluit.
start-source-starting = Starten

## SearchProgress

search-progress-why-helper = AniBeam start zijn zoekhulp; dit duurt een seconde of twee.
search-progress-why-starting = { $endpoint } loopt via een hulp die start als een zoekopdracht hem nodig heeft. De eerste zoekopdracht wacht erop, tot { $seconds } seconden; de volgende slaan dit over.
search-progress-why-reading = De releases in groepen verdelen en vergelijken met de bibliotheek.
search-progress-why-ondemand = Een hulp leest { $endpoint } opnieuw voor elke nieuwe zoekopdracht, wat meestal tien tot twintig seconden duurt.
search-progress-why-slow = { $endpoint } antwoordt nu traag.
search-progress-why-usual = Dit duurt meestal een seconde of twee.
search-progress-why-and-slow = { $why } Dit duurt langer dan normaal; annuleer om een andere bron te proberen.
search-progress-searching = Zoeken
search-progress-seconds = { $seconds } s
search-progress-try = Probeer { $endpoint }

## ReleaseTable

release-table-col-release = Release
release-table-col-group = Groep
release-table-col-res = Res
release-table-col-rip = Rip
release-table-col-ep = Afl
release-table-col-size = Grootte
release-table-col-seeders = Seeders
release-table-col-leechers = Leechers
release-table-col-downloads = Downloads
release-table-col-released = Uitgebracht
release-table-tick-all = Elke release aanvinken
release-table-untick-all = Elke release uitvinken
release-table-on-disk = Op schijf
release-table-other-on-disk = Andere release op schijf
release-table-season-batch = Batch seizoen { $season }
release-table-batch = Batch
release-table-via = via { $source }
release-table-season-episode = S{ $season } { $episode }
release-table-date-format = d MMMM yyyy, HH:mm
release-table-download = Download
release-table-more = en nog { $count }. Verfijn of sorteer de lijst om andere eerst te zien.
release-table-pick = Kies deze release

## DownloadsPage

downloads-title = Downloads
downloads-builtin-client = Ingebouwde client
downloads-start-done = { $name } draait en staat aan
downloads-disk-film = Film
downloads-disk-watched = { $size } gekeken
downloads-disk-all-watched = alles gekeken
downloads-delete-watched-menu =
    { $count ->
        [one] Elke gekeken aflevering: { $count } aflevering, { $size }
       *[other] Elke gekeken aflevering: { $count } afleveringen, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Elke afgelopen serie die je keek: { $count } serie, { $size }
       *[other] Elke afgelopen serie die je keek: { $count } series, { $size }
    }
downloads-titles-and = { $list } en { $last }
downloads-titles-more = { $list } en nog { $count }
downloads-delete-finished-none = Geen afgelopen serie is helemaal gekeken
downloads-delete-watched-none = Niets gekeken staat op schijf
downloads-delete-finished-heading =
    { $count ->
        [one] { $count } afgelopen serie wissen, { $size }?
       *[other] { $count } afgelopen series wissen, { $size }?
    }
downloads-delete-finished-base = { $titles }. Alleen series die volgens de bron zijn afgelopen, met elke aflevering gekeken; een serie die nog loopt, of waarvan het einde een gok is, blijft. Elk bestand ervan gaat naar de prullenbak, extra's ook. Hun torrents verdwijnen ook uit { $client }, zodat ze opnieuw gedownload kunnen worden.
downloads-delete-watched-heading =
    { $count ->
        [one] { $count } gekeken aflevering wissen, { $size }?
       *[other] { $count } gekeken afleveringen wissen, { $size }?
    }
downloads-delete-watched-base = Elke aflevering die je hebt gekeken, in elke serie. Afleveringen die je nog niet keek en extra's blijven. De bestanden en hun ondertitels gaan naar de prullenbak, dus je kunt ze daar terughalen. Hun torrents verdwijnen ook uit { $client }, zodat ze opnieuw gedownload kunnen worden.
downloads-move-to-trash = Naar prullenbak
downloads-remove-group-torrents =
    { $count ->
        [one] De torrent weghalen
       *[other] De { $count } torrents weghalen
    }
downloads-run-span = { $from } t/m { $to }
downloads-run-pair = { $first } en { $second }
downloads-gap-missing =
    { $count ->
        [one] Aflevering { $runs } ontbreekt
       *[other] Afleveringen { $runs } ontbreken
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Aflevering { $runs } niet gedownload
       *[other] Afl. { $runs } niet gedownload
    }
downloads-gap-then-not-downloaded = { $runs } niet gedownload
downloads-trying-query = { $query } proberen
downloads-client-settings = Instellingen torrentclient
downloads-error-unknown = Er ging iets mis. Probeer het opnieuw.
downloads-search-all-off = Elke bron staat uit. Zet er een aan om te zoeken.
downloads-turn-on = { $name } aanzetten
downloads-search-no-sources = Er zijn nog geen bronnen om te doorzoeken. Voeg een bronlijst toe in Instellingen, onder Downloads.
downloads-search-none-started = Geen bron kon de zoekopdracht starten. Probeer het zo opnieuw.
downloads-auto-resolution = Verfijnd tot { $resolution }, de scherpste die er is
downloads-search-one-failed = Zoeken op { $name } werkte niet. Probeer het opnieuw, of doorzoek een andere bron.
downloads-search-all-failed = Geen van de bronnen kon zoeken. Probeer het zo opnieuw.
downloads-narrowing-on = Verfijnen op { $name }
downloads-refine-zero = Dan blijft er niets over, dus de lijst blijft zoals hij is.
downloads-refine-no-match = Geen release past bij die term.
downloads-refine-empty = Typ een term om op te verfijnen.
downloads-refine-no-change = Dat verandert niets.
downloads-movie-tick-one = Vink één release aan en druk dan op Downloaden als film.
downloads-group-downloading = Downloaden
downloads-group-downloading-line = Nu bezig; Activiteit rechts heeft de details.
downloads-group-seeding = Seeden
downloads-group-seeding-line = Gedownload en nog aan het delen.
downloads-group-done = Klaar
downloads-group-done-line = Gedownload; de client deelt ze niet.
downloads-group-stopped = Gestopt
downloads-group-stopped-line = Gestopt voordat ze klaar waren.
downloads-group-broken = Bestanden ontbreken
downloads-group-broken-line = De client vindt de bestanden niet: ze wijzen naar een map die er niet is. De torrents weghalen ruimt de client op.
downloads-sub-nofeed = Kan niet volgen
downloads-sub-nofeed-line = Hun bron beantwoordt zoekopdrachten maar heeft geen feed, dus er komen geen nieuwe afleveringen. Zeg op en volg de serie op een bron die er wel een heeft.
downloads-sub-complete = Compleet
downloads-sub-complete-line = Elke aflevering staat in de bibliotheek en er wordt niets meer uitgezonden: deze abonnementen hebben niets meer te doen.
downloads-sub-airing = Loopt
downloads-sub-airing-line = Nieuwe afleveringen komen nog, de eerstvolgende eerst.
downloads-sub-incomplete = Incompleet
downloads-sub-incomplete-line = Uitzending afgelopen met afleveringen die er niet zijn, of niets zegt hoe de serie ervoor staat.
downloads-sub-missing = Niet in de bibliotheek
downloads-sub-missing-line = Hun map bevat niets dat de bibliotheek heeft gevonden: er is nog niets binnen, of het slaat op waar de bibliotheek niet kijkt.
downloads-torrents =
    { $count ->
        [one] { $count } torrent
       *[other] { $count } torrents
    }
downloads-rule-film-on-disk = Film, op schijf
downloads-rule-all-here = Alle { $total } hier
downloads-rule-next =
    { $predicted ->
        [yes] Afl. { $episode } verwacht over { $time }
       *[no] Afl. { $episode } over { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] Afl. { $episode } t/m { $through } verwacht over { $time }
       *[no] Afl. { $episode } t/m { $through } over { $time }
    }
downloads-rule-have-of = { $have } van { $total } hier
downloads-rule-have =
    { $count ->
        [one] { $count } aflevering hier
       *[other] { $count } afleveringen hier
    }
downloads-tag-not-downloaded = Niet gedownload
downloads-tag-missing = { $count } ontbreken
downloads-tag-n-not-downloaded = { $count } niet gedownload
downloads-tag-unknown = Onbekend
downloads-speed-down = { $speed }/s omlaag
downloads-time-left = nog { $time }
downloads-repair-trying = Kortere zoekopdrachten proberen
downloads-sent-to-client = { $title } naar { $client } gestuurd
downloads-files-torrents = wat de torrents downloadden
downloads-files-torrents-remote = wat de torrents downloadden, gewist door { $client }
downloads-files-folder-and = { $folder } en { $torrents }
downloads-unsubscribe-heading = Abonnement op { $title } opzeggen?
downloads-unsubscribe-base = De feed en downloadregel verdwijnen uit { $client }, dus er komt niets nieuws binnen.
downloads-unsubscribe-complete-heading = Alle { $count } complete series opzeggen?
downloads-unsubscribe-complete-base = Hun feeds en downloadregels verdwijnen uit { $client }. Elke aflevering ervan staat in de bibliotheek.
downloads-files-folders-and-torrents = hun mappen en wat hun torrents downloadden
downloads-one-at-a-time = Ze gaan één voor één; de pagina toont elke stap.
downloads-subscription-stays = { $base } Het abonnement blijft.
downloads-cancel-heading = { $title } annuleren?
downloads-cancel-group-base =
    { $count ->
        [one] De download stopt en verdwijnt uit { $client }.
       *[other] De { $count } downloads stoppen en verdwijnen uit { $client }.
    }
downloads-files-so-far-it = wat hij tot nu toe heeft gedownload
downloads-files-so-far-they = wat ze tot nu toe hebben gedownload
downloads-cancel-download = Download annuleren
downloads-keep-downloading = Blijf downloaden
downloads-remove-heading = { $title } weghalen?
downloads-remove-group-base =
    { $count ->
        [one] De torrent verdwijnt uit { $client }; de bestanden blijven tenzij je hieronder aanvinkt.
       *[other] De { $count } torrents verdwijnen uit { $client }; de bestanden blijven tenzij je hieronder aanvinkt.
    }
downloads-remove-detail-base =
    { $count ->
        [one] De { $count } torrent verdwijnt uit { $client }; de bestanden blijven tenzij je hieronder aanvinkt.
       *[other] De { $count } torrents verdwijnen uit { $client }; de bestanden blijven tenzij je hieronder aanvinkt.
    }
downloads-remove-broken-heading = De torrents van alle { $count } series met ontbrekende bestanden weghalen?
downloads-remove-broken-base = Hun { $count } torrents verdwijnen uit { $client }. Ze wijzen naar mappen die er niet zijn, dus er wordt geen bestand aangeraakt; abonnementen blijven.
downloads-remove-torrent-heading = Deze torrent weghalen?
downloads-files-it-downloaded = wat hij downloadde
downloads-cancel-torrent-heading = Deze download annuleren?
downloads-cancel-many-heading =
    { $count ->
        [one] { $count } download annuleren?
       *[other] { $count } downloads annuleren?
    }
downloads-cancel-many-base = Elke download van deze serie stopt en verdwijnt uit { $client }.
downloads-cancel-all = Alles annuleren
downloads-summary-connecting = Verbinden.
downloads-summary-client-on = { $client } { $version } is gekoppeld.
downloads-summary-client-off = { $client } is niet gekoppeld. Controleer het adres en wachtwoord in Instellingen, onder Downloads.
downloads-summary-native = AniBeam doorzoekt je bronnen zelf.
downloads-summary-native-with = AniBeam doorzoekt je bronnen zelf. { $client }
downloads-summary-starting = Downloads starten.
downloads-summary-not-running = Downloads draait niet.
downloads-summary-not-started = Downloads is nog niet gestart.
downloads-summary-helper-ready = De zoekhulp { $version } is klaar. { $client }
downloads-subscribed-as = Geabonneerd als { $name }.
downloads-repaired =
    { $count ->
        [one] { $name } hersteld: de feed vindt { $count } release
       *[other] { $name } hersteld: de feed vindt { $count } releases
    }
downloads-repaired-keeps =
    { $count ->
        [one] { $name } hersteld: de feed vindt { $count } release en de regel houdt “{ $term }”
       *[other] { $name } hersteld: de feed vindt { $count } releases en de regel houdt “{ $term }”
    }
downloads-sent-torrents =
    { $count ->
        [one] { $count } torrent naar { $client } gestuurd.
       *[other] { $count } torrents naar { $client } gestuurd.
    }
downloads-added-torrents =
    { $count ->
        [one] { $count } torrent toegevoegd.
       *[other] { $count } torrents toegevoegd.
    }
downloads-trashed-paths =
    { $count ->
        [one] { $count } pad van { $title } naar de prullenbak verplaatst
       *[other] { $count } paden van { $title } naar de prullenbak verplaatst
    }
downloads-show-gone = Die serie staat niet meer in de bibliotheek
downloads-activity-hide = Activiteit verbergen
downloads-activity = Activiteit
downloads-moving =
    { $count ->
        [one] { $count } download bezig
       *[other] { $count } downloads bezig
    }
downloads-back = Terug naar Downloads
downloads-chip-search-ready = Zoeken klaar
downloads-chip-search-starting = Zoeken start
downloads-chip-search-off = Zoeken uit
downloads-chip-answers-at = { $summary } Het antwoordt op { $url }.
downloads-chip-helper-missing = De zoekhulp is niet geïnstalleerd. Klik voor de Downloads-instellingen, die zeggen hoe je hem toevoegt
downloads-chip-helper-failed = De zoekhulp is gestopt. Opnieuw verbinden start hem weer; de Downloads-instellingen zeggen wat er misging
downloads-chip-open-settings = Downloads-instellingen openen
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } uit
downloads-chip-client-not-connected = { $client } is niet gekoppeld. Verbind opnieuw, of controleer het adres in Instellingen
downloads-chip-client-remote = Een zoekhulp die je zelf draait gebruikt de client die in zijn eigen instellingen staat
downloads-chip-client-switch = Klik om van client te wisselen
downloads-connecting = Verbinden
downloads-reconnect = Opnieuw verbinden
downloads-search-placeholder = Zoek een serie, of een releasenaam
downloads-search-go-tip = Zoek releasenamen op { $names }
downloads-search-go-off = Elke bron staat uit.
downloads-suggest-airing =
    { $predicted ->
        [yes] Loopt, aflevering { $episode } verwacht over { $time }
       *[no] Loopt, aflevering { $episode } over { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] Loopt, afleveringen { $episode } t/m { $through } verwacht over { $time }
       *[no] Loopt, afleveringen { $episode } t/m { $through } over { $time }
    }
downloads-suggest-finished = Afgelopen
downloads-suggest-episodes = { $count } afleveringen
downloads-suggest-in-library = In je bibliotheek
downloads-suggest-on-anilist = Op AniList, niet in je bibliotheek
downloads-plan-to-watch = Later kijken
downloads-suggest-release-search = Zoek in plaats daarvan releasenamen voor “{ $query }” op { $names }
downloads-key-return = Enter
downloads-sources = Bronnen
downloads-source-hint-starting = starten
downloads-source-hint-down = draait niet
downloads-source-tip-down = { $name } heeft een hulpdienst nodig die niet draait. Klik om hem te starten.
downloads-source-tip-on = Zoekopdrachten gaan naar { $name }. Klik om het uit te zetten.
downloads-source-tip-off = Uitgezet. Klik om { $name } weer te vragen.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Controleren
    .tip = Vraagt of elke bron op aanvraag draait. Start niets.
downloads-check-busy = Controleren
downloads-back-to-grid = Terug naar het raster
    .tip = Terug naar { $title } en wat elke bron vond. Escape doet hetzelfde.
downloads-releases-on =
    { $count ->
        [one] { $count } release op { $name }
       *[other] { $count } releases op { $name }
    }
downloads-found-on = Gevonden op
downloads-found-searching = zoeken
downloads-found-failed = mislukt
downloads-found-cancelled = geannuleerd
downloads-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } releases
    }
downloads-found-retry-tip =
    { $why }
    Klik om het opnieuw te proberen
downloads-facet-groups = Releasegroep
downloads-facet-quality = Kwaliteit
downloads-facet-words = Woorden
downloads-filters-fewer = Minder filters
downloads-filters-more = Meer filters ({ $count })
downloads-narrow-placeholder = Verfijn op een term, of -term om weg te laten
downloads-best-match = Beste match
    .tip = Houdt de releases die op de beste lijken: { $query }
downloads-best-match-none = Geen release springt hier uit
downloads-strip-have-of = { $have } van { $total } afleveringen op schijf
downloads-strip-have =
    { $count ->
        [one] { $count } aflevering op schijf
       *[other] { $count } afleveringen op schijf
    }
downloads-strip-next =
    { $predicted ->
        [yes] aflevering { $episode } verwacht over { $time }
       *[no] aflevering { $episode } over { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] afleveringen { $episode } t/m { $through } verwacht over { $time }
       *[no] afleveringen { $episode } t/m { $through } over { $time }
    }
downloads-strip-complete = compleet
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = Geen van deze releases is voor een aflevering die je al hebt.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } release hier is het bestand dat je hebt, gemarkeerd als Op schijf
       *[other] { $count } releases hier zijn het bestand dat je hebt, gemarkeerd als Op schijf
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } release is voor een aflevering die je van een andere release hebt
       *[other] { $count } releases zijn voor een aflevering die je van een andere release hebt
    }
downloads-strip-both = { $first }; { $second }.
downloads-tick-missing =
    { $count ->
        [one] De { $count } ontbrekende aflevering aanvinken
       *[other] De { $count } ontbrekende afleveringen aanvinken
    }
    .tip = Voor elke aflevering die de bibliotheek hier niet heeft, de release met de meeste seeders
downloads-open-in-library = Openen in bibliotheek
downloads-not-this-show = Niet deze serie
downloads-no-releases = Geen releases
downloads-no-releases-body = Niets op { $name } past bij “{ $query }”. Een andere bron, of minder woorden, vindt het misschien.
downloads-try-source = Probeer { $name }
downloads-ticked =
    { $count ->
        [one] { $count } release aangevinkt
       *[other] { $count } releases aangevinkt
    }
downloads-nothing-ticked = Niets aangevinkt
downloads-ticked-size = { $size } samen.
downloads-tick-hint = Vink releases aan in de tabel, of download ze allemaal.
downloads-tick-all = Alles aanvinken
downloads-working = Bezig
downloads-download-ticked =
    { $count ->
        [one] { $count } release downloaden
       *[other] { $count } releases downloaden
    }
downloads-download-shown = De { $count } getoonde downloaden
downloads-download-all = Alle { $count } downloaden
downloads-download-movie = Downloaden als film
    .tip = Eén aangevinkte release, rechtstreeks naar de filmmap
downloads-subscribe = Abonneren
downloads-subscribe-body = Nieuwe releases die bij deze zoekopdracht passen worden gedownload zodra ze verschijnen, in de map die hier staat.
downloads-name-placeholder = Uit de eerste release gehaald
downloads-subscribe-search = Abonneren op deze zoekopdracht
downloads-subscribe-no-feed = Hier is geen feed om op te abonneren
downloads-preview-again = Toon de regel opnieuw
downloads-preview-show = Toon de regel die het maakt
downloads-preview-from = Van { $host }
downloads-preview-from-named = Van { $host } ({ $name })
downloads-preview-must = Moet { $terms } bevatten
downloads-preview-must-not = Mag { $terms } niet bevatten
downloads-helper-changed = Downloads staat op een hulp die niet antwoordt
downloads-helper-missing = Deze kopie van AniBeam kwam zonder downloads
downloads-helper-failed = Downloads kon niet starten
downloads-helper-changed-body = Zoeken en downloaden lopen via een klein hulpprogramma. Instellingen wijzen naar een ander dan dat van AniBeam zelf, en dat antwoordt niet. Schakel terug en het start vanzelf.
downloads-helper-missing-body = Hij is gebouwd zonder het hulpprogramma waarmee zoeken en downloaden werken. De installers op de releasepagina hebben het wel. Instellingen, Downloads zegt hoe je het aan deze kopie toevoegt.
downloads-helper-failed-body = Het hulpprogramma waarmee zoeken en downloaden werken stopte voordat het klaar was. Opnieuw starten lost dit meestal op.
downloads-helper-use-own = Die van AniBeam gebruiken
downloads-helper-start-again = Opnieuw starten
downloads-helper-show-how = Laat zien hoe
downloads-helper-what-went-wrong = Wat ging er mis
downloads-lens-gaps-count = Gaten in je bibliotheek  { $count }
downloads-lens-airing-count = Loopt  { $count }
downloads-lens-following-count = Gevolgd  { $count }
downloads-lens-downloaded-count = Gedownload  { $count }
downloads-lens-gaps = Gaten in je bibliotheek
downloads-lens-gaps-line = Uitgezonden afleveringen die niet op schijf staan. Open een serie om te zien wie ze heeft.
downloads-lens-gaps-none = Er ontbreekt niets: elke uitgezonden aflevering van elke serie staat op schijf.
downloads-lens-airing-line = Series in je bibliotheek met afleveringen die nog komen. Open er een om hem te volgen.
downloads-lens-airing-none = Niets in je bibliotheek loopt
downloads-lens-airing-none-body = Een serie komt hier zodra hij in je bibliotheek staat en er nog afleveringen komen. Nu uitgezonden toont alles wat loopt.
downloads-airing-now = Nu uitgezonden
downloads-lens-following = Gevolgd
downloads-card-next =
    { $predicted ->
        [yes] Aflevering { $episode } verwacht over { $time }
       *[no] Aflevering { $episode } over { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Afleveringen { $episode } t/m { $through } verwacht over { $time }
       *[no] Afleveringen { $episode } t/m { $through } over { $time }
    }
downloads-rules-unread = Je abonnementen konden niet worden gelezen.
downloads-lens-following-line = Feeds die AniBeam volgt, als de series die ze vullen.
downloads-none-yet = Nog geen.
downloads-reading = Lezen.
downloads-rules-error = AniBeam kon je abonnementen niet lezen. Probeer het zo opnieuw.
downloads-removing-to-go = Weghalen, nog { $count }
downloads-unsubscribe-all = Alle { $count } opzeggen
downloads-unsubscribe = Opzeggen
downloads-no-rules = Geen abonnementen
downloads-no-rules-body = Zoek hierboven en abonneer om een serie binnen te laten komen.
downloads-search = Zoeken
downloads-lens-downloaded = Gedownload
downloads-dl-line = { $size } op schijf. { $rest }
downloads-dl-client-off = { $client } is niet gekoppeld.
downloads-dl-unread = De downloadlijst kon niet worden gelezen.
downloads-dl-in-client =
    { $count ->
        [one] { $count } torrent, { $size } in { $client }.
       *[other] { $count } torrents, { $size } in { $client }.
    }
downloads-usage-watched = Gekeken
downloads-usage-not-watched = Niet gekeken
downloads-deleting = Wissen
downloads-delete-watched = Gekeken wissen
downloads-remove-all = Alle { $count } weghalen
downloads-on-disk = Op schijf
downloads-on-disk-line = In je bibliotheek zonder torrent in { $client }, de grootste eerst. Klik met rechts om er een te wissen.
downloads-nothing-downloaded = Niets gedownload
downloads-nothing-downloaded-body = Wat je downloadt of waarop je abonneert verschijnt hier, per serie.
downloads-client-off-title = { $client } is niet gekoppeld
downloads-client-off-body = Controleer of hij draait en verbind opnieuw. Het adres en wachtwoord staan in Instellingen, onder Downloads.
downloads-connect-again = Opnieuw verbinden
downloads-view-grid = Raster
downloads-view-list = Lijst
downloads-chip-subscribed = Geabonneerd
downloads-speed-up = { $speed }/s omhoog
downloads-broken-some = { $count } met ontbrekende bestanden
downloads-tag-watched = Gekeken

## PlayerDelays.qml

player-delays-title = Vertraging
player-delays-subtitles = Ondertitels
player-delays-audio = Audio
player-delays-subtitles-earlier = Ondertitels 0,1 s eerder
player-delays-subtitles-later = Ondertitels 0,1 s later
player-delays-audio-earlier = Audio 0,1 s eerder
player-delays-audio-later = Audio 0,1 s later
player-delays-reset = Tot het volgende bestand op 0 zetten
player-delays-none = Geen vertraging

## TrackPicker.qml

track-picker-off = Uit

## RollControls.qml

roll-controls-window = { $ahead } klaar houden: afleveringen { $next } t/m { $through } van { $total }
roll-controls-window-watched = { $ahead } klaar houden: afleveringen { $next } t/m { $through } van { $total }, { $watched } gekeken
roll-controls-more = Nog { $count } ophalen
roll-controls-more-tip = Download de volgende afleveringen nu. AniBeam houdt de volgende paar weer klaar zodra je bij bent.
roll-controls-fewer-tip = Houd er één minder klaar
roll-controls-ahead = { $count } vooruit
roll-controls-further-tip = Houd er één meer klaar
roll-controls-all = Alles ophalen
roll-controls-all-tip = Stop met een paar klaar houden en download elke resterende aflevering
roll-controls-all-question = Alle { $count } resterende afleveringen downloaden?

## PlayerChrome.qml

player-chrome-delays = Vertraging ondertitels en audio
player-chrome-audio = Audiospoor
player-chrome-subtitles = Ondertitels
player-chrome-subtitles-off = Ondertitels uit
player-chrome-casting-to = Casten naar { $device }
player-chrome-cast = Casten naar een tv
player-chrome-mark-watched = Als gekeken markeren
player-chrome-shortcuts = Sneltoetsen
player-chrome-fullscreen = Volledig scherm
player-chrome-exit-fullscreen = Volledig scherm sluiten
player-chrome-stay = Blijven
player-chrome-skip-intro = Intro overslaan
player-chrome-skip-outro = Outro overslaan
player-chrome-playing-on = Speelt op { $device }
player-chrome-replay = Opnieuw afspelen

## PlayerPage.qml

player-title = Speler
player-next = Volgende
player-extra-no-next = Een extra heeft geen volgende aflevering
player-last-episode = Dit is de laatste aflevering
player-extra-no-previous = Een extra heeft geen vorige aflevering
player-first-episode = Dit is de eerste aflevering
player-resumed-at = Hervat bij { $time }
player-start-over = Opnieuw beginnen
player-audio-track = Audio: { $track }
player-subtitles-track = Ondertitels: { $track }
player-subtitles-on = Ondertitels: aan
player-subtitles-off = Ondertitels uit
player-no-subtitles = Dit bestand heeft geen ondertitels
player-delay-seconds = { $value } s
player-audio-delay = Audiovertraging { $value }
player-subtitle-delay = Ondertitelvertraging { $value }
player-volume = Volume { $volume }
player-speed = Snelheid { $speed }×
player-not-while-casting = Niet tijdens casten
player-hold-fast = 2× zolang ingedrukt
player-no-chapters = Dit bestand heeft geen hoofdstukken
player-last-chapter = Dit is het laatste hoofdstuk
player-chapter = Hoofdstuk { $number }
player-chapter-titled = Hoofdstuk { $number }: { $title }
player-screenshot-saved = Opgeslagen in Pictures/AniBeam
player-skipped-intro = Intro overgeslagen
player-skipped-outro = Outro overgeslagen
player-undo = Ongedaan maken
player-refusal-hidden = de serie is verborgen
player-refusal-no-match = de serie is nog niet gekoppeld; koppel hem bij Serie-info
player-refusal-not-newer = je lijst is al voorbij deze aflevering
player-refusal-extra = dit is een extra
player-refusal-unmatched = het bestand is niet aan een aflevering gekoppeld
player-refusal-on-disk = het bestand staat alleen op schijf
player-not-tracked-because = Niet bijgehouden: { $reason }.
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = { $tracker } kon niet worden bijgewerkt
player-outcome-ok = { $tracker } ok
player-outcome-at = { $tracker } op { $progress }
player-outcome-tracked = Bijgehouden
player-outcome-not-tracked = Niet bijgehouden
player-outcome-could-not-update = Kon je lijst niet bijwerken
player-outcome-tracked-list = Bijgehouden: { $list }
player-outcome-not-tracked-list = Niet bijgehouden: { $list }
player-outcome-failed-list = Kon je lijst niet bijwerken: { $list }
player-rated = Score { $score }
player-score-failed = Kon de score niet opslaan
player-frame = Frame { $number } op { $time }
player-audio-title = Audio
player-subtitles-title = Ondertitels
player-cast-to = Casten naar
player-cast-looking = Chromecasts en tv's zoeken
player-cast-unreachable = Kon de tv niet bereiken
player-cast-tv = TV
player-cast-stop = Casten naar { $device } stoppen
player-cast-connecting = Verbinden
player-cast-loading = Laden
player-cast-playing = Speelt
player-cast-paused = Gepauzeerd
player-cast-finished = Afgelopen
player-cast-stopped = Gestopt
player-rate-prompt = De laatste aflevering is bijgehouden. Deze serie een score geven?
player-rate = Score geven
player-not-now = Niet nu

## EndpointReposPanel.qml

endpoint-repos-panel-title = Bronlijsten
endpoint-repos-panel-helper = Een bronlijst is een ondertekende set zoekplekken die je via het adres toevoegt. AniBeam controleert de handtekening en toont de sleutel vóór opslaan; daarna installeer je wat je wilt. Die komen bij Zoekbronnen hierboven.
endpoint-repos-panel-could-not-start = AniBeam kon dat niet starten. Probeer het zo opnieuw.
endpoint-repos-panel-needs-value = { $name } heeft een waarde nodig
endpoint-repos-panel-values-of = Waarden van { $name }
endpoint-repos-panel-secret-note = Bewaard in de geheimenopslag van deze machine, nooit in de bibliotheek of een export.
endpoint-repos-panel-secret-note-keep = Bewaard in de geheimenopslag van deze machine, nooit in de bibliotheek of een export. Een leeg veld behoudt wat is opgeslagen.
endpoint-repos-panel-asks-again = { $name } versie { $version } vraagt het opnieuw
endpoint-repos-panel-update-sends-to = De nieuwe versie stuurt dit naar { $host }.
endpoint-repos-panel-fields-changed = De velden zijn veranderd.
endpoint-repos-panel-needs-fields = Heeft { $fields } nodig voordat het zoekt.
endpoint-repos-panel-no-longer-offered = Wordt niet meer aangeboden door de bronlijst.
endpoint-repos-panel-update-requests-to = De update stuurt verzoeken naar { $host }.
endpoint-repos-panel-unreadable = Deze lijst kon niet worden gelezen. Lijsten vernieuwen probeert het opnieuw.
endpoint-repos-panel-add-list = Bronlijst toevoegen
endpoint-repos-panel-add-list-helper = Het adres van het indexbestand, of een pad ernaar op deze machine.
endpoint-repos-panel-checking = Controleren
endpoint-repos-panel-add = Toevoegen
endpoint-repos-panel-trust-question = { $name } vertrouwen?
endpoint-repos-panel-fingerprint = Vingerafdruk sleutel  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    { $count } bronnen, ondertekend. De bronnen bepalen waar je zoekopdrachten heen gaan; vertrouw een lijst alleen van iemand die je vertrouwt.
endpoint-repos-panel-preview-trusted =
    { $url }
    { $count } bronnen, ondertekend, al vertrouwd. De bronnen bepalen waar je zoekopdrachten heen gaan; vertrouw een lijst alleen van iemand die je vertrouwt.
endpoint-repos-panel-trust = Vertrouwen
endpoint-repos-panel-saving = Opslaan
endpoint-repos-panel-install = Installeren
endpoint-repos-panel-update = Bijwerken
endpoint-repos-panel-added-by-you = door jou toegevoegd
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Bijwerken naar v{ $version }
endpoint-repos-panel-add-value = Waarde toevoegen
endpoint-repos-panel-change = { $fields } wijzigen
endpoint-repos-panel-asks-for = Vraagt om je { $fields }.
endpoint-repos-panel-by-hand = Handmatig toevoegen
endpoint-repos-panel-by-hand-helper = Voor sites zonder lijst: elke feed met {"{"}query{"}"} op de plek van de zoekterm, ook via een proxy. Een sleutel in het adres gaat apart naar de wachtwoordopslag.
endpoint-repos-panel-name = Naam
endpoint-repos-panel-refreshing = Vernieuwen
endpoint-repos-panel-refresh = Lijsten vernieuwen

## ScheduleSheet.qml

schedule-episodes-range = Afleveringen { $first } t/m { $last }
schedule-day-format = ddd d MMM yyyy
schedule-month-format = MMMM yyyy
schedule-tile-day-format = d MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = Ergens in { $year }
schedule-no-date-yet = Nog geen datum
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = Geen datum
schedule-tile-last = Laatste
schedule-tile-out = Uit
schedule-ends-at = Eindigt bij aflevering { $number }
schedule-dated-by-site = { $count } gedateerd door { $site }
schedule-dated-by-record = { $count } gedateerd door het item
schedule-set-by-you = { $count } door jou ingesteld
schedule-guessed-weekly = { $count } wekelijks geschat
schedule-nothing-set = Nog niets ingesteld
schedule-saved = Opgeslagen
schedule-cleared = Schemanotities gewist
schedule-title = Afleveringen en data
schedule-season = Seizoen { $number }
schedule-ask-agent = Vraag { $agent }
schedule-done-pasting = Klaar met plakken
schedule-paste-list = Lijst plakken
schedule-agent-looking = { $agent } zoekt de data op. Er wordt niets opgeslagen tot je op Opslaan drukt.
schedule-agent-stopped = { $agent } is gestopt: { $error }
schedule-stop = Stop
schedule-paste-placeholder = Plak een releaselijst, een tabel of een bericht
schedule-paste-helper = Wat AniBeam leest verschijnt bij de afleveringen hieronder.
schedule-agent-hint = Met Claude Code op deze computer kan AniBeam het vragen de data voor je op te zoeken.
schedule-show-more = Meer afleveringen tonen
schedule-pick-hint = Klik op een aflevering om hem een datum te geven. Shift-klik kiest een reeks.
schedule-picked-one = Aflevering { $number }:
schedule-picked = { $count ->
    [one] { $count } aflevering:
   *[other] { $count } afleveringen:
}
schedule-date-placeholder = 7 jan, maart 2027, 2027
schedule-set-date = Datum instellen
schedule-weekly = Wekelijks
schedule-not-last = Niet de laatste
schedule-last-episode = Laatste aflevering
schedule-clear-date = Datum wissen

## ShowPage.qml

show-title = Serie
show-runtime-each = { $runtime } per stuk
show-add-to-list = Op lijst zetten
show-moving-to = Naar { $list } zetten
show-moved-to = Naar { $list } gezet
show-relation-related = Verwant
show-no-poster = Geen poster
show-reading-anilist = AniList lezen
show-not-in-library = Niet in je bibliotheek
show-get-episodes = Afleveringen ophalen
show-trailer = Trailer
show-your-score = Jouw score { $score }
show-rate-it = Score geven
show-anilist-score = AniList { $score }
show-forecast = { $percent }% voor jou
show-because-you-watched = Omdat je keek
show-hidden-loved = { $count ->
    [one] Er zit een verborgen tag achter die je geweldig vindt
   *[other] Er zitten { $count } verborgen tags achter die je geweldig vindt
}
show-you-gave-it = Je gaf het een { $score }
show-story = Verhaal
show-genres-tags = Genres en tags
show-studios = Studio's  { $studios }
show-related = Verwant
show-recommendations = Aanbevelingen
show-from-anilist = Van AniList
show-reading-rest = De rest van AniList lezen

## ConnectSheet.qml

connect-connected-to = Gekoppeld met { $name }
connect-connect-to = { $name } koppelen
connect-did-not-finish = Het inloggen is niet afgerond
connect-finish-in-browser = Rond af in je browser
connect-signed-in-anilist = Ingelogd als { $account }. Je lijst staat op de pagina Kijken.
connect-signed-in-mal = Ingelogd als { $account }. Afleveringen worden daar gemarkeerd terwijl je kijkt.
connect-setup-lead = { $name } logt AniBeam in via een eigen app van jou. Dat kost één keer ongeveer een minuut.
connect-page-open = De inlogpagina van { $name } staat open in je browser. Keur AniBeam daar goed en dit sluit vanzelf.
connect-page-opening = De inlogpagina van { $name } openen in je browser.
connect-open-watching = Kijken openen
connect-done = Klaar
connect-open-again = Open de inlogpagina opnieuw
connect-secret-where = Het is de lange tekenreeks onder "Client Secret" op de pagina van de app op myanimelist.net/apiconfig.
connect-client-secret = Client secret
connect-save-try-again = Opslaan en opnieuw proberen
connect-change-client-id = Client ID wijzigen
connect-step-create-anilist = Maak de client aan op AniList
connect-step-create-mal = Maak de app aan op MyAnimeList
connect-step-create-anilist-body = Open de ontwikkelaarsinstellingen van AniList, maak een nieuwe client aan en plak dit adres precies zoals het hier staat in het vak Redirect URL.
connect-step-create-mal-body = Open de API-pagina van MyAnimeList, maak een ID aan met App Type "other" en plak dit adres precies zoals het hier staat in het vak App Redirect URL.
connect-open-developer-page = Ontwikkelaarspagina openen
connect-copy-address = Adres kopiëren
connect-step-paste-id = Plak het ID van de app
connect-step-paste-id-anilist = AniList noemt het Client ID: het getal op de pagina van je nieuwe client.
connect-step-paste-id-mal = MyAnimeList noemt het Client ID: de lange tekenreeks op de pagina van je nieuwe app.
connect-client-id = Client ID
connect-client-secret-web = Client secret (alleen voor web-apps)
connect-step-connect-body = De browser opent, { $name } vraagt je in te loggen en AniBeam goed te keuren, en AniBeam vangt het antwoord op.

## SettingsPage

settings-title = Instellingen
settings-search-placeholder = Instellingen zoeken
settings-section-library = Bibliotheek
settings-section-accounts = Accounts
settings-section-appearance = Weergave
settings-section-playback = Afspelen
settings-section-subtitles = Ondertitels
settings-section-keys = Toetsenbord
settings-section-downloads = Downloads
settings-section-phone = Telefoonapp
settings-section-storage = Opslag en back-up
settings-section-help = Help
settings-section-about = Over
settings-section-developer = Ontwikkelaar
settings-status-library-empty = Nog geen mappen
settings-status-library =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    } in { $folders ->
        [one] { $folders } map
       *[other] { $folders } mappen
    }
settings-status-library-missing =
    { $gone ->
        [one] { $gone } map
       *[other] { $gone } mappen
    } niet gevonden, { $count ->
        [one] { $count } serie
       *[other] { $count } series
    } in { $folders ->
        [one] { $folders } map
       *[other] { $folders } mappen
    }
settings-status-accounts-anilist = Synct met AniList
settings-status-accounts-mal = Synct met MyAnimeList
settings-status-accounts-both = Synct met AniList en MyAnimeList
settings-status-accounts-none = Geen lijst gekoppeld
settings-status-appearance =
    { $mode ->
        [system] Volgt het systeem
        [light] Licht
       *[dark] Donker
    }, { $source ->
        [theme] themakleuren
       *[desktop] bureaubladkleuren
    }
settings-status-playback =
    { $skip ->
        [both] Slaat intro's en outro's over
        [intro] Slaat intro's over
        [outro] Slaat outro's over
       *[none] Speelt alles helemaal af
    }
settings-status-playback-audio =
    { $skip ->
        [both] Slaat intro's en outro's over
        [intro] Slaat intro's over
        [outro] Slaat outro's over
       *[none] Speelt alles helemaal af
    }, audio { $languages }
settings-status-subtitles =
    Grootte { $percent } procent, { $override ->
        [AsScripted] opgemaakte bestanden zoals gemaakt
        [Force] jouw stijl op elk bestand
       *[ScaleOnly] opgemaakte bestanden houden hun uiterlijk
    }
settings-status-keys = Elke toets waar AniBeam op reageert
settings-status-downloads-unread = Nog niet gelezen
settings-status-downloads-answered = qBittorrent antwoordde
settings-status-downloads-missing = Zoekhulp niet geïnstalleerd
settings-status-downloads-unconnected = Nog niet gekoppeld
settings-status-phone-shared = Gedeeld op je tailnet
settings-status-phone-off = Niet gedeeld
settings-status-storage = { $size } in beeldcache
settings-status-storage-unread = Afbeeldingen, export en import
settings-status-help = Veelgestelde vragen
settings-status-about-update = Versie { $version } is uit
settings-status-developer = Elke pagina timen
settings-status-developer-last = { $page } gebouwd in { $ms } ms
settings-results =
    { $count ->
        [one] { $count } instelling past
       *[other] { $count } instellingen passen
    }
settings-results-none = Niets past
settings-results-hint = Enter opent de eerste.
settings-results-none-hint = Probeer een eenvoudiger woord: thema, ondertitels, tracker, wachtwoord, map.
settings-find-folders =
    .words = map toevoegen scannen opnieuw weghalen bron pad collectie
settings-find-matching =
    .words = metadata serie-info bron anilist tvmaze volgorde koppelen
settings-find-hidden-shows =
    .words = verborgen tonen
settings-find-hidden-airing =
    .words = niet interessant uitgezonden opnieuw tonen
settings-find-calendar =
    .words = agenda kalender ics ical uitzending schema abonneren
settings-find-anime-lists =
    .words = anilist myanimelist mal inloggen koppelen login account ontkoppelen tracker
settings-find-main-list =
    .words = voortgang kaarten hoofdtracker
settings-find-sync-now =
    .words = vernieuwen voortgang lijsten tracker sync
settings-find-on-hold =
    .words = in de wacht gepauzeerd inactief stil dagen
settings-find-language =
    .words = taal vertaling locale nederlands engels
settings-find-mode =
    .words = donker licht systeem thema nacht
settings-find-colours-from =
    .words = terminal bureaublad systeem thema bron kitty
settings-find-dark-theme =
    .words = kleurenschema base16
settings-find-light-theme =
    .words = kleurenschema base16
settings-find-accent =
    .words = kleur markering
settings-find-text-size =
    .words = lettertype tekst woorden groter kleiner zoom schaal lezen toegankelijkheid
settings-find-density =
    .words = compact ruim ruimte grootte
settings-find-poster-size =
    .words = kaarten klein middel groot
settings-find-out-now =
    .words = uitgezonden nu uit kaarten lijst rijen stijl banner posters
settings-find-out-now-cards =
    .words = uitgezonden nu uit raster kolommen rijen indeling
settings-find-reduce-motion =
    .words = animatie beweging toegankelijkheid
settings-find-app-icon =
    .words = icoon logo launcher dock taakbalk venster donker
settings-find-skip-intros =
    .words = opening op automatisch overslaan aniskip hoofdstukken
settings-find-skip-outros =
    .words = ending ed aftiteling automatisch overslaan aniskip
settings-find-mpv-settings =
    .words = mpv.conf config geavanceerd
settings-find-discord =
    .words = discord rich presence status profiel
settings-find-next-episode =
    .words = volgende knop titel autoplay einde
settings-find-subtitle-languages =
    .words = taal spoor ondertitels subs
settings-find-audio-languages =
    .words = taal spoor audio dub japans
settings-find-start-over =
    .words = herstellen standaard ondertitelstijl
settings-find-size =
    .words = schaal grootte ondertitels
settings-find-position =
    .words = hoogte marge positie ondertitels
settings-find-styled =
    .words = ass overschrijven forceren
settings-find-font =
    .words = lettertype
settings-find-text-colour =
    .words = kleur
settings-find-outline =
    .words = rand omlijning
settings-find-background =
    .words = vak achtergrond doorzichtigheid
settings-find-bold =
    .words = vet dikte
settings-find-keys-player =
    .words = toetsenbord sneltoetsen toetsen hotkeys
settings-find-keys-player-around =
    .words = toetsenbord sneltoetsen volledig scherm volgende vorige aflevering terug
settings-find-keys-around =
    .words = toetsenbord sneltoetsen terug vooruit rechtsklik menu volledig scherm
settings-find-keys-anywhere =
    .words = toetsenbord sneltoetsen toetsen hotkeys instellingen afsluiten
settings-find-torrent-client =
    .words = qbittorrent wachtwoord gebruikersnaam adres webui seedbox
settings-find-search-helper =
    .words = anirss rss feeds token commando server geavanceerd
settings-find-next-few =
    .words = doorlopend afleveringen vooruit wissen gekeken houden schijfruimte
settings-find-unsubscribe =
    .words = opzeggen feeds afgelopen
settings-find-last-search =
    .words = downloads zoekopdracht onthouden
settings-find-search-sources =
    .words = endpoints feeds config anirss bronnen
settings-find-source-lists =
    .words = repo repos endpoint ondertekend endpoints jackett prowlarr bronlijst
settings-find-share-tailscale =
    .words = telefoon android tailnet op afstand host
settings-find-stream =
    .words = streamen afspelen casten afleveringen telefoon bestanden
settings-find-devices =
    .words = vertrouwen vertrouwd uitzetten blokkeren apparaat tailscale
settings-find-back-up =
    .words = exporteren back-up bestand json
settings-find-restore =
    .words = importeren herstellen bestand json
settings-find-image-cache =
    .words = posters wissen afbeeldingen schijf cache
settings-find-files =
    .words = database gegevens config cache map pad
settings-find-help-folders =
    .words = help map indeling namen afleveringen seizoenen films bestanden structuur
settings-find-help-recognised =
    .words = help koppelen verkeerde serie niet gevonden ongekoppeld herkend
settings-find-help-anilist =
    .words = help anilist myanimelist mal inloggen koppelen login tracker
settings-find-help-downloads =
    .words = help downloads leeg bronnen niets torrent qbittorrent
settings-find-help-data =
    .words = help gegevens bestanden back-up database privacy
settings-find-help-else =
    .words = help instellen introductie rondleiding melden bug probleem issue
settings-find-version =
    .words = build wat is nieuw changelog versie
settings-find-setup =
    .words = eerste start rondleiding introductie opnieuw instellen
settings-find-updates =
    .words = stabiel nightly kanaal controleren updates
settings-find-developer-options =
    .words = debug timing ontwikkelaar
settings-find-ratings =
    .words = imdb bronvermelding beoordelingen

## SettingsDownloadsTab

settings-downloads-title = Downloads
settings-downloads-lead = Waar AniBeam je downloads naartoe stuurt, en hoe het de releasefeeds doorzoekt.
settings-downloads-keep-none = Een aflevering afkijken wist de vorige.
settings-downloads-keep-line = Gekeken afleveringen die achter de laatste blijven. Met { $count } wist afkijken van aflevering { $finishing } aflevering 1.
settings-downloads-anirss-ready = Draait, versie { $version } op { $url }.
settings-downloads-anirss-starting = De zoekhulp starten.
settings-downloads-anirss-missing = De zoekhulp is niet geïnstalleerd. Installeer AniBeam opnieuw, of geef je eigen anirss op bij Commando.
settings-downloads-anirss-failed = De zoekhulp is gestopt. Opnieuw proberen start hem; Details zegt wat hij meldde.
settings-downloads-anirss-idle = Nog niet gestart. Hij start de eerste keer dat je zoekt op de pagina Downloads.
settings-downloads-torrent-trying = { $client } proberen.
settings-downloads-torrent-answered = { $client } { $version } antwoordde.
settings-downloads-torrent-failed = { $client } kon niet verbinden. Controleer adres en wachtwoord en test opnieuw.
settings-downloads-torrent-untried = Nog niet geprobeerd.
settings-downloads-mode-existing = Een qBittorrent die je zelf draait, met de Web UI aan.
settings-downloads-mode-managed = AniBeam draait een eigen qBittorrent naar ~/Downloads/Anime. Die blijft draaien nadat AniBeam sluit, zodat abonnementen blijven downloaden.
settings-downloads-mode-auto = Gebruikt qBittorrent op het adres hieronder, anders een eigen.
settings-downloads-password-saved = Er is hier een wachtwoord voor deze client opgeslagen.
settings-downloads-password-anirss = Het wachtwoord dat de zoekhulp opsloeg wordt gebruikt; sla er hier een op om daar niet meer van af te hangen.
settings-downloads-password-none = Geen wachtwoord opgeslagen. Onnodig als de client deze machine geen login vraagt.
settings-downloads-mismatch = De zoekhulp praat met { $helperUrl } terwijl AniBeam met { $url } praat. Een zoekhulp die je zelf draait leest het adres uit zijn eigen config.toml, dus hier van client wisselen verplaatst hem niet.
settings-downloads-torrent-title = Torrentclient
settings-downloads-torrent-helper = De qBittorrent die downloadt. Het wachtwoord staat in de wachtwoordopslag van het systeem.
settings-downloads-which-client = Welke client
settings-downloads-pick-for-me = Kies voor mij
settings-downloads-my-own = Mijn eigen
settings-downloads-built-in = Ingebouwd
settings-downloads-saved-clients = Bewaarde clients
settings-downloads-saved-clients-helper = Een seedbox en die op deze machine, bijvoorbeeld. De clientchip op de pagina Downloads wisselt er ook tussen.
settings-downloads-add-client = Toevoegen
settings-downloads-name = Naam
settings-downloads-name-placeholder = Genoemd naar het adres
settings-downloads-address = Adres
settings-downloads-address-managed-helper = Alleen deze machine kan hem bereiken, en er is geen wachtwoord nodig.
settings-downloads-address-helper = Het adres van de Web UI.
settings-downloads-username = Gebruikersnaam
settings-downloads-password = Wachtwoord
settings-downloads-secret-saved = Opgeslagen
settings-downloads-secret-unset = Niet ingesteld
settings-downloads-connection = Verbinding
settings-downloads-open-config = Config openen
settings-downloads-details = Details
settings-downloads-details-torrent = { $client } zei dit toen AniBeam het probeerde.
settings-downloads-test-trying = Proberen
settings-downloads-test-save = Opslaan en testen
settings-downloads-test = Testen
settings-downloads-searching = Zoeken
settings-downloads-search-helper = Zoekhulp
settings-downloads-searching-helper = AniBeam doorzoekt je bronnen zelf en stuurt wat je kiest naar de torrentclient. De zoekhulp is niet nodig.
settings-downloads-search-helper-helper = Het deel van AniBeam dat zoekt en abonneert, een programma dat anirss heet. AniBeam draait het voor je; wijzig dit alleen als je je eigen draait.
settings-downloads-runs-in = Zoeken loopt in
settings-downloads-runs-in-native-helper = Abonnementen lopen via de torrentclient, ook als AniBeam dicht is.
settings-downloads-runs-in-helper = De zoekhulp, een apart programma dat AniBeam draait of dat je zelf draait.
settings-downloads-engine-native = AniBeam zelf
settings-downloads-how-runs = Hoe het draait
settings-downloads-how-runs-sidecar = AniBeam start het bij openen en stopt het bij afsluiten.
settings-downloads-how-runs-remote = AniBeam gebruikt er een die je al draait, op het adres en met de sleutel hieronder.
settings-downloads-runs-sidecar = AniBeam draait het
settings-downloads-runs-remote = Ik draai het
settings-downloads-command = Commando
settings-downloads-command-helper = Een naam in PATH of een volledig pad. ~/.local/bin wordt ook doorzocht.
settings-downloads-server-address-helper = De basis van de server, zonder /api/v1.
settings-downloads-key = Sleutel
settings-downloads-key-saved-helper = Er is een sleutel opgeslagen. Typ een nieuwe om hem te vervangen, of maak het veld leeg om hem weg te halen.
settings-downloads-key-helper = De sleutel die de zoekhulp toonde bij het starten. Daarmee komt AniBeam binnen.
settings-downloads-details-anirss = De zoekhulp zei dit toen hij stopte.
settings-downloads-starting = Starten
settings-downloads-reconnect = Opnieuw verbinden
settings-downloads-save-first = Sla eerst je wijzigingen op
settings-downloads-roll-title = Houd de volgende paar klaar
settings-downloads-roll-helper = Een lange serie downloadt een paar afleveringen tegelijk; elke afgekeken start de volgende. De hele reeks hoeft nooit op schijf.
settings-downloads-roll-big = Voor lange series
settings-downloads-roll-big-helper = Alles boven { $count } afleveringen downloadt vanzelf een paar tegelijk. Via de rij op Downloads haal je toch alles in één keer op.
settings-downloads-roll-ahead = Afleveringen vooruit
settings-downloads-roll-ahead-helper = Hoeveel er klaarstaan na je huidige aflevering. 12 is zo'n seizoen.
settings-downloads-unit-episodes = afleveringen
settings-downloads-roll-over = Lang betekent meer dan
settings-downloads-roll-over-helper = Downloads met meer afleveringen dan dit halen er een paar tegelijk op.
settings-downloads-roll-watched = Gekeken afleveringen
settings-downloads-roll-watched-delete-helper = Gewist zodra je verder bent, zodat de schijf alleen bevat wat je nog moet kijken.
settings-downloads-roll-watched-keep-helper = Elk bestand blijft op de schijf nadat je het hebt gekeken.
settings-downloads-keep-all = Alles houden
settings-downloads-keep-behind = Achter houden
settings-downloads-subscriptions-title = Abonnementen en zoekopdrachten
settings-downloads-last-search = Laatste zoekopdracht houden
settings-downloads-last-search-helper = De pagina Downloads opent met je laatste zoekopdracht. Uit, opent hij leeg.
settings-downloads-prune = Afgelopen series opzeggen
settings-downloads-prune-helper = 16 dagen na de laatste aflevering van een serie vervalt de feed. De serie en de bestanden blijven.
settings-downloads-sources-title = Zoekbronnen
settings-downloads-sources-helper = Waar zoeken heen gaat, op volgorde. Vindt een bron niets, dan de volgende.
settings-downloads-sources-empty = Nog geen bronnen ingesteld
settings-downloads-sources-connect = Koppel de zoekhulp om ze te tonen
settings-downloads-sources-empty-helper = Bronnen uit een bronlijst hieronder verschijnen hier.
settings-downloads-config-file = Instellingenbestand van de zoekhulp
settings-downloads-config-native-helper = Bevat de opslagmappen, de tags en de regels voor beste match, die AniBeam zelf volgt als het de hulp draait. De eigen bronnen zijn alleen van de hulp.
settings-downloads-config-helper = Bevat de bronnen, de opslagmappen, de tags en de regels voor beste match.
settings-downloads-unsaved = Niet-opgeslagen wijzigingen
settings-downloads-unsaved-helper = Opslaan herstart de zoekhulp met de nieuwe instellingen.
settings-downloads-discard = Weggooien

## SettingsDeveloperTab

settings-developer-title = Ontwikkelaar
settings-developer-lead = Waarom een pagina traag is, gemeten. Elke schakelaar geldt voor de volgende pagina die je opent.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds } s
settings-developer-switches = Schakelaars
settings-developer-load-images = Afbeeldingen laden
settings-developer-load-images-helper = Uit wordt geen poster, banner of portret gelezen of gedecodeerd.
settings-developer-overlay = Overlay paginaladen
settings-developer-overlay-helper = Het laatste paginaladen en het geheugen, rechtsonder in het venster.
settings-developer-read-cache = Bibliotheekleesacties bewaren
settings-developer-read-cache-helper = Het antwoord op een leesactie van de hele bibliotheek blijft bewaard tot de bibliotheek verandert. Uit vraagt elke pagina het de core opnieuw.
settings-developer-poster-pool = Posters vooraf laden
settings-developer-poster-pool-helper = Decodeert de 60 posters die je het waarschijnlijkst ziet en houdt ze in het geheugen.
settings-developer-options = Ontwikkelaarsopties
settings-developer-options-helper = Zet dit tabblad, de overlay en het meten uit.
settings-developer-memory = Geheugen
settings-developer-memory-helper = Wat de kernel voor dit proces telt. Afbeeldingen houden gedecodeerde pixels in het geheugen en op de grafische kaart.
settings-developer-memory-unknown = Dit systeem zegt niet hoeveel geheugen de app gebruikt.
settings-developer-in-use = In gebruik
settings-developer-peak = Piek
settings-developer-heap = Heap
settings-developer-files-mapped = Gemapte bestanden
settings-developer-threads = Threads
settings-developer-give-back = Geheugen teruggeven
settings-developer-give-back-helper = Laat los wat de scene graph opnieuw kan opbouwen, ruimt JavaScript-garbage op en geeft vrije heap terug aan het systeem. Wat het vrijmaakt is wat de app vasthield zonder het nodig te hebben.
settings-developer-released = { $before } naar { $after }
settings-developer-free-now = Nu vrijmaken
settings-developer-images-total = Afbeeldingen sinds de start
settings-developer-images-total-line = { $decodes } gedecodeerd, { $pixels } aan pixels uit { $disk } op schijf, { $ms } ms
settings-developer-loads = Paginaladingen
settings-developer-loads-helper = Gebouwd is het wachten na een klik: de Qt-thread doet niets anders tot de pagina bestaat. Klaar is wanneer de laatste afbeelding binnen was.
settings-developer-touring = Timen, { $at } van { $steps }
settings-developer-tour = Elke pagina timen
settings-developer-copy-report = Rapport kopiëren
settings-developer-col-page = Pagina
settings-developer-col-built = Gebouwd
settings-developer-col-painted = Getekend
settings-developer-col-settled = Klaar
settings-developer-col-calls = Calls, data
settings-developer-col-decodes = Decodes, tijd, pixels
settings-developer-col-items = Items
settings-developer-col-offscreen = Afbeeldingen buiten beeld
settings-developer-col-memory = Geheugen
settings-developer-load-no-images = { $page }, geen afbeeldingen
settings-developer-offscreen-of = { $offscreen } van { $total }
settings-developer-loads-empty = Open een pagina en hij verschijnt hier.
settings-developer-load-images-off = { $page }, afbeeldingen uit
settings-developer-load-frames = { $frames } frames getekend tijdens het laden, het langste gat tussen twee { $gap }.
settings-developer-load-frames-timed-out = { $frames } frames getekend tijdens het laden, het langste gat tussen twee { $gap }. Na 15 s gestopt met wachten op de afbeeldingen.
settings-developer-built = Gebouwd
settings-developer-settled = Klaar
settings-developer-core-data = Data van de core
settings-developer-pixels = Gedecodeerde pixels
settings-developer-page-items = Items op de pagina
settings-developer-cached-images = Afbeeldingen al in geheugen
settings-developer-calls = Core-calls, traagste eerst
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  cache
settings-developer-call = { $name }  { $ms } ms  { $size }  core { $core }, naar QML { $qml }
settings-developer-decodes = Traagste decodes
settings-developer-decodes-line = { $count } decodes na elkaar op de afbeeldingsthread, { $span } van de eerste tot de laatste; { $requests } gevraagd, { $cached } al gedecodeerd, { $blocked } geblokkeerd
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } naar { $w }x{ $h }  { $size }
settings-developer-decodes-none = Niets gedecodeerd: elke afbeelding zat al in het geheugen.
settings-developer-images-were-off = Afbeeldingen stonden uit.
settings-developer-scene = Waar de pagina uit bestaat
settings-developer-scene-line = { $items } items, { $offscreen } buiten het venster; { $texts } teksten, { $shapes } vormen, { $icons } iconen; { $images } afbeeldingen, waarvan { $imagesOffscreen } buiten het venster met samen { $size }
settings-developer-copy-load = Deze lading kopiëren
settings-developer-tour-title = Elke pagina, afbeeldingen aan en uit
settings-developer-tour-helper = Het middelste van drie bezoeken aan elke pagina. Waar de twee kolommen gelijk zijn, maken afbeeldingen de pagina niet traag.
settings-developer-col-built-on = Gebouwd, afbeeldingen aan
settings-developer-col-built-off = Gebouwd, afbeeldingen uit
settings-developer-col-settled-on = Klaar, aan
settings-developer-col-settled-off = Klaar, uit
settings-developer-col-decodes-time = Decodes, tijd
settings-developer-col-data = Data
settings-developer-tour-peak = Piekgeheugen tijdens de ronde { $size }
settings-developer-events = Gebeurtenissen
settings-developer-events-helper = Alles wat de core zegt, elk niveau, nieuwste eerst. Elke gebeurtenis gaat als JSON naar de Qt-thread; de grootte is wat het versturen kostte.
settings-developer-event-rate = { $kind } { $count }/min, { $size }
settings-developer-events-filter = Filter op woorden, soort of onderdeel
settings-developer-event-plain = { $message }   (gewoon: { $plain })
settings-developer-copy-events = Gebeurtenissen kopiëren

## SettingsKeysTab

settings-keys-title = Toetsenbord
settings-keys-lead = Elke toets waar AniBeam op reageert. Druk overal op ? voor de toetsen die werken waar je bent.
settings-keys-player = In de speler
settings-keys-player-around = Navigeren in de speler
settings-keys-around = Navigeren
settings-keys-anywhere = Overal

## KeyHelp

key-help-title = Sneltoetsen
key-help-group-playback = Afspelen
key-help-group-navigation = Navigatie
key-help-group-library = Bibliotheek
key-help-key-space = Spatie / K
key-help-key-left-right = Links / Rechts
key-help-key-ctrl-right = Ctrl+Rechts
key-help-key-backspace = Backspace
key-help-key-up-down = Omhoog / Omlaag
key-help-key-hold = Video ingedrukt houden
key-help-key-page = Page Up / Down
key-help-key-home-end = Home / End
key-help-key-escape = Esc
key-help-key-alt-left = Alt+Links
key-help-key-alt-right = Alt+Rechts
key-help-key-shift-f10 = Shift+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / of Ctrl+F
key-help-key-ctrl-space = Ctrl+Spatie
key-help-key-shift-space = Shift+Spatie
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = Afspelen of pauzeren
key-help-seek-5 = 5 s spoelen
key-help-seek-10 = 10 s spoelen
key-help-skip = Intro of outro overslaan, anders 90 s
key-help-frame-step = Eén frame terug of vooruit
key-help-speed = Langzamer of sneller, in stappen van 0,25
key-help-normal-speed = Normale snelheid
key-help-volume = Volume 5 omhoog of omlaag
key-help-mute = Dempen
key-help-subtitles-toggle = Ondertitels uit en weer aan
key-help-subtitle-delay = Ondertitelvertraging 100 ms eerder of later
key-help-audio-delay = Audiovertraging 100 ms eerder of later
key-help-hold-speed = 2× snelheid zolang ingedrukt
key-help-screenshot = Screenshot opslaan in Pictures
key-help-episode = Volgende of vorige aflevering
key-help-chapter = Vorig of volgend hoofdstuk
key-help-start-end = Het begin, of de outro
key-help-fullscreen = Volledig scherm
key-help-escape-player = Sluit wat open is, anders volledig scherm verlaten, anders de speler verlaten
key-help-toggle-list = Deze lijst tonen of verbergen
key-help-library-search = Bibliotheek en zoeken
key-help-search-page = Deze pagina doorzoeken
key-help-activity = Activiteit
key-help-settings = Instellingen
key-help-quit = Afsluiten
key-help-forward = Vooruit
key-help-escape-app = Sluit wat open is, anders terug
key-help-context-menu = Het rechtsklikmenu
key-help-pick-card = Kies een kaart, om op meerdere te werken
key-help-pick-range = Kies elke kaart tot en met deze
key-help-pick-all = Kies elke getoonde kaart

## SettingsAboutTab

settings-about-title = Over
settings-about-lead = Welke AniBeam dit is, en hoe het hoort van de volgende.
settings-about-update-failed = De update is niet afgerond. Probeer het opnieuw, of haal hem van de releasepagina.
settings-about-update-ready = { $version } staat klaar om te installeren.
settings-about-update-command = { $version } is uit. Werk bij met { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = { $version } is uit. Deze kopie is uit de broncode gebouwd: pull en bouw opnieuw.
settings-about-update-elsewhere = { $version } is uit, zonder download voor deze computer.
settings-about-version = Versie
settings-about-build = AniBeam { $build }
settings-about-whats-new = Wat is nieuw
settings-about-install = { $version } installeren
settings-about-setup = Introductie
settings-about-setup-helper = De rondleiding waarmee een nieuwe installatie opent: je map, je lijst en downloads.
settings-about-run-setup = Introductie opnieuw
settings-about-updates = Updates
settings-about-nightly-helper = Bijna elke dag een nieuwe build. Nieuwer, en minder getest.
settings-about-stable-helper = Een nieuwe versie zodra die getest en uitgebracht is.
settings-about-stable = Stabiel
settings-about-nightly = Nightly
settings-about-check-now = Nu controleren
settings-about-developer = Ontwikkelaarsopties
settings-about-developer-helper = Voegt Ontwikkelaar toe: timet elke pagina, toont geheugen en het technische log, kan afbeeldingen uitzetten.
settings-about-credits = Credits
settings-about-ratings = Beoordelingen
settings-about-ratings-helper = Information courtesy of IMDb (https://www.imdb.com). Used with permission.
settings-about-series = Series en afleveringen
settings-about-series-helper = AniList en TVmaze. Overslaantijden van AniSkip.

## SettingsPlaybackTab

settings-playback-title = Afspelen
settings-playback-lead = Wat de speler uit zichzelf doet, zodat je er alleen naar grijpt als je wilt.
settings-playback-skipping = Overslaan
settings-playback-skipping-helper = Gebruikt de hoofdstukken, anders tijden die kijkers online delen. Ongedaan maken in de speler stopt overslaan tot AniBeam sluit.
settings-playback-skip-intros = Intro's overslaan
settings-playback-skip-outros = Outro's overslaan
settings-playback-skip-outros-helper = De outro bereiken markeert de aflevering ook als gekeken.
settings-playback-next-episode = Volgende aflevering
settings-playback-name-next = Volgende aflevering bij naam
settings-playback-name-next-helper = De knop Volgende aan het eind van een aflevering toont de titel van de volgende. Uit toont hij alleen het nummer.
settings-playback-languages = Talen
settings-playback-languages-helper = Taalcodes, liefste eerst. De eerste die een bestand heeft, wint. Een in de speler gekozen spoor wordt per serie onthouden.
settings-playback-subtitle-languages = Ondertiteltalen
settings-playback-audio-languages = Audiotalen
settings-playback-discord = Toon op Discord wat je kijkt
settings-playback-discord-check-failed = AniBeam kon Discord nu niet controleren. Probeer het zo opnieuw.
settings-playback-discord-helper = De serie, aflevering en poster verschijnen op je Discord-profiel terwijl je afspeelt. Er wordt niets verstuurd als dit uit staat of Discord dicht is.
settings-playback-discord-unavailable = Deze build heeft geen Discord-applicatie.
settings-playback-discord-connected = Verbonden met Discord
settings-playback-discord-not-connected = Niet verbonden. AniBeam verbindt terwijl een aflevering speelt en Discord open is
settings-playback-advanced = Geavanceerd
settings-playback-my-mpv = Mijn mpv-instellingen gebruiken
settings-playback-my-mpv-helper = Leest je mpv.conf. Opties die alleen bij het starten werken worden overgeslagen, en scripts laden nooit.
settings-playback-no-mpv-conf = Geen mpv-instellingenbestand gevonden. Zet je opties in mpv.conf en zet dit dan aan.

## SliderRow

slider-row-spoken-unit = { $name }, in { $unit }
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Accent rood
swatches-accent-orange = Accent oranje
swatches-accent-yellow = Accent geel
swatches-accent-green = Accent groen
swatches-accent-cyan = Accent cyaan
swatches-accent-blue = Accent blauw

## LookPreview

look-preview-dark = Donker
look-preview-light = Licht

## SettingsAccountsTab

settings-accounts-title = Accounts
settings-accounts-lead = Koppel AniList of MyAnimeList en elke aflevering die je afkijkt wordt daar voor je gemarkeerd. Aantallen gaan alleen omhoog.
settings-accounts-syncing = Je lijsten syncen
settings-accounts-synced = Lijsten gesynct
settings-accounts-lists = Je animelijsten
settings-accounts-lists-helper = Zonder lijst onthoudt AniBeam nog steeds wat je op deze computer keek. Een lijst op AniList of MyAnimeList brengt het naar je telefoon en je profiel.
settings-accounts-progress = Voortgang
settings-accounts-main-list = Hoofdlijst
settings-accounts-main-list-helper = De kaarten tonen voortgang van deze. De andere krijgt nog steeds elke update.
settings-accounts-local-lists = Lijsten op deze computer
settings-accounts-local-lists-helper =
    { $count ->
        [one] Eén serie heeft
       *[other] { $count } series hebben
    } alleen hier een status of score. Verhuis ze naar { $tracker } om ze op je telefoon en profiel te zien.
settings-accounts-review-move = Bekijken en verhuizen
settings-accounts-nothing-to-move = Niets meer te verhuizen
settings-accounts-sync-now = Nu syncen
settings-accounts-sync-now-helper = Leest elke gekoppelde lijst opnieuw. AniBeam doet dit ook bij het starten.
settings-accounts-sync = Syncen
settings-accounts-move-on-hold = Naar In de wacht
settings-accounts-move-on-hold-helper = Een serie op Kijken die je een tijd niet hebt aangeraakt gaat op je lijsten naar In de wacht. Afspelen zet hem terug. Een serie die op de volgende aflevering wacht blijft staan.
settings-accounts-after = Na
settings-accounts-after-line = Dagen zonder een aflevering af te spelen
settings-accounts-days-unit = dagen

## SettingsPhoneTab

settings-phone-title = Telefoonapp
settings-phone-lead = AniBeam op je telefoon kan deze bibliotheek afspelen, tonen wat er loopt en je downloads volgen, waar je ook bent.
settings-phone-online = Online
settings-phone-offline = Offline
settings-phone-untrusted = Krijgt niets van deze computer
settings-phone-sharing = Delen
settings-phone-sharing-helper = Alleen je eigen apparaten op Tailscale kunnen erbij. Afleveringen die je op de telefoon kijkt houden hier hun plek en voortgang.
settings-phone-share = Delen via Tailscale
settings-phone-stream = Afleveringen streamen
settings-phone-stream-on = Je apparaten kunnen de afleveringen op deze computer afspelen en casten.
settings-phone-stream-off = Je apparaten zien de bibliotheek en wat er loopt, maar kunnen hier niet van afspelen.
settings-phone-host-service = Hostdienst
settings-phone-not-running = Draait niet
settings-phone-running = Draait
settings-phone-service-off-helper = Voer het commando eenmaal uit: je apparaten bereiken AniBeam na inloggen, zonder venster.
settings-phone-service-on-helper = Je apparaten bereiken AniBeam zonder open venster. Zolang dit venster open is, deelt het in plaats van de dienst.
settings-phone-copy-command = Commando kopiëren
settings-phone-command-copied = Commando gekopieerd
settings-phone-nobody = Krijgt niets: Tailscale noemt er geen persoon bij
settings-phone-people = Personen
settings-phone-own-profile = Profiel van deze computer
settings-phone-person-own-devices =
    { $count ->
        [0] Deze computer
        [one] Deze computer en { $count } apparaat
       *[other] Deze computer en { $count } apparaten
    }
settings-phone-person-devices =
    { $count ->
        [one] { $count } apparaat
       *[other] { $count } apparaten
    }
settings-phone-devices = Apparaten op je tailnet
settings-phone-devices-helper = Elk apparaat op je tailnet kan deze computer bereiken zolang delen aan staat. Zet er een uit en hij krijgt niets meer van hier.
settings-phone-addresses = Adressen
settings-phone-addresses-helper = Typ er een in de telefoonapp de eerste keer dat hij erom vraagt.
settings-phone-copy = Kopieer
settings-phone-address-copied = Adres gekopieerd

## SettingsHelpTab

settings-help-title = Help
settings-help-lead = Korte antwoorden op veelgestelde vragen, elk direct naar de oplossing.
settings-help-folders = Hoe moeten mijn mappen eruitzien?
settings-help-folders-helper = Eén map per serie, met de afleveringen erin. Geef elk bestand het afleveringsnummer, zoals releases: "Frieren - 01.mkv" of "Frieren S01E01.mkv". Een tweede seizoen mag een eigen map zijn of een map "Season 2" in de serie. Films gaan in een map Movies, één bestand per film.
settings-help-for-example = Bijvoorbeeld
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = Nog geen map toegevoegd
settings-help-reads-folders =
    { $count ->
        [one] AniBeam leest één map
       *[other] AniBeam leest { $count } mappen
    }
settings-help-add-folder-helper = Voeg de map toe die je series bevat, niet elke serie apart.
settings-help-add-folder = Map toevoegen
settings-help-open-folders = Je mappen openen
settings-help-match = Waarom is mijn serie niet herkend?
settings-help-match-helper = AniBeam zoekt elke map op naam op, eerst op AniList, dan op TVmaze. Extra woorden, een typfout of een naam in een andere taal kunnen missen. Kies de juiste serie met de hand bij Serie-info; een keuze die jij maakt wordt nooit vervangen.
settings-help-all-recognised = Elke serie is herkend
settings-help-unrecognised =
    { $count ->
        [one] Eén serie is niet herkend
       *[other] { $count } series zijn niet herkend
    }
settings-help-open-show-info = Serie-info openen
settings-help-connect = Hoe koppel ik AniList?
settings-help-connect-helper = Koppel het en elke aflevering die je afkijkt wordt op je lijst gemarkeerd, zodat je telefoon en je profiel bijblijven. AniList vraagt je één keer een kleine eigen app te maken; AniBeam leidt je erdoor en het kost ongeveer een minuut. MyAnimeList werkt hetzelfde.
settings-help-both-connected = Beide zijn gekoppeld
settings-help-anilist-connected = AniList is gekoppeld
settings-help-mal-connected = MyAnimeList is gekoppeld
settings-help-none-connected = Nog geen van beide gekoppeld
settings-help-connect-anilist = AniList koppelen
settings-help-connect-mal = MyAnimeList koppelen
settings-help-downloads = Waarom is Downloads leeg?
settings-help-downloads-helper = AniBeam heeft geen plekken om te zoeken. Voeg een bronlijst toe in Instellingen, onder Downloads, en installeer de bronnen die je wilt. Downloaden heeft ook een torrentclient nodig: AniBeam kan een eigen draaien, of een qBittorrent gebruiken die je al hebt.
settings-help-sources = Bronnen en de torrentclient
settings-help-open-downloads = Downloads-instellingen openen
settings-help-data = Waar staan mijn gegevens?
settings-help-data-helper = Je bibliotheek, kijkgeschiedenis en instellingen blijven op deze computer. Logins voor AniList en MyAnimeList gaan naar de wachtwoordopslag van het systeem, of anders een bestand alleen voor jou. Alles past in één back-up via Opslag en back-up.
settings-help-library-history = Bibliotheek en geschiedenis
settings-help-show-folder = Map tonen
settings-help-backup = Back-up maken of herstellen
settings-help-open-storage = Opslag en back-up openen
settings-help-more = Iets anders?
settings-help-more-helper = Doorloop de introductie opnieuw of meld wat er misging. Een screenshot helpt veel.
settings-help-first-setup = De eerste introductie
settings-help-first-setup-helper = Je map, je lijst en downloads, stap voor stap.
settings-help-run-setup = Introductie opnieuw
settings-help-report = Probleem melden
settings-help-report-helper = Opent een bugmelding op GitHub in je browser, met deze versie al ingevuld.

## SettingsSubtitlesTab

settings-subtitles-title = Ondertitels
settings-subtitles-lead = Waarmee elke aflevering begint. Wijzigingen zie je meteen op het beeld hieronder.
settings-subtitles-reset-done = Ondertitels zien er weer uit zoals eerst
settings-subtitles-size-place = Grootte en plek
settings-subtitles-size = Grootte
settings-subtitles-position = Positie
settings-subtitles-position-helper = 100 staat op de onderrand. Lagere getallen tillen de regels hoger in beeld.
settings-subtitles-style = Stijl
settings-subtitles-styled = Opgemaakte ondertitels
settings-subtitles-as-made-helper = Opgemaakte (ASS) bestanden zien er precies uit zoals de makers ze instelden.
settings-subtitles-mine-helper = Jouw stijl hieronder gaat op elk bestand. Beeldtekst en karaoke kunnen breken.
settings-subtitles-size-only-helper = Opgemaakte ondertitels houden hun uiterlijk, met jouw grootte. Gewone krijgen je stijl.
settings-subtitles-as-made = Origineel
settings-subtitles-size-only = Alleen grootte
settings-subtitles-mine = Eigen
settings-subtitles-font = Lettertype
settings-subtitles-font-invalid = Typ een lettertypenaam, zoals Noto Sans.
settings-subtitles-bold = Vet
settings-subtitles-text-colour = Tekstkleur
settings-subtitles-colour-invalid = Typ een kleur zoals #ffcc00.
settings-subtitles-outline = Omlijning
settings-subtitles-outline-invalid = De breedte is een getal, 0 of meer.
settings-subtitles-outline-colour-invalid = Typ de kleur zoals #000000.
settings-subtitles-outline-helper = De breedte, dan de kleur.
settings-subtitles-shadow = Schaduw
settings-subtitles-shadow-invalid = Een getal, 0 of meer. 0 betekent geen schaduw.
settings-subtitles-shadow-helper = Hoe ver de schaduw valt, 0 voor geen.
settings-subtitles-box = Achtergrondvak
settings-subtitles-box-helper = Een donker vak achter de regels. 0 laat het weg.
settings-subtitles-start-over = Opnieuw beginnen
settings-subtitles-start-over-helper = Zet grootte, lettertype, kleuren, omlijning, vak en positie terug zoals ze waren.
settings-subtitles-reset = Herstellen

## SettingsDataTab

settings-data-title = Opslag en back-up
settings-data-lead = Zet je bibliotheek op een andere pc, bewaar een kopie of maak afbeeldingsruimte vrij.
settings-data-backed-up = Back-up gemaakt in { $path }
settings-data-restore-finished = Herstel klaar
settings-data-images-removed =
    { $count ->
        [one] { $count } afbeelding weggehaald
       *[other] { $count } afbeeldingen weggehaald
    }
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] { $count } map toegevoegd
       *[other] { $count } mappen toegevoegd
    }
settings-data-summary-series = { $count } series aangemaakt
settings-data-summary-matches =
    { $count ->
        [one] { $count } koppeling toegepast
       *[other] { $count } koppelingen toegepast
    }
settings-data-summary-views =
    { $views ->
        [one] { $views } weergave
       *[other] { $views } weergaven
    } en { $episodes ->
        [one] { $episodes } afgekeken aflevering
       *[other] { $episodes } afgekeken afleveringen
    } samengevoegd
settings-data-summary-resume-points =
    { $count ->
        [one] { $count } hervatpunt samengevoegd
       *[other] { $count } hervatpunten samengevoegd
    }
settings-data-summary-accounts =
    { $count ->
        [one] { $count } lijstaccount overgenomen
       *[other] { $count } lijstaccounts overgenomen
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } bestand neemt een correctie van jou over.
       *[other] { $count } bestanden nemen een correctie van jou over.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } map is niet gevonden op deze computer.
       *[other] { $count } mappen zijn niet gevonden op deze computer.
    }
settings-data-summary-left-out = Weggelaten: { $fields }.
settings-data-backup = Back-up
settings-data-back-up-library = Back-up van de bibliotheek
settings-data-back-up-helper = Eén bestand met mappen, koppelingen en wat je zag.
settings-data-last-written = Laatst geschreven naar { $path }
settings-data-open-folder = Map openen
settings-data-back-up = Back-up maken
settings-data-private = Privégegevens meenemen
settings-data-private-helper = Bevat ook logins voor AniList en MyAnimeList, kijkgeschiedenis en voorkeuren. Logins staan erin als platte tekst: deel het niet.
settings-data-restore-from = Herstellen uit een back-up
settings-data-restore-helper = Leest een back-up in deze bibliotheek. Wat op schijf staat wint; het bestand vult de rest aan.
settings-data-restore = Herstellen
settings-data-image-cache = Afbeeldingscache
settings-data-image-cache-helper = Posters, banners en portretten staan op schijf voor offline gebruik. Gewiste afbeeldingen komen terug zodra een pagina ze vraagt.
settings-data-cache-unmeasured = De afbeeldingscache kon niet worden gemeten
settings-data-cache-size =
    { $count ->
        [one] { $size } in { $count } afbeelding
       *[other] { $size } in { $count } afbeeldingen
    }
settings-data-cache-empty = Geen afbeeldingen in cache
settings-data-cache-unmeasured-helper = Probeer het zo opnieuw. De afbeeldingen zelf zijn in orde.
settings-data-remove-images =
    { $count ->
        [one] { $count } afbeelding weghalen?
       *[other] { $count } afbeeldingen weghalen?
    }
settings-data-files = Waar AniBeam zijn bestanden bewaart
settings-data-files-database = Bibliotheekdatabase
settings-data-files-data = Gegevens
settings-data-files-settings = Instellingen
settings-data-files-cache = Cache
settings-data-backup-filter = AniBeam-back-up (*.json)
settings-data-all-files-filter = Alle bestanden (*)
settings-data-backing-up = Back-up maken
settings-data-restoring = Herstellen

## SettingsLibraryTab

settings-library-title = Bibliotheek
settings-library-lead = De mappen die AniBeam bewaakt, en hoe het uitzoekt wat erin staat.
settings-library-not-scanned = Nog niet gescand
settings-library-scan-nothing-changed = Niets veranderd
settings-library-scan-added = { $count } toegevoegd
settings-library-scan-changed = { $count } gewijzigd
settings-library-scan-removed = { $count } weggehaald
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = Anime: series, films en hoe ze samenhangen. Wat je kijkt wordt gemarkeerd op je trackers.
settings-library-tvmaze-about = Andere televisie: live action, tekenfilms, documentaires. Er wordt niets op een tracker gemarkeerd.
settings-library-tried-first = Als eerste geprobeerd
settings-library-tried-second = Als tweede geprobeerd
settings-library-tried-third = Als derde geprobeerd
settings-library-tried-fourth = Als vierde geprobeerd
settings-library-tried-nth = Als { $number }e geprobeerd
settings-library-folders = Mappen
settings-library-folders-helper = Elke map in een van deze is een serie. Een bestand bovenin een map Movies is een film.
settings-library-rescan-started = Opnieuw scannen gestart
settings-library-load-failed = Je mappen konden niet worden gelezen
settings-library-load-failed-helper = AniBeam kon zijn lijst met mappen niet lezen. Probeer het zo opnieuw.
settings-library-no-folders = Nog geen mappen
settings-library-no-folders-helper = Wijs AniBeam de map waar je anime staat. Het vindt de series en films zelf.
settings-library-scanned = Gescand { $when }
settings-library-add-a-folder = Map toevoegen
settings-library-scan-all = Alles scannen
settings-library-scan-started = Scan gestart
settings-library-add-folder = Map toevoegen
settings-library-matching = Koppelen
settings-library-matching-helper = Een nieuwe map wordt in deze volgorde opgezocht; de eerste goede koppeling telt. Een handmatige blijft altijd.
settings-library-try-later = Later proberen
settings-library-try-first = Eerst proberen
settings-library-check-matches = Koppelingen controleren
settings-library-check-matches-helper = De pagina Serie-info toont elke serie met de info die AniBeam vond, en corrigeert een verkeerde.
settings-library-open-show-info = Serie-info openen
settings-library-hidden-shows = Verborgen series
settings-library-show-hidden = Verborgen series tonen
settings-library-show-hidden-helper = Brengt de series die je verborg terug op elke pagina tot AniBeam sluit.
settings-library-hidden-from-airing = Verborgen in Nu uitgezonden
settings-library-hidden-from-airing-helper = Series die je als niet interessant markeerde. Elke serie blijft weg uit Nu uitgezonden tot je hem terughaalt.
settings-library-hidden-from-airing-empty = Nog niets. Series die je in Nu uitgezonden als niet interessant markeert komen hier, zodat ze terug kunnen.
settings-library-show-again = Weer tonen
settings-library-calendar = Agenda
settings-library-calendar-feed = Uitzendtijden in een agenda
settings-library-calendar-feed-helper = Houdt een agendabestand bij met de komende afleveringen van de series op je lijsten, bijgewerkt als het schema verandert.
settings-library-calendar-file = Agendabestand
settings-library-calendar-file-helper = Abonneer je op dit bestand in je agenda-app.
settings-library-copy = Kopieer
settings-library-path-copied = Pad gekopieerd
settings-library-open-folder = Map openen
settings-library-notifications = Meldingen
settings-library-new-episodes = Nieuwe afleveringen
settings-library-new-episodes-helper = Een bureaubladmelding als een aflevering uitkomt van een serie op je Kijken-lijst. Klik erop om de serie te openen.
settings-library-folder-added = Map toegevoegd, scannen

## SettingsAppearanceTab

settings-appearance-language = Taal
settings-appearance-language-system = Zoals het systeem ({ $language })
settings-appearance-text-smaller = Kleiner
settings-appearance-text-normal = Normaal
settings-appearance-text-large = Groot
settings-appearance-text-larger = Groter
settings-appearance-text-largest = Grootst
settings-appearance-title = Weergave
settings-appearance-lead = Wijzigingen zie je meteen, hier en op elke pagina. Ze staan in ~/.config/anibeam/theme.toml, dat je ook met de hand kunt bewerken.
settings-appearance-colours = Kleuren
settings-appearance-mode = Modus
settings-appearance-mode-dark = Donker
settings-appearance-mode-light = Licht
settings-appearance-mode-system = Systeem
settings-appearance-colours-from = Kleuren van
settings-appearance-from-theme-helper = Het themapaar hieronder, voor donker en voor licht.
settings-appearance-from-desktop-helper = De kleuren van je terminal, anders het kleurenschema en accent van het bureaublad.
settings-appearance-from-desktop = Bureaublad
settings-appearance-from-theme = Thema
settings-appearance-dark-theme = Donker thema
settings-appearance-light-theme = Licht thema
settings-appearance-themes-helper = Themabestanden die je in ~/.config/anibeam/themes zet verschijnen in beide lijsten.
settings-appearance-accent = Accent
settings-appearance-accent-helper = Markeert wat geselecteerd is, afspeelt of nieuw is.
settings-appearance-icon = Icoon
settings-appearance-app-icon = App-icoon
settings-appearance-app-icon-helper = Dat van het venster, de taakbalk en de launcher. Donker wisselt het geel en het zwart om.
settings-appearance-icon-default = Standaard
settings-appearance-icon-default-name = Standaard app-icoon
settings-appearance-icon-dark = Donker
settings-appearance-icon-dark-name = Donker app-icoon
settings-appearance-size-motion = Grootte en beweging
settings-appearance-text-size = Tekstgrootte
settings-appearance-text-size-helper = Hoe groot de tekst overal in AniBeam is, bovenop de grootte van je systeem.
settings-appearance-density = Dichtheid
settings-appearance-density-helper = De ruimte rond alles, van krap tot luchtig.
settings-appearance-density-compact = Compact
settings-appearance-density-normal = Normaal
settings-appearance-density-comfortable = Ruim
settings-appearance-poster-size = Postergrootte
settings-appearance-poster-size-helper = Postergrootte in de bibliotheek en op Start.
settings-appearance-poster-small = Klein
settings-appearance-poster-medium = Middel
settings-appearance-poster-large = Groot
settings-appearance-out-now = Nu uit in Uitzending
settings-appearance-out-now-helper = Kaarten met de banner van elke serie, of een lijst met kleine posters en titels, drie rijen diep.
settings-appearance-out-now-cards = Kaarten
settings-appearance-out-now-list = Lijst
settings-appearance-out-now-grid = Nu uit-kaarten
settings-appearance-out-now-grid-helper = Hoeveel kaarten er staan, kolommen bij rijen, voordat Alle de rest opent.
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Minder beweging
settings-appearance-reduce-motion-helper = Slaat de bibliotheekanimatie bij openen en andere franje over.

## LookPane

look-pane-sample-series = Voorbeeldserie
look-pane-episode-code = Afl. { $number }
look-pane-library = Bibliotheek
look-pane-series-count =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    }
look-pane-search = Zoek titels of mappen
look-pane-search-hint = / of Ctrl+K
look-pane-all = Alles
look-pane-series = Series
look-pane-movies = Films
look-pane-plain = Gewoon
look-pane-selected = Geselecteerd
look-pane-errors =
    { $count ->
        [one] { $count } fout
       *[other] { $count } fouten
    }
look-pane-type-large = Groot
look-pane-type-normal = Normaal
look-pane-type-small = Klein, gedimd
look-pane-sample-switch = Voorbeeldschakelaar
look-pane-button = Knop
look-pane-scan = Scannen
look-pane-sample-status =
    { $count ->
        [one] { $count } nieuwe afl. in { $title }
       *[other] { $count } nieuwe afl. in { $title }
    }

## FeedPage

feed-title = Feed
feed-aired = { $episode } uitgezonden { $when }
feed-downloaded = Bestand gedownload { $when }
feed-countdown-days = { $days }d { $hours }u
feed-countdown-hours = { $hours }u { $minutes }m
feed-countdown-minutes = { $minutes }m
feed-section-this-week = Deze week
feed-new-episodes =
    { $count ->
        [one] { $count } nieuwe aflevering
       *[other] { $count } nieuwe afleveringen
    }
feed-section-earlier = Eerder
feed-latest-out = Laatst uit { $when }
feed-section-coming-soon = Binnenkort
feed-this-week-count = { $count } deze week
feed-section-everything-else = Al het andere
feed-out-this-week = { $count } uit deze week
feed-nothing-scheduled = Nog niets gepland.
feed-next = Volgende: { $title } over { $countdown }
feed-nothing-new = Niets nieuws deze week.
feed-new-this-week =
    { $count ->
        [one] { $count } nieuwe aflevering deze week.
       *[other] { $count } nieuwe afleveringen deze week.
    }
feed-sort-recent = Recent uitgekomen
feed-sort-upcoming = Binnenkort
feed-failed-title = De feed is niet geladen
feed-empty-title = Hier staat nog niets
feed-failed-body = AniBeam kon de feed nu niet lezen. Probeer het zo opnieuw.
feed-empty-body-folders = De feed vult zich als afleveringen uitkomen en binnenkomen. Nu uitgezonden toont wat er dit seizoen loopt.
feed-empty-body = Voeg de map toe waar je anime staat en de feed vult zich met de afleveringen.
feed-add-folder = Map toevoegen
feed-airing-now = Nu uitgezonden

## RecommendedPage

recommended-title = Aanbevolen voor jou
recommended-ranked-minutes = { $minutes } min geleden gerangschikt
recommended-ranked-hours = { $hours } u geleden gerangschikt
recommended-ranked-days = { $days } d geleden gerangschikt
recommended-scope-name = Aanbevolen uit
recommended-scope-all = Alles
recommended-scope-library = In bibliotheek
recommended-scope-list = Op lijst
recommended-ranking = Rangschikken
recommended-rank-again = Opnieuw rangschikken
recommended-no-scores-title = Geef eerst een paar series een score
recommended-no-list-title = Nog niets om van te leren
recommended-never-fitted-title = Je series rangschikken
recommended-nothing-left-title = Hier is niets meer
recommended-no-scores-body = Aanbevelingen leren van de scores op je lijst. Geef een paar series een score op AniList en ze verschijnen hier.
recommended-no-list-body = Koppel AniList in Instellingen of houd een lijst bij op dit apparaat; je scores sturen deze pagina.
recommended-never-fitted-body = Het eerste rangschikken leest AniList een minuut of twee. Het loopt op de achtergrond.
recommended-nothing-left-library-body = Alles in je bibliotheek dat past is gekeken. Alles vindt series die je nog niet hebt.
recommended-nothing-left-body = Alles op je lijsten dat past is gekeken. Alles vindt series die niet op je lijsten staan.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] 1 serie naar { $site } verhuizen
       *[other] { $count } series naar { $site } verhuizen
    }
move-lists-score = score { $score }
move-lists-title = Verhuis je lijst naar { $site }
move-lists-body =
    { $count ->
        [one] Eén serie heeft
       *[other] { $count } series hebben
    } alleen op deze computer een status of score. Verhuizen zet ze op je { $site }-lijst, waar je telefoon en je profiel ze zien. Voor deze series wordt vervangen wat { $site } nu heeft. Er wordt niets verstuurd tot je op Verhuizen drukt.
move-lists-head-show = Serie
move-lists-head-here = Op deze computer
move-lists-head-tracker = Nu op { $site }
move-lists-nothing = Niets
move-lists-not-on-list = Niet op je lijst
move-lists-later-hint = Later: Instellingen, Accounts
move-lists-not-now = Niet nu
move-lists-move =
    { $count ->
        [one] 1 serie verhuizen
       *[other] { $count } series verhuizen
    }

## StatsPage

stats-title = Stats
stats-month-jan = jan
stats-month-feb = feb
stats-month-mar = mrt
stats-month-apr = apr
stats-month-may = mei
stats-month-jun = jun
stats-month-jul = jul
stats-month-aug = aug
stats-month-sep = sep
stats-month-oct = okt
stats-month-nov = nov
stats-month-dec = dec
stats-nothing-yet = Nog niets
stats-ranked-episodes =
    { $count ->
        [one] { $count } afl.
       *[other] { $count } afl.
    }
stats-year-summary = Wat je keek in { $year }
stats-empty-title = Nog niets gekeken
stats-empty-body = Kijk een aflevering af en hij telt hier mee: uren, genres en de seizoenen die je keek.
stats-open-library = Bibliotheek
stats-hours-watched = uur gekeken
stats-episodes-label =
    { $count ->
        [one] aflevering
       *[other] afleveringen
    }
stats-series-finished = series afgerond
stats-none = Geen
stats-busiest-month = drukste maand
stats-by-month = Per maand
stats-month-hours = { $hours }u
stats-top-genres = Topgenres
stats-top-studios = Topstudio's
stats-seasons-watched = Gekeken seizoenen
stats-recap-title = Mijn animejaar
stats-recap-hours = uur
stats-recap-episodes = afleveringen
stats-recap-top-genre = topgenre
stats-save-image = Opslaan als afbeelding
stats-save-dialog-title = Terugblik opslaan
stats-save-filter-png = PNG-afbeelding (*.png)
stats-recap-saved = Terugblik opgeslagen
stats-recap-save-failed = Kon de terugblik niet opslaan

## TrackerRow

tracker-row-connected = Gekoppeld als { $user }, gesynct { $when }
tracker-row-unknown-user = ?
tracker-row-never = nooit
tracker-row-waiting = Wacht tot je goedkeurt in de browser
tracker-row-not-connected = Niet gekoppeld
tracker-row-open-again = Opnieuw openen
tracker-row-open-again-tip = Open de inlogpagina opnieuw
tracker-row-disconnect = Ontkoppelen
tracker-row-disconnect-question = { $name } ontkoppelen? Je login wordt weggehaald.
tracker-row-secret-saved = Er is een client secret voor deze app opgeslagen.
tracker-row-remove-secret = Weghalen
tracker-row-secret-removed = Client secret weggehaald
tracker-row-add-secret = Client secret toevoegen
tracker-row-secret-where = Het is de lange tekenreeks onder "Client Secret" op de pagina van de app op myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Client secret
tracker-row-save-connect = Opslaan en koppelen

## RemoveSheet

remove-unsubscribe-delete = Opzeggen en wissen
remove-unsubscribe-torrents = Opzeggen en torrents weghalen
remove-unsubscribe = Opzeggen
remove-delete-files = Weghalen en bestanden wissen
remove-torrents =
    { $count ->
        [one] De torrent weghalen
       *[other] Torrents weghalen
    }
remove-also-torrents =
    { $count ->
        [one] Ook de torrent weghalen
       *[other] Ook de { $count } torrents weghalen
    }
remove-torrents-go-with-files = Met de bestanden gaan ook de torrents weg.
remove-torrents-files-stay = Ze verdwijnen uit de client; de bestanden blijven.
remove-also-files = Ook de bestanden wissen
remove-files-default-where = wat de torrents downloadden
remove-files-detail-rule = Verplaatst { $where } naar de prullenbak en haalt de torrents weg.
remove-files-detail = Verplaatst { $where } naar de prullenbak.

## NotesSheet

notes-your-tracker = je tracker
notes-title = Notities
notes-placeholder = Waar je bleef, wie het aanraadde, welke release je moet halen
notes-over-limit = { $count } tekens boven de limiet
notes-saved-to = Opgeslagen bij { $sites }

## RecommendationCard

recommendation-card-available = Beschikbaar
recommendation-card-open-library = { $title } openen in je bibliotheek
recommendation-card-open-anilist = { $title } openen op AniList

## SearchField

search-field-placeholder = Zoek titels of mappen
search-field-hint = / of Ctrl+K
search-field-clear = Zoekopdracht wissen

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent }%
recommended-hover-hidden-tags =
    { $count ->
        [one] Een verborgen tag die je geweldig vindt
       *[other] { $count } verborgen tags die je geweldig vindt
    }
recommended-hover-because = Omdat je keek

## PersonCard

person-card-unknown = Onbekend
person-card-role-main = Hoofdrol
person-card-role-supporting = Bijrol
person-card-role-background = Achtergrond
person-card-the-character = het personage
person-card-open = { $name } openen op AniList

## StatusDot

status-dot-on-list = Op je lijst: { $status }

## TrackerNudge

tracker-nudge-title = Koppel je animelijst
tracker-nudge-body = AniBeam markeert afleveringen op AniList of MyAnimeList terwijl je kijkt.
tracker-nudge-connect-anilist = AniList koppelen
tracker-nudge-connect-mal = MyAnimeList koppelen
tracker-nudge-not-now = Niet nu

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } procent voor jou
recommended-card-forecast = { $percent }%
recommended-card-in-library = In bibliotheek

## InlineConfirm

inline-confirm-keep = Houden

## Tokens

tokens-cube-watched-tracked = Gekeken, online bijgehouden
    .inline = gekeken, online bijgehouden
tokens-cube-watched-here = Hier gekeken, nog niet bijgehouden
    .inline = hier gekeken, nog niet bijgehouden
tokens-cube-on-disk = Op schijf
    .inline = op schijf
tokens-cube-downloading = Downloaden
    .inline = wordt gedownload
tokens-cube-missing = Ontbreekt
    .inline = ontbreekt
tokens-cube-error = Fout
    .inline = fout
tokens-cube-not-downloaded = Niet gedownload
    .inline = niet gedownload
tokens-cube-not-aired = Nog niet uitgezonden
    .inline = nog niet uitgezonden
tokens-cube-available = Beschikbaar
    .inline = beschikbaar
tokens-cube-picked = Gekozen om op te halen
    .inline = gekozen om op te halen
tokens-cube-already-on-disk = Al op schijf
    .inline = al op schijf
tokens-cube-no-release = Geen release
    .inline = geen release

## Card

card-not-interested = Niet interessant

## PagePlaceholder

page-placeholder-coming-soon = Binnenkort

## ErrorLine

error-line-hide-details = Details verbergen
error-line-details = Details
error-line-details-tip = Toon letterlijk wat er misging, en kopieer het
error-line-copied = Gekopieerd.

## Weergaven (apps/linux/src/format.rs): tijden, aftellingen en duur

fmt-just-now = zojuist
fmt-now = nu
fmt-ago = { $span } geleden
fmt-in = over { $span }
fmt-seconds-short = { $n }s
fmt-minutes-short = { $n }m
fmt-hours-short = { $n }u
fmt-days-short = { $n }d
fmt-months-short = { $n }mnd
fmt-years-short = { $n }j
fmt-years-months-short = { $years }j { $months }mnd
fmt-countdown-days = { $d }d { $h }u { $m }m
fmt-countdown-hours = { $h }u { $m }m
fmt-countdown-days-seconds = { $d }d { $h }u { $m }m { $s }s
fmt-countdown-hours-seconds = { $h }u { $m }m { $s }s
fmt-countdown-minutes-seconds = { $m }m { $s }s
fmt-runtime-minutes = { $m } min
fmt-runtime-hours = { $h } u
fmt-runtime-hours-minutes = { $h } u { $m } min
fmt-episode = Aflevering
fmt-size-b = { $size } B
fmt-size-kb = { $size } KB
fmt-size-mb = { $size } MB
fmt-size-gb = { $size } GB
fmt-size-tb = { $size } TB
fmt-episode-span = Afleveringen { $first } t/m { $through }

## apps/linux/src/problem.rs: elke fout in gewone woorden, wat er gebeurde en wat je kunt doen

problem-report = Probeer het opnieuw. Blijft het gebeuren, meld het dan vanuit het activiteitenlog.
problem-try-later = Probeer het later opnieuw.
problem-helper-unreachable = De zoekhulp antwoordde niet.
problem-helper-unauthorized = De zoekhulp weigerde de login.
problem-helper-not-running = De zoekhulp draait niet.
problem-helper-missing = De zoekhulp is niet geïnstalleerd.
problem-helper-failed = De zoekhulp kon dat niet doen.
problem-helper-no-login = De zoekhulp kan niet inloggen bij de torrentclient.
problem-helper-no-login-next = Sla het wachtwoord van de torrentclient op in Instellingen, onder Downloads.
problem-torrent-unreachable = De torrentclient antwoordde niet.
problem-torrent-unauthorized = De torrentclient weigerde de login.
problem-torrent-not-running = De torrentclient draait niet.
problem-torrent-missing = De torrentclient is niet geïnstalleerd.
problem-torrent-failed = De torrentclient kon dat niet doen.
problem-downloads-unreachable-next = Controleer of hij draait en of het adres klopt in Instellingen, onder Downloads.
problem-downloads-unauthorized-next = Controleer de gebruikersnaam en het wachtwoord in Instellingen, onder Downloads.
problem-downloads-not-running-next = Start hem vanuit Instellingen, onder Downloads.
problem-downloads-missing-next = Instellingen, onder Downloads, zegt hoe je hem instelt.
problem-trash = { -app-name } kon de bestanden niet naar de prullenbak verplaatsen.
problem-trash-next = Controleer of de bestanden van jou zijn en de schijf niet alleen-lezen is.
problem-source-list = Die bronlijst werkte niet.
problem-source-list-next = Controleer het adres in Instellingen, onder Downloads, Bronlijsten.
problem-remote-unreachable = { $name } antwoordde niet.
problem-remote-unreachable-next = De site is misschien offline. Probeer het later opnieuw, of doorzoek een andere bron.
problem-remote-unauthorized = { $name } weigerde de sleutel die het kreeg.
problem-remote-unauthorized-next = Wijzig de sleutel in Instellingen, onder Downloads, Bronlijsten.
problem-remote-failed = { $name } kon dat niet doen.
problem-remote-failed-next = Probeer het later opnieuw, of doorzoek een andere bron.
problem-provider-sign-in = { $name } weigerde de login van { -app-name }.
problem-provider-sign-in-next = Koppel het opnieuw in Instellingen, onder Accounts.
problem-provider-turned-away = { $name } wees { -app-name } af.
problem-provider-busy = { $name } heeft het nu druk.
problem-provider-busy-next = Wacht een minuut en probeer het dan opnieuw.
problem-provider-no-answer = { $name } antwoordde niet.
problem-provider-no-answer-next = Controleer je internetverbinding en probeer het opnieuw.
problem-provider-trouble = { $name } heeft nu problemen.
problem-provider-trouble-next = Probeer het over een paar minuten opnieuw.
problem-provider-rejected = { $name } accepteerde dat niet.
problem-not-found = { -app-name } kon dat niet meer vinden.
problem-not-found-next = Het is misschien weggehaald. Probeer het opnieuw om de pagina vers te laden.
problem-invalid-next = Controleer het en probeer het opnieuw.
problem-unavailable = Die map is nu niet bereikbaar.
problem-unavailable-next = Sluit de schijf aan of controleer de map in Instellingen, onder Bibliotheek.
problem-not-connected = { $tracker } is niet gekoppeld.
problem-not-connected-next = Koppel het in Instellingen, onder Accounts.
problem-io = { -app-name } kon een bestand niet lezen of schrijven.
problem-io-next = Controleer of de schijf ruimte heeft en de map van jou is, en probeer het opnieuw.
problem-storage = { -app-name } kon je bibliotheek niet opslaan.
problem-storage-next = Controleer of de schijf ruimte heeft en probeer het opnieuw.
problem-keyring = De wachtwoordopslag van het systeem antwoordde niet.
problem-keyring-next = Ontgrendel de sleutelbos van je bureaublad en probeer het opnieuw.
problem-unsupported = { -app-name } kan dat hier nog niet.
problem-version = Deze back-up komt van een nieuwere { -app-name }.
problem-version-next = Werk { -app-name } bij in Instellingen, onder Over, en importeer opnieuw.
problem-internal = Er ging iets mis binnen { -app-name }.
problem-refused-hidden = Die serie is verborgen.
problem-refused-hidden-next = Maak hem eerst zichtbaar vanaf zijn pagina.
problem-refused-no-match = Die serie is nog niet gekoppeld.
problem-refused-no-match-next = Koppel hem eerst bij Serie-info.
problem-refused-not-newer = Je lijst is al voorbij deze aflevering.
problem-refused-extra = Dat bestand is een extra, geen aflevering.
problem-refused-unmatched = Dat bestand hoort niet bij een aflevering.
problem-refused-unmatched-next = Corrigeer het vanaf de pagina van de serie, onder Correcties.
problem-refused-on-disk = Dat bestand staat alleen op deze computer.

## apps/linux/src/library_view.rs: de sectietitels van de bibliotheek en het feit onder elke titel

library-view-airing = Loopt
library-view-no-date = Geen datum
library-view-type-tv = TV
library-view-type-movies = Films
library-view-type-extras = OVA, ONA en specials
library-view-type-music = Muziek
library-view-type-unknown = Onbekend
library-view-no-genre = Ongekoppeld
library-view-state-caught-up = Bij
library-view-state-not-started = Niet begonnen
library-view-next-episode = Volgende aflevering over { $span }
library-view-countdown-days = { $d }d { $h }u
library-view-countdown-hours = { $h }u { $m }m
library-view-countdown-minutes = { $m }m
library-view-in-progress = { $count } bezig
library-view-not-started = { $count } niet begonnen
library-view-all-watched = Alles gekeken
library-view-on-disk =
    { $count ->
        [one] { $count } aflevering
       *[other] { $count } afleveringen
    } op schijf
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } bestand
       *[other] { $count } bestanden
    } nog op schijf

## apps/linux/src/dbus/notify.rs: de bureaubladmelding voor een nieuwe aflevering

notify-episode-out = Aflevering { $episode } van { $title } is uit
notify-click-open = Klik om het te openen in { -app-name }
notify-click-get = Klik om de afleveringen op te halen in { -app-name }
notify-get-episodes = Afleveringen ophalen

## apps/linux/src/nowplaying.rs: de tweede regel van een mediawidget tijdens het afspelen

now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: waarom het scherm aan blijft

inhibit-reason = Een aflevering afspelen

## apps/linux/src/bridge/door.rs: waarom een waarde van een pagina niet bruikbaar is

door-not-a-choice = { $value } is geen geldige keuze
door-tokens-op-empty = een tokens-op heeft tokens nodig
door-not-a-refine-op = { $value } is geen verfijn-op
door-limit-too-low = moet minstens 1 zijn, kreeg { $limit }
door-not-a-torrent-action = { $value } is geen torrentactie

## apps/linux/src/bridge/shell.rs: de startmap bij de eerste start en de updatemelding

shell-starter-folder-failed = Kon { $folder } niet maken: { $error }
shell-update-gone = { -app-name } { $version } wordt niet meer aangeboden

## Watch together (#846)

onboarding-step-together = Samen kijken
onboarding-title-together = Kijk met vrienden
onboarding-body-together = Start een kamer vanuit de speler en stuur de code. Iedereen speelt zijn eigen kopie van de aflevering, en afspelen, pauzeren en spoelen blijven voor jullie allemaal gelijk.
onboarding-ready-together = Samen kijken
onboarding-ready-together-as = In kamers zien anderen je als { $name }.
onboarding-ready-together-none = Nog geen naam. Het kamerpaneel vraagt erom de eerste keer.
onboarding-together-you = Jij
onboarding-together-code = Kamercode
onboarding-together-episode = Frieren · Aflevering 3
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = Je naam in kamers
rail-room = Samen kijken (Ctrl+J)
rail-room-in = In een kamer om samen te kijken (Ctrl+J)
player-chrome-watch-together = Samen kijken
player-chrome-room = Kamer { $code } · { $count ->
    [one] 1 persoon
   *[other] { $count } personen
}
room-name-empty = Typ de naam die anderen in de kamer zien.
room-name-bad = Die naam werkt niet. Gebruik 1 tot 32 tekens.
room-error-not-found = Geen kamer met die code. Controleer hem bij wie hem stuurde; de kamer is misschien gesloten.
room-error-full = Die kamer is vol. Er passen 10 mensen in een kamer.
room-error-unreachable = Kon de kamerserver niet bereiken. Controleer je verbinding en probeer het opnieuw.
room-error-refused = De kamerserver weigerde dit. Probeer het over een minuut opnieuw.
room-error-proto = Die kamer draait op een nieuwere versie van kamers. Werk { -app-name } bij om mee te doen.
room-error-not-in-room = Je zit niet meer in die kamer. Start er een of doe opnieuw mee.
room-error-bad-code = Een kamercode is 6 letters en cijfers, zoals K7QM2X.
room-error-bad-relay = De kamerserver in Instellingen, Afspelen is geen webadres. Pas het daar aan en probeer het opnieuw.
room-panel-title = Samen kijken
room-panel-close = Sluiten
room-panel-connected = Verbonden
room-panel-reconnecting = Opnieuw verbinden
room-panel-name-label = Je naam in kamers
room-panel-name-placeholder = Je naam
room-panel-name-start = Kamer starten
room-panel-name-join = Kamer binnengaan
room-panel-start = Een kamer starten
room-panel-or-join = Of doe mee met een code
room-panel-join-label = Kamercode
room-panel-join = Meedoen
room-panel-you-are = Je bent { $name }
room-panel-change-name = Wijzigen
room-panel-code-label = Kamercode
room-panel-copy = Link kopiëren
room-panel-link-copied = Kamerlink gekopieerd
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = Deze aflevering staat niet in je bibliotheek, dus je kunt nog niet meekijken.
room-panel-open-episode = Aflevering openen
room-panel-people = { $count ->
    [one] 1 persoon
   *[other] { $count } personen
}
room-panel-host = Host
room-panel-you = Jij
room-panel-ready = Klaar
room-panel-opening = Openen
room-panel-missing = Heeft hem niet
room-panel-offset = Jouw verschuiving
room-panel-offset-earlier = Halve seconde eerder
room-panel-offset-later = Halve seconde later
room-panel-seconds = { $value } s
room-panel-suggested = De intro's wijzen op { $offset }
room-panel-use-suggested = Gebruiken
room-panel-leave = Kamer verlaten
player-room-speed = De snelheid volgt de kamer
settings-find-room-name =
    .words = samen kijken kamer naam vrienden
settings-find-room-relay =
    .words = samen kijken kamer relay server
key-help-key-ctrl-j = Ctrl+J
key-help-room = Samen kijken
settings-playback-together = Samen kijken
settings-playback-room-name = Je naam in kamers om samen te kijken
settings-playback-room-name-placeholder = Je naam
settings-playback-room-relay = Kamerserver
settings-playback-room-relay-helper = Hier worden kamers gemaakt. Wijzig dit alleen voor een server die je zelf draait.
