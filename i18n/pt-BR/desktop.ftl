# pt-BR translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = Início
nav-library = Biblioteca
nav-airing = No ar agora
nav-feed = Feed
nav-watching = Assistindo
nav-stats = Estatísticas
nav-metadata = Info da série
nav-downloads = Downloads
nav-settings = Configurações
nav-series = Série
nav-player = Player
nav-show = Série
nav-recommended = Recomendados

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } erro
   *[other] { $count } erros
}

## ScanSummary

scan-summary-nothing = Nada para mostrar ainda
scan-summary-found = { $count ->
    [one] { $count } série encontrada
   *[other] { $count } séries encontradas
}, { $recognised } reconhecidas
scan-summary-found-need = { $count ->
    [one] { $count } série encontrada
   *[other] { $count } séries encontradas
}, { $recognised } reconhecidas, { $need } precisam de você
scan-summary-episodes = { $count ->
    [one] { $count } episódio
   *[other] { $count } episódios
} no disco.
scan-summary-episodes-watched = { $count ->
    [one] { $count } episódio
   *[other] { $count } episódios
} no disco, { $watched } já assistidos no { $tracker }.
scan-summary-title = Sua pasta entrou
scan-summary-count-found = encontradas
scan-summary-count-recognised = reconhecidas
scan-summary-count-need = precisam de você
scan-summary-later = Depois
scan-summary-match = Vincular as { $count }
scan-summary-open-library = Abrir a biblioteca

## UpdateNotice

update-notice-check-failed = O AniBeam não conseguiu verificar atualizações. Confira sua conexão com a internet e tente de novo.
update-notice-up-to-date = O AniBeam está atualizado
update-notice-downloading = Baixando o AniBeam { $version }
update-notice-ready = O AniBeam { $version } está pronto
update-notice-failed = A atualização falhou
update-notice-on-aur = O AniBeam { $version } está no AUR
update-notice-out = Saiu o AniBeam { $version }
update-notice-out-titled = Saiu o AniBeam { $version } “{ $title }”
update-notice-progress = { $percent }%  ·  { $got } de { $total } MB
update-notice-restart-hint = Reinicie agora, ou ele abre na próxima vez
update-notice-from-source = Esta cópia foi compilada do código: faça pull e recompile
update-notice-no-download = Nenhum download aqui para este computador
update-notice-install-hint = Atualizar instala e reinicia o AniBeam
update-notice-retry = Repetir
update-notice-update = Atualizar
update-notice-restart = Reiniciar
update-notice-copied = Copiado
update-notice-copy = Copiar
update-notice-release-page = Página da versão
update-notice-later = Depois

## WhatsNew

whats-new-title = Novidades
whats-new-got-it = Entendi

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = d MMM HH:mm
activity-drawer-stage-library = Biblioteca
activity-drawer-stage-metadata = Metadados
activity-drawer-stage-trackers = Trackers
activity-drawer-stage-franchise = Franquia
activity-drawer-stage-playback = Reprodução
activity-drawer-stage-store = Dados
activity-drawer-stage-system = Sistema
activity-drawer-stage-downloads = Downloads
activity-drawer-level-info = Info
activity-drawer-level-warn = Aviso
activity-drawer-level-error = Erro
activity-drawer-copied-line = Linha copiada
activity-drawer-title = Atividade
activity-drawer-everything = Tudo
activity-drawer-problems = Problemas
activity-drawer-copied = Copiado
activity-drawer-copy = Copiar
activity-drawer-clear-every = Apagar tudo
activity-drawer-copy-tip = Clique para copiar a linha
activity-drawer-fold = Recolher
activity-drawer-empty-technical = Nada registrado ainda.
activity-drawer-empty-problems = Nenhum problema. Tudo funcionou.
activity-drawer-empty = Nada para relatar ainda. O que o AniBeam faz por você aparece aqui.

## ActivityColumn

activity-column-episode-run = Episódios { $first } a { $last }
activity-column-episode-list = Episódios { $list }
activity-column-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
activity-column-batch = Download em lote
activity-column-download = Download
activity-column-name-batch = { $title }, um lote
activity-column-name-episode = { $title }, episódio { $number }
activity-column-title = Atividade
activity-column-down = down
activity-column-up = up
activity-column-not-connected = { $client } não está conectado, então nada anda aqui.
activity-column-connect-again = Conectar de novo
activity-column-downloading = Baixando agora
activity-column-fewer = Menos
activity-column-more = E mais { $count }
activity-column-nothing-downloading = Nada sendo baixado.
activity-column-finished-today = Concluídos hoje
activity-column-today = Hoje
activity-column-today-empty = Buscas, downloads e novas séries seguidas aparecem aqui conforme acontecem.
activity-column-stopping = Parando
activity-column-starting = Iniciando
activity-column-start-tip = Começar a baixar
activity-column-stop-tip = Parar de baixar; o que já veio fica
activity-column-cancelling = Cancelando
activity-column-cancel-all-tip = Cancelar estes downloads
activity-column-cancel-tip = Cancelar este download
activity-column-size-of = { $got } de { $total }
activity-column-files-missing = Arquivos faltando
activity-column-time-format = HH:mm
activity-column-left = faltam { $time }
activity-column-waiting-peers = Esperando peers
activity-column-episodes-done = { $count ->
    [one] { $done } de { $count } episódio pronto
   *[other] { $done } de { $count } episódios prontos
}
activity-column-cube-skipped = Episódio { $number }: pulado
activity-column-cube-done = Episódio { $number }: pronto
activity-column-cube-progress = Episódio { $number }: { $percent }%

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Pronto
frame-strip-progress = { $doing }, { $done } de { $total }
frame-strip-idle = Nada em andamento
frame-strip-time-format = HH:mm
frame-finish-sign-in = Termine o login no { $tracker } primeiro
frame-did-not-work = Não deu certo.
frame-could-not-open = O AniBeam não conseguiu abrir isso. Copie e abra você mesmo.
frame-copy = Copiar
frame-copied = Copiado
frame-status-refused = O { $tracker } não aceitou
frame-status-refused-why = O { $tracker } não aceitou: { $why }
frame-lists-refused = O { $tracker } não aceitou { $count ->
    [one] 1 série
   *[other] { $count } séries
}; veja o log de atividade
frame-lists-moved = { $count ->
    [one] 1 série movida
   *[other] { $count } séries movidas
} para o { $tracker }
frame-notes-refused = O { $tracker } não aceitou suas notas
frame-notes-refused-why = O { $tracker } não aceitou suas notas: { $why }
frame-notes-saved = Notas salvas
frame-connected = Conectado ao { $tracker } como { $user }
frame-sign-in-stopped = O login no { $tracker } parou antes de terminar.
frame-toast-hide-details = Ocultar detalhes
frame-toast-details = Detalhes
frame-toast-details-tip = Mostrar o que deu errado, palavra por palavra, e copiar
frame-forward = Avançar
frame-copy-magnet = Copiar magnet
frame-copy-link = Copiar link do torrent
frame-magnet-copied = Magnet copiado
frame-link-copied = Link copiado
frame-open-release-page = Abrir página do release
frame-open-folder = Abrir pasta
frame-copy-path = Copiar caminho
frame-path-copied = Caminho copiado
frame-mark-unwatched = Não assistido a partir daqui
frame-mark-watched = Assistido até aqui
frame-copy-title = Copiar título
frame-open-on-anilist = Abrir no AniList
frame-list-home-online = Mude em cada lista conectada
frame-list-home-unmatched = Guardado neste dispositivo: nenhuma lista tem esta série
frame-list-home-not-carried = Guardado neste dispositivo: sua lista não tem esta série
frame-list-home-offline = Guardado neste dispositivo até você conectar o AniList ou o MyAnimeList
frame-moving-to = Movendo para { $list }
frame-moved-to = Movido para { $list }
frame-plan-added = { $title } vai para a sua lista Planejando
frame-plan-listed = { $title } já está na sua lista
frame-delete-watched = { $kind ->
    [film] Excluir filmes assistidos
   *[episode] Excluir episódios assistidos
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] Excluir o filme
       *[episode] Excluir o episódio
    }
   *[other] { $kind ->
        [film] Excluir os { $count } filmes
       *[episode] Excluir os { $count } episódios
    }
}
frame-series-gone = Essa série não existe mais
frame-delete-none-watched = Nenhum episódio assistido de { $title } no disco
frame-delete-none = Nada de { $title } no disco
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } filme assistido
           *[other] { $count } filmes assistidos
        }
       *[episode] { $count ->
            [one] { $count } episódio assistido
           *[other] { $count } episódios assistidos
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } filme
           *[other] { $count } filmes
        }
       *[episode] { $count ->
            [one] { $count } episódio
           *[other] { $count } episódios
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } extra
   *[other] { $count } extras
}
frame-delete-and = { $first } e { $second }
frame-delete-heading = Excluir { $what } de { $title }?
frame-delete-base = Os arquivos e as legendas deles vão para a lixeira, então dá para recuperá-los de lá. Os torrents saem do cliente de torrent, então podem ser baixados de novo.
frame-delete-confirm = Mover para a lixeira
frame-delete-done = { $what } de { $title } na lixeira
frame-delete-watched-done = { $count ->
    [one] { $count } episódio
   *[other] { $count } episódios
}, { $size }, na lixeira
frame-delete-nothing = Nada para excluir
frame-fix-not-drafted = Corrigido. Não foi possível rascunhar a issue
frame-fix-agent = Corrigido. { $agent } está analisando, e a issue abre quando terminar
frame-fix-opening = Corrigido. Abrindo a issue
frame-fix-not-saved = A correção não foi salva: { $error }
frame-fix-undone = { $count ->
    [one] { $count } arquivo lido pelo nome de novo
   *[other] { $count } arquivos lidos pelo nome de novo
}
frame-fix-done = { $count ->
    [one] Corrigida a leitura de { $count } arquivo
   *[other] Corrigida a leitura de { $count } arquivos
}
frame-report-failed = Não foi possível rascunhar a issue: { $error }
frame-report-untriaged = A issue está aberta no seu navegador, sem triagem. { $error }
frame-report-open = A issue está aberta no seu navegador, pronta para enviar
frame-downloads-not-read = Não lido.

## HomePage

home-title = Início
home-scan-started = Verificação iniciada
home-ago-minutes = há { $count } min
home-ago-hours = há { $count } h
home-ago-days = há { $count } d
home-today = Hoje
home-tomorrow = Amanhã
home-yesterday = Ontem
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d MMM
home-day-date-format = ddd, d MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = Sem data ainda
home-unit-days = { $count ->
    [one] dia
   *[other] dias
}
home-unit-hours = h
home-unit-minutes = min
home-week-late = { $count ->
    [one] { $count } episódio atrasado
   *[other] { $count } episódios atrasados
}
home-week-waiting = { $count ->
    [one] { $count } episódio lançado para assistir
   *[other] { $count } episódios lançados para assistir
}
home-week-coming = { $count ->
    [one] { $count } episódio nos próximos sete dias
   *[other] { $count } episódios nos próximos sete dias
}
home-week-next = A seguir: { $title } em { $time }
home-countdown-days = { $days } d { $hours } h
home-countdown-hours = { $hours } h { $minutes } min
home-countdown-minutes = { $minutes } min
home-out-not-downloaded = Ainda não baixado
home-out-downloading = Baixando
home-out-when = { $day } { $time }
home-next-up = A seguir
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, horário estimado
home-next-when-season = Temporada { $season }, { $span }, { $time }
home-next-when-season-estimated = Temporada { $season }, { $span }, { $time }, horário estimado
home-no-date = Sem data
home-tile-watched = assistido
home-tile-downloading = baixando
home-tile-out = lançado { $ago }
home-tile-not-out = não lançado
home-tile-estimated = estimado
home-tile-season-span = T{ $season } { $span }
home-tile-to-watch = { $count } para assistir
home-day-nothing = Nada
home-now = Agora { $time }
home-card-aired = { $episode } lançado { $ago }
home-continue = Continuar
home-continue-fact = { $count } séries para continuar
home-on-hold = Em pausa
home-on-hold-fact = { $count } séries para retomar
home-hero-open = Abrir { $title }
home-hero-pick-up = Continue de onde parou
home-hero-up-next = A seguir
home-hero-downloading = Baixando { $count ->
    [one] { $count } episódio
   *[other] { $count } episódios
}
home-hero-min-left = faltam { $count } min
home-hero-watched = { $watched } de { $total } assistidos
home-hero-resume = Retomar episódio { $number }
home-hero-play = Assistir episódio { $number }
home-hero-resume-unnumbered = Retomar episódio
home-hero-play-unnumbered = Assistir episódio
home-downloaded = Baixados recentemente
home-downloaded-fact = { $count } séries com arquivos novos nesta quinzena
home-week = Esta semana
home-week-show-empty = Mostrar dias vazios
home-week-hide-empty = Ocultar dias vazios
home-week-airing = No ar agora
home-week-out = Já saiu
home-week-coming-up = Em breve
home-recommended = Recomendados para você
home-recommended-fact = Da sua própria lista
home-recommended-scope = Recomendados de
home-recommended-all = Todos
home-recommended-in-library = Na biblioteca
home-recommended-on-list = Na lista
home-recommended-none-library = Não sobrou nada da sua biblioteca para recomendar. Todos mostra séries que você ainda não tem.
home-recommended-none-list = Não sobrou nada das suas listas para recomendar. Todos mostra séries fora das suas listas.
home-catch-up = Pôr em dia
home-catch-up-fact = { $count } séries com episódios esperando
home-fresh = Comece algo novo
home-fresh-fact = { $count } séries não iniciadas
home-genres = Gêneros
home-open-library = Abrir biblioteca
home-dropped = Abandonados
home-dropped-fact = { $count ->
    [one] { $count } arquivo ainda no disco
   *[other] { $count } arquivos ainda no disco
}
home-out-now = Já saiu
home-out-now-fact = Episódios novos desta semana, do AniList
home-popular = Populares da temporada
home-add-folder-title = Suas próprias séries também ficam aqui
home-add-folder-body = Inclua a pasta onde ficam seus animes e o Início se enche com o que você está assistindo.
home-add-folder-button = Incluir pasta
home-empty-scanned-title = Nada encontrado nas suas pastas ainda
home-empty-title = Nenhuma série ainda
home-empty-scanned-body = O AniBeam não achou séries nem filmes nas pastas que observa. Verifique de novo, ou confira as pastas nas Configurações.
home-empty-body = Inclua a pasta onde ficam seus animes e a biblioteca se monta sozinha.
home-scan-all = Verificar tudo
home-library-settings = Config. da biblioteca
home-failed-title = O Início não carregou
home-failed-body = O AniBeam não conseguiu ler a biblioteca agora. Tente de novo em instantes.

## Onboarding

onboarding-language = Idioma
onboarding-step-welcome = Boas-vindas
onboarding-step-library = Sua biblioteca
onboarding-step-lists = Sua lista
onboarding-step-downloads = Downloads
onboarding-step-ready = Pronto
onboarding-scan-reading = Lendo a pasta
onboarding-scan-found = { $count } séries encontradas. Continue quando quiser; o resto se completa sozinho.
onboarding-scan-starter-empty = Sua nova pasta está vazia por enquanto. As séries que você colocar nela aparecem aqui sozinhas.
onboarding-scan-no-video = Nenhum arquivo de vídeo apareceu. Confira se é a pasta certa.
onboarding-scan-your-shows = suas séries
onboarding-scan-looking-up = Buscando { $what }
onboarding-scan-looking-up-tally = Buscando { $what }, { $posters } de { $count } encontradas
onboarding-scan-posters = Baixando pôsteres
onboarding-scan-posters-tally = Baixando pôsteres, { $posters } de { $count } encontrados
onboarding-scan-seasons = Buscando temporadas relacionadas
onboarding-scan-seasons-tally = Buscando temporadas relacionadas, { $posters } de { $count } encontradas
onboarding-choose-folder = Escolha sua pasta de anime
onboarding-about = Um minuto de configuração. Sua biblioteca termina sozinha.
onboarding-step-later = Depois
onboarding-skip-setup = Pular configuração
onboarding-title-welcome = Boas-vindas ao AniBeam
onboarding-title-library = Onde estão seus animes?
onboarding-title-lists = Conecte sua lista de anime
onboarding-title-downloads = Como funcionam os downloads
onboarding-title-ready = Tudo pronto
onboarding-title-later = { $count ->
    [1] Pronto, uma coisa para depois
    [2] Pronto, duas coisas para depois
   *[other] Pronto, três coisas para depois
}
onboarding-body-welcome = O AniBeam guarda seus animes em um só lugar. Ele encontra as séries no seu disco, reproduz aqui mesmo e mantém o AniList ou o MyAnimeList em dia enquanto você assiste.
onboarding-body-library = Escolha a pasta onde você guarda suas séries. Cada pasta dentro dela vira uma série, com pôster, episódios e onde você parou. O AniBeam só lê a pasta. Ele nunca move nem renomeia um arquivo.
onboarding-body-lists = Entre uma vez, e cada episódio que você terminar aqui é marcado na sua lista, sem digitar nada. O login acontece no seu navegador, então o AniBeam nunca vê sua senha.
onboarding-body-downloads = Siga uma série na página Downloads e cada episódio novo chega à sua biblioteca assim que vai ao ar. Duas coisas fazem isso funcionar: um lugar para buscar e um cliente de torrent para baixar.
onboarding-body-ready = Tudo aqui pode ser mudado depois nas Configurações, e Configurações, Sobre roda esta configuração de novo.
onboarding-skip-step = Pular por agora
onboarding-get-started = Começar
onboarding-open-app = Abrir o AniBeam
onboarding-continue = Continuar
onboarding-payoff-library = Sua biblioteca
onboarding-payoff-library-line = Pôsteres, episódios e o que vem a seguir, para cada série nas suas pastas.
onboarding-payoff-plays = Reproduz aqui mesmo
onboarding-payoff-plays-line = Legendas, pular abertura, e lembra onde você parou.
onboarding-payoff-list = Sua lista, sincronizada
onboarding-payoff-list-line = Termine um episódio e ele conta no AniList ou no MyAnimeList.
onboarding-payoff-episodes = Os episódios chegam
onboarding-payoff-episodes-line = Siga uma série e cada episódio novo chega à sua biblioteca.
onboarding-films-too = Uma pasta de filmes também funciona: chame-a de Movies.
onboarding-choose-folder-button = Escolher pasta
onboarding-no-folder = Ainda não tenho uma pasta
onboarding-starter-where = Isso cria uma pasta Anime em { $dir } e a usa.
onboarding-add-another = Incluir outra pasta
onboarding-shelf-more = mais
onboarding-preview-show = Sua série
onboarding-preview-finished = Episódio 3 terminado aqui
onboarding-preview-on-list = Na sua lista: 3 assistidos
onboarding-preview-set-by = Marcado pelo AniBeam assim que os créditos rolam
onboarding-lists-connected = Conectado. O que você assiste aqui agora conta na sua lista.
onboarding-lists-no-account = Ainda sem conta? As duas são grátis, e uma basta. Você pode desconectar quando quiser em Configurações, Contas.
onboarding-search-title = Um lugar para buscar
onboarding-search-ready = { $count ->
    [one] { $count } fonte pronta para buscar.
   *[other] { $count } fontes prontas para buscar.
}
onboarding-search-none = O AniBeam não vem com fontes próprias. Você as inclui de um repo de fontes: um link que alguém compartilha com você e que você cola em Configurações, Downloads.
onboarding-sources-after = Abre depois da configuração
onboarding-sources-add = Incluir um repo depois
onboarding-fetch-title = Quem baixa
onboarding-fetch-auto = Deixar com o AniBeam
onboarding-fetch-recommended = Recomendado
onboarding-fetch-auto-line = O AniBeam traz o próprio qBittorrent e o roda discretamente em segundo plano. Nada para instalar, nada para preencher.
onboarding-fetch-existing = Já uso o qBittorrent
onboarding-fetch-existing-line = O AniBeam se conecta ao qBittorrent que você já roda. Você informa o endereço e o login dele em seguida, nas Configurações.
onboarding-ready-library = Biblioteca
onboarding-ready-library-none = Nenhuma pasta ainda. Fica em Configurações, Biblioteca quando você quiser.
onboarding-ready-library-series = { $count ->
    [one] { $count } pasta
   *[other] { $count } pastas
}, { $series } séries até agora
onboarding-ready-library-empty = { $count ->
    [one] { $count } pasta
   *[other] { $count } pastas
}, vazia por enquanto
onboarding-ready-library-scanning = { $count ->
    [one] { $count } pasta
   *[other] { $count } pastas
}, verificando
onboarding-ready-lists = Sua lista
onboarding-ready-lists-as = { $tracker }, como { $user }
onboarding-ready-lists-none = Ainda não conectado. Fica em Configurações, Contas quando você quiser.
onboarding-ready-downloads = Downloads
onboarding-ready-downloads-line = { $sources ->
    [0] Nenhuma fonte ainda
    [one] { $sources } fonte
   *[other] { $sources } fontes
}, { $client ->
    [own] seu próprio qBittorrent.
   *[bundled] o qBittorrent do AniBeam.
} { $next ->
    [ready] Tudo pronto.
    [opens] Configurações, Downloads abre em seguida.
   *[both] Os dois ficam em Configurações, Downloads.
}
onboarding-ready-open-downloads = Abrir Configurações, Downloads
onboarding-ready-set-up = Configurar agora

## Rail.qml

rail-home = Início
rail-library = Biblioteca
rail-airing = No ar
rail-feed = Feed
rail-stats = Estatísticas
rail-show-info = Info da série
rail-downloads = Downloads
rail-settings = Configurações
rail-downloads-complete =
    { $count ->
        [one] Downloads: { $count } série seguida está completa
       *[other] Downloads: { $count } séries seguidas estão completas
    }
rail-title-language-name = Idioma dos títulos: { $language }. Clique para o próximo
rail-title-language-english = Inglês
rail-title-language-tip = Idioma dos títulos: romaji, inglês ou japonês
rail-whats-new-in = Novidades da { $release }
rail-whats-new = Novidades
rail-settings-note = Configurações: { $note }

## FixSheet.qml

fix-kind-episode = Episódio
fix-kind-opening = Abertura
fix-kind-ending = Encerramento
fix-kind-preview = Prévia
fix-kind-special = Especial
fix-kind-other = Outro
fix-reads-season-episode = Temporada { $season } episódio { $number }
fix-reads-extra = { $kind } { $index }
fix-apply-failed = Não foi possível aplicar as correções.
fix-title = Corrigir a leitura dos arquivos
fix-lede = { $title }. Diga o que cada arquivo realmente é; toda verificação depois disso mantém.
fix-rescan = Verificar pasta de novo
fix-rescan-started = Lendo a pasta de novo
fix-filled-in = Preenchido
fix-fill-in = Preencher
fix-column-file = Arquivo
fix-column-reads-as = Lido como
fix-column-should-be = Deveria ser
fix-fixed = Corrigido
fix-number-placeholder = Nº
fix-index-placeholder = Nenhum
fix-keep = Manter a correção
fix-leave-as-read = Deixar este arquivo como foi lido
fix-read-by-name = Ler este arquivo pelo nome de novo
fix-report = Abrir uma issue no GitHub
fix-report-body = Abre uma issue preenchida para você enviar. Ela é pública e cita estes arquivos.
fix-report-body-agent = Abre uma issue preenchida para você enviar. Ela é pública e cita estes arquivos. { $agent } analisa antes.
fix-close-unapplied =
    { $count ->
        [one] Fechar sem aplicar { $count } correção?
       *[other] Fechar sem aplicar { $count } correções?
    }
fix-number-invalid = Corrija primeiro o número em vermelho
fix-close-anyway = Fechar assim mesmo
fix-keep-editing = Continuar editando
fix-apply-count =
    { $count ->
        [one] Aplicar { $count } correção
       *[other] Aplicar { $count } correções
    }

## MatchPanel.qml

match-matched-to = Vinculada a { $title }
match-applied = Vínculo aplicado
match-found-tvmaze = Série encontrada no TVmaze
match-found-anilist = Encontrada no AniList
match-found-anilist-season = Encontrada no AniList, temporada { $season }
match-found-mal = Encontrada no MyAnimeList
match-sites-and = { $first } e { $next }
match-matched-on = Vinculada em { $sites }
match-provider-silent = { $provider } não respondeu
match-title-walk = Vínculo, { $walk }
match-title = Vínculo
match-skip = Pular
match-refresh-info = Atualizar info
match-refreshing = Atualizando { $title }
match-clear-confirm = Remover: apaga o link e a contagem de episódios definida
match-clear = Remover vínculo
match-cleared = Vínculo removido
match-search-placeholder = Buscar no { $provider }
match-link-tvmaze = Cole um link do TVmaze ou do IMDb
match-link-anilist = Cole um link do AniList ou do MyAnimeList
match-look-up = Buscar
match-searching = Buscando no { $provider }
match-type-more = Digite pelo menos duas letras para buscar no { $provider }.
match-no-results = Nenhum título encontrado no { $provider }.

## WatchingPage.qml

watching-episode-code = Ep { $number }
watching-countdown-days = { $days } d { $hours } h
watching-countdown-hours = { $hours } h { $minutes } min
watching-countdown-minutes = { $minutes } min
watching-on-disk = No disco
watching-on-anilist = No AniList
watching-ready-to-start = Pronto para começar
watching-not-in-library = Fora da biblioteca
watching-get-to-start = Baixe episódios para começar
watching-anilist-updated = AniList atualizado { $when }
watching-airing = No ar
watching-next-in = Próximo: { $title } em { $wait }
watching-episodes-waiting = Episódios esperando
watching-series-behind =
    { $count ->
        [one] { $count } série atrasada
       *[other] { $count } séries atrasadas
    }
watching-up-to-date = Em dia
watching-anilist-only = Só no AniList
watching-planning-summary-off = O que você planeja assistir no AniList.
watching-planning-summary =
    { $count ->
        [one] { $count } série planejada, { $here } no disco.
       *[other] { $count } séries planejadas, { $here } no disco.
    }
watching-summary-off = O que você está assistindo ou reassistindo no AniList.
watching-summary-waiting =
    { $count ->
        [one] { $count } série tem episódios esperando.
       *[other] { $count } séries têm episódios esperando.
    }
watching-caught-up = Tudo em dia.
watching-start-watching = Começar a assistir
watching-play-next = Assistir próximo
watching-open-on-anilist = Abrir no AniList
watching-refreshing = Atualizando
watching-connect-title = O AniList não está conectado
watching-connect-body = Conecte e esta página lista as séries que você vê ou revê no AniList, com Assistir nas que você tem e contagem regressiva nas que estão no ar.
watching-connect-waiting = Esperando o navegador
watching-connect-anilist = Conectar AniList
watching-load-failed-title = A lista Assistindo não carregou
watching-load-failed-body = Não foi possível acessar o AniList. A lista volta na próxima atualização.
watching-planning-failed-title = A lista Planejando não carregou
watching-planning-failed-body = O cache dela não pôde ser lido. Tente de novo, ou ela volta na próxima atualização.
watching-planning-empty-title = Nada na sua lista Planejando
watching-planning-empty-body = Quero assistir, em No ar agora, ou Planejando no menu de lista de uma série, coloca uma série aqui.
watching-airing-now = No ar agora
watching-empty-title = Nada na sua lista Assistindo
watching-empty-body = Marque uma série como Assistindo no AniList e ela aparece aqui.
watching-open-anilist = Abrir o AniList

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] { $count } série ocultada
       *[other] { $count } séries ocultadas
    }
library-hidden-sync-stops =
    { $count ->
        [one] { $count } série ocultada, e a sincronização dela para
       *[other] { $count } séries ocultadas, e a sincronização delas para
    }
library-unhidden =
    { $count ->
        [one] { $count } série de volta, sincronizando de novo
       *[other] { $count } séries de volta, sincronizando de novo
    }
library-undo = Desfazer
library-clearing-score =
    { $count ->
        [one] Apagando a nota de { $count } série
       *[other] Apagando a nota de { $count } séries
    }
library-rating =
    { $count ->
        [one] Dando nota { $score } a { $count } série
       *[other] Dando nota { $score } a { $count } séries
    }
library-moving =
    { $count ->
        [one] Movendo { $count } série para { $list }
       *[other] Movendo { $count } séries para { $list }
    }
library-rescanning =
    { $count ->
        [one] Verificando { $count } série de novo
       *[other] Verificando { $count } séries de novo
    }
library-finding-matches =
    { $count ->
        [one] Buscando vínculo para { $count } série
       *[other] Buscando vínculo para { $count } séries
    }
library-sort-last-viewed = Vistos recentemente
library-sort-downloaded = Baixados recentemente
library-sort-title = Título
library-sort-progress = Progresso
library-sort-average-score = Nota média
library-sort-my-score = Minha nota
library-dir-a-to-z = A a Z
library-dir-z-to-a = Z a A
library-dir-oldest = Mais antigos
library-dir-newest = Mais novos
library-dir-least-watched = Menos assistidos
library-dir-most-watched = Mais assistidos
library-dir-lowest = Menor primeiro
library-dir-highest = Maior primeiro
library-group-none = Nada
library-group-season = Temporada
library-group-type = Tipo
library-group-genre = Gênero
library-group-watch-state = Status
library-scan-started = Verificação iniciada
library-tab-all = Tudo
library-tab-series = Séries
library-tab-movies = Filmes
library-tab-watched = Assistidos
library-tab-hidden = Ocultos
library-count-movies =
    { $count ->
        [one] { $count } filme
       *[other] { $count } filmes
    }
library-count-watched = { $count } assistidos
library-count-hidden = { $count } ocultos
library-count-series =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    }
library-title = Biblioteca
library-all-genres = Todos os gêneros
library-search-placeholder = Buscar títulos ou pastas
library-sort = Ordenar
library-group = Agrupar
library-franchises-together = Franquias juntas
library-watched-shows = Séries assistidas
library-load-failed-title = A biblioteca não carregou
library-no-matches = Nada encontrado para "{ $query }"
library-none-in-genre =
    { $what ->
        [series] Nenhuma série de { $genre } aqui
        [films] Nenhum filme de { $genre } aqui
        [watched] Nenhuma série assistida de { $genre } aqui
        [hidden] Nenhuma série oculta de { $genre } aqui
       *[shows] Nenhuma série de { $genre } aqui
    }
library-none-here =
    { $what ->
        [series] Nenhuma série aqui
        [films] Nenhum filme aqui
        [watched] Nenhuma série assistida aqui
        [hidden] Nenhuma série oculta aqui
       *[shows] Nenhuma série aqui
    }
library-load-failed-body = O AniBeam não conseguiu ler a biblioteca agora. Tente de novo em instantes.
library-no-matches-body = Confira a grafia, ou busque na biblioteca inteira em Tudo.
library-none-in-genre-body = Nada nesta visualização tem esse gênero. Todos os gêneros mostra o resto.
library-none-in-tab-body = Sua biblioteca ainda não tem nenhum destes. Tudo mostra o que ela tem.
library-all-filtered-body = Tudo na sua biblioteca está filtrado desta visualização.
library-clear-search = Limpar busca
library-show-all = Mostrar tudo
library-picked = { $count } escolhidos
library-pick-all = Todos
library-pick-all-tip = Escolher todos os cards mostrados (Ctrl+A)
library-unhide = Mostrar
library-hide = Ocultar
library-rate = Avaliar
library-rate-tip = Uma nota para cada escolha, em cada tracker conectado
library-needs-tracker = Precisa de um tracker conectado e de uma série vinculada
library-move-to-list = Mover para lista
library-rescan = Verificar de novo
library-find-match = Buscar vínculo
library-find-match-tip = Buscar cada escolha sem vínculo
library-all-matched = Todas as escolhas têm vínculo
library-empty-scanned-title = Nada encontrado nas suas pastas ainda
library-empty-title = Sua biblioteca está vazia
library-empty-scanned-body = O AniBeam não achou séries nem filmes nas pastas que observa. Verifique de novo, ou confira as pastas nas Configurações.
library-empty-body = Inclua a pasta onde ficam seus animes e a biblioteca se monta sozinha, ou importe uma exportação do AniBeam.
library-scan-all = Verificar tudo
library-add-folder = Incluir pasta
library-settings = Config. da biblioteca
library-import = Importar
library-import-title = Importar uma exportação do AniBeam
library-import-filter-export = Exportação do AniBeam (*.json)
library-import-filter-all = Todos os arquivos (*)
library-import-started = Importação iniciada

## MetadataPage.qml

metadata-title = Info da série
metadata-filter-all = Tudo { $count }
metadata-filter-series = Séries { $count }
metadata-filter-films = Filmes { $count }
metadata-filter-missing = Sumidos { $count }
metadata-scan-started = Verificação iniciada
metadata-sort-added = Incluídos
metadata-sort-season = Temporada
metadata-sort-a-to-z = A a Z
metadata-section-unmatched = Não reconhecidas
metadata-section-missing = Pasta não encontrada
metadata-section-incomplete = Episódios faltando
metadata-section-empty = Não baixadas
metadata-section-airing = No ar
metadata-section-complete = Completas
metadata-no-info = Sem info da série ainda
metadata-state-aired = { $have } de { $aired } exibidos
metadata-state-of = { $have } de { $expected }
metadata-state-of-extra = { $have } de { $expected } +{ $extra }
metadata-detail-none-aired = Nenhum dos { $aired } episódios exibidos está no disco
metadata-detail-aired = { $have } de { $aired } exibidos no disco

metadata-detail-aired-missing = { $have } de { $aired } exibidos no disco, { $missing } faltando
metadata-detail-none = Nenhum dos { $expected } episódios está no disco
metadata-detail-missing = { $have } de { $expected } no disco, { $missing } faltando
metadata-walk-position = { $at } de { $total }
metadata-walk-done = Essa foi a última
metadata-walk-left =
    { $count ->
        [one] Falta uma em Precisa de vínculo
       *[other] Faltam { $count } em Precisa de vínculo
    }
metadata-countdown-days = { $days } d { $hours } h
metadata-countdown-hours = { $hours } h { $minutes } min
metadata-countdown-minutes = { $minutes } min
metadata-next-in = Próximo em { $wait }
metadata-files-of = { $have } de { $expected }
metadata-files-on-disk = Arquivos no disco
metadata-forgot = { $title } removida
metadata-refreshed-failed = { $refreshed } atualizadas, { $failed } não puderam ser atualizadas
metadata-refreshed =
    { $count ->
        [one] { $count } série atualizada
       *[other] { $count } séries atualizadas
    }
metadata-found =
    { $count ->
        [one] { $count } série encontrada
       *[other] { $count } séries encontradas
    }
metadata-found-some-left =
    { $count ->
        [one] { $count } série encontrada, { $unmatched } ainda não reconhecidas
       *[other] { $count } séries encontradas, { $unmatched } ainda não reconhecidas
    }
metadata-found-nothing = Nada novo encontrado, { $unmatched } ainda não reconhecidas
metadata-all-recognised = Todas as séries foram reconhecidas
metadata-count =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    }
metadata-search-placeholder = Filtrar títulos
metadata-find-rest = Buscar o resto
metadata-find-rest-started = Buscando as pastas que o AniBeam ainda não conhece
metadata-find-rest-tip = Busca cada pasta que o AniBeam ainda não conhece, em cada site de info na ordem das Configurações
metadata-refresh-all = Atualizar tudo
metadata-refresh-all-started = Atualizando a info de todas as séries
metadata-refresh-all-tip = Baixa de novo a info de cada série reconhecida
metadata-crawling = Buscando ligações da franquia
metadata-crawl-progress = { $done } de { $total }
metadata-load-failed-title = A info das séries não carregou
metadata-empty-scanned-title = Nada encontrado nas suas pastas ainda
metadata-no-matches = Nada encontrado para "{ $query }"
metadata-nothing-under =
    { $filter ->
        [Series] Nada em Séries
        [Movies] Nada em Filmes
        [MissingFiles] Nada em Sumidos
       *[other] Nada em { $filter }
    }
metadata-load-failed-body = O AniBeam não conseguiu ler a biblioteca agora. Tente de novo em instantes.
metadata-empty-scanned-body = O AniBeam não achou séries nas pastas que observa. Verifique de novo, ou confira as pastas nas Configurações.
metadata-empty-body = Sua biblioteca está vazia. Inclua a pasta onde ficam seus animes e o AniBeam a verifica.
metadata-no-matches-body = Nenhum título nesta visualização tem essas palavras.
metadata-nothing-under-body = Nenhuma das suas séries se encaixa neste filtro agora. Tudo mostra o resto.
metadata-scan-all = Verificar tudo
metadata-add-folder = Incluir pasta
metadata-clear-search = Limpar busca
metadata-show-all = Mostrar tudo
metadata-refreshing = Atualizando { $title }
metadata-refresh-tip = Baixar de novo a info desta série
metadata-forget-armed = Remover
metadata-forget-confirm-tip = Clique de novo para remover esta série e o histórico dela do AniBeam
metadata-forget-tip = A pasta não está mais no disco: remover esta série e o histórico dela do AniBeam

## AiringPage.qml

airing-title = No ar agora
airing-sort-popular = Popularidade
airing-sort-score = Nota
airing-sort-trending = Em alta
airing-sort-favourites = Favoritos
airing-sort-soonest = Próximo episódio
airing-sort-aired = Episódios exibidos
airing-sort-episodes = Nº de episódios
airing-sort-start = Data de estreia
airing-sort-duration = Duração do episódio
airing-sort-title = Título
airing-dir-most-popular = Mais populares
airing-dir-least-popular = Menos populares
airing-dir-lowest = Menor primeiro
airing-dir-highest = Maior primeiro
airing-dir-least-trending = Menos em alta
airing-dir-most-trending = Mais em alta
airing-dir-fewest = Menos primeiro
airing-dir-most = Mais primeiro
airing-dir-soonest = Mais próximos
airing-dir-latest = Mais tardios
airing-dir-oldest = Mais antigos
airing-dir-newest = Mais novos
airing-dir-shortest = Mais curtos
airing-dir-longest = Mais longos
airing-dir-a-to-z = A a Z
airing-dir-z-to-a = Z a A
airing-season =
    { $season ->
        [Winter] Inverno { $year }
        [Spring] Primavera { $year }
        [Summer] Verão { $year }
        [Fall] Outono { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = há { $minutes } min
airing-ago-hours = há { $hours } h
airing-ago-days = há { $days } d
airing-episode-out = { $episode } lançado { $ago }
airing-no-date = Sem data ainda
airing-in-library = Na biblioteca
airing-out-today =
    { $count ->
        [one] { $count } episódio lançado hoje
       *[other] { $count } episódios lançados hoje
    }
airing-dismissed-one = { $title } não aparece mais aqui
airing-dismissed =
    { $count ->
        [one] { $count } série não aparece mais aqui
       *[other] { $count } séries não aparecem mais aqui
    }
airing-planned-already-one = { $title } já está na sua lista
airing-planned-already-all = Todas já estão na sua lista
airing-planned =
    { $count ->
        [one] { $count } série vai para a sua lista Planejando
       *[other] { $count } séries vão para a sua lista Planejando
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } série vai para a sua lista Planejando, { $already } já estavam lá
       *[other] { $count } séries vão para a sua lista Planejando, { $already } já estavam lá
    }
airing-play-next = Assistir próximo
airing-plan-to-watch = Quero assistir
airing-get-more-episodes = Baixar mais episódios
airing-get-episodes = Baixar episódios
airing-open-on-anilist = Abrir no AniList
airing-not-interested = Sem interesse
airing-not-interested-count = { $count } sem interesse
airing-not-interested-tip = Séries que você marcou como sem interesse, nas Configurações
airing-refreshing = Atualizando
airing-pick-season = Escolher outra temporada
airing-filter-all = Tudo
airing-filter-lists = Nas minhas listas
airing-filter-library = Na minha biblioteca
airing-hide-lists = Ocultar minhas listas
airing-hide-lists-tip = Deixar de fora as séries das suas listas no AniList ou no MAL
airing-hide-library = Ocultar minha biblioteca
airing-hide-library-tip = Deixar de fora as séries de que você tem episódios
airing-out-now = Já saiu
airing-out-last-day =
    { $count ->
        [one] { $count } episódio nas últimas 24 horas
       *[other] { $count } episódios nas últimas 24 horas
    }
airing-out-fewer = Menos
airing-out-all = Todos os { $count }
airing-sort = Ordenar
airing-load-failed-title = Não foi possível acessar o AniList
airing-load-failed-body = As séries no ar vêm do AniList. Elas voltam na próxima atualização.
airing-reading-title = Lendo o que está no ar
airing-nothing-now = Nada no ar agora
airing-nothing-for = Nada no ar em { $season }
airing-nothing-read = Nada lido ainda
airing-reading-body = Todas as séries no ar, do AniList. Na primeira vez leva alguns segundos.
airing-nothing-body = O AniList ainda não lista séries para esta temporada. Atualizar lê de novo, ou escolha outra temporada.
airing-nothing-read-body = Atualizar lê do AniList todas as séries no ar.
airing-hides-empty-title = Seus filtros não deixam nada aqui
airing-lists-empty-title = Nada aqui está nas suas listas
airing-library-empty-title = Nada aqui está na sua biblioteca
airing-hides-empty-body = Toda série aqui está nas suas listas ou na sua biblioteca, e você ocultou essas.
airing-lists-empty-body = Quero assistir, em um card, coloca a série na sua lista.
airing-library-empty-body = Baixar episódios, em um card, baixa a série para a sua biblioteca.
airing-show-everything = Mostrar tudo
airing-picked = { $count } escolhidas
airing-undo = Desfazer

## EpisodeRow.qml

episode-row-date-time-format = ddd, d MMM, HH:mm
episode-row-date-upcoming-format = ddd, d MMM
episode-row-date-format = d MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Clique para marcar como não assistido a partir daqui
episode-row-mark-watched-tip = { $state }. Clique para marcar como assistido até aqui
episode-row-next-up = A seguir
episode-row-versions =
    { $count ->
        [one] { $count } versão
       *[other] { $count } versões
    }
episode-row-versions-tip =
    { $count ->
        [one] O mesmo episódio de { $count } release. Clique para escolher qual toca
       *[other] O mesmo episódio de { $count } releases. Clique para escolher qual toca
    }
episode-row-downloading = Baixando { $percent }%
episode-row-get = Baixar episódio
episode-row-predicted = Previsto
episode-row-extra = Extra
episode-row-resume = Retomar em { $time }

## EpisodeCubes.qml

episode-cubes-one = Um episódio
episode-cubes-range =
    { $count ->
        [one] { $count } episódio, { $first } a { $last }
       *[other] { $count } episódios, { $first } a { $last }
    }
episode-cubes-tip = Episódio { $number }: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } extra
       *[other] { $count } extras
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } arquivo sem vínculo
       *[other] { $count } arquivos sem vínculo
    }
episodes-pane-on-disk = No disco
episodes-pane-not-on-disk = Fora do disco
episodes-pane-season = T{ $season }
episodes-pane-get-tip = Baixar episódio { $number }
episodes-pane-none-on-disk = Nenhum episódio no disco
episodes-pane-change-match = Mudar vínculo
episodes-pane-refreshing = Atualizando { $title }

## ShowDetail.qml

show-detail-not-in-library = Esta pasta ainda não está na biblioteca. Quando uma verificação a encontrar, o pôster e os episódios aparecem aqui.
show-detail-film-on-disk = O filme está no disco.
show-detail-film-not-on-disk = O filme ainda não está no disco.
show-detail-complete-line = A exibição terminou, e todos os { $total } episódios estão aqui. Não vem mais nada, então seguir a série não tem mais o que fazer.
show-detail-finished = Exibição concluída.
show-detail-airing = No ar.
show-detail-have-of-total = { $have } de { $total } episódios aqui.
show-detail-have =
    { $count ->
        [one] { $count } episódio aqui.
       *[other] { $count } episódios aqui.
    }
show-detail-next-predicted =
    { $count ->
        [one] { $span } está previsto em { $wait }.
       *[other] { $span } estão previstos em { $wait }.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span } vai ao ar em { $wait }.
       *[other] { $span } vão ao ar em { $wait }.
    }
show-detail-missing =
    { $count ->
        [one] O episódio { $list } não está aqui.
       *[other] Os episódios { $list } não estão aqui.
    }
show-detail-missing-more = Os episódios { $list } e mais { $more } não estão aqui.
show-detail-downloaded = Baixado
show-detail-downloading = Baixando { $percent }%
show-detail-taken-adding = Pego pela regra, incluindo
show-detail-taken = Pego pela regra
show-detail-left-out = Deixado de fora pela regra
show-detail-subscribed-reading = Seguindo. O qBittorrent está lendo o feed.
show-detail-taken-moving = A regra pega { $taken } { $releases ->
        [one] do { $releases } release
       *[other] dos { $releases } releases
    } do feed; { $moving ->
        [one] { $moving } está
       *[other] { $moving } estão
    } baixando abaixo.
show-detail-taken-held = A regra pega { $taken } { $releases ->
        [one] do { $releases } release
       *[other] dos { $releases } releases
    } do feed; { $torrents ->
        [one] { $torrents } torrent está
       *[other] { $torrents } torrents estão
    } no cliente.
show-detail-taken-adding-line = A regra pega { $taken } { $releases ->
        [one] do { $releases } release
       *[other] dos { $releases } releases
    } do feed; o qBittorrent está { $taken ->
        [one] incluindo-o.
       *[other] incluindo-os.
    }
show-detail-feed-holds-nothing = O feed ainda não tem nada. Os releases que se encaixam baixam assim que aparecem nele.
show-detail-nothing-fits = Nada no feed se encaixa na regra ainda. Os releases que se encaixam baixam assim que aparecem.
show-detail-complete = Completa
show-detail-open-in-library = Abrir na biblioteca
show-detail-feed-finds-nothing = Este feed não encontra nada
show-detail-feed-empty-query = Ele pede a { $source } por “{ $query }”, que volta vazio, então nada novo chega à regra. Uma busca mais curta costuma funcionar; as palavras que ela deixa de fora podem ficar na regra.
show-detail-feed-empty = Ele pergunta a { $source }, que volta vazio, então nada novo chega à regra. Uma busca mais curta costuma funcionar; as palavras que ela deixa de fora podem ficar na regra.
show-detail-feed-empty-earlier = O qBittorrent ainda mostra o que uma versão anterior deste feed encontrou.
show-detail-repairing = Consertando
show-detail-repair = Consertar
show-detail-unsubscribe = Deixar de seguir
show-detail-remove-torrents =
    { $count ->
        [one] Remover o torrent
       *[other] Remover { $count } torrents
    }
show-detail-searches = Busca
show-detail-quoted = “{ $words }”
show-detail-whole-feed = o feed inteiro
show-detail-from = De
show-detail-rule = Regra
show-detail-rule-unfollowable = Não baixa novos episódios: a fonte responde a buscas mas não tem feed
show-detail-rule-active = Ativa
show-detail-keeps = Mantém
show-detail-keeps-value = releases com “{ $words }”
show-detail-drops = Descarta
show-detail-drops-value = releases com { $words }
show-detail-or = { $first } ou { $second }
show-detail-last-took = Último pego
show-detail-saves-to = Salva em
show-detail-folder = Pasta
show-detail-in-the-feed = No feed
show-detail-feed-reading = Lendo o feed.
show-detail-feed-offers = O que o feed oferece agora, do mais novo.
show-detail-feed-is-empty = O feed está vazio.
show-detail-read-again = Ler de novo
show-detail-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
show-detail-download = Baixar
show-detail-torrents = Torrents
show-detail-torrents-moving =
    { $count ->
        [one] { $count } baixando agora; as barras enchem conforme avança.
       *[other] { $count } baixando agora; as barras enchem conforme avançam.
    }
show-detail-torrents-held = O que o cliente tem para esta pasta.
show-detail-torrents-waiting = Esperando o qBittorrent incluir o que a regra pega.
show-detail-torrents-none = O cliente não tem nada para esta pasta.

## SeriesPage.qml

series-trackers-both = { $first } e { $second }
series-count-cleared = Contagem de episódios apagada
series-count-set = Definida para { $count } episódios
series-held-one-undated = Episódio { $first } ainda sem data
series-held-range-undated = Episódios { $first } a { $last } ainda sem data
series-held-one-due = Episódio { $first } previsto em { $when }
series-held-range-due = Episódios { $first } a { $last } previstos em { $when }
series-held-month-format = MMMM 'de' yyyy
series-fix-schedule = Episódios e datas
series-fix-files = Arquivos
series-fix-rescan-folder = Verificar pasta de novo
series-fix-match = Vínculo
series-find-match = Buscar vínculo
series-rescan-show = Verificar série de novo
series-open-in-show-info = Abrir em Info da série
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } filme movido para a lixeira
           *[other] { $count } filmes movidos para a lixeira
        }
        [episode] { $count ->
            [one] { $count } episódio movido para a lixeira
           *[other] { $count } episódios movidos para a lixeira
        }
       *[file] { $count ->
            [one] { $count } arquivo movido para a lixeira
           *[other] { $count } arquivos movidos para a lixeira
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } filme escolhido.
           *[other] { $count } filmes escolhidos.
        }
        [episode] { $count ->
            [one] { $count } episódio escolhido.
           *[other] { $count } episódios escolhidos.
        }
       *[file] { $count ->
            [one] { $count } arquivo escolhido.
           *[other] { $count } arquivos escolhidos.
        }
    }

series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] Mover { $count } filme para a lixeira
           *[other] Mover { $count } filmes para a lixeira
        }
        [episode] { $count ->
            [one] Mover { $count } episódio para a lixeira
           *[other] Mover { $count } episódios para a lixeira
        }
       *[file] { $count ->
            [one] Mover { $count } arquivo para a lixeira
           *[other] Mover { $count } arquivos para a lixeira
        }
    }
series-delete-click = Clique nos arquivos para excluir.
series-delete-where = Eles vão para a lixeira, de onde dá para recuperá-los. O que você assistiu continua registrado.
series-delete-move = Mover para a lixeira
series-pick-none = Nenhum
series-pick-all = Todos
series-rescan-started = Verificação iniciada
series-rated = Nota { $score }
series-rated-cleared = Nota apagada
series-score-failed = A nota falhou
series-still-downloading = { $episode } ainda está baixando
series-format-series = Série
series-tracker-refused = O { $tracker } não aceitou
series-tracker-refused-why = O { $tracker } não aceitou: { $why }
series-watched-up-to = Assistido até { $episode }
series-unwatched-from = Não assistido a partir de { $episode }
series-undo = Desfazer
series-play-downloading = Baixando episódio { $number }
series-play-downloading-unnumbered = Baixando episódio
series-play-continue = Continuar
series-play-again = Assistir de novo
series-play-continue-episode = Continuar episódio { $number }
series-play-again-from = Rever do episódio { $number }
series-play-episode = Assistir episódio { $number }
series-page-not-tracked = Não registrada
series-not-tracked-on-disk =
    { $count ->
        [one] Não registrada, { $count } episódio no disco
       *[other] Não registrada, { $count } episódios no disco
    }
series-watched = { $count } assistidos
series-watched-of = { $watched } de { $total } assistidos
series-watched-of-estimate = { $watched } de { $total }+ assistidos
series-facts-episodes-estimate =
    { $count ->
        [one] { $count } episódio+
       *[other] { $count } episódios+
    }
series-facts-each = { $runtime } cada
series-facts-on-disk = { $count } no disco
series-next = Próximo: { $episode }
series-episodes-downloading =
    { $count ->
        [one] { $count } episódio baixando
       *[other] { $count } episódios baixando
    }
series-all-watched = Tudo assistido
series-no-poster = Sem pôster
series-next-season-span = T{ $season } { $span }
series-next-predicted = { $span } previsto para { $date }, em { $wait }
series-next-date-format = ddd, d MMM
series-next-airs =
    { $count ->
        [one] { $span } vai ao ar em { $wait }
       *[other] { $span } vão ao ar em { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] Baixando { $count } episódio
       *[other] Baixando { $count } episódios
    }
series-count-season-ends = A temporada { $season } termina no episódio { $number }
series-count-ends = Termina no episódio { $number }
series-count-gave-way = Você definiu { $count } episódios; isso não vale mais
series-count-set-by-you = { $line }, definido por você
series-change = Mudar
series-get-episodes = Baixar episódios
series-your-score = Sua nota { $score }
series-rate = Avaliar
series-page-average-score = Nota média { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = { $votes } votos
series-add-to-list = Incluir em uma lista
series-page-rewatched =
    { $count ->
        [one] Reassistido { $count } vez
       *[other] Reassistido { $count } vezes
    }
series-your-notes = Suas notas
series-add-notes = Incluir notas
series-notes-tip = Notas na entrada da sua lista, guardadas no { $sites }
series-unhide = Mostrar
series-hide = Ocultar
series-hidden-toast = { $title } está oculta aqui, e o AniBeam para de sincronizá-la com o AniList e o MyAnimeList
series-unhidden-toast = { $title } voltou e sincroniza de novo
series-hide-tip-tracked = Oculta de todas as páginas aqui, e o AniBeam para de sincronizá-la com o AniList e o MyAnimeList
series-hide-tip = Oculta de todas as listas
series-fixes = Correções
series-fixes-tip = Corrigir arquivos, a contagem de episódios, o vínculo, ou verificar de novo
series-quick-fix = Correção rápida
series-film = Filme
series-episodes = Episódios
series-extras = Aberturas, encerramentos e mais
series-extras-openings = Aberturas
series-extras-endings = Encerramentos
series-extras-previews = Prévias e trailers
series-extras-specials = Especiais
series-extras-other = Outros
series-extras-group = { $kind }, { $count }
series-extra-files = Arquivos a mais
series-extra-files-warning =
    { $count ->
        [one] Um arquivo passa dos { $total } episódios esperados: nome errado, duplicado ou especial. Renomeie ou remova.
       *[other] { $count } arquivos passam dos { $total } episódios esperados: nomes errados, duplicados ou especiais. Renomeie ou remova.
    }
series-synopsis = Sinopse
series-less = Menos
series-more = Mais
series-match = Vínculo
series-not-matched = Ainda sem vínculo, então não há pôster, sinopse nem registro. Encontre a info da série no AniList ou no TVmaze.
series-naming-title = Nomes que funcionam
series-naming-body = A pasta é o título da série, do jeito que o AniList escreve, com a temporada depois quando não é a primeira. Cada arquivo traz o número do episódio.
series-fix-how-files-read = Corrigir a leitura dos arquivos
series-characters = Personagens
series-related = Relacionados
series-related-filling = Ainda carregando do AniList.
series-tags = Tags
series-hide-spoilers = Ocultar spoilers
series-spoilers = Spoilers
series-all-tags = Todas as tags
series-tag = { $name }, { $rank }%
series-recommendations = Recomendações
series-relation-owned = Na sua biblioteca
series-relation-unreleased = Ainda não lançado
series-relation-on-anilist = No AniList
series-score-saved = Nota salva
series-score-cleared = Nota apagada

## GetShow.qml

get-show-the-client = o cliente de torrent
get-show-group-batch = lote de { $group }
get-show-run = { $first } a { $last }
get-show-episodes =
    { $count ->
        [one] Episódio { $list }
       *[other] Episódios { $list }
    }
get-show-episodes-inline =
    { $count ->
        [one] episódio { $list }
       *[other] episódios { $list }
    }
get-show-sending-to = Enviando para { $client }
get-show-go =
    { $count ->
        [one] Baixar { $count } episódio
       *[other] Baixar { $count } episódios
    }
get-show-go-replace =
    { $count ->
        [one] Baixar { $count } episódio e substituir
       *[other] Baixar { $count } episódios e substituir
    }
get-show-go-subscribe =
    { $count ->
        [one] Baixar { $count } episódio e seguir
       *[other] Baixar { $count } episódios e seguir
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] Baixar { $count } episódio, substituir e seguir
       *[other] Baixar { $count } episódios, substituir e seguir
    }
get-show-subscribe = Seguir para novos episódios
get-show-nothing-new = Nada novo para baixar
get-show-pick-episode = Escolha um episódio
get-show-replace-off = Desativado. As escolhas para { $episodes } ficam de fora, porque a biblioteca já as tem. Ative para trocá-las.
get-show-replace-on = Os arquivos que você tem para { $episodes } vão para a lixeira quando os novos terminarem.
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } está baixando agora: esse download é cancelado e o que ele gravou vai para a lixeira.
       *[other] { $episodes } estão baixando agora: esse download é cancelado e o que ele gravou vai para a lixeira.
    }
get-show-kind-on-demand = Inicia quando uma busca precisa
get-show-kind-rss-search = Busca RSS
get-show-kind-rss-feed = Feed RSS
get-show-kind-json-search = Busca JSON
get-show-kind-indexer-search = Busca em indexador
get-show-ready = Pronta
get-show-running = Rodando
get-show-not-running = Parada
get-show-not-running-start = Parada. Clique na fonte acima para iniciá-la.
get-show-switched-off = Desativada
get-show-click-to-see = Clique para ver o que ela tem desta série
get-show-searching = Buscando
get-show-searching-source = Buscando em { $name }
get-show-searching-elapsed = { $label }, { $seconds } s
get-show-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } releases
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } release, { $cover } de { $total } episódios cobertos
       *[other] { $count } releases, { $cover } de { $total } episódios cobertos
    }
get-show-on-your-list = Na sua lista
get-show-plan-to-watch = Quero assistir
get-show-on-your-list-tip = Está na sua lista
get-show-plan-to-watch-tip = Colocar na sua lista Planejando
get-show-open-series-tip = A página da série: detalhes, episódios e reprodução
get-show-next-in = Episódio { $number } em { $wait }
get-show-airing = No ar
get-show-finished = Concluída
get-show-total-episodes = { $count } episódios
get-show-downloading-episodes =
    { $count ->
        [one] Baixando { $count } episódio
       *[other] Baixando { $count } episódios
    }
get-show-downloading-torrents =
    { $count ->
        [one] Baixando { $count } torrent
       *[other] Baixando { $count } torrents
    }
get-show-in-library = Na sua biblioteca
get-show-not-in-library = Fora da sua biblioteca
get-show-downloading = Baixando
get-show-stop-all = Parar tudo
get-show-start-all = Iniciar tudo
get-show-stop-all-tip = Parar todos os downloads desta série; o que já veio fica
get-show-start-all-tip = Retomar todos os downloads desta série
get-show-cancel-all = Cancelar tudo
get-show-cancel-all-tip = Cancelar todos os downloads desta série e mover o que gravaram para a lixeira
get-show-sources = Fontes disponíveis
get-show-check-again = Verificar de novo
get-show-check-again-tip = Pergunta a cada provedor sob demanda se ele está rodando. Não inicia nada.
get-show-search-source = Buscar em { $name }
get-show-release-list = Todos os releases em lista
get-show-your-library = Sua biblioteca
get-show-library-on-disk = { $have } de { $total } no disco
get-show-library-downloading = { $count } baixando
get-show-library-nothing = Nada ainda
get-show-row-batch = { $group }, lote
get-show-batch-counting = contando os episódios, um torrent
get-show-batch-unlisted = episódios não listados, a série inteira pelo título, um torrent
get-show-batch-episodes =
    { $count ->
        [one] { $count } episódio, um torrent
       *[other] { $count } episódios, um torrent
    }
get-show-nothing-new-picked = Nada novo escolhido
get-show-nothing-picked = Nada escolhido
get-show-pick-provider = Escolha um provedor
get-show-plan-line = { $episodes } de { $groups } em { $source }.
get-show-and = { $first } e { $second }
get-show-library-has =
    { $count ->
        [one] A biblioteca tem { $episodes }. Ative Substituir episódios atuais abaixo para trocá-lo.
       *[other] A biblioteca tem { $episodes }. Ative Substituir episódios atuais abaixo para trocá-los.
    }
get-show-together = juntos
get-show-torrents-unit =
    { $count ->
        [one] torrent
       *[other] torrents
    }
get-show-replace = Substituir episódios atuais
get-show-sequential = Baixar em ordem
get-show-sequential-on = Ativado: cada novo torrent baixa as partes do início, então os primeiros episódios terminam primeiro e você pode assistir enquanto o resto chega. Um pouco mais lento no total.
get-show-sequential-off = Desativado: o cliente baixa as partes na ordem que o swarm oferece.
get-show-rolling = Manter os próximos prontos
get-show-rolling-on = Ativado: { $count } episódios baixam agora, e cada um que você termina inicia o próximo.
get-show-rolling-on-delete = Ativado: { $count } episódios baixam agora, e cada um que você termina inicia o próximo. Os episódios assistidos saem do disco.
get-show-rolling-off = Desativado: todos os episódios escolhidos baixam de uma vez.
get-show-follow-off = Desativado: novos episódios aparecem aqui como faltas, e nada baixa sozinho.
get-show-follow-no-feed = { $source } não tem feed para seguir, então novos episódios não baixam sozinhos dela.
get-show-follow-pick-group = Escolha um grupo primeiro: seguir acompanha um grupo e uma qualidade.
get-show-follow-batch-only = { $group } só tem o lote aqui, então não há feed de episódios avulsos para seguir.
get-show-follow-on = Ativado: um feed para { $feed } em { $source }. Cada episódio novo baixa assim que aparece.
get-show-confirm =
    { $count ->
        [one] { $episodes } está baixando agora. Substituí-lo cancela esse download e move o que ele gravou para a lixeira.
       *[other] { $episodes } estão baixando agora. Substituí-los cancela esse download e move o que ele gravou para a lixeira.
    }
get-show-confirm-go = Cancelar e substituir
get-show-state-error = Erro
get-show-state-files-missing = Arquivos faltando
get-show-state-stopped = Parado
get-show-state-waiting-peers = Esperando peers
get-show-state-metadata = Baixando metadados
get-show-state-queued = Na fila
get-show-state-checking = Verificando arquivos
get-show-state-allocating = Alocando
get-show-state-moving = Movendo arquivos
get-show-state-waiting = Esperando
get-show-a-batch = Um lote
get-show-no-group = Grupo fora do nome
get-show-likely = provavelmente { $source }
get-show-tracker = tracker { $tracker }
get-show-not-known = Ainda não se sabe
get-show-bytes-of = { $done } de { $size }
get-show-speed-down = { $speed }/s down
get-show-time-left = faltam { $wait }
get-show-stopping = Parando
get-show-starting = Iniciando
get-show-stop = Parar
get-show-start-tip = Voltar a baixar
get-show-stop-tip = Parar de baixar; o que já veio fica
get-show-cancelling = Cancelando
get-show-cancel-tip = Cancelar este download e mover o que ele gravou para a lixeira
get-show-episode-order = Baixar na ordem dos episódios
get-show-episode-order-on = Ativado: as partes vêm do início de cada arquivo, então os primeiros episódios terminam primeiro.
get-show-cube-skipped = Episódio { $number }: pulado
get-show-cube-done = Episódio { $number }: pronto
get-show-cube-percent = Episódio { $number }: { $percent }%
get-show-fact-torrent = Torrent
get-show-fact-release = Release
get-show-fact-from = De
get-show-fact-saving-to = Salvando em
get-show-span-counting = Contando
get-show-span-unknown = { $runs }?
get-show-span-episodes = Episódios { $runs }
get-show-cube-no-release = Episódio { $number }: nenhum release de { $group }
get-show-seeders = { $count } seeders
get-show-cube-on-disk = já no disco, clique para substituir
get-show-cube-downloading = baixando agora, clique para substituir

## TorrentRow

torrent-row-state-downloading = Baixando
torrent-row-state-stalled = Travado
torrent-row-state-metadata = Baixando metadados
torrent-row-state-queued = Na fila
torrent-row-state-stopped = Parado
torrent-row-state-done = Pronto
torrent-row-state-seeding = Semeando
torrent-row-state-checking = Verificando
torrent-row-state-moving = Movendo
torrent-row-state-error = Erro
torrent-row-state-files-missing = Arquivos faltando
torrent-row-state-making-room = Liberando espaço
torrent-row-state-finding-peers = Procurando peers
torrent-row-state-waiting = Esperando
torrent-row-done-of-size = { $done } de { $size }
torrent-row-speed-down = { $speed }/s down
torrent-row-speed-up = { $speed }/s up
torrent-row-time-left = faltam { $time }
torrent-row-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
torrent-row-rolling = contínuo, episódios { $next } a { $through } de { $total }
torrent-row-stop = Parar

## SourceRow

source-row-unavailable-fix = Conecte o disco e depois Verificar de novo. Se a pasta mudou de lugar, remova-a e inclua o novo local.
source-row-counts = { $count } séries
source-row-counts-movies =
    { $movies ->
        [one] { $count } séries, { $movies } pasta de filmes
       *[other] { $count } séries, { $movies } pastas de filmes
    }
source-row-unavailable = Indisponível
    .tip = O AniBeam não consegue acessar esta pasta. Se ela está num disco desconectado ou não montado, conecte-o e o AniBeam a pega de novo. Se ela mudou de lugar, remova-a aqui e inclua o novo local.
source-row-rescan = Verificar de novo
source-row-remove-question = Remover { $name }, { $count } séries e o histórico delas?

## StartSourceSheet

start-source-title = Iniciar { $service } para { $name }?
start-source-busy = Iniciando { $service }. { $name } é ativada quando ele responder; isso pode levar até um minuto.
start-source-body = { $service } é o serviço que busca em { $name } para o AniBeam. Iniciá-lo o roda em segundo plano neste computador, e ele continua rodando depois que o AniBeam fecha.
start-source-starting = Iniciando

## SearchProgress

search-progress-why-helper = O AniBeam está iniciando o ajudante de busca; isso leva um ou dois segundos.
search-progress-why-starting = { $endpoint } passa por um ajudante que inicia quando uma busca precisa. A primeira busca espera por ele, até { $seconds } segundos; as seguintes pulam isso.
search-progress-why-reading = Separando os releases em grupos e conferindo com a biblioteca.
search-progress-why-ondemand = Um ajudante lê { $endpoint } do zero a cada nova busca, o que costuma levar de dez a vinte segundos.
search-progress-why-slow = { $endpoint } está lento para responder agora.
search-progress-why-usual = Isso costuma levar um ou dois segundos.
search-progress-why-and-slow = { $why } Está demorando mais que o normal; cancele para tentar outra fonte.
search-progress-searching = Buscando
search-progress-seconds = { $seconds } s
search-progress-try = Tentar { $endpoint }

## ReleaseTable

release-table-col-release = Release
release-table-col-group = Grupo
release-table-col-res = Res
release-table-col-rip = Rip
release-table-col-ep = Ep
release-table-col-size = Tamanho
release-table-col-seeders = Seeders
release-table-col-leechers = Leechers
release-table-col-downloads = Downloads
release-table-col-released = Lançado
release-table-tick-all = Marcar todos os releases
release-table-untick-all = Desmarcar todos os releases
release-table-on-disk = No disco
release-table-other-on-disk = Outro release no disco
release-table-season-batch = Lote da temporada { $season }
release-table-batch = Lote
release-table-via = via { $source }
release-table-season-episode = T{ $season } { $episode }
release-table-date-format = d 'de' MMMM 'de' yyyy, HH:mm
release-table-download = Baixar
release-table-more = e mais { $count }. Filtre a lista ou ordene para ver outros primeiro.
release-table-pick = Escolher este release

## DownloadsPage

downloads-title = Downloads
downloads-builtin-client = Cliente embutido
downloads-start-done = { $name } está rodando e ativada
downloads-disk-film = Filme
downloads-disk-watched = { $size } assistidos
downloads-disk-all-watched = tudo assistido
downloads-delete-watched-menu =
    { $count ->
        [one] Todos os episódios assistidos: { $count } episódio, { $size }
       *[other] Todos os episódios assistidos: { $count } episódios, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Todas as séries concluídas que você assistiu: { $count } série, { $size }
       *[other] Todas as séries concluídas que você assistiu: { $count } séries, { $size }
    }
downloads-titles-and = { $list } e { $last }
downloads-titles-more = { $list } e mais { $count }
downloads-delete-finished-none = Nenhuma série concluída foi assistida até o fim
downloads-delete-watched-none = Nada assistido está no disco
downloads-delete-finished-heading =
    { $count ->
        [one] Excluir { $count } série concluída, { $size }?
       *[other] Excluir { $count } séries concluídas, { $size }?
    }
downloads-delete-finished-base = { $titles }. Só séries que o provedor diz que terminaram, com todos os episódios assistidos; uma série ainda no ar, ou cujo fim é um palpite, fica. Todos os arquivos delas vão para a lixeira, extras também. Os torrents saem do { $client } também, então podem ser baixados de novo.
downloads-delete-watched-heading =
    { $count ->
        [one] Excluir { $count } episódio assistido, { $size }?
       *[other] Excluir { $count } episódios assistidos, { $size }?
    }
downloads-delete-watched-base = Todos os episódios que você assistiu, em todas as séries. Episódios ainda não assistidos e extras ficam. Os arquivos e as legendas deles vão para a lixeira, então dá para recuperá-los de lá. Os torrents saem do { $client } também, então podem ser baixados de novo.
downloads-move-to-trash = Mover para a lixeira
downloads-remove-group-torrents =
    { $count ->
        [one] Remover o torrent
       *[other] Remover os { $count } torrents
    }
downloads-run-span = { $from } a { $to }
downloads-run-pair = { $first } e { $second }
downloads-gap-missing =
    { $count ->
        [one] Episódio { $runs } faltando
       *[other] Episódios { $runs } faltando
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Episódio { $runs } não baixado
       *[other] Episódios { $runs } não baixados
    }
downloads-gap-then-not-downloaded = { $runs } não baixados
downloads-trying-query = Tentando { $query }
downloads-client-settings = Config. do cliente de torrent
downloads-error-unknown = Algo deu errado. Tente de novo.
downloads-search-all-off = Todas as fontes estão desativadas. Ative uma para buscar.
downloads-turn-on = Ativar { $name }
downloads-search-no-sources = Ainda não há fontes para buscar. Inclua uma lista de fontes nas Configurações, em Downloads.
downloads-search-none-started = Nenhuma fonte conseguiu iniciar a busca. Tente de novo em instantes.
downloads-auto-resolution = Filtrado para { $resolution }, a melhor disponível
downloads-search-one-failed = A busca em { $name } não funcionou. Tente de novo, ou busque em outra fonte.
downloads-search-all-failed = Nenhuma fonte conseguiu buscar. Tente de novo em instantes.
downloads-narrowing-on = Filtrando em { $name }
downloads-refine-zero = Isso não deixaria nada, então a lista fica como está.
downloads-refine-no-match = Nenhum release tem esse termo.
downloads-refine-empty = Digite um termo para filtrar.
downloads-refine-no-change = Isso não mudaria nada.
downloads-movie-tick-one = Marque um release e clique em Baixar como filme.
downloads-group-downloading = Baixando
downloads-group-downloading-line = Em andamento; Atividade, à direita, tem os detalhes.
downloads-group-seeding = Semeando
downloads-group-seeding-line = Baixados e ainda compartilhando.
downloads-group-done = Prontos
downloads-group-done-line = Baixados; o cliente não está compartilhando.
downloads-group-stopped = Parados
downloads-group-stopped-line = Parados antes de terminar.
downloads-group-broken = Arquivos faltando
downloads-group-broken-line = O cliente não encontra os arquivos: eles apontam para uma pasta que não existe. Remover os torrents arruma o cliente.
downloads-sub-nofeed = Não dá para seguir
downloads-sub-nofeed-line = A fonte responde a buscas mas não tem feed, então não chegam novos episódios. Deixe de seguir e siga a série em uma fonte que tenha.
downloads-sub-complete = Completas
downloads-sub-complete-line = Todos os episódios estão na biblioteca e nada mais vai ao ar: seguir estas séries não tem mais o que fazer.
downloads-sub-airing = No ar
downloads-sub-airing-line = Novos episódios ainda por vir, os mais próximos primeiro.
downloads-sub-incomplete = Incompletas
downloads-sub-incomplete-line = A exibição terminou com episódios fora daqui, ou nada diz em que pé a série está.
downloads-sub-missing = Fora da biblioteca
downloads-sub-missing-line = A pasta delas não tem nada que a biblioteca tenha encontrado: nada chegou ainda, ou elas salvam onde a biblioteca não olha.
downloads-torrents =
    { $count ->
        [one] { $count } torrent
       *[other] { $count } torrents
    }
downloads-rule-film-on-disk = Filme, no disco
downloads-rule-all-here = Todos os { $total } aqui
downloads-rule-next =
    { $predicted ->
        [yes] Ep { $episode } previsto em { $time }
       *[no] Ep { $episode } em { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] Ep { $episode } a { $through } previstos em { $time }
       *[no] Ep { $episode } a { $through } em { $time }
    }
downloads-rule-have-of = { $have } de { $total } aqui
downloads-rule-have =
    { $count ->
        [one] { $count } episódio aqui
       *[other] { $count } episódios aqui
    }
downloads-tag-not-downloaded = Não baixado
downloads-tag-missing = { $count } faltando
downloads-tag-n-not-downloaded = { $count } não baixados
downloads-tag-unknown = Desconhecido
downloads-speed-down = { $speed }/s down
downloads-time-left = faltam { $time }
downloads-repair-trying = Tentando buscas mais curtas
downloads-sent-to-client = { $title } enviado para o { $client }
downloads-files-torrents = o que os torrents baixaram
downloads-files-torrents-remote = o que os torrents baixaram, excluído pelo { $client }
downloads-files-folder-and = { $folder } e { $torrents }
downloads-unsubscribe-heading = Deixar de seguir { $title }?
downloads-unsubscribe-base = O feed e a regra de download saem do { $client }, então nada novo chega.
downloads-unsubscribe-complete-heading = Deixar de seguir as { $count } séries completas?
downloads-unsubscribe-complete-base = Os feeds e as regras de download delas saem do { $client }. Todos os episódios delas estão na biblioteca.
downloads-files-folders-and-torrents = as pastas delas e o que os torrents baixaram
downloads-one-at-a-time = Elas saem uma de cada vez; a página mostra cada uma conforme sai.
downloads-subscription-stays = { $base } Você continua seguindo a série.
downloads-cancel-heading = Cancelar { $title }?
downloads-cancel-group-base =
    { $count ->
        [one] O download para e sai do { $client }.
       *[other] Os { $count } downloads param e saem do { $client }.
    }
downloads-files-so-far-it = o que ele baixou até agora
downloads-files-so-far-they = o que eles baixaram até agora
downloads-cancel-download = Cancelar o download
downloads-keep-downloading = Continuar baixando
downloads-remove-heading = Remover { $title }?
downloads-remove-group-base =
    { $count ->
        [one] O torrent sai do { $client }; os arquivos ficam, a não ser que você marque abaixo.
       *[other] Os { $count } torrents saem do { $client }; os arquivos ficam, a não ser que você marque abaixo.
    }
downloads-remove-detail-base =
    { $count ->
        [one] O { $count } torrent sai do { $client }; os arquivos ficam, a não ser que você marque abaixo.
       *[other] Os { $count } torrents saem do { $client }; os arquivos ficam, a não ser que você marque abaixo.
    }
downloads-remove-broken-heading = Remover os torrents das { $count } séries com arquivos faltando?
downloads-remove-broken-base = Os { $count } torrents saem do { $client }. Eles apontam para pastas que não existem, então nenhum arquivo é tocado; as séries seguidas continuam.
downloads-remove-torrent-heading = Remover este torrent?
downloads-files-it-downloaded = o que ele baixou
downloads-cancel-torrent-heading = Cancelar este download?
downloads-cancel-many-heading =
    { $count ->
        [one] Cancelar { $count } download?
       *[other] Cancelar { $count } downloads?
    }
downloads-cancel-many-base = Todos os downloads desta série param e saem do { $client }.
downloads-cancel-all = Cancelar todos
downloads-summary-connecting = Conectando.
downloads-summary-client-on = { $client } { $version } está conectado.
downloads-summary-client-off = { $client } não está conectado. Confira o endereço e a senha nas Configurações, em Downloads.
downloads-summary-native = O próprio AniBeam busca nas suas fontes.
downloads-summary-native-with = O próprio AniBeam busca nas suas fontes. { $client }
downloads-summary-starting = Iniciando os downloads.
downloads-summary-not-running = Os downloads não estão rodando.
downloads-summary-not-started = Os downloads ainda não iniciaram.
downloads-summary-helper-ready = O ajudante de busca { $version } está pronto. { $client }
downloads-subscribed-as = Seguindo como { $name }.
downloads-repaired =
    { $count ->
        [one] { $name } consertada: o feed encontra { $count } release
       *[other] { $name } consertada: o feed encontra { $count } releases
    }
downloads-repaired-keeps =
    { $count ->
        [one] { $name } consertada: o feed encontra { $count } release e a regra mantém “{ $term }”
       *[other] { $name } consertada: o feed encontra { $count } releases e a regra mantém “{ $term }”
    }
downloads-sent-torrents =
    { $count ->
        [one] { $count } torrent enviado para o { $client }.
       *[other] { $count } torrents enviados para o { $client }.
    }
downloads-added-torrents =
    { $count ->
        [one] { $count } torrent incluído.
       *[other] { $count } torrents incluídos.
    }
downloads-trashed-paths =
    { $count ->
        [one] { $count } caminho de { $title } movido para a lixeira
       *[other] { $count } caminhos de { $title } movidos para a lixeira
    }
downloads-show-gone = Essa série não está mais na biblioteca
downloads-activity-hide = Ocultar atividade
downloads-activity = Atividade
downloads-moving =
    { $count ->
        [one] { $count } download em andamento
       *[other] { $count } downloads em andamento
    }
downloads-back = Voltar para Downloads
downloads-chip-search-ready = Busca pronta
downloads-chip-search-starting = Busca iniciando
downloads-chip-search-off = Busca desativada
downloads-chip-answers-at = { $summary } Responde em { $url }.
downloads-chip-helper-missing = O ajudante de busca não está instalado. Clique para ver as config. de Downloads, que dizem como incluí-lo
downloads-chip-helper-failed = O ajudante de busca parou. Reconectar o inicia de novo; as config. de Downloads dizem o que deu errado
downloads-chip-open-settings = Abrir as config. de Downloads
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } desligado
downloads-chip-client-not-connected = { $client } não está conectado. Reconecte, ou confira o endereço nas Configurações
downloads-chip-client-remote = Um ajudante de busca que você roda usa o cliente indicado nas configurações dele
downloads-chip-client-switch = Clique para trocar de cliente
downloads-connecting = Conectando
downloads-reconnect = Reconectar
downloads-search-placeholder = Busque uma série ou um nome de release
downloads-search-go-tip = Buscar nomes de release em { $names }
downloads-search-go-off = Todas as fontes estão desativadas.
downloads-suggest-airing =
    { $predicted ->
        [yes] No ar, episódio { $episode } previsto em { $time }
       *[no] No ar, episódio { $episode } em { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] No ar, episódios { $episode } a { $through } previstos em { $time }
       *[no] No ar, episódios { $episode } a { $through } em { $time }
    }
downloads-suggest-finished = Concluída
downloads-suggest-episodes = { $count } episódios
downloads-suggest-in-library = Na sua biblioteca
downloads-suggest-on-anilist = No AniList, fora da sua biblioteca
downloads-plan-to-watch = Quero assistir
downloads-suggest-release-search = Buscar nomes de release com “{ $query }” em { $names }
downloads-suggest-release-search-none = Buscar nomes de release com “{ $query }” em nenhuma fonte
downloads-key-return = Enter
downloads-sources = Fontes
downloads-source-hint-starting = iniciando
downloads-source-hint-down = parada
downloads-source-tip-down = { $name } precisa de um serviço ajudante que não está rodando. Clique para iniciá-lo.
downloads-source-tip-on = As buscas perguntam a { $name }. Clique para desativar.
downloads-source-tip-off = Desativada. Clique para perguntar a { $name } de novo.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Verificar
    .tip = Pergunta se cada fonte sob demanda está rodando. Não inicia nada.
downloads-check-busy = Verificando
downloads-back-to-grid = Voltar à grade
    .tip = Voltar a { $title } e ao que cada provedor encontrou. Esc faz o mesmo.
downloads-releases-on =
    { $count ->
        [one] { $count } release em { $name }
       *[other] { $count } releases em { $name }
    }
downloads-found-on = Encontrado em
downloads-found-searching = buscando
downloads-found-failed = falhou
downloads-found-cancelled = cancelada
downloads-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } releases
    }
downloads-found-retry-tip =
    { $why }
    Clique para tentar de novo
downloads-facet-groups = Grupo do release
downloads-facet-quality = Qualidade
downloads-facet-words = Palavras
downloads-filters-fewer = Menos filtros
downloads-filters-more = Mais filtros ({ $count })
downloads-narrow-placeholder = Filtre por um termo, ou -termo para excluir
downloads-best-match = Melhor opção
    .tip = Mantém os releases que se parecem com o melhor: { $query }
downloads-best-match-none = Nenhum release se destaca aqui
downloads-strip-have-of = { $have } de { $total } episódios no disco
downloads-strip-have =
    { $count ->
        [one] { $count } episódio no disco
       *[other] { $count } episódios no disco
    }
downloads-strip-next =
    { $predicted ->
        [yes] episódio { $episode } previsto em { $time }
       *[no] episódio { $episode } em { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] episódios { $episode } a { $through } previstos em { $time }
       *[no] episódios { $episode } a { $through } em { $time }
    }
downloads-strip-complete = completa
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = Nenhum destes releases é de um episódio que você já tem.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } release aqui é o arquivo que você tem, marcado No disco
       *[other] { $count } releases aqui são os arquivos que você tem, marcados No disco
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } release é de um episódio que você tem de outro release
       *[other] { $count } releases são de episódios que você tem de outro release
    }
downloads-strip-both = { $first }; { $second }.
downloads-tick-missing =
    { $count ->
        [one] Marcar o { $count } episódio que falta
       *[other] Marcar os { $count } episódios que faltam
    }
    .tip = Para cada episódio aqui que a biblioteca não tem, o release com mais seeders
downloads-open-in-library = Abrir na biblioteca
downloads-not-this-show = Não é esta série
downloads-no-releases = Nenhum release
downloads-no-releases-body = Nada em { $name } corresponde a “{ $query }”. Outra fonte, ou menos palavras, pode encontrar.
downloads-try-source = Tentar { $name }
downloads-ticked =
    { $count ->
        [one] { $count } release marcado
       *[other] { $count } releases marcados
    }
downloads-nothing-ticked = Nada marcado
downloads-ticked-size = { $size } no total.
downloads-tick-hint = Marque releases na tabela, ou baixe todos.
downloads-tick-all = Marcar todos
downloads-working = Trabalhando
downloads-download-ticked =
    { $count ->
        [one] Baixar { $count } release
       *[other] Baixar { $count } releases
    }
downloads-download-shown = Baixar os { $count } mostrados
downloads-download-all = Baixar todos os { $count }
downloads-download-movie = Baixar como filme
    .tip = Um release marcado, direto para a pasta de filmes
downloads-subscribe = Seguir
downloads-subscribe-body = Novos releases que correspondem a esta busca baixam assim que aparecem, na pasta indicada aqui.
downloads-name-placeholder = Tirado do primeiro release
downloads-subscribe-search = Seguir esta busca
downloads-subscribe-no-feed = Não há feed para seguir aqui
downloads-preview-again = Mostrar a regra de novo
downloads-preview-show = Mostrar a regra que ela cria
downloads-preview-from = De { $host }
downloads-preview-from-named = De { $host } ({ $name })
downloads-preview-must = Precisa conter { $terms }
downloads-preview-must-not = Não pode conter { $terms }
downloads-helper-changed = Os downloads apontam para um ajudante que não responde
downloads-helper-missing = Esta cópia do AniBeam veio sem downloads
downloads-helper-failed = Os downloads não puderam iniciar
downloads-helper-changed-body = As buscas e os downloads passam por um pequeno programa ajudante. As Configurações apontam para outro que não o do AniBeam, e ele não está respondendo. Volte para o do AniBeam e ele inicia sozinho.
downloads-helper-missing-body = Ela foi compilada sem o programa ajudante por onde passam as buscas e os downloads. Os instaladores na página de versões o têm. Configurações, Downloads explica como incluí-lo nesta cópia.
downloads-helper-failed-body = O programa ajudante por onde passam as buscas e os downloads parou antes de ficar pronto. Iniciá-lo de novo resolve na maioria das vezes.
downloads-helper-use-own = Usar o do AniBeam
downloads-helper-start-again = Iniciar de novo
downloads-helper-show-how = Mostrar como
downloads-helper-what-went-wrong = O que deu errado
downloads-lens-gaps-count = Faltas na biblioteca  { $count }
downloads-lens-airing-count = No ar  { $count }
downloads-lens-following-count = Seguindo  { $count }
downloads-lens-downloaded-count = Baixados  { $count }
downloads-lens-gaps = Faltas na biblioteca
downloads-lens-gaps-line = Episódios já exibidos que não estão no disco. Abra uma série para ver quem tem.
downloads-lens-gaps-none = Nada faltando: todo episódio exibido de cada série está no disco.
downloads-lens-airing-line = Séries da sua biblioteca com episódios ainda por vir. Abra uma para segui-la.
downloads-lens-airing-none = Nada na sua biblioteca está no ar
downloads-lens-airing-none-body = Uma série aparece aqui quando está na sua biblioteca e tem episódios por vir. No ar agora lista tudo que está no ar.
downloads-airing-now = No ar agora
downloads-lens-following = Seguindo
downloads-card-next =
    { $predicted ->
        [yes] Episódio { $episode } previsto em { $time }
       *[no] Episódio { $episode } em { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Episódios { $episode } a { $through } previstos em { $time }
       *[no] Episódios { $episode } a { $through } em { $time }
    }
downloads-rules-unread = Não foi possível ler as séries seguidas.
downloads-lens-following-line = Feeds que o AniBeam observa, mostrados pelas séries que eles preenchem.
downloads-none-yet = Nenhuma ainda.
downloads-reading = Lendo.
downloads-rules-error = O AniBeam não conseguiu ler as séries seguidas. Tente de novo em instantes.
downloads-removing-to-go = Removendo, faltam { $count }
downloads-unsubscribe-all = Deixar de seguir as { $count }
downloads-unsubscribe = Deixar de seguir
downloads-no-rules = Nenhuma série seguida
downloads-no-rules-body = Busque acima e siga uma série para que ela continue chegando.
downloads-search = Buscar
downloads-lens-downloaded = Baixados
downloads-dl-line = { $size } no disco. { $rest }
downloads-dl-client-off = { $client } não está conectado.
downloads-dl-unread = Não foi possível ler a lista de downloads.
downloads-dl-in-client =
    { $count ->
        [one] { $count } torrent, { $size } no { $client }.
       *[other] { $count } torrents, { $size } no { $client }.
    }
downloads-usage-watched = Assistidos
downloads-usage-not-watched = Não assistidos
downloads-deleting = Excluindo
downloads-delete-watched = Excluir assistidos
downloads-remove-all = Remover os { $count }
downloads-on-disk = No disco
downloads-on-disk-line = Na sua biblioteca sem torrent no { $client }, os maiores primeiro. Clique com o botão direito em um para excluí-lo.
downloads-nothing-downloaded = Nada baixado
downloads-nothing-downloaded-body = O que você baixa ou segue aparece aqui, uma série por vez.
downloads-client-off-title = { $client } não está conectado
downloads-client-off-body = Confira se ele está rodando e conecte de novo. O endereço e a senha estão nas Configurações, em Downloads.
downloads-connect-again = Conectar de novo
downloads-view-grid = Grade
downloads-view-list = Lista
downloads-chip-subscribed = Seguindo
downloads-speed-up = { $speed }/s up
downloads-broken-some = { $count } com arquivos faltando
downloads-tag-watched = Assistido

## PlayerDelays.qml

player-delays-title = Atraso
player-delays-subtitles = Legendas
player-delays-audio = Áudio
player-delays-subtitles-earlier = Legendas 0,1 s antes
player-delays-subtitles-later = Legendas 0,1 s depois
player-delays-audio-earlier = Áudio 0,1 s antes
player-delays-audio-later = Áudio 0,1 s depois
player-delays-reset = Zerar até o próximo arquivo
player-delays-none = Sem atraso

## TrackPicker.qml

track-picker-off = Desativado

## RollControls.qml

roll-controls-window = Mantendo { $ahead } prontos: episódios { $next } a { $through } de { $total }
roll-controls-window-watched = Mantendo { $ahead } prontos: episódios { $next } a { $through } de { $total }, { $watched } assistidos
roll-controls-more = Baixar mais { $count }
roll-controls-more-tip = Baixar os próximos episódios agora. O AniBeam volta a manter os próximos prontos quando você alcançar.
roll-controls-fewer-tip = Manter um a menos à frente
roll-controls-ahead = { $count } à frente
roll-controls-further-tip = Manter um a mais à frente
roll-controls-all = Baixar tudo
roll-controls-all-tip = Parar de manter só alguns prontos e baixar todos os episódios que faltam
roll-controls-all-question = Baixar os { $count } episódios que faltam?

## PlayerChrome.qml

player-chrome-delays = Atraso de legenda e áudio
player-chrome-audio = Faixa de áudio
player-chrome-subtitles = Legendas
player-chrome-subtitles-off = Legendas desativadas
player-chrome-casting-to = Transmitindo para { $device }
player-chrome-cast = Transmitir para TV
player-chrome-mark-watched = Marcar como assistido
player-chrome-shortcuts = Atalhos de teclado
player-chrome-fullscreen = Tela cheia
player-chrome-exit-fullscreen = Sair da tela cheia
player-chrome-stay = Ficar
player-chrome-skip-intro = Pular abertura
player-chrome-skip-outro = Pular créditos
player-chrome-playing-on = Reproduzindo em { $device }
player-chrome-replay = Repetir

## PlayerPage.qml

player-title = Player
player-next = Próximo
player-extra-no-next = Um extra não tem próximo episódio
player-last-episode = Este é o último episódio
player-extra-no-previous = Um extra não tem episódio anterior
player-first-episode = Este é o primeiro episódio
player-resumed-at = Retomado em { $time }
player-start-over = Recomeçar
player-audio-track = Áudio: { $track }
player-subtitles-track = Legendas: { $track }
player-subtitles-on = Legendas: ativadas
player-subtitles-off = Legendas desativadas
player-no-subtitles = Este arquivo não tem legendas
player-delay-seconds = { $value } s
player-audio-delay = Atraso do áudio { $value }
player-subtitle-delay = Atraso da legenda { $value }
player-volume = Volume { $volume }
player-speed = Velocidade { $speed }×
player-not-while-casting = Não durante a transmissão
player-hold-fast = 2× enquanto segura
player-no-chapters = Este arquivo não tem capítulos
player-last-chapter = Este é o último capítulo
player-chapter = Capítulo { $number }
player-chapter-titled = Capítulo { $number }: { $title }
player-screenshot-saved = Salvo em Imagens/AniBeam
player-skipped-intro = Abertura pulada
player-skipped-outro = Créditos pulados
player-undo = Desfazer
player-refusal-hidden = a série está oculta
player-refusal-no-match = a série ainda não tem vínculo; vincule em Info da série
player-refusal-not-newer = sua lista já passou deste episódio
player-refusal-extra = isto é um extra
player-refusal-unmatched = o arquivo não está ligado a um episódio
player-refusal-on-disk = o arquivo está só no disco
player-not-tracked-because = Não registrado: { $reason }.
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = Não foi possível atualizar { $tracker }
player-outcome-ok = { $tracker }: ok
player-outcome-at = { $tracker } em { $progress }
player-outcome-tracked = Registrado
player-outcome-not-tracked = Não registrado
player-outcome-could-not-update = Não foi possível atualizar sua lista
player-outcome-tracked-list = Registrado: { $list }
player-outcome-not-tracked-list = Não registrado: { $list }
player-outcome-failed-list = Não foi possível atualizar sua lista: { $list }
player-rated = Nota { $score }
player-score-failed = Não foi possível salvar a nota
player-frame = Quadro { $number } em { $time }
player-audio-title = Áudio
player-subtitles-title = Legendas
player-cast-to = Transmitir para
player-cast-looking = Procurando Chromecasts e TVs
player-cast-unreachable = Não foi possível acessar a TV
player-cast-tv = TV
player-cast-stop = Parar de transmitir para { $device }
player-cast-connecting = Conectando
player-cast-loading = Carregando
player-cast-playing = Reproduzindo
player-cast-paused = Pausado
player-cast-finished = Terminado
player-cast-stopped = Parado
player-rate-prompt = O último episódio foi registrado. Avaliar esta série?
player-rate = Avaliar
player-not-now = Agora não

## EndpointReposPanel.qml

endpoint-repos-panel-title = Listas de fontes
endpoint-repos-panel-helper = Uma lista de fontes é um conjunto assinado de lugares de busca, incluído por endereço. O AniBeam confere a assinatura e mostra a chave antes de salvar, e você instala as que quiser. As instaladas entram em Fontes de busca, acima.
endpoint-repos-panel-could-not-start = O AniBeam não conseguiu iniciar isso. Tente de novo em instantes.
endpoint-repos-panel-needs-value = { $name } precisa de um valor
endpoint-repos-panel-values-of = Valores de { $name }
endpoint-repos-panel-secret-note = Guardado no chaveiro desta máquina, nunca na biblioteca nem numa exportação.
endpoint-repos-panel-secret-note-keep = Guardado no chaveiro desta máquina, nunca na biblioteca nem numa exportação. Um campo vazio mantém o que já está guardado.
endpoint-repos-panel-asks-again = { $name } versão { $version } pede de novo
endpoint-repos-panel-update-sends-to = A nova versão envia isso para { $host }.
endpoint-repos-panel-fields-changed = Os campos mudaram.
endpoint-repos-panel-needs-fields = Precisa de { $fields } antes de buscar.
endpoint-repos-panel-no-longer-offered = Não é mais oferecida pela lista de fontes.
endpoint-repos-panel-update-requests-to = A atualização envia requisições para { $host }.
endpoint-repos-panel-unreadable = Esta lista não pôde ser lida. Atualizar listas tenta de novo.
endpoint-repos-panel-add-list = Incluir lista de fontes
endpoint-repos-panel-add-list-helper = O endereço do arquivo de índice, ou o caminho de um nesta máquina.
endpoint-repos-panel-checking = Verificando
endpoint-repos-panel-add = Incluir
endpoint-repos-panel-trust-question = Confiar em { $name }?
endpoint-repos-panel-fingerprint = Impressão da chave  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    { $count } fontes, assinadas. As fontes decidem para onde vão suas buscas; só confie em uma lista de alguém em quem você confia.
endpoint-repos-panel-preview-trusted =
    { $url }
    { $count } fontes, assinadas, já confiável. As fontes decidem para onde vão suas buscas; só confie em uma lista de alguém em quem você confia.
endpoint-repos-panel-trust = Confiar
endpoint-repos-panel-saving = Salvando
endpoint-repos-panel-install = Instalar
endpoint-repos-panel-update = Atualizar
endpoint-repos-panel-added-by-you = incluída por você
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Atualizar para v{ $version }
endpoint-repos-panel-add-value = Incluir o valor
endpoint-repos-panel-change = Mudar { $fields }
endpoint-repos-panel-asks-for = Pede { $fields }.
endpoint-repos-panel-by-hand = Incluir uma à mão
endpoint-repos-panel-by-hand-helper = Para sites sem lista: qualquer feed com {"{"}query{"}"} na busca, inclusive via proxy. Uma chave no endereço fica à parte, no chaveiro do sistema.
endpoint-repos-panel-name = Nome
endpoint-repos-panel-refreshing = Atualizando
endpoint-repos-panel-refresh = Atualizar listas

## ScheduleSheet.qml

schedule-episodes-range = Episódios { $first } a { $last }
schedule-day-format = ddd, d MMM yyyy
schedule-month-format = MMMM 'de' yyyy
schedule-tile-day-format = d MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = Em algum momento de { $year }
schedule-no-date-yet = Sem data ainda
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = Sem data
schedule-tile-last = Último
schedule-tile-out = Lançado
schedule-ends-at = Termina no episódio { $number }
schedule-dated-by-site = { $count } datados pelo { $site }
schedule-dated-by-record = { $count } datados pelo registro
schedule-set-by-you = { $count } definidos por você
schedule-guessed-weekly = { $count } estimados semanalmente
schedule-nothing-set = Nada definido ainda
schedule-saved = Salvo
schedule-cleared = Notas de programação apagadas
schedule-title = Episódios e datas
schedule-season = Temporada { $number }
schedule-ask-agent = Perguntar ao { $agent }
schedule-done-pasting = Concluir
schedule-paste-list = Colar uma lista
schedule-agent-looking = { $agent } está buscando as datas. Nada é salvo até você clicar em Salvar.
schedule-agent-stopped = { $agent } parou: { $error }
schedule-stop = Parar
schedule-paste-placeholder = Cole uma lista de lançamentos, uma tabela ou um post
schedule-paste-helper = O que o AniBeam ler aparece nos episódios abaixo.
schedule-agent-hint = Com o Claude Code neste computador, o AniBeam pode pedir a ele que busque as datas para você.
schedule-show-more = Mostrar mais episódios
schedule-pick-hint = Clique em um episódio para dar uma data. Shift+clique escolhe um trecho.
schedule-picked-one = Episódio { $number }:
schedule-picked = { $count ->
    [one] { $count } episódio:
   *[other] { $count } episódios:
}
schedule-date-placeholder = 7 jan, março 2027, 2027
schedule-set-date = Definir data
schedule-weekly = Semanal
schedule-not-last = Não é o último
schedule-last-episode = Último episódio
schedule-clear-date = Apagar data

## ShowPage.qml

show-title = Série
show-runtime-each = { $runtime } cada
show-add-to-list = Incluir na lista
show-moving-to = Movendo para { $list }
show-moved-to = Movido para { $list }
show-relation-related = Relacionado
show-no-poster = Sem pôster
show-reading-anilist = Lendo o AniList
show-not-in-library = Fora da sua biblioteca
show-get-episodes = Baixar episódios
show-trailer = Trailer
show-your-score = Sua nota { $score }
show-rate-it = Avaliar
show-anilist-score = AniList { $score }
show-forecast = { $percent }% para você
show-because-you-watched = Porque você assistiu
show-hidden-loved = { $count ->
    [one] Uma tag oculta que você adora está por trás
   *[other] { $count } tags ocultas que você adora estão por trás
}
show-you-gave-it = Você deu { $score }
show-story = História
show-genres-tags = Gêneros e tags
show-studios = Estúdios  { $studios }
show-related = Relacionados
show-recommendations = Recomendações
show-from-anilist = Do AniList
show-reading-rest = Lendo o resto do AniList

## ConnectSheet.qml

connect-connected-to = Conectado ao { $name }
connect-connect-to = Conectar { $name }
connect-did-not-finish = O login não terminou
connect-finish-in-browser = Termine no navegador
connect-signed-in-anilist = Conectado como { $account }. Sua lista está na página Assistindo.
connect-signed-in-mal = Conectado como { $account }. Os episódios são marcados lá enquanto você assiste.
connect-setup-lead = O { $name } conecta o AniBeam por meio de um app seu. Leva cerca de um minuto, uma vez só.
connect-page-open = A página de login do { $name } está aberta no seu navegador. Aprove o AniBeam lá e isto se fecha sozinho.
connect-page-opening = Abrindo a página de login do { $name } no seu navegador.
connect-open-watching = Abrir Assistindo
connect-done = Pronto
connect-open-again = Abrir a página de login de novo
connect-secret-where = É o texto longo em "Client Secret" na página do app em myanimelist.net/apiconfig.
connect-client-secret = Client secret
connect-save-try-again = Salvar e tentar de novo
connect-change-client-id = Mudar o Client ID
connect-step-create-anilist = Crie o client no AniList
connect-step-create-mal = Crie o app no MyAnimeList
connect-step-create-anilist-body = Abra as configurações de desenvolvedor do AniList, crie um novo client e cole este endereço no campo Redirect URL exatamente como está aqui.
connect-step-create-mal-body = Abra a página da API do MyAnimeList, crie um ID com App Type "other" e cole este endereço no campo App Redirect URL exatamente como está aqui.
connect-open-developer-page = Abrir a página de desenvolvedor
connect-copy-address = Copiar o endereço
connect-step-paste-id = Cole o ID do app
connect-step-paste-id-anilist = O AniList o chama de Client ID: o número na página do seu novo client.
connect-step-paste-id-mal = O MyAnimeList o chama de Client ID: o texto longo na página do seu novo app.
connect-client-id = Client ID
connect-client-secret-web = Client secret (só para apps Web)
connect-step-connect-body = O navegador abre, o { $name } pede que você entre e aprove o AniBeam, e o AniBeam recebe a resposta.

## SettingsPage

settings-title = Configurações
settings-search-placeholder = Buscar configurações
settings-section-library = Biblioteca
settings-section-accounts = Contas
settings-section-appearance = Aparência
settings-section-playback = Reprodução
settings-section-subtitles = Legendas
settings-section-keys = Teclado
settings-section-downloads = Downloads
settings-section-phone = App do celular
settings-section-storage = Armazenamento e backup
settings-section-help = Ajuda
settings-section-about = Sobre
settings-section-developer = Desenvolvedor
settings-status-library-empty = Nenhuma pasta ainda
settings-status-library =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    } em { $folders ->
        [one] { $folders } pasta
       *[other] { $folders } pastas
    }
settings-status-library-missing =
    { $gone ->
        [one] { $gone } pasta não encontrada
       *[other] { $gone } pastas não encontradas
    }, { $count ->
        [one] { $count } série
       *[other] { $count } séries
    } em { $folders ->
        [one] { $folders } pasta
       *[other] { $folders } pastas
    }
settings-status-accounts-anilist = Sincronizando com o AniList
settings-status-accounts-mal = Sincronizando com o MyAnimeList
settings-status-accounts-both = Sincronizando com AniList e MyAnimeList
settings-status-accounts-none = Nenhuma lista conectada
settings-status-appearance =
    { $mode ->
        [system] Segue o sistema
        [light] Claro
       *[dark] Escuro
    }, { $source ->
        [theme] cores do tema
       *[desktop] cores do desktop
    }
settings-status-playback =
    { $skip ->
        [both] Pula aberturas e créditos
        [intro] Pula aberturas
        [outro] Pula créditos
       *[none] Reproduz tudo até o fim
    }
settings-status-playback-audio =
    { $skip ->
        [both] Pula aberturas e créditos
        [intro] Pula aberturas
        [outro] Pula créditos
       *[none] Reproduz tudo até o fim
    }, áudio { $languages }
settings-status-subtitles =
    Tamanho { $percent } por cento, { $override ->
        [AsScripted] arquivos estilizados como foram feitos
        [Force] seu estilo em todos os arquivos
       *[ScaleOnly] arquivos estilizados mantêm o visual
    }
settings-status-keys = Todas as teclas que o AniBeam entende
settings-status-downloads-unread = Ainda não lido
settings-status-downloads-answered = O qBittorrent respondeu
settings-status-downloads-missing = Ajudante de busca não instalado
settings-status-downloads-unconnected = Ainda não conectado
settings-status-phone-shared = Compartilhado na sua tailnet
settings-status-phone-off = Não compartilhado
settings-status-storage = { $size } de imagens em cache
settings-status-storage-unread = Imagens, exportação e importação
settings-status-help = Respostas ao que mais perguntam
settings-status-about-update = A versão { $version } saiu
settings-status-developer = Medindo cada página
settings-status-developer-last = { $page } montada em { $ms } ms
settings-results =
    { $count ->
        [one] { $count } configuração encontrada
       *[other] { $count } configurações encontradas
    }
settings-results-none = Nada encontrado
settings-results-hint = Enter abre a primeira.
settings-results-none-hint = Tente uma palavra mais simples: tema, legendas, lista, senha, pasta.

settings-find-folders =
    .words = incluir pasta verificar escanear remover fonte caminho coleção folder scan
settings-find-matching =
    .words = metadados info série provedor anilist tvmaze ordem vínculo vincular
settings-find-hidden-shows =
    .words = oculto ocultas mostrar revelar
settings-find-hidden-airing =
    .words = sem interesse no ar série mostrar de novo
settings-find-calendar =
    .words = calendário ics ical no ar programação assinar
settings-find-anime-lists =
    .words = anilist myanimelist mal entrar conectar login conta desconectar tracker lista
settings-find-main-list =
    .words = progresso cards principal tracker lista
settings-find-sync-now =
    .words = atualizar sincronizar progresso listas tracker
settings-find-on-hold =
    .words = em pausa pausado inativo parado dias
settings-find-language =
    .words = idioma língua tradução locale inglês português
settings-find-mode =
    .words = escuro claro sistema tema noite modo
settings-find-colours-from =
    .words = terminal desktop sistema tema origem kitty cores
settings-find-dark-theme =
    .words = cor cores esquema base16 escuro
settings-find-light-theme =
    .words = cor cores esquema base16 claro
settings-find-accent =
    .words = cor destaque realce
settings-find-text-size =
    .words = fonte letra palavras maior menor tamanho zoom escala ler acessibilidade texto
settings-find-density =
    .words = compacto confortável espaçamento tamanho densidade
settings-find-poster-size =
    .words = cards pequeno médio grande pôster
settings-find-out-now =
    .words = no ar já saiu cards lista linhas estilo banner pôsteres
settings-find-out-now-cards =
    .words = no ar já saiu grade colunas linhas layout
settings-find-reduce-motion =
    .words = animação acessibilidade movimento
settings-find-app-icon =
    .words = logo ícone launcher dock barra de tarefas janela escuro
settings-find-skip-intros =
    .words = abertura op pular automático aniskip capítulos
settings-find-skip-outros =
    .words = encerramento ed créditos pular automático aniskip

settings-find-mpv-settings =
    .words = mpv.conf config avançado configuração
settings-find-discord =
    .words = discord rich presence status perfil
settings-find-next-episode =
    .words = próximo botão título reprodução automática fim
settings-find-subtitle-languages =
    .words = idioma faixa legendas subs
settings-find-audio-languages =
    .words = idioma faixa dublagem japonês áudio
settings-find-start-over =
    .words = redefinir padrão estilo legenda
settings-find-size =
    .words = escala tamanho legenda
settings-find-position =
    .words = altura margem posição legenda
settings-find-styled =
    .words = ass sobrepor forçar estilizada
settings-find-font =
    .words = fonte tipo letra
settings-find-text-colour =
    .words = cor texto
settings-find-outline =
    .words = borda contorno
settings-find-background =
    .words = caixa opacidade fundo
settings-find-bold =
    .words = peso negrito
settings-find-keys-player =
    .words = teclado atalhos teclas
settings-find-keys-player-around =
    .words = teclado atalhos tela cheia próximo anterior episódio voltar
settings-find-keys-around =
    .words = teclado atalhos voltar avançar botão direito menu tela cheia
settings-find-keys-anywhere =
    .words = teclado atalhos teclas configurações sair
settings-find-torrent-client =
    .words = qbittorrent senha usuário endereço webui seedbox cliente torrent
settings-find-search-helper =
    .words = anirss rss feeds token comando servidor avançado ajudante
settings-find-next-few =
    .words = contínuo episódios à frente excluir assistidos manter disco espaço
settings-find-unsubscribe =
    .words = deixar de seguir feeds concluídas
settings-find-last-search =
    .words = downloads busca lembrar
settings-find-search-sources =
    .words = endpoints feeds config anirss fontes
settings-find-source-lists =
    .words = repo repos endpoint assinado endpoints jackett prowlarr lista fontes
settings-find-share-tailscale =
    .words = celular android tailnet remoto host compartilhar
settings-find-stream =
    .words = streaming reproduzir transmitir episódios celular arquivos
settings-find-devices =
    .words = confiar confiável desativar bloquear dispositivo tailscale
settings-find-back-up =
    .words = exportar backup arquivo json
settings-find-restore =
    .words = importar restaurar arquivo json
settings-find-image-cache =
    .words = pôsteres limpar imagens disco cache
settings-find-files =
    .words = banco de dados dados config cache pasta caminho
settings-find-help-folders =
    .words = ajuda pasta estrutura nomes episódios temporadas filmes arquivos
settings-find-help-recognised =
    .words = ajuda vínculo errado série não encontrada sem vínculo reconhecida
settings-find-help-anilist =
    .words = ajuda anilist myanimelist mal entrar conectar login tracker
settings-find-help-downloads =
    .words = ajuda downloads vazio fontes nada torrent qbittorrent
settings-find-help-data =
    .words = ajuda dados arquivos backup banco de dados privacidade
settings-find-help-else =
    .words = ajuda configuração introdução passo a passo relatar bug problema issue
settings-find-version =
    .words = versão build novidades changelog
settings-find-setup =
    .words = primeira execução passo a passo introdução configurar de novo
settings-find-updates =
    .words = estável nightly canal verificar atualizações
settings-find-developer-options =
    .words = debug depuração tempo desenvolvedor
settings-find-ratings =
    .words = imdb créditos notas

## SettingsDownloadsTab

settings-downloads-title = Downloads
settings-downloads-lead = Para onde o AniBeam envia o que você baixa, e como ele busca nos feeds de releases.
settings-downloads-keep-none = Terminar um episódio exclui o anterior.
settings-downloads-keep-line = Episódios assistidos mantidos antes do último visto. Com { $count }, terminar o episódio { $finishing } exclui o 1.
settings-downloads-anirss-ready = Rodando, versão { $version } em { $url }.
settings-downloads-anirss-starting = Iniciando o ajudante de busca.
settings-downloads-anirss-missing = O ajudante de busca não está instalado. Reinstale o AniBeam, ou indique seu próprio anirss em Comando.
settings-downloads-anirss-failed = O ajudante de busca parou. Tentar de novo o inicia; Detalhes mostra o que ele disse.
settings-downloads-anirss-idle = Ainda não iniciado. Ele inicia na primeira busca na página Downloads.
settings-downloads-torrent-trying = Tentando { $client }.
settings-downloads-torrent-answered = { $client } { $version } respondeu.
settings-downloads-torrent-failed = { $client } não conectou. Confira o endereço e a senha acima e teste de novo.
settings-downloads-torrent-untried = Ainda não testado.
settings-downloads-mode-existing = Um qBittorrent que você roda, com a Web UI ativada.
settings-downloads-mode-managed = O AniBeam roda um qBittorrent próprio em ~/Downloads/Anime. Ele continua depois que o AniBeam fecha, então as séries seguidas continuam baixando.
settings-downloads-mode-auto = Usa o qBittorrent do endereço abaixo ou roda um próprio.
settings-downloads-password-saved = Há uma senha salva aqui para este cliente.
settings-downloads-password-anirss = Usando a senha que o ajudante de busca salvou; salve uma aqui para não depender dela.
settings-downloads-password-none = Nenhuma senha salva. Um cliente que dispensa login para esta máquina não precisa de uma.
settings-downloads-mismatch = O ajudante de busca fala com { $helperUrl } enquanto o AniBeam fala com { $url }. Um ajudante de busca que você roda lê o endereço do próprio config.toml, então trocar de cliente aqui não o muda.
settings-downloads-torrent-title = Cliente de torrent
settings-downloads-torrent-helper = O qBittorrent que faz os downloads. A senha fica no chaveiro de senhas do sistema.
settings-downloads-which-client = Qual cliente
settings-downloads-pick-for-me = Escolher por mim
settings-downloads-my-own = O meu
settings-downloads-built-in = Embutido
settings-downloads-saved-clients = Clientes salvos
settings-downloads-saved-clients-helper = Uma seedbox e o desta máquina, por exemplo. O chip de cliente na página Downloads também alterna entre eles.
settings-downloads-add-client = Incluir
settings-downloads-name = Nome
settings-downloads-name-placeholder = Com o nome do endereço
settings-downloads-address = Endereço
settings-downloads-address-managed-helper = Só esta máquina o acessa, e ele não precisa de senha.
settings-downloads-address-helper = O endereço da Web UI.
settings-downloads-username = Usuário
settings-downloads-password = Senha
settings-downloads-secret-saved = Salva
settings-downloads-secret-unset = Não definida
settings-downloads-connection = Conexão
settings-downloads-open-config = Abrir a config
settings-downloads-details = Detalhes
settings-downloads-details-torrent = { $client } disse isto quando o AniBeam tentou.
settings-downloads-test-trying = Tentando
settings-downloads-test-save = Salvar e testar
settings-downloads-test = Testar
settings-downloads-searching = Busca
settings-downloads-search-helper = Ajudante de busca
settings-downloads-searching-helper = O próprio AniBeam busca nas fontes e manda sua escolha ao cliente de torrent. Não precisa do ajudante de busca.
settings-downloads-search-helper-helper = A parte do AniBeam que busca e segue séries, um programa chamado anirss. O AniBeam o roda para você; mude isto só se você roda o seu próprio.
settings-downloads-runs-in = As buscas rodam em
settings-downloads-runs-in-native-helper = Séries seguidas baixam pelo cliente de torrent, até com o AniBeam fechado.
settings-downloads-runs-in-helper = O ajudante de busca, um programa separado que o AniBeam ou você roda.
settings-downloads-engine-native = O próprio AniBeam
settings-downloads-how-runs = Como ele roda
settings-downloads-how-runs-sidecar = O AniBeam o inicia ao abrir e o para ao fechar.
settings-downloads-how-runs-remote = O AniBeam usa um que você já roda, no endereço e com a chave abaixo.
settings-downloads-runs-sidecar = O AniBeam roda
settings-downloads-runs-remote = Eu rodo
settings-downloads-command = Comando
settings-downloads-command-helper = Um nome no PATH ou um caminho completo. ~/.local/bin também é consultado.
settings-downloads-server-address-helper = A base do servidor, sem /api/v1.
settings-downloads-key = Chave
settings-downloads-key-saved-helper = Há uma chave salva. Digite uma nova para substituí-la, ou limpe o campo para removê-la.
settings-downloads-key-helper = A chave que o ajudante de busca mostrou ao iniciar. Ela dá acesso ao AniBeam.
settings-downloads-details-anirss = O ajudante de busca disse isto quando parou.
settings-downloads-starting = Iniciando
settings-downloads-reconnect = Reconectar
settings-downloads-save-first = Salve suas mudanças primeiro
settings-downloads-roll-title = Manter os próximos prontos
settings-downloads-roll-helper = Séries longas baixam aos poucos: cada episódio visto puxa o próximo. Nunca precisa caber tudo no disco.
settings-downloads-roll-big = Para séries longas
settings-downloads-roll-big-helper = Séries com mais de { $count } episódios baixam aos poucos sozinhas. A linha delas em Downloads ainda pode baixar tudo de uma vez.
settings-downloads-roll-ahead = Episódios à frente
settings-downloads-roll-ahead-helper = Quantos ficam baixados além do que você está vendo. 12 é cerca de uma temporada.
settings-downloads-unit-episodes = episódios
settings-downloads-roll-over = Longa é acima de
settings-downloads-roll-over-helper = Downloads com mais episódios que isso baixam alguns por vez.
settings-downloads-roll-watched = Episódios assistidos
settings-downloads-roll-watched-delete-helper = Excluídos quando você avança, então o disco só guarda o que falta assistir.
settings-downloads-roll-watched-keep-helper = Todo arquivo fica no disco depois que você assiste.
settings-downloads-keep-all = Manter todos
settings-downloads-keep-behind = Manter para trás
settings-downloads-subscriptions-title = Séries seguidas e buscas
settings-downloads-last-search = Manter a última busca
settings-downloads-last-search-helper = A página Downloads abre com a última busca. Desativado, abre vazia.
settings-downloads-prune = Deixar de seguir séries concluídas
settings-downloads-prune-helper = 16 dias depois que o último episódio de uma série vai ao ar, o feed dela é removido. A série e os arquivos ficam.
settings-downloads-sources-title = Fontes de busca
settings-downloads-sources-helper = Para onde vai a busca, na ordem de tentativa. Uma que não acha nada passa para a próxima.
settings-downloads-sources-empty = Nenhuma fonte configurada ainda
settings-downloads-sources-connect = Conecte o ajudante de busca para listá-las
settings-downloads-sources-empty-helper = Inclua uma lista abaixo e suas fontes aparecem aqui.
settings-downloads-config-file = Arquivo de config. do ajudante de busca
settings-downloads-config-native-helper = Guarda as pastas de destino, as tags e as regras de melhor opção, que o próprio AniBeam segue quando roda o ajudante. As fontes dele são só do ajudante.
settings-downloads-config-helper = Guarda as fontes, as pastas de destino, as tags e as regras de melhor opção.
settings-downloads-unsaved = Mudanças não salvas
settings-downloads-unsaved-helper = Salvar reinicia o ajudante de busca com as novas configurações.
settings-downloads-discard = Descartar

## SettingsDeveloperTab

settings-developer-title = Desenvolvedor
settings-developer-lead = Por que uma página está lenta, medido. Cada opção vale para a próxima página que você abrir.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds } s
settings-developer-switches = Opções
settings-developer-load-images = Carregar imagens
settings-developer-load-images-helper = Desativado, nenhum pôster, banner ou retrato é lido nem decodificado.
settings-developer-overlay = Sobreposição de carga
settings-developer-overlay-helper = A última carga de página e a memória, no canto inferior direito da janela.
settings-developer-read-cache = Guardar leituras da biblioteca
settings-developer-read-cache-helper = A resposta de uma leitura da biblioteca inteira fica guardada até a biblioteca mudar. Desativado, cada página pergunta ao core de novo.
settings-developer-poster-pool = Pré-carregar pôsteres
settings-developer-poster-pool-helper = Decodifica os 60 pôsteres que você mais provavelmente verá e os mantém na memória.
settings-developer-options = Opções de desenvolvedor
settings-developer-options-helper = Desativa esta aba, a sobreposição e as medições.
settings-developer-memory = Memória
settings-developer-memory-helper = O que o kernel conta para este processo. As imagens guardam pixels decodificados na memória e na placa de vídeo.
settings-developer-memory-unknown = Este sistema não informa quanta memória o app usa.
settings-developer-in-use = Em uso
settings-developer-peak = Pico
settings-developer-heap = Heap
settings-developer-files-mapped = Arquivos mapeados
settings-developer-threads = Threads
settings-developer-give-back = Devolver memória
settings-developer-give-back-helper = Descarta o que o scene graph pode reconstruir, coleta o lixo do JavaScript e devolve o heap livre ao sistema. O que for liberado é o que o app segurava sem precisar.
settings-developer-released = { $before } para { $after }
settings-developer-free-now = Liberar agora
settings-developer-images-total = Imagens desde o início
settings-developer-images-total-line = { $decodes } decodificadas, { $pixels } de pixels a partir de { $disk } no disco, { $ms } ms
settings-developer-loads = Cargas de página
settings-developer-loads-helper = Montada é a espera após um clique: a thread do Qt não faz mais nada até a página existir. Assentada é quando a última imagem chegou.
settings-developer-touring = Medindo, { $at } de { $steps }
settings-developer-tour = Medir todas as páginas
settings-developer-copy-report = Copiar relatório
settings-developer-col-page = Página
settings-developer-col-built = Montada
settings-developer-col-painted = Pintada
settings-developer-col-settled = Assentada
settings-developer-col-calls = Chamadas, dados
settings-developer-col-decodes = Decodificações, tempo, pixels
settings-developer-col-items = Itens
settings-developer-col-offscreen = Imagens fora da tela
settings-developer-col-memory = Memória
settings-developer-load-no-images = { $page }, sem imagens
settings-developer-offscreen-of = { $offscreen } de { $total }
settings-developer-loads-empty = Abra qualquer página e ela aparece aqui.
settings-developer-load-images-off = { $page }, imagens desativadas
settings-developer-load-frames = { $frames } quadros pintados durante a carga, o maior intervalo entre dois foi { $gap }.
settings-developer-load-frames-timed-out = { $frames } quadros pintados durante a carga, o maior intervalo entre dois foi { $gap }. Parou de esperar as imagens após 15 s.
settings-developer-built = Montada
settings-developer-settled = Assentada
settings-developer-core-data = Dados do core
settings-developer-pixels = Pixels decodificados
settings-developer-page-items = Itens na página
settings-developer-cached-images = Imagens já na memória
settings-developer-calls = Chamadas ao core, as mais lentas primeiro
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  em cache
settings-developer-call = { $name }  { $ms } ms  { $size }  core { $core }, para o QML { $qml }
settings-developer-decodes = Decodificações mais lentas
settings-developer-decodes-line = { $count } decodificações seguidas na thread de imagens, { $span } da primeira à última; { $requests } pedidas, { $cached } já decodificadas, { $blocked } bloqueadas
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } para { $w }x{ $h }  { $size }
settings-developer-decodes-none = Nada decodificado: todas as imagens já estavam na memória.
settings-developer-images-were-off = As imagens estavam desativadas.
settings-developer-scene = Do que a página é feita
settings-developer-scene-line = { $items } itens, { $offscreen } fora da janela; { $texts } textos, { $shapes } formas, { $icons } ícones; { $images } imagens, { $imagesOffscreen } delas fora da janela ocupando { $size }
settings-developer-copy-load = Copiar esta carga
settings-developer-tour-title = Todas as páginas, com e sem imagens
settings-developer-tour-helper = A mediana de três visitas a cada página. Onde as duas colunas batem, não são as imagens que deixam a página lenta.
settings-developer-col-built-on = Montada, com imagens
settings-developer-col-built-off = Montada, sem imagens
settings-developer-col-settled-on = Assentada, com
settings-developer-col-settled-off = Assentada, sem
settings-developer-col-decodes-time = Decodificações, tempo
settings-developer-col-data = Dados
settings-developer-tour-peak = Pico de memória durante a medição { $size }
settings-developer-events = Eventos
settings-developer-events-helper = Tudo o que o core diz, todos os níveis, do mais novo. Cada evento é copiado para a thread do Qt como JSON; o tamanho é o custo do envio.
settings-developer-event-rate = { $kind } { $count }/min, { $size }
settings-developer-events-filter = Filtrar por palavras, tipo ou etapa
settings-developer-event-plain = { $message }   (simples: { $plain })
settings-developer-copy-events = Copiar eventos

## SettingsKeysTab

settings-keys-title = Teclado
settings-keys-lead = Todas as teclas que o AniBeam entende. Aperte ? em qualquer tela para ver as dela.
settings-keys-player = No player
settings-keys-player-around = Navegando no player
settings-keys-around = Navegação
settings-keys-anywhere = Em qualquer lugar

## KeyHelp

key-help-title = Atalhos de teclado
key-help-group-playback = Reprodução
key-help-group-navigation = Navegação
key-help-group-library = Biblioteca
key-help-key-space = Espaço / K
key-help-key-left-right = Esquerda / Direita
key-help-key-ctrl-right = Ctrl+Direita
key-help-key-backspace = Backspace
key-help-key-up-down = Cima / Baixo
key-help-key-hold = Segurar o vídeo
key-help-key-page = Page Up / Down
key-help-key-home-end = Home / End
key-help-key-escape = Esc
key-help-key-alt-left = Alt+Esquerda
key-help-key-alt-right = Alt+Direita
key-help-key-shift-f10 = Shift+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / ou Ctrl+F
key-help-key-ctrl-space = Ctrl+Espaço
key-help-key-shift-space = Shift+Espaço
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = Reproduzir ou pausar
key-help-seek-5 = Pular 5 s
key-help-seek-10 = Pular 10 s
key-help-skip = Pular abertura ou créditos, senão 90 s
key-help-frame-step = Um quadro para trás ou para frente
key-help-speed = Mais lento ou mais rápido, de 0,25 em 0,25
key-help-normal-speed = Velocidade normal
key-help-volume = Volume 5 para cima ou para baixo
key-help-mute = Silenciar
key-help-subtitles-toggle = Desativar e reativar legendas
key-help-subtitle-delay = Legenda 100 ms antes ou depois
key-help-audio-delay = Áudio 100 ms antes ou depois
key-help-hold-speed = Velocidade 2× enquanto segura
key-help-screenshot = Salvar uma captura em Imagens
key-help-episode = Próximo episódio ou anterior
key-help-chapter = Capítulo anterior ou próximo
key-help-start-end = O início, ou os créditos
key-help-fullscreen = Tela cheia
key-help-escape-player = Fecha o que está aberto, senão sai da tela cheia, senão sai do player
key-help-toggle-list = Mostrar ou ocultar esta lista
key-help-library-search = Biblioteca e busca
key-help-search-page = Buscar nesta página
key-help-activity = Atividade
key-help-settings = Configurações
key-help-quit = Sair
key-help-forward = Avançar
key-help-escape-app = Fecha o que está aberto, senão volta
key-help-context-menu = O menu do botão direito
key-help-pick-card = Escolher um card, para agir em vários
key-help-pick-range = Escolher todos os cards até este
key-help-pick-all = Escolher todos os cards mostrados

## SettingsAboutTab

settings-about-title = Sobre
settings-about-lead = Que AniBeam é este, e como ele fica sabendo do próximo.
settings-about-update-failed = A atualização não terminou. Tente de novo, ou baixe pela página da versão.
settings-about-update-ready = A { $version } está pronta para instalar.
settings-about-update-command = A { $version } saiu. Atualize com { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = A { $version } saiu. Esta cópia foi compilada do código: faça pull e recompile.
settings-about-update-elsewhere = A { $version } saiu, sem download aqui para este computador.
settings-about-version = Versão
settings-about-build = AniBeam { $build }
settings-about-whats-new = Novidades
settings-about-install = Instalar { $version }
settings-about-setup = Configuração inicial
settings-about-setup-helper = O passo a passo de uma instalação nova: sua pasta, sua lista e os downloads.
settings-about-run-setup = Rodar a configuração de novo
settings-about-updates = Atualizações
settings-about-nightly-helper = Uma build nova quase todo dia. Mais nova, e menos testada.
settings-about-stable-helper = Uma versão nova quando ela foi testada e lançada.
settings-about-stable = Estável
settings-about-nightly = Nightly
settings-about-check-now = Verificar agora
settings-about-developer = Opções de desenvolvedor
settings-about-developer-helper = Inclui a seção Desenvolvedor: mede cada página, mostra a memória e o log técnico todo e pode desligar imagens.
settings-about-credits = Créditos
settings-about-ratings = Notas
settings-about-ratings-helper = Informações cedidas pelo IMDb (https://www.imdb.com). Usadas com permissão.
settings-about-series = Séries e episódios
settings-about-series-helper = AniList e TVmaze. Tempos de pulo do AniSkip.

## SettingsPlaybackTab

settings-playback-title = Reprodução
settings-playback-lead = O que o player faz sozinho, para você só mexer nele quando quiser.
settings-playback-skipping = Pular
settings-playback-skipping-helper = Usa os capítulos do arquivo, senão tempos de pulo compartilhados online. Desfazer no player desliga o pulo até o AniBeam fechar.
settings-playback-skip-intros = Pular aberturas
settings-playback-skip-outros = Pular créditos
settings-playback-skip-outros-helper = Chegar aos créditos também marca o episódio como assistido.
settings-playback-next-episode = Próximo episódio
settings-playback-name-next = Mostrar o nome do próximo episódio
settings-playback-name-next-helper = O botão Próximo no fim de um episódio mostra o título do seguinte. Desativado, mostra só o número.
settings-playback-languages = Idiomas
settings-playback-languages-helper = Códigos de idioma, do mais desejado. Vence o primeiro que o arquivo tiver. A faixa escolhida no player fica salva para a série.
settings-playback-subtitle-languages = Idiomas da legenda
settings-playback-audio-languages = Idiomas do áudio
settings-playback-discord = Mostrar o que você assiste no Discord
settings-playback-discord-check-failed = O AniBeam não conseguiu verificar o Discord agora. Tente de novo em instantes.
settings-playback-discord-helper = A série, o episódio e o pôster aparecem no seu perfil do Discord enquanto você assiste. Nada é enviado com isto desativado ou com o Discord fechado.
settings-playback-discord-unavailable = Esta versão não tem aplicativo do Discord.
settings-playback-discord-connected = Conectado ao Discord
settings-playback-discord-not-connected = Não conectado. O AniBeam conecta enquanto um episódio toca e o Discord está aberto
settings-playback-advanced = Avançado
settings-playback-my-mpv = Usar minhas config. do mpv
settings-playback-my-mpv-helper = Lê seu mpv.conf. Opções que só funcionam na inicialização são ignoradas, e scripts nunca carregam.
settings-playback-no-mpv-conf = Nenhum arquivo de config. do mpv encontrado. Coloque suas opções em mpv.conf e ative isto.

## SliderRow

slider-row-spoken-unit = { $name }, em { $unit }
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Destaque vermelho
swatches-accent-orange = Destaque laranja
swatches-accent-yellow = Destaque amarelo
swatches-accent-green = Destaque verde
swatches-accent-cyan = Destaque ciano
swatches-accent-blue = Destaque azul

## LookPreview

look-preview-dark = Escuro
look-preview-light = Claro

## SettingsAccountsTab

settings-accounts-title = Contas
settings-accounts-lead = Conecte o AniList ou o MyAnimeList e cada episódio que você termina é marcado lá para você. As contagens só sobem.
settings-accounts-syncing = Sincronizando suas listas
settings-accounts-synced = Listas sincronizadas
settings-accounts-lists = Suas listas de anime
settings-accounts-lists-helper = Sem uma, o AniBeam ainda lembra o que você assistiu neste computador. Uma lista no AniList ou no MyAnimeList leva isso ao seu celular e ao seu perfil.
settings-accounts-progress = Progresso

settings-accounts-main-list = Lista principal
settings-accounts-main-list-helper = Os cards mostram o progresso desta. A outra continua recebendo todas as atualizações.
settings-accounts-local-lists = Listas neste computador
settings-accounts-local-lists-helper =
    { $count ->
        [one] Uma série tem
       *[other] { $count } séries têm
    } status ou nota só aqui. Mova para o { $tracker } para vê-las no celular e no seu perfil.
settings-accounts-review-move = Revisar e mover
settings-accounts-nothing-to-move = Nada mais para mover
settings-accounts-sync-now = Sincronizar agora
settings-accounts-sync-now-helper = Lê de novo todas as listas conectadas. O AniBeam também faz isso ao iniciar.
settings-accounts-sync = Sincronizar
settings-accounts-move-on-hold = Mover para Em pausa
settings-accounts-move-on-hold-helper = Uma série em Assistindo que você não toca há um tempo vai para Em pausa nas suas listas. Assistir a move de volta. Uma série esperando o próximo episódio fica onde está.
settings-accounts-after = Depois de
settings-accounts-after-line = Dias sem assistir um episódio
settings-accounts-days-unit = dias

## SettingsPhoneTab

settings-phone-title = App do celular
settings-phone-lead = O AniBeam no celular pode reproduzir esta biblioteca, ver o que está no ar e acompanhar seus downloads, onde você estiver.
settings-phone-online = Online
settings-phone-offline = Offline
settings-phone-untrusted = Não recebe nada deste computador
settings-phone-sharing = Compartilhamento
settings-phone-sharing-helper = Só os seus dispositivos no Tailscale têm acesso. Episódios assistidos no celular mantêm a posição e o progresso aqui.
settings-phone-share = Compartilhar pelo Tailscale
settings-phone-stream = Transmitir episódios
settings-phone-stream-on = Seus dispositivos reproduzem e transmitem os episódios daqui.
settings-phone-stream-off = Seus dispositivos veem a biblioteca e o que está no ar, sem reproduzir.
settings-phone-host-service = Serviço de host
settings-phone-not-running = Parado
settings-phone-running = Rodando
settings-phone-service-off-helper = Rode o comando uma vez e seus dispositivos acessam o AniBeam a partir do login, sem janela aberta.
settings-phone-service-on-helper = Seus dispositivos acessam o AniBeam sem janela aberta. Com esta janela aberta, ela compartilha no lugar do serviço.
settings-phone-copy-command = Copiar comando
settings-phone-command-copied = Comando copiado
settings-phone-nobody = Não recebe nada: o Tailscale não indica uma pessoa para ele
settings-phone-people = Pessoas
settings-phone-own-profile = O perfil deste computador
settings-phone-person-own-devices =
    { $count ->
        [0] Este computador
        [one] Este computador e { $count } dispositivo
       *[other] Este computador e { $count } dispositivos
    }
settings-phone-person-devices =
    { $count ->
        [one] { $count } dispositivo
       *[other] { $count } dispositivos
    }
settings-phone-devices = Dispositivos na sua tailnet
settings-phone-devices-helper = Todo dispositivo da sua tailnet acessa este computador com o compartilhamento ativo. Desative um e ele não recebe nada.
settings-phone-addresses = Endereços
settings-phone-addresses-helper = Digite um destes no app do celular na primeira vez que ele pedir.
settings-phone-copy = Copiar
settings-phone-address-copied = Endereço copiado

## SettingsHelpTab

settings-help-title = Ajuda
settings-help-lead = Respostas curtas às dúvidas comuns, cada uma com um atalho para resolver.
settings-help-folders = Como devem ser minhas pastas?
settings-help-folders-helper = Uma pasta por série, episódios dentro. Cada arquivo leva o número, como nos releases: "Frieren - 01.mkv" ou "Frieren S01E01.mkv". A 2ª temporada vai em pasta própria ou em "Season 2" na série. Filmes: pasta Movies, um arquivo cada.
settings-help-for-example = Por exemplo
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = Nenhuma pasta incluída ainda
settings-help-reads-folders =
    { $count ->
        [one] O AniBeam lê uma pasta
       *[other] O AniBeam lê { $count } pastas
    }
settings-help-add-folder-helper = Inclua a pasta que contém suas séries, não cada série separada.
settings-help-add-folder = Incluir pasta
settings-help-open-folders = Abrir suas pastas
settings-help-match = Por que minha série não foi reconhecida?
settings-help-match-helper = O AniBeam busca cada pasta pelo nome, primeiro no AniList, depois no TVmaze. Palavras a mais, um erro de digitação ou um nome em outro idioma podem falhar. Escolha a série certa à mão em Info da série; uma escolha sua nunca é substituída.
settings-help-all-recognised = Todas as séries foram reconhecidas
settings-help-unrecognised =
    { $count ->
        [one] Uma série não foi reconhecida
       *[other] { $count } séries não foram reconhecidas
    }
settings-help-open-show-info = Abrir Info da série
settings-help-connect = Como conecto o AniList?
settings-help-connect-helper = Conecte e cada episódio que você termina é marcado na sua lista, e o celular e o perfil ficam em dia. O AniList pede que você crie um pequeno app seu uma vez; o AniBeam guia você, e leva cerca de um minuto. O MyAnimeList funciona igual.
settings-help-both-connected = Os dois estão conectados
settings-help-anilist-connected = O AniList está conectado
settings-help-mal-connected = O MyAnimeList está conectado
settings-help-none-connected = Nenhum conectado ainda
settings-help-connect-anilist = Conectar AniList
settings-help-connect-mal = Conectar MyAnimeList
settings-help-downloads = Por que Downloads está vazio?
settings-help-downloads-helper = O AniBeam não vem com lugares para buscar. Inclua uma lista de fontes nas Configurações, em Downloads, e instale dela as fontes que quiser. Baixar também precisa de um cliente de torrent: o AniBeam pode rodar o próprio, ou usar um qBittorrent que você já tem.
settings-help-sources = Fontes e cliente de torrent
settings-help-open-downloads = Abrir config. de Downloads
settings-help-data = Onde ficam meus dados?
settings-help-data-helper = Sua biblioteca, o que você viu e suas configurações ficam neste computador. Os logins do AniList e MyAnimeList vão para o chaveiro do sistema ou, sem ele, para um arquivo que só você lê. Faça backup de tudo num arquivo em Armazenamento e backup.
settings-help-library-history = Biblioteca e histórico
settings-help-show-folder = Mostrar a pasta
settings-help-backup = Backup ou restauração
settings-help-open-storage = Abrir Armazenamento e backup
settings-help-more = Outra coisa?
settings-help-more-helper = Refaça a configuração inicial ou diga o que deu errado. Uma captura ajuda muito.
settings-help-first-setup = A configuração inicial
settings-help-first-setup-helper = Sua pasta, sua lista e os downloads, um passo de cada vez.
settings-help-run-setup = Rodar a configuração de novo
settings-help-report = Relatar um problema
settings-help-report-helper = Abre um relatório de bug no GitHub no seu navegador, com esta versão já preenchida.

## SettingsSubtitlesTab

settings-subtitles-title = Legendas
settings-subtitles-lead = Com o que todo episódio começa. As mudanças aparecem no quadro abaixo enquanto você as faz.
settings-subtitles-reset-done = As legendas voltaram ao visual original
settings-subtitles-size-place = Tamanho e posição
settings-subtitles-size = Tamanho
settings-subtitles-position = Posição
settings-subtitles-position-helper = 100 fica na borda inferior. Números menores sobem as linhas na tela.
settings-subtitles-style = Estilo
settings-subtitles-styled = Legendas estilizadas
settings-subtitles-as-made-helper = Arquivos estilizados (ASS) aparecem exatamente como os autores os fizeram.
settings-subtitles-mine-helper = Seu estilo abaixo vale para todos os arquivos. Letreiros e karaokê podem quebrar.
settings-subtitles-size-only-helper = Legendas estilizadas mantêm o visual e usam seu tamanho. As simples usam seu estilo.
settings-subtitles-as-made = Original
settings-subtitles-size-only = Só tamanho
settings-subtitles-mine = Meu
settings-subtitles-font = Fonte
settings-subtitles-font-invalid = Digite o nome de uma fonte, como Noto Sans.
settings-subtitles-bold = Negrito
settings-subtitles-text-colour = Cor do texto
settings-subtitles-colour-invalid = Digite uma cor como #ffcc00.
settings-subtitles-outline = Contorno
settings-subtitles-outline-invalid = A largura é um número, 0 ou mais.
settings-subtitles-outline-colour-invalid = Digite a cor como #000000.
settings-subtitles-outline-helper = A largura, depois a cor.
settings-subtitles-shadow = Sombra
settings-subtitles-shadow-invalid = Um número, 0 ou mais. 0 é sem sombra.
settings-subtitles-shadow-helper = Quanto a sombra cai, 0 para nenhuma.
settings-subtitles-box = Caixa de fundo
settings-subtitles-box-helper = Uma caixa escura atrás do texto. 0 a remove.
settings-subtitles-start-over = Recomeçar
settings-subtitles-start-over-helper = Volta o tamanho, a fonte, as cores, o contorno, a caixa e a posição ao padrão.
settings-subtitles-reset = Redefinir

## SettingsDataTab

settings-data-title = Armazenamento e backup
settings-data-lead = Leve a biblioteca a outro computador, guarde uma cópia ou libere o espaço das imagens.
settings-data-backed-up = Backup salvo em { $path }
settings-data-restore-finished = Restauração concluída
settings-data-images-removed =
    { $count ->
        [one] { $count } imagem removida
       *[other] { $count } imagens removidas
    }
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] { $count } pasta incluída
       *[other] { $count } pastas incluídas
    }
settings-data-summary-series = { $count } séries criadas
settings-data-summary-matches =
    { $count ->
        [one] { $count } vínculo aplicado
       *[other] { $count } vínculos aplicados
    }
settings-data-summary-views =
    { $views ->
        [one] { $views } visualização
       *[other] { $views } visualizações
    } e { $episodes ->
        [one] { $episodes } episódio concluído
       *[other] { $episodes } episódios concluídos
    } mesclados
settings-data-summary-resume-points =
    { $count ->
        [one] { $count } ponto de retomada mesclado
       *[other] { $count } pontos de retomada mesclados
    }
settings-data-summary-accounts =
    { $count ->
        [one] { $count } conta de lista trazida
       *[other] { $count } contas de lista trazidas
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } arquivo usa uma correção sua.
       *[other] { $count } arquivos usam uma correção sua.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } pasta não foi encontrada neste computador.
       *[other] { $count } pastas não foram encontradas neste computador.
    }
settings-data-summary-left-out = Deixados de fora: { $fields }.
settings-data-backup = Backup
settings-data-back-up-library = Fazer backup da biblioteca
settings-data-back-up-helper = Um arquivo com suas pastas, vínculos e o que você viu.
settings-data-last-written = Última gravação em { $path }
settings-data-open-folder = Abrir pasta
settings-data-back-up = Fazer backup
settings-data-private = Incluir dados privados
settings-data-private-helper = Inclui logins do AniList e do MyAnimeList, histórico e preferências. Os logins ficam em texto simples, então guarde o arquivo só para você.
settings-data-restore-from = Restaurar de um backup
settings-data-restore-helper = Lê um backup para esta biblioteca. O que está no disco prevalece; o arquivo completa o resto.
settings-data-restore = Restaurar
settings-data-image-cache = Cache de imagens
settings-data-image-cache-helper = Pôsteres, banners e retratos ficam no disco para a biblioteca abrir offline. Imagens apagadas voltam quando uma página pede.
settings-data-cache-unmeasured = Não foi possível medir o cache de imagens
settings-data-cache-size =
    { $count ->
        [one] { $size } em { $count } imagem
       *[other] { $size } em { $count } imagens
    }
settings-data-cache-empty = Sem imagens em cache
settings-data-cache-unmeasured-helper = Tente de novo em instantes. As imagens em si estão bem.
settings-data-remove-images =
    { $count ->
        [one] Remover { $count } imagem?
       *[other] Remover { $count } imagens?
    }
settings-data-files = Onde o AniBeam guarda os arquivos
settings-data-files-database = Banco de dados da biblioteca
settings-data-files-data = Dados
settings-data-files-settings = Configurações
settings-data-files-cache = Cache
settings-data-backup-filter = Backup do AniBeam (*.json)
settings-data-all-files-filter = Todos os arquivos (*)
settings-data-backing-up = Fazendo backup
settings-data-restoring = Restaurando

## SettingsLibraryTab

settings-library-title = Biblioteca
settings-library-lead = As pastas que o AniBeam observa, e como ele descobre o que há nelas.
settings-library-not-scanned = Ainda não verificada
settings-library-scan-nothing-changed = Nada mudou
settings-library-scan-added = { $count } incluídas
settings-library-scan-changed = { $count } alteradas
settings-library-scan-removed = { $count } removidas
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = Anime: séries, filmes e como se relacionam. O que você assiste é marcado nos seus trackers.
settings-library-tvmaze-about = Outra TV: live action, desenhos, documentários. Nada é marcado em um tracker.
settings-library-tried-first = Tentado primeiro
settings-library-tried-second = Tentado em segundo
settings-library-tried-third = Tentado em terceiro
settings-library-tried-fourth = Tentado em quarto
settings-library-tried-nth = Tentado em { $number }º
settings-library-folders = Pastas
settings-library-folders-helper = Cada pasta dentro destas é uma série. Um arquivo na raiz de uma pasta Movies é um filme.
settings-library-rescan-started = Verificação iniciada
settings-library-load-failed = Não foi possível ler suas pastas
settings-library-load-failed-helper = O AniBeam não conseguiu ler a lista de pastas. Tente de novo em instantes.
settings-library-no-folders = Nenhuma pasta ainda
settings-library-no-folders-helper = Aponte o AniBeam para a pasta onde ficam seus animes. Ele encontra as séries e os filmes sozinho.
settings-library-scanned = Verificada { $when }
settings-library-add-a-folder = Incluir pasta
settings-library-scan-all = Verificar tudo
settings-library-scan-started = Verificação iniciada
settings-library-add-folder = Incluir pasta
settings-library-matching = Vínculo
settings-library-matching-helper = Uma pasta nova é buscada nesta ordem e fica com o primeiro bom vínculo. Um vínculo definido à mão nunca é substituído.
settings-library-try-later = Tentar depois
settings-library-try-first = Tentar primeiro
settings-library-check-matches = Conferir os vínculos
settings-library-check-matches-helper = A página Info da série lista cada série com a info que o AniBeam encontrou, e corrige uma errada.
settings-library-open-show-info = Abrir Info da série
settings-library-hidden-shows = Séries ocultas
settings-library-show-hidden = Mostrar séries ocultas
settings-library-show-hidden-helper = Mostra as séries ocultas em todas as páginas até o AniBeam fechar.
settings-library-hidden-from-airing = Ocultas em No ar agora
settings-library-hidden-from-airing-helper = Séries que você marcou como sem interesse. Cada uma fica fora de No ar agora até você trazê-la de volta.
settings-library-hidden-from-airing-empty = Nada ainda. As séries que você marca como sem interesse em No ar agora ficam aqui, para qualquer uma poder voltar.
settings-library-show-again = Mostrar de novo
settings-library-calendar = Calendário
settings-library-calendar-feed = Horários em um calendário
settings-library-calendar-feed-helper = Mantém um arquivo de calendário com os próximos episódios das suas listas, atualizado quando a grade muda.
settings-library-calendar-file = Arquivo do calendário
settings-library-calendar-file-helper = Assine este arquivo no seu app de calendário.
settings-library-copy = Copiar
settings-library-path-copied = Caminho copiado
settings-library-open-folder = Abrir pasta
settings-library-notifications = Notificações
settings-library-new-episodes = Novos episódios
settings-library-new-episodes-helper = Uma notificação na área de trabalho quando sai um episódio de uma série da sua lista Assistindo. Clique para abrir a série.
settings-library-folder-added = Pasta incluída, verificando

## SettingsAppearanceTab

settings-appearance-language = Idioma
settings-appearance-language-system = Igual ao sistema ({ $language })
settings-appearance-text-smaller = Menor
settings-appearance-text-normal = Normal
settings-appearance-text-large = Grande
settings-appearance-text-larger = Maior
settings-appearance-text-largest = Máximo
settings-appearance-title = Aparência
settings-appearance-lead = As mudanças aparecem na hora, aqui e em todas as páginas. Elas ficam em ~/.config/anibeam/theme.toml, que você também pode editar à mão.
settings-appearance-colours = Cores
settings-appearance-mode = Modo
settings-appearance-mode-dark = Escuro
settings-appearance-mode-light = Claro
settings-appearance-mode-system = Sistema
settings-appearance-colours-from = Cores de
settings-appearance-from-theme-helper = O par de temas abaixo, um escuro e um claro.
settings-appearance-from-desktop-helper = As cores do seu terminal; senão, o esquema de cores e o destaque do desktop.
settings-appearance-from-desktop = Desktop
settings-appearance-from-theme = Tema
settings-appearance-dark-theme = Tema escuro
settings-appearance-light-theme = Tema claro
settings-appearance-themes-helper = Arquivos de tema que você colocar em ~/.config/anibeam/themes aparecem nas duas listas.
settings-appearance-accent = Destaque
settings-appearance-accent-helper = Marca o que está selecionado, tocando ou novo.
settings-appearance-icon = Ícone
settings-appearance-app-icon = Ícone do app
settings-appearance-app-icon-helper = O da janela, da barra de tarefas e do launcher. Escuro troca o amarelo pelo preto.
settings-appearance-icon-default = Padrão
settings-appearance-icon-default-name = Ícone padrão do app
settings-appearance-icon-dark = Escuro
settings-appearance-icon-dark-name = Ícone escuro do app
settings-appearance-size-motion = Tamanho e movimento
settings-appearance-text-size = Tamanho do texto
settings-appearance-text-size-helper = O tamanho das palavras em todo o AniBeam, somado ao tamanho do próprio sistema.
settings-appearance-density = Densidade
settings-appearance-density-helper = O espaço em volta de tudo, de apertado a arejado.
settings-appearance-density-compact = Compacta
settings-appearance-density-normal = Normal
settings-appearance-density-comfortable = Confortável
settings-appearance-poster-size = Tamanho do pôster
settings-appearance-poster-size-helper = Tamanho na biblioteca e no Início.
settings-appearance-poster-small = Pequeno
settings-appearance-poster-medium = Médio
settings-appearance-poster-large = Grande
settings-appearance-out-now = Já saiu em No ar
settings-appearance-out-now-helper = Cards com o banner de cada série, ou uma lista de pôsteres pequenos e títulos, com três linhas.
settings-appearance-out-now-cards = Cards
settings-appearance-out-now-list = Lista
settings-appearance-out-now-grid = Cards de Já saiu
settings-appearance-out-now-grid-helper = Quantos cards aparecem, colunas por linhas, antes de Todos abrir o resto.
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Reduzir movimento
settings-appearance-reduce-motion-helper = Pula a abertura animada da biblioteca e outros movimentos só de enfeite.

## LookPane

look-pane-sample-series = Série de exemplo
look-pane-episode-code = Ep { $number }
look-pane-library = Biblioteca
look-pane-series-count =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    }
look-pane-search = Buscar títulos ou pastas
look-pane-search-hint = / ou Ctrl+K
look-pane-all = Tudo
look-pane-series = Séries
look-pane-movies = Filmes
look-pane-plain = Simples
look-pane-selected = Selecionado
look-pane-errors =
    { $count ->
        [one] { $count } erro
       *[other] { $count } erros
    }
look-pane-type-large = Grande
look-pane-type-normal = Normal
look-pane-type-small = Pequeno, suave
look-pane-sample-switch = Chave de exemplo
look-pane-button = Botão
look-pane-scan = Verificar
look-pane-sample-status =
    { $count ->
        [one] { $count } ep novo em { $title }
       *[other] { $count } eps novos em { $title }
    }

## FeedPage

feed-title = Feed
feed-aired = { $episode } foi ao ar { $when }
feed-downloaded = Arquivo baixado { $when }
feed-countdown-days = { $days } d { $hours } h
feed-countdown-hours = { $hours } h { $minutes } min

feed-countdown-minutes = { $minutes } min
feed-section-this-week = Esta semana
feed-new-episodes =
    { $count ->
        [one] { $count } episódio novo
       *[other] { $count } episódios novos
    }
feed-section-earlier = Antes
feed-latest-out = Último lançado { $when }
feed-section-coming-soon = Em breve
feed-this-week-count = { $count } esta semana
feed-section-everything-else = Todo o resto
feed-out-this-week = { $count } lançados esta semana
feed-nothing-scheduled = Nada programado ainda.
feed-next = Próximo: { $title } em { $countdown }
feed-nothing-new = Nada novo esta semana.
feed-new-this-week =
    { $count ->
        [one] { $count } episódio novo esta semana.
       *[other] { $count } episódios novos esta semana.
    }
feed-sort-recent = Lançados recentemente
feed-sort-upcoming = Em breve
feed-failed-title = O feed não carregou
feed-empty-title = Nada aqui ainda
feed-failed-body = O AniBeam não conseguiu ler o feed agora. Tente de novo em instantes.
feed-empty-body-folders = O feed se enche conforme os episódios vão ao ar e chegam. No ar agora mostra o que passa nesta temporada.
feed-empty-body = Inclua a pasta onde ficam seus animes e o feed se enche com os episódios dela.
feed-add-folder = Incluir pasta
feed-airing-now = No ar agora

## RecommendedPage

recommended-title = Recomendados para você
recommended-ranked-minutes = Ordenado há { $minutes } min
recommended-ranked-hours = Ordenado há { $hours } h
recommended-ranked-days = Ordenado há { $days } d
recommended-scope-name = Recomendados de
recommended-scope-all = Todos
recommended-scope-library = Na biblioteca
recommended-scope-list = Na lista
recommended-ranking = Ordenando
recommended-rank-again = Ordenar de novo
recommended-no-scores-title = Dê nota a algumas séries primeiro
recommended-no-list-title = Nada para aprender ainda
recommended-never-fitted-title = Ordenando suas séries
recommended-nothing-left-title = Não sobrou nada aqui
recommended-no-scores-body = As recomendações aprendem com as notas da sua lista. Dê nota a algumas séries no AniList e elas aparecem aqui.
recommended-no-list-body = Conecte o AniList nas Configurações ou tenha uma lista aqui, e suas notas ensinam esta página.
recommended-never-fitted-body = A primeira ordenação lê o AniList por um ou dois minutos. Ela roda em segundo plano.
recommended-nothing-left-library-body = Tudo na sua biblioteca que combina já foi assistido. Todos mostra séries que você ainda não tem.
recommended-nothing-left-body = Tudo nas suas listas que combina já foi assistido. Todos mostra séries fora das suas listas.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] Movendo 1 série para o { $site }
       *[other] Movendo { $count } séries para o { $site }
    }
move-lists-score = nota { $score }
move-lists-title = Mover sua lista para o { $site }
move-lists-body =
    { $count ->
        [one] Uma série tem
       *[other] { $count } séries têm
    } status ou nota só neste computador. Movê-las as coloca na sua lista do { $site }, onde seu celular e seu perfil as veem. Para estas séries, o que o { $site } tem agora é substituído. Nada é enviado até você clicar em Mover.
move-lists-head-show = Série
move-lists-head-here = Neste computador
move-lists-head-tracker = No { $site } agora
move-lists-nothing = Nada
move-lists-not-on-list = Fora da sua lista
move-lists-later-hint = Depois: Configurações, Contas
move-lists-not-now = Agora não
move-lists-move =
    { $count ->
        [one] Mover 1 série
       *[other] Mover { $count } séries
    }

## StatsPage

stats-title = Estatísticas
stats-month-jan = Jan
stats-month-feb = Fev
stats-month-mar = Mar
stats-month-apr = Abr
stats-month-may = Mai
stats-month-jun = Jun
stats-month-jul = Jul
stats-month-aug = Ago
stats-month-sep = Set
stats-month-oct = Out
stats-month-nov = Nov
stats-month-dec = Dez
stats-nothing-yet = Nada ainda
stats-ranked-episodes =
    { $count ->
        [one] { $count } ep
       *[other] { $count } eps
    }
stats-year-summary = O que você assistiu em { $year }
stats-empty-title = Nada assistido ainda
stats-empty-body = Termine um episódio e ele conta aqui: horas, gêneros e as temporadas que você assistiu.
stats-open-library = Biblioteca
stats-hours-watched = horas assistidas
stats-episodes-label =
    { $count ->
        [one] episódio
       *[other] episódios
    }
stats-series-finished = séries concluídas
stats-none = Nenhum
stats-busiest-month = mês mais ativo
stats-by-month = Por mês
stats-month-hours = { $hours } h
stats-top-genres = Top gêneros
stats-top-studios = Top estúdios
stats-seasons-watched = Temporadas assistidas
stats-recap-title = Meu ano em anime
stats-recap-hours = horas
stats-recap-episodes = episódios
stats-recap-top-genre = gênero favorito
stats-save-image = Salvar como imagem
stats-save-dialog-title = Salvar o resumo
stats-save-filter-png = Imagem PNG (*.png)
stats-recap-saved = Resumo salvo
stats-recap-save-failed = Não foi possível salvar o resumo

## TrackerRow

tracker-row-connected = Conectado como { $user }, sincronizado { $when }
tracker-row-unknown-user = ?
tracker-row-never = nunca
tracker-row-waiting = Esperando você aprovar no navegador
tracker-row-not-connected = Não conectado
tracker-row-open-again = Abrir de novo
tracker-row-open-again-tip = Abrir a página de login de novo
tracker-row-disconnect = Desconectar
tracker-row-disconnect-question = Desconectar { $name }? Seu login será removido.
tracker-row-secret-saved = Há um client secret salvo para este app.
tracker-row-remove-secret = Remover
tracker-row-secret-removed = Client secret removido
tracker-row-add-secret = Incluir client secret
tracker-row-secret-where = É o texto longo em "Client Secret" na página do app em myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Client secret
tracker-row-save-connect = Salvar e conectar

## RemoveSheet

remove-unsubscribe-delete = Deixar de seguir e excluir
remove-unsubscribe-torrents = Deixar de seguir e remover torrents
remove-unsubscribe = Deixar de seguir
remove-delete-files = Remover e excluir arquivos
remove-torrents =
    { $count ->
        [one] Remover o torrent
       *[other] Remover torrents
    }
remove-also-torrents =
    { $count ->
        [one] Remover também o torrent
       *[other] Remover também os { $count } torrents
    }
remove-torrents-go-with-files = Excluir os arquivos leva os torrents junto.
remove-torrents-files-stay = Eles saem do cliente; os arquivos ficam.
remove-also-files = Excluir também os arquivos
remove-files-default-where = o que os torrents baixaram
remove-files-detail-rule = Move { $where } para a lixeira e remove os torrents.
remove-files-detail = Move { $where } para a lixeira.

## NotesSheet

notes-your-tracker = sua lista
notes-title = Notas
notes-placeholder = Onde você parou, quem recomendou, qual release baixar
notes-over-limit = { $count } caracteres acima do limite
notes-saved-to = Salvo no { $sites }

## RecommendationCard

recommendation-card-available = Disponível
recommendation-card-open-library = Abrir { $title } na sua biblioteca
recommendation-card-open-anilist = Abrir { $title } no AniList

## SearchField

search-field-placeholder = Buscar títulos ou pastas
search-field-hint = / ou Ctrl+K
search-field-clear = Limpar a busca

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent }%
recommended-hover-hidden-tags =
    { $count ->
        [one] Uma tag oculta que você adora
       *[other] { $count } tags ocultas que você adora
    }
recommended-hover-because = Porque você assistiu

## PersonCard

person-card-unknown = Desconhecido
person-card-role-main = Principal
person-card-role-supporting = Coadjuvante
person-card-role-background = Figurante
person-card-the-character = o personagem
person-card-open = Abrir { $name } no AniList

## StatusDot

status-dot-on-list = Na sua lista: { $status }

## TrackerNudge

tracker-nudge-title = Conecte sua lista de anime
tracker-nudge-body = O AniBeam marca no AniList ou MyAnimeList o que você assiste.
tracker-nudge-connect-anilist = Conectar AniList
tracker-nudge-connect-mal = Conectar MyAnimeList
tracker-nudge-not-now = Agora não

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } por cento para você
recommended-card-forecast = { $percent }%
recommended-card-in-library = Na biblioteca

## InlineConfirm

inline-confirm-keep = Manter

## Tokens

tokens-cube-watched-tracked = Assistido, registrado online
    .inline = assistido, registrado online
tokens-cube-watched-here = Assistido aqui, ainda não registrado
    .inline = assistido aqui, ainda não registrado
tokens-cube-on-disk = No disco
    .inline = no disco
tokens-cube-downloading = Baixando
    .inline = baixando
tokens-cube-missing = Faltando
    .inline = faltando
tokens-cube-error = Erro
    .inline = erro
tokens-cube-not-downloaded = Não baixado
    .inline = não baixado
tokens-cube-not-aired = Ainda não exibido
    .inline = ainda não exibido
tokens-cube-available = Disponível
    .inline = disponível
tokens-cube-picked = Escolhido para baixar
    .inline = escolhido para baixar
tokens-cube-already-on-disk = Já no disco
    .inline = já no disco
tokens-cube-no-release = Sem release
    .inline = sem release

## Card

card-not-interested = Sem interesse

## PagePlaceholder

page-placeholder-coming-soon = Em breve

## ErrorLine

error-line-hide-details = Ocultar detalhes
error-line-details = Detalhes
error-line-details-tip = Mostrar o que deu errado, palavra por palavra, e copiar
error-line-copied = Copiado.

## Readouts (apps/linux/src/format.rs): horários, contagens regressivas e durações

fmt-just-now = agora mesmo
fmt-now = agora
fmt-ago = há { $span }
fmt-in = em { $span }
fmt-seconds-short = { $n } s
fmt-minutes-short = { $n } min
fmt-hours-short = { $n } h
fmt-days-short = { $n } d
fmt-months-short = { $n ->
    [one] { $n } mês
   *[other] { $n } meses
}
fmt-years-short = { $n } a
fmt-years-months-short = { $years } a { $months } m
fmt-countdown-days = { $d } d { $h } h { $m } min
fmt-countdown-hours = { $h } h { $m } min
fmt-countdown-days-seconds = { $d } d { $h } h { $m } min { $s } s
fmt-countdown-hours-seconds = { $h } h { $m } min { $s } s
fmt-countdown-minutes-seconds = { $m } min { $s } s
fmt-runtime-minutes = { $m } min
fmt-runtime-hours = { $h } h
fmt-runtime-hours-minutes = { $h } h { $m } min
fmt-episode = Episódio
fmt-size-b = { $size } B
fmt-size-kb = { $size } KB
fmt-size-mb = { $size } MB
fmt-size-gb = { $size } GB
fmt-size-tb = { $size } TB
fmt-episode-span = Episódios { $first } a { $through }

## apps/linux/src/problem.rs: cada erro de chamada em palavras simples, o que houve e o que fazer

problem-report = Tente de novo. Se continuar acontecendo, relate pelo log de atividade.
problem-try-later = Tente de novo mais tarde.
problem-helper-unreachable = O ajudante de busca não respondeu.
problem-helper-unauthorized = O ajudante de busca recusou o login.
problem-helper-not-running = O ajudante de busca não está rodando.
problem-helper-missing = O ajudante de busca não está instalado.
problem-helper-failed = O ajudante de busca não conseguiu fazer isso.
problem-helper-no-login = O ajudante de busca não consegue entrar no cliente de torrent.
problem-helper-no-login-next = Salve a senha do cliente de torrent nas Configurações, em Downloads.
problem-torrent-unreachable = O cliente de torrent não respondeu.
problem-torrent-unauthorized = O cliente de torrent recusou o login.
problem-torrent-not-running = O cliente de torrent não está rodando.
problem-torrent-missing = O cliente de torrent não está instalado.
problem-torrent-failed = O cliente de torrent não conseguiu fazer isso.
problem-downloads-unreachable-next = Confira se ele está rodando e se o endereço está certo nas Configurações, em Downloads.
problem-downloads-unauthorized-next = Confira o usuário e a senha nas Configurações, em Downloads.
problem-downloads-not-running-next = Inicie-o nas Configurações, em Downloads.
problem-downloads-missing-next = Configurações, em Downloads, explica como configurá-lo.
problem-trash = O { -app-name } não conseguiu mover os arquivos para a lixeira.
problem-trash-next = Confira se os arquivos são seus e se o disco não é somente leitura.
problem-source-list = Essa lista de fontes não funcionou.
problem-source-list-next = Confira o endereço dela nas Configurações, em Downloads, Listas de fontes.
problem-remote-unreachable = { $name } não respondeu.
problem-remote-unreachable-next = O site pode estar fora do ar. Tente de novo mais tarde, ou busque em outra fonte.
problem-remote-unauthorized = { $name } recusou a chave informada.
problem-remote-unauthorized-next = Mude a chave dela nas Configurações, em Downloads, Listas de fontes.
problem-remote-failed = { $name } não conseguiu fazer isso.
problem-remote-failed-next = Tente de novo mais tarde, ou busque em outra fonte.
problem-provider-sign-in = O { $name } recusou o login do { -app-name }.
problem-provider-sign-in-next = Conecte de novo nas Configurações, em Contas.
problem-provider-turned-away = O { $name } recusou o { -app-name }.
problem-provider-busy = O { $name } está ocupado agora.
problem-provider-busy-next = Espere um minuto e tente de novo.
problem-provider-no-answer = O { $name } não respondeu.
problem-provider-no-answer-next = Confira sua conexão com a internet e tente de novo.
problem-provider-trouble = O { $name } está com problemas agora.
problem-provider-trouble-next = Tente de novo em alguns minutos.
problem-provider-rejected = O { $name } não aceitou isso.
problem-not-found = O { -app-name } não encontrou mais isso.
problem-not-found-next = Pode ter sido removido. Tente de novo para recarregar a página.
problem-invalid-next = Confira e tente de novo.
problem-unavailable = Essa pasta não pode ser acessada agora.
problem-unavailable-next = Conecte o disco dela ou confira a pasta nas Configurações, em Biblioteca.
problem-not-connected = { $tracker } não está conectado.
problem-not-connected-next = Conecte nas Configurações, em Contas.
problem-io = O { -app-name } não conseguiu ler ou gravar um arquivo.
problem-io-next = Confira se o disco tem espaço e se a pasta é sua, e tente de novo.
problem-storage = O { -app-name } não conseguiu salvar sua biblioteca.
problem-storage-next = Confira se o disco tem espaço e tente de novo.
problem-keyring = O chaveiro de senhas do sistema não respondeu.
problem-keyring-next = Desbloqueie o chaveiro do desktop e tente de novo.
problem-unsupported = O { -app-name } ainda não faz isso aqui.
problem-version = Este backup vem de um { -app-name } mais novo.
problem-version-next = Atualize o { -app-name } nas Configurações, em Sobre, e importe de novo.
problem-internal = Algo deu errado dentro do { -app-name }.
problem-refused-hidden = Essa série está oculta.
problem-refused-hidden-next = Mostre-a de novo pela página dela primeiro.
problem-refused-no-match = Essa série ainda não tem vínculo.
problem-refused-no-match-next = Vincule em Info da série primeiro.
problem-refused-not-newer = Sua lista já passou deste episódio.
problem-refused-extra = Esse arquivo é um extra, não um episódio.
problem-refused-unmatched = Esse arquivo não está ligado a um episódio.
problem-refused-unmatched-next = Corrija pela página da série, em Correções.
problem-refused-on-disk = Esse arquivo está só neste computador.

## apps/linux/src/library_view.rs: os títulos das seções da biblioteca e o fato embaixo de cada uma
## (Abandonado, Em pausa e Assistindo reutilizam status-* do core.ftl)

library-view-airing = No ar
library-view-no-date = Sem data
library-view-type-tv = TV
library-view-type-movies = Filmes
library-view-type-extras = OVA, ONA e especiais
library-view-type-music = Música
library-view-type-unknown = Desconhecido
library-view-no-genre = Sem vínculo
library-view-state-caught-up = Em dia
library-view-state-not-started = Não iniciadas
library-view-next-episode = Próximo episódio em { $span }
library-view-countdown-days = { $d } d { $h } h
library-view-countdown-hours = { $h } h { $m } min
library-view-countdown-minutes = { $m } min
library-view-in-progress = { $count } em andamento
library-view-not-started = { $count } não iniciadas
library-view-all-watched = Tudo assistido
library-view-on-disk =
    { $count ->
        [one] { $count } episódio
       *[other] { $count } episódios
    } no disco
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } arquivo ainda
       *[other] { $count } arquivos ainda
    } no disco

## apps/linux/src/dbus/notify.rs: a notificação de desktop para um episódio novo
## (o botão Abrir reutiliza action-open do core.ftl)

notify-episode-out = Saiu o episódio { $episode } de { $title }
notify-click-open = Clique para abrir no { -app-name }
notify-click-get = Clique para baixar os episódios no { -app-name }
notify-get-episodes = Baixar episódios

## apps/linux/src/nowplaying.rs: a segunda linha de um widget de mídia enquanto um episódio toca

now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: por que a tela fica ligada, como o painel de energia mostra

inhibit-reason = Reproduzindo um episódio

## apps/linux/src/bridge/door.rs: por que um valor enviado por uma página não pode ser usado

door-not-a-choice = { $value } não é uma opção válida
door-tokens-op-empty = uma op tokens precisa de tokens
door-not-a-refine-op = { $value } não é uma op de refinamento
door-limit-too-low = precisa ser pelo menos 1, veio { $limit }
door-not-a-torrent-action = { $value } não é uma ação de torrent

## apps/linux/src/bridge/shell.rs: a pasta inicial da primeira execução e o aviso de atualização

shell-starter-folder-failed = Não foi possível criar { $folder }: { $error }
shell-update-gone = O { -app-name } { $version } não está mais disponível


## Watch together (#846)

onboarding-step-together = Assistir juntos
onboarding-title-together = Assista com amigos
onboarding-body-together = Crie uma sala no player e envie o código. Cada um reproduz a própria cópia do episódio, e reproduzir, pausar e pular ficam sincronizados para todos.
onboarding-ready-together = Assistir juntos
onboarding-ready-together-as = Nas salas você aparece como { $name }.
onboarding-ready-together-none = Ainda sem nome. O painel da sala pergunta na primeira vez.
onboarding-together-you = Você
onboarding-together-code = Código da sala
onboarding-together-episode = Frieren · Episódio 3
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = Seu nome nas salas
rail-room = Assistir juntos (Ctrl+J)
rail-room-in = Em uma sala de assistir juntos (Ctrl+J)
rail-join-room = Entrar na sala
rail-in-room = Na sala
player-chrome-watch-together = Assistir juntos
player-chrome-room = Sala { $code } · { $count ->
    [one] 1 pessoa
   *[other] { $count } pessoas
}
room-name-empty = Digite o nome que os outros na sala vão ver.
room-name-bad = Esse nome não funciona. Use de 1 a 32 caracteres.
room-error-not-found = Nenhuma sala tem esse código. Confira com quem enviou; a sala pode ter sido fechada.
room-error-full = Essa sala está cheia. As salas comportam 10 pessoas.
room-error-unreachable = Não foi possível alcançar o servidor de salas. Confira sua conexão e tente de novo.
room-error-refused = O servidor de salas recusou. Tente de novo em um minuto.
room-error-proto = Essa sala roda em uma versão mais nova das salas. Atualize o { -app-name } para entrar.
room-error-not-in-room = Você não está mais nessa sala. Crie ou entre em uma de novo.
room-error-bad-code = Um código de sala tem 6 letras e dígitos, como K7QM2X.
room-error-bad-relay = O servidor de salas em Configurações, Reprodução não é um endereço web. Corrija lá e tente de novo.
room-panel-title = Assistir juntos
room-panel-close = Fechar
room-panel-connected = Conectado
room-panel-reconnecting = Reconectando
room-panel-name-label = Seu nome nas salas
room-panel-name-placeholder = Seu nome
room-panel-name-start = Criar a sala
room-panel-name-join = Entrar na sala
room-panel-start = Criar uma sala
room-panel-or-join = Ou entre em uma com o código
room-panel-join-label = Código da sala
room-panel-join = Entrar
room-panel-you-are = Você aparece como { $name }
room-panel-change-name = Mudar
room-panel-code-label = Código da sala
room-panel-copy = Copiar link
room-panel-link-copied = Link da sala copiado
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = Este episódio não está na sua biblioteca, então você ainda não pode acompanhar.
room-panel-open-episode = Abrir o episódio
room-panel-people = { $count ->
    [one] 1 pessoa
   *[other] { $count } pessoas
}
room-panel-host = Anfitrião
room-panel-you = Você
room-panel-ready = Pronto
room-panel-opening = Abrindo
room-panel-missing = Não tem
room-panel-offset = Seu ajuste
room-panel-offset-earlier = Meio segundo antes
room-panel-offset-later = Meio segundo depois
room-panel-seconds = { $value } s
room-panel-suggested = As aberturas sugerem { $offset }
room-panel-use-suggested = Usar
room-panel-leave = Sair da sala
player-room-speed = A velocidade segue a sala
settings-find-room-name =
    .words = assistir juntos sala nome amigos
settings-find-room-relay =
    .words = assistir juntos sala relay servidor
key-help-key-ctrl-j = Ctrl+J
key-help-room = Assistir juntos
settings-playback-together = Assistir juntos
settings-playback-room-name = Seu nome nas salas de assistir juntos
settings-playback-room-name-placeholder = Seu nome
settings-playback-room-relay = Servidor de salas
settings-playback-room-relay-helper = As salas são criadas aqui. Mude só para um servidor que você mesmo roda.
