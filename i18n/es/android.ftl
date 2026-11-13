## Names that stay names

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Words many pages share. Use these before adding a page's own copy.

common-cancel = Cancelar
common-back = Atrás
common-save = Guardar
common-done = Hecho
common-close = Cerrar
common-delete = Eliminar
common-retry = Reintentar
common-copy = Copiar
common-clear = Borrar
common-on = Activado
common-off = Desactivado
common-play = Reproducir
common-hide = Ocultar
common-remove = Quitar
common-add = Añadir
common-test = Probar
common-reading = Leyendo...
common-working = Trabajando...
common-checking = Comprobando...
common-all = Todo
common-none = Ninguno
common-episodes = { $count ->
    [one] { $count } episodio
   *[other] { $count } episodios
}
common-episode-n = Episodio { $n }
common-join-comma = { $a }, { $b }

## Language (#793)

lang-follow-phone = Igual que el móvil
more-language = Idioma
more-language-phone = Igual que el móvil: { $lang }
lang-name-en = Inglés
lang-name-da = Danés
lang-name-de = Alemán
lang-name-es = Español
lang-name-fr = Francés
lang-name-pt-BR = Portugués (Brasil)
lang-name-it = Italiano
lang-name-nl = Neerlandés
lang-name-pl = Polaco
lang-name-ru = Ruso
lang-name-ja = Japonés
lang-name-ko = Coreano
lang-name-zh-CN = Chino (simplificado)

## Onboarding

onb-welcome-title = AniBeam
onb-welcome-note = Tu anime en este móvil. Busca una serie, mira quién la tiene, descárgala y mantén tus listas al día.
onb-language-title = Idioma
onb-language-note = AniBeam habla el idioma de tu móvil, salvo que elijas otro. Puedes cambiarlo después en Más.
onb-lists-title = Tus listas
onb-lists-note = Inicia sesión para que lo que veas aquí cuente en AniList o MyAnimeList. Puedes hacerlo después en Más.
onb-wifi-title = Solo Wi-Fi
onb-wifi-note = Los episodios pesan mucho. AniBeam descarga con Wi-Fi, salvo que digas lo contrario.
onb-storage-title = Espacio
onb-storage-note = Un límite de lo que AniBeam puede descargar. No reserva nada: el espacio sigue libre hasta que lo llenen los episodios.
onb-notify-title = Notificaciones
onb-notify-note = Mientras se descargan episodios, una notificación muestra el avance y otra avisa cuando uno está listo para ver. Las descargas se hacen igualmente.
onb-sites-title = Dónde buscar
onb-sites-note = AniBeam no trae nada donde buscar. Un repo es una lista firmada de sitios de búsqueda. Pega la dirección de un repo, comprueba su clave e instala los sitios que quieras. Puedes hacerlo después en Más.
onb-computer-title = Tu ordenador
onb-computer-note = AniBeam en un ordenador puede compartir su biblioteca con este teléfono por Tailscale: sus series aparecen en Biblioteca y sus episodios se reproducen aquí. Escribe su nombre de Tailscale y el puerto, y luego Probar. ¿Sin ordenador? Déjalo vacío y empieza; Más lo tiene para después.
onb-wifi-only = Solo con Wi-Fi
onb-wifi-on = Activado. Con datos móviles pregunta antes.
onb-wifi-off = Desactivado. Las descargas también usan datos móviles.
onb-storage-caption = 18 GB son unos 12 episodios. Desliza hasta todo el almacenamiento del móvil.
onb-use-cap = Usar este límite
onb-cap-on = Activado. Al llegar a { $cap }, AniBeam no descarga más hasta que elimines episodios.
onb-cap-off = Desactivado. AniBeam descarga hasta llenar el móvil.
onb-start = Empezar
onb-continue = Continuar
onb-back = Atrás

## Appearance

look-colours = Colores
look-mode = Modo
look-dark = Oscuro
look-light = Claro
look-system = Sistema
look-follows-phone-dark = Sigue el ajuste del móvil: oscuro.
look-follows-phone-light = Sigue el ajuste del móvil: claro.
look-changes-at-once = Los cambios se ven al instante, aquí y en todas las páginas.
look-dark-theme = Tema oscuro
look-light-theme = Tema claro
look-accent = Acento
look-accent-note = Marca lo seleccionado, lo que suena y lo nuevo.
look-size = Tamaño
look-text-size = Tamaño del texto
look-size-smaller = Menor
look-size-normal = Normal
look-size-large = Grande
look-size-larger = Mayor
look-size-largest = Máximo
look-text-size-note = El tamaño del texto en todo AniBeam, además del tamaño de tu móvil.

## The window: tabs and the sheets over every page

app-title = AniBeam
phone-nav-home = Inicio
nav-calendar = Calendario
phone-nav-library = Biblioteca
phone-nav-downloads = Descargas
nav-more = Más
app-mobile-title = ¿Descargar con datos móviles?
app-mobile-note = Un episodio suele pesar de 1 a 2 GB. Con una tarifa limitada, puede gastar los datos de un mes en una tarde.
app-mobile-keep = Solo con Wi-Fi
app-mobile-use = Usar datos móviles
app-versions-note = El mismo episodio en { $count } lanzamientos. Elige cuál reproducir.
app-versions-plays-now = { $release }, en reproducción
app-menu-get = Obtener episodios
app-menu-delete = Eliminar episodios

## More: the index and each section's line of how it stands

more-settings = Ajustes
more-search = Buscar ajustes
more-search-none = Sin resultados
more-search-hits = { $count ->
    [one] 1 ajuste
   *[other] { $count } ajustes
}
more-search-tap = Toca uno para ir a él
more-search-hint = Prueba con una palabra más simple: almacenamiento, Wi-Fi, tracker, sitio, ordenador.
more-accounts = Cuentas
more-streaming = Streaming
more-subtitles = Subtítulos
more-connection = Conexión
more-computer = Tu ordenador
more-icon = Icono
more-stats = Estadísticas
more-data = Datos
more-appearance = Apariencia
more-activity = Actividad
more-about = Acerca de
more-accounts-none = Ningún tracker conectado
more-accounts-syncing = Sincronizando con { $tracker }
more-accounts-syncing-both = Sincronizando con AniList y MAL
more-dl-status-wifi = Solo Wi-Fi, sin límite
more-dl-status-wifi-cap = Solo Wi-Fi, límite de { $cap }
more-dl-status-mobile = Wi-Fi y datos móviles, sin límite
more-dl-status-mobile-cap = Wi-Fi y datos móviles, límite de { $cap }
more-stream-status-auto = Auto, 720p con datos móviles o relé
more-stream-status-original = El archivo original, siempre
more-stream-status-converted = Convertido a { $quality }
more-sites-status-none = Aún sin repo
more-sites-status-on = { $sites ->
    [one] 1 sitio activo
   *[other] { $sites } sitios activos
}
more-sites-status-on-from = { $sites ->
    [one] 1 sitio activo
   *[other] { $sites } sitios activos
}, { $repos ->
    [one] de 1 repo
   *[other] de { $repos } repos
}
more-conn-status-unchecked = Sin comprobar
more-conn-status-ok = Todos los pasos funcionaron
more-conn-status-failed = { $total ->
    *[other] { $failed } de { $total } pasos fallaron
}
more-computer-status-none = Aún sin conectar
more-icon-default = Estándar
more-icon-dark = Oscuro
more-icon-line = Lineal
more-subs-status = Tamaño, posición, fuente, colores y fondo
more-data-status = Copia, restauración, caché de imágenes
more-activity-status = Lo que hizo AniBeam, lo más reciente primero
more-look-status-phone = Sigue al móvil
more-stats-status = { $count ->
    [one] { $hours } hora vista en { $year }
   *[other] { $hours } horas vistas en { $year }
}
more-stats-status-empty = Un año de anime

## More: the sections' own settings

more-accounts-note = Lo que veas aquí marca tus listas, y el progreso que traen se muestra en tus series.
more-storage = Espacio
more-storage-capped = Límite de { $cap }
more-storage-no-cap = Sin límite
more-storage-used = usado por episodios
more-storage-cap-caption = límite
more-storage-free = libres en el móvil
more-set-storage-cap = Límite de espacio
more-storage-cap-on = Al llegar a { $cap }, no se descarga nada más hasta que elimines episodios.
more-storage-cap-off = Desactivado: AniBeam descarga hasta llenar el móvil.
more-network = Red
more-network-wifi = Solo Wi-Fi
more-network-mobile = Wi-Fi y datos móviles
more-wifi-on = Activado. Las descargas esperan al Wi-Fi; con datos móviles pregunta antes.
more-set-stream-quality = Calidad de vídeo
more-stream-question = Cómo se reproduce aquí un episodio de tu ordenador.
more-stream-auto-note = Auto reproduce el archivo tal cual con Wi-Fi, y convertido a 720p con datos móviles o cuando Tailscale llega a tu ordenador a través de un relé.
more-stream-original-note = El archivo tal cual, siempre. Los archivos grandes necesitan una conexión rápida.
more-stream-converted-note = Se convierte en tu ordenador a { $quality }. Gasta menos datos; al saltar en el vídeo hay una breve espera.
more-set-feeds = Tus propios feeds
more-feeds-note = Se buscan antes que en los sitios
more-set-check = Comprobar la conexión
more-check-note = Prueba cada paso entre este móvil y los sitios de búsqueda: encontrarlos, llegar a ellos y la conexión segura.
more-check-copy = Copiar los resultados
more-set-computer = AniBeam de tu ordenador
more-computer-note = Su biblioteca aparece en Biblioteca y sus episodios se reproducen aquí. Un nombre como mi-ordenador o una dirección como 100.64.0.1.
more-computer-placeholder = mi-ordenador:8790
more-subs-note = Con lo que empieza cada episodio de este móvil. Uno que llega por streaming desde tu ordenador empieza con los ajustes de ese ordenador.
more-icon-note = El icono de tu pantalla de inicio y del cajón de apps.
more-icon-line-note = El icono lineal toma los colores de tu fondo de pantalla, como tus otros iconos de Material You.
more-icon-moved-note = Algunos lanzadores quitan el icono de la pantalla de inicio al cambiarlo. Sigue en el cajón de apps; arrástralo de vuelta desde ahí.
more-set-whats-new = Novedades
more-whats-new-note = Qué cambió en cada versión, la más reciente primero.
more-set-onboarding = Volver a empezar
more-onboarding-note = Repasa otra vez las preguntas del primer inicio.
more-set-version = Versión

## Status lines Rust writes on the pages

host-synced = Sincronizado { $when }.
host-last-try = Último intento { $when }: { $error }
host-asking = Leyendo...
host-reached = Conectado a { $host }, AniBeam { $version }.
host-saved = Guardado. Sincronizando...
host-none = Sin ordenador. Este teléfono se queda con su propia biblioteca.
acct-signed-in-as = Sesión iniciada como { $name }
acct-signed-in = Sesión iniciada
acct-not-in-build = No disponible en esta versión
acct-tap-to-sign-in = Sin iniciar sesión. Toca para iniciarla.
acct-sign-in-failed = Error al iniciar sesión: { $why }. Toca para reintentar.
acct-sign-in-not-started = No se pudo iniciar sesión: { $error }. Toca para reintentar.
acct-still-starting = AniBeam aún está arrancando. Inténtalo de nuevo en un momento.
home-recently-watched = Vistas recientemente
home-recently-downloaded = Descargadas recientemente
home-note-empty = En este móvil
home-note = { $count ->
    [one] En este móvil, 1 serie
   *[other] En este móvil, { $count } series
}
lib-this-phone = Este móvil
lib-note-phone = { $count ->
    [one] Este móvil, 1 serie
   *[other] Este móvil, { $count } series
}
lib-note-remote = { $device }, sincronizado { $when }
lib-empty-search = Sin resultados para "{ $query }".
lib-empty-phone = Aún no hay nada en este móvil.
lib-empty-remote = Ese dispositivo no tiene nada en su biblioteca.
cal-view-airing = En emisión
cal-note-none = Esta semana no se emite nada
cal-note = { $count ->
    [one] 1 episodio en los próximos 7 días
   *[other] { $count } episodios en los próximos 7 días
}
airing-asking = Leyendo AniList
airing-asking-newest = Leyendo lo más nuevo de AniList
phone-airing-filter-all = Todo { $count }
airing-filter-listed = En mis listas { $count }
airing-filter-owned = En este móvil { $count }
airing-this-season = Esta temporada
airing-next-season = Próxima temporada
airing-other-season = Otras
airing-hidden = { $count ->
    *[other] { $count } ocultas por no interesar
}
airing-plan-needs-tracker = Para marcar "Quiero verlo" hace falta AniList o MAL. Inicia sesión en Más, Cuentas.
airing-plan-failed = No se pudo marcar "Quiero verlo": { $error }
list-watching = Viendo
list-planning = Planeado
list-completed = Completado
list-paused = En pausa
list-dropped = Abandonado
list-repeating = Repitiendo
more-check-all-ok = Todos los pasos funcionaron.
more-check-failed = { $failed } de { $total } pasos fallaron. Una captura de esta lista muestra dónde.
more-check-row = { $step }  { $ms } ms
more-check-copied = Copiado. Pégalo en un mensaje.
more-version = AniBeam para Android { $version } (compilación { $build })
dl-note-waiting = Esperando al Wi-Fi. Las descargas siguen solas cuando el móvil tenga Wi-Fi.
dl-note-nothing = Nada en descarga.
dl-state-done = Hecho
dl-state-waiting = Esperando Wi-Fi
dl-state-paused = En pausa
dl-state-peers = Buscando pares
dl-state-checking = Comprobando
dl-state-error = Detenida por un error
dl-state-downloading = Descargando
dl-amount = { $got } de { $size }
dl-speed = { $rate }/s
dl-left = faltan { $span }
dl-episodes-range = Episodios { $from } a { $to }
dl-batch = Un lote
disk-storage-line = { $used } usados
disk-storage-line-free = { $used } usados, { $free } libres
disk-free = Este móvil tiene { $free } libres.
disk-free-below-cap = Este móvil tiene { $free } libres, menos que el límite: el móvil se llena antes de que AniBeam pare.

## More's search: the words a person might type for each setting

more-set-subs-size = Tamaño y posición de subtítulos
more-set-subs-style = Estilo de subtítulos
more-words-anilist = anilist tracker iniciar sesión cuenta conectar lista progreso
more-words-mal = mal myanimelist tracker iniciar sesión cuenta conectar lista progreso
more-words-storage-cap = espacio disco límite gb lleno almacenamiento eliminar tamaño
more-words-wifi = wifi wi-fi móvil datos red celular
more-words-notifications = notificación notificaciones aviso alerta progreso permiso permitir
more-words-stream-quality = auto original 1080p 720p 480p convertir datos ordenador reproducir
more-words-check-repos = actualizar refrescar repositorio repo endpoint
more-words-add-repo = repositorio repo endpoint confiar huella dirección url
more-words-add-site = personalizado endpoint feed rss torznab jackett prowlarr dirección
more-words-feeds = feed rss fuente quitar
more-words-check = red dns prueba https seguro conexión internet
more-words-computer = tailscale tailnet host dirección ordenador escritorio biblioteca streaming probar guardar dispositivo
more-words-whats-new = novedades cambios registro versión actualización
more-words-onboarding = bienvenida primer inicio empezar de nuevo configurar
more-words-version = compilación build acerca versión
more-words-stats = horas episodios géneros estudios temporadas año vistos resumen mes historial
more-words-backup = copia respaldo exportar archivo guardar mover ordenador json privado sesiones
more-words-restore = restaurar importar archivo cargar biblioteca combinar
more-words-cache = imágenes carteles banners borrar almacenamiento espacio caché
more-words-activity = registro eventos errores problemas copiar borrar historial qué pasó
more-words-colours = apariencia tema oscuro claro modo sistema acento color
more-words-text-size = apariencia fuente letra más grande más pequeño tamaño texto leer
more-words-icon = icono lanzador pantalla de inicio logo oscuro lineal tema material you
more-words-subs-size = subtítulos tamaño escala posición subir inferior
more-words-subs-style = subtítulos fuente letra color contorno sombra fondo negrita ass estilo restablecer
more-words-language = idioma traducción inglés español configuración regional

## Home

phone-home-title = Inicio
home-next-to-air = Próximo estreno
home-after-that = Después
home-coming-up = Próximamente
home-nothing-week = Nada en la próxima semana.
phone-home-empty-title = Aún no hay nada en este móvil
phone-home-empty-body = Las series que descargues aquí, y lo que se emite de ellas, aparecen en Inicio. Busca una en Descargas.

## Library

lib-title = Biblioteca
lib-refresh = Actualizar
lib-search-phone = Buscar en este móvil
lib-search-device = Buscar en { $device }
lib-find-in-downloads = Buscar "{ $query }" en Descargas
lib-empty-body = Los episodios que descargues en este móvil aparecen aquí. Busca una serie en Descargas.
lib-pick-show = Elige una serie para abrirla aquí.
lib-shows-caption = { $count ->
    [one] serie
   *[other] series
}
lib-on-phone = en el móvil
lib-free = libres
lib-hold-poster = Mantén pulsado un cartel para obtener más episodios o eliminarlos.
phone-nothing-on-phone = nada en el móvil
phone-on-phone = { $count } en el móvil

## Calendar

cal-title = Calendario
cal-week = Semana
cal-days = Días
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A %-d de %B
cal-today = Hoy
cal-tomorrow = Mañana
cal-yesterday = Ayer
cal-nothing = Nada
cal-next = Próximo
cal-get = Obtener
cal-day-note = { $date }, { $count }
cal-next-airing = Próxima emisión: { $title }
cal-nothing-aired-yesterday = Ayer no se emitió nada.
cal-nothing-aired-that-day = Ese día no se emitió nada.
cal-nothing-today = Hoy no se emite nada.
cal-nothing-tomorrow = Mañana no se emite nada.
cal-nothing-that-day = Ese día no se emite nada.
cal-empty-title = Aún no hay series en este móvil
cal-empty-body = Los episodios de las series que guardas aquí aparecen en su día, dos días atrás y una semana por delante.
phone-episodes-range = Episodios { $from } a { $to }
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = En el móvil
phone-out-ago = Salió { $ago }

## Airing now

phone-airing-out-now = Disponibles
airing-last-two-days = Los últimos dos días
airing-season-note = Las series de la temporada vienen de AniList.
airing-none-lists-title = Nada de lo que se emite está en tus listas
airing-none-lists-body = "Quiero verlo" en una serie la añade a tu lista.
airing-none-phone-title = Nada de lo que se emite está en este móvil
airing-none-phone-body = "Obtener episodios" en una serie la descarga aquí.
airing-marked = { $title } marcada como no interesa.
phone-airing-undo = Deshacer
airing-plan = Quiero verlo
airing-get-more = Obtener más episodios
airing-get = Obtener episodios
phone-airing-not-interested = No me interesa

## Seasons

season-winter = Invierno
season-spring = Primavera
season-summer = Verano
season-fall = Otoño
season-named = { $season } { $year }
season-pick = Elige temporada

## A show's page

series-back-home = Inicio
series-back-library = Biblioteca
series-back-downloads = Descargas
series-back-calendar = Calendario
phone-series-get-episodes = Obtener episodios
phone-series-episodes = Episodios
series-delete-episodes = Eliminar episodios
series-delete-tap = Toca los episodios que quieras eliminar.
phone-series-delete-picked = { $count ->
    [one] { $count } episodio elegido.
   *[other] { $count } episodios elegidos.
}
series-delete-note = Sus archivos se eliminan de este móvil. Lo que viste sigue registrado.
series-delete-button = { $count ->
    [one] Eliminar { $count } episodio
   *[other] Eliminar { $count } episodios
}
series-no-episodes = No hay episodios en este móvil ni programados.
series-versions = { $count ->
    [one] { $count } versión
   *[other] { $count } versiones
}
series-note-next-up = Siguiente
series-note-missing = Falta
series-note-not-here = No está en este móvil
series-note-not-downloaded = Sin descargar
series-note-downloading = Descargando { $percent }%
series-note-resume-at = Reanudar en { $time }
phone-episodes-estimate = { $count ->
    [one] { $count }? episodio
   *[other] { $count }? episodios
}
phone-next-airs = El episodio { $n } sale en { $time }
phone-watched = Visto
phone-not-watched = Sin ver
phone-progress = { $watched } de { $total } vistos, { $count } en este móvil
phone-on-this-phone = { $count } en este móvil
phone-play-film = Ver la película
phone-resume-film = Reanudar la película
phone-play-episode = Ver episodio { $n }
phone-resume-episode = Reanudar episodio { $n }
phone-about-date = hacia el { $date }

## A show's list status, score, notes and fixes

show-status-watching = Viendo
show-status-planning = Planeado
show-status-completed = Completado
show-status-paused = En pausa
show-status-dropped = Abandonado
show-status-repeating = Repitiendo
show-status-now = { $label }, ahora
phone-show-add-to-list = Añadir a lista
phone-show-your-score = Tu nota { $score }
show-rate = Puntuar
show-fixes = Corregir
show-your-notes = Tus notas
show-add-notes = Añadir notas
show-home-each = Cámbialo en cada lista conectada.
show-home-no-tracker = Se guarda en este móvil: ningún tracker tiene esta serie.
show-home-tracker-lacks = Se guarda en este móvil: tu tracker no tiene esta serie.
show-home-until-connected = Se guarda en este móvil hasta que conectes AniList o MyAnimeList.
show-sites-both = AniList y MyAnimeList
show-notes-title = Notas
show-notes-placeholder = Por dónde ibas, quién la recomendó, qué versión obtener
show-notes-over = { $count ->
    [one] { $count } carácter por encima del límite
   *[other] { $count } caracteres por encima del límite
}
show-notes-saved-to = Guardado en { $sites }.
show-notes-saved-to-tracker = Guardado en tu tracker.
show-notes-saving = Guardando tus notas.
show-notes-clearing = Borrando tus notas.
show-score-saving = Guardando tu nota.
show-score-saved = Nota guardada.
show-score-clearing = Borrando tu nota.
show-score-cleared = Nota borrada.
phone-show-moving-to = Pasando a { $label }.
phone-show-moved-to = Pasada a { $label }.
show-rescan-started = Releyendo carpeta.
show-still-starting = AniBeam aún está arrancando. Inténtalo de nuevo en un momento.
show-fix-match = Asociar
show-fix-find-match = Buscar serie
show-fix-rescan = Releer carpeta

## A show's characters, related shows and recommendations

show-characters = Personajes
phone-show-related = Relacionado
show-related-all = Ver las { $count }
show-related-filling = AniList aún está completando los datos.
phone-show-recommendations = Recomendaciones
show-card-owned = En tu biblioteca
show-card-unreleased = Aún sin estrenar
show-card-on-anilist = En AniList
show-rec-available = Disponible
show-relation-prequel = Precuela
show-relation-sequel = Secuela
show-relation-side-story = Paralela
show-relation-parent = Principal
show-relation-spin-off = Spin-off
show-relation-adaptation = Adaptación
show-relation-alternative = Alternativa
show-relation-summary = Resumen
show-relation-character = Personaje
show-relation-source = Origen
show-relation-compilation = Recopilación
show-relation-contains = Contiene
show-relation-other = Otra
show-role-main = Principal
show-role-supporting = Secundario
show-role-background = De fondo
show-format-tv = TV
show-format-tv-short = TV corto
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Especial
show-format-film = Película
show-entry-get = Obtener episodios
show-entry-open-anilist = Abrir en AniList
show-entry-line = { $line }, { $status }

## Fix a wrong match

phone-match-title = Asociar
match-refresh = Actualizar info
phone-match-clear = Desasociar
match-clear-it = Desasociar
match-clear-warning = Al desasociar se pierden el enlace con el tracker y el número de episodios fijado a mano. Toca de nuevo para desasociar.
match-tvmaze = TVmaze
match-search-anilist = Buscar en AniList
match-search-tvmaze = Buscar en TVmaze
match-paste-anilist = Pega un enlace de AniList o MyAnimeList
match-paste-tvmaze = Pega un enlace de TVmaze o IMDb
phone-match-look-up = Buscar
match-apply = Aplicar
match-look-up-all = Buscar todas las carpetas que AniBeam aún no reconoce
match-format-music = Música
phone-match-matched-on = Reconocida en { $sites }
phone-match-found-tvmaze = Serie encontrada en TVmaze
phone-match-found-anilist = Encontrada en AniList
phone-match-found-anilist-season = Encontrada en AniList, temporada { $season }
phone-match-found-mal = Encontrada en MyAnimeList
phone-match-searching = Buscando en { $name }
phone-match-type-more = Escribe al menos dos letras para buscar en { $name }.
match-none = No se encontró ningún título en { $name }.
match-looking-up = Buscando.
phone-match-matched-to = Asociada a { $title }.
phone-match-applied = Asociación hecha.
match-applying = Aplicando asociación.
phone-match-refreshing = Actualizando { $title }.
match-clearing = Quitando la asociación.
phone-match-cleared = Desasociada.
match-auto-looking = Buscando las carpetas que AniBeam aún no reconoce.

## Batches (the line under a release in a search)

inspect-reading = Leyendo su lista de archivos
inspect-holds = { $count ->
    [one] { $count } episodio: { $runs }
   *[other] { $count } episodios: { $runs }
}

## What's new

phone-notes-title = Novedades
notes-got-it = Entendido

## Move the list to a tracker

move-lists-label = Listas de este móvil
move-title = Pasar tu lista a { $site }
move-here = En este móvil: { $list }
move-there = Ahora en { $site }: { $list }
move-later-hint = Después: Más, Cuentas
move-not-now = Ahora no
move-nothing = Nada
move-not-on-list = No está en tu lista
move-nothing-left = No queda nada por pasar
move-side-status-score = { $status }, nota { $score }
move-side-score = nota { $score }
move-lead = { $count ->
    [one] Una serie tiene estado o nota solo en este móvil. Al pasarla, queda en tu lista de { $site }, donde la ven tu ordenador y tu perfil. En estas series, lo que { $site } tiene ahora se reemplaza. No se envía nada hasta que pulses Pasar.
   *[other] { $count } series tienen estado o nota solo en este móvil. Al pasarlas, quedan en tu lista de { $site }, donde las ven tu ordenador y tu perfil. En estas series, lo que { $site } tiene ahora se reemplaza. No se envía nada hasta que pulses Pasar.
}
move-label = { $count ->
    [one] Pasar { $count } serie
   *[other] Pasar { $count } series
}
move-waiting = { $count ->
    [one] Una serie tiene estado o nota solo aquí. Pásala a { $site } para verla en tu ordenador y en tu perfil.
   *[other] { $count } series tienen estado o nota solo aquí. Pásalas a { $site } para verlas en tu ordenador y en tu perfil.
}
move-moving = { $count ->
    [one] Pasando { $count } serie a { $site }
   *[other] Pasando { $count } series a { $site }
}
move-moved = { $count ->
    [one] { $count } serie pasada a { $site }
   *[other] { $count } series pasadas a { $site }
}
move-failed = { $count ->
    [one] { $site } no aceptó { $count } serie; mira el registro de actividad
   *[other] { $site } no aceptó { $count } series; mira el registro de actividad
}

## Downloads: find (search, shows, sources, releases)

common-join-and = { $a } y { $b }
find-join-sentences = { $a } { $b }

find-downloads = Descargas

find-lens-list = Tu lista
find-lens-gaps = Faltan
find-lens-airing = En emisión
find-lens-following = Siguiendo
find-lens-downloaded = Descargadas
find-lens-with-count = { $lens } { $count }
find-what-to-get = Qué obtener
find-lens-list-line = Viendo y Planeado de tu lista. Abre una para obtener sus episodios.
find-lens-gaps-line = Episodios emitidos que no están en este móvil. Abre una serie para ver quién los tiene.
find-lens-airing-line = Episodios nuevos por venir, los más próximos primero.
find-lens-following-line = Las suscripciones: los episodios nuevos se descargan solos. Abre una para dejar de seguirla.
find-lens-downloaded-line = Lo que descargó este móvil, serie por serie.

find-lens-gaps-empty = Nada falta: cada serie que empezaste tiene aquí sus episodios emitidos.
find-lens-airing-empty = Nada de este móvil está en emisión.
find-lens-following-empty = Aún no hay suscripciones. Abre una serie y activa Seguir.
find-lens-downloaded-empty = Aún no hay descargas.
find-list-empty = Tu lista está vacía. Busca una serie arriba y añádela.
find-list-reading = Leyendo tu lista...
find-list-signed-out = Inicia sesión en AniList en Más, Cuentas, y tu lista aparecerá aquí.
find-list-failed = No se pudo leer tu lista: { $error }

find-aired = emitidos
find-ledger-no-feed = sin episodios nuevos: su fuente no tiene feed
find-ledger-downloads = { $count ->
    [one] { $count } descarga
   *[other] { $count } descargas
}
find-ledger-downloaded = descargados
find-ledger-planning = Planeado
find-ledger-watching-of = Viendo, { $progress } de { $total }
find-ledger-watching-seen = Viendo, { $progress } vistos

find-gap-missing = faltan { $count }
find-gap-not-downloaded = { $count } sin descargar

find-list-status-planning = En tu lista: Planeado
find-list-status-watching = En tu lista: Viendo
find-list-status-already = Ya está en tu lista
find-going-planning = { $title } pasa a Planeado.
find-already-listed = { $title } ya está en tu lista.
find-plan-needs-tracker = Para añadir a tu lista hace falta AniList o MAL. Inicia sesión en Más, Cuentas.
find-plan-failed = No se añadió a tu lista: { $error }
find-show-gone = Esa serie ya no está en la biblioteca de este móvil.

## Shows as you type

find-shows = Series
find-suggesting = Leyendo AniList...
find-suggest-tap = Toca una para ver quién la tiene. Intro busca nombres de lanzamientos.
find-suggest-none = Ninguna serie con ese nombre. Intro busca nombres de lanzamientos.
find-suggest-hint = Las series aparecen al escribir. Intro busca nombres de lanzamientos.
find-suggest-failed = No se pudo consultar a AniList: { $error }
find-library-line = En tu biblioteca
find-library-line-none = En tu biblioteca, ninguno de { $total } en este móvil
find-library-line-some = En tu biblioteca, { $here } de { $total } en este móvil

## The show view

find-plan-to-watch = Quiero verlo
find-on-phone-count = { $count } en este móvil
find-airing = En emisión
find-dropped = { $count ->
    [one] Se omite { $count } lanzamiento que no nombra esta serie.
   *[other] Se omiten { $count } lanzamientos que no nombran esta serie.
}

find-following = Siguiendo
find-stop-following-ask = ¿Dejar de seguir { $name }?
find-unfollow-explain = Los episodios nuevos dejan de descargarse. Lo que hay en su carpeta puede quedarse o irse con ella: todas las descargas de ahí, también los episodios que obtuviste a mano.
find-stop-following = Dejar de seguir
find-stop-keep = Dejar, conservar episodios
find-stop-delete = Dejar y eliminar descargas de su carpeta
find-rule-taken = { $downloads ->
    [one] { $downloads } descarga
   *[other] { $downloads } descargas
}, { $files ->
    [one] { $files } archivo
   *[other] { $files } archivos
} en su carpeta
find-rule-paused = Está en pausa, así que no llegan episodios nuevos. { $taken }.
find-rule-no-feed = Su fuente no tiene feed, así que no llegan episodios nuevos. { $taken }.
find-rule-active = Los episodios nuevos se descargan solos. { $taken }.
find-unfollowed-deleted = Se dejó de seguir { $name } y se eliminaron las descargas de su carpeta.
find-unfollowed-kept = Se dejó de seguir { $name }. Sus episodios siguen en este móvil.
find-stop-failed = No se pudo terminar de dejar de seguir: { $error }

find-where = Dónde obtenerlo
find-where-pick = Elige una fuente para ver qué tiene.
find-where-nothing = No se descarga nada hasta que lo digas.
find-source-idle = Toca para ver
find-source-trying = Probando { $name }
find-source-searching = Buscando
find-source-silent = No responde
find-no-sites-title = Sin sitios de búsqueda
find-no-sites-body = Añade en Más un repo de sitios de búsqueda, o un feed propio.
find-open-more = Abrir Más
find-pick-source-title = Elige una fuente
find-pick-source-body = Sus lanzamientos aparecen aquí en una cuadrícula, un cubo por episodio.
find-releases = { $count ->
    [one] { $count } lanzamiento
   *[other] { $count } lanzamientos
}
find-unreachable = No se pudo llegar a { $host }. Algunas redes y países bloquean los sitios de lanzamientos; AniBeam ya esquiva los bloqueos por nombre, así que este bloquea el sitio mismo, o el sitio está caído. Una VPN lo esquiva, o prueba otra fuente.
find-the-source = La fuente

find-has = Qué tiene
find-has-note = Toca una fila para llevártelo todo, o episodios sueltos.
find-episodes-tag = Episodios
find-batch = Lote { $label }
find-row-meta = { common-episodes }  { $seeds } semillas  { $size }
find-on-this-phone = En este móvil
find-on-computer = En { $name }

find-plan-no-releases = Esta fuente no tiene lanzamientos de esta serie.
find-plan-nothing-picked = Nada elegido: toca una fila o sus cubos.
find-plan-line = { $episodes } de { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] Se omite { $count } episodio que ya tienes. Activa Reemplazar para obtenerlo otra vez.
   *[other] Se omiten los episodios que ya tienes. Activa Reemplazar para obtenerlos otra vez.
}
find-plan-follow-note = Los episodios nuevos de { $query } se descargarán solos.
find-plan-on-desk = Se descargan en { $name }, en su biblioteca.
find-follow-new = Seguir episodios nuevos
find-follow-new-note = Se descargan solos según se emiten.
find-replace = Reemplazar lo que tienes
find-replace-note = Vuelve a descargar los episodios elegidos sobre los de este móvil.
find-go = { $mode ->
    [replace] Obtener { common-episodes } y reemplazar
    [subscribe] Obtener { common-episodes } y suscribir
    [both] Obtener { common-episodes }, reemplazar y suscribir
   *[plain] Obtener { common-episodes }
}
find-go-pick = Elige episodios que obtener
find-every-release = Todos los lanzamientos de esta fuente

## Download to

find-download-to = Descargar en
find-this-phone = Este móvil
find-your-computer = Tu ordenador
find-desk-asking = Consultando a { $name }...
find-desk-silent = { $name } no responde. Elige Este móvil, o inténtalo cuando esté encendido.
find-sending-to = Enviando a { $name }...

## The release list

find-best-fit = Mejor opción
find-on-disk = En disco
find-get = Obtener
find-get-all = Obtener los { $count }
find-subscribe = Suscribir
find-back-step = Un paso atrás
find-narrow-placeholder = Filtra, o -palabra para excluir
find-best-fit-searches = Mejor opción busca: { $query }
find-fact-batch = lote
find-fact-seeds = { $count } semillas
find-fact-via = vía { $source }
find-searching-for = Buscando { $text }...
find-search-line = { $count ->
    [0] Nada en { $source } para eso
    [one] { $count } lanzamiento en { $source }
   *[other] { $count } lanzamientos en { $source }
}{ $fell ->
    [1] , tras no encontrar nada en la primera fuente
   *[other] { "" }
}{ $narrowed ->
    [1] , acotado a { $token }
   *[other] { "" }
}.
find-refined = { find-releases }, { $delta } { $dir ->
    [down] menos
   *[up] más
}.
find-refine-nothing-left = Así no queda nada.
find-refine-no-match = Ninguno lo tiene.
find-refine-empty = Escribe algo primero.
find-refine-no-change = Eso no cambia nada.

## Getting and subscribing

find-getting = Obteniendo { $what }.
find-subscribed = Suscrito: los episodios nuevos se descargan solos.
find-subscribed-desk = Suscrito en { $name }: los episodios nuevos se descargan allí solos.
find-subscribed-as = Suscrito como { $name }: los episodios nuevos se descargan solos.
find-downloading = { $count ->
    [one] Descargando { $count } lanzamiento.
   *[other] Descargando { $count } lanzamientos.
}
find-not-added = No se añadió: { $error }
find-not-subscribed = No se suscribió: { $error }
find-desk-refused = { $name } no lo aceptó: { $error }
find-desk-not-subscribed = No se suscribió en { $name }: { $error }

## Player

player-subtitles = Subtítulos
player-audio = Audio
phone-player-speed = Velocidad
player-connecting = Conectando
player-hold-speed = 2× velocidad
player-replay = Repetir
player-unlock = Desbloquear
player-next-stay = Quedarse
player-next-go = Siguiente
player-jump-back = −10 s
player-jump-forward = +10 s
player-speed-now = Velocidad { $speed }
player-skip-intro = Saltar intro
player-skip-outro = Saltar outro
phone-player-skipped-intro = Intro omitida
phone-player-skipped-outro = Outro omitido
phone-player-undo = Deshacer
phone-player-start-over = Reiniciar
phone-player-resumed-at = Reanudado en { $time }
player-picture-failed = No se pudo iniciar la imagen. El sonido sigue.
player-file-failed = No se pudo reproducir este archivo: { $why }.
player-streaming-from = Streaming desde { $host } a { $height }p
player-mpv-no-start = mpv no arrancó
player-mpv-no-start-why = mpv no arrancó: { $why }
player-android-only = el reproductor solo funciona en Android

## Player: delays

player-delay-title = Retardo
player-delay-note = Cada toque lo mueve 0,1 s. Hasta el próximo archivo.
phone-player-delay-seconds = { $value } s
player-sub-delay-now = Retardo de subtítulos { $value }
player-audio-delay-now = Retardo de audio { $value }

## Player: cast

player-cast-playing-on = Reproduciendo en { $device }
phone-player-cast-stop = Detener envío a { $device }
player-cast-searching = Buscando Chromecasts y TV
phone-player-cast-to = Enviar a
phone-player-cast-tv = TV
phone-player-cast-loading = Cargando
phone-player-cast-playing = Reproduciendo
phone-player-cast-paused = En pausa
phone-player-cast-finished = Terminado
phone-player-cast-stopped = Detenido

## Player: the tracker's word

phone-player-rated = Puntuado
phone-player-score-failed = No se pudo guardar la nota
player-marked-watched = Marcado como visto
player-tracked = Registrado
player-not-tracked = Sin registrar
player-track-update-failed = No se pudo actualizar tu lista
player-not-tracked-why = Sin registrar: { $reason }.
player-track-ok-at = { $tracker } en { $progress }
player-track-ok = { $tracker } ok
player-track-note = { $tracker }: { $note }
player-track-failed = No se pudo actualizar { $tracker }
player-track-line = { $head }: { $parts }
phone-player-refusal-hidden = la serie está oculta
phone-player-refusal-no-match = la serie no tiene coincidencia en el tracker
phone-player-refusal-not-newer = el tracker ya está más allá de este episodio
phone-player-refusal-extra = es un extra
phone-player-refusal-unmatched = el archivo no está asociado a un episodio
phone-player-refusal-on-disk = el archivo solo está en disco

## Subtitles

subs-line-default = Tamaño 1×, los archivos sin estilo usan el tuyo
subs-sample = ¿Dónde lo escondiste?
subs-size-place-title = Tamaño y posición
subs-size = Tamaño
subs-position = Posición
subs-position-helper = 100 es el borde inferior. Los números menores suben las líneas por la pantalla.
subs-style-title = Estilo
subs-styled = Subtítulos con estilo
subs-styled-as-made = Los archivos con estilo (ASS) se ven tal como los dejó su autor.
subs-styled-mine = Tu estilo de abajo se aplica a todos los archivos. Los rótulos y el karaoke pueden fallar.
subs-styled-size-only = Los archivos con estilo conservan su aspecto y toman tu tamaño. Los demás toman tu estilo.
subs-mode-as-made = Original
subs-mode-size-only = Solo tamaño
subs-mode-mine = Mío
subs-font = Fuente
subs-font-bad = Escribe el nombre de una fuente, como Noto Sans.
subs-bold = Negrita
subs-colour = Color del texto
subs-colour-bad = Escribe un color como #ffcc00.
subs-outline = Contorno
subs-outline-bad-width = El grosor es un número, 0 o más.
subs-outline-bad-colour = Escribe el color como #000000.
subs-outline-helper = Su grosor y luego su color.
subs-shadow = Sombra
subs-shadow-bad = Un número, 0 o más. 0 es sin sombra.
subs-shadow-helper = A qué distancia cae la sombra, 0 para ninguna.
subs-box = Cuadro de fondo
subs-box-helper = Un cuadro oscuro detrás de las líneas. 0 lo quita.
subs-reset-title = Desde cero
subs-reset-helper = Devuelve el tamaño, la fuente, los colores, el contorno, el cuadro y la posición a como venían.
subs-reset = Reiniciar

## Streaming

stream-quality-auto = Auto
stream-quality-original = Original
stream-app-starting = La app aún está arrancando
stream-no-computer = Aún no hay ordenador. Añade el tuyo en Más.
stream-off-on-host = El streaming está desactivado en { $host }. Activa Stream episodes allí, en Settings, Phone app.
stream-cannot-stream = { $host } usa AniBeam { $version }, que aún no puede hacer streaming. Actualízalo allí.
stream-no-session = El otro dispositivo no dio ninguna sesión
stream-answer-unreadable = No se pudo leer la respuesta: { $why }
stream-answered-status = { $name } respondió { $status }

## Stats

phone-stats-nothing-yet = Aún nada
phone-stats-empty-title = Aún no has visto nada
phone-stats-empty-body = Termina un episodio y contará aquí: horas, géneros y las temporadas que viste.
stats-hours-caption = horas vistas
stats-finished-caption = series terminadas
stats-busiest-caption = mes con más anime
stats-episodes-caption = { $count ->
    [one] episodio
   *[other] episodios
}
phone-stats-by-month = Por mes
phone-stats-top-genres = Top géneros
phone-stats-top-studios = Top estudios
stats-seasons = Temporadas vistas
stats-watched-most = Lo más visto
stats-year-line = Lo que viste en { $year }
stats-eps = { $count ->
    [one] { $count } ep.
   *[other] { $count } ep.
}
stats-month-1 = Ene
stats-month-2 = Feb
stats-month-3 = Mar
stats-month-4 = Abr
stats-month-5 = May
stats-month-6 = Jun
stats-month-7 = Jul
stats-month-8 = Ago
stats-month-9 = Sep
stats-month-10 = Oct
stats-month-11 = Nov
stats-month-12 = Dic
stats-month-letter-1 = E
stats-month-letter-2 = F
stats-month-letter-3 = M
stats-month-letter-4 = A
stats-month-letter-5 = M
stats-month-letter-6 = J
stats-month-letter-7 = J
stats-month-letter-8 = A
stats-month-letter-9 = S
stats-month-letter-10 = O
stats-month-letter-11 = N
stats-month-letter-12 = D

## Backup and image cache

backup-title = Copia de seguridad
backup-export = Copiar la biblioteca
backup-export-helper = Un archivo con tus carpetas, coincidencias y lo que has visto.
backup-private = Incluir datos privados
backup-private-helper = Añade las sesiones de AniList y MyAnimeList, el historial y las preferencias. Las sesiones se escriben en texto plano, así que guarda el archivo para ti.
backup-restore = Restaurar una copia
backup-restore-helper = Lee una copia en esta biblioteca. Lo que hay en disco manda; el archivo completa el resto.
backup-done = Copia hecha. El archivo está donde lo guardaste.
backup-not-saved = No se guardó. No se escribió nada.
backup-starting = AniBeam aún está arrancando. Inténtalo de nuevo en un momento.
backup-images-title = Caché de imágenes
backup-images-body = Los carteles, banners y retratos se guardan en el móvil para que la biblioteca funcione sin conexión. Las imágenes borradas se descargan de nuevo cuando las páginas las piden.
backup-keep = Mantener
backup-removed = { $count ->
    [one] { $count } imagen eliminada
   *[other] { $count } imágenes eliminadas
}
backup-images-none = Sin imágenes en caché
backup-images-unmeasured = No se pudo medir la caché de imágenes
backup-images-retry-note = Inténtalo de nuevo en un momento. Las imágenes en sí están bien.
backup-images-line = { $size } en { $count ->
    [one] { $count } imagen
   *[other] { $count } imágenes
}
backup-images-question = { $count ->
    [one] ¿Eliminar { $count } imagen?
   *[other] ¿Eliminar { $count } imágenes?
}

backup-sum-list = { $items }.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] { $count } carpeta añadida
   *[other] { $count } carpetas añadidas
}
backup-sum-series = { $count } series creadas
backup-sum-matches = { $count ->
    [one] { $count } coincidencia aplicada
   *[other] { $count } coincidencias aplicadas
}
backup-sum-views = { $views ->
    [one] { $views } visualización
   *[other] { $views } visualizaciones
} y { $episodes ->
    [one] { $episodes } episodio terminado
   *[other] { $episodes } episodios terminados
} combinados
backup-sum-resume = { $count ->
    [one] { $count } punto de reanudación combinado
   *[other] { $count } puntos de reanudación combinados
}
backup-sum-accounts = { $count ->
    [one] { $count } cuenta de lista importada
   *[other] { $count } cuentas de lista importadas
}
backup-sum-fixes = { $count ->
    [one] { $count } archivo lleva una corrección tuya.
   *[other] { $count } archivos llevan una corrección tuya.
}
backup-sum-missing = { $count ->
    [one] { $count } carpeta no se encontró en este móvil.
   *[other] { $count } carpetas no se encontraron en este móvil.
}
backup-sum-left-out = Omitido: { $fields }.

## Activity

activity-everything = Todo
activity-problems = Problemas
activity-copied = Copiado
activity-clear-every-line = Borrar todo
activity-fold = Contraer
activity-no-problems = Sin problemas. Todo funcionó.
activity-nothing-yet = Aún nada que contar. Lo que AniBeam haga por ti aparecerá aquí.
activity-time-today = %H:%M:%S
activity-time-other = %-d %b %H:%M

## Dates and numbers (companion format.rs)

fmt-clock = %H:%M
fmt-day-month = %-d %b
fmt-weekday-date-time = %a %-d %b %H:%M
fmt-weekday-date-year-time = %a %-d %b %Y %H:%M
fmt-stamp = %-d %b %H:%M
fmt-decimal-point = ,

fmt-today-at = Hoy { $time }
fmt-tomorrow-at = Mañana { $time }
fmt-yesterday-at = Ayer { $time }

fmt-span-days = { $d } d { $h } h
fmt-span-hours = { $h } h { $m } min
fmt-span-minutes = { $m } min { $s } s
fmt-span-seconds = { $s } s
phone-fmt-in = en { $span }
fmt-full-days = { $d } d { $h } h { $m } min { $s } s
fmt-full-hours = { $h } h { $m } min { $s } s
fmt-full-minutes = { $m } min { $s } s

fmt-ago-now = ahora mismo
fmt-ago-minutes = hace { $count } min
fmt-ago-hours = hace { $count } h
fmt-ago-days = hace { $count } d

fmt-bytes-b = { $value } B
fmt-bytes-kb = { $value } KB
fmt-bytes-mb = { $value } MB
fmt-bytes-gb = { $value } GB
fmt-bytes-tb = { $value } TB

## Talking to the computer (companion client.rs, sync.rs, address.rs)

client-not-found = El móvil no encuentra ningún nombre { $host }. ¿Está Tailscale conectado?
client-timeout = { $host } no responde. ¿Está encendido y con Tailscale conectado en ambos?
client-connect-failed = No se pudo llegar a { $host }. ¿Está encendido y con Tailscale conectado en ambos?
client-refused = { $host } está encendido, pero AniBeam está cerrado o no comparte. Abre AniBeam allí y activa Share over Tailscale en Settings, Phone app.
client-no-route = No hay ruta a { $host }. ¿Está Tailscale conectado?
client-invalid-input = No se pudo conectar con { $host }
client-unreachable = No se pudo llegar a { $host }: { $why }
client-cannot-find = El móvil no encuentra { $host }. ¿Está Tailscale conectado?
client-broke-off = La respuesta de { $host } se cortó: { $why }
client-answered = { $host } respondió { $status } para { $path }
client-not-anibeam-status = Algo respondió, pero no es AniBeam ({ $host } respondió { $status } para { $path })
client-not-anibeam-hello = Algo respondió, pero no es AniBeam (su respuesta no es el saludo de AniBeam)
client-not-anibeam-name = Algo respondió, pero no es AniBeam (se llama { $name })
client-too-new = El host usa la API { $host_api }; esta app lee la { $app_api }. Actualiza la app.
client-snapshot-unreadable = No se pudo leer la instantánea: { $why }
client-probe-unreadable = No se pudieron leer los detalles del archivo: { $why }
client-peers-unreadable = No se pudo leer la lista de AniBeams: { $why }
sync-save-failed = No se pudo guardar la instantánea: { $why }
address-https = El AniBeam de tu ordenador responde en http simple por Tailscale; quita el https://.
address-empty = Escribe el nombre o la dirección del host, por ejemplo my-computer:8790.
address-ipv6-close = Una dirección IPv6 necesita su ] de cierre.
address-ipv6-port = Pon el puerto después del ], como [fd7a::1]:8790.
address-bad-host = "{ $text }" no es un nombre de host.
address-bad-port = "{ $port }" no es un puerto.

## Series facts (companion library.rs)

clib-status-airing = En emisión
clib-status-not-yet = Aún sin emitir
clib-format-tv-short = TV corto
clib-format-movie = Película
clib-format-special = Especial
clib-format-music = Música

## Search sites and repos

repos-by-hand-title = Añadir uno a mano
repos-by-hand-note = Para un sitio que ningún repo incluye. Sirve cualquier feed con { "{query}" } donde va la búsqueda, también uno de Jackett o Prowlarr. Una clave o token en la dirección pasa al almacén secreto de este móvil.
repos-name-placeholder = Nombre
repos-url-placeholder = https://...{ "{query}" }...
repos-index-placeholder = https://.../index.json
repos-add-site = Añadir sitio
repos-add-title = Añadir un repo
repos-add-note = Pega la dirección del índice de un repo. AniBeam comprueba su firma y te muestra su clave antes de guardar nada.
repos-add-repo = Añadir repo
repos-trust-title = ¿Confiar en el repo?
repos-fingerprint = Huella de la clave
repos-trust-warning = Sus sitios deciden adónde van tus búsquedas. Confía solo en repos de gente de confianza.
repos-trust = Confiar
repos-open-link = Abrir { $url }
repos-saving = Guardando...
repos-install = Instalar
repos-update = Actualizar
repos-edit-note = Una clave o token escrito en la dirección pasa al almacén secreto de este móvil.
repos-edit-note-keys = Deja cada { "{key}" } en la dirección tal cual; su valor se guarda en este móvil.
repos-change-its = Cambiar su { $keys }
repos-check-repos = Buscar sitios nuevos en repos
repos-check-sites = Comprobar los sitios
repos-add-site-label = Añadir un sitio a mano
repos-needs-before-search = Necesita su { $labels } antes de buscar
repos-no-longer-offered = Su repo ya no lo ofrece
repos-update-host = La actualización envía peticiones a { $host }
repos-tap-change-custom = Toca para cambiar su nombre o dirección
repos-tap-change-keys = Toca para cambiar su { $labels }
repos-line-custom = { $host }  { $kind }  añadido por ti
repos-update-to = Actualizar a v{ $version }
repos-add-its-value = Añadir su valor
repos-asks-for = Pide tu { $labels }
repos-preview-line = { $count ->
    [one] { $count } sitio, firmado
   *[other] { $count } sitios, firmados
}
repos-preview-trusted = { $count ->
    [one] { $count } sitio, firmado, ya de confianza
   *[other] { $count } sitios, firmados, ya de confianza
}
repos-ask-kept-empty = Se guarda solo en este móvil. Un campo vacío conserva lo guardado.
repos-ask-kept-never = Se guarda solo en este móvil, nunca en la biblioteca ni en una exportación.
repos-ask-new-host = La nueva versión envía esto a { $host }, así que lo pide otra vez.
repos-ask-new = La nueva versión vuelve a pedir esto.
repos-type-value = Escribe un valor primero.
repos-health-answered = Respondió en { $ms } ms
repos-health-failed = Sin respuesta. { $step }: { $detail }

## Downloads

dl-title = Descargas
dl-wifi-only = Solo Wi-Fi
dl-wifi-and-mobile = Wi-Fi y datos móviles
dl-joined = { $first }, { $second }
dl-batch-files = { $count ->
    [one] Lote, { $count } archivo
   *[other] Lote, { $count } archivos
}
dl-batch-tag = Lote
dl-downloading-count = { $count ->
    [one] { $count } descargando
   *[other] { $count } descargando
}
dl-free-on-phone = libres en el móvil
dl-search-placeholder = Una serie o un nombre de lanzamiento
dl-sources = Fuentes
dl-downloading = Descargando
dl-queue-note = { $count ->
    [one] { $count } episodio
   *[other] { $count } descargas
}
dl-finished = Terminadas
dl-finished-note = { $count ->
    [one] { $count } descarga, en tu biblioteca
   *[other] { $count } descargas, en tu biblioteca
}
dl-get = Obtener
dl-pick = Elegir

## Storage and hidden shows

disk-on-phone = En este móvil
disk-watched = Vistos
disk-not-watched = Sin ver
disk-deleting = Eliminando
disk-delete-watched = Eliminar vistos
disk-hidden-shows = Series ocultas
disk-hidden-helper = Fuera de Inicio y de la biblioteca hasta que las recuperes.
disk-unhide = Mostrar
disk-delete-note = Los archivos y subtítulos se eliminan de este móvil para siempre.
disk-every-watched = Todos los episodios vistos
disk-every-finished = Todas las series terminadas que viste
disk-episodes-size = { $count ->
    [one] { $count } episodio, { $size }
   *[other] { $count } episodios, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } serie, { $size }
   *[other] { $count } series, { $size }
}
disk-list-two = { $a } y { $b }
disk-list-three = { $a }, { $b } y { $c }
disk-list-more = { $a }, { $b }, { $c } y { $count } más
disk-delete-finished-title = ¿Eliminar { $count ->
    [one] { $count } serie terminada
   *[other] { $count } series terminadas
}, { $size }?
disk-delete-watched-title = ¿Eliminar { $count ->
    [one] { $count } episodio visto
   *[other] { $count } episodios vistos
}, { $size }?
disk-delete-finished-body = { $names }. Solo series que el proveedor da por terminadas y con todos los episodios vistos; una serie aún en emisión, o cuyo final es una suposición, se queda. Se eliminan de este móvil todos sus archivos, extras incluidos. Sus descargas también desaparecen, así que se pueden descargar de nuevo.
disk-delete-watched-body = Todos los episodios que has visto, en todas las series. Los episodios sin ver y los extras se quedan. Los archivos y sus subtítulos se eliminan de este móvil para siempre.
disk-no-finished = Ninguna serie terminada está vista del todo
disk-nothing-watched = No hay nada visto en este móvil
disk-deleted = { $count ->
    [one] { $count } episodio eliminado
   *[other] { $count } episodios eliminados
}, { $size }
disk-delete-failed = No se eliminó nada: { $error }

## Download notifications

notify-card-head = AniBeam · ahora
notify-sample-called = Frieren episodio 7
notify-sample-called-ready = Frieren episodio 6
notify-sample-facts = 420 MB de 1,4 GB, 3,2 MB/s, faltan 5 min
notify-downloading = Descargando { $called }
notify-progress = { $percent }%, { $facts }
notify-progress-of = { $percent }%, { $done } de { $size }
notify-starting = Iniciando
notify-many = { $count ->
    [one] { $count } descarga
   *[other] { $count } descargas
}
notify-called-episode = { $show } episodio { $n }
notify-called-episodes = { $show } episodios { $from } a { $to }
notify-ready = { $called } está listo
notify-ready-many = { $called } están listos
notify-ready-batch = Un lote de { $show } está listo
notify-ready-text = Descargado en este móvil. Toca para abrir AniBeam.
notify-channel-downloads = Descargas
notify-channel-ready = Episodios listos
notify-fallback-title = Descargando
notify-row-on = Notificaciones
notify-row-allow = Permitir notificaciones
notify-row-open-settings = Abrir ajustes de notificaciones
notify-helper-on = Activadas. Las descargas aparecen en tus notificaciones mientras se hacen.
notify-helper-asks-next = Android pregunta ahora.
notify-helper-asks-next-later = Android pregunta ahora. Puedes cambiar de idea después en Más.
notify-helper-off = Desactivadas. Activa allí las notificaciones de AniBeam.
notify-question-title = ¿Ver las descargas en tus notificaciones?
notify-question-body = Mientras se descargan episodios, una notificación muestra el avance y otra avisa cuando uno está listo. Se descargan igualmente; esto solo decide si lo ves.
notify-allow = Permitir
notify-not-now = Ahora no
notify-pointer = Las descargas no aparecen en tus notificaciones.
notify-turn-on = Activar

## Feed

phone-feed-title = Feed
feed-recently-released = Estrenos recientes
feed-coming-soon = Próximamente
feed-this-week = Esta semana
feed-earlier = Antes
feed-everything-else = Todo lo demás
phone-feed-countdown-days = { $d } d { $h } h
phone-feed-countdown-hours = { $h } h { $m } min
phone-feed-countdown-minutes = { $m } min
phone-feed-next = Siguiente: { $title } en { $countdown }
phone-feed-nothing-scheduled = Aún no hay nada programado.
phone-feed-nothing-new = Nada nuevo esta semana.
phone-feed-new-episodes = { $count ->
    [one] { $count } episodio nuevo esta semana.
   *[other] { $count } episodios nuevos esta semana.
}
phone-feed-aired = emitido { $ago }
phone-feed-downloaded = descargado { $ago }

## Files and the file picker

files-already-picking = Ya se está eligiendo un archivo.
files-took-too-long = El selector de archivos tardó demasiado.
files-did-not-open = El selector de archivos no se abrió: { $error }
files-none-here = Aquí no hay selector de archivos.

## The computer's downloads

desk-sent-adding = Se enviaron { $what } a { $name }. Aún los está añadiendo.
desk-sent = Obteniendo { $what } en { $name }.

## Your computer's library

remote-watched = Visto
remote-movie = Película
remote-not-watched-yet = Sin ver
remote-watched-of = { $done } de { $total } vistos
remote-watched-count = { $count } vistos
remote-on-disk = { $count } en disco
remote-airs-in-season = T{ $season } episodio { $n } sale en { $countdown }
remote-airs-in = El episodio { $n } sale en { $countdown }
remote-play-episode = Ver episodio { $n }
remote-watch-again = Ver de nuevo el episodio { $n }
remote-update-host = Actualiza AniBeam en ese ordenador para reproducir esto aquí
remote-aired = emitido
remote-missing = Falta
remote-on-computer = En tu ordenador
remote-out = Salió { $ago }
remote-season-episode = Temporada { $season }, episodio { $n }

## Episodes and dates: a season's episode count and air dates, set by hand

more-words-ratings = imdb créditos puntuaciones valoraciones notas
show-format-series = Serie
dates-title = Episodios y fechas
dates-season-line = { $title } · Temporada { $season }
dates-paste-open = Pegar una lista
dates-paste-done = Hecho
dates-paste-hint = Pega una lista de estrenos, una tabla o una publicación
dates-paste-note = Lo que AniBeam lee aparece en los episodios de abajo.
dates-tap-hint = Toca un episodio para ponerle fecha.
dates-run = Elige un tramo
dates-field-hint = 7 ene, marzo 2027, 2027
dates-set = Fijar fecha
dates-no-date = Sin fecha aún
dates-weekly = Semanal
dates-last = Último
dates-not-last = No es el último
dates-clear = Quitar fecha
dates-save = Guardar
dates-saving = Guardando
dates-cancel = Cancelar
dates-change = Cambiar
dates-picked-one = Episodio { $episode }:
dates-picked-many =
    { $count ->
        [one] { $count } episodio:
       *[other] { $count } episodios:
    }
dates-saved = Guardado.
dates-cleared = Notas del calendario borradas.
dates-starting = AniBeam aún está arrancando. Inténtalo de nuevo en un momento.
dates-tile-out = Salió
dates-tile-last = Último
dates-tile-none = Sin fecha
dates-tile-guess = ≈ { $date }
dates-summary-none = Nada fijado aún
dates-summary-ends = Termina en el episodio { $episode }
dates-summary-record =
    { $count ->
        [one] { $count } con fecha de { $site }
       *[other] { $count } con fecha de { $site }
    }
dates-summary-record-unnamed =
    { $count ->
        [one] { $count } con fecha del registro
       *[other] { $count } con fecha del registro
    }
dates-summary-set =
    { $count ->
        [one] { $count } fijado por ti
       *[other] { $count } fijados por ti
    }
dates-summary-guessed =
    { $count ->
        [one] { $count } estimado semanal
       *[other] { $count } estimados semanales
    }
dates-told-one = Episodio { $episode }
dates-told-many = Episodios { $first } a { $last }
dates-told-undated = { $episodes } sin fecha aún
dates-told-due = { $episodes } en { $when }
dates-told-ends = Termina en el episodio { $episode }
dates-told-season-ends = La temporada { $season } termina en el episodio { $episode }
dates-told-by-you = { $told }, fijado por ti
dates-told-gave-way =
    { $count ->
        [one] Fijaste { $count } episodio; ya no se aplica
       *[other] Fijaste { $count } episodios; ya no se aplica
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
series-runtime-each = { $runtime } cada uno
series-status-airing = En emisión
series-status-finished = Finalizada
series-status-upcoming = Próximamente
series-status-cancelled = Cancelada
series-status-hiatus = En pausa
series-average-score = Nota media { $score }
series-imdb = IMDb { $score }
series-imdb-votes = IMDb { $score } de { $votes } votos
series-rewatched =
    { $count ->
        [one] Vuelta a ver { $count } vez
       *[other] Vuelta a ver { $count } veces
    }
series-not-tracked = Sin seguimiento
series-not-tracked-on-phone =
    { $count ->
        [one] Sin seguimiento, { $count } episodio en este móvil
       *[other] Sin seguimiento, { $count } episodios en este móvil
    }

## More, About

about-ratings = Notas
about-ratings-credit = Información cortesía de IMDb (https://www.imdb.com). Usada con permiso.

## Marking episodes on the series page

mark-menu-watched = Marcar como visto hasta aquí
mark-menu-unwatched = Marcar como no visto desde aquí
mark-menu-cancel = Cancelar
mark-watched-up-to = Visto hasta el episodio { $episode }
mark-unwatched-from = Sin ver desde el episodio { $episode }
mark-not-connected = { $reason } Inicia sesión en Más, Cuentas.

## The player

player-speed-not-while-casting = No al enviar

## Downloads

downloads-other-release-on-disk = Otra versión en disco

## Month patterns

dates-month-short = %b %Y
dates-month-long = %B de %Y
