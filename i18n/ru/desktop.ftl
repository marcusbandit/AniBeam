# ru translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = Главная
nav-library = Библиотека
nav-airing = В эфире
nav-feed = Лента
nav-watching = Смотрю
nav-stats = Статистика
nav-metadata = О сериале
nav-downloads = Загрузки
nav-settings = Настройки
nav-series = Сериал
nav-player = Плеер
nav-show = Сериал
nav-recommended = Рекомендации

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } ошибка
    [few] { $count } ошибки
    [many] { $count } ошибок
   *[other] { $count } ошибки
}

## ScanSummary

scan-summary-nothing = Пока нечего показать
scan-summary-found = Найдено: { $count ->
    [one] { $count } сериал
    [few] { $count } сериала
    [many] { $count } сериалов
   *[other] { $count } сериала
}, распознано { $recognised }
scan-summary-found-need = Найдено: { $count ->
    [one] { $count } сериал
    [few] { $count } сериала
    [many] { $count } сериалов
   *[other] { $count } сериала
}, распознано { $recognised }, ждут вас { $need }
scan-summary-episodes = { $count ->
    [one] { $count } серия
    [few] { $count } серии
    [many] { $count } серий
   *[other] { $count } серии
} на диске.
scan-summary-episodes-watched = { $count ->
    [one] { $count } серия
    [few] { $count } серии
    [many] { $count } серий
   *[other] { $count } серии
} на диске, { $watched } уже просмотрено на { $tracker }.
scan-summary-title = Папка добавлена
scan-summary-count-found = найдено
scan-summary-count-recognised = распознано
scan-summary-count-need = ждут вас
scan-summary-later = Позже
scan-summary-match = Привязать { $count }
scan-summary-open-library = Открыть библиотеку

## UpdateNotice

update-notice-check-failed = AniBeam не смог проверить обновления. Проверьте подключение к интернету и попробуйте снова.
update-notice-up-to-date = AniBeam обновлён
update-notice-downloading = Загрузка AniBeam { $version }
update-notice-ready = AniBeam { $version } готов
update-notice-failed = Обновление не удалось
update-notice-on-aur = AniBeam { $version } есть в AUR
update-notice-out = Вышел AniBeam { $version }
update-notice-out-titled = Вышел AniBeam { $version } «{ $title }»
update-notice-progress = { $percent }%  ·  { $got } из { $total } МБ
update-notice-restart-hint = Перезапустите сейчас или он откроется в следующий раз
update-notice-from-source = Эта копия собрана из исходников: сделайте pull и пересоберите
update-notice-no-download = Для этого компьютера здесь нет загрузки
update-notice-install-hint = «Обновить» установит его и перезапустит AniBeam
update-notice-retry = Повторить
update-notice-update = Обновить
update-notice-restart = Перезапустить
update-notice-copied = Скопировано
update-notice-copy = Копировать
update-notice-release-page = Страница релиза
update-notice-later = Позже

## WhatsNew

whats-new-title = Что нового
whats-new-got-it = Понятно

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = d MMM HH:mm
activity-drawer-stage-library = Библиотека
activity-drawer-stage-metadata = Метаданные
activity-drawer-stage-trackers = Трекеры
activity-drawer-stage-franchise = Франшиза
activity-drawer-stage-playback = Воспроизведение
activity-drawer-stage-store = Хранилище
activity-drawer-stage-system = Система
activity-drawer-stage-downloads = Загрузки
activity-drawer-level-info = Инфо
activity-drawer-level-warn = Внимание
activity-drawer-level-error = Ошибка
activity-drawer-copied-line = Строка скопирована
activity-drawer-title = Журнал
activity-drawer-everything = Всё
activity-drawer-problems = Проблемы
activity-drawer-copied = Скопировано
activity-drawer-copy = Копировать
activity-drawer-clear-every = Очистить все строки
activity-drawer-copy-tip = Нажмите, чтобы скопировать строку
activity-drawer-fold = Свернуть
activity-drawer-empty-technical = Пока ничего не записано.
activity-drawer-empty-problems = Проблем нет. Всё сработало.
activity-drawer-empty = Пока не о чем сообщить. Здесь появится то, что AniBeam делает для вас.

## ActivityColumn

activity-column-episode-run = Серии с { $first } по { $last }
activity-column-episode-list = Серии { $list }
activity-column-downloads = { $count ->
    [one] { $count } загрузка
    [few] { $count } загрузки
    [many] { $count } загрузок
   *[other] { $count } загрузки
}
activity-column-batch = Пакетная загрузка
activity-column-download = Загрузка
activity-column-name-batch = { $title }, пакет
activity-column-name-episode = { $title }, серия { $number }
activity-column-title = Активность
activity-column-down = приём
activity-column-up = отдача
activity-column-not-connected = { $client } не подключён, поэтому здесь ничего не движется.
activity-column-connect-again = Подключить снова
activity-column-downloading = Скачивается
activity-column-fewer = Свернуть
activity-column-more = И ещё { $count }
activity-column-nothing-downloading = Ничего не скачивается.
activity-column-finished-today = Завершено сегодня
activity-column-today = Сегодня
activity-column-today-empty = Поиски, загрузки и новые подписки появляются здесь по мере событий.
activity-column-stopping = Остановка
activity-column-starting = Запуск
activity-column-start-tip = Начать загрузку
activity-column-stop-tip = Остановить загрузку; скачанное останется
activity-column-cancelling = Отмена
activity-column-cancel-all-tip = Отменить эти загрузки
activity-column-cancel-tip = Отменить эту загрузку
activity-column-size-of = { $got } из { $total }
activity-column-files-missing = Файлы пропали
activity-column-time-format = HH:mm
activity-column-left = Осталось { $time }
activity-column-waiting-peers = Ждём пиров
activity-column-episodes-done = { $count ->
    [one] Готово { $done } из { $count } серии
   *[other] Готово { $done } из { $count } серий
}
activity-column-cube-skipped = Серия { $number }: пропущена
activity-column-cube-done = Серия { $number }: готово
activity-column-cube-progress = Серия { $number }: { $percent }%

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Готово
frame-strip-progress = { $doing }, { $done } из { $total }
frame-strip-idle = Ничего не выполняется
frame-strip-time-format = HH:mm
frame-finish-sign-in = Сначала завершите вход в { $tracker }
frame-did-not-work = Не получилось.
frame-could-not-open = AniBeam не смог это открыть. Скопируйте и откройте сами.
frame-copy = Копировать
frame-copied = Скопировано
frame-status-refused = { $tracker } не принял изменение
frame-status-refused-why = { $tracker } не принял изменение: { $why }
frame-lists-refused = { $tracker } не принял { $count ->
    [one] { $count } сериал
    [few] { $count } сериала
    [many] { $count } сериалов
   *[other] { $count } сериала
}; подробности в журнале
frame-lists-moved = В { $tracker } перенесено: { $count ->
    [one] { $count } сериал
    [few] { $count } сериала
    [many] { $count } сериалов
   *[other] { $count } сериала
}
frame-notes-refused = { $tracker } не принял ваши заметки
frame-notes-refused-why = { $tracker } не принял ваши заметки: { $why }
frame-notes-saved = Заметки сохранены
frame-connected = Подключено к { $tracker } как { $user }
frame-sign-in-stopped = Вход в { $tracker } прервался, не завершившись.
frame-toast-hide-details = Скрыть подробности
frame-toast-details = Подробности
frame-toast-details-tip = Показать, что пошло не так, дословно, и скопировать
frame-forward = Вперёд
frame-copy-magnet = Копировать magnet
frame-copy-link = Копировать ссылку на торрент
frame-magnet-copied = Magnet скопирован
frame-link-copied = Ссылка скопирована
frame-open-release-page = Открыть страницу релиза
frame-open-folder = Открыть папку
frame-copy-path = Копировать путь
frame-path-copied = Путь скопирован
frame-mark-unwatched = Не просмотрено, начиная с этой
frame-mark-watched = Просмотрено до этой
frame-copy-title = Копировать название
frame-open-on-anilist = Открыть на AniList
frame-list-home-online = Изменится в каждом подключённом списке
frame-list-home-unmatched = Хранится на этом устройстве: ни один трекер не знает этот сериал
frame-list-home-not-carried = Хранится на этом устройстве: в вашем трекере нет этого сериала
frame-list-home-offline = Хранится на этом устройстве, пока вы не подключите AniList или MyAnimeList
frame-moving-to = Перенос в «{ $list }»
frame-moved-to = Перенесено в «{ $list }»
frame-plan-added = «{ $title }» добавлен в «В планах»
frame-plan-listed = «{ $title }» уже в вашем списке
frame-delete-watched = { $kind ->
    [film] Удалить просмотренные фильмы
   *[episode] Удалить просмотренные серии
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] Удалить фильм
       *[episode] Удалить серию
    }
   *[other] { $kind ->
        [film] Удалить все фильмы ({ $count })
       *[episode] Удалить все серии ({ $count })
    }
}
frame-series-gone = Этого сериала больше нет
frame-delete-none-watched = На диске нет просмотренных серий «{ $title }»
frame-delete-none = На диске нет ничего из «{ $title }»
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } просмотренный фильм
            [few] { $count } просмотренных фильма
            [many] { $count } просмотренных фильмов
           *[other] { $count } просмотренного фильма
        }
       *[episode] { $count ->
            [one] { $count } просмотренную серию
            [few] { $count } просмотренные серии
            [many] { $count } просмотренных серий
           *[other] { $count } просмотренной серии
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } фильм
            [few] { $count } фильма
            [many] { $count } фильмов
           *[other] { $count } фильма
        }
       *[episode] { $count ->
            [one] { $count } серию
            [few] { $count } серии
            [many] { $count } серий
           *[other] { $count } серии
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } доп. материал
    [few] { $count } доп. материала
    [many] { $count } доп. материалов
   *[other] { $count } доп. материала
}
frame-delete-and = { $first } и { $second }
frame-delete-heading = Удалить { $what } из «{ $title }»?
frame-delete-base = Файлы и их субтитры попадут в корзину, оттуда их можно вернуть. Их торренты уйдут из торрент-клиента, так что их можно будет скачать снова.
frame-delete-confirm = В корзину
frame-delete-done = «{ $title }»: { $what } в корзине
frame-delete-watched-done = { $count ->
    [one] { $count } серия
    [few] { $count } серии
    [many] { $count } серий
   *[other] { $count } серии
}, { $size }, в корзине
frame-delete-nothing = Нечего удалять
frame-fix-not-drafted = Исправлено. Не удалось подготовить issue
frame-fix-agent = Исправлено. { $agent } разбирается, issue откроется, когда он закончит
frame-fix-opening = Исправлено. Открываем issue
frame-fix-not-saved = Исправление не сохранилось: { $error }
frame-fix-undone = { $count ->
    [one] { $count } файл снова читается по имени
    [few] { $count } файла снова читаются по имени
    [many] { $count } файлов снова читаются по имени
   *[other] { $count } файла снова читаются по имени
}
frame-fix-done = Исправлено чтение { $count ->
    [one] { $count } файла
   *[other] { $count } файлов
}
frame-report-failed = Не удалось подготовить issue: { $error }
frame-report-untriaged = Issue открыт в браузере, без разбора. { $error }
frame-report-open = Issue открыт в браузере и готов к отправке
frame-downloads-not-read = Не прочитано.

## HomePage

home-title = Главная
home-scan-started = Сканирование запущено
home-ago-minutes = { $count } мин назад
home-ago-hours = { $count } ч назад
home-ago-days = { $count } дн. назад
home-today = Сегодня
home-tomorrow = Завтра
home-yesterday = Вчера
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d MMM
home-day-date-format = ddd, d MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = Даты пока нет
home-unit-days = { $count ->
    [one] день
    [few] дня
    [many] дней
   *[other] дня
}
home-unit-hours = ч
home-unit-minutes = мин
home-week-late = { $count ->
    [one] { $count } серия запаздывает
    [few] { $count } серии запаздывают
    [many] { $count } серий запаздывают
   *[other] { $count } серии запаздывают
}
home-week-waiting = { $count ->
    [one] { $count } вышедшая серия ждёт просмотра
    [few] { $count } вышедшие серии ждут просмотра
    [many] { $count } вышедших серий ждут просмотра
   *[other] { $count } вышедшей серии ждут просмотра
}
home-week-coming = { $count ->
    [one] { $count } серия в ближайшие семь дней
    [few] { $count } серии в ближайшие семь дней
    [many] { $count } серий в ближайшие семь дней
   *[other] { $count } серии в ближайшие семь дней
}
home-week-next = Далее: «{ $title }» через { $time }
home-countdown-days = { $days } д { $hours } ч
home-countdown-hours = { $hours } ч { $minutes } мин
home-countdown-minutes = { $minutes } мин
home-out-not-downloaded = Ещё не скачано
home-out-downloading = Скачивается
home-out-when = { $day } { $time }
home-next-up = Далее
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, время примерное
home-next-when-season = Сезон { $season }, { $span }, { $time }
home-next-when-season-estimated = Сезон { $season }, { $span }, { $time }, время примерное
home-no-date = Без даты
home-tile-watched = просмотрено
home-tile-downloading = скачивается
home-tile-out = вышла { $ago }
home-tile-not-out = ещё не вышла
home-tile-estimated = примерно
home-tile-season-span = С{ $season } { $span }
home-tile-to-watch = { $count } к просмотру
home-day-nothing = Пусто
home-now = Сейчас { $time }
home-card-aired = { $episode } вышла { $ago }
home-continue = Продолжить
home-continue-fact = Можно продолжить: { $count }
home-on-hold = Отложено
home-on-hold-fact = Ждут возвращения: { $count }
home-hero-open = Открыть «{ $title }»
home-hero-pick-up = Продолжить с того же места
home-hero-up-next = Далее
home-hero-downloading = Скачивается { $count ->
    [one] { $count } серия
    [few] { $count } серии
    [many] { $count } серий
   *[other] { $count } серии
}
home-hero-min-left = Осталось { $count } мин
home-hero-watched = Просмотрено { $watched } из { $total }
home-hero-resume = Продолжить серию { $number }
home-hero-play = Смотреть серию { $number }
home-hero-resume-unnumbered = Продолжить серию
home-hero-play-unnumbered = Смотреть серию
home-downloaded = Недавно скачано
home-downloaded-fact = С новыми файлами за две недели: { $count }
home-week = Эта неделя
home-week-show-empty = Показать пустые дни
home-week-hide-empty = Скрыть пустые дни
home-week-airing = В эфире
home-week-out = Уже вышло
home-week-coming-up = Скоро
home-recommended = Рекомендуем вам
home-recommended-fact = По вашему списку
home-recommended-scope = Рекомендации из
home-recommended-all = Все
home-recommended-in-library = В библиотеке
home-recommended-on-list = В списке
home-recommended-none-library = В библиотеке больше нечего рекомендовать. «Все» находит сериалы, которых у вас ещё нет.
home-recommended-none-list = В ваших списках больше нечего рекомендовать. «Все» находит сериалы, которых нет в ваших списках.
home-catch-up = Наверстать
home-catch-up-fact = С ожидающими сериями: { $count }
home-fresh = Начать новое
home-fresh-fact = Не начато: { $count }
home-genres = Жанры
home-open-library = Открыть библиотеку
home-dropped = Брошено
home-dropped-fact = { $count ->
    [one] { $count } файл ещё на диске
    [few] { $count } файла ещё на диске
    [many] { $count } файлов ещё на диске
   *[other] { $count } файла ещё на диске
}
home-out-now = Уже вышло
home-out-now-fact = Новые серии этой недели, по данным AniList
home-popular = Популярное в этом сезоне
home-add-folder-title = Сюда попадут и ваши сериалы
home-add-folder-body = Добавьте папку с аниме, и на главной появится то, что вы смотрите.
home-add-folder-button = Добавить папку
home-empty-scanned-title = В папках пока ничего не найдено
home-empty-title = Сериалов пока нет
home-empty-scanned-body = AniBeam не нашёл ни сериалов, ни фильмов в отслеживаемых папках. Просканируйте их снова или проверьте папки в настройках.
home-empty-body = Добавьте папку с аниме, и библиотека заполнится сама.
home-scan-all = Сканировать всё
home-library-settings = Настройки библиотеки
home-failed-title = Главная не загрузилась
home-failed-body = AniBeam не смог прочитать библиотеку. Попробуйте ещё раз через минуту.

## Onboarding

onboarding-language = Язык
onboarding-step-welcome = Привет
onboarding-step-library = Библиотека
onboarding-step-lists = Ваш список
onboarding-step-downloads = Загрузки
onboarding-step-ready = Готово
onboarding-scan-reading = Чтение папки
onboarding-scan-found = Найдено сериалов: { $count }. Продолжайте, когда захотите, остальное заполнится само.
onboarding-scan-starter-empty = Новая папка пока пуста. Сериалы, которые вы в неё положите, появятся здесь сами.
onboarding-scan-no-video = Видеофайлов не найдено. Проверьте, та ли это папка.
onboarding-scan-your-shows = ваши сериалы
onboarding-scan-looking-up = Ищем { $what }
onboarding-scan-looking-up-tally = Ищем { $what }, найдено { $posters } из { $count }
onboarding-scan-posters = Загрузка постеров
onboarding-scan-posters-tally = Загрузка постеров, найдено { $posters } из { $count }
onboarding-scan-seasons = Поиск связанных сезонов
onboarding-scan-seasons-tally = Поиск связанных сезонов, найдено { $posters } из { $count }
onboarding-choose-folder = Выберите папку с аниме
onboarding-about = Минута настройки. Библиотека достроится сама.
onboarding-step-later = Позже
onboarding-skip-setup = Пропустить настройку
onboarding-title-welcome = Добро пожаловать в AniBeam
onboarding-title-library = Где ваше аниме?
onboarding-title-lists = Подключите свой список аниме
onboarding-title-downloads = Как работают загрузки
onboarding-title-ready = Всё готово
onboarding-title-later = { $count ->
    [1] Готово, одно дело на потом
    [2] Готово, два дела на потом
   *[other] Готово, три дела на потом
}
onboarding-body-welcome = AniBeam держит ваше аниме в одном месте. Он находит сериалы на диске, проигрывает их прямо здесь и обновляет AniList или MyAnimeList, пока вы смотрите.
onboarding-body-library = Выберите папку, где лежат ваши сериалы. Каждая папка внутри станет сериалом, с постером, сериями и местом, где вы остановились. AniBeam только читает папку. Он никогда не перемещает и не переименовывает файлы.
onboarding-body-lists = Войдите один раз, и каждая досмотренная здесь серия отметится в вашем списке, ничего вводить не нужно. Вход проходит в браузере, поэтому AniBeam не видит ваш пароль.
onboarding-body-downloads = Подпишитесь на сериал на странице «Загрузки», и каждая новая серия попадёт в библиотеку по мере выхода. Для этого нужны две вещи: где искать и торрент-клиент, который скачивает.
onboarding-body-ready = Всё это можно изменить позже в настройках, а в разделе «Настройки, О программе» эту настройку можно пройти снова.
onboarding-skip-step = Пока пропустить
onboarding-get-started = Начать
onboarding-open-app = Открыть AniBeam
onboarding-continue = Далее
onboarding-payoff-library = Ваша библиотека
onboarding-payoff-library-line = Постеры, серии и что смотреть дальше, для каждого сериала в ваших папках.
onboarding-payoff-plays = Играет прямо здесь
onboarding-payoff-plays-line = Субтитры, пропуск опенинга, и он помнит, где вы остановились.
onboarding-payoff-list = Ваш список, синхронно
onboarding-payoff-list-line = Досмотрите серию, и она засчитается на AniList или MyAnimeList.
onboarding-payoff-episodes = Серии приходят сами
onboarding-payoff-episodes-line = Подпишитесь на сериал, и каждая новая серия попадёт в библиотеку.
onboarding-films-too = Папка с фильмами тоже подойдёт: назовите её Movies.
onboarding-choose-folder-button = Выбрать папку
onboarding-no-folder = У меня пока нет папки
onboarding-starter-where = Будет создана и использована папка Anime в { $dir }.
onboarding-add-another = Добавить ещё папку
onboarding-shelf-more = ещё
onboarding-preview-show = Ваш сериал
onboarding-preview-finished = Серия 3 досмотрена здесь
onboarding-preview-on-list = В вашем списке: просмотрено 3
onboarding-preview-set-by = AniBeam отмечает это, как только пошли титры
onboarding-lists-connected = Подключено. Всё, что вы смотрите здесь, теперь учитывается в вашем списке.
onboarding-lists-no-account = Нет аккаунта? Оба сервиса бесплатны, и одного вполне хватит. Отключиться можно в любой момент: Настройки, Аккаунты.
onboarding-search-title = Где искать
onboarding-search-ready = { $count ->
    [one] { $count } источник готов к поиску.
    [few] { $count } источника готовы к поиску.
    [many] { $count } источников готовы к поиску.
   *[other] { $count } источника готовы к поиску.
}
onboarding-search-none = В AniBeam нет своих источников. Их добавляют из репозитория источников: это ссылка, которой с вами поделились, её нужно вставить в «Настройки, Загрузки».
onboarding-sources-after = Откроется после настройки
onboarding-sources-add = Добавить репозиторий после настройки
onboarding-fetch-title = Кто скачивает
onboarding-fetch-auto = Пусть AniBeam всё сделает
onboarding-fetch-recommended = Рекомендуется
onboarding-fetch-auto-line = У AniBeam свой qBittorrent, который тихо работает в фоне. Ничего не нужно устанавливать и заполнять.
onboarding-fetch-existing = У меня уже есть qBittorrent
onboarding-fetch-existing-line = AniBeam подключится к вашему qBittorrent. Его адрес и логин вы введёте дальше в настройках.
onboarding-ready-library = Библиотека
onboarding-ready-library-none = Папки пока нет. Её можно добавить в «Настройки, Библиотека».
onboarding-ready-library-series = { $count ->
    [one] { $count } папка
    [few] { $count } папки
    [many] { $count } папок
   *[other] { $count } папки
}, пока сериалов: { $series }
onboarding-ready-library-empty = { $count ->
    [one] { $count } папка
    [few] { $count } папки
    [many] { $count } папок
   *[other] { $count } папки
}, пока пусто
onboarding-ready-library-scanning = { $count ->
    [one] { $count } папка
    [few] { $count } папки
    [many] { $count } папок
   *[other] { $count } папки
}, сканирование
onboarding-ready-lists = Ваш список
onboarding-ready-lists-as = { $tracker }, как { $user }
onboarding-ready-lists-none = Пока не подключено. Это можно сделать в «Настройки, Аккаунты».
onboarding-ready-downloads = Загрузки
onboarding-ready-downloads-line = { $sources ->
    [0] Источников пока нет
    [one] { $sources } источник
    [few] { $sources } источника
    [many] { $sources } источников
   *[other] { $sources } источника
}, { $client ->
    [own] ваш qBittorrent.
   *[bundled] qBittorrent от AniBeam.
} { $next ->
    [ready] Всё готово.
    [opens] Дальше откроется «Настройки, Загрузки».
   *[both] Оба находятся в «Настройки, Загрузки».
}
onboarding-ready-open-downloads = Открыть «Настройки, Загрузки»
onboarding-ready-set-up = Настроить сейчас

## Rail.qml

rail-home = Главная
rail-library = Библиотека
rail-airing = Эфир
rail-feed = Лента
rail-stats = Статистика
rail-show-info = О сериале
rail-downloads = Загрузки
rail-settings = Настройки
rail-downloads-complete =
    { $count ->
        [one] Загрузки: { $count } подписка завершена
        [few] Загрузки: { $count } подписки завершены
        [many] Загрузки: { $count } подписок завершены
       *[other] Загрузки: { $count } подписки завершены
    }
rail-title-language-name = Язык названий: { $language }. Нажмите, чтобы сменить
rail-title-language-english = Английский
rail-title-language-tip = Язык названий: ромадзи, английский или японский
rail-whats-new-in = Что нового в { $release }
rail-whats-new = Что нового
rail-settings-note = Настройки: { $note }

## FixSheet.qml

fix-kind-episode = Серия
fix-kind-opening = Опенинг
fix-kind-ending = Эндинг
fix-kind-preview = Превью
fix-kind-special = Спешл
fix-kind-other = Другое
fix-reads-season-episode = Сезон { $season }, серия { $number }
# An extra with its number: "Opening 2"
fix-reads-extra = { $kind } { $index }
fix-apply-failed = Не удалось применить исправления.
fix-title = Исправить чтение файлов
fix-lede = «{ $title }». Укажите, что на самом деле в каждом файле; все следующие сканы это сохранят.
fix-rescan = Пересканировать папку
fix-rescan-started = Папка читается заново
fix-filled-in = Заполнено
fix-fill-in = Заполнить
fix-column-file = Файл
fix-column-reads-as = Читается как
fix-column-should-be = Должно быть
fix-fixed = Исправлено
# The number field's placeholder for an episode, and for an extra that may have none
fix-number-placeholder = №
fix-index-placeholder = Нет
fix-keep = Оставить исправление
fix-leave-as-read = Оставить файл как читается
fix-read-by-name = Снова читать файл по имени
fix-report = Создать issue на GitHub
fix-report-body = Откроет заполненный issue, который вы отправите сами. Он публичный и называет эти файлы.
fix-report-body-agent = Откроет заполненный issue, который вы отправите сами. Он публичный и называет эти файлы. Сначала его разберёт { $agent }.
fix-close-unapplied =
    { $count ->
        [one] Закрыть, не применив { $count } исправление?
        [few] Закрыть, не применив { $count } исправления?
        [many] Закрыть, не применив { $count } исправлений?
       *[other] Закрыть, не применив { $count } исправления?
    }
fix-number-invalid = Сначала исправьте число, выделенное красным
fix-close-anyway = Всё равно закрыть
fix-keep-editing = Продолжить правку
fix-apply-count =
    { $count ->
        [one] Применить { $count } исправление
        [few] Применить { $count } исправления
        [many] Применить { $count } исправлений
       *[other] Применить { $count } исправления
    }

## MatchPanel.qml

match-matched-to = Привязано к «{ $title }»
match-applied = Привязка задана
match-found-tvmaze = Сериал найден на TVmaze
match-found-anilist = Найдено на AniList
match-found-anilist-season = Найдено на AniList, сезон { $season }
match-found-mal = Найдено на MyAnimeList
# The sites a series is matched on, two at a time: "AniList and MyAnimeList"
match-sites-and = { $first } и { $next }
match-matched-on = Привязано на { $sites }
match-provider-silent = { $provider } не ответил
# The heading in a walk through several series, with "2 of 5"
match-title-walk = Привязка, { $walk }
match-title = Привязка
match-skip = Пропустить
match-refresh-info = Обновить данные
match-refreshing = Обновление: «{ $title }»
match-clear-confirm = Сбросить: удаляет связь и заданное число серий
match-clear = Сбросить привязку
match-cleared = Привязка сброшена
match-search-placeholder = Поиск в { $provider }
match-link-tvmaze = Вставьте ссылку на TVmaze или IMDb
match-link-anilist = Вставьте ссылку на AniList или MyAnimeList
match-look-up = Найти
match-searching = Поиск в { $provider }
match-type-more = Чтобы искать в { $provider }, введите хотя бы две буквы.
match-no-results = В { $provider } ничего не найдено.

## WatchingPage.qml

# A card's progress code, the number padded to two digits: "Ep 03"
watching-episode-code = Сер. { $number }
watching-countdown-days = { $days } д { $hours } ч
watching-countdown-hours = { $hours } ч { $minutes } мин
watching-countdown-minutes = { $minutes } мин
watching-on-disk = На диске
watching-on-anilist = На AniList
watching-ready-to-start = Можно начинать
watching-not-in-library = Нет в библиотеке
watching-get-to-start = Скачайте серии, чтобы начать
# $when is a relative time such as "3d ago"
watching-anilist-updated = AniList обновлён { $when }
watching-airing = В эфире
# $wait is a short countdown such as "2d 4h"
watching-next-in = Далее: «{ $title }» через { $wait }
watching-episodes-waiting = Ждут просмотра
watching-series-behind =
    { $count ->
        [one] Отстаёте в { $count } сериале
       *[other] Отстаёте в { $count } сериалах
    }
watching-up-to-date = Всё просмотрено
watching-anilist-only = Только на AniList
watching-planning-summary-off = Что вы планируете посмотреть, по AniList.
watching-planning-summary =
    { $count ->
        [one] В планах { $count } сериал, на диске { $here }.
        [few] В планах { $count } сериала, на диске { $here }.
        [many] В планах { $count } сериалов, на диске { $here }.
       *[other] В планах { $count } сериала, на диске { $here }.
    }
watching-summary-off = Что вы смотрите или пересматриваете, по AniList.
watching-summary-waiting =
    { $count ->
        [one] В { $count } сериале ждут новые серии.
       *[other] В { $count } сериалах ждут новые серии.
    }
watching-caught-up = Всё просмотрено.
watching-start-watching = Начать просмотр
watching-play-next = Смотреть дальше
watching-open-on-anilist = Открыть на AniList
watching-refreshing = Обновление
watching-connect-title = AniList не подключён
watching-connect-body = Подключите его, и здесь будут сериалы, что вы смотрите или пересматриваете на AniList: «Смотреть» у скачанных, отсчёт у выходящих.
watching-connect-waiting = Ждём браузер
watching-connect-anilist = Подключить AniList
watching-load-failed-title = Список «Смотрю» не загрузился
watching-load-failed-body = AniList недоступен. Список вернётся при следующем обновлении.
watching-planning-failed-title = Список «В планах» не загрузился
watching-planning-failed-body = Не удалось прочитать его кэш. Попробуйте снова, или он вернётся при следующем обновлении.
watching-planning-empty-title = В списке «В планах» пусто
watching-planning-empty-body = «В планы» на странице «В эфире» или «В планах» в меню списка сериала добавляет его сюда.
watching-airing-now = В эфире
watching-empty-title = В списке «Смотрю» пусто
watching-empty-body = Отметьте сериал как «Смотрю» на AniList, и он появится здесь.
watching-open-anilist = Открыть AniList

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] Скрыт { $count } сериал
        [few] Скрыто { $count } сериала
        [many] Скрыто { $count } сериалов
       *[other] Скрыто { $count } сериала
    }
library-hidden-sync-stops =
    { $count ->
        [one] Скрыт { $count } сериал, синхронизация с трекером остановлена
        [few] Скрыто { $count } сериала, синхронизация с трекером остановлена
        [many] Скрыто { $count } сериалов, синхронизация с трекером остановлена
       *[other] Скрыто { $count } сериала, синхронизация с трекером остановлена
    }
library-unhidden =
    { $count ->
        [one] { $count } сериал снова виден и синхронизируется
        [few] { $count } сериала снова видны и синхронизируются
        [many] { $count } сериалов снова видны и синхронизируются
       *[other] { $count } сериала снова видны и синхронизируются
    }
library-undo = Отменить
library-clearing-score =
    { $count ->
        [one] Удаление оценки у { $count } сериала
       *[other] Удаление оценки у { $count } сериалов
    }
library-rating =
    { $count ->
        [one] Оценка { $score } для { $count } сериала
       *[other] Оценка { $score } для { $count } сериалов
    }
# $list is the list's name, such as Watching
library-moving =
    { $count ->
        [one] Перенос { $count } сериала в «{ $list }»
       *[other] Перенос { $count } сериалов в «{ $list }»
    }
library-rescanning =
    { $count ->
        [one] Пересканирование { $count } сериала
       *[other] Пересканирование { $count } сериалов
    }
library-finding-matches =
    { $count ->
        [one] Поиск совпадения для { $count } сериала
       *[other] Поиск совпадений для { $count } сериалов
    }
library-sort-last-viewed = Недавно смотрели
library-sort-downloaded = Недавно скачано
library-sort-title = Название
library-sort-progress = Прогресс
library-sort-average-score = Средняя оценка
library-sort-my-score = Моя оценка
library-dir-a-to-z = А-Я
library-dir-z-to-a = Я-А
library-dir-oldest = Сначала старые
library-dir-newest = Сначала новые
library-dir-least-watched = Сначала менее просмотренные
library-dir-most-watched = Сначала более просмотренные
library-dir-lowest = Сначала низкие
library-dir-highest = Сначала высокие
library-group-none = Нет
library-group-season = Сезон
library-group-type = Тип
library-group-genre = Жанр
library-group-watch-state = Статус просмотра
library-scan-started = Сканирование запущено
library-tab-all = Все
library-tab-series = Сериалы
library-tab-movies = Фильмы
library-tab-watched = Просмотрено
library-tab-hidden = Скрытые
library-count-movies =
    { $count ->
        [one] { $count } фильм
        [few] { $count } фильма
        [many] { $count } фильмов
       *[other] { $count } фильма
    }
library-count-watched = { $count } просмотрено
library-count-hidden = { $count } скрыто
library-count-series =
    { $count ->
        [one] { $count } сериал
        [few] { $count } сериала
        [many] { $count } сериалов
       *[other] { $count } сериала
    }
library-title = Библиотека
library-all-genres = Все жанры
library-search-placeholder = Поиск по названиям и папкам
library-sort = Сортировка
library-group = Группы
library-franchises-together = Франшизы вместе
library-watched-shows = Просмотренные
library-load-failed-title = Библиотека не загрузилась
library-no-matches = Ничего не найдено по запросу «{ $query }»
# $what is the tab: series, films, watched, hidden or shows
library-none-in-genre =
    { $what ->
        [series] Здесь нет сериалов в жанре «{ $genre }»
        [films] Здесь нет фильмов в жанре «{ $genre }»
        [watched] Здесь нет просмотренного в жанре «{ $genre }»
        [hidden] Здесь нет скрытого в жанре «{ $genre }»
       *[shows] Здесь нет ничего в жанре «{ $genre }»
    }
library-none-here =
    { $what ->
        [series] Здесь нет сериалов
        [films] Здесь нет фильмов
        [watched] Здесь нет просмотренного
        [hidden] Здесь нет скрытого
       *[shows] Здесь пусто
    }
library-load-failed-body = AniBeam не смог прочитать библиотеку. Попробуйте ещё раз через минуту.
library-no-matches-body = Проверьте написание или ищите по всей библиотеке во вкладке «Все».
library-none-in-genre-body = Здесь нет ничего в этом жанре. «Все жанры» покажет остальное.
library-none-in-tab-body = В библиотеке пока нет такого. «Все» показывает всё, что в ней есть.
library-all-filtered-body = Всё в библиотеке скрыто фильтрами этого вида.
library-clear-search = Очистить поиск
library-show-all = Показать все
library-picked = Выбрано: { $count }
library-pick-all = Все
library-pick-all-tip = Выбрать все показанные карточки (Ctrl+A)
library-unhide = Показать
library-hide = Скрыть
library-rate = Оценить
library-rate-tip = Одна оценка для всех выбранных, на каждом подключённом трекере
library-needs-tracker = Нужен подключённый трекер и привязанный сериал
library-move-to-list = В список
library-rescan = Пересканировать
library-find-match = Найти совпадение
library-find-match-tip = Искать каждый выбранный сериал без привязки
library-all-matched = У всех выбранных есть привязка
library-empty-scanned-title = В папках пока ничего не найдено
library-empty-title = Библиотека пуста
library-empty-scanned-body = AniBeam не нашёл ни сериалов, ни фильмов в отслеживаемых папках. Просканируйте их снова или проверьте папки в настройках.
library-empty-body = Добавьте папку с аниме, и библиотека заполнится сама, или импортируйте экспорт AniBeam.
library-scan-all = Сканировать всё
library-add-folder = Добавить папку
library-settings = Настройки библиотеки
library-import = Импорт
library-import-title = Импорт экспорта AniBeam
library-import-filter-export = Экспорт AniBeam (*.json)
library-import-filter-all = Все файлы (*)
library-import-started = Импорт запущен

## MetadataPage.qml

metadata-title = О сериалах
metadata-filter-all = Все { $count }
metadata-filter-series = Сериалы { $count }
metadata-filter-films = Фильмы { $count }
metadata-filter-missing = Пропавшие { $count }
metadata-scan-started = Сканирование запущено
# Complete's own sort
metadata-sort-added = Добавлено
metadata-sort-season = Сезон
metadata-sort-a-to-z = А-Я
metadata-section-unmatched = Не распознано
metadata-section-missing = Папка не найдена
metadata-section-incomplete = Не хватает серий
metadata-section-empty = Не скачано
metadata-section-airing = В эфире
metadata-section-complete = Полностью
metadata-no-info = Данных о сериале пока нет
# A tile's state line, short: "5 of 7 aired", "9 of 12", "12 of 12 +2"; $expected may be "?"
metadata-state-aired = { $have } из { $aired } вышедших
metadata-state-of = { $have } из { $expected }
metadata-state-of-extra = { $have } из { $expected } +{ $extra }
metadata-detail-none-aired = Ни одной из { $aired } вышедших серий нет на диске
metadata-detail-aired = На диске { $have } из { $aired } вышедших
metadata-detail-aired-missing = На диске { $have } из { $aired } вышедших, не хватает { $missing }
metadata-detail-none = Ни одной из { $expected } серий нет на диске
metadata-detail-missing = На диске { $have } из { $expected }, не хватает { $missing }
# The match panel's place in a walk through the series that need a match: "2 of 5"
metadata-walk-position = { $at } из { $total }
metadata-walk-done = Это был последний
metadata-walk-left =
    { $count ->
        [one] Без привязки остался { $count }
       *[other] Без привязки осталось { $count }
    }
metadata-countdown-days = { $days } д { $hours } ч
metadata-countdown-hours = { $hours } ч { $minutes } мин
metadata-countdown-minutes = { $minutes } мин
# $wait is a short countdown such as "2d 4h"
metadata-next-in = Следующая через { $wait }
metadata-files-of = { $have } из { $expected }
metadata-files-on-disk = Файлы на диске
metadata-forgot = «{ $title }» забыт
metadata-refreshed-failed = Обновлено { $refreshed }, не удалось обновить { $failed }
metadata-refreshed =
    { $count ->
        [one] Обновлён { $count } сериал
        [few] Обновлено { $count } сериала
        [many] Обновлено { $count } сериалов
       *[other] Обновлено { $count } сериала
    }
metadata-found =
    { $count ->
        [one] Найден { $count } сериал
        [few] Найдено { $count } сериала
        [many] Найдено { $count } сериалов
       *[other] Найдено { $count } сериала
    }
metadata-found-some-left =
    { $count ->
        [one] Найден { $count } сериал, не распознано ещё { $unmatched }
        [few] Найдено { $count } сериала, не распознано ещё { $unmatched }
        [many] Найдено { $count } сериалов, не распознано ещё { $unmatched }
       *[other] Найдено { $count } сериала, не распознано ещё { $unmatched }
    }
metadata-found-nothing = Ничего нового не найдено, не распознано ещё { $unmatched }
metadata-all-recognised = Все сериалы распознаны
metadata-count =
    { $count ->
        [one] { $count } сериал
        [few] { $count } сериала
        [many] { $count } сериалов
       *[other] { $count } сериала
    }
metadata-search-placeholder = Фильтр по названию
metadata-find-rest = Найти остальные
metadata-find-rest-started = Поиск папок, которых AniBeam пока не знает
metadata-find-rest-tip = Ищет каждую папку, которую AniBeam пока не знает, на сайтах с данными о сериалах в порядке из настроек
metadata-refresh-all = Обновить всё
metadata-refresh-all-started = Обновление данных о всех сериалах
metadata-refresh-all-tip = Заново загружает данные о каждом распознанном сериале
metadata-crawling = Поиск связей франшизы
metadata-crawl-progress = { $done } из { $total }
metadata-load-failed-title = Данные о сериалах не загрузились
metadata-empty-scanned-title = В папках пока ничего не найдено
metadata-no-matches = Ничего не найдено по запросу «{ $query }»
# $filter is the filter's code from the core: Series, Movies or MissingFiles
metadata-nothing-under =
    { $filter ->
        [Series] В «Сериалах» пусто
        [Movies] В «Фильмах» пусто
        [MissingFiles] В «Пропавших» пусто
       *[other] В «{ $filter }» пусто
    }
metadata-load-failed-body = AniBeam не смог прочитать библиотеку. Попробуйте ещё раз через минуту.
metadata-empty-scanned-body = AniBeam не нашёл сериалов в отслеживаемых папках. Просканируйте их снова или проверьте папки в настройках.
metadata-empty-body = Библиотека пуста. Добавьте папку с аниме, и AniBeam её просканирует.
metadata-no-matches-body = Здесь нет названий с этими словами.
metadata-nothing-under-body = Сейчас ни один сериал не подходит под этот фильтр. «Все» покажет остальное.
metadata-scan-all = Сканировать всё
metadata-add-folder = Добавить папку
metadata-clear-search = Очистить поиск
metadata-show-all = Показать все
metadata-refreshing = Обновление: «{ $title }»
metadata-refresh-tip = Заново загрузить данные об этом сериале
metadata-forget-armed = Удалить
metadata-forget-confirm-tip = Нажмите ещё раз, чтобы удалить этот сериал и историю его просмотра из AniBeam
metadata-forget-tip = Папки больше нет на диске: удалить этот сериал и его историю из AniBeam

## AiringPage.qml

airing-title = В эфире
airing-sort-popular = Популярность
airing-sort-score = Оценка
airing-sort-trending = В тренде
airing-sort-favourites = В избранном
airing-sort-soonest = Следующая серия
airing-sort-aired = Вышло серий
airing-sort-episodes = Число серий
airing-sort-start = Дата начала
airing-sort-duration = Длина серии
airing-sort-title = Название
airing-dir-most-popular = Сначала популярные
airing-dir-least-popular = Сначала менее популярные
airing-dir-lowest = Сначала низкие
airing-dir-highest = Сначала высокие
airing-dir-least-trending = Сначала менее трендовые
airing-dir-most-trending = Сначала трендовые
airing-dir-fewest = Сначала меньше
airing-dir-most = Сначала больше
airing-dir-soonest = Сначала ближайшие
airing-dir-latest = Сначала поздние
airing-dir-oldest = Сначала старые
airing-dir-newest = Сначала новые
airing-dir-shortest = Сначала короткие
airing-dir-longest = Сначала длинные
airing-dir-a-to-z = А-Я
airing-dir-z-to-a = Я-А
# A season and its year; $season is the code Winter, Spring, Summer or Fall
airing-season =
    { $season ->
        [Winter] Зима { $year }
        [Spring] Весна { $year }
        [Summer] Лето { $year }
        [Fall] Осень { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = { $minutes } мин назад
airing-ago-hours = { $hours } ч назад
airing-ago-days = { $days } дн. назад
# $episode is the episode's name, such as "Episode 7"; $ago a time such as "2h ago"
airing-episode-out = { $episode } вышла { $ago }
airing-no-date = Даты пока нет
airing-in-library = В библиотеке
airing-out-today =
    { $count ->
        [one] Сегодня вышла { $count } серия
        [few] Сегодня вышло { $count } серии
        [many] Сегодня вышло { $count } серий
       *[other] Сегодня вышло { $count } серии
    }
airing-dismissed-one = «{ $title }» больше не появится здесь
airing-dismissed =
    { $count ->
        [one] { $count } сериал больше не появится здесь
        [few] { $count } сериала больше не появятся здесь
        [many] { $count } сериалов больше не появятся здесь
       *[other] { $count } сериала больше не появятся здесь
    }
airing-planned-already-one = «{ $title }» уже в вашем списке
airing-planned-already-all = Все они уже в вашем списке
airing-planned =
    { $count ->
        [one] { $count } сериал добавлен в «В планах»
        [few] { $count } сериала добавлено в «В планах»
        [many] { $count } сериалов добавлено в «В планах»
       *[other] { $count } сериала добавлено в «В планах»
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } сериал добавлен в «В планах», { $already } уже были там
        [few] { $count } сериала добавлено в «В планах», { $already } уже были там
        [many] { $count } сериалов добавлено в «В планах», { $already } уже были там
       *[other] { $count } сериала добавлено в «В планах», { $already } уже были там
    }
airing-play-next = Смотреть дальше
airing-plan-to-watch = В планы
airing-get-more-episodes = Скачать ещё серии
airing-get-episodes = Скачать серии
airing-open-on-anilist = Открыть на AniList
airing-not-interested = Не интересно
airing-not-interested-count = Не интересно: { $count }
airing-not-interested-tip = Сериалы, отмеченные как неинтересные, в настройках
airing-refreshing = Обновление
airing-pick-season = Другой сезон
airing-filter-all = Все
airing-filter-lists = В моих списках
airing-filter-library = В библиотеке
airing-hide-lists = Скрыть мои списки
airing-hide-lists-tip = Не показывать сериалы из ваших списков AniList или MAL
airing-hide-library = Скрыть библиотеку
airing-hide-library-tip = Не показывать сериалы, серии которых у вас есть
airing-out-now = Уже вышло
airing-out-last-day =
    { $count ->
        [one] { $count } серия за последние 24 часа
        [few] { $count } серии за последние 24 часа
        [many] { $count } серий за последние 24 часа
       *[other] { $count } серии за последние 24 часа
    }
airing-out-fewer = Свернуть
airing-out-all = Все { $count }
airing-sort = Сортировка
airing-load-failed-title = AniList недоступен
airing-load-failed-body = Сериалы в эфире берутся с AniList. Они вернутся при следующем обновлении.
airing-reading-title = Читаем, что в эфире
airing-nothing-now = Сейчас ничего не выходит
airing-nothing-for = Ничего не выходит: { $season }
airing-nothing-read = Пока ничего не прочитано
airing-reading-body = Все выходящие сериалы с AniList. В первый раз это занимает несколько секунд.
airing-nothing-body = На AniList пока нет сериалов этого сезона. «Обновить» прочитает их снова, или выберите другой сезон.
airing-nothing-read-body = «Обновить» прочитает все выходящие сериалы с AniList.
airing-hides-empty-title = Из-за скрытия здесь ничего не осталось
airing-lists-empty-title = Ничего из этого нет в ваших списках
airing-library-empty-title = Ничего из этого нет в библиотеке
airing-hides-empty-body = Все сериалы здесь есть в ваших списках или библиотеке, а их вы скрыли.
airing-lists-empty-body = «В планы» на карточке добавляет сериал в ваш список.
airing-library-empty-body = «Скачать серии» на карточке скачивает сериал в библиотеку.
airing-show-everything = Показать всё
airing-picked = Выбрано: { $count }
airing-undo = Отменить

## EpisodeRow.qml

episode-row-date-time-format = ddd, d MMM, HH:mm
episode-row-date-upcoming-format = ddd, d MMM
episode-row-date-format = d MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Нажмите, чтобы отметить непросмотренным начиная с этой
episode-row-mark-watched-tip = { $state }. Нажмите, чтобы отметить просмотренным до этой
episode-row-next-up = Следующая
episode-row-versions =
    { $count ->
        [one] { $count } версия
        [few] { $count } версии
        [many] { $count } версий
       *[other] { $count } версии
    }
episode-row-versions-tip =
    { $count ->
        [one] Одна и та же серия из { $count } релиза. Нажмите, чтобы выбрать, какую смотреть
       *[other] Одна и та же серия из { $count } релизов. Нажмите, чтобы выбрать, какую смотреть
    }
episode-row-downloading = Скачивается { $percent }%
episode-row-get = Скачать серию
episode-row-predicted = Ожидается
episode-row-extra = Доп.
episode-row-resume = Продолжить с { $time }

## EpisodeCubes.qml

episode-cubes-one = Одна серия
episode-cubes-range =
    { $count ->
        [one] { $count } серия, с { $first } по { $last }
        [few] { $count } серии, с { $first } по { $last }
        [many] { $count } серий, с { $first } по { $last }
       *[other] { $count } серии, с { $first } по { $last }
    }
# $state is a cube state as it reads mid-sentence, the .inline form of a tokens-cube-* message.
episode-cubes-tip = Серия { $number }: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } доп. материал
        [few] { $count } доп. материала
        [many] { $count } доп. материалов
       *[other] { $count } доп. материала
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } непривязанный файл
        [few] { $count } непривязанных файла
        [many] { $count } непривязанных файлов
       *[other] { $count } непривязанного файла
    }
episodes-pane-on-disk = На диске
episodes-pane-not-on-disk = Нет на диске
episodes-pane-season = С{ $season }
episodes-pane-get-tip = Скачать серию { $number }
episodes-pane-none-on-disk = На диске нет серий
episodes-pane-change-match = Сменить привязку
episodes-pane-refreshing = Обновление: «{ $title }»

## ShowDetail.qml

show-detail-not-in-library = Этой папки пока нет в библиотеке. Когда скан её найдёт, здесь появятся постер и серии.
show-detail-film-on-disk = Фильм на диске.
show-detail-film-not-on-disk = Фильма пока нет на диске.
show-detail-complete-line = Сериал вышел, и все { $total } серий здесь. Больше ничего не выйдет, так что подписке делать нечего.
show-detail-finished = Вышел полностью.
show-detail-airing = В эфире.
show-detail-have-of-total = Здесь { $have } из { $total } серий.
show-detail-have =
    { $count ->
        [one] Здесь { $count } серия.
        [few] Здесь { $count } серии.
        [many] Здесь { $count } серий.
       *[other] Здесь { $count } серии.
    }
# $span is "Episode 7" or "Episodes 7 to 9"; $count is 1 for one episode, more for several
show-detail-next-predicted =
    { $count ->
        [one] { $span } ожидается через { $wait }.
       *[other] { $span } ожидаются через { $wait }.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span } выйдет через { $wait }.
       *[other] { $span } выйдут через { $wait }.
    }
# $list is the episode numbers, joined with commas
show-detail-missing =
    { $count ->
        [one] Серии { $list } здесь нет.
       *[other] Серий { $list } здесь нет.
    }
show-detail-missing-more = Серий { $list } и ещё { $more } здесь нет.
show-detail-downloaded = Скачано
show-detail-downloading = Скачивается { $percent }%
show-detail-taken-adding = Взято правилом, добавляется
show-detail-taken = Взято правилом
show-detail-left-out = Пропущено правилом
show-detail-subscribed-reading = Подписка оформлена. qBittorrent читает ленту.
show-detail-taken-moving = Правило берёт { $taken } из { $releases ->
        [one] { $releases } релиза
       *[other] { $releases } релизов
    } в ленте; ниже { $moving ->
        [one] скачивается { $moving }
       *[other] скачиваются { $moving }
    }.
show-detail-taken-held = Правило берёт { $taken } из { $releases ->
        [one] { $releases } релиза
       *[other] { $releases } релизов
    } в ленте; в клиенте { $torrents ->
        [one] { $torrents } торрент
        [few] { $torrents } торрента
        [many] { $torrents } торрентов
       *[other] { $torrents } торрента
    }.
show-detail-taken-adding-line = Правило берёт { $taken } из { $releases ->
        [one] { $releases } релиза
       *[other] { $releases } релизов
    } в ленте; qBittorrent { $taken ->
        [one] добавляет его.
       *[other] добавляет их.
    }
show-detail-feed-holds-nothing = В ленте пока ничего нет. Подходящие релизы скачаются, как только появятся в ней.
show-detail-nothing-fits = В ленте пока нет ничего подходящего под правило. Подходящие релизы скачаются, как только появятся.
show-detail-complete = Полностью
show-detail-open-in-library = Открыть в библиотеке
show-detail-feed-finds-nothing = Эта лента ничего не находит
show-detail-feed-empty-query = Она запрашивает у { $source } «{ $query }», ответ пустой, и до правила ничего нового не доходит. Короткий запрос обычно срабатывает; отброшенные слова можно оставить в правиле.
show-detail-feed-empty = Она запрашивает { $source }, ответ пустой, и до правила ничего нового не доходит. Короткий запрос обычно срабатывает; отброшенные слова можно оставить в правиле.
show-detail-feed-empty-earlier = qBittorrent всё ещё показывает то, что нашла прежняя версия этой ленты.
show-detail-repairing = Исправление
show-detail-repair = Исправить
show-detail-unsubscribe = Отписаться
show-detail-remove-torrents =
    { $count ->
        [one] Удалить торрент
        [few] Удалить { $count } торрента
        [many] Удалить { $count } торрентов
       *[other] Удалить { $count } торрента
    }
show-detail-searches = Ищет
show-detail-quoted = «{ $words }»
show-detail-whole-feed = всю ленту
show-detail-from = Откуда
show-detail-rule = Правило
show-detail-rule-unfollowable = Новые серии не скачать: источник отвечает на поиск, но у него нет ленты
show-detail-rule-active = Активно
show-detail-keeps = Берёт
show-detail-keeps-value = релизы с «{ $words }»
show-detail-drops = Отбрасывает
# $words is the quoted words, joined with show-detail-or
show-detail-drops-value = релизы с { $words }
show-detail-or = { $first } или { $second }
show-detail-last-took = Последнее взятое
show-detail-saves-to = Сохраняет в
show-detail-folder = Папка
show-detail-in-the-feed = В ленте
show-detail-feed-reading = Чтение ленты.
show-detail-feed-offers = Что сейчас есть в ленте, сначала новое.
show-detail-feed-is-empty = Лента пуста.
show-detail-read-again = Прочитать снова
show-detail-seeders =
    { $count ->
        [one] { $count } сид
        [few] { $count } сида
        [many] { $count } сидов
       *[other] { $count } сида
    }
show-detail-download = Скачать
show-detail-torrents = Торренты
show-detail-torrents-moving =
    { $count ->
        [one] Скачивается { $count }; полоски заполняются по ходу.
       *[other] Скачиваются { $count }; полоски заполняются по ходу.
    }
show-detail-torrents-held = Что есть в клиенте для этой папки.
show-detail-torrents-waiting = Ждём, пока qBittorrent добавит то, что берёт правило.
show-detail-torrents-none = В клиенте нет ничего для этой папки.

## SeriesPage.qml

series-trackers-both = { $first } и { $second }
series-count-cleared = Число серий сброшено
series-count-set = Задано серий: { $count }
series-held-one-undated = У серии { $first } пока нет даты
series-held-range-undated = У серий с { $first } по { $last } пока нет даты
# $when is a month and year (series-held-month-format) or a year alone
series-held-one-due = Серия { $first } ожидается: { $when }
series-held-range-due = Серии с { $first } по { $last } ожидаются: { $when }
series-held-month-format = MMMM yyyy
series-fix-schedule = Серии и даты
series-fix-files = Файлы
series-fix-rescan-folder = Пересканировать папку
series-fix-match = Привязка
series-find-match = Найти совпадение
series-rescan-show = Пересканировать сериал
series-open-in-show-info = Открыть в «О сериалах»
# $kind is what the picks are: film, episode or file
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } фильм перемещён в корзину
            [few] { $count } фильма перемещено в корзину
            [many] { $count } фильмов перемещено в корзину
           *[other] { $count } фильма перемещено в корзину
        }
        [episode] { $count ->
            [one] { $count } серия перемещена в корзину
            [few] { $count } серии перемещено в корзину
            [many] { $count } серий перемещено в корзину
           *[other] { $count } серии перемещено в корзину
        }
       *[file] { $count ->
            [one] { $count } файл перемещён в корзину
            [few] { $count } файла перемещено в корзину
            [many] { $count } файлов перемещено в корзину
           *[other] { $count } файла перемещено в корзину
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] Выбран { $count } фильм.
            [few] Выбрано { $count } фильма.
            [many] Выбрано { $count } фильмов.
           *[other] Выбрано { $count } фильма.
        }
        [episode] { $count ->
            [one] Выбрана { $count } серия.
            [few] Выбрано { $count } серии.
            [many] Выбрано { $count } серий.
           *[other] Выбрано { $count } серии.
        }
       *[file] { $count ->
            [one] Выбран { $count } файл.
            [few] Выбрано { $count } файла.
            [many] Выбрано { $count } файлов.
           *[other] Выбрано { $count } файла.
        }
    }
series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] Удалить { $count } фильм в корзину
            [few] Удалить { $count } фильма в корзину
            [many] Удалить { $count } фильмов в корзину
           *[other] Удалить { $count } фильма в корзину
        }
        [episode] { $count ->
            [one] Удалить { $count } серию в корзину
            [few] Удалить { $count } серии в корзину
            [many] Удалить { $count } серий в корзину
           *[other] Удалить { $count } серии в корзину
        }
       *[file] { $count ->
            [one] Удалить { $count } файл в корзину
            [few] Удалить { $count } файла в корзину
            [many] Удалить { $count } файлов в корзину
           *[other] Удалить { $count } файла в корзину
        }
    }
series-delete-click = Нажмите на файлы, которые нужно удалить.
series-delete-where = Они попадут в корзину, откуда их можно вернуть. Отметки о просмотре сохранятся.
series-delete-move = В корзину
series-pick-none = Снять выбор
series-pick-all = Выбрать все
series-rescan-started = Пересканирование запущено
series-rated = Оценка { $score }
series-rated-cleared = Оценка удалена
series-score-failed = Оценка не сохранилась
series-still-downloading = { $episode } ещё скачивается
series-format-series = Сериал
series-tracker-refused = { $tracker } не принял изменение
series-tracker-refused-why = { $tracker } не принял изменение: { $why }
series-watched-up-to = Просмотрено до { $episode }
series-unwatched-from = Не просмотрено начиная с { $episode }
series-undo = Отменить
series-play-downloading = Скачивается серия { $number }
series-play-downloading-unnumbered = Скачивается серия
series-play-continue = Продолжить
series-play-again = Смотреть снова
series-play-continue-episode = Продолжить серию { $number }
series-play-again-from = Смотреть снова с серии { $number }
series-play-episode = Смотреть серию { $number }
series-page-not-tracked = Не отслеживается
series-not-tracked-on-disk =
    { $count ->
        [one] Не отслеживается, на диске { $count } серия
        [few] Не отслеживается, на диске { $count } серии
        [many] Не отслеживается, на диске { $count } серий
       *[other] Не отслеживается, на диске { $count } серии
    }
series-watched = Просмотрено { $count }
series-watched-of = Просмотрено { $watched } из { $total }
series-watched-of-estimate = Просмотрено { $watched } из { $total }+
series-facts-episodes-estimate =
    { $count ->
        [one] { $count }+ серия
        [few] { $count }+ серии
        [many] { $count }+ серий
       *[other] { $count }+ серии
    }
series-facts-each = по { $runtime }
series-facts-on-disk = { $count } на диске
series-next = Далее: { $episode }
series-episodes-downloading =
    { $count ->
        [one] Скачивается { $count } серия
        [few] Скачиваются { $count } серии
        [many] Скачивается { $count } серий
       *[other] Скачивается { $count } серии
    }
series-all-watched = Всё просмотрено
series-no-poster = Нет постера
# $span is "Episode 7" or "Episodes 7 to 9"
series-next-season-span = С{ $season } { $span }
series-next-predicted = { $span } ожидается { $date }, через { $wait }
series-next-date-format = ddd, d MMM
series-next-airs =
    { $count ->
        [one] { $span } выйдет через { $wait }
       *[other] { $span } выйдут через { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] Скачивается { $count } серия
        [few] Скачиваются { $count } серии
        [many] Скачивается { $count } серий
       *[other] Скачивается { $count } серии
    }
series-count-season-ends = Сезон { $season } заканчивается на серии { $number }
series-count-ends = Заканчивается на серии { $number }
series-count-gave-way = Вы задали { $count } серий; это больше не действует
# $line is what the user set, the parts joined with " · "
series-count-set-by-you = { $line }, задано вами
series-change = Изменить
series-get-episodes = Скачать серии
series-your-score = Ваша оценка { $score }
series-rate = Оценить
series-page-average-score = Средняя оценка { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = Голосов: { $votes }
series-add-to-list = В список
series-page-rewatched =
    { $count ->
        [one] Пересмотрено { $count } раз
        [few] Пересмотрено { $count } раза
        [many] Пересмотрено { $count } раз
       *[other] Пересмотрено { $count } раза
    }
series-your-notes = Ваши заметки
series-add-notes = Заметки
# $sites is AniList, MyAnimeList, or both
series-notes-tip = Заметки к записи в вашем списке, хранятся на { $sites }
series-unhide = Показать
series-hide = Скрыть
series-hidden-toast = «{ $title }» скрыт здесь, и AniBeam перестаёт синхронизировать его с AniList и MyAnimeList
series-unhidden-toast = «{ $title }» снова виден и синхронизируется
series-hide-tip-tracked = Скрыть со всех страниц здесь; AniBeam перестанет синхронизировать его с AniList и MyAnimeList
series-hide-tip = Скрыть из всех списков
series-fixes = Правки
series-fixes-tip = Исправить файлы, число серий, привязку или пересканировать
series-quick-fix = Быстрая правка
series-film = Фильм
series-episodes = Серии
series-extras = Опенинги, эндинги и другое
series-extras-openings = Опенинги
series-extras-endings = Эндинги
series-extras-previews = Превью и трейлеры
series-extras-specials = Спешлы
series-extras-other = Другое
# $kind is one of the series-extras- names above
series-extras-group = { $kind }, { $count }
series-extra-files = Лишние файлы
series-extra-files-warning =
    { $count ->
        [one] Один файл сверх { $total } ожидаемых серий: неверное имя, дубликат или спешл. Переименуйте или удалите его.
       *[other] Файлов сверх { $total } ожидаемых серий: { $count }. Это неверные имена, дубликаты или спешлы. Переименуйте или удалите их.
    }
series-synopsis = Описание
series-less = Меньше
series-more = Больше
series-match = Привязать
series-not-matched = Ещё не привязан, поэтому нет постера, описания и отслеживания. Найдите данные о нём на AniList или TVmaze.
series-naming-title = Имена, которые распознаются
series-naming-body = Папка называется как сериал, так, как его пишет AniList, а после названия идёт сезон, если он не первый. В каждом файле есть номер серии.
series-fix-how-files-read = Исправить чтение файлов
series-characters = Персонажи
series-related = Связанное
series-related-filling = Данные ещё подгружаются с AniList.
series-tags = Теги
series-hide-spoilers = Скрыть спойлеры
series-spoilers = Спойлеры
series-all-tags = Все теги
series-tag = { $name }, { $rank }%
series-recommendations = Рекомендации
series-relation-owned = В вашей библиотеке
series-relation-unreleased = Ещё не вышло
series-relation-on-anilist = На AniList
series-score-saved = Оценка сохранена
series-score-cleared = Оценка удалена

## GetShow.qml

get-show-the-client = торрент-клиент
get-show-group-batch = пакет { $group }
get-show-run = с { $first } по { $last }
# $list is the episode numbers and runs (get-show-run), joined with commas
get-show-episodes =
    { $count ->
        [one] Серия { $list }
       *[other] Серии { $list }
    }
# The same, in the middle of a sentence
get-show-episodes-inline =
    { $count ->
        [one] серии { $list }
       *[other] серий { $list }
    }
get-show-sending-to = Отправка в { $client }
get-show-go =
    { $count ->
        [one] Скачать { $count } серию
        [few] Скачать { $count } серии
        [many] Скачать { $count } серий
       *[other] Скачать { $count } серии
    }
get-show-go-replace =
    { $count ->
        [one] Скачать { $count } серию с заменой
        [few] Скачать { $count } серии с заменой
        [many] Скачать { $count } серий с заменой
       *[other] Скачать { $count } серии с заменой
    }
get-show-go-subscribe =
    { $count ->
        [one] Скачать { $count } серию и подписаться
        [few] Скачать { $count } серии и подписаться
        [many] Скачать { $count } серий и подписаться
       *[other] Скачать { $count } серии и подписаться
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] Скачать { $count } серию с заменой и подписаться
        [few] Скачать { $count } серии с заменой и подписаться
        [many] Скачать { $count } серий с заменой и подписаться
       *[other] Скачать { $count } серии с заменой и подписаться
    }
get-show-subscribe = Подписаться на новые серии
get-show-nothing-new = Нечего скачивать
get-show-pick-episode = Выберите серию
# $episodes is get-show-episodes-inline
get-show-replace-off = Выкл. Выбранные { $episodes } пропущены, потому что они уже есть в библиотеке. Включите, чтобы заменить их.
get-show-replace-on = Ваши файлы { $episodes } уйдут в корзину, когда новые скачаются.
# $episodes is get-show-episodes
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } сейчас скачивается: эта загрузка отменится, а скачанное уйдёт в корзину.
       *[other] { $episodes } сейчас скачиваются: эта загрузка отменится, а скачанное уйдёт в корзину.
    }
get-show-kind-on-demand = Запускается, когда нужен поиск
get-show-kind-rss-search = RSS-поиск
get-show-kind-rss-feed = RSS-лента
get-show-kind-json-search = JSON-поиск
get-show-kind-indexer-search = Поиск индексатора
get-show-ready = Готов
get-show-running = Работает
get-show-not-running = Не запущен
get-show-not-running-start = Не запущен. Нажмите на его источник выше, чтобы запустить.
get-show-switched-off = Выключен
get-show-click-to-see = Нажмите, чтобы увидеть, что в нём есть для этого сериала
get-show-searching = Поиск
get-show-searching-source = Поиск: { $name }
get-show-searching-elapsed = { $label }, { $seconds } с
get-show-releases =
    { $count ->
        [one] { $count } релиз
        [few] { $count } релиза
        [many] { $count } релизов
       *[other] { $count } релиза
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } релиз, покрыто { $cover } из { $total } серий
        [few] { $count } релиза, покрыто { $cover } из { $total } серий
        [many] { $count } релизов, покрыто { $cover } из { $total } серий
       *[other] { $count } релиза, покрыто { $cover } из { $total } серий
    }
get-show-on-your-list = В вашем списке
get-show-plan-to-watch = В планы
get-show-on-your-list-tip = Он есть в вашем списке
get-show-plan-to-watch-tip = Добавить в «В планах»
get-show-open-series-tip = Страница сериала: подробности, серии и просмотр
get-show-next-in = Серия { $number } через { $wait }
get-show-airing = В эфире
get-show-finished = Вышел
get-show-total-episodes = Серий: { $count }
get-show-downloading-episodes =
    { $count ->
        [one] Скачивается { $count } серия
        [few] Скачиваются { $count } серии
        [many] Скачивается { $count } серий
       *[other] Скачивается { $count } серии
    }
get-show-downloading-torrents =
    { $count ->
        [one] Скачивается { $count } торрент
        [few] Скачиваются { $count } торрента
        [many] Скачивается { $count } торрентов
       *[other] Скачивается { $count } торрента
    }
get-show-in-library = В вашей библиотеке
get-show-not-in-library = Нет в библиотеке
get-show-downloading = Скачивается
get-show-stop-all = Остановить все
get-show-start-all = Запустить все
get-show-stop-all-tip = Остановить все загрузки этого сериала; скачанное останется
get-show-start-all-tip = Снова запустить все загрузки этого сериала
get-show-cancel-all = Отменить все
get-show-cancel-all-tip = Отменить все загрузки этого сериала и переместить скачанное в корзину
get-show-sources = Доступные источники
get-show-check-again = Проверить снова
get-show-check-again-tip = Спрашивает каждый источник, запускаемый по запросу, работает ли он. Ничего не запускает.
get-show-search-source = Искать в { $name }
get-show-release-list = Все релизы списком
get-show-your-library = Ваша библиотека
get-show-library-on-disk = На диске { $have } из { $total }
get-show-library-downloading = Скачивается { $count }
get-show-library-nothing = Пока ничего
get-show-row-batch = { $group }, пакет
get-show-batch-counting = считаем серии, один торрент
get-show-batch-unlisted = серии не перечислены, весь сериал по названию, один торрент
get-show-batch-episodes =
    { $count ->
        [one] { $count } серия, один торрент
        [few] { $count } серии, один торрент
        [many] { $count } серий, один торрент
       *[other] { $count } серии, один торрент
    }
get-show-nothing-new-picked = Ничего нового не выбрано
get-show-nothing-picked = Ничего не выбрано
get-show-pick-provider = Выберите источник
# $groups is the release groups, joined with get-show-and
get-show-plan-line = { $episodes } от { $groups } на { $source }.
get-show-and = { $first } и { $second }
get-show-library-has =
    { $count ->
        [one] В библиотеке уже есть { $episodes }. Включите ниже «Заменить текущие серии», чтобы заменить её.
       *[other] В библиотеке уже есть { $episodes }. Включите ниже «Заменить текущие серии», чтобы заменить их.
    }
get-show-together = вместе
get-show-torrents-unit =
    { $count ->
        [one] торрент
        [few] торрента
        [many] торрентов
       *[other] торрента
    }
get-show-replace = Заменить текущие серии
get-show-sequential = Скачивать по порядку
get-show-sequential-on = Вкл.: каждый новый торрент качает части с начала, поэтому первые серии готовы раньше и их можно смотреть, пока качается остальное. В целом чуть медленнее.
get-show-sequential-off = Выкл.: клиент качает части в том порядке, в каком их отдаёт рой.
get-show-rolling = Держать несколько следующих наготове
get-show-rolling-on = Вкл.: сейчас скачивается { $count } сер., и каждая досмотренная запускает следующую.
get-show-rolling-on-delete = Вкл.: сейчас скачивается { $count } сер., и каждая досмотренная запускает следующую. Просмотренные серии удаляются с диска.
get-show-rolling-off = Выкл.: все выбранные серии скачиваются сразу.
get-show-follow-off = Выкл.: новые серии появятся здесь как пропуски, сами ничего не скачается.
get-show-follow-no-feed = У { $source } нет ленты для подписки, поэтому новые серии оттуда сами не скачаются.
get-show-follow-pick-group = Сначала выберите группу: подписка следит за одной группой и качеством.
get-show-follow-batch-only = У { $group } здесь только пакет, поэтому нет ленты отдельных серий для подписки.
# $feed is the release group and its quality
get-show-follow-on = Вкл.: лента { $feed } на { $source }. Каждая новая серия скачивается по мере выхода.
get-show-confirm =
    { $count ->
        [one] { $episodes } сейчас скачивается. Замена отменит эту загрузку и переместит скачанное в корзину.
       *[other] { $episodes } сейчас скачиваются. Замена отменит эту загрузку и переместит скачанное в корзину.
    }
get-show-confirm-go = Отменить и заменить
get-show-state-error = Ошибка
get-show-state-files-missing = Файлы пропали
get-show-state-stopped = Остановлено
get-show-state-waiting-peers = Ждём пиров
get-show-state-metadata = Получение метаданных
get-show-state-queued = В очереди
get-show-state-checking = Проверка файлов
get-show-state-allocating = Выделение места
get-show-state-moving = Перемещение файлов
get-show-state-waiting = Ожидание
get-show-a-batch = Пакет
get-show-no-group = Группы нет в названии
get-show-likely = вероятно, { $source }
get-show-tracker = трекер { $tracker }
get-show-not-known = Пока неизвестно
get-show-bytes-of = { $done } из { $size }
get-show-speed-down = ↓ { $speed }/с
get-show-time-left = Осталось { $wait }
get-show-stopping = Остановка
get-show-starting = Запуск
get-show-stop = Стоп
get-show-start-tip = Снова начать загрузку
get-show-stop-tip = Остановить загрузку; скачанное останется
get-show-cancelling = Отмена
get-show-cancel-tip = Отменить эту загрузку и переместить скачанное в корзину
get-show-episode-order = Скачивать по порядку серий
get-show-episode-order-on = Вкл.: части качаются с начала каждого файла, поэтому первые серии готовы раньше.
get-show-cube-skipped = Серия { $number }: пропущена
get-show-cube-done = Серия { $number }: готово
get-show-cube-percent = Серия { $number }: { $percent }%
get-show-fact-torrent = Торрент
get-show-fact-release = Релиз
get-show-fact-from = Откуда
get-show-fact-saving-to = Сохраняется в
get-show-span-counting = Подсчёт
get-show-span-unknown = { $runs }?
get-show-span-episodes = Серии { $runs }
get-show-cube-no-release = Серия { $number }: нет релиза от { $group }
get-show-seeders = Сидов: { $count }
get-show-cube-on-disk = уже на диске, нажмите, чтобы заменить
get-show-cube-downloading = скачивается, нажмите, чтобы заменить

## TorrentRow

torrent-row-state-downloading = Скачивается
torrent-row-state-stalled = Застряло
torrent-row-state-metadata = Получение метаданных
torrent-row-state-queued = В очереди
torrent-row-state-stopped = Остановлено
torrent-row-state-done = Готово
torrent-row-state-seeding = Раздаётся
torrent-row-state-checking = Проверка
torrent-row-state-moving = Перемещение
torrent-row-state-error = Ошибка
torrent-row-state-files-missing = Файлы пропали
torrent-row-state-making-room = Выделение места
torrent-row-state-finding-peers = Поиск пиров
torrent-row-state-waiting = Ожидание
torrent-row-done-of-size = { $done } из { $size }
torrent-row-speed-down = ↓ { $speed }/с
torrent-row-speed-up = ↑ { $speed }/с
torrent-row-time-left = Осталось { $time }
torrent-row-seeders =
    { $count ->
        [one] { $count } сид
        [few] { $count } сида
        [many] { $count } сидов
       *[other] { $count } сида
    }
torrent-row-rolling = скользящая, серии с { $next } по { $through } из { $total }
torrent-row-stop = Стоп

## SourceRow

source-row-unavailable-fix = Подключите диск и нажмите «Пересканировать». Если папка переехала, удалите её и добавьте новое место.
source-row-counts = Сериалов: { $count }
source-row-counts-movies =
    { $movies ->
        [one] Сериалов: { $count }, { $movies } папка с фильмами
        [few] Сериалов: { $count }, { $movies } папки с фильмами
        [many] Сериалов: { $count }, { $movies } папок с фильмами
       *[other] Сериалов: { $count }, { $movies } папки с фильмами
    }
source-row-unavailable = Недоступна
    .tip = AniBeam не может открыть эту папку. Если она на диске, который не подключён или не смонтирован, подключите его, и AniBeam снова её подхватит. Если папка переехала, удалите её здесь и добавьте новое место.
source-row-rescan = Пересканировать
source-row-remove-question = Удалить { $name }, сериалы ({ $count }) и их историю?

## StartSourceSheet

start-source-title = Запустить { $service } для { $name }?
start-source-busy = Запуск { $service }. { $name } включится, когда он ответит; это может занять до минуты.
start-source-body = { $service } это служба, которая ищет в { $name } для AniBeam. Запуск включает её в фоне на этом компьютере, и она продолжает работать после закрытия AniBeam.
start-source-starting = Запуск

## SearchProgress

search-progress-why-helper = AniBeam запускает помощника поиска; это займёт секунду-другую.
search-progress-why-starting = { $endpoint } работает через помощника, который запускается, когда нужен поиск. Первый поиск ждёт его до { $seconds } с; следующие этого не ждут.
search-progress-why-reading = Разбиваем релизы по группам и сверяем с библиотекой.
search-progress-why-ondemand = Помощник заново читает { $endpoint } для каждого нового поиска, обычно это занимает десять-двадцать секунд.
search-progress-why-slow = { $endpoint } сейчас медленно отвечает.
search-progress-why-usual = Обычно это занимает секунду-другую.
search-progress-why-and-slow = { $why } Это дольше обычного; отмените, чтобы попробовать другой источник.
search-progress-searching = Поиск
search-progress-seconds = { $seconds } с
search-progress-try = Попробовать { $endpoint }

## ReleaseTable

release-table-col-release = Релиз
release-table-col-group = Группа
release-table-col-res = Разр.
release-table-col-rip = Рип
release-table-col-ep = Сер.
release-table-col-size = Размер
release-table-col-seeders = Сиды
release-table-col-leechers = Личи
release-table-col-downloads = Загрузки
release-table-col-released = Вышел
release-table-tick-all = Отметить все релизы
release-table-untick-all = Снять отметку со всех релизов
release-table-on-disk = На диске
release-table-other-on-disk = На диске другой релиз
release-table-season-batch = Пакет сезона { $season }
release-table-batch = Пакет
release-table-via = через { $source }
release-table-season-episode = С{ $season } { $episode }
release-table-date-format = d MMMM yyyy, HH:mm
release-table-download = Скачать
release-table-more = и ещё { $count }. Сузьте список или отсортируйте его, чтобы увидеть другие.
release-table-pick = Выбрать этот релиз

## DownloadsPage

downloads-title = Загрузки
downloads-builtin-client = Встроенный клиент
downloads-start-done = { $name } запущен и включён
downloads-disk-film = Фильм
downloads-disk-watched = { $size } просмотрено
downloads-disk-all-watched = всё просмотрено
downloads-delete-watched-menu =
    { $count ->
        [one] Все просмотренные серии: { $count } серия, { $size }
        [few] Все просмотренные серии: { $count } серии, { $size }
        [many] Все просмотренные серии: { $count } серий, { $size }
       *[other] Все просмотренные серии: { $count } серии, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Все вышедшие и просмотренные сериалы: { $count } сериал, { $size }
        [few] Все вышедшие и просмотренные сериалы: { $count } сериала, { $size }
        [many] Все вышедшие и просмотренные сериалы: { $count } сериалов, { $size }
       *[other] Все вышедшие и просмотренные сериалы: { $count } сериала, { $size }
    }
downloads-titles-and = { $list } и { $last }
downloads-titles-more = { $list } и ещё { $count }
downloads-delete-finished-none = Нет вышедших сериалов, просмотренных до конца
downloads-delete-watched-none = На диске нет ничего просмотренного
downloads-delete-finished-heading =
    { $count ->
        [one] Удалить { $count } вышедший сериал, { $size }?
        [few] Удалить { $count } вышедших сериала, { $size }?
        [many] Удалить { $count } вышедших сериалов, { $size }?
       *[other] Удалить { $count } вышедшего сериала, { $size }?
    }
downloads-delete-finished-base = { $titles }. Только сериалы, которые по данным источника вышли полностью и просмотрены целиком; сериал, который ещё выходит или чей конец неизвестен, остаётся. Все их файлы, включая доп. материалы, уйдут в корзину. Их торренты уйдут и из { $client }, так что их можно будет скачать снова.
downloads-delete-watched-heading =
    { $count ->
        [one] Удалить { $count } просмотренную серию, { $size }?
        [few] Удалить { $count } просмотренные серии, { $size }?
        [many] Удалить { $count } просмотренных серий, { $size }?
       *[other] Удалить { $count } просмотренной серии, { $size }?
    }
downloads-delete-watched-base = Все просмотренные серии во всех сериалах. Непросмотренные серии и доп. материалы остаются. Файлы и их субтитры уйдут в корзину, оттуда их можно вернуть. Их торренты уйдут и из { $client }, так что их можно будет скачать снова.
downloads-move-to-trash = В корзину
downloads-remove-group-torrents =
    { $count ->
        [one] Удалить торрент
        [few] Удалить его { $count } торрента
        [many] Удалить его { $count } торрентов
       *[other] Удалить его { $count } торрента
    }
downloads-run-span = с { $from } по { $to }
downloads-run-pair = { $first } и { $second }
downloads-gap-missing =
    { $count ->
        [one] Нет серии { $runs }
       *[other] Нет серий { $runs }
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Серия { $runs } не скачана
       *[other] Серии { $runs } не скачаны
    }
downloads-gap-then-not-downloaded = { $runs } не скачано
downloads-trying-query = Пробуем { $query }
downloads-client-settings = Настройки торрент-клиента
downloads-error-unknown = Что-то пошло не так. Попробуйте снова.
downloads-search-all-off = Все источники выключены. Включите один, чтобы искать.
downloads-turn-on = Включить { $name }
downloads-search-no-sources = Пока нет источников для поиска. Добавьте список источников в настройках, раздел «Загрузки».
downloads-search-none-started = Ни один источник не смог начать поиск. Попробуйте ещё раз через минуту.
downloads-auto-resolution = Сужено до { $resolution }, лучшего из доступного
downloads-search-one-failed = Поиск в { $name } не сработал. Попробуйте снова или ищите в другом источнике.
downloads-search-all-failed = Ни один источник не смог выполнить поиск. Попробуйте ещё раз через минуту.
downloads-narrowing-on = Уточнение по { $name }
downloads-refine-zero = Ничего бы не осталось, поэтому список не изменён.
downloads-refine-no-match = Ни один релиз не подходит под это слово.
downloads-refine-empty = Введите слово для уточнения.
downloads-refine-no-change = Ничего бы не изменилось.
downloads-movie-tick-one = Отметьте один релиз и нажмите «Скачать как фильм».
downloads-group-downloading = Скачиваются
downloads-group-downloading-line = Идут сейчас; подробности в «Активности» справа.
downloads-group-seeding = Раздаются
downloads-group-seeding-line = Скачаны и ещё раздаются.
downloads-group-done = Готово
downloads-group-done-line = Скачаны; клиент их не раздаёт.
downloads-group-stopped = Остановлены
downloads-group-stopped-line = Остановлены до завершения.
downloads-group-broken = Файлы пропали
downloads-group-broken-line = Клиент не находит их файлы: они указывают на несуществующую папку. Удаление торрентов наведёт в клиенте порядок.
downloads-sub-nofeed = Нельзя подписаться
downloads-sub-nofeed-line = Их источник отвечает на поиск, но у него нет ленты, поэтому новые серии не приходят. Отпишитесь и подпишитесь на сериал в источнике с лентой.
downloads-sub-complete = Завершено
downloads-sub-complete-line = Все серии в библиотеке, и больше ничего не выйдет: этим подпискам делать нечего.
downloads-sub-airing = В эфире
downloads-sub-airing-line = Новые серии ещё выйдут, ближайшие первыми.
downloads-sub-incomplete = Неполные
downloads-sub-incomplete-line = Сериал вышел, но каких-то серий здесь нет, или неизвестно, в каком он состоянии.
downloads-sub-missing = Нет в библиотеке
downloads-sub-missing-line = В их папке нет ничего, что нашла библиотека: либо ещё ничего не пришло, либо сохраняется туда, куда библиотека не смотрит.
downloads-torrents =
    { $count ->
        [one] { $count } торрент
        [few] { $count } торрента
        [many] { $count } торрентов
       *[other] { $count } торрента
    }
downloads-rule-film-on-disk = Фильм, на диске
downloads-rule-all-here = Все { $total } здесь
downloads-rule-next =
    { $predicted ->
        [yes] Сер. { $episode } ожидается через { $time }
       *[no] Сер. { $episode } через { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] Сер. { $episode }-{ $through } ожидаются через { $time }
       *[no] Сер. { $episode }-{ $through } через { $time }
    }
downloads-rule-have-of = Здесь { $have } из { $total }
downloads-rule-have =
    { $count ->
        [one] Здесь { $count } серия
        [few] Здесь { $count } серии
        [many] Здесь { $count } серий
       *[other] Здесь { $count } серии
    }
downloads-tag-not-downloaded = Не скачано
downloads-tag-missing = Нет { $count }
downloads-tag-n-not-downloaded = Не скачано { $count }
downloads-tag-unknown = Неизвестно
downloads-speed-down = ↓ { $speed }/с
downloads-time-left = Осталось { $time }
downloads-repair-trying = Пробуем запросы короче
downloads-sent-to-client = «{ $title }» отправлен в { $client }
downloads-files-torrents = то, что скачали торренты
downloads-files-torrents-remote = то, что скачали торренты, удалит { $client }
downloads-files-folder-and = { $folder } и { $torrents }
downloads-unsubscribe-heading = Отписаться от «{ $title }»?
downloads-unsubscribe-base = Его лента и правило загрузки уйдут из { $client }, так что ничего нового не придёт.
downloads-unsubscribe-complete-heading = Отписаться от всех завершённых сериалов ({ $count })?
downloads-unsubscribe-complete-base = Их ленты и правила загрузки уйдут из { $client }. Все их серии уже в библиотеке.
downloads-files-folders-and-torrents = их папки и то, что скачали их торренты
downloads-one-at-a-time = Они удаляются по одной; страница показывает каждую по ходу.
downloads-subscription-stays = { $base } Подписка остаётся.
downloads-cancel-heading = Отменить «{ $title }»?
downloads-cancel-group-base =
    { $count ->
        [one] Его загрузка остановится и уйдёт из { $client }.
       *[other] Его загрузки ({ $count }) остановятся и уйдут из { $client }.
    }
downloads-files-so-far-it = то, что он уже скачал
downloads-files-so-far-they = то, что они уже скачали
downloads-cancel-download = Отменить загрузку
downloads-keep-downloading = Продолжить загрузку
downloads-remove-heading = Удалить «{ $title }»?
downloads-remove-group-base =
    { $count ->
        [one] Его торрент уйдёт из { $client }; файлы останутся, если не отметить ниже.
       *[other] Его торренты ({ $count }) уйдут из { $client }; файлы останутся, если не отметить ниже.
    }
downloads-remove-detail-base =
    { $count ->
        [one] Его { $count } торрент уйдёт из { $client }; файлы останутся, если не отметить ниже.
       *[other] Его торренты ({ $count }) уйдут из { $client }; файлы останутся, если не отметить ниже.
    }
downloads-remove-broken-heading = Удалить торренты всех сериалов с пропавшими файлами ({ $count })?
downloads-remove-broken-base = Их торренты ({ $count }) уйдут из { $client }. Они указывают на несуществующие папки, так что файлы не затрагиваются; подписки остаются.
downloads-remove-torrent-heading = Удалить этот торрент?
downloads-files-it-downloaded = то, что он скачал
downloads-cancel-torrent-heading = Отменить эту загрузку?
downloads-cancel-many-heading =
    { $count ->
        [one] Отменить { $count } загрузку?
        [few] Отменить { $count } загрузки?
        [many] Отменить { $count } загрузок?
       *[other] Отменить { $count } загрузки?
    }
downloads-cancel-many-base = Все загрузки этого сериала остановятся и уйдут из { $client }.
downloads-cancel-all = Отменить все
downloads-summary-connecting = Подключение.
downloads-summary-client-on = { $client } { $version } подключён.
downloads-summary-client-off = { $client } не подключён. Проверьте его адрес и пароль в настройках, раздел «Загрузки».
downloads-summary-native = AniBeam сам ищет в ваших источниках.
downloads-summary-native-with = AniBeam сам ищет в ваших источниках. { $client }
downloads-summary-starting = Запуск загрузок.
downloads-summary-not-running = Загрузки не работают.
downloads-summary-not-started = Загрузки ещё не запущены.
downloads-summary-helper-ready = Помощник поиска { $version } готов. { $client }
downloads-subscribed-as = Подписка «{ $name }» оформлена.
downloads-repaired =
    { $count ->
        [one] «{ $name }» исправлен: лента находит { $count } релиз
        [few] «{ $name }» исправлен: лента находит { $count } релиза
        [many] «{ $name }» исправлен: лента находит { $count } релизов
       *[other] «{ $name }» исправлен: лента находит { $count } релиза
    }
downloads-repaired-keeps =
    { $count ->
        [one] «{ $name }» исправлен: лента находит { $count } релиз, а правило оставляет «{ $term }»
        [few] «{ $name }» исправлен: лента находит { $count } релиза, а правило оставляет «{ $term }»
        [many] «{ $name }» исправлен: лента находит { $count } релизов, а правило оставляет «{ $term }»
       *[other] «{ $name }» исправлен: лента находит { $count } релиза, а правило оставляет «{ $term }»
    }
downloads-sent-torrents =
    { $count ->
        [one] { $count } торрент отправлен в { $client }.
        [few] { $count } торрента отправлено в { $client }.
        [many] { $count } торрентов отправлено в { $client }.
       *[other] { $count } торрента отправлено в { $client }.
    }
downloads-added-torrents =
    { $count ->
        [one] Добавлен { $count } торрент.
        [few] Добавлено { $count } торрента.
        [many] Добавлено { $count } торрентов.
       *[other] Добавлено { $count } торрента.
    }
downloads-trashed-paths =
    { $count ->
        [one] «{ $title }»: { $count } путь перемещён в корзину
        [few] «{ $title }»: { $count } пути перемещено в корзину
        [many] «{ $title }»: { $count } путей перемещено в корзину
       *[other] «{ $title }»: { $count } пути перемещено в корзину
    }
downloads-show-gone = Этого сериала больше нет в библиотеке
downloads-activity-hide = Скрыть активность
downloads-activity = Активность
downloads-moving =
    { $count ->
        [one] { $count } загрузка идёт
        [few] { $count } загрузки идут
        [many] { $count } загрузок идут
       *[other] { $count } загрузки идут
    }
downloads-back = Назад к загрузкам
downloads-chip-search-ready = Поиск готов
downloads-chip-search-starting = Поиск запускается
downloads-chip-search-off = Поиск выкл.
downloads-chip-answers-at = { $summary } Отвечает по адресу { $url }.
downloads-chip-helper-missing = Помощник поиска не установлен. Нажмите, чтобы открыть настройки загрузок, там написано, как его добавить
downloads-chip-helper-failed = Помощник поиска остановился. «Переподключить» запустит его снова; что пошло не так, написано в настройках загрузок
downloads-chip-open-settings = Открыть настройки загрузок
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } выкл.
downloads-chip-client-not-connected = { $client } не подключён. Переподключите или проверьте его адрес в настройках
downloads-chip-client-remote = Ваш собственный помощник поиска использует клиент из своих настроек
downloads-chip-client-switch = Нажмите, чтобы сменить клиент
downloads-connecting = Подключение
downloads-reconnect = Переподключить
downloads-search-placeholder = Найдите сериал или название релиза
downloads-search-go-tip = Искать названия релизов в { $names }
downloads-search-go-off = Все источники выключены.
downloads-suggest-airing =
    { $predicted ->
        [yes] В эфире, серия { $episode } ожидается через { $time }
       *[no] В эфире, серия { $episode } через { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] В эфире, серии с { $episode } по { $through } ожидаются через { $time }
       *[no] В эфире, серии с { $episode } по { $through } через { $time }
    }
downloads-suggest-finished = Вышел
downloads-suggest-episodes = Серий: { $count }
downloads-suggest-in-library = В вашей библиотеке
downloads-suggest-on-anilist = На AniList, нет в библиотеке
downloads-plan-to-watch = В планы
downloads-suggest-release-search = Искать «{ $query }» в названиях релизов в { $names }
downloads-suggest-release-search-none = Искать «{ $query }» в названиях релизов: нет источников
downloads-key-return = Enter
downloads-sources = Источники
downloads-source-hint-starting = запуск
downloads-source-hint-down = не запущен
downloads-source-tip-down = { $name } нужна служба-помощник, а она не запущена. Нажмите, чтобы запустить.
downloads-source-tip-on = Поиск идёт в { $name }. Нажмите, чтобы выключить.
downloads-source-tip-off = Выключен. Нажмите, чтобы снова искать в { $name }.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Проверить
    .tip = Спрашивает каждый источник, запускаемый по запросу, работает ли он. Ничего не запускает.
downloads-check-busy = Проверка
downloads-back-to-grid = Назад к сетке
    .tip = Назад к «{ $title }» и тому, что нашёл каждый источник. Escape делает то же.
downloads-releases-on =
    { $count ->
        [one] { $count } релиз на { $name }
        [few] { $count } релиза на { $name }
        [many] { $count } релизов на { $name }
       *[other] { $count } релиза на { $name }
    }
downloads-found-on = Найдено на
downloads-found-searching = поиск
downloads-found-failed = ошибка
downloads-found-cancelled = отменено
downloads-releases =
    { $count ->
        [one] { $count } релиз
        [few] { $count } релиза
        [many] { $count } релизов
       *[other] { $count } релиза
    }
downloads-found-retry-tip =
    { $why }
    Нажмите, чтобы повторить
downloads-facet-groups = Релиз-группа
downloads-facet-quality = Качество
downloads-facet-words = Слова
downloads-filters-fewer = Меньше фильтров
downloads-filters-more = Больше фильтров ({ $count })
downloads-narrow-placeholder = Уточнить словом или -слово, чтобы исключить
downloads-best-match = Лучшее
    .tip = Оставляет релизы, похожие на лучший: { $query }
downloads-best-match-none = Здесь ни один релиз не выделяется
downloads-strip-have-of = На диске { $have } из { $total } серий
downloads-strip-have =
    { $count ->
        [one] На диске { $count } серия
        [few] На диске { $count } серии
        [many] На диске { $count } серий
       *[other] На диске { $count } серии
    }
downloads-strip-next =
    { $predicted ->
        [yes] серия { $episode } ожидается через { $time }
       *[no] серия { $episode } через { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] серии с { $episode } по { $through } ожидаются через { $time }
       *[no] серии с { $episode } по { $through } через { $time }
    }
downloads-strip-complete = полностью
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = Ни один из этих релизов не относится к сериям, которые у вас уже есть.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } релиз здесь это ваш файл, он помечен «На диске»
        [few] { $count } релиза здесь это ваши файлы, они помечены «На диске»
        [many] { $count } релизов здесь это ваши файлы, они помечены «На диске»
       *[other] { $count } релиза здесь это ваши файлы, они помечены «На диске»
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } релиз относится к серии, которая у вас есть из другого релиза
        [few] { $count } релиза относятся к сериям, которые у вас есть из другого релиза
        [many] { $count } релизов относятся к сериям, которые у вас есть из другого релиза
       *[other] { $count } релиза относятся к сериям, которые у вас есть из другого релиза
    }
downloads-strip-both = { $first }; { $second }.
downloads-tick-missing =
    { $count ->
        [one] Отметить { $count } недостающую серию
        [few] Отметить { $count } недостающие серии
        [many] Отметить { $count } недостающих серий
       *[other] Отметить { $count } недостающей серии
    }
    .tip = Для каждой серии, которой нет в библиотеке, релиз с наибольшим числом сидов
downloads-open-in-library = Открыть в библиотеке
downloads-not-this-show = Не этот сериал
downloads-no-releases = Нет релизов
downloads-no-releases-body = На { $name } ничего не подходит под «{ $query }». Другой источник или меньше слов может помочь.
downloads-try-source = Попробовать { $name }
downloads-ticked =
    { $count ->
        [one] Отмечен { $count } релиз
        [few] Отмечено { $count } релиза
        [many] Отмечено { $count } релизов
       *[other] Отмечено { $count } релиза
    }
downloads-nothing-ticked = Ничего не отмечено
downloads-ticked-size = Всего { $size }.
downloads-tick-hint = Отметьте релизы в таблице или скачайте все.
downloads-tick-all = Отметить все
downloads-working = Выполняется
downloads-download-ticked =
    { $count ->
        [one] Скачать { $count } релиз
        [few] Скачать { $count } релиза
        [many] Скачать { $count } релизов
       *[other] Скачать { $count } релиза
    }
downloads-download-shown = Скачать показанные ({ $count })
downloads-download-all = Скачать все { $count }
downloads-download-movie = Скачать как фильм
    .tip = Один отмеченный релиз, сразу в папку фильмов
downloads-subscribe = Подписаться
downloads-subscribe-body = Новые релизы, подходящие под этот поиск, скачиваются по мере появления в указанную здесь папку.
downloads-name-placeholder = Берётся из первого релиза
downloads-subscribe-search = Подписаться на этот поиск
downloads-subscribe-no-feed = Здесь нет ленты для подписки
downloads-preview-again = Снова показать правило
downloads-preview-show = Показать правило, которое получится
downloads-preview-from = С { $host }
downloads-preview-from-named = С { $host } ({ $name })
downloads-preview-must = Должно содержать { $terms }
downloads-preview-must-not = Не должно содержать { $terms }
downloads-helper-changed = Загрузки настроены на помощника, который не отвечает
downloads-helper-missing = Эта копия AniBeam собрана без загрузок
downloads-helper-failed = Загрузки не запустились
downloads-helper-changed-body = Поиск и загрузки работают через маленькую программу-помощника. В настройках указан не встроенный помощник AniBeam, а другой, и он не отвечает. Вернитесь к встроенному, и он запустится сам.
downloads-helper-missing-body = Она собрана без программы-помощника, через которую работают поиск и загрузки. В установщиках со страницы релизов она есть. В разделе «Настройки, Загрузки» написано, как добавить её в эту копию.
downloads-helper-failed-body = Программа-помощник для поиска и загрузок остановилась, не успев запуститься. Повторный запуск чаще всего помогает.
downloads-helper-use-own = Встроенный помощник AniBeam
downloads-helper-start-again = Запустить снова
downloads-helper-show-how = Показать как
downloads-helper-what-went-wrong = Что пошло не так
downloads-lens-gaps-count = Пропуски в библиотеке  { $count }
downloads-lens-airing-count = В эфире  { $count }
downloads-lens-following-count = Подписки  { $count }
downloads-lens-downloaded-count = Скачано  { $count }
downloads-lens-gaps = Пропуски в библиотеке
downloads-lens-gaps-line = Вышедшие серии не на диске. Откройте сериал и найдите их.
downloads-lens-gaps-none = Ничего не пропущено: все вышедшие серии всех сериалов на диске.
downloads-lens-airing-line = Сериалы из библиотеки, у которых ещё выйдут серии. Откройте, чтобы подписаться.
downloads-lens-airing-none = Ничего из библиотеки сейчас не выходит
downloads-lens-airing-none-body = Сериал появится здесь, когда он в библиотеке и у него ещё выйдут серии. «В эфире» показывает всё, что выходит.
downloads-airing-now = В эфире
downloads-lens-following = Подписки
downloads-card-next =
    { $predicted ->
        [yes] Серия { $episode } ожидается через { $time }
       *[no] Серия { $episode } через { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Серии с { $episode } по { $through } ожидаются через { $time }
       *[no] Серии с { $episode } по { $through } через { $time }
    }
downloads-rules-unread = Не удалось прочитать подписки.
downloads-lens-following-line = Ленты, за которыми следит AniBeam, по сериалам, которые они пополняют.
downloads-none-yet = Пока нет.
downloads-reading = Чтение.
downloads-rules-error = AniBeam не смог прочитать подписки. Попробуйте ещё раз через минуту.
downloads-removing-to-go = Удаление, осталось { $count }
downloads-unsubscribe-all = Отписаться от всех { $count }
downloads-unsubscribe = Отписаться
downloads-no-rules = Подписок нет
downloads-no-rules-body = Найдите сериал выше и подпишитесь, чтобы серии приходили сами.
downloads-search = Поиск
downloads-lens-downloaded = Скачано
downloads-dl-line = { $size } на диске. { $rest }
downloads-dl-client-off = { $client } не подключён.
downloads-dl-unread = Не удалось прочитать список загрузок.
downloads-dl-in-client =
    { $count ->
        [one] { $count } торрент, { $size } в { $client }.
        [few] { $count } торрента, { $size } в { $client }.
        [many] { $count } торрентов, { $size } в { $client }.
       *[other] { $count } торрента, { $size } в { $client }.
    }
downloads-usage-watched = Просмотрено
downloads-usage-not-watched = Не просмотрено
downloads-deleting = Удаление
downloads-delete-watched = Удалить просмотренное
downloads-remove-all = Удалить все { $count }
downloads-on-disk = На диске
downloads-on-disk-line = В библиотеке, без торрента в { $client }, сначала самые большие. Нажмите правой кнопкой, чтобы удалить.
downloads-nothing-downloaded = Ничего не скачано
downloads-nothing-downloaded-body = Здесь появится то, что вы скачиваете или на что подписаны, по сериалам.
downloads-client-off-title = { $client } не подключён
downloads-client-off-body = Проверьте, что он запущен, и подключитесь снова. Его адрес и пароль в настройках, раздел «Загрузки».
downloads-connect-again = Подключить снова
downloads-view-grid = Сетка
downloads-view-list = Список
downloads-chip-subscribed = Подписка
downloads-speed-up = ↑ { $speed }/с
downloads-broken-some = С пропавшими файлами: { $count }
downloads-tag-watched = Просмотрено

## PlayerDelays.qml

player-delays-title = Задержка
player-delays-subtitles = Субтитры
player-delays-audio = Аудио
player-delays-subtitles-earlier = Субтитры на 0,1 с раньше
player-delays-subtitles-later = Субтитры на 0,1 с позже
player-delays-audio-earlier = Аудио на 0,1 с раньше
player-delays-audio-later = Аудио на 0,1 с позже
player-delays-reset = Сбросить до 0 до следующего файла
player-delays-none = Без задержки

## TrackPicker.qml

track-picker-off = Выкл.

## RollControls.qml

roll-controls-window = Наготове { $ahead }: серии с { $next } по { $through } из { $total }
roll-controls-window-watched = Наготове { $ahead }: серии с { $next } по { $through } из { $total }, просмотрено { $watched }
roll-controls-more = Скачать ещё { $count }
roll-controls-more-tip = Скачать следующие серии сейчас. AniBeam снова будет держать несколько наготове, когда вы догоните.
roll-controls-fewer-tip = Держать на одну меньше
roll-controls-ahead = { $count } вперёд
roll-controls-further-tip = Держать на одну больше
roll-controls-all = Скачать все
roll-controls-all-tip = Больше не держать несколько наготове и скачать все оставшиеся серии
roll-controls-all-question = Скачать все оставшиеся серии ({ $count })?

## PlayerChrome.qml

player-chrome-delays = Задержка субтитров и аудио
player-chrome-audio = Аудиодорожка
player-chrome-subtitles = Субтитры
player-chrome-subtitles-off = Субтитры выкл.
player-chrome-casting-to = Трансляция на { $device }
player-chrome-cast = Транслировать на ТВ
player-chrome-mark-watched = Отметить просмотренной
player-chrome-shortcuts = Горячие клавиши
player-chrome-fullscreen = Во весь экран
player-chrome-exit-fullscreen = Выйти из полноэкранного режима
player-chrome-stay = Остаться
player-chrome-skip-intro = Пропустить опенинг
player-chrome-skip-outro = Пропустить эндинг
player-chrome-playing-on = Воспроизводится на { $device }
player-chrome-replay = Повтор

## PlayerPage.qml

player-title = Плеер
player-next = Далее
player-extra-no-next = У доп. материала нет следующей серии
player-last-episode = Это последняя серия
player-extra-no-previous = У доп. материала нет предыдущей серии
player-first-episode = Это первая серия
player-resumed-at = Продолжено с { $time }
player-start-over = Сначала
player-audio-track = Аудио: { $track }
player-subtitles-track = Субтитры: { $track }
player-subtitles-on = Субтитры: вкл.
player-subtitles-off = Субтитры выкл.
player-no-subtitles = В этом файле нет субтитров
# A delay in seconds, signed: "+0.1 s", "0.0 s"
player-delay-seconds = { $value } с
player-audio-delay = Задержка аудио { $value }
player-subtitle-delay = Задержка субтитров { $value }
player-volume = Громкость { $volume }
player-speed = Скорость { $speed }×
player-not-while-casting = Не во время трансляции
player-hold-fast = 2×, пока удерживаете
player-no-chapters = В этом файле нет глав
player-last-chapter = Это последняя глава
player-chapter = Глава { $number }
player-chapter-titled = Глава { $number }: { $title }
player-screenshot-saved = Сохранено в Pictures/AniBeam
player-skipped-intro = Опенинг пропущен
player-skipped-outro = Эндинг пропущен
player-undo = Отменить
# Why a tracker was not updated, read after "Not tracked:" or a tracker's name
player-refusal-hidden = сериал скрыт
player-refusal-no-match = сериал ещё не привязан; привяжите его в «О сериалах»
player-refusal-not-newer = ваш список уже дальше этой серии
player-refusal-extra = это доп. материал
player-refusal-unmatched = файл не привязан к серии
player-refusal-on-disk = файл есть только на диске
player-not-tracked-because = Не отмечено: { $reason }.
# A tracker and what it said: "AniList: the show is hidden"
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = Не удалось обновить { $tracker }
player-outcome-ok = { $tracker }: ок
player-outcome-at = { $tracker }: { $progress }
player-outcome-tracked = Отмечено
player-outcome-not-tracked = Не отмечено
player-outcome-could-not-update = Не удалось обновить ваш список
# $list is each tracker's outcome joined by "; "
player-outcome-tracked-list = Отмечено: { $list }
player-outcome-not-tracked-list = Не отмечено: { $list }
player-outcome-failed-list = Не удалось обновить список: { $list }
player-rated = Оценка { $score }
player-score-failed = Не удалось сохранить оценку
player-frame = Кадр { $number }, { $time }
player-audio-title = Аудио
player-subtitles-title = Субтитры
player-cast-to = Транслировать на
player-cast-looking = Поиск Chromecast и телевизоров
player-cast-unreachable = Не удалось подключиться к телевизору
player-cast-tv = ТВ
player-cast-stop = Остановить трансляцию на { $device }
player-cast-connecting = Подключение
player-cast-loading = Загрузка
player-cast-playing = Воспроизводится
player-cast-paused = Пауза
player-cast-finished = Завершено
player-cast-stopped = Остановлено
player-rate-prompt = Последняя серия отмечена. Оценить сериал?
player-rate = Оценить
player-not-now = Не сейчас

## EndpointReposPanel.qml

endpoint-repos-panel-title = Списки источников
endpoint-repos-panel-helper = Список источников: подписанный набор мест для поиска, по адресу. AniBeam проверяет подпись и показывает ключ до сохранения, затем вы ставите нужное. Оно попадёт в «Источники поиска» выше.
endpoint-repos-panel-could-not-start = AniBeam не смог это запустить. Попробуйте ещё раз через минуту.
endpoint-repos-panel-needs-value = Для { $name } нужно значение
endpoint-repos-panel-values-of = Значения { $name }
endpoint-repos-panel-secret-note = Хранится в защищённом хранилище этого компьютера, никогда в библиотеке или экспорте.
endpoint-repos-panel-secret-note-keep = Хранится в защищённом хранилище этого компьютера, никогда в библиотеке или экспорте. Пустое поле сохраняет уже записанное значение.
endpoint-repos-panel-asks-again = { $name } версии { $version } спрашивает снова
endpoint-repos-panel-update-sends-to = Новая версия отправляет это на { $host }.
endpoint-repos-panel-fields-changed = Его поля изменились.
# $fields is the field names joined by ", "
endpoint-repos-panel-needs-fields = Перед поиском нужно указать: { $fields }.
endpoint-repos-panel-no-longer-offered = Больше не предлагается списком источников.
endpoint-repos-panel-update-requests-to = Обновление отправляет запросы на { $host }.
endpoint-repos-panel-unreadable = Этот список не читается. «Обновить списки» попробует снова.
endpoint-repos-panel-add-list = Добавить список источников
endpoint-repos-panel-add-list-helper = Адрес его индексного файла или путь к нему на этом компьютере.
endpoint-repos-panel-checking = Проверка
endpoint-repos-panel-add = Добавить
endpoint-repos-panel-trust-question = Доверять { $name }?
endpoint-repos-panel-fingerprint = Отпечаток ключа  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    Источников: { $count }, подписано. Его источники решают, куда уходят ваши поиски; доверяйте списку, только если доверяете его автору.
endpoint-repos-panel-preview-trusted =
    { $url }
    Источников: { $count }, подписано, уже доверенный. Его источники решают, куда уходят ваши поиски; доверяйте списку, только если доверяете его автору.
endpoint-repos-panel-trust = Доверять
endpoint-repos-panel-saving = Сохранение
endpoint-repos-panel-install = Установить
endpoint-repos-panel-update = Обновить
endpoint-repos-panel-added-by-you = добавлен вами
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Обновить до v{ $version }
endpoint-repos-panel-add-value = Указать значение
endpoint-repos-panel-change = Изменить: { $fields }
endpoint-repos-panel-asks-for = Запрашивает: { $fields }.
endpoint-repos-panel-by-hand = Добавить вручную
endpoint-repos-panel-by-hand-helper = Сайт не из списков: лента с {"{"}query{"}"} вместо запроса или поисковый прокси. Ключ из адреса хранится отдельно, в хранилище паролей.
endpoint-repos-panel-name = Имя
endpoint-repos-panel-refreshing = Обновление
endpoint-repos-panel-refresh = Обновить списки

## ScheduleSheet.qml

schedule-episodes-range = Серии с { $first } по { $last }
# Qt date formats: a full day, a month, and the short ones on an episode tile
schedule-day-format = ddd, d MMM yyyy
schedule-month-format = MMMM yyyy
schedule-tile-day-format = d MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = Когда-то в { $year }
schedule-no-date-yet = Даты пока нет
# A date guessed from a weekly run
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = Без даты
schedule-tile-last = Последняя
schedule-tile-out = Вышла
schedule-ends-at = Заканчивается на серии { $number }
schedule-dated-by-site = С датой от { $site }: { $count }
schedule-dated-by-record = С датой из записи: { $count }
schedule-set-by-you = Задано вами: { $count }
schedule-guessed-weekly = По еженедельному графику: { $count }
schedule-nothing-set = Пока ничего не задано
schedule-saved = Сохранено
schedule-cleared = Заметки о расписании удалены
schedule-title = Серии и даты
schedule-season = Сезон { $number }
schedule-ask-agent = Спросить { $agent }
schedule-done-pasting = Готово
schedule-paste-list = Вставить список
schedule-agent-looking = { $agent } ищет даты. Ничего не сохранится, пока вы не нажмёте «Сохранить».
schedule-agent-stopped = { $agent } остановился: { $error }
schedule-stop = Стоп
schedule-paste-placeholder = Вставьте список релизов, таблицу или пост
schedule-paste-helper = То, что прочитал AniBeam, видно на сериях ниже.
schedule-agent-hint = Если на этом компьютере есть Claude Code, AniBeam может попросить его найти даты за вас.
schedule-show-more = Показать больше серий
schedule-pick-hint = Нажмите на серию, чтобы задать дату. Shift+щелчок выбирает диапазон.
schedule-picked-one = Серия { $number }:
schedule-picked = { $count ->
    [one] { $count } серия:
    [few] { $count } серии:
    [many] { $count } серий:
   *[other] { $count } серии:
}
# What a typed date may look like; the parser reads these shapes
schedule-date-placeholder = 7 января, март 2027, 2027
schedule-set-date = Задать дату
schedule-weekly = Еженедельно
schedule-not-last = Не последняя
schedule-last-episode = Последняя серия
schedule-clear-date = Убрать дату

## ShowPage.qml

show-title = Сериал
# AniList's formats
# AniList's airing states
# $runtime is a length such as "24 min"
show-runtime-each = по { $runtime }
show-add-to-list = В список
show-moving-to = Перенос в «{ $list }»
show-moved-to = Перенесено в «{ $list }»
show-relation-related = Связанное
show-no-poster = Нет постера
show-reading-anilist = Чтение AniList
show-not-in-library = Нет в вашей библиотеке
show-get-episodes = Скачать серии
show-trailer = Трейлер
show-your-score = Ваша оценка { $score }
show-rate-it = Оценить
show-anilist-score = AniList { $score }
show-forecast = { $percent }% для вас
show-because-you-watched = Потому что вы смотрели
show-hidden-loved = { $count ->
    [one] За этим стоит скрытый тег, который вы любите
   *[other] За этим стоят скрытые теги, которые вы любите: { $count }
}
show-you-gave-it = Ваша оценка: { $score }
show-story = Сюжет
show-genres-tags = Жанры и теги
show-studios = Студии  { $studios }
show-related = Связанное
show-recommendations = Рекомендации
show-from-anilist = С AniList
show-reading-rest = Читаем остальное с AniList

## ConnectSheet.qml

# $name is the tracker: AniList or MyAnimeList
connect-connected-to = Подключено к { $name }
connect-connect-to = Подключить { $name }
connect-did-not-finish = Вход не завершился
connect-finish-in-browser = Завершите в браузере
# $account is the user's name on the tracker
connect-signed-in-anilist = Вы вошли как { $account }. Ваш список на странице «Смотрю».
connect-signed-in-mal = Вы вошли как { $account }. Серии отмечаются там по мере просмотра.
connect-setup-lead = { $name } пускает AniBeam через ваше собственное приложение. Это займёт около минуты, один раз.
connect-page-open = Страница входа { $name } открыта в браузере. Разрешите там доступ AniBeam, и это окно закроется само.
connect-page-opening = Открываем страницу входа { $name } в браузере.
connect-open-watching = Открыть «Смотрю»
connect-done = Готово
connect-open-again = Снова открыть страницу входа
connect-secret-where = Это длинная строка под «Client Secret» на странице приложения на myanimelist.net/apiconfig.
connect-client-secret = Client Secret
connect-save-try-again = Сохранить и повторить
connect-change-client-id = Изменить Client ID
connect-step-create-anilist = Создайте клиент на AniList
connect-step-create-mal = Создайте приложение на MyAnimeList
connect-step-create-anilist-body = Откройте настройки разработчика AniList, создайте новый клиент и вставьте этот адрес в поле Redirect URL точно как здесь.
connect-step-create-mal-body = Откройте страницу API MyAnimeList, создайте ID с App Type «other» и вставьте этот адрес в поле App Redirect URL точно как здесь.
connect-open-developer-page = Открыть страницу разработчика
connect-copy-address = Копировать адрес
connect-step-paste-id = Вставьте ID приложения
connect-step-paste-id-anilist = AniList называет его Client ID: это число на странице нового клиента.
connect-step-paste-id-mal = MyAnimeList называет его Client ID: это длинная строка на странице нового приложения.
connect-client-id = Client ID
connect-client-secret-web = Client Secret (только для веб-приложений)
connect-step-connect-body = Откроется браузер, { $name } попросит войти и разрешить доступ AniBeam, а AniBeam поймает ответ.

## SettingsPage

settings-title = Настройки
settings-search-placeholder = Поиск
settings-section-library = Библиотека
settings-section-accounts = Аккаунты
settings-section-appearance = Оформление
settings-section-playback = Воспроизведение
settings-section-subtitles = Субтитры
settings-section-keys = Клавиатура
settings-section-downloads = Загрузки
settings-section-phone = Телефон
settings-section-storage = Хранилище и бэкап
settings-section-help = Справка
settings-section-about = О программе
settings-section-developer = Разработчику
settings-status-library-empty = Папок пока нет
settings-status-library =
    { $count ->
        [one] { $count } сериал
        [few] { $count } сериала
        [many] { $count } сериалов
       *[other] { $count } сериала
    } в { $folders ->
        [one] { $folders } папке
       *[other] { $folders } папках
    }
settings-status-library-missing =
    { $gone ->
        [one] { $gone } папка не найдена
        [few] { $gone } папки не найдены
        [many] { $gone } папок не найдено
       *[other] { $gone } папки не найдены
    }, { $count ->
        [one] { $count } сериал
        [few] { $count } сериала
        [many] { $count } сериалов
       *[other] { $count } сериала
    } в { $folders ->
        [one] { $folders } папке
       *[other] { $folders } папках
    }
settings-status-accounts-anilist = Синхронизация с AniList
settings-status-accounts-mal = Синхронизация с MyAnimeList
settings-status-accounts-both = Синхронизация с AniList и MyAnimeList
settings-status-accounts-none = Список не подключён
settings-status-appearance =
    { $mode ->
        [system] Как в системе
        [light] Светлая
       *[dark] Тёмная
    }, { $source ->
        [theme] цвета темы
       *[desktop] цвета рабочего стола
    }
settings-status-playback =
    { $skip ->
        [both] Пропуск опенингов и эндингов
        [intro] Пропуск опенингов
        [outro] Пропуск эндингов
       *[none] Всё играет целиком
    }
settings-status-playback-audio =
    { $skip ->
        [both] Пропуск опенингов и эндингов
        [intro] Пропуск опенингов
        [outro] Пропуск эндингов
       *[none] Всё играет целиком
    }, аудио { $languages }
settings-status-subtitles =
    Размер { $percent } %, { $override ->
        [AsScripted] стилизованные файлы как есть
        [Force] ваш стиль во всех файлах
       *[ScaleOnly] стилизованные файлы сохраняют вид
    }
settings-status-keys = Все клавиши AniBeam
settings-status-downloads-unread = Ещё не прочитано
settings-status-downloads-answered = qBittorrent ответил
settings-status-downloads-missing = Помощник поиска не установлен
settings-status-downloads-unconnected = Ещё не подключено
settings-status-phone-shared = Доступно в вашем tailnet
settings-status-phone-off = Не доступно
settings-status-storage = Кэш изображений: { $size }
settings-status-storage-unread = Изображения, экспорт и импорт
settings-status-help = Ответы на частые вопросы
settings-status-about-update = Вышла версия { $version }
settings-status-developer = Замер времени каждой страницы
settings-status-developer-last = { $page } построена за { $ms } мс
settings-results =
    { $count ->
        [one] Найдена { $count } настройка
        [few] Найдено { $count } настройки
        [many] Найдено { $count } настроек
       *[other] Найдено { $count } настройки
    }
settings-results-none = Ничего не найдено
settings-results-hint = Enter открывает первую.
settings-results-none-hint = Попробуйте слово попроще: тема, субтитры, трекер, пароль, папка.

# The search's index: each row or group by its name, as its own tab shows it, and the
# words a person might type for it
settings-find-folders =
    .words = добавить папка скан сканировать пересканировать удалить источник путь коллекция
settings-find-matching =
    .words = метаданные данные о сериале источник anilist tvmaze порядок привязка
settings-find-hidden-shows =
    .words = скрытые скрыть показать
settings-find-hidden-airing =
    .words = не интересно эфир показать снова
settings-find-calendar =
    .words = календарь ics ical эфир расписание подписка
settings-find-anime-lists =
    .words = anilist myanimelist mal вход войти подключить логин аккаунт отключить трекер
settings-find-main-list =
    .words = прогресс карточки основной трекер
settings-find-sync-now =
    .words = обновить прогресс списки трекер синхронизация
settings-find-on-hold =
    .words = отложено пауза неактивные заброшенные дни
settings-find-language =
    .words = язык перевод локаль английский русский
settings-find-mode =
    .words = тёмная светлая система тема ночь режим
settings-find-colours-from =
    .words = терминал рабочий стол система тема источник kitty цвета
settings-find-dark-theme =
    .words = цвет цветовая схема base16 тёмная
settings-find-light-theme =
    .words = цвет цветовая схема base16 светлая
settings-find-accent =
    .words = цвет акцент выделение
settings-find-text-size =
    .words = шрифт текст слова больше меньше крупнее масштаб читать доступность
settings-find-density =
    .words = компактно просторно отступы размер плотность
settings-find-poster-size =
    .words = карточки маленький средний большой постеры размер
settings-find-out-now =
    .words = эфир уже вышло карточки список строки стиль баннер постеры
settings-find-out-now-cards =
    .words = эфир уже вышло сетка колонки строки раскладка
settings-find-reduce-motion =
    .words = анимация доступность движение
settings-find-app-icon =
    .words = логотип значок иконка лаунчер док панель задач окно тёмный
settings-find-skip-intros =
    .words = опенинг op авто пропуск aniskip главы
settings-find-skip-outros =
    .words = эндинг ed титры авто пропуск aniskip
settings-find-mpv-settings =
    .words = mpv.conf конфиг дополнительно продвинутые
settings-find-discord =
    .words = discord rich presence статус профиль
settings-find-next-episode =
    .words = следующая серия кнопка название автовоспроизведение конец
settings-find-subtitle-languages =
    .words = язык дорожка субтитры сабы
settings-find-audio-languages =
    .words = язык дорожка озвучка японский аудио
settings-find-start-over =
    .words = сброс сбросить по умолчанию стиль субтитров
settings-find-size =
    .words = масштаб размер субтитры
settings-find-position =
    .words = высота отступ положение субтитры
settings-find-styled =
    .words = ass переопределить принудительно стиль
settings-find-font =
    .words = шрифт гарнитура
settings-find-text-colour =
    .words = цвет текста
settings-find-outline =
    .words = контур обводка граница
settings-find-background =
    .words = подложка фон прозрачность
settings-find-bold =
    .words = жирный насыщенность
settings-find-keys-player =
    .words = клавиатура горячие клавиши сочетания плеер
settings-find-keys-player-around =
    .words = клавиатура сочетания полный экран следующая предыдущая серия назад
settings-find-keys-around =
    .words = клавиатура сочетания назад вперёд правый щелчок меню полный экран
settings-find-keys-anywhere =
    .words = клавиатура горячие клавиши сочетания настройки выход
settings-find-torrent-client =
    .words = qbittorrent пароль имя пользователя адрес webui seedbox торрент-клиент
settings-find-search-helper =
    .words = anirss rss ленты токен команда сервер дополнительно помощник
settings-find-next-few =
    .words = скользящая серии наперёд удалить просмотренные хранить место на диске
settings-find-unsubscribe =
    .words = отписаться ленты вышедшие завершённые
settings-find-last-search =
    .words = загрузки запрос запомнить поиск
settings-find-search-sources =
    .words = источники ленты конфиг anirss поиск
settings-find-source-lists =
    .words = репозиторий репо источник подписанный источники jackett prowlarr список
settings-find-share-tailscale =
    .words = телефон android tailnet удалённый хост tailscale
settings-find-stream =
    .words = стриминг воспроизведение трансляция серии телефон файлы
settings-find-devices =
    .words = доверие доверенные выключить блокировать устройство tailscale
settings-find-back-up =
    .words = экспорт бэкап резервная копия файл json
settings-find-restore =
    .words = импорт восстановить файл json
settings-find-image-cache =
    .words = постеры очистить изображения диск кэш
settings-find-files =
    .words = база данных данные конфиг кэш папка путь файлы
settings-find-help-folders =
    .words = справка папка структура имена серии сезоны фильмы файлы
settings-find-help-recognised =
    .words = справка привязка неверный сериал не найден не распознан
settings-find-help-anilist =
    .words = справка anilist myanimelist mal вход подключить логин трекер
settings-find-help-downloads =
    .words = справка загрузки пусто источники ничего торрент qbittorrent
settings-find-help-data =
    .words = справка данные файлы бэкап база данных приватность
settings-find-help-else =
    .words = справка настройка первый запуск обзор сообщить ошибка баг проблема issue
settings-find-version =
    .words = сборка версия что нового изменения
settings-find-setup =
    .words = первый запуск обзор настройка заново пройти снова
settings-find-updates =
    .words = стабильная nightly канал проверить обновления
settings-find-developer-options =
    .words = отладка время замер разработчик
settings-find-ratings =
    .words = imdb рейтинг благодарность

## SettingsDownloadsTab

settings-downloads-title = Загрузки
settings-downloads-lead = Куда AniBeam отправляет то, что вы скачиваете, и как он ищет в лентах релизов.
settings-downloads-keep-none = Досмотренная серия удаляет предыдущую.
settings-downloads-keep-line = Просмотренные серии, хранимые позади текущей. При { $count } досмотр серии { $finishing } удаляет серию 1.
settings-downloads-anirss-ready = Работает, версия { $version } на { $url }.
settings-downloads-anirss-starting = Запуск помощника поиска.
settings-downloads-anirss-missing = Помощник поиска не установлен. Переустановите AniBeam или укажите свой anirss в поле «Команда».
settings-downloads-anirss-failed = Помощник поиска остановился. «Повторить» запустит его; в «Подробностях» то, что он сообщил.
settings-downloads-anirss-idle = Ещё не запущен. Он запустится при первом поиске на странице «Загрузки».
settings-downloads-torrent-trying = Пробуем { $client }.
settings-downloads-torrent-answered = { $client } { $version } ответил.
settings-downloads-torrent-failed = { $client } не подключился. Проверьте адрес и пароль выше и повторите.
settings-downloads-torrent-untried = Ещё не проверялось.
settings-downloads-mode-existing = Ваш собственный qBittorrent с включённым веб-интерфейсом.
settings-downloads-mode-managed = AniBeam запускает свой qBittorrent, который качает в ~/Downloads/Anime. Он продолжает работать после выхода из AniBeam, поэтому подписки продолжают качать.
settings-downloads-mode-auto = Использует qBittorrent по адресу ниже или свой.
settings-downloads-password-saved = Пароль для этого клиента сохранён здесь.
settings-downloads-password-anirss = Используется пароль, сохранённый помощником поиска; сохраните пароль здесь, чтобы не зависеть от него.
settings-downloads-password-none = Пароль не сохранён. Клиенту без входа с этой машины он не нужен.
settings-downloads-mismatch = Помощник поиска обращается к { $helperUrl }, а AniBeam к { $url }. Ваш собственный помощник поиска берёт адрес из своего config.toml, поэтому смена клиента здесь его не затрагивает.
settings-downloads-torrent-title = Торрент-клиент
settings-downloads-torrent-helper = qBittorrent для загрузок. Пароль хранится в системном хранилище паролей.
settings-downloads-which-client = Какой клиент
settings-downloads-pick-for-me = Выбрать за меня
settings-downloads-my-own = Свой
settings-downloads-built-in = Встроенный
settings-downloads-saved-clients = Сохранённые клиенты
settings-downloads-saved-clients-helper = Например, сидбокс и клиент на этом компьютере. Переключаться между ними можно и чипом клиента на странице «Загрузки».
settings-downloads-add-client = Добавить
settings-downloads-name = Имя
settings-downloads-name-placeholder = По его адресу
settings-downloads-address = Адрес
settings-downloads-address-managed-helper = Доступен только с этого компьютера, пароль не нужен.
settings-downloads-address-helper = Адрес веб-интерфейса.
settings-downloads-username = Имя пользователя
settings-downloads-password = Пароль
settings-downloads-secret-saved = Сохранён
settings-downloads-secret-unset = Не задан
settings-downloads-connection = Подключение
settings-downloads-open-config = Открыть его конфиг
settings-downloads-details = Подробности
settings-downloads-details-torrent = Это ответил { $client }, когда AniBeam к нему обратился.
settings-downloads-test-trying = Проверка
settings-downloads-test-save = Сохранить и проверить
settings-downloads-test = Проверить
settings-downloads-searching = Поиск
settings-downloads-search-helper = Помощник поиска
settings-downloads-searching-helper = AniBeam сам ищет в ваших источниках и отправляет выбранное в торрент-клиент. Помощник поиска не нужен.
settings-downloads-search-helper-helper = Часть AniBeam, которая ищет и подписывается, программа anirss. AniBeam запускает её за вас; меняйте это, только если запускаете свою.
settings-downloads-runs-in = Где идёт поиск
settings-downloads-runs-in-native-helper = Подписки в торрент-клиенте: серии приходят и без AniBeam.
settings-downloads-runs-in-helper = Помощник поиска, отдельная программа, которую запускает AniBeam или вы сами.
settings-downloads-engine-native = Сам AniBeam
settings-downloads-how-runs = Как запускается
settings-downloads-how-runs-sidecar = AniBeam запускает его при открытии и останавливает при выходе.
settings-downloads-how-runs-remote = AniBeam использует уже запущенный вами, по адресу и ключу ниже.
settings-downloads-runs-sidecar = Запускает AniBeam
settings-downloads-runs-remote = Запускаю я
settings-downloads-command = Команда
settings-downloads-command-helper = Имя в PATH или полный путь. ~/.local/bin тоже просматривается.
settings-downloads-server-address-helper = Базовый адрес сервера, без /api/v1.
settings-downloads-key = Ключ
settings-downloads-key-saved-helper = Ключ сохранён. Введите новый, чтобы заменить его, или очистите поле, чтобы удалить.
settings-downloads-key-helper = Ключ, который помощник поиска показал при запуске. С ним AniBeam получает доступ.
settings-downloads-details-anirss = Это помощник поиска сообщил при остановке.
settings-downloads-starting = Запуск
settings-downloads-reconnect = Переподключить
settings-downloads-save-first = Сначала сохраните изменения
settings-downloads-roll-title = Держать несколько следующих наготове
settings-downloads-roll-helper = Длинный сериал качается по нескольку серий, досмотренная запускает следующую. Весь сериал на диске не нужен.
settings-downloads-roll-big = Для длинных сериалов
settings-downloads-roll-big-helper = Всё длиннее { $count } серий качается по нескольку. В его строке в «Загрузках» можно скачать и всё сразу.
settings-downloads-roll-ahead = Серий наперёд
settings-downloads-roll-ahead-helper = Сколько серий хранится скачанными после текущей. 12 это примерно сезон.
# The unit after a slider's number: "12 episodes"
settings-downloads-unit-episodes = серий
settings-downloads-roll-over = Длинный, если больше
settings-downloads-roll-over-helper = Загрузки, где серий больше этого числа, качаются по нескольку.
settings-downloads-roll-watched = Просмотренные серии
settings-downloads-roll-watched-delete-helper = Удаляются по мере просмотра: на диске только непросмотренное.
settings-downloads-roll-watched-keep-helper = Каждый файл остаётся на диске после просмотра.
settings-downloads-keep-all = Хранить все
settings-downloads-keep-behind = Хранить позади
settings-downloads-subscriptions-title = Подписки и поиск
settings-downloads-last-search = Помнить последний поиск
settings-downloads-last-search-helper = «Загрузки» с последним поиском, иначе пустые.
settings-downloads-prune = Отписываться от вышедших сериалов
settings-downloads-prune-helper = Через 16 дней после выхода последней серии лента сериала удаляется. Сериал и его файлы остаются.
settings-downloads-sources-title = Источники поиска
settings-downloads-sources-helper = Куда идёт поиск, по порядку. Пустой источник передаёт поиск следующему.
settings-downloads-sources-empty = Источники пока не настроены
settings-downloads-sources-connect = Подключите помощника поиска, чтобы увидеть их
settings-downloads-sources-empty-helper = Добавьте список ниже, источники появятся тут.
settings-downloads-config-file = Файл настроек помощника поиска
settings-downloads-config-native-helper = Папки сохранения, теги и правила выбора лучшего: AniBeam сам следует им при запуске помощника. Источники тут только для него.
settings-downloads-config-helper = Содержит источники, папки сохранения, теги и правила выбора лучшего.
settings-downloads-unsaved = Несохранённые изменения
settings-downloads-unsaved-helper = Сохранение перезапустит помощника поиска с новыми настройками.
settings-downloads-discard = Отменить

## SettingsDeveloperTab

settings-developer-title = Разработчику
settings-developer-lead = Почему страница медленная, в цифрах. Каждый переключатель действует со следующей открытой страницы.
settings-developer-ms = { $ms } мс
settings-developer-mb = { $mb } МБ
settings-developer-seconds = { $seconds } с
settings-developer-switches = Переключатели
settings-developer-load-images = Загружать изображения
settings-developer-load-images-helper = Если выключено, постеры, баннеры и портреты не читаются и не декодируются.
settings-developer-overlay = Оверлей загрузки страницы
settings-developer-overlay-helper = Последняя загрузка страницы и память, в правом нижнем углу окна.
settings-developer-read-cache = Кэшировать чтение библиотеки
settings-developer-read-cache-helper = Ответ на полное чтение библиотеки хранится, пока библиотека не изменится. Если выключено, каждая страница снова спрашивает ядро.
settings-developer-poster-pool = Прогревать постеры в фоне
settings-developer-poster-pool-helper = Декодирует 60 постеров, которые вы, скорее всего, увидите, и держит их в памяти.
settings-developer-options = Параметры разработчика
settings-developer-options-helper = Выключает эту вкладку, оверлей и замеры.
settings-developer-memory = Память
settings-developer-memory-helper = Сколько ядро ОС насчитывает этому процессу. Изображения держат декодированные пиксели в памяти и на видеокарте.
settings-developer-memory-unknown = Эта система не сообщает, сколько памяти использует приложение.
settings-developer-in-use = Занято
settings-developer-peak = Пик
settings-developer-heap = Куча
settings-developer-files-mapped = Отображённые файлы
settings-developer-threads = Потоки
settings-developer-give-back = Вернуть память
settings-developer-give-back-helper = Сбрасывает то, что граф сцены может перестроить, собирает мусор JavaScript и отдаёт свободную кучу системе. Освобождается то, что приложение держало без надобности.
settings-developer-released = { $before } → { $after }
settings-developer-free-now = Освободить
settings-developer-images-total = Изображения с запуска
settings-developer-images-total-line = Декодировано { $decodes }, { $pixels } пикселей из { $disk } на диске, { $ms } мс
settings-developer-loads = Загрузки страниц
settings-developer-loads-helper = «Построена» это ожидание после клика: поток Qt больше ничего не делает, пока страница не появится. «Устоялась» это когда пришло её последнее изображение.
settings-developer-touring = Замер, { $at } из { $steps }
settings-developer-tour = Замерить все страницы
settings-developer-copy-report = Копировать отчёт
settings-developer-col-page = Страница
settings-developer-col-built = Построена
settings-developer-col-painted = Отрисована
settings-developer-col-settled = Устоялась
settings-developer-col-calls = Вызовы, данные
settings-developer-col-decodes = Декодирования, время, пиксели
settings-developer-col-items = Элементы
settings-developer-col-offscreen = Изображения вне вида
settings-developer-col-memory = Память
settings-developer-load-no-images = { $page }, без изображений
settings-developer-offscreen-of = { $offscreen } из { $total }
settings-developer-loads-empty = Откройте любую страницу, и она появится здесь.
settings-developer-load-images-off = { $page }, изображения выкл.
settings-developer-load-frames = Во время загрузки отрисовано кадров: { $frames }, самый длинный промежуток между двумя { $gap }.
settings-developer-load-frames-timed-out = Во время загрузки отрисовано кадров: { $frames }, самый длинный промежуток между двумя { $gap }. Ожидание изображений прервано через 15 с.
settings-developer-built = Построена
settings-developer-settled = Устоялась
settings-developer-core-data = Данные от ядра
settings-developer-pixels = Декодировано пикселей
settings-developer-page-items = Элементов на странице
settings-developer-cached-images = Изображений уже в памяти
settings-developer-calls = Вызовы ядра, сначала медленные
settings-developer-call-cached = { $name }  { $ms } мс  { $size }  из кэша
settings-developer-call = { $name }  { $ms } мс  { $size }  ядро { $core }, в QML { $qml }
settings-developer-decodes = Самые медленные декодирования
settings-developer-decodes-line = { $count } декодирований подряд в потоке изображений, { $span } от первого до последнего; запрошено { $requests }, уже декодировано { $cached }, заблокировано { $blocked }
settings-developer-decode = { $ms } мс  { $srcW }x{ $srcH } → { $w }x{ $h }  { $size }
settings-developer-decodes-none = Ничего не декодировано: все изображения уже были в памяти.
settings-developer-images-were-off = Изображения были выключены.
settings-developer-scene = Из чего состоит страница
settings-developer-scene-line = Элементов { $items }, вне окна { $offscreen }; текстов { $texts }, фигур { $shapes }, значков { $icons }; изображений { $images }, из них вне окна { $imagesOffscreen }, занимают { $size }
settings-developer-copy-load = Копировать эту загрузку
settings-developer-tour-title = Все страницы, с изображениями и без
settings-developer-tour-helper = Средний из трёх заходов на каждую страницу. Где две колонки совпадают, страницу замедляют не изображения.
settings-developer-col-built-on = Построена, с изображениями
settings-developer-col-built-off = Построена, без изображений
settings-developer-col-settled-on = Устоялась, с ними
settings-developer-col-settled-off = Устоялась, без них
settings-developer-col-decodes-time = Декодирования, время
settings-developer-col-data = Данные
settings-developer-tour-peak = Пик памяти за замер { $size }
settings-developer-events = События
settings-developer-events-helper = Всё, что сообщает ядро, все уровни, сначала новое. Каждое событие копируется в поток Qt как JSON; размер показывает, сколько стоила отправка.
settings-developer-event-rate = { $kind } { $count }/мин, { $size }
settings-developer-events-filter = Фильтр по словам, типу или этапу
settings-developer-event-plain = { $message }   (просто: { $plain })
settings-developer-copy-events = Копировать события

## SettingsKeysTab

settings-keys-title = Клавиатура
settings-keys-lead = Все клавиши AniBeam. Нажмите ? в любом месте, чтобы увидеть здешние.
settings-keys-player = В плеере
settings-keys-player-around = Навигация в плеере
settings-keys-around = Навигация
settings-keys-anywhere = Везде

## KeyHelp

key-help-title = Горячие клавиши
key-help-group-playback = Воспроизведение
key-help-group-navigation = Навигация
key-help-group-library = Библиотека
# Key names on the chips. " / " splits one chip into two, so keep it between keys.
key-help-key-space = Пробел / K
key-help-key-left-right = ← / →
key-help-key-ctrl-right = Ctrl+→
key-help-key-backspace = Backspace
key-help-key-up-down = ↑ / ↓
key-help-key-hold = Удерживать видео
key-help-key-page = Page Up / Down
key-help-key-home-end = Home / End
key-help-key-escape = Esc
key-help-key-alt-left = Alt+←
key-help-key-alt-right = Alt+→
key-help-key-shift-f10 = Shift+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / или Ctrl+F
key-help-key-ctrl-space = Ctrl+Пробел
key-help-key-shift-space = Shift+Пробел
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = Пуск или пауза
key-help-seek-5 = Перемотка на 5 с
key-help-seek-10 = Перемотка на 10 с
key-help-skip = Пропустить опенинг или эндинг, иначе 90 с
key-help-frame-step = Кадр назад или вперёд
key-help-speed = Медленнее или быстрее, шаг 0,25
key-help-normal-speed = Обычная скорость
key-help-volume = Громкость выше или ниже на 5
key-help-mute = Без звука
key-help-subtitles-toggle = Выключить и вернуть субтитры
key-help-subtitle-delay = Субтитры на 100 мс раньше или позже
key-help-audio-delay = Аудио на 100 мс раньше или позже
key-help-hold-speed = Скорость 2×, пока удерживаете
key-help-screenshot = Сохранить скриншот в Pictures
key-help-episode = Следующая или предыдущая серия
key-help-chapter = Предыдущая или следующая глава
key-help-start-end = В начало или к эндингу
key-help-fullscreen = Во весь экран
key-help-escape-player = Закрыть открытое, иначе выйти из полноэкранного режима, иначе выйти из плеера
key-help-toggle-list = Показать или скрыть этот список
key-help-library-search = Библиотека и поиск
key-help-search-page = Поиск на этой странице
key-help-activity = Журнал
key-help-settings = Настройки
key-help-quit = Выход
key-help-forward = Вперёд
key-help-escape-app = Закрыть открытое, иначе назад
key-help-context-menu = Меню правой кнопки
key-help-pick-card = Отметить карточку
key-help-pick-range = Выбрать все карточки до этой
key-help-pick-all = Выбрать все показанные карточки

## SettingsAboutTab

settings-about-title = О программе
settings-about-lead = Какая это версия AniBeam и как она узнаёт о следующей.
settings-about-update-failed = Обновление не завершилось. Попробуйте снова или скачайте со страницы релиза.
settings-about-update-ready = { $version } готов к установке.
settings-about-update-command = Вышел { $version }. Обновите командой { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = Вышел { $version }. Эта копия собрана из исходников: сделайте pull и пересоберите.
settings-about-update-elsewhere = Вышел { $version }, но для этого компьютера здесь нет загрузки.
settings-about-version = Версия
settings-about-build = AniBeam { $build }
settings-about-whats-new = Что нового
settings-about-install = Установить { $version }
settings-about-setup = Настройка
settings-about-setup-helper = Обзор, с которого начинается новая установка: папка, список и загрузки.
settings-about-run-setup = Пройти настройку снова
settings-about-updates = Обновления
settings-about-nightly-helper = Новая сборка почти каждый день. Новее, но менее проверенная.
settings-about-stable-helper = Новая версия, когда она проверена и выпущена.
settings-about-stable = Стабильная
settings-about-nightly = Nightly
settings-about-check-now = Проверить
settings-about-developer = Параметры разработчика
settings-about-developer-helper = Добавляет раздел «Разработчику»: замер страниц, память, полный журнал и отключение картинок.
settings-about-credits = Благодарности
settings-about-ratings = Рейтинги
settings-about-ratings-helper = Данные предоставлены IMDb (https://www.imdb.com) с разрешения.
settings-about-series = Сериалы и серии
settings-about-series-helper = AniList и TVmaze. Время пропуска от AniSkip.

## SettingsPlaybackTab

settings-playback-title = Воспроизведение
settings-playback-lead = Что плеер делает сам, чтобы вы трогали его, только когда хотите.
settings-playback-skipping = Пропуск
settings-playback-skipping-helper = Берёт главы файла, иначе отметки от зрителей. «Отменить» в плеере выключает пропуск до закрытия AniBeam.
settings-playback-skip-intros = Пропускать опенинги
settings-playback-skip-outros = Пропускать эндинги
settings-playback-skip-outros-helper = Начало эндинга также отмечает серию просмотренной.
settings-playback-next-episode = Следующая серия
settings-playback-name-next = Показывать название следующей серии
settings-playback-name-next-helper = Кнопка «Далее» в конце серии показывает название следующей. Если выключено, только номер.
settings-playback-languages = Языки
settings-playback-languages-helper = Коды языков по важности. Берётся первый, что есть в файле. Дорожка из плеера запоминается для сериала.
settings-playback-subtitle-languages = Языки субтитров
settings-playback-audio-languages = Языки аудио
settings-playback-discord = Показывать в Discord, что вы смотрите
settings-playback-discord-check-failed = AniBeam не смог проверить Discord. Попробуйте ещё раз через минуту.
settings-playback-discord-helper = Сериал, серия и постер видны в вашем профиле Discord, пока идёт просмотр. Ничего не отправляется, когда это выключено или Discord закрыт.
settings-playback-discord-unavailable = В этой сборке нет приложения Discord.
settings-playback-discord-connected = Подключено к Discord
settings-playback-discord-not-connected = Не подключено. AniBeam подключается, когда идёт серия и Discord открыт
settings-playback-advanced = Дополнительно
settings-playback-my-mpv = Мои настройки mpv
settings-playback-my-mpv-helper = Читает ваш mpv.conf. Параметры, работающие только при запуске, пропускаются, скрипты не загружаются никогда.
settings-playback-no-mpv-conf = Файл настроек mpv не найден. Запишите параметры в mpv.conf и включите это.

## SliderRow

# The spoken name of a slider whose number counts something: "On hold after, in days"
slider-row-spoken-unit = { $name }, в единицах: { $unit }
# The readout beside a slider: "60 days", "1.00 ×"
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Красный
swatches-accent-orange = Оранжевый
swatches-accent-yellow = Жёлтый
swatches-accent-green = Зелёный
swatches-accent-cyan = Бирюзовый
swatches-accent-blue = Синий

## LookPreview

look-preview-dark = Тёмная
look-preview-light = Светлая

## SettingsAccountsTab

settings-accounts-title = Аккаунты
settings-accounts-lead = Подключите AniList или MyAnimeList, и каждая досмотренная серия будет отмечаться там за вас. Счётчики только растут.
settings-accounts-syncing = Синхронизация списков
settings-accounts-synced = Списки синхронизированы
settings-accounts-lists = Ваши списки аниме
settings-accounts-lists-helper = Без них AniBeam всё равно помнит, что вы смотрели на этом компьютере. Список на AniList или MyAnimeList переносит это на телефон и в профиль.
settings-accounts-progress = Прогресс
settings-accounts-main-list = Основной список
settings-accounts-main-list-helper = Карточки показывают прогресс из него. Второй тоже получает все обновления.
settings-accounts-local-lists = Списки на этом компьютере
settings-accounts-local-lists-helper =
    { $count ->
        [one] У { $count } сериала
       *[other] У { $count } сериалов
    } статус или оценка есть только здесь. Перенесите их в { $tracker }, чтобы видеть на телефоне и в профиле.
settings-accounts-review-move = Проверить и перенести
settings-accounts-nothing-to-move = Переносить больше нечего
settings-accounts-sync-now = Синхронизировать сейчас
settings-accounts-sync-now-helper = Заново читает каждый подключённый список. AniBeam делает это и при запуске.
settings-accounts-sync = Синхронизировать
settings-accounts-move-on-hold = Переносить в «Отложено»
settings-accounts-move-on-hold-helper = Сериал из «Смотрю», который вы давно не открывали, переносится в «Отложено» в ваших списках. Просмотр возвращает его обратно. Сериал, ждущий следующую серию, остаётся на месте.
settings-accounts-after = Через
settings-accounts-after-line = Дней без просмотра серий
settings-accounts-days-unit = дн.

## SettingsPhoneTab

settings-phone-title = Телефон
settings-phone-lead = AniBeam на телефоне может играть эту библиотеку, показывать, что в эфире, и следить за загрузками, где бы вы ни были.
settings-phone-online = В сети
settings-phone-offline = Не в сети
settings-phone-untrusted = Ничего не получает с этого компьютера
settings-phone-sharing = Доступ
settings-phone-sharing-helper = Доступ только у ваших устройств в Tailscale. Просмотренное на телефоне хранит место и прогресс тут.
settings-phone-share = Открыть доступ через Tailscale
settings-phone-stream = Стриминг серий
settings-phone-stream-on = Ваши устройства смотрят и транслируют серии отсюда.
settings-phone-stream-off = Ваши устройства видят библиотеку и что в эфире, но не могут играть отсюда.
settings-phone-host-service = Служба хоста
settings-phone-not-running = Не запущена
settings-phone-running = Работает
settings-phone-service-off-helper = Выполните команду раз, и устройства видят AniBeam сразу после входа, без окна.
settings-phone-service-on-helper = Ваши устройства подключаются к AniBeam без открытого окна. Пока это окно открыто, доступ даёт оно вместо службы.
settings-phone-copy-command = Копировать команду
settings-phone-command-copied = Команда скопирована
# Tailscale ties the device to no person, so this computer cannot tell whose it is.
settings-phone-nobody = Ничего не получает: Tailscale не привязывает его к человеку
# The people whose devices this computer serves, its own person first.
settings-phone-people = Люди
# The chip on the person this computer belongs to.
settings-phone-own-profile = Профиль этого компьютера
# The owner's line: this computer plus how many of their other devices it serves.
settings-phone-person-own-devices =
    { $count ->
        [0] Этот компьютер
        [one] Этот компьютер и { $count } устройство
        [few] Этот компьютер и { $count } устройства
        [many] Этот компьютер и { $count } устройств
       *[other] Этот компьютер и { $count } устройства
    }
# Anyone else's line: how many of their devices this computer serves.
settings-phone-person-devices =
    { $count ->
        [one] { $count } устройство
        [few] { $count } устройства
        [many] { $count } устройств
       *[other] { $count } устройства
    }
settings-phone-devices = Устройства в вашем tailnet
settings-phone-devices-helper = Пока доступ открыт, любое устройство в tailnet видит этот компьютер. Выключенное ничего не получит.
settings-phone-addresses = Адреса
settings-phone-addresses-helper = Введите один из них в приложение на телефоне, когда оно спросит в первый раз.
settings-phone-copy = Копировать
settings-phone-address-copied = Адрес скопирован

## SettingsHelpTab

settings-help-title = Справка
settings-help-lead = Короткие ответы на частые вопросы, со ссылкой на решение.
settings-help-folders = Как должны выглядеть папки?
settings-help-folders-helper = Папка на сериал, серии внутри. В имени файла номер, как в релизах: «Frieren - 01.mkv» или «Frieren S01E01.mkv». Сезон 2: своя папка или «Season 2» внутри. Фильмы: папка Movies, файл на фильм.
settings-help-for-example = Например
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = Папка ещё не добавлена
settings-help-reads-folders =
    { $count ->
        [one] AniBeam читает { $count } папку
        [few] AniBeam читает { $count } папки
        [many] AniBeam читает { $count } папок
       *[other] AniBeam читает { $count } папки
    }
settings-help-add-folder-helper = Добавьте папку, где лежат сериалы, а не каждый сериал по отдельности.
settings-help-add-folder = Добавить папку
settings-help-open-folders = Открыть папки
settings-help-match = Почему сериал не распознался?
settings-help-match-helper = AniBeam ищет каждую папку по имени, сначала на AniList, потом на TVmaze. Лишние слова, опечатка или название на другом языке могут помешать. Выберите нужный сериал вручную в «О сериалах»; ваш выбор никогда не заменяется.
settings-help-all-recognised = Все сериалы распознаны
settings-help-unrecognised =
    { $count ->
        [one] Не распознан { $count } сериал
        [few] Не распознано { $count } сериала
        [many] Не распознано { $count } сериалов
       *[other] Не распознано { $count } сериала
    }
settings-help-open-show-info = Открыть «О сериалах»
settings-help-connect = Как подключить AniList?
settings-help-connect-helper = Подключите его, и каждая досмотренная серия отметится в списке, а телефон и профиль не отстанут. AniList один раз просит создать своё мини-приложение; AniBeam проведёт по шагам, это около минуты. MyAnimeList так же.
settings-help-both-connected = Оба подключены
settings-help-anilist-connected = AniList подключён
settings-help-mal-connected = MyAnimeList подключён
settings-help-none-connected = Пока ничего не подключено
settings-help-connect-anilist = Подключить AniList
settings-help-connect-mal = Подключить MyAnimeList
settings-help-downloads = Почему в загрузках пусто?
settings-help-downloads-helper = В AniBeam нет своих мест для поиска. Добавьте список источников в настройках, в «Загрузках», и поставьте нужные. Для загрузки нужен и торрент-клиент: AniBeam запустит свой или возьмёт ваш qBittorrent.
settings-help-sources = Источники и торренты
settings-help-open-downloads = Открыть настройки загрузок
settings-help-data = Где мои данные?
settings-help-data-helper = Библиотека, история и настройки хранятся на этом компьютере. Входы в AniList и MyAnimeList в системном хранилище паролей, а без него в файле только для вас. Бэкап всего в один файл в «Хранилище и бэкап».
settings-help-library-history = Библиотека и история
settings-help-show-folder = Показать папку
settings-help-backup = Бэкап и возврат
settings-help-open-storage = Открыть «Хранилище и бэкап»
settings-help-more = Что-то другое?
settings-help-more-helper = Повторите настройку или опишите сбой. Скриншот очень поможет.
settings-help-first-setup = Первая настройка
settings-help-first-setup-helper = Папка, список и загрузки, по шагу за раз.
settings-help-run-setup = Пройти настройку снова
settings-help-report = Сообщить о проблеме
settings-help-report-helper = Открывает отчёт об ошибке на GitHub в браузере, с уже указанной версией.

## SettingsSubtitlesTab

settings-subtitles-title = Субтитры
settings-subtitles-lead = С чем начинается каждая серия. Изменения сразу видны на кадре ниже.
settings-subtitles-reset-done = Субтитры выглядят как изначально
settings-subtitles-size-place = Размер и положение
settings-subtitles-size = Размер
settings-subtitles-position = Положение
settings-subtitles-position-helper = 100 это нижний край. Меньшие числа поднимают строки выше по кадру.
settings-subtitles-style = Стиль
settings-subtitles-styled = Стилизованные субтитры
settings-subtitles-as-made-helper = Стилизованные файлы (ASS) выглядят точно так, как их задумали авторы.
settings-subtitles-mine-helper = Ваш стиль ниже применяется ко всем файлам. Надписи и караоке могут сломаться.
settings-subtitles-size-only-helper = Оформленные субтитры берут только ваш размер, простые ваш стиль.
settings-subtitles-as-made = Как есть
settings-subtitles-size-only = Только размер
settings-subtitles-mine = Мой
settings-subtitles-font = Шрифт
settings-subtitles-font-invalid = Введите название шрифта, например Noto Sans.
settings-subtitles-bold = Жирный
settings-subtitles-text-colour = Цвет текста
settings-subtitles-colour-invalid = Введите цвет вида #ffcc00.
settings-subtitles-outline = Контур
settings-subtitles-outline-invalid = Толщина это число, 0 или больше.
settings-subtitles-outline-colour-invalid = Введите цвет вида #000000.
settings-subtitles-outline-helper = Толщина, затем цвет.
settings-subtitles-shadow = Тень
settings-subtitles-shadow-invalid = Число, 0 или больше. 0 значит без тени.
settings-subtitles-shadow-helper = Насколько далеко падает тень, 0 без тени.
settings-subtitles-box = Подложка
settings-subtitles-box-helper = Тёмный фон за текстом. 0 убирает его.
settings-subtitles-start-over = Начать заново
settings-subtitles-start-over-helper = Возвращает размер, шрифт, цвета, контур, подложку и положение к исходным.
settings-subtitles-reset = Сбросить

## SettingsDataTab

settings-data-title = Хранилище и бэкап
settings-data-lead = Перенос библиотеки на другой ПК, бэкап и очистка места от картинок.
settings-data-backed-up = Бэкап сохранён в { $path }
settings-data-restore-finished = Восстановление завершено
settings-data-images-removed =
    { $count ->
        [one] Удалено { $count } изображение
        [few] Удалено { $count } изображения
        [many] Удалено { $count } изображений
       *[other] Удалено { $count } изображения
    }
# What the last restore did: the parts below, joined with commas, make one sentence
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] добавлена { $count } папка
        [few] добавлено { $count } папки
        [many] добавлено { $count } папок
       *[other] добавлено { $count } папки
    }
settings-data-summary-series = создано сериалов: { $count }
settings-data-summary-matches =
    { $count ->
        [one] применена { $count } привязка
        [few] применено { $count } привязки
        [many] применено { $count } привязок
       *[other] применено { $count } привязки
    }
settings-data-summary-views =
    объединено: { $views ->
        [one] { $views } просмотр
        [few] { $views } просмотра
        [many] { $views } просмотров
       *[other] { $views } просмотра
    } и { $episodes ->
        [one] { $episodes } досмотренная серия
        [few] { $episodes } досмотренные серии
        [many] { $episodes } досмотренных серий
       *[other] { $episodes } досмотренной серии
    }
settings-data-summary-resume-points =
    { $count ->
        [one] объединена { $count } точка продолжения
        [few] объединено { $count } точки продолжения
        [many] объединено { $count } точек продолжения
       *[other] объединено { $count } точки продолжения
    }
settings-data-summary-accounts =
    { $count ->
        [one] перенесён { $count } аккаунт списка
        [few] перенесено { $count } аккаунта списков
        [many] перенесено { $count } аккаунтов списков
       *[other] перенесено { $count } аккаунта списков
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } файл получил вашу правку.
        [few] { $count } файла получили вашу правку.
        [many] { $count } файлов получили вашу правку.
       *[other] { $count } файла получили вашу правку.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } папка не найдена на этом компьютере.
        [few] { $count } папки не найдены на этом компьютере.
        [many] { $count } папок не найдено на этом компьютере.
       *[other] { $count } папки не найдены на этом компьютере.
    }
settings-data-summary-left-out = Пропущено: { $fields }.
settings-data-backup = Бэкап
settings-data-back-up-library = Сохранить бэкап библиотеки
settings-data-back-up-helper = Один файл: папки, привязки, история.
settings-data-last-written = Последний сохранён в { $path }
settings-data-open-folder = Открыть папку
settings-data-back-up = Сохранить
settings-data-private = Включить личные данные
settings-data-private-helper = Плюс входы в AniList и MyAnimeList, история и настройки. Входы пишутся открытым текстом: держите файл при себе.
settings-data-restore-from = Восстановить из бэкапа
settings-data-restore-helper = Загружает бэкап в эту библиотеку. Данные на диске важнее; файл дополняет остальное.
settings-data-restore = Восстановить
settings-data-image-cache = Кэш изображений
settings-data-image-cache-helper = Постеры, баннеры и портреты хранятся на диске для работы офлайн. Удалённые скачаются, когда понадобятся.
settings-data-cache-unmeasured = Не удалось измерить кэш изображений
settings-data-cache-size =
    { $count ->
        [one] { $size } в { $count } изображении
       *[other] { $size } в { $count } изображениях
    }
settings-data-cache-empty = Нет кэшированных изображений
settings-data-cache-unmeasured-helper = Попробуйте ещё раз через минуту. С самими изображениями всё в порядке.
settings-data-remove-images =
    { $count ->
        [one] Удалить { $count } изображение?
        [few] Удалить { $count } изображения?
        [many] Удалить { $count } изображений?
       *[other] Удалить { $count } изображения?
    }
settings-data-files = Где AniBeam хранит файлы
settings-data-files-database = База библиотеки
settings-data-files-data = Данные
settings-data-files-settings = Настройки
settings-data-files-cache = Кэш
settings-data-backup-filter = Бэкап AniBeam (*.json)
settings-data-all-files-filter = Все файлы (*)
settings-data-backing-up = Сохранение
settings-data-restoring = Восстановление

## SettingsLibraryTab

settings-library-title = Библиотека
settings-library-lead = Папки, за которыми следит AniBeam, и как он их разбирает.
settings-library-not-scanned = Ещё не сканировалось
settings-library-scan-nothing-changed = Ничего не изменилось
settings-library-scan-added = добавлено { $count }
settings-library-scan-changed = изменено { $count }
settings-library-scan-removed = удалено { $count }
# The last scan of one folder: "Anime: 3 added, 1 removed"
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = Аниме: сериалы, фильмы и их связи. Просмотренное отмечается в ваших трекерах.
settings-library-tvmaze-about = Остальное ТВ: игровое кино, мультфильмы, документалистика. В трекерах ничего не отмечается.
settings-library-tried-first = Первым
settings-library-tried-second = Вторым
settings-library-tried-third = Третьим
settings-library-tried-fourth = Четвёртым
settings-library-tried-nth = { $number }-м
settings-library-folders = Папки
settings-library-folders-helper = Каждая папка в них это сериал. Файл в корне папки Movies это фильм.
settings-library-rescan-started = Пересканирование запущено
settings-library-load-failed = Не удалось прочитать папки
settings-library-load-failed-helper = AniBeam не смог прочитать список папок. Попробуйте ещё раз через минуту.
settings-library-no-folders = Папок пока нет
settings-library-no-folders-helper = Укажите AniBeam папку с аниме. Сериалы и фильмы он найдёт сам.
settings-library-scanned = Просканировано { $when }
settings-library-add-a-folder = Добавить папку
settings-library-scan-all = Сканировать всё
settings-library-scan-started = Сканирование запущено
settings-library-add-folder = Добавить папку
settings-library-matching = Привязка
settings-library-matching-helper = Новая папка ищется по порядку до первого хорошего совпадения. Ручная привязка не заменяется.
settings-library-try-later = Позже
settings-library-try-first = Первым
settings-library-check-matches = Проверить привязки
settings-library-check-matches-helper = «О сериалах» показывает найденное по всем сериалам и даёт исправить ошибки.
settings-library-open-show-info = Открыть «О сериалах»
settings-library-hidden-shows = Скрытые сериалы
settings-library-show-hidden = Показывать скрытые
settings-library-show-hidden-helper = Показывает скрытые сериалы везде до закрытия AniBeam.
settings-library-hidden-from-airing = Скрыто из «В эфире»
settings-library-hidden-from-airing-helper = Сериалы, отмеченные как неинтересные. Каждый не показывается в «В эфире», пока вы его не вернёте.
settings-library-hidden-from-airing-empty = Пока пусто. Сериалы, отмеченные как неинтересные в «В эфире», собираются здесь, и любой можно вернуть.
settings-library-show-again = Вернуть
settings-library-calendar = Календарь
settings-library-calendar-feed = Время выхода в календаре
settings-library-calendar-feed-helper = Ведёт файл календаря с ближайшими сериями сериалов из ваших списков и обновляет его при смене расписания.
settings-library-calendar-file = Файл календаря
settings-library-calendar-file-helper = Подпишитесь на этот файл в приложении календаря.
settings-library-copy = Копировать
settings-library-path-copied = Путь скопирован
settings-library-open-folder = Открыть папку
settings-library-notifications = Уведомления
settings-library-new-episodes = Новые серии
settings-library-new-episodes-helper = Уведомление на рабочем столе о новой серии из «Смотрю». Клик открывает сериал.
settings-library-folder-added = Папка добавлена, сканирование

## SettingsAppearanceTab

settings-appearance-language = Язык
# $language is the system's language in its own name ("Dansk")
settings-appearance-language-system = Как в системе ({ $language })
settings-appearance-text-smaller = Мельче
settings-appearance-text-normal = Обычный
settings-appearance-text-large = Крупный
settings-appearance-text-larger = Крупнее
settings-appearance-text-largest = Максимум
settings-appearance-title = Оформление
settings-appearance-lead = Изменения видны сразу, здесь и на всех страницах. Они хранятся в ~/.config/anibeam/theme.toml, который можно править и вручную.
settings-appearance-colours = Цвета
settings-appearance-mode = Режим
settings-appearance-mode-dark = Тёмный
settings-appearance-mode-light = Светлый
settings-appearance-mode-system = Системный
settings-appearance-colours-from = Цвета из
settings-appearance-from-theme-helper = Две темы ниже: тёмная и светлая.
settings-appearance-from-desktop-helper = Цвета вашего терминала, а если их нет, цветовая схема и акцент рабочего стола.
settings-appearance-from-desktop = Рабочий стол
settings-appearance-from-theme = Тема
settings-appearance-dark-theme = Тёмная тема
settings-appearance-light-theme = Светлая тема
settings-appearance-themes-helper = Файлы тем, положенные в ~/.config/anibeam/themes, появятся в обоих списках.
settings-appearance-accent = Акцент
settings-appearance-accent-helper = Выделяет выбранное, текущее и новое.
settings-appearance-icon = Значок
settings-appearance-app-icon = Значок приложения
settings-appearance-app-icon-helper = У окна, на панели задач и в лаунчере. Тёмный меняет местами жёлтый и чёрный.
settings-appearance-icon-default = Обычный
settings-appearance-icon-default-name = Обычный значок приложения
settings-appearance-icon-dark = Тёмный
settings-appearance-icon-dark-name = Тёмный значок приложения
settings-appearance-size-motion = Размер и движение
settings-appearance-text-size = Размер текста
settings-appearance-text-size-helper = Размер текста везде в AniBeam, поверх системного размера.
settings-appearance-density = Плотность
settings-appearance-density-helper = Простор вокруг всего, от тесного до свободного.
settings-appearance-density-compact = Компактно
settings-appearance-density-normal = Обычно
settings-appearance-density-comfortable = Просторно
settings-appearance-poster-size = Размер постеров
settings-appearance-poster-size-helper = Насколько большой сериал в библиотеке и на главной.
settings-appearance-poster-small = Маленький
settings-appearance-poster-medium = Средний
settings-appearance-poster-large = Большой
settings-appearance-out-now = «Уже вышло» в эфире
settings-appearance-out-now-helper = Карточки с баннерами или список мини-постеров с названиями, три ряда.
settings-appearance-out-now-cards = Карточки
settings-appearance-out-now-list = Список
settings-appearance-out-now-grid = Карточки «Уже вышло»
settings-appearance-out-now-grid-helper = Число карточек, столбцы на ряды, до кнопки «Все».
# A grid of cards, columns by rows: "4 × 2"
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Меньше движения
settings-appearance-reduce-motion-helper = Убирает заставку библиотеки и прочую анимацию для красоты.

## LookPane

look-pane-sample-series = Пример сериала
look-pane-episode-code = Сер. { $number }
look-pane-library = Библиотека
look-pane-series-count =
    { $count ->
        [one] { $count } сериал
        [few] { $count } сериала
        [many] { $count } сериалов
       *[other] { $count } сериала
    }
look-pane-search = Поиск по названиям и папкам
look-pane-search-hint = / или Ctrl+K
look-pane-all = Все
look-pane-series = Сериалы
look-pane-movies = Фильмы
look-pane-plain = Обычный
look-pane-selected = Выбранный
look-pane-errors =
    { $count ->
        [one] { $count } ошибка
        [few] { $count } ошибки
        [many] { $count } ошибок
       *[other] { $count } ошибки
    }
look-pane-type-large = Крупный
look-pane-type-normal = Обычный
look-pane-type-small = Мелкий, тусклый
look-pane-sample-switch = Пример переключателя
look-pane-button = Кнопка
look-pane-scan = Скан
look-pane-sample-status =
    { $count ->
        [one] { $title }: { $count } новая серия
        [few] { $title }: { $count } новые серии
        [many] { $title }: { $count } новых серий
       *[other] { $title }: { $count } новой серии
    }

## FeedPage

feed-title = Лента
feed-aired = { $episode } вышла { $when }
feed-downloaded = Файл скачан { $when }
feed-countdown-days = { $days } д { $hours } ч
feed-countdown-hours = { $hours } ч { $minutes } мин
feed-countdown-minutes = { $minutes } мин
feed-section-this-week = Эта неделя
feed-new-episodes =
    { $count ->
        [one] { $count } новая серия
        [few] { $count } новые серии
        [many] { $count } новых серий
       *[other] { $count } новой серии
    }
feed-section-earlier = Раньше
feed-latest-out = Последняя вышла { $when }
feed-section-coming-soon = Скоро
feed-this-week-count = { $count } на этой неделе
feed-section-everything-else = Всё остальное
feed-out-this-week = { $count } вышло на этой неделе
feed-nothing-scheduled = Пока ничего не запланировано.
feed-next = Далее: «{ $title }» через { $countdown }
feed-nothing-new = На этой неделе ничего нового.
feed-new-this-week =
    { $count ->
        [one] { $count } новая серия на этой неделе.
        [few] { $count } новые серии на этой неделе.
        [many] { $count } новых серий на этой неделе.
       *[other] { $count } новой серии на этой неделе.
    }
feed-sort-recent = Недавно вышедшие
feed-sort-upcoming = Скоро
feed-failed-title = Лента не загрузилась
feed-empty-title = Пока пусто
feed-failed-body = AniBeam не смог прочитать ленту. Попробуйте ещё раз через минуту.
feed-empty-body-folders = Лента заполняется по мере выхода и появления серий. «В эфире» показывает, что выходит в этом сезоне.
feed-empty-body = Добавьте папку с аниме, и лента заполнится её сериями.
feed-add-folder = Добавить папку
feed-airing-now = В эфире

## RecommendedPage

recommended-title = Рекомендуем вам
recommended-ranked-minutes = Подобрано { $minutes } мин назад
recommended-ranked-hours = Подобрано { $hours } ч назад
recommended-ranked-days = Подобрано { $days } дн. назад
recommended-scope-name = Рекомендации из
recommended-scope-all = Все
recommended-scope-library = В библиотеке
recommended-scope-list = В списке
recommended-ranking = Подбор
recommended-rank-again = Подобрать заново
recommended-no-scores-title = Сначала оцените несколько сериалов
recommended-no-list-title = Пока не на чем учиться
recommended-never-fitted-title = Подбираем сериалы
recommended-nothing-left-title = Здесь больше ничего нет
recommended-no-scores-body = Рекомендации учатся на оценках в вашем списке. Оцените несколько сериалов на AniList, и они появятся здесь.
recommended-no-list-body = Подключите AniList в настройках или ведите список тут, и страница учтёт оценки.
recommended-never-fitted-body = Первый подбор читает AniList минуту-другую. Он идёт в фоне.
recommended-nothing-left-library-body = Всё подходящее в вашей библиотеке просмотрено. «Все» находит сериалы, которых у вас ещё нет.
recommended-nothing-left-body = Всё подходящее в ваших списках просмотрено. «Все» находит сериалы, которых нет в ваших списках.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] Перенос { $count } сериала в { $site }
       *[other] Перенос { $count } сериалов в { $site }
    }
move-lists-score = оценка { $score }
move-lists-title = Перенести список в { $site }
move-lists-body =
    { $count ->
        [one] У { $count } сериала
       *[other] У { $count } сериалов
    } статус или оценка есть только на этом компьютере. При переносе они попадут в ваш список { $site }, где их увидят телефон и профиль. Для этих сериалов то, что сейчас в { $site }, будет заменено. Ничего не отправится, пока вы не нажмёте «Перенести».
move-lists-head-show = Сериал
move-lists-head-here = На этом компьютере
move-lists-head-tracker = Сейчас в { $site }
move-lists-nothing = Ничего
move-lists-not-on-list = Нет в вашем списке
move-lists-later-hint = Позже: Настройки, Аккаунты
move-lists-not-now = Не сейчас
move-lists-move =
    { $count ->
        [one] Перенести { $count } сериал
        [few] Перенести { $count } сериала
        [many] Перенести { $count } сериалов
       *[other] Перенести { $count } сериала
    }

## StatsPage

stats-title = Статистика
stats-month-jan = Янв
stats-month-feb = Фев
stats-month-mar = Мар
stats-month-apr = Апр
stats-month-may = Май
stats-month-jun = Июн
stats-month-jul = Июл
stats-month-aug = Авг
stats-month-sep = Сен
stats-month-oct = Окт
stats-month-nov = Ноя
stats-month-dec = Дек
stats-nothing-yet = Пока ничего
stats-ranked-episodes =
    { $count ->
        [one] { $count } сер.
       *[other] { $count } сер.
    }
stats-year-summary = Что вы смотрели в { $year }
stats-empty-title = Пока ничего не просмотрено
stats-empty-body = Досмотрите серию, и она засчитается здесь: часы, жанры и просмотренные сезоны.
stats-open-library = Библиотека
stats-hours-watched = часов просмотра
stats-episodes-label =
    { $count ->
        [one] серия
        [few] серии
        [many] серий
       *[other] серии
    }
stats-series-finished = сериалов досмотрено
stats-none = Нет
stats-busiest-month = самый активный месяц
stats-by-month = По месяцам
stats-month-hours = { $hours } ч
stats-top-genres = Любимые жанры
stats-top-studios = Любимые студии
stats-seasons-watched = Просмотренные сезоны
stats-recap-title = Мой год в аниме
stats-recap-hours = часов
stats-recap-episodes = серий
stats-recap-top-genre = любимый жанр
stats-save-image = Сохранить картинкой
stats-save-dialog-title = Сохранить итоги
stats-save-filter-png = Изображение PNG (*.png)
stats-recap-saved = Итоги сохранены
stats-recap-save-failed = Не удалось сохранить итоги

## TrackerRow

tracker-row-connected = Подключено как { $user }, синхронизировано { $when }
tracker-row-unknown-user = ?
tracker-row-never = никогда
tracker-row-waiting = Ждём, пока вы разрешите доступ в браузере
tracker-row-not-connected = Не подключено
tracker-row-open-again = Открыть снова
tracker-row-open-again-tip = Снова открыть страницу входа
tracker-row-disconnect = Отключить
tracker-row-disconnect-question = Отключить { $name }? Ваш вход будет удалён.
tracker-row-secret-saved = Для этого приложения сохранён Client Secret.
tracker-row-remove-secret = Удалить
tracker-row-secret-removed = Client Secret удалён
tracker-row-add-secret = Добавить Client Secret
tracker-row-secret-where = Это длинная строка под «Client Secret» на странице приложения на myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Client Secret
tracker-row-save-connect = Сохранить и подключить

## RemoveSheet

remove-unsubscribe-delete = Отписаться и удалить
remove-unsubscribe-torrents = Отписаться и удалить торренты
remove-unsubscribe = Отписаться
remove-delete-files = Удалить вместе с файлами
remove-torrents =
    { $count ->
        [one] Удалить торрент
       *[other] Удалить торренты
    }
remove-also-torrents =
    { $count ->
        [one] Также удалить его торрент
        [few] Также удалить его { $count } торрента
        [many] Также удалить его { $count } торрентов
       *[other] Также удалить его { $count } торрента
    }
remove-torrents-go-with-files = Удаление файлов заберёт и торренты.
remove-torrents-files-stay = Они уйдут из клиента; файлы останутся.
remove-also-files = Также удалить файлы
remove-files-default-where = то, что скачали торренты
remove-files-detail-rule = Перемещает { $where } в корзину и удаляет торренты.
remove-files-detail = Перемещает { $where } в корзину.
## NotesSheet

notes-your-tracker = вашем трекере
notes-title = Заметки
notes-placeholder = Где остановились, кто посоветовал, какой релиз скачать
notes-over-limit = { $count ->
    [one] На { $count } символ больше лимита
    [few] На { $count } символа больше лимита
    [many] На { $count } символов больше лимита
   *[other] На { $count } символа больше лимита
}
notes-saved-to = Сохранено на { $sites }

## RecommendationCard

recommendation-card-available = Есть
recommendation-card-open-library = Открыть «{ $title }» в библиотеке
recommendation-card-open-anilist = Открыть «{ $title }» на AniList

## SearchField

search-field-placeholder = Поиск по названиям и папкам
search-field-hint = / или Ctrl+K
search-field-clear = Очистить поиск

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent }%
recommended-hover-hidden-tags =
    { $count ->
        [one] Скрытый тег, который вы любите
       *[other] Скрытые теги, которые вы любите: { $count }
    }
recommended-hover-because = Потому что вы смотрели

## PersonCard

person-card-unknown = Неизвестно
person-card-role-main = Главный
person-card-role-supporting = 2-й план
person-card-role-background = Фоновый
person-card-the-character = персонаж
person-card-open = Открыть { $name } на AniList

## StatusDot

status-dot-on-list = В вашем списке: { $status }

## TrackerNudge

tracker-nudge-title = Список аниме
tracker-nudge-body = Отметки в AniList и MAL.
tracker-nudge-connect-anilist = Подключить AniList
tracker-nudge-connect-mal = Подключить MyAnimeList
tracker-nudge-not-now = Не сейчас

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } % для вас
recommended-card-forecast = { $percent }%
recommended-card-in-library = В библиотеке

## InlineConfirm

inline-confirm-keep = Оставить

## Tokens

tokens-cube-watched-tracked = Просмотрено, отмечено онлайн
    .inline = просмотрено, отмечено онлайн
tokens-cube-watched-here = Просмотрено здесь, ещё не отмечено
    .inline = просмотрено здесь, ещё не отмечено
tokens-cube-on-disk = На диске
    .inline = на диске
tokens-cube-downloading = Скачивается
    .inline = скачивается
tokens-cube-missing = Не хватает
    .inline = не хватает
tokens-cube-error = Ошибка
    .inline = ошибка
tokens-cube-not-downloaded = Не скачано
    .inline = не скачано
tokens-cube-not-aired = Ещё не вышла
    .inline = ещё не вышла
tokens-cube-available = Есть
    .inline = есть
tokens-cube-picked = Выбрано для загрузки
    .inline = выбрано для загрузки
tokens-cube-already-on-disk = Уже на диске
    .inline = уже на диске
tokens-cube-no-release = Нет релиза
    .inline = нет релиза

## Card

card-not-interested = Не интересно

## PagePlaceholder

page-placeholder-coming-soon = Скоро

## ErrorLine

error-line-hide-details = Скрыть подробности
error-line-details = Подробности
error-line-details-tip = Показать, что пошло не так, дословно, и скопировать
error-line-copied = Скопировано.

## Readouts (apps/linux/src/format.rs): times, countdowns and lengths

fmt-just-now = только что
fmt-now = сейчас
fmt-ago = { $span } назад
fmt-in = через { $span }
fmt-seconds-short = { $n } с
fmt-minutes-short = { $n } мин
fmt-hours-short = { $n } ч
fmt-days-short = { $n } д
fmt-months-short = { $n } мес
fmt-years-short = { $n } г
fmt-years-months-short = { $years } г { $months } мес
# $h, $m and $s arrive as two digits ("04") after the first unit.
fmt-countdown-days = { $d } д { $h } ч { $m } мин
fmt-countdown-hours = { $h } ч { $m } мин
fmt-countdown-days-seconds = { $d } д { $h } ч { $m } мин { $s } с
fmt-countdown-hours-seconds = { $h } ч { $m } мин { $s } с
fmt-countdown-minutes-seconds = { $m } мин { $s } с
fmt-runtime-minutes = { $m } мин
fmt-runtime-hours = { $h } ч
fmt-runtime-hours-minutes = { $h } ч { $m } мин
fmt-episode = Серия
# $size is already written with the language's decimal mark ("1,5")
fmt-size-b = { $size } Б
fmt-size-kb = { $size } КБ
fmt-size-mb = { $size } МБ
fmt-size-gb = { $size } ГБ
fmt-size-tb = { $size } ТБ
fmt-episode-span = Серии с { $first } по { $through }

## apps/linux/src/problem.rs: every call error in a person's words, what happened and what to do

problem-report = Попробуйте снова. Если повторяется, сообщите об этом из журнала.
problem-try-later = Попробуйте позже.
problem-helper-unreachable = Помощник поиска не ответил.
problem-helper-unauthorized = Помощник поиска отклонил вход.
problem-helper-not-running = Помощник поиска не запущен.
problem-helper-missing = Помощник поиска не установлен.
problem-helper-failed = Помощник поиска не смог это сделать.
problem-helper-no-login = Помощник поиска не может войти в торрент-клиент.
problem-helper-no-login-next = Сохраните пароль торрент-клиента в настройках, раздел «Загрузки».
problem-torrent-unreachable = Торрент-клиент не ответил.
problem-torrent-unauthorized = Торрент-клиент отклонил вход.
problem-torrent-not-running = Торрент-клиент не запущен.
problem-torrent-missing = Торрент-клиент не установлен.
problem-torrent-failed = Торрент-клиент не смог это сделать.
problem-downloads-unreachable-next = Проверьте, что он запущен и его адрес верный в настройках, раздел «Загрузки».
problem-downloads-unauthorized-next = Проверьте имя пользователя и пароль в настройках, раздел «Загрузки».
problem-downloads-not-running-next = Запустите его в настройках, раздел «Загрузки».
problem-downloads-missing-next = В настройках, раздел «Загрузки», написано, как его настроить.
problem-trash = { -app-name } не смог переместить файлы в корзину.
problem-trash-next = Проверьте, что файлы ваши и диск не только для чтения.
problem-source-list = Этот список источников не сработал.
problem-source-list-next = Проверьте его адрес в настройках: «Загрузки», «Списки источников».
# $name is a search source or a site, by its own name.
problem-remote-unreachable = { $name } не ответил.
problem-remote-unreachable-next = Возможно, сайт не работает. Попробуйте позже или ищите в другом источнике.
problem-remote-unauthorized = { $name } отклонил переданный ключ.
problem-remote-unauthorized-next = Смените его ключ в настройках: «Загрузки», «Списки источников».
problem-remote-failed = { $name } не смог это сделать.
problem-remote-failed-next = Попробуйте позже или ищите в другом источнике.
# $name is AniList, MyAnimeList, TVmaze and the like.
problem-provider-sign-in = { $name } отклонил вход { -app-name }.
problem-provider-sign-in-next = Подключите его снова в настройках, раздел «Аккаунты».
problem-provider-turned-away = { $name } отказал { -app-name }.
problem-provider-busy = { $name } сейчас перегружен.
problem-provider-busy-next = Подождите минуту и попробуйте снова.
problem-provider-no-answer = { $name } не ответил.
problem-provider-no-answer-next = Проверьте подключение к интернету и попробуйте снова.
problem-provider-trouble = У { $name } сейчас проблемы.
problem-provider-trouble-next = Попробуйте через несколько минут.
problem-provider-rejected = { $name } не принял это.
problem-not-found = { -app-name } больше не может это найти.
problem-not-found-next = Возможно, это удалено. Попробуйте снова, чтобы загрузить страницу заново.
problem-invalid-next = Проверьте и попробуйте снова.
problem-unavailable = Эта папка сейчас недоступна.
problem-unavailable-next = Подключите диск или проверьте папку в настройках, раздел «Библиотека».
problem-not-connected = { $tracker } не подключён.
problem-not-connected-next = Подключите его в настройках, раздел «Аккаунты».
problem-io = { -app-name } не смог прочитать или записать файл.
problem-io-next = Проверьте, что на диске есть место и папка ваша, и попробуйте снова.
problem-storage = { -app-name } не смог сохранить библиотеку.
problem-storage-next = Проверьте, что на диске есть место, и попробуйте снова.
problem-keyring = Системное хранилище паролей не ответило.
problem-keyring-next = Разблокируйте связку ключей рабочего стола и попробуйте снова.
problem-unsupported = { -app-name } пока не умеет этого здесь.
problem-version = Этот бэкап из более новой версии { -app-name }.
problem-version-next = Обновите { -app-name } в настройках, раздел «О программе», и импортируйте снова.
problem-internal = Внутри { -app-name } что-то пошло не так.
problem-refused-hidden = Этот сериал скрыт.
problem-refused-hidden-next = Сначала покажите его на его странице.
problem-refused-no-match = Этот сериал ещё не привязан.
problem-refused-no-match-next = Сначала привяжите его в «О сериалах».
problem-refused-not-newer = Ваш список уже дальше этой серии.
problem-refused-extra = Этот файл доп. материал, а не серия.
problem-refused-unmatched = Этот файл не привязан к серии.
problem-refused-unmatched-next = Исправьте это на странице сериала, в разделе «Правки».
problem-refused-on-disk = Этот файл есть только на этом компьютере.

## apps/linux/src/library_view.rs: the library page's section titles and the one fact under each
## (Dropped, On hold and Watching reuse status-* from core.ftl)

library-view-airing = В эфире
library-view-no-date = Без даты
library-view-type-tv = ТВ
library-view-type-movies = Фильмы
library-view-type-extras = OVA, ONA и спешлы
library-view-type-music = Музыка
library-view-type-unknown = Неизвестно
# The Genre section for shows with no genre yet, most often because they are not matched.
library-view-no-genre = Без привязки
library-view-state-caught-up = Всё просмотрено
library-view-state-not-started = Не начато
# $span is a short countdown: library-view-countdown-*.
library-view-next-episode = Следующая серия через { $span }
library-view-countdown-days = { $d } д { $h } ч
library-view-countdown-hours = { $h } ч { $m } мин
library-view-countdown-minutes = { $m } мин
library-view-in-progress = В процессе: { $count }
library-view-not-started = Не начато: { $count }
library-view-all-watched = Всё просмотрено
library-view-on-disk =
    На диске { $count ->
        [one] { $count } серия
        [few] { $count } серии
        [many] { $count } серий
       *[other] { $count } серии
    }
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } файл ещё на диске
        [few] { $count } файла ещё на диске
        [many] { $count } файлов ещё на диске
       *[other] { $count } файла ещё на диске
    }

## apps/linux/src/dbus/notify.rs: the desktop notification for a new episode
## (its Open button reuses action-open from core.ftl)

# $episode is the episode's number, $title the show's.
notify-episode-out = Вышла серия { $episode } «{ $title }»
notify-click-open = Нажмите, чтобы открыть в { -app-name }
notify-click-get = Нажмите, чтобы скачать серии в { -app-name }
notify-get-episodes = Скачать серии

## apps/linux/src/nowplaying.rs: the second line a media widget shows while an episode plays

# $episode is episode-number from core.ftl ("Episode 12").
now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: why the screen stays on, as the desktop's power panel shows it

inhibit-reason = Воспроизведение серии

## apps/linux/src/bridge/door.rs: why a value a page sent cannot be used (an error's detail line)

door-not-a-choice = { $value } не является допустимым вариантом
door-tokens-op-empty = операции tokens нужны токены
door-not-a-refine-op = { $value } не является операцией уточнения
door-limit-too-low = должно быть не меньше 1, получено { $limit }
door-not-a-torrent-action = { $value } не является действием с торрентом

## apps/linux/src/bridge/shell.rs: the first-run starter folder and the update notice

# $folder is a path, $error the system's own reason.
shell-starter-folder-failed = Не удалось создать { $folder }: { $error }
shell-update-gone = { -app-name } { $version } больше не предлагается

## Watch together (#846)

onboarding-step-together = Вместе
onboarding-title-together = Смотрите с друзьями
onboarding-body-together = Создайте комнату в плеере и отправьте её код. Каждый смотрит свою копию серии, а воспроизведение, пауза и перемотка идут у всех синхронно.
onboarding-ready-together = Совместный просмотр
onboarding-ready-together-as = В комнатах вас видят как { $name }.
onboarding-ready-together-none = Имени пока нет. Панель комнаты спросит его в первый раз.
onboarding-together-you = Вы
onboarding-together-code = Код комнаты
onboarding-together-episode = Frieren · Серия 3
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = Ваше имя в комнатах
rail-room = Совместный просмотр (Ctrl+J)
rail-room-in = В комнате совместного просмотра (Ctrl+J)
player-chrome-watch-together = Смотреть вместе
player-chrome-room = Комната { $code } · { $count ->
    [one] { $count } человек
    [few] { $count } человека
    [many] { $count } человек
   *[other] { $count } человека
}
room-name-empty = Введите имя, которое увидят остальные в комнате.
room-name-bad = Это имя не подходит. Используйте от 1 до 32 символов.
room-error-not-found = Комнаты с таким кодом нет. Уточните его у того, кто прислал; возможно, комната закрылась.
room-error-full = Эта комната заполнена. В комнате до 10 человек.
room-error-unreachable = Не удалось связаться с сервером комнат. Проверьте подключение и попробуйте снова.
room-error-refused = Сервер комнат отклонил запрос. Попробуйте через минуту.
room-error-proto = Эта комната работает на более новой версии комнат. Обновите { -app-name }, чтобы войти.
room-error-not-in-room = Вы больше не в этой комнате. Создайте новую или войдите снова.
room-error-bad-code = Код комнаты состоит из 6 букв и цифр, например K7QM2X.
room-error-bad-relay = Сервер комнат в «Настройки, Воспроизведение» не является веб-адресом. Исправьте его там и попробуйте снова.
room-panel-title = Совместный просмотр
room-panel-close = Закрыть
room-panel-connected = Подключено
room-panel-reconnecting = Переподключение
room-panel-name-label = Ваше имя в комнатах
room-panel-name-placeholder = Ваше имя
room-panel-name-start = Создать комнату
room-panel-name-join = Войти в комнату
room-panel-start = Создать комнату
room-panel-or-join = Или войдите по коду
room-panel-join-label = Код комнаты
room-panel-join = Войти
room-panel-you-are = Вас видят как { $name }
room-panel-change-name = Изменить
room-panel-code-label = Код комнаты
room-panel-copy = Копировать ссылку
room-panel-link-copied = Ссылка на комнату скопирована
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = Этой серии нет в вашей библиотеке, поэтому смотреть вместе пока нельзя.
room-panel-open-episode = Открыть серию
room-panel-people = { $count ->
    [one] { $count } человек
    [few] { $count } человека
    [many] { $count } человек
   *[other] { $count } человека
}
room-panel-host = Ведущий
room-panel-you = Вы
room-panel-ready = Готово
room-panel-opening = Открывает
room-panel-missing = Нет серии
room-panel-offset = Ваш сдвиг
room-panel-offset-earlier = На полсекунды раньше
room-panel-offset-later = На полсекунды позже
room-panel-seconds = { $value } с
room-panel-suggested = По опенингам: { $offset }
room-panel-use-suggested = Применить
room-panel-leave = Выйти из комнаты
player-room-speed = Скорость задаёт комната
settings-find-room-name =
    .words = совместный просмотр вместе комната имя друзья
settings-find-room-relay =
    .words = совместный просмотр вместе комната ретранслятор сервер
key-help-key-ctrl-j = Ctrl+J
key-help-room = Совместный просмотр
settings-playback-together = Совместный просмотр
settings-playback-room-name = Ваше имя в комнатах совместного просмотра
settings-playback-room-name-placeholder = Ваше имя
settings-playback-room-relay = Сервер комнат
settings-playback-room-relay-helper = Здесь создаются комнаты. Меняйте его, только если сервер ваш.
