## Names that stay names

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Words many pages share. Use these before adding a page's own copy.

common-cancel = Cancelar
common-back = Voltar
common-save = Salvar
common-done = Pronto
common-close = Fechar
common-delete = Excluir
common-retry = Repetir
common-copy = Copiar
common-clear = Limpar
common-on = Ativado
common-off = Desativado
common-play = Assistir
common-hide = Ocultar
common-remove = Remover
common-add = Incluir
common-test = Testar
common-reading = Lendo...
common-working = Aguarde...
common-checking = Verificando...
common-all = Todos
common-none = Nenhum
common-episodes = { $count ->
    [one] { $count } episódio
   *[other] { $count } episódios
}
common-episode-n = Episódio { $n }
common-join-comma = { $a }, { $b }

## Language (#793)

lang-follow-phone = Igual ao celular
more-language = Idioma
more-language-phone = Igual ao celular: { $lang }
lang-name-en = Inglês
lang-name-da = Dinamarquês
lang-name-de = Alemão
lang-name-es = Espanhol
lang-name-fr = Francês
lang-name-pt-BR = Português (Brasil)
lang-name-it = Italiano
lang-name-nl = Holandês
lang-name-pl = Polonês
lang-name-ru = Russo
lang-name-ja = Japonês
lang-name-ko = Coreano
lang-name-zh-CN = Chinês (simplificado)

## Onboarding

onb-welcome-title = AniBeam
onb-welcome-note = Seus animes neste celular. Encontre uma série, veja quem tem, baixe e mantenha suas listas em dia.
onb-language-title = Idioma
onb-language-note = O AniBeam usa o idioma do seu celular, a menos que você escolha outro. Dá para mudar depois em Mais.
onb-lists-title = Suas listas
onb-lists-note = Entre para que o que você assiste aqui conte no AniList ou no MyAnimeList. Dá para fazer isso depois em Mais.
onb-wifi-title = Só Wi-Fi
onb-wifi-note = Os episódios são grandes. O AniBeam baixa só no Wi-Fi, a menos que você diga o contrário.
onb-storage-title = Espaço
onb-storage-note = Um limite de quanto o AniBeam pode baixar. Ele não reserva nada: o espaço continua livre até os episódios o ocuparem.
onb-notify-title = Notificações
onb-notify-note = Enquanto os episódios baixam, uma notificação mostra o andamento, e outra avisa quando um está pronto para assistir. Os downloads acontecem de qualquer jeito.
onb-sites-title = Sites de busca
onb-sites-note = O AniBeam não vem com nada para buscar. Um repositório é uma lista assinada de sites de busca. Cole o endereço de um repositório, confira a chave e instale os sites que quiser. Dá para fazer isso depois em Mais.
onb-wifi-only = Só no Wi-Fi
onb-wifi-on = Ativado. Dados móveis pedem permissão.
onb-wifi-off = Desativado. Os downloads usam dados móveis também.
onb-storage-caption = 18 GB dão cerca de 12 episódios. Deslize até o armazenamento total do celular.
onb-use-cap = Usar este limite
onb-cap-on = Ativado. Em { $cap }, o AniBeam não baixa mais nada até você excluir episódios.
onb-cap-off = Desativado. O AniBeam baixa até o celular encher.
onb-start = Começar
onb-continue = Continuar
onb-back = Voltar

## Appearance

look-colours = Cores
look-mode = Modo
look-dark = Escuro
look-light = Claro
look-system = Sistema
look-follows-phone-dark = Segue a configuração do celular: escuro.
look-follows-phone-light = Segue a configuração do celular: claro.
look-changes-at-once = As mudanças aparecem na hora, aqui e em todas as telas.
look-dark-theme = Tema escuro
look-light-theme = Tema claro
look-accent = Destaque
look-accent-note = Marca o que está selecionado, tocando ou novo.
look-size = Tamanho
look-text-size = Texto
look-size-smaller = Menor
look-size-normal = Normal
look-size-large = Grande
look-size-larger = Maior
look-size-largest = Máximo
look-text-size-note = O tamanho das palavras em todo o AniBeam, somado ao tamanho de fonte do celular.

## The window: tabs and the sheets over every page

app-name = AniBeam
nav-home = Início
nav-calendar = Calendário
nav-library = Biblioteca
nav-downloads = Downloads
nav-more = Mais
app-mobile-title = Baixar com dados móveis?
app-mobile-note = Um episódio costuma ter de 1 a 2 GB. Em um plano limitado, isso pode gastar o mês inteiro de dados em uma noite.
app-mobile-keep = Só no Wi-Fi
app-mobile-use = Usar dados móveis
app-versions-note = O mesmo episódio em { $count } versões. Escolha qual assistir.
app-versions-plays-now = { $release }, em reprodução
app-menu-get = Baixar episódios
app-menu-delete = Excluir episódios

## More: the index and each section's line of how it stands

more-settings = Configurações
more-search = Buscar configurações
more-search-none = Nada encontrado
more-search-hits = { $count ->
    [one] 1 configuração
   *[other] { $count } configurações
}
more-search-tap = Toque para ir até lá
more-search-hint = Tente uma palavra mais simples: armazenamento, Wi-Fi, lista, site, computador.
more-accounts = Contas
more-streaming = Streaming
more-subtitles = Legendas
more-connection = Conexão
more-computer = Seu computador
more-icon = Ícone do app
more-stats = Estatísticas
more-data = Dados
more-appearance = Aparência
more-activity = Atividade
more-about = Sobre
more-accounts-none = Nenhuma conta conectada
more-accounts-syncing = Sincronizando com { $tracker }
more-accounts-syncing-both = Sincronizando com AniList e MAL
more-dl-status-wifi = Só Wi-Fi, sem limite
more-dl-status-wifi-cap = Só Wi-Fi, limite de { $cap }
more-dl-status-mobile = Wi-Fi e dados móveis, sem limite
more-dl-status-mobile-cap = Wi-Fi e dados móveis, limite de { $cap }
more-stream-status-auto = Auto, 720p em dados móveis ou relay
more-stream-status-original = O arquivo original, sempre
more-stream-status-converted = Convertido para { $quality }
more-sites-status-none = Sem repo ainda
more-sites-status-on = { $sites ->
    [one] 1 site ativo
   *[other] { $sites } sites ativos
}
more-sites-status-on-from = { $sites ->
    [one] 1 site ativo
   *[other] { $sites } sites ativos
}, { $repos ->
    [one] de 1 repo
   *[other] de { $repos } repos
}
more-conn-status-unchecked = Ainda não verificado
more-conn-status-ok = Todas as etapas passaram
more-conn-status-failed = { $total ->
    *[other] { $failed } de { $total } etapas falharam
}
more-computer-status-none = Ainda não conectado
more-icon-default = Padrão
more-icon-dark = Escuro
more-icon-line = Contorno
more-subs-status = Tamanho, posição, fonte, cores e caixa
more-data-status = Backup, restauração, cache de imagens
more-activity-status = O que o AniBeam fez, do mais recente
more-look-status-phone = Segue o celular
more-stats-status = { $count ->
    [one] { $hours } hora assistida em { $year }
   *[other] { $hours } horas assistidas em { $year }
}
more-stats-status-empty = Um ano assistindo

## More: the sections' own settings

more-accounts-note = O que você assiste aqui atualiza suas listas, e o progresso delas aparece nas suas séries.
more-storage = Espaço
more-storage-capped = Limite de { $cap }
more-storage-no-cap = Sem limite
more-storage-used = usados por episódios
more-storage-cap-caption = limite
more-storage-free = livres no celular
more-set-storage-cap = Limite
more-storage-cap-on = Em { $cap }, nada mais é baixado até você excluir episódios.
more-storage-cap-off = Desativado: o AniBeam baixa até o celular encher.
more-network = Rede
more-network-wifi = Só Wi-Fi
more-network-mobile = Wi-Fi e dados móveis
more-wifi-on = Ativado. Os downloads esperam o Wi-Fi; dados móveis pedem permissão.
more-set-stream-quality = Qualidade do vídeo
more-stream-question = Como um episódio do seu computador toca aqui.
more-stream-auto-note = Auto toca o arquivo como ele é no Wi-Fi, e convertido para 720p em dados móveis ou quando o Tailscale chega ao seu computador por um relay.
more-stream-original-note = O arquivo como ele é, sempre. Arquivos grandes pedem uma conexão rápida.
more-stream-converted-note = Convertido no seu computador para { $quality }. Gasta menos dados; avançar o vídeo demora um pouco.
more-set-feeds = Seus feeds
more-feeds-note = Buscados antes dos sites
more-set-check = Verificar a conexão
more-check-note = Testa cada etapa entre este celular e os sites de busca: encontrá-los, alcançá-los e a conexão segura.
more-check-copy = Copiar os resultados
more-set-computer = AniBeam do seu computador
more-computer-note = A biblioteca dele aparece em Biblioteca, e os episódios tocam aqui. Um nome como my-computer ou um endereço como 100.64.0.1.
more-computer-placeholder = my-computer:8790
more-subs-note = Com o que todo episódio neste celular começa. Um episódio transmitido do seu computador começa com as configurações desse computador.
more-icon-note = O ícone na tela inicial e na gaveta de apps.
more-icon-line-note = O ícone de contorno usa as cores do seu papel de parede, como seus outros ícones do Material You.
more-icon-moved-note = Alguns launchers tiram o ícone da tela inicial quando ele muda. Ele continua na gaveta de apps; arraste-o de volta de lá.
more-set-whats-new = Novidades
more-whats-new-note = O que mudou em cada versão, da mais recente.
more-set-onboarding = Rever a intro
more-onboarding-note = Refazer as perguntas da primeira execução.
more-set-version = Versão

## Status lines Rust writes on the pages

host-synced = Sincronizado { $when }.
host-last-try = Última tentativa, { $when }: { $error }
host-asking = Contatando...
host-reached = Conectado a { $host }, AniBeam { $version }.
host-saved = Salvo. Atualizando...
acct-signed-in-as = Conectado como { $name }
acct-signed-in = Conectado
acct-not-in-build = Indisponível nesta versão
acct-tap-to-sign-in = Não conectado. Toque para entrar.
acct-sign-in-failed = Falha ao entrar: { $why }. Toque para tentar de novo.
acct-sign-in-not-started = O login não iniciou: { $error }. Toque para tentar de novo.
acct-still-starting = O AniBeam ainda está iniciando. Tente de novo em instantes.
home-recently-watched = Vistos recentemente
home-recently-downloaded = Baixados recentemente
home-note-empty = Neste celular
home-note = { $count ->
    [one] Neste celular, 1 série
   *[other] Neste celular, { $count } séries
}
lib-this-phone = Este celular
lib-note-phone = { $count ->
    [one] Este celular, 1 série
   *[other] Este celular, { $count } séries
}
lib-note-remote = { $device }, sincronizado { $when }
lib-empty-search = Nada encontrado para "{ $query }".
lib-empty-phone = Nada neste celular ainda.
lib-empty-remote = Esse dispositivo não tem nada na biblioteca.
cal-view-airing = No ar agora
cal-note-none = Nada vai ao ar esta semana
cal-note = { $count ->
    [one] 1 episódio nos próximos 7 dias
   *[other] { $count } episódios nos próximos 7 dias
}
airing-asking = Lendo o AniList
airing-asking-newest = Lendo no AniList os mais novos
airing-filter-all = Tudo { $count }
airing-filter-listed = Nas listas { $count }
airing-filter-owned = No celular { $count }
airing-this-season = Esta temporada
airing-next-season = Próxima
airing-other-season = Outras
airing-hidden = { $count ->
    *[other] { $count } ocultas por falta de interesse
}
airing-plan-needs-tracker = Para planejar assistir, é preciso o AniList ou o MAL. Entre em Mais, Contas.
airing-plan-failed = Não foi possível planejar assistir: { $error }
list-watching = Assistindo
list-planning = Planejando
list-completed = Concluído
list-paused = Em pausa
list-dropped = Abandonado
list-repeating = Reassistindo
more-check-all-ok = Todas as etapas passaram.
more-check-failed = { $failed } de { $total } etapas falharam. Uma captura de tela desta lista mostra onde.
more-check-row = { $step }  { $ms } ms
more-check-copied = Copiado. Cole em uma mensagem.
more-version = AniBeam para Android { $version } (build { $build })
dl-note-waiting = Esperando o Wi-Fi. Os downloads continuam sozinhos quando o celular estiver no Wi-Fi.
dl-note-nothing = Nada sendo baixado.
dl-state-done = Pronto
dl-state-waiting = Esperando o Wi-Fi
dl-state-paused = Pausado
dl-state-peers = Procurando peers
dl-state-checking = Verificando
dl-state-error = Parado por um erro
dl-state-downloading = Baixando
dl-amount = { $got } de { $size }
dl-speed = { $rate }/s
dl-left = faltam { $span }
dl-episodes-range = Episódios { $from } a { $to }
dl-batch = Um lote
disk-storage-line = { $used } usados
disk-storage-line-free = { $used } usados, { $free } livres
disk-free = Este celular tem { $free } livres.
disk-free-below-cap = Este celular tem { $free } livres, menos que o limite: o celular enche antes de o AniBeam parar.

## More's search: the words a person might type for each setting

more-set-subs-size = Tamanho e posição da legenda
more-set-subs-style = Estilo da legenda
more-words-anilist = anilist tracker lista entrar login conta conectar progresso acompanhar
more-words-mal = mal myanimelist tracker lista entrar login conta conectar progresso acompanhar
more-words-storage-cap = espaço disco limite gb cheio armazenamento apagar excluir tamanho
more-words-wifi = wifi wi-fi dados móveis celular rede
more-words-notifications = notificação notificações avisar alerta progresso permissão permitir
more-words-stream-quality = auto original 1080p 720p 480p converter dados computador reproduzir qualidade
more-words-check-repos = atualizar repo repositório endpoint
more-words-add-repo = repo repositório endpoint confiar impressão chave endereço url
more-words-add-site = site personalizado endpoint feed rss torznab jackett prowlarr endereço
more-words-feeds = feed rss fonte remover
more-words-check = rede dns teste https seguro conexão alcançar internet
more-words-computer = tailscale tailnet host endereço desktop biblioteca streaming teste salvar dispositivo computador pc
more-words-whats-new = novidades changelog notas atualização mudanças
more-words-onboarding = introdução primeira vez execução começar de novo configurar
more-words-version = versão build sobre
more-words-stats = horas episódios gêneros estúdios temporadas ano assistido resumo mês mais ativo histórico estatísticas
more-words-backup = backup exportar arquivo salvar copiar mover computador json privado logins
more-words-restore = restaurar importar arquivo carregar biblioteca mesclar
more-words-cache = imagens pôsteres banners limpar armazenamento espaço cache
more-words-activity = log registro eventos erros problemas copiar limpar histórico o que aconteceu
more-words-colours = aparência tema escuro claro modo sistema destaque cor cores
more-words-text-size = aparência fonte maior menor tamanho texto palavras ler
more-words-icon = ícone launcher tela inicial logo escuro contorno material you
more-words-subs-size = legendas tamanho escala posição subir parte de baixo
more-words-subs-style = legendas fonte cor contorno sombra caixa negrito ass estilizadas redefinir
more-words-language = idioma língua tradução inglês português locale

## Home

home-title = Início
home-next-to-air = Próximos a sair
home-after-that = Depois
home-coming-up = Em breve
home-nothing-week = Nada na próxima semana.
home-empty-title = Nada neste celular ainda
home-empty-body = As séries que você baixa aqui, e o que vai ao ar delas, aparecem no Início. Encontre uma em Downloads.

## Library

lib-title = Biblioteca
lib-refresh = Atualizar
lib-search-phone = Buscar neste celular
lib-search-device = Buscar em { $device }
lib-find-in-downloads = Buscar "{ $query }" em Downloads
lib-empty-body = Os episódios que você baixa neste celular ficam aqui. Encontre uma série em Downloads.
lib-pick-show = Escolha uma série para abri-la aqui.
lib-shows-caption = { $count ->
    [one] série
   *[other] séries
}
lib-on-phone = no celular
lib-free = livres
lib-hold-poster = Segure um pôster para baixar mais episódios ou excluí-los.
phone-nothing-on-phone = nada no celular
phone-on-phone = { $count } no celular

## Calendar

cal-title = Calendário
cal-week = Semana
cal-days = Dias
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A, %-d de %B
cal-today = Hoje
cal-tomorrow = Amanhã
cal-yesterday = Ontem
cal-nothing = Nada
cal-next = Próximo
cal-get = Baixar
cal-day-note = { $date }, { $count }
cal-next-airing = Próxima exibição: { $title }
cal-nothing-aired-yesterday = Nada foi ao ar ontem.
cal-nothing-aired-that-day = Nada foi ao ar nesse dia.
cal-nothing-today = Nada vai ao ar hoje.
cal-nothing-tomorrow = Nada vai ao ar amanhã.
cal-nothing-that-day = Nada vai ao ar nesse dia.
cal-empty-title = Nenhuma série neste celular ainda
cal-empty-body = Os episódios das séries que você guarda aqui aparecem no dia deles: dois dias atrás e uma semana à frente.
phone-episodes-range = Episódios { $from } a { $to }
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = No celular
phone-out-ago = Lançado { $ago }

## Airing now

airing-out-now = Já saiu
airing-last-two-days = Últimos dois dias
airing-season-note = As séries da temporada vêm do AniList.
airing-none-lists-title = Nada no ar está nas suas listas
airing-none-lists-body = Quero assistir, em uma série, coloca ela na sua lista.
airing-none-phone-title = Nada no ar está neste celular
airing-none-phone-body = Baixar episódios, em uma série, baixa ela aqui.
airing-marked = { $title } foi marcada como sem interesse.
airing-undo = Desfazer
airing-plan = Quero assistir
airing-get-more = Baixar mais episódios
airing-get = Baixar episódios
airing-not-interested = Sem interesse

## Seasons

season-winter = Inverno
season-spring = Primavera
season-summer = Verão
season-fall = Outono
season-named = { $season } { $year }
season-pick = Temporada

## A show's page

series-back-home = Início
series-back-library = Biblioteca
series-back-downloads = Downloads
series-back-calendar = Calendário
series-get-episodes = Baixar episódios
series-episodes = Episódios
series-delete-episodes = Excluir episódios
series-delete-tap = Toque nos episódios para excluir.
series-delete-picked = { $count ->
    [one] { $count } episódio selecionado.
   *[other] { $count } episódios selecionados.
}
series-delete-note = Os arquivos são excluídos deste celular. O que você assistiu continua registrado.
series-delete-button = { $count ->
    [one] Excluir { $count } episódio
   *[other] Excluir { $count } episódios
}
series-no-episodes = Nenhum episódio neste celular e nenhum programado.
series-versions = { $count ->
    [one] { $count } versão
   *[other] { $count } versões
}
series-note-next-up = A seguir
series-note-missing = Faltando
series-note-not-here = Fora deste celular
series-note-not-downloaded = Não baixado
series-note-downloading = Baixando { $percent }%
series-note-resume-at = Retomar em { $time }
phone-episodes-estimate = { $count ->
    [one] { $count }? episódio
   *[other] { $count }? episódios
}
phone-next-airs = Episódio { $n } vai ao ar em { $time }
phone-watched = Assistido
phone-not-watched = Ainda não assistido
phone-progress = { $watched } de { $total } assistidos, { $count } neste celular
phone-on-this-phone = { $count } neste celular
phone-play-film = Assistir ao filme
phone-resume-film = Retomar o filme
phone-play-episode = Assistir ep. { $n }
phone-resume-episode = Retomar ep. { $n }
phone-about-date = cerca de { $date }

## A show's list status, score, notes and fixes

show-status-watching = Assistindo
show-status-planning = Planejando
show-status-completed = Concluído
show-status-paused = Em pausa
show-status-dropped = Abandonado
show-status-repeating = Reassistindo
show-status-now = { $label }, agora
show-add-to-list = Incluir na lista
show-your-score = Sua nota { $score }
show-rate = Avaliar
show-fixes = Correções
show-your-notes = Suas notas
show-add-notes = Incluir notas
show-home-each = Mude em cada lista conectada.
show-home-no-tracker = Guardado neste celular: nenhuma lista tem esta série.
show-home-tracker-lacks = Guardado neste celular: sua lista não tem esta série.
show-home-until-connected = Guardado neste celular até você conectar o AniList ou o MyAnimeList.
show-sites-both = AniList e MyAnimeList
show-notes-title = Notas
show-notes-placeholder = Onde você parou, quem indicou, qual versão baixar
show-notes-over = { $count ->
    [one] { $count } caractere acima do limite
   *[other] { $count } caracteres acima do limite
}
show-notes-saved-to = Salvo em { $sites }.
show-notes-saved-to-tracker = Salvo na sua lista.
show-notes-saving = Salvando suas notas.
show-notes-clearing = Apagando suas notas.
show-score-saving = Salvando sua nota.
show-score-saved = Nota salva.
show-score-clearing = Apagando sua nota.
show-score-cleared = Nota apagada.
show-moving-to = Movendo para { $label }.
show-moved-to = Movido para { $label }.
show-rescan-started = Varredura iniciada.
show-still-starting = O AniBeam ainda está iniciando. Tente de novo em instantes.
show-fix-match = Vínculo
show-fix-find-match = Achar vínculo
show-fix-rescan = Reler pasta

## A show's characters, related shows and recommendations

show-characters = Personagens
show-related = Relacionados
show-related-all = Ver todos os { $count }
show-related-filling = Ainda carregando do AniList.
show-recommendations = Recomendações
show-card-owned = Na sua biblioteca
show-card-unreleased = Ainda não lançado
show-card-on-anilist = No AniList
show-rec-available = Disponível
show-relation-prequel = Prequela
show-relation-sequel = Sequência
show-relation-side-story = Paralela
show-relation-parent = Origem
show-relation-spin-off = Derivado
show-relation-adaptation = Adaptação
show-relation-alternative = Alternativa
show-relation-summary = Resumo
show-relation-character = Personagem
show-relation-source = Fonte
show-relation-compilation = Compilação
show-relation-contains = Contém
show-relation-other = Outro
show-role-main = Principal
show-role-supporting = Coadjuvante
show-role-background = Figurante
show-format-tv = TV
show-format-tv-short = TV curta
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Especial
show-format-film = Filme
show-entry-get = Baixar episódios
show-entry-open-anilist = Abrir no AniList
show-entry-line = { $line }, { $status }

## Fix a wrong match

match-title = Vínculo
match-refresh = Atualizar info
match-clear = Remover vínculo
match-clear-it = Remover
match-clear-warning = Remover apaga o vínculo com a lista e a contagem de episódios definida à mão. Toque de novo para remover.
match-tvmaze = TVmaze
match-search-anilist = Buscar no AniList
match-search-tvmaze = Buscar no TVmaze
match-paste-anilist = Cole um link do AniList ou MyAnimeList
match-paste-tvmaze = Cole um link do TVmaze ou IMDb
match-look-up = Buscar
match-apply = Aplicar
match-look-up-all = Buscar todas as pastas que o AniBeam ainda não conhece
match-format-music = Música
match-join-and = { $first } e { $second }
match-matched-on = Vinculado em { $sites }
match-found-tvmaze = Série encontrada no TVmaze
match-found-anilist = Encontrado no AniList
match-found-anilist-season = Encontrado no AniList, temporada { $season }
match-found-mal = Encontrado no MyAnimeList
match-searching = Buscando em { $name }
match-type-more = Digite pelo menos duas letras para buscar em { $name }.
match-none = Nenhum título encontrado em { $name }.
match-looking-up = Buscando.
match-matched-to = Vinculado a { $title }.
match-applied = Vínculo aplicado.
match-applying = Aplicando o vínculo.
match-refreshing = Atualizando { $title }.
match-clearing = Removendo o vínculo.
match-cleared = Vínculo removido.
match-auto-looking = Buscando as pastas que o AniBeam ainda não conhece.

## Batches (the line under a release in a search)

inspect-reading = Lendo a lista de arquivos
inspect-holds = { $count ->
    [one] { $count } episódio nele: { $runs }
   *[other] { $count } episódios nele: { $runs }
}

## What's new

notes-title = Novidades
notes-got-it = Entendi

## Move the list to a tracker

move-lists-label = Listas neste celular
move-title = Mover sua lista para o { $site }
move-here = Neste celular: { $list }
move-there = No { $site } agora: { $list }
move-later-hint = Depois: Mais, Contas
move-not-now = Agora não
move-nothing = Nada
move-not-on-list = Fora da sua lista
move-nothing-left = Nada mais para mover
move-side-status-score = { $status }, nota { $score }
move-side-score = nota { $score }
move-lead = { $count ->
    [one] Uma série tem status ou nota só neste celular. Movê-la coloca na sua lista do { $site }, onde seu computador e seu perfil enxergam. Para essas séries, o que o { $site } tem agora é substituído. Nada é enviado até você tocar em Mover.
   *[other] { $count } séries têm status ou nota só neste celular. Movê-los coloca na sua lista do { $site }, onde seu computador e seu perfil enxergam. Para essas séries, o que o { $site } tem agora é substituído. Nada é enviado até você tocar em Mover.
}
move-label = { $count ->
    [one] Mover { $count } série
   *[other] Mover { $count } séries
}
move-waiting = { $count ->
    [one] Uma série tem status ou nota só aqui. Mova para o { $site } para ver no seu computador e no seu perfil.
   *[other] { $count } séries têm status ou nota só aqui. Mova para o { $site } para ver no seu computador e no seu perfil.
}
move-moving = { $count ->
    [one] Movendo { $count } série para o { $site }
   *[other] Movendo { $count } séries para o { $site }
}
move-moved = { $count ->
    [one] { $count } série movida para o { $site }
   *[other] { $count } séries movidas para o { $site }
}
move-failed = { $count ->
    [one] O { $site } não aceitou { $count } série; veja o log de atividade
   *[other] O { $site } não aceitou { $count } séries; veja o log de atividade
}

## Downloads: find (search, shows, sources, releases)

find-join-comma = { $a }, { $b }
find-join-and = { $a } e { $b }
find-join-sentences = { $a } { $b }

find-downloads = Downloads

find-lens-list = Sua lista
find-lens-gaps = Faltando
find-lens-airing = No ar
find-lens-following = Seguindo
find-lens-downloaded = Baixados
find-lens-with-count = { $lens } { $count }
find-what-to-get = O que baixar
find-lens-list-line = Assistindo e Planejando na sua lista. Abra uma para baixar os episódios.
find-lens-gaps-line = Episódios já exibidos que não estão neste celular. Abra uma série para ver quem tem.
find-lens-airing-line = Novos episódios por vir, os mais próximos primeiro.
find-lens-following-line = As séries seguidas: os novos episódios baixam sozinhos. Abra uma para parar.
find-lens-downloaded-line = O que este celular baixou, uma série por vez.

find-lens-gaps-empty = Sem faltas: toda série que você começou tem os episódios exibidos aqui.
find-lens-airing-empty = Nada neste celular está no ar.
find-lens-following-empty = Nenhuma série seguida ainda. Abra uma série e ative Seguir.
find-lens-downloaded-empty = Nada baixado ainda.
find-list-empty = Nada na sua lista ainda. Busque uma série acima e inclua.
find-list-reading = Lendo sua lista...
find-list-signed-out = Entre no AniList em Mais, Contas, e sua lista aparece aqui.
find-list-failed = Não foi possível ler sua lista: { $error }

find-aired = exibidos
find-ledger-no-feed = sem novos episódios: a fonte não tem feed
find-ledger-downloads = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
find-ledger-downloaded = baixados
find-ledger-planning = Planejando
find-ledger-watching-of = Assistindo, { $progress } de { $total }
find-ledger-watching-seen = Assistindo, { $progress } vistos

find-gap-missing = { $count } faltando
find-gap-not-downloaded = { $count } não baixados

find-list-status-planning = Na sua lista: Planejando
find-list-status-watching = Na sua lista: Assistindo
find-list-status-already = Já está na sua lista
find-going-planning = { $title } vai para Planejando.
find-already-listed = { $title } já está na sua lista.
find-plan-needs-tracker = Para incluir na lista, é preciso o AniList ou o MAL. Entre em Mais, Contas.
find-plan-failed = Não incluído na lista: { $error }
find-show-gone = Essa série não está mais na biblioteca deste celular.

## Shows as you type

find-shows = Séries
find-suggesting = Lendo o AniList...
find-suggest-tap = Toque em uma para ver quem tem. Enter busca nomes de releases.
find-suggest-none = Nenhuma série com esse nome. Enter busca nomes de releases.
find-suggest-hint = As séries aparecem enquanto você digita. Enter busca nomes de releases.
find-suggest-failed = Não foi possível consultar o AniList: { $error }
find-library-line = Na sua biblioteca
find-library-line-none = Na sua biblioteca, nenhum de { $total } neste celular
find-library-line-some = Na sua biblioteca, { $here } de { $total } neste celular

## The show view

find-plan-to-watch = Quero assistir
find-on-phone-count = { $count } neste celular
find-airing = No ar
find-dropped = { $count ->
    [one] { $count } release que não cita esta série foi deixado de fora.
   *[other] { $count } releases que não citam esta série foram deixados de fora.
}

find-following = Seguindo
find-stop-following-ask = Deixar de seguir { $name }?
find-unfollow-explain = Os novos episódios deixam de ser baixados. O que está na pasta pode ficar ou sair junto: todo download ali, inclusive os episódios que você baixou à mão.
find-stop-following = Deixar de seguir
find-stop-keep = Parar, manter os episódios
find-stop-delete = Parar e excluir os downloads da pasta
find-rule-taken = { $downloads ->
    [one] { $downloads } download
   *[other] { $downloads } downloads
}, { $files ->
    [one] { $files } arquivo
   *[other] { $files } arquivos
} na pasta
find-rule-paused = Está em pausa, então não chegam novos episódios. { $taken }.
find-rule-no-feed = A fonte não tem feed, então não chegam novos episódios. { $taken }.
find-rule-active = Os novos episódios baixam sozinhos. { $taken }.
find-unfollowed-deleted = Deixou de seguir { $name } e excluiu os downloads da pasta.
find-unfollowed-kept = Deixou de seguir { $name }. Os episódios continuam neste celular.
find-stop-failed = Não foi possível concluir: { $error }

find-where = Onde baixar
find-where-pick = Escolha uma fonte para ver o que ela tem.
find-where-nothing = Nada é baixado até você mandar.
find-source-idle = Consultar
find-source-trying = Tentando { $name }
find-source-searching = Buscando
find-source-silent = Sem resposta
find-no-sites-title = Sem sites de busca
find-no-sites-body = Inclua um repositório de sites de busca, ou um feed seu, em Mais.
find-open-more = Abrir Mais
find-pick-source-title = Escolha uma fonte
find-pick-source-body = Os releases dela aparecem aqui em uma grade, um episódio por cubo.
find-releases = { $count ->
    [one] { $count } release
   *[other] { $count } releases
}
find-unreachable = Não foi possível acessar { $host }. Algumas redes e países bloqueiam sites de releases; o AniBeam já contorna bloqueios por nome, então esta rede bloqueia o próprio site, ou o site está fora do ar. Uma VPN resolve, ou tente outra fonte.
find-the-source = A fonte

find-has = O que ela tem
find-has-note = Toque em uma linha para pegar tudo, ou episódios avulsos.
find-episodes-tag = Episódios
find-batch = Lote { $label }
find-row-meta = { common-episodes }  { $seeds } seeds  { $size }
find-on-this-phone = Neste celular
find-on-computer = Em { $name }

find-plan-no-releases = Esta fonte não tem releases desta série.
find-plan-nothing-picked = Nada escolhido: toque em uma linha ou nos cubos.
find-plan-line = { $episodes } de { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] { $count } episódio que você já tem foi deixado de fora. Ative Substituir para baixar de novo.
   *[other] Os episódios que você já tem foram deixados de fora. Ative Substituir para baixar de novo.
}
find-plan-follow-note = Os novos episódios de { $query } serão baixados sozinhos.
find-plan-on-desk = Eles baixam em { $name }, na biblioteca dele.
find-follow-new = Seguir novos episódios
find-follow-new-note = Baixam sozinhos conforme vão ao ar.
find-replace = Substituir o que você tem
find-replace-note = Baixa de novo os episódios escolhidos por cima dos que estão neste celular.
find-go = { $mode ->
    [replace] Baixar { common-episodes } e substituir
    [subscribe] Baixar { common-episodes } e seguir
    [both] Baixar { common-episodes }, substituir e seguir
   *[plain] Baixar { common-episodes }
}
find-go-pick = Escolha episódios
find-every-release = Todos os releases desta fonte

## Download to

find-download-to = Baixar em
find-this-phone = Este celular
find-your-computer = Seu computador
find-desk-asking = Consultando { $name }...
find-desk-silent = { $name } não responde. Escolha Este celular, ou tente de novo quando ele estiver ligado.
find-sending-to = Enviando para { $name }...

## The release list

find-best-fit = Melhor opção
find-on-disk = No disco
find-get = Baixar
find-get-all = Baixar os { $count }
find-subscribe = Seguir
find-back-step = Voltar um passo
find-narrow-placeholder = Filtre, ou -palavra para excluir
find-best-fit-searches = A melhor opção busca: { $query }
find-fact-batch = lote
find-fact-seeds = { $count } seeds
find-fact-via = via { $source }
find-searching-for = Buscando { $text }...
find-search-line = { $count ->
    [0] Nada em { $source } para essa busca
    [one] { $count } release em { $source }
   *[other] { $count } releases em { $source }
}{ $fell ->
    [1] , depois que a primeira fonte não teve nada
   *[other] { "" }
}{ $narrowed ->
    [1] , filtrado para { $token }
   *[other] { "" }
}.
find-refined = { find-releases }, { $dir ->
    [down] menos
   *[up] mais
} { $delta }.
find-refine-nothing-left = Assim não sobra nada.
find-refine-no-match = Nenhum release tem isso.
find-refine-empty = Digite algo primeiro.
find-refine-no-change = Isso não muda nada.

## Getting and subscribing

find-getting = Baixando { $what }.
find-subscribed = Seguindo: os novos episódios baixam sozinhos.
find-subscribed-desk = Seguindo em { $name }: os novos episódios baixam lá sozinhos.
find-subscribed-as = Seguindo como { $name }: os novos episódios baixam sozinhos.
find-downloading = { $count ->
    [one] Baixando { $count } release.
   *[other] Baixando { $count } releases.
}
find-not-added = Não incluído: { $error }
find-not-subscribed = Não está seguindo: { $error }
find-desk-refused = { $name } não aceitou: { $error }
find-desk-not-subscribed = Não está seguindo em { $name }: { $error }

## Player

player-subtitles = Legendas
player-audio = Áudio
player-speed = Veloc.
player-connecting = Conectando
player-hold-speed = Veloc. 2×
player-replay = Repetir
player-unlock = Liberar
player-next-stay = Ficar
player-next-go = Próximo
player-jump-back = −10 s
player-jump-forward = +10 s
player-speed-now = Veloc. { $speed }
player-skip-intro = Pular abertura
player-skip-outro = Pular créditos
player-skipped-intro = Abertura pulada
player-skipped-outro = Créditos pulados
player-undo = Desfazer
player-start-over = Recomeçar
player-resumed-at = Retomado em { $time }
player-picture-failed = A imagem não iniciou. O som continua.
player-file-failed = Este arquivo não foi reproduzido: { $why }.
player-streaming-from = Transmitindo de { $host } em { $height }p
player-mpv-no-start = o mpv não iniciou
player-mpv-no-start-why = o mpv não iniciou: { $why }
player-android-only = o player só roda no Android

## Player: delays

player-delay-title = Atraso
player-delay-note = Cada toque move 0,1 s. Vale até o próximo arquivo.
player-delay-seconds = { $value } s
player-sub-delay-now = Atraso da legenda { $value }
player-audio-delay-now = Atraso do áudio { $value }

## Player: cast

player-cast-playing-on = Reproduzindo em { $device }
player-cast-stop = Parar de transmitir para { $device }
player-cast-searching = Procurando Chromecasts e TVs
player-cast-to = Transmitir
player-cast-tv = TV
player-cast-loading = Carregando
player-cast-playing = Tocando
player-cast-paused = Pausado
player-cast-finished = Concluído
player-cast-stopped = Parado

## Player: the tracker's word

player-rated = Avaliado
player-score-failed = Não foi possível salvar a nota
player-marked-watched = Marcado como visto
player-tracked = Registrado
player-not-tracked = Não registrado
player-track-update-failed = Não foi possível atualizar sua lista
player-not-tracked-why = Não registrado: { $reason }.
player-track-ok-at = { $tracker } em { $progress }
player-track-ok = { $tracker }: ok
player-track-note = { $tracker }: { $note }
player-track-failed = Não foi possível atualizar { $tracker }
player-track-line = { $head }: { $parts }
player-refusal-hidden = a série está oculta
player-refusal-no-match = a série não tem vínculo com a lista
player-refusal-not-newer = a lista já passou deste episódio
player-refusal-extra = isto é um extra
player-refusal-unmatched = o arquivo não está ligado a um episódio
player-refusal-on-disk = o arquivo só está no disco

## Subtitles

subs-line-default = Tamanho 1×, arquivos simples usam seu estilo
subs-sample = Onde você escondeu isso?
subs-size-place-title = Tamanho e posição
subs-size = Tamanho
subs-position = Posição
subs-position-helper = 100 fica na borda inferior. Números menores sobem as linhas na tela.
subs-style-title = Estilo
subs-styled = Legendas estilizadas
subs-styled-as-made = Arquivos estilizados (ASS) ficam exatamente como os autores definiram.
subs-styled-mine = Seu estilo abaixo vale para todos os arquivos. Letreiros e karaokê podem quebrar.
subs-styled-size-only = Arquivos estilizados mantêm o visual e usam seu tamanho. Arquivos simples usam seu estilo.
subs-mode-as-made = Original
subs-mode-size-only = Só tamanho
subs-mode-mine = Meu
subs-font = Fonte
subs-font-bad = Digite o nome de uma fonte, como Noto Sans.
subs-bold = Negrito
subs-colour = Cor do texto
subs-colour-bad = Digite uma cor como #ffcc00.
subs-outline = Contorno
subs-outline-bad-width = A largura é um número, 0 ou mais.
subs-outline-bad-colour = Digite a cor como #000000.
subs-outline-helper = Primeiro a largura, depois a cor.
subs-shadow = Sombra
subs-shadow-bad = Um número, 0 ou mais. 0 significa sem sombra.
subs-shadow-helper = Quanto a sombra cai, 0 para nenhuma.
subs-box = Caixa de fundo
subs-box-helper = Uma caixa escura atrás das linhas. 0 deixa sem caixa.
subs-reset-title = Restaurar
subs-reset-helper = Volta o tamanho, a fonte, as cores, o contorno, a caixa e a posição ao padrão.
subs-reset = Redefinir

## Streaming

stream-quality-auto = Auto
stream-quality-original = Original
stream-app-starting = O app ainda está iniciando
stream-off-on-host = O streaming está desativado em { $host }. Ative "Stream episodes" lá, em Settings, Phone app.
stream-cannot-stream = { $host } roda o AniBeam { $version }, que ainda não faz streaming. Atualize-o lá.
stream-no-session = O outro dispositivo não deu uma sessão
stream-answer-unreadable = A resposta não pôde ser lida: { $why }
stream-answered-status = { $name } respondeu { $status }

## Stats

stats-nothing-yet = Nada ainda
stats-empty-title = Nada assistido ainda
stats-empty-body = Termine um episódio e ele conta aqui: horas, gêneros e as temporadas que você assistiu.
stats-hours-caption = horas assistidas
stats-finished-caption = séries concluídas
stats-busiest-caption = mês mais ativo
stats-episodes-caption = { $count ->
    [one] episódio
   *[other] episódios
}
stats-by-month = Por mês
stats-top-genres = Top gêneros
stats-top-studios = Top estúdios
stats-seasons = Temporadas vistas
stats-watched-most = Mais assistidos
stats-year-line = O que você assistiu em { $year }
stats-eps = { $count ->
    [one] { $count } ep.
   *[other] { $count } eps.
}
stats-month-1 = Jan
stats-month-2 = Fev
stats-month-3 = Mar
stats-month-4 = Abr
stats-month-5 = Mai
stats-month-6 = Jun
stats-month-7 = Jul
stats-month-8 = Ago
stats-month-9 = Set
stats-month-10 = Out
stats-month-11 = Nov
stats-month-12 = Dez
stats-month-letter-1 = J
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

backup-title = Backup
backup-export = Backup da biblioteca
backup-export-helper = Um arquivo com suas pastas, vínculos e o que você assistiu.
backup-private = Incluir dados privados
backup-private-helper = Inclui logins do AniList e do MyAnimeList, histórico e preferências. Os logins ficam em texto simples, então guarde o arquivo só para você.
backup-restore = Restaurar backup
backup-restore-helper = Lê um backup para esta biblioteca. O que está no disco prevalece; o arquivo completa o resto.
backup-done = Backup feito. O arquivo está onde você salvou.
backup-not-saved = Não salvo. Nada foi gravado.
backup-starting = O AniBeam ainda está iniciando. Tente de novo em instantes.
backup-images-title = Imagens
backup-images-body = Pôsteres, banners e retratos ficam no celular para a biblioteca funcionar offline. As imagens apagadas são baixadas de novo quando as telas precisam.
backup-keep = Manter
backup-removed = { $count ->
    [one] { $count } imagem removida
   *[other] { $count } imagens removidas
}
backup-images-none = Sem imagens em cache
backup-images-unmeasured = Não foi possível medir o cache de imagens
backup-images-retry-note = Tente de novo em instantes. As imagens em si estão bem.
backup-images-line = { $size } em { $count ->
    [one] { $count } imagem
   *[other] { $count } imagens
}
backup-images-question = { $count ->
    [one] Remover { $count } imagem?
   *[other] Remover { $count } imagens?
}

backup-sum-list = { $items }.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] { $count } pasta adicionada
   *[other] { $count } pastas adicionadas
}
backup-sum-series = { $count ->
    [one] { $count } série criada
   *[other] { $count } séries criadas
}
backup-sum-matches = { $count ->
    [one] { $count } vínculo aplicado
   *[other] { $count } vínculos aplicados
}
backup-sum-views = Mesclados: { $views ->
    [one] { $views } registro de exibição
   *[other] { $views } registros de exibição
} e { $episodes ->
    [one] { $episodes } episódio concluído
   *[other] { $episodes } episódios concluídos
}
backup-sum-resume = { $count ->
    [one] { $count } ponto de retomada mesclado
   *[other] { $count } pontos de retomada mesclados
}
backup-sum-accounts = { $count ->
    [one] { $count } conta de lista importada
   *[other] { $count } contas de lista importadas
}
backup-sum-fixes = { $count ->
    [one] { $count } arquivo usa uma correção que você fez.
   *[other] { $count } arquivos usam uma correção que você fez.
}
backup-sum-missing = { $count ->
    [one] { $count } pasta não foi encontrada neste celular.
   *[other] { $count } pastas não foram encontradas neste celular.
}
backup-sum-left-out = Não incluído: { $fields }.

## Activity

activity-everything = Tudo
activity-problems = Problemas
activity-copied = Copiado
activity-clear-every-line = Apagar tudo
activity-fold = Recolher
activity-no-problems = Nenhum problema. Tudo funcionou.
activity-nothing-yet = Nada a relatar ainda. O que o AniBeam faz por você aparece aqui.
activity-time-today = %H:%M:%S
activity-time-other = %-d %b %H:%M

## Dates and numbers (companion format.rs)

fmt-clock = %H:%M
fmt-day-month = %-d %b
fmt-weekday-date-time = %a, %-d %b %H:%M
fmt-weekday-date-year-time = %a, %-d %b %Y %H:%M
fmt-stamp = %-d %b %H:%M
fmt-decimal-point = ,

fmt-today-at = Hoje às { $time }
fmt-tomorrow-at = Amanhã às { $time }
fmt-yesterday-at = Ontem às { $time }

fmt-span-days = { $d } d { $h } h
fmt-span-hours = { $h } h { $m } min
fmt-span-minutes = { $m } min { $s } s
fmt-span-seconds = { $s } s
fmt-in = em { $span }
fmt-full-days = { $d } d { $h } h { $m } min { $s } s
fmt-full-hours = { $h } h { $m } min { $s } s
fmt-full-minutes = { $m } min { $s } s

fmt-ago-now = agora mesmo
fmt-ago-minutes = há { $count } min
fmt-ago-hours = há { $count } h
fmt-ago-days = há { $count } d

fmt-bytes-b = { $value } B
fmt-bytes-kb = { $value } KB
fmt-bytes-mb = { $value } MB
fmt-bytes-gb = { $value } GB
fmt-bytes-tb = { $value } TB

## Talking to the computer (companion client.rs, sync.rs, address.rs)

client-not-found = O celular não encontra { $host }. O Tailscale está conectado?
client-timeout = { $host } não respondeu. Está ligado, e o Tailscale está conectado nos dois?
client-connect-failed = Não foi possível acessar { $host }. Está ligado, e o Tailscale está conectado nos dois?
client-refused = { $host } está ligado, mas o AniBeam nele está fechado ou não está compartilhando. Abra o AniBeam nele e ative Share over Tailscale em Settings, Phone app.
client-no-route = Sem rota para { $host }. O Tailscale está conectado?
client-invalid-input = Não foi possível conectar a { $host }
client-unreachable = Não foi possível acessar { $host }: { $why }
client-cannot-find = O celular não encontra { $host }. O Tailscale está conectado?
client-broke-off = A resposta de { $host } foi interrompida: { $why }
client-answered = { $host } respondeu { $status } para { $path }
client-not-anibeam-status = Algo respondeu, mas não é o AniBeam ({ $host } respondeu { $status } para { $path })
client-not-anibeam-hello = Algo respondeu, mas não é o AniBeam (a resposta não é o hello do AniBeam)
client-not-anibeam-name = Algo respondeu, mas não é o AniBeam (ele se chama { $name })
client-too-new = O host fala a API { $host_api }; este app lê a { $app_api }. Atualize o app.
client-snapshot-unreadable = Não foi possível ler o snapshot: { $why }
client-probe-unreadable = Não foi possível ler os detalhes do arquivo: { $why }
client-peers-unreadable = Não foi possível ler a lista de AniBeams: { $why }
sync-save-failed = Não foi possível salvar o snapshot: { $why }
address-https = O AniBeam do seu computador responde em http simples pelo Tailscale; tire o https://.
address-empty = Digite o nome ou o endereço do host, por exemplo my-computer:8790.
address-ipv6-close = Um endereço IPv6 precisa do ] de fechamento.
address-ipv6-port = Coloque a porta depois do ], como [fd7a::1]:8790.
address-bad-host = "{ $text }" não é um nome de host.
address-bad-port = "{ $port }" não é uma porta.

## Series facts (companion library.rs)

clib-status-airing = No ar
clib-status-not-yet = Ainda não exibido
clib-format-tv-short = TV curta
clib-format-movie = Filme
clib-format-special = Especial
clib-format-music = Música

## Search sites and repos

repos-by-hand-title = Incluir à mão
repos-by-hand-note = Para um site que nenhum repo tem. Qualquer feed com { "{query}" } onde vai a busca, inclusive um feed do Jackett ou do Prowlarr. Uma chave ou token no endereço vai para o armazenamento seguro deste celular.
repos-name-placeholder = Nome
repos-url-placeholder = https://...{ "{query}" }...
repos-index-placeholder = https://.../index.json
repos-add-site = Incluir site
repos-add-title = Incluir repo
repos-add-note = Cole o endereço do índice de um repo. O AniBeam confere a assinatura e mostra a chave antes de salvar qualquer coisa.
repos-add-repo = Incluir repo
repos-trust-title = Confiar neste repo?
repos-fingerprint = Impressão da chave
repos-trust-warning = Os sites dele decidem para onde vão suas buscas. Confie só em repos de quem você conhece.
repos-trust = Confiar
repos-open-link = Abrir { $url }
repos-saving = Salvando...
repos-install = Instalar
repos-update = Atualizar
repos-edit-note = Uma chave ou token digitado no endereço vai para o armazenamento seguro deste celular.
repos-edit-note-keys = Deixe cada { "{key}" } no endereço como está; o valor fica neste celular.
repos-change-its = Mudar { $keys }
repos-check-repos = Buscar novos sites nos repos
repos-check-sites = Verificar os sites
repos-add-site-label = Incluir um site à mão
repos-needs-before-search = Precisa de { $labels } para buscar
repos-no-longer-offered = Não é mais oferecido pelo repo
repos-update-host = A atualização envia requisições para { $host }
repos-tap-change-custom = Toque para mudar o nome ou o endereço
repos-tap-change-keys = Toque para mudar { $labels }
repos-line-custom = { $host }  { $kind }  incluído por você
repos-update-to = Atualizar para v{ $version }
repos-add-its-value = Incluir o valor
repos-asks-for = Pede { $labels }
repos-preview-line = { $count ->
    [one] { $count } site, assinado
   *[other] { $count } sites, assinados
}
repos-preview-trusted = { $count ->
    [one] { $count } site, assinado, já confiável
   *[other] { $count } sites, assinados, já confiáveis
}
repos-ask-kept-empty = Guardado só neste celular. Um campo vazio mantém o que já está guardado.
repos-ask-kept-never = Guardado só neste celular, nunca na biblioteca nem em exportações.
repos-ask-new-host = A nova versão envia isso para { $host }, então pede de novo.
repos-ask-new = A nova versão pede esses dados de novo.
repos-type-value = Digite um valor primeiro.
repos-health-answered = Respondeu em { $ms } ms
repos-health-failed = Não respondeu. { $step }: { $detail }

## Downloads

dl-title = Downloads
dl-wifi-only = Só Wi-Fi
dl-wifi-and-mobile = Wi-Fi e dados móveis
dl-joined = { $first }, { $second }
dl-batch-files = { $count ->
    [one] Lote, { $count } arquivo
   *[other] Lote, { $count } arquivos
}
dl-batch-tag = Lote
dl-downloading-count = { $count ->
    [one] { $count } baixando
   *[other] { $count } baixando
}
dl-free-on-phone = livres no celular
dl-search-placeholder = Uma série ou nome de release
dl-sources = Fontes
dl-downloading = Baixando
dl-queue-note = { $count ->
    [one] { $count } episódio
   *[other] { $count } downloads
}
dl-finished = Concluídos
dl-finished-note = { $count ->
    [one] { $count } download, na sua biblioteca
   *[other] { $count } downloads, na sua biblioteca
}
dl-get = Baixar
dl-pick = Escolher

## Storage and hidden shows

disk-on-phone = Neste celular
disk-watched = Assistidos
disk-not-watched = Não assistidos
disk-deleting = Excluindo
disk-delete-watched = Excluir assistidos
disk-hidden-shows = Séries ocultas
disk-hidden-helper = Fora do Início e da biblioteca até você trazê-las de volta.
disk-unhide = Mostrar
disk-delete-note = Arquivos e legendas são excluídos deste celular em definitivo.
disk-every-watched = Episódios assistidos
disk-every-finished = Toda série concluída que você assistiu
disk-episodes-size = { $count ->
    [one] { $count } episódio, { $size }
   *[other] { $count } episódios, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } série, { $size }
   *[other] { $count } séries, { $size }
}
disk-list-two = { $a } e { $b }
disk-list-three = { $a }, { $b } e { $c }
disk-list-more = { $a }, { $b }, { $c } e mais { $count }
disk-delete-finished-title = Excluir { $count ->
    [one] { $count } série concluída
   *[other] { $count } séries concluídas
}, { $size }?
disk-delete-watched-title = Excluir { $count ->
    [one] { $count } episódio assistido
   *[other] { $count } episódios assistidos
}, { $size }?
disk-delete-finished-body = { $names }. Só séries que o provedor diz estarem concluídas, com todos os episódios assistidos; uma série ainda no ar, ou cujo fim é um palpite, fica. Todos os arquivos delas são excluídos deste celular, extras também. Os downloads delas saem junto, então podem ser baixados de novo.
disk-delete-watched-body = Todos os episódios que você assistiu, em todas as séries. Episódios ainda não assistidos e extras ficam. Os arquivos e as legendas são excluídos deste celular em definitivo.
disk-no-finished = Nenhuma série concluída foi assistida até o fim
disk-nothing-watched = Nada assistido neste celular
disk-deleted = { $count ->
    [one] { $count } episódio excluído
   *[other] { $count } episódios excluídos
}, { $size }
disk-delete-failed = Nada foi excluído: { $error }

## Download notifications

notify-card-head = AniBeam · agora
notify-sample-called = Frieren episódio 7
notify-sample-called-ready = Frieren episódio 6
notify-sample-facts = 420 MB de 1,4 GB, 3,2 MB/s, faltam 5 min
notify-downloading = Baixando { $called }
notify-progress = { $percent }%, { $facts }
notify-progress-of = { $percent }%, { $done } de { $size }
notify-starting = Iniciando
notify-many = { $count ->
    [one] { $count } download
   *[other] { $count } downloads
}
notify-called-episode = { $show } episódio { $n }
notify-called-episodes = { $show } episódios { $from } a { $to }
notify-ready = { $called } está pronto
notify-ready-many = { $called } estão prontos
notify-ready-batch = Um lote de { $show } está pronto
notify-ready-text = Baixado neste celular. Toque para abrir o AniBeam.
notify-channel-downloads = Downloads
notify-channel-ready = Episódios prontos
notify-fallback-title = Baixando
notify-row-on = Notificações
notify-row-allow = Permitir notificações
notify-row-open-settings = Abrir config. de notificações
notify-helper-on = Ativado. Os downloads aparecem nas notificações enquanto rodam.
notify-helper-asks-next = O Android pergunta.
notify-helper-asks-next-later = O Android pergunta. Dá para mudar de ideia depois em Mais.
notify-helper-off = Desativado. Ative as notificações do AniBeam lá.
notify-question-title = Ver downloads nas notificações?
notify-question-body = Enquanto os episódios baixam, uma notificação mostra o andamento, e outra avisa quando um está pronto. Eles baixam de qualquer jeito; isso só decide se você vê.
notify-allow = Permitir
notify-not-now = Agora não
notify-pointer = Os downloads não aparecem nas suas notificações.
notify-turn-on = Ativar

## Feed

feed-title = Feed
feed-recently-released = Lançados recentemente
feed-coming-soon = Em breve
feed-this-week = Esta semana
feed-earlier = Antes
feed-everything-else = Todo o resto
feed-countdown-days = { $d } d { $h } h
feed-countdown-hours = { $h } h { $m } min
feed-countdown-minutes = { $m } min
feed-next = Próximo: { $title } em { $countdown }
feed-nothing-scheduled = Nada programado ainda.
feed-nothing-new = Nada novo esta semana.
feed-new-episodes = { $count ->
    [one] { $count } novo episódio esta semana.
   *[other] { $count } novos episódios esta semana.
}
feed-aired = exibido { $ago }
feed-downloaded = baixado { $ago }

## Files and the file picker

files-already-picking = Já há um arquivo sendo escolhido.
files-took-too-long = O seletor de arquivos demorou demais.
files-did-not-open = O seletor de arquivos não abriu: { $error }
files-none-here = Não há seletor de arquivos aqui.

## The computer's downloads

desk-sent-adding = Enviado para { $name }: { $what }. Ele ainda está adicionando.
desk-sent = Baixando { $what } em { $name }.

## Your computer's library

remote-watched = Assistido
remote-movie = Filme
remote-not-watched-yet = Ainda não assistido
remote-watched-of = { $done } de { $total } assistidos
remote-watched-count = { $count } assistidos
remote-on-disk = { $count } no disco
remote-airs-in-season = T{ $season } episódio { $n } vai ao ar em { $countdown }
remote-airs-in = Episódio { $n } vai ao ar em { $countdown }
remote-play-episode = Assistir ep. { $n }
remote-watch-again = Reassistir ep. { $n }
remote-update-host = Atualize o AniBeam naquele computador para assistir aqui
remote-aired = exibido
remote-missing = Faltando
remote-on-computer = No seu computador
remote-out = Lançado { $ago }
remote-season-episode = Temporada { $season }, episódio { $n }
