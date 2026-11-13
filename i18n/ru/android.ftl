## Названия, которые остаются названиями

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Общие слова страниц

common-cancel = Отмена
common-back = Назад
common-save = Сохранить
common-done = Готово
common-close = Закрыть
common-delete = Удалить
common-retry = Повторить
common-copy = Копировать
common-clear = Очистить
common-on = Вкл.
common-off = Выкл.
common-play = Смотреть
common-hide = Скрыть
common-remove = Убрать
common-add = Добавить
common-test = Тест
common-reading = Чтение...
common-working = Выполняется...
common-checking = Проверка...
common-all = Все
common-none = Нет
common-episodes = { $count ->
    [one] { $count } серия
    [few] { $count } серии
    [many] { $count } серий
   *[other] { $count } серии
}
common-episode-n = Серия { $n }
common-join-comma = { $a }, { $b }

## Язык (#793)

lang-follow-phone = Как на телефоне
more-language = Язык
more-language-phone = Как на телефоне: { $lang }
lang-name-en = Английский
lang-name-da = Датский
lang-name-de = Немецкий
lang-name-es = Испанский
lang-name-fr = Французский
lang-name-pt-BR = Португальский (Бразилия)
lang-name-it = Итальянский
lang-name-nl = Нидерландский
lang-name-pl = Польский
lang-name-ru = Русский
lang-name-ja = Японский
lang-name-ko = Корейский
lang-name-zh-CN = Китайский (упрощённый)

## Первый запуск

onb-welcome-title = AniBeam
onb-welcome-note = Ваше аниме на этом телефоне. Найдите сериал, узнайте, где он есть, скачайте и ведите свои списки.
onb-language-title = Язык
onb-language-note = AniBeam говорит на языке телефона, пока вы не выберете другой. Позже его можно сменить в разделе «Ещё».
onb-lists-title = Ваши списки
onb-lists-note = Войдите, чтобы просмотренное здесь учитывалось в AniList или MyAnimeList. Это можно сделать позже в разделе «Ещё».
onb-wifi-title = Только Wi-Fi
onb-wifi-note = Серии весят много. AniBeam качает по Wi-Fi, если не сказать иначе.
onb-storage-title = Память
onb-storage-note = Лимит того, сколько AniBeam может скачать. Место не резервируется: оно свободно, пока его не займут серии.
onb-notify-title = Уведомления
onb-notify-note = Пока серии скачиваются, уведомление показывает ход загрузки, а другое сообщает, что серия готова. Загрузки идут и без них.
onb-sites-title = Сайты поиска
onb-sites-note = В AniBeam изначально нечего искать. Репозиторий - это подписанный список сайтов для поиска. Вставьте адрес репозитория, проверьте его ключ и установите нужные сайты. Это можно сделать позже в разделе «Ещё».
onb-wifi-only = Только по Wi-Fi
onb-wifi-on = Вкл. Перед загрузкой по мобильному интернету будет вопрос.
onb-wifi-off = Выкл. Загрузки идут и по мобильному интернету.
onb-storage-caption = 18 ГБ - примерно 12 серий. Выберите любой лимит, вплоть до всей памяти телефона.
onb-use-cap = Задать лимит
onb-cap-on = Вкл. При { $cap } AniBeam ничего не скачивает, пока вы не удалите серии.
onb-cap-off = Выкл. AniBeam качает, пока память телефона не заполнится.
onb-start = Начать
onb-continue = Далее
onb-back = Назад

## Оформление

look-colours = Цвета
look-mode = Режим
look-dark = Тёмный
look-light = Светлый
look-system = Системный
look-follows-phone-dark = Как в настройках телефона: тёмная тема.
look-follows-phone-light = Как в настройках телефона: светлая тема.
look-changes-at-once = Изменения видны сразу, здесь и на всех страницах.
look-dark-theme = Тёмная тема
look-light-theme = Светлая тема
look-accent = Акцент
look-accent-note = Выделяет выбранное, текущее и новое.
look-size = Размер
look-text-size = Размер текста
look-size-smaller = Мельче
look-size-normal = Обычный
look-size-large = Крупный
look-size-larger = Крупнее
look-size-largest = Максимум
look-text-size-note = Размер текста везде в AniBeam, поверх размера шрифта телефона.

## Окно: вкладки и панели поверх страниц

app-name = AniBeam
nav-home = Главная
nav-calendar = Календарь
nav-library = Библиотека
nav-downloads = Загрузки
nav-more = Ещё
app-mobile-title = Скачивать по мобильному интернету?
app-mobile-note = Одна серия часто весит 1-2 ГБ. На тарифе с лимитом за вечер может уйти трафик на месяц.
app-mobile-keep = Только Wi-Fi
app-mobile-use = Разрешить и мобильный
app-versions-note = Одна и та же серия из { $count ->
    [one] { $count } релиза
    [few] { $count } релизов
    [many] { $count } релизов
   *[other] { $count } релизов
}. Выберите, какой смотреть.
app-versions-plays-now = { $release }, воспроизводится
app-menu-get = Скачать серии
app-menu-delete = Удалить серии

## Ещё: список и состояние каждого раздела

more-settings = Настройки
more-search = Поиск по настройкам
more-search-none = Ничего не найдено
more-search-hits = { $count ->
    [one] { $count } настройка
    [few] { $count } настройки
    [many] { $count } настроек
   *[other] { $count } настройки
}
more-search-tap = Нажмите, чтобы перейти
more-search-hint = Попробуйте слово попроще: память, Wi-Fi, трекер, сайт, компьютер.
more-accounts = Аккаунты
more-streaming = Стриминг
more-subtitles = Субтитры
more-connection = Соединение
more-computer = Ваш компьютер
more-icon = Значок
more-stats = Статистика
more-data = Данные
more-appearance = Оформление
more-activity = Журнал
more-about = О приложении
more-accounts-none = Трекер не подключён
more-accounts-syncing = Синхронизация с { $tracker }
more-accounts-syncing-both = Синхронизация с AniList и MAL
more-dl-status-wifi = Wi-Fi, без лимита
more-dl-status-wifi-cap = Wi-Fi, лимит { $cap }
more-dl-status-mobile = Wi-Fi и мобильный интернет, без лимита
more-dl-status-mobile-cap = Wi-Fi и мобильный интернет, лимит { $cap }
more-stream-status-auto = Авто, 720p в мобильной сети или через ретранслятор
more-stream-status-original = Всегда исходный файл
more-stream-status-converted = Конвертация в { $quality }
more-sites-status-none = Нет репозиториев
more-sites-status-on = { $sites ->
    [one] { $sites } сайт включён
    [few] { $sites } сайта включено
    [many] { $sites } сайтов включено
   *[other] { $sites } сайта включено
}
more-sites-status-on-from = { $sites ->
    [one] { $sites } сайт включён
    [few] { $sites } сайта включено
    [many] { $sites } сайтов включено
   *[other] { $sites } сайта включено
}, { $repos ->
    [one] из { $repos } репозитория
    [few] из { $repos } репозиториев
    [many] из { $repos } репозиториев
   *[other] из { $repos } репозиториев
}
more-conn-status-unchecked = Ещё не проверялось
more-conn-status-ok = Все шаги пройдены
more-conn-status-failed = Не прошли шаги: { $failed } из { $total }
more-computer-status-none = Ещё не подключён
more-icon-default = Обычный
more-icon-dark = Тёмный
more-icon-line = Контурный
more-subs-status = Размер, положение, шрифт, цвета и подложка
more-data-status = Бэкап, восстановление, кэш изображений
more-activity-status = Что делал AniBeam, сначала новое
more-look-status-phone = Как на телефоне
more-stats-status = { $hours } ч просмотра за { $year } г.
more-stats-status-empty = Год просмотра

## Ещё: настройки разделов

more-accounts-note = Всё, что вы смотрите здесь, отмечается в ваших списках, а прогресс из них виден в сериалах.
more-storage = Память
more-storage-capped = Лимит { $cap }
more-storage-no-cap = Без лимита
more-storage-used = занято сериями
more-storage-cap-caption = лимит
more-storage-free = свободно на телефоне
more-set-storage-cap = Лимит памяти
more-storage-cap-on = При { $cap } ничего не скачивается, пока вы не удалите серии.
more-storage-cap-off = Выкл.: AniBeam качает, пока память телефона не заполнится.
more-network = Сеть
more-network-wifi = Только Wi-Fi
more-network-mobile = Wi-Fi и мобильный интернет
more-wifi-on = Вкл. Загрузки ждут Wi-Fi; по мобильному интернету сначала будет вопрос.
more-set-stream-quality = Качество потока
more-stream-question = Как здесь воспроизводится серия с вашего компьютера.
more-stream-auto-note = Авто: по Wi-Fi файл играет как есть, а в мобильной сети или когда Tailscale соединяет с компьютером через ретранслятор, он конвертируется в 720p.
more-stream-original-note = Всегда исходный файл. Большим файлам нужно быстрое соединение.
more-stream-converted-note = Конвертируется на вашем компьютере в { $quality }. Трафика уходит меньше; перемотка занимает секунду.
more-set-feeds = Свои ленты
more-feeds-note = Ищутся раньше сайтов
more-set-check = Проверка соединения
more-check-note = Проверяет каждый шаг между телефоном и сайтами поиска: поиск адреса, доступ и защищённое соединение.
more-check-copy = Копировать результаты
more-set-computer = AniBeam на вашем компьютере
more-computer-note = Его библиотека видна в разделе «Библиотека», а серии играют здесь. Имя вроде my-computer или адрес вроде 100.64.0.1.
more-computer-placeholder = my-computer:8790
more-subs-note = С чем начинается каждая серия на этом телефоне. Серия с вашего компьютера начинается с настроек этого компьютера.
more-icon-note = Значок на главном экране и в списке приложений.
more-icon-line-note = Контурный значок берёт цвета обоев, как другие значки Material You.
more-icon-moved-note = Некоторые лаунчеры убирают значок с главного экрана при смене. В списке приложений он остаётся: перетащите его оттуда обратно.
more-set-whats-new = Что нового
more-whats-new-note = Что изменилось в каждой версии, сначала новое.
more-set-onboarding = Настроить заново
more-onboarding-note = Пройти вопросы первого запуска ещё раз.
more-set-version = Версия

## Строки состояния, которые Rust пишет на страницах

host-synced = Обновлено { $when }.
host-last-try = Последняя попытка { $when }: { $error }
host-asking = Запрос...
host-reached = Подключено к { $host }, AniBeam { $version }.
host-saved = Сохранено. Синхронизация...
acct-signed-in-as = Вы вошли как { $name }
acct-signed-in = Вход выполнен
acct-not-in-build = Недоступно в этой сборке
acct-tap-to-sign-in = Вход не выполнен. Нажмите, чтобы войти.
acct-sign-in-failed = Не удалось войти: { $why }. Нажмите, чтобы повторить.
acct-sign-in-not-started = Вход не начался: { $error }. Нажмите, чтобы повторить.
acct-still-starting = AniBeam ещё запускается. Попробуйте ещё раз через минуту.
home-recently-watched = Недавно смотрели
home-recently-downloaded = Недавно скачано
home-note-empty = На этом телефоне
home-note = { $count ->
    [one] На этом телефоне { $count } сериал
    [few] На этом телефоне { $count } сериала
    [many] На этом телефоне { $count } сериалов
   *[other] На этом телефоне { $count } сериала
}
lib-this-phone = Этот телефон
lib-note-phone = { $count ->
    [one] Этот телефон, { $count } сериал
    [few] Этот телефон, { $count } сериала
    [many] Этот телефон, { $count } сериалов
   *[other] Этот телефон, { $count } сериала
}
lib-note-remote = { $device }, синхронизировано { $when }
lib-empty-search = Ничего не найдено по запросу «{ $query }».
lib-empty-phone = На этом телефоне пока пусто.
lib-empty-remote = В библиотеке этого устройства пусто.
cal-view-airing = В эфире
cal-note-none = На неделе ничего не выходит
cal-note = { $count ->
    [one] { $count } серия в ближайшие 7 дней
    [few] { $count } серии в ближайшие 7 дней
    [many] { $count } серий в ближайшие 7 дней
   *[other] { $count } серии в ближайшие 7 дней
}
airing-asking = Запрос к AniList
airing-asking-newest = Запрос к AniList за новинками
airing-filter-all = Все { $count }
airing-filter-listed = В списках { $count }
airing-filter-owned = На телефоне { $count }
airing-this-season = Этот сезон
airing-next-season = Следующий сезон
airing-other-season = Другие
airing-hidden = Скрыто как неинтересное: { $count }
airing-plan-needs-tracker = Чтобы добавить в планы, нужен AniList или MAL. Войдите: Ещё, Аккаунты.
airing-plan-failed = Не удалось добавить в планы: { $error }
list-watching = Смотрю
list-planning = В планах
list-completed = Просмотрено
list-paused = Отложено
list-dropped = Брошено
list-repeating = Пересматриваю
more-check-all-ok = Все шаги пройдены.
more-check-failed = Не прошли шаги: { $failed } из { $total }. По скриншоту этого списка будет видно, где.
more-check-row = { $step }  { $ms } мс
more-check-copied = Скопировано. Вставьте в сообщение.
more-version = AniBeam для Android { $version } (сборка { $build })
dl-note-waiting = Ждём Wi-Fi. Загрузки продолжатся сами, когда телефон подключится к Wi-Fi.
dl-note-nothing = Ничего не скачивается.
dl-state-done = Готово
dl-state-waiting = Ждёт Wi-Fi
dl-state-paused = Пауза
dl-state-peers = Поиск пиров
dl-state-checking = Проверка
dl-state-error = Остановлено из-за ошибки
dl-state-downloading = Скачивается
dl-amount = { $got } из { $size }
dl-speed = { $rate }/с
dl-left = осталось { $span }
dl-episodes-range = Серии с { $from } по { $to }
dl-batch = Пакет
disk-storage-line = занято { $used }
disk-storage-line-free = занято { $used }, свободно { $free }
disk-free = Свободно на телефоне: { $free }.
disk-free-below-cap = Свободно на телефоне: { $free }, меньше лимита: память заполнится раньше, чем AniBeam остановится.

## Поиск по настройкам: слова, которые человек может ввести

more-set-subs-size = Размер и положение субтитров
more-set-subs-style = Стиль субтитров
more-words-anilist = anilist трекер вход войти аккаунт подключить список прогресс
more-words-mal = mal myanimelist трекер вход войти аккаунт подключить список прогресс
more-words-storage-cap = место диск лимит предел гб память заполнена удалить размер
more-words-wifi = wifi wi-fi мобильный интернет сотовая сеть данные
more-words-notifications = уведомления оповещения прогресс разрешение шторка разрешить
more-words-stream-quality = авто оригинал 1080p 720p 480p конвертация трафик компьютер воспроизведение
more-words-check-repos = обновить обновление репозиторий репо
more-words-add-repo = репозиторий репо доверие отпечаток адрес url
more-words-add-site = свой сайт лента rss torznab jackett prowlarr адрес
more-words-feeds = лента rss источник удалить
more-words-check = сеть dns проверка https защищённое соединение интернет
more-words-computer = tailscale tailnet хост адрес компьютер библиотека стриминг проверка сохранить устройство
more-words-whats-new = изменения список изменений обновление
more-words-onboarding = первый запуск настройка заново сначала
more-words-version = сборка версия
more-words-stats = часы серии жанры студии сезоны год просмотрено итоги месяц история
more-words-backup = резервная копия бэкап экспорт файл сохранить копия перенос компьютер json личные входы
more-words-restore = восстановить импорт файл загрузить библиотека слияние
more-words-cache = изображения постеры баннеры очистить память место кэш
more-words-activity = журнал события ошибки проблемы копировать очистить история что произошло
more-words-colours = оформление тема тёмная светлая режим система акцент цвет
more-words-text-size = оформление шрифт больше меньше крупнее размер текст читать
more-words-icon = значок иконка лаунчер главный экран логотип тёмный контурный тематический material you
more-words-subs-size = субтитры размер масштаб положение позиция выше низ
more-words-subs-style = субтитры шрифт цвет контур тень подложка жирный ass стиль сбросить
more-words-language = язык перевод английский локаль

## Главная

home-title = Главная
home-next-to-air = Скоро выйдет
home-after-that = Потом
home-coming-up = Скоро
home-nothing-week = На ближайшую неделю ничего нет.
home-empty-title = На телефоне пока ничего нет
home-empty-body = Скачанные здесь сериалы и их новые серии появятся на главной. Найдите сериал в разделе «Загрузки».

## Библиотека

lib-title = Библиотека
lib-refresh = Обновить
lib-search-phone = Искать на телефоне
lib-search-device = Искать на { $device }
lib-find-in-downloads = Найти «{ $query }» в Загрузках
lib-empty-body = Серии, скачанные на этот телефон, появятся здесь. Найдите сериал в разделе «Загрузки».
lib-pick-show = Выберите сериал, чтобы открыть его здесь.
lib-shows-caption = { $count ->
    [one] сериал
    [few] сериала
    [many] сериалов
   *[other] сериала
}
lib-on-phone = на телефоне
lib-free = свободно
lib-hold-poster = Удерживайте постер, чтобы скачать ещё серий или удалить их.
phone-nothing-on-phone = нет на телефоне
phone-on-phone = { $count } на телефоне

## Календарь

cal-title = Календарь
cal-week = Неделя
cal-days = Дни
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A, %-d %B
cal-today = Сегодня
cal-tomorrow = Завтра
cal-yesterday = Вчера
cal-nothing = Пусто
cal-next = Далее
cal-get = Скачать
cal-day-note = { $date }, { $count }
cal-next-airing = Следующий выход: { $title }
cal-nothing-aired-yesterday = Вчера ничего не выходило.
cal-nothing-aired-that-day = В тот день ничего не выходило.
cal-nothing-today = Сегодня выходов нет.
cal-nothing-tomorrow = Завтра выходов нет.
cal-nothing-that-day = В этот день выходов нет.
cal-empty-title = На телефоне пока нет сериалов
cal-empty-body = Серии сериалов, которые вы храните здесь, появляются в день выхода: два дня назад и неделя вперёд.
phone-episodes-range = Серии с { $from } по { $to }
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = На телефоне
phone-out-ago = Вышла { $ago }

## В эфире

airing-out-now = Уже вышло
airing-last-two-days = Последние два дня
airing-season-note = Сериалы сезона берутся с AniList.
airing-none-lists-title = В эфире нет сериалов из ваших списков
airing-none-lists-body = Кнопка «В планы» на сериале добавляет его в ваш список.
airing-none-phone-title = В эфире нет сериалов с телефона
airing-none-phone-body = Кнопка «Скачать серии» на сериале скачивает его сюда.
airing-marked = «{ $title }» отмечен как неинтересный.
airing-undo = Отменить
airing-plan = В планы
airing-get-more = Скачать ещё серии
airing-get = Скачать серии
airing-not-interested = Не интересно

## Сезоны

season-winter = Зима
season-spring = Весна
season-summer = Лето
season-fall = Осень
season-named = { $season } { $year }
season-pick = Выберите сезон

## Страница сериала

series-back-home = Главная
series-back-library = Библиотека
series-back-downloads = Загрузки
series-back-calendar = Календарь
series-get-episodes = Скачать серии
series-episodes = Серии
series-delete-episodes = Удалить серии
series-delete-tap = Нажмите на серии, которые нужно удалить.
series-delete-picked = { $count ->
    [one] Выбрана { $count } серия.
    [few] Выбрано { $count } серии.
    [many] Выбрано { $count } серий.
   *[other] Выбрано { $count } серии.
}
series-delete-note = Файлы будут удалены с этого телефона. Отметки о просмотре сохранятся.
series-delete-button = { $count ->
    [one] Удалить { $count } серию
    [few] Удалить { $count } серии
    [many] Удалить { $count } серий
   *[other] Удалить { $count } серии
}
series-no-episodes = На телефоне нет серий, и новых не ожидается.
series-versions = { $count ->
    [one] { $count } версия
    [few] { $count } версии
    [many] { $count } версий
   *[other] { $count } версии
}
series-note-next-up = Следующая
series-note-missing = Не хватает
series-note-not-here = Нет на телефоне
series-note-not-downloaded = Не скачана
series-note-downloading = Скачивается { $percent }%
series-note-resume-at = Продолжить с { $time }
phone-episodes-estimate = { $count ->
    [one] { $count }? серия
    [few] { $count }? серии
    [many] { $count }? серий
   *[other] { $count }? серии
}
phone-next-airs = Серия { $n } выйдет через { $time }
phone-watched = Просмотрено
phone-not-watched = Пока не смотрели
phone-progress = Просмотрено { $watched } из { $total }, на телефоне { $count }
phone-on-this-phone = { $count } на этом телефоне
phone-play-film = Смотреть фильм
phone-resume-film = Продолжить фильм
phone-play-episode = Смотреть серию { $n }
phone-resume-episode = Продолжить серию { $n }
phone-about-date = примерно { $date }

## Статус в списке, оценка, заметки и правки

show-status-watching = Смотрю
show-status-planning = В планах
show-status-completed = Просмотрено
show-status-paused = Отложено
show-status-dropped = Брошено
show-status-repeating = Пересматриваю
show-status-now = { $label }, выбрано
show-add-to-list = В список
show-your-score = Ваша оценка { $score }
show-rate = Оценить
show-fixes = Правки
show-your-notes = Ваши заметки
show-add-notes = Заметки
show-home-each = Изменение попадёт в каждый подключённый список.
show-home-no-tracker = Хранится на этом телефоне: ни один трекер не знает этот сериал.
show-home-tracker-lacks = Хранится на этом телефоне: в вашем трекере нет этого сериала.
show-home-until-connected = Хранится на этом телефоне, пока вы не подключите AniList или MyAnimeList.
show-sites-both = AniList и MyAnimeList
show-notes-title = Заметки
show-notes-placeholder = Где остановились, кто посоветовал, какой релиз скачать
show-notes-over = { $count ->
    [one] На { $count } символ больше лимита
    [few] На { $count } символа больше лимита
    [many] На { $count } символов больше лимита
   *[other] На { $count } символа больше лимита
}
show-notes-saved-to = Сохранено в { $sites }.
show-notes-saved-to-tracker = Сохранено в вашем трекере.
show-notes-saving = Сохранение заметок.
show-notes-clearing = Удаление заметок.
show-score-saving = Сохранение оценки.
show-score-saved = Оценка записана.
show-score-clearing = Удаление оценки.
show-score-cleared = Оценка удалена.
show-moving-to = Перенос в «{ $label }».
show-moved-to = Перенесено в «{ $label }».
show-rescan-started = Скан запущен.
show-still-starting = AniBeam ещё запускается. Попробуйте ещё раз через минуту.
show-fix-match = Привязка
show-fix-find-match = Найти совпадение
show-fix-rescan = Сканировать папку

## Персонажи, связанные сериалы и рекомендации

show-characters = Персонажи
show-related = Связанное
show-related-all = Показать все { $count }
show-related-filling = Данные ещё подгружаются с AniList.
show-recommendations = Рекомендации
show-card-owned = В вашей библиотеке
show-card-unreleased = Ещё не вышло
show-card-on-anilist = На AniList
show-rec-available = Есть
show-relation-prequel = Приквел
show-relation-sequel = Сиквел
show-relation-side-story = Побочная
show-relation-parent = Основная
show-relation-spin-off = Спин-офф
show-relation-adaptation = Адаптация
show-relation-alternative = Альтернатива
show-relation-summary = Пересказ
show-relation-character = Персонаж
show-relation-source = Источник
show-relation-compilation = Компиляция
show-relation-contains = Содержит
show-relation-other = Другое
show-role-main = Главный
show-role-supporting = Второстепенный
show-role-background = Фоновый
show-format-tv = ТВ
show-format-tv-short = Короткий ТВ
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Спешл
show-format-film = Фильм
show-entry-get = Скачать серии
show-entry-open-anilist = Открыть на AniList
show-entry-line = { $line }, { $status }

## Исправить неверное совпадение

match-title = Привязка
match-refresh = Обновить данные
match-clear = Сбросить
match-clear-it = Сбросить
match-clear-warning = Сброс удаляет связь с трекером и заданное вручную число серий. Нажмите ещё раз, чтобы сбросить.
match-tvmaze = TVmaze
match-search-anilist = Поиск в AniList
match-search-tvmaze = Поиск в TVmaze
match-paste-anilist = Вставьте ссылку на AniList или MyAnimeList
match-paste-tvmaze = Вставьте ссылку на TVmaze или IMDb
match-look-up = Найти
match-apply = Применить
match-look-up-all = Найти все папки, которых AniBeam пока не знает
match-format-music = Музыка
match-join-and = { $first } и { $second }
match-matched-on = Найдено: { $sites }
match-found-tvmaze = Сериал найден на TVmaze
match-found-anilist = Найдено на AniList
match-found-anilist-season = Найдено на AniList, сезон { $season }
match-found-mal = Найдено на MyAnimeList
match-searching = Поиск: { $name }
match-type-more = Чтобы искать в { $name }, введите хотя бы две буквы.
match-none = { $name }: ничего не найдено.
match-looking-up = Поиск.
match-matched-to = Привязано к «{ $title }».
match-applied = Привязка задана.
match-applying = Применение привязки.
match-refreshing = Обновление: «{ $title }».
match-clearing = Сброс привязки.
match-cleared = Привязка сброшена.
match-auto-looking = Поиск папок, которых AniBeam пока не знает.

## Пакеты (строка под релизом в поиске)

inspect-reading = Чтение списка файлов
inspect-holds = { $count ->
    [one] В нём { $count } серия: { $runs }
    [few] В нём { $count } серии: { $runs }
    [many] В нём { $count } серий: { $runs }
   *[other] В нём { $count } серии: { $runs }
}

## Что нового

notes-title = Что нового
notes-got-it = Понятно

## Перенос списка в трекер

move-lists-label = Списки на этом телефоне
move-title = Перенести список в { $site }
move-here = На телефоне: { $list }
move-there = Сейчас в { $site }: { $list }
move-later-hint = Позже: Ещё, Аккаунты
move-not-now = Не сейчас
move-nothing = Ничего
move-not-on-list = Нет в вашем списке
move-nothing-left = Переносить больше нечего
move-side-status-score = { $status }, оценка { $score }
move-side-score = оценка { $score }
move-lead = { $count ->
    [one] У { $count } сериала статус или оценка есть только на этом телефоне. При переносе они попадут в ваш список { $site }, где их увидят ваш компьютер и профиль. Для этих сериалов то, что сейчас в { $site }, будет заменено. Ничего не отправится, пока вы не нажмёте «Перенести».
    [few] У { $count } сериалов статус или оценка есть только на этом телефоне. При переносе они попадут в ваш список { $site }, где их увидят ваш компьютер и профиль. Для этих сериалов то, что сейчас в { $site }, будет заменено. Ничего не отправится, пока вы не нажмёте «Перенести».
    [many] У { $count } сериалов статус или оценка есть только на этом телефоне. При переносе они попадут в ваш список { $site }, где их увидят ваш компьютер и профиль. Для этих сериалов то, что сейчас в { $site }, будет заменено. Ничего не отправится, пока вы не нажмёте «Перенести».
   *[other] У { $count } сериалов статус или оценка есть только на этом телефоне. При переносе они попадут в ваш список { $site }, где их увидят ваш компьютер и профиль. Для этих сериалов то, что сейчас в { $site }, будет заменено. Ничего не отправится, пока вы не нажмёте «Перенести».
}
move-label = { $count ->
    [one] Перенести { $count } сериал
    [few] Перенести { $count } сериала
    [many] Перенести { $count } сериалов
   *[other] Перенести { $count } сериала
}
move-waiting = { $count ->
    [one] У { $count } сериала статус или оценка есть только здесь. Перенесите их в { $site }, чтобы видеть на компьютере и в профиле.
    [few] У { $count } сериалов статус или оценка есть только здесь. Перенесите их в { $site }, чтобы видеть на компьютере и в профиле.
    [many] У { $count } сериалов статус или оценка есть только здесь. Перенесите их в { $site }, чтобы видеть на компьютере и в профиле.
   *[other] У { $count } сериалов статус или оценка есть только здесь. Перенесите их в { $site }, чтобы видеть на компьютере и в профиле.
}
move-moving = { $count ->
    [one] Перенос { $count } сериала в { $site }
    [few] Перенос { $count } сериалов в { $site }
    [many] Перенос { $count } сериалов в { $site }
   *[other] Перенос { $count } сериалов в { $site }
}
move-moved = { $count ->
    [one] Перенесён { $count } сериал в { $site }
    [few] Перенесено { $count } сериала в { $site }
    [many] Перенесено { $count } сериалов в { $site }
   *[other] Перенесено { $count } сериала в { $site }
}
move-failed = { $count ->
    [one] { $site } не принял { $count } сериал; подробности в журнале
    [few] { $site } не принял { $count } сериала; подробности в журнале
    [many] { $site } не принял { $count } сериалов; подробности в журнале
   *[other] { $site } не принял { $count } сериала; подробности в журнале
}

## Загрузки: поиск (сериалы, источники, релизы)

find-join-comma = { $a }, { $b }
find-join-and = { $a } и { $b }
find-join-sentences = { $a } { $b }

find-downloads = Загрузки

find-lens-list = Список
find-lens-gaps = Пропуски
find-lens-airing = В эфире
find-lens-following = Подписки
find-lens-downloaded = Скачано
find-lens-with-count = { $lens } { $count }
find-what-to-get = Что скачать
find-lens-list-line = «Смотрю» и «В планах» из вашего списка. Откройте сериал, чтобы скачать его серии.
find-lens-gaps-line = Вышедшие серии, которых нет на телефоне. Откройте сериал, чтобы увидеть, где они есть.
find-lens-airing-line = Новые серии, которые ещё выйдут, ближайшие первыми.
find-lens-following-line = Подписки: новые серии скачиваются сами. Откройте подписку, чтобы отключить её.
find-lens-downloaded-line = Что скачано на телефон, по сериалам.

find-lens-gaps-empty = Пропусков нет: у каждого начатого сериала вышедшие серии на месте.
find-lens-airing-empty = Ничего из скачанного на телефоне сейчас не выходит.
find-lens-following-empty = Подписок пока нет. Откройте сериал и включите «Подписаться».
find-lens-downloaded-empty = Пока ничего не скачано.
find-list-empty = В вашем списке пока пусто. Найдите сериал выше и добавьте его.
find-list-reading = Чтение списка...
find-list-signed-out = Войдите в AniList: Ещё, Аккаунты, и ваш список появится здесь.
find-list-failed = Не удалось прочитать список: { $error }

find-aired = вышло
find-ledger-no-feed = новых серий нет: у источника нет ленты
find-ledger-downloads = { $count ->
    [one] { $count } загрузка
    [few] { $count } загрузки
    [many] { $count } загрузок
   *[other] { $count } загрузки
}
find-ledger-downloaded = скачано
find-ledger-planning = В планах
find-ledger-watching-of = Смотрю, { $progress } из { $total }
find-ledger-watching-seen = Смотрю, просмотрено { $progress }

find-gap-missing = не хватает { $count }
find-gap-not-downloaded = не скачано { $count }

find-list-status-planning = В списке: «В планах»
find-list-status-watching = В списке: «Смотрю»
find-list-status-already = Уже в вашем списке
find-going-planning = «{ $title }» добавляется в «В планах».
find-already-listed = «{ $title }» уже в вашем списке.
find-plan-needs-tracker = Чтобы добавить в список, нужен AniList или MAL. Войдите: Ещё, Аккаунты.
find-plan-failed = Не добавлено в список: { $error }
find-show-gone = Этого сериала больше нет в библиотеке на этом телефоне.

## Сериалы по мере набора

find-shows = Сериалы
find-suggesting = Запрос к AniList...
find-suggest-tap = Нажмите на сериал, чтобы увидеть, где он есть. Кнопка ввода ищет по названиям релизов.
find-suggest-none = Нет сериала с таким названием. Кнопка ввода ищет по названиям релизов.
find-suggest-hint = Сериалы появляются по мере набора. Кнопка ввода ищет по названиям релизов.
find-suggest-failed = Не удалось запросить AniList: { $error }
find-library-line = В вашей библиотеке
find-library-line-none = В вашей библиотеке: на телефоне 0 из { $total }
find-library-line-some = В вашей библиотеке: на телефоне { $here } из { $total }

## Страница сериала в поиске

find-plan-to-watch = В планы
find-on-phone-count = { $count } на телефоне
find-airing = В эфире
find-dropped = { $count ->
    [one] Скрыт { $count } релиз, в названии которого нет этого сериала.
    [few] Скрыто { $count } релиза, в названии которых нет этого сериала.
    [many] Скрыто { $count } релизов, в названии которых нет этого сериала.
   *[other] Скрыто { $count } релиза, в названии которых нет этого сериала.
}

find-following = Подписка
find-stop-following-ask = Отписаться от «{ $name }»?
find-unfollow-explain = Новые серии перестанут скачиваться. Содержимое папки подписки можно оставить или удалить вместе с ней: все загрузки в ней, включая серии, скачанные вручную.
find-stop-following = Отписаться
find-stop-keep = Отписаться, серии оставить
find-stop-delete = Отписаться и удалить загрузки из папки
find-rule-taken = { $downloads ->
    [one] { $downloads } загрузка
    [few] { $downloads } загрузки
    [many] { $downloads } загрузок
   *[other] { $downloads } загрузки
}, { $files ->
    [one] { $files } файл
    [few] { $files } файла
    [many] { $files } файлов
   *[other] { $files } файла
} в папке
find-rule-paused = Подписка на паузе, новые серии не придут. { $taken }.
find-rule-no-feed = У источника нет ленты, новые серии не придут. { $taken }.
find-rule-active = Новые серии скачиваются сами. { $taken }.
find-unfollowed-deleted = Подписка на «{ $name }» отменена, загрузки из её папки удалены.
find-unfollowed-kept = Подписка на «{ $name }» отменена. Серии остаются на этом телефоне.
find-stop-failed = Отмена подписки не завершилась: { $error }

find-where = Где скачать
find-where-pick = Выберите источник, чтобы увидеть, что в нём есть.
find-where-nothing = Ничего не скачается, пока вы не скажете.
find-source-idle = Запросить
find-source-trying = Пробуем: { $name }
find-source-searching = Поиск
find-source-silent = Не отвечает
find-no-sites-title = Сайтов поиска пока нет
find-no-sites-body = Добавьте репозиторий сайтов поиска или свою ленту в разделе «Ещё».
find-open-more = Открыть «Ещё»
find-pick-source-title = Выберите источник
find-pick-source-body = Его релизы появятся здесь сеткой: серия на кубик.
find-releases = { $count ->
    [one] { $count } релиз
    [few] { $count } релиза
    [many] { $count } релизов
   *[other] { $count } релиза
}
find-unreachable = { $host } недоступен. Некоторые сети и страны блокируют сайты с релизами; AniBeam уже обходит блокировки по именам, так что здесь блокируется сам сайт или он не работает. Поможет VPN, или попробуйте другой источник.
find-the-source = Источник

find-has = Что есть
find-has-note = Нажмите на строку, чтобы взять всё, или на отдельные серии.
find-episodes-tag = Серии
find-batch = Пакет { $label }
find-row-meta = { common-episodes }  сиды { $seeds }  { $size }
find-on-this-phone = На телефоне
find-on-computer = На { $name }

find-plan-no-releases = У этого источника нет релизов этого сериала.
find-plan-nothing-picked = Ничего не выбрано: нажмите на строку или её кубики.
find-plan-line = { $episodes } от { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] { $count } серия, которая у вас уже есть, пропущена. Включите «Заменить», чтобы скачать её снова.
    [few] { $count } серии, которые у вас уже есть, пропущены. Включите «Заменить», чтобы скачать их снова.
    [many] { $count } серий, которые у вас уже есть, пропущено. Включите «Заменить», чтобы скачать их снова.
   *[other] { $count } серии, которые у вас уже есть, пропущены. Включите «Заменить», чтобы скачать их снова.
}
find-plan-follow-note = Новые серии по запросу «{ $query }» будут скачиваться сами.
find-plan-on-desk = Они скачаются на { $name } в его библиотеку.
find-follow-new = Подписка на новые
find-follow-new-note = Скачиваются сами по мере выхода.
find-replace = Заменить уже скачанное
find-replace-note = Скачивает выбранные серии заново поверх тех, что на телефоне.
find-go = { $mode ->
    [replace] Скачать: { common-episodes }, заменить
    [subscribe] Скачать: { common-episodes } и подписаться
    [both] Скачать: { common-episodes }, заменить и подписаться
   *[plain] Скачать: { common-episodes }
}
find-go-pick = Выберите серии
find-every-release = Все релизы этого источника

## Куда скачать

find-download-to = Скачать на
find-this-phone = Этот телефон
find-your-computer = Ваш компьютер
find-desk-asking = Запрос к { $name }...
find-desk-silent = { $name } не отвечает. Выберите «Этот телефон» или повторите, когда он включён.
find-sending-to = Отправка на { $name }...

## Список релизов

find-best-fit = Лучшее
find-on-disk = На диске
find-get = Скачать
find-get-all = Скачать все { $count }
find-subscribe = Подписаться
find-back-step = Шаг назад
find-narrow-placeholder = Уточнить или -слово, чтобы исключить
find-best-fit-searches = Лучшее ищет: { $query }
find-fact-batch = пакет
find-fact-seeds = сиды { $count }
find-fact-via = через { $source }
find-searching-for = Поиск: { $text }...
find-search-line = { $count ->
    [0] На { $source } по этому запросу ничего
    [one] { $count } релиз на { $source }
    [few] { $count } релиза на { $source }
    [many] { $count } релизов на { $source }
   *[other] { $count } релиза на { $source }
}{ $fell ->
    [1] , после того как первый источник ничего не дал
   *[other] { "" }
}{ $narrowed ->
    [1] , сужено до { $token }
   *[other] { "" }
}.
find-refined = { find-releases }, { $dir ->
    [down] меньше на { $delta }
   *[up] больше на { $delta }
}.
find-refine-nothing-left = Ничего не осталось.
find-refine-no-match = Нет релиза с таким.
find-refine-empty = Сначала введите что-нибудь.
find-refine-no-change = Ничего не меняется.

## Скачивание и подписка

find-getting = Скачивание: { $what }.
find-subscribed = Подписка оформлена: новые серии скачиваются сами.
find-subscribed-desk = Подписка оформлена на { $name }: новые серии скачиваются там сами.
find-subscribed-as = Подписка «{ $name }» оформлена: новые серии скачиваются сами.
find-downloading = { $count ->
    [one] Скачивается { $count } релиз.
    [few] Скачиваются { $count } релиза.
    [many] Скачиваются { $count } релизов.
   *[other] Скачиваются { $count } релиза.
}
find-not-added = Не добавлено: { $error }
find-not-subscribed = Подписка не оформлена: { $error }
find-desk-refused = { $name } не принял: { $error }
find-desk-not-subscribed = Подписка на { $name } не оформлена: { $error }

## Плеер

player-subtitles = Субтитры
player-audio = Аудио
player-speed = Скорость
player-connecting = Подключение
player-hold-speed = Скорость 2×
player-replay = Повтор
player-unlock = Разблокировать
player-next-stay = Остаться
player-next-go = Далее
player-jump-back = −10 с
player-jump-forward = +10 с
player-speed-now = Скорость { $speed }
player-skip-intro = Пропустить
player-skip-outro = Пропустить
player-skipped-intro = Опенинг пропущен
player-skipped-outro = Эндинг пропущен
player-undo = Отменить
player-start-over = Сначала
player-resumed-at = Продолжено с { $time }
player-picture-failed = Не удалось включить изображение. Звук продолжает играть.
player-file-failed = Файл не воспроизводится: { $why }.
player-streaming-from = Поток с { $host }, { $height }p
player-mpv-no-start = mpv не запустился
player-mpv-no-start-why = mpv не запустился: { $why }
player-android-only = плеер работает только на Android

## Плеер: задержки

player-delay-title = Задержка
player-delay-note = Каждое нажатие сдвигает на 0,1 с. До следующего файла.
player-delay-seconds = { $value } с
player-sub-delay-now = Задержка субтитров { $value }
player-audio-delay-now = Задержка аудио { $value }

## Плеер: трансляция на устройство

player-cast-playing-on = Играет на { $device }
player-cast-stop = Остановить трансляцию на { $device }
player-cast-searching = Поиск Chromecast и ТВ
player-cast-to = Вывести на
player-cast-tv = ТВ
player-cast-loading = Загрузка
player-cast-playing = Играет
player-cast-paused = Пауза
player-cast-finished = Завершено
player-cast-stopped = Остановлено

## Плеер: слова трекера

player-rated = Оценено
player-score-failed = Не удалось сохранить оценку
player-marked-watched = Просмотрено
player-tracked = Учтено
player-not-tracked = Не учтено
player-track-update-failed = Не удалось обновить список
player-not-tracked-why = Не учтено: { $reason }.
player-track-ok-at = { $tracker }: серия { $progress }
player-track-ok = { $tracker }: ок
player-track-note = { $tracker }: { $note }
player-track-failed = Не удалось обновить { $tracker }
player-track-line = { $head }: { $parts }
player-refusal-hidden = сериал скрыт
player-refusal-no-match = у сериала нет привязки к трекеру
player-refusal-not-newer = трекер уже дальше этой серии
player-refusal-extra = это экстра
player-refusal-unmatched = файл не сопоставлен с серией
player-refusal-on-disk = файл есть только на диске

## Субтитры

subs-line-default = Размер 1×, обычные файлы берут ваш стиль
subs-sample = Куда ты его спрятал?
subs-size-place-title = Размер и положение
subs-size = Размер
subs-position = Положение
subs-position-helper = 100 у нижнего края. Меньшие значения поднимают строки выше.
subs-style-title = Стиль
subs-styled = Стили в субтитрах
subs-styled-as-made = Файлы со стилем (ASS) выглядят так, как задумали авторы.
subs-styled-mine = Ваш стиль ниже применяется ко всем файлам. Надписи и караоке могут поломаться.
subs-styled-size-only = Файлы со стилем сохраняют вид и берут ваш размер. Обычные файлы берут ваш стиль.
subs-mode-as-made = Как есть
subs-mode-size-only = Только размер
subs-mode-mine = Свой
subs-font = Шрифт
subs-font-bad = Введите название шрифта, например Noto Sans.
subs-bold = Жирный
subs-colour = Цвет текста
subs-colour-bad = Введите цвет, например #ffcc00.
subs-outline = Контур
subs-outline-bad-width = Толщина: число, 0 или больше.
subs-outline-bad-colour = Введите цвет, например #000000.
subs-outline-helper = Толщина, затем цвет.
subs-shadow = Тень
subs-shadow-bad = Число, 0 или больше. 0 означает без тени.
subs-shadow-helper = Насколько далеко падает тень, 0 без неё.
subs-box = Подложка
subs-box-helper = Тёмная подложка за строками. 0 убирает её.
subs-reset-title = Сброс
subs-reset-helper = Возвращает размер, шрифт, цвета, контур, подложку и положение к исходным.
subs-reset = Сбросить

## Стриминг

stream-quality-auto = Авто
stream-quality-original = Оригинал
stream-app-starting = Приложение ещё запускается
stream-off-on-host = Стриминг на { $host } выключен. Включите там Stream episodes: Settings, Phone app.
stream-cannot-stream = На { $host } запущен AniBeam { $version }, который пока не умеет стримить. Обновите его там.
stream-no-session = Другое устройство не выдало сеанс
stream-answer-unreadable = Ответ не удалось прочитать: { $why }
stream-answered-status = { $name } ответил { $status }

## Статистика

stats-nothing-yet = Пока ничего
stats-empty-title = Ничего не просмотрено
stats-empty-body = Досмотрите серию, и она появится здесь: часы, жанры и сезоны, которые вы смотрели.
stats-hours-caption = часов просмотра
stats-finished-caption = завершённых сериалов
stats-busiest-caption = лучший месяц
stats-episodes-caption = { $count ->
    [one] серия
    [few] серии
    [many] серий
   *[other] серии
}
stats-by-month = По месяцам
stats-top-genres = Топ жанров
stats-top-studios = Топ студий
stats-seasons = Просмотренные сезоны
stats-watched-most = Топ просмотров
stats-year-line = Что вы смотрели в { $year } году
stats-eps = { $count } сер.
stats-month-1 = янв
stats-month-2 = фев
stats-month-3 = мар
stats-month-4 = апр
stats-month-5 = май
stats-month-6 = июн
stats-month-7 = июл
stats-month-8 = авг
stats-month-9 = сен
stats-month-10 = окт
stats-month-11 = ноя
stats-month-12 = дек
stats-month-letter-1 = Я
stats-month-letter-2 = Ф
stats-month-letter-3 = М
stats-month-letter-4 = А
stats-month-letter-5 = М
stats-month-letter-6 = И
stats-month-letter-7 = И
stats-month-letter-8 = А
stats-month-letter-9 = С
stats-month-letter-10 = О
stats-month-letter-11 = Н
stats-month-letter-12 = Д

## Резервная копия и кэш изображений

backup-title = Бэкап
backup-export = Создать копию библиотеки
backup-export-helper = Один файл с вашими папками, привязками и тем, что вы посмотрели.
backup-private = Включить личные данные
backup-private-helper = Добавляет входы в AniList и MyAnimeList, историю просмотров и настройки. Данные для входа записываются открытым текстом, храните файл в тайне.
backup-restore = Восстановить из копии
backup-restore-helper = Читает копию в эту библиотеку. Побеждает то, что на диске; файл заполняет остальное.
backup-done = Копия создана. Файл лежит там, куда вы его сохранили.
backup-not-saved = Не сохранено. Ничего не записано.
backup-starting = AniBeam ещё запускается. Попробуйте ещё раз через минуту.
backup-images-title = Кэш изображений
backup-images-body = Постеры, баннеры и портреты хранятся на телефоне, чтобы библиотека работала офлайн. Очищенные изображения загрузятся снова, когда понадобятся страницам.
backup-keep = Оставить
backup-removed = { $count ->
    [one] Удалено { $count } изображение
    [few] Удалено { $count } изображения
    [many] Удалено { $count } изображений
   *[other] Удалено { $count } изображения
}
backup-images-none = Кэш изображений пуст
backup-images-unmeasured = Не удалось измерить кэш изображений
backup-images-retry-note = Попробуйте ещё раз через минуту. С самими изображениями всё в порядке.
backup-images-line = { $size }, { $count ->
    [one] { $count } изображение
    [few] { $count } изображения
    [many] { $count } изображений
   *[other] { $count } изображения
}
backup-images-question = { $count ->
    [one] Удалить { $count } изображение?
    [few] Удалить { $count } изображения?
    [many] Удалить { $count } изображений?
   *[other] Удалить { $count } изображения?
}

backup-sum-list = { $items }.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] { $count } папка добавлена
    [few] { $count } папки добавлено
    [many] { $count } папок добавлено
   *[other] { $count } папки добавлено
}
backup-sum-series = { $count ->
    [one] { $count } сериал создан
    [few] { $count } сериала создано
    [many] { $count } сериалов создано
   *[other] { $count } сериала создано
}
backup-sum-matches = { $count ->
    [one] { $count } привязка применена
    [few] { $count } привязки применено
    [many] { $count } привязок применено
   *[other] { $count } привязки применено
}
backup-sum-views = { $views ->
    [one] { $views } просмотр
    [few] { $views } просмотра
    [many] { $views } просмотров
   *[other] { $views } просмотра
} и { $episodes ->
    [one] { $episodes } завершённая серия
    [few] { $episodes } завершённые серии
    [many] { $episodes } завершённых серий
   *[other] { $episodes } завершённые серии
} объединены
backup-sum-resume = { $count ->
    [one] { $count } точка продолжения объединена
    [few] { $count } точки продолжения объединено
    [many] { $count } точек продолжения объединено
   *[other] { $count } точки продолжения объединено
}
backup-sum-accounts = { $count ->
    [one] { $count } аккаунт трекера добавлен
    [few] { $count } аккаунта трекера добавлено
    [many] { $count } аккаунтов трекера добавлено
   *[other] { $count } аккаунта трекера добавлено
}
backup-sum-fixes = { $count ->
    [one] Ваша правка применена к { $count } файлу.
    [few] Ваша правка применена к { $count } файлам.
    [many] Ваша правка применена к { $count } файлам.
   *[other] Ваша правка применена к { $count } файлам.
}
backup-sum-missing = { $count ->
    [one] Не найдена { $count } папка на этом телефоне.
    [few] Не найдены { $count } папки на этом телефоне.
    [many] Не найдено { $count } папок на этом телефоне.
   *[other] Не найдены { $count } папки на этом телефоне.
}
backup-sum-left-out = Пропущено: { $fields }.

## Журнал

activity-everything = Всё
activity-problems = Проблемы
activity-copied = Готово
activity-clear-every-line = Очистить все строки
activity-fold = Свернуть
activity-no-problems = Проблем нет. Всё сработало.
activity-nothing-yet = Пока нечего сообщить. Здесь появится то, что AniBeam делает для вас.
activity-time-today = %H:%M:%S
activity-time-other = %-d %b, %H:%M

## Даты и числа (companion format.rs)

fmt-clock = %H:%M
fmt-day-month = %-d %b
fmt-weekday-date-time = %a, %-d %b, %H:%M
fmt-weekday-date-year-time = %a, %-d %b %Y, %H:%M
fmt-stamp = %-d %b, %H:%M
fmt-decimal-point = ,

fmt-today-at = Сегодня, { $time }
fmt-tomorrow-at = Завтра, { $time }
fmt-yesterday-at = Вчера, { $time }

fmt-span-days = { $d } д { $h } ч
fmt-span-hours = { $h } ч { $m } мин
fmt-span-minutes = { $m } мин { $s } с
fmt-span-seconds = { $s } с
fmt-in = через { $span }
fmt-full-days = { $d } д { $h } ч { $m } мин { $s } с
fmt-full-hours = { $h } ч { $m } мин { $s } с
fmt-full-minutes = { $m } мин { $s } с

fmt-ago-now = только что
fmt-ago-minutes = { $count } мин назад
fmt-ago-hours = { $count } ч назад
fmt-ago-days = { $count } д назад

fmt-bytes-b = { $value } Б
fmt-bytes-kb = { $value } КБ
fmt-bytes-mb = { $value } МБ
fmt-bytes-gb = { $value } ГБ
fmt-bytes-tb = { $value } ТБ

## Связь с компьютером (companion client.rs, sync.rs, address.rs)

client-not-found = Телефон не находит «{ $host }». Tailscale подключён?
client-timeout = { $host } не отвечает. Он включён, и Tailscale подключён на обоих?
client-connect-failed = Не удалось связаться с { $host }. Он включён, и Tailscale подключён на обоих?
client-refused = { $host } включён, но AniBeam на нём закрыт или не делится библиотекой. Откройте AniBeam на нём и включите Share over Tailscale: Settings, Phone app.
client-no-route = Нет маршрута до { $host }. Tailscale подключён?
client-invalid-input = Не удалось подключиться к { $host }
client-unreachable = Не удалось связаться с { $host }: { $why }
client-cannot-find = Телефон не находит { $host }. Tailscale подключён?
client-broke-off = Ответ от { $host } оборвался: { $why }
client-answered = { $host } ответил { $status } на { $path }
client-not-anibeam-status = Что-то ответило, но это не AniBeam ({ $host } ответил { $status } на { $path })
client-not-anibeam-hello = Что-то ответило, но это не AniBeam (ответ не похож на приветствие AniBeam)
client-not-anibeam-name = Что-то ответило, но это не AniBeam (оно называет себя { $name })
client-too-new = Хост говорит на API { $host_api }, а это приложение читает { $app_api }. Обновите приложение.
client-snapshot-unreadable = Снимок не удалось прочитать: { $why }
client-probe-unreadable = Сведения о файле не удалось прочитать: { $why }
client-peers-unreadable = Список AniBeam не удалось прочитать: { $why }
sync-save-failed = Не удалось сохранить снимок: { $why }
address-https = AniBeam на вашем компьютере отвечает по обычному http через Tailscale; уберите https://.
address-empty = Введите имя или адрес хоста, например my-computer:8790.
address-ipv6-close = В адресе IPv6 нужна закрывающая ].
address-ipv6-port = Порт ставьте после ], например [fd7a::1]:8790.
address-bad-host = «{ $text }» не похоже на имя хоста.
address-bad-port = «{ $port }» не является портом.

## Факты о сериале (companion library.rs)

clib-status-airing = В эфире
clib-status-not-yet = Не вышел
clib-format-tv-short = Короткий ТВ
clib-format-movie = Фильм
clib-format-special = Спешл
clib-format-music = Музыка

## Сайты поиска и репозитории

repos-by-hand-title = Добавить вручную
repos-by-hand-note = Для сайта, которого нет ни в одном репозитории. Подойдёт любая лента, где { "{query}" } стоит на месте поискового запроса, в том числе лента Jackett или Prowlarr. Ключ или токен в адресе переносится в защищённое хранилище этого телефона.
repos-name-placeholder = Название
repos-url-placeholder = https://...{ "{query}" }...
repos-index-placeholder = https://.../index.json
repos-add-site = Добавить
repos-add-title = Репозиторий
repos-add-note = Вставьте адрес индекса репозитория. AniBeam проверит подпись и покажет вам ключ, прежде чем что-то сохранить.
repos-add-repo = Добавить
repos-trust-title = Доверять репозиторию?
repos-fingerprint = Отпечаток ключа
repos-trust-warning = Сайты репозитория олимитяют, куда уходят ваши поиски. Доверяйте только репозиториям от тех, кому доверяете.
repos-trust = Доверять
repos-open-link = Открыть { $url }
repos-saving = Сохранение...
repos-install = Установить
repos-update = Обновить
repos-edit-note = Ключ или токен, введённый в адрес, переносится в защищённое хранилище этого телефона.
repos-edit-note-keys = Оставьте каждый { "{key}" } в адресе как есть; его значение хранится на этом телефоне.
repos-change-its = Изменить: { $keys }
repos-check-repos = Проверить репозитории на новые сайты
repos-check-sites = Проверить сайты
repos-add-site-label = Добавить сайт вручную
repos-needs-before-search = Перед поиском нужно: { $labels }
repos-no-longer-offered = Репозиторий больше не предлагает
repos-update-host = Обновление отправляет запросы на { $host }
repos-tap-change-custom = Нажмите, чтобы изменить название или адрес
repos-tap-change-keys = Нажмите, чтобы изменить: { $labels }
repos-line-custom = { $host }  { $kind }  добавлен вами
repos-update-to = Обновить до v{ $version }
repos-add-its-value = Добавить значение
repos-asks-for = Запрашивает: { $labels }
repos-preview-line = { $count ->
    [one] { $count } сайт, подписан
    [few] { $count } сайта, подписаны
    [many] { $count } сайтов, подписаны
   *[other] { $count } сайта, подписаны
}
repos-preview-trusted = { $count ->
    [one] { $count } сайт, подписан, уже в доверенных
    [few] { $count } сайта, подписаны, уже в доверенных
    [many] { $count } сайтов, подписаны, уже в доверенных
   *[other] { $count } сайта, подписаны, уже в доверенных
}
repos-ask-kept-empty = Хранится только на этом телефоне. Пустое поле сохраняет прежнее значение.
repos-ask-kept-never = Хранится только на этом телефоне, никогда не попадает в библиотеку или экспорт.
repos-ask-new-host = Новая версия отправляет это на { $host }, поэтому спрашивает снова.
repos-ask-new = Новая версия снова запрашивает эти значения.
repos-type-value = Сначала введите значение.
repos-health-answered = Ответил за { $ms } мс
repos-health-failed = Не ответил. { $step }: { $detail }

## Загрузки

dl-title = Загрузки
dl-wifi-only = Только Wi-Fi
dl-wifi-and-mobile = Wi-Fi и мобильный интернет
dl-joined = { $first }, { $second }
dl-batch-files = { $count ->
    [one] Пакет, { $count } файл
    [few] Пакет, { $count } файла
    [many] Пакет, { $count } файлов
   *[other] Пакет, { $count } файла
}
dl-batch-tag = Пакет
dl-downloading-count = скачивается { $count }
dl-free-on-phone = свободно на телефоне
dl-search-placeholder = Сериал или название релиза
dl-sources = Источники
dl-downloading = Скачивается
dl-queue-note = { $count ->
    [one] { $count } загрузка
    [few] { $count } загрузки
    [many] { $count } загрузок
   *[other] { $count } загрузки
}
dl-finished = Завершено
dl-finished-note = { $count ->
    [one] { $count } загрузка, в вашей библиотеке
    [few] { $count } загрузки, в вашей библиотеке
    [many] { $count } загрузок, в вашей библиотеке
   *[other] { $count } загрузки, в вашей библиотеке
}
dl-get = Скачать
dl-pick = Выбрать

## Память и скрытые сериалы

disk-on-phone = На телефоне
disk-watched = Просмотрено
disk-not-watched = Не просмотрено
disk-deleting = Удаление
disk-delete-watched = Удалить просмотренное
disk-hidden-shows = Скрытые сериалы
disk-hidden-helper = Скрыты на главной и в библиотеке, пока вы их не вернёте.
disk-unhide = Вернуть
disk-delete-note = Файлы и субтитры будут удалены с этого телефона навсегда.
disk-every-watched = Все просмотренные серии
disk-every-finished = Все завершённые сериалы, которые вы досмотрели
disk-episodes-size = { $count ->
    [one] { $count } серия, { $size }
    [few] { $count } серии, { $size }
    [many] { $count } серий, { $size }
   *[other] { $count } серии, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } сериал, { $size }
    [few] { $count } сериала, { $size }
    [many] { $count } сериалов, { $size }
   *[other] { $count } сериала, { $size }
}
disk-list-two = { $a } и { $b }
disk-list-three = { $a }, { $b } и { $c }
disk-list-more = { $a }, { $b }, { $c } и ещё { $count }
disk-delete-finished-title = Удалить { $count ->
    [one] { $count } завершённый сериал
    [few] { $count } завершённых сериала
    [many] { $count } завершённых сериалов
   *[other] { $count } завершённых сериала
}, { $size }?
disk-delete-watched-title = Удалить { $count ->
    [one] { $count } просмотренную серию
    [few] { $count } просмотренные серии
    [many] { $count } просмотренных серий
   *[other] { $count } просмотренные серии
}, { $size }?
disk-delete-finished-body = { $names }. Только сериалы, которые по данным провайдера завершены и просмотрены целиком; выходящий сейчас сериал или тот, у которого конец приблизительный, остаётся. Все их файлы удаляются с этого телефона, включая дополнительные материалы. Их загрузки тоже убираются, поэтому их можно скачать снова.
disk-delete-watched-body = Все просмотренные вами серии, во всех сериалах. Непросмотренные серии и дополнительные материалы остаются. Файлы и их субтитры удаляются с этого телефона навсегда.
disk-no-finished = Нет завершённого сериала, просмотренного целиком
disk-nothing-watched = Просмотренного на телефоне нет
disk-deleted = { $count ->
    [one] Удалена { $count } серия, { $size }
    [few] Удалено { $count } серии, { $size }
    [many] Удалено { $count } серий, { $size }
   *[other] Удалено { $count } серии, { $size }
}
disk-delete-failed = Ничего не удалено: { $error }

## Уведомления о загрузках

notify-card-head = AniBeam · сейчас
notify-sample-called = Фрирен, серия 7
notify-sample-called-ready = Фрирен, серия 6
notify-sample-facts = 420 МБ из 1,4 ГБ, 3,2 МБ/с, осталось 5 мин
notify-downloading = Скачивается: { $called }
notify-progress = { $percent }%, { $facts }
notify-progress-of = { $percent }%, { $done } из { $size }
notify-starting = Запуск
notify-many = { $count ->
    [one] { $count } загрузка
    [few] { $count } загрузки
    [many] { $count } загрузок
   *[other] { $count } загрузки
}
notify-called-episode = { $show }, серия { $n }
notify-called-episodes = { $show }, серии с { $from } по { $to }
notify-ready = Готово: { $called }
notify-ready-many = Готово: { $called }
notify-ready-batch = Пакет { $show } готов
notify-ready-text = Скачано на этот телефон. Нажмите, чтобы открыть AniBeam.
notify-channel-downloads = Загрузки
notify-channel-ready = Готовые серии
notify-fallback-title = Скачивание
notify-row-on = Уведомления
notify-row-allow = Разрешить уведомления
notify-row-open-settings = Открыть настройки уведомлений
notify-helper-on = Вкл. Загрузки отображаются в уведомлениях, пока идут.
notify-helper-asks-next = Дальше спросит Android.
notify-helper-asks-next-later = Дальше спросит Android. Передумать можно позже в разделе «Ещё».
notify-helper-off = Выкл. Включите уведомления AniBeam в системных настройках.
notify-question-title = Показывать загрузки в уведомлениях?
notify-question-body = Пока серии скачиваются, уведомление показывает ход загрузки, а другое сообщает, что серия готова. Загрузка идёт в любом случае; это лишь решает, увидите ли вы её.
notify-allow = Разрешить
notify-not-now = Не сейчас
notify-pointer = Загрузки не показываются в уведомлениях.
notify-turn-on = Включить

## Лента

feed-title = Лента
feed-recently-released = Недавние релизы
feed-coming-soon = Скоро
feed-this-week = Эта неделя
feed-earlier = Раньше
feed-everything-else = Остальное
feed-countdown-days = { $d } д { $h } ч
feed-countdown-hours = { $h } ч { $m } мин
feed-countdown-minutes = { $m } мин
feed-next = Далее: { $title } через { $countdown }
feed-nothing-scheduled = Пока ничего не запланировано.
feed-nothing-new = На этой неделе нового нет.
feed-new-episodes = { $count ->
    [one] { $count } новая серия на этой неделе.
    [few] { $count } новые серии на этой неделе.
    [many] { $count } новых серий на этой неделе.
   *[other] { $count } новые серии на этой неделе.
}
feed-aired = вышло { $ago }
feed-downloaded = скачано { $ago }

## Файлы и выбор файла

files-already-picking = Файл уже выбирается.
files-took-too-long = Выбор файла занял слишком много времени.
files-did-not-open = Выбор файла не открылся: { $error }
files-none-here = Здесь нет выбора файлов.

## Загрузки на компьютере

desk-sent-adding = Отправлено на { $name }: { $what }. Они ещё добавляются.
desk-sent = Скачивание на { $name }: { $what }.

## Библиотека вашего компьютера

remote-watched = Просмотрено
remote-movie = Фильм
remote-not-watched-yet = Пока не смотрели
remote-watched-of = Просмотрено { $done } из { $total }
remote-watched-count = просмотрено { $count }
remote-on-disk = на диске { $count }
remote-airs-in-season = Сезон { $season }, серия { $n } выйдет через { $countdown }
remote-airs-in = Серия { $n } выйдет через { $countdown }
remote-play-episode = Смотреть серию { $n }
remote-watch-again = Пересмотреть серию { $n }
remote-update-host = Обновите AniBeam на том компьютере, чтобы смотреть это здесь
remote-aired = вышло
remote-missing = Нет
remote-on-computer = На вашем компьютере
remote-out = Вышла { $ago }
remote-season-episode = Сезон { $season }, серия { $n }
