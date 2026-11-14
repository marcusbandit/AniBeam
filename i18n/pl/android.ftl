## Names that stay names

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Words many pages share. Use these before adding a page's own copy.

common-cancel = Anuluj
common-back = Wstecz
common-save = Zapisz
common-done = Gotowe
common-close = Zamknij
common-delete = Usuń
common-retry = Ponów
common-copy = Kopiuj
common-clear = Wyczyść
common-on = Wł.
common-off = Wył.
common-play = Odtwórz
common-hide = Ukryj
common-remove = Usuń
common-add = Dodaj
common-test = Testuj
common-reading = Wczytywanie...
common-working = Pracuję...
common-checking = Sprawdzanie...
common-all = Wszystkie
common-none = Brak
common-episodes = { $count ->
    [one] { $count } odcinek
    [few] { $count } odcinki
    [many] { $count } odcinków
   *[other] { $count } odcinków
}
common-episode-n = Odcinek { $n }
common-join-comma = { $a }, { $b }

## Language (#793)

lang-follow-phone = Jak w telefonie
more-language = Język
more-language-phone = Jak w telefonie: { $lang }
lang-name-en = Angielski
lang-name-da = Duński
lang-name-de = Niemiecki
lang-name-es = Hiszpański
lang-name-fr = Francuski
lang-name-pt-BR = Portugalski (Brazylia)
lang-name-it = Włoski
lang-name-nl = Niderlandzki
lang-name-pl = Polski
lang-name-ru = Rosyjski
lang-name-ja = Japoński
lang-name-ko = Koreański
lang-name-zh-CN = Chiński (uproszczony)

## Onboarding

onb-welcome-title = AniBeam
onb-welcome-note = Twoje anime w telefonie. Znajdź serial, zobacz, skąd go pobrać, pobierz i synchronizuj swoje listy.
onb-language-title = Język
onb-language-note = AniBeam mówi językiem telefonu, chyba że wybierzesz inny. Możesz to zmienić później w Więcej.
onb-lists-title = Twoje listy
onb-lists-note = Zaloguj się, żeby to, co tu oglądasz, liczyło się na AniList lub MyAnimeList. Możesz to zrobić później w Więcej.
onb-wifi-title = Tylko Wi-Fi
onb-wifi-note = Odcinki są duże. AniBeam pobiera przez Wi-Fi, chyba że zdecydujesz inaczej.
onb-storage-title = Pamięć
onb-storage-note = Limit tego, ile AniBeam może pobrać. Niczego nie rezerwuje: miejsce zostaje wolne, dopóki nie zapełnią go odcinki.
onb-notify-title = Powiadomienia
onb-notify-note = Gdy odcinki się pobierają, jedno powiadomienie pokazuje postęp, a drugie informuje, że odcinek jest gotowy do oglądania. Pobieranie działa tak czy inaczej.
onb-sites-title = Witryny
onb-sites-note = AniBeam nie ma domyślnie niczego do przeszukiwania. Repozytorium to podpisana lista witryn wyszukiwania. Wklej adres repozytorium, sprawdź jego klucz, a potem zainstaluj wybrane witryny. Możesz to zrobić później w Więcej.
onb-computer-title = Twój komputer
onb-computer-note = AniBeam na komputerze może udostępnić swoją bibliotekę temu telefonowi przez Tailscale: jego seriale pojawią się w Bibliotece, a odcinki odtworzą się tutaj. Wpisz jego nazwę w Tailscale i port, a potem Testuj. Nie masz komputera? Zostaw puste i zacznij; później znajdziesz to w Więcej.
onb-wifi-only = Tylko przez Wi-Fi
onb-wifi-on = Włączone. Przy danych komórkowych AniBeam zapyta.
onb-wifi-off = Wyłączone. Pobieranie używa też danych komórkowych.
onb-storage-caption = 18 GB to około 12 odcinków. Przesuń suwak w dowolne miejsce, aż do całej pamięci telefonu.
onb-use-cap = Użyj limitu
onb-cap-on = Włączone. Przy { $cap } AniBeam nie pobierze nic więcej, dopóki nie usuniesz odcinków.
onb-cap-off = Wyłączone. AniBeam pobiera, aż telefon się zapełni.
onb-start = Zacznij
onb-continue = Dalej
onb-back = Wstecz

## Appearance

look-colours = Kolory
look-mode = Tryb
look-dark = Ciemny
look-light = Jasny
look-system = Systemowy
look-follows-phone-dark = Zgodnie z ustawieniem telefonu: ciemny.
look-follows-phone-light = Zgodnie z ustawieniem telefonu: jasny.
look-changes-at-once = Zmiany widać od razu, tutaj i na każdej stronie.
look-dark-theme = Ciemny motyw
look-light-theme = Jasny motyw
look-accent = Akcent
look-accent-note = Oznacza to, co jest wybrane, odtwarzane lub nowe.
look-size = Rozmiar
look-text-size = Rozmiar tekstu
look-size-smaller = Mniejszy
look-size-normal = Normalny
look-size-large = Duży
look-size-larger = Większy
look-size-largest = Największy
look-text-size-note = Jak duży jest tekst w całym AniBeam, dodatkowo do rozmiaru ustawionego w telefonie.

## The window: tabs and the sheets over every page

app-title = AniBeam
phone-nav-home = Start
nav-calendar = Kalendarz
phone-nav-library = Biblioteka
phone-nav-downloads = Pobrane
nav-more = Więcej
app-mobile-title = Pobierać przez dane komórkowe?
app-mobile-note = Jeden odcinek to często 1 do 2 GB. W planie z limitem może to zużyć miesięczny pakiet danych w jeden wieczór.
app-mobile-keep = Tylko Wi-Fi
app-mobile-use = Użyj też danych
app-versions-note = { $count ->
    [one] Ten sam odcinek z { $count } wydania. Wybierz to, które ma się odtworzyć.
    [few] Ten sam odcinek z { $count } wydań. Wybierz to, które ma się odtworzyć.
    [many] Ten sam odcinek z { $count } wydań. Wybierz to, które ma się odtworzyć.
   *[other] Ten sam odcinek z { $count } wydań. Wybierz to, które ma się odtworzyć.
}
app-versions-plays-now = { $release }, odtwarzane teraz
app-menu-get = Pobierz odcinki
app-menu-delete = Usuń odcinki

## More: the index and each section's line of how it stands

more-settings = Ustawienia
more-search = Szukaj ustawień
more-search-none = Brak wyników
more-search-hits = { $count ->
    [one] 1 ustawienie
    [few] { $count } ustawienia
    [many] { $count } ustawień
   *[other] { $count } ustawień
}
more-search-tap = Dotknij, aby tam przejść
more-search-hint = Spróbuj prostszego słowa: pamięć, Wi-Fi, konto, witryna, komputer.
more-accounts = Konta
more-streaming = Przesyłanie
more-subtitles = Napisy
more-connection = Połączenie
more-computer = Twój komputer
more-icon = Ikona
more-stats = Statystyki
more-data = Dane
more-appearance = Wygląd
more-activity = Aktywność
more-about = Info
more-accounts-none = Brak połączonego konta
more-accounts-syncing = Synchronizacja z { $tracker }
more-accounts-syncing-both = Synchronizacja z AniList i MAL
more-dl-status-wifi = Tylko Wi-Fi, bez limitu
more-dl-status-wifi-cap = Tylko Wi-Fi, limit { $cap }
more-dl-status-mobile = Wi-Fi i dane komórkowe, bez limitu
more-dl-status-mobile-cap = Wi-Fi i dane komórkowe, limit { $cap }
more-stream-status-auto = Auto, 720p przez dane komórkowe lub przekaźnik
more-stream-status-original = Zawsze oryginalny plik
more-stream-status-converted = Konwersja do { $quality }
more-sites-status-none = Brak repo
more-sites-status-on = { $sites ->
    [one] 1 witryna włączona
    [few] { $sites } witryny włączone
    [many] { $sites } witryn włączonych
   *[other] { $sites } witryn włączonych
}
more-sites-status-on-from = { $sites ->
    [one] 1 witryna włączona
    [few] { $sites } witryny włączone
    [many] { $sites } witryn włączonych
   *[other] { $sites } witryn włączonych
}, { $repos ->
    [one] z 1 repozytorium
    [few] z { $repos } repozytoriów
    [many] z { $repos } repozytoriów
   *[other] z { $repos } repozytoriów
}
more-conn-status-unchecked = Niesprawdzone
more-conn-status-ok = Każdy krok się udał
more-conn-status-failed = { $total ->
    *[other] { $failed } z { $total } kroków nie powiodło się
}
more-computer-status-none = Jeszcze nie połączono
more-icon-default = Domyślna
more-icon-dark = Ciemna
more-icon-line = Liniowa
more-subs-status = Rozmiar, miejsce, czcionka, kolory i tło
more-data-status = Kopia, przywracanie, pamięć obrazów
more-activity-status = Co robił AniBeam, od najnowszych
more-look-status-phone = Zgodnie z telefonem
more-stats-status = Obejrzano { $hours } h w { $year }
more-stats-status-empty = Rok oglądania

## More: the sections' own settings

more-accounts-note = To, co tu oglądasz, oznacza Twoje listy, a postęp z nich widać przy serialach.
more-storage = Pamięć
more-storage-capped = Limit { $cap }
more-storage-no-cap = Bez limitu
more-storage-used = zajęte przez odcinki
more-storage-cap-caption = limit
more-storage-free = wolne w telefonie
more-set-storage-cap = Limit pamięci
more-storage-cap-on = Przy { $cap } nic więcej się nie pobierze, dopóki nie usuniesz odcinków.
more-storage-cap-off = Wyłączone: AniBeam pobiera, aż telefon się zapełni.
more-network = Sieć
more-network-wifi = Tylko Wi-Fi
more-network-mobile = Wi-Fi i dane komórkowe
more-wifi-on = Włączone. Pobieranie czeka na Wi-Fi; przy danych komórkowych AniBeam zapyta.
more-set-stream-quality = Jakość przesyłania
more-stream-question = Jak odcinek z Twojego komputera odtwarza się tutaj.
more-stream-auto-note = Auto odtwarza plik bez zmian przez Wi-Fi, a po konwersji do 720p przez dane komórkowe lub gdy Tailscale łączy się z Twoim komputerem przez przekaźnik.
more-stream-original-note = Plik bez zmian, za każdym razem. Duże pliki wymagają szybkiego łącza.
more-stream-converted-note = Konwertowany na Twoim komputerze do { $quality }. Zużywa mniej danych; przewijanie chwilę trwa.
more-set-feeds = Własne kanały
more-feeds-note = Przeszukiwane przed witrynami
more-set-check = Sprawdź połączenie
more-check-note = Testuje każdy krok między tym telefonem a witrynami wyszukiwania: znalezienie ich, połączenie z nimi i bezpieczne połączenie.
more-check-copy = Skopiuj wyniki
more-set-computer = AniBeam na Twoim komputerze
more-computer-note = Jego biblioteka pojawia się w Bibliotece, a jego odcinki odtwarzają się tutaj. Nazwa, jak moj-komputer, albo adres, jak 100.64.0.1.
more-computer-placeholder = moj-komputer:8790
more-subs-note = Od tego zaczyna każdy odcinek na tym telefonie. Odcinek przesyłany z komputera zaczyna od ustawień tego komputera.
more-icon-note = Ikona na ekranie głównym i w szufladzie aplikacji.
more-icon-line-note = Ikona liniowa przyjmuje kolory tapety, jak Twoje inne ikony Material You.
more-icon-moved-note = Niektóre launchery usuwają ikonę z ekranu głównego po jej zmianie. Zostaje w szufladzie aplikacji; przeciągnij ją stamtąd z powrotem.
more-set-whats-new = Co nowego
more-whats-new-note = Co zmieniło się w każdej wersji, od najnowszej.
more-set-onboarding = Od nowa
more-onboarding-note = Przejdź jeszcze raz przez pytania z pierwszego uruchomienia.
more-set-version = Wersja

## Status lines Rust writes on the pages

host-synced = Sync { $when }.
host-last-try = Ostatnia próba { $when }: { $error }
host-asking = Pytam...
host-reached = Połączono z { $host }, AniBeam { $version }.
host-saved = Zapisano. Sync...
host-none = Brak komputera. Ten telefon korzysta tylko z własnej biblioteki.
acct-signed-in-as = Zalogowano jako { $name }
acct-signed-in = Zalogowano
acct-not-in-build = Niedostępne w tej wersji
acct-tap-to-sign-in = Nie zalogowano. Dotknij, aby się zalogować.
acct-sign-in-failed = Logowanie nie powiodło się: { $why }. Dotknij, aby spróbować ponownie.
acct-sign-in-not-started = Nie udało się rozpocząć logowania: { $error }. Dotknij, aby spróbować ponownie.
acct-still-starting = AniBeam jeszcze się uruchamia. Spróbuj za chwilę.
home-recently-watched = Ostatnio oglądane
home-recently-downloaded = Ostatnio pobrane
home-note-empty = W telefonie
home-note = { $count ->
    [one] W telefonie: 1 serial
    [few] W telefonie: { $count } seriale
    [many] W telefonie: { $count } seriali
   *[other] W telefonie: { $count } seriali
}
lib-this-phone = Ten telefon
lib-note-phone = { $count ->
    [one] Ten telefon: 1 serial
    [few] Ten telefon: { $count } seriale
    [many] Ten telefon: { $count } seriali
   *[other] Ten telefon: { $count } seriali
}
lib-note-remote = { $device }, zsynchronizowano { $when }
lib-empty-search = Brak wyników dla „{ $query }”.
lib-empty-phone = Na tym telefonie jeszcze nic nie ma.
lib-empty-remote = To urządzenie nie ma nic w bibliotece.
cal-view-airing = Na antenie
cal-note-none = Nic w tym tygodniu
cal-note = { $count ->
    [one] 1 odcinek w ciągu 7 dni
    [few] { $count } odcinki w ciągu 7 dni
    [many] { $count } odcinków w ciągu 7 dni
   *[other] { $count } odcinków w ciągu 7 dni
}
airing-asking = Pytam AniList
airing-asking-newest = Pytam AniList o najnowsze
phone-airing-filter-all = Wszystko { $count }
airing-filter-listed = Na listach { $count }
airing-filter-owned = W telefonie { $count }
airing-this-season = Ten sezon
airing-next-season = Następny sezon
airing-other-season = Inne
airing-hidden = Ukryte jako „nie interesuje”: { $count }
airing-plan-needs-tracker = „Planuję obejrzeć” wymaga AniList lub MAL. Zaloguj się w Więcej, Konta.
airing-plan-failed = Nie udało się dodać do planowanych: { $error }
list-watching = Oglądam
list-planning = Planuję
list-completed = Ukończone
list-paused = Wstrzymane
list-dropped = Porzucone
list-repeating = Powtarzam
more-check-all-ok = Każdy krok się udał.
more-check-failed = { $failed } z { $total } kroków nie powiodło się. Zrzut ekranu tej listy pokaże, które.
more-check-row = { $step }  { $ms } ms
more-check-copied = Skopiowano. Wklej to do wiadomości.
more-version = AniBeam na Androida { $version } (kompilacja { $build })
dl-note-waiting = Czekam na Wi-Fi. Pobieranie ruszy samo, gdy telefon połączy się z Wi-Fi.
dl-note-nothing = Nic się nie pobiera.
dl-state-done = Gotowe
dl-state-waiting = Czeka na Wi-Fi
dl-state-paused = Wstrzymane
dl-state-peers = Szukam peerów
dl-state-checking = Sprawdzanie
dl-state-error = Zatrzymano przez błąd
dl-state-downloading = Pobieranie
dl-amount = { $got } z { $size }
dl-speed = { $rate }/s
dl-left = { $span } do końca
dl-episodes-range = Odcinki { $from }-{ $to }
dl-batch = Paczka
disk-storage-line = { $used } zajęte
disk-storage-line-free = { $used } zajęte, { $free } wolne
disk-free = Na telefonie zostało { $free } wolnego miejsca.
disk-free-below-cap = Na telefonie zostało { $free } wolnego miejsca, mniej niż limit: telefon zapełni się, zanim AniBeam przestanie pobierać.

## More's search: the words a person might type for each setting

more-set-subs-size = Rozmiar i położenie napisów
more-set-subs-style = Styl napisów
more-words-anilist = anilist konto logowanie zaloguj połącz lista postęp śledzenie
more-words-mal = mal myanimelist konto logowanie zaloguj połącz lista postęp śledzenie
more-words-storage-cap = miejsce dysk limit gb pełna pamięć usuń rozmiar
more-words-wifi = wifi wi-fi dane komórkowe sieć mobilne internet
more-words-notifications = powiadomienia powiadom alert postęp uprawnienie pasek zezwól
more-words-stream-quality = auto oryginał 1080p 720p 480p konwersja dane komputer odtwarzanie jakość
more-words-check-repos = odśwież aktualizuj repo repozytorium endpoint
more-words-add-repo = repo repozytorium endpoint zaufaj odcisk adres url
more-words-add-site = własny endpoint kanał rss torznab jackett prowlarr adres
more-words-feeds = kanał rss źródło usuń
more-words-check = sieć dns test https bezpieczne połączenie internet
more-words-computer = tailscale tailnet host adres komputer biblioteka przesyłanie test zapisz urządzenie
more-words-whats-new = dziennik zmian changelog nowości aktualizacja
more-words-onboarding = wprowadzenie pierwsze uruchomienie początek od nowa
more-words-version = kompilacja wersja build info
more-words-stats = godziny odcinki gatunki studia sezony rok obejrzane podsumowanie miesiąc historia
more-words-backup = kopia zapasowa backup eksport plik zapisz kopiuj przenieś komputer json prywatne logowania
more-words-restore = przywróć import plik wczytaj biblioteka scal
more-words-cache = obrazy plakaty banery wyczyść pamięć miejsce cache
more-words-activity = dziennik zdarzenia błędy problemy kopiuj wyczyść historia co się stało
more-words-colours = wygląd motyw ciemny jasny tryb system akcent kolor
more-words-text-size = wygląd czcionka większy mniejszy rozmiar tekst czytanie
more-words-icon = ikona launcher ekran główny logo ciemna liniowa motywowana material you
more-words-subs-size = napisy rozmiar skala położenie pozycja wyżej dół
more-words-subs-style = napisy czcionka kolor obrys cień tło pogrubienie ass styl reset
more-words-language = język tłumaczenie angielski lokalizacja language

## Home

phone-home-title = Start
home-next-to-air = Najbliższe
home-after-that = Potem
home-coming-up = Wkrótce
home-nothing-week = Nic w najbliższym tygodniu.
phone-home-empty-title = Na telefonie jeszcze nic nie ma
phone-home-empty-body = Seriale pobrane tutaj i to, co dla nich wychodzi, pojawią się na Starcie. Znajdź serial w Pobranych.

## Library

lib-title = Biblioteka
lib-refresh = Odśwież
lib-search-phone = Szukaj w telefonie
lib-search-device = Szukaj w: { $device }
lib-find-in-downloads = Szukaj „{ $query }” w Pobranych
lib-empty-body = Odcinki pobrane na ten telefon trafiają tutaj. Znajdź serial w Pobranych.
lib-pick-show = Wybierz serial, aby go tu otworzyć.
lib-shows-caption = { $count ->
    [one] serial
    [few] seriale
    [many] seriali
   *[other] seriali
}
lib-on-phone = w telefonie
lib-free = wolne
lib-hold-poster = Przytrzymaj plakat, aby pobrać więcej odcinków lub je usunąć.
phone-nothing-on-phone = brak w telefonie
phone-on-phone = { $count } w telefonie

## Calendar

cal-title = Kalendarz
cal-week = Tydzień
cal-days = Dni
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A, %-d %B
cal-today = Dziś
cal-tomorrow = Jutro
cal-yesterday = Wczoraj
cal-nothing = Brak
cal-next = Następny
cal-get = Pobierz
cal-day-note = { $date }, { $count }
cal-next-airing = Najbliższa emisja: { $title }
cal-nothing-aired-yesterday = Wczoraj nic nie wyszło.
cal-nothing-aired-that-day = Tego dnia nic nie wyszło.
cal-nothing-today = Dziś nic nie wychodzi.
cal-nothing-tomorrow = Jutro nic nie wychodzi.
cal-nothing-that-day = Tego dnia nic nie wychodzi.
cal-empty-title = Na telefonie nie ma jeszcze seriali
cal-empty-body = Odcinki seriali, które tu trzymasz, pojawiają się w dniu emisji: dwa dni wstecz i tydzień naprzód.
phone-episodes-range = Odcinki { $from }-{ $to }
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = W telefonie
phone-out-ago = Wyszło { $ago }

## Airing now

phone-airing-out-now = Już wyszło
airing-last-two-days = Ostatnie dwa dni
airing-season-note = Seriale sezonu pochodzą z AniList.
airing-none-lists-title = Nic z emisji nie ma na Twoich listach
airing-none-lists-body = „Planuję obejrzeć” przy serialu dodaje go do Twojej listy.
airing-none-phone-title = Nic z emisji nie ma na tym telefonie
airing-none-phone-body = „Pobierz odcinki” przy serialu pobiera go tutaj.
airing-marked = Serial { $title } oznaczono jako nieinteresujący.
phone-airing-undo = Cofnij
airing-plan = Planuję obejrzeć
airing-get-more = Pobierz więcej
airing-get = Pobierz odcinki
phone-airing-not-interested = Nie interesuje

## Seasons

season-winter = Zima
season-spring = Wiosna
season-summer = Lato
season-fall = Jesień
season-named = { $season } { $year }
season-pick = Wybierz sezon

## A show's page

series-back-home = Start
series-back-library = Biblioteka
series-back-downloads = Pobrane
series-back-calendar = Kalendarz
phone-series-get-episodes = Pobierz odcinki
phone-series-episodes = Odcinki
series-delete-episodes = Usuń odcinki
series-delete-tap = Dotknij odcinków do usunięcia.
phone-series-delete-picked = { $count ->
    [one] Wybrano { $count } odcinek.
    [few] Wybrano { $count } odcinki.
    [many] Wybrano { $count } odcinków.
   *[other] Wybrano { $count } odcinków.
}
series-delete-note = Ich pliki zostaną usunięte z tego telefonu. Historia oglądania zostaje zapisana.
series-delete-button = { $count ->
    [one] Usuń { $count } odcinek
    [few] Usuń { $count } odcinki
    [many] Usuń { $count } odcinków
   *[other] Usuń { $count } odcinków
}
series-no-episodes = Brak odcinków w telefonie i żadnych zaplanowanych.
series-versions = { $count ->
    [one] { $count } wersja
    [few] { $count } wersje
    [many] { $count } wersji
   *[other] { $count } wersji
}
series-note-next-up = Następny
series-note-missing = Brakuje
series-note-not-here = Nie ma w telefonie
series-note-not-downloaded = Nie pobrano
series-note-downloading = Pobieranie { $percent }%
series-note-resume-at = Wznów od { $time }
phone-episodes-estimate = { $count ->
    [one] { $count }? odcinek
    [few] { $count }? odcinki
    [many] { $count }? odcinków
   *[other] { $count }? odcinków
}
phone-next-airs = Odcinek { $n } za { $time }
phone-watched = Obejrzane
phone-not-watched = Jeszcze nieobejrzane
phone-progress = Obejrzano { $watched } z { $total }, { $count } w telefonie
phone-on-this-phone = { $count } w telefonie
phone-play-film = Odtwórz film
phone-resume-film = Wznów film
phone-play-episode = Odtwórz odcinek { $n }
phone-resume-episode = Wznów odcinek { $n }
phone-about-date = około { $date }

## A show's list status, score, notes and fixes

show-status-watching = Oglądam
show-status-planning = Planuję
show-status-completed = Ukończone
show-status-paused = Wstrzymane
show-status-dropped = Porzucone
show-status-repeating = Powtarzam
show-status-now = Teraz: { $label }
phone-show-add-to-list = Dodaj do listy
phone-show-your-score = Twoja ocena { $score }
show-rate = Oceń
show-fixes = Poprawki
show-your-notes = Twoje notatki
show-add-notes = Dodaj notatki
show-home-each = Zmień to na każdej połączonej liście.
show-home-no-tracker = Zapisane w telefonie: żaden serwis nie ma tego serialu.
show-home-tracker-lacks = Zapisane w telefonie: Twój serwis nie ma tego serialu.
show-home-until-connected = Zapisane w telefonie, dopóki nie połączysz AniList lub MyAnimeList.
show-sites-both = AniList i MyAnimeList
show-notes-title = Notatki
show-notes-placeholder = Na czym stanęło, kto polecił, które wydanie pobrać
show-notes-over = { $count ->
    [one] { $count } znak ponad limit
    [few] { $count } znaki ponad limit
    [many] { $count } znaków ponad limit
   *[other] { $count } znaków ponad limit
}
show-notes-saved-to = Zapisano w: { $sites }.
show-notes-saved-to-tracker = Zapisano na Twojej liście.
show-notes-saving = Zapisywanie notatek.
show-notes-clearing = Czyszczenie notatek.
show-score-saving = Zapisywanie oceny.
show-score-saved = Zapisano ocenę.
show-score-clearing = Usuwanie oceny.
show-score-cleared = Ocena usunięta.
phone-show-moving-to = Przenoszenie do: { $label }.
phone-show-moved-to = Przeniesiono: { $label }.
show-rescan-started = Skanowanie ruszyło.
show-still-starting = AniBeam jeszcze się uruchamia. Spróbuj za chwilę.
show-fix-match = Dopasuj
show-fix-find-match = Znajdź pasujący
show-fix-rescan = Przeskanuj folder

## A show's characters, related shows and recommendations

show-characters = Postacie
phone-show-related = Powiązane
show-related-all = Wszystkie { $count }
show-related-filling = AniList wciąż uzupełnia dane.
phone-show-recommendations = Polecane
show-card-owned = W bibliotece
show-card-unreleased = Jeszcze niewydane
show-card-on-anilist = Na AniList
show-rec-available = Dostępne
show-relation-prequel = Prequel
show-relation-sequel = Sequel
show-relation-side-story = Poboczna
show-relation-parent = Główna
show-relation-spin-off = Spin-off
show-relation-adaptation = Adaptacja
show-relation-alternative = Alternatywa
show-relation-summary = Skrót
show-relation-character = Postać
show-relation-source = Źródło
show-relation-compilation = Kompilacja
show-relation-contains = Zawiera
show-relation-other = Inne
show-role-main = Główna
show-role-supporting = Drugoplanowa
show-role-background = Tło
show-format-tv = TV
show-format-tv-short = TV krótki
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Specjal
show-format-film = Film
show-entry-get = Pobierz odcinki
show-entry-open-anilist = Otwórz w AniList
show-entry-line = { $line }, { $status }

## Fix a wrong match

phone-match-title = Dopasuj
match-refresh = Odśwież info
phone-match-clear = Usuń powiązanie
match-clear-it = Wyczyść
match-clear-warning = Wyczyszczenie usuwa powiązanie z serwisem i ręcznie ustawioną liczbę odcinków. Dotknij ponownie, aby wyczyścić.
match-tvmaze = TVmaze
match-search-anilist = Szukaj w AniList
match-search-tvmaze = Szukaj w TVmaze
match-paste-anilist = Wklej link do AniList lub MyAnimeList
match-paste-tvmaze = Wklej link do TVmaze lub IMDb
phone-match-look-up = Znajdź
match-apply = Zastosuj
match-look-up-all = Znajdź każdy folder, którego AniBeam jeszcze nie zna
match-format-music = Muzyka
phone-match-matched-on = Dopasowano w: { $sites }
phone-match-found-tvmaze = Znaleziono serial w TVmaze
phone-match-found-anilist = Znaleziono w AniList
phone-match-found-anilist-season = Znaleziono w AniList, sezon { $season }
phone-match-found-mal = Znaleziono w MyAnimeList
phone-match-searching = Szukanie: { $name }
phone-match-type-more = Wpisz co najmniej dwie litery, aby przeszukać: { $name }.
match-none = Brak tytułów w: { $name }.
match-looking-up = Wyszukiwanie.
phone-match-matched-to = Dopasowano do: { $title }.
phone-match-applied = Zastosowano.
match-applying = Stosowanie dopasowania.
phone-match-refreshing = Odświeżanie: { $title }.
match-clearing = Czyszczenie dopasowania.
phone-match-cleared = Wyczyszczono.
match-auto-looking = Szukanie folderów, których AniBeam jeszcze nie zna.

## Batches (the line under a release in a search)

inspect-reading = Czytanie listy plików
inspect-holds = { $count ->
    [one] Zawiera { $count } odcinek: { $runs }
    [few] Zawiera { $count } odcinki: { $runs }
    [many] Zawiera { $count } odcinków: { $runs }
   *[other] Zawiera { $count } odcinków: { $runs }
}

## What's new

phone-notes-title = Co nowego
notes-got-it = Jasne

## Move the list to a tracker

move-lists-label = Listy w telefonie
move-title = Przenieś swoją listę do: { $site }
move-here = W telefonie: { $list }
move-there = Teraz w { $site }: { $list }
move-later-hint = Później: Więcej, Konta
move-not-now = Nie teraz
move-nothing = Brak
move-not-on-list = Poza listą
move-nothing-left = Nic do przeniesienia
move-side-status-score = { $status }, ocena { $score }
move-side-score = ocena { $score }
move-lead = { $count ->
    [one] Jeden serial ma status lub ocenę tylko w tym telefonie. Przeniesienie doda go do Twojej listy w { $site }, gdzie zobaczą go Twój komputer i profil. Dla tego serialu to, co ma teraz { $site }, zostanie zastąpione. Nic nie zostanie wysłane, dopóki nie naciśniesz przycisku Przenieś.
    [few] { $count } seriale mają status lub ocenę tylko w tym telefonie. Przeniesienie doda je do Twojej listy w { $site }, gdzie zobaczą je Twój komputer i profil. Dla tych seriali to, co ma teraz { $site }, zostanie zastąpione. Nic nie zostanie wysłane, dopóki nie naciśniesz przycisku Przenieś.
    [many] { $count } seriali ma status lub ocenę tylko w tym telefonie. Przeniesienie doda je do Twojej listy w { $site }, gdzie zobaczą je Twój komputer i profil. Dla tych seriali to, co ma teraz { $site }, zostanie zastąpione. Nic nie zostanie wysłane, dopóki nie naciśniesz przycisku Przenieś.
   *[other] { $count } seriali ma status lub ocenę tylko w tym telefonie. Przeniesienie doda je do Twojej listy w { $site }, gdzie zobaczą je Twój komputer i profil. Dla tych seriali to, co ma teraz { $site }, zostanie zastąpione. Nic nie zostanie wysłane, dopóki nie naciśniesz przycisku Przenieś.
}
move-label = { $count ->
    [one] Przenieś { $count } serial
    [few] Przenieś { $count } seriale
    [many] Przenieś { $count } seriali
   *[other] Przenieś { $count } seriali
}
move-waiting = { $count ->
    [one] Jeden serial ma status lub ocenę tylko tutaj. Przenieś go do: { $site }, aby zobaczyć go na komputerze i w profilu.
    [few] { $count } seriale mają status lub ocenę tylko tutaj. Przenieś je do: { $site }, aby zobaczyć je na komputerze i w profilu.
    [many] { $count } seriali ma status lub ocenę tylko tutaj. Przenieś je do: { $site }, aby zobaczyć je na komputerze i w profilu.
   *[other] { $count } seriali ma status lub ocenę tylko tutaj. Przenieś je do: { $site }, aby zobaczyć je na komputerze i w profilu.
}
move-moving = { $count ->
    [one] Przenoszenie { $count } serialu do: { $site }
    [few] Przenoszenie { $count } seriali do: { $site }
    [many] Przenoszenie { $count } seriali do: { $site }
   *[other] Przenoszenie { $count } seriali do: { $site }
}
move-moved = { $count ->
    [one] Przeniesiono { $count } serial do: { $site }
    [few] Przeniesiono { $count } seriale do: { $site }
    [many] Przeniesiono { $count } seriali do: { $site }
   *[other] Przeniesiono { $count } seriali do: { $site }
}
move-failed = { $count ->
    [one] Do { $site } nie trafił { $count } serial; zobacz dziennik aktywności
    [few] Do { $site } nie trafiły { $count } seriale; zobacz dziennik aktywności
    [many] Do { $site } nie trafiło { $count } seriali; zobacz dziennik aktywności
   *[other] Do { $site } nie trafiło { $count } seriali; zobacz dziennik aktywności
}

## Downloads: find (search, shows, sources, releases)

common-join-and = { $a } i { $b }
find-join-sentences = { $a } { $b }

find-downloads = Pobrane

find-lens-list = Twoja lista
find-lens-gaps = Braki
find-lens-airing = Emitowane
find-lens-following = Obserwowane
find-lens-downloaded = Pobrane
find-lens-with-count = { $lens } { $count }
find-what-to-get = Co pobrać
find-lens-list-line = Oglądam i Planuję z Twojej listy. Otwórz serial, aby pobrać jego odcinki.
find-lens-gaps-line = Wyemitowane odcinki, których nie ma na tym telefonie. Otwórz serial, aby zobaczyć, kto je ma.
find-lens-airing-line = Nowe odcinki, które dopiero wyjdą, od najbliższych.
find-lens-following-line = Subskrypcje: nowe odcinki pobierają się same. Otwórz jedną, aby ją zatrzymać.
find-lens-downloaded-line = To, co pobrał ten telefon, serial po serialu.

find-lens-gaps-empty = Niczego nie brakuje: każdy rozpoczęty serial ma tu wszystkie wyemitowane odcinki.
find-lens-airing-empty = Nic w telefonie nie jest w emisji.
find-lens-following-empty = Brak subskrypcji. Otwórz serial i włącz Obserwuj.
find-lens-downloaded-empty = Jeszcze nic nie pobrano.
find-list-empty = Na Twojej liście jeszcze nic nie ma. Wyszukaj serial powyżej i dodaj go.
find-list-reading = Wczytywanie listy...
find-list-signed-out = Zaloguj się do AniList w Więcej, Konta, a Twoja lista pojawi się tutaj.
find-list-failed = Nie udało się wczytać listy: { $error }

find-aired = wyszło
find-ledger-no-feed = brak nowych odcinków: źródło nie ma kanału
find-ledger-downloads = { $count ->
    [one] { $count } pobranie
    [few] { $count } pobrania
    [many] { $count } pobrań
   *[other] { $count } pobrań
}
find-ledger-downloaded = pobrane
find-ledger-planning = Planuję
find-ledger-watching-of = Oglądam, { $progress } z { $total }
find-ledger-watching-seen = Oglądam, obejrzano { $progress }

find-gap-missing = brakuje: { $count }
find-gap-not-downloaded = nie pobrano: { $count }

find-list-status-planning = Na Twojej liście: Planuję
find-list-status-watching = Na Twojej liście: Oglądam
find-list-status-already = Już na Twojej liście
find-going-planning = { $title } trafia na listę Planuję.
find-already-listed = { $title } jest już na Twojej liście.
find-plan-needs-tracker = Dodanie do listy wymaga AniList lub MAL. Zaloguj się w Więcej, Konta.
find-plan-failed = Nie dodano do listy: { $error }
find-show-gone = Tego serialu nie ma już w bibliotece na tym telefonie.

## Shows as you type

find-shows = Seriale
find-suggesting = Pytam AniList...
find-suggest-tap = Dotknij jednego, aby zobaczyć, kto go ma. Enter szuka nazw wydań.
find-suggest-none = Brak serialu o tej nazwie. Enter szuka zamiast tego nazw wydań.
find-suggest-hint = Seriale pojawiają się podczas pisania. Enter szuka nazw wydań.
find-suggest-failed = Nie udało się zapytać AniList: { $error }
find-library-line = W bibliotece
find-library-line-none = W bibliotece: żadnego z { $total } w telefonie
find-library-line-some = W bibliotece: { $here } z { $total } w telefonie

## The show view

find-plan-to-watch = Planuję obejrzeć
find-on-phone-count = { $count } w telefonie
find-airing = Emitowane
find-dropped = { $count ->
    [one] Pominięto { $count } wydanie, które nie wymienia tego serialu.
    [few] Pominięto { $count } wydania, które nie wymieniają tego serialu.
    [many] Pominięto { $count } wydań, które nie wymieniają tego serialu.
   *[other] Pominięto { $count } wydań, które nie wymieniają tego serialu.
}

find-following = Obserwowane
find-stop-following-ask = Przestać obserwować: { $name }?
find-unfollow-explain = Nowe odcinki przestaną się pobierać. Odcinki pobrane przez subskrypcję mogą zostać albo zniknąć razem z nią. Odcinki pobrane ręcznie zostają w każdym przypadku.
find-stop-following = Nie obserwuj
find-stop-keep = Zatrzymaj, zostaw odcinki
find-stop-delete = Zatrzymaj i usuń jej odcinki
find-rule-taken = { $downloads ->
    [one] { $downloads } pobranie
    [few] { $downloads } pobrania
    [many] { $downloads } pobrań
   *[other] { $downloads } pobrań
}, { $files ->
    [one] { $files } plik
    [few] { $files } pliki
    [many] { $files } plików
   *[other] { $files } plików
} w folderze subskrypcji
find-rule-paused = Jest wstrzymana, więc nowe odcinki nie przychodzą. { $taken }.
find-rule-no-feed = Jej źródło nie ma kanału, więc nowe odcinki nie przychodzą. { $taken }.
find-rule-active = Nowe odcinki pobierają się same. { $taken }.
find-unfollowed-deleted = Przestano obserwować: { $name }. Usunięto odcinki subskrypcji.
find-unfollowed-kept = Przestano obserwować: { $name }. Odcinki zostają na tym telefonie.
find-stop-failed = Nie udało się zatrzymać: { $error }

find-where = Skąd pobrać
find-where-pick = Wybierz źródło, aby zobaczyć, co ma.
find-where-nothing = Nic się nie pobiera, dopóki tego nie zlecisz.
find-source-idle = Zapytaj
find-source-trying = Próba: { $name }
find-source-searching = Szukanie
find-source-silent = Nie odpowiada
find-no-sites-title = Brak witryn wyszukiwania
find-no-sites-body = Dodaj repozytorium witryn wyszukiwania albo własny kanał w Więcej.
find-open-more = Otwórz Więcej
find-pick-source-title = Wybierz źródło
find-pick-source-body = Jego wydania pojawią się tu jako siatka, jeden odcinek na kostkę.
find-releases = { $count ->
    [one] { $count } wydanie
    [few] { $count } wydania
    [many] { $count } wydań
   *[other] { $count } wydań
}
find-unreachable = Nie udało się połączyć z { $host }. Niektóre sieci i kraje blokują witryny z wydaniami; AniBeam omija już blokady po nazwach, więc ta blokuje samą witrynę albo witryna nie działa. Pomoże VPN albo wypróbuj inne źródło.
find-the-source = Źródło

find-has = Co ma
find-has-note = Dotknij wiersza, aby wziąć wszystko, albo pojedyncze odcinki.
find-episodes-tag = Odcinki
find-batch = Paczka { $label }
find-row-meta = { common-episodes }  { $seeds ->
    [one] { $seeds } seed
    [few] { $seeds } seedy
    [many] { $seeds } seedów
   *[other] { $seeds } seedów
}  { $size }
find-on-this-phone = W telefonie
find-on-computer = Na { $name }

find-plan-no-releases = To źródło nie ma wydań tego serialu.
find-plan-nothing-picked = Nic nie wybrano: dotknij wiersza lub jego kostek.
find-plan-line = { $episodes } od { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] Pominięto { $count } odcinek, który już masz. Włącz Zastąp, aby pobrać go ponownie.
    [few] Pominięto odcinki, które już masz. Włącz Zastąp, aby pobrać je ponownie.
    [many] Pominięto odcinki, które już masz. Włącz Zastąp, aby pobrać je ponownie.
   *[other] Pominięto odcinki, które już masz. Włącz Zastąp, aby pobrać je ponownie.
}
find-plan-follow-note = Nowe odcinki z wyszukiwania „{ $query }” będą pobierać się same.
find-plan-on-desk = Pobiorą się na komputerze { $name }, do jego biblioteki.
find-follow-new = Obserwuj nowe odcinki
find-follow-new-note = Pobierają się same, gdy wychodzą.
find-replace = Zastąp to, co masz
find-replace-note = Pobiera wybrane odcinki ponownie, na miejsce tych z telefonu.
find-go = { $mode ->
    [replace] Pobierz { common-episodes } i zastąp
    [subscribe] Pobierz { common-episodes } i subskrybuj
    [both] Pobierz { common-episodes }, zastąp i subskrybuj
   *[plain] Pobierz { common-episodes }
}
find-go-pick = Wybierz odcinki do pobrania
find-every-release = Każde wydanie z tego źródła

## Download to

find-download-to = Pobierz na
find-this-phone = Ten telefon
find-your-computer = Twój komputer
find-desk-asking = Pytam: { $name }...
find-desk-silent = { $name } nie odpowiada. Wybierz Ten telefon albo spróbuj, gdy będzie włączony.
find-sending-to = Wysyłanie do: { $name }...

## The release list

find-best-fit = Najlepsze
find-on-disk = Na dysku
find-get = Pobierz
find-get-all = Pobierz { $count }
find-subscribe = Subskrybuj
find-back-step = Krok wstecz
find-narrow-placeholder = Zawęź albo -słowo, aby pominąć
find-best-fit-searches = Najlepsze dopasowanie szuka: { $query }
find-fact-batch = paczka
find-fact-seeds = { $count ->
    [one] { $count } seed
    [few] { $count } seedy
    [many] { $count } seedów
   *[other] { $count } seedów
}
find-fact-via = przez { $source }
find-searching-for = Szukanie: { $text }...
find-search-line = { $count ->
    [0] Nic w { $source } dla tego zapytania
    [one] { $count } wydanie w { $source }
    [few] { $count } wydania w { $source }
    [many] { $count } wydań w { $source }
   *[other] { $count } wydań w { $source }
}{ $fell ->
    [1] , po tym, jak pierwsze źródło nic nie dało
   *[other] { "" }
}{ $narrowed ->
    [1] , zawężone do { $token }
   *[other] { "" }
}.
find-refined = { find-releases }, { $dir ->
    [down] o { $delta } mniej
   *[up] o { $delta } więcej
}.
find-refine-nothing-left = Nic nie zostaje.
find-refine-no-match = Żadne wydanie tego nie ma.
find-refine-empty = Najpierw coś wpisz.
find-refine-no-change = To nic nie zmienia.

## Getting and subscribing

find-getting = Pobieranie: { $what }.
find-subscribed = Subskrypcja włączona: nowe odcinki pobierają się same.
find-subscribed-desk = Subskrypcja na komputerze { $name }: nowe odcinki pobierają się tam same.
find-subscribed-as = Subskrypcja „{ $name }”: nowe odcinki pobierają się same.
find-downloading = { $count ->
    [one] Pobieranie { $count } wydania.
    [few] Pobieranie { $count } wydań.
    [many] Pobieranie { $count } wydań.
   *[other] Pobieranie { $count } wydań.
}
find-not-added = Nie dodano: { $error }
find-not-subscribed = Nie zasubskrybowano: { $error }
find-desk-refused = Odrzucone przez { $name }: { $error }
find-desk-not-subscribed = Nie zasubskrybowano na komputerze { $name }: { $error }

## Player

player-subtitles = Napisy
player-audio = Dźwięk
phone-player-speed = Prędkość
player-connecting = Łączenie
player-hold-speed = Prędkość 2×
player-replay = Powtórz
player-unlock = Odblokuj
player-next-stay = Zostań
player-next-go = Dalej
player-jump-back = −10 s
player-jump-forward = +10 s
player-speed-now = Prędkość { $speed }
player-skip-intro = Pomiń intro
player-skip-outro = Pomiń outro
phone-player-skipped-intro = Pominięto intro
phone-player-skipped-outro = Pominięto outro
phone-player-undo = Cofnij
phone-player-start-over = Od początku
phone-player-resumed-at = Wznowiono od { $time }
player-picture-failed = Nie udało się uruchomić obrazu. Dźwięk gra dalej.
player-file-failed = Ten plik się nie odtworzył: { $why }.
player-streaming-from = Przesyłanie z { $host } w { $height }p
player-mpv-no-start = mpv się nie uruchomił
player-mpv-no-start-why = mpv się nie uruchomił: { $why }
player-android-only = odtwarzacz działa tylko na Androidzie

## Player: delays

player-delay-title = Opóźnienie
player-delay-note = Każde dotknięcie przesuwa o 0,1 s. Do następnego pliku.
phone-player-delay-seconds = { $value } s
player-sub-delay-now = Opóźnienie napisów { $value }
player-audio-delay-now = Opóźnienie dźwięku { $value }

## Player: cast

player-cast-playing-on = Odtwarzanie na: { $device }
phone-player-cast-stop = Zatrzymaj przesyłanie na: { $device }
player-cast-searching = Szukanie Chromecastów i telewizorów
phone-player-cast-to = Prześlij na
phone-player-cast-tv = TV
phone-player-cast-loading = Wczytywanie
phone-player-cast-playing = Odtwarzanie
phone-player-cast-paused = Wstrzymano
phone-player-cast-finished = Zakończono
phone-player-cast-stopped = Zatrzymano

## Player: the tracker's word

phone-player-rated = Oceniono
phone-player-score-failed = Nie udało się zapisać oceny
player-marked-watched = Obejrzane
player-tracked = Zapisano
player-not-tracked = Nie zapisano
player-track-update-failed = Nie udało się zaktualizować listy
player-not-tracked-why = Nie zapisano: { $reason }.
player-track-ok-at = { $tracker }: { $progress }
player-track-ok = { $tracker }: OK
player-track-note = { $tracker }: { $note }
player-track-failed = Nie udało się zaktualizować: { $tracker }
player-track-line = { $head }: { $parts }
phone-player-refusal-hidden = serial jest ukryty
phone-player-refusal-no-match = serial nie ma dopasowania w serwisie
phone-player-refusal-not-newer = lista jest już dalej niż ten odcinek
phone-player-refusal-extra = to dodatek
phone-player-refusal-unmatched = plik nie jest dopasowany do odcinka
phone-player-refusal-on-disk = plik jest tylko na dysku

## Subtitles

subs-line-default = Rozmiar 1×, zwykłe pliki dostają Twój styl
subs-sample = Gdzie to schowano?
subs-size-place-title = Rozmiar i miejsce
subs-size = Rozmiar
subs-position = Położenie
subs-position-helper = 100 to dolna krawędź. Mniejsze liczby podnoszą napisy wyżej w kadrze.
subs-style-title = Styl
subs-styled = Napisy ze stylem
subs-styled-as-made = Pliki ze stylem (ASS) wyglądają dokładnie tak, jak ustawili ich autorzy.
subs-styled-mine = Twój styl poniżej trafia na każdy plik. Znaki i karaoke mogą się zepsuć.
subs-styled-size-only = Pliki napisów ze stylem zachowują wygląd i przyjmują Twój rozmiar. Zwykłe pliki przyjmują Twój styl.
subs-mode-as-made = Oryginał
subs-mode-size-only = Rozmiar
subs-mode-mine = Mój
subs-font = Czcionka
subs-font-bad = Wpisz nazwę czcionki, np. Noto Sans.
subs-bold = Pogrubienie
subs-colour = Kolor tekstu
subs-colour-bad = Wpisz kolor, np. #ffcc00.
subs-outline = Obrys
subs-outline-bad-width = Szerokość to liczba, 0 lub więcej.
subs-outline-bad-colour = Wpisz kolor, np. #000000.
subs-outline-helper = Najpierw szerokość, potem kolor.
subs-shadow = Cień
subs-shadow-bad = Liczba, 0 lub więcej. 0 oznacza brak cienia.
subs-shadow-helper = Jak daleko pada cień, 0 oznacza brak.
subs-box = Tło napisów
subs-box-helper = Ciemne pole za napisami. 0 pomija je.
subs-reset-title = Od nowa
subs-reset-helper = Przywraca rozmiar, czcionkę, kolory, obrys, tło i położenie do wartości początkowych.
subs-reset = Resetuj

## Streaming

stream-quality-auto = Auto
stream-quality-original = Oryginał
stream-app-starting = Aplikacja jeszcze się uruchamia
stream-no-computer = Nie ma jeszcze komputera. Dodaj swój w Więcej.
stream-off-on-host = Przesyłanie jest wyłączone na { $host }. Włącz tam „Stream episodes” w Settings, Phone app.
stream-cannot-stream = { $host } działa na AniBeam { $version }, który jeszcze nie umie przesyłać. Zaktualizuj go tam.
stream-no-session = Drugie urządzenie nie dało sesji
stream-answer-unreadable = Nie udało się odczytać odpowiedzi: { $why }
stream-answered-status = { $name } odpowiada kodem { $status }

## Stats

phone-stats-nothing-yet = Jeszcze nic
phone-stats-empty-title = Jeszcze nic nie obejrzano
phone-stats-empty-body = Skończ odcinek, a policzy się tutaj: godziny, gatunki i obejrzane sezony.
stats-hours-caption = h obejrzane
stats-finished-caption = ukończone seriale
stats-busiest-caption = najlepszy miesiąc
stats-episodes-caption = { $count ->
    [one] odcinek
    [few] odcinki
    [many] odcinków
   *[other] odcinków
}
phone-stats-by-month = Miesiące
phone-stats-top-genres = Gatunki
phone-stats-top-studios = Studia
stats-seasons = Obejrzane sezony
stats-watched-most = Ulubione
stats-year-line = Co obejrzano w { $year }
stats-eps = { $count } odc.
stats-month-1 = sty
stats-month-2 = lut
stats-month-3 = mar
stats-month-4 = kwi
stats-month-5 = maj
stats-month-6 = cze
stats-month-7 = lip
stats-month-8 = sie
stats-month-9 = wrz
stats-month-10 = paź
stats-month-11 = lis
stats-month-12 = gru
stats-month-letter-1 = S
stats-month-letter-2 = L
stats-month-letter-3 = M
stats-month-letter-4 = K
stats-month-letter-5 = M
stats-month-letter-6 = C
stats-month-letter-7 = L
stats-month-letter-8 = S
stats-month-letter-9 = W
stats-month-letter-10 = P
stats-month-letter-11 = L
stats-month-letter-12 = G

## Backup and image cache

backup-title = Backup
backup-export = Utwórz kopię biblioteki
backup-export-helper = Jeden plik z folderami, dopasowaniami i tym, co obejrzano.
backup-private = Dołącz dane prywatne
backup-private-helper = Dodaje logowania do AniList i MyAnimeList, historię oglądania i preferencje. Logowania są zapisane zwykłym tekstem, więc nie udostępniaj pliku.
backup-restore = Przywróć z kopii
backup-restore-helper = Wczytuje kopię do tej biblioteki. Wygrywa to, co jest na dysku; plik uzupełnia resztę.
backup-done = Kopia gotowa. Plik jest tam, gdzie go zapisano.
backup-not-saved = Nie zapisano. Nic nie zostało zapisane.
backup-starting = AniBeam jeszcze się uruchamia. Spróbuj za chwilę.
backup-images-title = Pamięć obrazów
backup-images-body = Plakaty, banery i portrety są trzymane w telefonie, żeby biblioteka działała offline. Wyczyszczone obrazy pobierają się znów, gdy strony ich potrzebują.
backup-keep = Zachowaj
backup-removed = { $count ->
    [one] Usunięto { $count } obraz
    [few] Usunięto { $count } obrazy
    [many] Usunięto { $count } obrazów
   *[other] Usunięto { $count } obrazów
}
backup-images-none = Brak obrazów
backup-images-unmeasured = Nie udało się zmierzyć pamięci obrazów
backup-images-retry-note = Spróbuj za chwilę. Same obrazy są w porządku.
backup-images-line = { $size } w { $count ->
    [one] { $count } obrazie
    [few] { $count } obrazach
    [many] { $count } obrazach
   *[other] { $count } obrazach
}
backup-images-question = { $count ->
    [one] Usunąć { $count } obraz?
    [few] Usunąć { $count } obrazy?
    [many] Usunąć { $count } obrazów?
   *[other] Usunąć { $count } obrazów?
}

backup-sum-list = { $items }.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] dodano { $count } folder
    [few] dodano { $count } foldery
    [many] dodano { $count } folderów
   *[other] dodano { $count } folderów
}
backup-sum-series = { $count ->
    [one] utworzono { $count } serial
    [few] utworzono { $count } seriale
    [many] utworzono { $count } seriali
   *[other] utworzono { $count } seriali
}
backup-sum-matches = { $count ->
    [one] zastosowano { $count } dopasowanie
    [few] zastosowano { $count } dopasowania
    [many] zastosowano { $count } dopasowań
   *[other] zastosowano { $count } dopasowań
}
backup-sum-views = scalono { $views ->
    [one] { $views } wpis oglądania
    [few] { $views } wpisy oglądania
    [many] { $views } wpisów oglądania
   *[other] { $views } wpisów oglądania
} i { $episodes ->
    [one] { $episodes } ukończony odcinek
    [few] { $episodes } ukończone odcinki
    [many] { $episodes } ukończonych odcinków
   *[other] { $episodes } ukończonych odcinków
}
backup-sum-resume = { $count ->
    [one] scalono { $count } punkt wznowienia
    [few] scalono { $count } punkty wznowienia
    [many] scalono { $count } punktów wznowienia
   *[other] scalono { $count } punktów wznowienia
}
backup-sum-accounts = { $count ->
    [one] dodano { $count } konto listy
    [few] dodano { $count } konta list
    [many] dodano { $count } kont list
   *[other] dodano { $count } kont list
}
backup-sum-fixes = { $count ->
    [one] { $count } plik przyjmuje Twoją poprawkę.
    [few] { $count } pliki przyjmują Twoją poprawkę.
    [many] { $count } plików przyjmuje Twoją poprawkę.
   *[other] { $count } plików przyjmuje Twoją poprawkę.
}
backup-sum-missing = { $count ->
    [one] Nie znaleziono { $count } folderu na tym telefonie.
    [few] Nie znaleziono { $count } folderów na tym telefonie.
    [many] Nie znaleziono { $count } folderów na tym telefonie.
   *[other] Nie znaleziono { $count } folderów na tym telefonie.
}
backup-sum-left-out = Pominięto: { $fields }.

## Activity

activity-everything = Wszystko
activity-problems = Problemy
activity-copied = Skopiowano
activity-clear-every-line = Wyczyść wszystko
activity-fold = Zwiń
activity-no-problems = Brak problemów. Wszystko działało.
activity-nothing-yet = Na razie nic do zgłoszenia. To, co AniBeam robi dla Ciebie, pojawi się tutaj.
activity-time-today = %H:%M:%S
activity-time-other = %-d %b %H:%M

## Dates and numbers (companion format.rs)

fmt-clock = %H:%M
fmt-day-month = %-d %b
fmt-weekday-date-time = %a %-d %b %H:%M
fmt-weekday-date-year-time = %a %-d %b %Y %H:%M
fmt-stamp = %-d %b %H:%M
fmt-decimal-point = ,

fmt-today-at = Dziś { $time }
fmt-tomorrow-at = Jutro { $time }
fmt-yesterday-at = Wczoraj { $time }

fmt-span-days = { $d } d { $h } h
fmt-span-hours = { $h } h { $m } min
fmt-span-minutes = { $m } min { $s } s
fmt-span-seconds = { $s } s
phone-fmt-in = za { $span }
fmt-full-days = { $d } d { $h } h { $m } min { $s } s
fmt-full-hours = { $h } h { $m } min { $s } s
fmt-full-minutes = { $m } min { $s } s

fmt-ago-now = przed chwilą
fmt-ago-minutes = { $count } min temu
fmt-ago-hours = { $count } h temu
fmt-ago-days = { $count } d temu

fmt-bytes-b = { $value } B
fmt-bytes-kb = { $value } KB
fmt-bytes-mb = { $value } MB
fmt-bytes-gb = { $value } GB
fmt-bytes-tb = { $value } TB

## Talking to the computer (companion client.rs, sync.rs, address.rs)

client-not-found = Telefon nie może znaleźć nazwy { $host }. Czy Tailscale jest połączony?
client-timeout = { $host } nie odpowiada. Czy jest włączony i czy Tailscale jest połączony na obu urządzeniach?
client-connect-failed = Nie udało się połączyć z { $host }. Czy jest włączony i czy Tailscale jest połączony na obu urządzeniach?
client-refused = { $host } jest włączony, ale AniBeam na nim jest zamknięty albo niczego nie udostępnia. Otwórz na nim AniBeam i włącz „Share over Tailscale” w Settings, Phone app.
client-no-route = Brak trasy do { $host }. Czy Tailscale jest połączony?
client-invalid-input = Nie udało się połączyć z { $host }
client-unreachable = Nie udało się połączyć z { $host }: { $why }
client-cannot-find = Telefon nie może znaleźć { $host }. Czy Tailscale jest połączony?
client-broke-off = Odpowiedź od { $host } urwała się: { $why }
client-answered = { $host } odpowiada kodem { $status } dla { $path }
client-not-anibeam-status = Coś odpowiedziało, ale to nie AniBeam ({ $host } odpowiada kodem { $status } dla { $path })
client-not-anibeam-hello = Coś odpowiedziało, ale to nie AniBeam (odpowiedź nie jest powitaniem AniBeam)
client-not-anibeam-name = Coś odpowiedziało, ale to nie AniBeam (przedstawia się jako { $name })
client-too-new = Host mówi API { $host_api }; ta aplikacja czyta { $app_api }. Zaktualizuj aplikację.
client-snapshot-unreadable = Nie udało się odczytać migawki: { $why }
client-probe-unreadable = Nie udało się odczytać szczegółów pliku: { $why }
client-peers-unreadable = Nie udało się odczytać listy AniBeamów: { $why }
sync-save-failed = Nie udało się zapisać migawki: { $why }
address-https = AniBeam na Twoim komputerze odpowiada zwykłym http przez Tailscale; usuń https://.
address-empty = Wpisz nazwę lub adres hosta, na przykład moj-komputer:8790.
address-ipv6-close = Adres IPv6 wymaga zamykającego ].
address-ipv6-port = Wpisz port po ], jak [fd7a::1]:8790.
address-bad-host = „{ $text }” nie jest nazwą hosta.
address-bad-port = „{ $port }” nie jest portem.

## Series facts (companion library.rs)

clib-status-airing = Emitowane
clib-status-not-yet = Przed emisją
clib-format-tv-short = TV krótki
clib-format-movie = Film
clib-format-special = Specjal
clib-format-music = Muzyka

## Search sites and repos

repos-by-hand-title = Dodaj ręcznie
repos-by-hand-note = Dla witryny, której nie ma w żadnym repozytorium. Dowolny kanał z { "{query}" } w miejscu wyszukiwania, także kanał Jackett lub Prowlarr. Klucz lub token w adresie trafia do magazynu sekretów tego telefonu.
repos-name-placeholder = Nazwa
repos-url-placeholder = https://...{ "{query}" }...
repos-index-placeholder = https://.../index.json
repos-add-site = Dodaj
repos-add-title = Dodaj repo
repos-add-note = Wklej adres indeksu repozytorium. AniBeam sprawdzi jego podpis i pokaże klucz, zanim cokolwiek zostanie zapisane.
repos-add-repo = Dodaj repo
repos-trust-title = Zaufać repo?
repos-fingerprint = Odcisk klucza
repos-trust-warning = Jego witryny decydują, dokąd trafiają Twoje wyszukiwania. Ufaj repozytorium tylko od zaufanej osoby.
repos-trust = Zaufaj
repos-open-link = Otwórz { $url }
repos-saving = Zapisuję...
repos-install = Zainstaluj
repos-update = Aktualizuj
repos-edit-note = Klucz lub token wpisany w adresie trafia do magazynu sekretów tego telefonu.
repos-edit-note-keys = Zostaw każde { "{key}" } w adresie bez zmian; jego wartość zostaje w tym telefonie.
repos-change-its = Zmień: { $keys }
repos-check-repos = Sprawdź nowe witryny w repo
repos-check-sites = Sprawdź witryny
repos-add-site-label = Dodaj witrynę ręcznie
repos-needs-before-search = Przed wyszukiwaniem wymaga: { $labels }
repos-no-longer-offered = Repozytorium już tego nie oferuje
repos-update-host = Aktualizacja wysyła żądania do { $host }
repos-tap-change-custom = Dotknij, aby zmienić nazwę lub adres
repos-tap-change-keys = Dotknij, aby zmienić: { $labels }
repos-line-custom = { $host }  { $kind }  dodane przez Ciebie
repos-update-to = Aktualizuj do v{ $version }
repos-add-its-value = Dodaj wartość
repos-asks-for = Prosi o: { $labels }
repos-preview-line = { $count ->
    [one] { $count } witryna, podpisana
    [few] { $count } witryny, podpisane
    [many] { $count } witryn, podpisanych
   *[other] { $count } witryn, podpisanych
}
repos-preview-trusted = { $count ->
    [one] { $count } witryna, podpisana, już zaufana
    [few] { $count } witryny, podpisane, już zaufane
    [many] { $count } witryn, podpisanych, już zaufanych
   *[other] { $count } witryn, podpisanych, już zaufanych
}
repos-ask-kept-empty = Zapisane tylko w tym telefonie. Puste pole zachowuje zapisaną wartość.
repos-ask-kept-never = Zapisane tylko w tym telefonie, nigdy w bibliotece ani eksporcie.
repos-ask-new-host = Nowa wersja wysyła to do { $host }, więc pyta ponownie.
repos-ask-new = Nowa wersja ponownie prosi o te dane.
repos-type-value = Najpierw wpisz wartość.
repos-health-answered = Odpowiedź w { $ms } ms
repos-health-failed = Brak odpowiedzi. { $step }: { $detail }

## Downloads

dl-title = Pobrane
dl-wifi-only = Tylko Wi-Fi
dl-wifi-and-mobile = Wi-Fi i dane komórkowe
dl-joined = { $first }, { $second }
dl-batch-files = { $count ->
    [one] Paczka, { $count } plik
    [few] Paczka, { $count } pliki
    [many] Paczka, { $count } plików
   *[other] Paczka, { $count } plików
}
dl-batch-tag = Paczka
dl-downloading-count = { $count } w toku
dl-free-on-phone = wolne w telefonie
dl-search-placeholder = Serial lub nazwa wydania
dl-sources = Źródła
dl-downloading = Pobieranie
dl-queue-note = { $count ->
    [one] { $count } pobranie
    [few] { $count } pobrania
    [many] { $count } pobrań
   *[other] { $count } pobrań
}
dl-finished = Zakończone
dl-finished-note = { $count ->
    [one] { $count } pobranie, w Twojej bibliotece
    [few] { $count } pobrania, w Twojej bibliotece
    [many] { $count } pobrań, w Twojej bibliotece
   *[other] { $count } pobrań, w Twojej bibliotece
}
dl-get = Pobierz
dl-pick = Wybierz

## Storage and hidden shows

disk-on-phone = W telefonie
disk-watched = Obejrzane
disk-not-watched = Nieobejrzane
disk-deleting = Usuwanie
disk-delete-watched = Usuń obejrzane
disk-hidden-shows = Ukryte seriale
disk-hidden-helper = Poza Startem i biblioteką, dopóki ich nie przywrócisz.
disk-unhide = Pokaż
disk-delete-note = Pliki i napisy zostaną usunięte z tego telefonu na stałe.
disk-every-watched = Każdy obejrzany odcinek
disk-every-finished = Każdy ukończony i obejrzany serial
disk-episodes-size = { $count ->
    [one] { $count } odcinek, { $size }
    [few] { $count } odcinki, { $size }
    [many] { $count } odcinków, { $size }
   *[other] { $count } odcinków, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } serial, { $size }
    [few] { $count } seriale, { $size }
    [many] { $count } seriali, { $size }
   *[other] { $count } seriali, { $size }
}
disk-list-two = { $a } i { $b }
disk-list-three = { $a }, { $b } i { $c }
disk-list-more = { $a }, { $b }, { $c } i jeszcze { $count }
disk-delete-finished-title = Usunąć { $count ->
    [one] { $count } ukończony serial
    [few] { $count } ukończone seriale
    [many] { $count } ukończonych seriali
   *[other] { $count } ukończonych seriali
}, { $size }?
disk-delete-watched-title = Usunąć { $count ->
    [one] { $count } obejrzany odcinek
    [few] { $count } obejrzane odcinki
    [many] { $count } obejrzanych odcinków
   *[other] { $count } obejrzanych odcinków
}, { $size }?
disk-delete-finished-body = { $names }. Tylko seriale, które według dostawcy się zakończyły, z każdym odcinkiem obejrzanym; serial wciąż emitowany albo taki, którego koniec jest domysłem, zostaje. Każdy ich plik zostanie usunięty z tego telefonu, także dodatki. Ich pobrania też znikną, więc można je pobrać ponownie.
disk-delete-watched-body = Każdy obejrzany odcinek, w każdym serialu. Odcinki jeszcze nieobejrzane i dodatki zostają. Pliki i ich napisy zostaną usunięte z tego telefonu na stałe.
disk-no-finished = Żaden ukończony serial nie jest obejrzany do końca
disk-nothing-watched = Na telefonie nie ma nic obejrzanego
disk-deleted = Usunięto { $count ->
    [one] { $count } odcinek
    [few] { $count } odcinki
    [many] { $count } odcinków
   *[other] { $count } odcinków
}, { $size }
disk-delete-failed = Nic nie usunięto: { $error }

## Download notifications

notify-card-head = AniBeam · teraz
notify-sample-called = Frieren, odcinek 7
notify-sample-called-ready = Frieren, odcinek 6
notify-sample-facts = 420 MB z 1,4 GB, 3,2 MB/s, jeszcze 5 min
notify-downloading = Pobieranie: { $called }
notify-progress = { $percent }%, { $facts }
notify-progress-of = { $percent }%, { $done } z { $size }
notify-starting = Uruchamianie
notify-many = { $count ->
    [one] { $count } pobranie
    [few] { $count } pobrania
    [many] { $count } pobrań
   *[other] { $count } pobrań
}
notify-called-episode = { $show }, odcinek { $n }
notify-called-episodes = { $show }, odcinki { $from } do { $to }
notify-ready = { $called } jest gotowy
notify-ready-many = { $called } są gotowe
notify-ready-batch = Paczka { $show } jest gotowa
notify-ready-text = Pobrano na ten telefon. Dotknij, aby otworzyć AniBeam.
notify-channel-downloads = Pobieranie
notify-channel-ready = Gotowe odcinki
notify-fallback-title = Pobieranie
notify-row-on = Powiadomienia
notify-row-allow = Zezwalaj na powiadomienia
notify-row-open-settings = Otwórz ustawienia powiadomień
notify-helper-on = Włączone. Pobieranie widać w powiadomieniach, gdy trwa.
notify-helper-asks-next = Android zapyta zaraz.
notify-helper-asks-next-later = Android zapyta w następnym kroku. Możesz zmienić zdanie później w Więcej.
notify-helper-off = Wyłączone. Włącz tam powiadomienia dla AniBeam.
notify-question-title = Pokazywać pobieranie w powiadomieniach?
notify-question-body = Gdy odcinki się pobierają, jedno powiadomienie pokazuje postęp, a drugie informuje, kiedy odcinek jest gotowy. Pobierają się tak czy inaczej; to tylko decyduje, czy to widzisz.
notify-allow = Zezwól
notify-not-now = Nie teraz
notify-pointer = Pobieranie nie jest widoczne w powiadomieniach.
notify-turn-on = Włącz

## Feed

phone-feed-title = Nowości
feed-recently-released = Ostatnio wydane
feed-coming-soon = Wkrótce
feed-this-week = Ten tydzień
feed-earlier = Wcześniej
feed-everything-else = Reszta
phone-feed-countdown-days = { $d } d { $h } h
phone-feed-countdown-hours = { $h } h { $m } min
phone-feed-countdown-minutes = { $m } min
phone-feed-next = Następny: { $title } za { $countdown }
phone-feed-nothing-scheduled = Nic jeszcze nie zaplanowano.
phone-feed-nothing-new = Nic nowego w tym tygodniu.
phone-feed-new-episodes = { $count ->
    [one] { $count } nowy odcinek w tym tygodniu.
    [few] { $count } nowe odcinki w tym tygodniu.
    [many] { $count } nowych odcinków w tym tygodniu.
   *[other] { $count } nowych odcinków w tym tygodniu.
}
phone-feed-aired = wyszło { $ago }
phone-feed-downloaded = pobrano { $ago }

## Files and the file picker

files-already-picking = Plik jest już wybierany.
files-took-too-long = Wybieranie pliku trwało zbyt długo.
files-did-not-open = Nie udało się otworzyć selektora plików: { $error }
files-none-here = Brak selektora plików.

## The computer's downloads

desk-sent-adding = Wysłano na komputer { $name }: { $what }. Wciąż je dodaje.
desk-sent = Pobieranie na komputerze { $name }: { $what }.

## Your computer's library

remote-watched = Obejrzane
remote-movie = Film
remote-not-watched-yet = Jeszcze nieobejrzane
remote-watched-of = Obejrzano { $done } z { $total }
remote-watched-count = Obejrzano: { $count }
remote-on-disk = { $count } na dysku
remote-airs-in-season = S{ $season } Odcinek { $n } za { $countdown }
remote-airs-in = Odcinek { $n } za { $countdown }
remote-play-episode = Odtwórz odcinek { $n }
remote-watch-again = Obejrzyj odcinek { $n } ponownie
remote-update-host = Zaktualizuj AniBeam na tamtym komputerze, aby odtwarzać to tutaj
remote-aired = wyszło
remote-missing = Brakuje
remote-on-computer = Na Twoim komputerze
remote-out = Wyszło { $ago }
remote-season-episode = Sezon { $season }, odcinek { $n }

## Wyszukiwarka w Więcej: słowa, które ktoś mógłby wpisać dla każdego ustawienia

more-words-ratings = imdb autorzy oceny ocena notowania

## Postacie, powiązane seriale i polecenia serialu

show-format-series = Serial

## Odcinki i daty: liczba odcinków i daty emisji sezonu, ustawiane ręcznie

dates-title = Odcinki i daty
dates-season-line = { $title } · Sezon { $season }
dates-paste-open = Wklej listę
dates-paste-done = Gotowe
dates-paste-hint = Wklej listę premier, tabelę lub wpis
dates-paste-note = To, co AniBeam odczyta, pojawi się przy odcinkach poniżej.
dates-tap-hint = Dotknij odcinka, aby ustawić datę.
dates-run = Wybierz zakres
dates-field-hint = 7 sty, marzec 2027, 2027
dates-set = Ustaw datę
dates-no-date = Bez daty
dates-weekly = Co tydzień
dates-last = Ostatni odcinek
dates-not-last = Nie ostatni
dates-clear = Wyczyść datę
dates-save = Zapisz
dates-saving = Zapisywanie
dates-cancel = Anuluj
dates-change = Zmień
dates-picked-one = Odcinek { $episode }:
dates-picked-many =
    { $count ->
        [one] { $count } odcinek:
        [few] { $count } odcinki:
        [many] { $count } odcinków:
       *[other] { $count } odcinków:
    }
dates-saved = Zapisano.
dates-cleared = Notatki wyczyszczone.
dates-starting = AniBeam jeszcze się uruchamia. Spróbuj za chwilę.
dates-tile-out = Wyszło
dates-tile-last = Ostatni
dates-tile-none = Brak daty
dates-tile-guess = ≈ { $date }
dates-summary-none = Nic nie ustawiono
dates-summary-ends = Kończy się na odcinku { $episode }
dates-summary-record =
    { $count ->
        [one] { $count }: daty z { $site }
        [few] { $count }: daty z { $site }
        [many] { $count }: daty z { $site }
       *[other] { $count }: daty z { $site }
    }
dates-summary-record-unnamed =
    { $count ->
        [one] { $count }: daty z bazy
        [few] { $count }: daty z bazy
        [many] { $count }: daty z bazy
       *[other] { $count }: daty z bazy
    }
dates-summary-set =
    { $count ->
        [one] Ręcznie: { $count }
        [few] Ręcznie: { $count }
        [many] Ręcznie: { $count }
       *[other] Ręcznie: { $count }
    }
dates-summary-guessed =
    { $count ->
        [one] Szacunek co tydzień: { $count }
        [few] Szacunek co tydzień: { $count }
        [many] Szacunek co tydzień: { $count }
       *[other] Szacunek co tydzień: { $count }
    }
dates-told-one = Odcinek { $episode }
dates-told-many = Odcinki od { $first } do { $last }
dates-told-undated = { $episodes }: jeszcze bez daty
dates-told-due = { $episodes }: za { $when }
dates-told-ends = Kończy się na odcinku { $episode }
dates-told-season-ends = Sezon { $season } kończy się na odcinku { $episode }
dates-told-by-you = { $told }, ustawiono ręcznie
dates-told-gave-way =
    { $count ->
        [one] Ręcznie ustawiono { $count } odcinek; to już nie obowiązuje
        [few] Ręcznie ustawiono { $count } odcinki; to już nie obowiązuje
        [many] Ręcznie ustawiono { $count } odcinków; to już nie obowiązuje
       *[other] Ręcznie ustawiono { $count } odcinka; to już nie obowiązuje
    }

## Strona serialu

series-runtime =
    { $hours ->
        [0] { $minutes } min
       *[other] { $minutes ->
            [0] { $hours } h
           *[other] { $hours } h { $minutes } min
        }
    }
series-runtime-each = po { $runtime }
series-status-airing = Emitowane
series-status-finished = Zakończone
series-status-upcoming = Wkrótce
series-status-cancelled = Anulowane
series-status-hiatus = Na przerwie
series-average-score = Średnia ocena { $score }
series-imdb = IMDb { $score }
series-imdb-votes = IMDb { $score }, głosów: { $votes }
series-rewatched =
    { $count ->
        [one] Obejrzano ponownie { $count } raz
        [few] Obejrzano ponownie { $count } razy
        [many] Obejrzano ponownie { $count } razy
       *[other] Obejrzano ponownie { $count } raza
    }
series-not-tracked = Nie zapisano
series-not-tracked-on-phone =
    { $count ->
        [one] Nie zapisano, { $count } odcinek w telefonie
        [few] Nie zapisano, { $count } odcinki w telefonie
        [many] Nie zapisano, { $count } odcinków w telefonie
       *[other] Nie zapisano, { $count } odcinka w telefonie
    }

## Więcej, Info

about-ratings = Oceny
about-ratings-credit = Informacje dzięki uprzejmości IMDb (https://www.imdb.com). Użyte za zgodą.

## Oznaczanie odcinków na stronie serialu

mark-menu-watched = Oznacz jako obejrzane do tego miejsca
mark-menu-unwatched = Oznacz jako nieobejrzane od tego miejsca
mark-menu-cancel = Anuluj
mark-watched-up-to = Obejrzano do odcinka { $episode }
mark-unwatched-from = Nieobejrzane od odcinka { $episode }
mark-not-connected = { $reason } Zaloguj się w Więcej, Konta.

## Odtwarzacz

player-speed-not-while-casting = Nie przy przesyłaniu

## Pobrane

downloads-other-release-on-disk = Inne wydanie na dysku

## Month patterns

dates-month-short = %b %Y
dates-month-long = %b %Y
