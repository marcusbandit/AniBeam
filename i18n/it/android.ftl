## Names that stay names

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Words many pages share. Use these before adding a page's own copy.

common-cancel = Annulla
common-back = Indietro
common-save = Salva
common-done = Fine
common-close = Chiudi
common-delete = Elimina
common-retry = Riprova
common-copy = Copia
common-paste = Incolla
common-select-all = Seleziona tutto
common-clear = Cancella
common-on = On
common-off = Off
common-play = Riproduci
common-hide = Nascondi
common-remove = Rimuovi
common-add = Aggiungi
common-test = Prova
common-reading = Lettura...
common-working = Attendi...
common-checking = Controllo...
common-all = Tutti
common-none = Nessuno
common-episodes = { $count ->
    [one] { $count } episodio
   *[other] { $count } episodi
}
common-episode-n = Episodio { $n }
common-join-comma = { $a }, { $b }

## Language (#793)

lang-follow-phone = Come il telefono
more-language = Lingua
more-language-phone = Come il telefono: { $lang }
lang-name-en = Inglese
lang-name-da = Danese
lang-name-de = Tedesco
lang-name-es = Spagnolo
lang-name-fr = Francese
lang-name-pt-BR = Portoghese (Brasile)
lang-name-it = Italiano
lang-name-nl = Olandese
lang-name-pl = Polacco
lang-name-ru = Russo
lang-name-ja = Giapponese
lang-name-ko = Coreano
lang-name-zh-CN = Cinese (semplificato)

## Onboarding

onb-welcome-title = AniBeam
onb-welcome-note = I tuoi anime su questo telefono. Trova una serie, scopri chi ce l'ha, scaricala e tieni aggiornate le tue liste.
onb-language-title = Lingua
onb-language-note = AniBeam parla la lingua del telefono, a meno che tu non ne scelga un'altra. Puoi cambiarla in seguito in Altro.
onb-lists-title = Le tue liste
onb-lists-note = Accedi per far contare quello che guardi qui su AniList o MyAnimeList. Puoi farlo in seguito in Altro.
onb-wifi-title = Solo Wi-Fi
onb-wifi-note = Gli episodi sono pesanti. AniBeam scarica solo con il Wi-Fi, salvo diversa scelta.
onb-storage-title = Spazio
onb-storage-note = Un limite a quanto AniBeam può scaricare. Non riserva nulla: lo spazio resta libero finché gli episodi non lo riempiono.
onb-notify-title = Notifiche
onb-notify-note = Durante il download una notifica mostra a che punto sono gli episodi e un'altra avvisa quando uno è pronto da guardare. I download vanno avanti comunque.
onb-sites-title = Siti di ricerca
onb-sites-note = AniBeam non include nulla da cercare. Un repo è un elenco firmato di siti di ricerca. Incolla l'indirizzo di un repo, controlla la sua chiave e installa i siti che vuoi. Puoi farlo in seguito in Altro.
onb-computer-title = Il tuo computer
onb-computer-note = AniBeam su un computer può condividere la sua libreria con questo telefono tramite Tailscale: le sue serie compaiono in Libreria e i loro episodi si riproducono qui. Scrivi il suo nome Tailscale e la porta, poi Prova. Nessun computer? Lascia vuoto e inizia; lo trovi più tardi in Altro.
onb-wifi-only = Solo con il Wi-Fi
onb-wifi-on = On. Con i dati mobili chiede prima.
onb-wifi-off = Off. I download usano anche i dati mobili.
onb-storage-caption = 18 GB sono circa 12 episodi. Trascina fino a tutta la memoria del telefono.
onb-use-cap = Usa il limite
onb-cap-on = On. A { $cap } AniBeam non scarica più nulla finché non elimini episodi.
onb-cap-off = Off. AniBeam scarica finché il telefono non è pieno.
onb-start = Inizia
onb-continue = Continua
onb-back = Indietro

## Appearance

look-colours = Colori
look-mode = Modalità
look-dark = Scuro
look-light = Chiaro
look-system = Sistema
look-follows-phone-dark = Segue l'impostazione del telefono: scuro.
look-follows-phone-light = Segue l'impostazione del telefono: chiaro.
look-changes-at-once = Le modifiche si vedono subito, qui e in ogni pagina.
look-dark-theme = Tema scuro
look-light-theme = Tema chiaro
look-accent = Accento
look-accent-note = Segna ciò che è selezionato, in riproduzione o nuovo.
look-size = Dimensione
look-text-size = Testo
look-size-smaller = Più piccolo
look-size-normal = Normale
look-size-large = Grande
look-size-larger = Più grande
look-size-largest = Massimo
look-text-size-note = Quanto sono grandi le parole in tutta AniBeam, oltre alla dimensione scelta nel telefono.

## The window: tabs and the sheets over every page

app-title = AniBeam
phone-nav-home = Home
nav-calendar = Calendario
phone-nav-library = Libreria
phone-nav-downloads = Download
nav-more = Altro
app-mobile-title = Scaricare con i dati mobili?
app-mobile-note = Un episodio pesa spesso da 1 a 2 GB. Con un piano limitato può consumare i dati di un mese in una sera.
app-mobile-keep = Solo Wi-Fi
app-mobile-use = Usa anche i dati mobili
app-versions-note = Lo stesso episodio da { $count } release. Scegli quella da riprodurre.
app-versions-plays-now = { $release }, in riproduzione
app-menu-get = Scarica episodi
app-menu-delete = Elimina episodi

## More: the index and each section's line of how it stands

more-settings = Impostazioni
more-search = Cerca impostazioni
more-search-none = Nessun risultato
more-search-hits = { $count ->
    [one] 1 impostazione
   *[other] { $count } impostazioni
}
more-search-tap = Tocca per andarci
more-search-hint = Prova una parola più semplice: spazio, Wi-Fi, tracker, sito, computer.
more-accounts = Account
more-streaming = Streaming
more-subtitles = Sottotitoli
more-connection = Connessione
more-computer = Il tuo computer
more-icon = Icona app
more-stats = Statistiche
more-data = Dati
more-appearance = Aspetto
more-activity = Attività
more-about = Info
more-accounts-none = Nessun tracker collegato
more-accounts-syncing = Sincronizza con { $tracker }
more-accounts-syncing-both = Sincronizza con AniList e MAL
more-dl-status-wifi = Solo Wi-Fi, illimitato
more-dl-status-wifi-cap = Solo Wi-Fi, limite { $cap }
more-dl-status-mobile = Wi-Fi e dati mobili, nessun limite
more-dl-status-mobile-cap = Wi-Fi e dati mobili, limite { $cap }
more-stream-status-auto = Auto, 720p con dati mobili o relay
more-stream-status-original = Sempre il file originale
more-stream-status-converted = Convertito in { $quality }
more-sites-status-none = Nessun repo
more-sites-status-on = { $sites ->
    [one] 1 sito attivo
   *[other] { $sites } siti attivi
}
more-sites-status-on-from = { $sites ->
    [one] 1 sito attivo
   *[other] { $sites } siti attivi
}, { $repos ->
    [one] da 1 repo
   *[other] da { $repos } repo
}
more-conn-status-unchecked = Da controllare
more-conn-status-ok = Tutti i passaggi riusciti
more-conn-status-failed = { $failed ->
    [one] 1 passaggio su { $total } non riuscito
   *[other] { $failed } passaggi su { $total } non riusciti
}
more-computer-status-none = Non ancora collegato
more-icon-default = Predefinita
more-icon-dark = Scura
more-icon-line = Line art
more-subs-status = Dimensione, posizione, font, colori e riquadro
more-data-status = Backup, ripristino, cache immagini
more-activity-status = Cosa ha fatto AniBeam, dalle più recenti
more-look-status-phone = Come il telefono
more-stats-status = { $count ->
    [one] { $hours } ora guardata nel { $year }
   *[other] { $hours } ore guardate nel { $year }
}
more-stats-status-empty = Un anno di visione

## More: the sections' own settings

more-accounts-note = Quello che guardi qui aggiorna le tue liste, e i progressi fatti lì compaiono sulle tue serie.
more-storage = Spazio
more-storage-capped = Limite { $cap }
more-storage-no-cap = Illimitato
more-storage-used = usati dagli episodi
more-storage-cap-caption = limite
more-storage-free = liberi sul telefono
more-set-storage-cap = Limite spazio
more-storage-cap-on = A { $cap }, non si scarica più nulla finché non elimini episodi.
more-storage-cap-off = Off: AniBeam scarica finché il telefono non è pieno.
more-network = Rete
more-network-wifi = Solo Wi-Fi
more-network-mobile = Wi-Fi e dati mobili
more-wifi-on = On. I download aspettano il Wi-Fi; con i dati mobili chiede prima.
more-set-stream-quality = Qualità streaming
more-stream-question = Come si riproduce qui un episodio dal tuo computer.
more-stream-auto-note = Auto riproduce il file così com'è con il Wi-Fi, e convertito a 720p con i dati mobili o quando Tailscale raggiunge il computer tramite un relay.
more-stream-original-note = Il file così com'è, sempre. I file grandi richiedono una connessione veloce.
more-stream-converted-note = Convertito sul tuo computer in { $quality }. Usa meno dati; lo spostamento nel video richiede un attimo.
more-set-feeds = I tuoi feed
more-feeds-note = Cercati prima dei siti
more-set-check = Controlla la connessione
more-check-note = Prova ogni passaggio tra questo telefono e i siti di ricerca: trovarli, raggiungerli e stabilire una connessione sicura.
more-check-copy = Copia i risultati
more-set-computer = AniBeam sul tuo computer
more-computer-note = La sua libreria compare in Libreria e i suoi episodi si riproducono qui. Un nome come my-computer o un indirizzo come 100.64.0.1.
more-computer-placeholder = my-computer:8790
more-subs-note = Con cosa parte ogni episodio su questo telefono. Uno in streaming dal tuo computer parte con le impostazioni di quel computer.
more-icon-note = L'icona nella schermata Home e nel drawer delle app.
more-icon-line-note = Line art prende i colori dello sfondo, come le altre icone Material You.
more-icon-moved-note = Alcuni launcher tolgono l'icona dalla Home quando cambia. Resta nel drawer delle app: trascinala da lì.
more-set-whats-new = Novità
more-whats-new-note = Cosa è cambiato in ogni versione, dalla più recente.
more-set-onboarding = Ripeti setup
more-onboarding-note = Ripercorri ancora una volta le domande del primo avvio.
more-set-version = Versione

## Status lines Rust writes on the pages

host-synced = Sincronizzato { $when }.
host-last-try = Ultimo tentativo { $when }: { $error }
host-asking = Richiesta...
host-reached = { $host } raggiunto, AniBeam { $version }.
host-saved = Salvato. Sync...
host-none = Nessun computer. Questo telefono usa solo la sua libreria.
acct-signed-in-as = Accesso come { $name }
acct-signed-in = Accesso fatto
acct-not-in-build = Non disponibile in questa build
acct-tap-to-sign-in = Accesso non eseguito. Tocca per accedere.
acct-sign-in-failed = Accesso non riuscito: { $why }. Tocca per riprovare.
acct-sign-in-not-started = L'accesso non è partito: { $error }. Tocca per riprovare.
acct-still-starting = AniBeam si sta ancora avviando. Riprova tra un attimo.
home-recently-watched = Guardati di recente
home-recently-downloaded = Scaricati di recente
home-note-empty = Sul telefono
home-note = { $count ->
    [one] Su questo telefono, 1 serie
   *[other] Su questo telefono, { $count } serie
}
lib-this-phone = Telefono
lib-note-phone = { $count ->
    [one] Telefono, 1 serie
   *[other] Telefono, { $count } serie
}
lib-note-remote = { $device }, sincronizzato { $when }
lib-empty-search = Nessun risultato per "{ $query }".
lib-empty-phone = Ancora nulla su questo telefono.
lib-empty-remote = Quel dispositivo non ha nulla nella libreria.
cal-view-airing = In onda
cal-note-none = Nulla in onda in settimana
cal-note = { $count ->
    [one] 1 episodio nei prossimi 7 giorni
   *[other] { $count } episodi nei prossimi 7 giorni
}
airing-asking = Chiedo ad AniList
airing-asking-newest = Richiesta ad AniList delle novità
phone-airing-filter-all = Tutto { $count }
airing-filter-listed = Nelle liste { $count }
airing-filter-owned = Sul telefono { $count }
airing-this-season = Questa
airing-next-season = Prossima
airing-other-season = Altre
airing-hidden = { $count ->
    *[other] { $count } nascoste perché non interessano
}
airing-plan-needs-tracker = Per Da vedere serve AniList o MAL. Accedi in Altro, Account.
airing-plan-failed = Da vedere non è andato a buon fine: { $error }
list-watching = In visione
list-planning = Da vedere
list-completed = Completato
list-paused = In pausa
list-dropped = Abbandonato
list-repeating = Lo rivedo
more-check-all-ok = Tutti i passaggi riusciti.
more-check-failed = { $failed ->
    [one] 1 passaggio su { $total } non riuscito. Uno screenshot di questo elenco dice quale.
   *[other] { $failed } passaggi su { $total } non riusciti. Uno screenshot di questo elenco dice quali.
}
more-check-row = { $step }  { $ms } ms
more-check-copied = Copiato. Incollalo in un messaggio.
more-version = AniBeam per Android { $version } (build { $build })
dl-note-waiting = In attesa del Wi-Fi. I download riprendono da soli quando il telefono è sul Wi-Fi.
dl-note-nothing = Nessun download in corso.
dl-state-done = Fatto
dl-state-waiting = In attesa del Wi-Fi
dl-state-paused = In pausa
dl-state-peers = Ricerca peer
dl-state-checking = Controllo
dl-state-error = Fermato da un errore
dl-state-downloading = Download
dl-amount = { $got } di { $size }
dl-speed = { $rate }/s
dl-left = { $span } rimasti
dl-episodes-range = Episodi da { $from } a { $to }
dl-batch = Un batch
disk-storage-line = { $used } usati
disk-storage-line-free = { $used } usati, { $free } liberi
disk-free = Su questo telefono ci sono { $free } liberi.
disk-free-below-cap = Su questo telefono ci sono { $free } liberi, meno del limite: il telefono si riempie prima che AniBeam si fermi.

## More's search: the words a person might type for each setting

more-set-subs-size = Dimensioni sottotitoli
more-set-subs-style = Stile sottotitoli
more-words-anilist = anilist tracker accedi accesso login account collega lista progressi
more-words-mal = mal myanimelist tracker accedi accesso login account collega lista progressi
more-words-storage-cap = spazio disco limite gb pieno memoria elimina dimensione
more-words-wifi = wifi wi-fi dati mobili cellulare rete
more-words-notifications = notifica notifiche avviso progressi permesso tendina consenti
more-words-stream-quality = auto originale 1080p 720p 480p converti dati computer riproduci
more-words-check-repos = aggiorna aggiornamento repo endpoint
more-words-add-repo = repo endpoint fiducia impronta indirizzo url
more-words-add-site = sito personalizzato endpoint feed rss torznab jackett prowlarr indirizzo
more-words-feeds = feed rss fonte rimuovi
more-words-check = rete dns prova https sicura raggiungere internet
more-words-computer = tailscale tailnet host indirizzo desktop libreria streaming prova salva dispositivo
more-words-whats-new = novità changelog note modifiche aggiornamento
more-words-onboarding = configurazione primo avvio ricomincia
more-words-version = build info
more-words-stats = ore episodi generi studi stagioni anno guardato riepilogo mese storico cronologia
more-words-backup = backup esporta file salva copia sposta computer json privati accessi
more-words-restore = ripristina importa file carica libreria unisci
more-words-cache = immagini locandine banner cancella spazio memoria cache
more-words-activity = registro eventi errori problemi copia cancella cronologia cosa è successo
more-words-colours = aspetto tema scuro chiaro modalità sistema colore principale
more-words-text-size = aspetto font testo più grande più piccolo dimensione parole leggere
more-words-icon = icona launcher home schermata logo scuro line art tematica material you
more-words-subs-size = sottotitoli didascalie dimensione scala posizione alza basso
more-words-subs-style = sottotitoli didascalie font colore contorno ombra riquadro grassetto ass stile ripristina
more-words-language = lingua traduzione inglese locale

## Home

phone-home-title = Home
home-next-to-air = Prossima uscita
home-after-that = Dopo
home-coming-up = In arrivo
home-nothing-week = Niente nella prossima settimana.
phone-home-empty-title = Ancora nulla su questo telefono
phone-home-empty-body = Le serie che scarichi qui, e cosa va in onda per loro, compaiono nella Home. Trovane una in Download.

## Library

lib-title = Libreria
lib-refresh = Aggiorna
lib-search-phone = Cerca sul telefono
lib-search-device = Cerca su { $device }
lib-find-in-downloads = Cerca "{ $query }" in Download
lib-empty-body = Gli episodi che scarichi su questo telefono finiscono qui. Trova una serie in Download.
lib-pick-show = Scegli una serie per aprirla qui.
lib-shows-caption = { $count ->
    [one] serie
   *[other] serie
}
lib-on-phone = sul telefono
lib-free = liberi
lib-hold-poster = Tieni premuta una locandina per scaricare altri episodi o eliminarli.
phone-nothing-on-phone = niente sul telefono
phone-on-phone = { $count } sul telefono

## Calendar

cal-title = Calendario
cal-week = Settimana
cal-days = Giorni
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A %-d %B
cal-today = Oggi
cal-tomorrow = Domani
cal-yesterday = Ieri
cal-nothing = Niente
cal-next = Prossimo
cal-get = Scarica
cal-day-note = { $date }, { $count }
cal-next-airing = Prossima: { $title }
cal-nothing-aired-yesterday = Ieri niente in onda.
cal-nothing-aired-that-day = Niente in onda quel giorno.
cal-nothing-today = Niente in onda oggi.
cal-nothing-tomorrow = Niente in onda domani.
cal-nothing-that-day = Niente in onda quel giorno.
cal-empty-title = Ancora nessuna serie su questo telefono
cal-empty-body = Gli episodi delle serie che tieni qui compaiono nel loro giorno, due giorni indietro e una settimana avanti.
phone-episodes-range = Episodi da { $from } a { $to }
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = Sul telefono
phone-out-ago = Uscito { $ago }

## Airing now

phone-airing-out-now = Usciti ora
airing-last-two-days = Ultimi due giorni
airing-season-note = Le serie della stagione vengono da AniList.
airing-none-lists-title = Nulla in onda è nelle tue liste
airing-none-lists-body = Da vedere su una serie la mette nella tua lista.
airing-none-phone-title = Nulla in onda è su questo telefono
airing-none-phone-body = Scarica episodi su una serie per averla qui.
airing-marked = { $title } è segnata come non interessante.
phone-airing-undo = Annulla
airing-plan = Da vedere
airing-get-more = Scarica altri episodi
airing-get = Scarica episodi
phone-airing-not-interested = Non mi interessa

## Seasons

season-winter = Inverno
season-spring = Primavera
season-summer = Estate
season-fall = Autunno
season-named = { $season } { $year }
season-pick = Scegli stagione

## A show's page

series-back-home = Home
series-back-library = Libreria
series-back-downloads = Download
series-back-calendar = Calendario
phone-series-get-episodes = Scarica episodi
phone-series-episodes = Episodi
series-delete-episodes = Elimina episodi
series-delete-tap = Tocca gli episodi da eliminare.
phone-series-delete-picked = { $count ->
    [one] { $count } episodio scelto.
   *[other] { $count } episodi scelti.
}
series-delete-note = I loro file vengono eliminati da questo telefono. Quello che hai guardato resta tracciato.
series-delete-button = { $count ->
    [one] Elimina { $count } episodio
   *[other] Elimina { $count } episodi
}
series-no-episodes = Nessun episodio su questo telefono e nessuno in programma.
series-versions = { $count ->
    [one] { $count } versione
   *[other] { $count } versioni
}
series-note-next-up = Prossimo
series-note-missing = Mancante
series-note-not-here = Non sul telefono
series-note-not-downloaded = Non scaricato
series-note-downloading = Download { $percent }%
series-note-resume-at = Riprendi da { $time }
phone-episodes-estimate = { $count ->
    [one] { $count }? episodio
   *[other] { $count }? episodi
}
phone-next-airs = L'episodio { $n } va in onda tra { $time }
phone-watched = Visto
phone-not-watched = Non ancora visto
phone-progress = { $watched } di { $total } visti, { $count } sul telefono
phone-on-this-phone = { $count } su questo telefono
phone-play-film = Riproduci il film
phone-resume-film = Riprendi il film
phone-play-episode = Riproduci ep. { $n }
phone-resume-episode = Riprendi ep. { $n }
phone-about-date = circa { $date }

## A show's list status, score, notes and fixes

show-status-watching = In visione
show-status-planning = Da vedere
show-status-completed = Completato
show-status-paused = In pausa
show-status-dropped = Abbandonato
show-status-repeating = Lo rivedo
show-status-now = { $label }, attuale
phone-show-add-to-list = Aggiungi a lista
phone-show-your-score = Voto { $score }
show-rate = Vota
show-fixes = Correggi
show-your-notes = Le tue note
show-add-notes = Aggiungi note
show-home-each = Cambialo su ogni lista collegata.
show-home-no-tracker = Salvato su questo telefono: nessun tracker ha questa serie.
show-home-tracker-lacks = Salvato su questo telefono: il tuo tracker non ha questa serie.
show-home-until-connected = Salvato su questo telefono finché non colleghi AniList o MyAnimeList.
show-sites-both = AniList e MyAnimeList
show-notes-title = Note
show-notes-placeholder = A che punto sei, chi te l'ha consigliato, quale release scaricare
show-notes-over = { $count ->
    [one] { $count } carattere oltre il limite
   *[other] { $count } caratteri oltre il limite
}
show-notes-saved-to = Salvato su { $sites }.
show-notes-saved-to-tracker = Salvato sul tuo tracker.
show-notes-saving = Salvataggio delle note.
show-notes-clearing = Cancellazione delle note.
show-score-saving = Salvataggio del voto.
show-score-saved = Voto salvato.
show-score-clearing = Cancellazione del voto.
show-score-cleared = Voto cancellato.
phone-show-moving-to = Spostamento in { $label }.
phone-show-moved-to = Spostato in { $label }.
show-rescan-started = Scansione avviata.
show-still-starting = AniBeam si sta ancora avviando. Riprova tra un attimo.
show-fix-match = Abbina
show-fix-find-match = Cerca serie
show-fix-rescan = Riscansiona

## A show's characters, related shows and recommendations

show-characters = Personaggi
phone-show-related = Correlate
show-related-all = Mostra tutte { $count }
show-related-filling = Ancora in caricamento da AniList.
phone-show-recommendations = Consigliate
show-card-owned = Nella tua libreria
show-card-unreleased = Non ancora uscita
show-card-on-anilist = Su AniList
show-rec-available = Disponibile
show-relation-prequel = Prequel
show-relation-sequel = Sequel
show-relation-side-story = Side story
show-relation-parent = Principale
show-relation-spin-off = Spin-off
show-relation-adaptation = Adattamento
show-relation-alternative = Alternativa
show-relation-summary = Riassunto
show-relation-character = Personaggio
show-relation-source = Fonte
show-relation-compilation = Raccolta
show-relation-contains = Contiene
show-relation-other = Altro
show-role-main = Principale
show-role-supporting = Secondario
show-role-background = Comparsa
show-format-tv = TV
show-format-tv-short = TV breve
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Speciale
show-format-film = Film
show-entry-get = Scarica episodi
show-entry-open-anilist = Apri su AniList
show-entry-line = { $line }, { $status }

## Fix a wrong match

phone-match-title = Abbina
match-refresh = Aggiorna info
phone-match-clear = Rimuovi
match-clear-it = Rimuovi
match-clear-warning = Rimuovere elimina il legame con il tracker e il numero di episodi impostato a mano. Tocca ancora per rimuovere.
match-tvmaze = TVmaze
match-search-anilist = Cerca su AniList
match-search-tvmaze = Cerca su TVmaze
match-paste-anilist = Incolla un link di AniList o MyAnimeList
match-paste-tvmaze = Incolla un link di TVmaze o IMDb
phone-match-look-up = Cerca
match-apply = Applica
match-look-up-all = Cerca ogni cartella che AniBeam non conosce ancora
match-format-music = Musica
phone-match-matched-on = Abbinata su { $sites }
phone-match-found-tvmaze = Serie trovata su TVmaze
phone-match-found-anilist = Trovata su AniList
phone-match-found-anilist-season = Trovata su AniList, stagione { $season }
phone-match-found-mal = Trovata su MyAnimeList
phone-match-searching = Ricerca su { $name }
phone-match-type-more = Scrivi almeno due lettere per cercare su { $name }.
match-none = Nessun titolo trovato su { $name }.
match-looking-up = Ricerca in corso.
phone-match-matched-to = Abbinata a { $title }.
phone-match-applied = Abbinamento fatto.
match-applying = Applico l'abbinamento.
phone-match-refreshing = Aggiornamento di { $title }.
match-clearing = Rimuovo l'abbinamento.
phone-match-cleared = Abbinamento tolto.
match-auto-looking = Ricerca delle cartelle che AniBeam non conosce ancora.

## Batches (the line under a release in a search)

inspect-reading = Lettura dell'elenco file
inspect-holds = { $count ->
    [one] { $count } episodio: { $runs }
   *[other] { $count } episodi: { $runs }
}

## What's new

phone-notes-title = Novità
notes-got-it = Ho capito

## Move the list to a tracker

move-lists-label = Liste su questo telefono
move-title = Sposta la lista su { $site }
move-here = Su questo telefono: { $list }
move-there = Ora su { $site }: { $list }
move-later-hint = Dopo: Altro, Account
move-not-now = Non ora
move-nothing = Niente
move-not-on-list = Non nella tua lista
move-nothing-left = Nulla da spostare
move-side-status-score = { $status }, voto { $score }
move-side-score = voto { $score }
move-lead = { $count ->
    [one] Una serie ha uno stato o un voto solo su questo telefono. Spostandola finisce nella tua lista { $site }, dove la vedono il computer e il tuo profilo. Per questa serie, quello che { $site } ha ora viene sostituito. Non viene inviato nulla finché non premi Sposta.
   *[other] { $count } serie hanno uno stato o un voto solo su questo telefono. Spostandole finiscono nella tua lista { $site }, dove le vedono il computer e il tuo profilo. Per queste serie, quello che { $site } ha ora viene sostituito. Non viene inviato nulla finché non premi Sposta.
}
move-label = { $count ->
    [one] Sposta { $count } serie
   *[other] Sposta { $count } serie
}
move-waiting = { $count ->
    [one] Una serie ha uno stato o un voto solo qui. Spostala su { $site } per vederla sul computer e nel tuo profilo.
   *[other] { $count } serie hanno uno stato o un voto solo qui. Spostale su { $site } per vederle sul computer e nel tuo profilo.
}
move-moving = { $count ->
    [one] Spostamento di { $count } serie su { $site }
   *[other] Spostamento di { $count } serie su { $site }
}
move-moved = { $count ->
    [one] { $count } serie spostata su { $site }
   *[other] { $count } serie spostate su { $site }
}
move-failed = { $count ->
    [one] { $site } non ha accettato { $count } serie; vedi il registro attività
   *[other] { $site } non ha accettato { $count } serie; vedi il registro attività
}

## Downloads: find (search, shows, sources, releases)

common-join-and = { $a } e { $b }
find-join-sentences = { $a } { $b }

find-downloads = Download

find-lens-list = La tua lista
find-lens-gaps = Mancanti
find-lens-airing = In onda
find-lens-following = Seguite
find-lens-downloaded = Scaricate
find-lens-with-count = { $lens } { $count }
find-what-to-get = Cosa scaricare
find-lens-list-line = In visione e Da vedere nella tua lista. Aprine una per scaricarne gli episodi.
find-lens-gaps-line = Episodi già andati in onda che non sono su questo telefono. Apri una serie per vedere chi li ha.
find-lens-airing-line = Nuovi episodi ancora in arrivo, i più vicini per primi.
find-lens-following-line = Le iscrizioni: i nuovi episodi si scaricano da soli. Aprine una per fermarla.
find-lens-downloaded-line = Quello che questo telefono ha scaricato, una serie alla volta.

find-lens-gaps-empty = Nessun episodio mancante: ogni serie iniziata ha qui i suoi episodi già andati in onda.
find-lens-airing-empty = Nulla su questo telefono è in onda.
find-lens-following-empty = Ancora nessuna iscrizione. Apri una serie e attiva Segui.
find-lens-downloaded-empty = Ancora nulla di scaricato.
find-list-empty = Ancora nulla nella tua lista. Cerca una serie qui sopra e aggiungila.
find-list-reading = Lettura della tua lista...
find-list-signed-out = Accedi ad AniList in Altro, Account, e la tua lista comparirà qui.
find-list-failed = Impossibile leggere la tua lista: { $error }

find-aired = uscito
find-ledger-no-feed = nessun nuovo episodio: la fonte non ha un feed
find-ledger-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } download
}
find-ledger-downloaded = scaricato
find-ledger-planning = Da vedere
find-ledger-watching-of = In visione, { $progress } di { $total }
find-ledger-watching-seen = In visione, { $progress } visti

find-gap-missing = { $count } mancanti
find-gap-not-downloaded = { $count } non scaricati

find-list-status-planning = Nella tua lista: Da vedere
find-list-status-watching = Nella tua lista: In visione
find-list-status-already = Già nella tua lista
find-going-planning = { $title } va in Da vedere.
find-already-listed = { $title } è già nella tua lista.
find-plan-needs-tracker = Per aggiungere alla lista serve AniList o MAL. Accedi in Altro, Account.
find-plan-failed = Non aggiunta alla lista: { $error }
find-show-gone = Quella serie non è più nella libreria di questo telefono.

## Shows as you type

find-shows = Serie
find-suggesting = Chiedo ad AniList...
find-suggest-tap = Tocca una serie per vedere chi ce l'ha. Invio cerca i nomi delle release.
find-suggest-none = Nessuna serie con quel nome. Invio cerca invece i nomi delle release.
find-suggest-hint = Le serie compaiono mentre scrivi. Invio cerca i nomi delle release.
find-suggest-failed = Impossibile interrogare AniList: { $error }
find-library-line = Nella tua libreria
find-library-line-none = Nella tua libreria, nessuno di { $total } sul telefono
find-library-line-some = Nella tua libreria, { $here } di { $total } sul telefono

## The show view

find-plan-to-watch = Da vedere
find-on-phone-count = { $count } su questo telefono
find-airing = In onda
find-dropped = { $count ->
    [one] { $count } release che non nomina questa serie è stata esclusa.
   *[other] { $count } release che non nominano questa serie sono state escluse.
}

find-following = Seguita
find-stop-following-ask = Smettere di seguire { $name }?
find-unfollow-explain = I nuovi episodi non si scaricano più. Quello che c'è nella sua cartella può restare o sparire con lei: ogni download, compresi gli episodi presi a mano.
find-stop-following = Smetti di seguire
find-stop-keep = Smetti, tieni gli episodi
find-stop-delete = Smetti ed elimina i download della cartella
find-rule-taken = { $downloads } download, { $files } file nella sua cartella
find-rule-paused = È in pausa, quindi non arrivano nuovi episodi. { $taken }.
find-rule-no-feed = La fonte non ha un feed, quindi non arrivano nuovi episodi. { $taken }.
find-rule-active = I nuovi episodi si scaricano da soli. { $taken }.
find-unfollowed-deleted = Hai smesso di seguire { $name } e i download della sua cartella sono stati eliminati.
find-unfollowed-kept = Hai smesso di seguire { $name }. I suoi episodi restano su questo telefono.
find-stop-failed = L'interruzione non è finita: { $error }

find-where = Dove scaricarla
find-where-pick = Scegli una fonte per vedere cosa ha.
find-where-nothing = Non si scarica nulla finché non lo dici tu.
find-source-idle = Cerca
find-source-trying = Provo { $name }
find-source-searching = Ricerca
find-source-silent = Non risponde
find-no-sites-title = Nessun sito di ricerca
find-no-sites-body = Aggiungi un repo di siti di ricerca, o un feed tuo, in Altro.
find-open-more = Apri Altro
find-pick-source-title = Scegli una fonte
find-pick-source-body = Le sue release compaiono qui in una griglia, un cubo per episodio.
find-releases = { $count ->
    [one] { $count } release
   *[other] { $count } release
}
find-unreachable = Impossibile raggiungere { $host }. Alcune reti e paesi bloccano i siti di release; AniBeam aggira già i blocchi sui nomi, quindi questo blocca il sito stesso, oppure il sito è fuori servizio. Una VPN lo aggira, oppure prova un'altra fonte.
find-the-source = La fonte

find-has = Cosa ha
find-has-note = Tocca una riga per prendere tutto, o i singoli episodi.
find-episodes-tag = Episodi
find-batch = Batch { $label }
find-row-meta = { common-episodes }  { $seeds } seed  { $size }
find-on-this-phone = Sul telefono
find-on-computer = Su { $name }

find-plan-no-releases = Questa fonte non ha release di questa serie.
find-plan-nothing-picked = Nulla di scelto: tocca una riga o i suoi cubi.
find-plan-line = { $episodes } da { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] { $count } episodio che hai già è stato escluso. Attiva Sostituisci per scaricarlo di nuovo.
   *[other] Gli episodi che hai già sono stati esclusi. Attiva Sostituisci per scaricarli di nuovo.
}
find-plan-follow-note = I nuovi episodi da { $query } si scaricheranno da soli.
find-plan-on-desk = Si scaricano su { $name }, nella sua libreria.
find-follow-new = Segui i nuovi episodi
find-follow-new-note = Si scaricano da soli quando vanno in onda.
find-replace = Sostituisci quelli che hai
find-replace-note = Scarica di nuovo gli episodi scelti sopra quelli su questo telefono.
find-go = { $mode ->
    [replace] Scarica { common-episodes } e sostituisci
    [subscribe] Scarica { common-episodes } e iscriviti
    [both] Scarica { common-episodes }, sostituisci e iscriviti
   *[plain] Scarica { common-episodes }
}
find-go-pick = Scegli episodi
find-every-release = Ogni release di questa fonte

## Download to

find-download-to = Scarica su
find-this-phone = Telefono
find-your-computer = Il tuo computer
find-desk-asking = Richiesta a { $name }...
find-desk-silent = { $name } non risponde. Scegli Telefono, o riprova quando è acceso.
find-sending-to = Invio a { $name }...

## The release list

find-best-fit = Migliore
find-on-disk = Su disco
find-get = Scarica
find-get-all = Scarica { $count }
find-subscribe = Iscriviti
find-back-step = Indietro di uno
find-narrow-placeholder = Restringi, o -parola per escludere
find-best-fit-searches = La migliore cerca: { $query }
find-fact-batch = batch
find-fact-seeds = { $count } seed
find-fact-via = via { $source }
find-searching-for = Ricerca di { $text }...
find-search-line = { $count ->
    [0] Nulla su { $source } per questa ricerca
    [one] { $count } release su { $source }
   *[other] { $count } release su { $source }
}{ $fell ->
    [1] , dopo che la prima fonte non aveva nulla
   *[other] { "" }
}{ $narrowed ->
    [1] , ristretta a { $token }
   *[other] { "" }
}.
find-refined = { find-releases }, { $dir ->
    [down] { $delta } in meno
   *[up] { $delta } in più
}.
find-refine-nothing-left = Così non resta nulla.
find-refine-no-match = Nessuna release lo ha.
find-refine-empty = Scrivi prima qualcosa.
find-refine-no-change = Non cambia nulla.

## Getting and subscribing

find-getting = Download di { $what }.
find-subscribed = Iscritto: i nuovi episodi si scaricano da soli.
find-subscribed-desk = Iscritto su { $name }: i nuovi episodi si scaricano lì da soli.
find-subscribed-as = Iscritto come { $name }: i nuovi episodi si scaricano da soli.
find-downloading = { $count ->
    [one] Download di { $count } release.
   *[other] Download di { $count } release.
}
find-not-added = Non aggiunto: { $error }
find-not-subscribed = Non iscritto: { $error }
find-desk-refused = { $name } non l'ha accettato: { $error }
find-desk-not-subscribed = Non iscritto su { $name }: { $error }

## Player

player-subtitles = Sottotitoli
player-audio = Audio
phone-player-speed = Velocità
player-connecting = Connessione
player-hold-speed = Velocità 2×
player-replay = Rivedi
player-unlock = Sblocca
player-next-stay = Resta
player-next-go = Prossimo
player-jump-back = −10 s
player-jump-forward = +10 s
player-speed-now = Velocità { $speed }
player-skip-intro = Salta intro
player-skip-outro = Salta outro
phone-player-skipped-intro = Intro saltata
phone-player-skipped-outro = Outro saltata
phone-player-undo = Annulla
phone-player-start-over = Ricomincia
phone-player-resumed-at = Ripreso da { $time }
player-picture-failed = Impossibile avviare il video. L'audio continua.
player-file-failed = Questo file non si è riprodotto: { $why }.
player-streaming-from = Streaming da { $host } a { $height }p
player-mpv-no-start = mpv non si è avviato
player-mpv-no-start-why = mpv non si è avviato: { $why }
player-android-only = il player funziona solo su Android

## Player: delays

player-delay-title = Ritardo
player-delay-note = Ogni tocco lo sposta di 0,1 s. Fino al file successivo.
phone-player-delay-seconds = { $value } s
player-sub-delay-now = Ritardo sottotitoli { $value }
player-audio-delay-now = Ritardo audio { $value }

## Player: cast

player-cast-playing-on = Riproduce su { $device }
phone-player-cast-stop = Interrompi il cast su { $device }
player-cast-searching = Ricerca di Chromecast e TV
phone-player-cast-to = Trasmetti a
phone-player-cast-tv = TV
phone-player-cast-loading = Caricamento
phone-player-cast-playing = Riproduce
phone-player-cast-paused = In pausa
phone-player-cast-finished = Terminato
phone-player-cast-stopped = Fermato

## Player: the tracker's word

phone-player-rated = Votato
phone-player-score-failed = Impossibile salvare il voto
player-marked-watched = Segnato come visto
player-tracked = Tracciato
player-not-tracked = Non tracciato
player-track-update-failed = Impossibile aggiornare la lista
player-not-tracked-why = Non tracciato: { $reason }.
player-track-ok-at = { $tracker }, ep. { $progress }
player-track-ok = { $tracker } ok
player-track-note = { $tracker }: { $note }
player-track-failed = Impossibile aggiornare { $tracker }
player-track-line = { $head }: { $parts }
phone-player-refusal-hidden = la serie è nascosta
phone-player-refusal-no-match = la serie non ha un abbinamento sul tracker
phone-player-refusal-not-newer = il tracker è già oltre questo episodio
phone-player-refusal-extra = è un extra
phone-player-refusal-unmatched = il file non è abbinato a un episodio
phone-player-refusal-on-disk = il file è solo su disco

## Subtitles

subs-line-default = Dimensione 1×, i file semplici usano il tuo stile
subs-sample = Dove l'hai nascosto?
subs-size-place-title = Dimensione e posizione
subs-size = Dimensione
subs-position = Posizione
subs-position-helper = 100 sta sul bordo inferiore. I numeri più bassi alzano le righe nel video.
subs-style-title = Stile
subs-styled = Sottotitoli con stile
subs-styled-as-made = I file con stile (ASS) restano esattamente come li hanno impostati gli autori.
subs-styled-mine = Il tuo stile qui sotto vale per ogni file. Cartelli e karaoke possono rompersi.
subs-styled-size-only = I file con stile mantengono l'aspetto e prendono la tua dimensione. I file semplici prendono il tuo stile.
subs-mode-as-made = Originale
subs-mode-size-only = Solo misura
subs-mode-mine = Mio
subs-font = Font
subs-font-bad = Scrivi il nome di un font, come Noto Sans.
subs-bold = Grassetto
subs-colour = Colore testo
subs-colour-bad = Scrivi un colore come #ffcc00.
subs-outline = Contorno
subs-outline-bad-width = Lo spessore è un numero, 0 o più.
subs-outline-bad-colour = Scrivi il colore come #000000.
subs-outline-helper = Prima lo spessore, poi il colore.
subs-shadow = Ombra
subs-shadow-bad = Un numero, 0 o più. 0 significa nessuna ombra.
subs-shadow-helper = Quanto cade lontano l'ombra, 0 per nessuna.
subs-box = Riquadro di sfondo
subs-box-helper = Un riquadro scuro dietro le righe. 0 lo toglie.
subs-reset-title = Ricomincia
subs-reset-helper = Riporta dimensione, font, colori, contorno, riquadro e posizione come erano all'inizio.
subs-reset = Azzera

## Streaming

stream-quality-auto = Auto
stream-quality-original = Originale
stream-app-starting = L'app si sta ancora avviando
stream-no-computer = Ancora nessun computer. Aggiungi il tuo in Altro.
stream-off-on-host = Lo streaming è disattivato su { $host }. Attiva Stream episodes lì, in Settings, Phone app.
stream-cannot-stream = { $host } usa AniBeam { $version }, che non può ancora fare streaming. Aggiornalo lì.
stream-no-session = L'altro dispositivo non ha dato una sessione
stream-answer-unreadable = La risposta non si è letta: { $why }
stream-answered-status = { $name } ha risposto { $status }

## Stats

phone-stats-nothing-yet = Ancora nulla
phone-stats-empty-title = Ancora nulla di guardato
phone-stats-empty-body = Finisci un episodio e conta qui: ore, generi e le stagioni che hai guardato.
stats-hours-caption = ore guardate
stats-finished-caption = serie finite
stats-busiest-caption = mese più intenso
stats-episodes-caption = { $count ->
    [one] episodio
   *[other] episodi
}
phone-stats-by-month = Per mese
phone-stats-top-genres = Generi top
phone-stats-top-studios = Studi top
stats-seasons = Stagioni guardate
stats-watched-most = Più guardati
stats-year-line = Cosa hai guardato nel { $year }
stats-eps = { $count ->
    [one] { $count } ep.
   *[other] { $count } ep.
}
stats-month-1 = Gen
stats-month-2 = Feb
stats-month-3 = Mar
stats-month-4 = Apr
stats-month-5 = Mag
stats-month-6 = Giu
stats-month-7 = Lug
stats-month-8 = Ago
stats-month-9 = Set
stats-month-10 = Ott
stats-month-11 = Nov
stats-month-12 = Dic
stats-month-letter-1 = G
stats-month-letter-2 = F
stats-month-letter-3 = M
stats-month-letter-4 = A
stats-month-letter-5 = M
stats-month-letter-6 = G
stats-month-letter-7 = L
stats-month-letter-8 = A
stats-month-letter-9 = S
stats-month-letter-10 = O
stats-month-letter-11 = N
stats-month-letter-12 = D

## Backup and image cache

backup-title = Backup
backup-export = Backup della libreria
backup-export-helper = Un file con le tue cartelle, gli abbinamenti e quello che hai guardato.
backup-private = Includi dati privati
backup-private-helper = Aggiunge gli accessi ad AniList e MyAnimeList, la cronologia di visione e le preferenze. Gli accessi sono scritti in chiaro, quindi tieni il file per te.
backup-restore = Ripristina da un backup
backup-restore-helper = Legge un backup in questa libreria. Quello che c'è su disco ha la precedenza; il file completa il resto.
backup-done = Backup eseguito. Il file è dove l'hai salvato.
backup-not-saved = Non salvato. Non è stato scritto nulla.
backup-starting = AniBeam si sta ancora avviando. Riprova tra un attimo.
backup-images-title = Cache immagini
backup-images-body = Locandine, banner e ritratti restano sul telefono, così la libreria funziona offline. Le immagini cancellate si scaricano di nuovo quando le pagine le richiedono.
backup-keep = Tieni
backup-removed = { $count ->
    [one] { $count } immagine rimossa
   *[other] { $count } immagini rimosse
}
backup-images-none = Cache vuota
backup-images-unmeasured = Impossibile misurare la cache immagini
backup-images-retry-note = Riprova tra un attimo. Le immagini stesse vanno bene.
backup-images-line = { $size } in { $count ->
    [one] { $count } immagine
   *[other] { $count } immagini
}
backup-images-question = { $count ->
    [one] Rimuovere { $count } immagine?
   *[other] Rimuovere { $count } immagini?
}

backup-sum-list = { $items }.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] { $count } cartella aggiunta
   *[other] { $count } cartelle aggiunte
}
backup-sum-series = { $count ->
    [one] { $count } serie creata
   *[other] { $count } serie create
}
backup-sum-matches = { $count ->
    [one] { $count } abbinamento applicato
   *[other] { $count } abbinamenti applicati
}
backup-sum-views = { $views ->
    [one] { $views } visione
   *[other] { $views } visioni
} e { $episodes ->
    [one] { $episodes } episodio finito
   *[other] { $episodes } episodi finiti
} uniti
backup-sum-resume = { $count ->
    [one] { $count } punto di ripresa unito
   *[other] { $count } punti di ripresa uniti
}
backup-sum-accounts = { $count ->
    [one] { $count } account di liste importato
   *[other] { $count } account di liste importati
}
backup-sum-fixes = { $count ->
    [one] { $count } file prende una correzione che hai fatto.
   *[other] { $count } file prendono una correzione che hai fatto.
}
backup-sum-missing = { $count ->
    [one] { $count } cartella non è stata trovata su questo telefono.
   *[other] { $count } cartelle non sono state trovate su questo telefono.
}
backup-sum-left-out = Escluso: { $fields }.

## Activity

activity-everything = Tutto
activity-problems = Problemi
activity-copied = Copiato
activity-clear-every-line = Cancella ogni riga
activity-fold = Comprimi
activity-no-problems = Nessun problema. Tutto ha funzionato.
activity-nothing-yet = Ancora nulla da segnalare. Quello che AniBeam fa per te compare qui.
activity-time-today = %H:%M:%S
activity-time-other = %-d %b %H:%M

## Dates and numbers (companion format.rs)

fmt-clock = %H:%M
fmt-day-month = %-d %b
fmt-weekday-date-time = %a %-d %b %H:%M
fmt-weekday-date-year-time = %a %-d %b %Y %H:%M
fmt-stamp = %-d %b %H:%M
fmt-decimal-point = ,

fmt-today-at = Oggi { $time }
fmt-tomorrow-at = Domani { $time }
fmt-yesterday-at = Ieri { $time }

fmt-span-days = { $d } g { $h } h
fmt-span-hours = { $h } h { $m } min
fmt-span-minutes = { $m } min { $s } s
fmt-span-seconds = { $s } s
phone-fmt-in = tra { $span }
fmt-full-days = { $d } g { $h } h { $m } min { $s } s
fmt-full-hours = { $h } h { $m } min { $s } s
fmt-full-minutes = { $m } min { $s } s

fmt-ago-now = adesso
fmt-ago-minutes = { $count } min fa
fmt-ago-hours = { $count } h fa
fmt-ago-days = { $count } g fa

fmt-bytes-b = { $value } B
fmt-bytes-kb = { $value } kB
fmt-bytes-mb = { $value } MB
fmt-bytes-gb = { $value } GB
fmt-bytes-tb = { $value } TB

## Talking to the computer (companion client.rs, sync.rs, address.rs)

client-not-found = { $host } non è un nome che il telefono riesce a trovare. Tailscale è connesso?
client-timeout = { $host } non ha risposto. È acceso e Tailscale è connesso su entrambi?
client-connect-failed = Impossibile raggiungere { $host }. È acceso e Tailscale è connesso su entrambi?
client-refused = { $host } è acceso, ma AniBeam lì è chiuso o non condivide. Apri AniBeam e attiva Condividi tramite Tailscale in Impostazioni, App telefono.
client-no-route = Nessun percorso verso { $host }. Tailscale è connesso?
client-invalid-input = Impossibile connettersi a { $host }
client-unreachable = Impossibile raggiungere { $host }: { $why }
client-cannot-find = Il telefono non trova { $host }. Tailscale è connesso?
client-broke-off = La risposta di { $host } si è interrotta: { $why }
client-answered = { $host } ha risposto { $status } per { $path }
client-not-anibeam-status = Qualcosa ha risposto, ma non AniBeam ({ $host } ha risposto { $status } per { $path })
client-not-anibeam-hello = Qualcosa ha risposto, ma non AniBeam (la sua risposta non è l'hello di AniBeam)
client-not-anibeam-name = Qualcosa ha risposto, ma non AniBeam (si presenta come { $name })
client-too-new = L'host parla API { $host_api }; questa app legge { $app_api }. Aggiorna l'app.
client-snapshot-unreadable = L'istantanea non si è letta: { $why }
client-probe-unreadable = I dettagli del file non si sono letti: { $why }
client-peers-unreadable = L'elenco degli AniBeam non si è letto: { $why }
sync-save-failed = Impossibile salvare l'istantanea: { $why }
address-https = L'AniBeam del tuo computer risponde in http semplice tramite Tailscale; togli https://.
address-empty = Scrivi il nome o l'indirizzo dell'host, per esempio my-computer:8790.
address-ipv6-close = Un indirizzo IPv6 vuole la ] di chiusura.
address-ipv6-port = Metti la porta dopo la ], come [fd7a::1]:8790.
address-bad-host = "{ $text }" non è un nome host.
address-bad-port = "{ $port }" non è una porta.

## Series facts (companion library.rs)

clib-status-airing = In onda
clib-status-not-yet = Non ancora uscita
clib-format-tv-short = TV breve
clib-format-movie = Film
clib-format-special = Speciale
clib-format-music = Musica

## Search sites and repos

repos-by-hand-title = Aggiungine uno
repos-by-hand-note = Per un sito che nessun repo ha. Qualsiasi feed con { "{query}" } dove va la ricerca, compreso un feed Jackett o Prowlarr. Una chiave o un token nell'indirizzo viene spostato nell'archivio segreto di questo telefono.
repos-name-placeholder = Nome
repos-url-placeholder = https://...{ "{query}" }...
repos-index-placeholder = https://.../index.json
repos-add-site = Nuovo sito
repos-add-title = Aggiungi repo
repos-add-note = Incolla l'indirizzo dell'indice di un repo. AniBeam controlla la firma e ti mostra la chiave prima di salvare qualsiasi cosa.
repos-add-repo = Nuovo repo
repos-trust-title = Ti fidi del repo?
repos-fingerprint = Impronta chiave
repos-trust-warning = I suoi siti decidono dove vanno le tue ricerche. Fidati di un repo solo se viene da qualcuno di cui ti fidi.
repos-trust = Fidati
repos-open-link = Apri { $url }
repos-saving = Salvo...
repos-install = Installa
repos-update = Aggiorna
repos-edit-note = Una chiave o un token scritto nell'indirizzo viene spostato nell'archivio segreto di questo telefono.
repos-edit-note-keys = Lascia ogni { "{key}" } nell'indirizzo com'è; il suo valore resta su questo telefono.
repos-change-its = Cambia { $keys }
repos-check-repos = Cerca nuovi siti nei repo
repos-check-sites = Controlla i siti
repos-add-site-label = Aggiungi un sito a mano
repos-needs-before-search = Servono { $labels } prima di cercare
repos-no-longer-offered = Non più offerto dal suo repo
repos-update-host = L'aggiornamento invia richieste a { $host }
repos-tap-change-custom = Tocca per cambiare nome o indirizzo
repos-tap-change-keys = Tocca per cambiare { $labels }
repos-line-custom = { $host }  { $kind }  aggiunto da te
repos-update-to = Aggiorna a v{ $version }
repos-add-its-value = Aggiungi valore
repos-asks-for = Chiede { $labels }
repos-preview-line = { $count ->
    [one] { $count } sito, firmato
   *[other] { $count } siti, firmati
}
repos-preview-trusted = { $count ->
    [one] { $count } sito, firmato, già fidato
   *[other] { $count } siti, firmati, già fidati
}
repos-ask-kept-empty = Resta solo su questo telefono. Un campo vuoto mantiene quanto salvato.
repos-ask-kept-never = Resta solo su questo telefono, mai nella libreria o in un'esportazione.
repos-ask-new-host = La nuova versione lo invia a { $host }, quindi lo chiede di nuovo.
repos-ask-new = La nuova versione li chiede di nuovo.
repos-type-value = Scrivi prima un valore.
repos-health-answered = Risposta in { $ms } ms
repos-health-failed = Nessuna risposta. { $step }: { $detail }

## Downloads

dl-title = Download
dl-wifi-only = Solo Wi-Fi
dl-wifi-and-mobile = Wi-Fi e dati mobili
dl-joined = { $first }, { $second }
dl-batch-files = { $count ->
    [one] Batch, { $count } file
   *[other] Batch, { $count } file
}
dl-batch-tag = Batch
dl-downloading-count = { $count ->
    [one] { $count } in download
   *[other] { $count } in download
}
dl-free-on-phone = liberi sul telefono
dl-search-placeholder = Una serie, o il nome di una release
dl-sources = Fonti
dl-downloading = In download
dl-queue-note = { $count ->
    [one] { $count } episodio
   *[other] { $count } download
}
dl-finished = Finiti
dl-finished-note = { $count ->
    [one] { $count } download, nella tua libreria
   *[other] { $count } download, nella tua libreria
}
dl-get = Scarica
dl-pick = Scegli

## Storage and hidden shows

disk-on-phone = Sul telefono
disk-watched = Guardati
disk-not-watched = Non guardati
disk-deleting = Eliminazione
disk-delete-watched = Elimina guardati
disk-hidden-shows = Serie nascoste
disk-hidden-helper = Fuori da Home e libreria finché non le rimetti.
disk-unhide = Mostra
disk-delete-note = File e sottotitoli vengono eliminati da questo telefono per sempre.
disk-every-watched = Ogni episodio guardato
disk-every-finished = Ogni serie finita che hai guardato
disk-episodes-size = { $count ->
    [one] { $count } episodio, { $size }
   *[other] { $count } episodi, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } serie, { $size }
   *[other] { $count } serie, { $size }
}
disk-list-two = { $a } e { $b }
disk-list-three = { $a }, { $b } e { $c }
disk-list-more = { $a }, { $b }, { $c } e altre { $count }
disk-delete-finished-title = Eliminare { $count ->
    [one] { $count } serie finita
   *[other] { $count } serie finite
}, { $size }?
disk-delete-watched-title = Eliminare { $count ->
    [one] { $count } episodio guardato
   *[other] { $count } episodi guardati
}, { $size }?
disk-delete-finished-body = { $names }. Solo le serie che il provider dà per finite, con ogni episodio guardato; una serie ancora in onda, o la cui fine è una stima, resta. Ogni loro file viene eliminato da questo telefono, extra compresi. Spariscono anche i loro download, quindi si possono scaricare di nuovo.
disk-delete-watched-body = Ogni episodio che hai guardato, in ogni serie. Gli episodi non ancora guardati e gli extra restano. I file e i loro sottotitoli vengono eliminati da questo telefono per sempre.
disk-no-finished = Nessuna serie finita è guardata fino in fondo
disk-nothing-watched = Su questo telefono non c'è nulla di guardato
disk-deleted = { $count ->
    [one] { $count } episodio eliminato
   *[other] { $count } episodi eliminati
}, { $size }
disk-delete-failed = Non è stato eliminato nulla: { $error }

## Download notifications

notify-card-head = AniBeam · adesso
notify-sample-called = Frieren episodio 7
notify-sample-called-ready = Frieren episodio 6
notify-sample-facts = 420 MB di 1,4 GB, 3,2 MB/s, 5 min rimasti
notify-downloading = Download di { $called }
notify-progress = { $percent }%, { $facts }
notify-progress-of = { $percent }%, { $done } di { $size }
notify-starting = Avvio
notify-many = { $count ->
    [one] { $count } download
   *[other] { $count } download
}
notify-called-episode = { $show } episodio { $n }
notify-called-episodes = { $show } episodi da { $from } a { $to }
notify-ready = { $called } è pronto
notify-ready-many = { $called } sono pronti
notify-ready-batch = Un batch di { $show } è pronto
notify-ready-text = Scaricato su questo telefono. Tocca per aprire AniBeam.
notify-channel-downloads = Download
notify-channel-ready = Episodi pronti
notify-fallback-title = Download
notify-row-on = Notifiche
notify-row-allow = Consenti notifiche
notify-row-open-settings = Apri impostazioni notifiche
notify-helper-on = On. I download compaiono nelle notifiche mentre sono in corso.
notify-helper-asks-next = Poi Android chiede.
notify-helper-asks-next-later = Android chiede subito dopo. Puoi cambiare idea in seguito in Altro.
notify-helper-off = Off. Attiva lì le notifiche per AniBeam.
notify-question-title = Vedere i download nelle notifiche?
notify-question-body = Durante il download una notifica mostra a che punto sono gli episodi e un'altra avvisa quando uno è pronto. Si scaricano comunque; questo decide solo se lo vedi.
notify-allow = Consenti
notify-not-now = Non ora
notify-pointer = I download non compaiono nelle notifiche.
notify-turn-on = Attiva

## Feed

phone-feed-title = Feed
feed-recently-released = Usciti di recente
feed-coming-soon = In arrivo
feed-this-week = Questa settimana
feed-earlier = Prima
feed-everything-else = Tutto il resto
phone-feed-countdown-days = { $d } g { $h } h
phone-feed-countdown-hours = { $h } h { $m } min
phone-feed-countdown-minutes = { $m } min
phone-feed-next = Prossimo: { $title } tra { $countdown }
phone-feed-nothing-scheduled = Ancora nulla in programma.
phone-feed-nothing-new = Niente di nuovo
phone-feed-new-episodes = { $count ->
    [one] { $count } nuovo episodio questa settimana.
   *[other] { $count } nuovi episodi questa settimana.
}
phone-feed-aired = uscito { $ago }
phone-feed-downloaded = scaricato { $ago }

## Files and the file picker

files-already-picking = Un file è già in fase di scelta.
files-took-too-long = La scelta del file ha impiegato troppo.
files-did-not-open = Il selettore di file non si è aperto: { $error }
files-none-here = Qui non c'è un selettore di file.

## The computer's downloads

desk-sent-adding = Inviato { $what } a { $name }. Sta ancora aggiungendo.
desk-sent = Download di { $what } su { $name }.

## Your computer's library

remote-watched = Visto
remote-movie = Film
remote-not-watched-yet = Non ancora visto
remote-watched-of = { $done } di { $total } visti
remote-watched-count = { $count } visti
remote-on-disk = { $count } su disco
remote-airs-in-season = S{ $season } episodio { $n } va in onda tra { $countdown }
remote-airs-in = L'episodio { $n } va in onda tra { $countdown }
remote-play-episode = Riproduci ep. { $n }
remote-watch-again = Rivedi ep. { $n }
remote-update-host = Aggiorna AniBeam su quel computer per riprodurlo qui
remote-aired = uscito
remote-missing = Mancante
remote-on-computer = Sul tuo computer
remote-out = Uscito { $ago }
remote-season-episode = Stagione { $season }, episodio { $n }

## Episodes and dates: a season's episode count and air dates, set by hand

more-words-ratings = imdb crediti punteggi voti valutazioni
show-format-series = Serie
dates-title = Episodi e date
dates-season-line = { $title } · Stagione { $season }
dates-paste-open = Incolla lista
dates-paste-done = Fatto
dates-paste-hint = Incolla un elenco di uscite, una tabella o un post
dates-paste-note = Quello che AniBeam legge compare sugli episodi qui sotto.
dates-tap-hint = Tocca un episodio per datarlo.
dates-run = Intervallo
dates-field-hint = 7 gen, marzo 2027, 2027
dates-set = Imposta data
dates-no-date = Nessuna data
dates-weekly = Settimanale
dates-last = Ultimo
dates-not-last = Non l'ultimo
dates-clear = Rimuovi data
dates-save = Salva
dates-saving = Salvataggio
dates-cancel = Annulla
dates-change = Cambia
dates-picked-one = Episodio { $episode }:
dates-picked-many =
    { $count ->
        [one] { $count } episodio:
       *[other] { $count } episodi:
    }
dates-saved = Salvato.
dates-cleared = Note del calendario cancellate.
dates-starting = AniBeam si sta ancora avviando. Riprova tra un momento.
dates-tile-out = Uscito
dates-tile-last = Ultimo
dates-tile-none = Senza data
dates-tile-guess = ≈ { $date }
dates-summary-none = Ancora niente
dates-summary-ends = Finisce all'episodio { $episode }
dates-summary-record =
    { $count ->
        [one] { $count } datato da { $site }
       *[other] { $count } datati da { $site }
    }
dates-summary-record-unnamed =
    { $count ->
        [one] { $count } datato dal record
       *[other] { $count } datati dal record
    }
dates-summary-set =
    { $count ->
        [one] { $count } impostato da te
       *[other] { $count } impostati da te
    }
dates-summary-guessed =
    { $count ->
        [one] { $count } stimato settimanale
       *[other] { $count } stimati settimanali
    }
dates-told-one = Episodio { $episode }
dates-told-many = Episodi da { $first } a { $last }
dates-told-undated = { $episodes } senza data
dates-told-due = { $episodes } tra { $when }
dates-told-ends = Finisce all'episodio { $episode }
dates-told-season-ends = La stagione { $season } finisce all'episodio { $episode }
dates-told-by-you = { $told }, impostato da te
dates-told-gave-way =
    { $count ->
        [one] Hai impostato { $count } episodio; non vale più
       *[other] Hai impostato { $count } episodi; non valgono più
    }

## The series page

series-runtime =
    { $hours ->
        [0] { $minutes } min
       *[other] { $minutes ->
            [0] { $hours } h
           *[other] { $hours } h { $minutes } min
        }
    }
series-runtime-each = { $runtime } ciascuno
series-status-airing = In onda
series-status-finished = Conclusa
series-status-upcoming = In arrivo
series-status-cancelled = Cancellata
series-status-hiatus = In pausa
series-average-score = Voto medio { $score }
series-imdb = IMDb { $score }
series-imdb-votes = IMDb { $score } da { $votes } voti
series-rewatched =
    { $count ->
        [one] Rivista { $count } volta
       *[other] Rivista { $count } volte
    }
series-not-tracked = Non tracciata
series-not-tracked-on-phone =
    { $count ->
        [one] Non tracciata, { $count } episodio su questo telefono
       *[other] Non tracciata, { $count } episodi su questo telefono
    }

## More, About

about-ratings = Voti
about-ratings-credit = Informazioni per gentile concessione di IMDb (https://www.imdb.com). Usate con permesso.

## Marking episodes on the series page

mark-menu-watched = Segna come visto fino a qui
mark-menu-unwatched = Segna come non visto da qui
mark-menu-cancel = Annulla
mark-watched-up-to = Visto fino all'episodio { $episode }
mark-unwatched-from = Non visto dall'episodio { $episode }
mark-not-connected = { $reason } Accedi in Altro, Account.

## The player

player-speed-not-while-casting = Non durante il cast

## Downloads

downloads-other-release-on-disk = Altra release su disco

## Month patterns

dates-month-short = %b %Y
dates-month-long = %B %Y
