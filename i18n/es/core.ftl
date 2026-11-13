# es translation of en/core.ftl. An ID missing here is shown in English.

## El host del tailnet

tailnet-person-no-name = Sin nombre en Tailscale

## Recuentos

count-episodes =
    { $count ->
        [one] { $count } episodio
       *[other] { $count } episodios
    }
count-shows =
    { $count ->
        [one] { $count } serie
       *[other] { $count } series
    }

## Acciones que usa cada pantalla

action-cancel = Cancelar
action-close = Cerrar
action-try-again = Reintentar
action-remove = Quitar
action-delete = Eliminar
action-open = Abrir
action-save = Guardar
action-apply = Aplicar
action-clear = Borrar
action-refresh = Actualizar
action-play = Reproducir
action-connect = Conectar
action-start = Empezar
action-back = Atrás
action-see-all = Ver todo
action-open-series = Abrir serie
action-open-settings = Abrir ajustes

## Las listas que guarda un tracker

status-watching = Viendo
status-completed = Completado
status-paused = En pausa
status-on-hold = En espera
status-dropped = Abandonado
status-planning = Planeado
status-rewatching = Repitiendo

## Episodios

episode-number = Episodio { $number }

## Las palabras de AniList para una serie: formato, estado de emisión, relación entre series

anilist-format-tv = TV
anilist-format-tv-short = TV corto
anilist-format-film = Película
anilist-format-special = Especial
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Música
anilist-status-finished = Finalizado
anilist-status-airing = En emisión
anilist-status-not-yet-aired = Sin estrenar
anilist-status-cancelled = Cancelado
anilist-status-hiatus = En pausa
anilist-relation-adaptation = Adaptación
anilist-relation-prequel = Precuela
anilist-relation-sequel = Secuela
anilist-relation-parent = Historia principal
anilist-relation-side-story = Historia paralela
anilist-relation-character = Personaje
anilist-relation-summary = Resumen
anilist-relation-alternative = Alternativa
anilist-relation-spin-off = Spin-off
anilist-relation-other = Otra
anilist-relation-source = Original
anilist-relation-compilation = Recopilación
anilist-relation-contains = Contiene

## El registro de actividad: un problema cuyo origen no tenía palabras propias

event-problem-library = Algo falló en tu biblioteca. Las opciones de desarrollador tienen los detalles.
event-problem-metadata = Algo falló con la información de una serie. Las opciones de desarrollador tienen los detalles.
event-problem-trackers = Algo falló con tus listas. Las opciones de desarrollador tienen los detalles.
event-problem-franchise = Algo falló con las series relacionadas. Las opciones de desarrollador tienen los detalles.
event-problem-playback = Algo falló en la reproducción. Las opciones de desarrollador tienen los detalles.
event-problem-store = Algo falló al guardar tus datos. Las opciones de desarrollador tienen los detalles.
event-problem-system = Algo falló en AniBeam. Las opciones de desarrollador tienen los detalles.
event-problem-downloads = Algo falló en las descargas. Las opciones de desarrollador tienen los detalles.

## Errores (contract::CoreError)

core-error-not-found-source = No se encontró la carpeta { $id }.
core-error-not-found-series = No se encontró la serie { $id }.
core-error-not-found-file = No se encontró el archivo { $id }.
core-error-not-found-session = No se encontró la sesión de reproducción { $id }.
core-error-not-found-job = No se encontró la tarea { $id }.
core-error-invalid = { $field } no válido: { $message }.
core-error-invalid-ended = { $field } no válido: { $message }
core-error-unavailable = Esta carpeta no está disponible: { $path }
core-error-not-connected = { $tracker } no está conectado.
core-error-refused-hidden = No se puede: la serie está oculta.
core-error-refused-no-match = No se puede: la serie aún no está reconocida; reconócela en Info de la serie.
core-error-refused-not-newer = No se puede: tu lista ya va por delante de este episodio.
core-error-refused-extra = No se puede: esto es un extra.
core-error-refused-unmatched = No se puede: el archivo no corresponde a ningún episodio.
core-error-refused-on-disk = No se puede: el archivo solo está en el disco.
core-error-provider = { $provider } no respondió: { $message }
core-error-io = No se pudo leer o escribir un archivo: { $message }
core-error-storage = No se pudo guardar tu biblioteca: { $message }
core-error-keyring = No se pudo acceder al almacén de contraseñas del sistema: { $message }
core-error-unsupported = { $what } aún no es compatible.
core-error-version = Esta exportación es de un AniBeam más nuevo (formato { $found }). Actualiza AniBeam para importarla.
core-error-internal = AniBeam tuvo un error interno: { $message }
core-error-service = { $service }: { $message }

## core/src/repos/mod.rs

core-repos-paste-address = pega la dirección de un repo
core-repos-must-be-https = la dirección debe ser https
core-repos-unreachable = no se pudo llegar al repo: { $message }
core-repos-not-found = no hay nada en esa dirección (un repo privado de GitHub también responde 404)
core-repos-http-status = el repo respondió HTTP { $status }
core-repos-unreadable = no se pudo leer { $path }: { $error }
core-repos-no-such-endpoint = no hay ningún endpoint instalado con ese id
core-repos-key-changed = el repo está firmado ahora con una clave distinta de la que aceptaste. Quítalo y vuelve a añadirlo para aceptar la nueva clave.
core-repos-stale = el repo sirvió un índice más antiguo (serie { $serial } tras { $kept }); se mantiene el más nuevo
core-repos-key-not-shown = la clave del repo no es la que se te mostró. Vuelve a añadirlo y compara la huella.
core-repos-repo-added = Se añadió una lista de fuentes de descarga.
core-repos-repo-removed = Se quitó una lista de fuentes de descarga.
core-repos-no-field = { $name } no tiene el campo { $key }
core-repos-needs-field = { $name } necesita su { $label }
core-repos-host-only = { $name }: { $label } es solo un host o host:puerto, como 192.0.2.10:9696
core-repos-not-added = ese repo no está añadido
core-repos-not-offered = { $repo } no ofrece { $id }
core-repos-already-installed = { $name } ya está instalado
core-repos-taken-by-you = ya hay instalado un endpoint llamado { $id } (añadido por ti); quítalo primero
core-repos-taken-by-repo = ya hay instalado un endpoint llamado { $id } (de otro repo); quítalo primero
core-repos-source-added = Se añadió una fuente de descarga.
core-repos-source-updated = Se actualizó una fuente de descarga.
core-repos-source-removed = Se quitó una fuente de descarga.
core-repos-source-changed = Se cambió una fuente de descarga.
core-repos-give-name = ponle un nombre
core-repos-put-query = pon {"{"}query{"}"} en la dirección, donde va la búsqueda
core-repos-name-taken = ya hay instalado un endpoint llamado { $id }
core-repos-only-by-hand = solo se puede editar una fuente añadida a mano
core-repos-keep-key = deja {"{"}{ $key }{"}"} en la dirección y cambia su valor con Cambiar { $label }, o quita el sitio y vuelve a añadirlo sin él
core-repos-note-kind = anirss no puede buscar en el tipo { $kind }, así que el escritorio lo omite; el móvil sí lo busca
core-repos-note-by-hand = anirss tiene su propio { $id }, escrito a mano; el escritorio busca en ese
core-repos-note-theirs = Este anirss lo ejecutas tú y el escritorio nunca cambia su configuración: añade { $id } a la configuración de ese anirss para que el escritorio busque en él
core-repos-note-sandbox = Esta ejecución nunca escribe en la configuración de anirss, así que aquí el escritorio no busca en { $id }
core-repos-note-old = anirss { $version } no acepta endpoints de { -app-name }; actualízalo a { $min } o posterior para que el escritorio busque en este

## core/src/repos/index.rs

core-repos-key-not-ed25519 = la clave del índice no es una clave ed25519
core-repos-key-not-base64 = la clave del índice no son 32 bytes en base64
core-repos-key-invalid = la clave del índice no es una clave ed25519 válida
core-repos-too-large = el archivo es demasiado grande para ser un repo
core-repos-not-a-repo = esto no es un repo de endpoints de { -app-name }
core-repos-format-version = el repo tiene formato versión { $found }; este { -app-name } lee la versión { $reads }. Actualiza { -app-name }.
core-repos-payload-not-base64 = el contenido del repo no está en base64
core-repos-signature-malformed = la firma del repo está mal formada
core-repos-index-unreadable = el índice del repo no se puede leer: { $error }
core-repos-signature-mismatch = la firma del repo no coincide con su contenido
core-repos-check-scheme = { $id }: la dirección debe empezar por https://
core-repos-check-scheme-http = { $id }: la dirección debe empezar por http:// o https://
core-repos-check-auth-key = { $id }: clave de autenticación incorrecta { $key }
core-repos-check-host-torznab = { $id }: solo una dirección torznab puede indicar su host como {"{"}{ $key }{"}"}
core-repos-check-host-placeholder = { $id }: un marcador en el host

## core/src/playback/tracks.rs

core-playback-track-number = Pista { $number }
core-playback-track-file = { $label } (archivo)
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = Inglés
core-playback-lang-japanese = Japonés
core-playback-lang-spanish = Español
core-playback-lang-portuguese = Portugués
core-playback-lang-french = Francés
core-playback-lang-german = Alemán
core-playback-lang-italian = Italiano
core-playback-lang-russian = Ruso
core-playback-lang-chinese = Chino
core-playback-lang-korean = Coreano
core-playback-lang-arabic = Árabe
core-playback-lang-hindi = Hindi
core-playback-lang-indonesian = Indonesio
core-playback-lang-malay = Malayo
core-playback-lang-thai = Tailandés
core-playback-lang-vietnamese = Vietnamita
core-playback-lang-polish = Polaco
core-playback-lang-turkish = Turco
core-playback-lang-dutch = Neerlandés
core-playback-lang-swedish = Sueco
core-playback-lang-norwegian = Noruego
core-playback-lang-danish = Danés
core-playback-lang-finnish = Finés
core-playback-lang-hebrew = Hebreo
core-playback-lang-ukrainian = Ucraniano
core-playback-lang-czech = Checo
core-playback-lang-hungarian = Húngaro
core-playback-lang-greek = Griego
core-playback-lang-romanian = Rumano
core-playback-lang-filipino = Filipino
core-playback-region-brazil = Brasil
core-playback-region-portugal = Portugal
core-playback-region-latin-america = Latinoamérica
core-playback-region-spain = España
core-playback-region-simplified = Simplificado
core-playback-region-traditional = Tradicional

## core/src/contract/enums.rs

core-jobs-scan = Revisando tus carpetas
    .stopped = La revisión de tus carpetas se detuvo por un problema.
core-jobs-auto-match = Reconociendo series
    .stopped = El reconocimiento de series se detuvo por un problema.
core-jobs-search = Buscando una serie
    .stopped = La búsqueda de una serie se detuvo por un problema.
core-jobs-resolve-link = Leyendo un enlace
    .stopped = La lectura de un enlace se detuvo por un problema.
core-jobs-apply-match = Reconociendo una serie
    .stopped = El reconocimiento de una serie se detuvo por un problema.
core-jobs-refresh = Actualizando una serie
    .stopped = La actualización de una serie se detuvo por un problema.
core-jobs-refresh-all = Actualizando la información de las series
    .stopped = La actualización de la información de las series se detuvo por un problema.
core-jobs-refresh-airing = Comprobando qué se emite
    .stopped = La comprobación de qué se emite se detuvo por un problema.
core-jobs-refresh-imdb = Actualizando valoraciones
    .stopped = La actualización de valoraciones se detuvo por un problema.
core-jobs-clear-images = Borrando imágenes
    .stopped = El borrado de imágenes se detuvo por un problema.
core-jobs-fill-images = Descargando imágenes
    .stopped = La descarga de imágenes se detuvo por un problema.
core-jobs-connect-tracker = Conectando tu lista
    .stopped = La conexión de tu lista se detuvo por un problema.
core-jobs-mark = Marcando un episodio como visto
    .stopped = El marcado de un episodio como visto se detuvo por un problema.
core-jobs-set-progress = Guardando tu progreso
    .stopped = El guardado de tu progreso se detuvo por un problema.
core-jobs-score = Guardando tu nota
    .stopped = El guardado de tu nota se detuvo por un problema.
core-jobs-refresh-progress = Leyendo el progreso de tu lista
    .stopped = La lectura del progreso de tu lista se detuvo por un problema.
core-jobs-refresh-watching = Leyendo tu lista Viendo
    .stopped = La lectura de tu lista Viendo se detuvo por un problema.
core-jobs-refresh-airing-now = Comprobando esta temporada
    .stopped = La comprobación de esta temporada se detuvo por un problema.
core-jobs-refresh-recommended = Buscando series para ti
    .stopped = La búsqueda de series para ti se detuvo por un problema.
core-jobs-read-show-page = Leyendo una serie
    .stopped = La lectura de una serie se detuvo por un problema.
core-jobs-on-hold = Pasando las series inactivas a En espera
    .stopped = El paso de series inactivas a En espera se detuvo por un problema.
core-jobs-set-status = Cambiando un estado de lista
    .stopped = El cambio de estado de lista se detuvo por un problema.
core-jobs-move-lists = Pasando tu lista a tu tracker
    .stopped = El paso de tu lista a tu tracker se detuvo por un problema.
core-jobs-notes = Guardando tus notas
    .stopped = El guardado de tus notas se detuvo por un problema.
core-jobs-crawl = Buscando series relacionadas
    .stopped = La búsqueda de series relacionadas se detuvo por un problema.
core-jobs-skip-windows = Buscando el opening y el ending
    .stopped = La búsqueda del opening y el ending se detuvo por un problema.
core-jobs-export = Guardando tu biblioteca en un archivo
    .stopped = El guardado de tu biblioteca en un archivo se detuvo por un problema.
core-jobs-import = Cargando tu biblioteca desde un archivo
    .stopped = La carga de tu biblioteca desde un archivo se detuvo por un problema.
core-jobs-subscriptions = Revisando tus suscripciones
    .stopped = La revisión de tus suscripciones se detuvo por un problema.
core-jobs-connect-downloads = Conectando la app de descargas
    .stopped = La conexión de la app de descargas se detuvo por un problema.
core-jobs-search-releases = Buscando episodios
    .stopped = La búsqueda de episodios se detuvo por un problema.
core-jobs-refine-releases = Afinando los episodios
    .stopped = El filtrado de episodios se detuvo por un problema.
core-jobs-subscribe = Suscribiéndose a una serie
    .stopped = La suscripción a una serie se detuvo por un problema.
core-jobs-download = Añadiendo descargas
    .stopped = La adición de descargas se detuvo por un problema.
core-jobs-remove-subscription = Quitando una suscripción
    .stopped = La eliminación de una suscripción se detuvo por un problema.
core-jobs-control-torrents = Cambiando tus descargas
    .stopped = El cambio de tus descargas se detuvo por un problema.
core-jobs-test-torrent-client = Probando la app de descargas
    .stopped = La prueba de la app de descargas se detuvo por un problema.
core-jobs-remove-show = Quitando una serie
    .stopped = La eliminación de una serie se detuvo por un problema.
core-jobs-repair-subscription = Reparando una suscripción
    .stopped = La reparación de una suscripción se detuvo por un problema.
core-jobs-check-endpoints = Comprobando tus fuentes de descarga
    .stopped = La comprobación de tus fuentes de descarga se detuvo por un problema.
core-jobs-start-endpoint = Iniciando una fuente de descarga
    .stopped = El inicio de una fuente de descarga se detuvo por un problema.
core-jobs-schedule-agent = Pidiendo un horario a tu agente de código
    .stopped = La petición de un horario a tu agente de código se detuvo por un problema.

## core/src/airing_now.rs

core-airing-now-now = En emisión
core-airing-now-winter = Invierno { $year }
core-airing-now-spring = Primavera { $year }
core-airing-now-summer = Verano { $year }
core-airing-now-fall = Otoño { $year }
core-airing-now-planned = { $title } añadida a tu lista Planeado en { $site }.
core-airing-now-planned-both = { $title } añadida a tu lista Planeado en { $site } y { $other }.
core-airing-now-already-on =
    { $title } ya está en tu lista de { $site } como { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }.
core-airing-now-plan-failed = No se pudo añadir { $title } a tu lista Planeado. Inténtalo más tarde.

## core/src/aired.rs

core-aired-episode-out = Ya está disponible el episodio { $number } de { $title }.

## core/src/cast/mod.rs

core-cast-connecting = Enviando a { $device }.
core-cast-ended = Se terminó de enviar a { $device }.
core-cast-stopped = Se dejó de enviar a { $device }.
core-cast-failed = Falló el envío a { $device }. Comprueba que esté en la misma red.
core-cast-not-built = El envío a otros dispositivos no está en esta versión
core-cast-serve-failed = no se pudo servir el episodio: { $error }
core-cast-device-gone = Ese dispositivo ya no está en la red
core-cast-not-a-playlist = no es una dirección de lista de reproducción
core-cast-nothing = No se está enviando nada
core-cast-still-starting = El dispositivo aún está arrancando
core-cast-no-route = este dispositivo no tiene ruta hasta la tele
core-cast-no-control = la tele no dio ninguna dirección de control
core-cast-not-handed = el otro ordenador no entregó el episodio: { $error }
core-cast-empty-playlist = el otro ordenador envió una lista de reproducción vacía
core-cast-refused-both = { $error }; también rechazó el archivo tal cual ({ $first })
core-cast-chromecast-stopped = el Chromecast se detuvo
core-cast-hevc = { $error }. Este episodio es HEVC, que solo decodifican los Chromecast más nuevos (con Google TV, o Ultra)

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = no se pudo llegar al Chromecast: { $error }
core-cast-chromecast-broke = se cortó la conexión con el Chromecast: { $error }
core-cast-chromecast-no-player = el Chromecast no inició su reproductor
core-cast-chromecast-refused-player = el Chromecast no quiso iniciar su reproductor ({ $reason })
core-cast-no-reason = sin motivo
core-cast-chromecast-cannot-play = el Chromecast no pudo reproducir esto ({ $kind })
core-cast-chromecast-error = el Chromecast se detuvo con un error al reproducir esto

## core/src/cast/dlna.rs

core-cast-tv-no-answer = la tele no respondió: { $error }
core-cast-tv-refused = la tele rechazó { $action }
core-cast-tv-refused-code = la tele rechazó { $action } ({ $code })

## core/src/cast/media.rs

core-cast-no-video = Este archivo no tiene vídeo.
core-cast-hi10p = Este episodio es H.264 de 10 bits (Hi10P), que ninguna tele ni Chromecast decodifica. Enviarlo requiere convertirlo, y eso aún no existe.
core-cast-video-codec = El vídeo de este episodio ({ $codec }) no se puede enviar.
core-cast-audio-codec = El audio de este episodio ({ $codec }) no se puede enviar.

## core/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Se empezó a descargar un episodio de { $folder }.
       *[other] Se empezaron a descargar { $count } episodios de { $folder }.
    }
core-phone-subscribed = Suscrito a { $name }.
core-phone-rule-stopped = No se buscan episodios nuevos de { $name }: su fuente ya no existe o necesita su clave otra vez.
core-phone-new-episode = Se está descargando un episodio nuevo de { $name }.
core-phone-name-taken = { $name }: otra fuente ya tiene ese nombre
core-phone-nothing-to-download = nada que descargar
core-phone-over-cap = esto llevaría a { -app-name } por encima de su límite de espacio ({ $used } de { $cap } GB usados o en camino); elimina episodios o sube el límite
core-phone-empty-search = la búsqueda está vacía
core-phone-no-results = sin resultados para { $query }
core-phone-own-engine = un motor de torrents propio

## core/src/phone/check.rs

core-phone-check-connected = conectado
core-phone-check-refused = rechazado: { $error }
core-phone-check-dropped = sin respuesta (descartado)
core-phone-check-finding = { $label }: buscando su dirección
core-phone-check-failed = falló: { $error }
core-phone-check-no-answer = sin respuesta
core-phone-check-private = { $label }: buscándolo de forma privada
core-phone-check-skipped-address = omitido, es una dirección
core-phone-check-skipped-home = omitido, nombre de red doméstica
core-phone-check-no-record = sin dirección pública, se deja al DNS de esta red
core-phone-check-no-resolver = ningún resolvedor respondió
core-phone-check-reaching = { $label }: conectando con { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } bytes
core-phone-check-setup = configuración

## core/src/playback/session.rs

core-playback-not-saved = No se guardó tu progreso en { $folder }. Reconoce la serie primero.
core-playback-still-downloading = el episodio { $number } aún se está descargando, { $done } % hecho

## core/src/playback/skip.rs

core-playback-bad-duration = una duración es un número positivo de segundos

## core/src/agent.rs

core-agent-no-answer = el agente no dio respuesta
core-agent-searching = Buscando { $query } en la web
core-agent-searching-schedule = Buscando el horario en la web
core-agent-reading = Leyendo { $url }
core-agent-reading-page = Leyendo una página
core-agent-using = Usando { $tool }
core-agent-none = un agente de código en este ordenador
core-agent-ended = { $name } terminó sin respuesta
core-agent-too-long = { $name } tardó demasiado

## core/src/core.rs

core-core-guest-trackers = las cuentas de tracker de ese ordenador son de su dueño, así que este dispositivo no marca nada allí
core-core-guest-not-interested = No me interesa en ese ordenador es la lista de su dueño, así que este dispositivo no cambia nada allí
core-core-guest-recommended = las recomendaciones de ese ordenador salen de la lista de su dueño
core-core-nothing-picked = no hay nada elegido

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider } no es un proveedor de reconocimiento
core-prefs-provider-count = { $provider } aparece { $count } veces; cada proveedor aparece una vez
core-prefs-outside = { $value } está fuera del rango de { $low } a { $high }
core-prefs-empty = vacío

## core/src/recommended/mod.rs

core-recommended-ranking = Ordenando series para ti

## core/src/recommended/sync.rs

core-recommended-finding-people = Buscando gente con tus gustos
core-recommended-reading-lists = Leyendo sus listas
core-recommended-reading-yours = Leyendo tus series
core-recommended-reading-suggestions = Leyendo series que sugerir

## core/src/recommended/page.rs

core-recommended-no-anime = { -anilist } no tiene el anime { $id }

## core/src/host/mod.rs

core-host-off = Desactivado
core-host-starting = Iniciando
core-host-serving = Sirviendo en { $addresses }
core-host-no-tailscale = Tailscale no está conectado
core-host-cannot-listen = No se pudo escuchar
core-host-port-taken = El puerto { $port } está ocupado
core-host-cannot-listen-on = No se pudo escuchar en { $address }: { $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text } no es una dirección y un puerto

## core/src/host/keys.rs

core-host-not-a-key = { $key } no es una clave de dispositivo

## core/src/net/limiter.rs

core-net-timed-out = tiempo agotado tras { $ms } ms
core-net-rate-limited = { $provider } limitó las peticiones

## core/src/net/mod.rs

core-net-waiting = { $host }: se agotó el tiempo hace un momento; { -app-name } espera { $seconds } s antes de volver a preguntar

## core/src/net/anilist.rs

core-net-no-data = sin datos

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } no tiene descarga para este ordenador
core-update-cannot-make = No se pudo crear { $path }: { $error }
core-update-cannot-mark = No se pudo hacer ejecutable la nueva AppImage: { $error }
core-update-cannot-replace = No se pudo reemplazar { $path }: { $error }
core-update-not-here = Esta copia no se actualiza desde aquí
core-update-cannot-start = No se pudo iniciar la descarga: { $error }
core-update-did-not-start = La descarga no empezó: { $error }
core-update-cannot-write = No se pudo escribir { $path }: { $error }
core-update-stopped = La descarga se detuvo: { $error }
core-update-cancelled = Cancelada
core-update-ended-early = La descarga terminó antes de tiempo ({ $got } de { $size } bytes)
core-update-checksum = La descarga no coincide con la suma de comprobación de GitHub
core-update-cannot-move = No se pudo colocar la descarga en su sitio: { $error }
core-update-cannot-open-dmg = No se pudo abrir el dmg: { $error }
core-update-cannot-read-dmg = No se pudo leer el dmg: { $error }
core-update-no-app = El dmg no contiene ninguna app
core-update-cannot-copy = No se pudo copiar la nueva app junto a esta: { $error }

## core/src/calendar.rs

core-calendar-season-episode = { $title } - Temporada { $season } Episodio { $number }
core-calendar-episode = { $title } - Episodio { $number }

## core/src/paths.rs

core-paths-no-home = no hay carpeta personal

## core/src/first_run.rs

core-first-run-not-absolute = la carpeta de vídeos debe ser una ruta absoluta

## core/src/presence/mod.rs

core-presence-extra = Extra

## core/src/trackers/accounts.rs

core-trackers-no-client-id = No hay ID de cliente para { $tracker }.
core-trackers-mal-reconnect = Vuelve a conectar MyAnimeList en Ajustes.
core-trackers-mal-wants-secret = MyAnimeList pide el secreto de cliente de esta app. Añádelo en Trackers, en Ajustes, o registra la app con App Type "other".
core-trackers-token-reply-unreadable = respuesta de token ilegible: { $error }
core-trackers-token-reply-no-token = la respuesta de token no traía token de acceso

## core/src/trackers/mod.rs

core-trackers-no-token = no hay token de acceso guardado, vuelve a conectar en Ajustes

## core/src/trackers/cache.rs

core-trackers-no-user-id = la cuenta no trae id de usuario

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = no se eligió ninguna serie para pasar
core-trackers-moved-up =
    { $count ->
        [one] Se pasó { $count } serie de este ordenador a { $site }.
       *[other] Se pasaron { $count } series de este ordenador a { $site }.
    }
core-trackers-moved-up-some-refused =
    Se { $moved ->
        [one] pasó { $moved } serie
       *[other] pasaron { $moved } series
    } a { $site }; { $site } no aceptó { $failed ->
        [one] { $failed } serie
       *[other] { $failed } series
    }.
core-trackers-move-refused = { $site } no aceptó { $show }: { $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = Se agotó el tiempo de autorización, inténtalo de nuevo.
core-trackers-oauth-state-mismatch = el estado no coincide, posible CSRF, cancelado.
core-trackers-oauth-no-code = no hay código en la respuesta
core-trackers-oauth-no-token = no hay access_token en la respuesta
core-trackers-oauth-waiting = Esperando a que inicies sesión en { $site } en tu navegador.
core-trackers-oauth-connected = { $site } conectado como { $username }.
core-trackers-oauth-no-name = el perfil llegó sin nombre
core-trackers-oauth-could-not-start = No se pudo iniciar sesión
core-trackers-oauth-port-failed = no se pudo abrir el puerto { $port } de este ordenador: { $error }
core-trackers-oauth-port-busy = otro programa está usando el puerto { $port } de este ordenador. Espera un momento y reinténtalo, o cierra el programa que lo usa si sigue ocupado.
core-trackers-oauth-provider-error = { $site } devolvió un error: { $error }
core-trackers-page-connected = { $site } está conectado a { $app }
core-trackers-page-close-tab = Puedes cerrar esta pestaña.
core-trackers-page-failed-title = { $app }, falló el inicio de sesión
core-trackers-page-failed = El inicio de sesión no terminó
core-trackers-page-go-back = Vuelve a { $app } e inténtalo de nuevo.
core-trackers-page-finishing-title = { $app }, terminando
core-trackers-page-finishing = Terminando
core-trackers-page-finishing-body = { $site } respondió y AniBeam está terminando de iniciar sesión. Vuelve a { $app }, que te avisará cuando acabe.
core-trackers-page-cancelled-title = { $app }, inicio de sesión cancelado
core-trackers-page-cancelled = Se canceló este inicio de sesión
core-trackers-page-cancelled-body = Puedes cerrar esta pestaña. Empieza de nuevo desde { $app } si querías hacerlo.

## core/src/trackers/on_hold.rs

core-trackers-on-hold-moving = Pasando a En espera en { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] Se pasó { $count } serie a En espera en { $site }.
       *[other] Se pasaron { $count } series a En espera en { $site }.
    }
core-trackers-on-hold-moved-two =
    { $count ->
        [one] Se pasó { $count } serie a En espera en { $site } y { $second } en { $secondSite }.
       *[other] Se pasaron { $count } series a En espera en { $site } y { $second } en { $secondSite }.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } serie no se pudo pasar a En espera.
       *[other] { $failed } series no se pudieron pasar a En espera.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] Se pasó { $count } serie a En espera en { $site }.
       *[other] Se pasaron { $count } series a En espera en { $site }.
    } { $failed ->
        [one] { $failed } serie no se pudo pasar a En espera.
       *[other] { $failed } series no se pudieron pasar a En espera.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] Se pasó { $count } serie a En espera en { $site } y { $second } en { $secondSite }.
       *[other] Se pasaron { $count } series a En espera en { $site } y { $second } en { $secondSite }.
    } { $failed ->
        [one] { $failed } serie no se pudo pasar a En espera.
       *[other] { $failed } series no se pudieron pasar a En espera.
    }
core-trackers-series-fallback = serie { $id }
core-trackers-back-on-watching = { $name } vuelve a estar en Viendo en { $site }.
core-trackers-back-on-watching-two = { $name } vuelve a estar en Viendo en { $site } y { $secondSite }.
core-trackers-back-on-watching-failed = { $name } no pudo volver a Viendo. Inténtalo más tarde.

## core/src/trackers/writes.rs

core-trackers-rate-limited = { $tracker } limitó las peticiones, inténtalo dentro de un minuto.
core-trackers-auth-expired = La sesión de { $tracker } caducó, vuelve a conectar en Ajustes.
core-trackers-entry-not-found = No se encontró la entrada en { $tracker }.
core-trackers-server-error = Error del servidor de { $tracker } ({ $status }), inténtalo más tarde.
core-trackers-error-see-log = Error de { $tracker }, mira el registro de actividad.
core-trackers-episode-not-whole = { $episode } no es un episodio entero mayor que cero
core-trackers-score-out-of-range = { $score } está fuera del rango de 0 a 10
core-trackers-rewatch-not-settable = repetir no es un estado que se pueda poner a mano
core-trackers-notes-too-long = las notas tienen un máximo de { $max } caracteres
core-trackers-nothing-picked = no hay nada elegido
core-trackers-notes-not-on-list = Aún no está en tu lista de { $tracker }, añádela primero a una lista.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } y { -mal }
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
       *[repeating] Repitiendo
    }
core-trackers-marked = Episodio { $episode } de { $show } marcado en { core-trackers-sites }.
core-trackers-progress-set = Tu progreso en { $show } es ahora { $progress } en { core-trackers-sites }.
core-trackers-scored = { $show } puntuada con { $score } en { core-trackers-sites }.
core-trackers-score-cleared = Se borró tu nota de { $show } en { core-trackers-sites }.
core-trackers-status-set = { $show } pasa a { core-trackers-status } en { core-trackers-sites }.
core-trackers-notes-saved = Se guardaron tus notas de { $show } en { core-trackers-sites }.
core-trackers-notes-cleared = Se borraron tus notas de { $show } en { core-trackers-sites }.
core-trackers-update-failed = No se pudo actualizar { $site }. Inténtalo más tarde.
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
           *[other] { $done } series
        }
    }
core-trackers-written-done =
    { $action ->
        [clear] Se borró tu nota de { core-trackers-written-who } en { core-trackers-sites }.
        [status] { core-trackers-written-who } pasa a { core-trackers-status } en { core-trackers-sites }.
       *[score] Nota { $score } para { core-trackers-written-who } en { core-trackers-sites }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] { $failed } serie no se pudo actualizar: { $reason }.
           *[other] { $failed } series no se pudieron actualizar: { $reason }.
        }
       *[no] { $failed ->
            [one] { $failed } serie no se pudo actualizar.
           *[other] { $failed } series no se pudieron actualizar.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] { $failed } no se { $failed ->
            [one] pudo
           *[other] pudieron
        } actualizar: { $reason }.
       *[no] { $failed } no se { $failed ->
            [one] pudo
           *[other] pudieron
        } actualizar.
    }
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } serie no tiene correspondencia en ningún tracker.
       *[other] {" "}{ $noMatch } series no tienen correspondencia en ningún tracker.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } serie está oculta, así que no se sincroniza.
       *[other] {" "}{ $hidden } series están ocultas, así que no se sincronizan.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } serie solo está en un tracker que no está conectado.
       *[other] {" "}{ $notConnected } series solo están en un tracker que no está conectado.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } serie se guarda en este ordenador, sin tracker al que enviarla.
       *[other] {" "}{ $kept } series se guardan en este ordenador, sin tracker al que enviarlas.
    }

## core/src/metadata/automatch.rs

core-metadata-matched-to = { $show } reconocida como { $title }.
core-metadata-auto-matched =
    { $count ->
        [one] { $count } serie reconocida.
       *[other] { $count } series reconocidas.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } serie aún sin reconocer.
       *[other] { $count } series aún sin reconocer.
    }
core-metadata-auto-matched-some =
    { $matched ->
        [one] { $matched } serie reconocida
       *[other] { $matched } series reconocidas
    }; { $unmatched } aún sin reconocer.
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
        [yes] { $hitFolder } reconocida como { $hitTitle }.
       *[no] { $matched ->
            [one] { $matched } serie reconocida.
           *[other] { $matched } series reconocidas.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] No se encontró correspondencia para { $missName }.
       *[no] { $unmatched ->
            [one] No se encontró correspondencia para { $unmatched } serie.
           *[other] No se encontró correspondencia para { $unmatched } series.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] { $failed } serie no se pudo buscar; inténtalo más tarde.
       *[other] { $failed } series no se pudieron buscar; inténtalo más tarde.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } serie ya estaba reconocida.
       *[other] { $had } series ya estaban reconocidas.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } serie ya se está reconociendo.
       *[other] { $busy } series ya se están reconociendo.
    }

## core/src/metadata/apply.rs

core-metadata-link-unreadable = No se pudo leer ese enlace. Pega la URL de una página de AniList, MyAnimeList, TVmaze o IMDb.
core-metadata-no-mal-entry = AniList no tiene ninguna entrada para ese id de MyAnimeList.
core-metadata-no-imdb-show = TVmaze no tiene ninguna serie para ese título de IMDb.
core-metadata-search-on = la búsqueda en { $provider }
core-metadata-query-too-short = Escribe al menos dos caracteres para buscar.
core-metadata-not-a-link = no es un enlace
core-metadata-refresh-unsupported = la actualización de una correspondencia solo de MAL o de TMDB
core-metadata-matched = { $show } reconocida.
core-metadata-updated = Se actualizó la información de { $show }.
core-metadata-update-failed = No se pudo actualizar la información de { $show }. Inténtalo más tarde.
core-metadata-progress-refresh = actualización
core-metadata-progress-backfill = completado
core-metadata-updated-many =
    { $count ->
        [one] Se actualizó la información de { $count } serie.
       *[other] Se actualizó la información de { $count } series.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] No se pudo actualizar la información de { $count } serie. Inténtalo más tarde.
       *[other] No se pudo actualizar la información de { $count } series. Inténtalo más tarde.
    }
core-metadata-updated-some =
    { $count ->
        [one] Se actualizó la información de { $count } serie; { $failed } no se pudieron actualizar.
       *[other] Se actualizó la información de { $count } series; { $failed } no se pudieron actualizar.
    }

## core/src/metadata/fetch.rs

core-metadata-no-such-show = TVmaze no tiene la serie { $id }
core-metadata-no-such-media = AniList no tiene el medio { $id }

## core/src/metadata/imdb.rs

core-metadata-progress-imdb-links = enlaces de IMDb de Wikidata
core-metadata-progress-imdb-ratings = valoraciones de IMDb

## core/src/franchise/layout.rs

core-franchise-relation-character = Personajes en común

## core/src/images/mod.rs

core-images-progress-fetching = descargando imágenes

## core/src/transfer/export.rs

core-transfer-exported =
    { $count ->
        [one] Se guardó { $count } serie en { $folder }.
       *[other] Se guardaron { $count } series en { $folder }.
    }

## core/src/transfer/format.rs

core-transfer-not-json = no es JSON: { $error }
core-transfer-wrong-format = no es un documento { $format }

## core/src/transfer/import.rs

core-transfer-imported = Se cargó tu biblioteca desde el archivo.

## core/src/library/delete.rs

core-library-no-such-file = no hay ningún archivo { $id } en la biblioteca
core-library-file-elsewhere-not-removed = el archivo { $id } pertenece a otra serie, así que no se quitó nada
core-library-file-outside = { $path } no es un archivo dentro de una carpeta de la biblioteca, así que no se quitó nada
core-library-episodes-removed =
    { $count ->
        [one] { $count } episodio
       *[other] { $count } episodios
    } de { $title } { $place ->
        [deleted] { $count ->
            [one] eliminado
           *[other] eliminados
        }
       *[trash] { $count ->
            [one] movido
           *[other] movidos
        } a la papelera
    }.
core-library-finished-removed =
    { $count ->
        [one] { $count } serie terminada
       *[other] { $count } series terminadas
    } { $place ->
        [deleted] { $count ->
            [one] eliminada
           *[other] eliminadas
        }
       *[trash] { $count ->
            [one] movida
           *[other] movidas
        } a la papelera
    }.
core-library-watched-removed =
    { $count ->
        [one] { $count } episodio visto
       *[other] { $count } episodios vistos
    } de { $series } series { $place ->
        [deleted] { $count ->
            [one] eliminado
           *[other] eliminados
        }
       *[trash] { $count ->
            [one] movido
           *[other] movidos
        } a la papelera
    }.

## core/src/library/detail.rs

core-library-fix-one-file =
    El único archivo aquí se lee como { $read ->
        [episode] episodio { $number }
       *[label] { $label }
    }. Parece { $what ->
        [film] la película
       *[episode] su único episodio
    }.
core-library-fix-specials =
    { $count ->
        [one] { $count } especial parece
       *[other] { $count } especiales parecen
    } { $episodes ->
        [one] el episodio que falta
       *[other] los episodios que faltan
    }, { $numbers }.
core-library-fix-numbered-on = Los episodios { $from } parecen los { $to } de esta ficha, numerados a continuación de una temporada anterior.
core-library-runs-span = { $first } a { $last }
core-library-runs-list = { $init } y { $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = un número de episodio va de 0 a 10000
core-library-fix-season-range = una temporada llega hasta 200
core-library-fix-extra-range = el índice de un extra llega hasta 10000
core-library-file-elsewhere-not-fixed = el archivo { $id } pertenece a otra serie, así que no se corrigió nada
core-library-fix-undone =
    { $count ->
        [one] Se vuelve a leer { $count } archivo de { $title } por su nombre.
       *[other] Se vuelven a leer { $count } archivos de { $title } por su nombre.
    }
core-library-fixed =
    { $count ->
        [one] Se corrigió cómo se lee { $count } archivo de { $title }.
       *[other] Se corrigió cómo se leen { $count } archivos de { $title }.
    }

## core/src/library/reads.rs

core-library-hid-one = { $title } oculta.
core-library-back-one = { $title } ha vuelto.
core-library-hid-many = { $count } series ocultas.
core-library-back-many = { $count } series han vuelto.
core-library-episode-count-set = { $title } tiene ahora { $count } episodios.
core-library-episode-count-cleared = Se borró el número de episodios de { $title }.

## core/src/library/schedule.rs

core-library-schedule-cleared = Se borraron las notas de horario de { $title }.
core-library-schedule-saved = Se guardaron { $count } notas de horario de { $title }.

## core/src/library/scan.rs

core-library-source-not-absolute = la ruta de una fuente debe ser absoluta
core-library-source-exists = ya es una fuente
core-library-source-nested = está dentro de una fuente existente
core-library-source-added = Se añadió la carpeta { $folder } a tu biblioteca.
core-library-source-removed = Se quitó la carpeta { $folder } de tu biblioteca.
core-library-source-removed-with-shows =
    { $count ->
        [one] Se quitó de tu biblioteca la carpeta { $folder } y su { $count } serie.
       *[other] Se quitó de tu biblioteca la carpeta { $folder } y sus { $count } series.
    }
core-library-series-forgotten = Se quitó { $name } de tu biblioteca.
core-library-source-back = La carpeta { $folder } ha vuelto.
core-library-source-unavailable = La carpeta { $folder } no está disponible. Comprueba que su disco esté conectado.
core-library-source-unreadable = No se pudo leer la carpeta { $folder }. Comprueba que su disco esté conectado.
core-library-source-looks-empty = La carpeta { $folder } parece vacía, así que tus series se dejaron como estaban.
core-library-source-reconcile-failed = No se pudieron actualizar las series de { $folder }.
core-library-scan-done =
    Carpetas revisadas: { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } serie nueva
           *[other] { $added } series nuevas
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } serie cambiada
           *[other] { $changed } series cambiadas
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }{ $removed } { $removed ->
            [one] desaparecida
           *[other] desaparecidas
        }
    }.

## core/src/library/report.rs

core-library-agent-no-answer = sin respuesta en { $span }
core-library-agent-stopped-code = se detuvo con el código { $code }
core-library-agent-stopped = se detuvo
core-library-agent-answered-nothing = no respondió nada
core-library-span-minutes =
    { $count ->
        [one] { $count } minuto
       *[other] { $count } minutos
    }
core-library-span-seconds =
    { $count ->
        [one] { $count } segundo
       *[other] { $count } segundos
    }
core-library-report-no-fix = ninguno de estos archivos tiene una corrección que notificar
core-library-agent-no-triage = { $agent } no escribió ningún triaje: { $why }
core-transfer-progress-sources = fuentes
core-transfer-progress-series = series
core-transfer-progress-history = historial
core-transfer-progress-accounts = cuentas
core-transfer-progress-preferences = preferencias
core-transfer-progress-done = hecho

## core/src/downloads/anirss.rs

core-downloads-anirss-no-answer = anirss no respondió: { $message }
core-downloads-anirss-answered = anirss respondió { $status }: { $text }
core-downloads-no-free-port = no hay ningún puerto libre: { $error }
core-downloads-anirss-exited = anirss se cerró ({ $status })
core-downloads-anirss-exited-said = anirss se cerró ({ $status }): { $said }
core-downloads-anirss-slow-start = anirss no arrancó en { $seconds } segundos
core-downloads-anirss-slow-start-said = anirss no arrancó en { $seconds } segundos: { $said }
core-downloads-anirss-install-mac = Instala anirss { $version } o posterior con: { $command }
core-downloads-anirss-install-linux = Instala anirss { $version } o posterior: el paquete anirss del AUR, o { $command }
core-downloads-anirss-too-old = anirss { $version } es demasiado antiguo para AniBeam, que necesita { $minimum } o posterior. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } no puede guardar por separado el inicio de sesión de cada cliente de torrents, y AniBeam lo necesita para cambiar de cliente; anirss { $minimum } y posteriores sí pueden. { $update }.
core-downloads-anirss-update-mac = Actualízalo con: { $command }
core-downloads-anirss-update-linux = Actualiza el paquete anirss, o ejecuta: { $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = el archivo torrent no llegó: { $message }
core-downloads-torrent-file-answered = el archivo torrent respondió { $status }

## core/src/downloads/feed.rs

core-downloads-feed-answered = el feed respondió { $status }
core-downloads-not-a-feed = no es un feed: { $error }

## core/src/downloads/jackett.rs

core-downloads-jackett-missing = Jackett no está instalado: no se encontró { $program }. Instálalo, o cambia el comando en Ajustes.
core-downloads-endpoint-start-failed = { $name } no pudo arrancar: { $why }
core-downloads-endpoint-start-timeout = { $name } arrancó pero no respondió en { $seconds } segundos.
core-downloads-command-exited = el comando terminó con { $status }

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirss se detuvo
core-downloads-anirss-missing = { $install }. O apunta Ajustes a un servidor en marcha.
core-downloads-anirss-command-missing = No se encontró { $command }. { $install }. O apunta Ajustes a un servidor en marcha.
core-downloads-anirss-start-failed = { $command } no pudo arrancar: { $error }
core-downloads-anirss-too-old-said = { $tooOld } Dijo: { $message }
core-downloads-no-torrent-client = Ningún qBittorrent responde en { $address }, y AniBeam no tiene uno propio que ejecutar. Instala qBittorrent y activa su interfaz web, o instala qbittorrent-nox, que AniBeam ejecutará por ti
core-downloads-own-client-gone = El qBittorrent de AniBeam ya no está: qbittorrent-nox no está instalado. Reinstala AniBeam, o apunta Ajustes a un qBittorrent tuyo
core-downloads-no-port = no hay puerto en { $address }
core-downloads-own-client-settings = no se pueden escribir los ajustes de qBittorrent: { $error }
core-downloads-own-client-start-failed = { $command } no pudo arrancar: { $error }
core-downloads-own-client-slow-start = El qBittorrent de AniBeam no arrancó
core-downloads-own-client-slow-start-said = El qBittorrent de AniBeam no arrancó: { $said }
core-downloads-not-ready = Las descargas no están listas. Revisa Descargas en Ajustes.
core-downloads-ready = Las descargas están listas.
core-downloads-helper-sources =
    { $count ->
        [one] Las descargas buscan ahora con el propio AniBeam, que no lee la { $count } fuente de la configuración del ayudante de búsqueda. Añádela en Ajustes, Descargas, o elige allí el ayudante de búsqueda.
       *[other] Las descargas buscan ahora con el propio AniBeam, que no lee las { $count } fuentes de la configuración del ayudante de búsqueda. Añádelas en Ajustes, Descargas, o elige allí el ayudante de búsqueda.
    }
core-downloads-client-connected = La app de descargas está conectada.
core-downloads-client-not-connected = La app de descargas no está conectada. Revisa su dirección en Ajustes.
core-downloads-not-on-demand = { $name } no es un endpoint bajo demanda
core-downloads-source-running = La fuente de descarga está en marcha.
core-downloads-source-not-running = { $name } no está en marcha. Actívala en las fuentes y confirma para iniciarla.
core-downloads-empty-query = escribe algo que buscar
core-downloads-first-endpoint = el primer endpoint
core-downloads-search-starting = Iniciando la búsqueda
core-downloads-search-starting-endpoint = Iniciando { $endpoint }
core-downloads-search-searching-endpoint = Buscando en { $endpoint }
core-downloads-search-reading = Leyendo los resultados
core-downloads-subscribed = Suscrito a { $name }.
core-downloads-client-no-rule = qBittorrent no tiene ninguna regla llamada { $name }
core-downloads-trying-search = Probando "{ $search }"
core-downloads-nothing-selected = no hay nada seleccionado para descargar
core-downloads-added =
    { $count ->
        [one] Se añadió { $count } descarga a tus descargas.
       *[other] Se añadieron { $count } descargas a tus descargas.
    }
core-downloads-movie-link = el enlace de una película debe ser un magnet o una URL http(s)
core-downloads-added-movie = Se añadió una película a tus descargas.
core-downloads-unsubscribed = Ya no estás suscrito a { $name }.
core-downloads-library-folder = { $folder } es la carpeta de la biblioteca de { $name }, con episodios que esta suscripción no descargó: quita la suscripción sin sus archivos, o quita { $name } desde su página
core-downloads-no-torrents-named = no se indicó ningún torrent
core-downloads-paused =
    { $count ->
        [one] { $count } descarga en pausa.
       *[other] { $count } descargas en pausa.
    }
core-downloads-resumed =
    { $count ->
        [one] { $count } descarga reanudada.
       *[other] { $count } descargas reanudadas.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] Se eliminó { $count } descarga con sus archivos.
       *[other] Se eliminaron { $count } descargas con sus archivos.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] Se quitó { $count } descarga de tu lista.
       *[other] Se quitaron { $count } descargas de tu lista.
    }
core-downloads-nothing-to-remove = nada que quitar
core-downloads-removed-show = Se quitó { $title } de tus descargas.
core-downloads-removed-the-show = Se quitó la serie de tus descargas.
core-downloads-deleted-still-listed = Los archivos eliminados siguen en la lista de la app de descargas.
core-downloads-feed-url = un feed es una URL http(s)
core-downloads-rule-needs-name = una regla tiene un nombre
core-downloads-no-rule = no hay ninguna regla llamada { $name }
core-downloads-feed-no-search = el feed no lleva ninguna búsqueda que acortar
core-downloads-no-repair = { $name } no necesita reparación.
core-downloads-repaired = Se reparó la suscripción a { $name }.
core-downloads-repair-failed = ninguna búsqueda más corta encuentra nada; incluso "{ $search }" vuelve vacía
core-downloads-no-saved-client = no hay ningún cliente de torrents guardado { $id }
core-downloads-client-switched = Las descargas usan ahora otra app de descargas.
core-downloads-host-not-running = no está en marcha en el host
core-downloads-host-late = no respondió a tiempo

## core/src/downloads/native.rs

core-downloads-native-empty-query = búsqueda vacía
core-downloads-native-no-results = sin resultados para { $query }
core-downloads-native-feed-taken = un feed llamado { $name } ya lee otra dirección
core-downloads-native-no-items = no se indicó ningún elemento
core-downloads-native-bad-link = los enlaces deben ser magnet: o URL http(s), se recibió { $link }
core-downloads-native-no-links = no se indicó ningún enlace
core-downloads-native-no-names = no se indicó ningún nombre de suscripción
core-downloads-native-files-elsewhere = el cliente de torrents guarda sus archivos en otra máquina, así que solo pueden irse con sus torrents
core-downloads-native-unknown-feeds = feeds desconocidos: { $names }
core-downloads-native-outside-folders = no se quitan archivos fuera de las carpetas de descarga: { $paths }
core-downloads-refuse-home-folder = { $path } es la carpeta personal o la contiene, así que no se quitó nada
core-downloads-refuse-library-folder = { $path } es una carpeta de la biblioteca o contiene una, así que no se quitó nada

## core/src/downloads/paths.rs

core-downloads-no-default-folder = qBittorrent no indica ninguna carpeta de descarga predeterminada; pon una en qBittorrent
core-downloads-name-empty = el nombre no puede estar vacío
core-downloads-name-slash = el nombre no puede contener / ni \
core-downloads-name-not-folder = { $name } no es un nombre de carpeta

## core/src/downloads/prune.rs

core-downloads-pruned = Ya no estás suscrito a { $title }, terminó de emitirse.
core-downloads-prune-failed = No se pudo cancelar la suscripción a { $title }. Revisa Descargas en Ajustes.

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path } no es una ruta absoluta simple, así que no se quitó nada
core-downloads-refuse-root = { $path } es la raíz del sistema de archivos, así que no se quitó nada
core-downloads-refuse-outside-library = { $path } no está dentro de una carpeta de la biblioteca, así que no se quitó nada
core-downloads-refuse-save-folder = { $path } es la carpeta donde se guardó el torrent, no lo que descargó, así que no se quitó nada

## core/src/downloads/watch.rs

core-downloads-episode-missed = El episodio { $episode } de { $title } no ha aparecido tres días después de emitirse. Tu feed sigue buscándolo.

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = el comando de anirss no puede estar vacío
core-downloads-settings-anirss-address = la dirección de anirss debe empezar por http:// o https://
core-downloads-settings-client-address = la dirección de { $client } debe empezar por http:// o https://
core-downloads-settings-bad-id = { $id } no es un id de cliente: usa letras, cifras, - y _
core-downloads-settings-same-id = dos clientes tienen el id { $id }
core-downloads-settings-saved-address = la dirección de { $name } debe empezar por http:// o https://
core-downloads-settings-same-name = dos clientes se llaman { $name }; dale a cada uno un nombre propio

## core/src/downloads/qbt.rs

core-downloads-qbt-no-answer = qBittorrent no respondió en { $address }: { $message }
core-downloads-qbt-password-rejected = qBittorrent rechazó la contraseña; corrígela en Ajustes
core-downloads-qbt-banned = qBittorrent ha bloqueado esta dirección tras varios inicios de sesión fallidos; espera, o reinicia qBittorrent
core-downloads-qbt-login-rejected = qBittorrent rechazó el usuario o la contraseña
core-downloads-qbt-login-failed = el inicio de sesión de qBittorrent en { $address } respondió { $status }; se reintentará en la próxima llamada
core-downloads-qbt-not-web-ui = { $address } respondió { $status }, pero no como la interfaz web de qBittorrent. Revisa la dirección en Ajustes, y que la interfaz web esté activada en qBittorrent
core-downloads-qbt-not-added = qBittorrent no añadió los torrents: { $said }
core-downloads-qbt-some-not-added = qBittorrent no pudo añadir { $count } de los torrents: { $said }
core-downloads-qbt-answered = qBittorrent respondió { $status }: { $line }

## core/src/downloads/replace.rs

core-downloads-replace-bad-item = el elemento { $item } no es uno de los { $count } lanzamientos
core-downloads-not-replaced = No se reemplazó el episodio { $episode }.
core-downloads-replaced = Se reemplazó el episodio { $episode } por la nueva descarga.

## core/src/downloads/rolling.rs

core-downloads-no-roll = no hay ninguna descarga continua { $id }
core-downloads-rolled = { $title }: ahora en los episodios { $from } a { $to } de { $total }.
core-downloads-roll-ahead = mantén al menos un episodio por delante
core-downloads-roll-nothing = nada que obtener

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = una fuente de búsqueda JSON viene de un repo de endpoints cuya definición asigna sus campos
core-releases-endpoint-no-name = falta `name`
core-releases-endpoint-bad-name = { $name }: usa solo letras, cifras, '.', '_', '-'
core-releases-endpoint-json = { $name }: { $why }
core-releases-endpoint-unknown-kind = { $name }: tipo desconocido { $kind } (válidos: { $valid })
core-releases-endpoint-no-url = { $name }: falta `url`
core-releases-endpoint-not-http = { $name }: la dirección debe empezar por http:// o https://
core-releases-endpoint-needs-query = { $name }: la url { $kind } debe contener el marcador {"{"}query{"}"}
core-releases-endpoint-no-query = { $name }: una url { $kind } añade la búsqueda por sí misma, así que no lleva {"{"}query{"}"}

## core/src/releases/fetch.rs

core-releases-indexer-error = el indexador respondió con el error { $code }
core-releases-bad-feed = feed incorrecto: { $error }
core-releases-feed-answered = el feed respondió { $status }

## core/src/releases/json_feed.rs

core-releases-not-feed-json = no es el JSON del feed: { $error }

## core/src/releases/json.rs

core-releases-not-json = no es JSON: { $error }
core-releases-search-error-said = la búsqueda respondió con un error: { $message }
core-releases-search-error = la búsqueda respondió con un error
core-releases-no-results-field = la respuesta no tiene resultados en { $path }
core-releases-results-not-list = los resultados de la respuesta no son una lista
core-releases-search-answered = la búsqueda respondió { $status }

## core/src/releases/native.rs

core-releases-no-endpoints = aún no hay endpoints: añade un repo, o uno a mano, en Endpoints, en Más
core-releases-no-source-named = no hay ninguna fuente llamada { $name }
core-releases-empty-query = escribe algo que buscar
core-releases-cannot-follow = { $name } no se puede seguir: responde a búsquedas, pero no tiene feed donde buscar episodios nuevos
core-releases-endpoint-gone = { $name } ya no está disponible: se quitó, se desactivó o le falta su clave

## core/src/releases/quirks.rs

core-releases-quirks-variant = one_variant { $word } debe ser una sola palabra corta
core-releases-quirks-pattern-count = show_feed admite de 1 a { $most } patrones de página
core-releases-quirks-pattern-long = un patrón de página de show_feed es demasiado largo
core-releases-quirks-pattern-bad = patrón de página de show_feed: { $error }
core-releases-quirks-pattern-slug = un patrón de página de show_feed necesita un grupo llamado slug
core-releases-quirks-url-https = la url de show_feed debe ser https
core-releases-quirks-url-host = la url de show_feed debe estar en el propio host del endpoint
core-releases-quirks-url-slug = la url de show_feed lleva {"{"}slug{"}"} una vez y ningún otro marcador

## core/src/releases/refinement.rs

core-releases-refine-no-tokens = op 'tokens' necesita una lista `tokens` no vacía

## core/src/releases/grid.rs

core-releases-group-batch = Lote de { $group }
core-releases-episodes-phrase =
    { $count ->
        [one] Episodio { $list }
       *[other] Episodios { $list }
    }
core-releases-episode-range = { $from } a { $to }

## core/src/subscriptions.rs

core-subscriptions-platform = anirss en esta plataforma
core-subscriptions-unreadable = no se pudo leer su respuesta ({ $message })
core-subscriptions-stopped = se detuvo sin decir por qué
core-subscriptions-stopped-said = se detuvo: { $said }

## Names that stay the same in every language

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList
