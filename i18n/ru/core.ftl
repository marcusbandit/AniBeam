# ru translation of en/core.ftl. An ID missing here is shown in English.

## Хост в tailnet

# A person on the tailnet whom Tailscale gives neither a name nor a login.
tailnet-person-no-name = Нет имени в Tailscale

## Счётчики

count-episodes =
    { $count ->
        [one] { $count } серия
        [few] { $count } серии
        [many] { $count } серий
       *[other] { $count } серии
    }
count-shows =
    { $count ->
        [one] { $count } сериал
        [few] { $count } сериала
        [many] { $count } сериалов
       *[other] { $count } сериала
    }

## Общие действия

action-cancel = Отмена
action-close = Закрыть
action-try-again = Повторить
action-remove = Убрать
action-delete = Удалить
action-open = Открыть
action-save = Сохранить
action-apply = Применить
action-clear = Очистить
action-refresh = Обновить
action-play = Смотреть
action-connect = Подключить
action-start = Начать
action-back = Назад
action-see-all = Все
action-open-series = Открыть сериал
action-open-settings = Открыть настройки

## Списки трекера

status-watching = Смотрю
status-completed = Просмотрено
status-paused = Отложено
status-on-hold = Отложено
status-dropped = Брошено
status-planning = В планах
status-rewatching = Пересматриваю

## Серии

episode-number = Серия { $number }

## Слова AniList: формат, статус выхода, связи

anilist-format-tv = ТВ
anilist-format-tv-short = Короткий ТВ
anilist-format-film = Фильм
anilist-format-special = Спешл
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Музыка
anilist-status-finished = Вышел
anilist-status-airing = В эфире
anilist-status-not-yet-aired = Ещё не вышел
anilist-status-cancelled = Отменён
anilist-status-hiatus = На перерыве
anilist-relation-adaptation = Адаптация
anilist-relation-prequel = Приквел
anilist-relation-sequel = Сиквел
anilist-relation-parent = Основная история
anilist-relation-side-story = Побочная история
anilist-relation-character = Персонаж
anilist-relation-summary = Пересказ
anilist-relation-alternative = Альтернатива
anilist-relation-spin-off = Спин-офф
anilist-relation-other = Другое
anilist-relation-source = Источник
anilist-relation-compilation = Компиляция
anilist-relation-contains = Содержит

## Журнал: проблема без собственного текста

event-problem-library = Что-то пошло не так с библиотекой. Подробности в параметрах разработчика.
event-problem-metadata = Что-то пошло не так с данными о сериале. Подробности в параметрах разработчика.
event-problem-trackers = Что-то пошло не так с вашими списками. Подробности в параметрах разработчика.
event-problem-franchise = Что-то пошло не так со связанными сериалами. Подробности в параметрах разработчика.
event-problem-playback = Что-то пошло не так с воспроизведением. Подробности в параметрах разработчика.
event-problem-store = Что-то пошло не так при сохранении данных. Подробности в параметрах разработчика.
event-problem-system = Что-то пошло не так в AniBeam. Подробности в параметрах разработчика.
event-problem-downloads = Что-то пошло не так с загрузками. Подробности в параметрах разработчика.

## Ошибки (contract::CoreError)

core-error-not-found-source = Папка { $id } не найдена.
core-error-not-found-series = Сериал { $id } не найден.
core-error-not-found-file = Файл { $id } не найден.
core-error-not-found-session = Сеанс воспроизведения { $id } не найден.
core-error-not-found-job = Задача { $id } не найдена.
# $field is the setting's own name; $message is already in the reader's language.
core-error-invalid = Неверное значение { $field }: { $message }.
# The same, for a $message that already ends in its own full stop, question mark or ellipsis.
core-error-invalid-ended = Неверное значение { $field }: { $message }
core-error-unavailable = Эта папка недоступна: { $path }
core-error-not-connected = { $tracker } не подключён.
core-error-refused-hidden = Нельзя: сериал скрыт.
core-error-refused-no-match = Нельзя: сериал ещё не привязан; привяжите его в «Данных о сериале».
core-error-refused-not-newer = Нельзя: ваш список уже дальше этой серии.
core-error-refused-extra = Нельзя: это дополнительный материал.
core-error-refused-unmatched = Нельзя: файл не привязан к серии.
core-error-refused-on-disk = Нельзя: файл есть только на диске.
core-error-provider = { $provider } не ответил: { $message }
core-error-io = Не удалось прочитать или записать файл: { $message }
core-error-storage = Не удалось сохранить библиотеку: { $message }
core-error-keyring = Нет доступа к системному хранилищу паролей: { $message }
core-error-unsupported = { $what } пока не поддерживается.
core-error-version = Этот экспорт из более новой версии AniBeam (формат { $found }). Обновите AniBeam, чтобы импортировать его.
core-error-internal = Внутренняя ошибка AniBeam: { $message }
core-error-service = { $service }: { $message }

# What the core says to a person outside downloads, releases, trackers, metadata, the
# library, franchise, images and transfer: jobs, playback, casting, the host, the phone,
# endpoint repos, updates and the rest.

## core/src/repos/mod.rs

core-repos-paste-address = вставьте адрес репозитория
core-repos-must-be-https = адрес должен начинаться с https
core-repos-unreachable = не удалось связаться с репозиторием: { $message }
core-repos-not-found = по этому адресу ничего нет (закрытый репозиторий GitHub тоже отвечает 404)
core-repos-http-status = репозиторий ответил HTTP { $status }
core-repos-unreadable = не удалось прочитать { $path }: { $error }
core-repos-no-such-endpoint = нет установленного источника с таким id
core-repos-key-changed = репозиторий теперь подписан другим ключом, не тем, которому вы доверились. Удалите его и добавьте снова, чтобы принять новый ключ.
core-repos-stale = репозиторий отдал более старый индекс (серия { $serial } после { $kept }); оставлен более новый
core-repos-key-not-shown = ключ репозитория не тот, что вам показали. Добавьте его снова и сравните отпечаток.
core-repos-repo-added = Добавлен список источников загрузки.
core-repos-repo-removed = Удалён список источников загрузки.
# $key is the field's key in quotes.
core-repos-no-field = У { $name } нет поля { $key }
core-repos-needs-field = Для { $name } нужно указать: { $label }
core-repos-host-only = { $name }: { $label } указывается только как хост или хост:порт, например 192.0.2.10:9696
core-repos-not-added = этот репозиторий не добавлен
core-repos-not-offered = { $repo } не предлагает { $id }
core-repos-already-installed = { $name } уже установлен
core-repos-taken-by-you = источник { $id } уже установлен (добавлен вами); сначала удалите его
core-repos-taken-by-repo = источник { $id } уже установлен (из другого репозитория); сначала удалите его
core-repos-source-added = Добавлен источник загрузки.
core-repos-source-updated = Обновлён источник загрузки.
core-repos-source-removed = Удалён источник загрузки.
core-repos-source-changed = Изменён источник загрузки.
core-repos-give-name = дайте ему имя
core-repos-put-query = вставьте {"{"}query{"}"} в адрес там, где идёт поисковый запрос
core-repos-name-taken = источник { $id } уже установлен
core-repos-only-by-hand = изменить можно только источник, добавленный вручную
# $key is a placeholder's name; it stays in braces in the address.
core-repos-keep-key = оставьте {"{"}{ $key }{"}"} в адресе и меняйте значение через «Изменить: { $label }», или удалите сайт и добавьте его заново без этого поля
core-repos-note-kind = anirss не умеет искать тип { $kind }, поэтому компьютер его пропускает; телефон ищет по нему
core-repos-note-by-hand = в anirss есть свой { $id }, написанный вручную; компьютер ищет по нему
core-repos-note-theirs = Этот anirss запущен вами, и компьютер никогда не меняет его конфиг: добавьте { $id } в конфиг этого anirss, чтобы компьютер искал по нему
core-repos-note-sandbox = Этот запуск не пишет в конфиг anirss, поэтому здесь компьютер не ищет по { $id }
core-repos-note-old = anirss { $version } не принимает источники от { -app-name }; обновите его до { $min } или новее, чтобы компьютер искал по нему

## core/src/repos/index.rs

core-repos-key-not-ed25519 = ключ индекса не является ключом ed25519
core-repos-key-not-base64 = ключ индекса не является 32 байтами в base64
core-repos-key-invalid = ключ индекса не является корректным ключом ed25519
core-repos-too-large = файл слишком большой для репозитория
core-repos-not-a-repo = это не репозиторий источников { -app-name }
core-repos-format-version = формат репозитория версии { $found }; эта версия { -app-name } читает версию { $reads }. Обновите { -app-name }.
core-repos-payload-not-base64 = содержимое репозитория не в base64
core-repos-signature-malformed = подпись репозитория повреждена
core-repos-index-unreadable = индекс репозитория не читается: { $error }
core-repos-signature-mismatch = подпись репозитория не совпадает с его содержимым
# $id is the source's id; the rest of these checks name a source by its id first.
core-repos-check-scheme = { $id }: адрес должен начинаться с https://
core-repos-check-scheme-http = { $id }: адрес должен начинаться с http:// или https://
# $key is the field's key in quotes.
core-repos-check-auth-key = { $id }: неверный ключ авторизации { $key }
core-repos-check-host-torznab = { $id }: только адрес torznab может указывать хост как {"{"}{ $key }{"}"}
core-repos-check-host-placeholder = { $id }: подстановка в имени хоста

## core/src/playback/tracks.rs

core-playback-track-number = Дорожка { $number }
# A sidecar subtitle file beside the video; $label is the rest of its name.
core-playback-track-file = { $label } (файл)
# A language and its region or script: "Portuguese (Brazil)", "Chinese (Simplified)".
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = Английский
core-playback-lang-japanese = Японский
core-playback-lang-spanish = Испанский
core-playback-lang-portuguese = Португальский
core-playback-lang-french = Французский
core-playback-lang-german = Немецкий
core-playback-lang-italian = Итальянский
core-playback-lang-russian = Русский
core-playback-lang-chinese = Китайский
core-playback-lang-korean = Корейский
core-playback-lang-arabic = Арабский
core-playback-lang-hindi = Хинди
core-playback-lang-indonesian = Индонезийский
core-playback-lang-malay = Малайский
core-playback-lang-thai = Тайский
core-playback-lang-vietnamese = Вьетнамский
core-playback-lang-polish = Польский
core-playback-lang-turkish = Турецкий
core-playback-lang-dutch = Нидерландский
core-playback-lang-swedish = Шведский
core-playback-lang-norwegian = Норвежский
core-playback-lang-danish = Датский
core-playback-lang-finnish = Финский
core-playback-lang-hebrew = Иврит
core-playback-lang-ukrainian = Украинский
core-playback-lang-czech = Чешский
core-playback-lang-hungarian = Венгерский
core-playback-lang-greek = Греческий
core-playback-lang-romanian = Румынский
core-playback-lang-filipino = Филиппинский
core-playback-region-brazil = Бразилия
core-playback-region-portugal = Португалия
core-playback-region-latin-america = Латинская Америка
core-playback-region-spain = Испания
core-playback-region-simplified = упрощённый
core-playback-region-traditional = традиционный

## contract/src/enums.rs

# What a job is doing, for a status line while it runs. .stopped is the line when it
# ended in a problem.
core-jobs-scan = Проверка папок
    .stopped = Проверка папок остановилась из-за проблемы.
core-jobs-auto-match = Привязка сериалов
    .stopped = Привязка сериалов остановилась из-за проблемы.
core-jobs-search = Поиск сериала
    .stopped = Поиск сериала остановился из-за проблемы.
core-jobs-resolve-link = Чтение ссылки
    .stopped = Чтение ссылки остановилось из-за проблемы.
core-jobs-apply-match = Привязка сериала
    .stopped = Привязка сериала остановилась из-за проблемы.
core-jobs-refresh = Обновление сериала
    .stopped = Обновление сериала остановилось из-за проблемы.
core-jobs-refresh-all = Обновление данных о сериалах
    .stopped = Обновление данных о сериалах остановилось из-за проблемы.
core-jobs-refresh-airing = Проверка того, что в эфире
    .stopped = Проверка того, что в эфире, остановилась из-за проблемы.
core-jobs-refresh-imdb = Обновление рейтингов
    .stopped = Обновление рейтингов остановилось из-за проблемы.
core-jobs-clear-images = Очистка изображений
    .stopped = Очистка изображений остановилась из-за проблемы.
core-jobs-fill-images = Загрузка изображений
    .stopped = Загрузка изображений остановилась из-за проблемы.
core-jobs-connect-tracker = Подключение списка
    .stopped = Подключение списка остановилось из-за проблемы.
core-jobs-mark = Отметка серии просмотренной
    .stopped = Отметка серии просмотренной остановилась из-за проблемы.
core-jobs-set-progress = Сохранение прогресса
    .stopped = Сохранение прогресса остановилось из-за проблемы.
core-jobs-score = Сохранение оценки
    .stopped = Сохранение оценки остановилось из-за проблемы.
core-jobs-refresh-progress = Чтение прогресса из списка
    .stopped = Чтение прогресса из списка остановилось из-за проблемы.
core-jobs-refresh-watching = Чтение списка «Смотрю»
    .stopped = Чтение списка «Смотрю» остановилось из-за проблемы.
core-jobs-refresh-airing-now = Проверка этого сезона
    .stopped = Проверка этого сезона остановилась из-за проблемы.
core-jobs-refresh-recommended = Подбор сериалов для вас
    .stopped = Подбор сериалов для вас остановился из-за проблемы.
core-jobs-read-show-page = Чтение сериала
    .stopped = Чтение сериала остановилось из-за проблемы.
core-jobs-on-hold = Перенос заброшенных сериалов в «Отложено»
    .stopped = Перенос заброшенных сериалов в «Отложено» остановился из-за проблемы.
core-jobs-set-status = Смена статуса в списке
    .stopped = Смена статуса в списке остановилась из-за проблемы.
core-jobs-move-lists = Перенос списка в трекер
    .stopped = Перенос списка в трекер остановился из-за проблемы.
core-jobs-notes = Сохранение заметок
    .stopped = Сохранение заметок остановилось из-за проблемы.
core-jobs-crawl = Поиск связанных сериалов
    .stopped = Поиск связанных сериалов остановился из-за проблемы.
core-jobs-skip-windows = Поиск опенинга и эндинга
    .stopped = Поиск опенинга и эндинга остановился из-за проблемы.
core-jobs-export = Сохранение библиотеки в файл
    .stopped = Сохранение библиотеки в файл остановилось из-за проблемы.
core-jobs-import = Загрузка библиотеки из файла
    .stopped = Загрузка библиотеки из файла остановилась из-за проблемы.
core-jobs-subscriptions = Проверка подписок
    .stopped = Проверка подписок остановилась из-за проблемы.
core-jobs-connect-downloads = Подключение торрент-клиента
    .stopped = Подключение торрент-клиента остановилось из-за проблемы.
core-jobs-search-releases = Поиск серий
    .stopped = Поиск серий остановился из-за проблемы.
core-jobs-refine-releases = Уточнение поиска серий
    .stopped = Уточнение поиска серий остановилось из-за проблемы.
core-jobs-subscribe = Подписка на сериал
    .stopped = Подписка на сериал остановилась из-за проблемы.
core-jobs-download = Добавление загрузок
    .stopped = Добавление загрузок остановилось из-за проблемы.
core-jobs-remove-subscription = Удаление подписки
    .stopped = Удаление подписки остановилось из-за проблемы.
core-jobs-control-torrents = Изменение загрузок
    .stopped = Изменение загрузок остановилось из-за проблемы.
core-jobs-test-torrent-client = Проверка торрент-клиента
    .stopped = Проверка торрент-клиента остановилась из-за проблемы.
core-jobs-remove-show = Удаление сериала
    .stopped = Удаление сериала остановилось из-за проблемы.
core-jobs-repair-subscription = Исправление подписки
    .stopped = Исправление подписки остановилось из-за проблемы.
core-jobs-check-endpoints = Проверка источников загрузки
    .stopped = Проверка источников загрузки остановилась из-за проблемы.
core-jobs-start-endpoint = Запуск источника загрузки
    .stopped = Запуск источника загрузки остановился из-за проблемы.
core-jobs-schedule-agent = Запрос расписания у ИИ-агента
    .stopped = Запрос расписания у ИИ-агента остановился из-за проблемы.

## core/src/airing_now.rs

core-airing-now-now = Сейчас в эфире
core-airing-now-winter = Зима { $year }
core-airing-now-spring = Весна { $year }
core-airing-now-summer = Лето { $year }
core-airing-now-fall = Осень { $year }
core-airing-now-planned = «{ $title }» добавлен в список «В планах» на { $site }.
core-airing-now-planned-both = «{ $title }» добавлен в список «В планах» на { $site } и { $other }.
# $status is the list the show is on: watching, repeating, planning, completed, paused or dropped.
core-airing-now-already-on =
    «{ $title }» уже в вашем списке { $site } со статусом «{ $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }».
core-airing-now-plan-failed = Не удалось добавить «{ $title }» в «В планах». Попробуйте позже.

## core/src/aired.rs

core-aired-episode-out = Вышла серия { $number } сериала «{ $title }».

## core/src/cast/mod.rs

core-cast-connecting = Трансляция на { $device }.
core-cast-ended = Трансляция на { $device } завершена.
core-cast-stopped = Трансляция на { $device } остановлена.
core-cast-failed = Не удалось начать трансляцию на { $device }. Проверьте, что устройство в той же сети.
core-cast-not-built = Трансляции нет в этой сборке
core-cast-serve-failed = не удалось отдать серию: { $error }
core-cast-device-gone = Этого устройства больше нет в сети
core-cast-not-a-playlist = это не адрес плейлиста
core-cast-nothing = Ничего не транслируется
core-cast-still-starting = Устройство ещё запускается
core-cast-no-route = у этого устройства нет маршрута до телевизора
core-cast-no-control = телевизор не сообщил адрес управления
core-cast-not-handed = другой компьютер не передал серию: { $error }
core-cast-empty-playlist = другой компьютер прислал пустой плейлист
# $error is why the TV refused the converted stream, $first why it refused the file.
core-cast-refused-both = { $error }; исходный файл он тоже не принял ({ $first })
core-cast-chromecast-stopped = Chromecast остановился
core-cast-hevc = { $error }. Эта серия в HEVC, который декодируют только новые Chromecast (с Google TV или Ultra)

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = не удалось связаться с Chromecast: { $error }
core-cast-chromecast-broke = соединение с Chromecast оборвалось: { $error }
core-cast-chromecast-no-player = Chromecast не запустил медиаплеер
# $reason is the Chromecast's own word for it, or core-cast-no-reason.
core-cast-chromecast-refused-player = Chromecast отказался запускать медиаплеер ({ $reason })
core-cast-no-reason = без причины
# $kind is the Chromecast's own word for the failure.
core-cast-chromecast-cannot-play = Chromecast не может это воспроизвести ({ $kind })
core-cast-chromecast-error = Chromecast остановился с ошибкой воспроизведения

## core/src/cast/dlna.rs

core-cast-tv-no-answer = телевизор не ответил: { $error }
# $action is the DLNA command; $code the TV's own error.
core-cast-tv-refused = телевизор отклонил { $action }
core-cast-tv-refused-code = телевизор отклонил { $action } ({ $code })

## core/src/cast/media.rs

core-cast-no-video = В этом файле нет видео.
core-cast-hi10p = Эта серия в 10-битном H.264 (Hi10P), который не декодирует ни один телевизор или Chromecast. Для трансляции нужно перекодирование, а его пока нет.
core-cast-video-codec = Видео этой серии ({ $codec }) нельзя транслировать.
core-cast-audio-codec = Аудио этой серии ({ $codec }) нельзя транслировать.

## core/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Начата загрузка { $count } серии «{ $folder }».
        [few] Начата загрузка { $count } серий «{ $folder }».
        [many] Начата загрузка { $count } серий «{ $folder }».
       *[other] Начата загрузка { $count } серии «{ $folder }».
    }
core-phone-subscribed = Подписка на «{ $name }» оформлена.
core-phone-rule-stopped = «{ $name }» не проверяется на новые серии: источник пропал или снова нужен его ключ.
core-phone-new-episode = Скачивается новая серия «{ $name }».
# $name is the source's name in quotes.
core-phone-name-taken = { $name }: у другого источника уже такое имя
core-phone-nothing-to-download = скачивать нечего
# $used and $cap are in GB, with one decimal.
core-phone-over-cap = так { -app-name } превысит лимит памяти (занято или скачивается { $used } из { $cap } ГБ); удалите серии или поднимите лимит
core-phone-empty-search = пустой запрос
core-phone-no-results = ничего не найдено по запросу { $query }
# Ends "... is not supported yet." (core-error-unsupported).
core-phone-own-engine = Собственный торрент-движок

## core/src/phone/check.rs

# The connection check: each line is a step for one host ($label), and what came of it.
core-phone-check-connected = подключено
core-phone-check-refused = отказ: { $error }
core-phone-check-dropped = нет ответа (пакеты отбрасываются)
core-phone-check-finding = { $label }: поиск адреса
core-phone-check-failed = ошибка: { $error }
core-phone-check-no-answer = нет ответа
core-phone-check-private = { $label }: защищённый поиск адреса
core-phone-check-skipped-address = пропущено, это адрес
core-phone-check-skipped-home = пропущено, имя в домашней сети
core-phone-check-no-record = нет публичного адреса, решает DNS этой сети
core-phone-check-no-resolver = ни один DNS-сервер не ответил
core-phone-check-reaching = { $label }: соединение с { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } байт
core-phone-check-setup = настройка

## core/src/playback/session.rs

core-playback-not-saved = Прогресс по «{ $folder }» не сохранён. Сначала привяжите сериал.
# $done is a whole percentage.
core-playback-still-downloading = серия { $number } ещё скачивается, готово { $done }%

## core/src/playback/skip.rs

core-playback-bad-duration = длительность должна быть положительным числом секунд

## core/src/agent.rs

# A coding agent looking up a show's schedule; these lines say what it is doing.
core-agent-no-answer = агент не дал ответа
core-agent-searching = Поиск в интернете: { $query }
core-agent-searching-schedule = Поиск расписания в интернете
core-agent-reading = Чтение { $url }
core-agent-reading-page = Чтение страницы
# $tool is the agent's own name for a tool.
core-agent-using = Использует { $tool }
# Ends "... is not supported yet." (core-error-unsupported).
core-agent-none = ИИ-агент на этом компьютере
# $name is the coding agent's name.
core-agent-ended = { $name } завершился без ответа
core-agent-too-long = { $name } отвечал слишком долго

## core/src/core.rs

# A guest device on another computer's library asks for something that is the owner's.
core-core-guest-trackers = аккаунты трекеров на том компьютере принадлежат его владельцу, поэтому это устройство ничего там не отмечает
core-core-guest-not-interested = «Не интересно» на том компьютере это список его владельца, поэтому это устройство ничего там не меняет
core-core-guest-recommended = рекомендации на том компьютере подобраны по списку его владельца
core-core-nothing-picked = ничего не выбрано

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider } не является источником данных
core-prefs-provider-count = { $provider } указан несколько раз ({ $count }); каждый источник указывается один раз
# A setting out of its bounds: $value is outside $low to $high.
core-prefs-outside = { $value } вне диапазона от { $low } до { $high }
core-prefs-empty = пусто

## core/src/recommended/mod.rs

core-recommended-ranking = Подбор сериалов для вас

## core/src/recommended/sync.rs

core-recommended-finding-people = Поиск людей с похожим вкусом
core-recommended-reading-lists = Чтение их списков
core-recommended-reading-yours = Чтение ваших сериалов
core-recommended-reading-suggestions = Чтение сериалов для рекомендаций

## core/src/recommended/page.rs

core-recommended-no-anime = На { -anilist } нет аниме { $id }

## core/src/host/mod.rs

# The tailnet host's status line in Settings.
core-host-off = Выкл.
core-host-starting = Запуск
# $addresses is a list of address:port, joined with commas.
core-host-serving = Доступно на { $addresses }
core-host-no-tailscale = Tailscale не подключён
core-host-cannot-listen = Не удалось открыть порт
core-host-port-taken = Порт { $port } занят
core-host-cannot-listen-on = Не удалось слушать { $address }: { $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text } не является адресом с портом

## core/src/host/keys.rs

# $key is the text given, in quotes.
core-host-not-a-key = { $key } не является ключом устройства

## core/src/net/limiter.rs

core-net-timed-out = время ожидания истекло через { $ms } мс
core-net-rate-limited = { $provider } ограничил частоту запросов

## core/src/net/mod.rs

core-net-waiting = { $host }: только что не ответил вовремя; { -app-name } ждёт { $seconds } с перед новым запросом

## core/src/net/anilist.rs

core-net-no-data = нет данных

## core/src/update/install.rs

core-update-no-download = Для этого компьютера нет загрузки { -app-name } { $version }
core-update-cannot-make = Не удалось создать { $path }: { $error }
core-update-cannot-mark = Не удалось сделать новый AppImage исполняемым: { $error }
core-update-cannot-replace = Не удалось заменить { $path }: { $error }
core-update-not-here = Эта копия обновляется не здесь
core-update-cannot-start = Не удалось начать загрузку: { $error }
core-update-did-not-start = Загрузка не началась: { $error }
core-update-cannot-write = Не удалось записать { $path }: { $error }
core-update-stopped = Загрузка остановилась: { $error }
core-update-cancelled = Отменено
# $got and $size are in bytes.
core-update-ended-early = Загрузка оборвалась ({ $got } из { $size } байт)
core-update-checksum = Загрузка не совпадает с контрольной суммой GitHub
core-update-cannot-move = Не удалось переместить загрузку на место: { $error }
core-update-cannot-open-dmg = Не удалось открыть dmg: { $error }
core-update-cannot-read-dmg = Не удалось прочитать dmg: { $error }
core-update-no-app = В dmg нет приложения
core-update-cannot-copy = Не удалось скопировать новое приложение рядом с этим: { $error }

## core/src/calendar.rs

# An episode's event in the calendar file.
core-calendar-season-episode = { $title }, сезон { $season }, серия { $number }
core-calendar-episode = { $title }, серия { $number }

## core/src/paths.rs

core-paths-no-home = нет домашней папки

## core/src/first_run.rs

core-first-run-not-absolute = папка с видео должна быть указана абсолютным путём

## core/src/presence/mod.rs

# Discord shows this in place of an episode line while an extra plays.
core-presence-extra = Доп. материал

# The core's words for the library group: trackers, metadata, library, franchise,
# images and transfer (#798).

## core/src/trackers/accounts.rs

# $tracker is the tracker's short name: AniList or MAL.
core-trackers-no-client-id = Для { $tracker } не задан Client ID.
core-trackers-mal-reconnect = Переподключите MyAnimeList в настройках.
core-trackers-mal-wants-secret = MyAnimeList требует Client Secret этого приложения. Добавьте его в настройках в разделе «Трекеры» или зарегистрируйте приложение с App Type «other».
# $error is the parser's own message.
core-trackers-token-reply-unreadable = ответ с токеном не читается: { $error }
core-trackers-token-reply-no-token = в ответе нет токена доступа

## core/src/trackers/mod.rs

core-trackers-no-token = токен доступа не сохранён, переподключитесь в настройках

## core/src/trackers/cache.rs

core-trackers-no-user-id = у аккаунта нет id пользователя

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = не выбраны сериалы для переноса
# $site is the tracker's full name: AniList or MyAnimeList.
core-trackers-moved-up =
    { $count ->
        [one] Перенесён { $count } сериал с этого компьютера в { $site }.
        [few] Перенесено { $count } сериала с этого компьютера в { $site }.
        [many] Перенесено { $count } сериалов с этого компьютера в { $site }.
       *[other] Перенесено { $count } сериала с этого компьютера в { $site }.
    }
core-trackers-moved-up-some-refused =
    Перенесено в { $site }: { $moved ->
        [one] { $moved } сериал
        [few] { $moved } сериала
        [many] { $moved } сериалов
       *[other] { $moved } сериала
    }; { $site } не принял { $failed ->
        [one] { $failed } сериал
        [few] { $failed } сериала
        [many] { $failed } сериалов
       *[other] { $failed } сериала
    }.
# $show is the show's folder name; $error is the tracker's answer, cleaned up.
core-trackers-move-refused = { $site } не принял «{ $show }»: { $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = Время авторизации истекло, попробуйте снова.
core-trackers-oauth-state-mismatch = не совпал state, возможна CSRF-атака, вход прерван.
core-trackers-oauth-no-code = в ответе нет кода
core-trackers-oauth-no-token = в ответе нет access_token
core-trackers-oauth-waiting = Ждём, пока вы войдёте в { $site } в браузере.
# $username is the account's name on the tracker.
core-trackers-oauth-connected = { $site } подключён как { $username }.
core-trackers-oauth-no-name = профиль пришёл без имени
# Shown before a colon and the reason.
core-trackers-oauth-could-not-start = Не удалось начать вход
# $error is the system's own message.
core-trackers-oauth-port-failed = не удалось открыть порт { $port } на этом компьютере: { $error }
core-trackers-oauth-port-busy = порт { $port } на этом компьютере занят другой программой. Подождите немного и попробуйте снова или закройте ту программу, если порт не освобождается.
# $error is the code the tracker sent back.
core-trackers-oauth-provider-error = { $site } вернул ошибку: { $error }
# The browser tab the sign-in leaves. $app is AniBeam's name, in the brand colour in the
# page's body; keep it as a variable.
core-trackers-page-connected = { $site } подключён к { $app }
core-trackers-page-close-tab = Эту вкладку можно закрыть.
core-trackers-page-failed-title = { $app }, вход не удался
core-trackers-page-failed = Вход не завершился
core-trackers-page-go-back = Вернитесь в { $app } и попробуйте снова.
core-trackers-page-finishing-title = { $app }, вход завершается
core-trackers-page-finishing = Вход завершается
core-trackers-page-finishing-body = { $site } ответил, и AniBeam завершает вход. Вернитесь в { $app }: там будет видно, когда всё готово.
core-trackers-page-cancelled-title = { $app }, вход отменён
core-trackers-page-cancelled = Этот вход отменён
core-trackers-page-cancelled-body = Эту вкладку можно закрыть. Если вход был нужен, начните заново из { $app }.

## core/src/trackers/on_hold.rs

# A progress label while the pass runs; $tracker is AniList or MAL.
core-trackers-on-hold-moving = Перенос в «Отложено» на { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] { $count } сериал перенесён в «Отложено» на { $site }.
        [few] { $count } сериала перенесено в «Отложено» на { $site }.
        [many] { $count } сериалов перенесено в «Отложено» на { $site }.
       *[other] { $count } сериала перенесено в «Отложено» на { $site }.
    }
# $second is how many moved on the second tracker, $secondSite.
core-trackers-on-hold-moved-two =
    { $count ->
        [one] В «Отложено» перенесено: { $count } сериал на { $site } и { $second } на { $secondSite }.
        [few] В «Отложено» перенесено: { $count } сериала на { $site } и { $second } на { $secondSite }.
        [many] В «Отложено» перенесено: { $count } сериалов на { $site } и { $second } на { $secondSite }.
       *[other] В «Отложено» перенесено: { $count } сериала на { $site } и { $second } на { $secondSite }.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } сериал не удалось перенести в «Отложено».
        [few] { $failed } сериала не удалось перенести в «Отложено».
        [many] { $failed } сериалов не удалось перенести в «Отложено».
       *[other] { $failed } сериала не удалось перенести в «Отложено».
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] { $count } сериал перенесён в «Отложено» на { $site }.
        [few] { $count } сериала перенесено в «Отложено» на { $site }.
        [many] { $count } сериалов перенесено в «Отложено» на { $site }.
       *[other] { $count } сериала перенесено в «Отложено» на { $site }.
    } { $failed ->
        [one] { $failed } сериал не удалось перенести в «Отложено».
        [few] { $failed } сериала не удалось перенести в «Отложено».
        [many] { $failed } сериалов не удалось перенести в «Отложено».
       *[other] { $failed } сериала не удалось перенести в «Отложено».
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] В «Отложено» перенесено: { $count } сериал на { $site } и { $second } на { $secondSite }.
        [few] В «Отложено» перенесено: { $count } сериала на { $site } и { $second } на { $secondSite }.
        [many] В «Отложено» перенесено: { $count } сериалов на { $site } и { $second } на { $secondSite }.
       *[other] В «Отложено» перенесено: { $count } сериала на { $site } и { $second } на { $secondSite }.
    } { $failed ->
        [one] { $failed } сериал не удалось перенести в «Отложено».
        [few] { $failed } сериала не удалось перенести в «Отложено».
        [many] { $failed } сериалов не удалось перенести в «Отложено».
       *[other] { $failed } сериала не удалось перенести в «Отложено».
    }
# A show's name when its row is gone.
core-trackers-series-fallback = сериал { $id }
# $name is the show's folder name.
core-trackers-back-on-watching = «{ $name }» снова в «Смотрю» на { $site }.
core-trackers-back-on-watching-two = «{ $name }» снова в «Смотрю» на { $site } и { $secondSite }.
core-trackers-back-on-watching-failed = Не удалось вернуть «{ $name }» в «Смотрю». Попробуйте позже.

## core/src/trackers/writes.rs

# A tracker's failure in words a person can act on. $tracker is AniList or MAL.
core-trackers-rate-limited = { $tracker } ограничил частоту запросов, попробуйте через минуту.
core-trackers-auth-expired = Вход в { $tracker } истёк, переподключитесь в настройках.
core-trackers-entry-not-found = Запись в { $tracker } не найдена.
# $status is the HTTP status code.
core-trackers-server-error = Ошибка сервера { $tracker } ({ $status }), попробуйте позже.
core-trackers-error-see-log = Ошибка { $tracker }, подробности в журнале.
# $episode is the number asked for, as it was typed.
core-trackers-episode-not-whole = { $episode } не является целым номером серии больше нуля
core-trackers-score-out-of-range = { $score } вне диапазона от 0 до 10
core-trackers-rewatch-not-settable = пересмотр нельзя выставить вручную
core-trackers-notes-too-long = заметки ограничены { $max } символами
core-trackers-nothing-picked = ничего не выбрано
core-trackers-notes-not-on-list = Сериала ещё нет в вашем списке { $tracker }, сначала добавьте его в список.
# Where a write landed, picked by $sites: anilist, mal or both. Used inside the lines below.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } и { -mal }
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
       *[repeating] Пересматриваю
    }
# $show is the show's folder name.
core-trackers-marked = Серия { $episode } «{ $show }» отмечена на { core-trackers-sites }.
core-trackers-progress-set = Прогресс «{ $show }» на { core-trackers-sites }: { $progress }.
core-trackers-scored = Оценка «{ $show }» на { core-trackers-sites }: { $score }.
core-trackers-score-cleared = Оценка «{ $show }» на { core-trackers-sites } удалена.
core-trackers-status-set = «{ $show }» на { core-trackers-sites }: статус «{ core-trackers-status }».
core-trackers-notes-saved = Заметки к «{ $show }» сохранены на { core-trackers-sites }.
core-trackers-notes-cleared = Заметки к «{ $show }» на { core-trackers-sites } удалены.
# $site is AniList or MyAnimeList.
core-trackers-update-failed = Не удалось обновить { $site }. Попробуйте позже.
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
        [yes] «{ $name }»
       *[no] { $done ->
            [one] { $done } сериал
            [few] { $done } сериала
            [many] { $done } сериалов
           *[other] { $done } сериала
        }
    }
core-trackers-written-done =
    { $action ->
        [clear] Оценка удалена на { core-trackers-sites }: { core-trackers-written-who }.
        [status] Статус «{ core-trackers-status }» на { core-trackers-sites }: { core-trackers-written-who }.
       *[score] Оценка { $score } на { core-trackers-sites }: { core-trackers-written-who }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] Не удалось обновить { $failed } сериал: { $reason }.
            [few] Не удалось обновить { $failed } сериала: { $reason }.
            [many] Не удалось обновить { $failed } сериалов: { $reason }.
           *[other] Не удалось обновить { $failed } сериала: { $reason }.
        }
       *[no] { $failed ->
            [one] Не удалось обновить { $failed } сериал.
            [few] Не удалось обновить { $failed } сериала.
            [many] Не удалось обновить { $failed } сериалов.
           *[other] Не удалось обновить { $failed } сериала.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] Не обновлено: { $failed }, { $reason }.
       *[no] Не обновлено: { $failed }.
    }
# The shows a write to many left out, one sentence per reason, each after a space.
# Keep each [0] first: it leaves the sentence out.
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } сериал не привязан к трекеру.
        [few] {" "}{ $noMatch } сериала не привязаны к трекеру.
        [many] {" "}{ $noMatch } сериалов не привязаны к трекеру.
       *[other] {" "}{ $noMatch } сериала не привязаны к трекеру.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } сериал скрыт, поэтому его синхронизация выключена.
        [few] {" "}{ $hidden } сериала скрыты, поэтому их синхронизация выключена.
        [many] {" "}{ $hidden } сериалов скрыты, поэтому их синхронизация выключена.
       *[other] {" "}{ $hidden } сериала скрыты, поэтому их синхронизация выключена.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } сериал есть только на неподключённом трекере.
        [few] {" "}{ $notConnected } сериала есть только на неподключённом трекере.
        [many] {" "}{ $notConnected } сериалов есть только на неподключённом трекере.
       *[other] {" "}{ $notConnected } сериала есть только на неподключённом трекере.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } сериал хранится на этом компьютере, трекера для отправки нет.
        [few] {" "}{ $kept } сериала хранятся на этом компьютере, трекера для отправки нет.
        [many] {" "}{ $kept } сериалов хранятся на этом компьютере, трекера для отправки нет.
       *[other] {" "}{ $kept } сериала хранятся на этом компьютере, трекера для отправки нет.
    }

## core/src/metadata/automatch.rs

# $show is the show's folder name; $title is the title AniList or TVmaze gives it.
core-metadata-matched-to = «{ $show }» привязан к «{ $title }».
core-metadata-auto-matched =
    { $count ->
        [one] Привязан { $count } сериал.
        [few] Привязано { $count } сериала.
        [many] Привязано { $count } сериалов.
       *[other] Привязано { $count } сериала.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } сериал ещё ждёт привязки.
        [few] { $count } сериала ещё ждут привязки.
        [many] { $count } сериалов ещё ждут привязки.
       *[other] { $count } сериала ещё ждут привязки.
    }
core-metadata-auto-matched-some =
    Привязано: { $matched ->
        [one] { $matched } сериал
        [few] { $matched } сериала
        [many] { $matched } сериалов
       *[other] { $matched } сериала
    }; ещё { $unmatched ->
        [one] ждёт
       *[other] ждут
    } привязки: { $unmatched }.
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
        [yes] «{ $hitFolder }» привязан к «{ $hitTitle }».
       *[no] { $matched ->
            [one] Привязан { $matched } сериал.
            [few] Привязано { $matched } сериала.
            [many] Привязано { $matched } сериалов.
           *[other] Привязано { $matched } сериала.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] Для «{ $missName }» совпадение не найдено.
       *[no] { $unmatched ->
            [one] Для { $unmatched } сериала совпадение не найдено.
            [few] Для { $unmatched } сериалов совпадение не найдено.
            [many] Для { $unmatched } сериалов совпадение не найдено.
           *[other] Для { $unmatched } сериалов совпадение не найдено.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] { $failed } сериал не удалось найти; попробуйте позже.
        [few] { $failed } сериала не удалось найти; попробуйте позже.
        [many] { $failed } сериалов не удалось найти; попробуйте позже.
       *[other] { $failed } сериала не удалось найти; попробуйте позже.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } сериал уже был привязан.
        [few] { $had } сериала уже были привязаны.
        [many] { $had } сериалов уже были привязаны.
       *[other] { $had } сериала уже были привязаны.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } сериал уже привязывается.
        [few] { $busy } сериала уже привязываются.
        [many] { $busy } сериалов уже привязываются.
       *[other] { $busy } сериала уже привязываются.
    }

## core/src/metadata/apply.rs

core-metadata-link-unreadable = Не удалось прочитать ссылку. Вставьте адрес страницы AniList, MyAnimeList, TVmaze или IMDb.
core-metadata-no-mal-entry = На AniList нет записи для этого id MyAnimeList.
core-metadata-no-imdb-show = На TVmaze нет сериала для этого тайтла IMDb.
# Shown before "is not supported yet". $provider is the provider's code: anilist, mal, tmdb.
core-metadata-search-on = Поиск в { $provider }
core-metadata-query-too-short = Для поиска введите хотя бы два символа.
core-metadata-not-a-link = это не ссылка
core-metadata-refresh-unsupported = Обновление привязки только к MAL или TMDB
# $show is the show's folder name.
core-metadata-matched = «{ $show }» привязан.
core-metadata-updated = Данные о «{ $show }» обновлены.
core-metadata-update-failed = Не удалось обновить данные о «{ $show }». Попробуйте позже.
# Progress labels while the whole library is updated, or shows only seen in passing
# are filled in.
core-metadata-progress-refresh = обновление
core-metadata-progress-backfill = дозагрузка
core-metadata-updated-many =
    { $count ->
        [one] Обновлены данные о { $count } сериале.
        [few] Обновлены данные о { $count } сериалах.
        [many] Обновлены данные о { $count } сериалах.
       *[other] Обновлены данные о { $count } сериалах.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] Не удалось обновить данные о { $count } сериале. Попробуйте позже.
        [few] Не удалось обновить данные о { $count } сериалах. Попробуйте позже.
        [many] Не удалось обновить данные о { $count } сериалах. Попробуйте позже.
       *[other] Не удалось обновить данные о { $count } сериалах. Попробуйте позже.
    }
core-metadata-updated-some =
    { $count ->
        [one] Обновлены данные о { $count } сериале; не удалось обновить: { $failed }.
        [few] Обновлены данные о { $count } сериалах; не удалось обновить: { $failed }.
        [many] Обновлены данные о { $count } сериалах; не удалось обновить: { $failed }.
       *[other] Обновлены данные о { $count } сериалах; не удалось обновить: { $failed }.
    }

## core/src/metadata/fetch.rs

# $id is the provider's own number for the show.
core-metadata-no-such-show = На TVmaze нет сериала { $id }
core-metadata-no-such-media = На AniList нет записи { $id }

## core/src/metadata/imdb.rs

# Progress labels while the ratings update.
core-metadata-progress-imdb-links = ссылки IMDb из Wikidata
core-metadata-progress-imdb-ratings = рейтинги IMDb

## core/src/franchise/layout.rs

# The words under a related show's title when the two share characters.
core-franchise-relation-character = Общие персонажи

## core/src/images/mod.rs

# A progress label while covers and banners download.
core-images-progress-fetching = загрузка изображений

## core/src/transfer/export.rs

# $folder is the name of the folder the file went to.
core-transfer-exported =
    { $count ->
        [one] { $count } сериал сохранён в { $folder }.
        [few] { $count } сериала сохранено в { $folder }.
        [many] { $count } сериалов сохранено в { $folder }.
       *[other] { $count } сериала сохранено в { $folder }.
    }

## core/src/transfer/format.rs

# $error is the JSON parser's own message.
core-transfer-not-json = это не JSON: { $error }
# $format is the file format's name, anibeam-export.
core-transfer-wrong-format = это не документ { $format }

## core/src/transfer/import.rs

core-transfer-imported = Библиотека загружена из файла.

## core/src/library/delete.rs

# $id is the file's number in the library.
core-library-no-such-file = в библиотеке нет файла { $id }
core-library-file-elsewhere-not-removed = файл { $id } относится к другому сериалу, поэтому ничего не удалено
core-library-file-outside = { $path } не является файлом в папке библиотеки, поэтому ничего не удалено
# $place is deleted where the system has no trash, trash otherwise. $title is the show's
# folder name.
core-library-episodes-removed =
    «{ $title }»: { $count ->
        [one] { $count } серия
        [few] { $count } серии
        [many] { $count } серий
       *[other] { $count } серии
    } { $place ->
        [deleted] удалено
       *[trash] в корзине
    }.
core-library-finished-removed =
    { $count ->
        [one] { $count } просмотренный сериал
        [few] { $count } просмотренных сериала
        [many] { $count } просмотренных сериалов
       *[other] { $count } просмотренных сериала
    }: { $place ->
        [deleted] удалено
       *[trash] перемещено в корзину
    }.
# $series is how many shows the episodes came from.
core-library-watched-removed =
    { $count ->
        [one] { $count } просмотренная серия
        [few] { $count } просмотренные серии
        [many] { $count } просмотренных серий
       *[other] { $count } просмотренной серии
    } из { $series ->
        [one] { $series } сериала
       *[other] { $series } сериалов
    }: { $place ->
        [deleted] удалено
       *[trash] перемещено в корзину
    }.

## core/src/library/detail.rs

# A fix the series page offers. $read is episode, with its $number, or label, with the
# name the file was read by in $label. $what is film or episode.
core-library-fix-one-file =
    Единственный файл здесь читается как { $read ->
        [episode] серия { $number }
       *[label] { $label }
    }. Похоже, это { $what ->
        [film] фильм
       *[episode] его единственная серия
    }.
# $numbers is a run of episode numbers, as core-library-runs-span or -list writes it;
# $episodes is how many there are.
core-library-fix-specials =
    { $count ->
        [one] { $count } спешл похож
        [few] { $count } спешла похожи
        [many] { $count } спешлов похожи
       *[other] { $count } спешла похожи
    } на { $episodes ->
        [one] недостающую серию
       *[other] недостающие серии
    } { $numbers }.
core-library-fix-numbered-on = Серии { $from } похожи на серии { $to } этой записи, пронумерованные дальше с прошлого сезона.
# Episode numbers in a row: "1 to 6".
core-library-runs-span = с { $first } по { $last }
# Episode numbers apart: "2, 4 and 7". $init is every number but the last, comma-separated.
core-library-runs-list = { $init } и { $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = номер серии от 0 до 10000
core-library-fix-season-range = номер сезона не больше 200
core-library-fix-extra-range = номер доп. материала не больше 10000
core-library-file-elsewhere-not-fixed = файл { $id } относится к другому сериалу, поэтому ничего не исправлено
# $title is the show's title.
core-library-fix-undone =
    { $count ->
        [one] «{ $title }»: { $count } файл снова читается по имени.
        [few] «{ $title }»: { $count } файла снова читаются по имени.
        [many] «{ $title }»: { $count } файлов снова читаются по имени.
       *[other] «{ $title }»: { $count } файла снова читаются по имени.
    }
core-library-fixed =
    { $count ->
        [one] «{ $title }»: исправлено чтение { $count } файла.
        [few] «{ $title }»: исправлено чтение { $count } файлов.
        [many] «{ $title }»: исправлено чтение { $count } файлов.
       *[other] «{ $title }»: исправлено чтение { $count } файла.
    }

## core/src/library/reads.rs

# $title is the show's title. The -many lines are for any count but one.
core-library-hid-one = «{ $title }» скрыт.
core-library-back-one = «{ $title }» снова виден.
core-library-hid-many = Скрыто сериалов: { $count }.
core-library-back-many = Снова видно сериалов: { $count }.
core-library-episode-count-set = Число серий «{ $title }»: { $count }.
core-library-episode-count-cleared = Число серий «{ $title }» сброшено.

## core/src/library/schedule.rs

core-library-schedule-cleared = Заметки о расписании «{ $title }» удалены.
core-library-schedule-saved = Сохранено заметок о расписании «{ $title }»: { $count }.

## core/src/library/scan.rs

core-library-source-not-absolute = путь к источнику должен быть абсолютным
core-library-source-exists = уже есть такой источник
core-library-source-nested = находится внутри уже добавленного источника
# $folder is the last part of the folder's path.
core-library-source-added = Папка { $folder } добавлена в библиотеку.
core-library-source-removed = Папка { $folder } убрана из библиотеки.
core-library-source-removed-with-shows =
    { $count ->
        [one] Папка { $folder } и её { $count } сериал убраны из библиотеки.
        [few] Папка { $folder } и её { $count } сериала убраны из библиотеки.
        [many] Папка { $folder } и её { $count } сериалов убраны из библиотеки.
       *[other] Папка { $folder } и её { $count } сериала убраны из библиотеки.
    }
# $name is the show's folder name.
core-library-series-forgotten = «{ $name }» убран из библиотеки.
core-library-source-back = Папка { $folder } снова доступна.
core-library-source-unavailable = Папка { $folder } недоступна. Проверьте, что диск подключён.
core-library-source-unreadable = Не удалось прочитать папку { $folder }. Проверьте, что диск подключён.
core-library-source-looks-empty = Папка { $folder } выглядит пустой, поэтому сериалы оставлены как были.
core-library-source-reconcile-failed = Не удалось обновить сериалы в { $folder }.
# What a scan found: the parts below that are not nought, in order, the ones after the
# first set off by a comma. $first names the part that leads.
core-library-scan-done =
    Папки проверены: { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } новый сериал
            [few] { $added } новых сериала
            [many] { $added } новых сериалов
           *[other] { $added } новых сериала
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } сериал изменён
            [few] { $changed } сериала изменено
            [many] { $changed } сериалов изменено
           *[other] { $changed } сериала изменено
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }{ $removed } пропало
    }.

## core/src/library/report.rs

# Why a coding agent wrote no triage for a quick fix report. $span is
# core-library-span-minutes or -seconds.
core-library-agent-no-answer = нет ответа за { $span }
core-library-agent-stopped-code = он остановился с кодом { $code }
core-library-agent-stopped = он остановился
core-library-agent-answered-nothing = он ничего не ответил
core-library-span-minutes =
    { $count ->
        [one] { $count } минуту
        [few] { $count } минуты
        [many] { $count } минут
       *[other] { $count } минуты
    }
core-library-span-seconds =
    { $count ->
        [one] { $count } секунду
        [few] { $count } секунды
        [many] { $count } секунд
       *[other] { $count } секунды
    }
core-library-report-no-fix = ни у одного из этих файлов нет правки для отчёта
# $agent is the agent's name, such as Claude Code; $why is one of the reasons above, or
# the agent's own last words.
core-library-agent-no-triage = { $agent } не написал разбор: { $why }
# Progress labels while a file is loaded, one per part of the library.
core-transfer-progress-sources = источники
core-transfer-progress-series = сериалы
core-transfer-progress-history = история
core-transfer-progress-accounts = аккаунты
core-transfer-progress-preferences = настройки
core-transfer-progress-done = готово

## core/src/downloads/anirss.rs

# $message is what the connection said
core-downloads-anirss-no-answer = anirss не ответил: { $message }
# $status is the HTTP status code, $text the body anirss sent
core-downloads-anirss-answered = anirss ответил { $status }: { $text }
core-downloads-no-free-port = нет свободного порта: { $error }
# $status is how the process ended, as the system words it
core-downloads-anirss-exited = anirss завершился ({ $status })
# $said is the last lines anirss printed
core-downloads-anirss-exited-said = anirss завершился ({ $status }): { $said }
core-downloads-anirss-slow-start = anirss не запустился за { $seconds } с
core-downloads-anirss-slow-start-said = anirss не запустился за { $seconds } с: { $said }
# $command is a command to type, kept as it is
core-downloads-anirss-install-mac = Установите anirss { $version } или новее командой: { $command }
core-downloads-anirss-install-linux = Установите anirss { $version } или новее: пакет anirss в AUR или { $command }
# $update is core-downloads-anirss-update-mac or -linux
core-downloads-anirss-too-old = anirss { $version } слишком старый для AniBeam, нужен { $minimum } или новее. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } не умеет хранить вход каждого торрент-клиента отдельно, а AniBeam это нужно для смены клиента; anirss { $minimum } и новее умеет. { $update }.
core-downloads-anirss-update-mac = Обновите его командой: { $command }
core-downloads-anirss-update-linux = Обновите пакет anirss или выполните: { $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = торрент-файл не получен: { $message }
core-downloads-torrent-file-answered = на запрос торрент-файла пришёл ответ { $status }

## core/src/downloads/feed.rs

core-downloads-feed-answered = лента ответила { $status }
core-downloads-not-a-feed = это не лента: { $error }

## core/src/downloads/jackett.rs

# $program is the command that was not found
core-downloads-jackett-missing = Jackett не установлен: { $program } не найден. Установите его или измените команду в настройках.
# $name is the endpoint, $why what went wrong
core-downloads-endpoint-start-failed = Не удалось запустить { $name }: { $why }
core-downloads-endpoint-start-timeout = { $name } запустился, но не ответил за { $seconds } с.
core-downloads-command-exited = команда завершилась с кодом { $status }

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirss остановился
# $install is core-downloads-anirss-install-mac or -linux
core-downloads-anirss-missing = { $install }. Или укажите в настройках уже запущенный сервер.
# $command is the anirss command set in Settings
core-downloads-anirss-command-missing = { $command } не найден. { $install }. Или укажите в настройках уже запущенный сервер.
core-downloads-anirss-start-failed = Не удалось запустить { $command }: { $error }
# $tooOld is core-downloads-anirss-too-old; $message is what anirss said
core-downloads-anirss-too-old-said = { $tooOld } Его ответ: { $message }
core-downloads-no-torrent-client = По адресу { $address } не отвечает qBittorrent, а своего у AniBeam нет. Установите qBittorrent и включите его веб-интерфейс или установите qbittorrent-nox, и AniBeam будет запускать его сам
core-downloads-own-client-gone = qBittorrent от AniBeam пропал: qbittorrent-nox не установлен. Переустановите AniBeam или укажите в настройках свой qBittorrent
core-downloads-no-port = в { $address } нет порта
core-downloads-own-client-settings = не удалось записать настройки qBittorrent: { $error }
core-downloads-own-client-start-failed = Не удалось запустить { $command }: { $error }
core-downloads-own-client-slow-start = qBittorrent от AniBeam не запустился
# $said is the last lines of its log
core-downloads-own-client-slow-start-said = qBittorrent от AniBeam не запустился: { $said }
core-downloads-not-ready = Загрузки не готовы. Проверьте раздел «Загрузки» в настройках.
core-downloads-ready = Загрузки готовы.
# $count is how many sources anirss's own config names
core-downloads-helper-sources =
    { $count ->
        [one] Теперь загрузки ищет сам AniBeam, и он не читает { $count } источник из конфига помощника поиска. Добавьте их в настройках, раздел «Загрузки», или выберите там помощника поиска.
        [few] Теперь загрузки ищет сам AniBeam, и он не читает { $count } источника из конфига помощника поиска. Добавьте их в настройках, раздел «Загрузки», или выберите там помощника поиска.
        [many] Теперь загрузки ищет сам AniBeam, и он не читает { $count } источников из конфига помощника поиска. Добавьте их в настройках, раздел «Загрузки», или выберите там помощника поиска.
       *[other] Теперь загрузки ищет сам AniBeam, и он не читает { $count } источника из конфига помощника поиска. Добавьте их в настройках, раздел «Загрузки», или выберите там помощника поиска.
    }
core-downloads-client-connected = Торрент-клиент подключён.
core-downloads-client-not-connected = Торрент-клиент не подключён. Проверьте его адрес в настройках.
core-downloads-not-on-demand = { $name } не является источником, запускаемым по запросу
core-downloads-source-running = Источник загрузки работает.
core-downloads-source-not-running = { $name } не запущен. Включите его в источниках и подтвердите запуск.
core-downloads-empty-query = введите, что искать
# What a search's progress line names when it was given no source
core-downloads-first-endpoint = первый источник
# A search's progress line. The desktop's search card compares the line with these, so
# keep the name in core-downloads-search-starting-endpoint a whole placeable.
core-downloads-search-starting = Запуск поиска
core-downloads-search-starting-endpoint = Запуск: { $endpoint }
core-downloads-search-searching-endpoint = Поиск: { $endpoint }
core-downloads-search-reading = Чтение результатов
# $name is the subscription's name
core-downloads-subscribed = Подписка на «{ $name }» оформлена.
core-downloads-client-no-rule = В qBittorrent нет правила «{ $name }»
# A progress line while a subscription's feed is narrowed; $search is the search tried
core-downloads-trying-search = Пробуем «{ $search }»
core-downloads-nothing-selected = ничего не выбрано для загрузки
core-downloads-added =
    { $count ->
        [one] Добавлена { $count } загрузка.
        [few] Добавлено { $count } загрузки.
        [many] Добавлено { $count } загрузок.
       *[other] Добавлено { $count } загрузки.
    }
core-downloads-movie-link = ссылка на фильм должна быть magnet или адресом http(s)
core-downloads-added-movie = Фильм добавлен в загрузки.
core-downloads-unsubscribed = Подписка на «{ $name }» отменена.
# $folder is a path, $name the library show that lives there
core-downloads-library-folder = { $folder } это папка библиотеки сериала «{ $name }», и в ней есть серии, скачанные не этой подпиской: удалите подписку без файлов или удалите «{ $name }» на его странице
core-downloads-no-torrents-named = не указаны торренты
core-downloads-paused =
    { $count ->
        [one] Приостановлена { $count } загрузка.
        [few] Приостановлено { $count } загрузки.
        [many] Приостановлено { $count } загрузок.
       *[other] Приостановлено { $count } загрузки.
    }
core-downloads-resumed =
    { $count ->
        [one] Возобновлена { $count } загрузка.
        [few] Возобновлено { $count } загрузки.
        [many] Возобновлено { $count } загрузок.
       *[other] Возобновлено { $count } загрузки.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] Удалена { $count } загрузка вместе с файлами.
        [few] Удалено { $count } загрузки вместе с файлами.
        [many] Удалено { $count } загрузок вместе с файлами.
       *[other] Удалено { $count } загрузки вместе с файлами.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] Из списка убрана { $count } загрузка.
        [few] Из списка убрано { $count } загрузки.
        [many] Из списка убрано { $count } загрузок.
       *[other] Из списка убрано { $count } загрузки.
    }
core-downloads-nothing-to-remove = убирать нечего
# $title is the show's title
core-downloads-removed-show = «{ $title }» убран из загрузок.
core-downloads-removed-the-show = Сериал убран из загрузок.
core-downloads-deleted-still-listed = Удалённые файлы всё ещё числятся в торрент-клиенте.
core-downloads-feed-url = лента указывается адресом http(s)
core-downloads-rule-needs-name = у правила должно быть имя
core-downloads-no-rule = нет правила «{ $name }»
core-downloads-feed-no-search = в ленте нет поискового запроса, который можно сократить
core-downloads-no-repair = «{ $name }» не нужно исправлять.
core-downloads-repaired = Подписка на «{ $name }» исправлена.
# $search is the shortest search tried
core-downloads-repair-failed = более короткий запрос ничего не находит; даже «{ $search }» ничего не даёт
# $id is the client's id, in quotes
core-downloads-no-saved-client = нет сохранённого торрент-клиента { $id }
core-downloads-client-switched = Загрузки теперь идут через другой торрент-клиент.
# After the name of anirss or qBittorrent and a colon
core-downloads-host-not-running = не запущен на хосте
core-downloads-host-late = не ответил вовремя

## core/src/downloads/native.rs

core-downloads-native-empty-query = пустой запрос
# $query is the search, in quotes
core-downloads-native-no-results = ничего не найдено по запросу { $query }
# $name is the feed's name, in quotes
core-downloads-native-feed-taken = лента { $name } уже читает другой адрес
core-downloads-native-no-items = ничего не передано
# $link is the link given, in quotes
core-downloads-native-bad-link = ссылки должны быть magnet: или адресами http(s), получено { $link }
core-downloads-native-no-links = ссылки не переданы
core-downloads-native-no-names = не переданы имена подписок
core-downloads-native-files-elsewhere = торрент-клиент хранит файлы на другой машине, поэтому их можно удалить только вместе с торрентами
# $names is a list of feed names
core-downloads-native-unknown-feeds = неизвестные ленты: { $names }
# $paths is a list of folders
core-downloads-native-outside-folders = отказ удалять файлы вне папок загрузки: { $paths }
# Removals refused; $path is the folder or file
core-downloads-refuse-home-folder = { $path } это домашняя папка или содержит её, поэтому ничего не удалено
core-downloads-refuse-library-folder = { $path } это папка библиотеки или содержит её, поэтому ничего не удалено

## core/src/downloads/paths.rs

core-downloads-no-default-folder = в qBittorrent не задана папка сохранения по умолчанию; задайте её в qBittorrent
core-downloads-name-empty = имя не может быть пустым
core-downloads-name-slash = имя не должно содержать / или \
# $name is the name given, in quotes
core-downloads-name-not-folder = { $name } не подходит как имя папки

## core/src/downloads/prune.rs

# $title is the show's title
core-downloads-pruned = Подписка на «{ $title }» отменена: сериал закончил выходить.
core-downloads-prune-failed = Не удалось отписаться от «{ $title }». Проверьте раздел «Загрузки» в настройках.

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path } не является простым абсолютным путём, поэтому ничего не удалено
core-downloads-refuse-root = { $path } это корень файловой системы, поэтому ничего не удалено
core-downloads-refuse-outside-library = { $path } не находится в папке библиотеки, поэтому ничего не удалено
core-downloads-refuse-save-folder = { $path } это папка, куда сохранён торрент, а не то, что он скачал, поэтому ничего не удалено

## core/src/downloads/watch.rs

# $episode is the episode's number, $title the show's
core-downloads-episode-missed = Серия { $episode } «{ $title }» не появилась через три дня после выхода. Лента продолжает её искать.

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = команда anirss не может быть пустой
core-downloads-settings-anirss-address = адрес anirss должен начинаться с http:// или https://
# $client is the torrent client's name, such as qBittorrent
core-downloads-settings-client-address = адрес { $client } должен начинаться с http:// или https://
# $id is the id given, in quotes
core-downloads-settings-bad-id = { $id } не подходит как id клиента: используйте буквы, цифры, - и _
core-downloads-settings-same-id = у двух клиентов одинаковый id { $id }
# $name is a saved client's name
core-downloads-settings-saved-address = адрес { $name } должен начинаться с http:// или https://
core-downloads-settings-same-name = два клиента называются { $name }; дайте каждому своё имя

## core/src/downloads/qbt.rs

# $address is where qBittorrent was asked, $message what the connection said
core-downloads-qbt-no-answer = qBittorrent не ответил по адресу { $address }: { $message }
core-downloads-qbt-password-rejected = qBittorrent не принял пароль; исправьте его в настройках
core-downloads-qbt-banned = qBittorrent заблокировал этот адрес после неудачных входов; подождите или перезапустите qBittorrent
core-downloads-qbt-login-rejected = qBittorrent не принял имя пользователя или пароль
# $status is the HTTP status code
core-downloads-qbt-login-failed = вход в qBittorrent по адресу { $address } ответил { $status }; повтор при следующем запросе
core-downloads-qbt-not-web-ui = { $address } ответил { $status }, но это не веб-интерфейс qBittorrent. Проверьте адрес в настройках и что веб-интерфейс включён в qBittorrent
# $said is what qBittorrent answered, in quotes
core-downloads-qbt-not-added = qBittorrent не добавил торренты: { $said }
core-downloads-qbt-some-not-added = qBittorrent не смог добавить торренты ({ $count }): { $said }
# $line is the first line of what qBittorrent answered
core-downloads-qbt-answered = qBittorrent ответил { $status }: { $line }

## core/src/downloads/replace.rs

# $item is the number of the release asked for, $count how many there are
core-downloads-replace-bad-item = элемента { $item } нет среди релизов (всего { $count })
# $episode is the episode's number
core-downloads-not-replaced = Серия { $episode } не заменена.
core-downloads-replaced = Серия { $episode } заменена новой загрузкой.

## core/src/downloads/rolling.rs

# $id is the rolling download's number
core-downloads-no-roll = нет скользящей загрузки { $id }
# $title is the show, $from and $to the episodes kept on disk out of $total
core-downloads-rolled = «{ $title }»: теперь серии с { $from } по { $to } из { $total }.
core-downloads-roll-ahead = держите хотя бы одну серию впрок
core-downloads-roll-nothing = скачивать нечего

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = источник поиска JSON берётся из репозитория источников, где описано, как читать его поля
core-releases-endpoint-no-name = не указано `name`
# $name is the endpoint's name, in quotes
core-releases-endpoint-bad-name = { $name }: используйте только буквы, цифры, '.', '_', '-'
# $why is core-releases-json-needs-repo
core-releases-endpoint-json = { $name }: { $why }
# $kind is the kind given, in quotes; $valid lists the kinds there are
core-releases-endpoint-unknown-kind = { $name }: неизвестный тип { $kind } (допустимые: { $valid })
core-releases-endpoint-no-url = { $name }: не указан `url`
core-releases-endpoint-not-http = { $name }: адрес должен начинаться с http:// или https://
# $kind is the endpoint's kind; {query} is written as it is in the address
core-releases-endpoint-needs-query = { $name }: url типа { $kind } должен содержать подстановку {"{"}query{"}"}
core-releases-endpoint-no-query = { $name }: url типа { $kind } сам добавляет запрос, поэтому {"{"}query{"}"} в нём не нужен

## core/src/releases/fetch.rs

# $code is the error code the indexer sent
core-releases-indexer-error = индексатор ответил ошибкой { $code }
core-releases-bad-feed = некорректная лента: { $error }
# $status is the HTTP status code
core-releases-feed-answered = лента ответила { $status }

## core/src/releases/json_feed.rs

core-releases-not-feed-json = это не JSON ленты: { $error }

## core/src/releases/json.rs

core-releases-not-json = это не JSON: { $error }
# $message is what the search said about itself
core-releases-search-error-said = поиск ответил ошибкой: { $message }
core-releases-search-error = поиск ответил ошибкой
# $path is where the results were looked for, in quotes
core-releases-no-results-field = в ответе нет результатов по пути { $path }
core-releases-results-not-list = результаты в ответе не являются списком
core-releases-search-answered = поиск ответил { $status }

## core/src/releases/native.rs

core-releases-no-endpoints = источников пока нет: добавьте репозиторий или источник вручную в разделе «Источники» в меню «Ещё»
core-releases-no-source-named = нет источника { $name }
core-releases-empty-query = введите, что искать
core-releases-cannot-follow = на { $name } нельзя подписаться: он отвечает на поиск, но у него нет ленты для проверки новых серий
core-releases-endpoint-gone = { $name } больше недоступен: удалён, выключен или потерял ключ

## core/src/releases/quirks.rs

# What is wrong with an endpoint definition's quirks. one_variant, show_feed, slug and
# {slug} are the definition's own words and stay as they are.
# $word is the word given, in quotes
core-releases-quirks-variant = one_variant { $word } должен быть одним коротким словом
core-releases-quirks-pattern-count = show_feed принимает от 1 до { $most } шаблонов страницы
core-releases-quirks-pattern-long = шаблон страницы show_feed слишком длинный
core-releases-quirks-pattern-bad = шаблон страницы show_feed: { $error }
core-releases-quirks-pattern-slug = шаблону страницы show_feed нужна группа с именем slug
core-releases-quirks-url-https = url в show_feed должен быть https
core-releases-quirks-url-host = url в show_feed должен быть на хосте самого источника
core-releases-quirks-url-slug = url в show_feed содержит {"{"}slug{"}"} ровно один раз и никаких других подстановок

## core/src/releases/refinement.rs

# 'tokens' and `tokens` name the call's own field and stay as they are
core-releases-refine-no-tokens = операции 'tokens' нужен непустой список `tokens`

## core/src/releases/grid.rs

# A release group's whole-season torrent; $group is the group's name
core-releases-group-batch = пакет { $group }
# $count is how many episodes, $list their numbers and runs ("1 to 4, 7")
core-releases-episodes-phrase =
    { $count ->
        [one] Серия { $list }
       *[other] Серии { $list }
    }
# A run of episode numbers
core-releases-episode-range = с { $from } по { $to }

## core/src/subscriptions.rs

# Fills core-error-unsupported: "anirss on this platform is not supported yet."
core-subscriptions-platform = anirss на этой платформе
# After "anirss: "; $message is why its answer did not parse
core-subscriptions-unreadable = его ответ не читается ({ $message })
core-subscriptions-stopped = он остановился без объяснения
# $said is what anirss printed
core-subscriptions-stopped-said = он остановился: { $said }

## Names that stay the same in every language

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = имя должно содержать от 1 до { $max } символов, без управляющих символов
core-prefs-room-relay = адрес ретранслятора должен начинаться с https:// или http://
core-room-service = Совместный просмотр
core-room-not-found = Комнаты с таким кодом нет. Возможно, она закрылась.
core-room-full = Эта комната заполнена.
core-room-proto = Этот ретранслятор работает на другой версии комнат. Обновите { -app-name }, чтобы войти.
core-room-unreachable = Не удалось связаться с ретранслятором: { $message }
core-room-refused = Ретранслятор отказал: { $message }
core-room-not-in = Вы не в комнате.
core-room-no-name = выберите имя, под которым вас увидят в комнате
core-room-bad-code = код комнаты состоит из 6 букв и цифр
core-room-created = Создана комната совместного просмотра. Её код: { $code }.
core-room-joined = Вы вошли в комнату совместного просмотра { $code }.
core-room-left = Вы вышли из комнаты совместного просмотра.
core-room-member-joined = { $name } теперь в комнате.
core-room-member-left = { $name } покидает комнату.
core-room-closed = Комната совместного просмотра закрыта.
core-room-lost = Связь с комнатой совместного просмотра потеряна. Пробуем снова.
core-room-back = Снова в комнате совместного просмотра.
