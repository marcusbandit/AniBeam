# pt-BR translation of en/core.ftl. An ID missing here is shown in English.

## O host na tailnet

tailnet-person-no-name = Sem nome no Tailscale

## Contagens

count-episodes =
    { $count ->
        [one] { $count } episódio
       *[other] { $count } episódios
    }
count-shows =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    }

## Ações que toda tela usa

action-cancel = Cancelar
action-close = Fechar
action-try-again = Tentar de novo
action-remove = Remover
action-delete = Excluir
action-open = Abrir
action-save = Salvar
action-apply = Aplicar
action-clear = Limpar
action-refresh = Atualizar
action-play = Assistir
action-connect = Conectar
action-start = Iniciar
action-back = Voltar
action-see-all = Ver tudo
action-open-series = Abrir série
action-open-settings = Abrir configurações

## As listas de um tracker

status-watching = Assistindo
status-completed = Concluído
status-paused = Pausado
status-on-hold = Em pausa
status-dropped = Abandonado
status-planning = Planejando
status-rewatching = Reassistindo

## Episódios

episode-number = Episódio { $number }

## As palavras do AniList para uma série: formato, exibição, relação entre séries

anilist-format-tv = TV
anilist-format-tv-short = TV curta
anilist-format-film = Filme
anilist-format-special = Especial
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Música
anilist-status-finished = Concluída
anilist-status-airing = Em exibição
anilist-status-not-yet-aired = Ainda não exibida
anilist-status-cancelled = Cancelada
anilist-status-hiatus = Em hiato
anilist-relation-adaptation = Adaptação
anilist-relation-prequel = Prequela
anilist-relation-sequel = Sequência
anilist-relation-parent = História principal
anilist-relation-side-story = História paralela
anilist-relation-character = Personagem
anilist-relation-summary = Resumo
anilist-relation-alternative = Alternativa
anilist-relation-spin-off = Derivado
anilist-relation-other = Outro
anilist-relation-source = Original
anilist-relation-compilation = Compilação
anilist-relation-contains = Contém

## O log de atividade: um problema sem palavras próprias

event-problem-library = Algo deu errado com a sua biblioteca. As opções de desenvolvedor têm os detalhes.
event-problem-metadata = Algo deu errado com as informações da série. As opções de desenvolvedor têm os detalhes.
event-problem-trackers = Algo deu errado com as suas listas. As opções de desenvolvedor têm os detalhes.
event-problem-franchise = Algo deu errado com as séries relacionadas. As opções de desenvolvedor têm os detalhes.
event-problem-playback = Algo deu errado com a reprodução. As opções de desenvolvedor têm os detalhes.
event-problem-store = Algo deu errado ao salvar os seus dados. As opções de desenvolvedor têm os detalhes.
event-problem-system = Algo deu errado com o Anibeam. As opções de desenvolvedor têm os detalhes.
event-problem-downloads = Algo deu errado com os downloads. As opções de desenvolvedor têm os detalhes.

## Erros (contract::CoreError)

core-error-not-found-source = Pasta { $id } não encontrada.
core-error-not-found-series = Série { $id } não encontrada.
core-error-not-found-file = Arquivo { $id } não encontrado.
core-error-not-found-session = Sessão de reprodução { $id } não encontrada.
core-error-not-found-job = Tarefa { $id } não encontrada.
core-error-invalid = { $field } inválido: { $message }.
core-error-invalid-ended = { $field } inválido: { $message }
core-error-unavailable = Esta pasta não está disponível: { $path }
core-error-not-connected = { $tracker } não está conectado.
core-error-refused-hidden = Não é possível: a série está oculta.
core-error-refused-no-match = Não é possível: a série ainda não tem vínculo; vincule em Informações da série.
core-error-refused-not-newer = Não é possível: a sua lista já passou deste episódio.
core-error-refused-extra = Não é possível: isto é um extra.
core-error-refused-unmatched = Não é possível: o arquivo não está ligado a um episódio.
core-error-refused-on-disk = Não é possível: o arquivo está só no disco.
# Why a list was not updated, read after "Not tracked:" or a tracker's name, on every shell's player
refusal-hidden = a série está oculta
refusal-no-match = a série ainda não tem vínculo; vincule em Info da série
refusal-not-newer = sua lista já passou deste episódio
refusal-extra = isto é um extra
refusal-unmatched = o arquivo não está ligado a um episódio
refusal-on-disk = o arquivo está só no disco
core-error-provider = { $provider } não respondeu: { $message }
core-error-io = Não foi possível ler ou gravar um arquivo: { $message }
core-error-storage = Não foi possível salvar a sua biblioteca: { $message }
core-error-keyring = Não foi possível acessar o chaveiro de senhas do sistema: { $message }
core-error-unsupported = { $what } ainda não é suportado.
core-error-version = Esta exportação é de um Anibeam mais novo (formato { $found }). Atualize o Anibeam para importá-la.
core-error-internal = O Anibeam teve um erro interno: { $message }
core-error-service = { $service }: { $message }

## releases/src/repos/mod.rs

core-repos-paste-address = cole o endereço de uma lista de fontes
core-repos-must-be-https = o endereço precisa usar https
core-repos-unreachable = não foi possível acessar a lista de fontes: { $message }
core-repos-not-found = nada encontrado nesse endereço (um repo privado do GitHub também retorna 404)
core-repos-http-status = a lista de fontes retornou HTTP { $status }
core-repos-unreadable = não foi possível ler { $path }: { $error }
core-repos-no-such-endpoint = nenhuma fonte instalada com esse ID
core-repos-key-changed = esta lista de fontes agora é assinada com uma chave diferente da que você confiou. Remova e inclua de novo para aceitar a nova chave.
core-repos-stale = a lista de fontes retornou um índice mais antigo (número { $serial } depois de { $kept }); mantendo o mais novo
core-repos-key-not-shown = a chave de assinatura não confere com a que foi mostrada a você. Inclua a lista de fontes de novo e compare a impressão digital.
core-repos-repo-added = Lista de fontes incluída.
core-repos-repo-removed = Lista de fontes removida.
core-repos-no-field = { $name } não tem o campo { $key }
core-repos-needs-field = { $name } precisa de { $label }
core-repos-host-only = { $name }: { $label } precisa ser só um host ou host:porta, como 192.0.2.10:9696
core-repos-not-added = essa lista de fontes não foi incluída
core-repos-not-offered = { $repo } não oferece { $id }
core-repos-already-installed = { $name } já está instalado
core-repos-taken-by-you = já existe uma fonte chamada { $id } instalada (incluída manualmente); remova-a primeiro
core-repos-taken-by-repo = já existe uma fonte chamada { $id } instalada (de outra lista de fontes); remova-a primeiro
core-repos-source-added = Fonte de download incluída.
core-repos-source-updated = Fonte de download atualizada.
core-repos-source-removed = Fonte de download removida.
core-repos-source-changed = Fonte de download editada.
core-repos-give-name = digite um nome
core-repos-put-query = inclua {"{"}query{"}"} no endereço onde vai o termo de busca
core-repos-name-taken = já existe uma fonte chamada { $id } instalada
core-repos-only-by-hand = só fontes incluídas manualmente podem ser editadas
core-repos-keep-key = mantenha {"{"}{ $key }{"}"} no endereço e atualize o valor em Mudar { $label }, ou remova e inclua o site de novo sem ele
core-repos-note-kind = o anirss não busca em fontes do tipo { $kind }, então o desktop a ignora; o app do celular consegue buscar nela
core-repos-note-by-hand = o anirss tem o próprio { $id } configurado manualmente; o desktop busca nesse
core-repos-note-theirs = Você gerencia esta instância do anirss, então o { -app-name } não edita a config dela: inclua { $id } na config do anirss para buscar nela pelo desktop
core-repos-note-sandbox = O modo sandbox não grava na config do anirss, então o desktop não vai buscar em { $id } aqui
core-repos-note-old = o anirss { $version } não aceita fontes do { -app-name }; atualize para { $min } ou mais novo para buscar aqui pelo desktop

## releases/src/repos/index.rs

core-repos-key-not-ed25519 = a chave do índice não é uma chave ed25519
core-repos-key-not-base64 = a chave do índice não tem 32 bytes em base64
core-repos-key-invalid = a chave do índice não é uma chave ed25519 válida
core-repos-too-large = o arquivo é grande demais para ser uma lista de fontes
core-repos-not-a-repo = isto não é uma lista de fontes válida do { -app-name }
core-repos-format-version = esta lista de fontes usa a versão de formato { $found }, mas o { -app-name } lê a versão { $reads }. Atualize o { -app-name }.
core-repos-payload-not-base64 = o conteúdo da lista de fontes não está em base64
core-repos-signature-malformed = a assinatura da lista de fontes está malformada
core-repos-index-unreadable = não foi possível ler o índice da lista de fontes: { $error }
core-repos-signature-mismatch = a assinatura da lista de fontes não confere com o conteúdo
core-repos-check-scheme = { $id }: o endereço precisa começar com https://
core-repos-check-scheme-http = { $id }: o endereço precisa começar com http:// ou https://
core-repos-check-auth-key = { $id }: chave de autenticação inválida { $key }
core-repos-check-host-torznab = { $id }: só um endereço torznab pode usar {"{"}{ $key }{"}"} como host
core-repos-check-host-placeholder = { $id }: marcador não permitido no host

## playback/src/tracks.rs

core-playback-track-number = Faixa { $number }
core-playback-track-file = { $label } (arquivo)
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = Inglês
core-playback-lang-japanese = Japonês
core-playback-lang-spanish = Espanhol
core-playback-lang-portuguese = Português
core-playback-lang-french = Francês
core-playback-lang-german = Alemão
core-playback-lang-italian = Italiano
core-playback-lang-russian = Russo
core-playback-lang-chinese = Chinês
core-playback-lang-korean = Coreano
core-playback-lang-arabic = Árabe
core-playback-lang-hindi = Hindi
core-playback-lang-indonesian = Indonésio
core-playback-lang-malay = Malaio
core-playback-lang-thai = Tailandês
core-playback-lang-vietnamese = Vietnamita
core-playback-lang-polish = Polonês
core-playback-lang-turkish = Turco
core-playback-lang-dutch = Holandês
core-playback-lang-swedish = Sueco
core-playback-lang-norwegian = Norueguês
core-playback-lang-danish = Dinamarquês
core-playback-lang-finnish = Finlandês
core-playback-lang-hebrew = Hebraico
core-playback-lang-ukrainian = Ucraniano
core-playback-lang-czech = Tcheco
core-playback-lang-hungarian = Húngaro
core-playback-lang-greek = Grego
core-playback-lang-romanian = Romeno
core-playback-lang-filipino = Filipino
core-playback-region-brazil = Brasil
core-playback-region-portugal = Portugal
core-playback-region-latin-america = América Latina
core-playback-region-spain = Espanha
core-playback-region-simplified = Simplificado
core-playback-region-traditional = Tradicional

## contract/src/enums.rs

core-jobs-scan = Verificando suas pastas
    .stopped = A verificação das pastas parou com um problema.
core-jobs-auto-match = Vinculando séries
    .stopped = O vínculo das séries parou com um problema.
core-jobs-search = Buscando uma série
    .stopped = A busca da série parou com um problema.
core-jobs-resolve-link = Lendo um link
    .stopped = A leitura do link parou com um problema.
core-jobs-apply-match = Vinculando uma série
    .stopped = O vínculo da série parou com um problema.
core-jobs-refresh = Atualizando uma série
    .stopped = A atualização da série parou com um problema.
core-jobs-refresh-all = Atualizando informações das séries
    .stopped = A atualização das informações parou com um problema.
core-jobs-refresh-airing = Verificando o que está no ar
    .stopped = A verificação do que está no ar parou com um problema.
core-jobs-refresh-imdb = Atualizando notas
    .stopped = A atualização das notas parou com um problema.
core-jobs-clear-images = Limpando imagens
    .stopped = A limpeza das imagens parou com um problema.
core-jobs-fill-images = Baixando imagens
    .stopped = O download das imagens parou com um problema.
core-jobs-connect-tracker = Conectando sua lista
    .stopped = A conexão da lista parou com um problema.
core-jobs-mark = Marcando episódio como assistido
    .stopped = A marcação do episódio parou com um problema.
core-jobs-set-progress = Definindo seu progresso
    .stopped = A definição do progresso parou com um problema.
core-jobs-score = Salvando sua nota
    .stopped = Salvar a nota parou com um problema.
core-jobs-refresh-progress = Lendo o progresso da lista
    .stopped = A leitura do progresso parou com um problema.
core-jobs-refresh-watching = Lendo sua lista Assistindo
    .stopped = A leitura da lista Assistindo parou com um problema.
core-jobs-refresh-airing-now = Verificando esta temporada
    .stopped = A verificação da temporada parou com um problema.
core-jobs-refresh-recommended = Buscando séries para você
    .stopped = A busca de séries para você parou com um problema.
core-jobs-read-show-page = Lendo uma série
    .stopped = A leitura da série parou com um problema.
core-jobs-on-hold = Movendo séries paradas para Em pausa
    .stopped = Mover séries para Em pausa parou com um problema.
core-jobs-set-status = Mudando um status da lista
    .stopped = A mudança de status parou com um problema.
core-jobs-move-lists = Movendo sua lista para o tracker
    .stopped = Mover a lista parou com um problema.
core-jobs-notes = Salvando suas notas
    .stopped = Salvar as notas parou com um problema.
core-jobs-crawl = Buscando séries relacionadas
    .stopped = A busca de séries relacionadas parou com um problema.
core-jobs-skip-windows = Encontrando a abertura e o encerramento
    .stopped = Encontrar a abertura e o encerramento parou com um problema.
core-jobs-export = Salvando sua biblioteca em um arquivo
    .stopped = Salvar a biblioteca em arquivo parou com um problema.
core-jobs-import = Carregando sua biblioteca de um arquivo
    .stopped = Carregar a biblioteca do arquivo parou com um problema.
core-jobs-subscriptions = Verificando as séries seguidas
    .stopped = A verificação das séries seguidas parou com um problema.
core-jobs-connect-downloads = Conectando o app de download
    .stopped = A conexão do app de download parou com um problema.
core-jobs-search-releases = Procurando episódios
    .stopped = A procura de episódios parou com um problema.
core-jobs-refine-releases = Refinando episódios
    .stopped = O refinamento dos episódios parou com um problema.
core-jobs-subscribe = Seguindo uma série
    .stopped = Seguir a série parou com um problema.
core-jobs-download = Incluindo downloads
    .stopped = Incluir downloads parou com um problema.
core-jobs-remove-subscription = Deixando de seguir uma série
    .stopped = Deixar de seguir a série parou com um problema.
core-jobs-control-torrents = Alterando seus downloads
    .stopped = A alteração dos downloads parou com um problema.
core-jobs-test-torrent-client = Testando o app de download
    .stopped = O teste do app de download parou com um problema.
core-jobs-remove-show = Removendo uma série
    .stopped = A remoção da série parou com um problema.
core-jobs-repair-subscription = Consertando uma série seguida
    .stopped = O conserto da série seguida parou com um problema.
core-jobs-check-endpoints = Verificando suas fontes de download
    .stopped = A verificação das fontes de download parou com um problema.
core-jobs-start-endpoint = Iniciando uma fonte de download
    .stopped = Iniciar a fonte de download parou com um problema.
core-jobs-schedule-agent = Pedindo a programação ao seu agente de código
    .stopped = Pedir a programação ao agente parou com um problema.

## metadata/src/airing_now.rs

core-airing-now-now = No ar agora
core-airing-now-winter = Inverno { $year }
core-airing-now-spring = Primavera { $year }
core-airing-now-summer = Verão { $year }
core-airing-now-fall = Outono { $year }
core-airing-now-planned = { $title } incluída na sua lista Planejando no { $site }.
core-airing-now-planned-both = { $title } incluída na sua lista Planejando no { $site } e no { $other }.
core-airing-now-already-on =
    { $title } já está na sua lista do { $site } como { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }.
core-airing-now-plan-failed = Não foi possível incluir { $title } na sua lista Planejando. Tente de novo mais tarde.

## metadata/src/aired.rs

core-aired-episode-out = Saiu o episódio { $number } de { $title }.

## playback/src/cast/mod.rs

core-cast-connecting = Transmitindo para { $device }.
core-cast-ended = Transmissão para { $device } concluída.
core-cast-stopped = Transmissão para { $device } interrompida.
core-cast-failed = A transmissão para { $device } falhou. Confira se ele está na mesma rede.
core-cast-not-built = A transmissão não faz parte desta versão
core-cast-serve-failed = não foi possível servir o episódio: { $error }
core-cast-device-gone = Esse dispositivo não está mais na rede
core-cast-not-a-playlist = não é um endereço de playlist
core-cast-nothing = Nada sendo transmitido
core-cast-still-starting = O dispositivo ainda está iniciando
core-cast-no-route = este dispositivo não tem rota até a TV
core-cast-no-control = a TV não deu um endereço de controle
core-cast-not-handed = o outro computador não conseguiu enviar o episódio: { $error }
core-cast-empty-playlist = o outro computador enviou uma playlist vazia
core-cast-refused-both = { $error }; o arquivo original também foi recusado ({ $first })
core-cast-chromecast-stopped = o Chromecast parou
core-cast-hevc = { $error }. Este episódio usa HEVC, que só Chromecasts mais novos (com Google TV ou Ultra) suportam

## playback/src/cast/castv2.rs

core-cast-chromecast-unreachable = não foi possível acessar o Chromecast: { $error }
core-cast-chromecast-broke = a conexão com o Chromecast caiu: { $error }
core-cast-chromecast-no-player = o Chromecast não iniciou o player de mídia
core-cast-chromecast-refused-player = o Chromecast não conseguiu iniciar o player de mídia ({ $reason })
core-cast-no-reason = sem motivo
core-cast-chromecast-cannot-play = o Chromecast não conseguiu reproduzir este arquivo ({ $kind })
core-cast-chromecast-error = o Chromecast parou com um erro ao reproduzir este arquivo

## playback/src/cast/dlna.rs

core-cast-tv-no-answer = a TV não respondeu: { $error }
core-cast-tv-refused = a TV recusou { $action }
core-cast-tv-refused-code = a TV recusou { $action } ({ $code })

## playback/src/cast/media.rs

core-cast-no-video = Este arquivo não tem vídeo.
core-cast-hi10p = Este episódio usa H.264 de 10 bits (Hi10P), que TVs e Chromecasts não conseguem decodificar. Transmiti-lo exige conversão, o que ainda não é suportado.
core-cast-video-codec = O vídeo deste episódio ({ $codec }) não pode ser transmitido.
core-cast-audio-codec = O áudio deste episódio ({ $codec }) não pode ser transmitido.

## downloads/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Download de um episódio de { $folder } iniciado.
       *[other] Download de { $count } episódios de { $folder } iniciado.
    }
core-phone-download-partial = Nem todos os releases de { $folder } puderam ser incluídos: { $error }
core-phone-subscribed = Seguindo { $name }.
core-phone-rule-stopped = { $name } não está verificando novos episódios: a fonte foi removida ou precisa da chave de novo.
core-phone-new-episode = Um novo episódio de { $name } está baixando.
core-phone-name-taken = { $name }: outra fonte já tem esse nome
core-phone-nothing-to-download = nada para baixar
core-phone-over-cap = isto passaria do limite de espaço do { -app-name } ({ $used } de { $cap } GB usados ou na fila); exclua episódios ou aumente o limite
core-phone-empty-search = a busca está vazia
core-phone-no-results = nenhum resultado para { $query }
core-phone-own-engine = um motor de torrent próprio

## downloads/src/phone/check.rs

core-phone-check-connected = conectado
core-phone-check-refused = recusado: { $error }
core-phone-check-dropped = sem resposta (descartado)
core-phone-check-finding = { $label }: resolvendo o endereço
core-phone-check-failed = falhou: { $error }
core-phone-check-no-answer = sem resposta
core-phone-check-private = { $label }: resolvendo de forma privada
core-phone-check-skipped-address = pulado, endereço IP
core-phone-check-skipped-home = pulado, nome da rede local
core-phone-check-no-record = sem registro público, usando o DNS da rede
core-phone-check-no-resolver = nenhum resolvedor DNS respondeu
core-phone-check-reaching = { $label }: conectando a { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } bytes
core-phone-check-setup = configuração

## playback/src/session.rs

core-playback-not-saved = Seu progresso em { $folder } não foi salvo. Vincule a série primeiro.
core-playback-still-downloading = o episódio { $number } ainda está baixando, { $done }% concluído

## playback/src/skip.rs

core-playback-bad-duration = a duração precisa ser um número positivo de segundos

## core/src/agent.rs

core-agent-no-answer = o agente não retornou resposta
core-agent-searching = Buscando { $query } na web
core-agent-searching-schedule = Buscando a programação na web
core-agent-reading = Lendo { $url }
core-agent-reading-page = Lendo uma página
core-agent-using = Usando { $tool }
core-agent-none = um agente de código neste computador
core-agent-ended = { $name } terminou sem resposta
core-agent-too-long = { $name } demorou demais

## core/src/core.rs

core-core-guest-trackers = as contas de lista naquele computador são do dono dele, então este dispositivo não pode atualizá-las
core-core-guest-not-interested = Sem interesse naquele computador pertence ao dono dele, então este dispositivo não pode alterá-lo
core-core-guest-recommended = as recomendações naquele computador são aprendidas com a lista do dono dele
core-core-nothing-picked = nada selecionado

## foundation/src/prefs.rs

core-prefs-not-a-provider = { $provider } não é um provedor de vínculo
core-prefs-provider-count = { $provider } aparece { $count } vezes; cada provedor só pode aparecer uma vez
core-prefs-outside = { $value } precisa estar entre { $low } e { $high }
core-prefs-empty = vazio

## metadata/src/recommended/mod.rs

core-recommended-ranking = Ordenando séries para você

## metadata/src/recommended/sync.rs

core-recommended-finding-people = Encontrando pessoas com o seu gosto
core-recommended-reading-lists = Lendo as listas delas
core-recommended-reading-yours = Lendo as suas séries
core-recommended-reading-suggestions = Lendo séries para sugerir

## metadata/src/recommended/page.rs

core-recommended-no-anime = O { -anilist } não tem o anime { $id }

## host/src/lib.rs

core-host-off = Desativado
core-host-starting = Iniciando
core-host-serving = Servindo em { $addresses }
core-host-no-tailscale = O Tailscale não está conectado
core-host-cannot-listen = Não foi possível escutar
core-host-port-taken = A porta { $port } está em uso
core-host-cannot-listen-on = Não foi possível escutar em { $address }: { $error }

## host/src/addr.rs

core-host-not-an-address = { $text } não é um endereço com porta válido

## host/src/keys.rs

core-host-not-a-key = { $key } não é uma chave de dispositivo

## net/src/limiter.rs

core-net-timed-out = tempo esgotado após { $ms } ms
core-net-rate-limited = { $provider } limitou as requisições

## net/src/lib.rs

core-net-waiting = { $host }: tempo esgotado há pouco; o { -app-name } vai esperar { $seconds } s antes de tentar de novo

## net/src/anilist.rs

core-net-no-data = sem dados

## core/src/update/install.rs

core-update-no-download = O { -app-name } { $version } não tem download para este computador
core-update-cannot-make = Não foi possível criar { $path }: { $error }
core-update-cannot-mark = Não foi possível tornar o novo AppImage executável: { $error }
core-update-cannot-replace = Não foi possível substituir { $path }: { $error }
core-update-not-here = Esta instalação não consegue se atualizar aqui
core-update-cannot-start = Não foi possível iniciar o download: { $error }
core-update-did-not-start = O download não iniciou: { $error }
core-update-cannot-write = Não foi possível gravar { $path }: { $error }
core-update-stopped = O download parou: { $error }
core-update-cancelled = Cancelado
core-update-ended-early = O download terminou antes da hora ({ $got } de { $size } bytes)
core-update-checksum = O download não confere com o checksum do GitHub
core-update-cannot-move = Não foi possível mover o download para o lugar: { $error }
core-update-cannot-open-dmg = Não foi possível abrir o DMG: { $error }
core-update-cannot-read-dmg = Não foi possível ler o DMG: { $error }
core-update-no-app = O DMG não contém nenhum app
core-update-cannot-copy = Não foi possível copiar o novo app ao lado deste: { $error }

## metadata/src/calendar.rs

core-calendar-season-episode = { $title } - Temporada { $season } Episódio { $number }
core-calendar-episode = { $title } - Episódio { $number }

## foundation/src/paths.rs

core-paths-no-home = sem pasta pessoal

## core/src/first_run.rs

core-first-run-not-absolute = a pasta de vídeos precisa ser um caminho absoluto

## playback/src/presence/mod.rs

core-presence-extra = Extra
core-presence-room = Assistindo juntos
core-presence-in-room = Em uma sala
core-presence-no-room = Esse convite do Discord ainda não tem sala. Peça para abrirem uma.

# As palavras do core para o grupo da biblioteca (#798).

## trackers/src/accounts.rs

core-trackers-no-client-id = Nenhum client ID definido para { $tracker }.
core-trackers-mal-reconnect = Reconecte o MyAnimeList nas Configurações.
core-trackers-mal-wants-secret = O MyAnimeList exige o client secret deste app. Inclua-o em Contas, nas Configurações do Anibeam, ou registre o app com App Type "other".
core-trackers-token-reply-unreadable = não foi possível ler a resposta do token: { $error }
core-trackers-token-reply-no-token = a resposta do token não incluiu um access token

## trackers/src/lib.rs

core-trackers-no-token = nenhum access token guardado, reconecte nas Configurações

## trackers/src/cache.rs

core-trackers-no-user-id = a resposta da conta não incluiu um ID de usuário

## trackers/src/move_up.rs

core-trackers-move-none-picked = nenhuma série selecionada para mover
core-trackers-moved-up =
    { $count ->
        [one] { $count } série movida deste computador para o { $site }.
       *[other] { $count } séries movidas deste computador para o { $site }.
    }
core-trackers-moved-up-some-refused =
    { $moved ->
        [one] { $moved } série movida
       *[other] { $moved } séries movidas
    } para o { $site }; não foi possível mover { $failed ->
        [one] { $failed } série
       *[other] { $failed } séries
    }.
core-trackers-move-refused = O { $site } não conseguiu atualizar { $show }: { $error }

## trackers/src/oauth.rs

core-trackers-oauth-timed-out = A autorização expirou, tente de novo.
core-trackers-oauth-state-mismatch = state não confere, possível CSRF, cancelado.
core-trackers-oauth-no-code = nenhum código no retorno
core-trackers-oauth-no-token = nenhum access_token no retorno
core-trackers-oauth-waiting = Esperando você entrar no { $site } pelo navegador.
core-trackers-oauth-connected = { $site } conectado como { $username }.
core-trackers-oauth-no-name = a resposta do perfil não incluiu um nome de usuário
core-trackers-oauth-could-not-start = Não foi possível iniciar o login
core-trackers-oauth-port-failed = não foi possível abrir a porta { $port } neste computador: { $error }
core-trackers-oauth-port-busy = a porta { $port } neste computador está em uso por outro programa. Espere um pouco e tente de novo, ou feche o programa que a usa se ela continuar ocupada.
core-trackers-oauth-provider-error = O { $site } retornou um erro: { $error }
core-trackers-page-connected = { $site } está conectado ao { $app }
core-trackers-page-close-tab = Você pode fechar esta aba.
core-trackers-page-failed-title = { $app }, falha ao entrar
core-trackers-page-failed = O login não terminou
core-trackers-page-go-back = Volte ao { $app } e tente de novo.
core-trackers-page-finishing-title = { $app }, quase lá
core-trackers-page-finishing = Quase lá
core-trackers-page-finishing-body = O { $site } respondeu e o Anibeam está concluindo o login. Volte ao { $app } para continuar.
core-trackers-page-cancelled-title = { $app }, login cancelado
core-trackers-page-cancelled = O login foi cancelado
core-trackers-page-cancelled-body = Você pode fechar esta aba. Comece de novo pelo { $app } se ainda quiser entrar.

## trackers/src/on_hold.rs

core-trackers-on-hold-moving = Movendo para Em pausa no { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] { $count } série movida para Em pausa no { $site }.
       *[other] { $count } séries movidas para Em pausa no { $site }.
    }
core-trackers-on-hold-moved-two =
    { $count ->
        [one] { $count } série movida para Em pausa no { $site } e { $second } no { $secondSite }.
       *[other] { $count } séries movidas para Em pausa no { $site } e { $second } no { $secondSite }.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } série não pôde ir para Em pausa.
       *[other] { $failed } séries não puderam ir para Em pausa.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] { $count } série movida para Em pausa no { $site }.
       *[other] { $count } séries movidas para Em pausa no { $site }.
    } { $failed ->
        [one] { $failed } série não pôde ir para Em pausa.
       *[other] { $failed } séries não puderam ir para Em pausa.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] { $count } série movida para Em pausa no { $site } e { $second } no { $secondSite }.
       *[other] { $count } séries movidas para Em pausa no { $site } e { $second } no { $secondSite }.
    } { $failed ->
        [one] { $failed } série não pôde ir para Em pausa.
       *[other] { $failed } séries não puderam ir para Em pausa.
    }
core-trackers-series-fallback = série { $id }
core-trackers-back-on-watching = { $name } voltou para Assistindo no { $site }.
core-trackers-back-on-watching-two = { $name } voltou para Assistindo no { $site } e no { $secondSite }.
core-trackers-back-on-watching-failed = { $name } não pôde voltar para Assistindo. Tente de novo mais tarde.

## trackers/src/writes.rs

core-trackers-rate-limited = O { $tracker } limitou as requisições, tente de novo em um minuto.
core-trackers-auth-expired = O login no { $tracker } expirou, reconecte nas Configurações.
core-trackers-entry-not-found = Entrada não encontrada no { $tracker }.
core-trackers-server-error = Erro no servidor do { $tracker } ({ $status }), tente de novo mais tarde.
core-trackers-error-see-log = Erro no { $tracker }, veja o log de atividade.
core-trackers-episode-not-whole = { $episode } precisa ser um número inteiro maior que 0
core-trackers-score-out-of-range = { $score } precisa estar entre 0 e 10
core-trackers-rewatch-not-settable = reassistir não pode ser definido manualmente
core-trackers-notes-too-long = as notas não podem passar de { $max } caracteres
core-trackers-nothing-picked = nada selecionado
core-trackers-notes-not-on-list = Ainda não está na sua lista do { $tracker }. Inclua em uma lista primeiro.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } e { -mal }
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
       *[repeating] Reassistindo
    }
core-trackers-marked = Episódio { $episode } de { $show } marcado no { core-trackers-sites }.
core-trackers-progress-set = Progresso em { $show } definido para { $progress } no { core-trackers-sites }.
core-trackers-scored = Nota { $score } para { $show } no { core-trackers-sites }.
core-trackers-score-cleared = Nota de { $show } apagada no { core-trackers-sites }.
core-trackers-status-set = { $show } definida como { core-trackers-status } no { core-trackers-sites }.
core-trackers-notes-saved = Notas sobre { $show } salvas no { core-trackers-sites }.
core-trackers-notes-cleared = Notas sobre { $show } apagadas no { core-trackers-sites }.
core-trackers-update-failed = Não foi possível atualizar o { $site }. Tente de novo mais tarde.
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
            [one] { $done } série
           *[other] { $done } séries
        }
    }
core-trackers-written-done =
    { $action ->
        [clear] Nota apagada para { core-trackers-written-who } no { core-trackers-sites }.
        [status] { core-trackers-written-who } definido como { core-trackers-status } no { core-trackers-sites }.
       *[score] Nota { $score } para { core-trackers-written-who } no { core-trackers-sites }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] { $failed } série não pôde ser atualizada: { $reason }.
           *[other] { $failed } séries não puderam ser atualizadas: { $reason }.
        }
       *[no] { $failed ->
            [one] { $failed } série não pôde ser atualizada.
           *[other] { $failed } séries não puderam ser atualizadas.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] { $failed } não puderam ser atualizadas: { $reason }.
       *[no] { $failed } não puderam ser atualizadas.
    }
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } série não tem vínculo com uma lista.
       *[other] {" "}{ $noMatch } séries não têm vínculo com uma lista.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } série está oculta, então a sincronização dela está desativada.
       *[other] {" "}{ $hidden } séries estão ocultas, então a sincronização delas está desativada.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } série só está em um tracker que não está conectado.
       *[other] {" "}{ $notConnected } séries só estão em um tracker que não está conectado.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } série fica neste computador, sem tracker para onde enviar.
       *[other] {" "}{ $kept } séries ficam neste computador, sem tracker para onde enviar.
    }

## metadata/src/automatch.rs

core-metadata-matched-to = { $show } vinculada a { $title }.
core-metadata-auto-matched =
    { $count ->
        [one] { $count } série vinculada.
       *[other] { $count } séries vinculadas.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } série ainda precisa de vínculo.
       *[other] { $count } séries ainda precisam de vínculo.
    }
core-metadata-auto-matched-some =
    { $matched ->
        [one] { $matched } série vinculada
       *[other] { $matched } séries vinculadas
    }; { $unmatched } ainda { $unmatched ->
        [one] precisa
       *[other] precisam
    } de vínculo.
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
        [yes] { $hitFolder } vinculada a { $hitTitle }.
       *[no] { $matched ->
            [one] { $matched } série vinculada.
           *[other] { $matched } séries vinculadas.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] Nenhum vínculo encontrado para { $missName }.
       *[no] { $unmatched ->
            [one] Nenhum vínculo encontrado para { $unmatched } série.
           *[other] Nenhum vínculo encontrado para { $unmatched } séries.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] { $failed } série não pôde ser buscada; tente de novo mais tarde.
       *[other] { $failed } séries não puderam ser buscadas; tente de novo mais tarde.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } série já tinha vínculo.
       *[other] { $had } séries já tinham vínculo.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } série já está sendo vinculada.
       *[other] { $busy } séries já estão sendo vinculadas.
    }

## metadata/src/apply.rs

core-metadata-link-unreadable = Não foi possível ler esse link. Cole o endereço de uma página do AniList, MyAnimeList, TVmaze ou IMDb.
core-metadata-no-mal-entry = O AniList não tem entrada para esse id do MyAnimeList.
core-metadata-no-imdb-show = O TVmaze não tem série para esse título do IMDb.
core-metadata-search-on = busca no { $provider }
core-metadata-query-too-short = Digite pelo menos dois caracteres para buscar.
core-metadata-not-a-link = não é um link
core-metadata-refresh-unsupported = atualizar um vínculo só do MAL ou do TMDB
core-metadata-matched = { $show } vinculada.
core-metadata-updated = Informações de { $show } atualizadas.
core-metadata-update-failed = Não foi possível atualizar as informações de { $show }. Tente de novo mais tarde.
core-metadata-progress-refresh = atualização
core-metadata-progress-backfill = preenchimento
core-metadata-updated-many =
    { $count ->
        [one] Informações de { $count } série atualizadas.
       *[other] Informações de { $count } séries atualizadas.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] Não foi possível atualizar as informações de { $count } série. Tente de novo mais tarde.
       *[other] Não foi possível atualizar as informações de { $count } séries. Tente de novo mais tarde.
    }
core-metadata-updated-some =
    { $count ->
        [one] Informações de { $count } série atualizadas; { $failed } não puderam ser atualizadas.
       *[other] Informações de { $count } séries atualizadas; { $failed } não puderam ser atualizadas.
    }

## metadata/src/fetch.rs

core-metadata-no-such-show = O TVmaze não tem a série { $id }
core-metadata-no-such-media = O AniList não tem a mídia { $id }

## metadata/src/imdb.rs

core-metadata-progress-imdb-links = links do IMDb no Wikidata
core-metadata-progress-imdb-ratings = notas do IMDb

## metadata/src/franchise/layout.rs

core-franchise-relation-character = Personagens em comum

## net/src/images.rs

core-images-progress-fetching = baixando imagens

## host/src/transfer/export.rs

core-transfer-exported =
    { $count ->
        [one] { $count } série salva em { $folder }.
       *[other] { $count } séries salvas em { $folder }.
    }

## host/src/transfer/format.rs

core-transfer-not-json = não é JSON: { $error }
core-transfer-wrong-format = não é um documento { $format }

## host/src/transfer/import.rs

core-transfer-imported = Sua biblioteca foi carregada do arquivo.

## library/src/delete.rs

core-library-no-such-file = nenhum arquivo { $id } na biblioteca
core-library-file-elsewhere-not-removed = o arquivo { $id } pertence a outra série, então nada foi removido
core-library-file-outside = { $path } não é um arquivo dentro de uma pasta da biblioteca, então nada foi removido
core-library-episodes-removed =
    { $count ->
        [one] { $count } episódio
       *[other] { $count } episódios
    } de { $title } { $place ->
        [deleted] { $count ->
            [one] excluído
           *[other] excluídos
        }
       *[trash] { $count ->
            [one] movido para a lixeira
           *[other] movidos para a lixeira
        }
    }.
core-library-finished-removed =
    { $count ->
        [one] { $count } série concluída
       *[other] { $count } séries concluídas
    } { $place ->
        [deleted] { $count ->
            [one] excluída
           *[other] excluídas
        }
       *[trash] { $count ->
            [one] movida para a lixeira
           *[other] movidas para a lixeira
        }
    }.
core-library-watched-removed =
    { $count ->
        [one] { $count } episódio assistido
       *[other] { $count } episódios assistidos
    } de { $series } séries { $place ->
        [deleted] { $count ->
            [one] excluído
           *[other] excluídos
        }
       *[trash] { $count ->
            [one] movido para a lixeira
           *[other] movidos para a lixeira
        }
    }.

## library/src/detail.rs

core-library-fix-one-file =
    O único arquivo aqui é lido como { $read ->
        [episode] episódio { $number }
       *[label] { $label }
    }. Parece { $what ->
        [film] o filme
       *[episode] o único episódio
    }.
core-library-fix-specials =
    { $count ->
        [one] { $count } especial parece
       *[other] { $count } especiais parecem
    } { $episodes ->
        [one] o episódio que falta
       *[other] os episódios que faltam
    }, { $numbers }.
core-library-fix-numbered-on = Os episódios { $from } parecem os episódios { $to }, com a numeração continuando de uma temporada anterior.
core-library-runs-span = { $first } a { $last }
core-library-runs-list = { $init } e { $last }

## library/src/fixes.rs

core-library-fix-episode-range = o número do episódio precisa estar entre 0 e 10000
core-library-fix-season-range = o número da temporada precisa ser 200 ou menos
core-library-fix-extra-range = o índice do extra precisa ser 10000 ou menos
core-library-file-elsewhere-not-fixed = o arquivo { $id } pertence a outra série, então nada foi corrigido
core-library-fix-undone =
    { $count ->
        [one] { $count } arquivo de { $title } voltou a ser lido pelo nome.
       *[other] { $count } arquivos de { $title } voltaram a ser lidos pelo nome.
    }
core-library-fixed =
    { $count ->
        [one] Corrigida a leitura de { $count } arquivo de { $title }.
       *[other] Corrigida a leitura de { $count } arquivos de { $title }.
    }

## library/src/reads.rs

core-library-hid-one = { $title } ocultada.
core-library-back-one = { $title } voltou.
core-library-hid-many = { $count } séries ocultadas.
core-library-back-many = { $count } séries voltaram.
core-library-episode-count-set = { $title } agora tem { $count } episódios.
core-library-episode-count-cleared = Contagem de episódios de { $title } apagada.

## library/src/schedule.rs

core-library-schedule-cleared = Notas de programação de { $title } apagadas.
core-library-schedule-saved = { $count } notas de programação salvas para { $title }.

## library/src/scan.rs

core-library-source-not-absolute = o caminho de uma fonte precisa ser absoluto
core-library-source-exists = já é uma fonte
core-library-source-nested = está dentro de uma fonte existente
core-library-source-added = Pasta { $folder } incluída na sua biblioteca.
core-library-source-removed = Pasta { $folder } removida da sua biblioteca.
core-library-source-removed-with-shows =
    { $count ->
        [one] Pasta { $folder } e a { $count } série dela removidas da sua biblioteca.
       *[other] Pasta { $folder } e as { $count } séries dela removidas da sua biblioteca.
    }
core-library-series-forgotten = { $name } removida da sua biblioteca.
core-library-source-back = A pasta { $folder } voltou.
core-library-source-unavailable = A pasta { $folder } não está disponível. Confira se o disco dela está conectado.
core-library-source-unreadable = Não foi possível ler a pasta { $folder }. Confira se o disco dela está conectado.
core-library-source-looks-empty = A pasta { $folder } parece vazia, então suas séries ficaram como estavam.
core-library-source-reconcile-failed = Não foi possível atualizar as séries em { $folder }.
core-library-scan-done =
    Pastas verificadas: { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } série nova
           *[other] { $added } séries novas
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } série alterada
           *[other] { $changed } séries alteradas
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }{ $removed } sumidas
    }.

## library/src/report.rs

core-library-agent-no-answer = sem resposta em { $span }
core-library-agent-stopped-code = parou com o código { $code }
core-library-agent-stopped = parou
core-library-agent-answered-nothing = não retornou nada
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
core-library-report-no-fix = nenhum destes arquivos tem correção para relatar
core-library-agent-no-triage = { $agent } não fez a triagem: { $why }
core-transfer-progress-sources = fontes
core-transfer-progress-series = séries
core-transfer-progress-history = histórico
core-transfer-progress-accounts = contas
core-transfer-progress-preferences = preferências
core-transfer-progress-done = pronto

## downloads/src/anirss.rs

core-downloads-anirss-no-answer = o anirss não respondeu: { $message }
core-downloads-anirss-answered = o anirss retornou HTTP { $status }: { $text }
core-downloads-no-free-port = nenhuma porta livre: { $error }
core-downloads-anirss-exited = o anirss fechou ({ $status })
core-downloads-anirss-exited-said = o anirss fechou ({ $status }): { $said }
core-downloads-anirss-slow-start = o anirss não iniciou em { $seconds } segundos
core-downloads-anirss-slow-start-said = o anirss não iniciou em { $seconds } segundos: { $said }
core-downloads-anirss-install-mac = Instale o anirss { $version } ou mais novo com: { $command }
core-downloads-anirss-install-linux = Instale o anirss { $version } ou mais novo: o pacote anirss no AUR, ou { $command }
core-downloads-anirss-too-old = O anirss { $version } é antigo demais para o Anibeam, que precisa da { $minimum } ou mais nova. { $update }.
core-downloads-anirss-lacks-feature = O anirss { $version } não consegue guardar o login de cada cliente de torrent separado, o que o Anibeam precisa para trocar de cliente; o anirss { $minimum } e mais novos conseguem. { $update }.
core-downloads-anirss-update-mac = Atualize com: { $command }
core-downloads-anirss-update-linux = Atualize o pacote anirss, ou rode: { $command }

## downloads/src/contents.rs

core-downloads-torrent-file-no-answer = não foi possível baixar o arquivo torrent: { $message }
core-downloads-torrent-file-answered = o arquivo torrent retornou HTTP { $status }

## downloads/src/feed.rs

core-downloads-feed-answered = o feed retornou HTTP { $status }
core-downloads-not-a-feed = não é um feed: { $error }

## downloads/src/jackett.rs

core-downloads-jackett-missing = O Jackett não está instalado: { $program } não foi encontrado. Instale-o, ou mude o comando nas Configurações.
core-downloads-endpoint-start-failed = { $name } não pôde iniciar: { $why }
core-downloads-endpoint-start-timeout = { $name } iniciou mas não respondeu em { $seconds } segundos.
core-downloads-command-exited = o comando terminou com { $status }

## downloads/src/lib.rs

core-downloads-anirss-stopped = o anirss parou
core-downloads-anirss-missing = { $install }. Você também pode apontar as Configurações para um servidor em execução.
core-downloads-anirss-command-missing = { $command } não foi encontrado. { $install }. Você também pode apontar as Configurações para um servidor em execução.
core-downloads-anirss-start-failed = { $command } não pôde iniciar: { $error }
core-downloads-anirss-too-old-said = { $tooOld } Ele disse: { $message }
core-downloads-no-torrent-client = Nenhum qBittorrent respondeu em { $address }, e o Anibeam não tem um cliente embutido disponível. Instale o qBittorrent e ative a Web UI dele, ou instale o qbittorrent-nox para o Anibeam rodá-lo automaticamente
core-downloads-own-client-gone = O qBittorrent embutido do Anibeam está faltando (o qbittorrent-nox não está instalado). Reinstale o Anibeam, ou aponte as Configurações para o seu próprio qBittorrent
core-downloads-no-port = nenhuma porta em { $address }
core-downloads-own-client-settings = não foi possível gravar as configurações do qBittorrent: { $error }
core-downloads-own-client-start-failed = { $command } não pôde iniciar: { $error }
core-downloads-own-client-slow-start = O qBittorrent do Anibeam não iniciou
core-downloads-own-client-slow-start-said = O qBittorrent do Anibeam não iniciou: { $said }
core-downloads-not-ready = Os downloads não estão prontos. Confira Downloads nas Configurações.
core-downloads-ready = Os downloads estão prontos.
core-downloads-helper-sources =
    { $count ->
        [one] Os downloads agora buscam direto no Anibeam, que não lê a { $count } fonte na config do ajudante de busca. Inclua-a em Configurações > Downloads, ou volte para o ajudante de busca lá.
       *[other] Os downloads agora buscam direto no Anibeam, que não lê as { $count } fontes na config do ajudante de busca. Inclua-as em Configurações > Downloads, ou volte para o ajudante de busca lá.
    }
core-downloads-client-connected = O app de download está conectado.
core-downloads-client-not-connected = O app de download não está conectado. Confira o endereço dele nas Configurações.
core-downloads-not-on-demand = { $name } não é uma fonte sob demanda
core-downloads-source-running = A fonte de download está rodando.
core-downloads-source-not-running = { $name } não está rodando. Ative-a nas suas fontes e confirme para iniciá-la.
core-downloads-empty-query = digite algo para buscar
core-downloads-first-endpoint = a primeira fonte
core-downloads-search-starting = Iniciando a busca
core-downloads-search-starting-endpoint = Iniciando { $endpoint }
core-downloads-search-searching-endpoint = Buscando em { $endpoint }
core-downloads-search-reading = Lendo os resultados
core-downloads-subscribed = Seguindo { $name }.
core-downloads-client-no-rule = O qBittorrent não tem nenhuma regra chamada { $name }
core-downloads-trying-search = Tentando "{ $search }"
core-downloads-nothing-selected = nada escolhido para baixar
core-downloads-added =
    { $count ->
        [one] { $count } download incluído nos seus downloads.
       *[other] { $count } downloads incluídos nos seus downloads.
    }
core-downloads-movie-link = o link de um filme precisa ser um magnet ou uma URL http(s)
core-downloads-added-movie = Um filme incluído nos seus downloads.
core-downloads-library-folder = { $folder } é a pasta da biblioteca de { $name } e contém episódios que esta série seguida não baixou. Deixe de seguir sem excluir os arquivos, ou remova { $name } pela página dela
core-downloads-no-torrents-named = nenhum torrent indicado
core-downloads-paused =
    { $count ->
        [one] { $count } download pausado.
       *[other] { $count } downloads pausados.
    }
core-downloads-resumed =
    { $count ->
        [one] { $count } download retomado.
       *[other] { $count } downloads retomados.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] { $count } download excluído com os arquivos.
       *[other] { $count } downloads excluídos com os arquivos.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] { $count } download removido da sua lista.
       *[other] { $count } downloads removidos da sua lista.
    }
core-downloads-nothing-to-remove = nada para remover
core-downloads-removed-show = { $title } removida dos seus downloads.
core-downloads-removed-the-show = A série foi removida dos seus downloads.
core-downloads-deleted-still-listed = Os arquivos excluídos ainda aparecem no app de download.
core-downloads-feed-url = a URL do feed precisa começar com http:// ou https://
core-downloads-rule-needs-name = o nome da regra não pode ficar vazio
core-downloads-no-rule = nenhuma regra chamada { $name }
core-downloads-feed-no-search = este feed não tem termo de busca para encurtar
core-downloads-no-repair = { $name } não precisa de conserto.
core-downloads-repaired = { $name } consertada.
core-downloads-repair-failed = nenhuma busca mais curta retornou resultados; até "{ $search }" voltou vazia
core-downloads-no-saved-client = nenhum cliente de torrent { $id } salvo
core-downloads-client-switched = Os downloads agora usam outro app de download.
core-downloads-host-not-running = não está rodando no host
core-downloads-host-late = não respondeu a tempo

## downloads/src/native.rs

core-downloads-native-empty-query = busca vazia
core-downloads-native-no-results = nenhum resultado para { $query }
core-downloads-native-feed-taken = um feed chamado { $name } já aponta para outro endereço
core-downloads-native-no-items = nenhum item informado
core-downloads-native-bad-link = os links precisam ser magnet: ou URLs http(s), veio { $link }
core-downloads-native-no-links = nenhum link informado
core-downloads-native-no-names = nenhum nome de série seguida informado
core-downloads-native-files-elsewhere = o cliente de torrent guarda os arquivos em outra máquina, então eles só podem ser excluídos junto com os torrents
core-downloads-native-unknown-feeds = feed(s) desconhecido(s): { $names }
core-downloads-native-outside-folders = recusando remover arquivos fora das pastas de download: { $paths }
core-downloads-refuse-home-folder = { $path } é ou contém a sua pasta pessoal, então nada foi removido
core-downloads-refuse-library-folder = { $path } é ou contém uma pasta da biblioteca, então nada foi removido

## downloads/src/paths.rs

core-downloads-no-default-folder = o qBittorrent não tem pasta padrão de destino definida; defina uma no qBittorrent
# $folder is the folder given
core-downloads-save-folder-not-absolute = { $folder } não é um caminho completo para uma pasta
# $folder is the folder given
core-downloads-save-folder-unusable = { $folder } não pode ser usada para downloads: escolha uma pasta existente que não seja a sua pasta pessoal
core-downloads-name-empty = o nome não pode ficar vazio
core-downloads-name-slash = o nome não pode conter / nem \
core-downloads-name-not-folder = { $name } não é um nome de pasta

## downloads/src/prune.rs

core-downloads-pruned = Deixou de seguir { $title } (a exibição terminou).
core-downloads-prune-failed = Não foi possível deixar de seguir { $title }. Confira Downloads nas Configurações.

## library/src/trash.rs

core-downloads-refuse-not-absolute = { $path } não é um caminho absoluto simples, então nada foi removido
core-downloads-refuse-root = { $path } é a raiz do sistema de arquivos, então nada foi removido
core-downloads-refuse-outside-library = { $path } não está dentro de uma pasta da biblioteca, então nada foi removido
core-downloads-refuse-save-folder = { $path } é a pasta de destino do torrent, não os arquivos baixados, então nada foi removido

## downloads/src/watch.rs

core-downloads-episode-missed = O episódio { $episode } de { $title } ainda não apareceu três dias depois de ir ao ar. Seu feed vai continuar procurando.

## downloads/src/settings.rs

core-downloads-settings-command-empty = o comando do anirss não pode ficar vazio
core-downloads-settings-anirss-address = o endereço do anirss precisa começar com http:// ou https://
core-downloads-settings-client-address = o endereço do { $client } precisa começar com http:// ou https://
core-downloads-settings-bad-id = { $id } não é um ID de cliente: use letras, números, - e _
core-downloads-settings-same-id = dois clientes têm o ID { $id }
core-downloads-settings-saved-address = o endereço de { $name } precisa começar com http:// ou https://
core-downloads-settings-same-name = dois clientes se chamam { $name }; dê um nome único a cada um

## downloads/src/qbt.rs

core-downloads-qbt-no-answer = O qBittorrent não respondeu em { $address }: { $message }
core-downloads-qbt-password-rejected = O qBittorrent recusou a senha; corrija nas Configurações
core-downloads-qbt-banned = O qBittorrent bloqueou este endereço após logins falhos; espere, ou reinicie o qBittorrent
core-downloads-qbt-login-rejected = O qBittorrent recusou o usuário ou a senha
core-downloads-qbt-login-failed = O login do qBittorrent em { $address } retornou HTTP { $status }; vai tentar de novo no próximo pedido
core-downloads-qbt-not-web-ui = { $address } retornou HTTP { $status }, o que não parece a Web UI do qBittorrent. Confira o endereço nas Configurações e veja se a Web UI está ativada no qBittorrent
core-downloads-qbt-not-added = O qBittorrent não incluiu os torrents: { $said }
core-downloads-qbt-some-not-added = O qBittorrent não conseguiu incluir { $count } dos torrents: { $said }
core-downloads-qbt-answered = O qBittorrent retornou HTTP { $status }: { $line }

## downloads/src/replace.rs

core-downloads-replace-bad-item = o item { $item } não é um dos { $count } releases
core-downloads-not-replaced = O episódio { $episode } não foi substituído.
core-downloads-replaced = Episódio { $episode } substituído pelo novo download.
core-downloads-no-video-moved = { $name } não tem nenhum vídeo, então o Anibeam não consegue reproduzi-lo. Agora ele está em { $folder }.
core-downloads-no-video = { $name } não tem nenhum vídeo, então o Anibeam não consegue reproduzi-lo.

## downloads/src/rolling.rs

core-downloads-no-roll = nenhum download contínuo { $id }
core-downloads-rolled = { $title }: agora nos episódios { $from } a { $to } de { $total }.
core-downloads-roll-ahead = mantenha pelo menos um episódio à frente
core-downloads-roll-nothing = nada para baixar

## releases/src/endpoints.rs

core-releases-json-needs-repo = uma fonte de busca JSON precisa vir de uma lista de fontes cuja definição mapeie os campos
core-releases-endpoint-no-name = falta `name`
core-releases-endpoint-bad-name = { $name }: use só letras, números, '.', '_', '-'
core-releases-endpoint-json = { $name }: { $why }
core-releases-endpoint-unknown-kind = { $name }: tipo desconhecido { $kind } (válidos: { $valid })
core-releases-endpoint-no-url = { $name }: falta `url`
core-releases-endpoint-not-http = { $name }: o endereço precisa começar com http:// ou https://
core-releases-endpoint-needs-query = { $name }: a URL de { $kind } precisa conter o marcador {"{"}query{"}"}
core-releases-endpoint-no-query = { $name }: uma URL de { $kind } inclui o termo de busca automaticamente, então não deve conter {"{"}query{"}"}

## releases/src/fetch.rs

core-releases-indexer-error = o indexador retornou o erro { $code }
core-releases-bad-feed = feed inválido: { $error }
core-releases-feed-answered = o feed retornou HTTP { $status }

## releases/src/json_feed.rs

core-releases-not-feed-json = não é o JSON do feed: { $error }

## releases/src/json.rs

core-releases-not-json = não é JSON: { $error }
core-releases-search-error-said = a busca retornou um erro: { $message }
core-releases-search-error = a busca retornou um erro
core-releases-no-results-field = a resposta não tem resultados em { $path }
core-releases-results-not-list = os resultados da resposta não são uma lista
core-releases-search-answered = a busca retornou HTTP { $status }

## releases/src/native.rs

core-releases-no-endpoints = nenhuma fonte de download ainda: inclua primeiro uma lista de fontes ou uma fonte personalizada
core-releases-no-source-named = nenhuma fonte chamada { $name }
core-releases-empty-query = digite algo para buscar
core-releases-cannot-follow = { $name } não pode ser seguida: permite busca, mas não tem feed para novos episódios
core-releases-endpoint-gone = { $name } não está mais disponível: removida, desativada ou sem a chave

## releases/src/quirks.rs

core-releases-quirks-variant = one_variant { $word } precisa ser uma palavra curta
core-releases-quirks-pattern-count = show_feed aceita de 1 a { $most } padrões de página
core-releases-quirks-pattern-long = um padrão de página de show_feed é longo demais
core-releases-quirks-pattern-bad = padrão de página de show_feed: { $error }
core-releases-quirks-pattern-slug = um padrão de página de show_feed precisa de um grupo chamado slug
core-releases-quirks-url-https = a url de show_feed precisa ser https
core-releases-quirks-url-host = a url de show_feed precisa estar no host do próprio endpoint
core-releases-quirks-url-slug = a url de show_feed leva {"{"}slug{"}"} uma vez e nenhum outro marcador

## releases/src/refinement.rs

core-releases-refine-no-tokens = a op 'tokens' precisa de uma lista `tokens` não vazia

## releases/src/grid.rs

core-releases-group-batch = lote de { $group }
core-releases-episodes-phrase =
    { $count ->
        [one] Episódio { $list }
       *[other] Episódios { $list }
    }
core-releases-episode-range = { $from } a { $to }

## Names that stay the same in every language

-app-name = Anibeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = o nome precisa ter de 1 a { $max } caracteres, sem caracteres de controle
core-prefs-room-relay = o endereço do relay precisa começar com https:// ou http://
core-room-service = Assistir juntos
core-room-not-found = Nenhuma sala tem esse código. Ela pode ter sido fechada.
core-room-full = Essa sala está cheia.
core-room-proto = Esse servidor usa outra versão do Assistir juntos. Atualize o { -app-name } para entrar.
core-room-needs-episode = Esse servidor exige um episódio antes de criar uma sala. Inicie um pelo player ou atualize o servidor.
core-room-unreachable = Não foi possível acessar o servidor de salas: { $message }
core-room-refused = O servidor de salas recusou o pedido: { $message }
core-room-not-in = Você não está em uma sala.
core-room-no-name = digite o nome de exibição que os outros na sala vão ver
core-room-bad-code = códigos de sala precisam ter 6 letras ou dígitos
core-room-created = Sala do Assistir juntos criada. Código: { $code }.
core-room-joined = Entrou na sala { $code } do Assistir juntos.
core-room-left = Saiu da sala do Assistir juntos.
core-room-member-joined = { $name } entrou na sala.
core-room-member-left = { $name } saiu da sala.
core-room-closed = A sala do Assistir juntos foi fechada.
core-room-lost = A conexão com a sala do Assistir juntos caiu. Reconectando.
core-room-back = Reconectado à sala do Assistir juntos.
core-room-media-asked = Você pediu o episódio ao anfitrião.
core-room-media-cannot = O anfitrião não pode compartilhar este episódio agora.
core-room-media-sent = Torrent do episódio enviado para { $name }.
core-room-media-sent-name = Nome do lançamento do episódio enviado para { $name }.
core-room-media-nothing = O anfitrião não tem uma cópia do episódio para compartilhar.
core-room-media-no-answer = O aplicativo do anfitrião não respondeu. Tente de novo em um instante.
core-room-media-torrent = Baixando o episódio pelo torrent do anfitrião.
core-room-media-searching = Procurando o lançamento do anfitrião nas suas fontes.
core-room-media-not-found = Nenhuma das suas fontes tem o lançamento do anfitrião.
core-room-media-no-sources = Todas as fontes estão desativadas. Ative uma para buscar.
core-room-media-no-start = Não foi possível iniciar o download: { $message }
core-room-media-client = o cliente de torrent recusou
core-room-media-lost = O cliente de torrent perdeu o download do episódio.
core-room-media-stalled = O download está parado há dez minutos. Tente de novo mais tarde.
core-room-media-not-in-library = O episódio terminou de baixar, mas não apareceu na sua biblioteca. Verifique se os downloads são salvos em uma pasta da biblioteca.
core-room-media-here = O episódio já está na sua biblioteca.
core-room-media-rest-missing = Não foi possível obter o resto da série. Só este episódio está sendo baixado.
