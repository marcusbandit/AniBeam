# pl translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = Start
nav-library = Biblioteka
nav-airing = Teraz w emisji
nav-feed = Aktualności
nav-watching = Oglądam
nav-stats = Statystyki
nav-metadata = Informacje
nav-downloads = Pobrane
nav-settings = Ustawienia
nav-series = Serial
nav-player = Odtwarzacz
nav-show = Serial
nav-recommended = Polecane

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } błąd
    [few] { $count } błędy
    [many] { $count } błędów
   *[other] { $count } błędu
}

## ScanSummary

scan-summary-nothing = Na razie nic tu nie ma
scan-summary-found = Znaleziono { $count ->
    [one] { $count } serial
    [few] { $count } seriale
    [many] { $count } seriali
   *[other] { $count } seriali
}, rozpoznano: { $recognised }
scan-summary-found-need = Znaleziono { $count ->
    [one] { $count } serial
    [few] { $count } seriale
    [many] { $count } seriali
   *[other] { $count } seriali
}, rozpoznano: { $recognised }, czeka na Ciebie: { $need }
scan-summary-episodes = { $count ->
    [one] { $count } odcinek
    [few] { $count } odcinki
    [many] { $count } odcinków
   *[other] { $count } odcinka
} na dysku.
scan-summary-episodes-watched = { $count ->
    [one] { $count } odcinek
    [few] { $count } odcinki
    [many] { $count } odcinków
   *[other] { $count } odcinka
} na dysku, już obejrzane w { $tracker }: { $watched }.
scan-summary-title = Twój folder jest dodany
scan-summary-count-found = znaleziono
scan-summary-count-recognised = rozpoznano
scan-summary-count-need = czeka na Ciebie
scan-summary-later = Później
scan-summary-match = Dopasuj ({ $count })
scan-summary-open-library = Otwórz bibliotekę

## UpdateNotice

update-notice-check-failed = AniBeam nie mógł sprawdzić aktualizacji. Sprawdź połączenie z internetem i spróbuj ponownie.
update-notice-up-to-date = AniBeam jest aktualny
update-notice-downloading = Pobieranie AniBeam { $version }
update-notice-ready = AniBeam { $version } jest gotowy
update-notice-failed = Aktualizacja nie powiodła się
update-notice-on-aur = AniBeam { $version } jest w AUR
update-notice-out = Wyszedł AniBeam { $version }
update-notice-out-titled = Wyszedł AniBeam { $version } „{ $title }”
update-notice-progress = { $percent }%  ·  { $got } z { $total } MB
update-notice-restart-hint = Uruchom ponownie teraz albo przy następnym otwarciu
update-notice-from-source = Ta kopia jest zbudowana ze źródeł: pobierz zmiany i przebuduj
update-notice-no-download = Brak pliku do pobrania dla tego komputera
update-notice-install-hint = Aktualizuj instaluje ją i uruchamia AniBeam ponownie
update-notice-retry = Ponów
update-notice-update = Aktualizuj
update-notice-restart = Uruchom ponownie
update-notice-copied = Skopiowano
update-notice-copy = Kopiuj
update-notice-release-page = Strona wydania
update-notice-later = Później

## WhatsNew

whats-new-title = Co nowego
whats-new-got-it = Jasne

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = d MMM HH:mm
activity-drawer-stage-library = Biblioteka
activity-drawer-stage-metadata = Metadane
activity-drawer-stage-trackers = Serwisy
activity-drawer-stage-franchise = Franczyza
activity-drawer-stage-playback = Odtwarzanie
activity-drawer-stage-store = Dane
activity-drawer-stage-system = System
activity-drawer-stage-downloads = Pobieranie
activity-drawer-level-info = Info
activity-drawer-level-warn = Ostrzeżenie
activity-drawer-level-error = Błąd
activity-drawer-copied-line = Skopiowano wiersz
activity-drawer-title = Aktywność
activity-drawer-everything = Wszystko
activity-drawer-problems = Problemy
activity-drawer-copied = Skopiowano
activity-drawer-copy = Kopiuj
activity-drawer-clear-every = Wyczyść wszystkie wiersze
activity-drawer-copy-tip = Kliknij, aby skopiować wiersz
activity-drawer-fold = Zwiń
activity-drawer-empty-technical = Jeszcze nic nie zapisano.
activity-drawer-empty-problems = Brak problemów. Wszystko zadziałało.
activity-drawer-empty = Na razie nic do zgłoszenia. Tu pojawi się to, co AniBeam robi dla Ciebie.

## ActivityColumn

activity-column-episode-run = Odcinki { $first }-{ $last }
activity-column-episode-list = Odcinki { $list }
activity-column-downloads = { $count ->
    [one] { $count } pobranie
    [few] { $count } pobrania
    [many] { $count } pobrań
   *[other] { $count } pobrania
}
activity-column-batch = Paczka
activity-column-download = Pobranie
activity-column-name-batch = { $title }, paczka
activity-column-name-episode = { $title }, odcinek { $number }
activity-column-title = Aktywność
activity-column-down = pobieranie
activity-column-up = wysyłanie
activity-column-not-connected = { $client } nie jest połączony, więc nic się tu nie rusza.
activity-column-connect-again = Połącz ponownie
activity-column-downloading = Teraz pobierane
activity-column-fewer = Mniej
activity-column-more = I jeszcze { $count }
activity-column-nothing-downloading = Nic się nie pobiera.
activity-column-finished-today = Ukończone dziś
activity-column-today = Dziś
activity-column-today-empty = Wyszukiwania, pobrania i nowe subskrypcje pojawiają się tu na bieżąco.
activity-column-stopping = Zatrzymywanie
activity-column-starting = Uruchamianie
activity-column-start-tip = Zacznij pobierać
activity-column-stop-tip = Zatrzymaj pobieranie; to, co już jest, zostaje
activity-column-cancelling = Anulowanie
activity-column-cancel-all-tip = Anuluj te pobrania
activity-column-cancel-tip = Anuluj to pobranie
activity-column-size-of = { $got } z { $total }
activity-column-files-missing = Brak plików
activity-column-time-format = HH:mm
activity-column-left = Zostało { $time }
activity-column-waiting-peers = Czekam na peerów
activity-column-episodes-done = { $count ->
    [one] Gotowe { $done } z { $count } odcinka
    [few] Gotowe { $done } z { $count } odcinków
    [many] Gotowe { $done } z { $count } odcinków
   *[other] Gotowe { $done } z { $count } odcinka
}
activity-column-cube-skipped = Odcinek { $number }: pominięty
activity-column-cube-done = Odcinek { $number }: gotowy
activity-column-cube-progress = Odcinek { $number }: { $percent }%

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Gotowe
frame-strip-progress = { $doing }, { $done } z { $total }
frame-strip-idle = Nic nie działa
frame-strip-time-format = HH:mm
frame-finish-sign-in = Najpierw dokończ logowanie do { $tracker }
frame-did-not-work = To nie zadziałało.
frame-could-not-open = AniBeam nie mógł tego otworzyć. Skopiuj i otwórz samodzielnie.
frame-copy = Kopiuj
frame-copied = Skopiowano
frame-status-refused = { $tracker } tego nie przyjął
frame-status-refused-why = { $tracker } tego nie przyjął: { $why }
frame-lists-refused = { $tracker } nie przyjął { $count ->
    [one] 1 serialu
    [few] { $count } seriali
    [many] { $count } seriali
   *[other] { $count } seriali
}; zobacz dziennik aktywności
frame-lists-moved = Przeniesiono { $count ->
    [one] 1 serial
    [few] { $count } seriale
    [many] { $count } seriali
   *[other] { $count } seriali
} do { $tracker }
frame-notes-refused = { $tracker } nie przyjął Twoich notatek
frame-notes-refused-why = { $tracker } nie przyjął Twoich notatek: { $why }
frame-notes-saved = Zapisano notatki
frame-connected = Połączono z { $tracker } jako { $user }
frame-sign-in-stopped = Logowanie do { $tracker } zatrzymało się przed końcem.
frame-toast-hide-details = Ukryj szczegóły
frame-toast-details = Szczegóły
frame-toast-details-tip = Pokaż dokładnie, co poszło nie tak, i skopiuj to
frame-forward = Dalej
frame-copy-magnet = Kopiuj magnet
frame-copy-link = Kopiuj link do torrenta
frame-magnet-copied = Skopiowano magnet
frame-link-copied = Skopiowano link
frame-open-release-page = Otwórz stronę wydania
frame-open-folder = Otwórz folder
frame-copy-path = Kopiuj ścieżkę
frame-path-copied = Skopiowano ścieżkę
frame-mark-unwatched = Oznacz jako nieobejrzane od tego miejsca
frame-mark-watched = Oznacz jako obejrzane do tego miejsca
frame-copy-title = Kopiuj tytuł
frame-open-on-anilist = Otwórz w AniList
frame-list-home-online = Zmień to na każdej połączonej liście
frame-list-home-unmatched = Zapisane na tym urządzeniu: żaden serwis nie ma tego serialu
frame-list-home-not-carried = Zapisane na tym urządzeniu: Twój serwis nie ma tego serialu
frame-list-home-offline = Zapisane na tym urządzeniu, dopóki nie połączysz AniList lub MyAnimeList
frame-moving-to = Przenoszenie do: { $list }
frame-moved-to = Przeniesiono do: { $list }
frame-plan-added = { $title } trafia na listę Planuję
frame-plan-listed = { $title } jest już na Twojej liście
frame-delete-watched = { $kind ->
    [film] Usuń obejrzane filmy
   *[episode] Usuń obejrzane odcinki
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] Usuń film
       *[episode] Usuń odcinek
    }
    [few] { $kind ->
        [film] Usuń wszystkie { $count } filmy
       *[episode] Usuń wszystkie { $count } odcinki
    }
    [many] { $kind ->
        [film] Usuń wszystkie { $count } filmów
       *[episode] Usuń wszystkie { $count } odcinków
    }
   *[other] { $kind ->
        [film] Usuń wszystkie { $count } filmu
       *[episode] Usuń wszystkie { $count } odcinka
    }
}
frame-series-gone = Ten serial zniknął
frame-delete-none-watched = Brak obejrzanych odcinków { $title } na dysku
frame-delete-none = Nic z { $title } nie ma na dysku
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } obejrzany film
            [few] { $count } obejrzane filmy
            [many] { $count } obejrzanych filmów
           *[other] { $count } obejrzanego filmu
        }
       *[episode] { $count ->
            [one] { $count } obejrzany odcinek
            [few] { $count } obejrzane odcinki
            [many] { $count } obejrzanych odcinków
           *[other] { $count } obejrzanego odcinka
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } film
            [few] { $count } filmy
            [many] { $count } filmów
           *[other] { $count } filmu
        }
       *[episode] { $count ->
            [one] { $count } odcinek
            [few] { $count } odcinki
            [many] { $count } odcinków
           *[other] { $count } odcinka
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } dodatek
    [few] { $count } dodatki
    [many] { $count } dodatków
   *[other] { $count } dodatku
}
frame-delete-and = { $first } i { $second }
frame-delete-heading = Usunąć z { $title }: { $what }?
frame-delete-base = Pliki i ich napisy trafią do kosza, więc możesz je stamtąd odzyskać. Ich torrenty znikną z klienta torrentów, więc można je pobrać ponownie.
frame-delete-confirm = Przenieś do kosza
frame-delete-done = { $title }: { $what } w koszu
frame-delete-watched-done = { $count ->
    [one] { $count } odcinek
    [few] { $count } odcinki
    [many] { $count } odcinków
   *[other] { $count } odcinka
}, { $size }, w koszu
frame-delete-nothing = Nic do usunięcia
frame-fix-not-drafted = Poprawiono. Nie udało się przygotować zgłoszenia
frame-fix-agent = Poprawiono. { $agent } to sprawdza, a zgłoszenie otworzy się, gdy skończy
frame-fix-opening = Poprawiono. Otwieranie zgłoszenia
frame-fix-not-saved = Poprawka się nie zapisała: { $error }
frame-fix-undone = { $count ->
    [one] { $count } plik znów odczytywany po nazwie
    [few] { $count } pliki znów odczytywane po nazwie
    [many] { $count } plików znów odczytywanych po nazwie
   *[other] { $count } pliku znów odczytywanego po nazwie
}
frame-fix-done = Poprawiono odczyt { $count ->
    [one] { $count } pliku
    [few] { $count } plików
    [many] { $count } plików
   *[other] { $count } pliku
}
frame-report-failed = Nie udało się przygotować zgłoszenia: { $error }
frame-report-untriaged = Zgłoszenie jest otwarte w przeglądarce, bez triażu. { $error }
frame-report-open = Zgłoszenie jest otwarte w przeglądarce, gotowe do wysłania
frame-downloads-not-read = Nie odczytano.

## HomePage

home-title = Start
home-scan-started = Skanowanie ruszyło
home-ago-minutes = { $count } min temu
home-ago-hours = { $count } godz. temu
home-ago-days = { $count } d temu
home-today = Dziś
home-tomorrow = Jutro
home-yesterday = Wczoraj
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d MMM
home-day-date-format = ddd d MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = Jeszcze bez daty
home-unit-days = { $count ->
    [one] dzień
    [few] dni
    [many] dni
   *[other] dnia
}
home-unit-hours = h
home-unit-minutes = min
home-week-late = { $count ->
    [one] { $count } odcinek się spóźnia
    [few] { $count } odcinki się spóźniają
    [many] { $count } odcinków się spóźnia
   *[other] { $count } odcinka się spóźnia
}
home-week-waiting = { $count ->
    [one] { $count } odcinek czeka na obejrzenie
    [few] { $count } odcinki czekają na obejrzenie
    [many] { $count } odcinków czeka na obejrzenie
   *[other] { $count } odcinka czeka na obejrzenie
}
home-week-coming = { $count ->
    [one] { $count } odcinek w ciągu 7 dni
    [few] { $count } odcinki w ciągu 7 dni
    [many] { $count } odcinków w ciągu 7 dni
   *[other] { $count } odcinka w ciągu 7 dni
}
home-week-next = Następny: { $title } za { $time }
home-countdown-days = { $days } d { $hours } h
home-countdown-hours = { $hours } h { $minutes } min
home-countdown-minutes = { $minutes } min
home-out-not-downloaded = Jeszcze nie pobrano
home-out-downloading = Pobieranie
home-out-when = { $day } { $time }
home-next-up = Następny
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, czas szacunkowy
home-next-when-season = Sezon { $season }, { $span }, { $time }
home-next-when-season-estimated = Sezon { $season }, { $span }, { $time }, czas szacunkowy
home-no-date = Bez daty
home-tile-watched = obejrzane
home-tile-downloading = pobieranie
home-tile-out = wyszło { $ago }
home-tile-not-out = jeszcze nie wyszło
home-tile-estimated = szacunkowo
home-tile-season-span = S{ $season } { $span }
home-tile-to-watch = { $count } do obejrzenia
home-day-nothing = Nic
home-now = Teraz { $time }
home-card-aired = { $episode } wyszedł { $ago }
home-continue = Kontynuuj
home-continue-fact = Seriale do kontynuowania: { $count }
home-on-hold = Wstrzymane
home-on-hold-fact = Seriale do powrotu: { $count }
home-hero-open = Otwórz { $title }
home-hero-pick-up = Oglądaj od miejsca przerwania
home-hero-up-next = Następny
home-hero-downloading = Pobieranie: { $count ->
    [one] { $count } odcinek
    [few] { $count } odcinki
    [many] { $count } odcinków
   *[other] { $count } odcinka
}
home-hero-min-left = Zostało { $count } min
home-hero-watched = Obejrzano { $watched } z { $total }
home-hero-resume = Wznów odcinek { $number }
home-hero-play = Odtwórz odcinek { $number }
home-hero-resume-unnumbered = Wznów odcinek
home-hero-play-unnumbered = Odtwórz odcinek
home-downloaded = Ostatnio pobrane
home-downloaded-fact = Seriale z nowymi plikami z ostatnich dwóch tygodni: { $count }
home-week = Ten tydzień
home-week-show-empty = Pokaż puste dni
home-week-hide-empty = Ukryj puste dni
home-week-airing = Teraz w emisji
home-week-out = Już wyszło
home-week-coming-up = Wkrótce
home-recommended = Polecane dla Ciebie
home-recommended-fact = Na podstawie Twojej listy
home-recommended-scope = Polecane z
home-recommended-all = Wszystko
home-recommended-in-library = W bibliotece
home-recommended-on-list = Na liście
home-recommended-none-library = W bibliotece nie zostało nic do polecenia. Wszystko znajduje seriale, których jeszcze nie masz.
home-recommended-none-list = Na Twoich listach nie zostało nic do polecenia. Wszystko znajduje seriale spoza list.
home-catch-up = Do nadrobienia
home-catch-up-fact = Seriale z czekającymi odcinkami: { $count }
home-fresh = Zacznij coś nowego
home-fresh-fact = Nierozpoczęte seriale: { $count }
home-genres = Gatunki
home-open-library = Otwórz bibliotekę
home-dropped = Porzucone
home-dropped-fact = { $count ->
    [one] { $count } plik wciąż na dysku
    [few] { $count } pliki wciąż na dysku
    [many] { $count } plików wciąż na dysku
   *[other] { $count } pliku wciąż na dysku
}
home-out-now = Już wyszło
home-out-now-fact = Nowe odcinki w tym tygodniu, z AniList
home-popular = Popularne w tym sezonie
home-add-folder-title = Twoje seriale też tu trafią
home-add-folder-body = Dodaj folder z anime, a Start wypełni się tym, co oglądasz.
home-add-folder-button = Dodaj folder
home-empty-scanned-title = W folderach jeszcze nic nie znaleziono
home-empty-title = Jeszcze brak seriali
home-empty-scanned-body = AniBeam nie znalazł seriali ani filmów w obserwowanych folderach. Przeskanuj je ponownie albo sprawdź foldery w Ustawieniach.
home-empty-body = Dodaj folder z anime, a biblioteka wypełni się sama.
home-scan-all = Skanuj wszystko
home-library-settings = Ustawienia biblioteki
home-failed-title = Start się nie wczytał
home-failed-body = AniBeam nie mógł teraz odczytać biblioteki. Spróbuj za chwilę.

## Onboarding

onboarding-language = Język
onboarding-step-welcome = Witaj
onboarding-step-library = Biblioteka
onboarding-step-lists = Twoja lista
onboarding-step-downloads = Pobieranie
onboarding-step-ready = Gotowe
onboarding-scan-reading = Czytanie folderu
onboarding-scan-found = Znalezione seriale: { $count }. Przejdź dalej, kiedy chcesz; reszta uzupełni się sama.
onboarding-scan-starter-empty = Twój nowy folder jest na razie pusty. Seriale, które do niego wrzucisz, pojawią się tu same.
onboarding-scan-no-video = Nie znaleziono plików wideo. Sprawdź, czy to właściwy folder.
onboarding-scan-your-shows = Twoich seriali
onboarding-scan-looking-up = Wyszukiwanie { $what }
onboarding-scan-looking-up-tally = Wyszukiwanie { $what }, znaleziono { $posters } z { $count }
onboarding-scan-posters = Pobieranie plakatów
onboarding-scan-posters-tally = Pobieranie plakatów, znaleziono { $posters } z { $count }
onboarding-scan-seasons = Szukanie powiązanych sezonów
onboarding-scan-seasons-tally = Szukanie powiązanych sezonów, znaleziono { $posters } z { $count }
onboarding-choose-folder = Wybierz folder z anime
onboarding-about = Minuta konfiguracji. Biblioteka dokończy się sama.
onboarding-step-later = Później
onboarding-skip-setup = Pomiń konfigurację
onboarding-title-welcome = Witaj w AniBeam
onboarding-title-library = Gdzie jest Twoje anime?
onboarding-title-lists = Połącz swoją listę anime
onboarding-title-downloads = Jak działa pobieranie
onboarding-title-ready = Wszystko gotowe
onboarding-title-later = { $count ->
    [1] Gotowe, jedna rzecz na później
    [2] Gotowe, dwie rzeczy na później
   *[other] Gotowe, trzy rzeczy na później
}
onboarding-body-welcome = AniBeam trzyma Twoje anime w jednym miejscu. Znajduje seriale na dysku, odtwarza je tutaj i na bieżąco aktualizuje AniList lub MyAnimeList, gdy oglądasz.
onboarding-body-library = Wybierz folder, w którym trzymasz seriale. Każdy folder w środku staje się serialem, z plakatem, odcinkami i miejscem, w którym skończyłeś. AniBeam tylko czyta folder. Nigdy nie przenosi ani nie zmienia nazw plików.
onboarding-body-lists = Zaloguj się raz, a każdy odcinek obejrzany tutaj zostanie oznaczony na Twojej liście, bez wpisywania czegokolwiek. Logowanie odbywa się w przeglądarce, więc AniBeam nigdy nie widzi Twojego hasła.
onboarding-body-downloads = Obserwuj serial na stronie Pobrane, a każdy nowy odcinek trafi do biblioteki zaraz po emisji. Potrzebne są do tego dwie rzeczy: miejsce do wyszukiwania i klient torrentów do pobierania.
onboarding-body-ready = Wszystko tu można zmienić później w Ustawieniach, a Ustawienia, Informacje uruchamia tę konfigurację ponownie.
onboarding-skip-step = Na razie pomiń
onboarding-get-started = Zaczynamy
onboarding-open-app = Otwórz AniBeam
onboarding-continue = Dalej
onboarding-payoff-library = Twoja biblioteka
onboarding-payoff-library-line = Plakaty, odcinki i to, co dalej, dla każdego serialu w Twoich folderach.
onboarding-payoff-plays = Odtwarza tutaj
onboarding-payoff-plays-line = Napisy, pomijanie intro i pamięta, gdzie skończyłeś.
onboarding-payoff-list = Twoja lista, zsynchronizowana
onboarding-payoff-list-line = Obejrzyj odcinek, a zaliczy się na AniList lub MyAnimeList.
onboarding-payoff-episodes = Odcinki przychodzą same
onboarding-payoff-episodes-line = Obserwuj serial, a każdy nowy odcinek trafi do biblioteki.
onboarding-films-too = Folder z filmami też działa: nazwij go Movies.
onboarding-choose-folder-button = Wybierz folder
onboarding-no-folder = Nie mam jeszcze folderu
onboarding-starter-where = To utworzy folder Anime w { $dir } i go użyje.
onboarding-add-another = Dodaj kolejny folder
onboarding-shelf-more = więcej
onboarding-preview-show = Twój serial
onboarding-preview-finished = Obejrzano tu odcinek 3
onboarding-preview-on-list = Na Twojej liście: obejrzane 3
onboarding-preview-set-by = Ustawione przez AniBeam, gdy tylko lecą napisy końcowe
onboarding-lists-connected = Połączono. To, co tu oglądasz, liczy się teraz na Twojej liście.
onboarding-lists-no-account = Nie masz konta? Oba są darmowe i jedno wystarczy. Możesz rozłączyć w każdej chwili w Ustawieniach, Konta.
onboarding-search-title = Gdzie szukać
onboarding-search-ready = { $count ->
    [one] { $count } źródło gotowe do wyszukiwania.
    [few] { $count } źródła gotowe do wyszukiwania.
    [many] { $count } źródeł gotowych do wyszukiwania.
   *[other] { $count } źródła gotowego do wyszukiwania.
}
onboarding-search-none = AniBeam nie ma własnych źródeł. Dodajesz je z repozytorium źródeł: linku, który ktoś Ci udostępni, a Ty wklejasz go w Ustawieniach, Pobieranie.
onboarding-sources-after = Otworzy się po konfiguracji
onboarding-sources-add = Dodaj repozytorium po konfiguracji
onboarding-fetch-title = Kto pobiera
onboarding-fetch-auto = Niech zajmie się tym AniBeam
onboarding-fetch-recommended = Zalecane
onboarding-fetch-auto-line = AniBeam ma własny qBittorrent i uruchamia go cicho w tle. Nic do instalowania, nic do wypełniania.
onboarding-fetch-existing = Już używam qBittorrent
onboarding-fetch-existing-line = AniBeam łączy się z qBittorrent, którego już używasz. Jego adres i login podasz zaraz w Ustawieniach.
onboarding-ready-library = Biblioteka
onboarding-ready-library-none = Jeszcze bez folderu. Znajdziesz to w Ustawieniach, Biblioteka, kiedy zechcesz.
onboarding-ready-library-series = { $count ->
    [one] { $count } folder
    [few] { $count } foldery
    [many] { $count } folderów
   *[other] { $count } folderu
}, seriale na razie: { $series }
onboarding-ready-library-empty = { $count ->
    [one] { $count } folder
    [few] { $count } foldery
    [many] { $count } folderów
   *[other] { $count } folderu
}, na razie pusto
onboarding-ready-library-scanning = { $count ->
    [one] { $count } folder
    [few] { $count } foldery
    [many] { $count } folderów
   *[other] { $count } folderu
}, skanowanie
onboarding-ready-lists = Twoja lista
onboarding-ready-lists-as = { $tracker }, jako { $user }
onboarding-ready-lists-none = Jeszcze nie połączono. Znajdziesz to w Ustawieniach, Konta, kiedy zechcesz.
onboarding-ready-downloads = Pobieranie
onboarding-ready-downloads-line = { $sources ->
    [0] Jeszcze bez źródeł
    [one] { $sources } źródło
    [few] { $sources } źródła
    [many] { $sources } źródeł
   *[other] { $sources } źródła
}, { $client ->
    [own] Twój własny qBittorrent.
   *[bundled] qBittorrent od AniBeam.
} { $next ->
    [ready] Wszystko gotowe.
    [opens] Następnie otworzą się Ustawienia, Pobieranie.
   *[both] Oba są w Ustawieniach, Pobieranie.
}
onboarding-ready-open-downloads = Otwórz Ustawienia, Pobieranie
onboarding-ready-set-up = Skonfiguruj teraz

## Rail.qml

rail-home = Start
rail-library = Biblioteka
rail-airing = Emisja
rail-feed = Aktualności
rail-stats = Statystyki
rail-show-info = Informacje
rail-downloads = Pobrane
rail-settings = Ustawienia
rail-downloads-complete =
    { $count ->
        [one] Pobrane: { $count } subskrypcja jest kompletna
        [few] Pobrane: { $count } subskrypcje są kompletne
        [many] Pobrane: { $count } subskrypcji jest kompletnych
       *[other] Pobrane: { $count } subskrypcji jest kompletnych
    }
rail-title-language-name = Język tytułów: { $language }. Kliknij, aby zmienić na następny
rail-title-language-english = Angielski
rail-title-language-tip = Język tytułów: romaji, angielski lub japoński
rail-whats-new-in = Co nowego w { $release }
rail-whats-new = Co nowego
rail-settings-note = Ustawienia: { $note }

## FixSheet.qml

fix-kind-episode = Odcinek
fix-kind-opening = Opening
fix-kind-ending = Ending
fix-kind-preview = Zapowiedź
fix-kind-special = Specjal
fix-kind-other = Inne
fix-reads-season-episode = Sezon { $season }, odcinek { $number }
# An extra with its number: "Opening 2"
fix-reads-extra = { $kind } { $index }
fix-apply-failed = Nie udało się zastosować poprawek.
fix-title = Popraw odczyt plików
fix-lede = { $title }. Wskaż, czym naprawdę jest każdy plik; każde kolejne skanowanie to zachowa.
fix-rescan = Przeskanuj folder
fix-rescan-started = Ponowne czytanie folderu
fix-filled-in = Uzupełniono
fix-fill-in = Uzupełnij
fix-column-file = Plik
fix-column-reads-as = Odczytany jako
fix-column-should-be = Powinien być
fix-fixed = Poprawiony
# The number field's placeholder for an episode, and for an extra that may have none
fix-number-placeholder = Nr
fix-index-placeholder = Brak
fix-keep = Zachowaj poprawkę
fix-leave-as-read = Zostaw ten plik tak, jak jest odczytany
fix-read-by-name = Znów odczytuj ten plik po nazwie
fix-report = Zgłoś problem na GitHub
fix-report-body = Otwiera wypełnione zgłoszenie do wysłania. Jest publiczne i wymienia te pliki.
fix-report-body-agent = Otwiera wypełnione zgłoszenie do wysłania. Jest publiczne i wymienia te pliki. Najpierw sprawdzi je { $agent }.
fix-close-unapplied =
    { $count ->
        [one] Zamknąć bez zastosowania { $count } poprawki?
        [few] Zamknąć bez zastosowania { $count } poprawek?
        [many] Zamknąć bez zastosowania { $count } poprawek?
       *[other] Zamknąć bez zastosowania { $count } poprawki?
    }
fix-number-invalid = Najpierw popraw liczbę zaznaczoną na czerwono
fix-close-anyway = Zamknij mimo to
fix-keep-editing = Edytuj dalej
fix-apply-count =
    { $count ->
        [one] Zastosuj { $count } poprawkę
        [few] Zastosuj { $count } poprawki
        [many] Zastosuj { $count } poprawek
       *[other] Zastosuj { $count } poprawki
    }

## MatchPanel.qml

match-matched-to = Dopasowano do { $title }
match-applied = Zastosowano dopasowanie
match-found-tvmaze = Znaleziono serial w TVmaze
match-found-anilist = Znaleziono w AniList
match-found-anilist-season = Znaleziono w AniList, sezon { $season }
match-found-mal = Znaleziono w MyAnimeList
# The sites a series is matched on, two at a time: "AniList and MyAnimeList"
match-sites-and = { $first } i { $next }
match-matched-on = Dopasowano w { $sites }
match-provider-silent = { $provider } nie odpowiedział
# The heading in a walk through several series, with "2 of 5"
match-title-walk = Dopasuj, { $walk }
match-title = Dopasuj
match-skip = Pomiń
match-refresh-info = Odśwież info
match-refreshing = Odświeżanie: { $title }
match-clear-confirm = Wyczyść: usuwa powiązanie i ustawioną liczbę odcinków
match-clear = Usuń dopasowanie
match-cleared = Usunięto dopasowanie
match-search-placeholder = Szukaj w { $provider }
match-link-tvmaze = Wklej link do TVmaze lub IMDb
match-link-anilist = Wklej link do AniList lub MyAnimeList
match-look-up = Znajdź
match-searching = Szukanie w { $provider }
match-type-more = Wpisz co najmniej dwie litery, aby przeszukać { $provider }.
match-no-results = Brak tytułów w { $provider }.

## WatchingPage.qml

# A card's progress code, the number padded to two digits: "Ep 03"
watching-episode-code = Odc. { $number }
watching-countdown-days = { $days } d { $hours } h
watching-countdown-hours = { $hours } h { $minutes } min
watching-countdown-minutes = { $minutes } min
watching-on-disk = Na dysku
watching-on-anilist = Na AniList
watching-ready-to-start = Gotowe do startu
watching-not-in-library = Nie ma w bibliotece
watching-get-to-start = Pobierz odcinki, aby zacząć
# $when is a relative time such as "3d ago"
watching-anilist-updated = AniList zaktualizowano { $when }
watching-airing = Emitowane
# $wait is a short countdown such as "2d 4h"
watching-next-in = Następny: { $title } za { $wait }
watching-episodes-waiting = Czekające odcinki
watching-series-behind =
    { $count ->
        [one] { $count } serial do nadrobienia
        [few] { $count } seriale do nadrobienia
        [many] { $count } seriali do nadrobienia
       *[other] { $count } seriali do nadrobienia
    }
watching-up-to-date = Na bieżąco
watching-anilist-only = Tylko na AniList
watching-planning-summary-off = Co planujesz obejrzeć na AniList.
watching-planning-summary =
    { $count ->
        [one] { $count } serial zaplanowany, na dysku: { $here }.
        [few] { $count } seriale zaplanowane, na dysku: { $here }.
        [many] { $count } seriali zaplanowanych, na dysku: { $here }.
       *[other] { $count } seriali zaplanowanych, na dysku: { $here }.
    }
watching-summary-off = Co oglądasz lub powtarzasz na AniList.
watching-summary-waiting =
    { $count ->
        [one] { $count } serial ma czekające odcinki.
        [few] { $count } seriale mają czekające odcinki.
        [many] { $count } seriali ma czekające odcinki.
       *[other] { $count } seriali ma czekające odcinki.
    }
watching-caught-up = Wszystko nadrobione.
watching-start-watching = Zacznij oglądać
watching-play-next = Odtwórz następny
watching-open-on-anilist = Otwórz w AniList
watching-refreshing = Odświeżanie
watching-connect-title = AniList nie jest połączony
watching-connect-body = Połącz go, a ta strona pokaże seriale, które oglądasz lub powtarzasz na AniList, z Odtwórz przy tych, które masz, i odliczaniem przy emitowanych.
watching-connect-waiting = Czekam na przeglądarkę
watching-connect-anilist = Połącz AniList
watching-load-failed-title = Lista Oglądam się nie wczytała
watching-load-failed-body = Brak połączenia z AniList. Lista wróci przy następnym odświeżeniu.
watching-planning-failed-title = Lista Planuję się nie wczytała
watching-planning-failed-body = Nie udało się odczytać jej pamięci podręcznej. Spróbuj ponownie albo wróci przy następnym odświeżeniu.
watching-planning-empty-title = Nic na liście Planuję
watching-planning-empty-body = Planuję obejrzeć w Teraz w emisji albo Planuję w menu listy serialu dodaje serial tutaj.
watching-airing-now = Teraz w emisji
watching-empty-title = Nic na liście Oglądam
watching-empty-body = Oznacz serial jako Oglądam na AniList, a pojawi się tutaj.
watching-open-anilist = Otwórz AniList

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] Ukryto { $count } serial
        [few] Ukryto { $count } seriale
        [many] Ukryto { $count } seriali
       *[other] Ukryto { $count } seriali
    }
library-hidden-sync-stops =
    { $count ->
        [one] Ukryto { $count } serial, a jego synchronizacja z serwisem jest wstrzymana
        [few] Ukryto { $count } seriale, a ich synchronizacja z serwisem jest wstrzymana
        [many] Ukryto { $count } seriali, a ich synchronizacja z serwisem jest wstrzymana
       *[other] Ukryto { $count } seriali, a ich synchronizacja z serwisem jest wstrzymana
    }
library-unhidden =
    { $count ->
        [one] Przywrócono { $count } serial, znów się synchronizuje
        [few] Przywrócono { $count } seriale, znów się synchronizują
        [many] Przywrócono { $count } seriali, znów się synchronizują
       *[other] Przywrócono { $count } seriali, znów się synchronizują
    }
library-undo = Cofnij
library-clearing-score =
    { $count ->
        [one] Usuwanie oceny { $count } serialu
        [few] Usuwanie ocen { $count } seriali
        [many] Usuwanie ocen { $count } seriali
       *[other] Usuwanie ocen { $count } seriali
    }
library-rating =
    { $count ->
        [one] Ocenianie { $count } serialu na { $score }
        [few] Ocenianie { $count } seriali na { $score }
        [many] Ocenianie { $count } seriali na { $score }
       *[other] Ocenianie { $count } seriali na { $score }
    }
# $list is the list's name, such as Watching
library-moving =
    { $count ->
        [one] Przenoszenie { $count } serialu do: { $list }
        [few] Przenoszenie { $count } seriali do: { $list }
        [many] Przenoszenie { $count } seriali do: { $list }
       *[other] Przenoszenie { $count } seriali do: { $list }
    }
library-rescanning =
    { $count ->
        [one] Ponowne skanowanie { $count } serialu
        [few] Ponowne skanowanie { $count } seriali
        [many] Ponowne skanowanie { $count } seriali
       *[other] Ponowne skanowanie { $count } seriali
    }
library-finding-matches =
    { $count ->
        [one] Szukanie dopasowania dla { $count } serialu
        [few] Szukanie dopasowania dla { $count } seriali
        [many] Szukanie dopasowania dla { $count } seriali
       *[other] Szukanie dopasowania dla { $count } seriali
    }
library-sort-last-viewed = Ostatnio oglądane
library-sort-downloaded = Ostatnio pobrane
library-sort-title = Tytuł
library-sort-progress = Postęp
library-sort-average-score = Średnia ocena
library-sort-my-score = Moja ocena
library-dir-a-to-z = A do Z
library-dir-z-to-a = Z do A
library-dir-oldest = Od najstarszych
library-dir-newest = Od najnowszych
library-dir-least-watched = Od najmniej obejrzanych
library-dir-most-watched = Od najbardziej obejrzanych
library-dir-lowest = Od najniższej
library-dir-highest = Od najwyższej
library-group-none = Brak
library-group-season = Sezon
library-group-type = Typ
library-group-genre = Gatunek
library-group-watch-state = Stan oglądania
library-scan-started = Skanowanie ruszyło
library-tab-all = Wszystko
library-tab-series = Seriale
library-tab-movies = Filmy
library-tab-watched = Obejrzane
library-tab-hidden = Ukryte
library-count-movies =
    { $count ->
        [one] { $count } film
        [few] { $count } filmy
        [many] { $count } filmów
       *[other] { $count } filmu
    }
library-count-watched = Obejrzane: { $count }
library-count-hidden = Ukryte: { $count }
library-count-series =
    { $count ->
        [one] { $count } serial
        [few] { $count } seriale
        [many] { $count } seriali
       *[other] { $count } serialu
    }
library-title = Biblioteka
library-all-genres = Wszystkie gatunki
library-search-placeholder = Szukaj tytułów lub folderów
library-sort = Sortuj
library-group = Grupuj
library-franchises-together = Franczyzy razem
library-watched-shows = Obejrzane seriale
library-load-failed-title = Biblioteka się nie wczytała
library-no-matches = Brak wyników dla „{ $query }”
# $what is the tab: series, films, watched, hidden or shows
library-none-in-genre =
    { $what ->
        [series] Brak seriali z gatunku { $genre }
        [films] Brak filmów z gatunku { $genre }
        [watched] Brak obejrzanych seriali z gatunku { $genre }
        [hidden] Brak ukrytych seriali z gatunku { $genre }
       *[shows] Brak seriali z gatunku { $genre }
    }
library-none-here =
    { $what ->
        [series] Brak seriali
        [films] Brak filmów
        [watched] Brak obejrzanych seriali
        [hidden] Brak ukrytych seriali
       *[shows] Brak seriali
    }
library-load-failed-body = AniBeam nie mógł teraz odczytać biblioteki. Spróbuj za chwilę.
library-no-matches-body = Sprawdź pisownię albo przeszukaj całą bibliotekę w zakładce Wszystko.
library-none-in-genre-body = Nic w tym widoku nie ma tego gatunku. Wszystkie gatunki pokazują resztę.
library-none-in-tab-body = Twoja biblioteka jeszcze ich nie ma. Wszystko pokazuje całą jej zawartość.
library-all-filtered-body = Wszystko w bibliotece jest odfiltrowane z tego widoku.
library-clear-search = Wyczyść wyszukiwanie
library-show-all = Pokaż wszystko
library-picked = Wybrano: { $count }
library-pick-all = Wszystkie
library-pick-all-tip = Wybierz każdą widoczną kartę (Ctrl+A)
library-unhide = Pokaż
library-hide = Ukryj
library-rate = Oceń
library-rate-tip = Jedna ocena dla każdego wybranego, w każdym połączonym serwisie
library-needs-tracker = Wymaga połączonego serwisu i dopasowanego serialu
library-move-to-list = Przenieś na listę
library-rescan = Przeskanuj
library-find-match = Znajdź dopasowanie
library-find-match-tip = Szukaj każdego wybranego bez dopasowania
library-all-matched = Każdy wybrany ma dopasowanie
library-empty-scanned-title = W folderach jeszcze nic nie znaleziono
library-empty-title = Biblioteka jest pusta
library-empty-scanned-body = AniBeam nie znalazł seriali ani filmów w obserwowanych folderach. Przeskanuj je ponownie albo sprawdź foldery w Ustawieniach.
library-empty-body = Dodaj folder z anime, a biblioteka wypełni się sama, albo zaimportuj eksport AniBeam.
library-scan-all = Skanuj wszystko
library-add-folder = Dodaj folder
library-settings = Ustawienia biblioteki
library-import = Importuj
library-import-title = Importuj eksport AniBeam
library-import-filter-export = Eksport AniBeam (*.json)
library-import-filter-all = Wszystkie pliki (*)
library-import-started = Import ruszył

## MetadataPage.qml

metadata-title = Informacje
metadata-filter-all = Wszystko { $count }
metadata-filter-series = Seriale { $count }
metadata-filter-films = Filmy { $count }
metadata-filter-missing = Brakujące { $count }
metadata-scan-started = Skanowanie ruszyło
# Complete's own sort
metadata-sort-added = Dodane
metadata-sort-season = Sezon
metadata-sort-a-to-z = A do Z
metadata-section-unmatched = Nierozpoznane
metadata-section-missing = Nie znaleziono folderu
metadata-section-incomplete = Brakujące odcinki
metadata-section-empty = Nie pobrano
metadata-section-airing = Emitowane
metadata-section-complete = Kompletne
metadata-no-info = Jeszcze brak informacji
# A tile's state line, short: "5 of 7 aired", "9 of 12", "12 of 12 +2"; $expected may be "?"
metadata-state-aired = { $have } z { $aired } wyemit.
metadata-state-of = { $have } z { $expected }
metadata-state-of-extra = { $have } z { $expected } +{ $extra }
metadata-detail-none-aired = Żadnego z { $aired } wyemitowanych odcinków nie ma na dysku
metadata-detail-aired = { $have } z { $aired } wyemitowanych na dysku
metadata-detail-aired-missing = { $have } z { $aired } wyemitowanych na dysku, brakuje: { $missing }
metadata-detail-none = Żadnego z { $expected } odcinków nie ma na dysku
metadata-detail-missing = { $have } z { $expected } na dysku, brakuje: { $missing }
# The match panel's place in a walk through the series that need a match: "2 of 5"
metadata-walk-position = { $at } z { $total }
metadata-walk-done = To był ostatni
metadata-walk-left =
    { $count ->
        [one] Jeden został w Wymaga dopasowania
        [few] { $count } zostały w Wymaga dopasowania
        [many] { $count } zostało w Wymaga dopasowania
       *[other] { $count } zostało w Wymaga dopasowania
    }
metadata-countdown-days = { $days } d { $hours } h
metadata-countdown-hours = { $hours } h { $minutes } min
metadata-countdown-minutes = { $minutes } min
# $wait is a short countdown such as "2d 4h"
metadata-next-in = Następny za { $wait }
metadata-files-of = { $have } z { $expected }
metadata-files-on-disk = Pliki na dysku
metadata-forgot = Usunięto { $title }
metadata-refreshed-failed = Odświeżono: { $refreshed }, nie udało się: { $failed }
metadata-refreshed =
    { $count ->
        [one] Odświeżono { $count } serial
        [few] Odświeżono { $count } seriale
        [many] Odświeżono { $count } seriali
       *[other] Odświeżono { $count } seriali
    }
metadata-found =
    { $count ->
        [one] Znaleziono { $count } serial
        [few] Znaleziono { $count } seriale
        [many] Znaleziono { $count } seriali
       *[other] Znaleziono { $count } seriali
    }
metadata-found-some-left =
    { $count ->
        [one] Znaleziono { $count } serial, wciąż nierozpoznane: { $unmatched }
        [few] Znaleziono { $count } seriale, wciąż nierozpoznane: { $unmatched }
        [many] Znaleziono { $count } seriali, wciąż nierozpoznane: { $unmatched }
       *[other] Znaleziono { $count } seriali, wciąż nierozpoznane: { $unmatched }
    }
metadata-found-nothing = Nie znaleziono nic nowego, wciąż nierozpoznane: { $unmatched }
metadata-all-recognised = Każdy serial jest rozpoznany
metadata-count =
    { $count ->
        [one] { $count } serial
        [few] { $count } seriale
        [many] { $count } seriali
       *[other] { $count } serialu
    }
metadata-search-placeholder = Filtruj tytuły
metadata-find-rest = Znajdź resztę
metadata-find-rest-started = Wyszukiwanie folderów, których AniBeam jeszcze nie zna
metadata-find-rest-tip = Wyszukuje każdy folder, którego AniBeam jeszcze nie zna, w każdym serwisie z informacjami, w kolejności z Ustawień
metadata-refresh-all = Odśwież wszystko
metadata-refresh-all-started = Aktualizowanie informacji o każdym serialu
metadata-refresh-all-tip = Pobiera ponownie informacje o każdym rozpoznanym serialu
metadata-crawling = Szukanie powiązań franczyzy
metadata-crawl-progress = { $done } z { $total }
metadata-load-failed-title = Informacje się nie wczytały
metadata-empty-scanned-title = W folderach jeszcze nic nie znaleziono
metadata-no-matches = Brak wyników dla „{ $query }”
# $filter is the filter's code from the core: Series, Movies or MissingFiles
metadata-nothing-under =
    { $filter ->
        [Series] Nic w Seriale
        [Movies] Nic w Filmy
        [MissingFiles] Nic w Brakujące
       *[other] Nic w { $filter }
    }
metadata-load-failed-body = AniBeam nie mógł teraz odczytać biblioteki. Spróbuj za chwilę.
metadata-empty-scanned-body = AniBeam nie znalazł seriali w obserwowanych folderach. Przeskanuj je ponownie albo sprawdź foldery w Ustawieniach.
metadata-empty-body = Biblioteka jest pusta. Dodaj folder z anime, a AniBeam go przeskanuje.
metadata-no-matches-body = Żaden tytuł w tym widoku nie zawiera tych słów.
metadata-nothing-under-body = Żaden z Twoich seriali nie pasuje teraz do tego filtra. Wszystko pokazuje resztę.
metadata-scan-all = Skanuj wszystko
metadata-add-folder = Dodaj folder
metadata-clear-search = Wyczyść wyszukiwanie
metadata-show-all = Pokaż wszystko
metadata-refreshing = Odświeżanie: { $title }
metadata-refresh-tip = Pobierz ponownie informacje o tym serialu
metadata-forget-armed = Usuń
metadata-forget-confirm-tip = Kliknij ponownie, aby usunąć ten serial i jego historię oglądania z AniBeam
metadata-forget-tip = Folderu nie ma już na dysku: usuń ten serial i jego historię z AniBeam

## AiringPage.qml

airing-title = Teraz w emisji
airing-sort-popular = Popularność
airing-sort-score = Ocena
airing-sort-trending = Na czasie
airing-sort-favourites = Ulubione
airing-sort-soonest = Następny odcinek
airing-sort-aired = Wyemitowane odcinki
airing-sort-episodes = Liczba odcinków
airing-sort-start = Data rozpoczęcia
airing-sort-duration = Długość odcinka
airing-sort-title = Tytuł
airing-dir-most-popular = Od najpopularniejszych
airing-dir-least-popular = Od najmniej popularnych
airing-dir-lowest = Od najniższej
airing-dir-highest = Od najwyższej
airing-dir-least-trending = Od najmniej na czasie
airing-dir-most-trending = Od najbardziej na czasie
airing-dir-fewest = Od najmniejszej
airing-dir-most = Od największej
airing-dir-soonest = Od najbliższych
airing-dir-latest = Od najpóźniejszych
airing-dir-oldest = Od najstarszych
airing-dir-newest = Od najnowszych
airing-dir-shortest = Od najkrótszych
airing-dir-longest = Od najdłuższych
airing-dir-a-to-z = A do Z
airing-dir-z-to-a = Z do A
# A season and its year; $season is the code Winter, Spring, Summer or Fall
airing-season =
    { $season ->
        [Winter] Zima { $year }
        [Spring] Wiosna { $year }
        [Summer] Lato { $year }
        [Fall] Jesień { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = { $minutes } min temu
airing-ago-hours = { $hours } godz. temu
airing-ago-days = { $days } d temu
# $episode is the episode's name, such as "Episode 7"; $ago a time such as "2h ago"
airing-episode-out = { $episode } wyszedł { $ago }
airing-no-date = Jeszcze bez daty
airing-in-library = W bibliotece
airing-out-today =
    { $count ->
        [one] { $count } odcinek wyszedł dziś
        [few] { $count } odcinki wyszły dziś
        [many] { $count } odcinków wyszło dziś
       *[other] { $count } odcinka wyszło dziś
    }
airing-dismissed-one = { $title } już się tu nie pojawi
airing-dismissed =
    { $count ->
        [one] { $count } serial już się tu nie pojawi
        [few] { $count } seriale już się tu nie pojawią
        [many] { $count } seriali już się tu nie pojawi
       *[other] { $count } seriali już się tu nie pojawi
    }
airing-planned-already-one = { $title } jest już na Twojej liście
airing-planned-already-all = Wszystkie są już na Twojej liście
airing-planned =
    { $count ->
        [one] { $count } serial trafia na listę Planuję
        [few] { $count } seriale trafiają na listę Planuję
        [many] { $count } seriali trafia na listę Planuję
       *[other] { $count } seriali trafia na listę Planuję
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } serial trafia na listę Planuję, już na niej były: { $already }
        [few] { $count } seriale trafiają na listę Planuję, już na niej były: { $already }
        [many] { $count } seriali trafia na listę Planuję, już na niej były: { $already }
       *[other] { $count } seriali trafia na listę Planuję, już na niej były: { $already }
    }
airing-play-next = Odtwórz następny
airing-plan-to-watch = Planuję obejrzeć
airing-get-more-episodes = Pobierz więcej odcinków
airing-get-episodes = Pobierz odcinki
airing-open-on-anilist = Otwórz w AniList
airing-not-interested = Nie interesuje
airing-not-interested-count = Nie interesuje: { $count }
airing-not-interested-tip = Seriale oznaczone jako nieinteresujące, w Ustawieniach
airing-refreshing = Odświeżanie
airing-pick-season = Wybierz inny sezon
airing-filter-all = Wszystko
airing-filter-lists = Na moich listach
airing-filter-library = W mojej bibliotece
airing-hide-lists = Ukryj moje listy
airing-hide-lists-tip = Pomiń seriale z Twoich list w AniList lub MAL
airing-hide-library = Ukryj moją bibliotekę
airing-hide-library-tip = Pomiń seriale, których odcinki masz
airing-out-now = Już wyszło
airing-out-last-day =
    { $count ->
        [one] { $count } odcinek w ostatnich 24 godzinach
        [few] { $count } odcinki w ostatnich 24 godzinach
        [many] { $count } odcinków w ostatnich 24 godzinach
       *[other] { $count } odcinka w ostatnich 24 godzinach
    }
airing-out-fewer = Mniej
airing-out-all = Wszystkie { $count }
airing-sort = Sortuj
airing-load-failed-title = Brak połączenia z AniList
airing-load-failed-body = Emitowane seriale pochodzą z AniList. Wrócą przy następnym odświeżeniu.
airing-reading-title = Czytanie, co jest w emisji
airing-nothing-now = Teraz nic nie jest emitowane
airing-nothing-for = Nic w emisji w sezonie { $season }
airing-nothing-read = Jeszcze nic nie odczytano
airing-reading-body = Każdy emitowany serial, z AniList. Za pierwszym razem trwa to kilka sekund.
airing-nothing-body = AniList nie ma jeszcze seriali na ten sezon. Odśwież odczyta go ponownie, możesz też wybrać inny sezon.
airing-nothing-read-body = Odśwież odczytuje z AniList każdy emitowany serial.
airing-hides-empty-title = Po ukryciu nic tu nie zostaje
airing-lists-empty-title = Nic tutaj nie jest na Twoich listach
airing-library-empty-title = Nic tutaj nie jest w Twojej bibliotece
airing-hides-empty-body = Każdy serial tutaj jest na Twoich listach lub w bibliotece, a te ukryłeś.
airing-lists-empty-body = Planuję obejrzeć na karcie dodaje serial do Twojej listy.
airing-library-empty-body = Pobierz odcinki na karcie pobiera serial do biblioteki.
airing-show-everything = Pokaż wszystko
airing-picked = Wybrano: { $count }
airing-undo = Cofnij

## EpisodeRow.qml

episode-row-date-time-format = ddd d MMM, HH:mm
episode-row-date-upcoming-format = ddd d MMM
episode-row-date-format = d MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Kliknij, aby oznaczyć jako nieobejrzane od tego miejsca
episode-row-mark-watched-tip = { $state }. Kliknij, aby oznaczyć jako obejrzane do tego miejsca
episode-row-next-up = Następny
episode-row-versions =
    { $count ->
        [one] { $count } wersja
        [few] { $count } wersje
        [many] { $count } wersji
       *[other] { $count } wersji
    }
episode-row-versions-tip =
    { $count ->
        [one] Ten sam odcinek z { $count } wydania. Kliknij, aby wybrać, które ma się odtworzyć
        [few] Ten sam odcinek z { $count } wydań. Kliknij, aby wybrać, które ma się odtworzyć
        [many] Ten sam odcinek z { $count } wydań. Kliknij, aby wybrać, które ma się odtworzyć
       *[other] Ten sam odcinek z { $count } wydania. Kliknij, aby wybrać, które ma się odtworzyć
    }
episode-row-downloading = Pobieranie { $percent }%
episode-row-get = Pobierz odcinek
episode-row-predicted = Przewidywany
episode-row-extra = Dodatek
episode-row-resume = Wznów od { $time }

## EpisodeCubes.qml

episode-cubes-one = Jeden odcinek
episode-cubes-range =
    { $count ->
        [one] { $count } odcinek, od { $first } do { $last }
        [few] { $count } odcinki, od { $first } do { $last }
        [many] { $count } odcinków, od { $first } do { $last }
       *[other] { $count } odcinka, od { $first } do { $last }
    }
# $state is a cube state as it reads mid-sentence, the .inline form of a tokens-cube-* message.
episode-cubes-tip = Odcinek { $number }: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } dodatek
        [few] { $count } dodatki
        [many] { $count } dodatków
       *[other] { $count } dodatku
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } niedopasowany plik
        [few] { $count } niedopasowane pliki
        [many] { $count } niedopasowanych plików
       *[other] { $count } niedopasowanego pliku
    }
episodes-pane-on-disk = Na dysku
episodes-pane-not-on-disk = Nie ma na dysku
episodes-pane-season = S{ $season }
episodes-pane-get-tip = Pobierz odcinek { $number }
episodes-pane-none-on-disk = Brak odcinków na dysku
episodes-pane-change-match = Zmień dopasowanie
episodes-pane-refreshing = Odświeżanie: { $title }

## ShowDetail.qml

show-detail-not-in-library = Tego folderu nie ma jeszcze w bibliotece. Gdy skanowanie go znajdzie, pojawią się tu jego plakat i odcinki.
show-detail-film-on-disk = Film jest na dysku.
show-detail-film-not-on-disk = Filmu nie ma jeszcze na dysku.
show-detail-complete-line = Emisja zakończona i wszystkie odcinki ({ $total }) są tutaj. Nic więcej nie przyjdzie, więc subskrypcja nie ma już nic do zrobienia.
show-detail-finished = Emisja zakończona.
show-detail-airing = W emisji.
show-detail-have-of-total = Tutaj: { $have } z { $total } odcinków.
show-detail-have =
    { $count ->
        [one] Tutaj: { $count } odcinek.
        [few] Tutaj: { $count } odcinki.
        [many] Tutaj: { $count } odcinków.
       *[other] Tutaj: { $count } odcinka.
    }
# $span is "Episode 7" or "Episodes 7 to 9"; $count is 1 for one episode, more for several
show-detail-next-predicted =
    { $count ->
        [one] { $span }: przewidywany za { $wait }.
        [few] { $span }: przewidywane za { $wait }.
        [many] { $span }: przewidywane za { $wait }.
       *[other] { $span }: przewidywane za { $wait }.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span }: emisja za { $wait }.
        [few] { $span }: emisja za { $wait }.
        [many] { $span }: emisja za { $wait }.
       *[other] { $span }: emisja za { $wait }.
    }
# $list is the episode numbers, joined with commas
show-detail-missing =
    { $count ->
        [one] Brak odcinka { $list }.
        [few] Brak odcinków { $list }.
        [many] Brak odcinków { $list }.
       *[other] Brak odcinków { $list }.
    }
show-detail-missing-more = Brak odcinków { $list } i jeszcze { $more }.
show-detail-downloaded = Pobrano
show-detail-downloading = Pobieranie { $percent }%
show-detail-taken-adding = Wzięte przez regułę, dodawanie
show-detail-taken = Wzięte przez regułę
show-detail-left-out = Pominięte przez regułę
show-detail-subscribed-reading = Zasubskrybowano. qBittorrent czyta kanał.
show-detail-taken-moving = Reguła bierze { $taken } z { $releases ->
        [one] { $releases } wydania
        [few] { $releases } wydań
        [many] { $releases } wydań
       *[other] { $releases } wydania
    } w kanale; pobierane poniżej: { $moving }.
show-detail-taken-held = Reguła bierze { $taken } z { $releases ->
        [one] { $releases } wydania
        [few] { $releases } wydań
        [many] { $releases } wydań
       *[other] { $releases } wydania
    } w kanale; torrenty w kliencie: { $torrents }.
show-detail-taken-adding-line = Reguła bierze { $taken } z { $releases ->
        [one] { $releases } wydania
        [few] { $releases } wydań
        [many] { $releases } wydań
       *[other] { $releases } wydania
    } w kanale; qBittorrent { $taken ->
        [one] je dodaje.
       *[other] je dodaje.
    }
show-detail-feed-holds-nothing = Kanał jeszcze nic nie zawiera. Pasujące wydania pobiorą się, gdy tylko się w nim pojawią.
show-detail-nothing-fits = Nic w kanale nie pasuje jeszcze do reguły. Pasujące wydania pobiorą się, gdy tylko się pojawią.
show-detail-complete = Kompletne
show-detail-open-in-library = Otwórz w bibliotece
show-detail-feed-finds-nothing = Ten kanał nic nie znajduje
show-detail-feed-empty-query = Pyta { $source } o „{ $query }”, co wraca puste, więc nic nowego nie dotrze do reguły. Krótsze wyszukiwanie zwykle coś zwraca; pominięte słowa mogą zostać w regule.
show-detail-feed-empty = Pyta { $source }, co wraca puste, więc nic nowego nie dotrze do reguły. Krótsze wyszukiwanie zwykle coś zwraca; pominięte słowa mogą zostać w regule.
show-detail-feed-empty-earlier = qBittorrent wciąż pokazuje to, co znalazła wcześniejsza wersja tego kanału.
show-detail-repairing = Naprawianie
show-detail-repair = Napraw
show-detail-unsubscribe = Anuluj subskrypcję
show-detail-remove-torrents =
    { $count ->
        [one] Usuń torrent
        [few] Usuń { $count } torrenty
        [many] Usuń { $count } torrentów
       *[other] Usuń { $count } torrenta
    }
show-detail-searches = Wyszukuje
show-detail-quoted = „{ $words }”
show-detail-whole-feed = cały kanał
show-detail-from = Z
show-detail-rule = Reguła
show-detail-rule-unfollowable = Nie może pobierać nowych odcinków: źródło odpowiada na wyszukiwania, ale nie ma kanału
show-detail-rule-active = Aktywna
show-detail-keeps = Zostawia
show-detail-keeps-value = wydania z „{ $words }”
show-detail-drops = Odrzuca
# $words is the quoted words, joined with show-detail-or
show-detail-drops-value = wydania z { $words }
show-detail-or = { $first } lub { $second }
show-detail-last-took = Ostatnio wzięła
show-detail-saves-to = Zapisuje do
show-detail-folder = Folder
show-detail-in-the-feed = W kanale
show-detail-feed-reading = Czytanie kanału.
show-detail-feed-offers = Co kanał oferuje teraz, od najnowszych.
show-detail-feed-is-empty = Kanał jest pusty.
show-detail-read-again = Czytaj ponownie
show-detail-seeders =
    { $count ->
        [one] { $count } seeder
        [few] { $count } seederzy
        [many] { $count } seederów
       *[other] { $count } seedera
    }
show-detail-download = Pobierz
show-detail-torrents = Torrenty
show-detail-torrents-moving =
    { $count ->
        [one] Pobierane teraz: { $count }; paski wypełniają się na bieżąco.
        [few] Pobierane teraz: { $count }; paski wypełniają się na bieżąco.
        [many] Pobierane teraz: { $count }; paski wypełniają się na bieżąco.
       *[other] Pobierane teraz: { $count }; paski wypełniają się na bieżąco.
    }
show-detail-torrents-held = Co klient trzyma dla tego folderu.
show-detail-torrents-waiting = Czekam, aż qBittorrent doda to, co bierze reguła.
show-detail-torrents-none = Klient nie ma nic dla tego folderu.

## SeriesPage.qml

series-trackers-both = { $first } i { $second }
series-count-cleared = Wyczyszczono liczbę odcinków
series-count-set = Ustawiono liczbę odcinków: { $count }
series-held-one-undated = Odcinek { $first } jeszcze bez daty
series-held-range-undated = Odcinki { $first }-{ $last } jeszcze bez daty
# $when is a month and year (series-held-month-format) or a year alone
series-held-one-due = Odcinek { $first } spodziewany: { $when }
series-held-range-due = Odcinki { $first }-{ $last } spodziewane: { $when }
series-held-month-format = MMMM yyyy
series-fix-schedule = Odcinki i daty
series-fix-files = Pliki
series-fix-rescan-folder = Przeskanuj folder
series-fix-match = Dopasuj
series-find-match = Znajdź dopasowanie
series-rescan-show = Przeskanuj serial
series-open-in-show-info = Otwórz w Informacjach
# $kind is what the picks are: film, episode or file
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } film przeniesiono do kosza
            [few] { $count } filmy przeniesiono do kosza
            [many] { $count } filmów przeniesiono do kosza
           *[other] { $count } filmu przeniesiono do kosza
        }
        [episode] { $count ->
            [one] { $count } odcinek przeniesiono do kosza
            [few] { $count } odcinki przeniesiono do kosza
            [many] { $count } odcinków przeniesiono do kosza
           *[other] { $count } odcinka przeniesiono do kosza
        }
       *[file] { $count ->
            [one] { $count } plik przeniesiono do kosza
            [few] { $count } pliki przeniesiono do kosza
            [many] { $count } plików przeniesiono do kosza
           *[other] { $count } pliku przeniesiono do kosza
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] Wybrano { $count } film.
            [few] Wybrano { $count } filmy.
            [many] Wybrano { $count } filmów.
           *[other] Wybrano { $count } filmu.
        }
        [episode] { $count ->
            [one] Wybrano { $count } odcinek.
            [few] Wybrano { $count } odcinki.
            [many] Wybrano { $count } odcinków.
           *[other] Wybrano { $count } odcinka.
        }
       *[file] { $count ->
            [one] Wybrano { $count } plik.
            [few] Wybrano { $count } pliki.
            [many] Wybrano { $count } plików.
           *[other] Wybrano { $count } pliku.
        }
    }
series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] Przenieś { $count } film do kosza
            [few] Przenieś { $count } filmy do kosza
            [many] Przenieś { $count } filmów do kosza
           *[other] Przenieś { $count } filmu do kosza
        }
        [episode] { $count ->
            [one] Przenieś { $count } odcinek do kosza
            [few] Przenieś { $count } odcinki do kosza
            [many] Przenieś { $count } odcinków do kosza
           *[other] Przenieś { $count } odcinka do kosza
        }
       *[file] { $count ->
            [one] Przenieś { $count } plik do kosza
            [few] Przenieś { $count } pliki do kosza
            [many] Przenieś { $count } plików do kosza
           *[other] Przenieś { $count } pliku do kosza
        }
    }
series-delete-click = Kliknij pliki do usunięcia.
series-delete-where = Trafią do kosza, skąd możesz je odzyskać. To, co obejrzałeś, zostaje na liście.
series-delete-move = Przenieś do kosza
series-pick-none = Odznacz wszystko
series-pick-all = Zaznacz wszystko
series-rescan-started = Ponowne skanowanie ruszyło
series-rated = Oceniono na { $score }
series-rated-cleared = Usunięto ocenę
series-score-failed = Nie udało się ocenić
series-still-downloading = { $episode } wciąż się pobiera
series-format-series = Serial
series-tracker-refused = { $tracker } tego nie przyjął
series-tracker-refused-why = { $tracker } tego nie przyjął: { $why }
series-watched-up-to = Obejrzane do: { $episode }
series-unwatched-from = Nieobejrzane od: { $episode }
series-undo = Cofnij
series-play-downloading = Pobieranie odcinka { $number }
series-play-downloading-unnumbered = Pobieranie odcinka
series-play-continue = Kontynuuj
series-play-again = Obejrzyj ponownie
series-play-continue-episode = Kontynuuj odcinek { $number }
series-play-again-from = Obejrzyj ponownie od odcinka { $number }
series-play-episode = Odtwórz odcinek { $number }
series-page-not-tracked = Nieśledzony
series-not-tracked-on-disk =
    { $count ->
        [one] Nieśledzony, { $count } odcinek na dysku
        [few] Nieśledzony, { $count } odcinki na dysku
        [many] Nieśledzony, { $count } odcinków na dysku
       *[other] Nieśledzony, { $count } odcinka na dysku
    }
series-watched = Obejrzane: { $count }
series-watched-of = Obejrzano { $watched } z { $total }
series-watched-of-estimate = Obejrzano { $watched } z { $total }+
series-facts-episodes-estimate =
    { $count ->
        [one] { $count }+ odcinek
        [few] { $count }+ odcinki
        [many] { $count }+ odcinków
       *[other] { $count }+ odcinka
    }
series-facts-each = po { $runtime }
series-facts-on-disk = Na dysku: { $count }
series-next = Następny: { $episode }
series-episodes-downloading =
    { $count ->
        [one] { $count } odcinek się pobiera
        [few] { $count } odcinki się pobierają
        [many] { $count } odcinków się pobiera
       *[other] { $count } odcinka się pobiera
    }
series-all-watched = Wszystko obejrzane
series-no-poster = Brak plakatu
# $span is "Episode 7" or "Episodes 7 to 9"
series-next-season-span = S{ $season } { $span }
series-next-predicted = { $span } przewidywany na { $date }, za { $wait }
series-next-date-format = ddd d MMM
series-next-airs =
    { $count ->
        [one] { $span }: emisja za { $wait }
        [few] { $span }: emisja za { $wait }
        [many] { $span }: emisja za { $wait }
       *[other] { $span }: emisja za { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] Pobieranie { $count } odcinka
        [few] Pobieranie { $count } odcinków
        [many] Pobieranie { $count } odcinków
       *[other] Pobieranie { $count } odcinka
    }
series-count-season-ends = Sezon { $season } kończy się na odcinku { $number }
series-count-ends = Kończy się na odcinku { $number }
series-count-gave-way = Ustawiłeś liczbę odcinków na { $count }; to już nie obowiązuje
# $line is what the user set, the parts joined with " · "
series-count-set-by-you = { $line }, ustawione przez Ciebie
series-change = Zmień
series-get-episodes = Pobierz odcinki
series-your-score = Twoja ocena { $score }
series-rate = Oceń
series-page-average-score = Średnia ocena { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = Głosy: { $votes }
series-add-to-list = Dodaj do listy
series-page-rewatched =
    { $count ->
        [one] Obejrzane ponownie { $count } raz
        [few] Obejrzane ponownie { $count } razy
        [many] Obejrzane ponownie { $count } razy
       *[other] Obejrzane ponownie { $count } razy
    }
series-your-notes = Twoje notatki
series-add-notes = Dodaj notatki
# $sites is AniList, MyAnimeList, or both
series-notes-tip = Notatki do wpisu na Twojej liście, zapisane w { $sites }
series-unhide = Pokaż
series-hide = Ukryj
series-hidden-toast = { $title } jest tu ukryty, a AniBeam przestaje go synchronizować z AniList i MyAnimeList
series-unhidden-toast = { $title } wrócił i znów się synchronizuje
series-hide-tip-tracked = Ukryty na każdej stronie tutaj, a AniBeam przestaje go synchronizować z AniList i MyAnimeList
series-hide-tip = Ukryty na każdej liście
series-fixes = Poprawki
series-fixes-tip = Popraw pliki, liczbę odcinków, dopasowanie albo przeskanuj
series-quick-fix = Szybka poprawka
series-film = Film
series-episodes = Odcinki
series-extras = Openingi, endingi i więcej
series-extras-openings = Openingi
series-extras-endings = Endingi
series-extras-previews = Zapowiedzi i zwiastuny
series-extras-specials = Specjale
series-extras-other = Inne
# $kind is one of the series-extras- names above
series-extras-group = { $kind }, { $count }
series-extra-files = Dodatkowe pliki
series-extra-files-warning =
    { $count ->
        [one] Jeden plik wykracza poza { $total } oczekiwanych odcinków: zła nazwa, duplikat albo specjal. Zmień jego nazwę albo go usuń.
        [few] { $count } pliki wykraczają poza { $total } oczekiwanych odcinków: złe nazwy, duplikaty albo specjale. Zmień ich nazwy albo je usuń.
        [many] { $count } plików wykracza poza { $total } oczekiwanych odcinków: złe nazwy, duplikaty albo specjale. Zmień ich nazwy albo je usuń.
       *[other] { $count } pliku wykracza poza { $total } oczekiwanych odcinków: złe nazwy, duplikaty albo specjale. Zmień ich nazwy albo je usuń.
    }
series-synopsis = Opis
series-less = Mniej
series-more = Więcej
series-match = Dopasuj
series-not-matched = Jeszcze nie dopasowany, więc brak plakatu, opisu i śledzenia. Znajdź informacje o nim w AniList lub TVmaze.
series-naming-title = Nazwy, które pasują
series-naming-body = Folder to tytuł serialu w zapisie z AniList, z sezonem po nim, jeśli to nie pierwszy. Każdy plik zawiera numer odcinka.
series-fix-how-files-read = Popraw odczyt plików
series-characters = Postacie
series-related = Powiązane
series-related-filling = AniList wciąż uzupełnia dane.
series-tags = Tagi
series-hide-spoilers = Ukryj spoilery
series-spoilers = Spoilery
series-all-tags = Wszystkie tagi
series-tag = { $name }, { $rank }%
series-recommendations = Polecane
series-relation-owned = W Twojej bibliotece
series-relation-unreleased = Jeszcze niewydane
series-relation-on-anilist = Na AniList
series-score-saved = Zapisano ocenę
series-score-cleared = Usunięto ocenę

## GetShow.qml

get-show-the-client = klient torrentów
get-show-group-batch = paczka { $group }
get-show-run = { $first }-{ $last }
# $list is the episode numbers and runs (get-show-run), joined with commas
get-show-episodes =
    { $count ->
        [one] Odcinek { $list }
        [few] Odcinki { $list }
        [many] Odcinki { $list }
       *[other] Odcinki { $list }
    }
# The same, in the middle of a sentence
get-show-episodes-inline =
    { $count ->
        [one] odcinek { $list }
        [few] odcinki { $list }
        [many] odcinki { $list }
       *[other] odcinki { $list }
    }
get-show-sending-to = Wysyłanie do: { $client }
get-show-go =
    { $count ->
        [one] Pobierz { $count } odcinek
        [few] Pobierz { $count } odcinki
        [many] Pobierz { $count } odcinków
       *[other] Pobierz { $count } odcinka
    }
get-show-go-replace =
    { $count ->
        [one] Pobierz { $count } odcinek i zastąp
        [few] Pobierz { $count } odcinki i zastąp
        [many] Pobierz { $count } odcinków i zastąp
       *[other] Pobierz { $count } odcinka i zastąp
    }
get-show-go-subscribe =
    { $count ->
        [one] Pobierz { $count } odcinek i subskrybuj
        [few] Pobierz { $count } odcinki i subskrybuj
        [many] Pobierz { $count } odcinków i subskrybuj
       *[other] Pobierz { $count } odcinka i subskrybuj
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] Pobierz { $count } odcinek, zastąp i subskrybuj
        [few] Pobierz { $count } odcinki, zastąp i subskrybuj
        [many] Pobierz { $count } odcinków, zastąp i subskrybuj
       *[other] Pobierz { $count } odcinka, zastąp i subskrybuj
    }
get-show-subscribe = Subskrybuj nowe odcinki
get-show-nothing-new = Nic nowego do pobrania
get-show-pick-episode = Wybierz odcinek
# $episodes is get-show-episodes-inline
get-show-replace-off = Wyłączone. Wybrane { $episodes } są pomijane, bo biblioteka je ma. Włącz, aby je wymienić.
get-show-replace-on = Pliki, które masz dla: { $episodes }, trafią do kosza, gdy nowe się pobiorą.
# $episodes is get-show-episodes
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } właśnie się pobiera: to pobieranie zostanie anulowane, a to, co zapisało, trafi do kosza.
        [few] { $episodes } właśnie się pobierają: to pobieranie zostanie anulowane, a to, co zapisało, trafi do kosza.
        [many] { $episodes } właśnie się pobierają: to pobieranie zostanie anulowane, a to, co zapisało, trafi do kosza.
       *[other] { $episodes } właśnie się pobierają: to pobieranie zostanie anulowane, a to, co zapisało, trafi do kosza.
    }
get-show-kind-on-demand = Uruchamiane, gdy wyszukiwanie go potrzebuje
get-show-kind-rss-search = Wyszukiwanie RSS
get-show-kind-rss-feed = Kanał RSS
get-show-kind-json-search = Wyszukiwanie JSON
get-show-kind-indexer-search = Wyszukiwanie w indekserze
get-show-ready = Gotowe
get-show-running = Działa
get-show-not-running = Nie działa
get-show-not-running-start = Nie działa. Kliknij jego źródło powyżej, aby je uruchomić.
get-show-switched-off = Wyłączone
get-show-click-to-see = Kliknij, aby zobaczyć, co ma dla tego serialu
get-show-searching = Szukanie
get-show-searching-source = Szukanie w { $name }
get-show-searching-elapsed = { $label }, { $seconds } s
get-show-releases =
    { $count ->
        [one] { $count } wydanie
        [few] { $count } wydania
        [many] { $count } wydań
       *[other] { $count } wydania
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } wydanie, obejmuje { $cover } z { $total } odcinków
        [few] { $count } wydania, obejmują { $cover } z { $total } odcinków
        [many] { $count } wydań, obejmują { $cover } z { $total } odcinków
       *[other] { $count } wydania, obejmują { $cover } z { $total } odcinków
    }
get-show-on-your-list = Na Twojej liście
get-show-plan-to-watch = Planuję obejrzeć
get-show-on-your-list-tip = Jest na Twojej liście
get-show-plan-to-watch-tip = Dodaj do listy Planuję
get-show-open-series-tip = Strona serialu: szczegóły, odcinki i odtwarzanie
get-show-next-in = Odcinek { $number } za { $wait }
get-show-airing = Emitowany
get-show-finished = Zakończony
get-show-total-episodes = Odcinki: { $count }
get-show-downloading-episodes =
    { $count ->
        [one] Pobieranie { $count } odcinka
        [few] Pobieranie { $count } odcinków
        [many] Pobieranie { $count } odcinków
       *[other] Pobieranie { $count } odcinka
    }
get-show-downloading-torrents =
    { $count ->
        [one] Pobieranie { $count } torrenta
        [few] Pobieranie { $count } torrentów
        [many] Pobieranie { $count } torrentów
       *[other] Pobieranie { $count } torrenta
    }
get-show-in-library = W Twojej bibliotece
get-show-not-in-library = Nie ma w Twojej bibliotece
get-show-downloading = Pobieranie
get-show-stop-all = Zatrzymaj wszystko
get-show-start-all = Uruchom wszystko
get-show-stop-all-tip = Zatrzymaj każde pobieranie tego serialu; to, co już jest, zostaje
get-show-start-all-tip = Uruchom ponownie każde pobieranie tego serialu
get-show-cancel-all = Anuluj wszystko
get-show-cancel-all-tip = Anuluj każde pobieranie tego serialu i przenieś to, co zapisały, do kosza
get-show-sources = Dostępne źródła
get-show-check-again = Sprawdź ponownie
get-show-check-again-tip = Pyta każdego dostawcę uruchamianego na żądanie, czy działa. Niczego nie uruchamia.
get-show-search-source = Szukaj w { $name }
get-show-release-list = Każde wydanie jako lista
get-show-your-library = Twoja biblioteka
get-show-library-on-disk = { $have } z { $total } na dysku
get-show-library-downloading = Pobieranie: { $count }
get-show-library-nothing = Jeszcze nic
get-show-row-batch = { $group }, paczka
get-show-batch-counting = liczenie odcinków, jeden torrent
get-show-batch-unlisted = odcinki niewymienione, cały serial po tytule, jeden torrent
get-show-batch-episodes =
    { $count ->
        [one] { $count } odcinek, jeden torrent
        [few] { $count } odcinki, jeden torrent
        [many] { $count } odcinków, jeden torrent
       *[other] { $count } odcinka, jeden torrent
    }
get-show-nothing-new-picked = Nie wybrano nic nowego
get-show-nothing-picked = Nic nie wybrano
get-show-pick-provider = Wybierz dostawcę
# $groups is the release groups, joined with get-show-and
get-show-plan-line = { $episodes } od { $groups } z { $source }.
get-show-and = { $first } i { $second }
get-show-library-has =
    { $count ->
        [one] Biblioteka ma: { $episodes }. Włącz poniżej Zastąp obecne odcinki, aby go wymienić.
        [few] Biblioteka ma: { $episodes }. Włącz poniżej Zastąp obecne odcinki, aby je wymienić.
        [many] Biblioteka ma: { $episodes }. Włącz poniżej Zastąp obecne odcinki, aby je wymienić.
       *[other] Biblioteka ma: { $episodes }. Włącz poniżej Zastąp obecne odcinki, aby je wymienić.
    }
get-show-together = razem
get-show-torrents-unit =
    { $count ->
        [one] torrent
        [few] torrenty
        [many] torrentów
       *[other] torrenta
    }
get-show-replace = Zastąp obecne odcinki
get-show-sequential = Pobieraj po kolei
get-show-sequential-on = Włączone: każdy nowy torrent pobiera części od początku, więc pierwsze odcinki kończą się najpierw i możesz oglądać, gdy reszta się pobiera. W sumie trochę wolniej.
get-show-sequential-off = Wyłączone: klient pobiera części w kolejności, w jakiej oferuje je rój.
get-show-rolling = Trzymaj kilka następnych w gotowości
get-show-rolling-on = Włączone: od razu pobiera się tyle odcinków: { $count }, a każdy obejrzany uruchamia następny.
get-show-rolling-on-delete = Włączone: od razu pobiera się tyle odcinków: { $count }, a każdy obejrzany uruchamia następny. Obejrzane odcinki znikają z dysku.
get-show-rolling-off = Wyłączone: każdy wybrany odcinek pobiera się od razu.
get-show-follow-off = Wyłączone: nowe odcinki pojawiają się tu jako braki i nic nie pobiera się samo.
get-show-follow-no-feed = { $source } nie ma kanału do obserwowania, więc nowe odcinki nie mogą się z niego pobierać same.
get-show-follow-pick-group = Najpierw wybierz grupę: subskrypcja obserwuje jedną grupę i jakość.
get-show-follow-batch-only = { $group } ma tu tylko paczkę, więc nie ma kanału pojedynczych odcinków do subskrypcji.
# $feed is the release group and its quality
get-show-follow-on = Włączone: kanał dla { $feed } z { $source }. Każdy nowy odcinek pobiera się, gdy się pojawi.
get-show-confirm =
    { $count ->
        [one] { $episodes } właśnie się pobiera. Zastąpienie anuluje to pobieranie i przeniesie to, co zapisało, do kosza.
        [few] { $episodes } właśnie się pobierają. Zastąpienie anuluje to pobieranie i przeniesie to, co zapisało, do kosza.
        [many] { $episodes } właśnie się pobierają. Zastąpienie anuluje to pobieranie i przeniesie to, co zapisało, do kosza.
       *[other] { $episodes } właśnie się pobierają. Zastąpienie anuluje to pobieranie i przeniesie to, co zapisało, do kosza.
    }
get-show-confirm-go = Anuluj i zastąp
get-show-state-error = Błąd
get-show-state-files-missing = Brak plików
get-show-state-stopped = Zatrzymane
get-show-state-waiting-peers = Czekam na peerów
get-show-state-metadata = Pobieranie metadanych
get-show-state-queued = W kolejce
get-show-state-checking = Sprawdzanie plików
get-show-state-allocating = Rezerwowanie miejsca
get-show-state-moving = Przenoszenie plików
get-show-state-waiting = Czeka
get-show-a-batch = Paczka
get-show-no-group = Brak grupy w nazwie
get-show-likely = prawdopodobnie { $source }
get-show-tracker = tracker { $tracker }
get-show-not-known = Jeszcze nieznane
get-show-bytes-of = { $done } z { $size }
get-show-speed-down = ↓ { $speed }/s
get-show-time-left = Zostało { $wait }
get-show-stopping = Zatrzymywanie
get-show-starting = Uruchamianie
get-show-stop = Zatrzymaj
get-show-start-tip = Wznów pobieranie
get-show-stop-tip = Zatrzymaj pobieranie; to, co już jest, zostaje
get-show-cancelling = Anulowanie
get-show-cancel-tip = Anuluj to pobieranie i przenieś to, co zapisało, do kosza
get-show-episode-order = Pobieraj w kolejności odcinków
get-show-episode-order-on = Włączone: części pobierają się od początku każdego pliku, więc pierwsze odcinki kończą się najpierw.
get-show-cube-skipped = Odcinek { $number }: pominięty
get-show-cube-done = Odcinek { $number }: gotowy
get-show-cube-percent = Odcinek { $number }: { $percent }%
get-show-fact-torrent = Torrent
get-show-fact-release = Wydanie
get-show-fact-from = Z
get-show-fact-saving-to = Zapis do
get-show-span-counting = Liczenie
get-show-span-unknown = { $runs }?
get-show-span-episodes = Odcinki { $runs }
get-show-cube-no-release = Odcinek { $number }: brak wydania od { $group }
get-show-seeders = Seederzy: { $count }
get-show-cube-on-disk = już na dysku, kliknij, aby zastąpić
get-show-cube-downloading = właśnie się pobiera, kliknij, aby zastąpić

## TorrentRow

torrent-row-state-downloading = Pobieranie
torrent-row-state-stalled = Utknęło
torrent-row-state-metadata = Pobieranie metadanych
torrent-row-state-queued = W kolejce
torrent-row-state-stopped = Zatrzymane
torrent-row-state-done = Gotowe
torrent-row-state-seeding = Wysyłanie
torrent-row-state-checking = Sprawdzanie
torrent-row-state-moving = Przenoszenie
torrent-row-state-error = Błąd
torrent-row-state-files-missing = Brak plików
torrent-row-state-making-room = Robienie miejsca
torrent-row-state-finding-peers = Szukam peerów
torrent-row-state-waiting = Czeka
torrent-row-done-of-size = { $done } z { $size }
torrent-row-speed-down = ↓ { $speed }/s
torrent-row-speed-up = ↑ { $speed }/s
torrent-row-time-left = Zostało { $time }
torrent-row-seeders =
    { $count ->
        [one] { $count } seeder
        [few] { $count } seederzy
        [many] { $count } seederów
       *[other] { $count } seedera
    }
torrent-row-rolling = kroczące, odcinki { $next }-{ $through } z { $total }
torrent-row-stop = Zatrzymaj

## SourceRow

source-row-unavailable-fix = Podłącz dysk, a potem Przeskanuj. Jeśli folder został przeniesiony, usuń go i dodaj nowe miejsce.
source-row-counts = Seriale: { $count }
source-row-counts-movies =
    { $movies ->
        [one] Seriale: { $count }, { $movies } folder z filmami
        [few] Seriale: { $count }, { $movies } foldery z filmami
        [many] Seriale: { $count }, { $movies } folderów z filmami
       *[other] Seriale: { $count }, { $movies } folderu z filmami
    }
source-row-unavailable = Niedostępny
    .tip = AniBeam nie ma dostępu do tego folderu. Jeśli jest na dysku, który nie jest podłączony lub zamontowany, podłącz go, a AniBeam znów go odczyta. Jeśli folder został przeniesiony, usuń go tutaj i dodaj nowe miejsce.
source-row-rescan = Przeskanuj
source-row-remove-question = Usunąć { $name }, seriale ({ $count }) i ich historię?

## StartSourceSheet

start-source-title = Uruchomić { $service } dla { $name }?
start-source-busy = Uruchamianie { $service }. { $name } włączy się, gdy odpowie; może to potrwać do minuty.
start-source-body = { $service } to usługa, która przeszukuje { $name } dla AniBeam. Uruchomienie włącza ją w tle na tym komputerze i działa dalej po zamknięciu AniBeam.
start-source-starting = Uruchamianie

## SearchProgress

search-progress-why-helper = AniBeam uruchamia pomocnika wyszukiwania; to potrwa sekundę lub dwie.
search-progress-why-starting = { $endpoint } działa przez pomocnika uruchamianego, gdy wyszukiwanie go potrzebuje. Pierwsze wyszukiwanie czeka na niego do { $seconds } s; kolejne już nie.
search-progress-why-reading = Dzielenie wydań na grupy i porównywanie ich z biblioteką.
search-progress-why-ondemand = Pomocnik czyta { $endpoint } od nowa przy każdym wyszukiwaniu, co zwykle trwa od dziesięciu do dwudziestu sekund.
search-progress-why-slow = { $endpoint } odpowiada teraz wolno.
search-progress-why-usual = Zwykle trwa to sekundę lub dwie.
search-progress-why-and-slow = { $why } Trwa to dłużej niż zwykle; anuluj, aby spróbować innego źródła.
search-progress-searching = Szukanie
search-progress-seconds = { $seconds } s
search-progress-try = Spróbuj { $endpoint }

## ReleaseTable

release-table-col-release = Wydanie
release-table-col-group = Grupa
release-table-col-res = Rozdz.
release-table-col-rip = Rip
release-table-col-ep = Odc.
release-table-col-size = Rozmiar
release-table-col-seeders = Seedy
release-table-col-leechers = Leechy
release-table-col-downloads = Pobrania
release-table-col-released = Wydano
release-table-tick-all = Zaznacz każde wydanie
release-table-untick-all = Odznacz każde wydanie
release-table-on-disk = Na dysku
release-table-other-on-disk = Inne wydanie na dysku
release-table-season-batch = Paczka sezonu { $season }
release-table-batch = Paczka
release-table-via = przez { $source }
release-table-season-episode = S{ $season } { $episode }
release-table-date-format = d MMMM yyyy, HH:mm
release-table-download = Pobierz
release-table-more = i jeszcze { $count }. Zawęź listę lub ją posortuj, aby zobaczyć inne najpierw.
release-table-pick = Wybierz to wydanie

## DownloadsPage

downloads-title = Pobrane
downloads-builtin-client = Wbudowany klient
downloads-start-done = { $name } działa i jest włączony
downloads-disk-film = Film
downloads-disk-watched = Obejrzane: { $size }
downloads-disk-all-watched = wszystko obejrzane
downloads-delete-watched-menu =
    { $count ->
        [one] Każdy obejrzany odcinek: { $count } odcinek, { $size }
        [few] Każdy obejrzany odcinek: { $count } odcinki, { $size }
        [many] Każdy obejrzany odcinek: { $count } odcinków, { $size }
       *[other] Każdy obejrzany odcinek: { $count } odcinka, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Każdy zakończony i obejrzany serial: { $count } serial, { $size }
        [few] Każdy zakończony i obejrzany serial: { $count } seriale, { $size }
        [many] Każdy zakończony i obejrzany serial: { $count } seriali, { $size }
       *[other] Każdy zakończony i obejrzany serial: { $count } serialu, { $size }
    }
downloads-titles-and = { $list } i { $last }
downloads-titles-more = { $list } i jeszcze { $count }
downloads-delete-finished-none = Żaden zakończony serial nie jest obejrzany do końca
downloads-delete-watched-none = Na dysku nie ma nic obejrzanego
downloads-delete-finished-heading =
    { $count ->
        [one] Usunąć { $count } zakończony serial, { $size }?
        [few] Usunąć { $count } zakończone seriale, { $size }?
        [many] Usunąć { $count } zakończonych seriali, { $size }?
       *[other] Usunąć { $count } zakończonego serialu, { $size }?
    }
downloads-delete-finished-base = { $titles }. Tylko seriale, które według dostawcy się zakończyły, z każdym odcinkiem obejrzanym; serial wciąż emitowany albo z szacowanym końcem zostaje. Każdy ich plik trafia do kosza, dodatki też. Ich torrenty znikają też z { $client }, więc można je pobrać ponownie.
downloads-delete-watched-heading =
    { $count ->
        [one] Usunąć { $count } obejrzany odcinek, { $size }?
        [few] Usunąć { $count } obejrzane odcinki, { $size }?
        [many] Usunąć { $count } obejrzanych odcinków, { $size }?
       *[other] Usunąć { $count } obejrzanego odcinka, { $size }?
    }
downloads-delete-watched-base = Każdy obejrzany odcinek, w każdym serialu. Nieobejrzane odcinki i dodatki zostają. Pliki i ich napisy trafiają do kosza, więc możesz je stamtąd odzyskać. Ich torrenty znikają też z { $client }, więc można je pobrać ponownie.
downloads-move-to-trash = Przenieś do kosza
downloads-remove-group-torrents =
    { $count ->
        [one] Usuń torrent
        [few] Usuń jego { $count } torrenty
        [many] Usuń jego { $count } torrentów
       *[other] Usuń jego { $count } torrenta
    }
downloads-run-span = { $from }-{ $to }
downloads-run-pair = { $first } i { $second }
downloads-gap-missing =
    { $count ->
        [one] Brak odcinka { $runs }
        [few] Brak odcinków { $runs }
        [many] Brak odcinków { $runs }
       *[other] Brak odcinków { $runs }
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Nie pobrano odcinka { $runs }
        [few] Nie pobrano odcinków { $runs }
        [many] Nie pobrano odcinków { $runs }
       *[other] Nie pobrano odcinków { $runs }
    }
downloads-gap-then-not-downloaded = nie pobrano { $runs }
downloads-trying-query = Próba: { $query }
downloads-client-settings = Ustawienia klienta torrentów
downloads-error-unknown = Coś poszło nie tak. Spróbuj ponownie.
downloads-search-all-off = Każde źródło jest wyłączone. Włącz jedno, aby szukać.
downloads-turn-on = Włącz { $name }
downloads-search-no-sources = Nie ma jeszcze źródeł do przeszukania. Dodaj listę źródeł w Ustawieniach, w sekcji Pobieranie.
downloads-search-none-started = Żadne źródło nie mogło rozpocząć wyszukiwania. Spróbuj za chwilę.
downloads-auto-resolution = Zawężono do { $resolution }, najostrzejszej dostępnej
downloads-search-one-failed = Wyszukiwanie w { $name } nie zadziałało. Spróbuj ponownie albo przeszukaj inne źródło.
downloads-search-all-failed = Żadne źródło nie mogło wyszukać. Spróbuj za chwilę.
downloads-narrowing-on = Zawężanie w { $name }
downloads-refine-zero = Nic by nie zostało, więc lista zostaje bez zmian.
downloads-refine-no-match = Żadne wydanie nie pasuje do tego słowa.
downloads-refine-empty = Wpisz słowo do zawężenia.
downloads-refine-no-change = To nic nie zmieni.
downloads-movie-tick-one = Zaznacz jedno wydanie, potem kliknij Pobierz jako film.
downloads-group-downloading = Pobieranie
downloads-group-downloading-line = W toku; szczegóły w Aktywności po prawej.
downloads-group-seeding = Wysyłanie
downloads-group-seeding-line = Pobrane i wciąż udostępniane.
downloads-group-done = Gotowe
downloads-group-done-line = Pobrane; klient ich nie udostępnia.
downloads-group-stopped = Zatrzymane
downloads-group-stopped-line = Zatrzymane przed końcem.
downloads-group-broken = Brak plików
downloads-group-broken-line = Klient nie znajduje ich plików: wskazują na folder, którego nie ma. Usunięcie torrentów porządkuje klienta.
downloads-sub-nofeed = Nie można obserwować
downloads-sub-nofeed-line = Ich źródło odpowiada na wyszukiwania, ale nie ma kanału, więc nowe odcinki nie przychodzą. Anuluj subskrypcję, a potem obserwuj serial w źródle z kanałem.
downloads-sub-complete = Kompletne
downloads-sub-complete-line = Każdy odcinek jest w bibliotece i nic więcej nie wyjdzie: te subskrypcje nie mają już nic do zrobienia.
downloads-sub-airing = W emisji
downloads-sub-airing-line = Nowe odcinki jeszcze przed nami, od najbliższych.
downloads-sub-incomplete = Niekompletne
downloads-sub-incomplete-line = Emisja zakończona, a odcinków brakuje, albo nie wiadomo, na jakim etapie jest serial.
downloads-sub-missing = Nie ma w bibliotece
downloads-sub-missing-line = Ich folder nie zawiera niczego, co znalazła biblioteka: nic jeszcze nie przyszło albo zapisuje tam, gdzie biblioteka nie zagląda.
downloads-torrents =
    { $count ->
        [one] { $count } torrent
        [few] { $count } torrenty
        [many] { $count } torrentów
       *[other] { $count } torrenta
    }
downloads-rule-film-on-disk = Film, na dysku
downloads-rule-all-here = Wszystkie tutaj: { $total }
downloads-rule-next =
    { $predicted ->
        [yes] Odc. { $episode } przewidywany za { $time }
       *[no] Odc. { $episode } za { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] Odc. { $episode }-{ $through } przewidywane za { $time }
       *[no] Odc. { $episode }-{ $through } za { $time }
    }
downloads-rule-have-of = Tutaj { $have } z { $total }
downloads-rule-have =
    { $count ->
        [one] Tutaj { $count } odcinek
        [few] Tutaj { $count } odcinki
        [many] Tutaj { $count } odcinków
       *[other] Tutaj { $count } odcinka
    }
downloads-tag-not-downloaded = Nie pobrano
downloads-tag-missing = Brakuje: { $count }
downloads-tag-n-not-downloaded = Nie pobrano: { $count }
downloads-tag-unknown = Nieznane
downloads-speed-down = ↓ { $speed }/s
downloads-time-left = Zostało { $time }
downloads-repair-trying = Próba krótszych wyszukiwań
downloads-sent-to-client = Wysłano { $title } do { $client }
downloads-files-torrents = to, co pobrały torrenty
downloads-files-torrents-remote = to, co pobrały torrenty, usunięte przez { $client }
downloads-files-folder-and = { $folder } i { $torrents }
downloads-unsubscribe-heading = Anulować subskrypcję { $title }?
downloads-unsubscribe-base = Jej kanał i reguła pobierania znikają z { $client }, więc nic nowego nie przyjdzie.
downloads-unsubscribe-complete-heading = Anulować subskrypcje wszystkich kompletnych seriali ({ $count })?
downloads-unsubscribe-complete-base = Ich kanały i reguły pobierania znikają z { $client }. Każdy ich odcinek jest w bibliotece.
downloads-files-folders-and-torrents = ich foldery i to, co pobrały ich torrenty
downloads-one-at-a-time = Znikają po kolei; strona pokazuje każdą na bieżąco.
downloads-subscription-stays = { $base } Subskrypcja zostaje.
downloads-cancel-heading = Anulować { $title }?
downloads-cancel-group-base =
    { $count ->
        [one] Jego pobieranie zatrzymuje się i znika z { $client }.
        [few] Jego { $count } pobrania zatrzymują się i znikają z { $client }.
        [many] Jego { $count } pobrań zatrzymuje się i znika z { $client }.
       *[other] Jego { $count } pobrania zatrzymuje się i znika z { $client }.
    }
downloads-files-so-far-it = to, co dotąd pobrało
downloads-files-so-far-they = to, co dotąd pobrały
downloads-cancel-download = Anuluj pobieranie
downloads-keep-downloading = Pobieraj dalej
downloads-remove-heading = Usunąć { $title }?
downloads-remove-group-base =
    { $count ->
        [one] Jego torrent znika z { $client }; pliki zostają, chyba że zaznaczysz poniżej.
        [few] Jego { $count } torrenty znikają z { $client }; pliki zostają, chyba że zaznaczysz poniżej.
        [many] Jego { $count } torrentów znika z { $client }; pliki zostają, chyba że zaznaczysz poniżej.
       *[other] Jego { $count } torrenta znika z { $client }; pliki zostają, chyba że zaznaczysz poniżej.
    }
downloads-remove-detail-base =
    { $count ->
        [one] Jego { $count } torrent znika z { $client }; pliki zostają, chyba że zaznaczysz poniżej.
        [few] Jego { $count } torrenty znikają z { $client }; pliki zostają, chyba że zaznaczysz poniżej.
        [many] Jego { $count } torrentów znika z { $client }; pliki zostają, chyba że zaznaczysz poniżej.
       *[other] Jego { $count } torrenta znika z { $client }; pliki zostają, chyba że zaznaczysz poniżej.
    }
downloads-remove-broken-heading = Usunąć torrenty wszystkich seriali z brakującymi plikami ({ $count })?
downloads-remove-broken-base = Ich torrenty ({ $count }) znikają z { $client }. Wskazują na foldery, których nie ma, więc żaden plik nie zostanie ruszony; subskrypcje zostają.
downloads-remove-torrent-heading = Usunąć ten torrent?
downloads-files-it-downloaded = to, co pobrał
downloads-cancel-torrent-heading = Anulować to pobieranie?
downloads-cancel-many-heading =
    { $count ->
        [one] Anulować { $count } pobranie?
        [few] Anulować { $count } pobrania?
        [many] Anulować { $count } pobrań?
       *[other] Anulować { $count } pobrania?
    }
downloads-cancel-many-base = Każde pobieranie tego serialu zatrzymuje się i znika z { $client }.
downloads-cancel-all = Anuluj wszystkie
downloads-summary-connecting = Łączenie.
downloads-summary-client-on = { $client } { $version } jest połączony.
downloads-summary-client-off = { $client } nie jest połączony. Sprawdź jego adres i hasło w Ustawieniach, w sekcji Pobieranie.
downloads-summary-native = AniBeam sam przeszukuje Twoje źródła.
downloads-summary-native-with = AniBeam sam przeszukuje Twoje źródła. { $client }
downloads-summary-starting = Uruchamianie pobierania.
downloads-summary-not-running = Pobieranie nie działa.
downloads-summary-not-started = Pobieranie jeszcze nie ruszyło.
downloads-summary-helper-ready = Pomocnik wyszukiwania { $version } jest gotowy. { $client }
downloads-subscribed-as = Zasubskrybowano jako { $name }.
downloads-repaired =
    { $count ->
        [one] Naprawiono { $name }: kanał znajduje { $count } wydanie
        [few] Naprawiono { $name }: kanał znajduje { $count } wydania
        [many] Naprawiono { $name }: kanał znajduje { $count } wydań
       *[other] Naprawiono { $name }: kanał znajduje { $count } wydania
    }
downloads-repaired-keeps =
    { $count ->
        [one] Naprawiono { $name }: kanał znajduje { $count } wydanie, a reguła zachowuje „{ $term }”
        [few] Naprawiono { $name }: kanał znajduje { $count } wydania, a reguła zachowuje „{ $term }”
        [many] Naprawiono { $name }: kanał znajduje { $count } wydań, a reguła zachowuje „{ $term }”
       *[other] Naprawiono { $name }: kanał znajduje { $count } wydania, a reguła zachowuje „{ $term }”
    }
downloads-sent-torrents =
    { $count ->
        [one] Wysłano { $count } torrent do { $client }.
        [few] Wysłano { $count } torrenty do { $client }.
        [many] Wysłano { $count } torrentów do { $client }.
       *[other] Wysłano { $count } torrenta do { $client }.
    }
downloads-added-torrents =
    { $count ->
        [one] Dodano { $count } torrent.
        [few] Dodano { $count } torrenty.
        [many] Dodano { $count } torrentów.
       *[other] Dodano { $count } torrenta.
    }
downloads-trashed-paths =
    { $count ->
        [one] { $title }: { $count } ścieżkę przeniesiono do kosza
        [few] { $title }: { $count } ścieżki przeniesiono do kosza
        [many] { $title }: { $count } ścieżek przeniesiono do kosza
       *[other] { $title }: { $count } ścieżki przeniesiono do kosza
    }
downloads-show-gone = Tego serialu nie ma już w bibliotece
downloads-activity-hide = Ukryj aktywność
downloads-activity = Aktywność
downloads-moving =
    { $count ->
        [one] { $count } pobranie w toku
        [few] { $count } pobrania w toku
        [many] { $count } pobrań w toku
       *[other] { $count } pobrania w toku
    }
downloads-back = Wróć do Pobranych
downloads-chip-search-ready = Wyszukiwanie gotowe
downloads-chip-search-starting = Wyszukiwanie startuje
downloads-chip-search-off = Wyszukiwanie wył.
downloads-chip-answers-at = { $summary } Odpowiada pod { $url }.
downloads-chip-helper-missing = Pomocnik wyszukiwania nie jest zainstalowany. Kliknij, aby przejść do ustawień Pobierania, które mówią, jak go dodać
downloads-chip-helper-failed = Pomocnik wyszukiwania się zatrzymał. Połącz ponownie uruchamia go znowu; ustawienia Pobierania mówią, co poszło nie tak
downloads-chip-open-settings = Otwórz ustawienia Pobierania
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } wył.
downloads-chip-client-not-connected = { $client } nie jest połączony. Połącz ponownie albo sprawdź jego adres w Ustawieniach
downloads-chip-client-remote = Pomocnik wyszukiwania uruchamiany przez Ciebie używa klienta podanego w jego własnych ustawieniach
downloads-chip-client-switch = Kliknij, aby zmienić klienta
downloads-connecting = Łączenie
downloads-reconnect = Połącz ponownie
downloads-search-placeholder = Szukaj serialu lub nazwy wydania
downloads-search-go-tip = Szukaj nazw wydań w { $names }
downloads-search-go-off = Każde źródło jest wyłączone.
downloads-suggest-airing =
    { $predicted ->
        [yes] W emisji, odcinek { $episode } przewidywany za { $time }
       *[no] W emisji, odcinek { $episode } za { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] W emisji, odcinki { $episode }-{ $through } przewidywane za { $time }
       *[no] W emisji, odcinki { $episode }-{ $through } za { $time }
    }
downloads-suggest-finished = Zakończony
downloads-suggest-episodes = Odcinki: { $count }
downloads-suggest-in-library = W Twojej bibliotece
downloads-suggest-on-anilist = Na AniList, nie w Twojej bibliotece
downloads-plan-to-watch = Planuję obejrzeć
downloads-suggest-release-search = Zamiast tego szukaj nazw wydań „{ $query }” w { $names }
downloads-suggest-release-search-none = Zamiast tego szukaj nazw wydań „{ $query }”, bez żadnego źródła
downloads-key-return = Enter
downloads-sources = Źródła
downloads-source-hint-starting = uruchamianie
downloads-source-hint-down = nie działa
downloads-source-tip-down = { $name } potrzebuje usługi pomocniczej, która nie działa. Kliknij, aby ją uruchomić.
downloads-source-tip-on = Wyszukiwania pytają { $name }. Kliknij, aby wyłączyć.
downloads-source-tip-off = Wyłączone. Kliknij, aby znów pytać { $name }.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Sprawdź
    .tip = Pyta, czy każde źródło uruchamiane na żądanie działa. Niczego nie uruchamia.
downloads-check-busy = Sprawdzanie
downloads-back-to-grid = Wróć do siatki
    .tip = Wróć do { $title } i tego, co znalazł każdy dostawca. Escape robi to samo.
downloads-releases-on =
    { $count ->
        [one] { $count } wydanie w { $name }
        [few] { $count } wydania w { $name }
        [many] { $count } wydań w { $name }
       *[other] { $count } wydania w { $name }
    }
downloads-found-on = Znaleziono w
downloads-found-searching = szukanie
downloads-found-failed = błąd
downloads-found-cancelled = anulowano
downloads-releases =
    { $count ->
        [one] { $count } wydanie
        [few] { $count } wydania
        [many] { $count } wydań
       *[other] { $count } wydania
    }
downloads-found-retry-tip =
    { $why }
    Kliknij, aby spróbować ponownie
downloads-facet-groups = Grupa
downloads-facet-quality = Jakość
downloads-facet-words = Słowa
downloads-filters-fewer = Mniej filtrów
downloads-filters-more = Więcej filtrów ({ $count })
downloads-narrow-placeholder = Zawęź słowem albo -słowo, aby pominąć
downloads-best-match = Najlepsze
    .tip = Zostawia wydania podobne do najlepszego: { $query }
downloads-best-match-none = Żadne wydanie się tu nie wyróżnia
downloads-strip-have-of = { $have } z { $total } odcinków na dysku
downloads-strip-have =
    { $count ->
        [one] { $count } odcinek na dysku
        [few] { $count } odcinki na dysku
        [many] { $count } odcinków na dysku
       *[other] { $count } odcinka na dysku
    }
downloads-strip-next =
    { $predicted ->
        [yes] odcinek { $episode } przewidywany za { $time }
       *[no] odcinek { $episode } za { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] odcinki { $episode }-{ $through } przewidywane za { $time }
       *[no] odcinki { $episode }-{ $through } za { $time }
    }
downloads-strip-complete = kompletne
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = Żadne z tych wydań nie dotyczy odcinka, który już masz.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } wydanie tutaj to plik, który masz, oznaczony Na dysku
        [few] { $count } wydania tutaj to pliki, które masz, oznaczone Na dysku
        [many] { $count } wydań tutaj to pliki, które masz, oznaczone Na dysku
       *[other] { $count } wydania tutaj to pliki, które masz, oznaczone Na dysku
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } wydanie dotyczy odcinka, który masz z innego wydania
        [few] { $count } wydania dotyczą odcinka, który masz z innego wydania
        [many] { $count } wydań dotyczy odcinka, który masz z innego wydania
       *[other] { $count } wydania dotyczy odcinka, który masz z innego wydania
    }
downloads-strip-both = { $first }; { $second }.
downloads-tick-missing =
    { $count ->
        [one] Zaznacz { $count } brakujący odcinek
        [few] Zaznacz { $count } brakujące odcinki
        [many] Zaznacz { $count } brakujących odcinków
       *[other] Zaznacz { $count } brakującego odcinka
    }
    .tip = Dla każdego odcinka, którego biblioteka nie ma, wydanie z największą liczbą seedów
downloads-open-in-library = Otwórz w bibliotece
downloads-not-this-show = Nie ten serial
downloads-no-releases = Brak wydań
downloads-no-releases-body = Nic w { $name } nie pasuje do „{ $query }”. Inne źródło albo mniej słów może coś znaleźć.
downloads-try-source = Spróbuj { $name }
downloads-ticked =
    { $count ->
        [one] Zaznaczono { $count } wydanie
        [few] Zaznaczono { $count } wydania
        [many] Zaznaczono { $count } wydań
       *[other] Zaznaczono { $count } wydania
    }
downloads-nothing-ticked = Nic nie zaznaczono
downloads-ticked-size = Razem { $size }.
downloads-tick-hint = Zaznacz wydania w tabeli albo pobierz je wszystkie.
downloads-tick-all = Zaznacz wszystko
downloads-working = Pracuję
downloads-download-ticked =
    { $count ->
        [one] Pobierz { $count } wydanie
        [few] Pobierz { $count } wydania
        [many] Pobierz { $count } wydań
       *[other] Pobierz { $count } wydania
    }
downloads-download-shown = Pobierz widoczne ({ $count })
downloads-download-all = Pobierz wszystkie ({ $count })
downloads-download-movie = Pobierz jako film
    .tip = Jedno zaznaczone wydanie, prosto do folderu z filmami
downloads-subscribe = Subskrybuj
downloads-subscribe-body = Nowe wydania pasujące do tego wyszukiwania pobierają się, gdy się pojawią, do podanego tu folderu.
downloads-name-placeholder = Wzięte z pierwszego wydania
downloads-subscribe-search = Subskrybuj to wyszukiwanie
downloads-subscribe-no-feed = Tu nie ma kanału do subskrypcji
downloads-preview-again = Pokaż regułę ponownie
downloads-preview-show = Pokaż regułę, którą tworzy
downloads-preview-from = Z { $host }
downloads-preview-from-named = Z { $host } ({ $name })
downloads-preview-must = Musi zawierać { $terms }
downloads-preview-must-not = Nie może zawierać { $terms }
downloads-helper-changed = Pobieranie wskazuje pomocnika, który nie odpowiada
downloads-helper-missing = Ta kopia AniBeam nie ma pobierania
downloads-helper-failed = Pobieranie nie mogło ruszyć
downloads-helper-changed-body = Wyszukiwanie i pobieranie działają przez mały program pomocniczy. Ustawienia wskazują inny niż własny AniBeam, a ten nie odpowiada. Przełącz z powrotem, a uruchomi się sam.
downloads-helper-missing-body = Zbudowano ją bez programu pomocniczego, przez który działają wyszukiwanie i pobieranie. Instalatory ze strony wydań go mają. Ustawienia, Pobieranie mówią, jak dodać go do tej kopii.
downloads-helper-failed-body = Program pomocniczy, przez który działają wyszukiwanie i pobieranie, zatrzymał się, zanim był gotowy. Ponowne uruchomienie zwykle to naprawia.
downloads-helper-use-own = Użyj własnego AniBeam
downloads-helper-start-again = Uruchom ponownie
downloads-helper-show-how = Pokaż jak
downloads-helper-what-went-wrong = Co poszło nie tak
downloads-lens-gaps-count = Braki w bibliotece  { $count }
downloads-lens-airing-count = W emisji  { $count }
downloads-lens-following-count = Obserwowane  { $count }
downloads-lens-downloaded-count = Pobrane  { $count }
downloads-lens-gaps = Braki w bibliotece
downloads-lens-gaps-line = Wyemitowane odcinki, których nie ma na dysku. Otwórz serial, aby zobaczyć, kto je ma.
downloads-lens-gaps-none = Niczego nie brakuje: każdy wyemitowany odcinek każdego serialu jest na dysku.
downloads-lens-airing-line = Seriale z biblioteki, którym zostały odcinki do emisji. Otwórz jeden, aby go obserwować.
downloads-lens-airing-none = Nic w bibliotece nie jest w emisji
downloads-lens-airing-none-body = Serial trafia tutaj, gdy jest w bibliotece i ma jeszcze odcinki do emisji. Teraz w emisji pokazuje wszystko, co jest emitowane.
downloads-airing-now = Teraz w emisji
downloads-lens-following = Obserwowane
downloads-card-next =
    { $predicted ->
        [yes] Odcinek { $episode } przewidywany za { $time }
       *[no] Odcinek { $episode } za { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Odcinki { $episode }-{ $through } przewidywane za { $time }
       *[no] Odcinki { $episode }-{ $through } za { $time }
    }
downloads-rules-unread = Nie udało się odczytać subskrypcji.
downloads-lens-following-line = Kanały obserwowane przez AniBeam, jako seriale, które wypełniają.
downloads-none-yet = Jeszcze brak.
downloads-reading = Czytanie.
downloads-rules-error = AniBeam nie mógł odczytać subskrypcji. Spróbuj za chwilę.
downloads-removing-to-go = Usuwanie, zostało { $count }
downloads-unsubscribe-all = Anuluj wszystkie ({ $count })
downloads-unsubscribe = Anuluj subskrypcję
downloads-no-rules = Brak subskrypcji
downloads-no-rules-body = Wyszukaj powyżej i zasubskrybuj, aby serial przychodził sam.
downloads-search = Szukaj
downloads-lens-downloaded = Pobrane
downloads-dl-line = { $size } na dysku. { $rest }
downloads-dl-client-off = { $client } nie jest połączony.
downloads-dl-unread = Nie udało się odczytać listy pobrań.
downloads-dl-in-client =
    { $count ->
        [one] { $count } torrent, { $size } w { $client }.
        [few] { $count } torrenty, { $size } w { $client }.
        [many] { $count } torrentów, { $size } w { $client }.
       *[other] { $count } torrenta, { $size } w { $client }.
    }
downloads-usage-watched = Obejrzane
downloads-usage-not-watched = Nieobejrzane
downloads-deleting = Usuwanie
downloads-delete-watched = Usuń obejrzane
downloads-remove-all = Usuń wszystkie ({ $count })
downloads-on-disk = Na dysku
downloads-on-disk-line = W bibliotece, bez torrenta w { $client }, od największych. Kliknij prawym przyciskiem, aby usunąć.
downloads-nothing-downloaded = Nic nie pobrano
downloads-nothing-downloaded-body = To, co pobierasz lub subskrybujesz, pojawia się tutaj, serial po serialu.
downloads-client-off-title = { $client } nie jest połączony
downloads-client-off-body = Sprawdź, czy działa, i połącz ponownie. Jego adres i hasło są w Ustawieniach, w sekcji Pobieranie.
downloads-connect-again = Połącz ponownie
downloads-view-grid = Siatka
downloads-view-list = Lista
downloads-chip-subscribed = Subskrybowane
downloads-speed-up = ↑ { $speed }/s
downloads-broken-some = Z brakującymi plikami: { $count }
downloads-tag-watched = Obejrzane

## PlayerDelays.qml

player-delays-title = Opóźnienie
player-delays-subtitles = Napisy
player-delays-audio = Dźwięk
player-delays-subtitles-earlier = Napisy 0,1 s wcześniej
player-delays-subtitles-later = Napisy 0,1 s później
player-delays-audio-earlier = Dźwięk 0,1 s wcześniej
player-delays-audio-later = Dźwięk 0,1 s później
player-delays-reset = Wyzeruj do następnego pliku
player-delays-none = Bez opóźnienia

## TrackPicker.qml

track-picker-off = Wył.

## RollControls.qml

roll-controls-window = W gotowości { $ahead }: odcinki { $next }-{ $through } z { $total }
roll-controls-window-watched = W gotowości { $ahead }: odcinki { $next }-{ $through } z { $total }, obejrzane: { $watched }
roll-controls-more = Pobierz jeszcze { $count }
roll-controls-more-tip = Pobierz następne odcinki teraz. AniBeam znów będzie trzymać kilka w gotowości, gdy nadrobisz.
roll-controls-fewer-tip = Trzymaj o jeden mniej do przodu
roll-controls-ahead = { $count } do przodu
roll-controls-further-tip = Trzymaj o jeden więcej do przodu
roll-controls-all = Pobierz wszystko
roll-controls-all-tip = Przestań trzymać kilka w gotowości i pobierz każdy pozostały odcinek
roll-controls-all-question = Pobrać wszystkie pozostałe odcinki ({ $count })?

## PlayerChrome.qml

player-chrome-delays = Opóźnienie napisów i dźwięku
player-chrome-audio = Ścieżka dźwiękowa
player-chrome-subtitles = Napisy
player-chrome-subtitles-off = Napisy wył.
player-chrome-casting-to = Przesyłanie na: { $device }
player-chrome-cast = Prześlij na TV
player-chrome-mark-watched = Oznacz jako obejrzane
player-chrome-shortcuts = Skróty klawiszowe
player-chrome-fullscreen = Pełny ekran
player-chrome-exit-fullscreen = Zamknij pełny ekran
player-chrome-stay = Zostań
player-chrome-skip-intro = Pomiń intro
player-chrome-skip-outro = Pomiń outro
player-chrome-playing-on = Odtwarzanie na: { $device }
player-chrome-replay = Powtórz

## PlayerPage.qml

player-title = Odtwarzacz
player-next = Dalej
player-extra-no-next = Dodatek nie ma następnego odcinka
player-last-episode = To ostatni odcinek
player-extra-no-previous = Dodatek nie ma poprzedniego odcinka
player-first-episode = To pierwszy odcinek
player-resumed-at = Wznowiono od { $time }
player-start-over = Od początku
player-audio-track = Dźwięk: { $track }
player-subtitles-track = Napisy: { $track }
player-subtitles-on = Napisy: wł.
player-subtitles-off = Napisy wył.
player-no-subtitles = Ten plik nie ma napisów
# A delay in seconds, signed: "+0.1 s", "0.0 s"
player-delay-seconds = { $value } s
player-audio-delay = Opóźnienie dźwięku { $value }
player-subtitle-delay = Opóźnienie napisów { $value }
player-volume = Głośność { $volume }
player-speed = Prędkość { $speed }×
player-not-while-casting = Nie podczas przesyłania
player-hold-fast = 2× po przytrzymaniu
player-no-chapters = Ten plik nie ma rozdziałów
player-last-chapter = To ostatni rozdział
player-chapter = Rozdział { $number }
player-chapter-titled = Rozdział { $number }: { $title }
player-screenshot-saved = Zapisano w Obrazy/AniBeam
player-skipped-intro = Pominięto intro
player-skipped-outro = Pominięto outro
player-undo = Cofnij
# Why a tracker was not updated, read after "Not tracked:" or a tracker's name
player-refusal-hidden = serial jest ukryty
player-refusal-no-match = serial nie jest jeszcze dopasowany; dopasuj go w Informacjach
player-refusal-not-newer = Twoja lista jest już dalej niż ten odcinek
player-refusal-extra = to dodatek
player-refusal-unmatched = plik nie jest dopasowany do odcinka
player-refusal-on-disk = plik jest tylko na dysku
player-not-tracked-because = Nie zapisano: { $reason }.
# A tracker and what it said: "AniList: the show is hidden"
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = Nie udało się zaktualizować { $tracker }
player-outcome-ok = { $tracker }: OK
player-outcome-at = { $tracker }: { $progress }
player-outcome-tracked = Zapisano
player-outcome-not-tracked = Nie zapisano
player-outcome-could-not-update = Nie udało się zaktualizować listy
# $list is each tracker's outcome joined by "; "
player-outcome-tracked-list = Zapisano: { $list }
player-outcome-not-tracked-list = Nie zapisano: { $list }
player-outcome-failed-list = Nie udało się zaktualizować listy: { $list }
player-rated = Oceniono na { $score }
player-score-failed = Nie udało się zapisać oceny
player-frame = Klatka { $number } w { $time }
player-audio-title = Dźwięk
player-subtitles-title = Napisy
player-cast-to = Prześlij na
player-cast-looking = Szukanie Chromecastów i telewizorów
player-cast-unreachable = Brak połączenia z telewizorem
player-cast-tv = TV
player-cast-stop = Zatrzymaj przesyłanie na: { $device }
player-cast-connecting = Łączenie
player-cast-loading = Wczytywanie
player-cast-playing = Odtwarzanie
player-cast-paused = Wstrzymano
player-cast-finished = Zakończono
player-cast-stopped = Zatrzymano
player-rate-prompt = Zapisano ostatni odcinek. Ocenić ten serial?
player-rate = Oceń
player-not-now = Nie teraz

## EndpointReposPanel.qml

endpoint-repos-panel-title = Listy źródeł
endpoint-repos-panel-helper = Lista źródeł to podpisany zestaw miejsc do przeszukiwania, dodawany po adresie. AniBeam sprawdza podpis i pokazuje Ci klucz, zanim cokolwiek zapisze, a potem instalujesz wybrane źródła. Zainstalowane dołączają do Źródeł wyszukiwania powyżej.
endpoint-repos-panel-could-not-start = AniBeam nie mógł tego uruchomić. Spróbuj za chwilę.
endpoint-repos-panel-needs-value = { $name } potrzebuje wartości
endpoint-repos-panel-values-of = Wartości { $name }
endpoint-repos-panel-secret-note = Przechowywane w magazynie haseł tego komputera, nigdy w bibliotece ani eksporcie.
endpoint-repos-panel-secret-note-keep = Przechowywane w magazynie haseł tego komputera, nigdy w bibliotece ani eksporcie. Puste pole zachowuje zapisaną wartość.
endpoint-repos-panel-asks-again = { $name } w wersji { $version } pyta ponownie
endpoint-repos-panel-update-sends-to = Nowa wersja wysyła to do { $host }.
endpoint-repos-panel-fields-changed = Jego pola się zmieniły.
# $fields is the field names joined by ", "
endpoint-repos-panel-needs-fields = Przed wyszukiwaniem potrzebuje pól: { $fields }.
endpoint-repos-panel-no-longer-offered = Jego lista źródeł już go nie oferuje.
endpoint-repos-panel-update-requests-to = Aktualizacja wysyła zapytania do { $host }.
endpoint-repos-panel-unreadable = Nie udało się odczytać tej listy. Odśwież listy spróbuje ponownie.
endpoint-repos-panel-add-list = Dodaj listę źródeł
endpoint-repos-panel-add-list-helper = Adres jej pliku indeksu albo ścieżka do niego na tym komputerze.
endpoint-repos-panel-checking = Sprawdzanie
endpoint-repos-panel-add = Dodaj
endpoint-repos-panel-trust-question = Zaufać { $name }?
endpoint-repos-panel-fingerprint = Odcisk klucza  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    Źródła: { $count }, podpisane. Jej źródła decydują, dokąd trafiają Twoje wyszukiwania; ufaj liście tylko od kogoś, komu ufasz.
endpoint-repos-panel-preview-trusted =
    { $url }
    Źródła: { $count }, podpisane, już zaufane. Jej źródła decydują, dokąd trafiają Twoje wyszukiwania; ufaj liście tylko od kogoś, komu ufasz.
endpoint-repos-panel-trust = Zaufaj
endpoint-repos-panel-saving = Zapisywanie
endpoint-repos-panel-install = Zainstaluj
endpoint-repos-panel-update = Aktualizuj
endpoint-repos-panel-added-by-you = dodane przez Ciebie
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Aktualizuj do v{ $version }
endpoint-repos-panel-add-value = Dodaj wartość
endpoint-repos-panel-change = Zmień { $fields }
endpoint-repos-panel-asks-for = Prosi o pola: { $fields }.
endpoint-repos-panel-by-hand = Dodaj ręcznie
endpoint-repos-panel-by-hand-helper = Dla witryn bez listy: dowolny kanał z {"{"}query{"}"} w miejscu wyszukiwania, też z proxy. Klucz z adresu trafia osobno do systemowego magazynu haseł.
endpoint-repos-panel-name = Nazwa
endpoint-repos-panel-refreshing = Odświeżanie
endpoint-repos-panel-refresh = Odśwież listy

## ScheduleSheet.qml

schedule-episodes-range = Odcinki { $first }-{ $last }
# Qt date formats: a full day, a month, and the short ones on an episode tile
schedule-day-format = ddd d MMM yyyy
schedule-month-format = MMMM yyyy
schedule-tile-day-format = d MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = Kiedyś w { $year }
schedule-no-date-yet = Jeszcze bez daty
# A date guessed from a weekly run
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = Bez daty
schedule-tile-last = Ostatni
schedule-tile-out = Wyszedł
schedule-ends-at = Kończy się na odcinku { $number }
schedule-dated-by-site = Daty z { $site }: { $count }
schedule-dated-by-record = Daty z wpisu: { $count }
schedule-set-by-you = Ustawione przez Ciebie: { $count }
schedule-guessed-weekly = Zgadnięte co tydzień: { $count }
schedule-nothing-set = Jeszcze nic nie ustawiono
schedule-saved = Zapisano
schedule-cleared = Wyczyszczono notatki harmonogramu
schedule-title = Odcinki i daty
schedule-season = Sezon { $number }
schedule-ask-agent = Zapytaj { $agent }
schedule-done-pasting = Gotowe
schedule-paste-list = Wklej listę
schedule-agent-looking = { $agent } szuka dat. Nic nie zostanie zapisane, dopóki nie klikniesz Zapisz.
schedule-agent-stopped = { $agent } się zatrzymał: { $error }
schedule-stop = Zatrzymaj
schedule-paste-placeholder = Wklej listę wydań, tabelę albo post
schedule-paste-helper = To, co AniBeam odczyta, pojawi się przy odcinkach poniżej.
schedule-agent-hint = Z Claude Code na tym komputerze AniBeam może poprosić go o wyszukanie dat.
schedule-show-more = Pokaż więcej odcinków
schedule-pick-hint = Kliknij odcinek, aby nadać mu datę. Shift i kliknięcie wybiera zakres.
schedule-picked-one = Odcinek { $number }:
schedule-picked = { $count ->
    [one] { $count } odcinek:
    [few] { $count } odcinki:
    [many] { $count } odcinków:
   *[other] { $count } odcinka:
}
# What a typed date may look like; the parser reads these shapes
schedule-date-placeholder = 7 stycznia, marzec 2027, 2027
schedule-set-date = Ustaw datę
schedule-weekly = Co tydzień
schedule-not-last = Nie ostatni
schedule-last-episode = Ostatni odcinek
schedule-clear-date = Wyczyść datę

## ShowPage.qml

show-title = Serial
# AniList's formats
# AniList's airing states
# $runtime is a length such as "24 min"
show-runtime-each = po { $runtime }
show-add-to-list = Dodaj do listy
show-moving-to = Przenoszenie do: { $list }
show-moved-to = Przeniesiono do: { $list }
show-relation-related = Powiązane
show-no-poster = Brak plakatu
show-reading-anilist = Czytanie AniList
show-not-in-library = Nie ma w Twojej bibliotece
show-get-episodes = Pobierz odcinki
show-trailer = Zwiastun
show-your-score = Twoja ocena { $score }
show-rate-it = Oceń
show-anilist-score = AniList { $score }
show-forecast = { $percent }% dla Ciebie
show-because-you-watched = Bo oglądałeś
show-hidden-loved = { $count ->
    [one] Stoi za tym ukryty tag, który lubisz
    [few] Stoją za tym { $count } ukryte tagi, które lubisz
    [many] Stoi za tym { $count } ukrytych tagów, które lubisz
   *[other] Stoi za tym { $count } ukrytego tagu, który lubisz
}
show-you-gave-it = Oceniłeś na { $score }
show-story = Fabuła
show-genres-tags = Gatunki i tagi
show-studios = Studia  { $studios }
show-related = Powiązane
show-recommendations = Polecane
show-from-anilist = Z AniList
show-reading-rest = Czytanie reszty z AniList

## ConnectSheet.qml

# $name is the tracker: AniList or MyAnimeList
connect-connected-to = Połączono z { $name }
connect-connect-to = Połącz { $name }
connect-did-not-finish = Logowanie się nie zakończyło
connect-finish-in-browser = Dokończ w przeglądarce
# $account is the user's name on the tracker
connect-signed-in-anilist = Zalogowano jako { $account }. Twoja lista jest na stronie Oglądam.
connect-signed-in-mal = Zalogowano jako { $account }. Odcinki są tam oznaczane, gdy oglądasz.
connect-setup-lead = { $name } loguje AniBeam przez Twoją własną aplikację. To zajmuje około minuty, raz.
connect-page-open = Strona logowania { $name } jest otwarta w przeglądarce. Zatwierdź tam AniBeam, a to okno zamknie się samo.
connect-page-opening = Otwieranie strony logowania { $name } w przeglądarce.
connect-open-watching = Otwórz Oglądam
connect-done = Gotowe
connect-open-again = Otwórz ponownie stronę logowania
connect-secret-where = To długi ciąg pod "Client Secret" na stronie aplikacji w myanimelist.net/apiconfig.
connect-client-secret = Client secret
connect-save-try-again = Zapisz i spróbuj ponownie
connect-change-client-id = Zmień Client ID
connect-step-create-anilist = Utwórz klienta w AniList
connect-step-create-mal = Utwórz aplikację w MyAnimeList
connect-step-create-anilist-body = Otwórz ustawienia deweloperskie AniList, utwórz nowego klienta i wklej ten adres w pole Redirect URL dokładnie tak, jak jest tutaj.
connect-step-create-mal-body = Otwórz stronę API MyAnimeList, utwórz ID z App Type "other" i wklej ten adres w pole App Redirect URL dokładnie tak, jak jest tutaj.
connect-open-developer-page = Otwórz stronę dla deweloperów
connect-copy-address = Kopiuj adres
connect-step-paste-id = Wklej ID aplikacji
connect-step-paste-id-anilist = AniList nazywa to Client ID: numer na stronie Twojego nowego klienta.
connect-step-paste-id-mal = MyAnimeList nazywa to Client ID: długi ciąg na stronie Twojej nowej aplikacji.
connect-client-id = Client ID
connect-client-secret-web = Client secret (tylko dla aplikacji Web)
connect-step-connect-body = Otworzy się przeglądarka, { $name } poprosi o zalogowanie i zatwierdzenie AniBeam, a AniBeam odbierze odpowiedź.

## SettingsPage

settings-title = Ustawienia
settings-search-placeholder = Szukaj ustawień
settings-section-library = Biblioteka
settings-section-accounts = Konta
settings-section-appearance = Wygląd
settings-section-playback = Odtwarzanie
settings-section-subtitles = Napisy
settings-section-keys = Klawiatura
settings-section-downloads = Pobieranie
settings-section-phone = Aplikacja na telefon
settings-section-storage = Pamięć i kopia
settings-section-help = Pomoc
settings-section-about = Informacje
settings-section-developer = Deweloper
settings-status-library-empty = Jeszcze brak folderów
settings-status-library =
    { $count ->
        [one] { $count } serial
        [few] { $count } seriale
        [many] { $count } seriali
       *[other] { $count } serialu
    } w { $folders ->
        [one] { $folders } folderze
        [few] { $folders } folderach
        [many] { $folders } folderach
       *[other] { $folders } folderu
    }
settings-status-library-missing =
    { $gone ->
        [one] Nie znaleziono { $gone } folderu
        [few] Nie znaleziono { $gone } folderów
        [many] Nie znaleziono { $gone } folderów
       *[other] Nie znaleziono { $gone } folderu
    }, { $count ->
        [one] { $count } serial
        [few] { $count } seriale
        [many] { $count } seriali
       *[other] { $count } serialu
    } w { $folders ->
        [one] { $folders } folderze
        [few] { $folders } folderach
        [many] { $folders } folderach
       *[other] { $folders } folderu
    }
settings-status-accounts-anilist = Synchronizacja z AniList
settings-status-accounts-mal = Synchronizacja z MyAnimeList
settings-status-accounts-both = Synchronizacja z AniList i MyAnimeList
settings-status-accounts-none = Brak połączonej listy
settings-status-appearance =
    { $mode ->
        [system] Zgodnie z systemem
        [light] Jasny
       *[dark] Ciemny
    }, { $source ->
        [theme] kolory motywu
       *[desktop] kolory pulpitu
    }
settings-status-playback =
    { $skip ->
        [both] Pomija intro i outro
        [intro] Pomija intro
        [outro] Pomija outro
       *[none] Odtwarza wszystko
    }
settings-status-playback-audio =
    { $skip ->
        [both] Pomija intro i outro
        [intro] Pomija intro
        [outro] Pomija outro
       *[none] Odtwarza wszystko
    }, dźwięk { $languages }
settings-status-subtitles =
    Rozmiar { $percent } procent, { $override ->
        [AsScripted] pliki ze stylem jak oryginał
        [Force] Twój styl w każdym pliku
       *[ScaleOnly] pliki ze stylem zachowują wygląd
    }
settings-status-keys = Skróty klawiszowe AniBeam
settings-status-downloads-unread = Jeszcze nie odczytano
settings-status-downloads-answered = qBittorrent odpowiedział
settings-status-downloads-missing = Pomocnik wyszukiwania nie jest zainstalowany
settings-status-downloads-unconnected = Jeszcze nie połączono
settings-status-phone-shared = Udostępnione w Twoim tailnecie
settings-status-phone-off = Nie udostępnione
settings-status-storage = Pamięć obrazów: { $size }
settings-status-storage-unread = Obrazy, eksport i import
settings-status-help = Odpowiedzi na najczęstsze pytania
settings-status-about-update = Wyszła wersja { $version }
settings-status-developer = Mierzenie czasu każdej strony
settings-status-developer-last = { $page } zbudowano w { $ms } ms
settings-results =
    { $count ->
        [one] Pasuje { $count } ustawienie
        [few] Pasują { $count } ustawienia
        [many] Pasuje { $count } ustawień
       *[other] Pasuje { $count } ustawienia
    }
settings-results-none = Nic nie pasuje
settings-results-hint = Enter otwiera pierwsze.
settings-results-none-hint = Spróbuj prostszego słowa: motyw, napisy, serwis, hasło, folder.

# The search's index: each row or group by its name, as its own tab shows it, and the
# words a person might type for it
settings-find-folders =
    .words = dodaj folder skanuj przeskanuj usuń źródło ścieżka kolekcja
settings-find-matching =
    .words = metadane informacje o serialu dostawca anilist tvmaze kolejność dopasowanie
settings-find-hidden-shows =
    .words = ukryte pokaż
settings-find-hidden-airing =
    .words = nie interesuje emisja pokaż ponownie
settings-find-calendar =
    .words = kalendarz ics ical emisja harmonogram subskrybuj
settings-find-anime-lists =
    .words = anilist myanimelist mal zaloguj połącz logowanie konto rozłącz serwis
settings-find-main-list =
    .words = postęp karty główny serwis
settings-find-sync-now =
    .words = odśwież postęp listy serwis synchronizuj
settings-find-on-hold =
    .words = wstrzymane pauza nieaktywne nieruszane dni
settings-find-language =
    .words = język tłumaczenie lokalizacja angielski polski language
settings-find-mode =
    .words = ciemny jasny system motyw noc tryb
settings-find-colours-from =
    .words = terminal pulpit system motyw źródło kitty kolory
settings-find-dark-theme =
    .words = kolor schemat base16 ciemny motyw
settings-find-light-theme =
    .words = kolor schemat base16 jasny motyw
settings-find-accent =
    .words = kolor wyróżnienie akcent
settings-find-text-size =
    .words = czcionka pismo tekst większy mniejszy powiększenie skala czytanie dostępność
settings-find-density =
    .words = kompaktowy wygodny odstępy rozmiar gęstość
settings-find-poster-size =
    .words = karty mały średni duży plakat
settings-find-out-now =
    .words = emisja już wyszło karty lista wiersze styl baner plakaty
settings-find-out-now-cards =
    .words = emisja już wyszło siatka kolumny wiersze układ
settings-find-reduce-motion =
    .words = animacja dostępność ruch
settings-find-app-icon =
    .words = logo launcher dok pasek zadań okno ciemna ikona
settings-find-skip-intros =
    .words = opening op automatycznie pomiń aniskip rozdziały intro
settings-find-skip-outros =
    .words = ending ed napisy końcowe automatycznie pomiń aniskip outro
settings-find-mpv-settings =
    .words = mpv.conf konfiguracja zaawansowane mpv
settings-find-discord =
    .words = discord rich presence status profil
settings-find-next-episode =
    .words = następny przycisk tytuł autoodtwarzanie koniec
settings-find-subtitle-languages =
    .words = język ścieżka napisy
settings-find-audio-languages =
    .words = język ścieżka dubbing japoński dźwięk
settings-find-start-over =
    .words = reset domyślny styl napisów
settings-find-size =
    .words = skala napisy rozmiar
settings-find-position =
    .words = wysokość margines napisy położenie
settings-find-styled =
    .words = ass nadpisz wymuś styl
settings-find-font =
    .words = krój czcionka
settings-find-text-colour =
    .words = kolor tekst color
settings-find-outline =
    .words = obramowanie obrys kontur
settings-find-background =
    .words = tło ramka przezroczystość krycie
settings-find-bold =
    .words = grubość pogrubienie
settings-find-keys-player =
    .words = klawiatura skróty klawisze skróty klawiszowe
settings-find-keys-player-around =
    .words = klawiatura skróty pełny ekran następny poprzedni odcinek wstecz
settings-find-keys-around =
    .words = klawiatura skróty wstecz dalej prawy przycisk menu pełny ekran
settings-find-keys-anywhere =
    .words = klawiatura skróty klawisze skróty klawiszowe ustawienia zakończ wyjdź
settings-find-torrent-client =
    .words = qbittorrent hasło użytkownik adres webui seedbox klient torrent
settings-find-search-helper =
    .words = anirss rss kanały token polecenie serwer zaawansowane
settings-find-next-few =
    .words = kroczące odcinki do przodu usuń obejrzane zachowaj miejsce na dysku
settings-find-unsubscribe =
    .words = czyszczenie kanały zakończone subskrypcja anuluj
settings-find-last-search =
    .words = pobieranie zapytanie zapamiętaj wyszukiwanie
settings-find-search-sources =
    .words = endpointy kanały konfiguracja anirss źródła
settings-find-source-lists =
    .words = repo repozytorium endpoint podpisane endpointy jackett prowlarr lista źródeł
settings-find-share-tailscale =
    .words = telefon android tailnet zdalny host tailscale udostępnij
settings-find-stream =
    .words = przesyłanie odtwarzanie cast odcinki telefon pliki stream
settings-find-devices =
    .words = zaufaj zaufane wyłącz zablokuj urządzenie tailscale
settings-find-back-up =
    .words = eksport kopia zapasowa backup plik json
settings-find-restore =
    .words = import przywróć plik json
settings-find-image-cache =
    .words = plakaty wyczyść obrazy dysk pamięć podręczna cache
settings-find-files =
    .words = baza danych dane konfiguracja cache folder ścieżka pliki
settings-find-help-folders =
    .words = pomoc folder układ nazwy odcinki sezony filmy pliki struktura
settings-find-help-recognised =
    .words = pomoc dopasowanie zły serial nie znaleziono niedopasowany rozpoznany
settings-find-help-anilist =
    .words = pomoc anilist myanimelist mal zaloguj połącz logowanie serwis
settings-find-help-downloads =
    .words = pomoc pobieranie puste źródła nic torrent qbittorrent
settings-find-help-data =
    .words = pomoc dane pliki kopia zapasowa baza danych prywatność
settings-find-help-else =
    .words = pomoc konfiguracja wprowadzenie przewodnik zgłoś błąd problem zgłoszenie
settings-find-version =
    .words = wersja kompilacja build co nowego changelog lista zmian
settings-find-setup =
    .words = pierwsze uruchomienie przewodnik wprowadzenie konfiguracja od nowa
settings-find-updates =
    .words = stabilny nightly kanał sprawdź aktualizacje
settings-find-developer-options =
    .words = debug pomiar czasu deweloper
settings-find-ratings =
    .words = imdb podziękowania źródło ocen

## SettingsDownloadsTab

settings-downloads-title = Pobieranie
settings-downloads-lead = Dokąd AniBeam wysyła to, co pobierasz, i jak przeszukuje kanały wydań.
settings-downloads-keep-none = Obejrzenie odcinka usuwa poprzedni.
settings-downloads-keep-line = Obejrzane odcinki trzymane za ostatnio skończonym. Przy { $count } skończenie odcinka { $finishing } usuwa odcinek 1.
settings-downloads-anirss-ready = Działa, wersja { $version } pod { $url }.
settings-downloads-anirss-starting = Uruchamianie pomocnika wyszukiwania.
settings-downloads-anirss-missing = Pomocnik wyszukiwania nie jest zainstalowany. Zainstaluj ponownie AniBeam albo wskaż własny anirss w polu Polecenie.
settings-downloads-anirss-failed = Pomocnik wyszukiwania się zatrzymał. Spróbuj ponownie go uruchomi; Szczegóły pokazują, co powiedział.
settings-downloads-anirss-idle = Jeszcze nie uruchomiony. Uruchomi się przy pierwszym wyszukiwaniu na stronie Pobrane.
settings-downloads-torrent-trying = Próba połączenia z { $client }.
settings-downloads-torrent-answered = { $client } { $version } odpowiedział.
settings-downloads-torrent-failed = { $client } nie odpowiada. Sprawdź adres i hasło powyżej i przetestuj znowu.
settings-downloads-torrent-untried = Jeszcze nie sprawdzono.
settings-downloads-mode-existing = qBittorrent uruchamiany przez Ciebie, z włączonym Web UI.
settings-downloads-mode-managed = AniBeam uruchamia własny qBittorrent, który pobiera do ~/Downloads/Anime. Działa dalej po zamknięciu AniBeam, więc subskrypcje dalej się pobierają.
settings-downloads-mode-auto = qBittorrent spod adresu niżej lub własny.
settings-downloads-password-saved = Hasło dla tego klienta jest tu zapisane.
settings-downloads-password-anirss = Używa hasła zapisanego przez pomocnika wyszukiwania; zapisz tu własne, aby od niego nie zależeć.
settings-downloads-password-none = Brak zapisanego hasła. Klient, który pomija logowanie dla tego komputera, go nie potrzebuje.
settings-downloads-mismatch = Pomocnik wyszukiwania łączy się z { $helperUrl }, a AniBeam z { $url }. Pomocnik uruchamiany przez Ciebie czyta adres z własnego config.toml, więc zmiana klienta tutaj go nie przenosi.
settings-downloads-torrent-title = Klient torrentów
settings-downloads-torrent-helper = qBittorrent, który pobiera. Jego hasło jest w systemowym magazynie haseł.
settings-downloads-which-client = Który klient
settings-downloads-pick-for-me = Wybierz za mnie
settings-downloads-my-own = Mój własny
settings-downloads-built-in = Wbudowany
settings-downloads-saved-clients = Zapisane klienty
settings-downloads-saved-clients-helper = Na przykład seedbox i ten na tym komputerze. Przełącza między nimi też chip klienta na stronie Pobrane.
settings-downloads-add-client = Dodaj
settings-downloads-name = Nazwa
settings-downloads-name-placeholder = Nazwany od adresu
settings-downloads-address = Adres
settings-downloads-address-managed-helper = Dostęp ma tylko ten komputer i nie potrzeba hasła.
settings-downloads-address-helper = Adres Web UI.
settings-downloads-username = Nazwa użytkownika
settings-downloads-password = Hasło
settings-downloads-secret-saved = Zapisane
settings-downloads-secret-unset = Nie ustawiono
settings-downloads-connection = Połączenie
settings-downloads-open-config = Otwórz jego konfigurację
settings-downloads-details = Szczegóły
settings-downloads-details-torrent = { $client } odpowiedział tak, gdy AniBeam próbował się połączyć.
settings-downloads-test-trying = Próba
settings-downloads-test-save = Zapisz i testuj
settings-downloads-test = Testuj
settings-downloads-searching = Wyszukiwanie
settings-downloads-search-helper = Pomocnik wyszukiwania
settings-downloads-searching-helper = AniBeam sam przeszukuje Twoje źródła i wysyła wybrane do klienta torrentów. Pomocnik wyszukiwania nie jest potrzebny.
settings-downloads-search-helper-helper = Część AniBeam, która wyszukuje i subskrybuje, program o nazwie anirss. AniBeam uruchamia go za Ciebie; zmieniaj to tylko, jeśli uruchamiasz własny.
settings-downloads-runs-in = Wyszukiwanie działa w
settings-downloads-runs-in-native-helper = Subskrypcje działają w kliencie torrentów, też przy zamkniętym AniBeam.
settings-downloads-runs-in-helper = Pomocnik wyszukiwania, osobny program, który uruchamia AniBeam albo Ty.
settings-downloads-engine-native = Sam AniBeam
settings-downloads-how-runs = Jak działa
settings-downloads-how-runs-sidecar = AniBeam uruchamia go przy otwarciu i zatrzymuje przy zamknięciu.
settings-downloads-how-runs-remote = AniBeam używa tego, który już uruchamiasz, pod adresem i z kluczem poniżej.
settings-downloads-runs-sidecar = Uruchamia AniBeam
settings-downloads-runs-remote = Uruchamiam sam
settings-downloads-command = Polecenie
settings-downloads-command-helper = Nazwa w PATH albo pełna ścieżka. ~/.local/bin też jest przeszukiwany.
settings-downloads-server-address-helper = Adres bazowy serwera, bez /api/v1.
settings-downloads-key = Klucz
settings-downloads-key-saved-helper = Klucz jest zapisany. Wpisz nowy, aby go zastąpić, albo wyczyść pole, aby go usunąć.
settings-downloads-key-helper = Klucz, który pomocnik wyszukiwania pokazał przy starcie. Wpuszcza AniBeam.
settings-downloads-details-anirss = Pomocnik wyszukiwania powiedział to, gdy się zatrzymał.
settings-downloads-starting = Uruchamianie
settings-downloads-reconnect = Połącz ponownie
settings-downloads-save-first = Najpierw zapisz zmiany
settings-downloads-roll-title = Trzymaj kilka następnych w gotowości
settings-downloads-roll-helper = Długi serial pobiera się po kilka odcinków naraz, a każdy obejrzany uruchamia następny. Całość nigdy nie musi mieścić się na dysku.
settings-downloads-roll-big = Dla długich seriali
settings-downloads-roll-big-helper = Pobrania z ponad { $count } odcinkami idą same po kilka naraz. Ich wiersz w Pobranych wciąż pobierze wszystko od razu.
settings-downloads-roll-ahead = Odcinki do przodu
settings-downloads-roll-ahead-helper = Ile odcinków zostaje pobranych za tym, który oglądasz. 12 to mniej więcej sezon.
# The unit after a slider's number: "12 episodes"
settings-downloads-unit-episodes = odc.
settings-downloads-roll-over = Długi to ponad
settings-downloads-roll-over-helper = Pobrania z większą liczbą odcinków niż ta pobierają się po kilka naraz.
settings-downloads-roll-watched = Obejrzane odcinki
settings-downloads-roll-watched-delete-helper = Usuwane, gdy jesteś dalej, więc na dysku jest tylko to, co zostało do obejrzenia.
settings-downloads-roll-watched-keep-helper = Każdy plik zostaje na dysku po obejrzeniu.
settings-downloads-keep-all = Zachowaj wszystkie
settings-downloads-keep-behind = Zachowaj za
settings-downloads-subscriptions-title = Subskrypcje i wyszukiwania
settings-downloads-last-search = Zapamiętaj ostatnie wyszukiwanie
settings-downloads-last-search-helper = Strona Pobrane otwiera się z ostatnim wyszukiwaniem. Wyłączone: pusta.
settings-downloads-prune = Anuluj subskrypcje zakończonych seriali
settings-downloads-prune-helper = 16 dni po emisji ostatniego odcinka kanał serialu jest usuwany. Serial i jego pliki zostają.
settings-downloads-sources-title = Źródła wyszukiwania
settings-downloads-sources-helper = Dokąd trafia wyszukiwanie, po kolei. Puste źródło przekazuje je dalej.
settings-downloads-sources-empty = Jeszcze nie ustawiono źródeł
settings-downloads-sources-connect = Połącz pomocnika wyszukiwania, aby je wyświetlić
settings-downloads-sources-empty-helper = Dodaj listę źródeł poniżej, a jej źródła pojawią się tutaj.
settings-downloads-config-file = Plik ustawień pomocnika wyszukiwania
settings-downloads-config-native-helper = Zawiera foldery zapisu, tagi i reguły najlepszego dopasowania, których AniBeam przestrzega, gdy uruchamia pomocnika. Jego własne źródła należą tylko do pomocnika.
settings-downloads-config-helper = Zawiera źródła, foldery zapisu, tagi i reguły najlepszego dopasowania.
settings-downloads-unsaved = Niezapisane zmiany
settings-downloads-unsaved-helper = Zapisanie uruchamia ponownie pomocnika wyszukiwania z nowymi ustawieniami.
settings-downloads-discard = Odrzuć

## SettingsDeveloperTab

settings-developer-title = Deweloper
settings-developer-lead = Dlaczego strona jest wolna, zmierzone. Każdy przełącznik działa od następnej otwartej strony.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds } s
settings-developer-switches = Przełączniki
settings-developer-load-images = Wczytuj obrazy
settings-developer-load-images-helper = Wyłączone: żaden plakat, baner ani portret nie jest czytany ani dekodowany.
settings-developer-overlay = Nakładka wczytywania stron
settings-developer-overlay-helper = Ostatnie wczytanie strony i pamięć, w prawym dolnym rogu okna.
settings-developer-read-cache = Zachowuj odczyty biblioteki
settings-developer-read-cache-helper = Odpowiedź na odczyt całej biblioteki jest trzymana, dopóki biblioteka się nie zmieni. Wyłączone: każda strona pyta rdzeń ponownie.
settings-developer-poster-pool = Rozgrzewaj plakaty w tle
settings-developer-poster-pool-helper = Dekoduje 60 plakatów, które najpewniej zobaczysz, i trzyma je w pamięci.
settings-developer-options = Opcje deweloperskie
settings-developer-options-helper = Wyłącza tę zakładkę, nakładkę i pomiary.
settings-developer-memory = Pamięć
settings-developer-memory-helper = Co jądro liczy dla tego procesu. Obrazy trzymają zdekodowane piksele w pamięci i na karcie graficznej.
settings-developer-memory-unknown = Ten system nie podaje, ile pamięci używa aplikacja.
settings-developer-in-use = W użyciu
settings-developer-peak = Szczyt
settings-developer-heap = Sterta
settings-developer-files-mapped = Zmapowane pliki
settings-developer-threads = Wątki
settings-developer-give-back = Oddaj pamięć
settings-developer-give-back-helper = Porzuca to, co graf sceny może odbudować, zbiera śmieci JavaScript i oddaje wolną stertę systemowi. To, co zwolni, aplikacja trzymała bez potrzeby.
settings-developer-released = { $before } do { $after }
settings-developer-free-now = Zwolnij teraz
settings-developer-images-total = Obrazy od startu
settings-developer-images-total-line = Zdekodowano: { $decodes }, { $pixels } pikseli z { $disk } na dysku, { $ms } ms
settings-developer-loads = Wczytania stron
settings-developer-loads-helper = Zbudowano to czekanie po kliknięciu: wątek Qt nie robi nic innego, dopóki strona nie powstanie. Ustalono to moment, gdy dotarł jej ostatni obraz.
settings-developer-touring = Pomiar, { $at } z { $steps }
settings-developer-tour = Zmierz każdą stronę
settings-developer-copy-report = Kopiuj raport
settings-developer-col-page = Strona
settings-developer-col-built = Zbudowano
settings-developer-col-painted = Narysowano
settings-developer-col-settled = Ustalono
settings-developer-col-calls = Wywołania, dane
settings-developer-col-decodes = Dekodowania, czas, piksele
settings-developer-col-items = Elementy
settings-developer-col-offscreen = Obrazy poza widokiem
settings-developer-col-memory = Pamięć
settings-developer-load-no-images = { $page }, bez obrazów
settings-developer-offscreen-of = { $offscreen } z { $total }
settings-developer-loads-empty = Otwórz dowolną stronę, a pojawi się tutaj.
settings-developer-load-images-off = { $page }, obrazy wyłączone
settings-developer-load-frames = Klatki narysowane podczas wczytywania: { $frames }, najdłuższa przerwa między dwiema: { $gap }.
settings-developer-load-frames-timed-out = Klatki narysowane podczas wczytywania: { $frames }, najdłuższa przerwa między dwiema: { $gap }. Przestano czekać na obrazy po 15 s.
settings-developer-built = Zbudowano
settings-developer-settled = Ustalono
settings-developer-core-data = Dane z rdzenia
settings-developer-pixels = Zdekodowane piksele
settings-developer-page-items = Elementy na stronie
settings-developer-cached-images = Obrazy już w pamięci
settings-developer-calls = Wywołania rdzenia, od najwolniejszych
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  z pamięci
settings-developer-call = { $name }  { $ms } ms  { $size }  rdzeń { $core }, do QML { $qml }
settings-developer-decodes = Najwolniejsze dekodowania
settings-developer-decodes-line = Dekodowania jedno po drugim w wątku obrazów: { $count }, { $span } od pierwszego do ostatniego; zażądano: { $requests }, już zdekodowane: { $cached }, zablokowane: { $blocked }
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } do { $w }x{ $h }  { $size }
settings-developer-decodes-none = Nic nie dekodowano: każdy obraz był już w pamięci.
settings-developer-images-were-off = Obrazy były wyłączone.
settings-developer-scene = Z czego składa się strona
settings-developer-scene-line = Elementy: { $items }, poza oknem: { $offscreen }; teksty: { $texts }, kształty: { $shapes }, ikony: { $icons }; obrazy: { $images }, poza oknem: { $imagesOffscreen }, zajmują { $size }
settings-developer-copy-load = Kopiuj to wczytanie
settings-developer-tour-title = Każda strona, z obrazami i bez
settings-developer-tour-helper = Środkowa z trzech wizyt na każdej stronie. Gdzie obie kolumny się zgadzają, to nie obrazy spowalniają stronę.
settings-developer-col-built-on = Zbudowano, obrazy wł.
settings-developer-col-built-off = Zbudowano, obrazy wył.
settings-developer-col-settled-on = Ustalono, wł.
settings-developer-col-settled-off = Ustalono, wył.
settings-developer-col-decodes-time = Dekodowania, czas
settings-developer-col-data = Dane
settings-developer-tour-peak = Szczyt pamięci podczas pomiaru { $size }
settings-developer-events = Zdarzenia
settings-developer-events-helper = Wszystko, co mówi rdzeń, każdy poziom, od najnowszych. Każde zdarzenie jest kopiowane do wątku Qt jako JSON; rozmiar to koszt wysłania.
settings-developer-event-rate = { $kind } { $count }/min, { $size }
settings-developer-events-filter = Filtruj po słowach, rodzaju lub etapie
settings-developer-event-plain = { $message }   (zwykły: { $plain })
settings-developer-copy-events = Kopiuj zdarzenia

## SettingsKeysTab

settings-keys-title = Klawiatura
settings-keys-lead = Skróty AniBeam. Naciśnij ? gdziekolwiek, by zobaczyć skróty dla danego miejsca.
settings-keys-player = W odtwarzaczu
settings-keys-player-around = Poruszanie się w odtwarzaczu
settings-keys-around = Poruszanie się
settings-keys-anywhere = Wszędzie

## KeyHelp

key-help-title = Skróty klawiszowe
key-help-group-playback = Odtwarzanie
key-help-group-navigation = Nawigacja
key-help-group-library = Biblioteka
# Key names on the chips. " / " splits one chip into two, so keep it between keys.
key-help-key-space = Spacja / K
key-help-key-left-right = Lewo / Prawo
key-help-key-ctrl-right = Ctrl+Prawo
key-help-key-backspace = Backspace
key-help-key-up-down = Góra / Dół
key-help-key-hold = Przytrzymaj obraz
key-help-key-page = Page Up / Down
key-help-key-home-end = Home / End
key-help-key-escape = Esc
key-help-key-alt-left = Alt+Lewo
key-help-key-alt-right = Alt+Prawo
key-help-key-shift-f10 = Shift+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / lub Ctrl+F
key-help-key-ctrl-space = Ctrl+Spacja
key-help-key-shift-space = Shift+Spacja
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = Odtwórz lub wstrzymaj
key-help-seek-5 = Przewiń o 5 s
key-help-seek-10 = Przewiń o 10 s
key-help-skip = Pomiń intro lub outro, w przeciwnym razie 90 s
key-help-frame-step = Jedna klatka wstecz lub do przodu
key-help-speed = Wolniej lub szybciej, co 0,25
key-help-normal-speed = Normalna prędkość
key-help-volume = Głośność w górę lub w dół o 5
key-help-mute = Wycisz
key-help-subtitles-toggle = Wyłącz i włącz napisy
key-help-subtitle-delay = Opóźnienie napisów 100 ms wcześniej lub później
key-help-audio-delay = Opóźnienie dźwięku 100 ms wcześniej lub później
key-help-hold-speed = Prędkość 2× po przytrzymaniu
key-help-screenshot = Zapisz zrzut ekranu w Obrazach
key-help-episode = Następny lub poprzedni odcinek
key-help-chapter = Poprzedni lub następny rozdział
key-help-start-end = Początek albo outro
key-help-fullscreen = Pełny ekran
key-help-escape-player = Zamknij to, co otwarte, w przeciwnym razie wyjdź z pełnego ekranu, a potem z odtwarzacza
key-help-toggle-list = Pokaż lub ukryj tę listę
key-help-library-search = Biblioteka i wyszukiwanie
key-help-search-page = Szukaj na tej stronie
key-help-activity = Aktywność
key-help-settings = Ustawienia
key-help-quit = Zakończ
key-help-forward = Dalej
key-help-escape-app = Zamknij to, co otwarte, w przeciwnym razie wstecz
key-help-context-menu = Menu prawego przycisku
key-help-pick-card = Wybierz kartę, aby działać na kilku
key-help-pick-range = Wybierz każdą kartę aż do tej
key-help-pick-all = Wybierz każdą widoczną kartę

## SettingsAboutTab

settings-about-title = Informacje
settings-about-lead = Który to AniBeam i jak dowiaduje się o następnym.
settings-about-update-failed = Aktualizacja się nie zakończyła. Spróbuj ponownie albo pobierz ją ze strony wydania.
settings-about-update-ready = { $version } jest gotowa do instalacji.
settings-about-update-command = Wyszła { $version }. Zaktualizuj poleceniem { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = Wyszła { $version }. Ta kopia jest zbudowana ze źródeł: pobierz zmiany i przebuduj.
settings-about-update-elsewhere = Wyszła { $version }, bez pliku do pobrania dla tego komputera.
settings-about-version = Wersja
settings-about-build = AniBeam { $build }
settings-about-whats-new = Co nowego
settings-about-install = Zainstaluj { $version }
settings-about-setup = Konfiguracja
settings-about-setup-helper = Przewodnik, od którego zaczyna nowa instalacja: Twój folder, Twoja lista i pobieranie.
settings-about-run-setup = Uruchom konfigurację ponownie
settings-about-updates = Aktualizacje
settings-about-nightly-helper = Nowa kompilacja niemal codziennie. Nowsza i mniej przetestowana.
settings-about-stable-helper = Nowa wersja, gdy zostanie przetestowana i wydana.
settings-about-stable = Stabilny
settings-about-nightly = Nightly
settings-about-check-now = Sprawdź teraz
settings-about-developer = Opcje deweloperskie
settings-about-developer-helper = Dodaje sekcję Deweloper: czas każdej strony, pamięć, pełny dziennik techniczny i wyłączanie obrazów.
settings-about-credits = Podziękowania
settings-about-ratings = Oceny
settings-about-ratings-helper = Information courtesy of IMDb (https://www.imdb.com). Used with permission.
settings-about-series = Seriale i odcinki
settings-about-series-helper = AniList i TVmaze. Czasy pomijania z AniSkip.

## SettingsPlaybackTab

settings-playback-title = Odtwarzanie
settings-playback-lead = Co odtwarzacz robi sam, żebyś sięgał po niego tylko wtedy, gdy chcesz.
settings-playback-skipping = Pomijanie
settings-playback-skipping-helper = Używa rozdziałów pliku, inaczej czasów pomijania od widzów z sieci. Cofnij w odtwarzaczu wyłącza je do zamknięcia AniBeam.
settings-playback-skip-intros = Pomijaj intro
settings-playback-skip-outros = Pomijaj outro
settings-playback-skip-outros-helper = Dojście do outro oznacza też odcinek jako obejrzany.
settings-playback-next-episode = Następny odcinek
settings-playback-name-next = Pokazuj tytuł następnego odcinka
settings-playback-name-next-helper = Przycisk Dalej na końcu odcinka pokazuje tytuł następnego. Wyłączone: tylko jego numer.
settings-playback-languages = Języki
settings-playback-languages-helper = Kody języków, od najważniejszego. Wygrywa pierwszy obecny w pliku. Wybór ścieżki w odtwarzaczu jest zapamiętywany dla serialu.
settings-playback-subtitle-languages = Języki napisów
settings-playback-audio-languages = Języki dźwięku
settings-playback-discord = Pokazuj na Discordzie, co oglądasz
settings-playback-discord-check-failed = AniBeam nie mógł teraz sprawdzić Discorda. Spróbuj za chwilę.
settings-playback-discord-helper = Serial, odcinek i plakat pojawiają się na Twoim profilu Discord podczas odtwarzania. Nic nie jest wysyłane, gdy to jest wyłączone lub Discord jest zamknięty.
settings-playback-discord-unavailable = Ta wersja nie ma aplikacji Discord.
settings-playback-discord-connected = Połączono z Discordem
settings-playback-discord-not-connected = Nie połączono. AniBeam łączy się, gdy odcinek gra, a Discord jest otwarty
settings-playback-advanced = Zaawansowane
settings-playback-my-mpv = Użyj moich ustawień mpv
settings-playback-my-mpv-helper = Czyta Twój mpv.conf. Opcje działające tylko przy starcie są pomijane, a skrypty nigdy się nie wczytują.
settings-playback-no-mpv-conf = Nie znaleziono pliku ustawień mpv. Wpisz opcje w mpv.conf, a potem włącz to.

## SliderRow

# The spoken name of a slider whose number counts something: "On hold after, in days"
slider-row-spoken-unit = { $name }, w jednostkach: { $unit }
# The readout beside a slider: "60 days", "1.00 ×"
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Akcent czerwony
swatches-accent-orange = Akcent pomarańczowy
swatches-accent-yellow = Akcent żółty
swatches-accent-green = Akcent zielony
swatches-accent-cyan = Akcent turkusowy
swatches-accent-blue = Akcent niebieski

## LookPreview

look-preview-dark = Ciemny
look-preview-light = Jasny

## SettingsAccountsTab

settings-accounts-title = Konta
settings-accounts-lead = Połącz AniList lub MyAnimeList, a każdy obejrzany odcinek zostanie tam oznaczony za Ciebie. Liczby tylko rosną.
settings-accounts-syncing = Synchronizacja list
settings-accounts-synced = Listy zsynchronizowane
settings-accounts-lists = Twoje listy anime
settings-accounts-lists-helper = Bez nich AniBeam i tak pamięta, co obejrzałeś na tym komputerze. Lista na AniList lub MyAnimeList przenosi to na Twój telefon i profil.
settings-accounts-progress = Postęp
settings-accounts-main-list = Główna lista
settings-accounts-main-list-helper = Karty pokazują postęp z tej listy. Druga i tak dostaje każdą aktualizację.
settings-accounts-local-lists = Listy na tym komputerze
settings-accounts-local-lists-helper =
    { $count ->
        [one] Jeden serial ma
        [few] { $count } seriale mają
        [many] { $count } seriali ma
       *[other] { $count } seriali ma
    } status lub ocenę tylko tutaj. Przenieś je do { $tracker }, aby widzieć je na telefonie i w profilu.
settings-accounts-review-move = Przejrzyj i przenieś
settings-accounts-nothing-to-move = Nic do przeniesienia
settings-accounts-sync-now = Synchronizuj teraz
settings-accounts-sync-now-helper = Ponownie czyta każdą połączoną listę. AniBeam robi to też przy starcie.
settings-accounts-sync = Synchronizuj
settings-accounts-move-on-hold = Przenoś do Wstrzymanych
settings-accounts-move-on-hold-helper = Serial z listy Oglądam, którego od dawna nie ruszałeś, trafia do Wstrzymanych na Twoich listach. Odtworzenie go przywraca. Serial czekający na następny odcinek zostaje na miejscu.
settings-accounts-after = Po
settings-accounts-after-line = Dni bez odtworzenia odcinka
settings-accounts-days-unit = dni

## SettingsPhoneTab

settings-phone-title = Aplikacja na telefon
settings-phone-lead = AniBeam na telefonie może odtwarzać tę bibliotekę, pokazywać, co jest w emisji, i śledzić Twoje pobrania, gdziekolwiek jesteś.
settings-phone-online = Online
settings-phone-offline = Offline
settings-phone-untrusted = Nic nie dostaje z tego komputera
settings-phone-sharing = Udostępnianie
settings-phone-sharing-helper = Dostęp mają tylko Twoje urządzenia w Tailscale. Odcinki obejrzane na telefonie zachowują tu miejsce i postęp.
settings-phone-share = Udostępniaj przez Tailscale
settings-phone-stream = Przesyłaj odcinki
settings-phone-stream-on = Twoje urządzenia mogą odtwarzać i przesyłać odcinki z tego komputera.
settings-phone-stream-off = Twoje urządzenia widzą bibliotekę i to, co jest w emisji, ale nie mogą stąd odtwarzać.
settings-phone-host-service = Usługa hosta
settings-phone-not-running = Nie działa
settings-phone-running = Działa
settings-phone-service-off-helper = Uruchom polecenie raz, a urządzenia dotrą do AniBeam od logowania, bez otwierania okna.
settings-phone-service-on-helper = Twoje urządzenia mają dostęp do AniBeam bez otwartego okna. Gdy to okno jest otwarte, udostępnia zamiast usługi.
settings-phone-copy-command = Kopiuj polecenie
settings-phone-command-copied = Skopiowano polecenie
# Tailscale ties the device to no person, so this computer cannot tell whose it is.
settings-phone-nobody = Nic nie dostaje: Tailscale nie przypisuje go do osoby
# The people whose devices this computer serves, its own person first.
settings-phone-people = Osoby
# The chip on the person this computer belongs to.
settings-phone-own-profile = Profil tego komputera
# The owner's line: this computer plus how many of their other devices it serves.
settings-phone-person-own-devices =
    { $count ->
        [0] Ten komputer
        [one] Ten komputer i { $count } urządzenie
        [few] Ten komputer i { $count } urządzenia
        [many] Ten komputer i { $count } urządzeń
       *[other] Ten komputer i { $count } urządzenia
    }
# Anyone else's line: how many of their devices this computer serves.
settings-phone-person-devices =
    { $count ->
        [one] { $count } urządzenie
        [few] { $count } urządzenia
        [many] { $count } urządzeń
       *[other] { $count } urządzenia
    }
settings-phone-devices = Urządzenia w Twoim tailnecie
settings-phone-devices-helper = Gdy udostępnianie działa, każde urządzenie w tailnecie ma dostęp do tego komputera. Wyłączone nic stąd nie dostanie.
settings-phone-addresses = Adresy
settings-phone-addresses-helper = Wpisz jeden z nich w aplikacji na telefonie, gdy pierwszy raz zapyta.
settings-phone-copy = Kopiuj
settings-phone-address-copied = Skopiowano adres

## SettingsHelpTab

settings-help-title = Pomoc
settings-help-lead = Krótkie odpowiedzi na częste pytania, każda prowadzi prosto do rozwiązania.
settings-help-folders = Jak powinny wyglądać moje foldery?
settings-help-folders-helper = Jeden folder na serial, z odcinkami w środku. Nazwij każdy plik numerem odcinka, jak w wydaniach: "Frieren - 01.mkv" albo "Frieren S01E01.mkv". Drugi sezon to osobny folder albo folder "Season 2" w folderze serialu. Filmy idą do folderu Movies, po jednym pliku.
settings-help-for-example = Na przykład
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = Jeszcze nie dodano folderu
settings-help-reads-folders =
    { $count ->
        [one] AniBeam czyta jeden folder
        [few] AniBeam czyta { $count } foldery
        [many] AniBeam czyta { $count } folderów
       *[other] AniBeam czyta { $count } folderu
    }
settings-help-add-folder-helper = Dodaj folder, w którym są Twoje seriale, a nie każdy serial osobno.
settings-help-add-folder = Dodaj folder
settings-help-open-folders = Otwórz foldery
settings-help-match = Dlaczego mój serial nie został rozpoznany?
settings-help-match-helper = AniBeam wyszukuje każdy folder po nazwie, najpierw w AniList, potem w TVmaze. Dodatkowe słowa, literówka albo nazwa w innym języku mogą nie trafić. Wybierz właściwy serial ręcznie w Informacjach; Twój wybór nigdy nie zostanie zastąpiony.
settings-help-all-recognised = Każdy serial jest rozpoznany
settings-help-unrecognised =
    { $count ->
        [one] Jeden serial nie jest rozpoznany
        [few] { $count } seriale nie są rozpoznane
        [many] { $count } seriali nie jest rozpoznanych
       *[other] { $count } seriali nie jest rozpoznanych
    }
settings-help-open-show-info = Otwórz Informacje
settings-help-connect = Jak połączyć AniList?
settings-help-connect-helper = Połącz go, a każdy obejrzany odcinek zostanie oznaczony na Twojej liście, więc telefon i profil są na bieżąco. AniList prosi, żebyś raz utworzył małą własną aplikację; AniBeam przeprowadzi Cię przez to, a zajmie to około minuty. MyAnimeList działa tak samo.
settings-help-both-connected = Oba są połączone
settings-help-anilist-connected = AniList jest połączony
settings-help-mal-connected = MyAnimeList jest połączony
settings-help-none-connected = Żaden nie jest jeszcze połączony
settings-help-connect-anilist = Połącz AniList
settings-help-connect-mal = Połącz MyAnimeList
settings-help-downloads = Dlaczego Pobrane są puste?
settings-help-downloads-helper = AniBeam nie ma żadnych miejsc do przeszukiwania. Dodaj listę źródeł w Ustawieniach, w sekcji Pobieranie, i zainstaluj z niej wybrane źródła. Pobieranie wymaga też klienta torrentów: AniBeam może uruchomić własny albo użyć qBittorrent, który już masz.
settings-help-sources = Źródła i klient torrentów
settings-help-open-downloads = Otwórz ustawienia Pobierania
settings-help-data = Gdzie są moje dane?
settings-help-data-helper = Biblioteka, historia oglądania i ustawienia zostają na tym komputerze. Logowania do AniList i MyAnimeList trafiają do systemowego magazynu haseł, a bez niego do pliku tylko dla Ciebie. Kopię wszystkiego w jednym pliku zrobisz w Pamięć i kopia.
settings-help-library-history = Biblioteka i historia
settings-help-show-folder = Pokaż folder
settings-help-backup = Kopia lub przywracanie
settings-help-open-storage = Otwórz Pamięć i kopia
settings-help-more = Coś innego?
settings-help-more-helper = Powtórz pierwszą konfigurację albo opisz problem. Zrzut ekranu bardzo pomaga.
settings-help-first-setup = Pierwsza konfiguracja
settings-help-first-setup-helper = Twój folder, Twoja lista i pobieranie, krok po kroku.
settings-help-run-setup = Uruchom konfigurację ponownie
settings-help-report = Zgłoś problem
settings-help-report-helper = Otwiera zgłoszenie błędu na GitHub w przeglądarce, z wpisaną tą wersją.

## SettingsSubtitlesTab

settings-subtitles-title = Napisy
settings-subtitles-lead = Od tego zaczyna każdy odcinek. Zmiany widać na kadrze poniżej na bieżąco.
settings-subtitles-reset-done = Napisy wyglądają jak na początku
settings-subtitles-size-place = Rozmiar i miejsce
settings-subtitles-size = Rozmiar
settings-subtitles-position = Położenie
settings-subtitles-position-helper = 100 to dolna krawędź. Mniejsze liczby podnoszą napisy wyżej w kadrze.
settings-subtitles-style = Styl
settings-subtitles-styled = Napisy ze stylem
settings-subtitles-as-made-helper = Pliki ze stylem (ASS) wyglądają dokładnie tak, jak ustawili ich autorzy.
settings-subtitles-mine-helper = Twój styl poniżej trafia na każdy plik. Znaki i karaoke mogą się zepsuć.
settings-subtitles-size-only-helper = Napisy ze stylem zachowują wygląd i biorą Twój rozmiar. Zwykłe pliki biorą Twój styl.
settings-subtitles-as-made = Oryginał
settings-subtitles-size-only = Rozmiar
settings-subtitles-mine = Mój
settings-subtitles-font = Czcionka
settings-subtitles-font-invalid = Wpisz nazwę czcionki, np. Noto Sans.
settings-subtitles-bold = Pogrubienie
settings-subtitles-text-colour = Kolor tekstu
settings-subtitles-colour-invalid = Wpisz kolor, np. #ffcc00.
settings-subtitles-outline = Obrys
settings-subtitles-outline-invalid = Szerokość to liczba, 0 lub więcej.
settings-subtitles-outline-colour-invalid = Wpisz kolor, np. #000000.
settings-subtitles-outline-helper = Najpierw szerokość, potem kolor.
settings-subtitles-shadow = Cień
settings-subtitles-shadow-invalid = Liczba, 0 lub więcej. 0 oznacza brak cienia.
settings-subtitles-shadow-helper = Jak daleko pada cień, 0 oznacza brak.
settings-subtitles-box = Tło napisów
settings-subtitles-box-helper = Ciemne tło za tekstem. 0 je wyłącza.
settings-subtitles-start-over = Od nowa
settings-subtitles-start-over-helper = Przywraca rozmiar, czcionkę, kolory, obrys, tło i położenie do ustawień początkowych.
settings-subtitles-reset = Resetuj

## SettingsDataTab

settings-data-title = Pamięć i kopia
settings-data-lead = Przenieś bibliotekę na inny komputer, zrób kopię albo zwolnij miejsce po obrazach.
settings-data-backed-up = Kopia zapisana w { $path }
settings-data-restore-finished = Przywracanie zakończone
settings-data-images-removed =
    { $count ->
        [one] Usunięto { $count } obraz
        [few] Usunięto { $count } obrazy
        [many] Usunięto { $count } obrazów
       *[other] Usunięto { $count } obrazu
    }
# What the last restore did: the parts below, joined with commas, make one sentence
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] dodano { $count } folder
        [few] dodano { $count } foldery
        [many] dodano { $count } folderów
       *[other] dodano { $count } folderu
    }
settings-data-summary-series = utworzone seriale: { $count }
settings-data-summary-matches =
    { $count ->
        [one] zastosowano { $count } dopasowanie
        [few] zastosowano { $count } dopasowania
        [many] zastosowano { $count } dopasowań
       *[other] zastosowano { $count } dopasowania
    }
settings-data-summary-views =
    scalono { $views ->
        [one] { $views } wyświetlenie
        [few] { $views } wyświetlenia
        [many] { $views } wyświetleń
       *[other] { $views } wyświetlenia
    } i { $episodes ->
        [one] { $episodes } obejrzany odcinek
        [few] { $episodes } obejrzane odcinki
        [many] { $episodes } obejrzanych odcinków
       *[other] { $episodes } obejrzanego odcinka
    }
settings-data-summary-resume-points =
    { $count ->
        [one] scalono { $count } punkt wznowienia
        [few] scalono { $count } punkty wznowienia
        [many] scalono { $count } punktów wznowienia
       *[other] scalono { $count } punktu wznowienia
    }
settings-data-summary-accounts =
    { $count ->
        [one] przeniesiono { $count } konto listy
        [few] przeniesiono { $count } konta list
        [many] przeniesiono { $count } kont list
       *[other] przeniesiono { $count } konta list
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } plik ma Twoją poprawkę.
        [few] { $count } pliki mają Twoją poprawkę.
        [many] { $count } plików ma Twoją poprawkę.
       *[other] { $count } pliku ma Twoją poprawkę.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } folderu nie znaleziono na tym komputerze.
        [few] { $count } folderów nie znaleziono na tym komputerze.
        [many] { $count } folderów nie znaleziono na tym komputerze.
       *[other] { $count } folderu nie znaleziono na tym komputerze.
    }
settings-data-summary-left-out = Pominięto: { $fields }.
settings-data-backup = Kopia
settings-data-back-up-library = Zrób kopię biblioteki
settings-data-back-up-helper = Jeden plik z folderami, dopasowaniami i historią oglądania.
settings-data-last-written = Ostatnio zapisano w { $path }
settings-data-open-folder = Otwórz folder
settings-data-back-up = Zrób kopię
settings-data-private = Dołącz dane prywatne
settings-data-private-helper = Dodaje logowania do AniList i MyAnimeList, historię oglądania i preferencje. Logowania są zapisane zwykłym tekstem, więc zachowaj plik dla siebie.
settings-data-restore-from = Przywróć z kopii
settings-data-restore-helper = Wczytuje kopię do tej biblioteki. Wygrywa to, co jest na dysku; plik uzupełnia resztę.
settings-data-restore = Przywróć
settings-data-image-cache = Pamięć obrazów
settings-data-image-cache-helper = Plakaty, banery i portrety są na dysku, więc biblioteka działa offline. Usunięte obrazy pobiorą się znów, gdy będą potrzebne.
settings-data-cache-unmeasured = Nie udało się zmierzyć pamięci obrazów
settings-data-cache-size =
    { $count ->
        [one] { $size } w { $count } obrazie
        [few] { $size } w { $count } obrazach
        [many] { $size } w { $count } obrazach
       *[other] { $size } w { $count } obrazu
    }
settings-data-cache-empty = Brak zapisanych obrazów
settings-data-cache-unmeasured-helper = Spróbuj za chwilę. Same obrazy są w porządku.
settings-data-remove-images =
    { $count ->
        [one] Usunąć { $count } obraz?
        [few] Usunąć { $count } obrazy?
        [many] Usunąć { $count } obrazów?
       *[other] Usunąć { $count } obrazu?
    }
settings-data-files = Gdzie AniBeam trzyma pliki
settings-data-files-database = Baza biblioteki
settings-data-files-data = Dane
settings-data-files-settings = Ustawienia
settings-data-files-cache = Pamięć podręczna
settings-data-backup-filter = Kopia AniBeam (*.json)
settings-data-all-files-filter = Wszystkie pliki (*)
settings-data-backing-up = Tworzenie kopii
settings-data-restoring = Przywracanie

## SettingsLibraryTab

settings-library-title = Biblioteka
settings-library-lead = Foldery obserwowane przez AniBeam i to, jak rozpoznaje, co w nich jest.
settings-library-not-scanned = Jeszcze nie skanowano
settings-library-scan-nothing-changed = Nic się nie zmieniło
settings-library-scan-added = Dodano: { $count }
settings-library-scan-changed = Zmieniono: { $count }
settings-library-scan-removed = Usunięto: { $count }
# The last scan of one folder: "Anime: 3 added, 1 removed"
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = Anime: seriale, filmy i ich powiązania. To, co oglądasz, jest oznaczane w Twoich serwisach.
settings-library-tvmaze-about = Inna telewizja: aktorskie, kreskówki, dokumenty. Nic nie jest oznaczane w serwisie.
settings-library-tried-first = Najpierw
settings-library-tried-second = Jako drugi
settings-library-tried-third = Jako trzeci
settings-library-tried-fourth = Jako czwarty
settings-library-tried-nth = Jako { $number }.
settings-library-folders = Foldery
settings-library-folders-helper = Każdy folder w jednym z nich to serial. Plik na najwyższym poziomie folderu Movies to film.
settings-library-rescan-started = Ponowne skanowanie ruszyło
settings-library-load-failed = Nie udało się odczytać folderów
settings-library-load-failed-helper = AniBeam nie mógł odczytać listy folderów. Spróbuj za chwilę.
settings-library-no-folders = Jeszcze brak folderów
settings-library-no-folders-helper = Wskaż AniBeam folder z anime. Sam znajdzie seriale i filmy.
settings-library-scanned = Skanowano { $when }
settings-library-add-a-folder = Dodaj folder
settings-library-scan-all = Skanuj wszystko
settings-library-scan-started = Skanowanie ruszyło
settings-library-add-folder = Dodaj folder
settings-library-matching = Dopasowywanie
settings-library-matching-helper = Dla nowego folderu wygrywa pierwsze dobre dopasowanie w tej kolejności. Ręcznego dopasowania nic nie zastąpi.
settings-library-try-later = Później
settings-library-try-first = Najpierw
settings-library-check-matches = Sprawdź dopasowania
settings-library-check-matches-helper = Strona Informacje pokazuje, co AniBeam znalazł dla każdego serialu, i pozwala poprawić błędy.
settings-library-open-show-info = Otwórz Informacje
settings-library-hidden-shows = Ukryte seriale
settings-library-show-hidden = Pokaż ukryte seriale
settings-library-show-hidden-helper = Przywraca ukryte seriale na każdą stronę do zamknięcia AniBeam.
settings-library-hidden-from-airing = Ukryte w Teraz w emisji
settings-library-hidden-from-airing-helper = Seriale oznaczone jako nieinteresujące. Każdy jest ukryty w Teraz w emisji, dopóki go nie przywrócisz.
settings-library-hidden-from-airing-empty = Jeszcze nic. Seriale oznaczone w Teraz w emisji jako nieinteresujące trafiają tutaj, więc każdy może wrócić.
settings-library-show-again = Pokaż ponownie
settings-library-calendar = Kalendarz
settings-library-calendar-feed = Godziny emisji w kalendarzu
settings-library-calendar-feed-helper = Prowadzi plik kalendarza z nadchodzącymi odcinkami seriali z Twoich list, aktualizowany przy zmianie harmonogramu.
settings-library-calendar-file = Plik kalendarza
settings-library-calendar-file-helper = Zasubskrybuj ten plik w swojej aplikacji kalendarza.
settings-library-copy = Kopiuj
settings-library-path-copied = Skopiowano ścieżkę
settings-library-open-folder = Otwórz folder
settings-library-notifications = Powiadomienia
settings-library-new-episodes = Nowe odcinki
settings-library-new-episodes-helper = Powiadomienie na pulpicie, gdy wychodzi odcinek serialu z Twojej listy Oglądam. Kliknij je, aby otworzyć serial.
settings-library-folder-added = Dodano folder, skanowanie

## SettingsAppearanceTab

settings-appearance-language = Język
# $language is the system's language in its own name ("Dansk")
settings-appearance-language-system = Jak w systemie ({ $language })
settings-appearance-text-smaller = Mniejszy
settings-appearance-text-normal = Normalny
settings-appearance-text-large = Duży
settings-appearance-text-larger = Większy
settings-appearance-text-largest = Największy
settings-appearance-title = Wygląd
settings-appearance-lead = Zmiany widać od razu, tutaj i na każdej stronie. Są zapisane w ~/.config/anibeam/theme.toml, który możesz też edytować ręcznie.
settings-appearance-colours = Kolory
settings-appearance-mode = Tryb
settings-appearance-mode-dark = Ciemny
settings-appearance-mode-light = Jasny
settings-appearance-mode-system = Systemowy
settings-appearance-colours-from = Kolory z
settings-appearance-from-theme-helper = Para motywów poniżej, jeden ciemny i jeden jasny.
settings-appearance-from-desktop-helper = Kolory Twojego terminala, a jeśli ich nie ma, schemat kolorów i akcent pulpitu.
settings-appearance-from-desktop = Pulpit
settings-appearance-from-theme = Motyw
settings-appearance-dark-theme = Ciemny motyw
settings-appearance-light-theme = Jasny motyw
settings-appearance-themes-helper = Pliki motywów wrzucone do ~/.config/anibeam/themes pojawiają się na obu listach.
settings-appearance-accent = Akcent
settings-appearance-accent-helper = Oznacza to, co jest wybrane, odtwarzane lub nowe.
settings-appearance-icon = Ikona
settings-appearance-app-icon = Ikona aplikacji
settings-appearance-app-icon-helper = Okna, paska zadań i launchera. Ciemna zamienia żółty z czarnym.
settings-appearance-icon-default = Domyślna
settings-appearance-icon-default-name = Domyślna ikona aplikacji
settings-appearance-icon-dark = Ciemna
settings-appearance-icon-dark-name = Ciemna ikona aplikacji
settings-appearance-size-motion = Rozmiar i ruch
settings-appearance-text-size = Rozmiar tekstu
settings-appearance-text-size-helper = Jak duży jest tekst w całym AniBeam, dodatkowo do rozmiaru ustawionego w systemie.
settings-appearance-density = Gęstość
settings-appearance-density-helper = Ile miejsca wokół wszystkiego, od ciasno do przestronnie.
settings-appearance-density-compact = Kompaktowa
settings-appearance-density-normal = Normalna
settings-appearance-density-comfortable = Przestronna
settings-appearance-poster-size = Rozmiar plakatów
settings-appearance-poster-size-helper = Jak duży jest serial w bibliotece i na Starcie.
settings-appearance-poster-small = Mały
settings-appearance-poster-medium = Średni
settings-appearance-poster-large = Duży
settings-appearance-out-now = Już wyszło w Emisji
settings-appearance-out-now-helper = Karty z banerem każdego serialu albo lista małych plakatów i tytułów, w trzech rzędach.
settings-appearance-out-now-cards = Karty
settings-appearance-out-now-list = Lista
settings-appearance-out-now-grid = Karty Już wyszło
settings-appearance-out-now-grid-helper = Ile kart widać, kolumny na rzędy, zanim Wszystkie otworzy resztę.
# A grid of cards, columns by rows: "4 × 2"
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Ogranicz ruch
settings-appearance-reduce-motion-helper = Pomija otwierające przejście biblioteki i inny ruch, który jest tylko na pokaz.

## LookPane

look-pane-sample-series = Przykładowy serial
look-pane-episode-code = Odc. { $number }
look-pane-library = Biblioteka
look-pane-series-count =
    { $count ->
        [one] { $count } serial
        [few] { $count } seriale
        [many] { $count } seriali
       *[other] { $count } serialu
    }
look-pane-search = Szukaj tytułów lub folderów
look-pane-search-hint = / lub Ctrl+K
look-pane-all = Wszystko
look-pane-series = Seriale
look-pane-movies = Filmy
look-pane-plain = Zwykły
look-pane-selected = Wybrany
look-pane-errors =
    { $count ->
        [one] { $count } błąd
        [few] { $count } błędy
        [many] { $count } błędów
       *[other] { $count } błędu
    }
look-pane-type-large = Duży
look-pane-type-normal = Normalny
look-pane-type-small = Mały, przygaszony
look-pane-sample-switch = Przykładowy przełącznik
look-pane-button = Przycisk
look-pane-scan = Skanuj
look-pane-sample-status =
    { $count ->
        [one] { $count } nowy odcinek w { $title }
        [few] { $count } nowe odcinki w { $title }
        [many] { $count } nowych odcinków w { $title }
       *[other] { $count } nowego odcinka w { $title }
    }

## FeedPage

feed-title = Aktualności
feed-aired = { $episode } wyemitowany { $when }
feed-downloaded = Plik pobrany { $when }
feed-countdown-days = { $days } d { $hours } h
feed-countdown-hours = { $hours } h { $minutes } min
feed-countdown-minutes = { $minutes } min
feed-section-this-week = Ten tydzień
feed-new-episodes =
    { $count ->
        [one] { $count } nowy odcinek
        [few] { $count } nowe odcinki
        [many] { $count } nowych odcinków
       *[other] { $count } nowego odcinka
    }
feed-section-earlier = Wcześniej
feed-latest-out = Ostatni wyszedł { $when }
feed-section-coming-soon = Wkrótce
feed-this-week-count = W tym tygodniu: { $count }
feed-section-everything-else = Wszystko inne
feed-out-this-week = Wychodzi w tym tygodniu: { $count }
feed-nothing-scheduled = Jeszcze nic nie zaplanowano.
feed-next = Następny: { $title } za { $countdown }
feed-nothing-new = Nic nowego w tym tygodniu.
feed-new-this-week =
    { $count ->
        [one] { $count } nowy odcinek w tym tygodniu.
        [few] { $count } nowe odcinki w tym tygodniu.
        [many] { $count } nowych odcinków w tym tygodniu.
       *[other] { $count } nowego odcinka w tym tygodniu.
    }
feed-sort-recent = Ostatnio wydane
feed-sort-upcoming = Wkrótce
feed-failed-title = Aktualności się nie wczytały
feed-empty-title = Jeszcze nic tu nie ma
feed-failed-body = AniBeam nie mógł teraz odczytać aktualności. Spróbuj za chwilę.
feed-empty-body-folders = Aktualności wypełniają się, gdy odcinki są emitowane i przychodzą. Teraz w emisji pokazuje, co leci w tym sezonie.
feed-empty-body = Dodaj folder z anime, a aktualności wypełnią się jego odcinkami.
feed-add-folder = Dodaj folder
feed-airing-now = Teraz w emisji

## RecommendedPage

recommended-title = Polecane dla Ciebie
recommended-ranked-minutes = Ułożono { $minutes } min temu
recommended-ranked-hours = Ułożono { $hours } godz. temu
recommended-ranked-days = Ułożono { $days } d temu
recommended-scope-name = Polecane z
recommended-scope-all = Wszystko
recommended-scope-library = W bibliotece
recommended-scope-list = Na liście
recommended-ranking = Układanie
recommended-rank-again = Ułóż ponownie
recommended-no-scores-title = Najpierw oceń kilka seriali
recommended-no-list-title = Jeszcze nie ma z czego się uczyć
recommended-never-fitted-title = Układanie Twoich seriali
recommended-nothing-left-title = Nic tu nie zostało
recommended-no-scores-body = Polecane uczą się z ocen na Twojej liście. Oceń kilka seriali na AniList, a pojawią się tutaj.
recommended-no-list-body = Połącz AniList w Ustawieniach albo prowadź listę na tym urządzeniu, a oceniane seriale nauczą tę stronę.
recommended-never-fitted-body = Pierwsze układanie czyta AniList przez minutę lub dwie. Działa w tle.
recommended-nothing-left-library-body = Wszystko pasujące w Twojej bibliotece jest obejrzane. Wszystko znajduje seriale, których jeszcze nie masz.
recommended-nothing-left-body = Wszystko pasujące na Twoich listach jest obejrzane. Wszystko znajduje seriale spoza list.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] Przenoszenie 1 serialu do { $site }
        [few] Przenoszenie { $count } seriali do { $site }
        [many] Przenoszenie { $count } seriali do { $site }
       *[other] Przenoszenie { $count } seriali do { $site }
    }
move-lists-score = ocena { $score }
move-lists-title = Przenieś swoją listę do { $site }
move-lists-body =
    { $count ->
        [one] Jeden serial ma
        [few] { $count } seriale mają
        [many] { $count } seriali ma
       *[other] { $count } seriali ma
    } status lub ocenę tylko na tym komputerze. Przeniesienie umieści je na Twojej liście w { $site }, gdzie zobaczą je telefon i profil. Dla tych seriali to, co jest teraz w { $site }, zostanie zastąpione. Nic nie zostanie wysłane, dopóki nie klikniesz Przenieś.
move-lists-head-show = Serial
move-lists-head-here = Na tym komputerze
move-lists-head-tracker = Teraz w { $site }
move-lists-nothing = Brak
move-lists-not-on-list = Poza listą
move-lists-later-hint = Później: Ustawienia, Konta
move-lists-not-now = Nie teraz
move-lists-move =
    { $count ->
        [one] Przenieś 1 serial
        [few] Przenieś { $count } seriale
        [many] Przenieś { $count } seriali
       *[other] Przenieś { $count } seriali
    }

## StatsPage

stats-title = Statystyki
stats-month-jan = Sty
stats-month-feb = Lut
stats-month-mar = Mar
stats-month-apr = Kwi
stats-month-may = Maj
stats-month-jun = Cze
stats-month-jul = Lip
stats-month-aug = Sie
stats-month-sep = Wrz
stats-month-oct = Paź
stats-month-nov = Lis
stats-month-dec = Gru
stats-nothing-yet = Jeszcze nic
stats-ranked-episodes =
    { $count ->
        [one] { $count } odc.
        [few] { $count } odc.
        [many] { $count } odc.
       *[other] { $count } odc.
    }
stats-year-summary = Co obejrzałeś w { $year }
stats-empty-title = Jeszcze nic nie obejrzano
stats-empty-body = Obejrzyj odcinek, a zaliczy się tutaj: godziny, gatunki i sezony, które oglądałeś.
stats-open-library = Biblioteka
stats-hours-watched = godzin oglądania
stats-episodes-label =
    { $count ->
        [one] odcinek
        [few] odcinki
        [many] odcinków
       *[other] odcinka
    }
stats-series-finished = ukończone seriale
stats-none = Brak
stats-busiest-month = najaktywniejszy miesiąc
stats-by-month = Według miesięcy
stats-month-hours = { $hours } h
stats-top-genres = Ulubione gatunki
stats-top-studios = Ulubione studia
stats-seasons-watched = Obejrzane sezony
stats-recap-title = Mój rok w anime
stats-recap-hours = godzin
stats-recap-episodes = odcinków
stats-recap-top-genre = ulubiony gatunek
stats-save-image = Zapisz jako obraz
stats-save-dialog-title = Zapisz podsumowanie
stats-save-filter-png = Obraz PNG (*.png)
stats-recap-saved = Zapisano podsumowanie
stats-recap-save-failed = Nie udało się zapisać podsumowania

## TrackerRow

tracker-row-connected = Połączono jako { $user }, synchronizacja { $when }
tracker-row-unknown-user = ?
tracker-row-never = nigdy
tracker-row-waiting = Czekam, aż zatwierdzisz w przeglądarce
tracker-row-not-connected = Nie połączono
tracker-row-open-again = Otwórz ponownie
tracker-row-open-again-tip = Otwórz ponownie stronę logowania
tracker-row-disconnect = Rozłącz
tracker-row-disconnect-question = Rozłączyć { $name }? Twoje logowanie zostanie usunięte.
tracker-row-secret-saved = Client secret dla tej aplikacji jest zapisany.
tracker-row-remove-secret = Usuń go
tracker-row-secret-removed = Usunięto client secret
tracker-row-add-secret = Dodaj client secret
tracker-row-secret-where = To długi ciąg pod "Client Secret" na stronie aplikacji w myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Client secret
tracker-row-save-connect = Zapisz i połącz

## RemoveSheet

remove-unsubscribe-delete = Anuluj subskrypcję i usuń
remove-unsubscribe-torrents = Anuluj subskrypcję i usuń torrenty
remove-unsubscribe = Anuluj subskrypcję
remove-delete-files = Usuń razem z plikami
remove-torrents =
    { $count ->
        [one] Usuń torrent
        [few] Usuń torrenty
        [many] Usuń torrenty
       *[other] Usuń torrenty
    }
remove-also-torrents =
    { $count ->
        [one] Usuń też jego torrent
        [few] Usuń też jego { $count } torrenty
        [many] Usuń też jego { $count } torrentów
       *[other] Usuń też jego { $count } torrenta
    }
remove-torrents-go-with-files = Usunięcie plików zabiera ze sobą torrenty.
remove-torrents-files-stay = Znikają z klienta; pliki zostają.
remove-also-files = Usuń też pliki
remove-files-default-where = to, co pobrały torrenty
remove-files-detail-rule = Przenosi { $where } do kosza i usuwa torrenty.
remove-files-detail = Przenosi { $where } do kosza.

## NotesSheet

notes-your-tracker = Twój serwis
notes-title = Notatki
notes-placeholder = Na czym stanęło, kto polecił, które wydanie pobrać
notes-over-limit = Za dużo znaków: { $count }
notes-saved-to = Zapisano w { $sites }

## RecommendationCard

recommendation-card-available = Dostępne
recommendation-card-open-library = Otwórz { $title } w bibliotece
recommendation-card-open-anilist = Otwórz { $title } w AniList

## SearchField

search-field-placeholder = Szukaj tytułów lub folderów
search-field-hint = / lub Ctrl+K
search-field-clear = Wyczyść wyszukiwanie

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent }%
recommended-hover-hidden-tags =
    { $count ->
        [one] Ukryty tag, który lubisz
        [few] { $count } ukryte tagi, które lubisz
        [many] { $count } ukrytych tagów, które lubisz
       *[other] { $count } ukrytego tagu, który lubisz
    }
recommended-hover-because = Bo oglądałeś

## PersonCard

person-card-unknown = Nieznany
person-card-role-main = Główna
person-card-role-supporting = Drugoplanowa
person-card-role-background = Tło
person-card-the-character = postać
person-card-open = Otwórz { $name } w AniList

## StatusDot

status-dot-on-list = Na Twojej liście: { $status }

## TrackerNudge

tracker-nudge-title = Połącz swoją listę anime
tracker-nudge-body = AniBeam oznacza odcinki na AniList lub MyAnimeList, gdy oglądasz.
tracker-nudge-connect-anilist = Połącz AniList
tracker-nudge-connect-mal = Połącz MyAnimeList
tracker-nudge-not-now = Nie teraz

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } procent dla Ciebie
recommended-card-forecast = { $percent }%
recommended-card-in-library = W bibliotece

## InlineConfirm

inline-confirm-keep = Zostaw

## Tokens

tokens-cube-watched-tracked = Obejrzane, zapisane online
    .inline = obejrzane, zapisane online
tokens-cube-watched-here = Obejrzane tutaj, jeszcze nie zapisane
    .inline = obejrzane tutaj, jeszcze nie zapisane
tokens-cube-on-disk = Na dysku
    .inline = na dysku
tokens-cube-downloading = Pobieranie
    .inline = pobieranie
tokens-cube-missing = Brak
    .inline = brak
tokens-cube-error = Błąd
    .inline = błąd
tokens-cube-not-downloaded = Nie pobrano
    .inline = nie pobrano
tokens-cube-not-aired = Jeszcze nie wyemitowany
    .inline = jeszcze nie wyemitowany
tokens-cube-available = Dostępny
    .inline = dostępny
tokens-cube-picked = Wybrany do pobrania
    .inline = wybrany do pobrania
tokens-cube-already-on-disk = Już na dysku
    .inline = już na dysku
tokens-cube-no-release = Brak wydania
    .inline = brak wydania

## Card

card-not-interested = Nie interesuje

## PagePlaceholder

page-placeholder-coming-soon = Wkrótce

## ErrorLine

error-line-hide-details = Ukryj szczegóły
error-line-details = Szczegóły
error-line-details-tip = Pokaż dokładnie, co poszło nie tak, i skopiuj to
error-line-copied = Skopiowano.

## Readouts (apps/linux/src/format.rs): times, countdowns and lengths

fmt-just-now = przed chwilą
fmt-now = teraz
fmt-ago = { $span } temu
fmt-in = za { $span }
fmt-seconds-short = { $n } s
fmt-minutes-short = { $n } min
fmt-hours-short = { $n } h
fmt-days-short = { $n } d
fmt-months-short = { $n } mies.
fmt-years-short = { $n } r.
fmt-years-months-short = { $years } r. { $months } mies.
# $h, $m and $s arrive as two digits ("04") after the first unit.
fmt-countdown-days = { $d } d { $h } h { $m } min
fmt-countdown-hours = { $h } h { $m } min
fmt-countdown-days-seconds = { $d } d { $h } h { $m } min { $s } s
fmt-countdown-hours-seconds = { $h } h { $m } min { $s } s
fmt-countdown-minutes-seconds = { $m } min { $s } s
fmt-runtime-minutes = { $m } min
fmt-runtime-hours = { $h } godz.
fmt-runtime-hours-minutes = { $h } godz. { $m } min
fmt-episode = Odcinek
# $size is already written with the language's decimal mark ("1,5")
fmt-size-b = { $size } B
fmt-size-kb = { $size } KB
fmt-size-mb = { $size } MB
fmt-size-gb = { $size } GB
fmt-size-tb = { $size } TB
fmt-episode-span = Odcinki { $first }-{ $through }

## apps/linux/src/problem.rs: every call error in a person's words, what happened and what to do

problem-report = Spróbuj ponownie. Jeśli to się powtarza, zgłoś to z dziennika aktywności.
problem-try-later = Spróbuj później.
problem-helper-unreachable = Pomocnik wyszukiwania nie odpowiedział.
problem-helper-unauthorized = Pomocnik wyszukiwania odrzucił logowanie.
problem-helper-not-running = Pomocnik wyszukiwania nie działa.
problem-helper-missing = Pomocnik wyszukiwania nie jest zainstalowany.
problem-helper-failed = Pomocnik wyszukiwania nie mógł tego zrobić.
problem-helper-no-login = Pomocnik wyszukiwania nie może zalogować się do klienta torrentów.
problem-helper-no-login-next = Zapisz hasło klienta torrentów w Ustawieniach, w sekcji Pobieranie.
problem-torrent-unreachable = Klient torrentów nie odpowiedział.
problem-torrent-unauthorized = Klient torrentów odrzucił logowanie.
problem-torrent-not-running = Klient torrentów nie działa.
problem-torrent-missing = Klient torrentów nie jest zainstalowany.
problem-torrent-failed = Klient torrentów nie mógł tego zrobić.
problem-downloads-unreachable-next = Sprawdź, czy działa i czy jego adres w Ustawieniach, w sekcji Pobieranie, jest poprawny.
problem-downloads-unauthorized-next = Sprawdź nazwę użytkownika i hasło w Ustawieniach, w sekcji Pobieranie.
problem-downloads-not-running-next = Uruchom go w Ustawieniach, w sekcji Pobieranie.
problem-downloads-missing-next = Ustawienia, sekcja Pobieranie, mówią, jak go skonfigurować.
problem-trash = { -app-name } nie mógł przenieść plików do kosza.
problem-trash-next = Sprawdź, czy pliki należą do Ciebie i czy dysk nie jest tylko do odczytu.
problem-source-list = Ta lista źródeł nie zadziałała.
problem-source-list-next = Sprawdź jej adres w Ustawieniach, w sekcji Pobieranie, Listy źródeł.
# $name is a search source or a site, by its own name.
problem-remote-unreachable = { $name } nie odpowiedział.
problem-remote-unreachable-next = Witryna może nie działać. Spróbuj później albo przeszukaj inne źródło.
problem-remote-unauthorized = { $name } odrzucił podany klucz.
problem-remote-unauthorized-next = Zmień jego klucz w Ustawieniach, w sekcji Pobieranie, Listy źródeł.
problem-remote-failed = { $name } nie mógł tego zrobić.
problem-remote-failed-next = Spróbuj później albo przeszukaj inne źródło.
# $name is AniList, MyAnimeList, TVmaze and the like.
problem-provider-sign-in = { $name } odrzucił logowanie { -app-name }.
problem-provider-sign-in-next = Połącz go ponownie w Ustawieniach, w sekcji Konta.
problem-provider-turned-away = { $name } odrzucił { -app-name }.
problem-provider-busy = { $name } jest teraz zajęty.
problem-provider-busy-next = Poczekaj minutę i spróbuj ponownie.
problem-provider-no-answer = { $name } nie odpowiedział.
problem-provider-no-answer-next = Sprawdź połączenie z internetem i spróbuj ponownie.
problem-provider-trouble = { $name } ma teraz problemy.
problem-provider-trouble-next = Spróbuj za kilka minut.
problem-provider-rejected = { $name } tego nie przyjął.
problem-not-found = { -app-name } nie może już tego znaleźć.
problem-not-found-next = Mogło zostać usunięte. Spróbuj ponownie, aby wczytać stronę od nowa.
problem-invalid-next = Sprawdź to i spróbuj ponownie.
problem-unavailable = Ten folder jest teraz niedostępny.
problem-unavailable-next = Podłącz jego dysk albo sprawdź folder w Ustawieniach, w sekcji Biblioteka.
problem-not-connected = { $tracker } nie jest połączony.
problem-not-connected-next = Połącz go w Ustawieniach, w sekcji Konta.
problem-io = { -app-name } nie mógł odczytać ani zapisać pliku.
problem-io-next = Sprawdź, czy na dysku jest miejsce i czy folder należy do Ciebie, i spróbuj ponownie.
problem-storage = { -app-name } nie mógł zapisać biblioteki.
problem-storage-next = Sprawdź, czy na dysku jest miejsce, i spróbuj ponownie.
problem-keyring = Systemowy magazyn haseł nie odpowiedział.
problem-keyring-next = Odblokuj pęk kluczy pulpitu i spróbuj ponownie.
problem-unsupported = { -app-name } jeszcze tego tutaj nie umie.
problem-version = Ta kopia pochodzi z nowszego { -app-name }.
problem-version-next = Zaktualizuj { -app-name } w Ustawieniach, w sekcji Informacje, i zaimportuj ją ponownie.
problem-internal = Coś poszło nie tak w { -app-name }.
problem-refused-hidden = Ten serial jest ukryty.
problem-refused-hidden-next = Najpierw przywróć go na jego stronie.
problem-refused-no-match = Ten serial nie jest jeszcze dopasowany.
problem-refused-no-match-next = Najpierw dopasuj go w Informacjach.
problem-refused-not-newer = Twoja lista jest już dalej niż ten odcinek.
problem-refused-extra = Ten plik to dodatek, nie odcinek.
problem-refused-unmatched = Ten plik nie jest powiązany z odcinkiem.
problem-refused-unmatched-next = Popraw to na stronie serialu, w sekcji Poprawki.
problem-refused-on-disk = Ten plik jest tylko na tym komputerze.

## apps/linux/src/library_view.rs: the library page's section titles and the one fact under each
## (Dropped, On hold and Watching reuse status-* from core.ftl)

library-view-airing = W emisji
library-view-no-date = Bez daty
library-view-type-tv = TV
library-view-type-movies = Filmy
library-view-type-extras = OVA, ONA i specjale
library-view-type-music = Muzyka
library-view-type-unknown = Nieznane
# The Genre section for shows with no genre yet, most often because they are not matched.
library-view-no-genre = Niedopasowane
library-view-state-caught-up = Na bieżąco
library-view-state-not-started = Nierozpoczęte
# $span is a short countdown: library-view-countdown-*.
library-view-next-episode = Następny odcinek za { $span }
library-view-countdown-days = { $d } d { $h } h
library-view-countdown-hours = { $h } h { $m } min
library-view-countdown-minutes = { $m } min
library-view-in-progress = W trakcie: { $count }
library-view-not-started = Nierozpoczęte: { $count }
library-view-all-watched = Wszystko obejrzane
library-view-on-disk =
    { $count ->
        [one] { $count } odcinek
        [few] { $count } odcinki
        [many] { $count } odcinków
       *[other] { $count } odcinka
    } na dysku
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } plik
        [few] { $count } pliki
        [many] { $count } plików
       *[other] { $count } pliku
    } wciąż na dysku

## apps/linux/src/dbus/notify.rs: the desktop notification for a new episode
## (its Open button reuses action-open from core.ftl)

# $episode is the episode's number, $title the show's.
notify-episode-out = Wyszedł odcinek { $episode } serialu { $title }
notify-click-open = Kliknij, aby otworzyć w { -app-name }
notify-click-get = Kliknij, aby pobrać odcinki w { -app-name }
notify-get-episodes = Pobierz odcinki

## apps/linux/src/nowplaying.rs: the second line a media widget shows while an episode plays

# $episode is episode-number from core.ftl ("Episode 12").
now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: why the screen stays on, as the desktop's power panel shows it

inhibit-reason = Odtwarzanie odcinka

## apps/linux/src/bridge/door.rs: why a value a page sent cannot be used (an error's detail line)

door-not-a-choice = { $value } to nieprawidłowy wybór
door-tokens-op-empty = operacja tokens wymaga tokenów
door-not-a-refine-op = { $value } to nie operacja zawężania
door-limit-too-low = musi wynosić co najmniej 1, podano { $limit }
door-not-a-torrent-action = { $value } to nie akcja torrenta

## apps/linux/src/bridge/shell.rs: the first-run starter folder and the update notice

# $folder is a path, $error the system's own reason.
shell-starter-folder-failed = Nie udało się utworzyć { $folder }: { $error }
shell-update-gone = { -app-name } { $version } nie jest już dostępny

## Watch together (#846)

onboarding-step-together = Wspólne oglądanie
onboarding-title-together = Oglądaj ze znajomymi
onboarding-body-together = Utwórz pokój w odtwarzaczu i wyślij jego kod. Każdy odtwarza własną kopię odcinka, a odtwarzanie, pauza i przewijanie pozostają u wszystkich zgrane.
onboarding-ready-together = Wspólne oglądanie
onboarding-ready-together-as = W pokojach widać Cię jako { $name }.
onboarding-ready-together-none = Jeszcze bez nazwy. Panel pokoju zapyta o nią za pierwszym razem.
onboarding-together-you = Ty
onboarding-together-code = Kod pokoju
onboarding-together-episode = Frieren · Odcinek 3
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = Twoja nazwa w pokojach
rail-room = Wspólne oglądanie (Ctrl+J)
rail-room-in = W pokoju wspólnego oglądania (Ctrl+J)
rail-join-room = Dołącz do pokoju
rail-in-room = W pokoju
player-chrome-watch-together = Oglądaj razem
player-chrome-room = Pokój { $code } · { $count ->
    [one] 1 osoba
    [few] { $count } osoby
    [many] { $count } osób
   *[other] { $count } osoby
}
room-name-empty = Wpisz nazwę, którą zobaczą inni w pokoju.
room-name-bad = Ta nazwa nie działa. Użyj od 1 do 32 znaków.
room-error-not-found = Nie ma pokoju o tym kodzie. Sprawdź go z osobą, która go wysłała; pokój mógł zostać zamknięty.
room-error-full = Ten pokój jest pełny. Pokój mieści 10 osób.
room-error-unreachable = Nie udało się połączyć z serwerem pokojów. Sprawdź połączenie i spróbuj ponownie.
room-error-refused = Serwer pokojów odrzucił prośbę. Spróbuj ponownie za minutę.
room-error-proto = Ten pokój działa na nowszej wersji pokojów. Zaktualizuj { -app-name }, aby dołączyć.
room-error-not-in-room = Nie jesteś już w tym pokoju. Utwórz pokój lub dołącz do niego ponownie.
room-error-bad-code = Kod pokoju to 6 liter i cyfr, na przykład K7QM2X.
room-error-bad-relay = Serwer pokojów w Ustawieniach, Odtwarzanie nie jest adresem internetowym. Popraw go tam i spróbuj ponownie.
room-panel-title = Wspólne oglądanie
room-panel-close = Zamknij
room-panel-connected = Połączono
room-panel-reconnecting = Ponowne łączenie
room-panel-name-label = Twoja nazwa w pokojach
room-panel-name-placeholder = Twoja nazwa
room-panel-name-start = Utwórz pokój
room-panel-name-join = Dołącz do pokoju
room-panel-start = Utwórz pokój
room-panel-or-join = Lub dołącz przez kod
room-panel-join-label = Kod pokoju
room-panel-join = Dołącz
room-panel-you-are = Widać Cię jako { $name }
room-panel-change-name = Zmień
room-panel-code-label = Kod pokoju
room-panel-copy = Kopiuj link
room-panel-link-copied = Skopiowano link do pokoju
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = Tego odcinka nie ma w Twojej bibliotece, więc nie możesz jeszcze oglądać razem.
room-panel-open-episode = Otwórz odcinek
room-panel-people = { $count ->
    [one] 1 osoba
    [few] { $count } osoby
    [many] { $count } osób
   *[other] { $count } osoby
}
room-panel-host = Gospodarz
room-panel-you = Ty
room-panel-ready = Gotowe
room-panel-opening = Otwieranie
room-panel-missing = Nie ma odcinka
room-panel-offset = Twoje przesunięcie
room-panel-offset-earlier = Pół sekundy wcześniej
room-panel-offset-later = Pół sekundy później
room-panel-seconds = { $value } s
room-panel-suggested = Czołówki sugerują { $offset }
room-panel-use-suggested = Użyj
room-panel-leave = Opuść pokój
player-room-speed = Prędkość ustala pokój
settings-find-room-name =
    .words = wspólne oglądanie razem pokój nazwa znajomi
settings-find-room-relay =
    .words = wspólne oglądanie razem pokój przekaźnik serwer
key-help-key-ctrl-j = Ctrl+J
key-help-room = Wspólne oglądanie
settings-playback-together = Wspólne oglądanie
settings-playback-room-name = Twoja nazwa w pokojach wspólnego oglądania
settings-playback-room-name-placeholder = Twoja nazwa
settings-playback-room-relay = Serwer pokojów
settings-playback-room-relay-helper = Tu powstają pokoje. Zmieniaj go tylko dla własnego serwera.
