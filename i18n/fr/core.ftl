# fr translation of en/core.ftl. An ID missing here is shown in English.

## Names that stay the same in every language. Messages use them; none is shown alone.

## The tailnet host

tailnet-person-no-name = Sans nom dans Tailscale

## Counts

count-episodes =
    { $count ->
        [one] { $count } épisode
       *[other] { $count } épisodes
    }
count-shows =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    }

## Actions every screen uses

action-cancel = Annuler
action-close = Fermer
action-try-again = Réessayer
action-remove = Retirer
action-delete = Supprimer
action-open = Ouvrir
action-save = Enregistrer
action-apply = Appliquer
action-clear = Effacer
action-refresh = Actualiser
action-play = Lire
action-connect = Connecter
action-start = Démarrer
action-back = Retour
action-see-all = Tout voir
action-open-series = Voir la série
action-open-settings = Ouvrir les réglages

## The lists a tracker keeps

status-watching = En cours
status-completed = Terminé
status-paused = En pause
status-on-hold = En attente
status-dropped = Abandonné
status-planning = À voir
status-rewatching = Revisionnage

## Episodes

episode-number = Épisode { $number }

## AniList's words for a show: its format, its airing status, how two shows relate

anilist-format-tv = TV
anilist-format-tv-short = TV court
anilist-format-film = Film
anilist-format-special = Spécial
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = Musique
anilist-status-finished = Terminée
anilist-status-airing = En diffusion
anilist-status-not-yet-aired = Pas encore diffusée
anilist-status-cancelled = Annulée
anilist-status-hiatus = En pause
anilist-relation-adaptation = Adaptation
anilist-relation-prequel = Préquelle
anilist-relation-sequel = Suite
anilist-relation-parent = Œuvre mère
anilist-relation-side-story = Annexe
anilist-relation-character = Personnage
anilist-relation-summary = Résumé
anilist-relation-alternative = Alternative
anilist-relation-spin-off = Spin-off
anilist-relation-other = Autre
anilist-relation-source = Source
anilist-relation-compilation = Compilation
anilist-relation-contains = Contient

## The activity log: a problem whose place had no words of its own

event-problem-library = Un problème est survenu avec votre bibliothèque. Les options développeur donnent les détails.
event-problem-metadata = Un problème est survenu avec les infos des séries. Les options développeur donnent les détails.
event-problem-trackers = Un problème est survenu avec vos listes. Les options développeur donnent les détails.
event-problem-franchise = Un problème est survenu avec les séries liées. Les options développeur donnent les détails.
event-problem-playback = Un problème est survenu pendant la lecture. Les options développeur donnent les détails.
event-problem-store = Un problème est survenu à l'enregistrement de vos données. Les options développeur donnent les détails.
event-problem-system = Un problème est survenu dans AniBeam. Les options développeur donnent les détails.
event-problem-downloads = Un problème est survenu avec les téléchargements. Les options développeur donnent les détails.

## Errors (contract::CoreError)

core-error-not-found-source = Dossier { $id } introuvable.
core-error-not-found-series = Série { $id } introuvable.
core-error-not-found-file = Fichier { $id } introuvable.
core-error-not-found-session = Session de lecture { $id } introuvable.
core-error-not-found-job = Tâche { $id } introuvable.
core-error-invalid = Valeur invalide pour { $field } : { $message }.
core-error-invalid-ended = Valeur invalide pour { $field } : { $message }
core-error-unavailable = Ce dossier n'est pas disponible : { $path }
core-error-not-connected = { $tracker } n'est pas connecté.
core-error-refused-hidden = Impossible : la série est masquée.
core-error-refused-no-match = Impossible : la série n'est pas encore associée ; associez-la dans Infos de la série.
core-error-refused-not-newer = Impossible : votre liste a déjà dépassé cet épisode.
core-error-refused-extra = Impossible : c'est un bonus.
core-error-refused-unmatched = Impossible : le fichier n'est associé à aucun épisode.
core-error-refused-on-disk = Impossible : le fichier n'existe que sur le disque.
core-error-provider = { $provider } n'a pas répondu : { $message }
core-error-io = Impossible de lire ou d'écrire un fichier : { $message }
core-error-storage = Impossible d'enregistrer votre bibliothèque : { $message }
core-error-keyring = Impossible de joindre le gestionnaire de mots de passe du système : { $message }
core-error-unsupported = { $what } n'est pas encore pris en charge.
core-error-version = Cet export vient d'un AniBeam plus récent (format { $found }). Mettez AniBeam à jour pour l'importer.
core-error-internal = AniBeam a rencontré une erreur interne : { $message }
core-error-service = { $service } : { $message }

## core/src/repos/mod.rs

core-repos-paste-address = collez l'adresse d'un dépôt
core-repos-must-be-https = l'adresse doit être en https
core-repos-unreachable = impossible de joindre le dépôt : { $message }
core-repos-not-found = rien à cette adresse (un dépôt GitHub privé répond aussi 404)
core-repos-http-status = le dépôt a répondu HTTP { $status }
core-repos-unreadable = impossible de lire { $path } : { $error }
core-repos-no-such-endpoint = aucune source installée avec cet identifiant
core-repos-key-changed = le dépôt est maintenant signé par une autre clé que celle que vous avez approuvée. Retirez-le puis ajoutez-le de nouveau pour accepter la nouvelle clé.
core-repos-stale = le dépôt a servi un index plus ancien (série { $serial } après { $kept }) ; le plus récent est conservé
core-repos-key-not-shown = la clé du dépôt n'est pas celle qui vous a été montrée. Ajoutez-le de nouveau et comparez l'empreinte.
core-repos-repo-added = Liste de sources de téléchargement ajoutée.
core-repos-repo-removed = Liste de sources de téléchargement retirée.
core-repos-no-field = { $name } n'a pas de champ { $key }
core-repos-needs-field = { $name } a besoin de son champ { $label }
core-repos-host-only = { $name } : le champ { $label } accepte seulement un hôte ou hôte:port, comme 192.0.2.10:9696
core-repos-not-added = ce dépôt n'est pas ajouté
core-repos-not-offered = { $repo } ne propose pas { $id }
core-repos-already-installed = { $name } est déjà installé
core-repos-taken-by-you = une source nommée { $id } est déjà installée (ajoutée par vous) ; retirez-la d'abord
core-repos-taken-by-repo = une source nommée { $id } est déjà installée (depuis un autre dépôt) ; retirez-la d'abord
core-repos-source-added = Source de téléchargement ajoutée.
core-repos-source-updated = Source de téléchargement mise à jour.
core-repos-source-removed = Source de téléchargement retirée.
core-repos-source-changed = Source de téléchargement modifiée.
core-repos-give-name = donnez-lui un nom
core-repos-put-query = mettez {"{"}query{"}"} dans l'adresse, là où va la recherche
core-repos-name-taken = une source nommée { $id } est déjà installée
core-repos-only-by-hand = seule une source ajoutée à la main peut être modifiée
core-repos-keep-key = gardez {"{"}{ $key }{"}"} dans l'adresse et changez sa valeur avec Modifier le champ { $label }, ou retirez le site et ajoutez-le de nouveau sans lui
core-repos-note-kind = anirss ne sait pas chercher avec le type { $kind }, donc l'ordinateur l'ignore ; le téléphone le cherche
core-repos-note-by-hand = anirss a son propre { $id }, écrit à la main ; l'ordinateur cherche celui-là
core-repos-note-theirs = Cet anirss est le vôtre, et l'ordinateur ne touche jamais à sa configuration : ajoutez { $id } à la configuration de cet anirss pour que l'ordinateur le cherche
core-repos-note-sandbox = Cette session n'écrit jamais dans la configuration d'anirss, donc l'ordinateur ne cherche pas { $id } ici
core-repos-note-old = anirss { $version } ne peut pas recevoir de sources d'{ -app-name } ; passez à la version { $min } ou plus récente pour que l'ordinateur cherche celle-ci

## core/src/repos/index.rs

core-repos-key-not-ed25519 = la clé de l'index n'est pas une clé ed25519
core-repos-key-not-base64 = la clé de l'index n'est pas 32 octets en base64
core-repos-key-invalid = la clé de l'index n'est pas une clé ed25519 valide
core-repos-too-large = le fichier est trop gros pour être un dépôt
core-repos-not-a-repo = ce n'est pas un dépôt de sources { -app-name }
core-repos-format-version = le dépôt est au format { $found } ; cet { -app-name } lit le format { $reads }. Mettez { -app-name } à jour.
core-repos-payload-not-base64 = le contenu du dépôt n'est pas en base64
core-repos-signature-malformed = la signature du dépôt est mal formée
core-repos-index-unreadable = l'index du dépôt est illisible : { $error }
core-repos-signature-mismatch = la signature du dépôt ne correspond pas à son contenu
core-repos-check-scheme = { $id } : l'adresse doit commencer par https://
core-repos-check-scheme-http = { $id } : l'adresse doit commencer par http:// ou https://
core-repos-check-auth-key = { $id } : clé d'authentification incorrecte { $key }
core-repos-check-host-torznab = { $id } : seule une adresse torznab peut donner son hôte sous la forme {"{"}{ $key }{"}"}
core-repos-check-host-placeholder = { $id } : un espace réservé dans l'hôte

## core/src/playback/tracks.rs

core-playback-track-number = Piste { $number }
core-playback-track-file = { $label } (fichier)
core-playback-lang-region = { $name } ({ $region })
core-playback-lang-english = Anglais
core-playback-lang-japanese = Japonais
core-playback-lang-spanish = Espagnol
core-playback-lang-portuguese = Portugais
core-playback-lang-french = Français
core-playback-lang-german = Allemand
core-playback-lang-italian = Italien
core-playback-lang-russian = Russe
core-playback-lang-chinese = Chinois
core-playback-lang-korean = Coréen
core-playback-lang-arabic = Arabe
core-playback-lang-hindi = Hindi
core-playback-lang-indonesian = Indonésien
core-playback-lang-malay = Malais
core-playback-lang-thai = Thaï
core-playback-lang-vietnamese = Vietnamien
core-playback-lang-polish = Polonais
core-playback-lang-turkish = Turc
core-playback-lang-dutch = Néerlandais
core-playback-lang-swedish = Suédois
core-playback-lang-norwegian = Norvégien
core-playback-lang-danish = Danois
core-playback-lang-finnish = Finnois
core-playback-lang-hebrew = Hébreu
core-playback-lang-ukrainian = Ukrainien
core-playback-lang-czech = Tchèque
core-playback-lang-hungarian = Hongrois
core-playback-lang-greek = Grec
core-playback-lang-romanian = Roumain
core-playback-lang-filipino = Filipino
core-playback-region-brazil = Brésil
core-playback-region-portugal = Portugal
core-playback-region-latin-america = Amérique latine
core-playback-region-spain = Espagne
core-playback-region-simplified = simplifié
core-playback-region-traditional = traditionnel

## core/src/contract/enums.rs

core-jobs-scan = Vérification de vos dossiers
    .stopped = La vérification de vos dossiers s'est arrêtée sur un problème.
core-jobs-auto-match = Association des séries
    .stopped = L'association des séries s'est arrêtée sur un problème.
core-jobs-search = Recherche d'une série
    .stopped = La recherche d'une série s'est arrêtée sur un problème.
core-jobs-resolve-link = Lecture d'un lien
    .stopped = La lecture d'un lien s'est arrêtée sur un problème.
core-jobs-apply-match = Association d'une série
    .stopped = L'association d'une série s'est arrêtée sur un problème.
core-jobs-refresh = Mise à jour d'une série
    .stopped = La mise à jour d'une série s'est arrêtée sur un problème.
core-jobs-refresh-all = Mise à jour des infos des séries
    .stopped = La mise à jour des infos des séries s'est arrêtée sur un problème.
core-jobs-refresh-airing = Vérification des diffusions
    .stopped = La vérification des diffusions s'est arrêtée sur un problème.
core-jobs-refresh-imdb = Mise à jour des notes
    .stopped = La mise à jour des notes s'est arrêtée sur un problème.
core-jobs-clear-images = Effacement des images
    .stopped = L'effacement des images s'est arrêté sur un problème.
core-jobs-fill-images = Récupération des images
    .stopped = La récupération des images s'est arrêtée sur un problème.
core-jobs-connect-tracker = Connexion de votre liste
    .stopped = La connexion de votre liste s'est arrêtée sur un problème.
core-jobs-mark = Marquage d'un épisode comme vu
    .stopped = Le marquage d'un épisode comme vu s'est arrêté sur un problème.
core-jobs-set-progress = Mise à jour de votre progression
    .stopped = La mise à jour de votre progression s'est arrêtée sur un problème.
core-jobs-score = Enregistrement de votre note
    .stopped = L'enregistrement de votre note s'est arrêté sur un problème.
core-jobs-refresh-progress = Lecture de votre progression
    .stopped = La lecture de votre progression s'est arrêtée sur un problème.
core-jobs-refresh-watching = Lecture de votre liste En cours
    .stopped = La lecture de votre liste En cours s'est arrêtée sur un problème.
core-jobs-refresh-airing-now = Vérification de cette saison
    .stopped = La vérification de cette saison s'est arrêtée sur un problème.
core-jobs-refresh-recommended = Recherche de séries pour vous
    .stopped = La recherche de séries pour vous s'est arrêtée sur un problème.
core-jobs-read-show-page = Lecture d'une série
    .stopped = La lecture d'une série s'est arrêtée sur un problème.
core-jobs-on-hold = Passage des séries inactives en attente
    .stopped = Le passage des séries inactives en attente s'est arrêté sur un problème.
core-jobs-set-status = Changement d'un statut de liste
    .stopped = Le changement d'un statut de liste s'est arrêté sur un problème.
core-jobs-move-lists = Transfert de votre liste vers votre suivi
    .stopped = Le transfert de votre liste vers votre suivi s'est arrêté sur un problème.
core-jobs-notes = Enregistrement de vos notes
    .stopped = L'enregistrement de vos notes s'est arrêté sur un problème.
core-jobs-crawl = Recherche des séries liées
    .stopped = La recherche des séries liées s'est arrêtée sur un problème.
core-jobs-skip-windows = Recherche de l'opening et de l'ending
    .stopped = La recherche de l'opening et de l'ending s'est arrêtée sur un problème.
core-jobs-export = Enregistrement de votre bibliothèque dans un fichier
    .stopped = L'enregistrement de votre bibliothèque dans un fichier s'est arrêté sur un problème.
core-jobs-import = Chargement de votre bibliothèque depuis un fichier
    .stopped = Le chargement de votre bibliothèque depuis un fichier s'est arrêté sur un problème.
core-jobs-subscriptions = Vérification de vos abonnements
    .stopped = La vérification de vos abonnements s'est arrêtée sur un problème.
core-jobs-connect-downloads = Connexion de l'appli de téléchargement
    .stopped = La connexion de l'appli de téléchargement s'est arrêtée sur un problème.
core-jobs-search-releases = Recherche d'épisodes
    .stopped = La recherche d'épisodes s'est arrêtée sur un problème.
core-jobs-refine-releases = Affinage des épisodes
    .stopped = L'affinage des épisodes s'est arrêté sur un problème.
core-jobs-subscribe = Abonnement à une série
    .stopped = L'abonnement à une série s'est arrêté sur un problème.
core-jobs-download = Ajout de téléchargements
    .stopped = L'ajout de téléchargements s'est arrêté sur un problème.
core-jobs-remove-subscription = Suppression d'un abonnement
    .stopped = La suppression d'un abonnement s'est arrêtée sur un problème.
core-jobs-control-torrents = Modification de vos téléchargements
    .stopped = La modification de vos téléchargements s'est arrêtée sur un problème.
core-jobs-test-torrent-client = Test de l'appli de téléchargement
    .stopped = Le test de l'appli de téléchargement s'est arrêté sur un problème.
core-jobs-remove-show = Suppression d'une série
    .stopped = La suppression d'une série s'est arrêtée sur un problème.
core-jobs-repair-subscription = Réparation d'un abonnement
    .stopped = La réparation d'un abonnement s'est arrêtée sur un problème.
core-jobs-check-endpoints = Vérification de vos sources de téléchargement
    .stopped = La vérification de vos sources de téléchargement s'est arrêtée sur un problème.
core-jobs-start-endpoint = Démarrage d'une source de téléchargement
    .stopped = Le démarrage d'une source de téléchargement s'est arrêté sur un problème.
core-jobs-schedule-agent = Demande d'un planning à votre agent de code
    .stopped = La demande d'un planning à votre agent de code s'est arrêtée sur un problème.

## core/src/airing_now.rs

core-airing-now-now = En diffusion
core-airing-now-winter = Hiver { $year }
core-airing-now-spring = Printemps { $year }
core-airing-now-summer = Été { $year }
core-airing-now-fall = Automne { $year }
core-airing-now-planned = { $title } ajoutée à votre liste À voir sur { $site }.
core-airing-now-planned-both = { $title } ajoutée à votre liste À voir sur { $site } et { $other }.
core-airing-now-already-on =
    { $title } est déjà dans votre liste { $site }, en { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }.
core-airing-now-plan-failed = Impossible d'ajouter { $title } à votre liste À voir. Réessayez plus tard.

## core/src/aired.rs

core-aired-episode-out = L'épisode { $number } de { $title } est sorti.

## core/src/cast/mod.rs

core-cast-connecting = Diffusion vers { $device }.
core-cast-ended = Diffusion vers { $device } terminée.
core-cast-stopped = Diffusion vers { $device } arrêtée.
core-cast-failed = La diffusion vers { $device } a échoué. Vérifiez qu'il est sur le même réseau.
core-cast-not-built = La diffusion n'est pas incluse dans cette version
core-cast-serve-failed = impossible de servir l'épisode : { $error }
core-cast-device-gone = Cet appareil n'est plus sur le réseau
core-cast-not-a-playlist = ce n'est pas une adresse de playlist
core-cast-nothing = Aucune diffusion en cours
core-cast-still-starting = L'appareil démarre encore
core-cast-no-route = cet appareil ne peut pas joindre la TV
core-cast-no-control = la TV n'a donné aucune adresse de contrôle
core-cast-not-handed = l'autre ordinateur n'a pas transmis l'épisode : { $error }
core-cast-empty-playlist = l'autre ordinateur a envoyé une playlist vide
core-cast-refused-both = { $error } ; elle a aussi refusé le fichier tel quel ({ $first })
core-cast-chromecast-stopped = le Chromecast s'est arrêté
core-cast-hevc = { $error }. Cet épisode est en HEVC, que seuls les Chromecast récents (avec Google TV, ou Ultra) décodent

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = impossible de joindre le Chromecast : { $error }
core-cast-chromecast-broke = la connexion au Chromecast a été coupée : { $error }
core-cast-chromecast-no-player = le Chromecast n'a pas lancé son lecteur
core-cast-chromecast-refused-player = le Chromecast refuse de lancer son lecteur ({ $reason })
core-cast-no-reason = sans raison
core-cast-chromecast-cannot-play = le Chromecast n'a pas pu lire ceci ({ $kind })
core-cast-chromecast-error = le Chromecast s'est arrêté sur une erreur en lisant ceci

## core/src/cast/dlna.rs

core-cast-tv-no-answer = la TV n'a pas répondu : { $error }
core-cast-tv-refused = la TV a refusé { $action }
core-cast-tv-refused-code = la TV a refusé { $action } ({ $code })

## core/src/cast/media.rs

core-cast-no-video = Ce fichier n'a pas de vidéo.
core-cast-hi10p = Cet épisode est en H.264 10 bits (Hi10P), qu'aucune TV ni aucun Chromecast ne décode. Il faudrait le transcoder, ce qui n'existe pas encore.
core-cast-video-codec = La vidéo de cet épisode ({ $codec }) ne peut pas être diffusée.
core-cast-audio-codec = L'audio de cet épisode ({ $codec }) ne peut pas être diffusé.

## core/src/phone/mod.rs

core-phone-download-started =
    { $count ->
        [one] Téléchargement d'un épisode de { $folder } lancé.
       *[other] Téléchargement de { $count } épisodes de { $folder } lancé.
    }
core-phone-subscribed = Abonné à { $name }.
core-phone-rule-stopped = { $name } n'est plus vérifiée pour les nouveaux épisodes : sa source a disparu ou demande de nouveau sa clé.
core-phone-new-episode = Un nouvel épisode de { $name } est en téléchargement.
core-phone-name-taken = { $name } : une autre source porte ce nom
core-phone-nothing-to-download = rien à télécharger
core-phone-over-cap = { -app-name } dépasserait sa limite de stockage ({ $used } sur { $cap } Go utilisés ou en cours) ; supprimez des épisodes ou relevez la limite
core-phone-empty-search = la recherche est vide
core-phone-no-results = aucun résultat pour { $query }
core-phone-own-engine = un moteur torrent intégré

## core/src/phone/check.rs

core-phone-check-connected = connecté
core-phone-check-refused = refusé : { $error }
core-phone-check-dropped = pas de réponse (ignoré)
core-phone-check-finding = { $label } : recherche de son adresse
core-phone-check-failed = échec : { $error }
core-phone-check-no-answer = pas de réponse
core-phone-check-private = { $label } : recherche privée
core-phone-check-skipped-address = ignoré, c'est une adresse
core-phone-check-skipped-home = ignoré, nom du réseau local
core-phone-check-no-record = pas d'adresse publique, laissé au DNS de ce réseau
core-phone-check-no-resolver = aucun résolveur n'a répondu
core-phone-check-reaching = { $label } : connexion à { $ip }
core-phone-check-http = HTTP { $status }, { $bytes } octets
core-phone-check-setup = préparation

## core/src/playback/session.rs

core-playback-not-saved = Votre progression sur { $folder } n'a pas été enregistrée. Associez d'abord la série.
core-playback-still-downloading = l'épisode { $number } est encore en téléchargement, { $done } % fait

## core/src/playback/skip.rs

core-playback-bad-duration = une durée est un nombre positif de secondes

## core/src/agent.rs

core-agent-no-answer = l'agent n'a donné aucune réponse
core-agent-searching = Recherche de { $query } sur le web
core-agent-searching-schedule = Recherche du planning sur le web
core-agent-reading = Lecture de { $url }
core-agent-reading-page = Lecture d'une page
core-agent-using = Utilisation de { $tool }
core-agent-none = un agent de code sur cet ordinateur
core-agent-ended = { $name } s'est arrêté sans réponse
core-agent-too-long = { $name } a mis trop de temps

## core/src/core.rs

core-core-guest-trackers = les comptes de suivi de cet ordinateur sont ceux de son propriétaire, donc cet appareil n'y marque rien
core-core-guest-not-interested = Sans intérêt sur cet ordinateur est la liste de son propriétaire, donc cet appareil n'y change rien
core-core-guest-recommended = les recommandations de cet ordinateur viennent de la liste de son propriétaire
core-core-nothing-picked = rien de choisi

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider } n'est pas une source d'association
core-prefs-provider-count = { $provider } apparaît { $count } fois ; chaque source n'apparaît qu'une fois
core-prefs-outside = { $value } est hors de l'intervalle { $low } à { $high }
core-prefs-empty = vide

## core/src/recommended/mod.rs

core-recommended-ranking = Classement des séries pour vous

## core/src/recommended/sync.rs

core-recommended-finding-people = Recherche de gens qui ont vos goûts
core-recommended-reading-lists = Lecture de leurs listes
core-recommended-reading-yours = Lecture de vos séries
core-recommended-reading-suggestions = Lecture des séries à suggérer

## core/src/recommended/page.rs

core-recommended-no-anime = { -anilist } n'a pas d'anime { $id }

## core/src/host/mod.rs

core-host-off = Désactivé
core-host-starting = Démarrage
core-host-serving = En service sur { $addresses }
core-host-no-tailscale = Tailscale n'est pas connecté
core-host-cannot-listen = Écoute impossible
core-host-port-taken = Le port { $port } est déjà pris
core-host-cannot-listen-on = Impossible d'écouter sur { $address } : { $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text } n'est pas une adresse avec un port

## core/src/host/keys.rs

core-host-not-a-key = { $key } n'est pas une clé d'appareil

## core/src/net/limiter.rs

core-net-timed-out = délai dépassé après { $ms } ms
core-net-rate-limited = { $provider } limite le nombre de requêtes

## core/src/net/mod.rs

core-net-waiting = { $host } : délai dépassé il y a un instant ; { -app-name } attend { $seconds } s avant de le relancer

## core/src/net/anilist.rs

core-net-no-data = aucune donnée

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } n'a pas de téléchargement pour cet ordinateur
core-update-cannot-make = Impossible de créer { $path } : { $error }
core-update-cannot-mark = Impossible de rendre la nouvelle AppImage exécutable : { $error }
core-update-cannot-replace = Impossible de remplacer { $path } : { $error }
core-update-not-here = Cette copie ne se met pas à jour ici
core-update-cannot-start = Impossible de lancer le téléchargement : { $error }
core-update-did-not-start = Le téléchargement n'a pas démarré : { $error }
core-update-cannot-write = Impossible d'écrire { $path } : { $error }
core-update-stopped = Le téléchargement s'est arrêté : { $error }
core-update-cancelled = Annulé
core-update-ended-early = Le téléchargement s'est arrêté trop tôt ({ $got } sur { $size } octets)
core-update-checksum = Le téléchargement ne correspond pas à la somme de contrôle de GitHub
core-update-cannot-move = Impossible de mettre le téléchargement en place : { $error }
core-update-cannot-open-dmg = Impossible d'ouvrir le dmg : { $error }
core-update-cannot-read-dmg = Impossible de lire le dmg : { $error }
core-update-no-app = Le dmg ne contient aucune appli
core-update-cannot-copy = Impossible de copier la nouvelle appli à côté de celle-ci : { $error }

## core/src/calendar.rs

core-calendar-season-episode = { $title } - Saison { $season } Épisode { $number }
core-calendar-episode = { $title } - Épisode { $number }

## core/src/paths.rs

core-paths-no-home = aucun dossier personnel

## core/src/first_run.rs

core-first-run-not-absolute = le dossier des vidéos doit être un chemin absolu

## core/src/presence/mod.rs

core-presence-extra = Bonus

## core/src/trackers/accounts.rs

core-trackers-no-client-id = Aucun identifiant client défini pour { $tracker }.
core-trackers-mal-reconnect = Reconnectez MyAnimeList dans les réglages.
core-trackers-mal-wants-secret = MyAnimeList demande le secret client de cette appli. Ajoutez-le sous Suivi dans les réglages, ou enregistrez l'appli avec le type « other ».
core-trackers-token-reply-unreadable = réponse de jeton illisible : { $error }
core-trackers-token-reply-no-token = la réponse ne contenait aucun jeton d'accès

## core/src/trackers/mod.rs

core-trackers-no-token = aucun jeton d'accès enregistré, reconnectez-vous dans les réglages

## core/src/trackers/cache.rs

core-trackers-no-user-id = le compte n'a pas d'identifiant utilisateur

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = aucune série choisie à transférer
core-trackers-moved-up =
    { $count ->
        [one] { $count } série transférée de cet ordinateur vers { $site }.
       *[other] { $count } séries transférées de cet ordinateur vers { $site }.
    }
core-trackers-moved-up-some-refused =
    { $moved ->
        [one] { $moved } série transférée
       *[other] { $moved } séries transférées
    } vers { $site } ; { $site } n'a pas accepté { $failed ->
        [one] { $failed } série
       *[other] { $failed } séries
    }.
core-trackers-move-refused = { $site } n'a pas accepté { $show } : { $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = L'autorisation a expiré, réessayez.
core-trackers-oauth-state-mismatch = état différent, possible CSRF, abandon.
core-trackers-oauth-no-code = aucun code dans le retour
core-trackers-oauth-no-token = aucun access_token dans le retour
core-trackers-oauth-waiting = En attente de votre connexion à { $site } dans votre navigateur.
core-trackers-oauth-connected = { $site } connecté en tant que { $username }.
core-trackers-oauth-no-name = le profil est revenu sans nom
core-trackers-oauth-could-not-start = La connexion n'a pas pu démarrer
core-trackers-oauth-port-failed = le port { $port } de cet ordinateur n'a pas pu être ouvert : { $error }
core-trackers-oauth-port-busy = le port { $port } de cet ordinateur est utilisé par un autre programme. Attendez un instant et réessayez, ou fermez le programme qui l'utilise s'il reste occupé.
core-trackers-oauth-provider-error = { $site } a renvoyé une erreur : { $error }
core-trackers-page-connected = { $site } est connecté à { $app }
core-trackers-page-close-tab = Vous pouvez fermer cet onglet.
core-trackers-page-failed-title = { $app }, échec de la connexion
core-trackers-page-failed = La connexion n'a pas abouti
core-trackers-page-go-back = Revenez à { $app } et réessayez.
core-trackers-page-finishing-title = { $app }, finalisation en cours
core-trackers-page-finishing = Finalisation en cours
core-trackers-page-finishing-body = { $site } a répondu et AniBeam finalise la connexion. Revenez à { $app }, qui vous dira quand c'est fait.
core-trackers-page-cancelled-title = { $app }, connexion annulée
core-trackers-page-cancelled = Cette connexion a été annulée
core-trackers-page-cancelled-body = Vous pouvez fermer cet onglet. Recommencez depuis { $app } si c'était voulu.

## core/src/trackers/on_hold.rs

core-trackers-on-hold-moving = Passage en attente sur { $tracker }
core-trackers-on-hold-moved =
    { $count ->
        [one] { $count } série passée en attente sur { $site }.
       *[other] { $count } séries passées en attente sur { $site }.
    }
core-trackers-on-hold-moved-two =
    { $count ->
        [one] { $count } série passée en attente sur { $site } et { $second } sur { $secondSite }.
       *[other] { $count } séries passées en attente sur { $site } et { $second } sur { $secondSite }.
    }
core-trackers-on-hold-failed =
    { $failed ->
        [one] { $failed } série n'a pas pu passer en attente.
       *[other] { $failed } séries n'ont pas pu passer en attente.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
        [one] { $count } série passée en attente sur { $site }.
       *[other] { $count } séries passées en attente sur { $site }.
    } { $failed ->
        [one] { $failed } série n'a pas pu passer en attente.
       *[other] { $failed } séries n'ont pas pu passer en attente.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
        [one] { $count } série passée en attente sur { $site } et { $second } sur { $secondSite }.
       *[other] { $count } séries passées en attente sur { $site } et { $second } sur { $secondSite }.
    } { $failed ->
        [one] { $failed } série n'a pas pu passer en attente.
       *[other] { $failed } séries n'ont pas pu passer en attente.
    }
core-trackers-series-fallback = série { $id }
core-trackers-back-on-watching = { $name } est de retour dans En cours sur { $site }.
core-trackers-back-on-watching-two = { $name } est de retour dans En cours sur { $site } et { $secondSite }.
core-trackers-back-on-watching-failed = { $name } n'a pas pu revenir dans En cours. Réessayez plus tard.

## core/src/trackers/writes.rs

core-trackers-rate-limited = { $tracker } limite les requêtes, réessayez dans une minute.
core-trackers-auth-expired = La connexion à { $tracker } a expiré, reconnectez-vous dans les réglages.
core-trackers-entry-not-found = Entrée introuvable sur { $tracker }.
core-trackers-server-error = Erreur serveur sur { $tracker } ({ $status }), réessayez plus tard.
core-trackers-error-see-log = Erreur sur { $tracker }, voir le journal d'activité.
core-trackers-episode-not-whole = { $episode } n'est pas un numéro d'épisode entier supérieur à zéro
core-trackers-score-out-of-range = { $score } est hors de l'intervalle 0 à 10
core-trackers-rewatch-not-settable = le revisionnage n'est pas un statut à choisir à la main
core-trackers-notes-too-long = les notes sont limitées à { $max } caractères
core-trackers-nothing-picked = rien de choisi
core-trackers-notes-not-on-list = Pas encore dans votre liste { $tracker }, ajoutez-la d'abord à une liste.
core-trackers-sites =
    { $sites ->
        [both] { -anilist } et { -mal }
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
       *[repeating] Revisionnage
    }
core-trackers-marked = Épisode { $episode } de { $show } marqué sur { core-trackers-sites }.
core-trackers-progress-set = Progression sur { $show } mise à { $progress } sur { core-trackers-sites }.
core-trackers-scored = { $show } noté { $score } sur { core-trackers-sites }.
core-trackers-score-cleared = Note de { $show } effacée sur { core-trackers-sites }.
core-trackers-status-set = { $show } passé en { core-trackers-status } sur { core-trackers-sites }.
core-trackers-notes-saved = Vos notes sur { $show } enregistrées sur { core-trackers-sites }.
core-trackers-notes-cleared = Vos notes sur { $show } effacées sur { core-trackers-sites }.
core-trackers-update-failed = Impossible de mettre à jour { $site }. Réessayez plus tard.
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
        [clear] Note effacée pour { core-trackers-written-who } sur { core-trackers-sites }.
        [status] { core-trackers-written-who } : { core-trackers-status } sur { core-trackers-sites }.
       *[score] Note { $score } pour { core-trackers-written-who } sur { core-trackers-sites }.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed ->
            [one] { $failed } série n'a pas pu être mise à jour : { $reason }.
           *[other] { $failed } séries n'ont pas pu être mises à jour : { $reason }.
        }
       *[no] { $failed ->
            [one] { $failed } série n'a pas pu être mise à jour.
           *[other] { $failed } séries n'ont pas pu être mises à jour.
        }
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] Échec pour { $failed } : { $reason }.
       *[no] Échec pour { $failed }.
    }
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
        [one] {" "}{ $noMatch } série n'est associée à aucun suivi.
       *[other] {" "}{ $noMatch } séries ne sont associées à aucun suivi.
    }{ $hidden ->
        [0] {""}
        [one] {" "}{ $hidden } série est masquée, sa synchro est donc coupée.
       *[other] {" "}{ $hidden } séries sont masquées, leur synchro est donc coupée.
    }{ $notConnected ->
        [0] {""}
        [one] {" "}{ $notConnected } série n'est que sur un suivi non connecté.
       *[other] {" "}{ $notConnected } séries ne sont que sur un suivi non connecté.
    }{ $kept ->
        [0] {""}
        [one] {" "}{ $kept } série est gardée sur cet ordinateur, sans suivi où l'envoyer.
       *[other] {" "}{ $kept } séries sont gardées sur cet ordinateur, sans suivi où les envoyer.
    }

## core/src/metadata/automatch.rs

core-metadata-matched-to = { $show } associée à { $title }.
core-metadata-auto-matched =
    { $count ->
        [one] { $count } série associée.
       *[other] { $count } séries associées.
    }
core-metadata-auto-unmatched =
    { $count ->
        [one] { $count } série attend encore une association.
       *[other] { $count } séries attendent encore une association.
    }
core-metadata-auto-matched-some =
    { $matched ->
        [one] { $matched } série associée
       *[other] { $matched } séries associées
    } ; { $unmatched } { $unmatched ->
        [one] attend
       *[other] attendent
    } encore une association.
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
        [yes] { $hitFolder } associée à { $hitTitle }.
       *[no] { $matched ->
            [one] { $matched } série associée.
           *[other] { $matched } séries associées.
        }
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] Aucune association trouvée pour { $missName }.
       *[no] { $unmatched ->
            [one] Aucune association trouvée pour { $unmatched } série.
           *[other] Aucune association trouvée pour { $unmatched } séries.
        }
    }
core-metadata-find-failed =
    { $failed ->
        [one] { $failed } série n'a pas pu être cherchée ; réessayez plus tard.
       *[other] { $failed } séries n'ont pas pu être cherchées ; réessayez plus tard.
    }
core-metadata-find-had =
    { $had ->
        [one] { $had } série était déjà associée.
       *[other] { $had } séries étaient déjà associées.
    }
core-metadata-find-busy =
    { $busy ->
        [one] { $busy } série est déjà en cours d'association.
       *[other] { $busy } séries sont déjà en cours d'association.
    }

## core/src/metadata/apply.rs

core-metadata-link-unreadable = Impossible de lire ce lien. Collez l'URL d'une page AniList, MyAnimeList, TVmaze ou IMDb.
core-metadata-no-mal-entry = AniList n'a aucune entrée pour cet identifiant MyAnimeList.
core-metadata-no-imdb-show = TVmaze n'a aucune série pour ce titre IMDb.
core-metadata-search-on = la recherche sur { $provider }
core-metadata-query-too-short = Saisissez au moins deux caractères pour chercher.
core-metadata-not-a-link = ce n'est pas un lien
core-metadata-refresh-unsupported = l'actualisation d'une association MAL seule ou TMDB
core-metadata-matched = { $show } associée.
core-metadata-updated = Infos de { $show } mises à jour.
core-metadata-update-failed = Impossible de mettre à jour les infos de { $show }. Réessayez plus tard.
core-metadata-progress-refresh = actualisation
core-metadata-progress-backfill = complément
core-metadata-updated-many =
    { $count ->
        [one] Infos de { $count } série mises à jour.
       *[other] Infos de { $count } séries mises à jour.
    }
core-metadata-update-failed-many =
    { $count ->
        [one] Impossible de mettre à jour les infos de { $count } série. Réessayez plus tard.
       *[other] Impossible de mettre à jour les infos de { $count } séries. Réessayez plus tard.
    }
core-metadata-updated-some =
    { $count ->
        [one] Infos de { $count } série mises à jour ; échec pour { $failed }.
       *[other] Infos de { $count } séries mises à jour ; échec pour { $failed }.
    }

## core/src/metadata/fetch.rs

core-metadata-no-such-show = TVmaze n'a pas de série { $id }
core-metadata-no-such-media = AniList n'a pas de média { $id }

## core/src/metadata/imdb.rs

core-metadata-progress-imdb-links = liens IMDb de Wikidata
core-metadata-progress-imdb-ratings = notes IMDb

## core/src/franchise/layout.rs

core-franchise-relation-character = Mêmes personnages

## core/src/images/mod.rs

core-images-progress-fetching = récupération des images

## core/src/transfer/export.rs

core-transfer-exported =
    { $count ->
        [one] { $count } série enregistrée dans { $folder }.
       *[other] { $count } séries enregistrées dans { $folder }.
    }

## core/src/transfer/format.rs

core-transfer-not-json = ce n'est pas du JSON : { $error }
core-transfer-wrong-format = ce n'est pas un document { $format }

## core/src/transfer/import.rs

core-transfer-imported = Bibliothèque chargée depuis le fichier.

## core/src/library/delete.rs

core-library-no-such-file = aucun fichier { $id } dans la bibliothèque
core-library-file-elsewhere-not-removed = le fichier { $id } appartient à une autre série, donc rien n'a été retiré
core-library-file-outside = { $path } n'est pas un fichier dans un dossier de la bibliothèque, donc rien n'a été retiré
core-library-episodes-removed =
    { $count ->
        [one] { $count } épisode de { $title } { $place ->
            [deleted] supprimé
           *[trash] mis à la corbeille
        }.
       *[other] { $count } épisodes de { $title } { $place ->
            [deleted] supprimés
           *[trash] mis à la corbeille
        }.
    }
core-library-finished-removed =
    { $count ->
        [one] { $count } série terminée { $place ->
            [deleted] supprimée
           *[trash] mise à la corbeille
        }.
       *[other] { $count } séries terminées { $place ->
            [deleted] supprimées
           *[trash] mises à la corbeille
        }.
    }
core-library-watched-removed =
    { $count ->
        [one] { $count } épisode vu de { $series } séries { $place ->
            [deleted] supprimé
           *[trash] mis à la corbeille
        }.
       *[other] { $count } épisodes vus de { $series } séries { $place ->
            [deleted] supprimés
           *[trash] mis à la corbeille
        }.
    }

## core/src/library/detail.rs

core-library-fix-one-file =
    Le seul fichier ici se lit comme { $read ->
        [episode] l'épisode { $number }
       *[label] { $label }
    }. On dirait { $what ->
        [film] le film
       *[episode] son seul épisode
    }.
core-library-fix-specials =
    { $count ->
        [one] { $count } spécial ressemble
       *[other] { $count } spéciaux ressemblent
    } { $episodes ->
        [one] à l'épisode manquant
       *[other] aux épisodes manquants
    } { $numbers }.
core-library-fix-numbered-on = Les épisodes { $from } semblent être les { $to } de cette fiche, numérotés à la suite d'une saison précédente.
core-library-runs-span = { $first } à { $last }
core-library-runs-list = { $init } et { $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = un numéro d'épisode va de 0 à 10000
core-library-fix-season-range = une saison va jusqu'à 200
core-library-fix-extra-range = l'index d'un bonus va jusqu'à 10000
core-library-file-elsewhere-not-fixed = le fichier { $id } appartient à une autre série, donc rien n'a été corrigé
core-library-fix-undone =
    { $count ->
        [one] { $count } fichier de { $title } est de nouveau lu d'après son nom.
       *[other] { $count } fichiers de { $title } sont de nouveau lus d'après leur nom.
    }
core-library-fixed =
    { $count ->
        [one] Lecture de { $count } fichier de { $title } corrigée.
       *[other] Lecture de { $count } fichiers de { $title } corrigée.
    }

## core/src/library/reads.rs

core-library-hid-one = { $title } masquée.
core-library-back-one = { $title } est de retour.
core-library-hid-many = { $count } séries masquées.
core-library-back-many = { $count } séries sont de retour.
core-library-episode-count-set = { $title } compte maintenant { $count } épisodes.
core-library-episode-count-cleared = Nombre d'épisodes de { $title } effacé.

## core/src/library/schedule.rs

core-library-schedule-cleared = Notes de planning de { $title } effacées.
core-library-schedule-saved = { $count } notes de planning enregistrées pour { $title }.

## core/src/library/scan.rs

core-library-source-not-absolute = le chemin d'une source doit être absolu
core-library-source-exists = c'est déjà une source
core-library-source-nested = se trouve dans une source existante
core-library-source-added = Dossier { $folder } ajouté à votre bibliothèque.
core-library-source-removed = Dossier { $folder } retiré de votre bibliothèque.
core-library-source-removed-with-shows =
    { $count ->
        [one] Dossier { $folder } et sa { $count } série retirés de votre bibliothèque.
       *[other] Dossier { $folder } et ses { $count } séries retirés de votre bibliothèque.
    }
core-library-series-forgotten = { $name } retirée de votre bibliothèque.
core-library-source-back = Le dossier { $folder } est de retour.
core-library-source-unavailable = Le dossier { $folder } n'est pas disponible. Vérifiez que son disque est branché.
core-library-source-unreadable = Impossible de lire le dossier { $folder }. Vérifiez que son disque est branché.
core-library-source-looks-empty = Le dossier { $folder } semble vide, donc vos séries sont restées telles quelles.
core-library-source-reconcile-failed = Impossible de mettre à jour les séries de { $folder }.
core-library-scan-done =
    Dossiers vérifiés : { $added ->
        [0] {""}
       *[other] { $added ->
            [one] { $added } nouvelle série
           *[other] { $added } nouvelles séries
        }
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }{ $changed ->
            [one] { $changed } série modifiée
           *[other] { $changed } séries modifiées
        }
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }{ $removed ->
            [one] { $removed } manquante
           *[other] { $removed } manquantes
        }
    }.

## core/src/library/report.rs

core-library-agent-no-answer = pas de réponse en { $span }
core-library-agent-stopped-code = il s'est arrêté avec le code { $code }
core-library-agent-stopped = il s'est arrêté
core-library-agent-answered-nothing = il n'a rien répondu
core-library-span-minutes =
    { $count ->
        [one] { $count } minute
       *[other] { $count } minutes
    }
core-library-span-seconds =
    { $count ->
        [one] { $count } seconde
       *[other] { $count } secondes
    }
core-library-report-no-fix = aucun de ces fichiers n'a de correction à signaler
core-library-agent-no-triage = { $agent } n'a écrit aucun tri : { $why }
core-transfer-progress-sources = sources
core-transfer-progress-series = séries
core-transfer-progress-history = historique
core-transfer-progress-accounts = comptes
core-transfer-progress-preferences = préférences
core-transfer-progress-done = terminé

## core/src/downloads/anirss.rs

core-downloads-anirss-no-answer = anirss n'a pas répondu : { $message }
core-downloads-anirss-answered = anirss a répondu { $status } : { $text }
core-downloads-no-free-port = aucun port libre : { $error }
core-downloads-anirss-exited = anirss s'est arrêté ({ $status })
core-downloads-anirss-exited-said = anirss s'est arrêté ({ $status }) : { $said }
core-downloads-anirss-slow-start = anirss n'a pas démarré en { $seconds } secondes
core-downloads-anirss-slow-start-said = anirss n'a pas démarré en { $seconds } secondes : { $said }
core-downloads-anirss-install-mac = Installez anirss { $version } ou plus récent avec : { $command }
core-downloads-anirss-install-linux = Installez anirss { $version } ou plus récent : le paquet anirss de l'AUR, ou { $command }
core-downloads-anirss-too-old = anirss { $version } est trop ancien pour AniBeam, qui demande la { $minimum } ou plus récente. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version } ne sait pas garder la connexion de chaque client torrent à part, ce dont AniBeam a besoin pour changer de client ; anirss { $minimum } et plus récent le savent. { $update }.
core-downloads-anirss-update-mac = Mettez-le à jour avec : { $command }
core-downloads-anirss-update-linux = Mettez à jour le paquet anirss, ou lancez : { $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = le fichier torrent n'est pas arrivé : { $message }
core-downloads-torrent-file-answered = le fichier torrent a répondu { $status }

## core/src/downloads/feed.rs

core-downloads-feed-answered = le flux a répondu { $status }
core-downloads-not-a-feed = ce n'est pas un flux : { $error }

## core/src/downloads/jackett.rs

core-downloads-jackett-missing = Jackett n'est pas installé : { $program } est introuvable. Installez-le, ou changez la commande dans les réglages.
core-downloads-endpoint-start-failed = { $name } n'a pas pu démarrer : { $why }
core-downloads-endpoint-start-timeout = { $name } a démarré mais n'a pas répondu en { $seconds } secondes.
core-downloads-command-exited = la commande s'est terminée avec { $status }

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirss s'est arrêté
core-downloads-anirss-missing = { $install }. Ou indiquez un serveur en marche dans les réglages.
core-downloads-anirss-command-missing = { $command } est introuvable. { $install }. Ou indiquez un serveur en marche dans les réglages.
core-downloads-anirss-start-failed = { $command } n'a pas pu démarrer : { $error }
core-downloads-anirss-too-old-said = { $tooOld } Il a dit : { $message }
core-downloads-no-torrent-client = Aucun qBittorrent ne répond à { $address }, et AniBeam n'en a pas à lancer lui-même. Installez qBittorrent et activez son interface Web, ou installez qbittorrent-nox, qu'AniBeam lancera pour vous
core-downloads-own-client-gone = Le qBittorrent d'AniBeam a disparu : qbittorrent-nox n'est pas installé. Réinstallez AniBeam, ou indiquez votre propre qBittorrent dans les réglages
core-downloads-no-port = aucun port dans { $address }
core-downloads-own-client-settings = impossible d'écrire les réglages de qBittorrent : { $error }
core-downloads-own-client-start-failed = { $command } n'a pas pu démarrer : { $error }
core-downloads-own-client-slow-start = Le qBittorrent d'AniBeam n'a pas démarré
core-downloads-own-client-slow-start-said = Le qBittorrent d'AniBeam n'a pas démarré : { $said }
core-downloads-not-ready = Les téléchargements ne sont pas prêts. Vérifiez Téléchargements dans les réglages.
core-downloads-ready = Les téléchargements sont prêts.
core-downloads-helper-sources =
    { $count ->
        [one] Les téléchargements cherchent maintenant avec AniBeam lui-même, qui ne lit pas la { $count } source de la configuration de l'assistant de recherche. Ajoutez-la dans Réglages, Téléchargements, ou choisissez-y l'assistant de recherche.
       *[other] Les téléchargements cherchent maintenant avec AniBeam lui-même, qui ne lit pas les { $count } sources de la configuration de l'assistant de recherche. Ajoutez-les dans Réglages, Téléchargements, ou choisissez-y l'assistant de recherche.
    }
core-downloads-client-connected = L'appli de téléchargement est connectée.
core-downloads-client-not-connected = L'appli de téléchargement n'est pas connectée. Vérifiez son adresse dans les réglages.
core-downloads-not-on-demand = { $name } n'est pas une source à la demande
core-downloads-source-running = La source de téléchargement est en marche.
core-downloads-source-not-running = { $name } n'est pas en marche. Activez-la dans les sources et confirmez pour la démarrer.
core-downloads-empty-query = saisissez quelque chose à chercher
core-downloads-first-endpoint = la première source
core-downloads-search-starting = Lancement de la recherche
core-downloads-search-starting-endpoint = Lancement de { $endpoint }
core-downloads-search-searching-endpoint = Recherche sur { $endpoint }
core-downloads-search-reading = Lecture des résultats
core-downloads-subscribed = Abonné à { $name }.
core-downloads-client-no-rule = qBittorrent n'a aucune règle nommée { $name }
core-downloads-trying-search = Essai de « { $search } »
core-downloads-nothing-selected = rien de sélectionné à télécharger
core-downloads-added =
    { $count ->
        [one] { $count } téléchargement ajouté à vos téléchargements.
       *[other] { $count } téléchargements ajoutés à vos téléchargements.
    }
core-downloads-movie-link = le lien d'un film doit être un magnet ou une URL http(s)
core-downloads-added-movie = Film ajouté à vos téléchargements.
core-downloads-unsubscribed = Désabonné de { $name }.
core-downloads-library-folder = { $folder } est le dossier de bibliothèque de { $name }, avec des épisodes que cet abonnement n'a pas téléchargés : retirez l'abonnement sans ses fichiers, ou retirez { $name } depuis sa page
core-downloads-no-torrents-named = aucun torrent indiqué
core-downloads-paused =
    { $count ->
        [one] { $count } téléchargement mis en pause.
       *[other] { $count } téléchargements mis en pause.
    }
core-downloads-resumed =
    { $count ->
        [one] { $count } téléchargement repris.
       *[other] { $count } téléchargements repris.
    }
core-downloads-deleted-with-files =
    { $count ->
        [one] { $count } téléchargement supprimé avec ses fichiers.
       *[other] { $count } téléchargements supprimés avec leurs fichiers.
    }
core-downloads-removed-from-list =
    { $count ->
        [one] { $count } téléchargement retiré de votre liste.
       *[other] { $count } téléchargements retirés de votre liste.
    }
core-downloads-nothing-to-remove = rien à retirer
core-downloads-removed-show = { $title } retirée de vos téléchargements.
core-downloads-removed-the-show = Série retirée de vos téléchargements.
core-downloads-deleted-still-listed = Les fichiers supprimés restent listés dans l'appli de téléchargement.
core-downloads-feed-url = un flux est une URL http(s)
core-downloads-rule-needs-name = une règle a besoin d'un nom
core-downloads-no-rule = aucune règle nommée { $name }
core-downloads-feed-no-search = le flux n'a aucune recherche à raccourcir
core-downloads-no-repair = { $name } n'a pas besoin de réparation.
core-downloads-repaired = Abonnement à { $name } réparé.
core-downloads-repair-failed = aucune recherche plus courte ne trouve quoi que ce soit ; même « { $search } » ne renvoie rien
core-downloads-no-saved-client = aucun client torrent enregistré { $id }
core-downloads-client-switched = Les téléchargements utilisent maintenant une autre appli de téléchargement.
core-downloads-host-not-running = ne tourne pas sur l'hôte
core-downloads-host-late = n'a pas répondu à temps

## core/src/downloads/native.rs

core-downloads-native-empty-query = recherche vide
core-downloads-native-no-results = aucun résultat pour { $query }
core-downloads-native-feed-taken = un flux nommé { $name } lit déjà une autre adresse
core-downloads-native-no-items = aucun élément fourni
core-downloads-native-bad-link = les liens doivent être des URL magnet: ou http(s), reçu { $link }
core-downloads-native-no-links = aucun lien fourni
core-downloads-native-no-names = aucun nom d'abonnement fourni
core-downloads-native-files-elsewhere = le client torrent garde ses fichiers sur une autre machine, ils ne peuvent donc partir qu'avec ses torrents
core-downloads-native-unknown-feeds = flux inconnu(s) : { $names }
core-downloads-native-outside-folders = refus de supprimer des fichiers hors des dossiers de téléchargement : { $paths }
core-downloads-refuse-home-folder = { $path } est le dossier personnel ou le contient, donc rien n'a été retiré
core-downloads-refuse-library-folder = { $path } est un dossier de bibliothèque ou en contient un, donc rien n'a été retiré

## core/src/downloads/paths.rs

core-downloads-no-default-folder = qBittorrent n'indique aucun dossier d'enregistrement par défaut ; définissez-en un dans qBittorrent
core-downloads-name-empty = le nom ne peut pas être vide
core-downloads-name-slash = le nom ne doit contenir ni / ni \
core-downloads-name-not-folder = { $name } n'est pas un nom de dossier

## core/src/downloads/prune.rs

core-downloads-pruned = Désabonné de { $title }, sa diffusion est terminée.
core-downloads-prune-failed = Impossible de se désabonner de { $title }. Vérifiez Téléchargements dans les réglages.

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path } n'est pas un simple chemin absolu, donc rien n'a été retiré
core-downloads-refuse-root = { $path } est la racine du système de fichiers, donc rien n'a été retiré
core-downloads-refuse-outside-library = { $path } n'est pas dans un dossier de bibliothèque, donc rien n'a été retiré
core-downloads-refuse-save-folder = { $path } est le dossier où le torrent a été enregistré, pas ce qu'il a téléchargé, donc rien n'a été retiré

## core/src/downloads/watch.rs

core-downloads-episode-missed = L'épisode { $episode } de { $title } n'est toujours pas arrivé trois jours après sa diffusion. Votre flux continue de le chercher.

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = la commande anirss ne peut pas être vide
core-downloads-settings-anirss-address = l'adresse d'anirss doit commencer par http:// ou https://
core-downloads-settings-client-address = l'adresse de { $client } doit commencer par http:// ou https://
core-downloads-settings-bad-id = { $id } n'est pas un identifiant de client : utilisez des lettres, des chiffres, - et _
core-downloads-settings-same-id = deux clients ont l'identifiant { $id }
core-downloads-settings-saved-address = l'adresse de { $name } doit commencer par http:// ou https://
core-downloads-settings-same-name = deux clients s'appellent { $name } ; donnez à chacun son propre nom

## core/src/downloads/qbt.rs

core-downloads-qbt-no-answer = qBittorrent n'a pas répondu à { $address } : { $message }
core-downloads-qbt-password-rejected = qBittorrent a refusé le mot de passe ; corrigez-le dans les réglages
core-downloads-qbt-banned = qBittorrent a banni cette adresse après des connexions échouées ; patientez, ou redémarrez qBittorrent
core-downloads-qbt-login-rejected = qBittorrent a refusé le nom d'utilisateur ou le mot de passe
core-downloads-qbt-login-failed = la connexion à qBittorrent sur { $address } a répondu { $status } ; nouvel essai au prochain appel
core-downloads-qbt-not-web-ui = { $address } a répondu { $status }, mais pas comme l'interface Web de qBittorrent. Vérifiez l'adresse dans les réglages, et que l'interface Web est activée dans qBittorrent
core-downloads-qbt-not-added = qBittorrent n'a pas ajouté les torrents : { $said }
core-downloads-qbt-some-not-added = qBittorrent n'a pas pu ajouter { $count } des torrents : { $said }
core-downloads-qbt-answered = qBittorrent a répondu { $status } : { $line }

## core/src/downloads/replace.rs

core-downloads-replace-bad-item = l'élément { $item } ne fait pas partie des { $count } releases
core-downloads-not-replaced = L'épisode { $episode } n'a pas été remplacé.
core-downloads-replaced = Épisode { $episode } remplacé par le nouveau téléchargement.

## core/src/downloads/rolling.rs

core-downloads-no-roll = aucun téléchargement glissant { $id }
core-downloads-rolled = { $title } : maintenant sur les épisodes { $from } à { $to } sur { $total }.
core-downloads-roll-ahead = gardez au moins un épisode d'avance
core-downloads-roll-nothing = rien à récupérer

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = une source de recherche JSON vient d'un dépôt de sources dont la définition associe ses champs
core-releases-endpoint-no-name = `name` manquant
core-releases-endpoint-bad-name = { $name } : utilisez seulement des lettres, des chiffres, '.', '_', '-'
core-releases-endpoint-json = { $name } : { $why }
core-releases-endpoint-unknown-kind = { $name } : type inconnu { $kind } (valides : { $valid })
core-releases-endpoint-no-url = { $name } : `url` manquant
core-releases-endpoint-not-http = { $name } : l'adresse doit commencer par http:// ou https://
core-releases-endpoint-needs-query = { $name } : l'url { $kind } doit contenir l'espace réservé {"{"}query{"}"}
core-releases-endpoint-no-query = { $name } : une url { $kind } ajoute elle-même la recherche, elle ne prend donc pas de {"{"}query{"}"}

## core/src/releases/fetch.rs

core-releases-indexer-error = l'indexeur a répondu l'erreur { $code }
core-releases-bad-feed = flux incorrect : { $error }
core-releases-feed-answered = le flux a répondu { $status }

## core/src/releases/json_feed.rs

core-releases-not-feed-json = ce n'est pas le JSON du flux : { $error }

## core/src/releases/json.rs

core-releases-not-json = ce n'est pas du JSON : { $error }
core-releases-search-error-said = la recherche a répondu par une erreur : { $message }
core-releases-search-error = la recherche a répondu par une erreur
core-releases-no-results-field = la réponse n'a pas de résultats à { $path }
core-releases-results-not-list = les résultats de la réponse ne sont pas une liste
core-releases-search-answered = la recherche a répondu { $status }

## core/src/releases/native.rs

core-releases-no-endpoints = aucun site pour l'instant : ajoutez un dépôt, ou un site à la main, sous Sites dans Plus
core-releases-no-source-named = aucune source nommée { $name }
core-releases-empty-query = saisissez quelque chose à chercher
core-releases-cannot-follow = { $name } ne peut pas être suivi : il répond aux recherches, mais n'a pas de flux pour vérifier les nouveaux épisodes
core-releases-endpoint-gone = { $name } n'est plus disponible : retiré, désactivé ou sans sa clé

## core/src/releases/quirks.rs

core-releases-quirks-variant = one_variant { $word } doit être un mot court
core-releases-quirks-pattern-count = show_feed prend de 1 à { $most } modèles de page
core-releases-quirks-pattern-long = un modèle de page show_feed est trop long
core-releases-quirks-pattern-bad = modèle de page show_feed : { $error }
core-releases-quirks-pattern-slug = un modèle de page show_feed a besoin d'un groupe nommé slug
core-releases-quirks-url-https = l'url show_feed doit être en https
core-releases-quirks-url-host = l'url show_feed doit être sur l'hôte même de la source
core-releases-quirks-url-slug = l'url show_feed prend {"{"}slug{"}"} une fois et aucun autre espace réservé

## core/src/releases/refinement.rs

core-releases-refine-no-tokens = l'opération 'tokens' demande une liste `tokens` non vide

## core/src/releases/grid.rs

core-releases-group-batch = Lot de { $group }
core-releases-episodes-phrase =
    { $count ->
        [one] Épisode { $list }
       *[other] Épisodes { $list }
    }
core-releases-episode-range = { $from } à { $to }

## core/src/subscriptions.rs

core-subscriptions-platform = anirss sur ce système
core-subscriptions-unreadable = sa réponse est illisible ({ $message })
core-subscriptions-stopped = il s'est arrêté sans dire pourquoi
core-subscriptions-stopped-said = il s'est arrêté : { $said }

## Names that stay the same in every language

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList
