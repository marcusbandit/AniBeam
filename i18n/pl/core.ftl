# pl translation of en/core.ftl. An ID missing here is shown in English.

## The tailnet host

# A person on the tailnet whom Tailscale gives neither a name nor a login.
tailnet-person-no-name = Brak nazwy w Tailscale

## Counts

count-episodes =
    { $count ->
        [one] { $count } odcinek
        [few] { $count } odcinki
       *[many] { $count } odcinków
    }
count-shows =
    { $count ->
        [one] { $count } serial
        [few] { $count } seriale
       *[many] { $count } seriali
    }

## Actions every screen uses

action-cancel = Anuluj
action-close = Zamknij
action-try-again = Spróbuj ponownie
action-remove = Usuń
action-delete = Usuń
action-open = Otwórz
action-save = Zapisz
action-apply = Zastosuj
action-clear = Wyczyść
action-refresh = Odśwież
action-play = Odtwórz
action-connect = Połącz
action-start = Start
action-back = Wstecz
action-see-all = Pokaż wszystko
action-open-series = Otwórz serial
action-open-settings = Otwórz ustawienia

## The lists a tracker keeps

status-watching = Oglądam
status-completed = Ukończone
status-paused = Wstrzymane
status-on-hold = Wstrzymane
status-dropped = Porzucone
status-planning = Planuję
status-rewatching = Powtarzam

## Episodes

episode-number = Odcinek { $number }

## AniList's words for a show: its format, its airing status, how two shows relate

anilist-format-tv = TV
anilist-format-tv-short = TV krótki
anilist-format-film = Film
anilist-format-special = Specjal
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Muzyka
anilist-status-finished = Zakończony
anilist-status-airing = Emitowany
anilist-status-not-yet-aired = Jeszcze nie emitowany
anilist-status-cancelled = Anulowany
anilist-status-hiatus = Wstrzymany
anilist-relation-adaptation = Adaptacja
anilist-relation-prequel = Prequel
anilist-relation-sequel = Sequel
anilist-relation-parent = Główna historia
anilist-relation-side-story = Poboczna
anilist-relation-character = Postać
anilist-relation-summary = Skrót
anilist-relation-alternative = Alternatywa
anilist-relation-spin-off = Spin-off
anilist-relation-other = Inne
anilist-relation-source = Źródło
anilist-relation-compilation = Kompilacja
anilist-relation-contains = Zawiera

## The activity log: a problem whose place had no words of its own

event-problem-library = Coś poszło nie tak z Twoją biblioteką. Szczegóły są w opcjach programisty.
event-problem-metadata = Coś poszło nie tak z informacjami o serialu. Szczegóły są w opcjach programisty.
event-problem-trackers = Coś poszło nie tak z Twoimi listami. Szczegóły są w opcjach programisty.
event-problem-franchise = Coś poszło nie tak z powiązanymi serialami. Szczegóły są w opcjach programisty.
event-problem-playback = Coś poszło nie tak z odtwarzaniem. Szczegóły są w opcjach programisty.
event-problem-store = Coś poszło nie tak przy zapisie Twoich danych. Szczegóły są w opcjach programisty.
event-problem-system = Coś poszło nie tak z AniBeam. Szczegóły są w opcjach programisty.
event-problem-downloads = Coś poszło nie tak z pobieraniem. Szczegóły są w opcjach programisty.

## Errors (contract::CoreError)

core-error-not-found-source = Nie znaleziono folderu { $id }.
core-error-not-found-series = Nie znaleziono serialu { $id }.
core-error-not-found-file = Nie znaleziono pliku { $id }.
core-error-not-found-session = Nie znaleziono sesji odtwarzania { $id }.
core-error-not-found-job = Nie znaleziono zadania { $id }.
# $field is the setting's own name; $message is already in the reader's language.
core-error-invalid = Nieprawidłowe pole { $field }: { $message }.
# The same, for a $message that already ends in its own full stop, question mark or ellipsis.
core-error-invalid-ended = Nieprawidłowe pole { $field }: { $message }
core-error-unavailable = Ten folder jest niedostępny: { $path }
core-error-not-connected = { $tracker } nie jest połączony.
core-error-refused-hidden = Nie można tego zrobić: serial jest ukryty.
core-error-refused-no-match = Nie można tego zrobić: serial nie jest jeszcze dopasowany; dopasuj go w Informacjach o serialu.
core-error-refused-not-newer = Nie można tego zrobić: Twoja lista jest już dalej niż ten odcinek.
core-error-refused-extra = Nie można tego zrobić: to dodatek.
core-error-refused-unmatched = Nie można tego zrobić: plik nie jest dopasowany do odcinka.
core-error-refused-on-disk = Nie można tego zrobić: plik jest tylko na dysku.
core-error-provider = { $provider } nie odpowiedział: { $message }
core-error-io = Nie udało się odczytać ani zapisać pliku: { $message }
core-error-storage = Nie udało się zapisać biblioteki: { $message }
core-error-keyring = Brak dostępu do systemowego magazynu haseł: { $message }
core-error-unsupported = { $what }: jeszcze nieobsługiwane.
core-error-version = Ten eksport pochodzi z nowszego AniBeam (format { $found }). Zaktualizuj AniBeam, aby go zaimportować.
core-error-internal = Wewnętrzny błąd AniBeam: { $message }
core-error-service = { $service }: { $message }

# What the core says to a person outside downloads, releases, trackers, metadata, the
# library, franchise, images and transfer: jobs, playback, casting, the host, the phone,
# endpoint repos, updates and the rest.

## core/src/repos/mod.rs

core-repos-paste-address = wklej adres repozytorium
core-repos-must-be-https = adres musi zaczynać się od https
core-repos-unreachable = nie udało się połączyć z repozytorium: { $message }
core-repos-not-found = pod tym adresem nic nie ma (prywatne repozytorium GitHub też odpowiada 404)
core-repos-http-status = repozytorium odpowiedziało HTTP { $status }
core-repos-unreadable = nie udało się odczytać { $path }: { $error }
core-repos-no-such-endpoint = brak zainstalowanego endpointu o tym id
core-repos-key-changed = repozytorium jest teraz podpisane innym kluczem niż ten, któremu zaufałeś. Usuń je i dodaj ponownie, aby przyjąć nowy klucz.
core-repos-stale = repozytorium podało starszy indeks (numer { $serial } po { $kept }); zostaje nowszy
core-repos-key-not-shown = klucz repozytorium nie jest tym, który Ci pokazano. Dodaj je ponownie i porównaj odcisk.
core-repos-repo-added = Dodano listę źródeł pobierania.
core-repos-repo-removed = Usunięto listę źródeł pobierania.
# $key is the field's key in quotes.
core-repos-no-field = { $name } nie ma pola { $key }
core-repos-needs-field = { $name }: brakuje pola { $label }
core-repos-host-only = { $name }: { $label } to tylko host albo host:port, np. 192.0.2.10:9696
core-repos-not-added = to repozytorium nie jest dodane
core-repos-not-offered = { $repo } nie oferuje { $id }
core-repos-already-installed = { $name } jest już zainstalowany
core-repos-taken-by-you = endpoint o nazwie { $id } jest już zainstalowany (dodany przez Ciebie); najpierw go usuń
core-repos-taken-by-repo = endpoint o nazwie { $id } jest już zainstalowany (z innego repozytorium); najpierw go usuń
core-repos-source-added = Dodano źródło pobierania.
core-repos-source-updated = Zaktualizowano źródło pobierania.
core-repos-source-removed = Usunięto źródło pobierania.
core-repos-source-changed = Zmieniono źródło pobierania.
core-repos-give-name = nadaj mu nazwę
core-repos-put-query = wstaw {"{"}query{"}"} w adresie tam, gdzie ma trafić wyszukiwanie
core-repos-name-taken = endpoint o nazwie { $id } jest już zainstalowany
core-repos-only-by-hand = edytować można tylko źródło dodane ręcznie
# $key is a placeholder's name; it stays in braces in the address.
core-repos-keep-key = zostaw {"{"}{ $key }{"}"} w adresie i zmień jego wartość przez Zmień { $label }, albo usuń witrynę i dodaj ją ponownie bez niego
core-repos-note-kind = anirss nie umie przeszukiwać rodzaju { $kind }, więc komputer go pomija; telefon go przeszukuje
core-repos-note-by-hand = anirss ma własny { $id }, napisany ręcznie; komputer przeszukuje tamten
core-repos-note-theirs = Ten anirss uruchamiasz sam, a komputer nigdy nie zmienia jego konfiguracji: dodaj { $id } do konfiguracji tego anirss, aby komputer go przeszukiwał
core-repos-note-sandbox = To uruchomienie nigdy nie zapisuje do konfiguracji anirss, więc komputer nie przeszukuje tu { $id }
core-repos-note-old = anirss { $version } nie przyjmuje endpointów z { -app-name }; zaktualizuj go do { $min } lub nowszego, aby komputer to przeszukiwał

## core/src/repos/index.rs

core-repos-key-not-ed25519 = klucz indeksu nie jest kluczem ed25519
core-repos-key-not-base64 = klucz indeksu to nie 32 bajty w base64
core-repos-key-invalid = klucz indeksu nie jest prawidłowym kluczem ed25519
core-repos-too-large = plik jest za duży, by był repozytorium
core-repos-not-a-repo = to nie jest repozytorium endpointów { -app-name }
core-repos-format-version = repozytorium ma format w wersji { $found }; ten { -app-name } czyta wersję { $reads }. Zaktualizuj { -app-name }.
core-repos-payload-not-base64 = zawartość repozytorium nie jest w base64
core-repos-signature-malformed = podpis repozytorium jest uszkodzony
core-repos-index-unreadable = nie da się odczytać indeksu repozytorium: { $error }
core-repos-signature-mismatch = podpis repozytorium nie pasuje do jego zawartości
# $id is the source's id; the rest of these checks name a source by its id first.
core-repos-check-scheme = { $id }: adres musi zaczynać się od https://
core-repos-check-scheme-http = { $id }: adres musi zaczynać się od http:// lub https://
# $key is the field's key in quotes.
core-repos-check-auth-key = { $id }: zły klucz uwierzytelniania { $key }
core-repos-check-host-torznab = { $id }: tylko adres torznab może podać host jako {"{"}{ $key }{"}"}
core-repos-check-host-placeholder = { $id }: symbol zastępczy w hoście

## core/src/playback/tracks.rs

core-playback-track-number = Ścieżka { $number }
# A sidecar subtitle file beside the video; $label is the rest of its name.
core-playback-track-file = { $label } (plik)
# A language and its region or script: "Portuguese (Brazil)", "Chinese (Simplified)".
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = Angielski
core-playback-lang-japanese = Japoński
core-playback-lang-spanish = Hiszpański
core-playback-lang-portuguese = Portugalski
core-playback-lang-french = Francuski
core-playback-lang-german = Niemiecki
core-playback-lang-italian = Włoski
core-playback-lang-russian = Rosyjski
core-playback-lang-chinese = Chiński
core-playback-lang-korean = Koreański
core-playback-lang-arabic = Arabski
core-playback-lang-hindi = Hindi
core-playback-lang-indonesian = Indonezyjski
core-playback-lang-malay = Malajski
core-playback-lang-thai = Tajski
core-playback-lang-vietnamese = Wietnamski
core-playback-lang-polish = Polski
core-playback-lang-turkish = Turecki
core-playback-lang-dutch = Niderlandzki
core-playback-lang-swedish = Szwedzki
core-playback-lang-norwegian = Norweski
core-playback-lang-danish = Duński
core-playback-lang-finnish = Fiński
core-playback-lang-hebrew = Hebrajski
core-playback-lang-ukrainian = Ukraiński
core-playback-lang-czech = Czeski
core-playback-lang-hungarian = Węgierski
core-playback-lang-greek = Grecki
core-playback-lang-romanian = Rumuński
core-playback-lang-filipino = Filipiński
core-playback-region-brazil = Brazylia
core-playback-region-portugal = Portugalia
core-playback-region-latin-america = Ameryka Łacińska
core-playback-region-spain = Hiszpania
core-playback-region-simplified = uproszczony
core-playback-region-traditional = tradycyjny

## core/src/contract/enums.rs

# What a job is doing, for a status line while it runs. .stopped is the line when it
# ended in a problem.
core-jobs-scan = Sprawdzanie folderów
    .stopped = Sprawdzanie folderów: wystąpił problem.
core-jobs-auto-match = Dopasowywanie seriali
    .stopped = Dopasowywanie seriali: wystąpił problem.
core-jobs-search = Szukanie serialu
    .stopped = Szukanie serialu: wystąpił problem.
core-jobs-resolve-link = Odczytywanie linku
    .stopped = Odczytywanie linku: wystąpił problem.
core-jobs-apply-match = Dopasowywanie serialu
    .stopped = Dopasowywanie serialu: wystąpił problem.
core-jobs-refresh = Aktualizowanie serialu
    .stopped = Aktualizowanie serialu: wystąpił problem.
core-jobs-refresh-all = Aktualizowanie informacji o serialach
    .stopped = Aktualizowanie informacji o serialach: wystąpił problem.
core-jobs-refresh-airing = Sprawdzanie, co jest emitowane
    .stopped = Sprawdzanie, co jest emitowane: wystąpił problem.
core-jobs-refresh-imdb = Aktualizowanie ocen
    .stopped = Aktualizowanie ocen: wystąpił problem.
core-jobs-clear-images = Czyszczenie obrazów
    .stopped = Czyszczenie obrazów: wystąpił problem.
core-jobs-fill-images = Pobieranie obrazów
    .stopped = Pobieranie obrazów: wystąpił problem.
core-jobs-connect-tracker = Łączenie listy
    .stopped = Łączenie listy: wystąpił problem.
core-jobs-mark = Oznaczanie odcinka jako obejrzanego
    .stopped = Oznaczanie odcinka jako obejrzanego: wystąpił problem.
core-jobs-set-progress = Ustawianie postępu
    .stopped = Ustawianie postępu: wystąpił problem.
core-jobs-score = Zapisywanie oceny
    .stopped = Zapisywanie oceny: wystąpił problem.
core-jobs-refresh-progress = Odczytywanie postępu z listy
    .stopped = Odczytywanie postępu z listy: wystąpił problem.
core-jobs-refresh-watching = Odczytywanie listy Oglądam
    .stopped = Odczytywanie listy Oglądam: wystąpił problem.
core-jobs-refresh-airing-now = Sprawdzanie tego sezonu
    .stopped = Sprawdzanie tego sezonu: wystąpił problem.
core-jobs-refresh-recommended = Szukanie seriali dla Ciebie
    .stopped = Szukanie seriali dla Ciebie: wystąpił problem.
core-jobs-read-show-page = Odczytywanie serialu
    .stopped = Odczytywanie serialu: wystąpił problem.
core-jobs-on-hold = Przenoszenie nieruszanych seriali do Wstrzymanych
    .stopped = Przenoszenie nieruszanych seriali do Wstrzymanych: wystąpił problem.
core-jobs-set-status = Zmiana statusu na liście
    .stopped = Zmiana statusu na liście: wystąpił problem.
core-jobs-move-lists = Przenoszenie listy do serwisu
    .stopped = Przenoszenie listy do serwisu: wystąpił problem.
core-jobs-notes = Zapisywanie notatek
    .stopped = Zapisywanie notatek: wystąpił problem.
core-jobs-crawl = Szukanie powiązanych seriali
    .stopped = Szukanie powiązanych seriali: wystąpił problem.
core-jobs-skip-windows = Szukanie openingu i endingu
    .stopped = Szukanie openingu i endingu: wystąpił problem.
core-jobs-export = Zapisywanie biblioteki do pliku
    .stopped = Zapisywanie biblioteki do pliku: wystąpił problem.
core-jobs-import = Wczytywanie biblioteki z pliku
    .stopped = Wczytywanie biblioteki z pliku: wystąpił problem.
core-jobs-subscriptions = Sprawdzanie subskrypcji
    .stopped = Sprawdzanie subskrypcji: wystąpił problem.
core-jobs-connect-downloads = Łączenie z programem do pobierania
    .stopped = Łączenie z programem do pobierania: wystąpił problem.
core-jobs-search-releases = Szukanie odcinków
    .stopped = Szukanie odcinków: wystąpił problem.
core-jobs-refine-releases = Zawężanie odcinków
    .stopped = Zawężanie odcinków: wystąpił problem.
core-jobs-subscribe = Subskrybowanie serialu
    .stopped = Subskrybowanie serialu: wystąpił problem.
core-jobs-download = Dodawanie pobrań
    .stopped = Dodawanie pobrań: wystąpił problem.
core-jobs-remove-subscription = Usuwanie subskrypcji
    .stopped = Usuwanie subskrypcji: wystąpił problem.
core-jobs-control-torrents = Zmienianie pobrań
    .stopped = Zmienianie pobrań: wystąpił problem.
core-jobs-test-torrent-client = Testowanie programu do pobierania
    .stopped = Testowanie programu do pobierania: wystąpił problem.
core-jobs-remove-show = Usuwanie serialu
    .stopped = Usuwanie serialu: wystąpił problem.
core-jobs-repair-subscription = Naprawianie subskrypcji
    .stopped = Naprawianie subskrypcji: wystąpił problem.
core-jobs-check-endpoints = Sprawdzanie źródeł pobierania
    .stopped = Sprawdzanie źródeł pobierania: wystąpił problem.
core-jobs-start-endpoint = Uruchamianie źródła pobierania
    .stopped = Uruchamianie źródła pobierania: wystąpił problem.
core-jobs-schedule-agent = Pytanie agenta kodującego o harmonogram
    .stopped = Pytanie agenta kodującego o harmonogram: wystąpił problem.

## core/src/airing_now.rs

core-airing-now-now = Teraz w emisji
core-airing-now-winter = Zima { $year }
core-airing-now-spring = Wiosna { $year }
core-airing-now-summer = Lato { $year }
core-airing-now-fall = Jesień { $year }
core-airing-now-planned = Dodano { $title } do listy Planuję w { $site }.
core-airing-now-planned-both = Dodano { $title } do listy Planuję w { $site } i { $other }.
# $status is the list the show is on: watching, repeating, planning, completed, paused or dropped.
core-airing-now-already-on =
    { $title } jest już na Twojej liście w { $site } jako { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }.
core-airing-now-plan-failed = Nie udało się dodać { $title } do listy Planuję. Spróbuj później.

## core/src/aired.rs

core-aired-episode-out = Wyszedł odcinek { $number } serialu { $title }.

## core/src/cast/mod.rs

core-cast-connecting = Przesyłanie na: { $device }.
core-cast-ended = Zakończono przesyłanie na: { $device }.
core-cast-stopped = Zatrzymano przesyłanie na: { $device }.
core-cast-failed = Przesyłanie na { $device } nie powiodło się. Sprawdź, czy urządzenie jest w tej samej sieci.
core-cast-not-built = Przesyłanie nie jest częścią tej wersji
core-cast-serve-failed = nie udało się udostępnić odcinka: { $error }
core-cast-device-gone = Tego urządzenia nie ma już w sieci
core-cast-not-a-playlist = to nie jest adres playlisty
core-cast-nothing = Nic nie jest przesyłane
core-cast-still-starting = Urządzenie jeszcze się uruchamia
core-cast-no-route = to urządzenie nie ma trasy do telewizora
core-cast-no-control = telewizor nie podał adresu sterowania
core-cast-not-handed = drugi komputer nie przekazał odcinka: { $error }
core-cast-empty-playlist = drugi komputer wysłał pustą playlistę
# $error is why the TV refused the converted stream, $first why it refused the file.
core-cast-refused-both = { $error }; odrzucił też plik w oryginale ({ $first })
core-cast-chromecast-stopped = Chromecast się zatrzymał
core-cast-hevc = { $error }. Ten odcinek jest w HEVC, który dekodują tylko nowsze Chromecasty (z Google TV albo Ultra)

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = nie udało się połączyć z Chromecastem: { $error }
core-cast-chromecast-broke = połączenie z Chromecastem zostało przerwane: { $error }
core-cast-chromecast-no-player = Chromecast nie uruchomił odtwarzacza
# $reason is the Chromecast's own word for it, or core-cast-no-reason.
core-cast-chromecast-refused-player = Chromecast nie chciał uruchomić odtwarzacza ({ $reason })
core-cast-no-reason = bez powodu
# $kind is the Chromecast's own word for the failure.
core-cast-chromecast-cannot-play = Chromecast nie mógł tego odtworzyć ({ $kind })
core-cast-chromecast-error = Chromecast zatrzymał się z błędem podczas odtwarzania

## core/src/cast/dlna.rs

core-cast-tv-no-answer = telewizor nie odpowiedział: { $error }
# $action is the DLNA command; $code the TV's own error.
core-cast-tv-refused = telewizor odrzucił { $action }
core-cast-tv-refused-code = telewizor odrzucił { $action } ({ $code })

## core/src/cast/media.rs

core-cast-no-video = Ten plik nie ma obrazu.
core-cast-hi10p = Ten odcinek jest w 10-bitowym H.264 (Hi10P), którego nie dekoduje żaden telewizor ani Chromecast. Przesłanie go wymaga transkodowania, którego jeszcze nie ma.
core-cast-video-codec = Obrazu tego odcinka ({ $codec }) nie da się przesłać.
core-cast-audio-codec = Dźwięku tego odcinka ({ $codec }) nie da się przesłać.

## core/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Zaczęto pobierać odcinek z { $folder }.
        [few] Zaczęto pobierać { $count } odcinki z { $folder }.
        [many] Zaczęto pobierać { $count } odcinków z { $folder }.
       *[other] Zaczęto pobierać { $count } odcinka z { $folder }.
    }
core-phone-subscribed = Zasubskrybowano { $name }.
core-phone-rule-stopped = { $name } nie jest sprawdzany pod kątem nowych odcinków: jego źródło zniknęło albo znowu potrzebuje klucza.
core-phone-new-episode = Pobiera się nowy odcinek { $name }.
# $name is the source's name in quotes.
core-phone-name-taken = { $name }: inne źródło ma już tę nazwę
core-phone-nothing-to-download = nic do pobrania
# $used and $cap are in GB, with one decimal.
core-phone-over-cap = to przekroczyłoby limit pamięci { -app-name } ({ $used } z { $cap } GB zajęte lub w drodze); usuń odcinki albo podnieś limit
core-phone-empty-search = wyszukiwanie jest puste
core-phone-no-results = brak wyników dla { $query }
# Ends "... is not supported yet." (core-error-unsupported).
core-phone-own-engine = własny silnik torrentów

## core/src/phone/check.rs

# The connection check: each line is a step for one host ($label), and what came of it.
core-phone-check-connected = połączono
core-phone-check-refused = odrzucono: { $error }
core-phone-check-dropped = brak odpowiedzi (odrzucone)
core-phone-check-finding = { $label }: szukanie adresu
core-phone-check-failed = błąd: { $error }
core-phone-check-no-answer = brak odpowiedzi
core-phone-check-private = { $label }: szukanie prywatne
core-phone-check-skipped-address = pominięto, to adres
core-phone-check-skipped-home = pominięto, nazwa w sieci domowej
core-phone-check-no-record = brak publicznego adresu, zostawione DNS tej sieci
core-phone-check-no-resolver = żaden serwer DNS nie odpowiedział
core-phone-check-reaching = { $label }: łączenie z { $ip }
core-phone-check-http = HTTP { $status }, bajty: { $bytes }
core-phone-check-setup = konfiguracja

## core/src/playback/session.rs

core-playback-not-saved = Postęp w { $folder } nie został zapisany. Najpierw dopasuj serial.
# $done is a whole percentage.
core-playback-still-downloading = odcinek { $number } wciąż się pobiera, gotowe { $done }%

## core/src/playback/skip.rs

core-playback-bad-duration = czas trwania to dodatnia liczba sekund

## core/src/agent.rs

# A coding agent looking up a show's schedule; these lines say what it is doing.
core-agent-no-answer = agent nie dał odpowiedzi
core-agent-searching = Szukanie w sieci: { $query }
core-agent-searching-schedule = Szukanie harmonogramu w sieci
core-agent-reading = Czytanie { $url }
core-agent-reading-page = Czytanie strony
# $tool is the agent's own name for a tool.
core-agent-using = Używa: { $tool }
# Ends "... is not supported yet." (core-error-unsupported).
core-agent-none = agent kodujący na tym komputerze
# $name is the coding agent's name.
core-agent-ended = { $name } zakończył bez odpowiedzi
core-agent-too-long = { $name } działał za długo

## core/src/core.rs

# A guest device on another computer's library asks for something that is the owner's.
core-core-guest-trackers = konta serwisów na tamtym komputerze należą do jego właściciela, więc to urządzenie niczego tam nie oznacza
core-core-guest-not-interested = „Nie interesuje” na tamtym komputerze to lista jego właściciela, więc to urządzenie niczego tam nie zmienia
core-core-guest-recommended = polecane na tamtym komputerze wynikają z listy jego właściciela
core-core-nothing-picked = nic nie wybrano

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider } nie jest dostawcą dopasowań
core-prefs-provider-count = { $provider } występuje { $count } razy; każdy dostawca występuje raz
# A setting out of its bounds: $value is outside $low to $high.
core-prefs-outside = { $value } jest poza zakresem od { $low } do { $high }
core-prefs-empty = puste

## core/src/recommended/mod.rs

core-recommended-ranking = Układanie seriali dla Ciebie

## core/src/recommended/sync.rs

core-recommended-finding-people = Szukanie osób o podobnym guście
core-recommended-reading-lists = Czytanie ich list
core-recommended-reading-yours = Czytanie Twoich seriali
core-recommended-reading-suggestions = Czytanie seriali do polecenia

## core/src/recommended/page.rs

core-recommended-no-anime = { -anilist } nie ma anime { $id }

## core/src/host/mod.rs

# The tailnet host's status line in Settings.
core-host-off = Wył.
core-host-starting = Uruchamianie
# $addresses is a list of address:port, joined with commas.
core-host-serving = Udostępnia na { $addresses }
core-host-no-tailscale = Tailscale nie jest połączony
core-host-cannot-listen = Nie można nasłuchiwać
core-host-port-taken = Port { $port } jest zajęty
core-host-cannot-listen-on = Nie można nasłuchiwać na { $address }: { $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text } to nie adres z portem

## core/src/host/keys.rs

# $key is the text given, in quotes.
core-host-not-a-key = { $key } to nie klucz urządzenia

## core/src/net/limiter.rs

core-net-timed-out = przekroczono czas po { $ms } ms
core-net-rate-limited = { $provider } ogranicza liczbę zapytań

## core/src/net/mod.rs

core-net-waiting = { $host }: przed chwilą przekroczono czas; { -app-name } czeka { $seconds } s, zanim zapyta ponownie

## core/src/net/anilist.rs

core-net-no-data = brak danych

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } nie ma pliku do pobrania dla tego komputera
core-update-cannot-make = Nie udało się utworzyć { $path }: { $error }
core-update-cannot-mark = Nie udało się oznaczyć nowego AppImage jako wykonywalnego: { $error }
core-update-cannot-replace = Nie udało się zastąpić { $path }: { $error }
core-update-not-here = Ta kopia nie jest aktualizowana tutaj
core-update-cannot-start = Nie udało się rozpocząć pobierania: { $error }
core-update-did-not-start = Pobieranie się nie rozpoczęło: { $error }
core-update-cannot-write = Nie udało się zapisać { $path }: { $error }
core-update-stopped = Pobieranie zatrzymało się: { $error }
core-update-cancelled = Anulowano
# $got and $size are in bytes.
core-update-ended-early = Pobieranie skończyło się za wcześnie ({ $got } z { $size } bajtów)
core-update-checksum = Pobrany plik nie zgadza się z sumą kontrolną z GitHub
core-update-cannot-move = Nie udało się przenieść pobranego pliku na miejsce: { $error }
core-update-cannot-open-dmg = Nie udało się otworzyć dmg: { $error }
core-update-cannot-read-dmg = Nie udało się odczytać dmg: { $error }
core-update-no-app = W dmg nie ma aplikacji
core-update-cannot-copy = Nie udało się skopiować nowej aplikacji obok tej: { $error }

## core/src/calendar.rs

# An episode's event in the calendar file.
core-calendar-season-episode = { $title } - Sezon { $season } Odcinek { $number }
core-calendar-episode = { $title } - Odcinek { $number }

## core/src/paths.rs

core-paths-no-home = brak katalogu domowego

## core/src/first_run.rs

core-first-run-not-absolute = folder z filmami musi być ścieżką bezwzględną

## core/src/presence/mod.rs

# Discord shows this in place of an episode line while an extra plays.
core-presence-extra = Dodatek

# The core's words for the library group: trackers, metadata, library, franchise,
# images and transfer (#798).

## core/src/trackers/accounts.rs

# $tracker is the tracker's short name: AniList or MAL.
core-trackers-no-client-id = Brak ustawionego Client ID dla { $tracker }.
core-trackers-mal-reconnect = Połącz ponownie MyAnimeList w Ustawieniach.
core-trackers-mal-wants-secret = MyAnimeList wymaga client secret tej aplikacji. Dodaj go w Ustawieniach, w sekcji Serwisy, albo zarejestruj aplikację z App Type "other".
# $error is the parser's own message.
core-trackers-token-reply-unreadable = nieczytelna odpowiedź z tokenem: { $error }
core-trackers-token-reply-no-token = odpowiedź nie zawierała tokenu dostępu

## core/src/trackers/mod.rs

core-trackers-no-token = brak zapisanego tokenu dostępu, połącz ponownie w Ustawieniach

## core/src/trackers/cache.rs

core-trackers-no-user-id = konto nie ma id użytkownika

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = nie wybrano seriali do przeniesienia
# $site is the tracker's full name: AniList or MyAnimeList.
core-trackers-moved-up =
    { $count ->
        [one] Przeniesiono { $count } serial z tego komputera do { $site }.
        [few] Przeniesiono { $count } seriale z tego komputera do { $site }.
        [many] Przeniesiono { $count } seriali z tego komputera do { $site }.
       *[other] Przeniesiono { $count } seriali z tego komputera do { $site }.
    }
core-trackers-moved-up-some-refused =
    Przeniesiono { $moved ->
        [one] { $moved } serial
        [few] { $moved } seriale
        [many] { $moved } seriali
       *[other] { $moved } seriali
    } do { $site }; { $site } nie przyjął: { $failed ->
        [one] { $failed } serial
        [few] { $failed } seriale
        [many] { $failed } seriali
       *[other] { $failed } seriali
    }.
# $show is the show's folder name; $error is the tracker's answer, cleaned up.
core-trackers-move-refused = { $site } nie przyjął { $show }: { $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = Upłynął czas autoryzacji, spróbuj ponownie.
core-trackers-oauth-state-mismatch = niezgodny parametr state, możliwy atak CSRF, przerwano.
core-trackers-oauth-no-code = brak kodu w odpowiedzi
core-trackers-oauth-no-token = brak access_token w odpowiedzi
core-trackers-oauth-waiting = Czekam, aż zalogujesz się do { $site } w przeglądarce.
# $username is the account's name on the tracker.
core-trackers-oauth-connected = Połączono { $site } jako { $username }.
core-trackers-oauth-no-name = profil wrócił bez nazwy
# Shown before a colon and the reason.
core-trackers-oauth-could-not-start = Nie udało się rozpocząć logowania
# $error is the system's own message.
core-trackers-oauth-port-failed = nie udało się otworzyć portu { $port } na tym komputerze: { $error }
core-trackers-oauth-port-busy = port { $port } na tym komputerze jest zajęty przez inny program. Poczekaj chwilę i spróbuj ponownie albo zamknij ten program, jeśli port wciąż jest zajęty.
# $error is the code the tracker sent back.
core-trackers-oauth-provider-error = { $site } zwrócił błąd: { $error }
# The browser tab the sign-in leaves. $app is AniBeam's name, in the brand colour in the
# page's body; keep it as a variable.
core-trackers-page-connected = { $site } jest połączony z { $app }
core-trackers-page-close-tab = Możesz zamknąć tę kartę.
core-trackers-page-failed-title = { $app }, logowanie nie powiodło się
core-trackers-page-failed = Logowanie się nie zakończyło
core-trackers-page-go-back = Wróć do { $app } i spróbuj ponownie.
core-trackers-page-finishing-title = { $app }, jeszcze kończy
core-trackers-page-finishing = Jeszcze kończy
core-trackers-page-finishing-body = { $site } odpowiedział, a AniBeam kończy logowanie. Wróć do { $app }, który powie, kiedy skończy.
core-trackers-page-cancelled-title = { $app }, logowanie anulowane
core-trackers-page-cancelled = To logowanie zostało anulowane
core-trackers-page-cancelled-body = Możesz zamknąć tę kartę. Zacznij od nowa w { $app }, jeśli chcesz.

## core/src/trackers/on_hold.rs

# A progress label while the pass runs; $tracker is AniList or MAL.
core-trackers-on-hold-moving = Przenoszenie do Wstrzymanych w { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] Przeniesiono { $count } serial do Wstrzymanych w { $site }.
        [few] Przeniesiono { $count } seriale do Wstrzymanych w { $site }.
        [many] Przeniesiono { $count } seriali do Wstrzymanych w { $site }.
       *[other] Przeniesiono { $count } seriali do Wstrzymanych w { $site }.
    }
# $second is how many moved on the second tracker, $secondSite.
core-trackers-on-hold-moved-two =
    { $count ->
        [one] Przeniesiono { $count } serial do Wstrzymanych w { $site } i { $second } w { $secondSite }.
        [few] Przeniesiono { $count } seriale do Wstrzymanych w { $site } i { $second } w { $secondSite }.
        [many] Przeniesiono { $count } seriali do Wstrzymanych w { $site } i { $second } w { $secondSite }.
       *[other] Przeniesiono { $count } seriali do Wstrzymanych w { $site } i { $second } w { $secondSite }.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } serialu nie udało się przenieść do Wstrzymanych.
        [few] { $failed } seriali nie udało się przenieść do Wstrzymanych.
        [many] { $failed } seriali nie udało się przenieść do Wstrzymanych.
       *[other] { $failed } seriali nie udało się przenieść do Wstrzymanych.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] Przeniesiono { $count } serial do Wstrzymanych w { $site }.
        [few] Przeniesiono { $count } seriale do Wstrzymanych w { $site }.
        [many] Przeniesiono { $count } seriali do Wstrzymanych w { $site }.
       *[other] Przeniesiono { $count } seriali do Wstrzymanych w { $site }.
    } { $failed ->
        [one] { $failed } serialu nie udało się przenieść do Wstrzymanych.
        [few] { $failed } seriali nie udało się przenieść do Wstrzymanych.
        [many] { $failed } seriali nie udało się przenieść do Wstrzymanych.
       *[other] { $failed } seriali nie udało się przenieść do Wstrzymanych.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] Przeniesiono { $count } serial do Wstrzymanych w { $site } i { $second } w { $secondSite }.
        [few] Przeniesiono { $count } seriale do Wstrzymanych w { $site } i { $second } w { $secondSite }.
        [many] Przeniesiono { $count } seriali do Wstrzymanych w { $site } i { $second } w { $secondSite }.
       *[other] Przeniesiono { $count } seriali do Wstrzymanych w { $site } i { $second } w { $secondSite }.
    } { $failed ->
        [one] { $failed } serialu nie udało się przenieść do Wstrzymanych.
        [few] { $failed } seriali nie udało się przenieść do Wstrzymanych.
        [many] { $failed } seriali nie udało się przenieść do Wstrzymanych.
       *[other] { $failed } seriali nie udało się przenieść do Wstrzymanych.
    }
# A show's name when its row is gone.
core-trackers-series-fallback = serial { $id }
# $name is the show's folder name.
core-trackers-back-on-watching = { $name } wrócił do Oglądam w { $site }.
core-trackers-back-on-watching-two = { $name } wrócił do Oglądam w { $site } i { $secondSite }.
core-trackers-back-on-watching-failed = Nie udało się przywrócić { $name } do Oglądam. Spróbuj później.

## core/src/trackers/writes.rs

# A tracker's failure in words a person can act on. $tracker is AniList or MAL.
core-trackers-rate-limited = { $tracker } ogranicza liczbę zapytań, spróbuj za minutę.
core-trackers-auth-expired = Logowanie do { $tracker } wygasło, połącz ponownie w Ustawieniach.
core-trackers-entry-not-found = Nie znaleziono wpisu w { $tracker }.
# $status is the HTTP status code.
core-trackers-server-error = Błąd serwera { $tracker } ({ $status }), spróbuj później.
core-trackers-error-see-log = Błąd { $tracker }, zobacz dziennik aktywności.
# $episode is the number asked for, as it was typed.
core-trackers-episode-not-whole = { $episode } to nie jest cały numer odcinka większy od zera
core-trackers-score-out-of-range = { $score } jest poza zakresem od 0 do 10
core-trackers-rewatch-not-settable = powtórka to nie status do ustawienia ręcznie
core-trackers-notes-too-long = notatki mogą mieć najwyżej { $max } znaków
core-trackers-nothing-picked = nic nie wybrano
core-trackers-notes-not-on-list = Jeszcze nie ma na Twojej liście w { $tracker }, najpierw dodaj do listy.
# Where a write landed, picked by $sites: anilist, mal or both. Used inside the lines below.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } i { -mal }
        [anilist] { -anilist }
       *[mal] { -mal }
    }
# A list status as the status pill names it, picked by $status.
core-trackers-status =
    { $status ->
        [watching] { status-watching }
        [planning] { status-planning }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[repeating] Powtarzam
    }
# $show is the show's folder name.
core-trackers-marked = Oznaczono odcinek { $episode } serialu { $show } w { core-trackers-sites }.
core-trackers-progress-set = Ustawiono postęp { $show } na { $progress } w { core-trackers-sites }.
core-trackers-scored = Oceniono { $show } na { $score } w { core-trackers-sites }.
core-trackers-score-cleared = Usunięto Twoją ocenę { $show } w { core-trackers-sites }.
core-trackers-status-set = Ustawiono { $show } jako { core-trackers-status } w { core-trackers-sites }.
core-trackers-notes-saved = Zapisano Twoje notatki do { $show } w { core-trackers-sites }.
core-trackers-notes-cleared = Usunięto Twoje notatki do { $show } w { core-trackers-sites }.
# $site is AniList or MyAnimeList.
core-trackers-update-failed = Nie udało się zaktualizować { $site }. Spróbuj później.
# One write to many shows: what landed, what failed and what was left out, in one line.
# $done shows were written and $failed were not; $named is yes when the one show
# written goes by its $name. $action is score, clear or status; $hasReason is yes when
# $reason holds the first tracker's answer.
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
            [one] { $done } serial
            [few] { $done } seriale
            [many] { $done } seriali
           *[other] { $done } seriali
        }
    }
core-trackers-written-done =
    { $action ->
        [clear] Usunięto Twoją ocenę: { core-trackers-written-who }, w { core-trackers-sites }.
        [status] Ustawiono { core-trackers-written-who } jako { core-trackers-status } w { core-trackers-sites }.
       *[score] Oceniono { core-trackers-written-who } na { $score } w { core-trackers-sites }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] { $failed } serialu nie udało się zaktualizować: { $reason }.
            [few] { $failed } seriali nie udało się zaktualizować: { $reason }.
            [many] { $failed } seriali nie udało się zaktualizować: { $reason }.
           *[other] { $failed } seriali nie udało się zaktualizować: { $reason }.
        }
       *[no] { $failed ->
            [one] { $failed } serialu nie udało się zaktualizować.
            [few] { $failed } seriali nie udało się zaktualizować.
            [many] { $failed } seriali nie udało się zaktualizować.
           *[other] { $failed } seriali nie udało się zaktualizować.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] Nie udało się zaktualizować: { $failed }, { $reason }.
       *[no] Nie udało się zaktualizować: { $failed }.
    }
# The shows a write to many left out, one sentence per reason, each after a space.
# Keep each [0] first: it leaves the sentence out.
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } serial nie ma dopasowania w serwisie.
        [few] {" "}{ $noMatch } seriale nie mają dopasowania w serwisie.
        [many] {" "}{ $noMatch } seriali nie ma dopasowania w serwisie.
       *[other] {" "}{ $noMatch } seriali nie ma dopasowania w serwisie.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } serial jest ukryty, więc jego synchronizacja jest wyłączona.
        [few] {" "}{ $hidden } seriale są ukryte, więc ich synchronizacja jest wyłączona.
        [many] {" "}{ $hidden } seriali jest ukrytych, więc ich synchronizacja jest wyłączona.
       *[other] {" "}{ $hidden } seriali jest ukrytych, więc ich synchronizacja jest wyłączona.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } serial jest tylko w serwisie, który nie jest połączony.
        [few] {" "}{ $notConnected } seriale są tylko w serwisie, który nie jest połączony.
        [many] {" "}{ $notConnected } seriali jest tylko w serwisie, który nie jest połączony.
       *[other] {" "}{ $notConnected } seriali jest tylko w serwisie, który nie jest połączony.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } serial zostaje na tym komputerze, bez serwisu, do którego można wysłać.
        [few] {" "}{ $kept } seriale zostają na tym komputerze, bez serwisu, do którego można wysłać.
        [many] {" "}{ $kept } seriali zostaje na tym komputerze, bez serwisu, do którego można wysłać.
       *[other] {" "}{ $kept } seriali zostaje na tym komputerze, bez serwisu, do którego można wysłać.
    }

## core/src/metadata/automatch.rs

# $show is the show's folder name; $title is the title AniList or TVmaze gives it.
core-metadata-matched-to = Dopasowano { $show } do { $title }.
core-metadata-auto-matched =
    { $count ->
        [one] Dopasowano { $count } serial.
        [few] Dopasowano { $count } seriale.
        [many] Dopasowano { $count } seriali.
       *[other] Dopasowano { $count } seriali.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } serial wciąż czeka na dopasowanie.
        [few] { $count } seriale wciąż czekają na dopasowanie.
        [many] { $count } seriali wciąż czeka na dopasowanie.
       *[other] { $count } seriali wciąż czeka na dopasowanie.
    }
core-metadata-auto-matched-some =
    Dopasowano { $matched ->
        [one] { $matched } serial
        [few] { $matched } seriale
        [many] { $matched } seriali
       *[other] { $matched } seriali
    }; { $unmatched } wciąż { $unmatched ->
        [one] czeka
        [few] czekają
        [many] czeka
       *[other] czeka
    } na dopasowanie.
# Find a match's one line: the sentences below that have something to say, in order,
# each after the first set off by a space. $first names the sentence that leads.
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
# $hitNamed is yes when the one show matched goes by its folder name, $hitFolder, and
# the title it matched, $hitTitle.
core-metadata-find-matched =
    { $hitNamed ->
        [yes] Dopasowano { $hitFolder } do { $hitTitle }.
       *[no] { $matched ->
            [one] Dopasowano { $matched } serial.
            [few] Dopasowano { $matched } seriale.
            [many] Dopasowano { $matched } seriali.
           *[other] Dopasowano { $matched } seriali.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] Nie znaleziono dopasowania dla { $missName }.
       *[no] { $unmatched ->
            [one] Nie znaleziono dopasowania dla { $unmatched } serialu.
            [few] Nie znaleziono dopasowania dla { $unmatched } seriali.
            [many] Nie znaleziono dopasowania dla { $unmatched } seriali.
           *[other] Nie znaleziono dopasowania dla { $unmatched } seriali.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] { $failed } serialu nie udało się wyszukać; spróbuj później.
        [few] { $failed } seriali nie udało się wyszukać; spróbuj później.
        [many] { $failed } seriali nie udało się wyszukać; spróbuj później.
       *[other] { $failed } seriali nie udało się wyszukać; spróbuj później.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } serial miał już dopasowanie.
        [few] { $had } seriale miały już dopasowanie.
        [many] { $had } seriali miało już dopasowanie.
       *[other] { $had } seriali miało już dopasowanie.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } serial jest już dopasowywany.
        [few] { $busy } seriale są już dopasowywane.
        [many] { $busy } seriali jest już dopasowywanych.
       *[other] { $busy } seriali jest już dopasowywanych.
    }

## core/src/metadata/apply.rs

core-metadata-link-unreadable = Nie udało się odczytać tego linku. Wklej adres strony z AniList, MyAnimeList, TVmaze lub IMDb.
core-metadata-no-mal-entry = AniList nie ma wpisu dla tego id z MyAnimeList.
core-metadata-no-imdb-show = TVmaze nie ma serialu dla tego tytułu z IMDb.
# Shown before "is not supported yet". $provider is the provider's code: anilist, mal, tmdb.
core-metadata-search-on = wyszukiwanie w { $provider }
core-metadata-query-too-short = Wpisz co najmniej dwa znaki, aby wyszukać.
core-metadata-not-a-link = to nie link
core-metadata-refresh-unsupported = odświeżanie dopasowania tylko z MAL lub TMDB
# $show is the show's folder name.
core-metadata-matched = Dopasowano { $show }.
core-metadata-updated = Zaktualizowano informacje o { $show }.
core-metadata-update-failed = Nie udało się zaktualizować informacji o { $show }. Spróbuj później.
# Progress labels while the whole library is updated, or shows only seen in passing
# are filled in.
core-metadata-progress-refresh = odświeżanie
core-metadata-progress-backfill = uzupełnianie
core-metadata-updated-many =
    { $count ->
        [one] Zaktualizowano informacje o { $count } serialu.
        [few] Zaktualizowano informacje o { $count } serialach.
        [many] Zaktualizowano informacje o { $count } serialach.
       *[other] Zaktualizowano informacje o { $count } serialach.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] Nie udało się zaktualizować informacji o { $count } serialu. Spróbuj później.
        [few] Nie udało się zaktualizować informacji o { $count } serialach. Spróbuj później.
        [many] Nie udało się zaktualizować informacji o { $count } serialach. Spróbuj później.
       *[other] Nie udało się zaktualizować informacji o { $count } serialach. Spróbuj później.
    }
core-metadata-updated-some =
    { $count ->
        [one] Zaktualizowano informacje o { $count } serialu; nie udało się: { $failed }.
        [few] Zaktualizowano informacje o { $count } serialach; nie udało się: { $failed }.
        [many] Zaktualizowano informacje o { $count } serialach; nie udało się: { $failed }.
       *[other] Zaktualizowano informacje o { $count } serialach; nie udało się: { $failed }.
    }

## core/src/metadata/fetch.rs

# $id is the provider's own number for the show.
core-metadata-no-such-show = TVmaze nie ma serialu { $id }
core-metadata-no-such-media = AniList nie ma pozycji { $id }

## core/src/metadata/imdb.rs

# Progress labels while the ratings update.
core-metadata-progress-imdb-links = linki IMDb z Wikidata
core-metadata-progress-imdb-ratings = oceny IMDb

## core/src/franchise/layout.rs

# The words under a related show's title when the two share characters.
core-franchise-relation-character = Wspólne postacie

## core/src/images/mod.rs

# A progress label while covers and banners download.
core-images-progress-fetching = pobieranie obrazów

## core/src/transfer/export.rs

# $folder is the name of the folder the file went to.
core-transfer-exported =
    { $count ->
        [one] Zapisano { $count } serial w { $folder }.
        [few] Zapisano { $count } seriale w { $folder }.
        [many] Zapisano { $count } seriali w { $folder }.
       *[other] Zapisano { $count } seriali w { $folder }.
    }

## core/src/transfer/format.rs

# $error is the JSON parser's own message.
core-transfer-not-json = to nie JSON: { $error }
# $format is the file format's name, anibeam-export.
core-transfer-wrong-format = to nie jest dokument { $format }

## core/src/transfer/import.rs

core-transfer-imported = Wczytano bibliotekę z pliku.

## core/src/library/delete.rs

# $id is the file's number in the library.
core-library-no-such-file = brak pliku { $id } w bibliotece
core-library-file-elsewhere-not-removed = plik { $id } należy do innego serialu, więc niczego nie usunięto
core-library-file-outside = { $path } nie jest plikiem w folderze biblioteki, więc niczego nie usunięto
# $place is deleted where the system has no trash, trash otherwise. $title is the show's
# folder name.
core-library-episodes-removed =
    { $title }: { $count ->
        [one] { $count } odcinek
        [few] { $count } odcinki
        [many] { $count } odcinków
       *[other] { $count } odcinków
    } { $place ->
        [deleted] usunięto
       *[trash] przeniesiono do kosza
    }.
core-library-finished-removed =
    { $count ->
        [one] { $count } ukończony serial
        [few] { $count } ukończone seriale
        [many] { $count } ukończonych seriali
       *[other] { $count } ukończonych seriali
    }: { $place ->
        [deleted] usunięto
       *[trash] przeniesiono do kosza
    }.
# $series is how many shows the episodes came from.
core-library-watched-removed =
    { $count ->
        [one] { $count } obejrzany odcinek
        [few] { $count } obejrzane odcinki
        [many] { $count } obejrzanych odcinków
       *[other] { $count } obejrzanych odcinków
    } z seriali ({ $series }): { $place ->
        [deleted] usunięto
       *[trash] przeniesiono do kosza
    }.

## core/src/library/detail.rs

# A fix the series page offers. $read is episode, with its $number, or label, with the
# name the file was read by in $label. $what is film or episode.
core-library-fix-one-file =
    Jedyny plik tutaj jest odczytany jako { $read ->
        [episode] odcinek { $number }
       *[label] { $label }
    }. Wygląda na { $what ->
        [film] film
       *[episode] jedyny odcinek
    }.
# $numbers is a run of episode numbers, as core-library-runs-span or -list writes it;
# $episodes is how many there are.
core-library-fix-specials =
    { $count ->
        [one] { $count } specjal wygląda
        [few] { $count } specjale wyglądają
        [many] { $count } specjali wygląda
       *[other] { $count } specjali wygląda
    } na { $episodes ->
        [one] brakujący odcinek
        [few] brakujące odcinki
        [many] brakujące odcinki
       *[other] brakujące odcinki
    } { $numbers }.
core-library-fix-numbered-on = Odcinki { $from } wyglądają na odcinki { $to } tego wpisu, numerowane dalej od wcześniejszego sezonu.
# Episode numbers in a row: "1 to 6".
core-library-runs-span = { $first } do { $last }
# Episode numbers apart: "2, 4 and 7". $init is every number but the last, comma-separated.
core-library-runs-list = { $init } i { $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = numer odcinka mieści się w zakresie od 0 do 10000
core-library-fix-season-range = sezon może mieć numer najwyżej 200
core-library-fix-extra-range = indeks dodatku może wynosić najwyżej 10000
core-library-file-elsewhere-not-fixed = plik { $id } należy do innego serialu, więc niczego nie poprawiono
# $title is the show's title.
core-library-fix-undone =
    { $count ->
        [one] { $title }: { $count } plik znów jest odczytywany po nazwie.
        [few] { $title }: { $count } pliki znów są odczytywane po nazwie.
        [many] { $title }: { $count } plików znów jest odczytywanych po nazwie.
       *[other] { $title }: { $count } plików znów jest odczytywanych po nazwie.
    }
core-library-fixed =
    { $count ->
        [one] { $title }: poprawiono odczyt { $count } pliku.
        [few] { $title }: poprawiono odczyt { $count } plików.
        [many] { $title }: poprawiono odczyt { $count } plików.
       *[other] { $title }: poprawiono odczyt { $count } plików.
    }

## core/src/library/reads.rs

# $title is the show's title. The -many lines are for any count but one.
core-library-hid-one = Ukryto { $title }.
core-library-back-one = { $title } wrócił.
core-library-hid-many = Ukryto seriale: { $count }.
core-library-back-many = Przywrócono seriale: { $count }.
core-library-episode-count-set = { $title } ma teraz odcinków: { $count }.
core-library-episode-count-cleared = Wyczyszczono liczbę odcinków { $title }.

## core/src/library/schedule.rs

core-library-schedule-cleared = Wyczyszczono notatki harmonogramu { $title }.
core-library-schedule-saved = Zapisano notatki harmonogramu dla { $title }: { $count }.

## core/src/library/scan.rs

core-library-source-not-absolute = ścieżka źródła musi być bezwzględna
core-library-source-exists = to już jest źródło
core-library-source-nested = zagnieżdżone w istniejącym źródle
# $folder is the last part of the folder's path.
core-library-source-added = Dodano folder { $folder } do biblioteki.
core-library-source-removed = Usunięto folder { $folder } z biblioteki.
core-library-source-removed-with-shows =
    { $count ->
        [one] Usunięto z biblioteki folder { $folder } i jego { $count } serial.
        [few] Usunięto z biblioteki folder { $folder } i jego { $count } seriale.
        [many] Usunięto z biblioteki folder { $folder } i jego { $count } seriali.
       *[other] Usunięto z biblioteki folder { $folder } i jego { $count } seriali.
    }
# $name is the show's folder name.
core-library-series-forgotten = Usunięto { $name } z biblioteki.
core-library-source-back = Folder { $folder } wrócił.
core-library-source-unavailable = Folder { $folder } jest niedostępny. Sprawdź, czy jego dysk jest podłączony.
core-library-source-unreadable = Nie udało się odczytać folderu { $folder }. Sprawdź, czy jego dysk jest podłączony.
core-library-source-looks-empty = Folder { $folder } wygląda na pusty, więc seriale zostały bez zmian.
core-library-source-reconcile-failed = Nie udało się zaktualizować seriali w { $folder }.
# What a scan found: the parts below that are not nought, in order, the ones after the
# first set off by a comma. $first names the part that leads.
core-library-scan-done =
    Sprawdzono foldery: { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } nowy serial
            [few] { $added } nowe seriale
            [many] { $added } nowych seriali
           *[other] { $added } nowych seriali
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } zmieniony serial
            [few] { $changed } zmienione seriale
            [many] { $changed } zmienionych seriali
           *[other] { $changed } zmienionych seriali
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }brakuje: { $removed }
    }.

## core/src/library/report.rs

# Why a coding agent wrote no triage for a quick fix report. $span is
# core-library-span-minutes or -seconds.
core-library-agent-no-answer = brak odpowiedzi w ciągu { $span }
core-library-agent-stopped-code = zatrzymał się z kodem { $code }
core-library-agent-stopped = zatrzymał się
core-library-agent-answered-nothing = nic nie odpowiedział
core-library-span-minutes =
    { $count ->
        [one] { $count } minuty
        [few] { $count } minut
        [many] { $count } minut
       *[other] { $count } minuty
    }
core-library-span-seconds =
    { $count ->
        [one] { $count } sekundy
        [few] { $count } sekund
        [many] { $count } sekund
       *[other] { $count } sekundy
    }
core-library-report-no-fix = żaden z tych plików nie ma poprawki do zgłoszenia
# $agent is the agent's name, such as Claude Code; $why is one of the reasons above, or
# the agent's own last words.
core-library-agent-no-triage = { $agent } nie napisał triażu: { $why }
# Progress labels while a file is loaded, one per part of the library.
core-transfer-progress-sources = źródła
core-transfer-progress-series = seriale
core-transfer-progress-history = historia
core-transfer-progress-accounts = konta
core-transfer-progress-preferences = ustawienia
core-transfer-progress-done = gotowe

## core/src/downloads/anirss.rs

# $message is what the connection said
core-downloads-anirss-no-answer = anirss nie odpowiedział: { $message }
# $status is the HTTP status code, $text the body anirss sent
core-downloads-anirss-answered = anirss odpowiedział { $status }: { $text }
core-downloads-no-free-port = brak wolnego portu: { $error }
# $status is how the process ended, as the system words it
core-downloads-anirss-exited = anirss zakończył działanie ({ $status })
# $said is the last lines anirss printed
core-downloads-anirss-exited-said = anirss zakończył działanie ({ $status }): { $said }
core-downloads-anirss-slow-start = anirss nie uruchomił się w ciągu { $seconds } s
core-downloads-anirss-slow-start-said = anirss nie uruchomił się w ciągu { $seconds } s: { $said }
# $command is a command to type, kept as it is
core-downloads-anirss-install-mac = Zainstaluj anirss { $version } lub nowszy poleceniem: { $command }
core-downloads-anirss-install-linux = Zainstaluj anirss { $version } lub nowszy: pakiet anirss z AUR albo { $command }
# $update is core-downloads-anirss-update-mac or -linux
core-downloads-anirss-too-old = anirss { $version } jest za stary dla AniBeam, który wymaga { $minimum } lub nowszego. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } nie umie trzymać osobno logowania każdego klienta torrentów, czego AniBeam potrzebuje do przełączania klientów; anirss { $minimum } i nowsze to umieją. { $update }.
core-downloads-anirss-update-mac = Zaktualizuj go poleceniem: { $command }
core-downloads-anirss-update-linux = Zaktualizuj pakiet anirss albo uruchom: { $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = plik torrent nie dotarł: { $message }
core-downloads-torrent-file-answered = plik torrent odpowiedział { $status }

## core/src/downloads/feed.rs

core-downloads-feed-answered = kanał odpowiedział { $status }
core-downloads-not-a-feed = to nie kanał: { $error }

## core/src/downloads/jackett.rs

# $program is the command that was not found
core-downloads-jackett-missing = Jackett nie jest zainstalowany: nie znaleziono { $program }. Zainstaluj go albo zmień polecenie w Ustawieniach.
# $name is the endpoint, $why what went wrong
core-downloads-endpoint-start-failed = Nie udało się uruchomić { $name }: { $why }
core-downloads-endpoint-start-timeout = { $name } uruchomił się, ale nie odpowiedział w ciągu { $seconds } s.
core-downloads-command-exited = polecenie zakończyło się kodem { $status }

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirss się zatrzymał
# $install is core-downloads-anirss-install-mac or -linux
core-downloads-anirss-missing = { $install }. Albo wskaż w Ustawieniach działający serwer.
# $command is the anirss command set in Settings
core-downloads-anirss-command-missing = Nie znaleziono { $command }. { $install }. Albo wskaż w Ustawieniach działający serwer.
core-downloads-anirss-start-failed = Nie udało się uruchomić { $command }: { $error }
# $tooOld is core-downloads-anirss-too-old; $message is what anirss said
core-downloads-anirss-too-old-said = { $tooOld } Odpowiedź: { $message }
core-downloads-no-torrent-client = Żaden qBittorrent nie odpowiada pod { $address }, a AniBeam nie ma własnego do uruchomienia. Zainstaluj qBittorrent i włącz jego Web UI albo zainstaluj qbittorrent-nox, który AniBeam uruchomi za Ciebie
core-downloads-own-client-gone = qBittorrent AniBeam zniknął: qbittorrent-nox nie jest zainstalowany. Zainstaluj ponownie AniBeam albo wskaż w Ustawieniach własny qBittorrent
core-downloads-no-port = brak portu w { $address }
core-downloads-own-client-settings = nie można zapisać ustawień qBittorrent: { $error }
core-downloads-own-client-start-failed = Nie udało się uruchomić { $command }: { $error }
core-downloads-own-client-slow-start = qBittorrent AniBeam się nie uruchomił
# $said is the last lines of its log
core-downloads-own-client-slow-start-said = qBittorrent AniBeam się nie uruchomił: { $said }
core-downloads-not-ready = Pobieranie nie jest gotowe. Sprawdź Pobieranie w Ustawieniach.
core-downloads-ready = Pobieranie jest gotowe.
# $count is how many sources anirss's own config names
core-downloads-helper-sources =
    { $count ->
        [one] Pobieranie szuka teraz samym AniBeam, który nie czyta { $count } źródła z konfiguracji pomocnika wyszukiwania. Dodaj je w Ustawieniach, Pobieranie, albo wybierz tam pomocnika wyszukiwania.
        [few] Pobieranie szuka teraz samym AniBeam, który nie czyta { $count } źródeł z konfiguracji pomocnika wyszukiwania. Dodaj je w Ustawieniach, Pobieranie, albo wybierz tam pomocnika wyszukiwania.
        [many] Pobieranie szuka teraz samym AniBeam, który nie czyta { $count } źródeł z konfiguracji pomocnika wyszukiwania. Dodaj je w Ustawieniach, Pobieranie, albo wybierz tam pomocnika wyszukiwania.
       *[other] Pobieranie szuka teraz samym AniBeam, który nie czyta { $count } źródła z konfiguracji pomocnika wyszukiwania. Dodaj je w Ustawieniach, Pobieranie, albo wybierz tam pomocnika wyszukiwania.
    }
core-downloads-client-connected = Program do pobierania jest połączony.
core-downloads-client-not-connected = Program do pobierania nie jest połączony. Sprawdź jego adres w Ustawieniach.
core-downloads-not-on-demand = { $name } nie jest endpointem uruchamianym na żądanie
core-downloads-source-running = Źródło pobierania działa.
core-downloads-source-not-running = { $name } nie działa. Włącz je w źródłach i potwierdź, aby je uruchomić.
core-downloads-empty-query = wpisz, czego szukasz
# What a search's progress line names when it was given no source
core-downloads-first-endpoint = pierwszy endpoint
# A search's progress line. The desktop's search card compares the line with these, so
# keep the name in core-downloads-search-starting-endpoint a whole placeable.
core-downloads-search-starting = Rozpoczynanie wyszukiwania
core-downloads-search-starting-endpoint = Uruchamianie: { $endpoint }
core-downloads-search-searching-endpoint = Przeszukiwanie: { $endpoint }
core-downloads-search-reading = Czytanie wyników
# $name is the subscription's name
core-downloads-subscribed = Zasubskrybowano { $name }.
core-downloads-client-no-rule = qBittorrent nie ma reguły o nazwie { $name }
# A progress line while a subscription's feed is narrowed; $search is the search tried
core-downloads-trying-search = Próba: „{ $search }”
core-downloads-nothing-selected = nie wybrano nic do pobrania
core-downloads-added =
    { $count ->
        [one] Dodano { $count } pobranie.
        [few] Dodano { $count } pobrania.
        [many] Dodano { $count } pobrań.
       *[other] Dodano { $count } pobrania.
    }
core-downloads-movie-link = link do filmu musi być magnetem lub adresem http(s)
core-downloads-added-movie = Dodano film do pobrań.
core-downloads-unsubscribed = Anulowano subskrypcję { $name }.
# $folder is a path, $name the library show that lives there
core-downloads-library-folder = { $folder } to folder biblioteki serialu { $name }, z odcinkami, których ta subskrypcja nie pobrała: usuń subskrypcję bez jej plików albo usuń { $name } z jego strony
core-downloads-no-torrents-named = nie podano nazw torrentów
core-downloads-paused =
    { $count ->
        [one] Wstrzymano { $count } pobranie.
        [few] Wstrzymano { $count } pobrania.
        [many] Wstrzymano { $count } pobrań.
       *[other] Wstrzymano { $count } pobrania.
    }
core-downloads-resumed =
    { $count ->
        [one] Wznowiono { $count } pobranie.
        [few] Wznowiono { $count } pobrania.
        [many] Wznowiono { $count } pobrań.
       *[other] Wznowiono { $count } pobrania.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] Usunięto { $count } pobranie razem z plikami.
        [few] Usunięto { $count } pobrania razem z plikami.
        [many] Usunięto { $count } pobrań razem z plikami.
       *[other] Usunięto { $count } pobrania razem z plikami.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] Usunięto { $count } pobranie z listy.
        [few] Usunięto { $count } pobrania z listy.
        [many] Usunięto { $count } pobrań z listy.
       *[other] Usunięto { $count } pobrania z listy.
    }
core-downloads-nothing-to-remove = nic do usunięcia
# $title is the show's title
core-downloads-removed-show = Usunięto { $title } z pobrań.
core-downloads-removed-the-show = Usunięto serial z pobrań.
core-downloads-deleted-still-listed = Usunięte pliki wciąż są na liście w programie do pobierania.
core-downloads-feed-url = kanał to adres http(s)
core-downloads-rule-needs-name = reguła musi mieć nazwę
core-downloads-no-rule = brak reguły o nazwie { $name }
core-downloads-feed-no-search = kanał nie zawiera wyszukiwania do skrócenia
core-downloads-no-repair = { $name } nie wymaga naprawy.
core-downloads-repaired = Naprawiono subskrypcję { $name }.
# $search is the shortest search tried
core-downloads-repair-failed = żadne krótsze wyszukiwanie niczego nie znajduje; nawet „{ $search }” wraca puste
# $id is the client's id, in quotes
core-downloads-no-saved-client = brak zapisanego klienta torrentów { $id }
core-downloads-client-switched = Pobieranie używa teraz innego programu do pobierania.
# After the name of anirss or qBittorrent and a colon
core-downloads-host-not-running = nie działa na hoście
core-downloads-host-late = nie odpowiedział na czas

## core/src/downloads/native.rs

core-downloads-native-empty-query = puste zapytanie
# $query is the search, in quotes
core-downloads-native-no-results = brak wyników dla { $query }
# $name is the feed's name, in quotes
core-downloads-native-feed-taken = kanał o nazwie { $name } czyta już inny adres
core-downloads-native-no-items = nie podano pozycji
# $link is the link given, in quotes
core-downloads-native-bad-link = linki muszą być adresami magnet: lub http(s), podano { $link }
core-downloads-native-no-links = nie podano linków
core-downloads-native-no-names = nie podano nazw subskrypcji
core-downloads-native-files-elsewhere = klient torrentów trzyma pliki na innej maszynie, więc mogą zniknąć tylko razem z torrentami
# $names is a list of feed names
core-downloads-native-unknown-feeds = nieznane kanały: { $names }
# $paths is a list of folders
core-downloads-native-outside-folders = odmowa usunięcia plików spoza folderów pobierania: { $paths }
# Removals refused; $path is the folder or file
core-downloads-refuse-home-folder = { $path } to folder domowy lub go zawiera, więc niczego nie usunięto
core-downloads-refuse-library-folder = { $path } to folder biblioteki lub go zawiera, więc niczego nie usunięto

## core/src/downloads/paths.rs

core-downloads-no-default-folder = qBittorrent nie ma domyślnego folderu zapisu; ustaw go w qBittorrent
core-downloads-name-empty = nazwa nie może być pusta
core-downloads-name-slash = nazwa nie może zawierać / ani \
# $name is the name given, in quotes
core-downloads-name-not-folder = { $name } to nie nazwa folderu

## core/src/downloads/prune.rs

# $title is the show's title
core-downloads-pruned = Anulowano subskrypcję { $title }, emisja się zakończyła.
core-downloads-prune-failed = Nie udało się anulować subskrypcji { $title }. Sprawdź Pobieranie w Ustawieniach.

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path } nie jest zwykłą ścieżką bezwzględną, więc niczego nie usunięto
core-downloads-refuse-root = { $path } to katalog główny systemu plików, więc niczego nie usunięto
core-downloads-refuse-outside-library = { $path } nie jest w folderze biblioteki, więc niczego nie usunięto
core-downloads-refuse-save-folder = { $path } to folder, w którym zapisano torrent, a nie to, co pobrał, więc niczego nie usunięto

## core/src/downloads/watch.rs

# $episode is the episode's number, $title the show's
core-downloads-episode-missed = Odcinek { $episode } serialu { $title } nie pojawił się trzy dni po emisji. Twój kanał wciąż go szuka.

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = polecenie anirss nie może być puste
core-downloads-settings-anirss-address = adres anirss musi zaczynać się od http:// lub https://
# $client is the torrent client's name, such as qBittorrent
core-downloads-settings-client-address = adres { $client } musi zaczynać się od http:// lub https://
# $id is the id given, in quotes
core-downloads-settings-bad-id = { $id } to nie id klienta: użyj liter, cyfr, - i _
core-downloads-settings-same-id = dwa klienty mają id { $id }
# $name is a saved client's name
core-downloads-settings-saved-address = adres { $name } musi zaczynać się od http:// lub https://
core-downloads-settings-same-name = dwa klienty nazywają się { $name }; nadaj każdemu własną nazwę

## core/src/downloads/qbt.rs

# $address is where qBittorrent was asked, $message what the connection said
core-downloads-qbt-no-answer = qBittorrent nie odpowiedział pod { $address }: { $message }
core-downloads-qbt-password-rejected = qBittorrent odrzucił hasło; popraw je w Ustawieniach
core-downloads-qbt-banned = qBittorrent zablokował ten adres po nieudanych logowaniach; poczekaj albo uruchom ponownie qBittorrent
core-downloads-qbt-login-rejected = qBittorrent odrzucił nazwę użytkownika lub hasło
# $status is the HTTP status code
core-downloads-qbt-login-failed = logowanie do qBittorrent pod { $address } odpowiedziało { $status }; kolejna próba przy następnym wywołaniu
core-downloads-qbt-not-web-ui = { $address } odpowiedział { $status }, ale nie jako Web UI qBittorrent. Sprawdź adres w Ustawieniach i czy Web UI jest włączone w qBittorrent
# $said is what qBittorrent answered, in quotes
core-downloads-qbt-not-added = qBittorrent nie dodał torrentów: { $said }
core-downloads-qbt-some-not-added = qBittorrent nie mógł dodać części torrentów ({ $count }): { $said }
# $line is the first line of what qBittorrent answered
core-downloads-qbt-answered = qBittorrent odpowiedział { $status }: { $line }

## core/src/downloads/replace.rs

# $item is the number of the release asked for, $count how many there are
core-downloads-replace-bad-item = pozycja { $item } nie należy do wydań (jest ich { $count })
# $episode is the episode's number
core-downloads-not-replaced = Odcinek { $episode } nie został zastąpiony.
core-downloads-replaced = Zastąpiono odcinek { $episode } nowym pobraniem.

## core/src/downloads/rolling.rs

# $id is the rolling download's number
core-downloads-no-roll = brak pobierania kroczącego { $id }
# $title is the show, $from and $to the episodes kept on disk out of $total
core-downloads-rolled = { $title }: teraz odcinki od { $from } do { $to } z { $total }.
core-downloads-roll-ahead = trzymaj co najmniej jeden odcinek do przodu
core-downloads-roll-nothing = nic do pobrania

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = źródło wyszukiwania JSON pochodzi z repozytorium endpointów, którego definicja mapuje jego pola
core-releases-endpoint-no-name = brak `name`
# $name is the endpoint's name, in quotes
core-releases-endpoint-bad-name = { $name }: używaj tylko liter, cyfr, '.', '_', '-'
# $why is core-releases-json-needs-repo
core-releases-endpoint-json = { $name }: { $why }
# $kind is the kind given, in quotes; $valid lists the kinds there are
core-releases-endpoint-unknown-kind = { $name }: nieznany rodzaj { $kind } (dozwolone: { $valid })
core-releases-endpoint-no-url = { $name }: brak `url`
core-releases-endpoint-not-http = { $name }: adres musi zaczynać się od http:// lub https://
# $kind is the endpoint's kind; {query} is written as it is in the address
core-releases-endpoint-needs-query = { $name }: url { $kind } musi zawierać symbol {"{"}query{"}"}
core-releases-endpoint-no-query = { $name }: url { $kind } sam dodaje wyszukiwanie, więc nie przyjmuje {"{"}query{"}"}

## core/src/releases/fetch.rs

# $code is the error code the indexer sent
core-releases-indexer-error = indekser odpowiedział błędem { $code }
core-releases-bad-feed = zły kanał: { $error }
# $status is the HTTP status code
core-releases-feed-answered = kanał odpowiedział { $status }

## core/src/releases/json_feed.rs

core-releases-not-feed-json = to nie JSON kanału: { $error }

## core/src/releases/json.rs

core-releases-not-json = to nie JSON: { $error }
# $message is what the search said about itself
core-releases-search-error-said = wyszukiwanie odpowiedziało błędem: { $message }
core-releases-search-error = wyszukiwanie odpowiedziało błędem
# $path is where the results were looked for, in quotes
core-releases-no-results-field = odpowiedź nie ma wyników pod { $path }
core-releases-results-not-list = wyniki w odpowiedzi nie są listą
core-releases-search-answered = wyszukiwanie odpowiedziało { $status }

## core/src/releases/native.rs

core-releases-no-endpoints = brak endpointów: dodaj repozytorium albo endpoint ręcznie w Więcej, Endpointy
core-releases-no-source-named = brak źródła o nazwie { $name }
core-releases-empty-query = wpisz, czego szukasz
core-releases-cannot-follow = { $name } nie da się obserwować: odpowiada na wyszukiwania, ale nie ma kanału do sprawdzania nowych odcinków
core-releases-endpoint-gone = { $name } nie jest już dostępny: usunięty, wyłączony albo bez klucza

## core/src/releases/quirks.rs

# What is wrong with an endpoint definition's quirks. one_variant, show_feed, slug and
# {slug} are the definition's own words and stay as they are.
# $word is the word given, in quotes
core-releases-quirks-variant = one_variant { $word } musi być jednym krótkim słowem
core-releases-quirks-pattern-count = show_feed przyjmuje od 1 do { $most } wzorców stron
core-releases-quirks-pattern-long = wzorzec strony show_feed jest za długi
core-releases-quirks-pattern-bad = wzorzec strony show_feed: { $error }
core-releases-quirks-pattern-slug = wzorzec strony show_feed potrzebuje grupy o nazwie slug
core-releases-quirks-url-https = url show_feed musi być https
core-releases-quirks-url-host = url show_feed musi być na hoście endpointu
core-releases-quirks-url-slug = url show_feed przyjmuje {"{"}slug{"}"} raz i żadnego innego symbolu

## core/src/releases/refinement.rs

# 'tokens' and `tokens` name the call's own field and stay as they are
core-releases-refine-no-tokens = op 'tokens' wymaga niepustej listy `tokens`

## core/src/releases/grid.rs

# A release group's whole-season torrent; $group is the group's name
core-releases-group-batch = paczka { $group }
# $count is how many episodes, $list their numbers and runs ("1 to 4, 7")
core-releases-episodes-phrase =
    { $count ->
        [one] Odcinek { $list }
        [few] Odcinki { $list }
        [many] Odcinki { $list }
       *[other] Odcinki { $list }
    }
# A run of episode numbers
core-releases-episode-range = { $from } do { $to }

## Names that stay the same in every language

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = nazwa ma od 1 do { $max } znaków, bez znaków sterujących
core-prefs-room-relay = adres przekaźnika musi zaczynać się od https:// lub http://
core-room-service = Wspólne oglądanie
core-room-not-found = Nie ma pokoju o tym kodzie. Mógł zostać zamknięty.
core-room-full = Ten pokój jest pełny.
core-room-proto = Ten przekaźnik używa innej wersji pokojów. Zaktualizuj { -app-name }, aby dołączyć.
core-room-unreachable = Nie udało się połączyć z przekaźnikiem: { $message }
core-room-refused = Przekaźnik odmówił: { $message }
core-room-not-in = Nie jesteś w żadnym pokoju.
core-room-no-name = wybierz nazwę, pod którą widzą Cię inni w pokoju
core-room-bad-code = kod pokoju to 6 liter i cyfr
core-room-created = Utworzono pokój wspólnego oglądania. Jego kod to { $code }.
core-room-joined = Dołączono do pokoju wspólnego oglądania { $code }.
core-room-left = Opuszczono pokój wspólnego oglądania.
core-room-member-joined = { $name } dołącza do pokoju.
core-room-member-left = { $name } opuszcza pokój.
core-room-closed = Pokój wspólnego oglądania został zamknięty.
core-room-lost = Utracono połączenie z pokojem wspólnego oglądania. Ponawianie.
core-room-back = Ponownie w pokoju wspólnego oglądania.
