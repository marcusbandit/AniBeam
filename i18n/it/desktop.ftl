# it translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = Home
nav-library = Libreria
nav-airing = In onda ora
nav-feed = Feed
nav-watching = In visione
nav-stats = Statistiche
nav-metadata = Info serie
nav-downloads = Download
nav-settings = Impostazioni
nav-series = Serie
nav-player = Player
nav-show = Serie
nav-recommended = Consigliati

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } errore
   *[other] { $count } errori
}

## ScanSummary

scan-summary-nothing = Ancora niente da mostrare
scan-summary-found = { $count ->
    [one] Trovata { $count } serie
   *[other] Trovate { $count } serie
}, { $recognised } riconosciute
scan-summary-found-need = { $count ->
    [one] Trovata { $count } serie
   *[other] Trovate { $count } serie
}, { $recognised } riconosciute, { $need } richiedono te
scan-summary-episodes = { $count ->
    [one] { $count } episodio
   *[other] { $count } episodi
} su disco.
scan-summary-episodes-watched = { $count ->
    [one] { $count } episodio
   *[other] { $count } episodi
} su disco, { $watched } già visti su { $tracker }.
scan-summary-title = La tua cartella è dentro
scan-summary-count-found = trovate
scan-summary-count-recognised = riconosciute
scan-summary-count-need = richiedono te
scan-summary-later = Più tardi
scan-summary-match = Abbina le { $count }
scan-summary-open-library = Apri la libreria

## UpdateNotice

update-notice-check-failed = AniBeam non è riuscito a controllare gli aggiornamenti. Controlla la connessione a internet e riprova.
update-notice-up-to-date = AniBeam è aggiornato
update-notice-downloading = Download di AniBeam { $version }
update-notice-ready = AniBeam { $version } è pronto
update-notice-failed = L'aggiornamento non è riuscito
update-notice-on-aur = AniBeam { $version } è su AUR
update-notice-out = È uscito AniBeam { $version }
update-notice-out-titled = È uscito AniBeam { $version } “{ $title }”
update-notice-progress = { $percent }%  ·  { $got } di { $total } MB
update-notice-restart-hint = Riavvia ora, o si apre la prossima volta
update-notice-from-source = Questa copia è compilata dai sorgenti: fai pull e ricompila
update-notice-no-download = Nessun download qui per questo computer
update-notice-install-hint = Aggiorna lo installa e riavvia AniBeam
update-notice-retry = Riprova
update-notice-update = Aggiorna
update-notice-restart = Riavvia
update-notice-copied = Copiato
update-notice-copy = Copia
update-notice-release-page = Pagina release
update-notice-later = Più tardi

## WhatsNew

whats-new-title = Novità
whats-new-got-it = Ho capito

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = d MMM HH:mm
activity-drawer-stage-library = Libreria
activity-drawer-stage-metadata = Metadati
activity-drawer-stage-trackers = Tracker
activity-drawer-stage-franchise = Franchise
activity-drawer-stage-playback = Riproduzione
activity-drawer-stage-store = Archivio
activity-drawer-stage-system = Sistema
activity-drawer-stage-downloads = Download
activity-drawer-level-info = Info
activity-drawer-level-warn = Avviso
activity-drawer-level-error = Errore
activity-drawer-copied-line = Riga copiata
activity-drawer-title = Attività
activity-drawer-everything = Tutto
activity-drawer-problems = Problemi
activity-drawer-copied = Copiato
activity-drawer-copy = Copia
activity-drawer-clear-every = Cancella tutte le righe
activity-drawer-copy-tip = Clicca per copiare la riga
activity-drawer-fold = Comprimi
activity-drawer-empty-technical = Ancora niente nel registro.
activity-drawer-empty-problems = Nessun problema. Tutto ha funzionato.
activity-drawer-empty = Ancora niente da segnalare. Quello che AniBeam fa per te compare qui.

## ActivityColumn

activity-column-episode-run = Episodi da { $first } a { $last }
activity-column-episode-list = Episodi { $list }
activity-column-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } download
}
activity-column-batch = Download batch
activity-column-download = Download
activity-column-name-batch = { $title }, un batch
activity-column-name-episode = { $title }, episodio { $number }
activity-column-title = Attività
activity-column-down = giù
activity-column-up = su
activity-column-not-connected = { $client } non è collegato, quindi qui non si muove nulla.
activity-column-connect-again = Ricollega
activity-column-downloading = In download ora
activity-column-fewer = Meno
activity-column-more = E altri { $count }
activity-column-nothing-downloading = Nessun download in corso.
activity-column-finished-today = Finiti oggi
activity-column-today = Oggi
activity-column-today-empty = Ricerche, download e nuove iscrizioni compaiono qui mentre succedono.
activity-column-stopping = Arresto
activity-column-starting = Avvio
activity-column-start-tip = Avvia il download
activity-column-stop-tip = Ferma il download; quello che ha scaricato resta
activity-column-cancelling = Annullamento
activity-column-cancel-all-tip = Annulla questi download
activity-column-cancel-tip = Annulla questo download
activity-column-size-of = { $got } di { $total }
activity-column-files-missing = File mancanti
activity-column-time-format = HH:mm
activity-column-left = { $time } rimasti
activity-column-waiting-peers = In attesa di peer
activity-column-episodes-done = { $count ->
    [one] { $done } di { $count } episodio fatto
   *[other] { $done } di { $count } episodi fatti
}
activity-column-cube-skipped = Episodio { $number }: saltato
activity-column-cube-done = Episodio { $number }: fatto
activity-column-cube-progress = Episodio { $number }: { $percent }%

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Pronto
frame-strip-progress = { $doing }, { $done } di { $total }
frame-strip-idle = Niente in esecuzione
frame-strip-time-format = HH:mm
frame-finish-sign-in = Prima completa l'accesso a { $tracker }
frame-did-not-work = Non ha funzionato.
frame-could-not-open = AniBeam non è riuscito ad aprirlo. Copialo e aprilo tu.
frame-copy = Copia
frame-copied = Copiato
frame-status-refused = { $tracker } non l'ha accettato
frame-status-refused-why = { $tracker } non l'ha accettato: { $why }
frame-lists-refused = { $tracker } non ha accettato { $count ->
    [one] 1 serie
   *[other] { $count } serie
}; vedi il registro attività
frame-lists-moved = { $count ->
    [one] 1 serie spostata
   *[other] { $count } serie spostate
} su { $tracker }
frame-notes-refused = { $tracker } non ha accettato le tue note
frame-notes-refused-why = { $tracker } non ha accettato le tue note: { $why }
frame-notes-saved = Note salvate
frame-connected = Collegato a { $tracker } come { $user }
frame-sign-in-stopped = L'accesso a { $tracker } si è fermato prima di finire.
frame-toast-hide-details = Nascondi dettagli
frame-toast-details = Dettagli
frame-toast-details-tip = Mostra cosa è andato storto, parola per parola, e copialo
frame-forward = Avanti
frame-copy-magnet = Copia magnet
frame-copy-link = Copia link torrent
frame-magnet-copied = Magnet copiato
frame-link-copied = Link copiato
frame-open-release-page = Apri pagina release
frame-open-folder = Apri cartella
frame-copy-path = Copia percorso
frame-path-copied = Percorso copiato
frame-mark-unwatched = Segna non visti da qui
frame-mark-watched = Segna visti fino a qui
frame-copy-title = Copia titolo
frame-open-on-anilist = Apri su AniList
frame-list-home-online = Cambialo su ogni lista collegata
frame-list-home-unmatched = Salvato su questo dispositivo: nessun tracker ha questa serie
frame-list-home-not-carried = Salvato su questo dispositivo: il tuo tracker non ha questa serie
frame-list-home-offline = Salvato su questo dispositivo finché non colleghi AniList o MyAnimeList
frame-moving-to = Spostamento in { $list }
frame-moved-to = Spostato in { $list }
frame-plan-added = { $title } va nella tua lista Da vedere
frame-plan-listed = { $title } è già nella tua lista
frame-delete-watched = { $kind ->
    [film] Elimina film visti
   *[episode] Elimina episodi visti
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] Elimina il film
       *[episode] Elimina l'episodio
    }
   *[other] { $kind ->
        [film] Elimina tutti i { $count } film
       *[episode] Elimina tutti i { $count } episodi
    }
}
frame-series-gone = Quella serie non c'è più
frame-delete-none-watched = Nessun episodio visto di { $title } su disco
frame-delete-none = Niente di { $title } su disco
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } film visto
           *[other] { $count } film visti
        }
       *[episode] { $count ->
            [one] { $count } episodio visto
           *[other] { $count } episodi visti
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } film
           *[other] { $count } film
        }
       *[episode] { $count ->
            [one] { $count } episodio
           *[other] { $count } episodi
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } extra
   *[other] { $count } extra
}
frame-delete-and = { $first } e { $second }
frame-delete-heading = Eliminare { $what } di { $title }?
frame-delete-base = I file e i loro sottotitoli vanno nel cestino, quindi puoi recuperarli da lì. I loro torrent escono dal client torrent, quindi si possono scaricare di nuovo.
frame-delete-confirm = Sposta nel cestino
frame-delete-done = { $what } di { $title } nel cestino
frame-delete-watched-done = { $count ->
    [one] { $count } episodio
   *[other] { $count } episodi
}, { $size }, nel cestino
frame-delete-nothing = Niente da eliminare
frame-fix-not-drafted = Corretto. Non è stato possibile preparare la segnalazione
frame-fix-agent = Corretto. { $agent } ci sta guardando, e la segnalazione si apre quando ha finito
frame-fix-opening = Corretto. Apertura della segnalazione
frame-fix-not-saved = La correzione non è stata salvata: { $error }
frame-fix-undone = { $count ->
    [one] { $count } file torna a essere letto dal nome
   *[other] { $count } file tornano a essere letti dal nome
}
frame-fix-done = { $count ->
    [one] Corretto come si legge { $count } file
   *[other] Corretto come si leggono { $count } file
}
frame-report-failed = Non è stato possibile preparare la segnalazione: { $error }
frame-report-untriaged = La segnalazione è aperta nel browser, senza triage. { $error }
frame-report-open = La segnalazione è aperta nel browser, pronta da inviare
frame-downloads-not-read = Non letto.

## HomePage

home-title = Home
home-scan-started = Scansione avviata
home-ago-minutes = { $count } min fa
home-ago-hours = { $count } h fa
home-ago-days = { $count } g fa
home-today = Oggi
home-tomorrow = Domani
home-yesterday = Ieri
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d MMM
home-day-date-format = ddd d MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = Ancora senza data
home-unit-days = { $count ->
    [one] giorno
   *[other] giorni
}
home-unit-hours = h
home-unit-minutes = m
home-week-late = { $count ->
    [one] { $count } episodio in ritardo
   *[other] { $count } episodi in ritardo
}
home-week-waiting = { $count ->
    [one] { $count } episodio uscito da guardare
   *[other] { $count } episodi usciti da guardare
}
home-week-coming = { $count ->
    [one] { $count } episodio nei prossimi sette giorni
   *[other] { $count } episodi nei prossimi sette giorni
}
home-week-next = Prossimo: { $title } tra { $time }
home-countdown-days = { $days } g { $hours } h
home-countdown-hours = { $hours } h { $minutes } m
home-countdown-minutes = { $minutes } m
home-out-not-downloaded = Non ancora scaricato
home-out-downloading = In download
home-out-when = { $day } { $time }
home-next-up = Prossimo
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, orario stimato
home-next-when-season = Stagione { $season }, { $span }, { $time }
home-next-when-season-estimated = Stagione { $season }, { $span }, { $time }, orario stimato
home-no-date = Nessuna data
home-tile-watched = visto
home-tile-downloading = in download
home-tile-out = uscito { $ago }
home-tile-not-out = non ancora uscito
home-tile-estimated = stimato
home-tile-season-span = S{ $season } { $span }
home-tile-to-watch = { $count } da vedere
home-day-nothing = Niente
home-now = Ora { $time }
home-card-aired = { $episode } uscito { $ago }
home-continue = Continua
home-continue-fact = { $count } serie da continuare
home-on-hold = In pausa
home-on-hold-fact = { $count } serie da riprendere
home-hero-open = Apri { $title }
home-hero-pick-up = Riprendi da dove hai smesso
home-hero-up-next = Prossimo
home-hero-downloading = { $count ->
    [one] Download di { $count } episodio
   *[other] Download di { $count } episodi
}
home-hero-min-left = { $count } min rimasti
home-hero-watched = { $watched } di { $total } visti
home-hero-resume = Riprendi ep. { $number }
home-hero-play = Riproduci ep. { $number }
home-hero-resume-unnumbered = Riprendi episodio
home-hero-play-unnumbered = Riproduci episodio
home-downloaded = Scaricati di recente
home-downloaded-fact = { $count } serie con file nuovi nelle ultime due settimane
home-week = Questa settimana
home-week-show-empty = Mostra giorni vuoti
home-week-hide-empty = Nascondi giorni vuoti
home-week-airing = In onda ora
home-week-out = Usciti ora
home-week-coming-up = In arrivo
home-recommended = Consigliati per te
home-recommended-fact = Dalla tua lista
home-recommended-scope = Consigliati da
home-recommended-all = Tutti
home-recommended-in-library = In libreria
home-recommended-on-list = In lista
home-recommended-none-library = Non resta niente da consigliare nella tua libreria. Tutti trova serie che non hai ancora.
home-recommended-none-list = Non resta niente da consigliare nelle tue liste. Tutti trova serie che non hai in lista.
home-catch-up = Mettiti in pari
home-catch-up-fact = { $count } serie con episodi in attesa
home-fresh = Inizia qualcosa di nuovo
home-fresh-fact = { $count } serie non iniziate
home-genres = Generi
home-open-library = Apri libreria
home-dropped = Abbandonati
home-dropped-fact = { $count ->
    [one] { $count } file ancora su disco
   *[other] { $count } file ancora su disco
}
home-out-now = Usciti ora
home-out-now-fact = Nuovi episodi di questa settimana, da AniList
home-popular = Popolari questa stagione
home-add-folder-title = Anche le tue serie vanno qui
home-add-folder-body = Aggiungi la cartella dei tuoi anime e la Home si riempie di quello che stai guardando.
home-add-folder-button = Aggiungi cartella
home-empty-scanned-title = Ancora niente nelle tue cartelle
home-empty-title = Ancora nessuna serie
home-empty-scanned-body = AniBeam non ha trovato serie né film nelle cartelle che controlla. Scansionale di nuovo, o controlla le cartelle nelle Impostazioni.
home-empty-body = Aggiungi la cartella dei tuoi anime e la libreria si riempie da sola.
home-scan-all = Scansiona tutto
home-library-settings = Impostazioni libreria
home-failed-title = La Home non si è caricata
home-failed-body = AniBeam non è riuscito a leggere la libreria. Riprova tra un attimo.

## Onboarding

onboarding-language = Lingua
onboarding-step-welcome = Benvenuto
onboarding-step-library = La tua libreria
onboarding-step-lists = La tua lista
onboarding-step-downloads = Download
onboarding-step-ready = Pronto
onboarding-scan-reading = Lettura della cartella
onboarding-scan-found = Trovate { $count } serie. Vai avanti quando vuoi; il resto si completa da solo.
onboarding-scan-starter-empty = La tua nuova cartella per ora è vuota. Le serie che ci metti compaiono qui da sole.
onboarding-scan-no-video = Nessun file video trovato. Controlla che sia la cartella giusta.
onboarding-scan-your-shows = le tue serie
onboarding-scan-looking-up = Ricerca di { $what }
onboarding-scan-looking-up-tally = Ricerca di { $what }, { $posters } di { $count } trovate
onboarding-scan-posters = Download delle locandine
onboarding-scan-posters-tally = Download delle locandine, { $posters } di { $count } trovate
onboarding-scan-seasons = Ricerca delle stagioni correlate
onboarding-scan-seasons-tally = Ricerca delle stagioni correlate, { $posters } di { $count } trovate
onboarding-choose-folder = Scegli la cartella degli anime
onboarding-about = Un minuto di configurazione. La libreria si completa da sola.
onboarding-step-later = Più tardi
onboarding-skip-setup = Salta configurazione
onboarding-title-welcome = Benvenuto in AniBeam
onboarding-title-library = Dove sono i tuoi anime?
onboarding-title-lists = Collega la tua lista di anime
onboarding-title-downloads = Come funzionano i download
onboarding-title-ready = È tutto pronto
onboarding-title-later = { $count ->
    [1] Pronto, una cosa per dopo
    [2] Pronto, due cose per dopo
   *[other] Pronto, tre cose per dopo
}
onboarding-body-welcome = AniBeam tiene i tuoi anime in un solo posto. Trova le serie sul disco, le riproduce qui e tiene aggiornati AniList o MyAnimeList mentre guardi.
onboarding-body-library = Scegli la cartella in cui tieni le tue serie. Ogni cartella al suo interno diventa una serie, con la sua locandina, i suoi episodi e il punto in cui eri arrivato. AniBeam legge soltanto la cartella. Non sposta né rinomina mai un file.
onboarding-body-lists = Accedi una volta, e ogni episodio che finisci qui viene segnato nella tua lista, senza scrivere nulla. L'accesso avviene nel browser, quindi AniBeam non vede mai la tua password.
onboarding-body-downloads = Segui una serie nella pagina Download e ogni nuovo episodio arriva nella libreria appena va in onda. Servono due cose: un posto dove cercare e un client torrent per scaricare.
onboarding-body-ready = Tutto qui si può cambiare più tardi nelle Impostazioni, e Impostazioni, Info ripete questa configurazione.
onboarding-skip-step = Salta per ora
onboarding-get-started = Iniziamo
onboarding-open-app = Apri AniBeam
onboarding-continue = Continua
onboarding-payoff-library = La tua libreria
onboarding-payoff-library-line = Locandine, episodi e cosa viene dopo, per ogni serie nelle tue cartelle.
onboarding-payoff-plays = Riproduce qui
onboarding-payoff-plays-line = Sottotitoli, salto delle intro, e ricorda dove ti eri fermato.
onboarding-payoff-list = La tua lista, sincronizzata
onboarding-payoff-list-line = Finisci un episodio e conta su AniList o MyAnimeList.
onboarding-payoff-episodes = Arrivano gli episodi
onboarding-payoff-episodes-line = Segui una serie e ogni nuovo episodio arriva nella tua libreria.
onboarding-films-too = Funziona anche una cartella di film: chiamala Movies.
onboarding-choose-folder-button = Scegli cartella
onboarding-no-folder = Non ho ancora una cartella
onboarding-starter-where = Crea una cartella Anime in { $dir } e la usa.
onboarding-add-another = Aggiungi un'altra cartella
onboarding-shelf-more = altre
onboarding-preview-show = La tua serie
onboarding-preview-finished = Episodio 3 finito qui
onboarding-preview-on-list = Nella tua lista: 3 visti
onboarding-preview-set-by = Impostato da AniBeam appena partono i titoli di coda
onboarding-lists-connected = Collegato. Quello che guardi qui ora conta nella tua lista.
onboarding-lists-no-account = Non hai un account? Sono entrambi gratis, e uno basta. Puoi scollegarlo quando vuoi in Impostazioni, Account.
onboarding-search-title = Un posto dove cercare
onboarding-search-ready = { $count ->
    [one] { $count } fonte pronta per la ricerca.
   *[other] { $count } fonti pronte per la ricerca.
}
onboarding-search-none = AniBeam non include fonti proprie. Le aggiungi da un repo di fonti: un link che qualcuno condivide con te, da incollare in Impostazioni, Download.
onboarding-sources-after = Si apre dopo la configurazione
onboarding-sources-add = Aggiungi un repo dopo la configurazione
onboarding-fetch-title = Chi scarica
onboarding-fetch-auto = Ci pensa AniBeam
onboarding-fetch-recommended = Consigliato
onboarding-fetch-auto-line = AniBeam porta il suo qBittorrent e lo fa girare in silenzio in background. Niente da installare, niente da compilare.
onboarding-fetch-existing = Uso già qBittorrent
onboarding-fetch-existing-line = AniBeam si collega al qBittorrent che usi già. Subito dopo inserisci indirizzo e accesso nelle Impostazioni.
onboarding-ready-library = Libreria
onboarding-ready-library-none = Ancora nessuna cartella. La trovi in Impostazioni, Libreria quando vuoi.
onboarding-ready-library-series = { $count ->
    [one] { $count } cartella
   *[other] { $count } cartelle
}, { $series } serie finora
onboarding-ready-library-empty = { $count ->
    [one] { $count } cartella
   *[other] { $count } cartelle
}, vuota per ora
onboarding-ready-library-scanning = { $count ->
    [one] { $count } cartella
   *[other] { $count } cartelle
}, scansione in corso
onboarding-ready-lists = La tua lista
onboarding-ready-lists-as = { $tracker }, come { $user }
onboarding-ready-lists-none = Non ancora collegata. La trovi in Impostazioni, Account quando vuoi.
onboarding-ready-downloads = Download
onboarding-ready-downloads-line = { $sources ->
    [0] Ancora nessuna fonte
    [one] { $sources } fonte
   *[other] { $sources } fonti
}, { $client ->
    [own] il tuo qBittorrent.
   *[bundled] il qBittorrent di AniBeam.
} { $next ->
    [ready] Tutto pronto.
    [opens] Poi si apre Impostazioni, Download.
   *[both] Entrambi sono in Impostazioni, Download.
}
onboarding-ready-open-downloads = Apri Impostazioni, Download
onboarding-ready-set-up = Configura ora

## Rail.qml

rail-home = Home
rail-library = Libreria
rail-airing = In onda
rail-feed = Feed
rail-stats = Statistiche
rail-show-info = Info serie
rail-downloads = Download
rail-settings = Impostazioni
rail-downloads-complete =
    { $count ->
        [one] Download: { $count } iscrizione completata
       *[other] Download: { $count } iscrizioni completate
    }
rail-title-language-name = Lingua dei titoli: { $language }. Premi per la successiva
rail-title-language-english = Inglese
rail-title-language-tip = Lingua dei titoli: romaji, inglese o giapponese
rail-whats-new-in = Novità in { $release }
rail-whats-new = Novità
rail-settings-note = Impostazioni: { $note }

## FixSheet.qml

fix-kind-episode = Episodio
fix-kind-opening = Opening
fix-kind-ending = Ending
fix-kind-preview = Anteprima
fix-kind-special = Speciale
fix-kind-other = Altro
fix-reads-season-episode = Stagione { $season } episodio { $number }
fix-reads-extra = { $kind } { $index }
fix-apply-failed = Impossibile applicare le correzioni.
fix-title = Correggi la lettura dei file
fix-lede = { $title }. Indica cos'è davvero ogni file; ogni scansione successiva lo mantiene.
fix-rescan = Riscansiona cartella
fix-rescan-started = Rilettura della cartella
fix-filled-in = Compilato
fix-fill-in = Compila
fix-column-file = File
fix-column-reads-as = Letto come
fix-column-should-be = Dovrebbe essere
fix-fixed = Corretto
fix-number-placeholder = N.
fix-index-placeholder = Nessuno
fix-keep = Mantieni la correzione
fix-leave-as-read = Lascia il file come viene letto
fix-read-by-name = Leggi di nuovo il file dal nome
fix-report = Apri una segnalazione su GitHub
fix-report-body = Apre una segnalazione compilata da inviare. È pubblica e nomina questi file.
fix-report-body-agent = Apre una segnalazione compilata da inviare. È pubblica e nomina questi file. Prima ci guarda { $agent }.
fix-close-unapplied =
    { $count ->
        [one] Chiudere senza applicare { $count } correzione?
       *[other] Chiudere senza applicare { $count } correzioni?
    }
fix-number-invalid = Prima correggi i numeri in rosso
fix-close-anyway = Chiudi comunque
fix-keep-editing = Continua a modificare
fix-apply-count =
    { $count ->
        [one] Applica { $count } correzione
       *[other] Applica { $count } correzioni
    }

## MatchPanel.qml

match-matched-to = Abbinata a { $title }
match-applied = Abbinamento applicato
match-found-tvmaze = Serie trovata su TVmaze
match-found-anilist = Trovata su AniList
match-found-anilist-season = Trovata su AniList, stagione { $season }
match-found-mal = Trovata su MyAnimeList
match-sites-and = { $first } e { $next }
match-matched-on = Abbinata su { $sites }
match-provider-silent = { $provider } non ha risposto
match-title-walk = Abbina, { $walk }
match-title = Abbina
match-skip = Salta
match-refresh-info = Aggiorna info
match-refreshing = Aggiornamento di { $title }
match-clear-confirm = Rimuovi: elimina il legame e il numero di episodi impostato
match-clear = Rimuovi abbinamento
match-cleared = Abbinamento rimosso
match-search-placeholder = Cerca su { $provider }
match-link-tvmaze = Incolla un link di TVmaze o IMDb
match-link-anilist = Incolla un link di AniList o MyAnimeList
match-look-up = Cerca
match-searching = Ricerca su { $provider }
match-type-more = Scrivi almeno due lettere per cercare su { $provider }.
match-no-results = Nessun titolo trovato su { $provider }.

## WatchingPage.qml

watching-episode-code = Ep { $number }
watching-countdown-days = { $days } g { $hours } h
watching-countdown-hours = { $hours } h { $minutes } m
watching-countdown-minutes = { $minutes } m
watching-on-disk = Su disco
watching-on-anilist = Su AniList
watching-ready-to-start = Pronta da iniziare
watching-not-in-library = Non in libreria
watching-get-to-start = Scarica episodi per iniziare
watching-anilist-updated = AniList aggiornato { $when }
watching-airing = In onda
watching-next-in = Prossimo: { $title } tra { $wait }
watching-episodes-waiting = Episodi in attesa
watching-series-behind =
    { $count ->
        [one] { $count } serie indietro
       *[other] { $count } serie indietro
    }
watching-up-to-date = In pari
watching-anilist-only = Solo su AniList
watching-planning-summary-off = Quello che hai in programma di vedere su AniList.
watching-planning-summary =
    { $count ->
        [one] { $count } serie da vedere, { $here } su disco.
       *[other] { $count } serie da vedere, { $here } su disco.
    }
watching-summary-off = Quello che stai guardando o rivedendo su AniList.
watching-summary-waiting =
    { $count ->
        [one] { $count } serie ha episodi in attesa.
       *[other] { $count } serie hanno episodi in attesa.
    }
watching-caught-up = Sei in pari.
watching-start-watching = Inizia a guardare
watching-play-next = Riproduci il prossimo
watching-open-on-anilist = Apri su AniList
watching-refreshing = Aggiornamento
watching-connect-title = AniList non è collegato
watching-connect-body = Collegalo e questa pagina elenca le serie che stai guardando o rivedendo su AniList, con Riproduci su quelle che hai e un conto alla rovescia su quelle in onda.
watching-connect-waiting = In attesa del browser
watching-connect-anilist = Collega AniList
watching-load-failed-title = La lista In visione non si è caricata
watching-load-failed-body = Impossibile raggiungere AniList. La lista torna al prossimo aggiornamento.
watching-planning-failed-title = La lista Da vedere non si è caricata
watching-planning-failed-body = Impossibile leggere la sua cache. Riprova, o torna al prossimo aggiornamento.
watching-planning-empty-title = Niente nella lista Da vedere
watching-planning-empty-body = Da vedere in In onda ora, o Da vedere nel menu lista di una serie, la mette qui.
watching-airing-now = In onda ora
watching-empty-title = Niente nella lista In visione
watching-empty-body = Segna una serie come In visione su AniList e arriva qui.
watching-open-anilist = Apri AniList

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] { $count } serie nascosta
       *[other] { $count } serie nascoste
    }
library-hidden-sync-stops =
    { $count ->
        [one] { $count } serie nascosta, e la sincronizzazione con il tracker si ferma
       *[other] { $count } serie nascoste, e la sincronizzazione con il tracker si ferma
    }
library-unhidden =
    { $count ->
        [one] { $count } serie tornata, e di nuovo sincronizzata
       *[other] { $count } serie tornate, e di nuovo sincronizzate
    }
library-undo = Annulla
library-clearing-score =
    { $count ->
        [one] Cancellazione del voto di { $count } serie
       *[other] Cancellazione del voto di { $count } serie
    }
library-rating =
    { $count ->
        [one] Voto { $score } a { $count } serie
       *[other] Voto { $score } a { $count } serie
    }
library-moving =
    { $count ->
        [one] Spostamento di { $count } serie in { $list }
       *[other] Spostamento di { $count } serie in { $list }
    }
library-rescanning =
    { $count ->
        [one] Nuova scansione di { $count } serie
       *[other] Nuova scansione di { $count } serie
    }
library-finding-matches =
    { $count ->
        [one] Ricerca di un abbinamento per { $count } serie
       *[other] Ricerca di un abbinamento per { $count } serie
    }
library-sort-last-viewed = Visti di recente
library-sort-downloaded = Scaricati di recente
library-sort-title = Titolo
library-sort-progress = Progressi
library-sort-average-score = Voto medio
library-sort-my-score = Il mio voto
library-dir-a-to-z = Dalla A alla Z
library-dir-z-to-a = Dalla Z alla A
library-dir-oldest = Prima i più vecchi
library-dir-newest = Prima i più recenti
library-dir-least-watched = Prima i meno visti
library-dir-most-watched = Prima i più visti
library-dir-lowest = Prima i più bassi
library-dir-highest = Prima i più alti
library-group-none = Niente
library-group-season = Stagione
library-group-type = Tipo
library-group-genre = Genere
library-group-watch-state = Stato visione
library-scan-started = Scansione avviata
library-tab-all = Tutto
library-tab-series = Serie
library-tab-movies = Film
library-tab-watched = Visti
library-tab-hidden = Nascosti
library-count-movies =
    { $count ->
        [one] { $count } film
       *[other] { $count } film
    }
library-count-watched = { $count } visti
library-count-hidden = { $count } nascosti
library-count-series =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serie
    }
library-title = Libreria
library-all-genres = Tutti i generi
library-search-placeholder = Cerca titoli o cartelle
library-sort = Ordina
library-group = Raggruppa
library-franchises-together = Franchise insieme
library-watched-shows = Serie viste
library-load-failed-title = La libreria non si è caricata
library-no-matches = Nessun risultato per "{ $query }"
library-none-in-genre =
    { $what ->
        [series] Nessuna serie { $genre } qui
        [films] Nessun film { $genre } qui
        [watched] Nessuna serie { $genre } vista qui
        [hidden] Nessuna serie { $genre } nascosta qui
       *[shows] Nessuna serie { $genre } qui
    }
library-none-here =
    { $what ->
        [series] Nessuna serie qui
        [films] Nessun film qui
        [watched] Nessuna serie vista qui
        [hidden] Nessuna serie nascosta qui
       *[shows] Nessuna serie qui
    }
library-load-failed-body = AniBeam non è riuscito a leggere la libreria. Riprova tra un attimo.
library-no-matches-body = Controlla l'ortografia, o cerca in tutta la libreria da Tutto.
library-none-in-genre-body = Niente in questa vista ha quel genere. Tutti i generi mostra il resto.
library-none-in-tab-body = La tua libreria non ne ha ancora. Tutto mostra ogni cosa che contiene.
library-all-filtered-body = Tutto nella tua libreria è escluso da questa vista.
library-clear-search = Cancella ricerca
library-show-all = Mostra tutto
library-picked = { $count } scelti
library-pick-all = Tutti
library-pick-all-tip = Scegli tutte le schede mostrate (Ctrl+A)
library-unhide = Mostra
library-hide = Nascondi
library-rate = Vota
library-rate-tip = Un voto per ogni scelta, su ogni tracker collegato
library-needs-tracker = Serve un tracker collegato e una serie abbinata
library-move-to-list = Sposta in lista
library-rescan = Riscansiona
library-find-match = Trova abbinamento
library-find-match-tip = Cerca ogni scelta senza abbinamento
library-all-matched = Ogni scelta è abbinata
library-empty-scanned-title = Ancora niente nelle tue cartelle
library-empty-title = La tua libreria è vuota
library-empty-scanned-body = AniBeam non ha trovato serie né film nelle cartelle che controlla. Scansionale di nuovo, o controlla le cartelle nelle Impostazioni.
library-empty-body = Aggiungi la cartella dei tuoi anime e la libreria si riempie da sola, oppure importa un'esportazione di AniBeam.
library-scan-all = Scansiona tutto
library-add-folder = Aggiungi cartella
library-settings = Impostazioni libreria
library-import = Importa
library-import-title = Importa un'esportazione di AniBeam
library-import-filter-export = Esportazione di AniBeam (*.json)
library-import-filter-all = Tutti i file (*)
library-import-started = Importazione avviata

## MetadataPage.qml

metadata-title = Info serie
metadata-filter-all = Tutte { $count }
metadata-filter-series = Serie { $count }
metadata-filter-films = Film { $count }
metadata-filter-missing = Mancanti { $count }
metadata-scan-started = Scansione avviata
metadata-sort-added = Aggiunte
metadata-sort-season = Stagione
metadata-sort-a-to-z = A-Z
metadata-section-unmatched = Non riconosciute
metadata-section-missing = Cartella non trovata
metadata-section-incomplete = Episodi mancanti
metadata-section-empty = Non scaricate
metadata-section-airing = In onda
metadata-section-complete = Complete
metadata-no-info = Ancora nessuna info
metadata-state-aired = { $have } di { $aired } usciti
metadata-state-of = { $have } di { $expected }
metadata-state-of-extra = { $have } di { $expected } +{ $extra }
metadata-detail-none-aired = Nessuno dei { $aired } episodi usciti è su disco
metadata-detail-aired = { $have } di { $aired } usciti su disco
metadata-detail-aired-missing = { $have } di { $aired } usciti su disco, { $missing } mancanti
metadata-detail-none = Nessuno dei { $expected } episodi è su disco
metadata-detail-missing = { $have } di { $expected } su disco, { $missing } mancanti
metadata-walk-position = { $at } di { $total }
metadata-walk-done = Era l'ultima
metadata-walk-left =
    { $count ->
        [one] Ne resta una in Da abbinare
       *[other] Ne restano { $count } in Da abbinare
    }
metadata-countdown-days = { $days } g { $hours } h
metadata-countdown-hours = { $hours } h { $minutes } m
metadata-countdown-minutes = { $minutes } m
metadata-next-in = Prossimo tra { $wait }
metadata-files-of = { $have } di { $expected }
metadata-files-on-disk = File su disco
metadata-forgot = { $title } dimenticata
metadata-refreshed-failed = { $refreshed } aggiornate, { $failed } non aggiornate
metadata-refreshed =
    { $count ->
        [one] { $count } serie aggiornata
       *[other] { $count } serie aggiornate
    }
metadata-found =
    { $count ->
        [one] Trovata { $count } serie
       *[other] Trovate { $count } serie
    }
metadata-found-some-left =
    { $count ->
        [one] Trovata { $count } serie, { $unmatched } ancora non riconosciute
       *[other] Trovate { $count } serie, { $unmatched } ancora non riconosciute
    }
metadata-found-nothing = Niente di nuovo, { $unmatched } ancora non riconosciute
metadata-all-recognised = Ogni serie è riconosciuta
metadata-count =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serie
    }
metadata-search-placeholder = Filtra titoli
metadata-find-rest = Trova le altre
metadata-find-rest-started = Ricerca delle cartelle che AniBeam non conosce ancora
metadata-find-rest-tip = Cerca ogni cartella che AniBeam non conosce ancora, su ogni sito di info nell'ordine dato dalle Impostazioni
metadata-refresh-all = Aggiorna tutto
metadata-refresh-all-started = Aggiornamento delle info di ogni serie
metadata-refresh-all-tip = Scarica di nuovo le info di ogni serie riconosciuta
metadata-crawling = Ricerca dei legami del franchise
metadata-crawl-progress = { $done } di { $total }
metadata-load-failed-title = Info serie non si è caricata
metadata-empty-scanned-title = Ancora niente nelle tue cartelle
metadata-no-matches = Nessun risultato per "{ $query }"
metadata-nothing-under =
    { $filter ->
        [Series] Niente in Serie
        [Movies] Niente in Film
        [MissingFiles] Niente in Mancanti
       *[other] Niente in { $filter }
    }
metadata-load-failed-body = AniBeam non è riuscito a leggere la libreria. Riprova tra un attimo.
metadata-empty-scanned-body = AniBeam non ha trovato serie nelle cartelle che controlla. Scansionale di nuovo, o controlla le cartelle nelle Impostazioni.
metadata-empty-body = La tua libreria è vuota. Aggiungi la cartella dei tuoi anime e AniBeam la scansiona.
metadata-no-matches-body = Nessun titolo in questa vista contiene quelle parole.
metadata-nothing-under-body = Nessuna delle tue serie rientra in questo filtro ora. Tutte mostra il resto.
metadata-scan-all = Scansiona tutto
metadata-add-folder = Aggiungi cartella
metadata-clear-search = Cancella ricerca
metadata-show-all = Mostra tutto
metadata-refreshing = Aggiornamento di { $title }
metadata-refresh-tip = Scarica di nuovo le info di questa serie
metadata-forget-armed = Rimuovila
metadata-forget-confirm-tip = Clicca di nuovo per rimuovere questa serie e la sua cronologia da AniBeam
metadata-forget-tip = La cartella non è più sul disco: rimuovi questa serie e la sua cronologia da AniBeam

## AiringPage.qml

airing-title = In onda ora
airing-sort-popular = Popolarità
airing-sort-score = Voto
airing-sort-trending = Tendenza
airing-sort-favourites = Preferiti
airing-sort-soonest = Prossimo episodio
airing-sort-aired = Episodi usciti
airing-sort-episodes = Numero di episodi
airing-sort-start = Data di inizio
airing-sort-duration = Durata episodio
airing-sort-title = Titolo
airing-dir-most-popular = Prima i più popolari
airing-dir-least-popular = Prima i meno popolari
airing-dir-lowest = Prima i più bassi
airing-dir-highest = Prima i più alti
airing-dir-least-trending = Prima i meno di tendenza
airing-dir-most-trending = Prima i più di tendenza
airing-dir-fewest = Prima i meno
airing-dir-most = Prima i più
airing-dir-soonest = Prima i più vicini
airing-dir-latest = Prima i più lontani
airing-dir-oldest = Prima i più vecchi
airing-dir-newest = Prima i più recenti
airing-dir-shortest = Prima i più corti
airing-dir-longest = Prima i più lunghi
airing-dir-a-to-z = Dalla A alla Z
airing-dir-z-to-a = Dalla Z alla A
airing-season =
    { $season ->
        [Winter] Inverno { $year }
        [Spring] Primavera { $year }
        [Summer] Estate { $year }
        [Fall] Autunno { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = { $minutes } min fa
airing-ago-hours = { $hours } h fa
airing-ago-days = { $days } g fa
airing-episode-out = { $episode } uscito { $ago }
airing-no-date = Ancora senza data
airing-in-library = In libreria
airing-out-today =
    { $count ->
        [one] { $count } episodio uscito oggi
       *[other] { $count } episodi usciti oggi
    }
airing-dismissed-one = { $title } non comparirà più qui
airing-dismissed =
    { $count ->
        [one] { $count } serie non comparirà più qui
       *[other] { $count } serie non compariranno più qui
    }
airing-planned-already-one = { $title } è già nella tua lista
airing-planned-already-all = Sono già tutte nella tua lista
airing-planned =
    { $count ->
        [one] { $count } serie va nella tua lista Da vedere
       *[other] { $count } serie vanno nella tua lista Da vedere
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } serie va nella tua lista Da vedere, { $already } c'erano già
       *[other] { $count } serie vanno nella tua lista Da vedere, { $already } c'erano già
    }
airing-play-next = Riproduci il prossimo
airing-plan-to-watch = Da vedere
airing-get-more-episodes = Scarica altri episodi
airing-get-episodes = Scarica episodi
airing-open-on-anilist = Apri su AniList
airing-not-interested = Non mi interessa
airing-not-interested-count = { $count } non interessanti
airing-not-interested-tip = Le serie che hai segnato come non interessanti, nelle Impostazioni
airing-refreshing = Aggiornamento
airing-pick-season = Scegli un'altra stagione
airing-filter-all = Tutto
airing-filter-lists = Nelle mie liste
airing-filter-library = Nella mia libreria
airing-hide-lists = Nascondi le mie liste
airing-hide-lists-tip = Escludi le serie nelle tue liste di AniList o MAL
airing-hide-library = Nascondi la mia libreria
airing-hide-library-tip = Escludi le serie di cui hai episodi
airing-out-now = Usciti ora
airing-out-last-day =
    { $count ->
        [one] { $count } episodio nelle ultime 24 ore
       *[other] { $count } episodi nelle ultime 24 ore
    }
airing-out-fewer = Meno
airing-out-all = Tutti { $count }
airing-sort = Ordina
airing-load-failed-title = Impossibile raggiungere AniList
airing-load-failed-body = Le serie in onda vengono da AniList. Tornano al prossimo aggiornamento.
airing-reading-title = Lettura di cosa è in onda
airing-nothing-now = Niente in onda ora
airing-nothing-for = Niente in onda per { $season }
airing-nothing-read = Ancora niente letto
airing-reading-body = Ogni serie in onda, da AniList. La prima volta ci vuole qualche secondo.
airing-nothing-body = AniList non elenca ancora serie per questa stagione. Aggiorna la rilegge, oppure scegli un'altra stagione.
airing-nothing-read-body = Aggiorna legge da AniList ogni serie in onda.
airing-hides-empty-title = Con quello che nascondi qui non resta niente
airing-lists-empty-title = Niente qui è nelle tue liste
airing-library-empty-title = Niente qui è nella tua libreria
airing-hides-empty-body = Ogni serie qui è nelle tue liste o nella tua libreria, e le hai nascoste.
airing-lists-empty-body = Da vedere su una scheda la mette nella tua lista.
airing-library-empty-body = Scarica episodi su una scheda la scarica nella tua libreria.
airing-show-everything = Mostra tutto
airing-picked = { $count } scelte
airing-undo = Annulla

## EpisodeRow.qml

episode-row-date-time-format = ddd d MMM, HH:mm
episode-row-date-upcoming-format = ddd d MMM
episode-row-date-format = d MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Clicca per segnare non visti da qui
episode-row-mark-watched-tip = { $state }. Clicca per segnare visti fino a qui
episode-row-next-up = Prossimo
episode-row-versions =
    { $count ->
        [one] { $count } versione
       *[other] { $count } versioni
    }
episode-row-versions-tip =
    { $count ->
        [one] Lo stesso episodio da { $count } release. Clicca per scegliere quale riprodurre
       *[other] Lo stesso episodio da { $count } release. Clicca per scegliere quale riprodurre
    }
episode-row-downloading = Download { $percent }%
episode-row-get = Scarica episodio
episode-row-predicted = Previsto
episode-row-extra = Extra
episode-row-resume = Riprendi da { $time }

## EpisodeCubes.qml

episode-cubes-one = Un episodio
episode-cubes-range =
    { $count ->
        [one] { $count } episodio, da { $first } a { $last }
       *[other] { $count } episodi, da { $first } a { $last }
    }
episode-cubes-tip = Episodio { $number }: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } extra
       *[other] { $count } extra
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } file non abbinato
       *[other] { $count } file non abbinati
    }
episodes-pane-on-disk = Su disco
episodes-pane-not-on-disk = Non su disco
episodes-pane-season = S{ $season }
episodes-pane-get-tip = Scarica episodio { $number }
episodes-pane-none-on-disk = Nessun episodio su disco
episodes-pane-change-match = Cambia abbinamento
episodes-pane-refreshing = Aggiornamento di { $title }

## ShowDetail.qml

show-detail-not-in-library = Questa cartella non è ancora nella libreria. Quando una scansione la trova, qui compaiono la sua locandina e i suoi episodi.
show-detail-film-on-disk = Il film è su disco.
show-detail-film-not-on-disk = Il film non è ancora su disco.
show-detail-complete-line = Andata in onda per intero, e tutti i { $total } episodi sono qui. Non arriverà altro, quindi l'iscrizione non ha più niente da fare.
show-detail-finished = Conclusa.
show-detail-airing = In onda.
show-detail-have-of-total = { $have } di { $total } episodi qui.
show-detail-have =
    { $count ->
        [one] { $count } episodio qui.
       *[other] { $count } episodi qui.
    }
show-detail-next-predicted =
    { $count ->
        [one] { $span } è previsto tra { $wait }.
       *[other] { $span } sono previsti tra { $wait }.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span } va in onda tra { $wait }.
       *[other] { $span } vanno in onda tra { $wait }.
    }
show-detail-missing =
    { $count ->
        [one] L'episodio { $list } non è qui.
       *[other] Gli episodi { $list } non sono qui.
    }
show-detail-missing-more = Gli episodi { $list } e altri { $more } non sono qui.
show-detail-downloaded = Scaricato
show-detail-downloading = Download { $percent }%
show-detail-taken-adding = Preso dalla regola, in aggiunta
show-detail-taken = Preso dalla regola
show-detail-left-out = Escluso dalla regola
show-detail-subscribed-reading = Iscritto. qBittorrent sta leggendo il feed.
show-detail-taken-moving = La regola prende { $taken } { $releases ->
        [one] della { $releases } release
       *[other] delle { $releases } release
    } del feed; { $moving ->
        [one] { $moving } è
       *[other] { $moving } sono
    } in download qui sotto.
show-detail-taken-held = La regola prende { $taken } { $releases ->
        [one] della { $releases } release
       *[other] delle { $releases } release
    } del feed; { $torrents ->
        [one] { $torrents } torrent è
       *[other] { $torrents } torrent sono
    } nel client.
show-detail-taken-adding-line = La regola prende { $taken } { $releases ->
        [one] della { $releases } release
       *[other] delle { $releases } release
    } del feed; qBittorrent { $taken ->
        [one] la sta aggiungendo.
       *[other] le sta aggiungendo.
    }
show-detail-feed-holds-nothing = Il feed non contiene ancora nulla. Le release adatte si scaricano appena compaiono.
show-detail-nothing-fits = Niente nel feed rientra ancora nella regola. Le release adatte si scaricano appena compaiono.
show-detail-complete = Completa
show-detail-open-in-library = Apri in libreria
show-detail-feed-finds-nothing = Questo feed non trova nulla
show-detail-feed-empty-query = Chiede a { $source } “{ $query }”, che torna vuoto, quindi niente di nuovo arriva alla regola. Di solito una ricerca più corta risponde; le parole che toglie possono restare nella regola.
show-detail-feed-empty = Chiede a { $source }, che torna vuoto, quindi niente di nuovo arriva alla regola. Di solito una ricerca più corta risponde; le parole che toglie possono restare nella regola.
show-detail-feed-empty-earlier = qBittorrent mostra ancora quello che ha trovato una versione precedente di questo feed.
show-detail-repairing = Riparazione
show-detail-repair = Ripara
show-detail-unsubscribe = Annulla iscrizione
show-detail-remove-torrents =
    { $count ->
        [one] Rimuovi il torrent
       *[other] Rimuovi { $count } torrent
    }
show-detail-searches = Ricerche
show-detail-quoted = “{ $words }”
show-detail-whole-feed = tutto il feed
show-detail-from = Da
show-detail-rule = Regola
show-detail-rule-unfollowable = Impossibile scaricare nuovi episodi: la sua fonte risponde alle ricerche ma non ha un feed
show-detail-rule-active = Attiva
show-detail-keeps = Tiene
show-detail-keeps-value = release con “{ $words }”
show-detail-drops = Scarta
show-detail-drops-value = release con { $words }
show-detail-or = { $first } o { $second }
show-detail-last-took = Ultima presa
show-detail-saves-to = Salva in
show-detail-folder = Cartella
show-detail-in-the-feed = Nel feed
show-detail-feed-reading = Lettura del feed.
show-detail-feed-offers = Cosa offre ora il feed, dal più recente.
show-detail-feed-is-empty = Il feed è vuoto.
show-detail-read-again = Rileggi
show-detail-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeder
    }
show-detail-download = Scarica
show-detail-torrents = Torrent
show-detail-torrents-moving =
    { $count ->
        [one] { $count } in download ora; le barre si riempiono man mano.
       *[other] { $count } in download ora; le barre si riempiono man mano.
    }
show-detail-torrents-held = Cosa ha il client per questa cartella.
show-detail-torrents-waiting = In attesa che qBittorrent aggiunga quello che prende la regola.
show-detail-torrents-none = Il client non ha nulla per questa cartella.

## SeriesPage.qml

series-trackers-both = { $first } e { $second }
series-count-cleared = Numero di episodi cancellato
series-count-set = Impostato a { $count } episodi
series-held-one-undated = Episodio { $first } ancora senza data
series-held-range-undated = Episodi da { $first } a { $last } ancora senza data
series-held-one-due = Episodio { $first } previsto per { $when }
series-held-range-due = Episodi da { $first } a { $last } previsti per { $when }
series-held-month-format = MMMM yyyy
series-fix-schedule = Episodi e date
series-fix-files = File
series-fix-rescan-folder = Riscansiona cartella
series-fix-match = Abbina
series-find-match = Trova abbinamento
series-rescan-show = Riscansiona serie
series-open-in-show-info = Apri in Info serie
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } film spostato nel cestino
           *[other] { $count } film spostati nel cestino
        }
        [episode] { $count ->
            [one] { $count } episodio spostato nel cestino
           *[other] { $count } episodi spostati nel cestino
        }
       *[file] { $count ->
            [one] { $count } file spostato nel cestino
           *[other] { $count } file spostati nel cestino
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } film scelto.
           *[other] { $count } film scelti.
        }
        [episode] { $count ->
            [one] { $count } episodio scelto.
           *[other] { $count } episodi scelti.
        }
       *[file] { $count ->
            [one] { $count } file scelto.
           *[other] { $count } file scelti.
        }
    }
series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] Sposta { $count } film nel cestino
           *[other] Sposta { $count } film nel cestino
        }
        [episode] { $count ->
            [one] Sposta { $count } episodio nel cestino
           *[other] Sposta { $count } episodi nel cestino
        }
       *[file] { $count ->
            [one] Sposta { $count } file nel cestino
           *[other] Sposta { $count } file nel cestino
        }
    }
series-delete-click = Clicca i file da eliminare.
series-delete-where = Vanno nel cestino, da cui puoi recuperarli. Quello che hai visto resta tracciato.
series-delete-move = Sposta nel cestino
series-pick-none = Nessuno
series-pick-all = Tutti
series-rescan-started = Scansione avviata
series-rated = Voto { $score }
series-rated-cleared = Voto cancellato
series-score-failed = Voto non salvato
series-still-downloading = { $episode } è ancora in download
series-format-series = Serie
series-tracker-refused = { $tracker } non l'ha accettato
series-tracker-refused-why = { $tracker } non l'ha accettato: { $why }
series-watched-up-to = Visti fino a { $episode }
series-unwatched-from = Non visti da { $episode }
series-undo = Annulla
series-play-downloading = Download dell'episodio { $number }
series-play-downloading-unnumbered = Download dell'episodio
series-play-continue = Continua
series-play-again = Guarda di nuovo
series-play-continue-episode = Continua ep. { $number }
series-play-again-from = Guarda di nuovo dall'ep. { $number }
series-play-episode = Riproduci ep. { $number }
series-page-not-tracked = Non tracciata
series-not-tracked-on-disk =
    { $count ->
        [one] Non tracciata, { $count } episodio su disco
       *[other] Non tracciata, { $count } episodi su disco
    }
series-watched = { $count } visti
series-watched-of = { $watched } di { $total } visti
series-watched-of-estimate = { $watched } di { $total }+ visti
series-facts-episodes-estimate =
    { $count ->
        [one] { $count } episodio+
       *[other] { $count } episodi+
    }
series-facts-each = { $runtime } l'uno
series-facts-on-disk = { $count } su disco
series-next = Prossimo: { $episode }
series-episodes-downloading =
    { $count ->
        [one] { $count } episodio in download
       *[other] { $count } episodi in download
    }
series-all-watched = Tutti visti
series-no-poster = Nessuna locandina
series-next-season-span = S{ $season } { $span }
series-next-predicted = { $span } previsto per { $date }, tra { $wait }
series-next-date-format = ddd d MMM
series-next-airs =
    { $count ->
        [one] { $span } va in onda tra { $wait }
       *[other] { $span } vanno in onda tra { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] Download di { $count } episodio
       *[other] Download di { $count } episodi
    }
series-count-season-ends = La stagione { $season } finisce all'episodio { $number }
series-count-ends = Finisce all'episodio { $number }
series-count-gave-way = Avevi impostato { $count } episodi; non vale più
series-count-set-by-you = { $line }, impostato da te
series-change = Cambia
series-get-episodes = Scarica episodi
series-your-score = Il tuo voto { $score }
series-rate = Vota
series-page-average-score = Voto medio { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = { $votes } voti
series-add-to-list = Aggiungi a una lista
series-page-rewatched =
    { $count ->
        [one] Rivista { $count } volta
       *[other] Rivista { $count } volte
    }
series-your-notes = Le tue note
series-add-notes = Aggiungi note
series-notes-tip = Note sulla voce della tua lista, salvate su { $sites }
series-unhide = Mostra
series-hide = Nascondi
series-hidden-toast = { $title } è nascosta qui, e AniBeam smette di sincronizzarla con AniList e MyAnimeList
series-unhidden-toast = { $title } è tornata, e si sincronizza di nuovo
series-hide-tip-tracked = Nascosta da ogni pagina qui, e AniBeam smette di sincronizzarla con AniList e MyAnimeList
series-hide-tip = Nascosta da ogni lista
series-fixes = Correzioni
series-fixes-tip = Correggi file, numero di episodi, abbinamento, o riscansiona
series-quick-fix = Correzione rapida
series-film = Film
series-episodes = Episodi
series-extras = Opening, ending e altro
series-extras-openings = Opening
series-extras-endings = Ending
series-extras-previews = Anteprime e trailer
series-extras-specials = Speciali
series-extras-other = Altro
series-extras-group = { $kind }, { $count }
series-extra-files = File in più
series-extra-files-warning =
    { $count ->
        [one] Un file supera i { $total } episodi previsti: nome sbagliato, duplicato o speciale. Rinominalo o rimuovilo.
       *[other] { $count } file superano i { $total } episodi previsti: nomi sbagliati, duplicati o speciali. Rinominali o rimuovili.
    }
series-synopsis = Trama
series-less = Meno
series-more = Altro
series-match = Abbina
series-not-matched = Non ancora abbinata, quindi niente locandina, trama o tracciamento. Trova le sue info su AniList o TVmaze.
series-naming-title = Nomi che funzionano
series-naming-body = La cartella è il titolo della serie, scritto come su AniList, con la stagione dopo quando non è la prima. Ogni file porta il numero dell'episodio.
series-fix-how-files-read = Correggi la lettura dei file
series-characters = Personaggi
series-related = Correlate
series-related-filling = Ancora in caricamento da AniList.
series-tags = Tag
series-hide-spoilers = Nascondi spoiler
series-spoilers = Spoiler
series-all-tags = Tutti i tag
series-tag = { $name }, { $rank }%
series-recommendations = Consigliate
series-relation-owned = Nella tua libreria
series-relation-unreleased = Non ancora uscita
series-relation-on-anilist = Su AniList
series-score-saved = Voto salvato
series-score-cleared = Voto cancellato

## GetShow.qml

get-show-the-client = il client torrent
get-show-group-batch = Batch di { $group }
get-show-run = da { $first } a { $last }
get-show-episodes =
    { $count ->
        [one] Episodio { $list }
       *[other] Episodi { $list }
    }
get-show-episodes-inline =
    { $count ->
        [one] l'episodio { $list }
       *[other] gli episodi { $list }
    }
get-show-sending-to = Invio a { $client }
get-show-go =
    { $count ->
        [one] Scarica { $count } episodio
       *[other] Scarica { $count } episodi
    }
get-show-go-replace =
    { $count ->
        [one] Scarica { $count } episodio e sostituisci
       *[other] Scarica { $count } episodi e sostituisci
    }
get-show-go-subscribe =
    { $count ->
        [one] Scarica { $count } episodio e iscriviti
       *[other] Scarica { $count } episodi e iscriviti
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] Scarica { $count } episodio, sostituisci e iscriviti
       *[other] Scarica { $count } episodi, sostituisci e iscriviti
    }
get-show-subscribe = Iscriviti ai nuovi episodi
get-show-nothing-new = Niente di nuovo da scaricare
get-show-pick-episode = Scegli un episodio
get-show-replace-off = Off. Le scelte per { $episodes } sono escluse, perché la libreria le ha già. Attivalo per sostituirle.
get-show-replace-on = I file che hai per { $episodes } vanno nel cestino quando i nuovi sono finiti.
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } è in download ora: quel download viene annullato e quello che ha scritto va nel cestino.
       *[other] { $episodes } sono in download ora: quel download viene annullato e quello che ha scritto va nel cestino.
    }
get-show-kind-on-demand = Si avvia quando serve a una ricerca
get-show-kind-rss-search = Ricerca RSS
get-show-kind-rss-feed = Feed RSS
get-show-kind-json-search = Ricerca JSON
get-show-kind-indexer-search = Ricerca indexer
get-show-ready = Pronta
get-show-running = Attiva
get-show-not-running = Non attiva
get-show-not-running-start = Non attiva. Clicca la sua fonte qui sopra per avviarla.
get-show-switched-off = Spenta
get-show-click-to-see = Clicca per vedere cosa ha per questa serie
get-show-searching = Ricerca
get-show-searching-source = Ricerca su { $name }
get-show-searching-elapsed = { $label }, { $seconds } s
get-show-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } release
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } release, { $cover } di { $total } episodi coperti
       *[other] { $count } release, { $cover } di { $total } episodi coperti
    }
get-show-on-your-list = Nella tua lista
get-show-plan-to-watch = Da vedere
get-show-on-your-list-tip = È nella tua lista
get-show-plan-to-watch-tip = Mettila nella tua lista Da vedere
get-show-open-series-tip = La pagina della serie: dettagli, episodi e riproduzione
get-show-next-in = Episodio { $number } tra { $wait }
get-show-airing = In onda
get-show-finished = Conclusa
get-show-total-episodes = { $count } episodi
get-show-downloading-episodes =
    { $count ->
        [one] Download di { $count } episodio
       *[other] Download di { $count } episodi
    }
get-show-downloading-torrents =
    { $count ->
        [one] Download di { $count } torrent
       *[other] Download di { $count } torrent
    }
get-show-in-library = Nella tua libreria
get-show-not-in-library = Non nella tua libreria
get-show-downloading = In download
get-show-stop-all = Ferma tutto
get-show-start-all = Avvia tutto
get-show-stop-all-tip = Ferma ogni download di questa serie; quello che hanno scaricato resta
get-show-start-all-tip = Riavvia ogni download di questa serie
get-show-cancel-all = Annulla tutto
get-show-cancel-all-tip = Annulla ogni download di questa serie e sposta nel cestino quello che hanno scritto
get-show-sources = Fonti disponibili
get-show-check-again = Controlla di nuovo
get-show-check-again-tip = Chiede a ogni provider su richiesta se è attivo. Non avvia nulla.
get-show-search-source = Cerca su { $name }
get-show-release-list = Tutte le release in una lista
get-show-your-library = La tua libreria
get-show-library-on-disk = { $have } di { $total } su disco
get-show-library-downloading = { $count } in download
get-show-library-nothing = Ancora niente
get-show-row-batch = { $group }, batch
get-show-batch-counting = conteggio degli episodi, un torrent
get-show-batch-unlisted = episodi non elencati, l'intera serie dal titolo, un torrent
get-show-batch-episodes =
    { $count ->
        [one] { $count } episodio, un torrent
       *[other] { $count } episodi, un torrent
    }
get-show-nothing-new-picked = Niente di nuovo scelto
get-show-nothing-picked = Niente di scelto
get-show-pick-provider = Scegli un provider
get-show-plan-line = { $episodes } da { $groups } su { $source }.
get-show-and = { $first } e { $second }
get-show-library-has =
    { $count ->
        [one] La libreria ha { $episodes }. Attiva Sostituisci episodi attuali qui sotto per sostituirlo.
       *[other] La libreria ha { $episodes }. Attiva Sostituisci episodi attuali qui sotto per sostituirli.
    }
get-show-together = insieme
get-show-torrents-unit =
    { $count ->
        [one] torrent
       *[other] torrent
    }
get-show-replace = Sostituisci episodi attuali
get-show-sequential = Scarica in ordine
get-show-sequential-on = On: ogni nuovo torrent scarica i pezzi dall'inizio, così i primi episodi finiscono prima e puoi guardare mentre arriva il resto. Nel complesso un po' più lento.
get-show-sequential-off = Off: il client scarica i pezzi nell'ordine in cui li offre lo swarm.
get-show-rolling = Tieni pronti i prossimi
get-show-rolling-on = On: { $count } episodi si scaricano ora, e ognuno che finisci avvia il successivo.
get-show-rolling-on-delete = On: { $count } episodi si scaricano ora, e ognuno che finisci avvia il successivo. Gli episodi visti lasciano il disco.
get-show-rolling-off = Off: ogni episodio scelto si scarica subito.
get-show-follow-off = Off: i nuovi episodi compaiono qui come mancanti, e niente si scarica da solo.
get-show-follow-no-feed = { $source } non ha un feed da seguire, quindi da lì i nuovi episodi non possono scaricarsi da soli.
get-show-follow-pick-group = Prima scegli un gruppo: un'iscrizione segue un gruppo e una qualità.
get-show-follow-batch-only = Qui { $group } ha solo il batch, quindi non c'è un feed di singoli episodi a cui iscriversi.
get-show-follow-on = On: un feed per { $feed } su { $source }. Ogni nuovo episodio si scarica appena compare.
get-show-confirm =
    { $count ->
        [one] { $episodes } è in download ora. Sostituirlo annulla quel download e sposta nel cestino quello che ha scritto.
       *[other] { $episodes } sono in download ora. Sostituirli annulla quel download e sposta nel cestino quello che ha scritto.
    }
get-show-confirm-go = Annulla e sostituisci
get-show-state-error = Errore
get-show-state-files-missing = File mancanti
get-show-state-stopped = Fermato
get-show-state-waiting-peers = In attesa di peer
get-show-state-metadata = Download metadati
get-show-state-queued = In coda
get-show-state-checking = Controllo file
get-show-state-allocating = Allocazione
get-show-state-moving = Spostamento file
get-show-state-waiting = In attesa
get-show-a-batch = Un batch
get-show-no-group = Gruppo non nel nome
get-show-likely = probabilmente { $source }
get-show-tracker = tracker { $tracker }
get-show-not-known = Ancora sconosciuto
get-show-bytes-of = { $done } di { $size }
get-show-speed-down = { $speed }/s giù
get-show-time-left = { $wait } rimasti
get-show-stopping = Arresto
get-show-starting = Avvio
get-show-stop = Ferma
get-show-start-tip = Riavvia il download
get-show-stop-tip = Ferma il download; quello che ha scaricato resta
get-show-cancelling = Annullamento
get-show-cancel-tip = Annulla questo download e sposta nel cestino quello che ha scritto
get-show-episode-order = Scarica in ordine di episodio
get-show-episode-order-on = On: i pezzi arrivano dall'inizio di ogni file, così i primi episodi finiscono prima.
get-show-cube-skipped = Episodio { $number }: saltato
get-show-cube-done = Episodio { $number }: fatto
get-show-cube-percent = Episodio { $number }: { $percent }%
get-show-fact-torrent = Torrent
get-show-fact-release = Release
get-show-fact-from = Da
get-show-fact-saving-to = Salva in
get-show-span-counting = Conteggio
get-show-span-unknown = { $runs }?
get-show-span-episodes = Episodi { $runs }
get-show-cube-no-release = Episodio { $number }: nessuna release da { $group }
get-show-seeders = { $count } seeder
get-show-cube-on-disk = già su disco, clicca per sostituirlo
get-show-cube-downloading = in download ora, clicca per sostituirlo

## TorrentRow

torrent-row-state-downloading = Download
torrent-row-state-stalled = Bloccato
torrent-row-state-metadata = Download metadati
torrent-row-state-queued = In coda
torrent-row-state-stopped = Fermato
torrent-row-state-done = Fatto
torrent-row-state-seeding = In seed
torrent-row-state-checking = Controllo
torrent-row-state-moving = Spostamento
torrent-row-state-error = Errore
torrent-row-state-files-missing = File mancanti
torrent-row-state-making-room = Libero spazio
torrent-row-state-finding-peers = Ricerca peer
torrent-row-state-waiting = In attesa
torrent-row-done-of-size = { $done } di { $size }
torrent-row-speed-down = { $speed }/s giù
torrent-row-speed-up = { $speed }/s su
torrent-row-time-left = { $time } rimasti
torrent-row-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeder
    }
torrent-row-rolling = a rotazione, episodi da { $next } a { $through } di { $total }
torrent-row-stop = Ferma

## SourceRow

source-row-unavailable-fix = Collega il suo disco, poi Riscansiona. Se è stata spostata, rimuovila e aggiungi la nuova posizione.
source-row-counts = { $count } serie
source-row-counts-movies =
    { $movies ->
        [one] { $count } serie, { $movies } cartella di film
       *[other] { $count } serie, { $movies } cartelle di film
    }
source-row-unavailable = Non disponibile
    .tip = AniBeam non raggiunge questa cartella. Se è su un disco non collegato o non montato, collegalo e AniBeam la ritrova. Se è stata spostata, rimuovila qui e aggiungi la nuova posizione.
source-row-rescan = Riscansiona
source-row-remove-question = Rimuovere { $name }, { $count } serie e la loro cronologia?

## StartSourceSheet

start-source-title = Avviare { $service } per { $name }?
start-source-busy = Avvio di { $service }. { $name } si accende quando risponde; può volerci fino a un minuto.
start-source-body = { $service } è il servizio che cerca su { $name } per AniBeam. Avviarlo lo fa girare in background su questo computer, e continua a girare dopo la chiusura di AniBeam.
start-source-starting = Avvio

## SearchProgress

search-progress-why-helper = AniBeam sta avviando il suo helper di ricerca; ci vuole un secondo o due.
search-progress-why-starting = { $endpoint } passa da un helper che si avvia quando serve a una ricerca. La prima ricerca lo aspetta, fino a { $seconds } secondi; le successive saltano questo passaggio.
search-progress-why-reading = Divido le release in gruppi e le confronto con la libreria.
search-progress-why-ondemand = Un helper rilegge { $endpoint } per ogni nuova ricerca, e di solito ci vogliono da dieci a venti secondi.
search-progress-why-slow = { $endpoint } è lento a rispondere in questo momento.
search-progress-why-usual = Di solito ci vuole un secondo o due.
search-progress-why-and-slow = { $why } Ci sta mettendo più del solito; annulla per provare un'altra fonte.
search-progress-searching = Ricerca
search-progress-seconds = { $seconds } s
search-progress-try = Prova { $endpoint }

## ReleaseTable

release-table-col-release = Release
release-table-col-group = Gruppo
release-table-col-res = Ris.
release-table-col-rip = Rip
release-table-col-ep = Ep
release-table-col-size = Dim.
release-table-col-seeders = Seeder
release-table-col-leechers = Leecher
release-table-col-downloads = Download
release-table-col-released = Uscita
release-table-tick-all = Spunta tutte le release
release-table-untick-all = Togli la spunta a tutte le release
release-table-on-disk = Su disco
release-table-other-on-disk = Altra release su disco
release-table-season-batch = Batch stagione { $season }
release-table-batch = Batch
release-table-via = via { $source }
release-table-season-episode = S{ $season } { $episode }
release-table-date-format = d MMMM yyyy, HH:mm
release-table-download = Scarica
release-table-more = e altre { $count }. Restringi la lista o ordinala per vedere prima le altre.
release-table-pick = Scegli questa release

## DownloadsPage

downloads-title = Download
downloads-builtin-client = Client integrato
downloads-start-done = { $name } è attiva e accesa
downloads-disk-film = Film
downloads-disk-watched = { $size } visti
downloads-disk-all-watched = tutto visto
downloads-delete-watched-menu =
    { $count ->
        [one] Ogni episodio visto: { $count } episodio, { $size }
       *[other] Ogni episodio visto: { $count } episodi, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Ogni serie conclusa che hai visto: { $count } serie, { $size }
       *[other] Ogni serie conclusa che hai visto: { $count } serie, { $size }
    }
downloads-titles-and = { $list } e { $last }
downloads-titles-more = { $list } e altre { $count }
downloads-delete-finished-none = Nessuna serie conclusa è stata vista per intero
downloads-delete-watched-none = Niente di visto è su disco
downloads-delete-finished-heading =
    { $count ->
        [one] Eliminare { $count } serie conclusa, { $size }?
       *[other] Eliminare { $count } serie concluse, { $size }?
    }
downloads-delete-finished-base = { $titles }. Solo le serie che il provider dà per concluse, con ogni episodio visto; una serie ancora in onda, o con una fine solo stimata, resta. Ogni loro file va nel cestino, anche gli extra. Anche i loro torrent escono da { $client }, quindi si possono scaricare di nuovo.
downloads-delete-watched-heading =
    { $count ->
        [one] Eliminare { $count } episodio visto, { $size }?
       *[other] Eliminare { $count } episodi visti, { $size }?
    }
downloads-delete-watched-base = Ogni episodio che hai visto, in ogni serie. Gli episodi non ancora visti e gli extra restano. I file e i loro sottotitoli vanno nel cestino, quindi puoi recuperarli da lì. Anche i loro torrent escono da { $client }, quindi si possono scaricare di nuovo.
downloads-move-to-trash = Sposta nel cestino
downloads-remove-group-torrents =
    { $count ->
        [one] Rimuovi il torrent
       *[other] Rimuovi i suoi { $count } torrent
    }
downloads-run-span = da { $from } a { $to }
downloads-run-pair = { $first } e { $second }
downloads-gap-missing =
    { $count ->
        [one] Episodio { $runs } mancante
       *[other] Episodi { $runs } mancanti
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Episodio { $runs } non scaricato
       *[other] Episodi { $runs } non scaricati
    }
downloads-gap-then-not-downloaded = { $runs } non scaricati
downloads-trying-query = Provo { $query }
downloads-client-settings = Impostazioni client torrent
downloads-error-unknown = Qualcosa è andato storto. Riprova.
downloads-search-all-off = Tutte le fonti sono spente. Accendine una per cercare.
downloads-turn-on = Accendi { $name }
downloads-search-no-sources = Non ci sono ancora fonti in cui cercare. Aggiungi un elenco di fonti nelle Impostazioni, in Download.
downloads-search-none-started = Nessuna fonte è riuscita ad avviare la ricerca. Riprova tra un attimo.
downloads-auto-resolution = Ristretto a { $resolution }, la più nitida disponibile
downloads-search-one-failed = La ricerca su { $name } non ha funzionato. Riprova, o cerca su un'altra fonte.
downloads-search-all-failed = Nessuna fonte è riuscita a cercare. Riprova tra un attimo.
downloads-narrowing-on = Restringo su { $name }
downloads-refine-zero = Non resterebbe niente, quindi la lista resta com'è.
downloads-refine-no-match = Nessuna release corrisponde a quel termine.
downloads-refine-empty = Scrivi un termine per restringere.
downloads-refine-no-change = Non cambierebbe nulla.
downloads-movie-tick-one = Spunta una release, poi premi Scarica come film.
downloads-group-downloading = In download
downloads-group-downloading-line = In movimento ora; Attività sulla destra ha i dettagli.
downloads-group-seeding = In seed
downloads-group-seeding-line = Scaricati e ancora in condivisione.
downloads-group-done = Fatti
downloads-group-done-line = Scaricati; il client non li sta condividendo.
downloads-group-stopped = Fermati
downloads-group-stopped-line = Fermati prima di finire.
downloads-group-broken = File mancanti
downloads-group-broken-line = Il client non trova i loro file: puntano a una cartella che non c'è. Rimuovere i torrent mette in ordine il client.
downloads-sub-nofeed = Non seguibili
downloads-sub-nofeed-line = La loro fonte risponde alle ricerche ma non ha un feed, quindi non arrivano nuovi episodi. Annulla l'iscrizione, poi segui la serie su una fonte che ce l'ha.
downloads-sub-complete = Complete
downloads-sub-complete-line = Ogni episodio è nella libreria e non andrà in onda altro: queste iscrizioni non hanno più niente da fare.
downloads-sub-airing = In onda
downloads-sub-airing-line = Nuovi episodi ancora in arrivo, i più vicini per primi.
downloads-sub-incomplete = Incomplete
downloads-sub-incomplete-line = Concluse con episodi che non sono qui, o niente dice a che punto è la serie.
downloads-sub-missing = Non in libreria
downloads-sub-missing-line = La loro cartella non contiene niente che la libreria abbia trovato: non è ancora arrivato nulla, o salva dove la libreria non guarda.
downloads-torrents =
    { $count ->
        [one] { $count } torrent
       *[other] { $count } torrent
    }
downloads-rule-film-on-disk = Film, su disco
downloads-rule-all-here = Tutti i { $total } qui
downloads-rule-next =
    { $predicted ->
        [yes] Ep { $episode } previsto tra { $time }
       *[no] Ep { $episode } tra { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] Ep da { $episode } a { $through } previsti tra { $time }
       *[no] Ep da { $episode } a { $through } tra { $time }
    }
downloads-rule-have-of = { $have } di { $total } qui
downloads-rule-have =
    { $count ->
        [one] { $count } episodio qui
       *[other] { $count } episodi qui
    }
downloads-tag-not-downloaded = Non scaricato
downloads-tag-missing = { $count } mancanti
downloads-tag-n-not-downloaded = { $count } non scaricati
downloads-tag-unknown = Sconosciuto
downloads-speed-down = { $speed }/s giù
downloads-time-left = { $time } rimasti
downloads-repair-trying = Provo ricerche più corte
downloads-sent-to-client = { $title } inviato a { $client }
downloads-files-torrents = quello che hanno scaricato i torrent
downloads-files-torrents-remote = quello che hanno scaricato i torrent, eliminato da { $client }
downloads-files-folder-and = { $folder } e { $torrents }
downloads-unsubscribe-heading = Annullare l'iscrizione a { $title }?
downloads-unsubscribe-base = Il suo feed e la sua regola di download escono da { $client }, quindi non arriva nulla di nuovo.
downloads-unsubscribe-complete-heading = Annullare l'iscrizione a tutte le { $count } serie complete?
downloads-unsubscribe-complete-base = I loro feed e le loro regole di download escono da { $client }. Ogni loro episodio è nella libreria.
downloads-files-folders-and-torrents = le loro cartelle e quello che hanno scaricato i loro torrent
downloads-one-at-a-time = Vanno una alla volta; la pagina mostra ognuna man mano.
downloads-subscription-stays = { $base } L'iscrizione resta.
downloads-cancel-heading = Annullare { $title }?
downloads-cancel-group-base =
    { $count ->
        [one] Il suo download si ferma ed esce da { $client }.
       *[other] I suoi { $count } download si fermano ed escono da { $client }.
    }
downloads-files-so-far-it = quello che ha scaricato finora
downloads-files-so-far-they = quello che hanno scaricato finora
downloads-cancel-download = Annulla il download
downloads-keep-downloading = Continua il download
downloads-remove-heading = Rimuovere { $title }?
downloads-remove-group-base =
    { $count ->
        [one] Il suo torrent esce da { $client }; i file restano a meno che tu non spunti qui sotto.
       *[other] I suoi { $count } torrent escono da { $client }; i file restano a meno che tu non spunti qui sotto.
    }
downloads-remove-detail-base =
    { $count ->
        [one] Il suo { $count } torrent esce da { $client }; i file restano a meno che tu non spunti qui sotto.
       *[other] I suoi { $count } torrent escono da { $client }; i file restano a meno che tu non spunti qui sotto.
    }
downloads-remove-broken-heading = Rimuovere i torrent di tutte le { $count } serie con file mancanti?
downloads-remove-broken-base = I loro { $count } torrent escono da { $client }. Puntano a cartelle che non ci sono, quindi nessun file viene toccato; le iscrizioni restano.
downloads-remove-torrent-heading = Rimuovere questo torrent?
downloads-files-it-downloaded = quello che ha scaricato
downloads-cancel-torrent-heading = Annullare questo download?
downloads-cancel-many-heading =
    { $count ->
        [one] Annullare { $count } download?
       *[other] Annullare { $count } download?
    }
downloads-cancel-many-base = Ogni download di questa serie si ferma ed esce da { $client }.
downloads-cancel-all = Annullali tutti
downloads-summary-connecting = Connessione.
downloads-summary-client-on = { $client } { $version } è collegato.
downloads-summary-client-off = { $client } non è collegato. Controlla indirizzo e password nelle Impostazioni, in Download.
downloads-summary-native = AniBeam cerca da solo nelle tue fonti.
downloads-summary-native-with = AniBeam cerca da solo nelle tue fonti. { $client }
downloads-summary-starting = Avvio dei download.
downloads-summary-not-running = I download non sono attivi.
downloads-summary-not-started = I download non sono ancora partiti.
downloads-summary-helper-ready = L'helper di ricerca { $version } è pronto. { $client }
downloads-subscribed-as = Iscritto come { $name }.
downloads-repaired =
    { $count ->
        [one] { $name } riparata: il feed trova { $count } release
       *[other] { $name } riparata: il feed trova { $count } release
    }
downloads-repaired-keeps =
    { $count ->
        [one] { $name } riparata: il feed trova { $count } release e la regola tiene “{ $term }”
       *[other] { $name } riparata: il feed trova { $count } release e la regola tiene “{ $term }”
    }
downloads-sent-torrents =
    { $count ->
        [one] { $count } torrent inviato a { $client }.
       *[other] { $count } torrent inviati a { $client }.
    }
downloads-added-torrents =
    { $count ->
        [one] { $count } torrent aggiunto.
       *[other] { $count } torrent aggiunti.
    }
downloads-trashed-paths =
    { $count ->
        [one] { $count } percorso di { $title } spostato nel cestino
       *[other] { $count } percorsi di { $title } spostati nel cestino
    }
downloads-show-gone = Quella serie non è più nella libreria
downloads-activity-hide = Nascondi attività
downloads-activity = Attività
downloads-moving =
    { $count ->
        [one] { $count } download in corso
       *[other] { $count } download in corso
    }
downloads-back = Torna ai Download
downloads-chip-search-ready = Ricerca pronta
downloads-chip-search-starting = Avvio ricerca
downloads-chip-search-off = Ricerca spenta
downloads-chip-answers-at = { $summary } Risponde su { $url }.
downloads-chip-helper-missing = L'helper di ricerca non è installato. Clicca per le impostazioni dei Download, che spiegano come aggiungerlo
downloads-chip-helper-failed = L'helper di ricerca si è fermato. Ricollega lo riavvia; le impostazioni dei Download dicono cosa è andato storto
downloads-chip-open-settings = Apri le impostazioni dei Download
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } spento
downloads-chip-client-not-connected = { $client } non è collegato. Ricollega, o controlla il suo indirizzo nelle Impostazioni
downloads-chip-client-remote = Un helper di ricerca gestito da te usa il client indicato nelle sue impostazioni
downloads-chip-client-switch = Clicca per cambiare client
downloads-connecting = Connessione
downloads-reconnect = Ricollega
downloads-search-placeholder = Cerca una serie, o il nome di una release
downloads-search-go-tip = Cerca nomi di release su { $names }
downloads-search-go-off = Tutte le fonti sono spente.
downloads-suggest-airing =
    { $predicted ->
        [yes] In onda, episodio { $episode } previsto tra { $time }
       *[no] In onda, episodio { $episode } tra { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] In onda, episodi da { $episode } a { $through } previsti tra { $time }
       *[no] In onda, episodi da { $episode } a { $through } tra { $time }
    }
downloads-suggest-finished = Conclusa
downloads-suggest-episodes = { $count } episodi
downloads-suggest-in-library = Nella tua libreria
downloads-suggest-on-anilist = Su AniList, non nella tua libreria
downloads-plan-to-watch = Da vedere
downloads-suggest-release-search = Cerca invece nomi di release per “{ $query }” su { $names }
downloads-suggest-release-search-none = Cerca invece nomi di release per “{ $query }” su nessuna fonte
downloads-key-return = Invio
downloads-sources = Fonti
downloads-source-hint-starting = avvio
downloads-source-hint-down = non attiva
downloads-source-tip-down = { $name } richiede un servizio helper che non è attivo. Clicca per avviarlo.
downloads-source-tip-on = Le ricerche interrogano { $name }. Clicca per spegnerla.
downloads-source-tip-off = Spenta. Clicca per interrogare di nuovo { $name }.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Controlla
    .tip = Chiede se ogni fonte su richiesta è attiva. Non avvia nulla.
downloads-check-busy = Controllo
downloads-back-to-grid = Torna alla griglia
    .tip = Torna a { $title } e a quello che ha trovato ogni provider. Esc fa lo stesso.
downloads-releases-on =
    { $count ->
        [one] { $count } release su { $name }
       *[other] { $count } release su { $name }
    }
downloads-found-on = Trovate su
downloads-found-searching = ricerca
downloads-found-failed = non riuscita
downloads-found-cancelled = annullata
downloads-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } release
    }
downloads-found-retry-tip =
    { $why }
    Clicca per riprovare
downloads-facet-groups = Gruppo
downloads-facet-quality = Qualità
downloads-facet-words = Parole
downloads-filters-fewer = Meno filtri
downloads-filters-more = Altri filtri ({ $count })
downloads-narrow-placeholder = Restringi con un termine, o -termine per escludere
downloads-best-match = Migliore
    .tip = Tiene le release simili alla migliore: { $query }
downloads-best-match-none = Nessuna release spicca qui
downloads-strip-have-of = { $have } di { $total } episodi su disco
downloads-strip-have =
    { $count ->
        [one] { $count } episodio su disco
       *[other] { $count } episodi su disco
    }
downloads-strip-next =
    { $predicted ->
        [yes] episodio { $episode } previsto tra { $time }
       *[no] episodio { $episode } tra { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] episodi da { $episode } a { $through } previsti tra { $time }
       *[no] episodi da { $episode } a { $through } tra { $time }
    }
downloads-strip-complete = completa
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = Nessuna di queste release è per un episodio che hai già.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } release qui è il file che hai, segnata Su disco
       *[other] { $count } release qui sono il file che hai, segnate Su disco
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } release è per un episodio che hai da un'altra release
       *[other] { $count } release sono per un episodio che hai da un'altra release
    }
downloads-strip-both = { $first }; { $second }.
downloads-tick-missing =
    { $count ->
        [one] Spunta { $count } episodio mancante
       *[other] Spunta i { $count } episodi mancanti
    }
    .tip = Per ogni episodio qui che la libreria non ha, la release con più seeder
downloads-open-in-library = Apri in libreria
downloads-not-this-show = Non è questa serie
downloads-no-releases = Nessuna release
downloads-no-releases-body = Niente su { $name } corrisponde a “{ $query }”. Un'altra fonte, o meno parole, potrebbero trovarla.
downloads-try-source = Prova { $name }
downloads-ticked =
    { $count ->
        [one] { $count } release spuntata
       *[other] { $count } release spuntate
    }
downloads-nothing-ticked = Niente di spuntato
downloads-ticked-size = { $size } in tutto.
downloads-tick-hint = Spunta le release nella tabella, o scaricale tutte.
downloads-tick-all = Spunta tutto
downloads-working = Attendi
downloads-download-ticked =
    { $count ->
        [one] Scarica { $count } release
       *[other] Scarica { $count } release
    }
downloads-download-shown = Scarica le { $count } mostrate
downloads-download-all = Scarica tutte e { $count }
downloads-download-movie = Scarica come film
    .tip = Una release spuntata, direttamente nella cartella dei film
downloads-subscribe = Iscriviti
downloads-subscribe-body = Le nuove release che corrispondono a questa ricerca si scaricano appena compaiono, nella cartella indicata qui.
downloads-name-placeholder = Preso dalla prima release
downloads-subscribe-search = Iscriviti a questa ricerca
downloads-subscribe-no-feed = Qui non c'è un feed a cui iscriversi
downloads-preview-again = Mostra di nuovo la regola
downloads-preview-show = Mostra la regola che crea
downloads-preview-from = Da { $host }
downloads-preview-from-named = Da { $host } ({ $name })
downloads-preview-must = Deve contenere { $terms }
downloads-preview-must-not = Non deve contenere { $terms }
downloads-helper-changed = I download usano un helper che non risponde
downloads-helper-missing = Questa copia di AniBeam è senza download
downloads-helper-failed = Impossibile avviare i download
downloads-helper-changed-body = Ricerche e download passano da un piccolo programma helper. Le Impostazioni ne indicano uno diverso da quello di AniBeam, e non risponde. Torna a quello di AniBeam e si avvia da solo.
downloads-helper-missing-body = È stata compilata senza il programma helper da cui passano ricerche e download. Gli installer nella pagina delle release lo includono. Impostazioni, Download spiega come aggiungerlo a questa copia.
downloads-helper-failed-body = Il programma helper da cui passano ricerche e download si è fermato prima di essere pronto. Riavviarlo di solito risolve.
downloads-helper-use-own = Usa quello di AniBeam
downloads-helper-start-again = Riavvia
downloads-helper-show-how = Mostrami come
downloads-helper-what-went-wrong = Cosa è andato storto
downloads-lens-gaps-count = Mancanti in libreria  { $count }
downloads-lens-airing-count = In onda  { $count }
downloads-lens-following-count = Seguite  { $count }
downloads-lens-downloaded-count = Scaricate  { $count }
downloads-lens-gaps = Mancanti in libreria
downloads-lens-gaps-line = Episodi usciti che non sono su disco. Apri una serie per vedere chi li ha.
downloads-lens-gaps-none = Non manca niente: ogni episodio uscito di ogni serie è su disco.
downloads-lens-airing-line = Serie nella tua libreria con episodi ancora in arrivo. Aprine una per seguirla.
downloads-lens-airing-none = Niente nella tua libreria è in onda
downloads-lens-airing-none-body = Una serie arriva qui quando è nella tua libreria e ha episodi ancora in arrivo. In onda ora elenca tutto quello che va in onda.
downloads-airing-now = In onda ora
downloads-lens-following = Seguite
downloads-card-next =
    { $predicted ->
        [yes] Episodio { $episode } previsto tra { $time }
       *[no] Episodio { $episode } tra { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Episodi da { $episode } a { $through } previsti tra { $time }
       *[no] Episodi da { $episode } a { $through } tra { $time }
    }
downloads-rules-unread = Impossibile leggere le tue iscrizioni.
downloads-lens-following-line = I feed che AniBeam controlla, mostrati come le serie che riempiono.
downloads-none-yet = Ancora nessuna.
downloads-reading = Lettura.
downloads-rules-error = AniBeam non è riuscito a leggere le tue iscrizioni. Riprova tra un attimo.
downloads-removing-to-go = Rimozione, ne mancano { $count }
downloads-unsubscribe-all = Annulla tutte e { $count }
downloads-unsubscribe = Annulla iscrizione
downloads-no-rules = Nessuna iscrizione
downloads-no-rules-body = Cerca qui sopra e iscriviti per continuare a ricevere una serie.
downloads-search = Cerca
downloads-lens-downloaded = Scaricate
downloads-dl-line = { $size } su disco. { $rest }
downloads-dl-client-off = { $client } non è collegato.
downloads-dl-unread = Impossibile leggere la lista dei download.
downloads-dl-in-client =
    { $count ->
        [one] { $count } torrent, { $size } in { $client }.
       *[other] { $count } torrent, { $size } in { $client }.
    }
downloads-usage-watched = Visti
downloads-usage-not-watched = Non visti
downloads-deleting = Eliminazione
downloads-delete-watched = Elimina visti
downloads-remove-all = Rimuovi tutti e { $count }
downloads-on-disk = Su disco
downloads-on-disk-line = Nella tua libreria senza torrent in { $client }, prima i più grandi. Clic destro su uno per eliminarlo.
downloads-nothing-downloaded = Niente di scaricato
downloads-nothing-downloaded-body = Quello che scarichi o a cui ti iscrivi compare qui, una serie alla volta.
downloads-client-off-title = { $client } non è collegato
downloads-client-off-body = Controlla che sia in esecuzione, poi ricollega. Indirizzo e password sono nelle Impostazioni, in Download.
downloads-connect-again = Ricollega
downloads-view-grid = Griglia
downloads-view-list = Lista
downloads-chip-subscribed = Iscritto
downloads-speed-up = { $speed }/s su
downloads-broken-some = { $count } con file mancanti
downloads-tag-watched = Visto

## PlayerDelays.qml

player-delays-title = Ritardo
player-delays-subtitles = Sottotitoli
player-delays-audio = Audio
player-delays-subtitles-earlier = Sottotitoli 0,1 s prima
player-delays-subtitles-later = Sottotitoli 0,1 s dopo
player-delays-audio-earlier = Audio 0,1 s prima
player-delays-audio-later = Audio 0,1 s dopo
player-delays-reset = Torna a 0 fino al file successivo
player-delays-none = Nessun ritardo

## TrackPicker.qml

track-picker-off = Off

## RollControls.qml

roll-controls-window = { $ahead } pronti: episodi da { $next } a { $through } di { $total }
roll-controls-window-watched = { $ahead } pronti: episodi da { $next } a { $through } di { $total }, { $watched } visti
roll-controls-more = Scarica altri { $count }
roll-controls-more-tip = Scarica subito i prossimi episodi. AniBeam torna a tenerne pronti alcuni quando sei in pari.
roll-controls-fewer-tip = Tienine uno in meno pronto
roll-controls-ahead = { $count } pronti
roll-controls-further-tip = Tienine uno in più pronto
roll-controls-all = Scarica tutti
roll-controls-all-tip = Smetti di tenerne pochi pronti e scarica ogni episodio rimasto
roll-controls-all-question = Scaricare tutti i { $count } episodi rimasti?

## PlayerChrome.qml

player-chrome-delays = Ritardo sottotitoli e audio
player-chrome-audio = Traccia audio
player-chrome-subtitles = Sottotitoli
player-chrome-subtitles-off = Sottotitoli off
player-chrome-casting-to = Trasmissione a { $device }
player-chrome-cast = Trasmetti a una TV
player-chrome-mark-watched = Segna come visto
player-chrome-shortcuts = Scorciatoie da tastiera
player-chrome-fullscreen = Schermo intero
player-chrome-exit-fullscreen = Esci da schermo intero
player-chrome-stay = Resta
player-chrome-skip-intro = Salta intro
player-chrome-skip-outro = Salta outro
player-chrome-playing-on = In riproduzione su { $device }
player-chrome-replay = Rivedi

## PlayerPage.qml

player-title = Player
player-next = Prossimo
player-extra-no-next = Un extra non ha un episodio successivo
player-last-episode = Questo è l'ultimo episodio
player-extra-no-previous = Un extra non ha un episodio precedente
player-first-episode = Questo è il primo episodio
player-resumed-at = Ripreso da { $time }
player-start-over = Ricomincia
player-audio-track = Audio: { $track }
player-subtitles-track = Sottotitoli: { $track }
player-subtitles-on = Sottotitoli: on
player-subtitles-off = Sottotitoli off
player-no-subtitles = Questo file non ha sottotitoli
player-delay-seconds = { $value } s
player-audio-delay = Ritardo audio { $value }
player-subtitle-delay = Ritardo sottotitoli { $value }
player-volume = Volume { $volume }
player-speed = Velocità { $speed }×
player-not-while-casting = Non durante la trasmissione
player-hold-fast = 2× tenendo premuto
player-no-chapters = Questo file non ha capitoli
player-last-chapter = Questo è l'ultimo capitolo
player-chapter = Capitolo { $number }
player-chapter-titled = Capitolo { $number }: { $title }
player-screenshot-saved = Salvato in Immagini/AniBeam
player-skipped-intro = Intro saltata
player-skipped-outro = Outro saltata
player-undo = Annulla
player-refusal-hidden = la serie è nascosta
player-refusal-no-match = la serie non è ancora abbinata; abbinala in Info serie
player-refusal-not-newer = la tua lista è già oltre questo episodio
player-refusal-extra = è un extra
player-refusal-unmatched = il file non è abbinato a un episodio
player-refusal-on-disk = il file è solo su disco
player-not-tracked-because = Non tracciato: { $reason }.
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = Impossibile aggiornare { $tracker }
player-outcome-ok = { $tracker } ok
player-outcome-at = { $tracker } a { $progress }
player-outcome-tracked = Tracciato
player-outcome-not-tracked = Non tracciato
player-outcome-could-not-update = Impossibile aggiornare la tua lista
player-outcome-tracked-list = Tracciato: { $list }
player-outcome-not-tracked-list = Non tracciato: { $list }
player-outcome-failed-list = Impossibile aggiornare la tua lista: { $list }
player-rated = Voto { $score }
player-score-failed = Impossibile salvare il voto
player-frame = Fotogramma { $number } a { $time }
player-audio-title = Audio
player-subtitles-title = Sottotitoli
player-cast-to = Trasmetti a
player-cast-looking = Ricerca di Chromecast e TV
player-cast-unreachable = Impossibile raggiungere la TV
player-cast-tv = TV
player-cast-stop = Interrompi la trasmissione a { $device }
player-cast-connecting = Connessione
player-cast-loading = Caricamento
player-cast-playing = In riproduzione
player-cast-paused = In pausa
player-cast-finished = Terminato
player-cast-stopped = Fermato
player-rate-prompt = Ultimo episodio tracciato. Vuoi votare questa serie?
player-rate = Vota
player-not-now = Non ora

## EndpointReposPanel.qml

endpoint-repos-panel-title = Elenchi di fonti
endpoint-repos-panel-helper = Un elenco di fonti è un insieme firmato di posti in cui cercare, aggiunto dal suo indirizzo. AniBeam controlla la firma e ti mostra la chiave prima di salvare qualsiasi cosa, poi installi le fonti che vuoi. Quelle installate si aggiungono a Fonti di ricerca qui sopra.
endpoint-repos-panel-could-not-start = AniBeam non è riuscito ad avviarlo. Riprova tra un attimo.
endpoint-repos-panel-needs-value = { $name } richiede un valore
endpoint-repos-panel-values-of = Valori di { $name }
endpoint-repos-panel-secret-note = Salvato nell'archivio segreto di questa macchina, mai nella libreria o in un'esportazione.
endpoint-repos-panel-secret-note-keep = Salvato nell'archivio segreto di questa macchina, mai nella libreria o in un'esportazione. Un campo lasciato vuoto mantiene il valore salvato.
endpoint-repos-panel-asks-again = { $name } versione { $version } chiede di nuovo
endpoint-repos-panel-update-sends-to = La nuova versione lo invia a { $host }.
endpoint-repos-panel-fields-changed = I suoi campi sono cambiati.
endpoint-repos-panel-needs-fields = Richiede { $fields } prima di cercare.
endpoint-repos-panel-no-longer-offered = Non più offerta dal suo elenco di fonti.
endpoint-repos-panel-update-requests-to = L'aggiornamento invia richieste a { $host }.
endpoint-repos-panel-unreadable = Impossibile leggere questo elenco. Aggiorna elenchi riprova.
endpoint-repos-panel-add-list = Aggiungi un elenco di fonti
endpoint-repos-panel-add-list-helper = L'indirizzo del suo file indice, o il percorso di uno su questa macchina.
endpoint-repos-panel-checking = Controllo
endpoint-repos-panel-add = Aggiungi
endpoint-repos-panel-trust-question = Ti fidi di { $name }?
endpoint-repos-panel-fingerprint = Impronta della chiave  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    { $count } fonti, firmate. Le sue fonti decidono dove vanno le tue ricerche; fidati di un elenco solo se viene da qualcuno di cui ti fidi.
endpoint-repos-panel-preview-trusted =
    { $url }
    { $count } fonti, firmate, già fidate. Le sue fonti decidono dove vanno le tue ricerche; fidati di un elenco solo se viene da qualcuno di cui ti fidi.
endpoint-repos-panel-trust = Fidati
endpoint-repos-panel-saving = Salvataggio
endpoint-repos-panel-install = Installa
endpoint-repos-panel-update = Aggiorna
endpoint-repos-panel-added-by-you = aggiunta da te
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Aggiorna a v{ $version }
endpoint-repos-panel-add-value = Aggiungi il valore
endpoint-repos-panel-change = Cambia { $fields }
endpoint-repos-panel-asks-for = Chiede { $fields }.
endpoint-repos-panel-by-hand = Aggiungine una a mano
endpoint-repos-panel-by-hand-helper = Per siti senza elenco: qualsiasi feed con {"{"}query{"}"} al posto della ricerca, anche da un proxy. Una chiave nell'indirizzo resta a parte, nel portachiavi di sistema.
endpoint-repos-panel-name = Nome
endpoint-repos-panel-refreshing = Aggiornamento
endpoint-repos-panel-refresh = Aggiorna elenchi

## ScheduleSheet.qml

schedule-episodes-range = Episodi da { $first } a { $last }
schedule-day-format = ddd d MMM yyyy
schedule-month-format = MMMM yyyy
schedule-tile-day-format = d MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = Nel corso del { $year }
schedule-no-date-yet = Ancora senza data
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = Nessuna data
schedule-tile-last = Ultimo
schedule-tile-out = Uscito
schedule-ends-at = Finisce all'episodio { $number }
schedule-dated-by-site = { $count } datati da { $site }
schedule-dated-by-record = { $count } datati dalla voce
schedule-set-by-you = { $count } impostati da te
schedule-guessed-weekly = { $count } stimati settimanali
schedule-nothing-set = Ancora niente impostato
schedule-saved = Salvato
schedule-cleared = Note sul calendario cancellate
schedule-title = Episodi e date
schedule-season = Stagione { $number }
schedule-ask-agent = Chiedi a { $agent }
schedule-done-pasting = Fine
schedule-paste-list = Incolla una lista
schedule-agent-looking = { $agent } sta cercando le date. Non si salva niente finché non premi Salva.
schedule-agent-stopped = { $agent } si è fermato: { $error }
schedule-stop = Ferma
schedule-paste-placeholder = Incolla una lista di uscite, una tabella o un post
schedule-paste-helper = Quello che AniBeam legge compare sugli episodi qui sotto.
schedule-agent-hint = Con Claude Code su questo computer, AniBeam può chiedergli di cercare le date per te.
schedule-show-more = Mostra altri episodi
schedule-pick-hint = Clicca un episodio per dargli una data. Maiusc+clic sceglie una serie di episodi.
schedule-picked-one = Episodio { $number }:
schedule-picked = { $count ->
    [one] { $count } episodio:
   *[other] { $count } episodi:
}
schedule-date-placeholder = 7 gennaio, marzo 2027, 2027
schedule-set-date = Imposta data
schedule-weekly = Settimanale
schedule-not-last = Non l'ultimo
schedule-last-episode = Ultimo episodio
schedule-clear-date = Cancella data

## ShowPage.qml

show-title = Serie
show-runtime-each = { $runtime } l'uno
show-add-to-list = Aggiungi a lista
show-moving-to = Spostamento in { $list }
show-moved-to = Spostata in { $list }
show-relation-related = Correlata
show-no-poster = Nessuna locandina
show-reading-anilist = Lettura da AniList
show-not-in-library = Non nella tua libreria
show-get-episodes = Scarica episodi
show-trailer = Trailer
show-your-score = Il tuo voto { $score }
show-rate-it = Votala
show-anilist-score = AniList { $score }
show-forecast = { $percent }% per te
show-because-you-watched = Perché hai visto
show-hidden-loved = { $count ->
    [one] Dietro c'è un tag nascosto che ami
   *[other] Dietro ci sono { $count } tag nascosti che ami
}
show-you-gave-it = Le hai dato { $score }
show-story = Trama
show-genres-tags = Generi e tag
show-studios = Studi  { $studios }
show-related = Correlate
show-recommendations = Consigliate
show-from-anilist = Da AniList
show-reading-rest = Lettura del resto da AniList

## ConnectSheet.qml

connect-connected-to = Collegato a { $name }
connect-connect-to = Collega { $name }
connect-did-not-finish = L'accesso non è terminato
connect-finish-in-browser = Completa nel browser
connect-signed-in-anilist = Accesso come { $account }. La tua lista è nella pagina In visione.
connect-signed-in-mal = Accesso come { $account }. Gli episodi vengono segnati lì mentre guardi.
connect-setup-lead = { $name } fa accedere AniBeam tramite una tua app. Ci vuole circa un minuto, una volta sola.
connect-page-open = La pagina di accesso di { $name } è aperta nel browser. Approva AniBeam lì e questa finestra si chiude da sola.
connect-page-opening = Apertura della pagina di accesso di { $name } nel browser.
connect-open-watching = Apri In visione
connect-done = Fatto
connect-open-again = Riapri la pagina di accesso
connect-secret-where = È la stringa lunga sotto "Client Secret" nella pagina dell'app su myanimelist.net/apiconfig.
connect-client-secret = Client secret
connect-save-try-again = Salva e riprova
connect-change-client-id = Cambia il Client ID
connect-step-create-anilist = Crea il client su AniList
connect-step-create-mal = Crea l'app su MyAnimeList
connect-step-create-anilist-body = Apri le impostazioni sviluppatore di AniList, crea un nuovo client e incolla questo indirizzo nel campo Redirect URL esattamente com'è qui.
connect-step-create-mal-body = Apri la pagina API di MyAnimeList, crea un ID con App Type "other" e incolla questo indirizzo nel campo App Redirect URL esattamente com'è qui.
connect-open-developer-page = Apri la pagina sviluppatore
connect-copy-address = Copia l'indirizzo
connect-step-paste-id = Incolla l'ID dell'app
connect-step-paste-id-anilist = AniList lo chiama Client ID: il numero nella pagina del tuo nuovo client.
connect-step-paste-id-mal = MyAnimeList lo chiama Client ID: la stringa lunga nella pagina della tua nuova app.
connect-client-id = Client ID
connect-client-secret-web = Client secret (solo per app Web)
connect-step-connect-body = Si apre il browser, { $name } ti chiede di accedere e approvare AniBeam, e AniBeam riceve la risposta.

## SettingsPage

settings-title = Impostazioni
settings-search-placeholder = Cerca impostazioni
settings-section-library = Libreria
settings-section-accounts = Account
settings-section-appearance = Aspetto
settings-section-playback = Riproduzione
settings-section-subtitles = Sottotitoli
settings-section-keys = Tastiera
settings-section-downloads = Download
settings-section-phone = App telefono
settings-section-storage = Spazio e backup
settings-section-help = Aiuto
settings-section-about = Info
settings-section-developer = Sviluppatore
settings-status-library-empty = Ancora nessuna cartella
settings-status-library =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serie
    } in { $folders ->
        [one] { $folders } cartella
       *[other] { $folders } cartelle
    }
settings-status-library-missing =
    { $gone ->
        [one] { $gone } cartella non trovata
       *[other] { $gone } cartelle non trovate
    }, { $count ->
        [one] { $count } serie
       *[other] { $count } serie
    } in { $folders ->
        [one] { $folders } cartella
       *[other] { $folders } cartelle
    }
settings-status-accounts-anilist = Sincronizza con AniList
settings-status-accounts-mal = Sincronizza con MyAnimeList
settings-status-accounts-both = Sincronizza con AniList e MyAnimeList
settings-status-accounts-none = Nessuna lista collegata
settings-status-appearance =
    { $mode ->
        [system] Segue il sistema
        [light] Chiaro
       *[dark] Scuro
    }, { $source ->
        [theme] colori del tema
       *[desktop] colori del desktop
    }
settings-status-playback =
    { $skip ->
        [both] Salta intro e outro
        [intro] Salta le intro
        [outro] Salta le outro
       *[none] Riproduce tutto
    }
settings-status-playback-audio =
    { $skip ->
        [both] Salta intro e outro
        [intro] Salta le intro
        [outro] Salta le outro
       *[none] Riproduce tutto
    }, audio { $languages }
settings-status-subtitles =
    Dimensione { $percent } per cento, { $override ->
        [AsScripted] file con stile come sono
        [Force] il tuo stile su ogni file
       *[ScaleOnly] i file con stile mantengono il loro aspetto
    }
settings-status-keys = Ogni tasto a cui AniBeam risponde
settings-status-downloads-unread = Non ancora letto
settings-status-downloads-answered = qBittorrent ha risposto
settings-status-downloads-missing = Helper di ricerca non installato
settings-status-downloads-unconnected = Non ancora collegato
settings-status-phone-shared = Condivisa sulla tua tailnet
settings-status-phone-off = Non condivisa
settings-status-storage = { $size } di immagini in cache
settings-status-storage-unread = Immagini, esportazione e importazione
settings-status-help = Risposte alle domande più frequenti
settings-status-about-update = È uscita la versione { $version }
settings-status-developer = Misura ogni pagina
settings-status-developer-last = { $page } creata in { $ms } ms
settings-results =
    { $count ->
        [one] { $count } impostazione trovata
       *[other] { $count } impostazioni trovate
    }
settings-results-none = Nessun risultato
settings-results-hint = Invio apre la prima.
settings-results-none-hint = Prova una parola più semplice: tema, sottotitoli, tracker, password, cartella.

settings-find-folders =
    .words = aggiungi cartella scansione riscansiona rimuovi fonte percorso collezione
settings-find-matching =
    .words = metadati info serie provider anilist tvmaze ordine abbina abbinamento
settings-find-hidden-shows =
    .words = nascoste nascondi mostra
settings-find-hidden-airing =
    .words = non mi interessa in onda mostra di nuovo
settings-find-calendar =
    .words = calendario ics ical in onda programmazione iscriviti
settings-find-anime-lists =
    .words = anilist myanimelist mal accedi collega login account scollega tracker
settings-find-main-list =
    .words = progressi schede principale tracker
settings-find-sync-now =
    .words = aggiorna progressi liste tracker sincronizza
settings-find-on-hold =
    .words = in pausa sospese inattive ferme giorni
settings-find-language =
    .words = lingua traduzione locale inglese italiano
settings-find-mode =
    .words = scuro chiaro sistema tema notte
settings-find-colours-from =
    .words = terminale desktop sistema tema fonte kitty
settings-find-dark-theme =
    .words = colore schema base16
settings-find-light-theme =
    .words = colore schema base16
settings-find-accent =
    .words = colore evidenziazione accento
settings-find-text-size =
    .words = font carattere parole più grande più piccolo zoom scala leggere accessibilità
settings-find-density =
    .words = compatto comodo spaziatura dimensione
settings-find-poster-size =
    .words = schede piccole medie grandi locandine
settings-find-out-now =
    .words = in onda usciti ora schede lista righe stile banner locandine
settings-find-out-now-cards =
    .words = in onda usciti ora griglia colonne righe layout
settings-find-reduce-motion =
    .words = animazioni accessibilità movimento
settings-find-app-icon =
    .words = logo launcher dock barra applicazioni finestra scuro icona
settings-find-skip-intros =
    .words = opening op automatico salta aniskip capitoli intro
settings-find-skip-outros =
    .words = ending ed titoli di coda automatico salta aniskip outro
settings-find-mpv-settings =
    .words = mpv.conf configurazione avanzate
settings-find-discord =
    .words = discord rich presence stato profilo
settings-find-next-episode =
    .words = prossimo episodio pulsante titolo riproduzione automatica fine
settings-find-subtitle-languages =
    .words = lingua traccia sottotitoli sub
settings-find-audio-languages =
    .words = lingua traccia doppiaggio dub giapponese audio
settings-find-start-over =
    .words = ripristina predefinito stile sottotitoli
settings-find-size =
    .words = scala dimensione sottotitoli
settings-find-position =
    .words = altezza margine posizione sottotitoli
settings-find-styled =
    .words = ass sovrascrivi forza stile
settings-find-font =
    .words = carattere font
settings-find-text-colour =
    .words = colore testo
settings-find-outline =
    .words = bordo contorno
settings-find-background =
    .words = riquadro sfondo opacità
settings-find-bold =
    .words = grassetto peso
settings-find-keys-player =
    .words = tastiera scorciatoie tasti hotkey
settings-find-keys-player-around =
    .words = tastiera scorciatoie schermo intero prossimo precedente episodio indietro
settings-find-keys-around =
    .words = tastiera scorciatoie indietro avanti clic destro menu schermo intero
settings-find-keys-anywhere =
    .words = tastiera scorciatoie tasti hotkey impostazioni esci
settings-find-torrent-client =
    .words = qbittorrent password nome utente indirizzo webui seedbox client torrent
settings-find-search-helper =
    .words = anirss rss feed token comando server avanzate helper
settings-find-next-few =
    .words = rotazione episodi pronti elimina visti tieni disco spazio
settings-find-unsubscribe =
    .words = annulla iscrizione feed concluse finite
settings-find-last-search =
    .words = download ricerca ricorda
settings-find-search-sources =
    .words = endpoint feed configurazione anirss fonti
settings-find-source-lists =
    .words = repo endpoint firmati elenchi fonti jackett prowlarr
settings-find-share-tailscale =
    .words = telefono android tailnet remoto host condividi tailscale
settings-find-stream =
    .words = streaming riproduci trasmetti episodi telefono file
settings-find-devices =
    .words = fidati fiducia spegni blocca dispositivo tailscale
settings-find-back-up =
    .words = esporta backup file json
settings-find-restore =
    .words = importa ripristina file json
settings-find-image-cache =
    .words = locandine cancella immagini disco cache
settings-find-files =
    .words = database dati configurazione cache cartella percorso
settings-find-help-folders =
    .words = aiuto cartella struttura nomi episodi stagioni film file
settings-find-help-recognised =
    .words = aiuto abbina sbagliata serie non trovata non abbinata riconosciuta
settings-find-help-anilist =
    .words = aiuto anilist myanimelist mal accedi collega login tracker
settings-find-help-downloads =
    .words = aiuto download vuoto fonti niente torrent qbittorrent
settings-find-help-data =
    .words = aiuto dati file backup database privacy
settings-find-help-else =
    .words = aiuto configurazione onboarding guida segnala bug problema
settings-find-version =
    .words = build novità changelog versione
settings-find-setup =
    .words = primo avvio guida onboarding ripeti configurazione
settings-find-updates =
    .words = stabile nightly canale controlla aggiornamenti
settings-find-developer-options =
    .words = debug tempi sviluppatore
settings-find-ratings =
    .words = imdb crediti voti

## SettingsDownloadsTab

settings-downloads-title = Download
settings-downloads-lead = Dove AniBeam invia quello che scarichi, e come cerca nei feed delle release.
settings-downloads-keep-none = Finire un episodio elimina quello prima.
settings-downloads-keep-line = Episodi visti tenuti prima di quello appena finito. Con { $count }, finire l'episodio { $finishing } elimina l'episodio 1.
settings-downloads-anirss-ready = Attivo, versione { $version } su { $url }.
settings-downloads-anirss-starting = Avvio dell'helper di ricerca.
settings-downloads-anirss-missing = L'helper di ricerca non è installato. Reinstalla AniBeam, o indica il tuo anirss in Comando.
settings-downloads-anirss-failed = L'helper di ricerca si è fermato. Riprova lo avvia; Dettagli mostra cosa ha detto.
settings-downloads-anirss-idle = Non ancora avviato. Parte la prima volta che cerchi nella pagina Download.
settings-downloads-torrent-trying = Provo { $client }.
settings-downloads-torrent-answered = { $client } { $version } ha risposto.
settings-downloads-torrent-failed = { $client } non si è collegato. Controlla indirizzo e password qui sopra e riprova.
settings-downloads-torrent-untried = Non ancora provato.
settings-downloads-mode-existing = Un qBittorrent gestito da te, con la Web UI attiva.
settings-downloads-mode-managed = AniBeam fa girare un suo qBittorrent in ~/Downloads/Anime. Continua dopo la chiusura di AniBeam, così le iscrizioni continuano a scaricare.
settings-downloads-mode-auto = Usa un qBittorrent che risponde all'indirizzo qui sotto, altrimenti ne avvia uno suo.
settings-downloads-password-saved = Qui è salvata una password per questo client.
settings-downloads-password-anirss = Usa la password salvata dall'helper di ricerca; salvane una qui per non dipenderne più.
settings-downloads-password-none = Nessuna password salvata. Un client che salta l'accesso per questa macchina non ne ha bisogno.
settings-downloads-mismatch = L'helper di ricerca parla con { $helperUrl } mentre AniBeam parla con { $url }. Un helper di ricerca gestito da te legge l'indirizzo dal suo config.toml, quindi cambiare client qui non lo sposta.
settings-downloads-torrent-title = Client torrent
settings-downloads-torrent-helper = Il qBittorrent che fa i download. La sua password è salvata nell'archivio password di sistema.
settings-downloads-which-client = Quale client
settings-downloads-pick-for-me = Scegli tu
settings-downloads-my-own = Il mio
settings-downloads-built-in = Integrato
settings-downloads-saved-clients = Client salvati
settings-downloads-saved-clients-helper = Per esempio una seedbox e quello su questa macchina. Anche il chip del client nella pagina Download passa dall'uno all'altro.
settings-downloads-add-client = Aggiungi
settings-downloads-name = Nome
settings-downloads-name-placeholder = Preso dal suo indirizzo
settings-downloads-address = Indirizzo
settings-downloads-address-managed-helper = Solo questa macchina lo raggiunge, e non serve password.
settings-downloads-address-helper = L'indirizzo della Web UI.
settings-downloads-username = Nome utente
settings-downloads-password = Password
settings-downloads-secret-saved = Salvata
settings-downloads-secret-unset = Non impostata
settings-downloads-connection = Connessione
settings-downloads-open-config = Apri la configurazione
settings-downloads-details = Dettagli
settings-downloads-details-torrent = { $client } ha detto questo quando AniBeam l'ha provato.
settings-downloads-test-trying = Prova in corso
settings-downloads-test-save = Salva e prova
settings-downloads-test = Prova
settings-downloads-searching = Ricerca
settings-downloads-search-helper = Helper di ricerca
settings-downloads-searching-helper = AniBeam cerca da solo nelle tue fonti e invia quello che scegli al client torrent. L'helper di ricerca non serve.
settings-downloads-search-helper-helper = La parte di AniBeam che cerca e gestisce le iscrizioni, un programma chiamato anirss. AniBeam lo avvia per te; cambia qui solo se ne usi uno tuo.
settings-downloads-runs-in = Le ricerche girano in
settings-downloads-runs-in-native-helper = Le iscrizioni restano al client torrent: i nuovi episodi arrivano anche ad AniBeam chiuso.
settings-downloads-runs-in-helper = L'helper di ricerca, un programma separato che avvia AniBeam o che gestisci tu.
settings-downloads-engine-native = AniBeam stesso
settings-downloads-how-runs = Come gira
settings-downloads-how-runs-sidecar = AniBeam lo avvia all'apertura e lo ferma alla chiusura.
settings-downloads-how-runs-remote = AniBeam usa uno che gestisci già, all'indirizzo e con la chiave qui sotto.
settings-downloads-runs-sidecar = Lo avvia AniBeam
settings-downloads-runs-remote = Lo gestisco io
settings-downloads-command = Comando
settings-downloads-command-helper = Un nome nel PATH o un percorso completo. Viene cercato anche ~/.local/bin.
settings-downloads-server-address-helper = La base del server, senza /api/v1.
settings-downloads-key = Chiave
settings-downloads-key-saved-helper = È salvata una chiave. Scrivine una nuova per sostituirla, o svuota il campo per rimuoverla.
settings-downloads-key-helper = La chiave che l'helper di ricerca ha mostrato all'avvio. Fa entrare AniBeam.
settings-downloads-details-anirss = L'helper di ricerca ha detto questo quando si è fermato.
settings-downloads-starting = Avvio
settings-downloads-reconnect = Ricollega
settings-downloads-save-first = Prima salva le modifiche
settings-downloads-roll-title = Tieni pronti i prossimi
settings-downloads-roll-helper = Una serie lunga scarica pochi episodi alla volta, e ognuno che finisci avvia il successivo. Non serve che tutto stia sul disco.
settings-downloads-roll-big = Per serie lunghe
settings-downloads-roll-big-helper = Sopra i { $count } episodi si scarica da solo, pochi alla volta. La sua riga in Download può comunque scaricare tutto insieme.
settings-downloads-roll-ahead = Episodi in anticipo
settings-downloads-roll-ahead-helper = Quanti restano scaricati dopo quello a cui sei. 12 sono circa una stagione.
settings-downloads-unit-episodes = episodi
settings-downloads-roll-over = Lunga significa oltre
settings-downloads-roll-over-helper = I download con più episodi di così si scaricano pochi alla volta.
settings-downloads-roll-watched = Episodi visti
settings-downloads-roll-watched-delete-helper = Eliminati quando sei più avanti, così il disco tiene solo quello che resta da vedere.
settings-downloads-roll-watched-keep-helper = Ogni file resta sul disco dopo averlo visto.
settings-downloads-keep-all = Tieni tutti
settings-downloads-keep-behind = Tieni indietro
settings-downloads-subscriptions-title = Iscrizioni e ricerche
settings-downloads-last-search = Ricorda l'ultima ricerca
settings-downloads-last-search-helper = La pagina Download si apre con l'ultima ricerca fatta. Se è off, si apre vuota.
settings-downloads-prune = Annulla l'iscrizione alle serie concluse
settings-downloads-prune-helper = 16 giorni dopo la messa in onda dell'ultimo episodio, il feed viene eliminato. La serie e i suoi file restano.
settings-downloads-sources-title = Fonti di ricerca
settings-downloads-sources-helper = Dove va una ricerca, in ordine di tentativo. Se una fonte non trova nulla, tocca alla successiva.
settings-downloads-sources-empty = Ancora nessuna fonte configurata
settings-downloads-sources-connect = Collega l'helper di ricerca per elencarle
settings-downloads-sources-empty-helper = Aggiungi un elenco di fonti qui sotto e le sue fonti compaiono qui.
settings-downloads-config-file = File di impostazioni dell'helper di ricerca
settings-downloads-config-native-helper = Contiene le cartelle di salvataggio, i tag e le regole della migliore corrispondenza, che AniBeam segue quando avvia l'helper. Le sue fonti sono solo dell'helper.
settings-downloads-config-helper = Contiene le fonti, le cartelle di salvataggio, i tag e le regole della migliore corrispondenza.
settings-downloads-unsaved = Modifiche non salvate
settings-downloads-unsaved-helper = Salvare riavvia l'helper di ricerca con le nuove impostazioni.
settings-downloads-discard = Scarta

## SettingsDeveloperTab

settings-developer-title = Sviluppatore
settings-developer-lead = Perché una pagina è lenta, misurato. Ogni interruttore vale per la prossima pagina che apri.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds } s
settings-developer-switches = Interruttori
settings-developer-load-images = Carica immagini
settings-developer-load-images-helper = Se è off, nessuna locandina, banner o ritratto viene letto o decodificato.
settings-developer-overlay = Overlay caricamento pagina
settings-developer-overlay-helper = L'ultimo caricamento di pagina e la memoria, nell'angolo in basso a destra della finestra.
settings-developer-read-cache = Tieni le letture della libreria
settings-developer-read-cache-helper = La risposta a una lettura dell'intera libreria resta finché la libreria non cambia. Se è off, ogni pagina interroga di nuovo il core.
settings-developer-poster-pool = Prepara le locandine in background
settings-developer-poster-pool-helper = Decodifica le 60 locandine che vedrai più probabilmente e le tiene in memoria.
settings-developer-options = Opzioni sviluppatore
settings-developer-options-helper = Spegne questa scheda, l'overlay e le misure.
settings-developer-memory = Memoria
settings-developer-memory-helper = Quello che il kernel conta per questo processo. Le immagini tengono i pixel decodificati in memoria e sulla scheda grafica.
settings-developer-memory-unknown = Questo sistema non dice quanta memoria usa l'app.
settings-developer-in-use = In uso
settings-developer-peak = Picco
settings-developer-heap = Heap
settings-developer-files-mapped = File mappati
settings-developer-threads = Thread
settings-developer-give-back = Restituisci memoria
settings-developer-give-back-helper = Scarta quello che lo scene graph può ricostruire, raccoglie la garbage JavaScript e restituisce l'heap libero al sistema. Quello che libera è ciò che l'app teneva senza averne bisogno.
settings-developer-released = Da { $before } a { $after }
settings-developer-free-now = Libera ora
settings-developer-images-total = Immagini dall'avvio
settings-developer-images-total-line = { $decodes } decodificate, { $pixels } di pixel da { $disk } su disco, { $ms } ms
settings-developer-loads = Caricamenti di pagina
settings-developer-loads-helper = Creata è l'attesa dopo un clic: il thread Qt non fa altro finché la pagina non esiste. Completa è quando è arrivata la sua ultima immagine.
settings-developer-touring = Misura, { $at } di { $steps }
settings-developer-tour = Misura ogni pagina
settings-developer-copy-report = Copia report
settings-developer-col-page = Pagina
settings-developer-col-built = Creata
settings-developer-col-painted = Disegnata
settings-developer-col-settled = Completa
settings-developer-col-calls = Chiamate, dati
settings-developer-col-decodes = Decodifiche, tempo, pixel
settings-developer-col-items = Elementi
settings-developer-col-offscreen = Immagini fuori vista
settings-developer-col-memory = Memoria
settings-developer-load-no-images = { $page }, nessuna immagine
settings-developer-offscreen-of = { $offscreen } di { $total }
settings-developer-loads-empty = Apri una pagina qualsiasi e compare qui.
settings-developer-load-images-off = { $page }, immagini off
settings-developer-load-frames = { $frames } fotogrammi disegnati durante il caricamento, la pausa più lunga tra due { $gap }.
settings-developer-load-frames-timed-out = { $frames } fotogrammi disegnati durante il caricamento, la pausa più lunga tra due { $gap }. Smesso di aspettare le immagini dopo 15 s.
settings-developer-built = Creata
settings-developer-settled = Completa
settings-developer-core-data = Dati dal core
settings-developer-pixels = Pixel decodificati
settings-developer-page-items = Elementi nella pagina
settings-developer-cached-images = Immagini già in memoria
settings-developer-calls = Chiamate al core, prima le più lente
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  in cache
settings-developer-call = { $name }  { $ms } ms  { $size }  core { $core }, a QML { $qml }
settings-developer-decodes = Decodifiche più lente
settings-developer-decodes-line = { $count } decodifiche una dopo l'altra sul thread immagini, { $span } dalla prima all'ultima; { $requests } richieste, { $cached } già decodificate, { $blocked } bloccate
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } a { $w }x{ $h }  { $size }
settings-developer-decodes-none = Niente decodificato: ogni immagine era già in memoria.
settings-developer-images-were-off = Le immagini erano off.
settings-developer-scene = Di cosa è fatta la pagina
settings-developer-scene-line = { $items } elementi, { $offscreen } fuori dalla finestra; { $texts } testi, { $shapes } forme, { $icons } icone; { $images } immagini, { $imagesOffscreen } fuori dalla finestra con { $size }
settings-developer-copy-load = Copia questo caricamento
settings-developer-tour-title = Ogni pagina, immagini on e off
settings-developer-tour-helper = La mediana di tre visite a ogni pagina. Dove le due colonne coincidono, non sono le immagini a rallentare la pagina.
settings-developer-col-built-on = Creata, immagini on
settings-developer-col-built-off = Creata, immagini off
settings-developer-col-settled-on = Completa, on
settings-developer-col-settled-off = Completa, off
settings-developer-col-decodes-time = Decodifiche, tempo
settings-developer-col-data = Dati
settings-developer-tour-peak = Picco di memoria durante la misura { $size }
settings-developer-events = Eventi
settings-developer-events-helper = Tutto quello che dice il core, ogni livello, dal più recente. Ogni evento viene copiato sul thread Qt come JSON; la dimensione è quanto è costato inviarlo.
settings-developer-event-rate = { $kind } { $count }/min, { $size }
settings-developer-events-filter = Filtra per parole, tipo o fase
settings-developer-event-plain = { $message }   (semplice: { $plain })
settings-developer-copy-events = Copia eventi

## SettingsKeysTab

settings-keys-title = Tastiera
settings-keys-lead = Ogni tasto a cui AniBeam risponde. Premi ? ovunque per i tasti che funzionano dove sei.
settings-keys-player = Nel player
settings-keys-player-around = Muoversi nel player
settings-keys-around = Muoversi
settings-keys-anywhere = Ovunque

## KeyHelp

key-help-title = Scorciatoie da tastiera
key-help-group-playback = Riproduzione
key-help-group-navigation = Navigazione
key-help-group-library = Libreria
key-help-key-space = Spazio / K
key-help-key-left-right = Sinistra / Destra
key-help-key-ctrl-right = Ctrl+Destra
key-help-key-backspace = Backspace
key-help-key-up-down = Su / Giù
key-help-key-hold = Tieni premuto sul video
key-help-key-page = Pag Su / Giù
key-help-key-home-end = Home / Fine
key-help-key-escape = Esc
key-help-key-alt-left = Alt+Sinistra
key-help-key-alt-right = Alt+Destra
key-help-key-shift-f10 = Maiusc+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / o Ctrl+F
key-help-key-ctrl-space = Ctrl+Spazio
key-help-key-shift-space = Maiusc+Spazio
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = Riproduci o pausa
key-help-seek-5 = Salta di 5 s
key-help-seek-10 = Salta di 10 s
key-help-skip = Salta intro o outro, altrimenti 90 s
key-help-frame-step = Un fotogramma indietro o avanti
key-help-speed = Più lento o più veloce, a passi di 0,25
key-help-normal-speed = Velocità normale
key-help-volume = Volume su o giù di 5
key-help-mute = Muto
key-help-subtitles-toggle = Sottotitoli off e di nuovo on
key-help-subtitle-delay = Ritardo sottotitoli 100 ms prima o dopo
key-help-audio-delay = Ritardo audio 100 ms prima o dopo
key-help-hold-speed = Velocità 2× tenendo premuto
key-help-screenshot = Salva uno screenshot in Immagini
key-help-episode = Episodio successivo o precedente
key-help-chapter = Capitolo precedente o successivo
key-help-start-end = L'inizio, o l'outro
key-help-fullscreen = Schermo intero
key-help-escape-player = Chiudi ciò che è aperto, altrimenti esci da schermo intero, altrimenti esci dal player
key-help-toggle-list = Mostra o nascondi questa lista
key-help-library-search = Libreria e ricerca
key-help-search-page = Cerca in questa pagina
key-help-activity = Attività
key-help-settings = Impostazioni
key-help-quit = Esci
key-help-forward = Avanti
key-help-escape-app = Chiudi ciò che è aperto, altrimenti indietro
key-help-context-menu = Il menu del clic destro
key-help-pick-card = Scegli una scheda, per agire su più schede
key-help-pick-range = Scegli tutte le schede fino a questa
key-help-pick-all = Scegli tutte le schede mostrate

## SettingsAboutTab

settings-about-title = Info
settings-about-lead = Quale AniBeam è questo, e come viene a sapere del prossimo.
settings-about-update-failed = L'aggiornamento non è terminato. Riprova, o scaricalo dalla pagina della release.
settings-about-update-ready = { $version } è pronto da installare.
settings-about-update-command = È uscita la { $version }. Aggiornala con { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = È uscita la { $version }. Questa copia è compilata dai sorgenti: fai pull e ricompila.
settings-about-update-elsewhere = È uscita la { $version }, senza un download qui per questo computer.
settings-about-version = Versione
settings-about-build = AniBeam { $build }
settings-about-whats-new = Novità
settings-about-install = Installa { $version }
settings-about-setup = Configurazione
settings-about-setup-helper = La guida con cui si apre una nuova installazione: la tua cartella, la tua lista e i download.
settings-about-run-setup = Ripeti configurazione
settings-about-updates = Aggiornamenti
settings-about-nightly-helper = Una nuova build quasi ogni giorno. Più recente, e meno testata.
settings-about-stable-helper = Una nuova versione quando è stata testata e pubblicata.
settings-about-stable = Stabile
settings-about-nightly = Nightly
settings-about-check-now = Controlla ora
settings-about-developer = Opzioni sviluppatore
settings-about-developer-helper = Aggiunge la sezione Sviluppatore: tempi delle pagine, memoria, registro tecnico completo e immagini spegnibili.
settings-about-credits = Crediti
settings-about-ratings = Voti
settings-about-ratings-helper = Dati concessi da IMDb (https://www.imdb.com). Usati con permesso.
settings-about-series = Serie ed episodi
settings-about-series-helper = AniList e TVmaze. Tempi di salto da AniSkip.

## SettingsPlaybackTab

settings-playback-title = Riproduzione
settings-playback-lead = Cosa fa il player da solo, così lo tocchi solo quando vuoi.
settings-playback-skipping = Salto
settings-playback-skipping-helper = Usa i capitoli del file, o i tempi di salto condivisi online da altri. Annulla nel player sospende il salto finché chiudi AniBeam.
settings-playback-skip-intros = Salta le intro
settings-playback-skip-outros = Salta le outro
settings-playback-skip-outros-helper = Arrivare all'outro segna anche l'episodio come visto.
settings-playback-next-episode = Prossimo episodio
settings-playback-name-next = Mostra il titolo del prossimo episodio
settings-playback-name-next-helper = Il pulsante Prossimo alla fine di un episodio mostra il titolo del successivo. Se è off, mostra solo il numero.
settings-playback-languages = Lingue
settings-playback-languages-helper = Codici di lingua, dal più voluto. Vince il primo che il file ha. Una traccia scelta nel player viene ricordata per quella serie.
settings-playback-subtitle-languages = Lingue dei sottotitoli
settings-playback-audio-languages = Lingue dell'audio
settings-playback-discord = Mostra cosa guardi su Discord
settings-playback-discord-check-failed = AniBeam non è riuscito a controllare Discord. Riprova tra un attimo.
settings-playback-discord-helper = Serie, episodio e locandina compaiono sul tuo profilo Discord mentre guardi. Non viene inviato nulla quando è off o Discord è chiuso.
settings-playback-discord-unavailable = Questa build non ha un'applicazione Discord.
settings-playback-discord-connected = Collegato a Discord
settings-playback-discord-not-connected = Non collegato. AniBeam si collega mentre un episodio è in riproduzione e Discord è aperto
settings-playback-advanced = Avanzate
settings-playback-my-mpv = Usa le mie impostazioni mpv
settings-playback-my-mpv-helper = Legge il tuo mpv.conf. Le opzioni che funzionano solo all'avvio vengono saltate, e gli script non vengono mai caricati.
settings-playback-no-mpv-conf = Nessun file di impostazioni mpv trovato. Metti le tue opzioni in mpv.conf, poi attiva questa opzione.

## SliderRow

slider-row-spoken-unit = { $name }, in { $unit }
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Accento rosso
swatches-accent-orange = Accento arancione
swatches-accent-yellow = Accento giallo
swatches-accent-green = Accento verde
swatches-accent-cyan = Accento ciano
swatches-accent-blue = Accento blu

## LookPreview

look-preview-dark = Scuro
look-preview-light = Chiaro

## SettingsAccountsTab

settings-accounts-title = Account
settings-accounts-lead = Collega AniList o MyAnimeList e ogni episodio che finisci viene segnato lì per te. I conteggi salgono e basta.
settings-accounts-syncing = Sincronizzazione delle liste
settings-accounts-synced = Liste sincronizzate
settings-accounts-lists = Le tue liste di anime
settings-accounts-lists-helper = Anche senza, AniBeam ricorda cosa hai visto su questo computer. Una lista su AniList o MyAnimeList lo porta sul telefono e sul tuo profilo.
settings-accounts-progress = Progressi
settings-accounts-main-list = Lista principale
settings-accounts-main-list-helper = Le schede mostrano i progressi di questa. L'altra riceve comunque ogni aggiornamento.
settings-accounts-local-lists = Liste su questo computer
settings-accounts-local-lists-helper =
    { $count ->
        [one] Una serie ha
       *[other] { $count } serie hanno
    } uno stato o un voto solo qui. Spostale su { $tracker } per vederle sul telefono e sul tuo profilo.
settings-accounts-review-move = Controlla e sposta
settings-accounts-nothing-to-move = Niente da spostare
settings-accounts-sync-now = Sincronizza ora
settings-accounts-sync-now-helper = Rilegge ogni lista collegata. AniBeam lo fa anche all'avvio.
settings-accounts-sync = Sincronizza
settings-accounts-move-on-hold = Sposta in In pausa
settings-accounts-move-on-hold-helper = Una serie In visione che non tocchi da un po' passa a In pausa nelle tue liste. Riprodurla la riporta indietro. Una serie che aspetta il prossimo episodio resta dov'è.
settings-accounts-after = Dopo
settings-accounts-after-line = Giorni senza riprodurre un episodio
settings-accounts-days-unit = giorni

## SettingsPhoneTab

settings-phone-title = App telefono
settings-phone-lead = AniBeam sul telefono può riprodurre questa libreria, vedere cosa va in onda e seguire i tuoi download, ovunque tu sia.
settings-phone-online = Online
settings-phone-offline = Offline
settings-phone-untrusted = Non riceve nulla da questo computer
settings-phone-sharing = Condivisione
settings-phone-sharing-helper = Solo i tuoi dispositivi su Tailscale possono raggiungerla. Gli episodi visti sul telefono mantengono posizione e progressi qui.
settings-phone-share = Condividi via Tailscale
settings-phone-stream = Streaming episodi
settings-phone-stream-on = I dispositivi riproducono e trasmettono gli episodi da qui.
settings-phone-stream-off = I dispositivi vedono la libreria e cosa è in onda, ma non riproducono da qui.
settings-phone-host-service = Servizio host
settings-phone-not-running = Non attivo
settings-phone-running = Attivo
settings-phone-service-off-helper = Esegui il comando una volta e i tuoi dispositivi raggiungono AniBeam dall'accesso in poi, senza finestre aperte.
settings-phone-service-on-helper = I tuoi dispositivi raggiungono AniBeam senza finestre aperte. Mentre questa finestra è aperta, condivide al posto del servizio.
settings-phone-copy-command = Copia comando
settings-phone-command-copied = Comando copiato
settings-phone-nobody = Non riceve nulla: Tailscale non lo associa a nessuna persona
settings-phone-people = Persone
settings-phone-own-profile = Profilo di questo computer
settings-phone-person-own-devices =
    { $count ->
        [0] Questo computer
        [one] Questo computer e { $count } dispositivo
       *[other] Questo computer e { $count } dispositivi
    }
settings-phone-person-devices =
    { $count ->
        [one] { $count } dispositivo
       *[other] { $count } dispositivi
    }
settings-phone-devices = Dispositivi sulla tua tailnet
settings-phone-devices-helper = Con la condivisione attiva, i dispositivi della tailnet accedono a questo computer. Uno spento non riceve nulla da qui.
settings-phone-addresses = Indirizzi
settings-phone-addresses-helper = Scrivine uno nell'app del telefono la prima volta che lo chiede.
settings-phone-copy = Copia
settings-phone-address-copied = Indirizzo copiato

## SettingsHelpTab

settings-help-title = Aiuto
settings-help-lead = Risposte brevi alle domande più frequenti, ognuna con un rimedio diretto.
settings-help-folders = Come devono essere le mie cartelle?
settings-help-folders-helper = Una cartella per serie, con gli episodi dentro. Ogni file porta il numero dell'episodio, come le release: "Frieren - 01.mkv" o "Frieren S01E01.mkv". Una seconda stagione può avere una cartella sua o una "Season 2" nella serie. I film vanno in una cartella Movies, un file ciascuno.
settings-help-for-example = Per esempio
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = Ancora nessuna cartella aggiunta
settings-help-reads-folders =
    { $count ->
        [one] AniBeam legge una cartella
       *[other] AniBeam legge { $count } cartelle
    }
settings-help-add-folder-helper = Aggiungi la cartella che contiene le tue serie, non ogni serie da sola.
settings-help-add-folder = Aggiungi cartella
settings-help-open-folders = Apri le tue cartelle
settings-help-match = Perché la mia serie non è stata riconosciuta?
settings-help-match-helper = AniBeam cerca ogni cartella per nome, prima su AniList, poi su TVmaze. Parole in più, un errore di battitura o un nome in un'altra lingua possono sbagliare. Scegli a mano la serie giusta in Info serie; una tua scelta non viene mai sostituita.
settings-help-all-recognised = Ogni serie è riconosciuta
settings-help-unrecognised =
    { $count ->
        [one] Una serie non è riconosciuta
       *[other] { $count } serie non sono riconosciute
    }
settings-help-open-show-info = Apri Info serie
settings-help-connect = Come collego AniList?
settings-help-connect-helper = Collegalo e ogni episodio che finisci viene segnato nella tua lista, così telefono e profilo restano aggiornati. AniList ti chiede di creare una tua piccola app una volta; AniBeam ti guida, e ci vuole circa un minuto. MyAnimeList funziona allo stesso modo.
settings-help-both-connected = Sono collegati entrambi
settings-help-anilist-connected = AniList è collegato
settings-help-mal-connected = MyAnimeList è collegato
settings-help-none-connected = Nessuno dei due è ancora collegato
settings-help-connect-anilist = Collega AniList
settings-help-connect-mal = Collega MyAnimeList
settings-help-downloads = Perché Download è vuoto?
settings-help-downloads-helper = AniBeam non include posti in cui cercare. Aggiungi un elenco di fonti nelle Impostazioni, in Download, e installa da lì le fonti che vuoi. Per scaricare serve anche un client torrent: AniBeam può avviarne uno suo, o usare un qBittorrent che hai già.
settings-help-sources = Fonti e client torrent
settings-help-open-downloads = Apri impostazioni Download
settings-help-data = Dove sono i miei dati?
settings-help-data-helper = La tua libreria, cosa hai visto e le tue impostazioni restano su questo computer. Gli accessi ad AniList e MyAnimeList vanno nell'archivio password di sistema, o in un file leggibile solo da te dove non c'è. Fai il backup di tutto in un file da Spazio e backup.
settings-help-library-history = Libreria e cronologia
settings-help-show-folder = Mostra la cartella
settings-help-backup = Backup o ripristino
settings-help-open-storage = Apri Spazio e backup
settings-help-more = Altro?
settings-help-more-helper = Ripeti la prima configurazione, o dicci cosa è andato storto. Uno screenshot aiuta molto.
settings-help-first-setup = La prima configurazione
settings-help-first-setup-helper = Cartella, lista e download, un passo alla volta.
settings-help-run-setup = Ripeti configurazione
settings-help-report = Segnala un problema
settings-help-report-helper = Apre una segnalazione di bug su GitHub nel browser, con questa versione già compilata.

## SettingsSubtitlesTab

settings-subtitles-title = Sottotitoli
settings-subtitles-lead = Con cosa parte ogni episodio. Le modifiche si vedono sul fotogramma qui sotto mentre le fai.
settings-subtitles-reset-done = I sottotitoli sono tornati come all'inizio
settings-subtitles-size-place = Dimensione e posizione
settings-subtitles-size = Dimensione
settings-subtitles-position = Posizione
settings-subtitles-position-helper = 100 sta sul bordo inferiore. I numeri più bassi alzano le righe nel fotogramma.
settings-subtitles-style = Stile
settings-subtitles-styled = Sottotitoli con stile
settings-subtitles-as-made-helper = I file con stile (ASS) restano esattamente come li hanno impostati gli autori.
settings-subtitles-mine-helper = Il tuo stile qui sotto vale per ogni file. Cartelli e karaoke possono rompersi.
settings-subtitles-size-only-helper = I file con stile tengono l'aspetto e la tua dimensione. I semplici usano il tuo stile.
settings-subtitles-as-made = Originale
settings-subtitles-size-only = Solo dimensione
settings-subtitles-mine = Il mio
settings-subtitles-font = Font
settings-subtitles-font-invalid = Scrivi il nome di un font, come Noto Sans.
settings-subtitles-bold = Grassetto
settings-subtitles-text-colour = Colore testo
settings-subtitles-colour-invalid = Scrivi un colore come #ffcc00.
settings-subtitles-outline = Contorno
settings-subtitles-outline-invalid = Lo spessore è un numero, 0 o più.
settings-subtitles-outline-colour-invalid = Scrivi il colore come #000000.
settings-subtitles-outline-helper = Lo spessore, poi il colore.
settings-subtitles-shadow = Ombra
settings-subtitles-shadow-invalid = Un numero, 0 o più. 0 significa nessuna ombra.
settings-subtitles-shadow-helper = Quanto cade lontano l'ombra, 0 per nessuna.
settings-subtitles-box = Riquadro di sfondo
settings-subtitles-box-helper = Un riquadro scuro dietro le righe. 0 lo toglie.
settings-subtitles-start-over = Ricomincia
settings-subtitles-start-over-helper = Riporta dimensione, font, colori, contorno, riquadro e posizione come erano all'inizio.
settings-subtitles-reset = Ripristina

## SettingsDataTab

settings-data-title = Spazio e backup
settings-data-lead = Sposta la libreria su un altro computer, fanne una copia o libera spazio dalle immagini.
settings-data-backed-up = Backup in { $path }
settings-data-restore-finished = Ripristino completato
settings-data-images-removed =
    { $count ->
        [one] { $count } immagine rimossa
       *[other] { $count } immagini rimosse
    }
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] { $count } cartella aggiunta
       *[other] { $count } cartelle aggiunte
    }
settings-data-summary-series = { $count } serie create
settings-data-summary-matches =
    { $count ->
        [one] { $count } abbinamento applicato
       *[other] { $count } abbinamenti applicati
    }
settings-data-summary-views =
    { $views ->
        [one] { $views } visione
       *[other] { $views } visioni
    } e { $episodes ->
        [one] { $episodes } episodio finito
       *[other] { $episodes } episodi finiti
    } uniti
settings-data-summary-resume-points =
    { $count ->
        [one] { $count } punto di ripresa unito
       *[other] { $count } punti di ripresa uniti
    }
settings-data-summary-accounts =
    { $count ->
        [one] { $count } account di lista importato
       *[other] { $count } account di lista importati
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } file usa una correzione che hai fatto.
       *[other] { $count } file usano una correzione che hai fatto.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } cartella non è stata trovata su questo computer.
       *[other] { $count } cartelle non sono state trovate su questo computer.
    }
settings-data-summary-left-out = Esclusi: { $fields }.
settings-data-backup = Backup
settings-data-back-up-library = Backup della libreria
settings-data-back-up-helper = Un file con le tue cartelle, gli abbinamenti e quello che hai visto.
settings-data-last-written = Ultimo salvataggio in { $path }
settings-data-open-folder = Apri cartella
settings-data-back-up = Backup
settings-data-private = Includi dati privati
settings-data-private-helper = Aggiunge gli accessi ad AniList e MyAnimeList, la cronologia di visione e le preferenze. Gli accessi sono scritti in chiaro, quindi tieni il file per te.
settings-data-restore-from = Ripristina da un backup
settings-data-restore-helper = Legge un backup in questa libreria. Vince quello che è su disco; il file completa il resto.
settings-data-restore = Ripristina
settings-data-image-cache = Cache immagini
settings-data-image-cache-helper = Locandine, banner e ritratti restano su disco, così la libreria va offline. Le immagini cancellate si riscaricano se servono.
settings-data-cache-unmeasured = Impossibile misurare la cache immagini
settings-data-cache-size =
    { $count ->
        [one] { $size } in { $count } immagine
       *[other] { $size } in { $count } immagini
    }
settings-data-cache-empty = Nessuna immagine in cache
settings-data-cache-unmeasured-helper = Riprova tra un attimo. Le immagini stanno bene.
settings-data-remove-images =
    { $count ->
        [one] Rimuovere { $count } immagine?
       *[other] Rimuovere { $count } immagini?
    }
settings-data-files = Dove AniBeam tiene i suoi file
settings-data-files-database = Database della libreria
settings-data-files-data = Dati
settings-data-files-settings = Impostazioni
settings-data-files-cache = Cache
settings-data-backup-filter = Backup di AniBeam (*.json)
settings-data-all-files-filter = Tutti i file (*)
settings-data-backing-up = Backup in corso
settings-data-restoring = Ripristino

## SettingsLibraryTab

settings-library-title = Libreria
settings-library-lead = Le cartelle che AniBeam controlla, e come capisce cosa contengono.
settings-library-not-scanned = Non ancora scansionata
settings-library-scan-nothing-changed = Niente è cambiato
settings-library-scan-added = { $count } aggiunte
settings-library-scan-changed = { $count } cambiate
settings-library-scan-removed = { $count } rimosse
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = Anime: serie, film e come sono collegati. Quello che guardi viene segnato sui tuoi tracker.
settings-library-tvmaze-about = Altra televisione: live action, cartoni, documentari. Niente viene segnato su un tracker.
settings-library-tried-first = Provato per primo
settings-library-tried-second = Provato per secondo
settings-library-tried-third = Provato per terzo
settings-library-tried-fourth = Provato per quarto
settings-library-tried-nth = Provato per { $number }º
settings-library-folders = Cartelle
settings-library-folders-helper = Ogni cartella qui dentro è una serie. Un file in cima a una cartella Movies è un film.
settings-library-rescan-started = Nuova scansione avviata
settings-library-load-failed = Impossibile leggere le tue cartelle
settings-library-load-failed-helper = AniBeam non è riuscito a leggere il suo elenco di cartelle. Riprova tra un attimo.
settings-library-no-folders = Ancora nessuna cartella
settings-library-no-folders-helper = Indica ad AniBeam la cartella dei tuoi anime. Trova da solo serie e film.
settings-library-scanned = Scansionata { $when }
settings-library-add-a-folder = Aggiungi cartella
settings-library-scan-all = Scansiona tutto
settings-library-scan-started = Scansione avviata
settings-library-add-folder = Aggiungi cartella
settings-library-matching = Abbinamento
settings-library-matching-helper = Ogni cartella nuova si cerca in quest'ordine e tiene il primo abbinamento valido. Quelli manuali restano sempre.
settings-library-try-later = Prova dopo
settings-library-try-first = Prova prima
settings-library-check-matches = Controlla gli abbinamenti
settings-library-check-matches-helper = La pagina Info serie elenca ogni serie con le info che AniBeam ha trovato, e corregge quelle sbagliate.
settings-library-open-show-info = Apri Info serie
settings-library-hidden-shows = Serie nascoste
settings-library-show-hidden = Mostra serie nascoste
settings-library-show-hidden-helper = Riporta le serie nascoste in ogni pagina finché AniBeam non si chiude.
settings-library-hidden-from-airing = Nascoste da In onda ora
settings-library-hidden-from-airing-helper = Serie segnate come non interessanti. Ognuna resta fuori da In onda ora finché non la riporti.
settings-library-hidden-from-airing-empty = Ancora niente. Le serie che segni come non interessanti in In onda ora si raccolgono qui, così ognuna può tornare.
settings-library-show-again = Mostra di nuovo
settings-library-calendar = Calendario
settings-library-calendar-feed = Orari di messa in onda in un calendario
settings-library-calendar-feed-helper = Tiene un file calendario con i prossimi episodi delle serie nelle tue liste, aggiornato quando cambia la programmazione.
settings-library-calendar-file = File calendario
settings-library-calendar-file-helper = Iscriviti a questo file nella tua app calendario.
settings-library-copy = Copia
settings-library-path-copied = Percorso copiato
settings-library-open-folder = Apri cartella
settings-library-notifications = Notifiche
settings-library-new-episodes = Nuovi episodi
settings-library-new-episodes-helper = Una notifica sul desktop quando va in onda un episodio di una serie nella tua lista In visione. Cliccala per aprire la serie.
settings-library-folder-added = Cartella aggiunta, scansione in corso

## SettingsAppearanceTab

settings-appearance-language = Lingua
settings-appearance-language-system = Come il sistema ({ $language })
settings-appearance-text-smaller = Più piccolo
settings-appearance-text-normal = Normale
settings-appearance-text-large = Grande
settings-appearance-text-larger = Più grande
settings-appearance-text-largest = Massimo
settings-appearance-title = Aspetto
settings-appearance-lead = Le modifiche si vedono subito, qui e in ogni pagina. Stanno in ~/.config/anibeam/theme.toml, che puoi anche modificare a mano.
settings-appearance-colours = Colori
settings-appearance-mode = Modalità
settings-appearance-mode-dark = Scuro
settings-appearance-mode-light = Chiaro
settings-appearance-mode-system = Sistema
settings-appearance-colours-from = Colori da
settings-appearance-from-theme-helper = La coppia di temi qui sotto, uno scuro e uno chiaro.
settings-appearance-from-desktop-helper = I colori del tuo terminale, altrimenti lo schema colori e l'accento del desktop.
settings-appearance-from-desktop = Desktop
settings-appearance-from-theme = Tema
settings-appearance-dark-theme = Tema scuro
settings-appearance-light-theme = Tema chiaro
settings-appearance-themes-helper = I file di tema che metti in ~/.config/anibeam/themes compaiono in entrambe le liste.
settings-appearance-accent = Accento
settings-appearance-accent-helper = Segna ciò che è selezionato, in riproduzione o nuovo.
settings-appearance-icon = Icona
settings-appearance-app-icon = Icona app
settings-appearance-app-icon-helper = Quella della finestra, della barra delle applicazioni e del launcher. Scura scambia il giallo e il nero.
settings-appearance-icon-default = Predefinita
settings-appearance-icon-default-name = Icona app predefinita
settings-appearance-icon-dark = Scura
settings-appearance-icon-dark-name = Icona app scura
settings-appearance-size-motion = Dimensione e movimento
settings-appearance-text-size = Dimensione testo
settings-appearance-text-size-helper = Quanto sono grandi le parole in tutta AniBeam, oltre alla dimensione del sistema.
settings-appearance-density = Densità
settings-appearance-density-helper = Lo spazio attorno a tutto, da stretto ad arioso.
settings-appearance-density-compact = Compatta
settings-appearance-density-normal = Normale
settings-appearance-density-comfortable = Comoda
settings-appearance-poster-size = Dimensione locandine
settings-appearance-poster-size-helper = Grandezza delle serie in libreria e in Home.
settings-appearance-poster-small = Piccole
settings-appearance-poster-medium = Medie
settings-appearance-poster-large = Grandi
settings-appearance-out-now = Usciti ora in In onda
settings-appearance-out-now-helper = Schede con il banner di ogni serie, o una lista di piccole locandine e titoli, profonda tre righe.
settings-appearance-out-now-cards = Schede
settings-appearance-out-now-list = Lista
settings-appearance-out-now-grid = Schede Usciti ora
settings-appearance-out-now-grid-helper = Quante schede si vedono, colonne per righe, prima che Tutti apra il resto.
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Riduci movimento
settings-appearance-reduce-motion-helper = Salta l'animazione d'apertura della libreria e altri movimenti solo decorativi.

## LookPane

look-pane-sample-series = Serie di esempio
look-pane-episode-code = Ep { $number }
look-pane-library = Libreria
look-pane-series-count =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serie
    }
look-pane-search = Cerca titoli o cartelle
look-pane-search-hint = / o Ctrl+K
look-pane-all = Tutto
look-pane-series = Serie
look-pane-movies = Film
look-pane-plain = Semplice
look-pane-selected = Selezionato
look-pane-errors =
    { $count ->
        [one] { $count } errore
       *[other] { $count } errori
    }
look-pane-type-large = Grande
look-pane-type-normal = Normale
look-pane-type-small = Piccolo, tenue
look-pane-sample-switch = Interruttore di esempio
look-pane-button = Pulsante
look-pane-scan = Scansiona
look-pane-sample-status =
    { $count ->
        [one] { $count } nuovo episodio in { $title }
       *[other] { $count } nuovi episodi in { $title }
    }

## FeedPage

feed-title = Feed
feed-aired = { $episode } uscito { $when }
feed-downloaded = File scaricato { $when }
feed-countdown-days = { $days } g { $hours } h
feed-countdown-hours = { $hours } h { $minutes } m
feed-countdown-minutes = { $minutes } m
feed-section-this-week = Questa settimana
feed-new-episodes =
    { $count ->
        [one] { $count } nuovo episodio
       *[other] { $count } nuovi episodi
    }
feed-section-earlier = Prima
feed-latest-out = Ultimo uscito { $when }
feed-section-coming-soon = In arrivo
feed-this-week-count = { $count } questa settimana
feed-section-everything-else = Tutto il resto
feed-out-this-week = { $count } usciti questa settimana
feed-nothing-scheduled = Ancora niente in programma.
feed-next = Prossimo: { $title } tra { $countdown }
feed-nothing-new = Niente di nuovo questa settimana.
feed-new-this-week =
    { $count ->
        [one] { $count } nuovo episodio questa settimana.
       *[other] { $count } nuovi episodi questa settimana.
    }
feed-sort-recent = Usciti di recente
feed-sort-upcoming = In arrivo
feed-failed-title = Il feed non si è caricato
feed-empty-title = Ancora niente qui
feed-failed-body = AniBeam non è riuscito a leggere il feed. Riprova tra un attimo.
feed-empty-body-folders = Il feed si riempie man mano che gli episodi escono e arrivano. In onda ora mostra cosa c'è in questa stagione.
feed-empty-body = Aggiungi la cartella dei tuoi anime e il feed si riempie dei suoi episodi.
feed-add-folder = Aggiungi cartella
feed-airing-now = In onda ora

## RecommendedPage

recommended-title = Consigliati per te
recommended-ranked-minutes = Ordinati { $minutes } min fa
recommended-ranked-hours = Ordinati { $hours } h fa
recommended-ranked-days = Ordinati { $days } g fa
recommended-scope-name = Consigliati da
recommended-scope-all = Tutti
recommended-scope-library = In libreria
recommended-scope-list = In lista
recommended-ranking = Ordinamento
recommended-rank-again = Riordina
recommended-no-scores-title = Prima vota qualche serie
recommended-no-list-title = Ancora niente da cui imparare
recommended-never-fitted-title = Ordino le tue serie
recommended-nothing-left-title = Qui non resta niente
recommended-no-scores-body = I consigli imparano dai voti nella tua lista. Vota qualche serie su AniList e compaiono qui.
recommended-no-list-body = Collega AniList in Impostazioni o tieni una lista qui: le serie che voti guidano questa pagina.
recommended-never-fitted-body = Il primo ordinamento legge AniList per un minuto o due. Gira in background.
recommended-nothing-left-library-body = Tutto quello che va bene nella tua libreria è già visto. Tutti trova serie che non hai ancora.
recommended-nothing-left-body = Tutto quello che va bene nelle tue liste è già visto. Tutti trova serie che non hai in lista.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] Spostamento di 1 serie su { $site }
       *[other] Spostamento di { $count } serie su { $site }
    }
move-lists-score = voto { $score }
move-lists-title = Sposta la tua lista su { $site }
move-lists-body =
    { $count ->
        [one] Una serie ha
       *[other] { $count } serie hanno
    } uno stato o un voto solo su questo computer. Spostandole finiscono nella tua lista di { $site }, dove le vedono il telefono e il tuo profilo. Per queste serie, quello che { $site } ha ora viene sostituito. Non si invia nulla finché non premi Sposta.
move-lists-head-show = Serie
move-lists-head-here = Su questo computer
move-lists-head-tracker = Ora su { $site }
move-lists-nothing = Niente
move-lists-not-on-list = Non nella tua lista
move-lists-later-hint = Dopo: Impostazioni, Account
move-lists-not-now = Non ora
move-lists-move =
    { $count ->
        [one] Sposta 1 serie
       *[other] Sposta { $count } serie
    }

## StatsPage

stats-title = Statistiche
stats-month-jan = Gen
stats-month-feb = Feb
stats-month-mar = Mar
stats-month-apr = Apr
stats-month-may = Mag
stats-month-jun = Giu
stats-month-jul = Lug
stats-month-aug = Ago
stats-month-sep = Set
stats-month-oct = Ott
stats-month-nov = Nov
stats-month-dec = Dic
stats-nothing-yet = Ancora niente
stats-ranked-episodes =
    { $count ->
        [one] { $count } ep
       *[other] { $count } ep
    }
stats-year-summary = Cosa hai visto nel { $year }
stats-empty-title = Ancora niente di visto
stats-empty-body = Finisci un episodio e conta qui: ore, generi e le stagioni che hai visto.
stats-open-library = Libreria
stats-hours-watched = ore guardate
stats-episodes-label =
    { $count ->
        [one] episodio
       *[other] episodi
    }
stats-series-finished = serie finite
stats-none = Nessuno
stats-busiest-month = mese più pieno
stats-by-month = Per mese
stats-month-hours = { $hours } h
stats-top-genres = Generi preferiti
stats-top-studios = Studi preferiti
stats-seasons-watched = Stagioni viste
stats-recap-title = Il mio anno in anime
stats-recap-hours = ore
stats-recap-episodes = episodi
stats-recap-top-genre = genere preferito
stats-save-image = Salva come immagine
stats-save-dialog-title = Salva il riepilogo
stats-save-filter-png = Immagine PNG (*.png)
stats-recap-saved = Riepilogo salvato
stats-recap-save-failed = Impossibile salvare il riepilogo

## TrackerRow

tracker-row-connected = Collegato come { $user }, sincronizzato { $when }
tracker-row-unknown-user = ?
tracker-row-never = mai
tracker-row-waiting = In attesa della tua approvazione nel browser
tracker-row-not-connected = Non collegato
tracker-row-open-again = Riapri
tracker-row-open-again-tip = Riapri la pagina di accesso
tracker-row-disconnect = Scollega
tracker-row-disconnect-question = Scollegare { $name }? Il tuo accesso verrà rimosso.
tracker-row-secret-saved = Per questa app è salvato un client secret.
tracker-row-remove-secret = Rimuovilo
tracker-row-secret-removed = Client secret rimosso
tracker-row-add-secret = Aggiungi client secret
tracker-row-secret-where = È la stringa lunga sotto "Client Secret" nella pagina dell'app su myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Client secret
tracker-row-save-connect = Salva e collega

## RemoveSheet

remove-unsubscribe-delete = Annulla iscrizione ed elimina
remove-unsubscribe-torrents = Annulla iscrizione e rimuovi torrent
remove-unsubscribe = Annulla iscrizione
remove-delete-files = Rimuovi ed elimina file
remove-torrents =
    { $count ->
        [one] Rimuovi il torrent
       *[other] Rimuovi i torrent
    }
remove-also-torrents =
    { $count ->
        [one] Rimuovi anche il suo torrent
       *[other] Rimuovi anche i suoi { $count } torrent
    }
remove-torrents-go-with-files = Eliminare i file si porta via anche i torrent.
remove-torrents-files-stay = Escono dal client; i file restano.
remove-also-files = Elimina anche i file
remove-files-default-where = quello che hanno scaricato i torrent
remove-files-detail-rule = Sposta { $where } nel cestino e rimuove i torrent.
remove-files-detail = Sposta { $where } nel cestino.

## NotesSheet

notes-your-tracker = il tuo tracker
notes-title = Note
notes-placeholder = A che punto sei, chi te l'ha consigliata, quale release scaricare
notes-over-limit = { $count } caratteri oltre il limite
notes-saved-to = Salvate su { $sites }

## RecommendationCard

recommendation-card-available = Disponibile
recommendation-card-open-library = Apri { $title } nella tua libreria
recommendation-card-open-anilist = Apri { $title } su AniList

## SearchField

search-field-placeholder = Cerca titoli o cartelle
search-field-hint = / o Ctrl+K
search-field-clear = Cancella la ricerca

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent }%
recommended-hover-hidden-tags =
    { $count ->
        [one] Un tag nascosto che ami
       *[other] { $count } tag nascosti che ami
    }
recommended-hover-because = Perché hai visto

## PersonCard

person-card-unknown = Sconosciuto
person-card-role-main = Principale
person-card-role-supporting = Secondario
person-card-role-background = Comparsa
person-card-the-character = il personaggio
person-card-open = Apri { $name } su AniList

## StatusDot

status-dot-on-list = Nella tua lista: { $status }

## TrackerNudge

tracker-nudge-title = Collega la tua lista di anime
tracker-nudge-body = AniBeam segna gli episodi su AniList o MyAnimeList mentre guardi.
tracker-nudge-connect-anilist = Collega AniList
tracker-nudge-connect-mal = Collega MyAnimeList
tracker-nudge-not-now = Non ora

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } per cento per te
recommended-card-forecast = { $percent }%
recommended-card-in-library = In libreria

## InlineConfirm

inline-confirm-keep = Tieni

## Tokens

tokens-cube-watched-tracked = Visto, tracciato online
    .inline = visto, tracciato online
tokens-cube-watched-here = Visto qui, non ancora tracciato
    .inline = visto qui, non ancora tracciato
tokens-cube-on-disk = Su disco
    .inline = su disco
tokens-cube-downloading = In download
    .inline = in download
tokens-cube-missing = Mancante
    .inline = mancante
tokens-cube-error = Errore
    .inline = errore
tokens-cube-not-downloaded = Non scaricato
    .inline = non scaricato
tokens-cube-not-aired = Non ancora in onda
    .inline = non ancora in onda
tokens-cube-available = Disponibile
    .inline = disponibile
tokens-cube-picked = Scelto da scaricare
    .inline = scelto da scaricare
tokens-cube-already-on-disk = Già su disco
    .inline = già su disco
tokens-cube-no-release = Nessuna release
    .inline = nessuna release

## Card

card-not-interested = Non mi interessa

## PagePlaceholder

page-placeholder-coming-soon = In arrivo

## ErrorLine

error-line-hide-details = Nascondi dettagli
error-line-details = Dettagli
error-line-details-tip = Mostra cosa è andato storto, parola per parola, e copialo
error-line-copied = Copiato.

## Letture (apps/linux/src/format.rs): orari, conti alla rovescia e durate

fmt-just-now = proprio ora
fmt-now = ora
fmt-ago = { $span } fa
fmt-in = tra { $span }
fmt-seconds-short = { $n } s
fmt-minutes-short = { $n } min
fmt-hours-short = { $n } h
fmt-days-short = { $n } g
fmt-months-short =
    { $n ->
        [one] { $n } mese
       *[other] { $n } mesi
    }
fmt-years-short =
    { $n ->
        [one] { $n } anno
       *[other] { $n } anni
    }
fmt-years-months-short = { $years } a { $months } m
fmt-countdown-days = { $d } g { $h } h { $m } m
fmt-countdown-hours = { $h } h { $m } m
fmt-countdown-days-seconds = { $d } g { $h } h { $m } m { $s } s
fmt-countdown-hours-seconds = { $h } h { $m } m { $s } s
fmt-countdown-minutes-seconds = { $m } m { $s } s
fmt-runtime-minutes = { $m } min
fmt-runtime-hours = { $h } h
fmt-runtime-hours-minutes = { $h } h { $m } min
fmt-episode = Episodio
fmt-size-b = { $size } B
fmt-size-kb = { $size } KB
fmt-size-mb = { $size } MB
fmt-size-gb = { $size } GB
fmt-size-tb = { $size } TB
fmt-episode-span = Episodi da { $first } a { $through }

## apps/linux/src/problem.rs: ogni errore di chiamata detto a parole, cosa è successo e cosa fare

problem-report = Riprova. Se continua a succedere, segnalalo dal registro attività.
problem-try-later = Riprova più tardi.
problem-helper-unreachable = L'helper di ricerca non ha risposto.
problem-helper-unauthorized = L'helper di ricerca ha rifiutato l'accesso.
problem-helper-not-running = L'helper di ricerca non è attivo.
problem-helper-missing = L'helper di ricerca non è installato.
problem-helper-failed = L'helper di ricerca non è riuscito a farlo.
problem-helper-no-login = L'helper di ricerca non riesce ad accedere al client torrent.
problem-helper-no-login-next = Salva la password del client torrent nelle Impostazioni, in Download.
problem-torrent-unreachable = Il client torrent non ha risposto.
problem-torrent-unauthorized = Il client torrent ha rifiutato l'accesso.
problem-torrent-not-running = Il client torrent non è attivo.
problem-torrent-missing = Il client torrent non è installato.
problem-torrent-failed = Il client torrent non è riuscito a farlo.
problem-downloads-unreachable-next = Controlla che sia in esecuzione e che il suo indirizzo sia giusto nelle Impostazioni, in Download.
problem-downloads-unauthorized-next = Controlla nome utente e password nelle Impostazioni, in Download.
problem-downloads-not-running-next = Avvialo dalle Impostazioni, in Download.
problem-downloads-missing-next = Impostazioni, in Download, spiega come configurarlo.
problem-trash = { -app-name } non è riuscito a spostare i file nel cestino.
problem-trash-next = Controlla che i file siano tuoi e che il disco non sia di sola lettura.
problem-source-list = Quell'elenco di fonti non ha funzionato.
problem-source-list-next = Controlla il suo indirizzo nelle Impostazioni, in Download, Elenchi di fonti.
problem-remote-unreachable = { $name } non ha risposto.
problem-remote-unreachable-next = Il sito potrebbe essere giù. Riprova più tardi, o cerca su un'altra fonte.
problem-remote-unauthorized = { $name } ha rifiutato la chiave ricevuta.
problem-remote-unauthorized-next = Cambia la sua chiave nelle Impostazioni, in Download, Elenchi di fonti.
problem-remote-failed = { $name } non è riuscito a farlo.
problem-remote-failed-next = Riprova più tardi, o cerca su un'altra fonte.
problem-provider-sign-in = { $name } ha rifiutato l'accesso di { -app-name }.
problem-provider-sign-in-next = Ricollegalo nelle Impostazioni, in Account.
problem-provider-turned-away = { $name } ha respinto { -app-name }.
problem-provider-busy = { $name } è occupato in questo momento.
problem-provider-busy-next = Aspetta un minuto, poi riprova.
problem-provider-no-answer = { $name } non ha risposto.
problem-provider-no-answer-next = Controlla la connessione a internet, poi riprova.
problem-provider-trouble = { $name } ha dei problemi in questo momento.
problem-provider-trouble-next = Riprova tra qualche minuto.
problem-provider-rejected = { $name } non l'ha accettato.
problem-not-found = { -app-name } non lo trova più.
problem-not-found-next = Potrebbe essere stato rimosso. Riprova per ricaricare la pagina.
problem-invalid-next = Controlla e riprova.
problem-unavailable = Quella cartella non è raggiungibile in questo momento.
problem-unavailable-next = Collega il suo disco o controlla la cartella nelle Impostazioni, in Libreria.
problem-not-connected = { $tracker } non è collegato.
problem-not-connected-next = Collegalo nelle Impostazioni, in Account.
problem-io = { -app-name } non è riuscito a leggere o scrivere un file.
problem-io-next = Controlla che il disco abbia spazio e che la cartella sia tua, poi riprova.
problem-storage = { -app-name } non è riuscito a salvare la tua libreria.
problem-storage-next = Controlla che il disco abbia spazio, poi riprova.
problem-keyring = L'archivio password di sistema non ha risposto.
problem-keyring-next = Sblocca il portachiavi del desktop, poi riprova.
problem-unsupported = { -app-name } non può ancora farlo qui.
problem-version = Questo backup viene da un { -app-name } più recente.
problem-version-next = Aggiorna { -app-name } nelle Impostazioni, in Info, poi importalo di nuovo.
problem-internal = Qualcosa è andato storto dentro { -app-name }.
problem-refused-hidden = Quella serie è nascosta.
problem-refused-hidden-next = Prima mostrala dalla sua pagina.
problem-refused-no-match = Quella serie non è ancora abbinata.
problem-refused-no-match-next = Prima abbinala in Info serie.
problem-refused-not-newer = La tua lista è già oltre questo episodio.
problem-refused-extra = Quel file è un extra, non un episodio.
problem-refused-unmatched = Quel file non è legato a un episodio.
problem-refused-unmatched-next = Correggilo dalla pagina della serie, in Correzioni.
problem-refused-on-disk = Quel file è solo su questo computer.

## apps/linux/src/library_view.rs: titoli delle sezioni della libreria e il dato sotto ognuna

library-view-airing = In onda
library-view-no-date = Nessuna data
library-view-type-tv = TV
library-view-type-movies = Film
library-view-type-extras = OVA, ONA e speciali
library-view-type-music = Musica
library-view-type-unknown = Sconosciuto
library-view-no-genre = Non abbinate
library-view-state-caught-up = In pari
library-view-state-not-started = Non iniziate
library-view-next-episode = Prossimo episodio tra { $span }
library-view-countdown-days = { $d } g { $h } h
library-view-countdown-hours = { $h } h { $m } m
library-view-countdown-minutes = { $m } m
library-view-in-progress = { $count } in corso
library-view-not-started = { $count } non iniziate
library-view-all-watched = Tutti visti
library-view-on-disk =
    { $count ->
        [one] { $count } episodio
       *[other] { $count } episodi
    } su disco
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } file
       *[other] { $count } file
    } ancora su disco

## apps/linux/src/dbus/notify.rs: la notifica desktop per un nuovo episodio

notify-episode-out = È uscito l'episodio { $episode } di { $title }
notify-click-open = Clicca per aprirlo in { -app-name }
notify-click-get = Clicca per scaricare gli episodi in { -app-name }
notify-get-episodes = Scarica episodi

## apps/linux/src/nowplaying.rs: la seconda riga di un widget multimediale durante un episodio

now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: perché lo schermo resta acceso

inhibit-reason = Riproduzione di un episodio

## apps/linux/src/bridge/door.rs: perché un valore inviato da una pagina non si può usare

door-not-a-choice = { $value } non è una scelta valida
door-tokens-op-empty = un'op tokens richiede dei token
door-not-a-refine-op = { $value } non è un'op di restringimento
door-limit-too-low = deve essere almeno 1, ricevuto { $limit }
door-not-a-torrent-action = { $value } non è un'azione torrent

## apps/linux/src/bridge/shell.rs: la cartella iniziale del primo avvio e l'avviso di aggiornamento

shell-starter-folder-failed = Impossibile creare { $folder }: { $error }
shell-update-gone = { -app-name } { $version } non è più disponibile
