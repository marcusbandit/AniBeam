# it translation of en/core.ftl. An ID missing here is shown in English.

## Il nodo nella tailnet

tailnet-person-no-name = Nessun nome in Tailscale

## Conteggi

count-episodes =
    { $count ->
        [one] { $count } episodio
       *[other] { $count } episodi
    }
count-shows =
    { $count ->
        [one] { $count } serie
       *[other] { $count } serie
    }

## Azioni usate in ogni schermata

action-cancel = Annulla
action-close = Chiudi
action-try-again = Riprova
action-remove = Rimuovi
action-delete = Elimina
action-open = Apri
action-save = Salva
action-apply = Applica
action-clear = Cancella
action-refresh = Aggiorna
action-play = Riproduci
action-connect = Collega
action-start = Avvia
action-back = Indietro
action-see-all = Vedi tutto
action-open-series = Apri serie
action-open-settings = Apri impostazioni

## Le liste di un tracker

status-watching = In visione
status-completed = Completato
status-paused = In pausa
status-on-hold = In pausa
status-dropped = Abbandonato
status-planning = Da vedere
status-rewatching = Lo rivedo

## Episodi

episode-number = Episodio { $number }

## Le parole di AniList per una serie: formato, stato di messa in onda, relazioni

anilist-format-tv = TV
anilist-format-tv-short = TV breve
anilist-format-film = Film
anilist-format-special = Speciale
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Musica
anilist-status-finished = Concluso
anilist-status-airing = In onda
anilist-status-not-yet-aired = Non ancora in onda
anilist-status-cancelled = Cancellato
anilist-status-hiatus = In pausa
anilist-relation-adaptation = Adattamento
anilist-relation-prequel = Prequel
anilist-relation-sequel = Sequel
anilist-relation-parent = Storia principale
anilist-relation-side-story = Side story
anilist-relation-character = Personaggio
anilist-relation-summary = Riassunto
anilist-relation-alternative = Alternativa
anilist-relation-spin-off = Spin-off
anilist-relation-other = Altro
anilist-relation-source = Fonte
anilist-relation-compilation = Raccolta
anilist-relation-contains = Contiene

## Registro attività: un problema senza parole proprie

event-problem-library = Qualcosa è andato storto con la tua libreria. I dettagli sono nelle opzioni sviluppatore.
event-problem-metadata = Qualcosa è andato storto con le info delle serie. I dettagli sono nelle opzioni sviluppatore.
event-problem-trackers = Qualcosa è andato storto con le tue liste. I dettagli sono nelle opzioni sviluppatore.
event-problem-franchise = Qualcosa è andato storto con le serie correlate. I dettagli sono nelle opzioni sviluppatore.
event-problem-playback = Qualcosa è andato storto con la riproduzione. I dettagli sono nelle opzioni sviluppatore.
event-problem-store = Qualcosa è andato storto nel salvataggio dei tuoi dati. I dettagli sono nelle opzioni sviluppatore.
event-problem-system = Qualcosa è andato storto in AniBeam. I dettagli sono nelle opzioni sviluppatore.
event-problem-downloads = Qualcosa è andato storto con i download. I dettagli sono nelle opzioni sviluppatore.

## Errori (contract::CoreError)

core-error-not-found-source = Cartella { $id } non trovata.
core-error-not-found-series = Serie { $id } non trovata.
core-error-not-found-file = File { $id } non trovato.
core-error-not-found-session = Sessione di riproduzione { $id } non trovata.
core-error-not-found-job = Attività { $id } non trovata.
core-error-invalid = { $field } non valido: { $message }.
core-error-invalid-ended = { $field } non valido: { $message }
core-error-unavailable = Questa cartella non è disponibile: { $path }
core-error-not-connected = { $tracker } non è collegato.
core-error-refused-hidden = Impossibile: la serie è nascosta.
core-error-refused-no-match = Impossibile: la serie non è ancora abbinata; abbinala in Info serie.
core-error-refused-not-newer = Impossibile: la tua lista è già oltre questo episodio.
core-error-refused-extra = Impossibile: questo è un extra.
core-error-refused-unmatched = Impossibile: il file non è abbinato a un episodio.
core-error-refused-on-disk = Impossibile: il file è solo su disco.
core-error-provider = { $provider } non ha risposto: { $message }
core-error-io = Impossibile leggere o scrivere un file: { $message }
core-error-storage = Impossibile salvare la tua libreria: { $message }
core-error-keyring = Impossibile raggiungere l'archivio password di sistema: { $message }
core-error-unsupported = { $what } non è ancora supportato.
core-error-version = Questa esportazione viene da un AniBeam più recente (formato { $found }). Aggiorna AniBeam per importarla.
core-error-internal = AniBeam ha avuto un errore interno: { $message }
core-error-service = { $service }: { $message }

## core/src/repos/mod.rs

core-repos-paste-address = incolla l'indirizzo di un repo
core-repos-must-be-https = l'indirizzo deve essere https
core-repos-unreachable = impossibile raggiungere il repo: { $message }
core-repos-not-found = niente a quell'indirizzo (anche un repo GitHub privato risponde 404)
core-repos-http-status = il repo ha risposto HTTP { $status }
core-repos-unreadable = impossibile leggere { $path }: { $error }
core-repos-no-such-endpoint = nessun endpoint installato con quell'id
core-repos-key-changed = ora il repo è firmato con una chiave diversa da quella di cui ti sei fidato. Rimuovilo e aggiungilo di nuovo per accettare la nuova chiave.
core-repos-stale = il repo ha servito un indice più vecchio (seriale { $serial } dopo { $kept }); tengo quello più recente
core-repos-key-not-shown = la chiave del repo non è quella che ti è stata mostrata. Aggiungilo di nuovo e confronta l'impronta.
core-repos-repo-added = Aggiunto un elenco di fonti di download.
core-repos-repo-removed = Rimosso un elenco di fonti di download.
core-repos-no-field = { $name } non ha il campo { $key }
core-repos-needs-field = { $name } richiede { $label }
core-repos-host-only = { $name }: { $label } è solo un host o host:porta, come 192.0.2.10:9696
core-repos-not-added = quel repo non è aggiunto
core-repos-not-offered = { $repo } non offre { $id }
core-repos-already-installed = { $name } è già installato
core-repos-taken-by-you = un endpoint chiamato { $id } è già installato (aggiunto da te); rimuovilo prima
core-repos-taken-by-repo = un endpoint chiamato { $id } è già installato (da un altro repo); rimuovilo prima
core-repos-source-added = Aggiunta una fonte di download.
core-repos-source-updated = Aggiornata una fonte di download.
core-repos-source-removed = Rimossa una fonte di download.
core-repos-source-changed = Modificata una fonte di download.
core-repos-give-name = dagli un nome
core-repos-put-query = metti {"{"}query{"}"} nell'indirizzo dove va la ricerca
core-repos-name-taken = un endpoint chiamato { $id } è già installato
core-repos-only-by-hand = si può modificare solo una fonte aggiunta a mano
core-repos-keep-key = lascia {"{"}{ $key }{"}"} nell'indirizzo e cambia il suo valore con Cambia { $label }, oppure rimuovi il sito e aggiungilo di nuovo senza
core-repos-note-kind = anirss non sa cercare nel tipo { $kind }, quindi il desktop lo esclude; il telefono lo cerca
core-repos-note-by-hand = anirss ha un suo { $id }, scritto a mano; il desktop cerca quello
core-repos-note-theirs = Questo anirss lo gestisci tu e il desktop non ne modifica mai la configurazione: aggiungi { $id } alla configurazione di quell'anirss perché il desktop lo cerchi
core-repos-note-sandbox = Questa esecuzione non scrive mai nella configurazione di anirss, quindi qui il desktop non cerca { $id }
core-repos-note-old = anirss { $version } non può ricevere endpoint da { -app-name }; aggiornalo alla { $min } o successiva perché il desktop lo cerchi

## core/src/repos/index.rs

core-repos-key-not-ed25519 = la chiave dell'indice non è una chiave ed25519
core-repos-key-not-base64 = la chiave dell'indice non è di 32 byte in base64
core-repos-key-invalid = la chiave dell'indice non è una chiave ed25519 valida
core-repos-too-large = il file è troppo grande per essere un repo
core-repos-not-a-repo = questo non è un repo di endpoint di { -app-name }
core-repos-format-version = il repo è nel formato versione { $found }; questo { -app-name } legge la versione { $reads }. Aggiorna { -app-name }.
core-repos-payload-not-base64 = il contenuto del repo non è in base64
core-repos-signature-malformed = la firma del repo non è valida
core-repos-index-unreadable = l'indice del repo non si legge: { $error }
core-repos-signature-mismatch = la firma del repo non corrisponde al contenuto
core-repos-check-scheme = { $id }: l'indirizzo deve iniziare con https://
core-repos-check-scheme-http = { $id }: l'indirizzo deve iniziare con http:// o https://
core-repos-check-auth-key = { $id }: chiave di autenticazione { $key } non valida
core-repos-check-host-torznab = { $id }: solo un indirizzo torznab può indicare il suo host come {"{"}{ $key }{"}"}
core-repos-check-host-placeholder = { $id }: un segnaposto nell'host

## core/src/playback/tracks.rs

core-playback-track-number = Traccia { $number }
core-playback-track-file = { $label } (file)
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = Inglese
core-playback-lang-japanese = Giapponese
core-playback-lang-spanish = Spagnolo
core-playback-lang-portuguese = Portoghese
core-playback-lang-french = Francese
core-playback-lang-german = Tedesco
core-playback-lang-italian = Italiano
core-playback-lang-russian = Russo
core-playback-lang-chinese = Cinese
core-playback-lang-korean = Coreano
core-playback-lang-arabic = Arabo
core-playback-lang-hindi = Hindi
core-playback-lang-indonesian = Indonesiano
core-playback-lang-malay = Malese
core-playback-lang-thai = Thailandese
core-playback-lang-vietnamese = Vietnamita
core-playback-lang-polish = Polacco
core-playback-lang-turkish = Turco
core-playback-lang-dutch = Olandese
core-playback-lang-swedish = Svedese
core-playback-lang-norwegian = Norvegese
core-playback-lang-danish = Danese
core-playback-lang-finnish = Finlandese
core-playback-lang-hebrew = Ebraico
core-playback-lang-ukrainian = Ucraino
core-playback-lang-czech = Ceco
core-playback-lang-hungarian = Ungherese
core-playback-lang-greek = Greco
core-playback-lang-romanian = Rumeno
core-playback-lang-filipino = Filippino
core-playback-region-brazil = Brasile
core-playback-region-portugal = Portogallo
core-playback-region-latin-america = America Latina
core-playback-region-spain = Spagna
core-playback-region-simplified = Semplificato
core-playback-region-traditional = Tradizionale

## core/src/contract/enums.rs

core-jobs-scan = Controllo delle cartelle
    .stopped = Il controllo delle cartelle si è fermato per un problema.
core-jobs-auto-match = Abbinamento delle serie
    .stopped = L'abbinamento delle serie si è fermato per un problema.
core-jobs-search = Ricerca di una serie
    .stopped = La ricerca di una serie si è fermata per un problema.
core-jobs-resolve-link = Lettura di un link
    .stopped = La lettura di un link si è fermata per un problema.
core-jobs-apply-match = Abbinamento di una serie
    .stopped = L'abbinamento di una serie si è fermato per un problema.
core-jobs-refresh = Aggiornamento di una serie
    .stopped = L'aggiornamento di una serie si è fermato per un problema.
core-jobs-refresh-all = Aggiornamento info serie
    .stopped = L'aggiornamento delle info serie si è fermato per un problema.
core-jobs-refresh-airing = Controllo di cosa è in onda
    .stopped = Il controllo di cosa è in onda si è fermato per un problema.
core-jobs-refresh-imdb = Aggiornamento dei voti
    .stopped = L'aggiornamento dei voti si è fermato per un problema.
core-jobs-clear-images = Cancellazione delle immagini
    .stopped = La cancellazione delle immagini si è fermata per un problema.
core-jobs-fill-images = Download delle immagini
    .stopped = Il download delle immagini si è fermato per un problema.
core-jobs-connect-tracker = Collegamento della tua lista
    .stopped = Il collegamento della tua lista si è fermato per un problema.
core-jobs-mark = Episodio segnato come visto
    .stopped = Segnare l'episodio come visto si è fermato per un problema.
core-jobs-set-progress = Impostazione dei progressi
    .stopped = L'impostazione dei progressi si è fermata per un problema.
core-jobs-score = Salvataggio del voto
    .stopped = Il salvataggio del voto si è fermato per un problema.
core-jobs-refresh-progress = Lettura dei progressi della lista
    .stopped = La lettura dei progressi della lista si è fermata per un problema.
core-jobs-refresh-watching = Lettura della lista In visione
    .stopped = La lettura della lista In visione si è fermata per un problema.
core-jobs-refresh-airing-now = Controllo di questa stagione
    .stopped = Il controllo di questa stagione si è fermato per un problema.
core-jobs-refresh-recommended = Ricerca di serie per te
    .stopped = La ricerca di serie per te si è fermata per un problema.
core-jobs-read-show-page = Lettura di una serie
    .stopped = La lettura di una serie si è fermata per un problema.
core-jobs-on-hold = Spostamento delle serie ferme in In pausa
    .stopped = Lo spostamento delle serie ferme in In pausa si è fermato per un problema.
core-jobs-set-status = Cambio di stato nella lista
    .stopped = Il cambio di stato nella lista si è fermato per un problema.
core-jobs-move-lists = Spostamento della lista sul tracker
    .stopped = Lo spostamento della lista sul tracker si è fermato per un problema.
core-jobs-notes = Salvataggio delle note
    .stopped = Il salvataggio delle note si è fermato per un problema.
core-jobs-crawl = Ricerca delle serie correlate
    .stopped = La ricerca delle serie correlate si è fermata per un problema.
core-jobs-skip-windows = Ricerca di opening ed ending
    .stopped = La ricerca di opening ed ending si è fermata per un problema.
core-jobs-export = Salvataggio della libreria in un file
    .stopped = Il salvataggio della libreria in un file si è fermato per un problema.
core-jobs-import = Caricamento della libreria da un file
    .stopped = Il caricamento della libreria da un file si è fermato per un problema.
core-jobs-subscriptions = Controllo delle iscrizioni
    .stopped = Il controllo delle iscrizioni si è fermato per un problema.
core-jobs-connect-downloads = Collegamento dell'app di download
    .stopped = Il collegamento dell'app di download si è fermato per un problema.
core-jobs-search-releases = Ricerca di episodi
    .stopped = La ricerca di episodi si è fermata per un problema.
core-jobs-refine-releases = Restringimento degli episodi
    .stopped = Il restringimento degli episodi si è fermato per un problema.
core-jobs-subscribe = Iscrizione a una serie
    .stopped = L'iscrizione a una serie si è fermata per un problema.
core-jobs-download = Aggiunta dei download
    .stopped = L'aggiunta dei download si è fermata per un problema.
core-jobs-remove-subscription = Rimozione di un'iscrizione
    .stopped = La rimozione di un'iscrizione si è fermata per un problema.
core-jobs-control-torrents = Modifica dei download
    .stopped = La modifica dei download si è fermata per un problema.
core-jobs-test-torrent-client = Prova dell'app di download
    .stopped = La prova dell'app di download si è fermata per un problema.
core-jobs-remove-show = Rimozione di una serie
    .stopped = La rimozione di una serie si è fermata per un problema.
core-jobs-repair-subscription = Riparazione di un'iscrizione
    .stopped = La riparazione di un'iscrizione si è fermata per un problema.
core-jobs-check-endpoints = Controllo delle fonti di download
    .stopped = Il controllo delle fonti di download si è fermato per un problema.
core-jobs-start-endpoint = Avvio di una fonte di download
    .stopped = L'avvio di una fonte di download si è fermato per un problema.
core-jobs-schedule-agent = Richiesta del calendario al tuo agente di coding
    .stopped = La richiesta del calendario al tuo agente di coding si è fermata per un problema.

## core/src/airing_now.rs

core-airing-now-now = In onda ora
core-airing-now-winter = Inverno { $year }
core-airing-now-spring = Primavera { $year }
core-airing-now-summer = Estate { $year }
core-airing-now-fall = Autunno { $year }
core-airing-now-planned = { $title } aggiunta alla tua lista Da vedere su { $site }.
core-airing-now-planned-both = { $title } aggiunta alla tua lista Da vedere su { $site } e { $other }.
core-airing-now-already-on =
    { $title } è già nella tua lista di { $site } come { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }.
core-airing-now-plan-failed = Impossibile aggiungere { $title } alla tua lista Da vedere. Riprova più tardi.

## core/src/aired.rs

core-aired-episode-out = È uscito l'episodio { $number } di { $title }.

## core/src/cast/mod.rs

core-cast-connecting = Trasmissione a { $device }.
core-cast-ended = Trasmissione a { $device } terminata.
core-cast-stopped = Trasmissione a { $device } interrotta.
core-cast-failed = Trasmissione a { $device } non riuscita. Controlla che sia sulla stessa rete.
core-cast-not-built = La trasmissione non fa parte di questa build
core-cast-serve-failed = impossibile servire l'episodio: { $error }
core-cast-device-gone = Quel dispositivo non è più sulla rete
core-cast-not-a-playlist = non è l'indirizzo di una playlist
core-cast-nothing = Nessuna trasmissione in corso
core-cast-still-starting = Il dispositivo si sta ancora avviando
core-cast-no-route = questo dispositivo non ha un percorso verso la TV
core-cast-no-control = la TV non ha dato un indirizzo di controllo
core-cast-not-handed = l'altro computer non ha passato l'episodio: { $error }
core-cast-empty-playlist = l'altro computer ha inviato una playlist vuota
core-cast-refused-both = { $error }; ha rifiutato anche il file così com'è ({ $first })
core-cast-chromecast-stopped = il Chromecast si è fermato
core-cast-hevc = { $error }. Questo episodio è in HEVC, che decodificano solo i Chromecast più recenti (con Google TV, o Ultra)

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = impossibile raggiungere il Chromecast: { $error }
core-cast-chromecast-broke = la connessione al Chromecast si è interrotta: { $error }
core-cast-chromecast-no-player = il Chromecast non ha avviato il suo lettore multimediale
core-cast-chromecast-refused-player = il Chromecast non ha voluto avviare il suo lettore multimediale ({ $reason })
core-cast-no-reason = nessun motivo
core-cast-chromecast-cannot-play = il Chromecast non è riuscito a riprodurlo ({ $kind })
core-cast-chromecast-error = il Chromecast si è fermato con un errore durante la riproduzione

## core/src/cast/dlna.rs

core-cast-tv-no-answer = la TV non ha risposto: { $error }
core-cast-tv-refused = la TV ha rifiutato { $action }
core-cast-tv-refused-code = la TV ha rifiutato { $action } ({ $code })

## core/src/cast/media.rs

core-cast-no-video = Questo file non ha video.
core-cast-hi10p = Questo episodio è H.264 a 10 bit (Hi10P), che nessuna TV né Chromecast decodifica. Per trasmetterlo serve una transcodifica, che non è ancora disponibile.
core-cast-video-codec = Il video di questo episodio ({ $codec }) non si può trasmettere.
core-cast-audio-codec = L'audio di questo episodio ({ $codec }) non si può trasmettere.

## core/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Download avviato di un episodio di { $folder }.
       *[other] Download avviato di { $count } episodi di { $folder }.
    }
core-phone-subscribed = Iscritto a { $name }.
core-phone-rule-stopped = { $name } non viene controllata per nuovi episodi: la sua fonte non c'è più o le serve di nuovo la chiave.
core-phone-new-episode = Download di un nuovo episodio di { $name }.
core-phone-name-taken = { $name }: un'altra fonte ha quel nome
core-phone-nothing-to-download = niente da scaricare
core-phone-over-cap = { -app-name } supererebbe il suo limite di spazio ({ $used } di { $cap } GB usati o in arrivo); elimina episodi o alza il limite
core-phone-empty-search = la ricerca è vuota
core-phone-no-results = nessun risultato per { $query }
core-phone-own-engine = un motore torrent proprio

## core/src/phone/check.rs

core-phone-check-connected = connesso
core-phone-check-refused = rifiutato: { $error }
core-phone-check-dropped = nessuna risposta (scartato)
core-phone-check-finding = { $label }: ricerca dell'indirizzo
core-phone-check-failed = non riuscito: { $error }
core-phone-check-no-answer = nessuna risposta
core-phone-check-private = { $label }: ricerca privata
core-phone-check-skipped-address = saltato, è un indirizzo
core-phone-check-skipped-home = saltato, nome della rete di casa
core-phone-check-no-record = nessun indirizzo pubblico, lasciato al DNS di questa rete
core-phone-check-no-resolver = nessun resolver ha risposto
core-phone-check-reaching = { $label }: connessione a { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } byte
core-phone-check-setup = configurazione

## core/src/playback/session.rs

core-playback-not-saved = I tuoi progressi su { $folder } non sono stati salvati. Prima abbina la serie.
core-playback-still-downloading = l'episodio { $number } è ancora in download, al { $done }%

## core/src/playback/skip.rs

core-playback-bad-duration = una durata è un numero positivo di secondi

## core/src/agent.rs

core-agent-no-answer = l'agente non ha dato risposta
core-agent-searching = Ricerca sul web di { $query }
core-agent-searching-schedule = Ricerca sul web del calendario
core-agent-reading = Lettura di { $url }
core-agent-reading-page = Lettura di una pagina
core-agent-using = Uso di { $tool }
core-agent-none = un agente di coding su questo computer
core-agent-ended = { $name } ha finito senza risposta
core-agent-too-long = { $name } ci ha messo troppo

## core/src/core.rs

core-core-guest-trackers = gli account tracker su quel computer sono del suo proprietario, quindi questo dispositivo lì non segna nulla
core-core-guest-not-interested = Non mi interessa su quel computer è la lista del suo proprietario, quindi questo dispositivo lì non cambia nulla
core-core-guest-recommended = i consigli su quel computer sono basati sulla lista del suo proprietario
core-core-nothing-picked = niente di scelto

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider } non è un provider di abbinamento
core-prefs-provider-count = { $provider } compare { $count } volte; ogni provider compare una volta
core-prefs-outside = { $value } è fuori da { $low } a { $high }
core-prefs-empty = vuoto

## core/src/recommended/mod.rs

core-recommended-ranking = Ordino le serie per te

## core/src/recommended/sync.rs

core-recommended-finding-people = Cerco persone con i tuoi gusti
core-recommended-reading-lists = Leggo le loro liste
core-recommended-reading-yours = Leggo le tue serie
core-recommended-reading-suggestions = Leggo le serie da consigliare

## core/src/recommended/page.rs

core-recommended-no-anime = { -anilist } non ha l'anime { $id }

## core/src/host/mod.rs

core-host-off = Off
core-host-starting = Avvio
core-host-serving = In servizio su { $addresses }
core-host-no-tailscale = Tailscale non è connesso
core-host-cannot-listen = Impossibile mettersi in ascolto
core-host-port-taken = La porta { $port } è occupata
core-host-cannot-listen-on = Impossibile mettersi in ascolto su { $address }: { $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text } non è un indirizzo con porta

## core/src/host/keys.rs

core-host-not-a-key = { $key } non è una chiave di dispositivo

## core/src/net/limiter.rs

core-net-timed-out = tempo scaduto dopo { $ms } ms
core-net-rate-limited = { $provider } ha limitato le richieste

## core/src/net/mod.rs

core-net-waiting = { $host }: tempo scaduto poco fa; { -app-name } aspetta { $seconds } s prima di richiedere

## core/src/net/anilist.rs

core-net-no-data = nessun dato

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } non ha un download per questo computer
core-update-cannot-make = Impossibile creare { $path }: { $error }
core-update-cannot-mark = Impossibile rendere eseguibile la nuova AppImage: { $error }
core-update-cannot-replace = Impossibile sostituire { $path }: { $error }
core-update-not-here = Questa copia non si aggiorna da qui
core-update-cannot-start = Impossibile avviare il download: { $error }
core-update-did-not-start = Il download non è partito: { $error }
core-update-cannot-write = Impossibile scrivere { $path }: { $error }
core-update-stopped = Il download si è fermato: { $error }
core-update-cancelled = Annullato
core-update-ended-early = Il download è finito prima del tempo ({ $got } di { $size } byte)
core-update-checksum = Il download non corrisponde al checksum di GitHub
core-update-cannot-move = Impossibile spostare il download al suo posto: { $error }
core-update-cannot-open-dmg = Impossibile aprire il dmg: { $error }
core-update-cannot-read-dmg = Impossibile leggere il dmg: { $error }
core-update-no-app = Il dmg non contiene nessuna app
core-update-cannot-copy = Impossibile copiare la nuova app accanto a questa: { $error }

## core/src/calendar.rs

core-calendar-season-episode = { $title } - Stagione { $season } Episodio { $number }
core-calendar-episode = { $title } - Episodio { $number }

## core/src/paths.rs

core-paths-no-home = nessuna cartella home

## core/src/first_run.rs

core-first-run-not-absolute = la cartella dei video deve essere un percorso assoluto

## core/src/presence/mod.rs

core-presence-extra = Extra

## core/src/trackers/accounts.rs

core-trackers-no-client-id = Nessun client ID impostato per { $tracker }.
core-trackers-mal-reconnect = Ricollega MyAnimeList nelle Impostazioni.
core-trackers-mal-wants-secret = MyAnimeList vuole il client secret di questa app. Aggiungilo in Tracker nelle Impostazioni, o registra l'app con App Type "other".
core-trackers-token-reply-unreadable = risposta del token illeggibile: { $error }
core-trackers-token-reply-no-token = la risposta del token non conteneva un access token

## core/src/trackers/mod.rs

core-trackers-no-token = nessun access token salvato, ricollega nelle Impostazioni

## core/src/trackers/cache.rs

core-trackers-no-user-id = l'account non ha un id utente

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = nessuna serie scelta da spostare
core-trackers-moved-up =
    { $count ->
        [one] { $count } serie spostata da questo computer a { $site }.
       *[other] { $count } serie spostate da questo computer a { $site }.
    }
core-trackers-moved-up-some-refused =
    { $moved ->
        [one] { $moved } serie spostata
       *[other] { $moved } serie spostate
    } su { $site }; { $site } non ha accettato { $failed ->
        [one] { $failed } serie
       *[other] { $failed } serie
    }.
core-trackers-move-refused = { $site } non ha accettato { $show }: { $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = Autorizzazione scaduta, riprova.
core-trackers-oauth-state-mismatch = state non corrispondente, possibile CSRF, interrotto.
core-trackers-oauth-no-code = nessun code nella callback
core-trackers-oauth-no-token = nessun access_token nella callback
core-trackers-oauth-waiting = In attesa che tu acceda a { $site } nel browser.
core-trackers-oauth-connected = { $site } collegato come { $username }.
core-trackers-oauth-no-name = il profilo è tornato senza nome
core-trackers-oauth-could-not-start = Impossibile avviare l'accesso
core-trackers-oauth-port-failed = impossibile aprire la porta { $port } su questo computer: { $error }
core-trackers-oauth-port-busy = la porta { $port } su questo computer è usata da un altro programma. Aspetta un attimo e riprova, o chiudi il programma che la usa se resta occupata.
core-trackers-oauth-provider-error = { $site } ha restituito un errore: { $error }
core-trackers-page-connected = { $site } è collegato a { $app }
core-trackers-page-close-tab = Puoi chiudere questa scheda.
core-trackers-page-failed-title = { $app }, accesso non riuscito
core-trackers-page-failed = L'accesso non è terminato
core-trackers-page-go-back = Torna su { $app } e riprova.
core-trackers-page-finishing-title = { $app }, quasi fatto
core-trackers-page-finishing = Quasi fatto
core-trackers-page-finishing-body = { $site } ha risposto e AniBeam sta completando l'accesso. Torna su { $app }, che ti dirà quando ha finito.
core-trackers-page-cancelled-title = { $app }, accesso annullato
core-trackers-page-cancelled = Questo accesso è stato annullato
core-trackers-page-cancelled-body = Puoi chiudere questa scheda. Se volevi accedere, ricomincia da { $app }.

## core/src/trackers/on_hold.rs

core-trackers-on-hold-moving = Spostamento in In pausa su { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] { $count } serie spostata in In pausa su { $site }.
       *[other] { $count } serie spostate in In pausa su { $site }.
    }
core-trackers-on-hold-moved-two =
    { $count ->
        [one] { $count } serie spostata in In pausa su { $site } e { $second } su { $secondSite }.
       *[other] { $count } serie spostate in In pausa su { $site } e { $second } su { $secondSite }.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } serie non è stata spostata in In pausa.
       *[other] { $failed } serie non sono state spostate in In pausa.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] { $count } serie spostata in In pausa su { $site }.
       *[other] { $count } serie spostate in In pausa su { $site }.
    } { $failed ->
        [one] { $failed } serie non è stata spostata in In pausa.
       *[other] { $failed } serie non sono state spostate in In pausa.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] { $count } serie spostata in In pausa su { $site } e { $second } su { $secondSite }.
       *[other] { $count } serie spostate in In pausa su { $site } e { $second } su { $secondSite }.
    } { $failed ->
        [one] { $failed } serie non è stata spostata in In pausa.
       *[other] { $failed } serie non sono state spostate in In pausa.
    }
core-trackers-series-fallback = serie { $id }
core-trackers-back-on-watching = { $name } è tornata in In visione su { $site }.
core-trackers-back-on-watching-two = { $name } è tornata in In visione su { $site } e { $secondSite }.
core-trackers-back-on-watching-failed = Impossibile rimettere { $name } in In visione. Riprova più tardi.

## core/src/trackers/writes.rs

core-trackers-rate-limited = { $tracker } ha limitato le richieste, riprova tra un minuto.
core-trackers-auth-expired = Accesso a { $tracker } scaduto, ricollega nelle Impostazioni.
core-trackers-entry-not-found = Voce di { $tracker } non trovata.
core-trackers-server-error = Errore del server di { $tracker } ({ $status }), riprova più tardi.
core-trackers-error-see-log = Errore di { $tracker }, vedi il registro attività.
core-trackers-episode-not-whole = { $episode } non è un episodio intero maggiore di zero
core-trackers-score-out-of-range = { $score } è fuori da 0 a 10
core-trackers-rewatch-not-settable = rivedere non è uno stato da impostare a mano
core-trackers-notes-too-long = le note sono limitate a { $max } caratteri
core-trackers-nothing-picked = niente di scelto
core-trackers-notes-not-on-list = Non ancora nella tua lista di { $tracker }, aggiungila prima a una lista.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } e { -mal }
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
       *[repeating] Lo rivedo
    }
core-trackers-marked = Episodio { $episode } di { $show } segnato su { core-trackers-sites }.
core-trackers-progress-set = Progressi di { $show } impostati a { $progress } su { core-trackers-sites }.
core-trackers-scored = Voto { $score } a { $show } su { core-trackers-sites }.
core-trackers-score-cleared = Voto di { $show } cancellato su { core-trackers-sites }.
core-trackers-status-set = { $show } impostata come { core-trackers-status } su { core-trackers-sites }.
core-trackers-notes-saved = Note su { $show } salvate su { core-trackers-sites }.
core-trackers-notes-cleared = Note su { $show } cancellate su { core-trackers-sites }.
core-trackers-update-failed = Impossibile aggiornare { $site }. Riprova più tardi.
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
           *[other] { $done } serie
        }
    }
core-trackers-written-done =
    { $action ->
        [clear] Voto cancellato per { core-trackers-written-who } su { core-trackers-sites }.
        [status] { core-trackers-written-who } impostata come { core-trackers-status } su { core-trackers-sites }.
       *[score] Voto { $score } a { core-trackers-written-who } su { core-trackers-sites }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] Impossibile aggiornare { $failed } serie: { $reason }.
           *[other] Impossibile aggiornare { $failed } serie: { $reason }.
        }
       *[no] { $failed ->
            [one] Impossibile aggiornare { $failed } serie.
           *[other] Impossibile aggiornare { $failed } serie.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] Impossibile aggiornarne { $failed }: { $reason }.
       *[no] Impossibile aggiornarne { $failed }.
    }
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } serie non ha un abbinamento sul tracker.
       *[other] {" "}{ $noMatch } serie non hanno un abbinamento sul tracker.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } serie è nascosta, quindi la sua sincronizzazione è off.
       *[other] {" "}{ $hidden } serie sono nascoste, quindi la loro sincronizzazione è off.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } serie è solo su un tracker non collegato.
       *[other] {" "}{ $notConnected } serie sono solo su un tracker non collegato.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } serie resta su questo computer, senza un tracker a cui inviarla.
       *[other] {" "}{ $kept } serie restano su questo computer, senza un tracker a cui inviarle.
    }

## core/src/metadata/automatch.rs

core-metadata-matched-to = { $show } abbinata a { $title }.
core-metadata-auto-matched =
    { $count ->
        [one] { $count } serie abbinata.
       *[other] { $count } serie abbinate.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } serie ha ancora bisogno di un abbinamento.
       *[other] { $count } serie hanno ancora bisogno di un abbinamento.
    }
core-metadata-auto-matched-some =
    { $matched ->
        [one] { $matched } serie abbinata
       *[other] { $matched } serie abbinate
    }; { $unmatched } { $unmatched ->
        [one] ha
       *[other] hanno
    } ancora bisogno di un abbinamento.
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
        [yes] { $hitFolder } abbinata a { $hitTitle }.
       *[no] { $matched ->
            [one] { $matched } serie abbinata.
           *[other] { $matched } serie abbinate.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] Nessun abbinamento trovato per { $missName }.
       *[no] { $unmatched ->
            [one] Nessun abbinamento trovato per { $unmatched } serie.
           *[other] Nessun abbinamento trovato per { $unmatched } serie.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] Impossibile cercare { $failed } serie; riprova più tardi.
       *[other] Impossibile cercare { $failed } serie; riprova più tardi.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } serie era già abbinata.
       *[other] { $had } serie erano già abbinate.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } serie è già in fase di abbinamento.
       *[other] { $busy } serie sono già in fase di abbinamento.
    }

## core/src/metadata/apply.rs

core-metadata-link-unreadable = Impossibile leggere quel link. Incolla l'URL di una pagina AniList, MyAnimeList, TVmaze o IMDb.
core-metadata-no-mal-entry = AniList non ha una voce per quell'id di MyAnimeList.
core-metadata-no-imdb-show = TVmaze non ha una serie per quel titolo IMDb.
core-metadata-search-on = la ricerca su { $provider }
core-metadata-query-too-short = Scrivi almeno due caratteri per cercare.
core-metadata-not-a-link = non è un link
core-metadata-refresh-unsupported = l'aggiornamento di un abbinamento solo MAL o TMDB
core-metadata-matched = { $show } abbinata.
core-metadata-updated = Info di { $show } aggiornate.
core-metadata-update-failed = Impossibile aggiornare le info di { $show }. Riprova più tardi.
core-metadata-progress-refresh = aggiornamento
core-metadata-progress-backfill = completamento
core-metadata-updated-many =
    { $count ->
        [one] Info aggiornate per { $count } serie.
       *[other] Info aggiornate per { $count } serie.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] Impossibile aggiornare le info di { $count } serie. Riprova più tardi.
       *[other] Impossibile aggiornare le info di { $count } serie. Riprova più tardi.
    }
core-metadata-updated-some =
    { $count ->
        [one] Info aggiornate per { $count } serie; { $failed } non si sono potute aggiornare.
       *[other] Info aggiornate per { $count } serie; { $failed } non si sono potute aggiornare.
    }

## core/src/metadata/fetch.rs

core-metadata-no-such-show = TVmaze non ha la serie { $id }
core-metadata-no-such-media = AniList non ha il media { $id }

## core/src/metadata/imdb.rs

core-metadata-progress-imdb-links = link IMDb di Wikidata
core-metadata-progress-imdb-ratings = voti IMDb

## core/src/franchise/layout.rs

core-franchise-relation-character = Personaggi in comune

## core/src/images/mod.rs

core-images-progress-fetching = download delle immagini

## core/src/transfer/export.rs

core-transfer-exported =
    { $count ->
        [one] { $count } serie salvata in { $folder }.
       *[other] { $count } serie salvate in { $folder }.
    }

## core/src/transfer/format.rs

core-transfer-not-json = non è JSON: { $error }
core-transfer-wrong-format = non è un documento { $format }

## core/src/transfer/import.rs

core-transfer-imported = Libreria caricata dal file.

## core/src/library/delete.rs

core-library-no-such-file = nessun file { $id } nella libreria
core-library-file-elsewhere-not-removed = il file { $id } appartiene a un'altra serie, quindi non è stato rimosso nulla
core-library-file-outside = { $path } non è un file dentro una cartella della libreria, quindi non è stato rimosso nulla
core-library-episodes-removed =
    { $count ->
        [one] { $count } episodio di { $title } { $place ->
            [deleted] eliminato
           *[trash] spostato nel cestino
        }
       *[other] { $count } episodi di { $title } { $place ->
            [deleted] eliminati
           *[trash] spostati nel cestino
        }
    }.
core-library-finished-removed =
    { $count ->
        [one] { $count } serie finita { $place ->
            [deleted] eliminata
           *[trash] spostata nel cestino
        }
       *[other] { $count } serie finite { $place ->
            [deleted] eliminate
           *[trash] spostate nel cestino
        }
    }.
core-library-watched-removed =
    { $count ->
        [one] { $count } episodio visto
       *[other] { $count } episodi visti
    } di { $series } serie { $place ->
        [deleted] { $count ->
            [one] eliminato
           *[other] eliminati
        }
       *[trash] { $count ->
            [one] spostato nel cestino
           *[other] spostati nel cestino
        }
    }.

## core/src/library/detail.rs

core-library-fix-one-file =
    L'unico file qui viene letto come { $read ->
        [episode] episodio { $number }
       *[label] { $label }
    }. Sembra { $what ->
        [film] il film
       *[episode] il suo unico episodio
    }.
core-library-fix-specials =
    { $count ->
        [one] { $count } speciale sembra
       *[other] { $count } speciali sembrano
    } { $episodes ->
        [one] l'episodio mancante
       *[other] gli episodi mancanti
    } { $numbers }.
core-library-fix-numbered-on = Gli episodi { $from } sembrano i { $to } di questa voce, numerati di seguito a una stagione precedente.
core-library-runs-span = da { $first } a { $last }
core-library-runs-list = { $init } e { $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = il numero di un episodio va da 0 a 10000
core-library-fix-season-range = una stagione arriva fino a 200
core-library-fix-extra-range = l'indice di un extra arriva fino a 10000
core-library-file-elsewhere-not-fixed = il file { $id } appartiene a un'altra serie, quindi non è stato corretto nulla
core-library-fix-undone =
    { $count ->
        [one] { $count } file di { $title } torna a essere letto dal nome.
       *[other] { $count } file di { $title } tornano a essere letti dal nome.
    }
core-library-fixed =
    { $count ->
        [one] Corretto come si legge { $count } file di { $title }.
       *[other] Corretto come si leggono { $count } file di { $title }.
    }

## core/src/library/reads.rs

core-library-hid-one = { $title } nascosta.
core-library-back-one = { $title } è tornata.
core-library-hid-many = { $count } serie nascoste.
core-library-back-many = { $count } serie sono tornate.
core-library-episode-count-set = { $title } ora ha { $count } episodi.
core-library-episode-count-cleared = Numero di episodi di { $title } cancellato.

## core/src/library/schedule.rs

core-library-schedule-cleared = Note sul calendario di { $title } cancellate.
core-library-schedule-saved = { $count } note sul calendario salvate per { $title }.

## core/src/library/scan.rs

core-library-source-not-absolute = il percorso di una fonte deve essere assoluto
core-library-source-exists = è già una fonte
core-library-source-nested = è dentro una fonte esistente
core-library-source-added = Cartella { $folder } aggiunta alla tua libreria.
core-library-source-removed = Cartella { $folder } rimossa dalla tua libreria.
core-library-source-removed-with-shows =
    { $count ->
        [one] Cartella { $folder } e la sua { $count } serie rimosse dalla tua libreria.
       *[other] Cartella { $folder } e le sue { $count } serie rimosse dalla tua libreria.
    }
core-library-series-forgotten = { $name } rimossa dalla tua libreria.
core-library-source-back = La cartella { $folder } è tornata.
core-library-source-unavailable = La cartella { $folder } non è disponibile. Controlla che il suo disco sia collegato.
core-library-source-unreadable = Impossibile leggere la cartella { $folder }. Controlla che il suo disco sia collegato.
core-library-source-looks-empty = La cartella { $folder } sembra vuota, quindi le tue serie sono rimaste com'erano.
core-library-source-reconcile-failed = Impossibile aggiornare le serie in { $folder }.
core-library-scan-done =
    Cartelle controllate: { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } serie nuova
           *[other] { $added } serie nuove
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } serie cambiata
           *[other] { $changed } serie cambiate
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }{ $removed } mancanti
    }.

## core/src/library/report.rs

core-library-agent-no-answer = nessuna risposta entro { $span }
core-library-agent-stopped-code = si è fermato con codice { $code }
core-library-agent-stopped = si è fermato
core-library-agent-answered-nothing = non ha risposto nulla
core-library-span-minutes =
    { $count ->
        [one] { $count } minuto
       *[other] { $count } minuti
    }
core-library-span-seconds =
    { $count ->
        [one] { $count } secondo
       *[other] { $count } secondi
    }
core-library-report-no-fix = nessuno di questi file ha una correzione da segnalare
core-library-agent-no-triage = { $agent } non ha scritto un triage: { $why }
core-transfer-progress-sources = fonti
core-transfer-progress-series = serie
core-transfer-progress-history = cronologia
core-transfer-progress-accounts = account
core-transfer-progress-preferences = preferenze
core-transfer-progress-done = fatto

## core/src/downloads/anirss.rs

core-downloads-anirss-no-answer = anirss non ha risposto: { $message }
core-downloads-anirss-answered = anirss ha risposto { $status }: { $text }
core-downloads-no-free-port = nessuna porta libera: { $error }
core-downloads-anirss-exited = anirss è uscito ({ $status })
core-downloads-anirss-exited-said = anirss è uscito ({ $status }): { $said }
core-downloads-anirss-slow-start = anirss non è partito entro { $seconds } secondi
core-downloads-anirss-slow-start-said = anirss non è partito entro { $seconds } secondi: { $said }
core-downloads-anirss-install-mac = Installa anirss { $version } o successivo con: { $command }
core-downloads-anirss-install-linux = Installa anirss { $version } o successivo: il pacchetto anirss su AUR, oppure { $command }
core-downloads-anirss-too-old = anirss { $version } è troppo vecchio per AniBeam, che richiede la { $minimum } o successiva. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } non sa tenere separato l'accesso di ogni client torrent, cosa che serve ad AniBeam per cambiare client; anirss { $minimum } e successivi lo fanno. { $update }.
core-downloads-anirss-update-mac = Aggiornalo con: { $command }
core-downloads-anirss-update-linux = Aggiorna il pacchetto anirss, oppure esegui: { $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = il file torrent non è arrivato: { $message }
core-downloads-torrent-file-answered = il file torrent ha risposto { $status }

## core/src/downloads/feed.rs

core-downloads-feed-answered = il feed ha risposto { $status }
core-downloads-not-a-feed = non è un feed: { $error }

## core/src/downloads/jackett.rs

core-downloads-jackett-missing = Jackett non è installato: { $program } non trovato. Installalo, o cambia il comando nelle Impostazioni.
core-downloads-endpoint-start-failed = Impossibile avviare { $name }: { $why }
core-downloads-endpoint-start-timeout = { $name } si è avviato ma non ha risposto entro { $seconds } secondi.
core-downloads-command-exited = il comando è uscito con { $status }

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirss si è fermato
core-downloads-anirss-missing = { $install }. Oppure indica nelle Impostazioni un server già attivo.
core-downloads-anirss-command-missing = { $command } non trovato. { $install }. Oppure indica nelle Impostazioni un server già attivo.
core-downloads-anirss-start-failed = Impossibile avviare { $command }: { $error }
core-downloads-anirss-too-old-said = { $tooOld } Ha detto: { $message }
core-downloads-no-torrent-client = Nessun qBittorrent risponde a { $address } e AniBeam non ne ha uno suo da avviare. Installa qBittorrent e attiva la sua Web UI, oppure installa qbittorrent-nox, che poi AniBeam avvia per te
core-downloads-own-client-gone = Il qBittorrent di AniBeam non c'è più: qbittorrent-nox non è installato. Reinstalla AniBeam, o indica nelle Impostazioni un tuo qBittorrent
core-downloads-no-port = nessuna porta in { $address }
core-downloads-own-client-settings = impossibile scrivere le impostazioni di qBittorrent: { $error }
core-downloads-own-client-start-failed = Impossibile avviare { $command }: { $error }
core-downloads-own-client-slow-start = Il qBittorrent di AniBeam non è partito
core-downloads-own-client-slow-start-said = Il qBittorrent di AniBeam non è partito: { $said }
core-downloads-not-ready = I download non sono pronti. Controlla Download nelle Impostazioni.
core-downloads-ready = I download sono pronti.
core-downloads-helper-sources =
    { $count ->
        [one] Ora i download cercano con AniBeam stesso, che non legge la { $count } fonte nella configurazione dell'helper di ricerca. Aggiungila in Impostazioni, Download, o scegli lì l'helper di ricerca.
       *[other] Ora i download cercano con AniBeam stesso, che non legge le { $count } fonti nella configurazione dell'helper di ricerca. Aggiungile in Impostazioni, Download, o scegli lì l'helper di ricerca.
    }
core-downloads-client-connected = L'app di download è collegata.
core-downloads-client-not-connected = L'app di download non è collegata. Controlla il suo indirizzo nelle Impostazioni.
core-downloads-not-on-demand = { $name } non è un endpoint su richiesta
core-downloads-source-running = La fonte di download è attiva.
core-downloads-source-not-running = { $name } non è attiva. Accendila nelle fonti e conferma per avviarla.
core-downloads-empty-query = scrivi qualcosa da cercare
core-downloads-first-endpoint = il primo endpoint
core-downloads-search-starting = Avvio della ricerca
core-downloads-search-starting-endpoint = Avvio di { $endpoint }
core-downloads-search-searching-endpoint = Ricerca su { $endpoint }
core-downloads-search-reading = Lettura dei risultati
core-downloads-subscribed = Iscritto a { $name }.
core-downloads-client-no-rule = qBittorrent non ha nessuna regola chiamata { $name }
core-downloads-trying-search = Provo "{ $search }"
core-downloads-nothing-selected = niente di selezionato da scaricare
core-downloads-added =
    { $count ->
        [one] { $count } download aggiunto ai tuoi download.
       *[other] { $count } download aggiunti ai tuoi download.
    }
core-downloads-movie-link = il link di un film deve essere un magnet o un URL http(s)
core-downloads-added-movie = Film aggiunto ai tuoi download.
core-downloads-unsubscribed = Iscrizione a { $name } annullata.
core-downloads-library-folder = { $folder } è la cartella in libreria di { $name }, con episodi che questa iscrizione non ha scaricato: rimuovi l'iscrizione senza i suoi file, o rimuovi { $name } dalla sua pagina
core-downloads-no-torrents-named = nessun torrent indicato
core-downloads-paused =
    { $count ->
        [one] { $count } download in pausa.
       *[other] { $count } download in pausa.
    }
core-downloads-resumed =
    { $count ->
        [one] { $count } download ripreso.
       *[other] { $count } download ripresi.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] { $count } download eliminato insieme ai file.
       *[other] { $count } download eliminati insieme ai file.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] { $count } download rimosso dalla tua lista.
       *[other] { $count } download rimossi dalla tua lista.
    }
core-downloads-nothing-to-remove = niente da rimuovere
core-downloads-removed-show = { $title } rimossa dai tuoi download.
core-downloads-removed-the-show = Serie rimossa dai tuoi download.
core-downloads-deleted-still-listed = I file eliminati sono ancora elencati nell'app di download.
core-downloads-feed-url = un feed è un URL http(s)
core-downloads-rule-needs-name = una regola ha un nome
core-downloads-no-rule = nessuna regola chiamata { $name }
core-downloads-feed-no-search = il feed non contiene una ricerca da accorciare
core-downloads-no-repair = { $name } non ha bisogno di riparazioni.
core-downloads-repaired = Iscrizione a { $name } riparata.
core-downloads-repair-failed = nessuna ricerca più corta trova qualcosa; anche "{ $search }" torna vuota
core-downloads-no-saved-client = nessun client torrent salvato { $id }
core-downloads-client-switched = Ora i download usano un'altra app di download.
core-downloads-host-not-running = non è attivo sull'host
core-downloads-host-late = non ha risposto in tempo

## core/src/downloads/native.rs

core-downloads-native-empty-query = ricerca vuota
core-downloads-native-no-results = nessun risultato per { $query }
core-downloads-native-feed-taken = un feed chiamato { $name } legge già un altro indirizzo
core-downloads-native-no-items = nessun elemento indicato
core-downloads-native-bad-link = i link devono essere magnet: o URL http(s), ricevuto { $link }
core-downloads-native-no-links = nessun link indicato
core-downloads-native-no-names = nessun nome di iscrizione indicato
core-downloads-native-files-elsewhere = il client torrent tiene i suoi file su un'altra macchina, quindi possono andarsene solo insieme ai torrent
core-downloads-native-unknown-feeds = feed sconosciuti: { $names }
core-downloads-native-outside-folders = rifiuto di rimuovere file fuori dalle cartelle di download: { $paths }
core-downloads-refuse-home-folder = { $path } è la cartella home o la contiene, quindi non è stato rimosso nulla
core-downloads-refuse-library-folder = { $path } è una cartella della libreria o ne contiene una, quindi non è stato rimosso nulla

## core/src/downloads/paths.rs

core-downloads-no-default-folder = qBittorrent non ha una cartella di salvataggio predefinita; impostane una in qBittorrent
core-downloads-name-empty = il nome non può essere vuoto
core-downloads-name-slash = il nome non deve contenere / o \
core-downloads-name-not-folder = { $name } non è un nome di cartella

## core/src/downloads/prune.rs

core-downloads-pruned = Iscrizione a { $title } annullata, ha finito di andare in onda.
core-downloads-prune-failed = Impossibile annullare l'iscrizione a { $title }. Controlla Download nelle Impostazioni.

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path } non è un semplice percorso assoluto, quindi non è stato rimosso nulla
core-downloads-refuse-root = { $path } è la radice del filesystem, quindi non è stato rimosso nulla
core-downloads-refuse-outside-library = { $path } non è dentro una cartella della libreria, quindi non è stato rimosso nulla
core-downloads-refuse-save-folder = { $path } è la cartella in cui è stato salvato il torrent, non ciò che ha scaricato, quindi non è stato rimosso nulla

## core/src/downloads/watch.rs

core-downloads-episode-missed = L'episodio { $episode } di { $title } non è comparso tre giorni dopo la messa in onda. Il tuo feed continua a cercarlo.

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = il comando di anirss non può essere vuoto
core-downloads-settings-anirss-address = l'indirizzo di anirss deve iniziare con http:// o https://
core-downloads-settings-client-address = l'indirizzo di { $client } deve iniziare con http:// o https://
core-downloads-settings-bad-id = { $id } non è un id di client: usa lettere, cifre, - e _
core-downloads-settings-same-id = due client hanno l'id { $id }
core-downloads-settings-saved-address = l'indirizzo di { $name } deve iniziare con http:// o https://
core-downloads-settings-same-name = due client si chiamano { $name }; dai a ognuno un nome diverso

## core/src/downloads/qbt.rs

core-downloads-qbt-no-answer = qBittorrent non ha risposto a { $address }: { $message }
core-downloads-qbt-password-rejected = qBittorrent ha rifiutato la password; correggila nelle Impostazioni
core-downloads-qbt-banned = qBittorrent ha bloccato questo indirizzo dopo accessi falliti; aspetta, o riavvia qBittorrent
core-downloads-qbt-login-rejected = qBittorrent ha rifiutato nome utente o password
core-downloads-qbt-login-failed = l'accesso a qBittorrent su { $address } ha risposto { $status }; riprovo alla prossima chiamata
core-downloads-qbt-not-web-ui = { $address } ha risposto { $status }, ma non come la Web UI di qBittorrent. Controlla l'indirizzo nelle Impostazioni e che la Web UI sia attiva in qBittorrent
core-downloads-qbt-not-added = qBittorrent non ha aggiunto i torrent: { $said }
core-downloads-qbt-some-not-added = qBittorrent non ha potuto aggiungere { $count } dei torrent: { $said }
core-downloads-qbt-answered = qBittorrent ha risposto { $status }: { $line }

## core/src/downloads/replace.rs

core-downloads-replace-bad-item = l'elemento { $item } non è una delle { $count } release
core-downloads-not-replaced = L'episodio { $episode } non è stato sostituito.
core-downloads-replaced = Episodio { $episode } sostituito con il nuovo download.

## core/src/downloads/rolling.rs

core-downloads-no-roll = nessun download a rotazione { $id }
core-downloads-rolled = { $title }: ora sugli episodi da { $from } a { $to } di { $total }.
core-downloads-roll-ahead = tieni almeno un episodio in anticipo
core-downloads-roll-nothing = niente da scaricare

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = una fonte di ricerca JSON viene da un repo di endpoint la cui definizione mappa i suoi campi
core-releases-endpoint-no-name = manca `name`
core-releases-endpoint-bad-name = { $name }: usa solo lettere, cifre, '.', '_', '-'
core-releases-endpoint-json = { $name }: { $why }
core-releases-endpoint-unknown-kind = { $name }: tipo { $kind } sconosciuto (validi: { $valid })
core-releases-endpoint-no-url = { $name }: manca `url`
core-releases-endpoint-not-http = { $name }: l'indirizzo deve iniziare con http:// o https://
core-releases-endpoint-needs-query = { $name }: l'url { $kind } deve contenere il segnaposto {"{"}query{"}"}
core-releases-endpoint-no-query = { $name }: un url { $kind } aggiunge da solo la ricerca, quindi non accetta {"{"}query{"}"}

## core/src/releases/fetch.rs

core-releases-indexer-error = l'indexer ha risposto con l'errore { $code }
core-releases-bad-feed = feed non valido: { $error }
core-releases-feed-answered = il feed ha risposto { $status }

## core/src/releases/json_feed.rs

core-releases-not-feed-json = non è il JSON del feed: { $error }

## core/src/releases/json.rs

core-releases-not-json = non è JSON: { $error }
core-releases-search-error-said = la ricerca ha risposto con un errore: { $message }
core-releases-search-error = la ricerca ha risposto con un errore
core-releases-no-results-field = la risposta non ha risultati in { $path }
core-releases-results-not-list = i risultati della risposta non sono una lista
core-releases-search-answered = la ricerca ha risposto { $status }

## core/src/releases/native.rs

core-releases-no-endpoints = ancora nessun endpoint: aggiungi un repo, o uno a mano, in Endpoint in Altro
core-releases-no-source-named = nessuna fonte chiamata { $name }
core-releases-empty-query = scrivi qualcosa da cercare
core-releases-cannot-follow = { $name } non si può seguire: risponde alle ricerche, ma non ha un feed per controllare i nuovi episodi
core-releases-endpoint-gone = { $name } non è più disponibile: rimossa, spenta o senza la sua chiave

## core/src/releases/quirks.rs

core-releases-quirks-variant = one_variant { $word } deve essere una parola breve
core-releases-quirks-pattern-count = show_feed accetta da 1 a { $most } pattern di pagina
core-releases-quirks-pattern-long = un pattern di pagina di show_feed è troppo lungo
core-releases-quirks-pattern-bad = pattern di pagina di show_feed: { $error }
core-releases-quirks-pattern-slug = un pattern di pagina di show_feed richiede un gruppo chiamato slug
core-releases-quirks-url-https = l'url di show_feed deve essere https
core-releases-quirks-url-host = l'url di show_feed deve essere sull'host dell'endpoint
core-releases-quirks-url-slug = l'url di show_feed accetta {"{"}slug{"}"} una volta e nessun altro segnaposto

## core/src/releases/refinement.rs

core-releases-refine-no-tokens = l'op 'tokens' richiede una lista `tokens` non vuota

## core/src/releases/grid.rs

core-releases-group-batch = Batch di { $group }
core-releases-episodes-phrase =
    { $count ->
        [one] Episodio { $list }
       *[other] Episodi { $list }
    }
core-releases-episode-range = da { $from } a { $to }

## Names that stay the same in every language

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = un nome ha da 1 a { $max } caratteri, senza caratteri di controllo
core-prefs-room-relay = l'indirizzo del relay deve iniziare con https:// o http://
core-room-service = Guarda insieme
core-room-not-found = Nessuna stanza ha quel codice. Potrebbe essere stata chiusa.
core-room-full = Quella stanza è piena.
core-room-proto = Quel relay usa un'altra versione delle stanze. Aggiorna { -app-name } per entrare.
core-room-unreachable = Impossibile raggiungere il relay: { $message }
core-room-refused = Il relay ha rifiutato: { $message }
core-room-not-in = Non sei in una stanza.
core-room-no-name = scegli il nome con cui ti vedono gli altri nella stanza
core-room-bad-code = un codice stanza ha 6 tra lettere e cifre
core-room-created = Stanza Guarda insieme avviata. Il codice è { $code }.
core-room-joined = Entrato nella stanza Guarda insieme { $code }.
core-room-left = Uscito dalla stanza Guarda insieme.
core-room-member-joined = { $name } è entrato nella stanza.
core-room-member-left = { $name } è uscito dalla stanza.
core-room-closed = La stanza Guarda insieme è stata chiusa.
core-room-lost = Connessione alla stanza Guarda insieme persa. Nuovo tentativo.
core-room-back = Di nuovo nella stanza Guarda insieme.
