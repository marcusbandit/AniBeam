# es translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = Inicio
nav-library = Biblioteca
nav-airing = En emisión
nav-feed = Feed
nav-watching = Viendo
nav-stats = Estadísticas
nav-metadata = Info de la serie
nav-downloads = Descargas
nav-settings = Ajustes
nav-series = Serie
nav-player = Reproductor
nav-show = Serie
nav-recommended = Recomendadas

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } error
   *[other] { $count } errores
}

## ScanSummary

scan-summary-nothing = Aún no hay nada que mostrar
scan-summary-found = { $count ->
    [one] { $count } serie encontrada
   *[other] { $count } series encontradas
}, { $recognised } reconocidas
scan-summary-found-need = { $count ->
    [one] { $count } serie encontrada
   *[other] { $count } series encontradas
}, { $recognised } reconocidas, { $need } te necesitan
scan-summary-episodes = { $count ->
    [one] { $count } episodio
   *[other] { $count } episodios
} en el disco.
scan-summary-episodes-watched = { $count ->
    [one] { $count } episodio
   *[other] { $count } episodios
} en el disco, { $watched } ya vistos en { $tracker }.
scan-summary-title = Tu carpeta ya está
scan-summary-count-found = encontradas
scan-summary-count-recognised = reconocidas
scan-summary-count-need = te necesitan
scan-summary-later = Más tarde
scan-summary-match = Reconocer { $count }
scan-summary-open-library = Abrir la biblioteca

## UpdateNotice

update-notice-check-failed = AniBeam no pudo buscar actualizaciones. Comprueba tu conexión a internet y vuelve a intentarlo.
update-notice-up-to-date = AniBeam está al día
update-notice-downloading = Descargando AniBeam { $version }
update-notice-ready = AniBeam { $version } está listo
update-notice-failed = La actualización falló
update-notice-on-aur = AniBeam { $version } está en el AUR
update-notice-out = Ya está aquí AniBeam { $version }
update-notice-out-titled = Ya está aquí AniBeam { $version } “{ $title }”
update-notice-progress = { $percent } %  ·  { $got } de { $total } MB
update-notice-restart-hint = Reinicia ahora, o se abrirá la próxima vez
update-notice-from-source = Esta copia está compilada desde el código: haz pull y recompila
update-notice-no-download = No hay descarga para este ordenador
update-notice-install-hint = Actualizar la instala y reinicia AniBeam
update-notice-retry = Reintentar
update-notice-update = Actualizar
update-notice-restart = Reiniciar
update-notice-copied = Copiado
update-notice-copy = Copiar
update-notice-release-page = Página de la versión
update-notice-later = Más tarde

## WhatsNew

whats-new-title = Novedades
whats-new-got-it = Entendido

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = d MMM HH:mm
activity-drawer-stage-library = Biblioteca
activity-drawer-stage-metadata = Metadatos
activity-drawer-stage-trackers = Trackers
activity-drawer-stage-franchise = Franquicia
activity-drawer-stage-playback = Reproducción
activity-drawer-stage-store = Almacén
activity-drawer-stage-system = Sistema
activity-drawer-stage-downloads = Descargas
activity-drawer-level-info = Info
activity-drawer-level-warn = Aviso
activity-drawer-level-error = Error
activity-drawer-copied-line = Línea copiada
activity-drawer-title = Actividad
activity-drawer-everything = Todo
activity-drawer-problems = Problemas
activity-drawer-copied = Copiado
activity-drawer-copy = Copiar
activity-drawer-clear-every = Borrar todas las líneas
activity-drawer-copy-tip = Haz clic para copiar la línea
activity-drawer-fold = Plegar
activity-drawer-empty-technical = Aún no hay nada registrado.
activity-drawer-empty-problems = Sin problemas. Todo funcionó.
activity-drawer-empty = Aún no hay nada que contar. Lo que AniBeam hace por ti aparece aquí.

## ActivityColumn

activity-column-episode-run = Episodios { $first } a { $last }
activity-column-episode-list = Episodios { $list }
activity-column-downloads = { $count ->
    [one] { $count } descarga
   *[other] { $count } descargas
}
activity-column-batch = Descarga en lote
activity-column-download = Descarga
activity-column-name-batch = { $title }, un lote
activity-column-name-episode = { $title }, episodio { $number }
activity-column-title = Actividad
activity-column-down = bajada
activity-column-up = subida
activity-column-not-connected = { $client } no está conectado, así que aquí no se mueve nada.
activity-column-connect-again = Volver a conectar
activity-column-downloading = Descargando ahora
activity-column-fewer = Menos
activity-column-more = Y { $count } más
activity-column-nothing-downloading = No se está descargando nada.
activity-column-finished-today = Terminado hoy
activity-column-today = Hoy
activity-column-today-empty = Las búsquedas, descargas y suscripciones nuevas aparecen aquí según ocurren.
activity-column-stopping = Deteniendo
activity-column-starting = Iniciando
activity-column-start-tip = Empezar a descargar
activity-column-stop-tip = Dejar de descargar; lo que tiene se queda
activity-column-cancelling = Cancelando
activity-column-cancel-all-tip = Cancelar estas descargas
activity-column-cancel-tip = Cancelar esta descarga
activity-column-size-of = { $got } de { $total }
activity-column-files-missing = Faltan archivos
activity-column-time-format = HH:mm
activity-column-left = faltan { $time }
activity-column-waiting-peers = Esperando pares
activity-column-episodes-done = { $count ->
    [one] { $done } de { $count } episodio hecho
   *[other] { $done } de { $count } episodios hechos
}
activity-column-cube-skipped = Episodio { $number }: omitido
activity-column-cube-done = Episodio { $number }: hecho
activity-column-cube-progress = Episodio { $number }: { $percent } %

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Listo
frame-strip-progress = { $doing }, { $done } de { $total }
frame-strip-idle = No hay nada en marcha
frame-strip-time-format = HH:mm
frame-finish-sign-in = Termina primero de iniciar sesión en { $tracker }
frame-did-not-work = Eso no funcionó.
frame-could-not-open = AniBeam no pudo abrir eso. Cópialo y ábrelo tú.
frame-copy = Copiar
frame-copied = Copiado
frame-status-refused = { $tracker } no lo aceptó
frame-status-refused-why = { $tracker } no lo aceptó: { $why }
frame-lists-refused = { $tracker } no aceptó { $count ->
    [one] 1 serie
   *[other] { $count } series
}; mira el registro de actividad
frame-lists-moved = { $count ->
    [one] 1 serie pasada
   *[other] { $count } series pasadas
} a { $tracker }
frame-notes-refused = { $tracker } no aceptó tus notas
frame-notes-refused-why = { $tracker } no aceptó tus notas: { $why }
frame-notes-saved = Notas guardadas
frame-connected = Conectado a { $tracker } como { $user }
frame-sign-in-stopped = El inicio de sesión en { $tracker } se detuvo antes de terminar.
frame-toast-hide-details = Ocultar detalles
frame-toast-details = Detalles
frame-toast-details-tip = Muestra qué falló, palabra por palabra, y cópialo
frame-forward = Adelante
frame-copy-magnet = Copiar magnet
frame-copy-link = Copiar enlace del torrent
frame-magnet-copied = Magnet copiado
frame-link-copied = Enlace copiado
frame-open-release-page = Abrir página del lanzamiento
frame-open-folder = Abrir carpeta
frame-copy-path = Copiar ruta
frame-path-copied = Ruta copiada
frame-mark-unwatched = Marcar sin ver desde aquí
frame-mark-watched = Marcar visto hasta aquí
frame-copy-title = Copiar título
frame-open-on-anilist = Abrir en AniList
frame-list-home-online = Cámbialo en cada lista conectada
frame-list-home-unmatched = Se guarda en este dispositivo: ningún tracker tiene esta serie
frame-list-home-not-carried = Se guarda en este dispositivo: tu tracker no tiene esta serie
frame-list-home-offline = Se guarda en este dispositivo hasta que conectes AniList o MyAnimeList
frame-moving-to = Pasando a { $list }
frame-moved-to = Pasada a { $list }
frame-plan-added = { $title } va a tu lista Planeado
frame-plan-listed = { $title } ya está en tu lista
frame-delete-watched = { $kind ->
    [film] Eliminar películas vistas
   *[episode] Eliminar episodios vistos
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] Eliminar la película
       *[episode] Eliminar el episodio
    }
   *[other] { $kind ->
        [film] Eliminar las { $count } películas
       *[episode] Eliminar los { $count } episodios
    }
}
frame-series-gone = Esa serie ya no está
frame-delete-none-watched = No hay episodios vistos de { $title } en el disco
frame-delete-none = No hay nada de { $title } en el disco
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } película vista
           *[other] { $count } películas vistas
        }
       *[episode] { $count ->
            [one] { $count } episodio visto
           *[other] { $count } episodios vistos
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } película
           *[other] { $count } películas
        }
       *[episode] { $count ->
            [one] { $count } episodio
           *[other] { $count } episodios
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } extra
   *[other] { $count } extras
}
frame-delete-and = { $first } y { $second }
frame-delete-heading = ¿Eliminar { $what } de { $title }?
frame-delete-base = Los archivos y sus subtítulos van a la papelera, así que puedes recuperarlos de ahí. Sus torrents salen del cliente de torrents, así que se pueden volver a descargar.
frame-delete-confirm = Mover a la papelera
frame-delete-done = { $what } de { $title }, a la papelera
frame-delete-watched-done = { $count ->
    [one] { $count } episodio
   *[other] { $count } episodios
}, { $size }, a la papelera
frame-delete-nothing = Nada que eliminar
frame-fix-not-drafted = Corregido. No se pudo redactar la incidencia
frame-fix-agent = Corregido. { $agent } lo está revisando y la incidencia se abrirá cuando termine
frame-fix-opening = Corregido. Abriendo la incidencia
frame-fix-not-saved = La corrección no se guardó: { $error }
frame-fix-undone = { $count ->
    [one] { $count } archivo
   *[other] { $count } archivos
} se vuelven a leer por su nombre
frame-fix-done = Se corrigió cómo se { $count ->
    [one] lee { $count } archivo
   *[other] leen { $count } archivos
}
frame-report-failed = No se pudo redactar la incidencia: { $error }
frame-report-untriaged = La incidencia está abierta en tu navegador, sin triaje. { $error }
frame-report-open = La incidencia está abierta en tu navegador, lista para enviar
frame-downloads-not-read = Sin leer.

## HomePage

home-title = Inicio
home-scan-started = Revisión iniciada
home-ago-minutes = hace { $count } min
home-ago-hours = hace { $count } h
home-ago-days = hace { $count } d
home-today = Hoy
home-tomorrow = Mañana
home-yesterday = Ayer
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d MMM
home-day-date-format = ddd d MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = Aún sin fecha
home-unit-days = { $count ->
    [one] día
   *[other] días
}
home-unit-hours = h
home-unit-minutes = min
home-week-late = { $count ->
    [one] { $count } episodio va con retraso
   *[other] { $count } episodios van con retraso
}
home-week-waiting = { $count ->
    [one] { $count } episodio disponible para ver
   *[other] { $count } episodios disponibles para ver
}
home-week-coming = { $count ->
    [one] { $count } episodio en los próximos siete días
   *[other] { $count } episodios en los próximos siete días
}
home-week-next = Siguiente: { $title } en { $time }
home-countdown-days = { $days } d { $hours } h
home-countdown-hours = { $hours } h { $minutes } min
home-countdown-minutes = { $minutes } min
home-out-not-downloaded = Aún sin descargar
home-out-downloading = Descargando
home-out-when = { $day } { $time }
home-next-up = Siguiente
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, hora estimada
home-next-when-season = Temporada { $season }, { $span }, { $time }
home-next-when-season-estimated = Temporada { $season }, { $span }, { $time }, hora estimada
home-no-date = Sin fecha
home-tile-watched = visto
home-tile-downloading = descargando
home-tile-out = salió { $ago }
home-tile-not-out = aún no ha salido
home-tile-estimated = estimado
home-tile-season-span = T{ $season } { $span }
home-tile-to-watch = { $count } por ver
home-day-nothing = Nada
home-now = Ahora { $time }
home-card-aired = { $episode } salió { $ago }
home-continue = Continuar
home-continue-fact = { $count } series que seguir
home-on-hold = En espera
home-on-hold-fact = { $count } series a las que volver
home-hero-open = Abrir { $title }
home-hero-pick-up = Sigue donde paraste
home-hero-up-next = Siguiente
home-hero-downloading = Descargando { $count ->
    [one] { $count } episodio
   *[other] { $count } episodios
}
home-hero-min-left = faltan { $count } min
home-hero-watched = { $watched } de { $total } vistos
home-hero-resume = Reanudar episodio { $number }
home-hero-play = Ver episodio { $number }
home-hero-resume-unnumbered = Reanudar episodio
home-hero-play-unnumbered = Ver episodio
home-downloaded = Descargadas recientemente
home-downloaded-fact = { $count } series con archivos nuevos esta quincena
home-week = Esta semana
home-week-show-empty = Mostrar días vacíos
home-week-hide-empty = Ocultar días vacíos
home-week-airing = En emisión
home-week-out = Disponibles
home-week-coming-up = Próximamente
home-recommended = Recomendadas para ti
home-recommended-fact = A partir de tu lista
home-recommended-scope = Recomendadas de
home-recommended-all = Todo
home-recommended-in-library = En la biblioteca
home-recommended-on-list = En la lista
home-recommended-none-library = No queda nada en tu biblioteca que recomendar. Todo busca series que aún no tienes.
home-recommended-none-list = No queda nada en tus listas que recomendar. Todo busca series que no tienes en ninguna lista.
home-catch-up = Ponte al día
home-catch-up-fact = { $count } series con episodios pendientes
home-fresh = Empieza algo nuevo
home-fresh-fact = { $count } series sin empezar
home-genres = Géneros
home-open-library = Abrir biblioteca
home-dropped = Abandonadas
home-dropped-fact = { $count ->
    [one] { $count } archivo aún en el disco
   *[other] { $count } archivos aún en el disco
}
home-out-now = Disponibles
home-out-now-fact = Episodios nuevos de esta semana, según AniList
home-popular = Populares esta temporada
home-add-folder-title = Tus series también van aquí
home-add-folder-body = Añade la carpeta donde guardas tu anime e Inicio se llena con lo que estás viendo.
home-add-folder-button = Añadir una carpeta
home-empty-scanned-title = Aún no se encontró nada en tus carpetas
home-empty-title = Aún no hay series
home-empty-scanned-body = AniBeam no encontró series ni películas en las carpetas que vigila. Vuelve a revisarlas, o comprueba las carpetas en Ajustes.
home-empty-body = Añade la carpeta donde guardas tu anime y la biblioteca se llena sola.
home-scan-all = Revisar todo
home-library-settings = Ajustes de biblioteca
home-failed-title = Inicio no se cargó
home-failed-body = AniBeam no pudo leer la biblioteca ahora mismo. Inténtalo de nuevo en un momento.

## Onboarding

onboarding-language = Idioma
onboarding-step-welcome = Bienvenida
onboarding-step-library = Tu biblioteca
onboarding-step-lists = Tu lista
onboarding-step-downloads = Descargas
onboarding-step-ready = Listo
onboarding-scan-reading = Leyendo la carpeta
onboarding-scan-found = { $count } series encontradas. Sigue cuando quieras; el resto se completa solo.
onboarding-scan-starter-empty = Tu nueva carpeta está vacía por ahora. Las series que metas en ella aparecerán aquí solas.
onboarding-scan-no-video = No apareció ningún archivo de vídeo. Comprueba que sea la carpeta correcta.
onboarding-scan-your-shows = tus series
onboarding-scan-looking-up = Buscando { $what }
onboarding-scan-looking-up-tally = Buscando { $what }, { $posters } de { $count } encontradas
onboarding-scan-posters = Descargando carteles
onboarding-scan-posters-tally = Descargando carteles, { $posters } de { $count } encontrados
onboarding-scan-seasons = Buscando temporadas relacionadas
onboarding-scan-seasons-tally = Buscando temporadas relacionadas, { $posters } de { $count } encontradas
onboarding-choose-folder = Elige tu carpeta de anime
onboarding-about = Un minuto de configuración. Tu biblioteca se termina sola.
onboarding-step-later = Más tarde
onboarding-skip-setup = Saltar configuración
onboarding-title-welcome = Te damos la bienvenida a AniBeam
onboarding-title-library = ¿Dónde está tu anime?
onboarding-title-lists = Conecta tu lista de anime
onboarding-title-downloads = Cómo funcionan las descargas
onboarding-title-ready = Todo listo
onboarding-title-later = { $count ->
    [1] Listo, una cosa para después
    [2] Listo, dos cosas para después
   *[other] Listo, tres cosas para después
}
onboarding-body-welcome = AniBeam reúne tu anime en un solo sitio. Encuentra las series de tu disco, las reproduce aquí mismo y mantiene AniList o MyAnimeList al día mientras ves.
onboarding-body-library = Elige la carpeta donde guardas tus series. Cada carpeta que contiene se convierte en una serie, con su cartel, sus episodios y por dónde ibas. AniBeam solo lee la carpeta. Nunca mueve ni renombra un archivo.
onboarding-body-lists = Inicia sesión una vez y cada episodio que termines aquí se marca en tu lista, sin escribir nada. El inicio de sesión ocurre en tu navegador, así que AniBeam nunca ve tu contraseña.
onboarding-body-downloads = Sigue una serie en la página Descargas y cada episodio nuevo llega a tu biblioteca cuando se emite. Para eso hacen falta dos cosas: un sitio donde buscar y un cliente de torrents que descargue.
onboarding-body-ready = Todo esto se puede cambiar después en Ajustes, y Ajustes, Acerca de vuelve a lanzar esta configuración.
onboarding-skip-step = Saltar por ahora
onboarding-get-started = Empezar
onboarding-open-app = Abrir AniBeam
onboarding-continue = Continuar
onboarding-payoff-library = Tu biblioteca
onboarding-payoff-library-line = Carteles, episodios y lo que viene, de cada serie de tus carpetas.
onboarding-payoff-plays = Se reproduce aquí
onboarding-payoff-plays-line = Subtítulos, salto de intro, y recuerda dónde paraste.
onboarding-payoff-list = Tu lista, sincronizada
onboarding-payoff-list-line = Termina un episodio y cuenta en AniList o MyAnimeList.
onboarding-payoff-episodes = Llegan episodios
onboarding-payoff-episodes-line = Sigue una serie y cada episodio nuevo llega a tu biblioteca.
onboarding-films-too = También sirve una carpeta de películas: llámala Movies.
onboarding-choose-folder-button = Elegir carpeta
onboarding-no-folder = Aún no tengo carpeta
onboarding-starter-where = Así se crea una carpeta Anime en { $dir } y se usa.
onboarding-add-another = Añadir otra carpeta
onboarding-shelf-more = más
onboarding-preview-show = Tu serie
onboarding-preview-finished = Episodio 3 terminado aquí
onboarding-preview-on-list = En tu lista: 3 vistos
onboarding-preview-set-by = Lo marca AniBeam en cuanto salen los créditos
onboarding-lists-connected = Conectado. Lo que veas aquí ya cuenta en tu lista.
onboarding-lists-no-account = ¿Aún sin cuenta? Las dos son gratis y con una basta. Puedes desconectarte cuando quieras en Ajustes, Cuentas.
onboarding-search-title = Un sitio donde buscar
onboarding-search-ready = { $count ->
    [one] { $count } fuente lista para buscar.
   *[other] { $count } fuentes listas para buscar.
}
onboarding-search-none = AniBeam no trae fuentes propias. Las añades desde un repo de fuentes: un enlace que alguien te comparte y que pegas en Ajustes, Descargas.
onboarding-sources-after = Se abre tras la configuración
onboarding-sources-add = Añadir un repo después
onboarding-fetch-title = Quién descarga
onboarding-fetch-auto = Que se encargue AniBeam
onboarding-fetch-recommended = Recomendado
onboarding-fetch-auto-line = AniBeam trae su propio qBittorrent y lo ejecuta discretamente en segundo plano. Nada que instalar, nada que rellenar.
onboarding-fetch-existing = Ya uso qBittorrent
onboarding-fetch-existing-line = AniBeam se conecta al qBittorrent que ya usas. Después introduces su dirección y su usuario en Ajustes.
onboarding-ready-library = Biblioteca
onboarding-ready-library-none = Aún sin carpeta. Está en Ajustes, Biblioteca cuando la quieras.
onboarding-ready-library-series = { $count ->
    [one] { $count } carpeta
   *[other] { $count } carpetas
}, { $series } series por ahora
onboarding-ready-library-empty = { $count ->
    [one] { $count } carpeta, vacía
   *[other] { $count } carpetas, vacías
} por ahora
onboarding-ready-library-scanning = { $count ->
    [one] { $count } carpeta
   *[other] { $count } carpetas
}, revisando
onboarding-ready-lists = Tu lista
onboarding-ready-lists-as = { $tracker }, como { $user }
onboarding-ready-lists-none = Aún sin conectar. Está en Ajustes, Cuentas cuando lo quieras.
onboarding-ready-downloads = Descargas
onboarding-ready-downloads-line = { $sources ->
    [0] Aún sin fuentes
    [one] { $sources } fuente
   *[other] { $sources } fuentes
}, { $client ->
    [own] tu propio qBittorrent.
   *[bundled] el qBittorrent de AniBeam.
} { $next ->
    [ready] Todo listo.
    [opens] A continuación se abre Ajustes, Descargas.
   *[both] Las dos cosas están en Ajustes, Descargas.
}
onboarding-ready-open-downloads = Abrir Ajustes, Descargas
onboarding-ready-set-up = Configurar ahora

## Rail.qml

rail-home = Inicio
rail-library = Biblioteca
rail-airing = En emisión
rail-feed = Feed
rail-stats = Estadísticas
rail-show-info = Info de la serie
rail-downloads = Descargas
rail-settings = Ajustes
rail-downloads-complete =
    { $count ->
        [one] Descargas: { $count } suscripción completa
       *[other] Descargas: { $count } suscripciones completas
    }
rail-title-language-name = Idioma de los títulos: { $language }. Pulsa para el siguiente
rail-title-language-english = Inglés
rail-title-language-tip = Idioma de los títulos: romaji, inglés o japonés
rail-whats-new-in = Novedades de { $release }
rail-whats-new = Novedades
rail-settings-note = Ajustes: { $note }

## FixSheet.qml

fix-kind-episode = Episodio
fix-kind-opening = Opening
fix-kind-ending = Ending
fix-kind-preview = Avance
fix-kind-special = Especial
fix-kind-other = Otro
fix-reads-season-episode = Temporada { $season } episodio { $number }
fix-reads-extra = { $kind } { $index }
fix-apply-failed = No se pudieron aplicar las correcciones.
fix-title = Corregir cómo se leen los archivos
fix-lede = { $title }. Indica qué es realmente cada archivo; cada revisión posterior lo respeta.
fix-rescan = Releer carpeta
fix-rescan-started = Releyendo la carpeta
fix-filled-in = Rellenado
fix-fill-in = Rellenar
fix-column-file = Archivo
fix-column-reads-as = Se lee como
fix-column-should-be = Debería ser
fix-fixed = Corregido
fix-number-placeholder = N.º
fix-index-placeholder = Ninguno
fix-keep = Mantener la corrección
fix-leave-as-read = Dejar este archivo como se lee
fix-read-by-name = Volver a leer este archivo por su nombre
fix-report = Abrir una incidencia en GitHub
fix-report-body = Abre una incidencia rellenada para que la envíes. Es pública y nombra estos archivos.
fix-report-body-agent = Abre una incidencia rellenada para que la envíes. Es pública y nombra estos archivos. { $agent } la revisa primero.
fix-close-unapplied =
    { $count ->
        [one] ¿Cerrar sin aplicar { $count } corrección?
       *[other] ¿Cerrar sin aplicar { $count } correcciones?
    }
fix-number-invalid = Antes hay que corregir un número en rojo
fix-close-anyway = Cerrar igualmente
fix-keep-editing = Seguir editando
fix-apply-count =
    { $count ->
        [one] Aplicar { $count } corrección
       *[other] Aplicar { $count } correcciones
    }

## MatchPanel.qml

match-matched-to = Reconocida como { $title }
match-applied = Reconocimiento aplicado
match-found-tvmaze = Serie encontrada en TVmaze
match-found-anilist = Encontrada en AniList
match-found-anilist-season = Encontrada en AniList, temporada { $season }
match-found-mal = Encontrada en MyAnimeList
match-sites-and = { $first } y { $next }
match-matched-on = Reconocida en { $sites }
match-provider-silent = { $provider } no respondió
match-title-walk = Reconocer, { $walk }
match-title = Reconocer
match-skip = Saltar
match-refresh-info = Actualizar info
match-refreshing = Actualizando { $title }
match-clear-confirm = Borrarlo: quita el enlace y el número de episodios fijado
match-clear = Borrar reconocimiento
match-cleared = Reconocimiento borrado
match-search-placeholder = Buscar en { $provider }
match-link-tvmaze = Pega un enlace de TVmaze o IMDb
match-link-anilist = Pega un enlace de AniList o MyAnimeList
match-look-up = Buscar
match-searching = Buscando en { $provider }
match-type-more = Escribe al menos dos letras para buscar en { $provider }.
match-no-results = No se encontraron títulos en { $provider }.

## WatchingPage.qml

watching-episode-code = Ep { $number }
watching-countdown-days = { $days } d { $hours } h
watching-countdown-hours = { $hours } h { $minutes } min
watching-countdown-minutes = { $minutes } min
watching-on-disk = En el disco
watching-on-anilist = En AniList
watching-ready-to-start = Lista para empezar
watching-not-in-library = No está en la biblioteca
watching-get-to-start = Obtén episodios para empezar
watching-anilist-updated = AniList actualizado { $when }
watching-airing = En emisión
watching-next-in = Siguiente: { $title } en { $wait }
watching-episodes-waiting = Episodios pendientes
watching-series-behind =
    { $count ->
        [one] { $count } serie con retraso
       *[other] { $count } series con retraso
    }
watching-up-to-date = Al día
watching-anilist-only = Solo en AniList
watching-planning-summary-off = Lo que tienes pensado ver en AniList.
watching-planning-summary =
    { $count ->
        [one] { $count } serie planeada, { $here } en el disco.
       *[other] { $count } series planeadas, { $here } en el disco.
    }
watching-summary-off = Lo que estás viendo o repitiendo en AniList.
watching-summary-waiting =
    { $count ->
        [one] { $count } serie tiene episodios pendientes.
       *[other] { $count } series tienen episodios pendientes.
    }
watching-caught-up = Todo al día.
watching-start-watching = Empezar a ver
watching-play-next = Ver siguiente
watching-open-on-anilist = Abrir en AniList
watching-refreshing = Actualizando
watching-connect-title = AniList no está conectado
watching-connect-body = Conéctalo y esta página muestra las series que estás viendo o repitiendo en AniList, con Reproducir en las que tienes y una cuenta atrás en las que se emiten.
watching-connect-waiting = Esperando al navegador
watching-connect-anilist = Conectar AniList
watching-load-failed-title = La lista Viendo no se cargó
watching-load-failed-body = No se pudo llegar a AniList. La lista vuelve en la próxima actualización.
watching-planning-failed-title = La lista Planeado no se cargó
watching-planning-failed-body = No se pudo leer su caché. Inténtalo de nuevo, o volverá en la próxima actualización.
watching-planning-empty-title = No hay nada en tu lista Planeado
watching-planning-empty-body = Quiero verlo en En emisión, o Planeado en el menú de lista de una serie, la pone aquí.
watching-airing-now = En emisión
watching-empty-title = No hay nada en tu lista Viendo
watching-empty-body = Marca una serie como Viendo en AniList y aparecerá aquí.
watching-open-anilist = Abrir AniList

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] { $count } serie oculta
       *[other] { $count } series ocultas
    }
library-hidden-sync-stops =
    { $count ->
        [one] { $count } serie oculta, y deja de sincronizarse con el tracker
       *[other] { $count } series ocultas, y dejan de sincronizarse con el tracker
    }
library-unhidden =
    { $count ->
        [one] { $count } serie de vuelta, y se sincroniza otra vez
       *[other] { $count } series de vuelta, y se sincronizan otra vez
    }
library-undo = Deshacer
library-clearing-score =
    { $count ->
        [one] Borrando la nota de { $count } serie
       *[other] Borrando la nota de { $count } series
    }
library-rating =
    { $count ->
        [one] Puntuando { $count } serie con { $score }
       *[other] Puntuando { $count } series con { $score }
    }
library-moving =
    { $count ->
        [one] Pasando { $count } serie a { $list }
       *[other] Pasando { $count } series a { $list }
    }
library-rescanning =
    { $count ->
        [one] Releyendo { $count } serie
       *[other] Releyendo { $count } series
    }
library-finding-matches =
    { $count ->
        [one] Buscando correspondencia para { $count } serie
       *[other] Buscando correspondencia para { $count } series
    }
library-sort-last-viewed = Vistas recientemente
library-sort-downloaded = Descargadas recientemente
library-sort-title = Título
library-sort-progress = Progreso
library-sort-average-score = Nota media
library-sort-my-score = Mi nota
library-dir-a-to-z = De la A a la Z
library-dir-z-to-a = De la Z a la A
library-dir-oldest = Más antiguas primero
library-dir-newest = Más nuevas primero
library-dir-least-watched = Menos vistas primero
library-dir-most-watched = Más vistas primero
library-dir-lowest = Más bajas primero
library-dir-highest = Más altas primero
library-group-none = Nada
library-group-season = Temporada
library-group-type = Tipo
library-group-genre = Género
library-group-watch-state = Estado
library-scan-started = Revisión iniciada
library-tab-all = Todo
library-tab-series = Series
library-tab-movies = Películas
library-tab-watched = Vistas
library-tab-hidden = Ocultas
library-count-movies =
    { $count ->
        [one] { $count } película
       *[other] { $count } películas
    }
library-count-watched = { $count } vistas
library-count-hidden = { $count } ocultas
library-count-series =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    }
library-title = Biblioteca
library-all-genres = Todos los géneros
library-search-placeholder = Buscar títulos o carpetas
library-sort = Ordenar
library-group = Agrupar
library-franchises-together = Franquicias juntas
library-watched-shows = Series vistas
library-load-failed-title = La biblioteca no se cargó
library-no-matches = Sin resultados para "{ $query }"
library-none-in-genre =
    { $what ->
        [series] No hay series de { $genre } aquí
        [films] No hay películas de { $genre } aquí
        [watched] No hay series vistas de { $genre } aquí
        [hidden] No hay series ocultas de { $genre } aquí
       *[shows] No hay series de { $genre } aquí
    }
library-none-here =
    { $what ->
        [series] No hay series aquí
        [films] No hay películas aquí
        [watched] No hay series vistas aquí
        [hidden] No hay series ocultas aquí
       *[shows] No hay series aquí
    }
library-load-failed-body = AniBeam no pudo leer la biblioteca ahora mismo. Inténtalo de nuevo en un momento.
library-no-matches-body = Revisa cómo está escrito, o busca en toda la biblioteca desde Todo.
library-none-in-genre-body = Nada en esta vista tiene ese género. Todos los géneros muestra el resto.
library-none-in-tab-body = Tu biblioteca aún no tiene nada de esto. Todo muestra todo lo que hay.
library-all-filtered-body = Todo lo de tu biblioteca queda fuera de esta vista por los filtros.
library-clear-search = Borrar búsqueda
library-show-all = Mostrar todo
library-picked = { $count } elegidas
library-pick-all = Todo
library-pick-all-tip = Elegir todas las tarjetas visibles (Ctrl+A)
library-unhide = Mostrar
library-hide = Ocultar
library-rate = Puntuar
library-rate-tip = Una nota para todas las elegidas, en cada tracker conectado
library-needs-tracker = Necesita un tracker conectado y una serie reconocida
library-move-to-list = Pasar a lista
library-rescan = Releer
library-find-match = Reconocer
library-find-match-tip = Buscar cada elegida sin reconocer
library-all-matched = Todas las elegidas están reconocidas
library-empty-scanned-title = Aún no se encontró nada en tus carpetas
library-empty-title = Tu biblioteca está vacía
library-empty-scanned-body = AniBeam no encontró series ni películas en las carpetas que vigila. Vuelve a revisarlas, o comprueba las carpetas en Ajustes.
library-empty-body = Añade la carpeta donde guardas tu anime y la biblioteca se llena sola, o importa una exportación de AniBeam.
library-scan-all = Revisar todo
library-add-folder = Añadir una carpeta
library-settings = Ajustes de biblioteca
library-import = Importar
library-import-title = Importar una exportación de AniBeam
library-import-filter-export = Exportación de AniBeam (*.json)
library-import-filter-all = Todos los archivos (*)
library-import-started = Importación iniciada

## MetadataPage.qml

metadata-title = Info de la serie
metadata-filter-all = Todo { $count }
metadata-filter-series = Series { $count }
metadata-filter-films = Películas { $count }
metadata-filter-missing = Incompletas { $count }
metadata-scan-started = Revisión iniciada
metadata-sort-added = Añadidas
metadata-sort-season = Temporada
metadata-sort-a-to-z = De la A a la Z
metadata-section-unmatched = Sin reconocer
metadata-section-missing = Carpeta no encontrada
metadata-section-incomplete = Faltan episodios
metadata-section-empty = Sin descargar
metadata-section-airing = En emisión
metadata-section-complete = Completas
metadata-no-info = Aún sin info de la serie
metadata-state-aired = { $have } de { $aired } emitidos
metadata-state-of = { $have } de { $expected }
metadata-state-of-extra = { $have } de { $expected } +{ $extra }
metadata-detail-none-aired = Ninguno de los { $aired } episodios emitidos está en el disco
metadata-detail-aired = { $have } de { $aired } emitidos en el disco
metadata-detail-aired-missing = { $have } de { $aired } emitidos en el disco, faltan { $missing }
metadata-detail-none = Ninguno de los { $expected } episodios está en el disco
metadata-detail-missing = { $have } de { $expected } en el disco, faltan { $missing }
metadata-walk-position = { $at } de { $total }
metadata-walk-done = Esa era la última
metadata-walk-left =
    { $count ->
        [one] Queda una en Sin reconocer
       *[other] Quedan { $count } en Sin reconocer
    }
metadata-countdown-days = { $days } d { $hours } h
metadata-countdown-hours = { $hours } h { $minutes } min
metadata-countdown-minutes = { $minutes } min
metadata-next-in = Siguiente en { $wait }
metadata-files-of = { $have } de { $expected }
metadata-files-on-disk = Archivos en el disco
metadata-forgot = { $title } olvidada
metadata-refreshed-failed = { $refreshed } actualizadas, { $failed } no se pudieron actualizar
metadata-refreshed =
    { $count ->
        [one] { $count } serie actualizada
       *[other] { $count } series actualizadas
    }
metadata-found =
    { $count ->
        [one] { $count } serie encontrada
       *[other] { $count } series encontradas
    }
metadata-found-some-left =
    { $count ->
        [one] { $count } serie encontrada, { $unmatched } aún sin reconocer
       *[other] { $count } series encontradas, { $unmatched } aún sin reconocer
    }
metadata-found-nothing = Nada nuevo encontrado, { $unmatched } aún sin reconocer
metadata-all-recognised = Todas las series están reconocidas
metadata-count =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    }
metadata-search-placeholder = Filtrar títulos
metadata-find-rest = Buscar el resto
metadata-find-rest-started = Buscando las carpetas que AniBeam aún no conoce
metadata-find-rest-tip = Busca cada carpeta que AniBeam aún no conoce, en cada sitio de info de series, en el orden que marca Ajustes
metadata-refresh-all = Actualizar todo
metadata-refresh-all-started = Actualizando la info de todas las series
metadata-refresh-all-tip = Vuelve a descargar la info de cada serie reconocida
metadata-crawling = Buscando vínculos de franquicia
metadata-crawl-progress = { $done } de { $total }
metadata-load-failed-title = La info de las series no se cargó
metadata-empty-scanned-title = Aún no se encontró nada en tus carpetas
metadata-no-matches = Sin resultados para "{ $query }"
metadata-nothing-under =
    { $filter ->
        [Series] Nada en Series
        [Movies] Nada en Películas
        [MissingFiles] Nada en Incompletas
       *[other] Nada en { $filter }
    }
metadata-load-failed-body = AniBeam no pudo leer la biblioteca ahora mismo. Inténtalo de nuevo en un momento.
metadata-empty-scanned-body = AniBeam no encontró series en las carpetas que vigila. Vuelve a revisarlas, o comprueba las carpetas en Ajustes.
metadata-empty-body = Tu biblioteca está vacía. Añade la carpeta donde guardas tu anime y AniBeam la revisará.
metadata-no-matches-body = Ningún título de esta vista tiene esas palabras.
metadata-nothing-under-body = Ninguna de tus series encaja ahora con este filtro. Todo muestra el resto.
metadata-scan-all = Revisar todo
metadata-add-folder = Añadir una carpeta
metadata-clear-search = Borrar búsqueda
metadata-show-all = Mostrar todo
metadata-refreshing = Actualizando { $title }
metadata-refresh-tip = Volver a descargar la info de esta serie
metadata-forget-armed = Quitarla
metadata-forget-confirm-tip = Haz clic otra vez para quitar esta serie y su historial de AniBeam
metadata-forget-tip = La carpeta ya no está en el disco: quita esta serie y su historial de AniBeam

## AiringPage.qml

airing-title = En emisión
airing-sort-popular = Popularidad
airing-sort-score = Nota
airing-sort-trending = Tendencia
airing-sort-favourites = Favoritos
airing-sort-soonest = Próximo episodio
airing-sort-aired = Episodios emitidos
airing-sort-episodes = Número de episodios
airing-sort-start = Fecha de inicio
airing-sort-duration = Duración del episodio
airing-sort-title = Título
airing-dir-most-popular = Más populares primero
airing-dir-least-popular = Menos populares primero
airing-dir-lowest = Más bajas primero
airing-dir-highest = Más altas primero
airing-dir-least-trending = Menos tendencia primero
airing-dir-most-trending = Más tendencia primero
airing-dir-fewest = Menos primero
airing-dir-most = Más primero
airing-dir-soonest = Más próximas primero
airing-dir-latest = Más lejanas primero
airing-dir-oldest = Más antiguas primero
airing-dir-newest = Más nuevas primero
airing-dir-shortest = Más cortas primero
airing-dir-longest = Más largas primero
airing-dir-a-to-z = De la A a la Z
airing-dir-z-to-a = De la Z a la A
airing-season =
    { $season ->
        [Winter] Invierno { $year }
        [Spring] Primavera { $year }
        [Summer] Verano { $year }
        [Fall] Otoño { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = hace { $minutes } min
airing-ago-hours = hace { $hours } h
airing-ago-days = hace { $days } d
airing-episode-out = { $episode } salió { $ago }
airing-no-date = Aún sin fecha
airing-in-library = En la biblioteca
airing-out-today =
    { $count ->
        [one] { $count } episodio sale hoy
       *[other] { $count } episodios salen hoy
    }
airing-dismissed-one = { $title } no volverá a aparecer aquí
airing-dismissed =
    { $count ->
        [one] { $count } serie no volverá a aparecer aquí
       *[other] { $count } series no volverán a aparecer aquí
    }
airing-planned-already-one = { $title } ya está en tu lista
airing-planned-already-all = Ya están todas en tu lista
airing-planned =
    { $count ->
        [one] { $count } serie va a tu lista Planeado
       *[other] { $count } series van a tu lista Planeado
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } serie va a tu lista Planeado, { $already } ya estaban
       *[other] { $count } series van a tu lista Planeado, { $already } ya estaban
    }
airing-play-next = Ver siguiente
airing-plan-to-watch = Quiero verlo
airing-get-more-episodes = Obtener más episodios
airing-get-episodes = Obtener episodios
airing-open-on-anilist = Abrir en AniList
airing-not-interested = No me interesa
airing-not-interested-count = { $count } no interesan
airing-not-interested-tip = Las series que marcaste como no interesa, en Ajustes
airing-refreshing = Actualizando
airing-pick-season = Elegir otra temporada
airing-filter-all = Todo
airing-filter-lists = En mis listas
airing-filter-library = En mi biblioteca
airing-hide-lists = Ocultar mis listas
airing-hide-lists-tip = Omitir las series de tus listas de AniList o MAL
airing-hide-library = Ocultar mi biblioteca
airing-hide-library-tip = Omitir las series de las que tienes episodios
airing-out-now = Disponibles
airing-out-last-day =
    { $count ->
        [one] { $count } episodio en las últimas 24 horas
       *[other] { $count } episodios en las últimas 24 horas
    }
airing-out-fewer = Menos
airing-out-all = Todos { $count }
airing-sort = Ordenar
airing-load-failed-title = No se pudo llegar a AniList
airing-load-failed-body = Las series en emisión vienen de AniList. Volverán en la próxima actualización.
airing-reading-title = Leyendo lo que se emite
airing-nothing-now = Ahora mismo no se emite nada
airing-nothing-for = No se emite nada en { $season }
airing-nothing-read = Aún no se ha leído nada
airing-reading-body = Todas las series en emisión, de AniList. La primera vez tarda unos segundos.
airing-nothing-body = AniList aún no tiene series para esta temporada. Actualizar vuelve a leerla, o elige otra temporada.
airing-nothing-read-body = Actualizar lee de AniList todas las series en emisión.
airing-hides-empty-title = Lo que ocultas no deja nada aquí
airing-lists-empty-title = Nada de esto está en tus listas
airing-library-empty-title = Nada de esto está en tu biblioteca
airing-hides-empty-body = Cada serie de aquí está en tus listas o en tu biblioteca, y ocultaste ambas.
airing-lists-empty-body = Quiero verlo en una tarjeta la añade a tu lista.
airing-library-empty-body = Obtener episodios en una tarjeta la descarga a tu biblioteca.
airing-show-everything = Mostrar todo
airing-picked = { $count } elegidas
airing-undo = Deshacer

## EpisodeRow.qml

episode-row-date-time-format = ddd d MMM, HH:mm
episode-row-date-upcoming-format = ddd d MMM
episode-row-date-format = d MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Haz clic para marcar sin ver desde aquí
episode-row-mark-watched-tip = { $state }. Haz clic para marcar visto hasta aquí
episode-row-next-up = Siguiente
episode-row-versions =
    { $count ->
        [one] { $count } versión
       *[other] { $count } versiones
    }
episode-row-versions-tip =
    { $count ->
        [one] El mismo episodio de { $count } lanzamiento. Haz clic para elegir cuál se reproduce
       *[other] El mismo episodio de { $count } lanzamientos. Haz clic para elegir cuál se reproduce
    }
episode-row-downloading = Descargando { $percent } %
episode-row-get = Obtener episodio
episode-row-predicted = Previsto
episode-row-extra = Extra
episode-row-resume = Reanudar en { $time }

## EpisodeCubes.qml

episode-cubes-one = Un episodio
episode-cubes-range =
    { $count ->
        [one] { $count } episodio, { $first } a { $last }
       *[other] { $count } episodios, { $first } a { $last }
    }
episode-cubes-tip = Episodio { $number }: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } extra
       *[other] { $count } extras
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } archivo sin reconocer
       *[other] { $count } archivos sin reconocer
    }
episodes-pane-on-disk = En el disco
episodes-pane-not-on-disk = No está en el disco
episodes-pane-season = T{ $season }
episodes-pane-get-tip = Obtener episodio { $number }
episodes-pane-none-on-disk = No hay episodios en el disco
episodes-pane-change-match = Cambiar reconocimiento
episodes-pane-refreshing = Actualizando { $title }

## ShowDetail.qml

show-detail-not-in-library = Esta carpeta aún no está en la biblioteca. Cuando una revisión la encuentre, aquí aparecerán su cartel y sus episodios.
show-detail-film-on-disk = La película está en el disco.
show-detail-film-not-on-disk = La película aún no está en el disco.
show-detail-complete-line = Terminó de emitirse y están aquí los { $total } episodios. No vendrá nada más, así que la suscripción ya no tiene nada que hacer.
show-detail-finished = Terminó de emitirse.
show-detail-airing = En emisión.
show-detail-have-of-total = { $have } de { $total } episodios aquí.
show-detail-have =
    { $count ->
        [one] { $count } episodio aquí.
       *[other] { $count } episodios aquí.
    }
show-detail-next-predicted =
    { $count ->
        [one] { $span } está previsto en { $wait }.
       *[other] { $span } están previstos en { $wait }.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span } se emite en { $wait }.
       *[other] { $span } se emiten en { $wait }.
    }
show-detail-missing =
    { $count ->
        [one] El episodio { $list } no está aquí.
       *[other] Los episodios { $list } no están aquí.
    }
show-detail-missing-more = Los episodios { $list } y { $more } más no están aquí.
show-detail-downloaded = Descargado
show-detail-downloading = Descargando { $percent } %
show-detail-taken-adding = Lo toma la regla, añadiendo
show-detail-taken = Lo toma la regla
show-detail-left-out = Lo descarta la regla
show-detail-subscribed-reading = Suscrito. qBittorrent está leyendo el feed.
show-detail-taken-moving = La regla toma { $taken } de { $releases ->
        [one] { $releases } lanzamiento
       *[other] { $releases } lanzamientos
    } del feed; { $moving ->
        [one] { $moving } se está descargando
       *[other] { $moving } se están descargando
    } abajo.
show-detail-taken-held = La regla toma { $taken } de { $releases ->
        [one] { $releases } lanzamiento
       *[other] { $releases } lanzamientos
    } del feed; { $torrents ->
        [one] { $torrents } torrent está
       *[other] { $torrents } torrents están
    } en el cliente.
show-detail-taken-adding-line = La regla toma { $taken } de { $releases ->
        [one] { $releases } lanzamiento
       *[other] { $releases } lanzamientos
    } del feed; qBittorrent { $taken ->
        [one] lo está añadiendo.
       *[other] los está añadiendo.
    }
show-detail-feed-holds-nothing = El feed aún no tiene nada. Los lanzamientos que encajen se descargan en cuanto aparezcan en él.
show-detail-nothing-fits = Aún no hay nada en el feed que encaje con la regla. Los lanzamientos que encajen se descargan en cuanto aparezcan.
show-detail-complete = Completa
show-detail-open-in-library = Abrir en la biblioteca
show-detail-feed-finds-nothing = Este feed no encuentra nada
show-detail-feed-empty-query = Pide a { $source } “{ $query }”, que vuelve vacío, así que nada nuevo puede llegar a la regla. Una búsqueda más corta suele dar resultados; las palabras que quite pueden quedarse en la regla.
show-detail-feed-empty = Pregunta a { $source }, que vuelve vacío, así que nada nuevo puede llegar a la regla. Una búsqueda más corta suele dar resultados; las palabras que quite pueden quedarse en la regla.
show-detail-feed-empty-earlier = qBittorrent aún muestra lo que encontró una versión anterior de este feed.
show-detail-repairing = Reparando
show-detail-repair = Reparar
show-detail-unsubscribe = Cancelar suscripción
show-detail-remove-torrents =
    { $count ->
        [one] Quitar el torrent
       *[other] Quitar { $count } torrents
    }
show-detail-searches = Busca
show-detail-quoted = “{ $words }”
show-detail-whole-feed = todo el feed
show-detail-from = De
show-detail-rule = Regla
show-detail-rule-unfollowable = No puede obtener episodios nuevos: su fuente responde a búsquedas pero no tiene feed
show-detail-rule-active = Activa
show-detail-keeps = Conserva
show-detail-keeps-value = lanzamientos con “{ $words }”
show-detail-drops = Descarta
show-detail-drops-value = lanzamientos con { $words }
show-detail-or = { $first } o { $second }
show-detail-last-took = Último tomado
show-detail-saves-to = Guarda en
show-detail-folder = Carpeta
show-detail-in-the-feed = En el feed
show-detail-feed-reading = Leyendo el feed.
show-detail-feed-offers = Lo que ofrece ahora el feed, lo más nuevo primero.
show-detail-feed-is-empty = El feed está vacío.
show-detail-read-again = Volver a leer
show-detail-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
show-detail-download = Descargar
show-detail-torrents = Torrents
show-detail-torrents-moving =
    { $count ->
        [one] { $count } descargándose ahora; las barras se llenan según avanza.
       *[other] { $count } descargándose ahora; las barras se llenan según avanzan.
    }
show-detail-torrents-held = Lo que el cliente tiene para esta carpeta.
show-detail-torrents-waiting = Esperando a que qBittorrent añada lo que toma la regla.
show-detail-torrents-none = El cliente no tiene nada para esta carpeta.

## SeriesPage.qml

series-trackers-both = { $first } y { $second }
series-count-cleared = Número de episodios borrado
series-count-set = Fijado en { $count } episodios
series-held-one-undated = Episodio { $first } aún sin fecha
series-held-range-undated = Episodios { $first } a { $last } aún sin fecha
series-held-one-due = Episodio { $first } previsto para { $when }
series-held-range-due = Episodios { $first } a { $last } previstos para { $when }
series-held-month-format = MMMM 'de' yyyy
series-fix-schedule = Episodios y fechas
series-fix-files = Archivos
series-fix-rescan-folder = Releer carpeta
series-fix-match = Reconocer
series-find-match = Reconocer
series-rescan-show = Releer serie
series-open-in-show-info = Abrir en Info de la serie
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } película movida a la papelera
           *[other] { $count } películas movidas a la papelera
        }
        [episode] { $count ->
            [one] { $count } episodio movido a la papelera
           *[other] { $count } episodios movidos a la papelera
        }
       *[file] { $count ->
            [one] { $count } archivo movido a la papelera
           *[other] { $count } archivos movidos a la papelera
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } película elegida.
           *[other] { $count } películas elegidas.
        }
        [episode] { $count ->
            [one] { $count } episodio elegido.
           *[other] { $count } episodios elegidos.
        }
       *[file] { $count ->
            [one] { $count } archivo elegido.
           *[other] { $count } archivos elegidos.
        }
    }
series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] Mover { $count } película a la papelera
           *[other] Mover { $count } películas a la papelera
        }
        [episode] { $count ->
            [one] Mover { $count } episodio a la papelera
           *[other] Mover { $count } episodios a la papelera
        }
       *[file] { $count ->
            [one] Mover { $count } archivo a la papelera
           *[other] Mover { $count } archivos a la papelera
        }
    }
series-delete-click = Haz clic en los archivos que quieras eliminar.
series-delete-where = Van a la papelera, de donde puedes recuperarlos. Lo que viste sigue registrado.
series-delete-move = Mover a la papelera
series-pick-none = Ninguno
series-pick-all = Todos
series-rescan-started = Relectura iniciada
series-rated = Nota { $score }
series-rated-cleared = Nota borrada
series-score-failed = La nota falló
series-still-downloading = { $episode } aún se está descargando
series-format-series = Serie
series-tracker-refused = { $tracker } no lo aceptó
series-tracker-refused-why = { $tracker } no lo aceptó: { $why }
series-watched-up-to = Visto hasta { $episode }
series-unwatched-from = Sin ver desde { $episode }
series-undo = Deshacer
series-play-downloading = Descargando episodio { $number }
series-play-downloading-unnumbered = Descargando episodio
series-play-continue = Continuar
series-play-again = Volver a ver
series-play-continue-episode = Continuar episodio { $number }
series-play-again-from = Volver a ver desde el episodio { $number }
series-play-episode = Ver episodio { $number }
series-page-not-tracked = Sin seguimiento
series-not-tracked-on-disk =
    { $count ->
        [one] Sin seguimiento, { $count } episodio en el disco
       *[other] Sin seguimiento, { $count } episodios en el disco
    }
series-watched = { $count } vistos
series-watched-of = { $watched } de { $total } vistos
series-watched-of-estimate = { $watched } de { $total }+ vistos
series-facts-episodes-estimate =
    { $count ->
        [one] { $count } episodio+
       *[other] { $count } episodios+
    }
series-facts-each = { $runtime } cada uno
series-facts-on-disk = { $count } en el disco
series-next = Siguiente: { $episode }
series-episodes-downloading =
    { $count ->
        [one] { $count } episodio descargándose
       *[other] { $count } episodios descargándose
    }
series-all-watched = Todo visto
series-no-poster = Sin cartel
series-next-season-span = T{ $season } { $span }
series-next-predicted = { $span } previsto para el { $date }, en { $wait }
series-next-date-format = ddd d MMM
series-next-airs =
    { $count ->
        [one] { $span } se emite en { $wait }
       *[other] { $span } se emiten en { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] Descargando { $count } episodio
       *[other] Descargando { $count } episodios
    }
series-count-season-ends = La temporada { $season } termina en el episodio { $number }
series-count-ends = Termina en el episodio { $number }
series-count-gave-way = Fijaste { $count } episodios; ya no se aplica
series-count-set-by-you = { $line }, fijado por ti
series-change = Cambiar
series-get-episodes = Obtener episodios
series-your-score = Tu nota { $score }
series-rate = Puntuar
series-page-average-score = Nota media { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = { $votes } votos
series-add-to-list = Añadir a una lista
series-page-rewatched =
    { $count ->
        [one] Vista otra vez { $count } vez
       *[other] Vista otra vez { $count } veces
    }
series-your-notes = Tus notas
series-add-notes = Añadir notas
series-notes-tip = Notas de tu entrada de lista, guardadas en { $sites }
series-unhide = Mostrar
series-hide = Ocultar
series-hidden-toast = { $title } se oculta aquí y AniBeam deja de sincronizarla con AniList y MyAnimeList
series-unhidden-toast = { $title } ha vuelto y se sincroniza otra vez
series-hide-tip-tracked = Oculta en todas las páginas, y AniBeam deja de sincronizarla con AniList y MyAnimeList
series-hide-tip = Oculta en todas las listas
series-fixes = Corregir
series-fixes-tip = Corrige archivos, el número de episodios, el reconocimiento, o relee
series-quick-fix = Arreglo rápido
series-film = Película
series-episodes = Episodios
series-extras = Openings, endings y más
series-extras-openings = Openings
series-extras-endings = Endings
series-extras-previews = Avances y tráileres
series-extras-specials = Especiales
series-extras-other = Otros
series-extras-group = { $kind }, { $count }
series-extra-files = Archivos de más
series-extra-files-warning =
    { $count ->
        [one] Hay un archivo más que los { $total } episodios esperados: mal nombrado, duplicado o especial. Renómbralo o quítalo.
       *[other] Hay { $count } archivos más que los { $total } episodios esperados: mal nombrados, duplicados o especiales. Renómbralos o quítalos.
    }
series-synopsis = Sinopsis
series-less = Menos
series-more = Más
series-match = Reconocer
series-not-matched = Aún sin reconocer, así que no hay cartel, sinopsis ni seguimiento. Busca su info en AniList o TVmaze.
series-naming-title = Nombres que se reconocen
series-naming-body = La carpeta es el título de la serie, tal como lo escribe AniList, con la temporada detrás si no es la primera. Cada archivo lleva el número de episodio.
series-fix-how-files-read = Corregir cómo se leen
series-characters = Personajes
series-related = Relacionadas
series-related-filling = Aún completándose desde AniList.
series-tags = Etiquetas
series-hide-spoilers = Ocultar spoilers
series-spoilers = Spoilers
series-all-tags = Todas las etiquetas
series-tag = { $name }, { $rank } %
series-recommendations = Recomendaciones
series-relation-owned = En tu biblioteca
series-relation-unreleased = Aún sin estrenar
series-relation-on-anilist = En AniList
series-score-saved = Nota guardada
series-score-cleared = Nota borrada

## GetShow.qml

get-show-the-client = el cliente de torrents
get-show-group-batch = Lote de { $group }
get-show-run = { $first } a { $last }
get-show-episodes =
    { $count ->
        [one] Episodio { $list }
       *[other] Episodios { $list }
    }
get-show-episodes-inline =
    { $count ->
        [one] episodio { $list }
       *[other] episodios { $list }
    }
get-show-sending-to = Enviando a { $client }
get-show-go =
    { $count ->
        [one] Obtener { $count } episodio
       *[other] Obtener { $count } episodios
    }
get-show-go-replace =
    { $count ->
        [one] Obtener { $count } episodio y reemplazar
       *[other] Obtener { $count } episodios y reemplazar
    }
get-show-go-subscribe =
    { $count ->
        [one] Obtener { $count } episodio y suscribirse
       *[other] Obtener { $count } episodios y suscribirse
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] Obtener { $count } episodio, reemplazar y suscribirse
       *[other] Obtener { $count } episodios, reemplazar y suscribirse
    }
get-show-subscribe = Suscribirse a episodios nuevos
get-show-nothing-new = Nada nuevo que obtener
get-show-pick-episode = Elige un episodio
get-show-replace-off = Desactivado. Lo elegido para { $episodes } se omite, porque ya está en la biblioteca. Actívalo para cambiarlo.
get-show-replace-on = Los archivos que tienes de { $episodes } van a la papelera cuando terminen los nuevos.
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } se está descargando ahora: esa descarga se cancela y lo que escribió va a la papelera.
       *[other] { $episodes } se están descargando ahora: esa descarga se cancela y lo que escribió va a la papelera.
    }
get-show-kind-on-demand = Se inicia cuando una búsqueda lo necesita
get-show-kind-rss-search = Búsqueda RSS
get-show-kind-rss-feed = Feed RSS
get-show-kind-json-search = Búsqueda JSON
get-show-kind-indexer-search = Búsqueda en indexador
get-show-ready = Lista
get-show-running = En marcha
get-show-not-running = Detenida
get-show-not-running-start = Detenida. Haz clic en su fuente, arriba, para iniciarla.
get-show-switched-off = Desactivada
get-show-click-to-see = Haz clic para ver qué tiene de esta serie
get-show-searching = Buscando
get-show-searching-source = Buscando en { $name }
get-show-searching-elapsed = { $label }, { $seconds } s
get-show-releases =
    { $count ->
        [one] { $count } lanzamiento
       *[other] { $count } lanzamientos
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } lanzamiento, cubre { $cover } de { $total } episodios
       *[other] { $count } lanzamientos, cubren { $cover } de { $total } episodios
    }
get-show-on-your-list = En tu lista
get-show-plan-to-watch = Quiero verlo
get-show-on-your-list-tip = Está en tu lista
get-show-plan-to-watch-tip = Ponerla en tu lista Planeado
get-show-open-series-tip = La página de la serie: sus detalles, episodios y reproducción
get-show-next-in = Episodio { $number } en { $wait }
get-show-airing = En emisión
get-show-finished = Finalizada
get-show-total-episodes = { $count } episodios
get-show-downloading-episodes =
    { $count ->
        [one] Descargando { $count } episodio
       *[other] Descargando { $count } episodios
    }
get-show-downloading-torrents =
    { $count ->
        [one] Descargando { $count } torrent
       *[other] Descargando { $count } torrents
    }
get-show-in-library = En tu biblioteca
get-show-not-in-library = No está en tu biblioteca
get-show-downloading = Descargando
get-show-stop-all = Detener todo
get-show-start-all = Iniciar todo
get-show-stop-all-tip = Detener todas las descargas de esta serie; lo que tienen se queda
get-show-start-all-tip = Reanudar todas las descargas de esta serie
get-show-cancel-all = Cancelar todo
get-show-cancel-all-tip = Cancelar todas las descargas de esta serie y mover lo que escribieron a la papelera
get-show-sources = Fuentes disponibles
get-show-check-again = Volver a comprobar
get-show-check-again-tip = Pregunta a cada proveedor bajo demanda si está en marcha. No inicia nada.
get-show-search-source = Buscar en { $name }
get-show-release-list = Todos los lanzamientos en lista
get-show-your-library = Tu biblioteca
get-show-library-on-disk = { $have } de { $total } en el disco
get-show-library-downloading = { $count } descargándose
get-show-library-nothing = Aún nada
get-show-row-batch = { $group }, lote
get-show-batch-counting = contando sus episodios, un torrent
get-show-batch-unlisted = episodios sin listar, toda la serie por su título, un torrent
get-show-batch-episodes =
    { $count ->
        [one] { $count } episodio, un torrent
       *[other] { $count } episodios, un torrent
    }
get-show-nothing-new-picked = No hay nada nuevo elegido
get-show-nothing-picked = No hay nada elegido
get-show-pick-provider = Elige un proveedor
get-show-plan-line = { $episodes } de { $groups } en { $source }.
get-show-and = { $first } y { $second }
get-show-library-has =
    { $count ->
        [one] La biblioteca tiene { $episodes }. Activa Reemplazar episodios actuales, abajo, para cambiarlo.
       *[other] La biblioteca tiene { $episodes }. Activa Reemplazar episodios actuales, abajo, para cambiarlos.
    }
get-show-together = en total
get-show-torrents-unit =
    { $count ->
        [one] torrent
       *[other] torrents
    }
get-show-replace = Reemplazar episodios actuales
get-show-sequential = Descargar en orden
get-show-sequential-on = Activado: cada torrent nuevo descarga sus piezas desde el principio, así que los primeros episodios terminan antes y puedes verlos mientras llega el resto. Un poco más lento en total.
get-show-sequential-off = Desactivado: el cliente descarga las piezas en el orden en que las ofrece el enjambre.
get-show-rolling = Tener listos los siguientes
get-show-rolling-on = Activado: se descargan { $count } episodios ahora, y cada uno que termines inicia el siguiente.
get-show-rolling-on-delete = Activado: se descargan { $count } episodios ahora, y cada uno que termines inicia el siguiente. Los episodios vistos salen del disco.
get-show-rolling-off = Desactivado: todos los episodios elegidos se descargan a la vez.
get-show-follow-off = Desactivado: los episodios nuevos aparecen aquí como huecos y nada se descarga solo.
get-show-follow-no-feed = { $source } no tiene feed que seguir, así que los episodios nuevos no pueden descargarse solos desde ahí.
get-show-follow-pick-group = Elige antes un grupo: una suscripción sigue un grupo y una calidad.
get-show-follow-batch-only = { $group } solo tiene aquí el lote, así que no hay feed de episodios sueltos al que suscribirse.
get-show-follow-on = Activado: un feed de { $feed } en { $source }. Cada episodio nuevo se descarga cuando aparece.
get-show-confirm =
    { $count ->
        [one] { $episodes } se está descargando ahora. Reemplazarlo cancela esa descarga y mueve lo que escribió a la papelera.
       *[other] { $episodes } se están descargando ahora. Reemplazarlos cancela esa descarga y mueve lo que escribió a la papelera.
    }
get-show-confirm-go = Cancelar y reemplazar
get-show-state-error = Error
get-show-state-files-missing = Faltan archivos
get-show-state-stopped = Detenida
get-show-state-waiting-peers = Esperando pares
get-show-state-metadata = Obteniendo metadatos
get-show-state-queued = En cola
get-show-state-checking = Comprobando archivos
get-show-state-allocating = Reservando espacio
get-show-state-moving = Moviendo archivos
get-show-state-waiting = Esperando
get-show-a-batch = Un lote
get-show-no-group = Grupo no indicado en el nombre
get-show-likely = probablemente { $source }
get-show-tracker = tracker { $tracker }
get-show-not-known = Aún no se sabe
get-show-bytes-of = { $done } de { $size }
get-show-speed-down = { $speed }/s bajada
get-show-time-left = faltan { $wait }
get-show-stopping = Deteniendo
get-show-starting = Iniciando
get-show-stop = Detener
get-show-start-tip = Reanudar la descarga
get-show-stop-tip = Dejar de descargar; lo que tiene se queda
get-show-cancelling = Cancelando
get-show-cancel-tip = Cancelar esta descarga y mover lo que escribió a la papelera
get-show-episode-order = Descargar en orden de episodios
get-show-episode-order-on = Activado: las piezas llegan desde el principio de cada archivo, así que los primeros episodios terminan antes.
get-show-cube-skipped = Episodio { $number }: omitido
get-show-cube-done = Episodio { $number }: hecho
get-show-cube-percent = Episodio { $number }: { $percent } %
get-show-fact-torrent = Torrent
get-show-fact-release = Lanzamiento
get-show-fact-from = De
get-show-fact-saving-to = Guardando en
get-show-span-counting = Contando
get-show-span-unknown = { $runs }?
get-show-span-episodes = Episodios { $runs }
get-show-cube-no-release = Episodio { $number }: sin lanzamiento de { $group }
get-show-seeders = { $count } seeders
get-show-cube-on-disk = ya está en el disco, haz clic para reemplazarlo
get-show-cube-downloading = descargándose ahora, haz clic para reemplazarlo

## TorrentRow

torrent-row-state-downloading = Descargando
torrent-row-state-stalled = Atascado
torrent-row-state-metadata = Obteniendo metadatos
torrent-row-state-queued = En cola
torrent-row-state-stopped = Detenido
torrent-row-state-done = Hecho
torrent-row-state-seeding = Compartiendo
torrent-row-state-checking = Comprobando
torrent-row-state-moving = Moviendo
torrent-row-state-error = Error
torrent-row-state-files-missing = Faltan archivos
torrent-row-state-making-room = Haciendo sitio
torrent-row-state-finding-peers = Buscando pares
torrent-row-state-waiting = Esperando
torrent-row-done-of-size = { $done } de { $size }
torrent-row-speed-down = { $speed }/s bajada
torrent-row-speed-up = { $speed }/s subida
torrent-row-time-left = faltan { $time }
torrent-row-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
torrent-row-rolling = continua, episodios { $next } a { $through } de { $total }
torrent-row-stop = Detener

## SourceRow

source-row-unavailable-fix = Conecta su disco y pulsa Releer. Si se movió, quítala y añade la nueva ubicación.
source-row-counts = { $count } series
source-row-counts-movies =
    { $movies ->
        [one] { $count } series, { $movies } carpeta de películas
       *[other] { $count } series, { $movies } carpetas de películas
    }
source-row-unavailable = No disponible
    .tip = AniBeam no puede acceder a esta carpeta. Si está en un disco que no está conectado o montado, conéctalo y AniBeam la recuperará. Si se movió, quítala aquí y añade la nueva ubicación.
source-row-rescan = Releer
source-row-remove-question = ¿Quitar { $name }, { $count } series y su historial?

## StartSourceSheet

start-source-title = ¿Iniciar { $service } para { $name }?
start-source-busy = Iniciando { $service }. { $name } se activa cuando responda; puede tardar hasta un minuto.
start-source-body = { $service } es el servicio que busca en { $name } para AniBeam. Iniciarlo lo ejecuta en segundo plano en este ordenador, y sigue en marcha después de cerrar AniBeam.
start-source-starting = Iniciando

## SearchProgress

search-progress-why-helper = AniBeam está iniciando su ayudante de búsqueda; tarda un segundo o dos.
search-progress-why-starting = { $endpoint } funciona con un ayudante que se inicia cuando una búsqueda lo necesita. La primera búsqueda lo espera, hasta { $seconds } segundos; las siguientes se lo saltan.
search-progress-why-reading = Separando los lanzamientos en grupos y comparándolos con la biblioteca.
search-progress-why-ondemand = Un ayudante lee { $endpoint } desde cero en cada búsqueda nueva, lo que suele tardar de diez a veinte segundos.
search-progress-why-slow = { $endpoint } tarda en responder ahora mismo.
search-progress-why-usual = Suele tardar un segundo o dos.
search-progress-why-and-slow = { $why } Está tardando más de lo normal; cancela para probar otra fuente.
search-progress-searching = Buscando
search-progress-seconds = { $seconds } s
search-progress-try = Probar { $endpoint }

## ReleaseTable

release-table-col-release = Lanzamiento
release-table-col-group = Grupo
release-table-col-res = Res
release-table-col-rip = Rip
release-table-col-ep = Ep
release-table-col-size = Tamaño
release-table-col-seeders = Seeders
release-table-col-leechers = Leechers
release-table-col-downloads = Descargas
release-table-col-released = Publicado
release-table-tick-all = Marcar todos los lanzamientos
release-table-untick-all = Desmarcar todos los lanzamientos
release-table-on-disk = En el disco
release-table-other-on-disk = Otro lanzamiento en el disco
release-table-season-batch = Lote de la temporada { $season }
release-table-batch = Lote
release-table-via = vía { $source }
release-table-season-episode = T{ $season } { $episode }
release-table-date-format = d 'de' MMMM 'de' yyyy, HH:mm
release-table-download = Descargar
release-table-more = y { $count } más. Acota la lista u ordénala para ver otros primero.
release-table-pick = Elegir este lanzamiento

## DownloadsPage

downloads-title = Descargas
downloads-builtin-client = Cliente integrado
downloads-start-done = { $name } está en marcha y activado
downloads-disk-film = Película
downloads-disk-watched = { $size } vistos
downloads-disk-all-watched = todo visto
downloads-delete-watched-menu =
    { $count ->
        [one] Todos los episodios vistos: { $count } episodio, { $size }
       *[other] Todos los episodios vistos: { $count } episodios, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Todas las series terminadas que viste: { $count } serie, { $size }
       *[other] Todas las series terminadas que viste: { $count } series, { $size }
    }
downloads-titles-and = { $list } y { $last }
downloads-titles-more = { $list } y { $count } más
downloads-delete-finished-none = Ninguna serie terminada está vista entera
downloads-delete-watched-none = No hay nada visto en el disco
downloads-delete-finished-heading =
    { $count ->
        [one] ¿Eliminar { $count } serie terminada, { $size }?
       *[other] ¿Eliminar { $count } series terminadas, { $size }?
    }
downloads-delete-finished-base = { $titles }. Solo series que el proveedor da por terminadas, con todos los episodios vistos; una serie aún en emisión, o cuyo final es una suposición, se queda. Todos sus archivos van a la papelera, también los extras. Sus torrents salen también de { $client }, así que se pueden volver a descargar.
downloads-delete-watched-heading =
    { $count ->
        [one] ¿Eliminar { $count } episodio visto, { $size }?
       *[other] ¿Eliminar { $count } episodios vistos, { $size }?
    }
downloads-delete-watched-base = Todos los episodios que has visto, de todas las series. Los episodios aún sin ver y los extras se quedan. Los archivos y sus subtítulos van a la papelera, así que puedes recuperarlos de ahí. Sus torrents salen también de { $client }, así que se pueden volver a descargar.
downloads-move-to-trash = Mover a la papelera
downloads-remove-group-torrents =
    { $count ->
        [one] Quitar el torrent
       *[other] Quitar sus { $count } torrents
    }
downloads-run-span = { $from } a { $to }
downloads-run-pair = { $first } y { $second }
downloads-gap-missing =
    { $count ->
        [one] Falta el episodio { $runs }
       *[other] Faltan los episodios { $runs }
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Episodio { $runs } sin descargar
       *[other] Episodios { $runs } sin descargar
    }
downloads-gap-then-not-downloaded = { $runs } sin descargar
downloads-trying-query = Probando { $query }
downloads-client-settings = Ajustes del cliente de torrents
downloads-error-unknown = Algo falló. Inténtalo de nuevo.
downloads-search-all-off = Todas las fuentes están desactivadas. Activa una para buscar.
downloads-turn-on = Activar { $name }
downloads-search-no-sources = Aún no hay fuentes donde buscar. Añade una lista de fuentes en Ajustes, en Descargas.
downloads-search-none-started = Ninguna fuente pudo iniciar la búsqueda. Inténtalo de nuevo en un momento.
downloads-auto-resolution = Acotado a { $resolution }, la mejor disponible
downloads-search-one-failed = La búsqueda en { $name } no funcionó. Inténtalo de nuevo, o busca en otra fuente.
downloads-search-all-failed = Ninguna fuente pudo buscar. Inténtalo de nuevo en un momento.
downloads-narrowing-on = Acotando por { $name }
downloads-refine-zero = Eso no dejaría nada, así que la lista se queda como está.
downloads-refine-no-match = Ningún lanzamiento coincide con ese término.
downloads-refine-empty = Escribe un término para acotar.
downloads-refine-no-change = Eso no cambiaría nada.
downloads-movie-tick-one = Marca un lanzamiento y pulsa Descargar como película.
downloads-group-downloading = Descargando
downloads-group-downloading-line = En marcha; Actividad, a la derecha, tiene el detalle.
downloads-group-seeding = Compartiendo
downloads-group-seeding-line = Descargados y aún compartiéndose.
downloads-group-done = Hecho
downloads-group-done-line = Descargados; el cliente no los está compartiendo.
downloads-group-stopped = Detenidos
downloads-group-stopped-line = Detenidos antes de terminar.
downloads-group-broken = Faltan archivos
downloads-group-broken-line = El cliente no encuentra sus archivos: apuntan a una carpeta que no existe. Quitar los torrents ordena el cliente.
downloads-sub-nofeed = No se puede seguir
downloads-sub-nofeed-line = Su fuente responde a búsquedas pero no tiene feed, así que no llegan episodios nuevos. Cancela la suscripción y sigue la serie en una fuente que lo tenga.
downloads-sub-complete = Completas
downloads-sub-complete-line = Todos los episodios están en la biblioteca y no se emitirá nada más: estas suscripciones ya no tienen nada que hacer.
downloads-sub-airing = En emisión
downloads-sub-airing-line = Aún quedan episodios nuevos, los más próximos primero.
downloads-sub-incomplete = Incompletas
downloads-sub-incomplete-line = Terminaron de emitirse con episodios que no están aquí, o nada indica cómo va la serie.
downloads-sub-missing = No están en la biblioteca
downloads-sub-missing-line = Su carpeta no tiene nada que la biblioteca haya encontrado: aún no ha llegado nada, o guarda donde la biblioteca no mira.
downloads-torrents =
    { $count ->
        [one] { $count } torrent
       *[other] { $count } torrents
    }
downloads-rule-film-on-disk = Película, en el disco
downloads-rule-all-here = Los { $total } aquí
downloads-rule-next =
    { $predicted ->
        [yes] Ep { $episode } previsto en { $time }
       *[no] Ep { $episode } en { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] Ep { $episode } a { $through } previstos en { $time }
       *[no] Ep { $episode } a { $through } en { $time }
    }
downloads-rule-have-of = { $have } de { $total } aquí
downloads-rule-have =
    { $count ->
        [one] { $count } episodio aquí
       *[other] { $count } episodios aquí
    }
downloads-tag-not-downloaded = Sin descargar
downloads-tag-missing = Faltan { $count }
downloads-tag-n-not-downloaded = { $count } sin descargar
downloads-tag-unknown = Desconocido
downloads-speed-down = { $speed }/s bajada
downloads-time-left = faltan { $time }
downloads-repair-trying = Probando búsquedas más cortas
downloads-sent-to-client = { $title } enviado a { $client }
downloads-files-torrents = lo que descargaron los torrents
downloads-files-torrents-remote = lo que descargaron los torrents, eliminado por { $client }
downloads-files-folder-and = { $folder } y { $torrents }
downloads-unsubscribe-heading = ¿Cancelar la suscripción a { $title }?
downloads-unsubscribe-base = Su feed y su regla de descarga salen de { $client }, así que no llega nada nuevo.
downloads-unsubscribe-complete-heading = ¿Cancelar la suscripción a las { $count } series completas?
downloads-unsubscribe-complete-base = Sus feeds y reglas de descarga salen de { $client }. Todos sus episodios están en la biblioteca.
downloads-files-folders-and-torrents = sus carpetas y lo que descargaron sus torrents
downloads-one-at-a-time = Se quitan de una en una; la página muestra cada una según avanza.
downloads-subscription-stays = { $base } La suscripción se queda.
downloads-cancel-heading = ¿Cancelar { $title }?
downloads-cancel-group-base =
    { $count ->
        [one] Su descarga se detiene y sale de { $client }.
       *[other] Sus { $count } descargas se detienen y salen de { $client }.
    }
downloads-files-so-far-it = lo que ha descargado hasta ahora
downloads-files-so-far-they = lo que han descargado hasta ahora
downloads-cancel-download = Cancelar la descarga
downloads-keep-downloading = Seguir descargando
downloads-remove-heading = ¿Quitar { $title }?
downloads-remove-group-base =
    { $count ->
        [one] Su torrent sale de { $client }; los archivos se quedan salvo que marques abajo.
       *[other] Sus { $count } torrents salen de { $client }; los archivos se quedan salvo que marques abajo.
    }
downloads-remove-detail-base =
    { $count ->
        [one] Su { $count } torrent sale de { $client }; los archivos se quedan salvo que marques abajo.
       *[other] Sus { $count } torrents salen de { $client }; los archivos se quedan salvo que marques abajo.
    }
downloads-remove-broken-heading = ¿Quitar los torrents de las { $count } series a las que les faltan archivos?
downloads-remove-broken-base = Sus { $count } torrents salen de { $client }. Apuntan a carpetas que no existen, así que no se toca ningún archivo; las suscripciones se quedan.
downloads-remove-torrent-heading = ¿Quitar este torrent?
downloads-files-it-downloaded = lo que descargó
downloads-cancel-torrent-heading = ¿Cancelar esta descarga?
downloads-cancel-many-heading =
    { $count ->
        [one] ¿Cancelar { $count } descarga?
       *[other] ¿Cancelar { $count } descargas?
    }
downloads-cancel-many-base = Todas las descargas de esta serie se detienen y salen de { $client }.
downloads-cancel-all = Cancelarlas todas
downloads-summary-connecting = Conectando.
downloads-summary-client-on = { $client } { $version } está conectado.
downloads-summary-client-off = { $client } no está conectado. Revisa su dirección y contraseña en Ajustes, en Descargas.
downloads-summary-native = AniBeam busca en tus fuentes por sí mismo.
downloads-summary-native-with = AniBeam busca en tus fuentes por sí mismo. { $client }
downloads-summary-starting = Iniciando las descargas.
downloads-summary-not-running = Las descargas no están en marcha.
downloads-summary-not-started = Las descargas aún no se han iniciado.
downloads-summary-helper-ready = El ayudante de búsqueda { $version } está listo. { $client }
downloads-subscribed-as = Suscrito como { $name }.
downloads-repaired =
    { $count ->
        [one] { $name } reparado: el feed encuentra { $count } lanzamiento
       *[other] { $name } reparado: el feed encuentra { $count } lanzamientos
    }
downloads-repaired-keeps =
    { $count ->
        [one] { $name } reparado: el feed encuentra { $count } lanzamiento y la regla conserva “{ $term }”
       *[other] { $name } reparado: el feed encuentra { $count } lanzamientos y la regla conserva “{ $term }”
    }
downloads-sent-torrents =
    { $count ->
        [one] { $count } torrent enviado a { $client }.
       *[other] { $count } torrents enviados a { $client }.
    }
downloads-added-torrents =
    { $count ->
        [one] { $count } torrent añadido.
       *[other] { $count } torrents añadidos.
    }
downloads-trashed-paths =
    { $count ->
        [one] { $count } ruta de { $title } movida a la papelera
       *[other] { $count } rutas de { $title } movidas a la papelera
    }
downloads-show-gone = Esa serie ya no está en la biblioteca
downloads-activity-hide = Ocultar actividad
downloads-activity = Actividad
downloads-moving =
    { $count ->
        [one] { $count } descarga en marcha
       *[other] { $count } descargas en marcha
    }
downloads-back = Volver a Descargas
downloads-chip-search-ready = Búsqueda lista
downloads-chip-search-starting = Iniciando búsqueda
downloads-chip-search-off = Búsqueda desactivada
downloads-chip-answers-at = { $summary } Responde en { $url }.
downloads-chip-helper-missing = El ayudante de búsqueda no está instalado. Haz clic para ir a los ajustes de Descargas, que explican cómo añadirlo
downloads-chip-helper-failed = El ayudante de búsqueda se detuvo. Reconectar lo vuelve a iniciar; los ajustes de Descargas dicen qué falló
downloads-chip-open-settings = Abrir los ajustes de Descargas
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } desactivado
downloads-chip-client-not-connected = { $client } no está conectado. Reconecta, o revisa su dirección en Ajustes
downloads-chip-client-remote = Un ayudante de búsqueda que ejecutas tú usa el cliente que indican sus propios ajustes
downloads-chip-client-switch = Haz clic para cambiar de cliente
downloads-connecting = Conectando
downloads-reconnect = Reconectar
downloads-search-placeholder = Busca una serie o el nombre de un lanzamiento
downloads-search-go-tip = Buscar nombres de lanzamientos en { $names }
downloads-search-go-off = Todas las fuentes están desactivadas.
downloads-suggest-airing =
    { $predicted ->
        [yes] En emisión, episodio { $episode } previsto en { $time }
       *[no] En emisión, episodio { $episode } en { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] En emisión, episodios { $episode } a { $through } previstos en { $time }
       *[no] En emisión, episodios { $episode } a { $through } en { $time }
    }
downloads-suggest-finished = Finalizada
downloads-suggest-episodes = { $count } episodios
downloads-suggest-in-library = En tu biblioteca
downloads-suggest-on-anilist = En AniList, no en tu biblioteca
downloads-plan-to-watch = Quiero verlo
downloads-suggest-release-search = Buscar en su lugar nombres de lanzamientos con “{ $query }” en { $names }
downloads-suggest-release-search-none = Buscar en su lugar nombres de lanzamientos con “{ $query }” en ninguna fuente
downloads-key-return = Intro
downloads-sources = Fuentes
downloads-source-hint-starting = iniciando
downloads-source-hint-down = detenida
downloads-source-tip-down = { $name } necesita un servicio auxiliar que no está en marcha. Haz clic para iniciarlo.
downloads-source-tip-on = Las búsquedas preguntan a { $name }. Haz clic para desactivarla.
downloads-source-tip-off = Desactivada. Haz clic para volver a preguntar a { $name }.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Comprobar
    .tip = Pregunta si cada fuente bajo demanda está en marcha. No inicia nada.
downloads-check-busy = Comprobando
downloads-back-to-grid = Volver a la cuadrícula
    .tip = Vuelve a { $title } y a lo que encontró cada proveedor. Escape hace lo mismo.
downloads-releases-on =
    { $count ->
        [one] { $count } lanzamiento en { $name }
       *[other] { $count } lanzamientos en { $name }
    }
downloads-found-on = Encontrado en
downloads-found-searching = buscando
downloads-found-failed = falló
downloads-found-cancelled = cancelada
downloads-releases =
    { $count ->
        [one] { $count } lanzamiento
       *[other] { $count } lanzamientos
    }
downloads-found-retry-tip =
    { $why }
    Haz clic para reintentar
downloads-facet-groups = Grupo
downloads-facet-quality = Calidad
downloads-facet-words = Palabras
downloads-filters-fewer = Menos filtros
downloads-filters-more = Más filtros ({ $count })
downloads-narrow-placeholder = Acota con un término, o -término para descartar
downloads-best-match = Mejor coincidencia
    .tip = Conserva los lanzamientos que se parecen al mejor: { $query }
downloads-best-match-none = Ningún lanzamiento destaca aquí
downloads-strip-have-of = { $have } de { $total } episodios en el disco
downloads-strip-have =
    { $count ->
        [one] { $count } episodio en el disco
       *[other] { $count } episodios en el disco
    }
downloads-strip-next =
    { $predicted ->
        [yes] episodio { $episode } previsto en { $time }
       *[no] episodio { $episode } en { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] episodios { $episode } a { $through } previstos en { $time }
       *[no] episodios { $episode } a { $through } en { $time }
    }
downloads-strip-complete = completa
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = Ninguno de estos lanzamientos es de un episodio que ya tengas.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } lanzamiento de aquí es el archivo que tienes, marcado En el disco
       *[other] { $count } lanzamientos de aquí son el archivo que tienes, marcados En el disco
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } lanzamiento es de un episodio que tienes de otro lanzamiento
       *[other] { $count } lanzamientos son de un episodio que tienes de otro lanzamiento
    }
downloads-strip-both = { $first }; { $second }.
downloads-tick-missing =
    { $count ->
        [one] Marcar el episodio que falta
       *[other] Marcar los { $count } episodios que faltan
    }
    .tip = Para cada episodio de aquí que no está en la biblioteca, el lanzamiento con más seeders
downloads-open-in-library = Abrir en la biblioteca
downloads-not-this-show = No es esta serie
downloads-no-releases = Sin lanzamientos
downloads-no-releases-body = Nada en { $name } coincide con “{ $query }”. Otra fuente, o menos palabras, quizá lo encuentren.
downloads-try-source = Probar { $name }
downloads-ticked =
    { $count ->
        [one] { $count } lanzamiento marcado
       *[other] { $count } lanzamientos marcados
    }
downloads-nothing-ticked = No hay nada marcado
downloads-ticked-size = { $size } en total.
downloads-tick-hint = Marca lanzamientos en la tabla, o descárgalos todos.
downloads-tick-all = Marcar todos
downloads-working = Trabajando
downloads-download-ticked =
    { $count ->
        [one] Descargar { $count } lanzamiento
       *[other] Descargar { $count } lanzamientos
    }
downloads-download-shown = Descargar los { $count } visibles
downloads-download-all = Descargar los { $count }
downloads-download-movie = Descargar como película
    .tip = Un lanzamiento marcado, directo a la carpeta de películas
downloads-subscribe = Suscribirse
downloads-subscribe-body = Los lanzamientos nuevos que coincidan con esta búsqueda se descargan cuando aparezcan, en la carpeta indicada aquí.
downloads-name-placeholder = Tomado del primer lanzamiento
downloads-subscribe-search = Suscribirse a esta búsqueda
downloads-subscribe-no-feed = Aquí no hay feed al que suscribirse
downloads-preview-again = Volver a mostrar la regla
downloads-preview-show = Mostrar la regla que crea
downloads-preview-from = De { $host }
downloads-preview-from-named = De { $host } ({ $name })
downloads-preview-must = Debe contener { $terms }
downloads-preview-must-not = No debe contener { $terms }
downloads-helper-changed = Descargas usa un ayudante que no responde
downloads-helper-missing = Esta copia de AniBeam vino sin descargas
downloads-helper-failed = Descargas no pudo iniciarse
downloads-helper-changed-body = Las búsquedas y descargas pasan por un pequeño programa ayudante. Los ajustes apuntan a uno distinto del de AniBeam, y no responde. Vuelve al de AniBeam y se iniciará solo.
downloads-helper-missing-body = Se compiló sin el programa ayudante por el que pasan las búsquedas y descargas. Los instaladores de la página de versiones lo traen. Ajustes, Descargas explica cómo añadirlo a esta copia.
downloads-helper-failed-body = El programa ayudante por el que pasan las búsquedas y descargas se detuvo antes de estar listo. Volver a iniciarlo lo arregla casi siempre.
downloads-helper-use-own = Usar el de AniBeam
downloads-helper-start-again = Volver a iniciar
downloads-helper-show-how = Enséñame cómo
downloads-helper-what-went-wrong = Qué falló
downloads-lens-gaps-count = Huecos en tu biblioteca  { $count }
downloads-lens-airing-count = En emisión  { $count }
downloads-lens-following-count = Siguiendo  { $count }
downloads-lens-downloaded-count = Descargado  { $count }
downloads-lens-gaps = Huecos en tu biblioteca
downloads-lens-gaps-line = Episodios emitidos que no están en el disco. Abre una serie para ver quién los tiene.
downloads-lens-gaps-none = No falta nada: todos los episodios emitidos de todas las series están en el disco.
downloads-lens-airing-line = Series de tu biblioteca con episodios aún por venir. Abre una para seguirla.
downloads-lens-airing-none = Nada de tu biblioteca está en emisión
downloads-lens-airing-none-body = Una serie aparece aquí cuando está en tu biblioteca y le quedan episodios por venir. En emisión muestra todo lo que se emite.
downloads-airing-now = En emisión
downloads-lens-following = Siguiendo
downloads-card-next =
    { $predicted ->
        [yes] Episodio { $episode } previsto en { $time }
       *[no] Episodio { $episode } en { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Episodios { $episode } a { $through } previstos en { $time }
       *[no] Episodios { $episode } a { $through } en { $time }
    }
downloads-rules-unread = No se pudieron leer tus suscripciones.
downloads-lens-following-line = Feeds que vigila AniBeam, según las series que llenan.
downloads-none-yet = Aún ninguna.
downloads-reading = Leyendo.
downloads-rules-error = AniBeam no pudo leer tus suscripciones. Inténtalo de nuevo en un momento.
downloads-removing-to-go = Quitando, faltan { $count }
downloads-unsubscribe-all = Cancelar las { $count }
downloads-unsubscribe = Cancelar suscripción
downloads-no-rules = Sin suscripciones
downloads-no-rules-body = Busca arriba y suscríbete para que una serie siga llegando.
downloads-search = Buscar
downloads-lens-downloaded = Descargado
downloads-dl-line = { $size } en el disco. { $rest }
downloads-dl-client-off = { $client } no está conectado.
downloads-dl-unread = No se pudo leer la lista de descargas.
downloads-dl-in-client =
    { $count ->
        [one] { $count } torrent, { $size } en { $client }.
       *[other] { $count } torrents, { $size } en { $client }.
    }
downloads-usage-watched = Visto
downloads-usage-not-watched = Sin ver
downloads-deleting = Eliminando
downloads-delete-watched = Eliminar lo visto
downloads-remove-all = Quitar los { $count }
downloads-on-disk = En el disco
downloads-on-disk-line = En tu biblioteca sin torrent en { $client }, los más grandes primero. Haz clic derecho en uno para eliminarlo.
downloads-nothing-downloaded = Nada descargado
downloads-nothing-downloaded-body = Lo que descargues o a lo que te suscribas aparece aquí, serie a serie.
downloads-client-off-title = { $client } no está conectado
downloads-client-off-body = Comprueba que esté en marcha y vuelve a conectar. Su dirección y contraseña están en Ajustes, en Descargas.
downloads-connect-again = Volver a conectar
downloads-view-grid = Cuadrícula
downloads-view-list = Lista
downloads-chip-subscribed = Suscrito
downloads-speed-up = { $speed }/s subida
downloads-broken-some = { $count } con archivos que faltan
downloads-tag-watched = Visto

## PlayerDelays.qml

player-delays-title = Retardo
player-delays-subtitles = Subtítulos
player-delays-audio = Audio
player-delays-subtitles-earlier = Subtítulos 0,1 s antes
player-delays-subtitles-later = Subtítulos 0,1 s después
player-delays-audio-earlier = Audio 0,1 s antes
player-delays-audio-later = Audio 0,1 s después
player-delays-reset = Volver a 0 hasta el próximo archivo
player-delays-none = Sin retardo

## TrackPicker.qml

track-picker-off = Ninguno

## RollControls.qml

roll-controls-window = Manteniendo { $ahead } listos: episodios { $next } a { $through } de { $total }
roll-controls-window-watched = Manteniendo { $ahead } listos: episodios { $next } a { $through } de { $total }, { $watched } vistos
roll-controls-more = Obtener { $count } más
roll-controls-more-tip = Descarga ahora los siguientes episodios. AniBeam vuelve a mantener listos unos pocos cuando te pongas al día.
roll-controls-fewer-tip = Mantener uno menos por delante
roll-controls-ahead = { $count } por delante
roll-controls-further-tip = Mantener uno más por delante
roll-controls-all = Obtener todos
roll-controls-all-tip = Dejar de mantener unos pocos listos y descargar todos los episodios que quedan
roll-controls-all-question = ¿Descargar los { $count } episodios que quedan?

## PlayerChrome.qml

player-chrome-delays = Retardo de subtítulos y audio
player-chrome-audio = Pista de audio
player-chrome-subtitles = Subtítulos
player-chrome-subtitles-off = Subtítulos desactivados
player-chrome-casting-to = Enviando a { $device }
player-chrome-cast = Enviar a una tele
player-chrome-mark-watched = Marcar como visto
player-chrome-shortcuts = Atajos de teclado
player-chrome-fullscreen = Pantalla completa
player-chrome-exit-fullscreen = Salir de pantalla completa
player-chrome-stay = Quedarse
player-chrome-skip-intro = Saltar intro
player-chrome-skip-outro = Saltar outro
player-chrome-playing-on = Reproduciendo en { $device }
player-chrome-replay = Repetir

## PlayerPage.qml

player-title = Reproductor
player-next = Siguiente
player-extra-no-next = Un extra no tiene episodio siguiente
player-last-episode = Este es el último episodio
player-extra-no-previous = Un extra no tiene episodio anterior
player-first-episode = Este es el primer episodio
player-resumed-at = Reanudado en { $time }
player-start-over = Empezar de nuevo
player-audio-track = Audio: { $track }
player-subtitles-track = Subtítulos: { $track }
player-subtitles-on = Subtítulos: activados
player-subtitles-off = Subtítulos desactivados
player-no-subtitles = Este archivo no tiene subtítulos
player-delay-seconds = { $value } s
player-audio-delay = Retardo de audio { $value }
player-subtitle-delay = Retardo de subtítulos { $value }
player-volume = Volumen { $volume }
player-speed = Velocidad { $speed }×
player-not-while-casting = No mientras se envía a otro dispositivo
player-hold-fast = 2× mientras lo mantienes
player-no-chapters = Este archivo no tiene capítulos
player-last-chapter = Este es el último capítulo
player-chapter = Capítulo { $number }
player-chapter-titled = Capítulo { $number }: { $title }
player-screenshot-saved = Guardada en Imágenes/AniBeam
player-skipped-intro = Intro saltada
player-skipped-outro = Outro saltado
player-undo = Deshacer
player-refusal-hidden = la serie está oculta
player-refusal-no-match = la serie aún no está reconocida; reconócela en Info de la serie
player-refusal-not-newer = tu lista ya va por delante de este episodio
player-refusal-extra = esto es un extra
player-refusal-unmatched = el archivo no corresponde a ningún episodio
player-refusal-on-disk = el archivo solo está en el disco
player-not-tracked-because = Sin registrar: { $reason }.
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = No se pudo actualizar { $tracker }
player-outcome-ok = { $tracker } ok
player-outcome-at = { $tracker } en { $progress }
player-outcome-tracked = Registrado
player-outcome-not-tracked = Sin registrar
player-outcome-could-not-update = No se pudo actualizar tu lista
player-outcome-tracked-list = Registrado: { $list }
player-outcome-not-tracked-list = Sin registrar: { $list }
player-outcome-failed-list = No se pudo actualizar tu lista: { $list }
player-rated = Nota { $score }
player-score-failed = No se pudo guardar la nota
player-frame = Fotograma { $number } en { $time }
player-audio-title = Audio
player-subtitles-title = Subtítulos
player-cast-to = Enviar a
player-cast-looking = Buscando Chromecasts y teles
player-cast-unreachable = No se pudo llegar a la tele
player-cast-tv = TV
player-cast-stop = Dejar de enviar a { $device }
player-cast-connecting = Conectando
player-cast-loading = Cargando
player-cast-playing = Reproduciendo
player-cast-paused = En pausa
player-cast-finished = Terminado
player-cast-stopped = Detenido
player-rate-prompt = Último episodio registrado. ¿Puntúas esta serie?
player-rate = Puntuar
player-not-now = Ahora no

## EndpointReposPanel.qml

endpoint-repos-panel-title = Listas de fuentes
endpoint-repos-panel-helper = Una lista de fuentes es un conjunto firmado de sitios de búsqueda, añadido por su dirección. AniBeam verifica la firma y muestra la clave antes de guardar nada; luego instalas las que quieras, que se suman a Fuentes de búsqueda.
endpoint-repos-panel-could-not-start = AniBeam no pudo iniciar eso. Inténtalo de nuevo en un momento.
endpoint-repos-panel-needs-value = { $name } necesita un valor
endpoint-repos-panel-values-of = Valores de { $name }
endpoint-repos-panel-secret-note = Se guarda en el almacén de secretos de este equipo, nunca en la biblioteca ni en una exportación.
endpoint-repos-panel-secret-note-keep = Se guarda en el almacén de secretos de este equipo, nunca en la biblioteca ni en una exportación. Un campo vacío conserva lo guardado.
endpoint-repos-panel-asks-again = { $name } versión { $version } vuelve a pedirlo
endpoint-repos-panel-update-sends-to = La nueva versión envía esto a { $host }.
endpoint-repos-panel-fields-changed = Sus campos cambiaron.
endpoint-repos-panel-needs-fields = Necesita { $fields } antes de buscar.
endpoint-repos-panel-no-longer-offered = Su lista de fuentes ya no la ofrece.
endpoint-repos-panel-update-requests-to = La actualización envía peticiones a { $host }.
endpoint-repos-panel-unreadable = No se pudo leer esta lista. Actualizar listas lo vuelve a intentar.
endpoint-repos-panel-add-list = Añadir una lista de fuentes
endpoint-repos-panel-add-list-helper = La dirección de su archivo de índice, o la ruta a uno en este equipo.
endpoint-repos-panel-checking = Comprobando
endpoint-repos-panel-add = Añadir
endpoint-repos-panel-trust-question = ¿Confiar en { $name }?
endpoint-repos-panel-fingerprint = Huella de la clave  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    { $count } fuentes, firmadas. Sus fuentes deciden adónde van tus búsquedas; confía en una lista solo si viene de alguien de confianza.
endpoint-repos-panel-preview-trusted =
    { $url }
    { $count } fuentes, firmadas, ya de confianza. Sus fuentes deciden adónde van tus búsquedas; confía en una lista solo si viene de alguien de confianza.
endpoint-repos-panel-trust = Confiar
endpoint-repos-panel-saving = Guardando
endpoint-repos-panel-install = Instalar
endpoint-repos-panel-update = Actualizar
endpoint-repos-panel-added-by-you = añadida por ti
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Actualizar a v{ $version }
endpoint-repos-panel-add-value = Añadir su valor
endpoint-repos-panel-change = Cambiar { $fields }
endpoint-repos-panel-asks-for = Pide { $fields }.
endpoint-repos-panel-by-hand = Añadir una a mano
endpoint-repos-panel-by-hand-helper = Para sitios sin lista: cualquier feed con {"{"}query{"}"} en la búsqueda, también de un proxy. Una clave en la dirección se guarda aparte, en el almacén de contraseñas.
endpoint-repos-panel-name = Nombre
endpoint-repos-panel-refreshing = Actualizando
endpoint-repos-panel-refresh = Actualizar listas

## ScheduleSheet.qml

schedule-episodes-range = Episodios { $first } a { $last }
schedule-day-format = ddd d MMM yyyy
schedule-month-format = MMMM 'de' yyyy
schedule-tile-day-format = d MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = En algún momento de { $year }
schedule-no-date-yet = Aún sin fecha
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = Sin fecha
schedule-tile-last = Último
schedule-tile-out = Salió
schedule-ends-at = Termina en el episodio { $number }
schedule-dated-by-site = { $count } con fecha de { $site }
schedule-dated-by-record = { $count } con fecha de la ficha
schedule-set-by-you = { $count } fijadas por ti
schedule-guessed-weekly = { $count } estimadas semanalmente
schedule-nothing-set = Aún no hay nada fijado
schedule-saved = Guardado
schedule-cleared = Notas de horario borradas
schedule-title = Episodios y fechas
schedule-season = Temporada { $number }
schedule-ask-agent = Preguntar a { $agent }
schedule-done-pasting = Listo
schedule-paste-list = Pegar una lista
schedule-agent-looking = { $agent } está buscando las fechas. No se guarda nada hasta que pulses Guardar.
schedule-agent-stopped = { $agent } se detuvo: { $error }
schedule-stop = Detener
schedule-paste-placeholder = Pega una lista de lanzamientos, una tabla o una publicación
schedule-paste-helper = Lo que AniBeam lee aparece en los episodios de abajo.
schedule-agent-hint = Con Claude Code en este ordenador, AniBeam puede pedirle que busque las fechas por ti.
schedule-show-more = Mostrar más episodios
schedule-pick-hint = Haz clic en un episodio para ponerle fecha. Mayús+clic elige una serie seguida.
schedule-picked-one = Episodio { $number }:
schedule-picked = { $count ->
    [one] { $count } episodio:
   *[other] { $count } episodios:
}
schedule-date-placeholder = 7 enero, marzo 2027, 2027
schedule-set-date = Fijar fecha
schedule-weekly = Semanal
schedule-not-last = No es el último
schedule-last-episode = Último episodio
schedule-clear-date = Borrar fecha

## ShowPage.qml

show-title = Serie
show-runtime-each = { $runtime } cada uno
show-add-to-list = Añadir a lista
show-moving-to = Pasando a { $list }
show-moved-to = Pasada a { $list }
show-relation-related = Relacionada
show-no-poster = Sin cartel
show-reading-anilist = Leyendo AniList
show-not-in-library = No está en tu biblioteca
show-get-episodes = Obtener episodios
show-trailer = Tráiler
show-your-score = Tu nota { $score }
show-rate-it = Puntuar
show-anilist-score = AniList { $score }
show-forecast = { $percent } % para ti
show-because-you-watched = Porque viste
show-hidden-loved = { $count ->
    [one] Detrás hay una etiqueta oculta que te encanta
   *[other] Detrás hay { $count } etiquetas ocultas que te encantan
}
show-you-gave-it = Le diste { $score }
show-story = Historia
show-genres-tags = Géneros y etiquetas
show-studios = Estudios  { $studios }
show-related = Relacionadas
show-recommendations = Recomendaciones
show-from-anilist = De AniList
show-reading-rest = Leyendo el resto de AniList

## ConnectSheet.qml

connect-connected-to = Conectado a { $name }
connect-connect-to = Conectar { $name }
connect-did-not-finish = El inicio de sesión no terminó
connect-finish-in-browser = Termina en tu navegador
connect-signed-in-anilist = Sesión iniciada como { $account }. Tu lista está en la página Viendo.
connect-signed-in-mal = Sesión iniciada como { $account }. Los episodios se marcan allí mientras ves.
connect-setup-lead = { $name } conecta AniBeam mediante una app tuya. Se tarda un minuto, una sola vez.
connect-page-open = La página de inicio de sesión de { $name } está abierta en tu navegador. Aprueba AniBeam allí y esto se cerrará solo.
connect-page-opening = Abriendo la página de inicio de sesión de { $name } en tu navegador.
connect-open-watching = Abrir Viendo
connect-done = Hecho
connect-open-again = Volver a abrir la página de inicio de sesión
connect-secret-where = Es la cadena larga bajo "Client Secret" en la página de la app en myanimelist.net/apiconfig.
connect-client-secret = Secreto de cliente
connect-save-try-again = Guardar y reintentar
connect-change-client-id = Cambiar el Client ID
connect-step-create-anilist = Crea el cliente en AniList
connect-step-create-mal = Crea la app en MyAnimeList
connect-step-create-anilist-body = Abre los ajustes de desarrollador de AniList, crea un cliente nuevo y pega esta dirección en su campo Redirect URL tal cual está aquí.
connect-step-create-mal-body = Abre la página de API de MyAnimeList, crea un ID con App Type "other" y pega esta dirección en su campo App Redirect URL tal cual está aquí.
connect-open-developer-page = Abrir la página de desarrollador
connect-copy-address = Copiar la dirección
connect-step-paste-id = Pega el ID de la app
connect-step-paste-id-anilist = AniList lo llama Client ID: el número de la página de tu nuevo cliente.
connect-step-paste-id-mal = MyAnimeList lo llama Client ID: la cadena larga de la página de tu nueva app.
connect-client-id = Client ID
connect-client-secret-web = Secreto de cliente (solo para apps web)
connect-step-connect-body = Se abre el navegador, { $name } te pide iniciar sesión y aprobar AniBeam, y AniBeam recoge la respuesta.

## SettingsPage

settings-title = Ajustes
settings-search-placeholder = Buscar ajustes
settings-section-library = Biblioteca
settings-section-accounts = Cuentas
settings-section-appearance = Apariencia
settings-section-playback = Reproducción
settings-section-subtitles = Subtítulos
settings-section-keys = Teclado
settings-section-downloads = Descargas
settings-section-phone = App del móvil
settings-section-storage = Almacenamiento y copia
settings-section-help = Ayuda
settings-section-about = Acerca de
settings-section-developer = Desarrollador
settings-status-library-empty = Aún no hay carpetas
settings-status-library =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    } en { $folders ->
        [one] { $folders } carpeta
       *[other] { $folders } carpetas
    }
settings-status-library-missing =
    { $gone ->
        [one] { $gone } carpeta no encontrada
       *[other] { $gone } carpetas no encontradas
    }, { $count ->
        [one] { $count } serie
       *[other] { $count } series
    } en { $folders ->
        [one] { $folders } carpeta
       *[other] { $folders } carpetas
    }
settings-status-accounts-anilist = Sincronizando con AniList
settings-status-accounts-mal = Sincronizando con MyAnimeList
settings-status-accounts-both = Sincronizando con AniList y MyAnimeList
settings-status-accounts-none = Ninguna lista conectada
settings-status-appearance =
    { $mode ->
        [system] Sigue al sistema
        [light] Claro
       *[dark] Oscuro
    }, { $source ->
        [theme] colores del tema
       *[desktop] colores del escritorio
    }
settings-status-playback =
    { $skip ->
        [both] Salta intros y outros
        [intro] Salta intros
        [outro] Salta outros
       *[none] Lo reproduce todo
    }
settings-status-playback-audio =
    { $skip ->
        [both] Salta intros y outros
        [intro] Salta intros
        [outro] Salta outros
       *[none] Lo reproduce todo
    }, audio { $languages }
settings-status-subtitles =
    Tamaño { $percent } por ciento, { $override ->
        [AsScripted] archivos con estilo tal cual
        [Force] tu estilo en todos los archivos
       *[ScaleOnly] los archivos con estilo conservan su aspecto
    }
settings-status-keys = Todas las teclas de AniBeam
settings-status-downloads-unread = Aún sin leer
settings-status-downloads-answered = qBittorrent respondió
settings-status-downloads-missing = Ayudante de búsqueda sin instalar
settings-status-downloads-unconnected = Aún sin conectar
settings-status-phone-shared = Compartido en tu tailnet
settings-status-phone-off = Sin compartir
settings-status-storage = { $size } de imágenes
settings-status-storage-unread = Imágenes, exportar e importar
settings-status-help = Respuestas a lo que más se pregunta
settings-status-about-update = Ya está aquí la versión { $version }
settings-status-developer = Midiendo cada página
settings-status-developer-last = { $page } construida en { $ms } ms
settings-results =
    { $count ->
        [one] { $count } ajuste coincide
       *[other] { $count } ajustes coinciden
    }
settings-results-none = Nada coincide
settings-results-hint = Intro abre el primero.
settings-results-none-hint = Prueba con una palabra más simple: tema, subtítulos, tracker, contraseña, carpeta.

settings-find-folders =
    .words = añadir carpeta revisar releer escanear quitar fuente ruta colección
settings-find-matching =
    .words = metadatos info serie proveedor anilist tvmaze orden reconocer
settings-find-hidden-shows =
    .words = ocultas ocultar mostrar
settings-find-hidden-airing =
    .words = no interesa emisión mostrar otra vez
settings-find-calendar =
    .words = calendario ics ical emisión horario suscribir
settings-find-anime-lists =
    .words = anilist myanimelist mal iniciar sesión conectar cuenta desconectar tracker
settings-find-main-list =
    .words = progreso tarjetas principal tracker
settings-find-sync-now =
    .words = actualizar sincronizar progreso listas tracker
settings-find-on-hold =
    .words = en espera pausa inactivas sin tocar días
settings-find-language =
    .words = idioma traducción configuración regional español inglés
settings-find-mode =
    .words = oscuro claro sistema tema noche
settings-find-colours-from =
    .words = terminal escritorio sistema tema origen kitty
settings-find-dark-theme =
    .words = color esquema base16
settings-find-light-theme =
    .words = color esquema base16
settings-find-accent =
    .words = color acento resaltado
settings-find-text-size =
    .words = fuente letra palabras más grande más pequeño zoom escala leer accesibilidad
settings-find-density =
    .words = compacto cómodo espaciado tamaño densidad
settings-find-poster-size =
    .words = tarjetas carteles pequeño mediano grande
settings-find-out-now =
    .words = emisión disponibles tarjetas lista filas estilo banner carteles
settings-find-out-now-cards =
    .words = emisión disponibles cuadrícula columnas filas diseño
settings-find-reduce-motion =
    .words = animación movimiento accesibilidad
settings-find-app-icon =
    .words = icono logo lanzador dock barra de tareas ventana oscuro
settings-find-skip-intros =
    .words = opening op intro saltar automático aniskip capítulos
settings-find-skip-outros =
    .words = ending ed créditos outro saltar automático aniskip
settings-find-mpv-settings =
    .words = mpv mpv.conf configuración avanzado
settings-find-discord =
    .words = discord rich presence estado perfil
settings-find-next-episode =
    .words = siguiente botón título reproducción automática final
settings-find-subtitle-languages =
    .words = idioma pista subtítulos
settings-find-audio-languages =
    .words = idioma pista doblaje japonés audio
settings-find-start-over =
    .words = restablecer predeterminado estilo subtítulos
settings-find-size =
    .words = escala tamaño subtítulos
settings-find-position =
    .words = altura margen posición subtítulos
settings-find-styled =
    .words = ass sobrescribir forzar estilo
settings-find-font =
    .words = fuente tipografía letra
settings-find-text-colour =
    .words = color texto
settings-find-outline =
    .words = borde contorno trazo
settings-find-background =
    .words = fondo caja opacidad
settings-find-bold =
    .words = negrita grosor
settings-find-keys-player =
    .words = teclado atajos teclas
settings-find-keys-player-around =
    .words = teclado atajos pantalla completa siguiente anterior episodio atrás
settings-find-keys-around =
    .words = teclado atajos atrás adelante clic derecho menú pantalla completa
settings-find-keys-anywhere =
    .words = teclado atajos teclas ajustes salir
settings-find-torrent-client =
    .words = qbittorrent contraseña usuario dirección webui seedbox cliente torrent
settings-find-search-helper =
    .words = anirss rss feeds token comando servidor avanzado ayudante
settings-find-next-few =
    .words = continua episodios por delante eliminar vistos conservar disco espacio
settings-find-unsubscribe =
    .words = cancelar suscripción feeds terminadas limpiar
settings-find-last-search =
    .words = descargas búsqueda recordar
settings-find-search-sources =
    .words = endpoints feeds configuración anirss fuentes
settings-find-source-lists =
    .words = repo repos endpoint firmado endpoints jackett prowlarr listas
settings-find-share-tailscale =
    .words = móvil android tailnet tailscale remoto host compartir
settings-find-stream =
    .words = streaming reproducir enviar episodios móvil archivos
settings-find-devices =
    .words = confianza de confianza desactivar bloquear dispositivo tailscale
settings-find-back-up =
    .words = exportar copia respaldo archivo json
settings-find-restore =
    .words = importar restaurar archivo json
settings-find-image-cache =
    .words = carteles borrar imágenes disco caché
settings-find-files =
    .words = base de datos datos configuración caché carpeta ruta
settings-find-help-folders =
    .words = ayuda carpeta estructura nombres episodios temporadas películas archivos
settings-find-help-recognised =
    .words = ayuda reconocer serie equivocada no encontrada sin reconocer
settings-find-help-anilist =
    .words = ayuda anilist myanimelist mal iniciar sesión conectar tracker
settings-find-help-downloads =
    .words = ayuda descargas vacío fuentes nada torrent qbittorrent
settings-find-help-data =
    .words = ayuda datos archivos copia base de datos privacidad
settings-find-help-else =
    .words = ayuda configuración bienvenida guía notificar fallo problema incidencia
settings-find-version =
    .words = versión compilación novedades cambios
settings-find-setup =
    .words = primer inicio guía bienvenida configurar otra vez
settings-find-updates =
    .words = estable nightly canal comprobar actualizaciones
settings-find-developer-options =
    .words = depuración debug tiempos desarrollador
settings-find-ratings =
    .words = imdb créditos valoraciones

## SettingsDownloadsTab

settings-downloads-title = Descargas
settings-downloads-lead = Adónde envía AniBeam lo que descargas, y cómo busca en los feeds de lanzamientos.
settings-downloads-keep-none = Terminar un episodio elimina el anterior.
settings-downloads-keep-line = Vistos que se guardan tras el que acabas de terminar. Con { $count }, terminar el episodio { $finishing } borra el 1.
settings-downloads-anirss-ready = En marcha, versión { $version } en { $url }.
settings-downloads-anirss-starting = Iniciando el ayudante de búsqueda.
settings-downloads-anirss-missing = El ayudante de búsqueda no está instalado. Reinstala AniBeam, o indica tu propio anirss en Comando.
settings-downloads-anirss-failed = El ayudante de búsqueda se detuvo. Reintentar lo inicia; Detalles muestra lo que dijo.
settings-downloads-anirss-idle = Aún sin iniciar. Se inicia la primera vez que buscas en la página Descargas.
settings-downloads-torrent-trying = Probando { $client }.
settings-downloads-torrent-answered = { $client } { $version } respondió.
settings-downloads-torrent-failed = { $client } no se conectó. Revisa arriba dirección y contraseña y vuelve a probar.
settings-downloads-torrent-untried = Aún sin probar.
settings-downloads-mode-existing = Un qBittorrent que ejecutas tú, con su interfaz web activada.
settings-downloads-mode-managed = AniBeam ejecuta un qBittorrent propio que descarga en ~/Downloads/Anime. Sigue en marcha cuando cierras AniBeam, así que las suscripciones siguen descargando.
settings-downloads-mode-auto = Usa el qBittorrent de la dirección de abajo, o uno propio.
settings-downloads-password-saved = Hay una contraseña guardada aquí para este cliente.
settings-downloads-password-anirss = Se usa la contraseña que guardó el ayudante de búsqueda; guarda una aquí para no depender de ella.
settings-downloads-password-none = No hay contraseña guardada. Un cliente que omite el inicio de sesión para este equipo no la necesita.
settings-downloads-mismatch = El ayudante de búsqueda habla con { $helperUrl } y AniBeam con { $url }. Un ayudante de búsqueda que ejecutas tú lee la dirección de su propio config.toml, así que cambiar de cliente aquí no lo mueve.
settings-downloads-torrent-title = Cliente de torrents
settings-downloads-torrent-helper = El qBittorrent que descarga. Su contraseña va al almacén de contraseñas del sistema.
settings-downloads-which-client = Qué cliente
settings-downloads-pick-for-me = Elige por mí
settings-downloads-my-own = El mío
settings-downloads-built-in = Integrado
settings-downloads-saved-clients = Clientes guardados
settings-downloads-saved-clients-helper = Por ejemplo, un seedbox y el de este equipo. El chip del cliente en la página Descargas también cambia entre ellos.
settings-downloads-add-client = Añadir
settings-downloads-name = Nombre
settings-downloads-name-placeholder = Con el nombre de su dirección
settings-downloads-address = Dirección
settings-downloads-address-managed-helper = Solo este equipo puede acceder a él, y no necesita contraseña.
settings-downloads-address-helper = La dirección de la interfaz web.
settings-downloads-username = Usuario
settings-downloads-password = Contraseña
settings-downloads-secret-saved = Guardada
settings-downloads-secret-unset = Sin definir
settings-downloads-connection = Conexión
settings-downloads-open-config = Abrir su configuración
settings-downloads-details = Detalles
settings-downloads-details-torrent = { $client } dijo esto cuando AniBeam lo probó.
settings-downloads-test-trying = Probando
settings-downloads-test-save = Guardar y probar
settings-downloads-test = Probar
settings-downloads-searching = Búsqueda
settings-downloads-search-helper = Ayudante de búsqueda
settings-downloads-searching-helper = AniBeam busca en tus fuentes por sí mismo y envía lo que eliges al cliente de torrents. No hace falta el ayudante de búsqueda.
settings-downloads-search-helper-helper = La parte de AniBeam que busca y se suscribe, un programa llamado anirss. AniBeam lo ejecuta por ti; cámbialo solo si ejecutas el tuyo.
settings-downloads-runs-in = Las búsquedas van en
settings-downloads-runs-in-native-helper = Las suscripciones siguen en el cliente de torrents: lo nuevo llega con AniBeam cerrado.
settings-downloads-runs-in-helper = El ayudante de búsqueda, un programa aparte que ejecuta AniBeam o tú.
settings-downloads-engine-native = El propio AniBeam
settings-downloads-how-runs = Cómo se ejecuta
settings-downloads-how-runs-sidecar = AniBeam lo inicia al abrirse y lo detiene al cerrarse.
settings-downloads-how-runs-remote = AniBeam usa uno que ya ejecutas, en la dirección y con la clave de abajo.
settings-downloads-runs-sidecar = Lo ejecuta AniBeam
settings-downloads-runs-remote = Lo ejecuto yo
settings-downloads-command = Comando
settings-downloads-command-helper = Un nombre en el PATH o una ruta completa. También se busca en ~/.local/bin.
settings-downloads-server-address-helper = La base del servidor, sin /api/v1.
settings-downloads-key = Clave
settings-downloads-key-saved-helper = Hay una clave guardada. Escribe una nueva para reemplazarla, o vacía el campo para quitarla.
settings-downloads-key-helper = La clave que mostró el ayudante de búsqueda al iniciarse. Deja entrar a AniBeam.
settings-downloads-details-anirss = El ayudante de búsqueda dijo esto al detenerse.
settings-downloads-starting = Iniciando
settings-downloads-reconnect = Reconectar
settings-downloads-save-first = Guarda antes tus cambios
settings-downloads-roll-title = Tener listos los siguientes
settings-downloads-roll-helper = Una serie larga se descarga por tandas, y cada episodio que terminas inicia el siguiente. Nunca tiene que caber entera en disco.
settings-downloads-roll-big = Para series largas
settings-downloads-roll-big-helper = Lo que pase de { $count } episodios se descarga solo unos pocos cada vez. Su fila en Descargas aún puede bajarlo todo de golpe.
settings-downloads-roll-ahead = Episodios por delante
settings-downloads-roll-ahead-helper = Cuántos quedan descargados tras el que ves. 12 es casi una temporada.
settings-downloads-unit-episodes = episodios
settings-downloads-roll-over = Larga es más de
settings-downloads-roll-over-helper = Las descargas con más episodios que esto van de pocos en pocos.
settings-downloads-roll-watched = Episodios vistos
settings-downloads-roll-watched-delete-helper = Se eliminan cuando avanzas, así que el disco solo guarda lo que queda por ver.
settings-downloads-roll-watched-keep-helper = Cada archivo se queda en el disco después de verlo.
settings-downloads-keep-all = Conservar todo
settings-downloads-keep-behind = Conservar detrás
settings-downloads-subscriptions-title = Suscripciones y búsquedas
settings-downloads-last-search = Recordar la última búsqueda
settings-downloads-last-search-helper = La página Descargas abre tu última búsqueda. Desactivado, se abre vacía.
settings-downloads-prune = Cancelar suscripción a series terminadas
settings-downloads-prune-helper = 16 días después de emitirse el último episodio de una serie, se quita su feed. La serie y sus archivos se quedan.
settings-downloads-sources-title = Fuentes de búsqueda
settings-downloads-sources-helper = Adónde va una búsqueda, en el orden de prueba. Si una no encuentra nada, pasa a la siguiente.
settings-downloads-sources-empty = Aún no hay fuentes configuradas
settings-downloads-sources-connect = Conecta el ayudante de búsqueda para verlas
settings-downloads-sources-empty-helper = Añade abajo una lista de fuentes para verlas aquí.
settings-downloads-config-file = Archivo de ajustes del ayudante de búsqueda
settings-downloads-config-native-helper = Guarda las carpetas de destino, las etiquetas y las reglas de mejor coincidencia, que el propio AniBeam sigue cuando ejecuta el ayudante. Sus fuentes son solo del ayudante.
settings-downloads-config-helper = Guarda las fuentes, las carpetas de destino, las etiquetas y las reglas de mejor coincidencia.
settings-downloads-unsaved = Cambios sin guardar
settings-downloads-unsaved-helper = Guardar reinicia el ayudante de búsqueda con los ajustes nuevos.
settings-downloads-discard = Descartar

## SettingsDeveloperTab

settings-developer-title = Desarrollador
settings-developer-lead = Por qué una página va lenta, medido. Cada interruptor se aplica a la próxima página que abras.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds } s
settings-developer-switches = Interruptores
settings-developer-load-images = Cargar imágenes
settings-developer-load-images-helper = Desactivado, no se lee ni decodifica ningún cartel, banner o retrato.
settings-developer-overlay = Capa de carga de página
settings-developer-overlay-helper = La última carga de página y la memoria, en la esquina inferior derecha de la ventana.
settings-developer-read-cache = Guardar lecturas de la biblioteca
settings-developer-read-cache-helper = La respuesta a una lectura de toda la biblioteca se guarda hasta que la biblioteca cambia. Desactivado, cada página vuelve a preguntar al núcleo.
settings-developer-poster-pool = Precargar carteles en segundo plano
settings-developer-poster-pool-helper = Decodifica los 60 carteles que es más probable que veas y los mantiene en memoria.
settings-developer-options = Opciones de desarrollador
settings-developer-options-helper = Desactiva esta pestaña, la capa y las mediciones.
settings-developer-memory = Memoria
settings-developer-memory-helper = Lo que el kernel cuenta para este proceso. Las imágenes guardan píxeles decodificados en memoria y en la tarjeta gráfica.
settings-developer-memory-unknown = Este sistema no indica cuánta memoria usa la app.
settings-developer-in-use = En uso
settings-developer-peak = Pico
settings-developer-heap = Heap
settings-developer-files-mapped = Archivos mapeados
settings-developer-threads = Hilos
settings-developer-give-back = Liberar memoria
settings-developer-give-back-helper = Suelta lo que el grafo de escena puede reconstruir, recoge la basura de JavaScript y devuelve el heap libre al sistema. Lo que libera es lo que la app retenía sin necesitarlo.
settings-developer-released = { $before } a { $after }
settings-developer-free-now = Liberar ahora
settings-developer-images-total = Imágenes desde el inicio
settings-developer-images-total-line = { $decodes } decodificadas, { $pixels } de píxeles a partir de { $disk } en disco, { $ms } ms
settings-developer-loads = Cargas de página
settings-developer-loads-helper = Construida es la espera tras un clic: el hilo de Qt no hace nada más hasta que la página existe. Asentada es cuando llegó su última imagen.
settings-developer-touring = Midiendo, { $at } de { $steps }
settings-developer-tour = Medir cada página
settings-developer-copy-report = Copiar informe
settings-developer-col-page = Página
settings-developer-col-built = Construida
settings-developer-col-painted = Pintada
settings-developer-col-settled = Asentada
settings-developer-col-calls = Llamadas, datos
settings-developer-col-decodes = Decodificaciones, tiempo, píxeles
settings-developer-col-items = Elementos
settings-developer-col-offscreen = Imágenes fuera de vista
settings-developer-col-memory = Memoria
settings-developer-load-no-images = { $page }, sin imágenes
settings-developer-offscreen-of = { $offscreen } de { $total }
settings-developer-loads-empty = Abre cualquier página y aparecerá aquí.
settings-developer-load-images-off = { $page }, imágenes desactivadas
settings-developer-load-frames = { $frames } fotogramas pintados mientras cargaba, el mayor hueco entre dos { $gap }.
settings-developer-load-frames-timed-out = { $frames } fotogramas pintados mientras cargaba, el mayor hueco entre dos { $gap }. Se dejó de esperar a sus imágenes tras 15 s.
settings-developer-built = Construida
settings-developer-settled = Asentada
settings-developer-core-data = Datos del núcleo
settings-developer-pixels = Píxeles decodificados
settings-developer-page-items = Elementos de la página
settings-developer-cached-images = Imágenes ya en memoria
settings-developer-calls = Llamadas al núcleo, las más lentas primero
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  en caché
settings-developer-call = { $name }  { $ms } ms  { $size }  núcleo { $core }, a QML { $qml }
settings-developer-decodes = Decodificaciones más lentas
settings-developer-decodes-line = { $count } decodificaciones seguidas en el hilo de imágenes, { $span } de la primera a la última; { $requests } pedidas, { $cached } ya decodificadas, { $blocked } bloqueadas
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } a { $w }x{ $h }  { $size }
settings-developer-decodes-none = Nada decodificado: todas las imágenes ya estaban en memoria.
settings-developer-images-were-off = Las imágenes estaban desactivadas.
settings-developer-scene = De qué está hecha la página
settings-developer-scene-line = { $items } elementos, { $offscreen } fuera de la ventana; { $texts } textos, { $shapes } formas, { $icons } iconos; { $images } imágenes, { $imagesOffscreen } de ellas fuera de la ventana con { $size }
settings-developer-copy-load = Copiar esta carga
settings-developer-tour-title = Cada página, con y sin imágenes
settings-developer-tour-helper = La mediana de tres visitas a cada página. Donde las dos columnas coinciden, las imágenes no son lo que hace lenta la página.
settings-developer-col-built-on = Construida, con imágenes
settings-developer-col-built-off = Construida, sin imágenes
settings-developer-col-settled-on = Asentada, con
settings-developer-col-settled-off = Asentada, sin
settings-developer-col-decodes-time = Decodificaciones, tiempo
settings-developer-col-data = Datos
settings-developer-tour-peak = Pico de memoria durante la medición { $size }
settings-developer-events = Eventos
settings-developer-events-helper = Todo lo que dice el núcleo, de cualquier nivel, lo más nuevo primero. Cada evento se copia al hilo de Qt como JSON; el tamaño es lo que costó enviarlo.
settings-developer-event-rate = { $kind } { $count }/min, { $size }
settings-developer-events-filter = Filtrar por palabras, tipo o etapa
settings-developer-event-plain = { $message }   (simple: { $plain })
settings-developer-copy-events = Copiar eventos

## SettingsKeysTab

settings-keys-title = Teclado
settings-keys-lead = Todas las teclas de AniBeam. Pulsa ? en cualquier sitio para ver las que sirven ahí.
settings-keys-player = En el reproductor
settings-keys-player-around = Moverse por el reproductor
settings-keys-around = Moverse por la app
settings-keys-anywhere = En cualquier sitio

## KeyHelp

key-help-title = Atajos de teclado
key-help-group-playback = Reproducción
key-help-group-navigation = Navegación
key-help-group-library = Biblioteca
key-help-key-space = Espacio / K
key-help-key-left-right = Izquierda / Derecha
key-help-key-ctrl-right = Ctrl+Derecha
key-help-key-backspace = Retroceso
key-help-key-up-down = Arriba / Abajo
key-help-key-hold = Mantén pulsado el vídeo
key-help-key-page = RePág / AvPág
key-help-key-home-end = Inicio / Fin
key-help-key-escape = Esc
key-help-key-alt-left = Alt+Izquierda
key-help-key-alt-right = Alt+Derecha
key-help-key-shift-f10 = Mayús+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / o Ctrl+F
key-help-key-ctrl-space = Ctrl+Espacio
key-help-key-shift-space = Mayús+Espacio
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = Reproducir o pausar
key-help-seek-5 = Saltar 5 s
key-help-seek-10 = Saltar 10 s
key-help-skip = Saltar la intro o el outro; si no, 90 s
key-help-frame-step = Un fotograma atrás o adelante
key-help-speed = Más lento o más rápido, en pasos de 0,25
key-help-normal-speed = Velocidad normal
key-help-volume = Subir o bajar el volumen 5
key-help-mute = Silenciar
key-help-subtitles-toggle = Quitar y volver a poner subtítulos
key-help-subtitle-delay = Retardo de subtítulos 100 ms antes o después
key-help-audio-delay = Retardo de audio 100 ms antes o después
key-help-hold-speed = Velocidad 2× mientras se mantiene
key-help-screenshot = Guardar una captura en Imágenes
key-help-episode = Episodio siguiente o anterior
key-help-chapter = Capítulo anterior o siguiente
key-help-start-end = El principio, o el outro
key-help-fullscreen = Pantalla completa
key-help-escape-player = Cerrar lo abierto; si no, salir de pantalla completa; si no, salir del reproductor
key-help-toggle-list = Mostrar u ocultar esta lista
key-help-library-search = Biblioteca y búsqueda
key-help-search-page = Buscar en esta página
key-help-activity = Actividad
key-help-settings = Ajustes
key-help-quit = Salir
key-help-forward = Adelante
key-help-escape-app = Cerrar lo abierto; si no, atrás
key-help-context-menu = El menú del clic derecho
key-help-pick-card = Elegir una tarjeta, para actuar sobre varias
key-help-pick-range = Elegir todas las tarjetas hasta esta
key-help-pick-all = Elegir todas las tarjetas visibles

## SettingsAboutTab

settings-about-title = Acerca de
settings-about-lead = Qué AniBeam es este, y cómo se entera del siguiente.
settings-about-update-failed = La actualización no terminó. Inténtalo de nuevo, o descárgala de la página de la versión.
settings-about-update-ready = { $version } está lista para instalar.
settings-about-update-command = Ya está aquí la { $version }. Actualiza con { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = Ya está aquí la { $version }. Esta copia está compilada desde el código: haz pull y recompila.
settings-about-update-elsewhere = Ya está aquí la { $version }, sin descarga para este ordenador.
settings-about-version = Versión
settings-about-build = AniBeam { $build }
settings-about-whats-new = Novedades
settings-about-install = Instalar { $version }
settings-about-setup = Configuración
settings-about-setup-helper = La guía con la que empieza una instalación nueva: tu carpeta, tu lista y las descargas.
settings-about-run-setup = Repetir configuración
settings-about-updates = Actualizaciones
settings-about-nightly-helper = Una versión nueva casi cada día. Más nueva, y menos probada.
settings-about-stable-helper = Una versión nueva cuando se ha probado y publicado.
settings-about-stable = Estable
settings-about-nightly = Nightly
settings-about-check-now = Comprobar ahora
settings-about-developer = Opciones de desarrollador
settings-about-developer-helper = Añade Desarrollador: mide cada página, muestra memoria y todo el registro técnico, y puede quitar imágenes.
settings-about-credits = Créditos
settings-about-ratings = Valoraciones
settings-about-ratings-helper = Información por cortesía de IMDb (https://www.imdb.com). Usada con permiso.
settings-about-series = Series y episodios
settings-about-series-helper = AniList y TVmaze. Tiempos de salto de AniSkip.

## SettingsPlaybackTab

settings-playback-title = Reproducción
settings-playback-lead = Lo que el reproductor hace solo, para que lo toques solo cuando quieras.
settings-playback-skipping = Saltos
settings-playback-skipping-helper = Usa los capítulos del archivo o, si no, saltos que otros subieron. Deshacer en el reproductor los quita hasta cerrar AniBeam.
settings-playback-skip-intros = Saltar intros
settings-playback-skip-outros = Saltar outros
settings-playback-skip-outros-helper = Llegar al outro también marca el episodio como visto.
settings-playback-next-episode = Siguiente episodio
settings-playback-name-next = Mostrar el nombre del siguiente
settings-playback-name-next-helper = El botón Siguiente al final de un episodio muestra el título del siguiente. Desactivado, solo muestra su número.
settings-playback-languages = Idiomas
settings-playback-languages-helper = Códigos de idioma, por preferencia. Gana el primero que tenga el archivo. La pista que elijas al ver se recuerda por serie.
settings-playback-subtitle-languages = Idiomas de subtítulos
settings-playback-audio-languages = Idiomas de audio
settings-playback-discord = Mostrar lo que ves en Discord
settings-playback-discord-check-failed = AniBeam no pudo comprobar Discord ahora mismo. Inténtalo de nuevo en un momento.
settings-playback-discord-helper = La serie, el episodio y el cartel aparecen en tu perfil de Discord mientras reproduces. No se envía nada con esto desactivado o con Discord cerrado.
settings-playback-discord-unavailable = Esta versión no tiene aplicación de Discord.
settings-playback-discord-connected = Conectado a Discord
settings-playback-discord-not-connected = Sin conectar. AniBeam se conecta mientras se reproduce un episodio y Discord está abierto
settings-playback-advanced = Avanzado
settings-playback-my-mpv = Usar mis ajustes de mpv
settings-playback-my-mpv-helper = Lee tu mpv.conf. Las opciones que solo funcionan al arrancar se omiten, y los scripts nunca se cargan.
settings-playback-no-mpv-conf = No se encontró ningún archivo de ajustes de mpv. Pon tus opciones en mpv.conf y activa esto.

## SliderRow

slider-row-spoken-unit = { $name }, en { $unit }
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Acento rojo
swatches-accent-orange = Acento naranja
swatches-accent-yellow = Acento amarillo
swatches-accent-green = Acento verde
swatches-accent-cyan = Acento cian
swatches-accent-blue = Acento azul

## LookPreview

look-preview-dark = Oscuro
look-preview-light = Claro

## SettingsAccountsTab

settings-accounts-title = Cuentas
settings-accounts-lead = Conecta AniList o MyAnimeList y cada episodio que termines se marcará allí por ti. Los recuentos solo suben.
settings-accounts-syncing = Sincronizando tus listas
settings-accounts-synced = Listas sincronizadas
settings-accounts-lists = Tus listas de anime
settings-accounts-lists-helper = Sin ninguna, AniBeam sigue recordando lo que viste en este ordenador. Una lista en AniList o MyAnimeList lo lleva a tu móvil y a tu perfil.
settings-accounts-progress = Progreso
settings-accounts-main-list = Lista principal
settings-accounts-main-list-helper = Las tarjetas muestran el progreso de esta. La otra recibe igualmente cada actualización.
settings-accounts-local-lists = Listas en este ordenador
settings-accounts-local-lists-helper =
    { $count ->
        [one] Una serie tiene
       *[other] { $count } series tienen
    } un estado o una nota solo aquí. Pásalas a { $tracker } para verlas en tu móvil y en tu perfil.
settings-accounts-review-move = Revisar y pasar
settings-accounts-nothing-to-move = No queda nada que pasar
settings-accounts-sync-now = Sincronizar ahora
settings-accounts-sync-now-helper = Vuelve a leer todas las listas conectadas. AniBeam también lo hace al arrancar.
settings-accounts-sync = Sincronizar
settings-accounts-move-on-hold = Pasar a En espera
settings-accounts-move-on-hold-helper = Una serie en Viendo que no tocas desde hace tiempo pasa a En espera en tus listas. Reproducirla la devuelve. Una serie que espera su próximo episodio se queda donde está.
settings-accounts-after = Tras
settings-accounts-after-line = Días sin reproducir un episodio
settings-accounts-days-unit = días

## SettingsPhoneTab

settings-phone-title = App del móvil
settings-phone-lead = AniBeam en tu móvil puede reproducir esta biblioteca, ver lo que se emite y seguir tus descargas, estés donde estés.
settings-phone-online = En línea
settings-phone-offline = Sin conexión
settings-phone-untrusted = No recibe nada de este ordenador
settings-phone-sharing = Compartir
settings-phone-sharing-helper = Solo tus dispositivos en Tailscale pueden acceder. Lo visto en el móvil guarda aquí su posición y progreso.
settings-phone-share = Compartir por Tailscale
settings-phone-stream = Emitir episodios
settings-phone-stream-on = Tus dispositivos pueden reproducir y enviar los episodios de este ordenador.
settings-phone-stream-off = Tus dispositivos ven la biblioteca y lo que se emite, pero no pueden reproducir desde aquí.
settings-phone-host-service = Servicio host
settings-phone-not-running = Detenido
settings-phone-running = En marcha
settings-phone-service-off-helper = Ejecuta el comando una vez: tus dispositivos verán AniBeam al iniciar sesión, sin ventana.
settings-phone-service-on-helper = Tus dispositivos llegan a AniBeam sin ninguna ventana abierta. Mientras esta ventana está abierta, comparte en lugar del servicio.
settings-phone-copy-command = Copiar comando
settings-phone-command-copied = Comando copiado
settings-phone-nobody = No recibe nada: Tailscale no le asigna ninguna persona
settings-phone-people = Personas
settings-phone-own-profile = Perfil de este ordenador
settings-phone-person-own-devices =
    { $count ->
        [0] Este ordenador
        [one] Este ordenador y { $count } dispositivo
       *[other] Este ordenador y { $count } dispositivos
    }
settings-phone-person-devices =
    { $count ->
        [one] { $count } dispositivo
       *[other] { $count } dispositivos
    }
settings-phone-devices = Dispositivos en tu tailnet
settings-phone-devices-helper = Cada dispositivo de tu tailnet puede acceder a este ordenador mientras se comparte. Desactiva uno y no recibirá nada de aquí.
settings-phone-addresses = Direcciones
settings-phone-addresses-helper = Escribe una de estas en la app del móvil la primera vez que la pida.
settings-phone-copy = Copiar
settings-phone-address-copied = Dirección copiada

## SettingsHelpTab

settings-help-title = Ayuda
settings-help-lead = Respuestas cortas a lo más preguntado, cada una con un atajo a la solución.
settings-help-folders = ¿Cómo deben ser mis carpetas?
settings-help-folders-helper = Una carpeta por serie, episodios dentro. Cada archivo lleva su número, como los lanzamientos: "Frieren - 01.mkv" o "Frieren S01E01.mkv". La temporada 2, en su carpeta o en "Season 2" dentro. Películas: carpeta Movies, una por archivo.
settings-help-for-example = Por ejemplo
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = Aún no hay ninguna carpeta
settings-help-reads-folders =
    { $count ->
        [one] AniBeam lee una carpeta
       *[other] AniBeam lee { $count } carpetas
    }
settings-help-add-folder-helper = Añade la carpeta que contiene tus series, no cada serie por separado.
settings-help-add-folder = Añadir una carpeta
settings-help-open-folders = Abrir tus carpetas
settings-help-match = ¿Por qué no se reconoció mi serie?
settings-help-match-helper = AniBeam busca cada carpeta por su nombre, primero en AniList y luego en TVmaze. Palabras de más, una errata o un nombre en otro idioma pueden fallar. Elige la serie correcta a mano en Info de la serie; lo que elijas nunca se reemplaza.
settings-help-all-recognised = Todas las series están reconocidas
settings-help-unrecognised =
    { $count ->
        [one] Una serie no está reconocida
       *[other] { $count } series no están reconocidas
    }
settings-help-open-show-info = Abrir Info de la serie
settings-help-connect = ¿Cómo conecto AniList?
settings-help-connect-helper = Conéctalo y cada episodio que termines se marcará en tu lista, para que tu móvil y tu perfil estén al día. AniList te pide crear una pequeña app propia una vez; AniBeam te guía y se tarda un minuto. MyAnimeList funciona igual.
settings-help-both-connected = Los dos están conectados
settings-help-anilist-connected = AniList está conectado
settings-help-mal-connected = MyAnimeList está conectado
settings-help-none-connected = Ninguno conectado aún
settings-help-connect-anilist = Conectar AniList
settings-help-connect-mal = Conectar MyAnimeList
settings-help-downloads = ¿Por qué está vacío Descargas?
settings-help-downloads-helper = AniBeam no trae sitios donde buscar. Añade una lista de fuentes en Ajustes, en Descargas, e instala de ella las fuentes que quieras. Para descargar también hace falta un cliente de torrents: AniBeam puede ejecutar uno propio, o usar un qBittorrent que ya tengas.
settings-help-sources = Fuentes y cliente de torrents
settings-help-open-downloads = Abrir ajustes de Descargas
settings-help-data = ¿Dónde están mis datos?
settings-help-data-helper = Tu biblioteca, historial y ajustes se quedan en este ordenador. Los inicios de sesión de AniList y MyAnimeList van al almacén de contraseñas del sistema o, si no, a un archivo que solo tú lees. Copia todo a un archivo en Almacenamiento y copia.
settings-help-library-history = Biblioteca e historial
settings-help-show-folder = Ver la carpeta
settings-help-backup = Copia o restauración
settings-help-open-storage = Abrir Almacenamiento y copia
settings-help-more = ¿Otra cosa?
settings-help-more-helper = Repite la primera configuración, o cuéntanos qué falló. Una captura ayuda mucho.
settings-help-first-setup = La primera configuración
settings-help-first-setup-helper = Tu carpeta, tu lista y las descargas, paso a paso.
settings-help-run-setup = Repetir configuración
settings-help-report = Notificar un fallo
settings-help-report-helper = Abre un informe de fallo en GitHub en tu navegador, con esta versión ya rellenada.

## SettingsSubtitlesTab

settings-subtitles-title = Subtítulos
settings-subtitles-lead = Cómo empieza cada episodio. Ves los cambios en el fotograma de abajo al hacerlos.
settings-subtitles-reset-done = Los subtítulos se ven como al principio
settings-subtitles-size-place = Tamaño y posición
settings-subtitles-size = Tamaño
settings-subtitles-position = Posición
settings-subtitles-position-helper = 100 queda en el borde inferior. Los números menores suben las líneas en el fotograma.
settings-subtitles-style = Estilo
settings-subtitles-styled = Subtítulos con estilo
settings-subtitles-as-made-helper = Los archivos con estilo (ASS) se ven exactamente como los dejaron sus autores.
settings-subtitles-mine-helper = Tu estilo de abajo se aplica a todos los archivos. Los carteles y el karaoke pueden romperse.
settings-subtitles-size-only-helper = Los subtítulos con estilo guardan su aspecto y usan tu tamaño. Los simples, tu estilo.
settings-subtitles-as-made = Original
settings-subtitles-size-only = Solo tamaño
settings-subtitles-mine = El mío
settings-subtitles-font = Fuente
settings-subtitles-font-invalid = Escribe el nombre de una fuente, como Noto Sans.
settings-subtitles-bold = Negrita
settings-subtitles-text-colour = Color del texto
settings-subtitles-colour-invalid = Escribe un color como #ffcc00.
settings-subtitles-outline = Contorno
settings-subtitles-outline-invalid = El grosor es un número, 0 o más.
settings-subtitles-outline-colour-invalid = Escribe el color como #000000.
settings-subtitles-outline-helper = Su grosor y luego su color.
settings-subtitles-shadow = Sombra
settings-subtitles-shadow-invalid = Un número, 0 o más. 0 significa sin sombra.
settings-subtitles-shadow-helper = Hasta dónde cae la sombra, 0 para ninguna.
settings-subtitles-box = Caja de fondo
settings-subtitles-box-helper = Una caja oscura detrás de las líneas. 0 la quita.
settings-subtitles-start-over = Empezar de nuevo
settings-subtitles-start-over-helper = Devuelve el tamaño, la fuente, los colores, el contorno, la caja y la posición a como venían.
settings-subtitles-reset = Restablecer

## SettingsDataTab

settings-data-title = Almacenamiento y copia
settings-data-lead = Lleva tu biblioteca a otro ordenador, guarda una copia a salvo o libera el espacio de las imágenes.
settings-data-backed-up = Copia guardada en { $path }
settings-data-restore-finished = Restauración terminada
settings-data-images-removed =
    { $count ->
        [one] { $count } imagen eliminada
       *[other] { $count } imágenes eliminadas
    }
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] { $count } carpeta añadida
       *[other] { $count } carpetas añadidas
    }
settings-data-summary-series = { $count } series creadas
settings-data-summary-matches =
    { $count ->
        [one] { $count } reconocimiento aplicado
       *[other] { $count } reconocimientos aplicados
    }
settings-data-summary-views =
    { $views ->
        [one] { $views } visionado
       *[other] { $views } visionados
    } y { $episodes ->
        [one] { $episodes } episodio terminado
       *[other] { $episodes } episodios terminados
    } combinados
settings-data-summary-resume-points =
    { $count ->
        [one] { $count } punto de reanudación combinado
       *[other] { $count } puntos de reanudación combinados
    }
settings-data-summary-accounts =
    { $count ->
        [one] { $count } cuenta de lista incorporada
       *[other] { $count } cuentas de lista incorporadas
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } archivo usa una corrección que hiciste.
       *[other] { $count } archivos usan una corrección que hiciste.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } carpeta no se encontró en este ordenador.
       *[other] { $count } carpetas no se encontraron en este ordenador.
    }
settings-data-summary-left-out = Omitido: { $fields }.
settings-data-backup = Copia de seguridad
settings-data-back-up-library = Hacer copia de la biblioteca
settings-data-back-up-helper = Un archivo: carpetas, reconocimientos e historial.
settings-data-last-written = Última copia en { $path }
settings-data-open-folder = Abrir carpeta
settings-data-back-up = Hacer copia
settings-data-private = Incluir datos privados
settings-data-private-helper = Añade los inicios de sesión de AniList y MyAnimeList, el historial y las preferencias. Van en texto plano: no compartas el archivo.
settings-data-restore-from = Restaurar una copia
settings-data-restore-helper = Lee una copia en esta biblioteca. Lo que hay en el disco manda; el archivo completa el resto.
settings-data-restore = Restaurar
settings-data-image-cache = Caché de imágenes
settings-data-image-cache-helper = Carteles, banners y retratos se guardan en disco para ver la biblioteca sin conexión. Lo borrado se baja otra vez si hace falta.
settings-data-cache-unmeasured = No se pudo medir la caché de imágenes
settings-data-cache-size =
    { $count ->
        [one] { $size } en { $count } imagen
       *[other] { $size } en { $count } imágenes
    }
settings-data-cache-empty = No hay imágenes en caché
settings-data-cache-unmeasured-helper = Inténtalo de nuevo en un momento. Las imágenes en sí están bien.
settings-data-remove-images =
    { $count ->
        [one] ¿Eliminar { $count } imagen?
       *[other] ¿Eliminar { $count } imágenes?
    }
settings-data-files = Dónde guarda AniBeam sus archivos
settings-data-files-database = Base de datos de la biblioteca
settings-data-files-data = Datos
settings-data-files-settings = Ajustes
settings-data-files-cache = Caché
settings-data-backup-filter = Copia de AniBeam (*.json)
settings-data-all-files-filter = Todos los archivos (*)
settings-data-backing-up = Haciendo copia
settings-data-restoring = Restaurando

## SettingsLibraryTab

settings-library-title = Biblioteca
settings-library-lead = Las carpetas que vigila AniBeam, y cómo averigua qué contienen.
settings-library-not-scanned = Aún sin revisar
settings-library-scan-nothing-changed = No cambió nada
settings-library-scan-added = { $count } añadidas
settings-library-scan-changed = { $count } cambiadas
settings-library-scan-removed = { $count } quitadas
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = Anime: series, películas y cómo se relacionan. Lo que ves se marca en tus trackers.
settings-library-tvmaze-about = Otra televisión: imagen real, dibujos, documentales. No se marca nada en ningún tracker.
settings-library-tried-first = Primero
settings-library-tried-second = Segundo
settings-library-tried-third = Tercero
settings-library-tried-fourth = Cuarto
settings-library-tried-nth = { $number }.º
settings-library-folders = Carpetas
settings-library-folders-helper = Cada carpeta dentro de estas es una serie. Un archivo suelto en Movies es una película.
settings-library-rescan-started = Relectura iniciada
settings-library-load-failed = No se pudieron leer tus carpetas
settings-library-load-failed-helper = AniBeam no pudo leer su lista de carpetas. Inténtalo de nuevo en un momento.
settings-library-no-folders = Aún no hay carpetas
settings-library-no-folders-helper = Indica a AniBeam la carpeta donde guardas tu anime. Encuentra las series y películas por sí mismo.
settings-library-scanned = Revisada { $when }
settings-library-add-a-folder = Añadir una carpeta
settings-library-scan-all = Revisar todo
settings-library-scan-started = Revisión iniciada
settings-library-add-folder = Añadir carpeta
settings-library-matching = Reconocimiento
settings-library-matching-helper = Una carpeta nueva se busca en este orden y se queda con la primera buena coincidencia. Lo que fijes a mano nunca se reemplaza.
settings-library-try-later = Probar después
settings-library-try-first = Probar primero
settings-library-check-matches = Revisar el reconocimiento
settings-library-check-matches-helper = La página Info de la serie lista cada serie con lo que AniBeam halló, y corrige lo erróneo.
settings-library-open-show-info = Abrir Info de la serie
settings-library-hidden-shows = Series ocultas
settings-library-show-hidden = Mostrar series ocultas
settings-library-show-hidden-helper = Recupera las series ocultas en cada página hasta cerrar AniBeam.
settings-library-hidden-from-airing = Ocultas en En emisión
settings-library-hidden-from-airing-helper = Series que marcaste como no interesa. Cada una queda fuera de En emisión hasta que la recuperes.
settings-library-hidden-from-airing-empty = Aún nada. Las series que marques como no interesa en En emisión se reúnen aquí, para que cualquiera pueda volver.
settings-library-show-again = Volver a mostrar
settings-library-calendar = Calendario
settings-library-calendar-feed = Horarios de emisión en un calendario
settings-library-calendar-feed-helper = Mantiene un archivo de calendario con los próximos episodios de las series de tus listas, actualizado cuando cambia el horario.
settings-library-calendar-file = Archivo de calendario
settings-library-calendar-file-helper = Suscríbete a este archivo en tu app de calendario.
settings-library-copy = Copiar
settings-library-path-copied = Ruta copiada
settings-library-open-folder = Abrir carpeta
settings-library-notifications = Notificaciones
settings-library-new-episodes = Episodios nuevos
settings-library-new-episodes-helper = Una notificación de escritorio cuando se emite un episodio de una serie de tu lista Viendo. Haz clic para abrir la serie.
settings-library-folder-added = Carpeta añadida, revisando

## SettingsAppearanceTab

settings-appearance-language = Idioma
settings-appearance-language-system = Igual que el sistema ({ $language })
settings-appearance-text-smaller = Menor
settings-appearance-text-normal = Normal
settings-appearance-text-large = Grande
settings-appearance-text-larger = Mayor
settings-appearance-text-largest = Máximo
settings-appearance-title = Apariencia
settings-appearance-lead = Los cambios se ven al instante, aquí y en todas las páginas. Se guardan en ~/.config/anibeam/theme.toml, que también puedes editar a mano.
settings-appearance-colours = Colores
settings-appearance-mode = Modo
settings-appearance-mode-dark = Oscuro
settings-appearance-mode-light = Claro
settings-appearance-mode-system = Sistema
settings-appearance-colours-from = Colores de
settings-appearance-from-theme-helper = El par de temas de abajo, uno oscuro y otro claro.
settings-appearance-from-desktop-helper = Los colores de tu terminal o, si no, el esquema de color y el acento del escritorio.
settings-appearance-from-desktop = Escritorio
settings-appearance-from-theme = Tema
settings-appearance-dark-theme = Tema oscuro
settings-appearance-light-theme = Tema claro
settings-appearance-themes-helper = Los archivos de tema que pongas en ~/.config/anibeam/themes aparecen en las dos listas.
settings-appearance-accent = Acento
settings-appearance-accent-helper = Marca lo seleccionado, lo que suena y lo nuevo.
settings-appearance-icon = Icono
settings-appearance-app-icon = Icono de la app
settings-appearance-app-icon-helper = El de la ventana, la barra de tareas y el lanzador. Oscuro cambia el amarillo por el negro.
settings-appearance-icon-default = Estándar
settings-appearance-icon-default-name = Icono estándar de la app
settings-appearance-icon-dark = Oscuro
settings-appearance-icon-dark-name = Icono oscuro de la app
settings-appearance-size-motion = Tamaño y movimiento
settings-appearance-text-size = Tamaño del texto
settings-appearance-text-size-helper = El tamaño de las letras en todo AniBeam, además del tamaño de tu sistema.
settings-appearance-density = Densidad
settings-appearance-density-helper = El espacio alrededor de todo, de apretado a holgado.
settings-appearance-density-compact = Compacta
settings-appearance-density-normal = Normal
settings-appearance-density-comfortable = Cómoda
settings-appearance-poster-size = Tamaño de los carteles
settings-appearance-poster-size-helper = El tamaño de una serie en la biblioteca y en Inicio.
settings-appearance-poster-small = Pequeño
settings-appearance-poster-medium = Mediano
settings-appearance-poster-large = Grande
settings-appearance-out-now = Disponibles en En emisión
settings-appearance-out-now-helper = Tarjetas con el banner de cada serie, o una lista de carteles pequeños y títulos, de tres filas.
settings-appearance-out-now-cards = Tarjetas
settings-appearance-out-now-list = Lista
settings-appearance-out-now-grid = Tarjetas de Disponibles
settings-appearance-out-now-grid-helper = Cuántas tarjetas se ven, columnas por filas, antes de que Todos abra el resto.
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Reducir movimiento
settings-appearance-reduce-motion-helper = Omite el barrido inicial de la biblioteca y otras animaciones decorativas.

## LookPane

look-pane-sample-series = Serie de ejemplo
look-pane-episode-code = Ep { $number }
look-pane-library = Biblioteca
look-pane-series-count =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    }
look-pane-search = Buscar títulos o carpetas
look-pane-search-hint = / o Ctrl+K
look-pane-all = Todo
look-pane-series = Series
look-pane-movies = Películas
look-pane-plain = Normal
look-pane-selected = Seleccionado
look-pane-errors =
    { $count ->
        [one] { $count } error
       *[other] { $count } errores
    }
look-pane-type-large = Grande
look-pane-type-normal = Normal
look-pane-type-small = Pequeño, tenue
look-pane-sample-switch = Interruptor de ejemplo
look-pane-button = Botón
look-pane-scan = Revisar
look-pane-sample-status =
    { $count ->
        [one] { $count } nuevo en { $title }
       *[other] { $count } nuevos en { $title }
    }

## FeedPage

feed-title = Feed
feed-aired = { $episode } se emitió { $when }
feed-downloaded = Archivo descargado { $when }
feed-countdown-days = { $days } d { $hours } h
feed-countdown-hours = { $hours } h { $minutes } min
feed-countdown-minutes = { $minutes } min
feed-section-this-week = Esta semana
feed-new-episodes =
    { $count ->
        [one] { $count } episodio nuevo
       *[other] { $count } episodios nuevos
    }
feed-section-earlier = Antes
feed-latest-out = El último salió { $when }
feed-section-coming-soon = Próximamente
feed-this-week-count = { $count } esta semana
feed-section-everything-else = Todo lo demás
feed-out-this-week = { $count } salen esta semana
feed-nothing-scheduled = Aún no hay nada programado.
feed-next = Siguiente: { $title } en { $countdown }
feed-nothing-new = Nada nuevo esta semana.
feed-new-this-week =
    { $count ->
        [one] { $count } episodio nuevo esta semana.
       *[other] { $count } episodios nuevos esta semana.
    }
feed-sort-recent = Recién salidos
feed-sort-upcoming = Próximamente
feed-failed-title = El feed no se cargó
feed-empty-title = Aún no hay nada aquí
feed-failed-body = AniBeam no pudo leer el feed ahora mismo. Inténtalo de nuevo en un momento.
feed-empty-body-folders = El feed se llena según se emiten y llegan los episodios. En emisión muestra lo que hay esta temporada.
feed-empty-body = Añade la carpeta donde guardas tu anime y el feed se llenará con sus episodios.
feed-add-folder = Añadir una carpeta
feed-airing-now = En emisión

## RecommendedPage

recommended-title = Recomendadas para ti
recommended-ranked-minutes = Ordenadas hace { $minutes } min
recommended-ranked-hours = Ordenadas hace { $hours } h
recommended-ranked-days = Ordenadas hace { $days } d
recommended-scope-name = Recomendadas de
recommended-scope-all = Todo
recommended-scope-library = En la biblioteca
recommended-scope-list = En la lista
recommended-ranking = Ordenando
recommended-rank-again = Volver a ordenar
recommended-no-scores-title = Puntúa antes algunas series
recommended-no-list-title = Aún no hay nada de lo que aprender
recommended-never-fitted-title = Ordenando tus series
recommended-nothing-left-title = No queda nada aquí
recommended-no-scores-body = Las recomendaciones aprenden de las notas de tu lista. Puntúa algunas series en AniList y aparecerán aquí.
recommended-no-list-body = Conecta AniList en Ajustes o usa una lista local, y las series que puntúes guiarán esta página.
recommended-never-fitted-body = La primera ordenación lee AniList durante un minuto o dos. Se hace en segundo plano.
recommended-nothing-left-library-body = Todo lo de tu biblioteca que encaja ya está visto. Todo busca series que aún no tienes.
recommended-nothing-left-body = Todo lo de tus listas que encaja ya está visto. Todo busca series que no tienes en ninguna lista.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] Pasando 1 serie a { $site }
       *[other] Pasando { $count } series a { $site }
    }
move-lists-score = nota { $score }
move-lists-title = Pasa tu lista a { $site }
move-lists-body =
    { $count ->
        [one] Una serie tiene
       *[other] { $count } series tienen
    } un estado o una nota solo en este ordenador. Pasarlas las pone en tu lista de { $site }, donde las ven tu móvil y tu perfil. Para estas series, se reemplaza lo que tiene ahora { $site }. No se envía nada hasta que pulses Pasar.
move-lists-head-show = Serie
move-lists-head-here = En este ordenador
move-lists-head-tracker = En { $site } ahora
move-lists-nothing = Nada
move-lists-not-on-list = No está en tu lista
move-lists-later-hint = Después: Ajustes, Cuentas
move-lists-not-now = Ahora no
move-lists-move =
    { $count ->
        [one] Pasar 1 serie
       *[other] Pasar { $count } series
    }

## StatsPage

stats-title = Estadísticas
stats-month-jan = Ene
stats-month-feb = Feb
stats-month-mar = Mar
stats-month-apr = Abr
stats-month-may = May
stats-month-jun = Jun
stats-month-jul = Jul
stats-month-aug = Ago
stats-month-sep = Sept
stats-month-oct = Oct
stats-month-nov = Nov
stats-month-dec = Dic
stats-nothing-yet = Aún nada
stats-ranked-episodes =
    { $count ->
        [one] { $count } ep
       *[other] { $count } eps
    }
stats-year-summary = Lo que viste en { $year }
stats-empty-title = Aún no has visto nada
stats-empty-body = Termina un episodio y contará aquí: horas, géneros y las temporadas que viste.
stats-open-library = Biblioteca
stats-hours-watched = horas vistas
stats-episodes-label =
    { $count ->
        [one] episodio
       *[other] episodios
    }
stats-series-finished = series terminadas
stats-none = Ninguno
stats-busiest-month = mes con más actividad
stats-by-month = Por mes
stats-month-hours = { $hours } h
stats-top-genres = Géneros principales
stats-top-studios = Estudios principales
stats-seasons-watched = Temporadas vistas
stats-recap-title = Mi año en anime
stats-recap-hours = horas
stats-recap-episodes = episodios
stats-recap-top-genre = género principal
stats-save-image = Guardar como imagen
stats-save-dialog-title = Guardar el resumen
stats-save-filter-png = Imagen PNG (*.png)
stats-recap-saved = Resumen guardado
stats-recap-save-failed = No se pudo guardar el resumen

## TrackerRow

tracker-row-connected = Conectado como { $user }, sincronizado { $when }
tracker-row-unknown-user = ?
tracker-row-never = nunca
tracker-row-waiting = Esperando a que apruebes en el navegador
tracker-row-not-connected = Sin conectar
tracker-row-open-again = Volver a abrir
tracker-row-open-again-tip = Volver a abrir la página de inicio de sesión
tracker-row-disconnect = Desconectar
tracker-row-disconnect-question = ¿Desconectar { $name }? Se eliminará tu inicio de sesión.
tracker-row-secret-saved = Hay un secreto de cliente guardado para esta app.
tracker-row-remove-secret = Quitarlo
tracker-row-secret-removed = Secreto de cliente quitado
tracker-row-add-secret = Añadir secreto de cliente
tracker-row-secret-where = Es la cadena larga bajo "Client Secret" en la página de la app en myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Secreto de cliente
tracker-row-save-connect = Guardar y conectar

## RemoveSheet

remove-unsubscribe-delete = Cancelar suscripción y eliminar
remove-unsubscribe-torrents = Cancelar suscripción y quitar torrents
remove-unsubscribe = Cancelar suscripción
remove-delete-files = Quitar y eliminar archivos
remove-torrents =
    { $count ->
        [one] Quitar el torrent
       *[other] Quitar torrents
    }
remove-also-torrents =
    { $count ->
        [one] Quitar también su torrent
       *[other] Quitar también sus { $count } torrents
    }
remove-torrents-go-with-files = Eliminar los archivos se lleva los torrents con ellos.
remove-torrents-files-stay = Salen del cliente; los archivos se quedan.
remove-also-files = Eliminar también los archivos
remove-files-default-where = lo que descargaron los torrents
remove-files-detail-rule = Mueve { $where } a la papelera y quita los torrents.
remove-files-detail = Mueve { $where } a la papelera.

## NotesSheet

notes-your-tracker = tu tracker
notes-title = Notas
notes-placeholder = Por dónde ibas, quién la recomendó, qué lanzamiento obtener
notes-over-limit = { $count } caracteres por encima del límite
notes-saved-to = Guardado en { $sites }

## RecommendationCard

recommendation-card-available = Disponible
recommendation-card-open-library = Abrir { $title } en tu biblioteca
recommendation-card-open-anilist = Abrir { $title } en AniList

## SearchField

search-field-placeholder = Buscar títulos o carpetas
search-field-hint = / o Ctrl+K
search-field-clear = Borrar la búsqueda

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent } %
recommended-hover-hidden-tags =
    { $count ->
        [one] Una etiqueta oculta que te encanta
       *[other] { $count } etiquetas ocultas que te encantan
    }
recommended-hover-because = Porque viste

## PersonCard

person-card-unknown = Desconocido
person-card-role-main = Principal
person-card-role-supporting = Secundario
person-card-role-background = Fondo
person-card-the-character = el personaje
person-card-open = Abrir { $name } en AniList

## StatusDot

status-dot-on-list = En tu lista: { $status }

## TrackerNudge

tracker-nudge-title = Conecta tu lista de anime
tracker-nudge-body = AniBeam marca los episodios en AniList o MyAnimeList mientras ves.
tracker-nudge-connect-anilist = Conectar AniList
tracker-nudge-connect-mal = Conectar MyAnimeList
tracker-nudge-not-now = Ahora no

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } por ciento para ti
recommended-card-forecast = { $percent } %
recommended-card-in-library = En la biblioteca

## InlineConfirm

inline-confirm-keep = Conservar

## Tokens

tokens-cube-watched-tracked = Visto, registrado en línea
    .inline = visto, registrado en línea
tokens-cube-watched-here = Visto aquí, aún sin registrar
    .inline = visto aquí, aún sin registrar
tokens-cube-on-disk = En el disco
    .inline = en el disco
tokens-cube-downloading = Descargando
    .inline = descargando
tokens-cube-missing = Falta
    .inline = falta
tokens-cube-error = Error
    .inline = error
tokens-cube-not-downloaded = Sin descargar
    .inline = sin descargar
tokens-cube-not-aired = Aún sin emitir
    .inline = aún sin emitir
tokens-cube-available = Disponible
    .inline = disponible
tokens-cube-picked = Elegido para obtener
    .inline = elegido para obtener
tokens-cube-already-on-disk = Ya en el disco
    .inline = ya en el disco
tokens-cube-no-release = Sin lanzamiento
    .inline = sin lanzamiento

## Card

card-not-interested = No me interesa

## PagePlaceholder

page-placeholder-coming-soon = Próximamente

## ErrorLine

error-line-hide-details = Ocultar detalles
error-line-details = Detalles
error-line-details-tip = Muestra qué falló, palabra por palabra, y cópialo
error-line-copied = Copiado.

## Readouts (apps/linux/src/format.rs): times, countdowns and lengths

fmt-just-now = justo ahora
fmt-now = ahora
fmt-ago = hace { $span }
fmt-in = en { $span }
fmt-seconds-short = { $n } s
fmt-minutes-short = { $n } min
fmt-hours-short = { $n } h
fmt-days-short = { $n } d
fmt-months-short = { $n } meses
fmt-years-short = { $n } a
fmt-years-months-short = { $years } a { $months } meses
fmt-countdown-days = { $d } d { $h } h { $m } min
fmt-countdown-hours = { $h } h { $m } min
fmt-countdown-days-seconds = { $d } d { $h } h { $m } min { $s } s
fmt-countdown-hours-seconds = { $h } h { $m } min { $s } s
fmt-countdown-minutes-seconds = { $m } min { $s } s
fmt-runtime-minutes = { $m } min
fmt-runtime-hours = { $h } h
fmt-runtime-hours-minutes = { $h } h { $m } min
fmt-episode = Episodio
fmt-size-b = { $size } B
fmt-size-kb = { $size } KB
fmt-size-mb = { $size } MB
fmt-size-gb = { $size } GB
fmt-size-tb = { $size } TB
fmt-episode-span = Episodios { $first } a { $through }

## apps/linux/src/problem.rs: every call error in a person's words, what happened and what to do

problem-report = Inténtalo de nuevo. Si sigue pasando, notifícalo desde el registro de actividad.
problem-try-later = Inténtalo más tarde.
problem-helper-unreachable = El ayudante de búsqueda no respondió.
problem-helper-unauthorized = El ayudante de búsqueda rechazó el inicio de sesión.
problem-helper-not-running = El ayudante de búsqueda no está en marcha.
problem-helper-missing = El ayudante de búsqueda no está instalado.
problem-helper-failed = El ayudante de búsqueda no pudo hacer eso.
problem-helper-no-login = El ayudante de búsqueda no puede iniciar sesión en el cliente de torrents.
problem-helper-no-login-next = Guarda la contraseña del cliente de torrents en Ajustes, en Descargas.
problem-torrent-unreachable = El cliente de torrents no respondió.
problem-torrent-unauthorized = El cliente de torrents rechazó el inicio de sesión.
problem-torrent-not-running = El cliente de torrents no está en marcha.
problem-torrent-missing = El cliente de torrents no está instalado.
problem-torrent-failed = El cliente de torrents no pudo hacer eso.
problem-downloads-unreachable-next = Comprueba que esté en marcha y que su dirección sea correcta en Ajustes, en Descargas.
problem-downloads-unauthorized-next = Revisa el usuario y la contraseña en Ajustes, en Descargas.
problem-downloads-not-running-next = Inícialo desde Ajustes, en Descargas.
problem-downloads-missing-next = Ajustes, en Descargas, explica cómo configurarlo.
problem-trash = { -app-name } no pudo mover los archivos a la papelera.
problem-trash-next = Comprueba que los archivos sean tuyos y que el disco no sea de solo lectura.
problem-source-list = Esa lista de fuentes no funcionó.
problem-source-list-next = Revisa su dirección en Ajustes, en Descargas, Listas de fuentes.
problem-remote-unreachable = { $name } no respondió.
problem-remote-unreachable-next = Puede que el sitio esté caído. Inténtalo más tarde, o busca en otra fuente.
problem-remote-unauthorized = { $name } rechazó la clave que se le dio.
problem-remote-unauthorized-next = Cambia su clave en Ajustes, en Descargas, Listas de fuentes.
problem-remote-failed = { $name } no pudo hacer eso.
problem-remote-failed-next = Inténtalo más tarde, o busca en otra fuente.
problem-provider-sign-in = { $name } rechazó el inicio de sesión de { -app-name }.
problem-provider-sign-in-next = Vuelve a conectarlo en Ajustes, en Cuentas.
problem-provider-turned-away = { $name } rechazó a { -app-name }.
problem-provider-busy = { $name } está ocupado ahora mismo.
problem-provider-busy-next = Espera un minuto y vuelve a intentarlo.
problem-provider-no-answer = { $name } no respondió.
problem-provider-no-answer-next = Comprueba tu conexión a internet y vuelve a intentarlo.
problem-provider-trouble = { $name } tiene problemas ahora mismo.
problem-provider-trouble-next = Inténtalo dentro de unos minutos.
problem-provider-rejected = { $name } no aceptó eso.
problem-not-found = { -app-name } ya no encuentra eso.
problem-not-found-next = Puede que se haya quitado. Vuelve a intentarlo para cargar la página de nuevo.
problem-invalid-next = Revísalo y vuelve a intentarlo.
problem-unavailable = Ahora mismo no se puede acceder a esa carpeta.
problem-unavailable-next = Conecta su disco o revisa la carpeta en Ajustes, en Biblioteca.
problem-not-connected = { $tracker } no está conectado.
problem-not-connected-next = Conéctalo en Ajustes, en Cuentas.
problem-io = { -app-name } no pudo leer o escribir un archivo.
problem-io-next = Comprueba que el disco tenga espacio y que la carpeta sea tuya, y vuelve a intentarlo.
problem-storage = { -app-name } no pudo guardar tu biblioteca.
problem-storage-next = Comprueba que el disco tenga espacio y vuelve a intentarlo.
problem-keyring = El almacén de contraseñas del sistema no respondió.
problem-keyring-next = Desbloquea el llavero de tu escritorio y vuelve a intentarlo.
problem-unsupported = { -app-name } aún no puede hacer eso aquí.
problem-version = Esta copia viene de un { -app-name } más nuevo.
problem-version-next = Actualiza { -app-name } en Ajustes, en Acerca de, y vuelve a importarla.
problem-internal = Algo falló dentro de { -app-name }.
problem-refused-hidden = Esa serie está oculta.
problem-refused-hidden-next = Muéstrala primero desde su página.
problem-refused-no-match = Esa serie aún no está reconocida.
problem-refused-no-match-next = Reconócela primero en Info de la serie.
problem-refused-not-newer = Tu lista ya va por delante de este episodio.
problem-refused-extra = Ese archivo es un extra, no un episodio.
problem-refused-unmatched = Ese archivo no está asociado a ningún episodio.
problem-refused-unmatched-next = Corrígelo desde la página de la serie, en Corregir.
problem-refused-on-disk = Ese archivo solo está en este ordenador.

## apps/linux/src/library_view.rs: the library page's section titles and the one fact under each

library-view-airing = En emisión
library-view-no-date = Sin fecha
library-view-type-tv = TV
library-view-type-movies = Películas
library-view-type-extras = OVA, ONA y especiales
library-view-type-music = Música
library-view-type-unknown = Desconocido
library-view-no-genre = Sin reconocer
library-view-state-caught-up = Al día
library-view-state-not-started = Sin empezar
library-view-next-episode = Siguiente episodio en { $span }
library-view-countdown-days = { $d } d { $h } h
library-view-countdown-hours = { $h } h { $m } min
library-view-countdown-minutes = { $m } min
library-view-in-progress = { $count } en curso
library-view-not-started = { $count } sin empezar
library-view-all-watched = Todo visto
library-view-on-disk =
    { $count ->
        [one] { $count } episodio
       *[other] { $count } episodios
    } en el disco
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } archivo
       *[other] { $count } archivos
    } aún en el disco

## apps/linux/src/dbus/notify.rs: the desktop notification for a new episode

notify-episode-out = Ya está disponible el episodio { $episode } de { $title }
notify-click-open = Haz clic para abrirlo en { -app-name }
notify-click-get = Haz clic para obtener los episodios en { -app-name }
notify-get-episodes = Obtener episodios

## apps/linux/src/nowplaying.rs: the second line a media widget shows while an episode plays

now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: why the screen stays on, as the desktop's power panel shows it

inhibit-reason = Reproduciendo un episodio

## apps/linux/src/bridge/door.rs: why a value a page sent cannot be used (an error's detail line)

door-not-a-choice = { $value } no es una opción válida
door-tokens-op-empty = una op tokens necesita tokens
door-not-a-refine-op = { $value } no es una op de filtrado
door-limit-too-low = debe ser al menos 1, se recibió { $limit }
door-not-a-torrent-action = { $value } no es una acción de torrent

## apps/linux/src/bridge/shell.rs: the first-run starter folder and the update notice

shell-starter-folder-failed = No se pudo crear { $folder }: { $error }
shell-update-gone = { -app-name } { $version } ya no está disponible

## Watch together (#846)

onboarding-step-together = Ver juntos
onboarding-title-together = Ver con amigos
onboarding-body-together = Abre una sala desde el reproductor y envía su código. Cada uno reproduce su propia copia del episodio, y reproducir, pausar y saltar van sincronizados para todos.
onboarding-ready-together = Ver juntos
onboarding-ready-together-as = En las salas apareces como { $name }.
onboarding-ready-together-none = Aún sin nombre. El panel de la sala lo pide la primera vez.
onboarding-together-you = Tú
onboarding-together-code = Código de sala
onboarding-together-episode = Frieren · Episodio 3
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = Tu nombre en las salas
rail-room = Ver juntos (Ctrl+J)
rail-room-in = En una sala para ver juntos (Ctrl+J)
rail-join-room = Unirse a sala
rail-in-room = En una sala
player-chrome-watch-together = Ver juntos
player-chrome-room = Sala { $code } · { $count ->
    [one] 1 persona
   *[other] { $count } personas
}
room-name-empty = Escribe el nombre que verán los demás en la sala.
room-name-bad = Ese nombre no vale. Usa de 1 a 32 caracteres.
room-error-not-found = Ninguna sala tiene ese código. Compruébalo con quien te lo envió; puede que la sala se haya cerrado.
room-error-full = Esa sala está llena. Caben 10 personas por sala.
room-error-unreachable = No se pudo llegar al servidor de salas. Comprueba tu conexión y vuelve a intentarlo.
room-error-refused = El servidor de salas lo rechazó. Vuelve a intentarlo en un minuto.
room-error-proto = Esa sala usa una versión más nueva de las salas. Actualiza { -app-name } para unirte.
room-error-not-in-room = Ya no estás en esa sala. Abre una o únete a otra.
room-error-bad-code = Un código de sala tiene 6 letras y dígitos, como K7QM2X.
room-error-bad-relay = El servidor de salas en Ajustes, Reproducción no es una dirección web. Corrígelo ahí y vuelve a intentarlo.
room-panel-title = Ver juntos
room-panel-close = Cerrar
room-panel-connected = Conectado
room-panel-reconnecting = Reconectando
room-panel-name-label = Tu nombre en las salas
room-panel-name-placeholder = Tu nombre
room-panel-name-start = Abrir la sala
room-panel-name-join = Unirse a la sala
room-panel-start = Abrir una sala
room-panel-or-join = O únete a una con su código
room-panel-join-label = Código de sala
room-panel-join = Unirse
room-panel-you-are = Apareces como { $name }
room-panel-change-name = Cambiar
room-panel-code-label = Código de sala
room-panel-copy = Copiar enlace
room-panel-link-copied = Enlace de la sala copiado
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = Este episodio no está en tu biblioteca, así que aún no puedes verlo con los demás.
room-panel-open-episode = Abrir el episodio
room-panel-people = { $count ->
    [one] 1 persona
   *[other] { $count } personas
}
room-panel-host = Anfitrión
room-panel-you = Tú
room-panel-ready = Listo
room-panel-opening = Abriendo
room-panel-missing = No lo tiene
room-panel-offset = Tu desfase
room-panel-offset-earlier = Medio segundo antes
room-panel-offset-later = Medio segundo después
room-panel-seconds = { $value } s
room-panel-suggested = Las intros sugieren { $offset }
room-panel-use-suggested = Usarlo
room-panel-leave = Salir de la sala
player-room-speed = La velocidad la marca la sala
settings-find-room-name =
    .words = ver juntos sala nombre amigos
settings-find-room-relay =
    .words = ver juntos sala relay servidor
key-help-key-ctrl-j = Ctrl+J
key-help-room = Ver juntos
settings-playback-together = Ver juntos
settings-playback-room-name = Tu nombre en las salas para ver juntos
settings-playback-room-name-placeholder = Tu nombre
settings-playback-room-relay = Servidor de salas
settings-playback-room-relay-helper = Aquí se crean las salas. Cámbialo solo por un servidor que tengas tú.
