# fr translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = Accueil
nav-library = Bibliothèque
nav-airing = En diffusion
nav-feed = Fil
nav-watching = En cours
nav-stats = Stats
nav-metadata = Infos de la série
nav-downloads = Téléchargements
nav-settings = Réglages
nav-series = Série
nav-player = Lecteur
nav-show = Série
nav-recommended = Recommandées

## StatusStrip

status-strip-errors = { $count ->
    [one] { $count } erreur
   *[other] { $count } erreurs
}

## ScanSummary

scan-summary-nothing = Rien à afficher pour l'instant
scan-summary-found = { $count ->
    [one] { $count } série trouvée
   *[other] { $count } séries trouvées
}, { $recognised } reconnues
scan-summary-found-need = { $count ->
    [one] { $count } série trouvée
   *[other] { $count } séries trouvées
}, { $recognised } reconnues, { $need } à voir avec vous
scan-summary-episodes = { $count ->
    [one] { $count } épisode
   *[other] { $count } épisodes
} sur le disque.
scan-summary-episodes-watched = { $count ->
    [one] { $count } épisode
   *[other] { $count } épisodes
} sur le disque, { $watched } déjà vus sur { $tracker }.
scan-summary-title = Votre dossier est chargé
scan-summary-count-found = trouvées
scan-summary-count-recognised = reconnues
scan-summary-count-need = à voir avec vous
scan-summary-later = Plus tard
scan-summary-match = Associer les { $count }
scan-summary-open-library = Ouvrir la bibliothèque

## UpdateNotice

update-notice-check-failed = AniBeam n'a pas pu vérifier les mises à jour. Vérifiez votre connexion Internet, puis réessayez.
update-notice-up-to-date = AniBeam est à jour
update-notice-downloading = Téléchargement d'AniBeam { $version }
update-notice-ready = AniBeam { $version } est prêt
update-notice-failed = La mise à jour a échoué
update-notice-on-aur = AniBeam { $version } est sur l'AUR
update-notice-out = AniBeam { $version } est sorti
update-notice-out-titled = AniBeam { $version } « { $title } » est sorti
update-notice-progress = { $percent } %  ·  { $got } sur { $total } Mo
update-notice-restart-hint = Redémarrez maintenant, ou il s'ouvrira la prochaine fois
update-notice-from-source = Cette copie est compilée depuis les sources : faites un pull et recompilez
update-notice-no-download = Aucun téléchargement ici pour cet ordinateur
update-notice-install-hint = Mettre à jour l'installe et redémarre AniBeam
update-notice-retry = Réessayer
update-notice-update = Mettre à jour
update-notice-restart = Redémarrer
update-notice-copied = Copié
update-notice-copy = Copier
update-notice-release-page = Page de la version
update-notice-later = Plus tard

## WhatsNew

whats-new-title = Nouveautés
whats-new-got-it = Compris

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = d MMM HH:mm
activity-drawer-stage-library = Bibliothèque
activity-drawer-stage-metadata = Métadonnées
activity-drawer-stage-trackers = Suivi
activity-drawer-stage-franchise = Franchise
activity-drawer-stage-playback = Lecture
activity-drawer-stage-store = Stockage
activity-drawer-stage-system = Système
activity-drawer-stage-downloads = Téléchargements
activity-drawer-level-info = Info
activity-drawer-level-warn = Alerte
activity-drawer-level-error = Erreur
activity-drawer-copied-line = Ligne copiée
activity-drawer-title = Activité
activity-drawer-everything = Tout
activity-drawer-problems = Problèmes
activity-drawer-copied = Copié
activity-drawer-copy = Copier
activity-drawer-clear-every = Tout effacer
activity-drawer-copy-tip = Cliquez pour copier la ligne
activity-drawer-fold = Replier
activity-drawer-empty-technical = Rien de journalisé pour l'instant.
activity-drawer-empty-problems = Aucun problème. Tout a fonctionné.
activity-drawer-empty = Rien à signaler pour l'instant. Ce qu'AniBeam fait pour vous s'affiche ici.

## ActivityColumn

activity-column-episode-run = Épisodes { $first } à { $last }
activity-column-episode-list = Épisodes { $list }
activity-column-downloads = { $count ->
    [one] { $count } téléchargement
   *[other] { $count } téléchargements
}
activity-column-batch = Lot
activity-column-download = Téléchargement
activity-column-name-batch = { $title }, un lot
activity-column-name-episode = { $title }, épisode { $number }
activity-column-title = Activité
activity-column-down = réception
activity-column-up = envoi
activity-column-not-connected = { $client } n'est pas connecté, donc rien ne bouge ici.
activity-column-connect-again = Reconnecter
activity-column-downloading = En téléchargement
activity-column-fewer = Moins
activity-column-more = Et { $count } de plus
activity-column-nothing-downloading = Aucun téléchargement en cours.
activity-column-finished-today = Terminés aujourd'hui
activity-column-today = Aujourd'hui
activity-column-today-empty = Recherches, téléchargements et nouveaux abonnements s'affichent ici au fil de l'eau.
activity-column-stopping = Arrêt
activity-column-starting = Démarrage
activity-column-start-tip = Lancer le téléchargement
activity-column-stop-tip = Arrêter le téléchargement ; ce qui est déjà là reste
activity-column-cancelling = Annulation
activity-column-cancel-all-tip = Annuler ces téléchargements
activity-column-cancel-tip = Annuler ce téléchargement
activity-column-size-of = { $got } sur { $total }
activity-column-files-missing = Fichiers manquants
activity-column-time-format = HH:mm
activity-column-left = { $time } restantes
activity-column-waiting-peers = En attente de pairs
activity-column-episodes-done = { $count ->
    [one] { $done } épisode sur { $count } terminé
   *[other] { $done } épisodes sur { $count } terminés
}
activity-column-cube-skipped = Épisode { $number } : ignoré
activity-column-cube-done = Épisode { $number } : terminé
activity-column-cube-progress = Épisode { $number } : { $percent } %

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = Prêt
frame-strip-progress = { $doing }, { $done } sur { $total }
frame-strip-idle = Rien en cours
frame-strip-time-format = HH:mm
frame-finish-sign-in = Terminez d'abord la connexion à { $tracker }
frame-did-not-work = Ça n'a pas marché.
frame-could-not-open = AniBeam n'a pas pu l'ouvrir. Copiez-le et ouvrez-le vous-même.
frame-copy = Copier
frame-copied = Copié
frame-status-refused = { $tracker } ne l'a pas accepté
frame-status-refused-why = { $tracker } ne l'a pas accepté : { $why }
frame-lists-refused = { $tracker } n'a pas accepté { $count ->
    [one] 1 série
   *[other] { $count } séries
} ; voir le journal d'activité
frame-lists-moved = { $count ->
    [one] 1 série transférée
   *[other] { $count } séries transférées
} vers { $tracker }
frame-notes-refused = { $tracker } n'a pas accepté vos notes
frame-notes-refused-why = { $tracker } n'a pas accepté vos notes : { $why }
frame-notes-saved = Notes enregistrées
frame-connected = Connecté à { $tracker } en tant que { $user }
frame-sign-in-stopped = La connexion à { $tracker } s'est arrêtée avant la fin.
frame-toast-hide-details = Masquer les détails
frame-toast-details = Détails
frame-toast-details-tip = Voir exactement ce qui n'a pas marché, et le copier
frame-forward = Avancer
frame-copy-magnet = Copier le magnet
frame-copy-link = Copier le lien torrent
frame-magnet-copied = Magnet copié
frame-link-copied = Lien copié
frame-open-release-page = Ouvrir la page de la release
frame-open-folder = Ouvrir le dossier
frame-copy-path = Copier le chemin
frame-path-copied = Chemin copié
frame-mark-unwatched = Non vu à partir d'ici
frame-mark-watched = Vu jusqu'ici
frame-copy-title = Copier le titre
frame-open-on-anilist = Ouvrir sur AniList
frame-list-home-online = Modifiez-le sur chaque liste connectée
frame-list-home-unmatched = Gardé sur cet appareil : aucun suivi ne propose cette série
frame-list-home-not-carried = Gardé sur cet appareil : votre suivi n'a pas cette série
frame-list-home-offline = Gardé sur cet appareil jusqu'à la connexion d'AniList ou MyAnimeList
frame-moving-to = Déplacement vers { $list }
frame-moved-to = Déplacé vers { $list }
frame-plan-added = { $title } passe dans votre liste À voir
frame-plan-listed = { $title } est déjà dans votre liste
frame-delete-watched = { $kind ->
    [film] Supprimer les films vus
   *[episode] Supprimer les épisodes vus
}
frame-delete-all = { $count ->
    [one] { $kind ->
        [film] Supprimer le film
       *[episode] Supprimer l'épisode
    }
   *[other] { $kind ->
        [film] Supprimer les { $count } films
       *[episode] Supprimer les { $count } épisodes
    }
}
frame-series-gone = Cette série n'existe plus
frame-delete-none-watched = Aucun épisode vu de { $title } sur le disque
frame-delete-none = Rien de { $title } sur le disque
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count ->
            [one] { $count } film vu
           *[other] { $count } films vus
        }
       *[episode] { $count ->
            [one] { $count } épisode vu
           *[other] { $count } épisodes vus
        }
    }
   *[no] { $kind ->
        [film] { $count ->
            [one] { $count } film
           *[other] { $count } films
        }
       *[episode] { $count ->
            [one] { $count } épisode
           *[other] { $count } épisodes
        }
    }
}
frame-delete-extras = { $count ->
    [one] { $count } bonus
   *[other] { $count } bonus
}
frame-delete-and = { $first } et { $second }
frame-delete-heading = Supprimer { $what } de { $title } ?
frame-delete-base = Les fichiers et leurs sous-titres vont à la corbeille, vous pouvez donc les y récupérer. Leurs torrents quittent le client torrent, ils peuvent donc être retéléchargés.
frame-delete-confirm = Mettre à la corbeille
frame-delete-done = { $what } de { $title } mis à la corbeille
frame-delete-watched-done = { $count ->
    [one] { $count } épisode
   *[other] { $count } épisodes
}, { $size }, mis à la corbeille
frame-delete-nothing = Rien à supprimer
frame-fix-not-drafted = Corrigé. Le ticket n'a pas pu être rédigé
frame-fix-agent = Corrigé. { $agent } s'en occupe, et le ticket s'ouvre quand c'est fait
frame-fix-opening = Corrigé. Ouverture du ticket
frame-fix-not-saved = La correction n'a pas été enregistrée : { $error }
frame-fix-undone = { $count ->
    [one] { $count } fichier de nouveau lu
   *[other] { $count } fichiers de nouveau lus
} d'après le nom
frame-fix-done = Lecture { $count ->
    [one] d'{ $count } fichier
   *[other] de { $count } fichiers
} corrigée
frame-report-failed = Le ticket n'a pas pu être rédigé : { $error }
frame-report-untriaged = Le ticket est ouvert dans votre navigateur, sans tri. { $error }
frame-report-open = Le ticket est ouvert dans votre navigateur, prêt à envoyer
frame-downloads-not-read = Non lu.

## HomePage

home-title = Accueil
home-scan-started = Analyse lancée
home-ago-minutes = il y a { $count } min
home-ago-hours = il y a { $count } h
home-ago-days = il y a { $count } j
home-today = Aujourd'hui
home-tomorrow = Demain
home-yesterday = Hier
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = d MMM
home-day-date-format = ddd d MMM
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = Pas encore de date
home-unit-days = { $count ->
    [one] jour
   *[other] jours
}
home-unit-hours = h
home-unit-minutes = min
home-week-late = { $count ->
    [one] { $count } épisode en retard
   *[other] { $count } épisodes en retard
}
home-week-waiting = { $count ->
    [one] { $count } épisode sorti à regarder
   *[other] { $count } épisodes sortis à regarder
}
home-week-coming = { $count ->
    [one] { $count } épisode dans les sept prochains jours
   *[other] { $count } épisodes dans les sept prochains jours
}
home-week-next = Ensuite : { $title } dans { $time }
home-countdown-days = { $days } j { $hours } h
home-countdown-hours = { $hours } h { $minutes } min
home-countdown-minutes = { $minutes } min
home-out-not-downloaded = Pas encore téléchargé
home-out-downloading = Téléchargement
home-out-when = { $day } { $time }
home-next-up = À suivre
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, heure estimée
home-next-when-season = Saison { $season }, { $span }, { $time }
home-next-when-season-estimated = Saison { $season }, { $span }, { $time }, heure estimée
home-no-date = Sans date
home-tile-watched = vu
home-tile-downloading = téléchargement
home-tile-out = sorti { $ago }
home-tile-not-out = pas encore sorti
home-tile-estimated = estimé
home-tile-season-span = S{ $season } { $span }
home-tile-to-watch = { $count } à voir
home-day-nothing = Rien
home-now = Maintenant { $time }
home-card-aired = { $episode } sorti { $ago }
home-continue = Continuer
home-continue-fact = { $count } séries à poursuivre
home-on-hold = En attente
home-on-hold-fact = { $count } séries à reprendre
home-hero-open = Ouvrir { $title }
home-hero-pick-up = Reprendre la lecture
home-hero-up-next = À suivre
home-hero-downloading = Téléchargement de { $count ->
    [one] { $count } épisode
   *[other] { $count } épisodes
}
home-hero-min-left = { $count } min restantes
home-hero-watched = { $watched } sur { $total } vus
home-hero-resume = Reprendre l'épisode { $number }
home-hero-play = Lire l'épisode { $number }
home-hero-resume-unnumbered = Reprendre l'épisode
home-hero-play-unnumbered = Lire l'épisode
home-downloaded = Téléchargé récemment
home-downloaded-fact = { $count } séries avec de nouveaux fichiers ces deux semaines
home-week = Cette semaine
home-week-show-empty = Afficher les jours vides
home-week-hide-empty = Masquer les jours vides
home-week-airing = En diffusion
home-week-out = Sortis
home-week-coming-up = À venir
home-recommended = Recommandées pour vous
home-recommended-fact = D'après votre liste
home-recommended-scope = Recommandées depuis
home-recommended-all = Tout
home-recommended-in-library = Bibliothèque
home-recommended-on-list = Liste
home-recommended-none-library = Plus rien à recommander dans votre bibliothèque. Tout trouve des séries que vous n'avez pas encore.
home-recommended-none-list = Plus rien à recommander dans vos listes. Tout trouve des séries absentes de vos listes.
home-catch-up = Rattraper
home-catch-up-fact = { $count } séries avec des épisodes en attente
home-fresh = Commencer autre chose
home-fresh-fact = { $count } séries pas commencées
home-genres = Genres
home-open-library = Ouvrir la bibliothèque
home-dropped = Abandonnées
home-dropped-fact = { $count ->
    [one] { $count } fichier encore sur le disque
   *[other] { $count } fichiers encore sur le disque
}
home-out-now = Sortis
home-out-now-fact = Nouveaux épisodes cette semaine, d'après AniList
home-popular = Populaires cette saison
home-add-folder-title = Vos propres séries vont ici aussi
home-add-folder-body = Ajoutez le dossier où vivent vos animes et l'accueil se remplit avec ce que vous regardez.
home-add-folder-button = Ajouter un dossier
home-empty-scanned-title = Rien trouvé dans vos dossiers pour l'instant
home-empty-title = Pas encore de série
home-empty-scanned-body = AniBeam n'a trouvé ni série ni film dans les dossiers qu'il surveille. Analysez-les de nouveau, ou vérifiez les dossiers dans les réglages.
home-empty-body = Ajoutez le dossier où vivent vos animes et la bibliothèque se remplit toute seule.
home-scan-all = Tout analyser
home-library-settings = Réglages de la bibliothèque
home-failed-title = L'accueil n'a pas chargé
home-failed-body = AniBeam n'a pas pu lire la bibliothèque. Réessayez dans un instant.

## Onboarding

onboarding-language = Langue
onboarding-step-welcome = Bienvenue
onboarding-step-library = Bibliothèque
onboarding-step-lists = Votre liste
onboarding-step-downloads = Téléchargements
onboarding-step-ready = Prêt
onboarding-scan-reading = Lecture du dossier
onboarding-scan-found = { $count } séries trouvées. Continuez quand vous voulez ; le reste se complète tout seul.
onboarding-scan-starter-empty = Votre nouveau dossier est vide pour l'instant. Les séries que vous y mettez apparaissent ici toutes seules.
onboarding-scan-no-video = Aucun fichier vidéo trouvé. Vérifiez que c'est le bon dossier.
onboarding-scan-your-shows = vos séries
onboarding-scan-looking-up = Recherche de { $what }
onboarding-scan-looking-up-tally = Recherche de { $what }, { $posters } sur { $count } trouvées
onboarding-scan-posters = Récupération des affiches
onboarding-scan-posters-tally = Récupération des affiches, { $posters } sur { $count } trouvées
onboarding-scan-seasons = Recherche des saisons liées
onboarding-scan-seasons-tally = Recherche des saisons liées, { $posters } sur { $count } trouvées
onboarding-choose-folder = Choisissez votre dossier d'animes
onboarding-about = Une minute de configuration. Votre bibliothèque se termine toute seule.
onboarding-step-later = Plus tard
onboarding-skip-setup = Passer la configuration
onboarding-title-welcome = Bienvenue dans AniBeam
onboarding-title-library = Où sont vos animes ?
onboarding-title-lists = Connectez votre liste
onboarding-title-downloads = Comment marchent les téléchargements
onboarding-title-ready = Tout est prêt
onboarding-title-later = { $count ->
    [1] Prêt, une chose pour plus tard
    [2] Prêt, deux choses pour plus tard
   *[other] Prêt, trois choses pour plus tard
}
onboarding-body-welcome = AniBeam garde vos animes au même endroit. Il trouve les séries sur votre disque, les lit ici même, et tient AniList ou MyAnimeList à jour pendant que vous regardez.
onboarding-body-library = Choisissez le dossier où vous rangez vos séries. Chaque dossier qu'il contient devient une série, avec son affiche, ses épisodes et l'endroit où vous vous êtes arrêté. AniBeam ne fait que lire le dossier. Il ne déplace ni ne renomme jamais un fichier.
onboarding-body-lists = Connectez-vous une fois, et chaque épisode terminé ici est marqué dans votre liste, sans rien saisir. La connexion se fait dans votre navigateur, donc AniBeam ne voit jamais votre mot de passe.
onboarding-body-downloads = Suivez une série sur la page Téléchargements et chaque nouvel épisode arrive dans votre bibliothèque dès sa diffusion. Il faut deux choses pour ça : un endroit où chercher, et un client torrent pour télécharger.
onboarding-body-ready = Tout ceci se modifie plus tard dans les réglages, et Réglages, À propos relance cette configuration.
onboarding-skip-step = Passer pour l'instant
onboarding-get-started = Commencer
onboarding-open-app = Ouvrir AniBeam
onboarding-continue = Continuer
onboarding-payoff-library = Votre bibliothèque
onboarding-payoff-library-line = Affiches, épisodes et la suite, pour chaque série de vos dossiers.
onboarding-payoff-plays = Lecture ici même
onboarding-payoff-plays-line = Sous-titres, saut des génériques, et il retient où vous vous êtes arrêté.
onboarding-payoff-list = Votre liste, synchronisée
onboarding-payoff-list-line = Terminez un épisode et il compte sur AniList ou MyAnimeList.
onboarding-payoff-episodes = Les épisodes arrivent
onboarding-payoff-episodes-line = Suivez une série et chaque nouvel épisode arrive dans votre bibliothèque.
onboarding-films-too = Un dossier de films marche aussi : appelez-le Movies.
onboarding-choose-folder-button = Choisir un dossier
onboarding-no-folder = Je n'ai pas encore de dossier
onboarding-starter-where = Cela crée un dossier Anime dans { $dir } et l'utilise.
onboarding-add-another = Ajouter un autre dossier
onboarding-shelf-more = de plus
onboarding-preview-show = Votre série
onboarding-preview-finished = Épisode 3 terminé ici
onboarding-preview-on-list = Dans votre liste : 3 vus
onboarding-preview-set-by = Mis à jour par AniBeam dès le générique de fin
onboarding-lists-connected = Connecté. Ce que vous regardez ici compte maintenant dans votre liste.
onboarding-lists-no-account = Pas encore de compte ? Les deux sont gratuits, et un seul suffit. Vous pouvez vous déconnecter à tout moment dans Réglages, Comptes.
onboarding-search-title = Un endroit où chercher
onboarding-search-ready = { $count ->
    [one] { $count } source prête à chercher.
   *[other] { $count } sources prêtes à chercher.
}
onboarding-search-none = AniBeam n'a aucune source à lui. Vous les ajoutez depuis un dépôt de sources : un lien qu'on vous partage, à coller dans Réglages, Téléchargements.
onboarding-sources-after = S'ouvre après la configuration
onboarding-sources-add = Ajouter un dépôt après la configuration
onboarding-fetch-title = Qui télécharge
onboarding-fetch-auto = Laisser AniBeam s'en charger
onboarding-fetch-recommended = Recommandé
onboarding-fetch-auto-line = AniBeam apporte son propre qBittorrent et le fait tourner discrètement en arrière-plan. Rien à installer, rien à remplir.
onboarding-fetch-existing = J'utilise déjà qBittorrent
onboarding-fetch-existing-line = AniBeam se connecte au qBittorrent que vous utilisez déjà. Vous saisirez ensuite son adresse et ses identifiants dans les réglages.
onboarding-ready-library = Bibliothèque
onboarding-ready-library-none = Pas encore de dossier. C'est dans Réglages, Bibliothèque quand vous voulez.
onboarding-ready-library-series = { $count ->
    [one] { $count } dossier
   *[other] { $count } dossiers
}, { $series } séries pour l'instant
onboarding-ready-library-empty = { $count ->
    [one] { $count } dossier
   *[other] { $count } dossiers
}, vide pour l'instant
onboarding-ready-library-scanning = { $count ->
    [one] { $count } dossier
   *[other] { $count } dossiers
}, analyse en cours
onboarding-ready-lists = Votre liste
onboarding-ready-lists-as = { $tracker }, en tant que { $user }
onboarding-ready-lists-none = Pas encore connectée. C'est dans Réglages, Comptes quand vous voulez.
onboarding-ready-downloads = Téléchargements
onboarding-ready-downloads-line = { $sources ->
    [0] Pas encore de source
    [one] { $sources } source
   *[other] { $sources } sources
}, { $client ->
    [own] votre propre qBittorrent.
   *[bundled] le qBittorrent d'AniBeam.
} { $next ->
    [ready] Tout est prêt.
    [opens] Réglages, Téléchargements s'ouvre ensuite.
   *[both] Les deux sont dans Réglages, Téléchargements.
}
onboarding-ready-open-downloads = Ouvrir Réglages, Téléchargements
onboarding-ready-set-up = Configurer maintenant

## Rail.qml

rail-home = Accueil
rail-library = Bibliothèque
rail-airing = Diffusion
rail-feed = Fil
rail-stats = Stats
rail-show-info = Infos de la série
rail-downloads = Téléchargements
rail-settings = Réglages
rail-downloads-complete =
    { $count ->
        [one] Téléchargements : { $count } abonnement est complet
       *[other] Téléchargements : { $count } abonnements sont complets
    }
rail-title-language-name = Langue des titres : { $language }. Appuyez pour la suivante
rail-title-language-english = Anglais
rail-title-language-tip = Langue des titres : romaji, anglais ou japonais
rail-whats-new-in = Nouveautés de { $release }
rail-whats-new = Nouveautés
rail-settings-note = Réglages : { $note }

## FixSheet.qml

fix-kind-episode = Épisode
fix-kind-opening = Opening
fix-kind-ending = Ending
fix-kind-preview = Aperçu
fix-kind-special = Spécial
fix-kind-other = Autre
fix-reads-season-episode = Saison { $season } épisode { $number }
fix-reads-extra = { $kind } { $index }
fix-apply-failed = Impossible d'appliquer les corrections.
fix-title = Corriger la lecture des fichiers
fix-lede = { $title }. Dites ce qu'est vraiment chaque fichier ; chaque analyse suivante le garde.
fix-rescan = Réanalyser le dossier
fix-rescan-started = Nouvelle lecture du dossier
fix-filled-in = Rempli
fix-fill-in = Remplir
fix-column-file = Fichier
fix-column-reads-as = Lu comme
fix-column-should-be = Devrait être
fix-fixed = Corrigé
fix-number-placeholder = N°
fix-index-placeholder = Aucun
fix-keep = Garder la correction
fix-leave-as-read = Laisser ce fichier tel qu'il est lu
fix-read-by-name = Relire ce fichier d'après son nom
fix-report = Signaler sur GitHub
fix-report-body = Ouvre un ticket prérempli à envoyer. Il est public et nomme ces fichiers.
fix-report-body-agent = Ouvre un ticket prérempli à envoyer. Il est public et nomme ces fichiers. { $agent } l'examine d'abord.
fix-close-unapplied =
    { $count ->
        [one] Fermer sans appliquer { $count } correction ?
       *[other] Fermer sans appliquer { $count } corrections ?
    }
fix-number-invalid = Un numéro en rouge est à corriger d'abord
fix-close-anyway = Fermer quand même
fix-keep-editing = Continuer
fix-apply-count =
    { $count ->
        [one] Appliquer { $count } correction
       *[other] Appliquer { $count } corrections
    }

## MatchPanel.qml

match-matched-to = Associée à { $title }
match-applied = Association appliquée
match-found-tvmaze = Série trouvée sur TVmaze
match-found-anilist = Trouvée sur AniList
match-found-anilist-season = Trouvée sur AniList, saison { $season }
match-found-mal = Trouvée sur MyAnimeList
match-sites-and = { $first } et { $next }
match-matched-on = Associée sur { $sites }
match-provider-silent = { $provider } n'a pas répondu
match-title-walk = Association, { $walk }
match-title = Association
match-skip = Passer
match-refresh-info = Actualiser les infos
match-refreshing = Actualisation de { $title }
match-clear-confirm = Dissocier : supprime le lien et le nombre d'épisodes fixé
match-clear = Dissocier
match-cleared = Association effacée
match-search-placeholder = Chercher sur { $provider }
match-link-tvmaze = Collez un lien TVmaze ou IMDb
match-link-anilist = Collez un lien AniList ou MyAnimeList
match-look-up = Chercher
match-searching = Recherche sur { $provider }
match-type-more = Saisissez au moins deux lettres pour chercher sur { $provider }.
match-no-results = Aucun titre trouvé sur { $provider }.

## WatchingPage.qml

watching-episode-code = Ép. { $number }
watching-countdown-days = { $days } j { $hours } h
watching-countdown-hours = { $hours } h { $minutes } min
watching-countdown-minutes = { $minutes } min
watching-on-disk = Sur le disque
watching-on-anilist = Sur AniList
watching-ready-to-start = Prête à commencer
watching-not-in-library = Pas dans la bibliothèque
watching-get-to-start = Obtenez des épisodes pour commencer
watching-anilist-updated = AniList mis à jour { $when }
watching-airing = En diffusion
watching-next-in = Suivant : { $title } dans { $wait }
watching-episodes-waiting = Épisodes en attente
watching-series-behind =
    { $count ->
        [one] { $count } série en retard
       *[other] { $count } séries en retard
    }
watching-up-to-date = À jour
watching-anilist-only = Sur AniList seulement
watching-planning-summary-off = Ce que vous prévoyez de regarder sur AniList.
watching-planning-summary =
    { $count ->
        [one] { $count } série prévue, { $here } sur le disque.
       *[other] { $count } séries prévues, { $here } sur le disque.
    }
watching-summary-off = Ce que vous regardez ou revisionnez sur AniList.
watching-summary-waiting =
    { $count ->
        [one] { $count } série a des épisodes en attente.
       *[other] { $count } séries ont des épisodes en attente.
    }
watching-caught-up = Tout est rattrapé.
watching-start-watching = Commencer
watching-play-next = Lire la suite
watching-open-on-anilist = Ouvrir sur AniList
watching-refreshing = Actualisation
watching-connect-title = AniList n'est pas connecté
watching-connect-body = Connectez-le : cette page liste ce que vous regardez ou revoyez sur AniList, avec Lire sur ce que vous avez et un décompte sur ce qui est diffusé.
watching-connect-waiting = En attente du navigateur
watching-connect-anilist = Connecter AniList
watching-load-failed-title = La liste En cours n'a pas chargé
watching-load-failed-body = Impossible de joindre AniList. La liste revient à la prochaine actualisation.
watching-planning-failed-title = La liste À voir n'a pas chargé
watching-planning-failed-body = Son cache est illisible. Réessayez, ou elle reviendra à la prochaine actualisation.
watching-planning-empty-title = Rien dans votre liste À voir
watching-planning-empty-body = À voir dans En diffusion, ou À voir dans le menu de liste d'une série, la met ici.
watching-airing-now = En diffusion
watching-empty-title = Rien dans votre liste En cours
watching-empty-body = Marquez une série En cours sur AniList et elle arrive ici.
watching-open-anilist = Ouvrir AniList

## LibraryPage.qml

library-hidden =
    { $count ->
        [one] { $count } série masquée
       *[other] { $count } séries masquées
    }
library-hidden-sync-stops =
    { $count ->
        [one] { $count } série masquée, et sa synchro s'arrête
       *[other] { $count } séries masquées, et leur synchro s'arrête
    }
library-unhidden =
    { $count ->
        [one] { $count } série de retour, et de nouveau synchronisée
       *[other] { $count } séries de retour, et de nouveau synchronisées
    }
library-undo = Annuler
library-clearing-score =
    { $count ->
        [one] Effacement de la note de { $count } série
       *[other] Effacement de la note de { $count } séries
    }
library-rating =
    { $count ->
        [one] Note { $score } pour { $count } série
       *[other] Note { $score } pour { $count } séries
    }
library-moving =
    { $count ->
        [one] Déplacement de { $count } série vers { $list }
       *[other] Déplacement de { $count } séries vers { $list }
    }
library-rescanning =
    { $count ->
        [one] Nouvelle analyse de { $count } série
       *[other] Nouvelle analyse de { $count } séries
    }
library-finding-matches =
    { $count ->
        [one] Recherche d'une association pour { $count } série
       *[other] Recherche d'une association pour { $count } séries
    }
library-sort-last-viewed = Vu récemment
library-sort-downloaded = Téléchargé récemment
library-sort-title = Titre
library-sort-progress = Progression
library-sort-average-score = Note moyenne
library-sort-my-score = Ma note
library-dir-a-to-z = A à Z
library-dir-z-to-a = Z à A
library-dir-oldest = Plus ancien d'abord
library-dir-newest = Plus récent d'abord
library-dir-least-watched = Moins vu d'abord
library-dir-most-watched = Plus vu d'abord
library-dir-lowest = Plus bas d'abord
library-dir-highest = Plus haut d'abord
library-group-none = Aucun
library-group-season = Saison
library-group-type = Type
library-group-genre = Genre
library-group-watch-state = État de visionnage
library-scan-started = Analyse lancée
library-tab-all = Tout
library-tab-series = Séries
library-tab-movies = Films
library-tab-watched = Vus
library-tab-hidden = Masqués
library-count-movies =
    { $count ->
        [one] { $count } film
       *[other] { $count } films
    }
library-count-watched = { $count } vus
library-count-hidden = { $count } masqués
library-count-series =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    }
library-title = Bibliothèque
library-all-genres = Tous les genres
library-search-placeholder = Titre ou dossier
library-sort = Trier
library-group = Grouper
library-franchises-together = Franchises regroupées
library-watched-shows = Séries vues
library-load-failed-title = La bibliothèque n'a pas chargé
library-no-matches = Aucun résultat pour « { $query } »
library-none-in-genre =
    { $what ->
        [series] Aucune série { $genre } ici
        [films] Aucun film { $genre } ici
        [watched] Aucune série vue { $genre } ici
        [hidden] Aucune série masquée { $genre } ici
       *[shows] Aucune série { $genre } ici
    }
library-none-here =
    { $what ->
        [series] Aucune série ici
        [films] Aucun film ici
        [watched] Aucune série vue ici
        [hidden] Aucune série masquée ici
       *[shows] Aucune série ici
    }
library-load-failed-body = AniBeam n'a pas pu lire la bibliothèque. Réessayez dans un instant.
library-no-matches-body = Vérifiez l'orthographe, ou cherchez dans toute la bibliothèque depuis Tout.
library-none-in-genre-body = Rien dans cette vue n'a ce genre. Tous les genres affiche le reste.
library-none-in-tab-body = Votre bibliothèque n'en a pas encore. Tout affiche tout ce qu'elle contient.
library-all-filtered-body = Toute votre bibliothèque est filtrée hors de cette vue.
library-clear-search = Effacer la recherche
library-show-all = Tout afficher
library-picked = { $count } choisies
library-pick-all = Tout
library-pick-all-tip = Choisir toutes les cartes affichées (Ctrl+A)
library-unhide = Afficher
library-hide = Masquer
library-rate = Noter
library-rate-tip = Une note pour chaque choix, sur chaque suivi connecté
library-needs-tracker = Demande un suivi connecté et une série associée
library-move-to-list = Changer de liste
library-rescan = Réanalyser
library-find-match = Associer
library-find-match-tip = Chercher chaque choix sans association
library-all-matched = Chaque choix a une association
library-empty-scanned-title = Rien trouvé dans vos dossiers pour l'instant
library-empty-title = Votre bibliothèque est vide
library-empty-scanned-body = AniBeam n'a trouvé ni série ni film dans les dossiers qu'il surveille. Analysez-les de nouveau, ou vérifiez les dossiers dans les réglages.
library-empty-body = Ajoutez le dossier où vivent vos animes et la bibliothèque se remplit toute seule, ou importez un export AniBeam.
library-scan-all = Tout analyser
library-add-folder = Ajouter un dossier
library-settings = Réglages de la bibliothèque
library-import = Importer
library-import-title = Importer un export AniBeam
library-import-filter-export = Export AniBeam (*.json)
library-import-filter-all = Tous les fichiers (*)
library-import-started = Import lancé

## MetadataPage.qml

metadata-title = Infos de la série
metadata-filter-all = Tout { $count }
metadata-filter-series = Séries { $count }
metadata-filter-films = Films { $count }
metadata-filter-missing = Manquants { $count }
metadata-scan-started = Analyse lancée
metadata-sort-added = Ajout
metadata-sort-season = Saison
metadata-sort-a-to-z = A à Z
metadata-section-unmatched = Non reconnues
metadata-section-missing = Dossier introuvable
metadata-section-incomplete = Épisodes manquants
metadata-section-empty = Non téléchargées
metadata-section-airing = En diffusion
metadata-section-complete = Complètes
metadata-no-info = Pas encore d'infos
metadata-state-aired = { $have } sur { $aired } diffusés
metadata-state-of = { $have } sur { $expected }
metadata-state-of-extra = { $have } sur { $expected } +{ $extra }
metadata-detail-none-aired = Aucun des { $aired } épisodes diffusés n'est sur le disque
metadata-detail-aired = { $have } sur { $aired } diffusés sur le disque
metadata-detail-aired-missing = { $have } sur { $aired } diffusés sur le disque, { $missing } manquants
metadata-detail-none = Aucun des { $expected } épisodes n'est sur le disque
metadata-detail-missing = { $have } sur { $expected } sur le disque, { $missing } manquants
metadata-walk-position = { $at } sur { $total }
metadata-walk-done = C'était la dernière
metadata-walk-left =
    { $count ->
        [one] Plus qu'une sous À associer
       *[other] Plus que { $count } sous À associer
    }
metadata-countdown-days = { $days } j { $hours } h
metadata-countdown-hours = { $hours } h { $minutes } min
metadata-countdown-minutes = { $minutes } min
metadata-next-in = Suivant dans { $wait }
metadata-files-of = { $have } sur { $expected }
metadata-files-on-disk = Fichiers sur le disque
metadata-forgot = { $title } oubliée
metadata-refreshed-failed = { $refreshed } actualisées, échec pour { $failed }
metadata-refreshed =
    { $count ->
        [one] { $count } série actualisée
       *[other] { $count } séries actualisées
    }
metadata-found =
    { $count ->
        [one] { $count } série trouvée
       *[other] { $count } séries trouvées
    }
metadata-found-some-left =
    { $count ->
        [one] { $count } série trouvée, { $unmatched } toujours non reconnues
       *[other] { $count } séries trouvées, { $unmatched } toujours non reconnues
    }
metadata-found-nothing = Rien de nouveau, { $unmatched } toujours non reconnues
metadata-all-recognised = Chaque série est reconnue
metadata-count =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    }
metadata-search-placeholder = Filtrer les titres
metadata-find-rest = Chercher le reste
metadata-find-rest-started = Recherche des dossiers qu'AniBeam ne connaît pas encore
metadata-find-rest-tip = Cherche chaque dossier qu'AniBeam ne connaît pas encore, sur chaque site d'infos dans l'ordre des réglages
metadata-refresh-all = Tout actualiser
metadata-refresh-all-started = Mise à jour des infos de chaque série
metadata-refresh-all-tip = Récupère de nouveau les infos de chaque série reconnue
metadata-crawling = Recherche des liens de franchise
metadata-crawl-progress = { $done } sur { $total }
metadata-load-failed-title = Les infos n'ont pas chargé
metadata-empty-scanned-title = Rien trouvé dans vos dossiers pour l'instant
metadata-no-matches = Aucun résultat pour « { $query } »
metadata-nothing-under =
    { $filter ->
        [Series] Rien sous Séries
        [Movies] Rien sous Films
        [MissingFiles] Rien sous Manquants
       *[other] Rien sous { $filter }
    }
metadata-load-failed-body = AniBeam n'a pas pu lire la bibliothèque. Réessayez dans un instant.
metadata-empty-scanned-body = AniBeam n'a trouvé aucune série dans les dossiers qu'il surveille. Analysez-les de nouveau, ou vérifiez les dossiers dans les réglages.
metadata-empty-body = Votre bibliothèque est vide. Ajoutez le dossier où vivent vos animes et AniBeam l'analyse.
metadata-no-matches-body = Aucun titre de cette vue ne contient ces mots.
metadata-nothing-under-body = Aucune de vos séries ne correspond à ce filtre pour l'instant. Tout affiche le reste.
metadata-scan-all = Tout analyser
metadata-add-folder = Ajouter un dossier
metadata-clear-search = Effacer la recherche
metadata-show-all = Tout afficher
metadata-refreshing = Actualisation de { $title }
metadata-refresh-tip = Récupérer de nouveau les infos de cette série
metadata-forget-armed = Retirer
metadata-forget-confirm-tip = Cliquez encore pour retirer cette série et son historique d'AniBeam
metadata-forget-tip = Le dossier n'est plus sur le disque : retirer cette série et son historique d'AniBeam

## AiringPage.qml

airing-title = En diffusion
airing-sort-popular = Popularité
airing-sort-score = Note
airing-sort-trending = Tendance
airing-sort-favourites = Favoris
airing-sort-soonest = Prochain épisode
airing-sort-aired = Épisodes diffusés
airing-sort-episodes = Nombre d'épisodes
airing-sort-start = Date de début
airing-sort-duration = Durée d'épisode
airing-sort-title = Titre
airing-dir-most-popular = Plus populaire d'abord
airing-dir-least-popular = Moins populaire d'abord
airing-dir-lowest = Plus bas d'abord
airing-dir-highest = Plus haut d'abord
airing-dir-least-trending = Moins tendance d'abord
airing-dir-most-trending = Plus tendance d'abord
airing-dir-fewest = Moins d'abord
airing-dir-most = Plus d'abord
airing-dir-soonest = Plus proche d'abord
airing-dir-latest = Plus tard d'abord
airing-dir-oldest = Plus ancien d'abord
airing-dir-newest = Plus récent d'abord
airing-dir-shortest = Plus court d'abord
airing-dir-longest = Plus long d'abord
airing-dir-a-to-z = A à Z
airing-dir-z-to-a = Z à A
airing-season =
    { $season ->
        [Winter] Hiver { $year }
        [Spring] Printemps { $year }
        [Summer] Été { $year }
        [Fall] Automne { $year }
       *[other] { $season } { $year }
    }
airing-ago-minutes = il y a { $minutes } min
airing-ago-hours = il y a { $hours } h
airing-ago-days = il y a { $days } j
airing-episode-out = { $episode } sorti { $ago }
airing-no-date = Pas encore de date
airing-in-library = Dans la bibliothèque
airing-out-today =
    { $count ->
        [one] { $count } épisode sorti aujourd'hui
       *[other] { $count } épisodes sortis aujourd'hui
    }
airing-dismissed-one = { $title } ne s'affichera plus ici
airing-dismissed =
    { $count ->
        [one] { $count } série ne s'affichera plus ici
       *[other] { $count } séries ne s'afficheront plus ici
    }
airing-planned-already-one = { $title } est déjà dans votre liste
airing-planned-already-all = Elles sont toutes déjà dans votre liste
airing-planned =
    { $count ->
        [one] { $count } série passe dans votre liste À voir
       *[other] { $count } séries passent dans votre liste À voir
    }
airing-planned-some-already =
    { $count ->
        [one] { $count } série passe dans votre liste À voir, { $already } y étaient déjà
       *[other] { $count } séries passent dans votre liste À voir, { $already } y étaient déjà
    }
airing-play-next = Lire la suite
airing-plan-to-watch = À voir
airing-get-more-episodes = Plus d'épisodes
airing-get-episodes = Obtenir des épisodes
airing-open-on-anilist = Ouvrir sur AniList
airing-not-interested = Sans intérêt
airing-not-interested-count = { $count } sans intérêt
airing-not-interested-tip = Les séries marquées sans intérêt, dans les réglages
airing-refreshing = Actualisation
airing-pick-season = Choisir une autre saison
airing-filter-all = Tout
airing-filter-lists = Dans mes listes
airing-filter-library = Dans ma bibliothèque
airing-hide-lists = Masquer mes listes
airing-hide-lists-tip = Exclure les séries de vos listes AniList ou MAL
airing-hide-library = Masquer ma bibliothèque
airing-hide-library-tip = Exclure les séries dont vous avez des épisodes
airing-out-now = Sortis
airing-out-last-day =
    { $count ->
        [one] { $count } épisode ces dernières 24 heures
       *[other] { $count } épisodes ces dernières 24 heures
    }
airing-out-fewer = Moins
airing-out-all = Tous les { $count }
airing-sort = Trier
airing-load-failed-title = Impossible de joindre AniList
airing-load-failed-body = Les séries en diffusion viennent d'AniList. Elles reviennent à la prochaine actualisation.
airing-reading-title = Lecture des séries en diffusion
airing-nothing-now = Rien en diffusion en ce moment
airing-nothing-for = Rien en diffusion pour { $season }
airing-nothing-read = Rien de lu pour l'instant
airing-reading-body = Chaque série en diffusion, depuis AniList. Cela prend quelques secondes la première fois.
airing-nothing-body = AniList ne liste encore aucune série pour cette saison. Actualiser la relit, ou choisissez une autre saison.
airing-nothing-read-body = Actualiser lit chaque série en diffusion depuis AniList.
airing-hides-empty-title = Vos filtres ne laissent rien ici
airing-lists-empty-title = Rien ici n'est dans vos listes
airing-library-empty-title = Rien ici n'est dans votre bibliothèque
airing-hides-empty-body = Chaque série ici est dans vos listes ou votre bibliothèque, et vous les avez masquées.
airing-lists-empty-body = À voir sur une carte l'ajoute à votre liste.
airing-library-empty-body = Obtenir des épisodes sur une carte la télécharge dans votre bibliothèque.
airing-show-everything = Tout afficher
airing-picked = { $count } choisies
airing-undo = Annuler

## EpisodeRow.qml

episode-row-date-time-format = ddd d MMM, HH:mm
episode-row-date-upcoming-format = ddd d MMM
episode-row-date-format = d MMM yyyy
episode-row-mark-unwatched-tip = { $state }. Cliquez pour marquer non vu à partir d'ici
episode-row-mark-watched-tip = { $state }. Cliquez pour marquer vu jusqu'ici
episode-row-next-up = À suivre
episode-row-versions =
    { $count ->
        [one] { $count } version
       *[other] { $count } versions
    }
episode-row-versions-tip =
    { $count ->
        [one] Le même épisode en { $count } release. Cliquez pour choisir laquelle lire
       *[other] Le même épisode en { $count } releases. Cliquez pour choisir laquelle lire
    }
episode-row-downloading = Téléchargement { $percent } %
episode-row-get = Obtenir l'épisode
episode-row-predicted = Prévu
episode-row-extra = Bonus
episode-row-resume = Reprendre à { $time }

## EpisodeCubes.qml

episode-cubes-one = Un épisode
episode-cubes-range =
    { $count ->
        [one] { $count } épisode, { $first } à { $last }
       *[other] { $count } épisodes, { $first } à { $last }
    }
episode-cubes-tip = Épisode { $number } : { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
        [one] { $count } bonus
       *[other] { $count } bonus
    }
episodes-pane-unmatched =
    { $count ->
        [one] { $count } fichier non associé
       *[other] { $count } fichiers non associés
    }
episodes-pane-on-disk = Sur le disque
episodes-pane-not-on-disk = Pas sur le disque
episodes-pane-season = S{ $season }
episodes-pane-get-tip = Obtenir l'épisode { $number }
episodes-pane-none-on-disk = Aucun épisode sur le disque
episodes-pane-change-match = Changer l'association
episodes-pane-refreshing = Actualisation de { $title }

## ShowDetail.qml

show-detail-not-in-library = Ce dossier n'est pas encore dans la bibliothèque. Dès qu'une analyse le trouve, son affiche et ses épisodes s'affichent ici.
show-detail-film-on-disk = Le film est sur le disque.
show-detail-film-not-on-disk = Le film n'est pas encore sur le disque.
show-detail-complete-line = Diffusion terminée, et les { $total } épisodes sont là. Plus rien ne viendra, l'abonnement n'a donc plus rien à faire.
show-detail-finished = Diffusion terminée.
show-detail-airing = En diffusion.
show-detail-have-of-total = { $have } épisodes sur { $total } ici.
show-detail-have =
    { $count ->
        [one] { $count } épisode ici.
       *[other] { $count } épisodes ici.
    }
show-detail-next-predicted =
    { $count ->
        [one] { $span } est prévu dans { $wait }.
       *[other] { $span } sont prévus dans { $wait }.
    }
show-detail-next-airs =
    { $count ->
        [one] { $span } sort dans { $wait }.
       *[other] { $span } sortent dans { $wait }.
    }
show-detail-missing =
    { $count ->
        [one] L'épisode { $list } n'est pas là.
       *[other] Les épisodes { $list } ne sont pas là.
    }
show-detail-missing-more = Les épisodes { $list } et { $more } autres ne sont pas là.
show-detail-downloaded = Téléchargé
show-detail-downloading = Téléchargement { $percent } %
show-detail-taken-adding = Pris par la règle, ajout
show-detail-taken = Pris par la règle
show-detail-left-out = Écarté par la règle
show-detail-subscribed-reading = Abonné. qBittorrent lit le flux.
show-detail-taken-moving = La règle prend { $taken } { $releases ->
        [one] sur { $releases } release du flux
       *[other] sur { $releases } releases du flux
    } ; { $moving ->
        [one] { $moving } est
       *[other] { $moving } sont
    } en téléchargement ci-dessous.
show-detail-taken-held = La règle prend { $taken } { $releases ->
        [one] sur { $releases } release du flux
       *[other] sur { $releases } releases du flux
    } ; { $torrents ->
        [one] { $torrents } torrent est
       *[other] { $torrents } torrents sont
    } dans le client.
show-detail-taken-adding-line = La règle prend { $taken } { $releases ->
        [one] sur { $releases } release du flux
       *[other] sur { $releases } releases du flux
    } ; qBittorrent { $taken ->
        [one] l'ajoute.
       *[other] les ajoute.
    }
show-detail-feed-holds-nothing = Le flux est encore vide. Les releases qui conviennent se téléchargent dès qu'elles y apparaissent.
show-detail-nothing-fits = Rien dans le flux ne correspond encore à la règle. Les releases qui conviennent se téléchargent dès qu'elles apparaissent.
show-detail-complete = Complet
show-detail-open-in-library = Ouvrir dans la bibliothèque
show-detail-feed-finds-nothing = Ce flux ne trouve rien
show-detail-feed-empty-query = Il demande « { $query } » à { $source }, qui ne renvoie rien, donc rien de nouveau ne peut atteindre la règle. Une recherche plus courte répond en général ; les mots retirés peuvent rester dans la règle.
show-detail-feed-empty = Il interroge { $source }, qui ne renvoie rien, donc rien de nouveau ne peut atteindre la règle. Une recherche plus courte répond en général ; les mots retirés peuvent rester dans la règle.
show-detail-feed-empty-earlier = qBittorrent affiche encore ce qu'une version précédente de ce flux a trouvé.
show-detail-repairing = Réparation
show-detail-repair = Réparer
show-detail-unsubscribe = Se désabonner
show-detail-remove-torrents =
    { $count ->
        [one] Retirer le torrent
       *[other] Retirer { $count } torrents
    }
show-detail-searches = Recherche
show-detail-quoted = « { $words } »
show-detail-whole-feed = tout le flux
show-detail-from = Depuis
show-detail-rule = Règle
show-detail-rule-unfollowable = Impossible de récupérer les nouveaux épisodes : sa source répond aux recherches mais n'a pas de flux
show-detail-rule-active = Active
show-detail-keeps = Garde
show-detail-keeps-value = les releases avec « { $words } »
show-detail-drops = Écarte
show-detail-drops-value = les releases avec { $words }
show-detail-or = { $first } ou { $second }
show-detail-last-took = Dernière prise
show-detail-saves-to = Enregistre dans
show-detail-folder = Dossier
show-detail-in-the-feed = Dans le flux
show-detail-feed-reading = Lecture du flux.
show-detail-feed-offers = Ce que le flux propose, du plus récent.
show-detail-feed-is-empty = Le flux est vide.
show-detail-read-again = Relire
show-detail-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
show-detail-download = Télécharger
show-detail-torrents = Torrents
show-detail-torrents-moving =
    { $count ->
        [one] { $count } en téléchargement ; les barres se remplissent au fur et à mesure.
       *[other] { $count } en téléchargement ; les barres se remplissent au fur et à mesure.
    }
show-detail-torrents-held = Ce que le client garde pour ce dossier.
show-detail-torrents-waiting = En attente que qBittorrent ajoute ce que la règle prend.
show-detail-torrents-none = Le client n'a rien pour ce dossier.

## SeriesPage.qml

series-trackers-both = { $first } et { $second }
series-count-cleared = Nombre d'épisodes effacé
series-count-set = Fixé à { $count } épisodes
series-held-one-undated = Épisode { $first } pas encore daté
series-held-range-undated = Épisodes { $first } à { $last } pas encore datés
series-held-one-due = Épisode { $first } prévu en { $when }
series-held-range-due = Épisodes { $first } à { $last } prévus en { $when }
series-held-month-format = MMMM yyyy
series-fix-schedule = Épisodes et dates
series-fix-files = Fichiers
series-fix-rescan-folder = Réanalyser le dossier
series-fix-match = Associer
series-find-match = Associer
series-rescan-show = Réanalyser la série
series-open-in-show-info = Ouvrir dans Infos de la série
series-delete-done =
    { $kind ->
        [film] { $count ->
            [one] { $count } film mis à la corbeille
           *[other] { $count } films mis à la corbeille
        }
        [episode] { $count ->
            [one] { $count } épisode mis à la corbeille
           *[other] { $count } épisodes mis à la corbeille
        }
       *[file] { $count ->
            [one] { $count } fichier mis à la corbeille
           *[other] { $count } fichiers mis à la corbeille
        }
    }
series-delete-picked =
    { $kind ->
        [film] { $count ->
            [one] { $count } film choisi.
           *[other] { $count } films choisis.
        }
        [episode] { $count ->
            [one] { $count } épisode choisi.
           *[other] { $count } épisodes choisis.
        }
       *[file] { $count ->
            [one] { $count } fichier choisi.
           *[other] { $count } fichiers choisis.
        }
    }
series-delete-move-picked =
    { $kind ->
        [film] { $count ->
            [one] Mettre { $count } film à la corbeille
           *[other] Mettre { $count } films à la corbeille
        }
        [episode] { $count ->
            [one] Mettre { $count } épisode à la corbeille
           *[other] Mettre { $count } épisodes à la corbeille
        }
       *[file] { $count ->
            [one] Mettre { $count } fichier à la corbeille
           *[other] Mettre { $count } fichiers à la corbeille
        }
    }
series-delete-click = Cliquez sur les fichiers à supprimer.
series-delete-where = Ils vont à la corbeille, où vous pouvez les récupérer. Ce que vous avez vu reste suivi.
series-delete-move = Mettre à la corbeille
series-pick-none = Ne rien choisir
series-pick-all = Tout choisir
series-rescan-started = Nouvelle analyse lancée
series-rated = Noté { $score }
series-rated-cleared = Note effacée
series-score-failed = La note a échoué
series-still-downloading = { $episode } est encore en téléchargement
series-format-series = Série
series-tracker-refused = { $tracker } ne l'a pas accepté
series-tracker-refused-why = { $tracker } ne l'a pas accepté : { $why }
series-watched-up-to = Vu jusqu'à { $episode }
series-unwatched-from = Non vu à partir de { $episode }
series-undo = Annuler
series-play-downloading = Téléchargement de l'épisode { $number }
series-play-downloading-unnumbered = Téléchargement de l'épisode
series-play-continue = Continuer
series-play-again = Revoir
series-play-continue-episode = Continuer l'épisode { $number }
series-play-again-from = Revoir depuis l'épisode { $number }
series-play-episode = Lire l'épisode { $number }
series-page-not-tracked = Non suivie
series-not-tracked-on-disk =
    { $count ->
        [one] Non suivie, { $count } épisode sur le disque
       *[other] Non suivie, { $count } épisodes sur le disque
    }
series-watched = { $count } vus
series-watched-of = { $watched } sur { $total } vus
series-watched-of-estimate = { $watched } sur { $total }+ vus
series-facts-episodes-estimate =
    { $count ->
        [one] { $count } épisode+
       *[other] { $count } épisodes+
    }
series-facts-each = { $runtime } chacun
series-facts-on-disk = { $count } sur le disque
series-next = Suivant : { $episode }
series-episodes-downloading =
    { $count ->
        [one] { $count } épisode en téléchargement
       *[other] { $count } épisodes en téléchargement
    }
series-all-watched = Tout vu
series-no-poster = Pas d'affiche
series-next-season-span = S{ $season } { $span }
series-next-predicted = { $span } prévu le { $date }, dans { $wait }
series-next-date-format = ddd d MMM
series-next-airs =
    { $count ->
        [one] { $span } sort dans { $wait }
       *[other] { $span } sortent dans { $wait }
    }
series-downloading-episodes =
    { $count ->
        [one] Téléchargement de { $count } épisode
       *[other] Téléchargement de { $count } épisodes
    }
series-count-season-ends = La saison { $season } finit à l'épisode { $number }
series-count-ends = Finit à l'épisode { $number }
series-count-gave-way = Vous aviez fixé { $count } épisodes ; ce n'est plus valable
series-count-set-by-you = { $line }, fixé par vous
series-change = Modifier
series-get-episodes = Obtenir des épisodes
series-your-score = Votre note { $score }
series-rate = Noter
series-page-average-score = Note moyenne { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = { $votes } votes
series-add-to-list = Ajouter à une liste
series-page-rewatched =
    { $count ->
        [one] Revue { $count } fois
       *[other] Revue { $count } fois
    }
series-your-notes = Vos notes
series-add-notes = Annoter
series-notes-tip = Notes sur votre entrée de liste, gardées sur { $sites }
series-unhide = Afficher
series-hide = Masquer
series-hidden-toast = { $title } est masquée ici, et AniBeam arrête de la synchroniser avec AniList et MyAnimeList
series-unhidden-toast = { $title } est de retour, et se synchronise de nouveau
series-hide-tip-tracked = Masquée sur chaque page ici, et AniBeam arrête de la synchroniser avec AniList et MyAnimeList
series-hide-tip = Masquée de chaque liste
series-fixes = Corriger
series-fixes-tip = Corriger les fichiers, le nombre d'épisodes, l'association, ou réanalyser
series-quick-fix = Correction rapide
series-film = Film
series-episodes = Épisodes
series-extras = Openings, endings et plus
series-extras-openings = Openings
series-extras-endings = Endings
series-extras-previews = Aperçus et bandes-annonces
series-extras-specials = Spéciaux
series-extras-other = Autres
series-extras-group = { $kind }, { $count }
series-extra-files = Fichiers en trop
series-extra-files-warning =
    { $count ->
        [one] Un fichier dépasse les { $total } épisodes attendus : mal nommé, en double ou spécial. Renommez-le ou retirez-le.
       *[other] { $count } fichiers dépassent les { $total } épisodes attendus : mal nommés, en double ou spéciaux. Renommez-les ou retirez-les.
    }
series-synopsis = Synopsis
series-less = Moins
series-more = Plus
series-match = Associer
series-not-matched = Pas encore associée, donc ni affiche, ni synopsis, ni suivi. Trouvez ses infos sur AniList ou TVmaze.
series-naming-title = Des noms qui s'associent
series-naming-body = Le dossier porte le titre de la série, tel qu'AniList l'écrit, suivi de la saison quand ce n'est pas la première. Chaque fichier porte le numéro de l'épisode.
series-fix-how-files-read = Corriger la lecture des fichiers
series-characters = Personnages
series-related = Liées
series-related-filling = AniList complète encore la liste.
series-tags = Tags
series-hide-spoilers = Masquer les spoilers
series-spoilers = Spoilers
series-all-tags = Tous les tags
series-tag = { $name }, { $rank } %
series-recommendations = Recommandations
series-relation-owned = Dans votre bibliothèque
series-relation-unreleased = Pas encore sortie
series-relation-on-anilist = Sur AniList
series-score-saved = Note enregistrée
series-score-cleared = Note effacée

## GetShow.qml

get-show-the-client = le client torrent
get-show-group-batch = Lot de { $group }
get-show-run = { $first } à { $last }
get-show-episodes =
    { $count ->
        [one] Épisode { $list }
       *[other] Épisodes { $list }
    }
get-show-episodes-inline =
    { $count ->
        [one] l'épisode { $list }
       *[other] les épisodes { $list }
    }
get-show-sending-to = Envoi à { $client }
get-show-go =
    { $count ->
        [one] Obtenir { $count } épisode
       *[other] Obtenir { $count } épisodes
    }
get-show-go-replace =
    { $count ->
        [one] Obtenir { $count } épisode et remplacer
       *[other] Obtenir { $count } épisodes et remplacer
    }
get-show-go-subscribe =
    { $count ->
        [one] Obtenir { $count } épisode et s'abonner
       *[other] Obtenir { $count } épisodes et s'abonner
    }
get-show-go-replace-subscribe =
    { $count ->
        [one] Obtenir { $count } épisode, remplacer et s'abonner
       *[other] Obtenir { $count } épisodes, remplacer et s'abonner
    }
get-show-subscribe = S'abonner aux nouveaux épisodes
get-show-nothing-new = Rien de nouveau à obtenir
get-show-pick-episode = Choisissez un épisode
get-show-replace-off = Désactivé. Les choix pour { $episodes } sont exclus, car la bibliothèque les a. Activez-le pour les remplacer.
get-show-replace-on = Les fichiers que vous avez pour { $episodes } vont à la corbeille dès que les nouveaux sont terminés.
get-show-replace-on-the-way =
    { $count ->
        [one] { $episodes } est en téléchargement : ce téléchargement est annulé et ce qu'il a écrit va à la corbeille.
       *[other] { $episodes } sont en téléchargement : ce téléchargement est annulé et ce qu'il a écrit va à la corbeille.
    }
get-show-kind-on-demand = Démarrée quand une recherche en a besoin
get-show-kind-rss-search = Recherche RSS
get-show-kind-rss-feed = Flux RSS
get-show-kind-json-search = Recherche JSON
get-show-kind-indexer-search = Recherche d'indexeur
get-show-ready = Prête
get-show-running = En marche
get-show-not-running = Arrêtée
get-show-not-running-start = Arrêtée. Cliquez sur sa source ci-dessus pour la démarrer.
get-show-switched-off = Désactivée
get-show-click-to-see = Cliquez pour voir ce qu'elle a pour cette série
get-show-searching = Recherche
get-show-searching-source = Recherche sur { $name }
get-show-searching-elapsed = { $label }, { $seconds } s
get-show-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } releases
    }
get-show-releases-covered =
    { $count ->
        [one] { $count } release, { $cover } épisodes couverts sur { $total }
       *[other] { $count } releases, { $cover } épisodes couverts sur { $total }
    }
get-show-on-your-list = Dans votre liste
get-show-plan-to-watch = À voir
get-show-on-your-list-tip = Elle est dans votre liste
get-show-plan-to-watch-tip = L'ajouter à votre liste À voir
get-show-open-series-tip = La page de la série : ses détails, ses épisodes et la lecture
get-show-next-in = Épisode { $number } dans { $wait }
get-show-airing = En diffusion
get-show-finished = Terminée
get-show-total-episodes = { $count } épisodes
get-show-downloading-episodes =
    { $count ->
        [one] Téléchargement de { $count } épisode
       *[other] Téléchargement de { $count } épisodes
    }
get-show-downloading-torrents =
    { $count ->
        [one] Téléchargement de { $count } torrent
       *[other] Téléchargement de { $count } torrents
    }
get-show-in-library = Dans votre bibliothèque
get-show-not-in-library = Pas dans votre bibliothèque
get-show-downloading = Téléchargement
get-show-stop-all = Tout arrêter
get-show-start-all = Tout lancer
get-show-stop-all-tip = Arrêter chaque téléchargement de cette série ; ce qui est déjà là reste
get-show-start-all-tip = Relancer chaque téléchargement de cette série
get-show-cancel-all = Tout annuler
get-show-cancel-all-tip = Annuler chaque téléchargement de cette série et mettre ce qu'ils ont écrit à la corbeille
get-show-sources = Sources disponibles
get-show-check-again = Revérifier
get-show-check-again-tip = Demande à chaque source à la demande si elle tourne. Ne démarre rien.
get-show-search-source = Chercher sur { $name }
get-show-release-list = Toutes les releases en liste
get-show-your-library = Votre bibliothèque
get-show-library-on-disk = { $have } sur { $total } sur le disque
get-show-library-downloading = { $count } en téléchargement
get-show-library-nothing = Rien pour l'instant
get-show-row-batch = { $group }, lot
get-show-batch-counting = comptage de ses épisodes, un torrent
get-show-batch-unlisted = épisodes non listés, toute la série d'après son titre, un torrent
get-show-batch-episodes =
    { $count ->
        [one] { $count } épisode, un torrent
       *[other] { $count } épisodes, un torrent
    }
get-show-nothing-new-picked = Rien de nouveau de choisi
get-show-nothing-picked = Rien de choisi
get-show-pick-provider = Choisissez une source
get-show-plan-line = { $episodes } de { $groups } sur { $source }.
get-show-and = { $first } et { $second }
get-show-library-has =
    { $count ->
        [one] La bibliothèque a { $episodes }. Activez Remplacer les épisodes actuels ci-dessous pour le remplacer.
       *[other] La bibliothèque a { $episodes }. Activez Remplacer les épisodes actuels ci-dessous pour les remplacer.
    }
get-show-together = ensemble
get-show-torrents-unit =
    { $count ->
        [one] torrent
       *[other] torrents
    }
get-show-replace = Remplacer les épisodes actuels
get-show-sequential = Télécharger dans l'ordre
get-show-sequential-on = Activé : chaque nouveau torrent récupère ses morceaux depuis le début, donc les premiers épisodes finissent en premier et vous pouvez regarder pendant que le reste arrive. Un peu plus lent au total.
get-show-sequential-off = Désactivé : le client récupère les morceaux dans l'ordre où l'essaim les propose.
get-show-rolling = Garder les suivants prêts
get-show-rolling-on = Activé : { $count } épisodes se téléchargent maintenant, et chaque épisode terminé lance le suivant.
get-show-rolling-on-delete = Activé : { $count } épisodes se téléchargent maintenant, et chaque épisode terminé lance le suivant. Les épisodes vus quittent le disque.
get-show-rolling-off = Désactivé : chaque épisode choisi se télécharge tout de suite.
get-show-follow-off = Désactivé : les nouveaux épisodes s'affichent ici comme manquants, et rien ne se télécharge tout seul.
get-show-follow-no-feed = { $source } n'a pas de flux à suivre, donc les nouveaux épisodes ne peuvent pas s'y télécharger tout seuls.
get-show-follow-pick-group = Choisissez d'abord un groupe : un abonnement suit un groupe et une qualité.
get-show-follow-batch-only = { $group } n'a que le lot ici, donc il n'y a pas de flux d'épisodes seuls auquel s'abonner.
get-show-follow-on = Activé : un flux pour { $feed } sur { $source }. Chaque nouvel épisode se télécharge dès qu'il apparaît.
get-show-confirm =
    { $count ->
        [one] { $episodes } est en téléchargement. Le remplacer annule ce téléchargement et met ce qu'il a écrit à la corbeille.
       *[other] { $episodes } sont en téléchargement. Les remplacer annule ce téléchargement et met ce qu'il a écrit à la corbeille.
    }
get-show-confirm-go = Annuler et remplacer
get-show-state-error = Erreur
get-show-state-files-missing = Fichiers manquants
get-show-state-stopped = Arrêté
get-show-state-waiting-peers = En attente de pairs
get-show-state-metadata = Métadonnées
get-show-state-queued = En file
get-show-state-checking = Vérification
get-show-state-allocating = Allocation
get-show-state-moving = Déplacement
get-show-state-waiting = En attente
get-show-a-batch = Un lot
get-show-no-group = Groupe absent du nom
get-show-likely = sans doute { $source }
get-show-tracker = tracker { $tracker }
get-show-not-known = Pas encore connu
get-show-bytes-of = { $done } sur { $size }
get-show-speed-down = { $speed }/s en réception
get-show-time-left = { $wait } restantes
get-show-stopping = Arrêt
get-show-starting = Démarrage
get-show-stop = Arrêter
get-show-start-tip = Relancer le téléchargement
get-show-stop-tip = Arrêter le téléchargement ; ce qui est déjà là reste
get-show-cancelling = Annulation
get-show-cancel-tip = Annuler ce téléchargement et mettre ce qu'il a écrit à la corbeille
get-show-episode-order = Télécharger dans l'ordre des épisodes
get-show-episode-order-on = Activé : les morceaux viennent du début de chaque fichier, donc les premiers épisodes finissent en premier.
get-show-cube-skipped = Épisode { $number } : ignoré
get-show-cube-done = Épisode { $number } : terminé
get-show-cube-percent = Épisode { $number } : { $percent } %
get-show-fact-torrent = Torrent
get-show-fact-release = Release
get-show-fact-from = Depuis
get-show-fact-saving-to = Enregistre dans
get-show-span-counting = Comptage
get-show-span-unknown = { $runs } ?
get-show-span-episodes = Épisodes { $runs }
get-show-cube-no-release = Épisode { $number } : aucune release de { $group }
get-show-seeders = { $count } seeders
get-show-cube-on-disk = déjà sur le disque, cliquez pour le remplacer
get-show-cube-downloading = en téléchargement, cliquez pour le remplacer

## TorrentRow

torrent-row-state-downloading = Téléchargement
torrent-row-state-stalled = Bloqué
torrent-row-state-metadata = Métadonnées
torrent-row-state-queued = En file
torrent-row-state-stopped = Arrêté
torrent-row-state-done = Terminé
torrent-row-state-seeding = Partage
torrent-row-state-checking = Vérification
torrent-row-state-moving = Déplacement
torrent-row-state-error = Erreur
torrent-row-state-files-missing = Fichiers manquants
torrent-row-state-making-room = Allocation
torrent-row-state-finding-peers = Recherche de pairs
torrent-row-state-waiting = En attente
torrent-row-done-of-size = { $done } sur { $size }
torrent-row-speed-down = { $speed }/s en réception
torrent-row-speed-up = { $speed }/s en envoi
torrent-row-time-left = { $time } restantes
torrent-row-seeders =
    { $count ->
        [one] { $count } seeder
       *[other] { $count } seeders
    }
torrent-row-rolling = glissant, épisodes { $next } à { $through } sur { $total }
torrent-row-stop = Arrêter

## SourceRow

source-row-unavailable-fix = Branchez son disque, puis Réanalyser. S'il a été déplacé, retirez-le et ajoutez le nouvel emplacement.
source-row-counts = { $count } séries
source-row-counts-movies =
    { $movies ->
        [one] { $count } séries, { $movies } dossier de films
       *[other] { $count } séries, { $movies } dossiers de films
    }
source-row-unavailable = Indisponible
    .tip = AniBeam ne peut pas joindre ce dossier. S'il est sur un disque débranché ou non monté, connectez-le et AniBeam le reprendra. S'il a été déplacé, retirez-le ici et ajoutez le nouvel emplacement.
source-row-rescan = Réanalyser
source-row-remove-question = Retirer { $name }, ses { $count } séries et leur historique ?

## StartSourceSheet

start-source-title = Démarrer { $service } pour { $name } ?
start-source-busy = Démarrage de { $service }. { $name } s'active dès qu'il répond ; cela peut prendre jusqu'à une minute.
start-source-body = { $service } est le service qui cherche sur { $name } pour AniBeam. Le démarrer le fait tourner en arrière-plan sur cet ordinateur, et il continue après la fermeture d'AniBeam.
start-source-starting = Démarrage

## SearchProgress

search-progress-why-helper = AniBeam démarre son assistant de recherche ; cela prend une seconde ou deux.
search-progress-why-starting = { $endpoint } passe par un assistant qui démarre quand une recherche en a besoin. La première recherche l'attend, jusqu'à { $seconds } secondes ; les suivantes non.
search-progress-why-reading = Répartition des releases en groupes et comparaison avec la bibliothèque.
search-progress-why-ondemand = Un assistant relit { $endpoint } pour chaque nouvelle recherche, ce qui prend en général dix à vingt secondes.
search-progress-why-slow = { $endpoint } met du temps à répondre en ce moment.
search-progress-why-usual = Cela prend en général une seconde ou deux.
search-progress-why-and-slow = { $why } C'est plus long que d'habitude ; annulez pour essayer une autre source.
search-progress-searching = Recherche
search-progress-seconds = { $seconds } s
search-progress-try = Essayer { $endpoint }

## ReleaseTable

release-table-col-release = Release
release-table-col-group = Groupe
release-table-col-res = Rés.
release-table-col-rip = Rip
release-table-col-ep = Ép.
release-table-col-size = Taille
release-table-col-seeders = Seeders
release-table-col-leechers = Leechers
release-table-col-downloads = Téléch.
release-table-col-released = Sortie
release-table-tick-all = Cocher toutes les releases
release-table-untick-all = Décocher toutes les releases
release-table-on-disk = Sur le disque
release-table-other-on-disk = Autre release sur le disque
release-table-season-batch = Lot de la saison { $season }
release-table-batch = Lot
release-table-via = via { $source }
release-table-season-episode = S{ $season } { $episode }
release-table-date-format = d MMMM yyyy, HH:mm
release-table-download = Télécharger
release-table-more = et { $count } de plus. Affinez la liste ou triez-la pour voir les autres en premier.
release-table-pick = Choisir cette release

## DownloadsPage

downloads-title = Téléchargements
downloads-builtin-client = Client intégré
downloads-start-done = { $name } tourne et est activé
downloads-disk-film = Film
downloads-disk-watched = { $size } vus
downloads-disk-all-watched = tout vu
downloads-delete-watched-menu =
    { $count ->
        [one] Chaque épisode vu : { $count } épisode, { $size }
       *[other] Chaque épisode vu : { $count } épisodes, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
        [one] Chaque série terminée et vue : { $count } série, { $size }
       *[other] Chaque série terminée et vue : { $count } séries, { $size }
    }
downloads-titles-and = { $list } et { $last }
downloads-titles-more = { $list } et { $count } autres
downloads-delete-finished-none = Aucune série terminée n'a été vue jusqu'au bout
downloads-delete-watched-none = Rien de vu n'est sur le disque
downloads-delete-finished-heading =
    { $count ->
        [one] Supprimer { $count } série terminée, { $size } ?
       *[other] Supprimer { $count } séries terminées, { $size } ?
    }
downloads-delete-finished-base = { $titles }. Seulement les séries que la source dit terminées, chaque épisode vu ; une série encore en diffusion, ou dont la fin est une estimation, reste. Tous leurs fichiers vont à la corbeille, bonus compris. Leurs torrents quittent aussi { $client }, ils peuvent donc être retéléchargés.
downloads-delete-watched-heading =
    { $count ->
        [one] Supprimer { $count } épisode vu, { $size } ?
       *[other] Supprimer { $count } épisodes vus, { $size } ?
    }
downloads-delete-watched-base = Chaque épisode que vous avez vu, dans chaque série. Les épisodes pas encore vus et les bonus restent. Les fichiers et leurs sous-titres vont à la corbeille, vous pouvez donc les y récupérer. Leurs torrents quittent aussi { $client }, ils peuvent donc être retéléchargés.
downloads-move-to-trash = Mettre à la corbeille
downloads-remove-group-torrents =
    { $count ->
        [one] Retirer le torrent
       *[other] Retirer ses { $count } torrents
    }
downloads-run-span = { $from } à { $to }
downloads-run-pair = { $first } et { $second }
downloads-gap-missing =
    { $count ->
        [one] Épisode { $runs } manquant
       *[other] Épisodes { $runs } manquants
    }
downloads-gap-not-downloaded =
    { $count ->
        [one] Épisode { $runs } non téléchargé
       *[other] Épisodes { $runs } non téléchargés
    }
downloads-gap-then-not-downloaded = { $runs } non téléchargés
downloads-trying-query = Essai de { $query }
downloads-client-settings = Réglages du client torrent
downloads-error-unknown = Un problème est survenu. Réessayez.
downloads-search-all-off = Chaque source est désactivée. Activez-en une pour chercher.
downloads-turn-on = Activer { $name }
downloads-search-no-sources = Pas encore de source. Ajoutez une liste dans Réglages, sous Téléchargements.
downloads-search-none-started = Aucune source n'a pu lancer la recherche. Réessayez dans un instant.
downloads-auto-resolution = Limité à { $resolution }, la meilleure proposée
downloads-search-one-failed = La recherche sur { $name } n'a pas marché. Réessayez, ou cherchez sur une autre source.
downloads-search-all-failed = Aucune source n'a pu chercher. Réessayez dans un instant.
downloads-narrowing-on = Affinage sur { $name }
downloads-refine-zero = Cela ne laisserait rien, donc la liste reste telle quelle.
downloads-refine-no-match = Aucune release ne correspond à ce terme.
downloads-refine-empty = Saisissez un terme pour affiner.
downloads-refine-no-change = Cela ne changerait rien.
downloads-movie-tick-one = Cochez une release, puis appuyez sur Télécharger comme film.
downloads-group-downloading = Téléchargement
downloads-group-downloading-line = En cours ; Activité à droite donne le détail.
downloads-group-seeding = Partage
downloads-group-seeding-line = Téléchargés et encore partagés.
downloads-group-done = Terminés
downloads-group-done-line = Téléchargés ; le client ne les partage pas.
downloads-group-stopped = Arrêtés
downloads-group-stopped-line = Arrêtés avant la fin.
downloads-group-broken = Fichiers manquants
downloads-group-broken-line = Le client ne trouve pas leurs fichiers : ils pointent vers un dossier qui n'existe pas. Retirer les torrents fait le ménage dans le client.
downloads-sub-nofeed = Impossible à suivre
downloads-sub-nofeed-line = Leur source répond aux recherches mais n'a pas de flux, donc aucun nouvel épisode n'arrive. Désabonnez-vous, puis suivez la série sur une source qui en a un.
downloads-sub-complete = Complets
downloads-sub-complete-line = Chaque épisode est dans la bibliothèque et plus rien ne sera diffusé : ces abonnements n'ont plus rien à faire.
downloads-sub-airing = En diffusion
downloads-sub-airing-line = De nouveaux épisodes à venir, les plus proches d'abord.
downloads-sub-incomplete = Incomplets
downloads-sub-incomplete-line = Diffusion terminée avec des épisodes absents, ou rien n'indique où en est la série.
downloads-sub-missing = Pas dans la bibliothèque
downloads-sub-missing-line = Leur dossier ne contient rien que la bibliothèque ait trouvé : rien n'est encore arrivé, ou il enregistre là où la bibliothèque ne regarde pas.
downloads-torrents =
    { $count ->
        [one] { $count } torrent
       *[other] { $count } torrents
    }
downloads-rule-film-on-disk = Film, sur le disque
downloads-rule-all-here = Les { $total } sont là
downloads-rule-next =
    { $predicted ->
        [yes] Ép. { $episode } prévu dans { $time }
       *[no] Ép. { $episode } dans { $time }
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] Ép. { $episode } à { $through } prévus dans { $time }
       *[no] Ép. { $episode } à { $through } dans { $time }
    }
downloads-rule-have-of = { $have } sur { $total } ici
downloads-rule-have =
    { $count ->
        [one] { $count } épisode ici
       *[other] { $count } épisodes ici
    }
downloads-tag-not-downloaded = Non téléchargé
downloads-tag-missing = { $count } manquants
downloads-tag-n-not-downloaded = { $count } non téléchargés
downloads-tag-unknown = Inconnu
downloads-speed-down = { $speed }/s en réception
downloads-time-left = { $time } restantes
downloads-repair-trying = Essai de recherches plus courtes
downloads-sent-to-client = { $title } envoyé à { $client }
downloads-files-torrents = ce que les torrents ont téléchargé
downloads-files-torrents-remote = ce que les torrents ont téléchargé, supprimé par { $client }
downloads-files-folder-and = { $folder } et { $torrents }
downloads-unsubscribe-heading = Se désabonner de { $title } ?
downloads-unsubscribe-base = Son flux et sa règle de téléchargement quittent { $client }, donc plus rien de nouveau n'arrive.
downloads-unsubscribe-complete-heading = Se désabonner des { $count } séries complètes ?
downloads-unsubscribe-complete-base = Leurs flux et règles de téléchargement quittent { $client }. Chacun de leurs épisodes est dans la bibliothèque.
downloads-files-folders-and-torrents = leurs dossiers et ce que leurs torrents ont téléchargé
downloads-one-at-a-time = Ils partent un par un ; la page les montre au fur et à mesure.
downloads-subscription-stays = { $base } L'abonnement reste.
downloads-cancel-heading = Annuler { $title } ?
downloads-cancel-group-base =
    { $count ->
        [one] Son téléchargement s'arrête et quitte { $client }.
       *[other] Ses { $count } téléchargements s'arrêtent et quittent { $client }.
    }
downloads-files-so-far-it = ce qu'il a téléchargé jusqu'ici
downloads-files-so-far-they = ce qu'ils ont téléchargé jusqu'ici
downloads-cancel-download = Annuler le téléchargement
downloads-keep-downloading = Continuer
downloads-remove-heading = Retirer { $title } ?
downloads-remove-group-base =
    { $count ->
        [one] Son torrent quitte { $client } ; les fichiers restent sauf si vous cochez ci-dessous.
       *[other] Ses { $count } torrents quittent { $client } ; les fichiers restent sauf si vous cochez ci-dessous.
    }
downloads-remove-detail-base =
    { $count ->
        [one] Son torrent quitte { $client } ; les fichiers restent sauf si vous cochez ci-dessous.
       *[other] Ses { $count } torrents quittent { $client } ; les fichiers restent sauf si vous cochez ci-dessous.
    }
downloads-remove-broken-heading = Retirer les torrents des { $count } séries aux fichiers manquants ?
downloads-remove-broken-base = Leurs { $count } torrents quittent { $client }. Ils pointent vers des dossiers qui n'existent pas, donc aucun fichier n'est touché ; les abonnements restent.
downloads-remove-torrent-heading = Retirer ce torrent ?
downloads-files-it-downloaded = ce qu'il a téléchargé
downloads-cancel-torrent-heading = Annuler ce téléchargement ?
downloads-cancel-many-heading =
    { $count ->
        [one] Annuler { $count } téléchargement ?
       *[other] Annuler { $count } téléchargements ?
    }
downloads-cancel-many-base = Chaque téléchargement de cette série s'arrête et quitte { $client }.
downloads-cancel-all = Tout annuler
downloads-summary-connecting = Connexion.
downloads-summary-client-on = { $client } { $version } est connecté.
downloads-summary-client-off = { $client } n'est pas connecté. Vérifiez son adresse et son mot de passe dans Réglages, sous Téléchargements.
downloads-summary-native = AniBeam cherche lui-même dans vos sources.
downloads-summary-native-with = AniBeam cherche lui-même dans vos sources. { $client }
downloads-summary-starting = Démarrage des téléchargements.
downloads-summary-not-running = Les téléchargements ne tournent pas.
downloads-summary-not-started = Les téléchargements n'ont pas encore démarré.
downloads-summary-helper-ready = L'assistant de recherche { $version } est prêt. { $client }
downloads-subscribed-as = Abonné sous { $name }.
downloads-repaired =
    { $count ->
        [one] { $name } réparé : le flux trouve { $count } release
       *[other] { $name } réparé : le flux trouve { $count } releases
    }
downloads-repaired-keeps =
    { $count ->
        [one] { $name } réparé : le flux trouve { $count } release et la règle garde « { $term } »
       *[other] { $name } réparé : le flux trouve { $count } releases et la règle garde « { $term } »
    }
downloads-sent-torrents =
    { $count ->
        [one] { $count } torrent envoyé à { $client }.
       *[other] { $count } torrents envoyés à { $client }.
    }
downloads-added-torrents =
    { $count ->
        [one] { $count } torrent ajouté.
       *[other] { $count } torrents ajoutés.
    }
downloads-trashed-paths =
    { $count ->
        [one] { $count } chemin de { $title } mis à la corbeille
       *[other] { $count } chemins de { $title } mis à la corbeille
    }
downloads-show-gone = Cette série n'est plus dans la bibliothèque
downloads-activity-hide = Masquer l'activité
downloads-activity = Activité
downloads-moving =
    { $count ->
        [one] { $count } téléchargement actif
       *[other] { $count } téléchargements actifs
    }
downloads-back = Retour aux téléchargements
downloads-chip-search-ready = Recherche prête
downloads-chip-search-starting = Recherche en démarrage
downloads-chip-search-off = Recherche coupée
downloads-chip-answers-at = { $summary } Il répond à { $url }.
downloads-chip-helper-missing = L'assistant de recherche n'est pas installé. Cliquez pour les réglages Téléchargements, qui expliquent comment l'ajouter
downloads-chip-helper-failed = L'assistant de recherche s'est arrêté. Reconnecter le relance ; les réglages Téléchargements disent ce qui n'a pas marché
downloads-chip-open-settings = Ouvrir les réglages Téléchargements
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } coupé
downloads-chip-client-not-connected = { $client } n'est pas connecté. Reconnectez-le, ou vérifiez son adresse dans les réglages
downloads-chip-client-remote = Un assistant de recherche que vous faites tourner utilise le client indiqué dans ses propres réglages
downloads-chip-client-switch = Cliquez pour changer de client
downloads-connecting = Connexion
downloads-reconnect = Reconnecter
downloads-search-placeholder = Chercher une série, ou un nom de release
downloads-search-go-tip = Chercher des noms de releases sur { $names }
downloads-search-go-off = Chaque source est désactivée.
downloads-suggest-airing =
    { $predicted ->
        [yes] En diffusion, épisode { $episode } prévu dans { $time }
       *[no] En diffusion, épisode { $episode } dans { $time }
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] En diffusion, épisodes { $episode } à { $through } prévus dans { $time }
       *[no] En diffusion, épisodes { $episode } à { $through } dans { $time }
    }
downloads-suggest-finished = Terminée
downloads-suggest-episodes = { $count } épisodes
downloads-suggest-in-library = Dans votre bibliothèque
downloads-suggest-on-anilist = Sur AniList, pas dans votre bibliothèque
downloads-plan-to-watch = À voir
downloads-suggest-release-search = Chercher plutôt des noms de releases pour « { $query } » sur { $names }
downloads-key-return = Entrée
downloads-sources = Sources
downloads-source-hint-starting = démarrage
downloads-source-hint-down = arrêtée
downloads-source-tip-down = { $name } a besoin d'un service qui ne tourne pas. Cliquez pour le démarrer.
downloads-source-tip-on = Les recherches interrogent { $name }. Cliquez pour la désactiver.
downloads-source-tip-off = Désactivée. Cliquez pour interroger de nouveau { $name }.
downloads-source-tip-url = { $tip } { $url }
downloads-check = Vérifier
    .tip = Demande si chaque source à la demande tourne. Ne démarre rien.
downloads-check-busy = Vérification
downloads-back-to-grid = Retour à la grille
    .tip = Retour à { $title } et à ce que chaque source a trouvé. Échap fait pareil.
downloads-releases-on =
    { $count ->
        [one] { $count } release sur { $name }
       *[other] { $count } releases sur { $name }
    }
downloads-found-on = Trouvé sur
downloads-found-searching = recherche
downloads-found-failed = échec
downloads-found-cancelled = annulé
downloads-releases =
    { $count ->
        [one] { $count } release
       *[other] { $count } releases
    }
downloads-found-retry-tip =
    { $why }
    Cliquez pour réessayer
downloads-facet-groups = Groupe
downloads-facet-quality = Qualité
downloads-facet-words = Mots
downloads-filters-fewer = Moins de filtres
downloads-filters-more = Plus de filtres ({ $count })
downloads-narrow-placeholder = Affiner par un terme, ou -terme pour exclure
downloads-best-match = Meilleur choix
    .tip = Garde les releases qui ressemblent à la meilleure : { $query }
downloads-best-match-none = Aucune release ne se démarque ici
downloads-strip-have-of = { $have } épisodes sur { $total } sur le disque
downloads-strip-have =
    { $count ->
        [one] { $count } épisode sur le disque
       *[other] { $count } épisodes sur le disque
    }
downloads-strip-next =
    { $predicted ->
        [yes] épisode { $episode } prévu dans { $time }
       *[no] épisode { $episode } dans { $time }
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] épisodes { $episode } à { $through } prévus dans { $time }
       *[no] épisodes { $episode } à { $through } dans { $time }
    }
downloads-strip-complete = complet
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = Aucune de ces releases ne concerne un épisode que vous avez déjà.
downloads-strip-on-disk =
    { $count ->
        [one] { $count } release ici est le fichier que vous avez, marquée Sur le disque
       *[other] { $count } releases ici sont le fichier que vous avez, marquées Sur le disque
    }
downloads-strip-other-on-disk =
    { $count ->
        [one] { $count } release concerne un épisode que vous avez d'une autre release
       *[other] { $count } releases concernent un épisode que vous avez d'une autre release
    }
downloads-strip-both = { $first } ; { $second }.
downloads-tick-missing =
    { $count ->
        [one] Cocher l'épisode manquant
       *[other] Cocher les { $count } épisodes manquants
    }
    .tip = Pour chaque épisode ici absent de la bibliothèque, la release qui a le plus de seeders
downloads-open-in-library = Ouvrir dans la bibliothèque
downloads-not-this-show = Pas cette série
downloads-no-releases = Aucune release
downloads-no-releases-body = Rien sur { $name } ne correspond à « { $query } ». Une autre source, ou moins de mots, peut la trouver.
downloads-try-source = Essayer { $name }
downloads-ticked =
    { $count ->
        [one] { $count } release cochée
       *[other] { $count } releases cochées
    }
downloads-nothing-ticked = Rien de coché
downloads-ticked-size = { $size } au total.
downloads-tick-hint = Cochez des releases dans le tableau, ou téléchargez-les toutes.
downloads-tick-all = Tout cocher
downloads-working = En cours
downloads-download-ticked =
    { $count ->
        [one] Télécharger { $count } release
       *[other] Télécharger { $count } releases
    }
downloads-download-shown = Télécharger les { $count } affichées
downloads-download-all = Télécharger les { $count }
downloads-download-movie = Télécharger comme film
    .tip = Une release cochée, directement dans le dossier des films
downloads-subscribe = S'abonner
downloads-subscribe-body = Les nouvelles releases qui correspondent à cette recherche se téléchargent dès leur sortie, dans le dossier indiqué ici.
downloads-name-placeholder = Repris de la première release
downloads-subscribe-search = S'abonner à cette recherche
downloads-subscribe-no-feed = Aucun flux auquel s'abonner ici
downloads-preview-again = Revoir la règle
downloads-preview-show = Voir la règle créée
downloads-preview-from = Depuis { $host }
downloads-preview-from-named = Depuis { $host } ({ $name })
downloads-preview-must = Doit contenir { $terms }
downloads-preview-must-not = Ne doit pas contenir { $terms }
downloads-helper-changed = Téléchargements pointe vers un assistant qui ne répond pas
downloads-helper-missing = Cette copie d'AniBeam est livrée sans téléchargements
downloads-helper-failed = Les téléchargements n'ont pas pu démarrer
downloads-helper-changed-body = Recherches et téléchargements passent par un petit programme assistant. Les réglages en indiquent un autre que celui d'AniBeam, et il ne répond pas. Revenez à celui d'AniBeam et il démarre tout seul.
downloads-helper-missing-body = Elle a été compilée sans le programme assistant par lequel passent recherches et téléchargements. Les installeurs de la page des versions l'incluent. Réglages, Téléchargements explique comment l'ajouter à cette copie.
downloads-helper-failed-body = Le programme assistant des recherches et téléchargements s'est arrêté avant d'être prêt. Le relancer règle le problème la plupart du temps.
downloads-helper-use-own = Utiliser celui d'AniBeam
downloads-helper-start-again = Relancer
downloads-helper-show-how = Montrez-moi
downloads-helper-what-went-wrong = Ce qui n'a pas marché
downloads-lens-gaps-count = Manques dans votre bibliothèque  { $count }
downloads-lens-airing-count = En diffusion  { $count }
downloads-lens-following-count = Suivies  { $count }
downloads-lens-downloaded-count = Téléchargées  { $count }
downloads-lens-gaps = Manques dans votre bibliothèque
downloads-lens-gaps-line = Épisodes diffusés absents du disque. Ouvrez une série pour voir qui les a.
downloads-lens-gaps-none = Rien ne manque : chaque épisode diffusé de chaque série est sur le disque.
downloads-lens-airing-line = Séries de votre bibliothèque avec des épisodes à venir. Ouvrez-en une pour la suivre.
downloads-lens-airing-none = Rien dans votre bibliothèque n'est en diffusion
downloads-lens-airing-none-body = Une série arrive ici dès qu'elle est dans votre bibliothèque et a des épisodes à venir. En diffusion liste tout ce qui passe.
downloads-airing-now = En diffusion
downloads-lens-following = Suivies
downloads-card-next =
    { $predicted ->
        [yes] Épisode { $episode } prévu dans { $time }
       *[no] Épisode { $episode } dans { $time }
    }
downloads-card-next-span =
    { $predicted ->
        [yes] Épisodes { $episode } à { $through } prévus dans { $time }
       *[no] Épisodes { $episode } à { $through } dans { $time }
    }
downloads-rules-unread = Impossible de lire vos abonnements.
downloads-lens-following-line = Les flux qu'AniBeam surveille, présentés par les séries qu'ils alimentent.
downloads-none-yet = Aucun pour l'instant.
downloads-reading = Lecture.
downloads-rules-error = AniBeam n'a pas pu lire vos abonnements. Réessayez dans un instant.
downloads-removing-to-go = Retrait, encore { $count }
downloads-unsubscribe-all = Se désabonner des { $count }
downloads-unsubscribe = Se désabonner
downloads-no-rules = Aucun abonnement
downloads-no-rules-body = Cherchez ci-dessus et abonnez-vous pour recevoir une série au fil de l'eau.
downloads-search = Chercher
downloads-lens-downloaded = Téléchargées
downloads-dl-line = { $size } sur le disque. { $rest }
downloads-dl-client-off = { $client } n'est pas connecté.
downloads-dl-unread = Impossible de lire la liste des téléchargements.
downloads-dl-in-client =
    { $count ->
        [one] { $count } torrent, { $size } dans { $client }.
       *[other] { $count } torrents, { $size } dans { $client }.
    }
downloads-usage-watched = Vu
downloads-usage-not-watched = Non vu
downloads-deleting = Suppression
downloads-delete-watched = Supprimer les vus
downloads-remove-all = Retirer les { $count }
downloads-on-disk = Sur le disque
downloads-on-disk-line = Dans votre bibliothèque sans torrent dans { $client }, les plus gros d'abord. Clic droit sur l'un pour le supprimer.
downloads-nothing-downloaded = Rien de téléchargé
downloads-nothing-downloaded-body = Ce que vous téléchargez ou suivez s'affiche ici, série par série.
downloads-client-off-title = { $client } n'est pas connecté
downloads-client-off-body = Vérifiez qu'il tourne, puis reconnectez-le. Son adresse et son mot de passe sont dans Réglages, sous Téléchargements.
downloads-connect-again = Reconnecter
downloads-view-grid = Grille
downloads-view-list = Liste
downloads-chip-subscribed = Abonné
downloads-speed-up = { $speed }/s en envoi
downloads-broken-some = { $count } aux fichiers manquants
downloads-tag-watched = Vu

## PlayerDelays.qml

player-delays-title = Décalage
player-delays-subtitles = Sous-titres
player-delays-audio = Audio
player-delays-subtitles-earlier = Sous-titres 0,1 s plus tôt
player-delays-subtitles-later = Sous-titres 0,1 s plus tard
player-delays-audio-earlier = Audio 0,1 s plus tôt
player-delays-audio-later = Audio 0,1 s plus tard
player-delays-reset = Remettre à 0 jusqu'au fichier suivant
player-delays-none = Aucun décalage

## TrackPicker.qml

track-picker-off = Aucun

## RollControls.qml

roll-controls-window = { $ahead } prêts d'avance : épisodes { $next } à { $through } sur { $total }
roll-controls-window-watched = { $ahead } prêts d'avance : épisodes { $next } à { $through } sur { $total }, { $watched } vus
roll-controls-more = { $count } de plus
roll-controls-more-tip = Télécharger les épisodes suivants maintenant. AniBeam garde de nouveau les suivants prêts une fois que vous avez rattrapé.
roll-controls-fewer-tip = En garder un de moins d'avance
roll-controls-ahead = { $count } d'avance
roll-controls-further-tip = En garder un de plus d'avance
roll-controls-all = Tout obtenir
roll-controls-all-tip = Arrêter d'en garder quelques-uns prêts et télécharger tous les épisodes restants
roll-controls-all-question = Télécharger les { $count } épisodes restants ?

## PlayerChrome.qml

player-chrome-delays = Décalage des sous-titres et de l'audio
player-chrome-audio = Piste audio
player-chrome-subtitles = Sous-titres
player-chrome-subtitles-off = Sous-titres désactivés
player-chrome-casting-to = Diffusion vers { $device }
player-chrome-cast = Diffuser sur une TV
player-chrome-mark-watched = Marquer comme vu
player-chrome-shortcuts = Raccourcis clavier
player-chrome-fullscreen = Plein écran
player-chrome-exit-fullscreen = Quitter le plein écran
player-chrome-stay = Rester
player-chrome-skip-intro = Passer l'intro
player-chrome-skip-outro = Passer l'outro
player-chrome-playing-on = Lecture sur { $device }
player-chrome-replay = Revoir

## PlayerPage.qml

player-title = Lecteur
player-next = Suivant
player-extra-no-next = Un bonus n'a pas d'épisode suivant
player-last-episode = C'est le dernier épisode
player-extra-no-previous = Un bonus n'a pas d'épisode précédent
player-first-episode = C'est le premier épisode
player-resumed-at = Reprise à { $time }
player-start-over = Reprendre au début
player-audio-track = Audio : { $track }
player-subtitles-track = Sous-titres : { $track }
player-subtitles-on = Sous-titres : activés
player-subtitles-off = Sous-titres désactivés
player-no-subtitles = Ce fichier n'a pas de sous-titres
player-delay-seconds = { $value } s
player-audio-delay = Décalage audio { $value }
player-subtitle-delay = Décalage des sous-titres { $value }
player-volume = Volume { $volume }
player-speed = Vitesse { $speed }×
player-not-while-casting = Pas pendant la diffusion
player-hold-fast = 2× en maintenant
player-no-chapters = Ce fichier n'a pas de chapitres
player-last-chapter = C'est le dernier chapitre
player-chapter = Chapitre { $number }
player-chapter-titled = Chapitre { $number } : { $title }
player-screenshot-saved = Enregistrée dans Images/AniBeam
player-skipped-intro = Intro passée
player-skipped-outro = Outro passée
player-undo = Annuler
player-refusal-hidden = la série est masquée
player-refusal-no-match = la série n'est pas encore associée ; associez-la dans Infos de la série
player-refusal-not-newer = votre liste a déjà dépassé cet épisode
player-refusal-extra = c'est un bonus
player-refusal-unmatched = le fichier n'est associé à aucun épisode
player-refusal-on-disk = le fichier n'existe que sur le disque
player-not-tracked-because = Non suivi : { $reason }.
player-outcome-said = { $tracker } : { $words }
player-outcome-failed = { $tracker } n'a pas pu être mis à jour
player-outcome-ok = { $tracker } ok
player-outcome-at = { $tracker } à { $progress }
player-outcome-tracked = Suivi
player-outcome-not-tracked = Non suivi
player-outcome-could-not-update = Impossible de mettre à jour votre liste
player-outcome-tracked-list = Suivi : { $list }
player-outcome-not-tracked-list = Non suivi : { $list }
player-outcome-failed-list = Impossible de mettre à jour votre liste : { $list }
player-rated = Noté { $score }
player-score-failed = Impossible d'enregistrer la note
player-frame = Image { $number } à { $time }
player-audio-title = Audio
player-subtitles-title = Sous-titres
player-cast-to = Diffuser vers
player-cast-looking = Recherche de Chromecast et de TV
player-cast-unreachable = Impossible de joindre la TV
player-cast-tv = TV
player-cast-stop = Arrêter la diffusion vers { $device }
player-cast-connecting = Connexion
player-cast-loading = Chargement
player-cast-playing = Lecture
player-cast-paused = En pause
player-cast-finished = Terminé
player-cast-stopped = Arrêté
player-rate-prompt = Dernier épisode suivi. Noter cette série ?
player-rate = Noter
player-not-now = Plus tard

## EndpointReposPanel.qml

endpoint-repos-panel-title = Listes de sources
endpoint-repos-panel-helper = Une liste de sources est un ensemble signé de sites, ajouté par son adresse. AniBeam vérifie la signature et montre la clé avant d'enregistrer ; installez ensuite les sources voulues, qui rejoignent Sources de recherche ci-dessus.
endpoint-repos-panel-could-not-start = AniBeam n'a pas pu le lancer. Réessayez dans un instant.
endpoint-repos-panel-needs-value = { $name } a besoin d'une valeur
endpoint-repos-panel-values-of = Valeurs de { $name }
endpoint-repos-panel-secret-note = Gardé dans le coffre secret de cette machine, jamais dans la bibliothèque ni dans un export.
endpoint-repos-panel-secret-note-keep = Gardé dans le coffre secret de cette machine, jamais dans la bibliothèque ni dans un export. Un champ laissé vide garde la valeur enregistrée.
endpoint-repos-panel-asks-again = { $name } version { $version } redemande
endpoint-repos-panel-update-sends-to = La nouvelle version l'envoie à { $host }.
endpoint-repos-panel-fields-changed = Ses champs ont changé.
endpoint-repos-panel-needs-fields = Il faut renseigner { $fields } avant de chercher.
endpoint-repos-panel-no-longer-offered = N'est plus proposée par sa liste de sources.
endpoint-repos-panel-update-requests-to = La mise à jour envoie des requêtes à { $host }.
endpoint-repos-panel-unreadable = Cette liste est illisible. Actualiser les listes la réessaie.
endpoint-repos-panel-add-list = Ajouter une liste de sources
endpoint-repos-panel-add-list-helper = L'adresse de son fichier d'index, ou son chemin local.
endpoint-repos-panel-checking = Vérification
endpoint-repos-panel-add = Ajouter
endpoint-repos-panel-trust-question = Faire confiance à { $name } ?
endpoint-repos-panel-fingerprint = Empreinte de la clé  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    { $count } sources, signées. Ses sources décident où vont vos recherches ; ne faites confiance qu'à une liste venant de quelqu'un de confiance.
endpoint-repos-panel-preview-trusted =
    { $url }
    { $count } sources, signées, déjà approuvées. Ses sources décident où vont vos recherches ; ne faites confiance qu'à une liste venant de quelqu'un de confiance.
endpoint-repos-panel-trust = Faire confiance
endpoint-repos-panel-saving = Enregistrement
endpoint-repos-panel-install = Installer
endpoint-repos-panel-update = Mettre à jour
endpoint-repos-panel-added-by-you = ajoutée par vous
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = Passer à la v{ $version }
endpoint-repos-panel-add-value = Ajouter sa valeur
endpoint-repos-panel-change = Modifier { $fields }
endpoint-repos-panel-asks-for = Demande votre { $fields }.
endpoint-repos-panel-by-hand = En ajouter une à la main
endpoint-repos-panel-by-hand-helper = Pour un site absent des listes : tout flux avec {"{"}query{"}"} à la place de la recherche, proxy de recherche compris. Une clé dans l'adresse est gardée à part, dans le trousseau du système s'il existe.
endpoint-repos-panel-name = Nom
endpoint-repos-panel-refreshing = Actualisation
endpoint-repos-panel-refresh = Actualiser les listes

## ScheduleSheet.qml

schedule-episodes-range = Épisodes { $first } à { $last }
schedule-day-format = ddd d MMM yyyy
schedule-month-format = MMMM yyyy
schedule-tile-day-format = d MMM
schedule-tile-month-format = MMM yyyy
schedule-some-time-in = Courant { $year }
schedule-no-date-yet = Pas encore de date
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = Sans date
schedule-tile-last = Dernier
schedule-tile-out = Sorti
schedule-ends-at = Finit à l'épisode { $number }
schedule-dated-by-site = { $count } datés par { $site }
schedule-dated-by-record = { $count } datés par la fiche
schedule-set-by-you = { $count } fixés par vous
schedule-guessed-weekly = { $count } estimés à la semaine
schedule-nothing-set = Rien de fixé pour l'instant
schedule-saved = Enregistré
schedule-cleared = Notes de planning effacées
schedule-title = Épisodes et dates
schedule-season = Saison { $number }
schedule-ask-agent = Demander à { $agent }
schedule-done-pasting = Collage terminé
schedule-paste-list = Coller une liste
schedule-agent-looking = { $agent } cherche les dates. Rien n'est enregistré avant que vous appuyiez sur Enregistrer.
schedule-agent-stopped = { $agent } s'est arrêté : { $error }
schedule-stop = Arrêter
schedule-paste-placeholder = Collez une liste de sorties, un tableau ou un message
schedule-paste-helper = Ce qu'AniBeam lit s'affiche sur les épisodes ci-dessous.
schedule-agent-hint = Avec Claude Code sur cet ordinateur, AniBeam peut lui demander de chercher les dates pour vous.
schedule-show-more = Plus d'épisodes
schedule-pick-hint = Cliquez sur un épisode pour le dater. Maj-clic choisit une suite.
schedule-picked-one = Épisode { $number } :
schedule-picked = { $count ->
    [one] { $count } épisode :
   *[other] { $count } épisodes :
}
schedule-date-placeholder = 7 janv., mars 2027, 2027
schedule-set-date = Dater
schedule-weekly = Hebdo
schedule-not-last = Pas le dernier
schedule-last-episode = Dernier épisode
schedule-clear-date = Effacer la date

## ShowPage.qml

show-title = Série
show-runtime-each = { $runtime } chacun
show-add-to-list = Ajouter à une liste
show-moving-to = Déplacement vers { $list }
show-moved-to = Déplacé vers { $list }
show-relation-related = Liée
show-no-poster = Pas d'affiche
show-reading-anilist = Lecture d'AniList
show-not-in-library = Pas dans votre bibliothèque
show-get-episodes = Obtenir des épisodes
show-trailer = Bande-annonce
show-your-score = Votre note { $score }
show-rate-it = Noter
show-anilist-score = AniList { $score }
show-forecast = { $percent } % pour vous
show-because-you-watched = Parce que vous avez vu
show-hidden-loved = { $count ->
    [one] Un tag caché que vous adorez l'explique
   *[other] { $count } tags cachés que vous adorez l'expliquent
}
show-you-gave-it = Vous lui avez donné { $score }
show-story = Histoire
show-genres-tags = Genres et tags
show-studios = Studios  { $studios }
show-related = Liées
show-recommendations = Recommandations
show-from-anilist = Depuis AniList
show-reading-rest = Lecture du reste depuis AniList

## ConnectSheet.qml

connect-connected-to = Connecté à { $name }
connect-connect-to = Connecter { $name }
connect-did-not-finish = La connexion n'a pas abouti
connect-finish-in-browser = Terminez dans votre navigateur
connect-signed-in-anilist = Connecté en tant que { $account }. Votre liste est sur la page En cours.
connect-signed-in-mal = Connecté en tant que { $account }. Les épisodes y sont marqués pendant que vous regardez.
connect-setup-lead = { $name } connecte AniBeam par une appli à vous. Cela prend une minute environ, une seule fois.
connect-page-open = La page de connexion de { $name } est ouverte dans votre navigateur. Approuvez AniBeam là-bas et ceci se fermera tout seul.
connect-page-opening = Ouverture de la page de connexion de { $name } dans votre navigateur.
connect-open-watching = Ouvrir En cours
connect-done = Terminé
connect-open-again = Rouvrir la page de connexion
connect-secret-where = C'est la longue chaîne sous « Client Secret » sur la page de l'appli, sur myanimelist.net/apiconfig.
connect-client-secret = Secret client
connect-save-try-again = Enregistrer et réessayer
connect-change-client-id = Changer le Client ID
connect-step-create-anilist = Créez le client sur AniList
connect-step-create-mal = Créez l'appli sur MyAnimeList
connect-step-create-anilist-body = Ouvrez les réglages développeur d'AniList, créez un nouveau client, et collez cette adresse dans son champ Redirect URL exactement comme ici.
connect-step-create-mal-body = Ouvrez la page API de MyAnimeList, créez un ID avec App Type « other », et collez cette adresse dans son champ App Redirect URL exactement comme ici.
connect-open-developer-page = Ouvrir la page développeur
connect-copy-address = Copier l'adresse
connect-step-paste-id = Collez l'ID de l'appli
connect-step-paste-id-anilist = AniList l'appelle Client ID : le nombre sur la page de votre nouveau client.
connect-step-paste-id-mal = MyAnimeList l'appelle Client ID : la longue chaîne sur la page de votre nouvelle appli.
connect-client-id = Client ID
connect-client-secret-web = Secret client (seulement pour les applis Web)
connect-step-connect-body = Le navigateur s'ouvre, { $name } vous demande de vous connecter et d'approuver AniBeam, et AniBeam récupère la réponse.

## SettingsPage

settings-title = Réglages
settings-search-placeholder = Chercher un réglage
settings-section-library = Bibliothèque
settings-section-accounts = Comptes
settings-section-appearance = Apparence
settings-section-playback = Lecture
settings-section-subtitles = Sous-titres
settings-section-keys = Clavier
settings-section-downloads = Téléchargements
settings-section-phone = Appli mobile
settings-section-storage = Stockage et sauvegarde
settings-section-help = Aide
settings-section-about = À propos
settings-section-developer = Développeur
settings-status-library-empty = Pas encore de dossier
settings-status-library =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    } dans { $folders ->
        [one] { $folders } dossier
       *[other] { $folders } dossiers
    }
settings-status-library-missing =
    { $gone ->
        [one] { $gone } dossier introuvable
       *[other] { $gone } dossiers introuvables
    }, { $count ->
        [one] { $count } série
       *[other] { $count } séries
    } dans { $folders ->
        [one] { $folders } dossier
       *[other] { $folders } dossiers
    }
settings-status-accounts-anilist = Synchro avec AniList
settings-status-accounts-mal = Synchro avec MyAnimeList
settings-status-accounts-both = Synchro avec AniList et MyAnimeList
settings-status-accounts-none = Aucune liste connectée
settings-status-appearance =
    { $mode ->
        [system] Suit le système
        [light] Clair
       *[dark] Sombre
    }, { $source ->
        [theme] couleurs du thème
       *[desktop] couleurs du bureau
    }
settings-status-playback =
    { $skip ->
        [both] Passe intros et outros
        [intro] Passe les intros
        [outro] Passe les outros
       *[none] Lit tout du début à la fin
    }
settings-status-playback-audio =
    { $skip ->
        [both] Passe intros et outros
        [intro] Passe les intros
        [outro] Passe les outros
       *[none] Lit tout du début à la fin
    }, audio { $languages }
settings-status-subtitles =
    Taille { $percent } pour cent, { $override ->
        [AsScripted] fichiers stylés tels quels
        [Force] votre style sur chaque fichier
       *[ScaleOnly] les fichiers stylés gardent leur look
    }
settings-status-keys = Tous les raccourcis clavier
settings-status-downloads-unread = Pas encore lu
settings-status-downloads-answered = qBittorrent a répondu
settings-status-downloads-missing = Assistant de recherche non installé
settings-status-downloads-unconnected = Pas encore connecté
settings-status-phone-shared = Partagé sur votre tailnet
settings-status-phone-off = Non partagé
settings-status-storage = { $size } d'images en cache
settings-status-storage-unread = Images, export et import
settings-status-help = Questions fréquentes
settings-status-about-update = La version { $version } est sortie
settings-status-developer = Chronométrage de chaque page
settings-status-developer-last = { $page } construite en { $ms } ms
settings-results =
    { $count ->
        [one] { $count } réglage correspond
       *[other] { $count } réglages correspondent
    }
settings-results-none = Aucun résultat
settings-results-hint = Entrée ouvre le premier.
settings-results-none-hint = Essayez un mot plus simple : thème, sous-titres, suivi, mot de passe, dossier.

settings-find-folders =
    .words = ajouter dossier analyser réanalyser retirer source chemin collection
settings-find-matching =
    .words = métadonnées infos série source anilist tvmaze ordre association associer
settings-find-hidden-shows =
    .words = masquées masqué afficher révéler
settings-find-hidden-airing =
    .words = sans intérêt diffusion réafficher
settings-find-calendar =
    .words = calendrier ics ical diffusion planning abonner
settings-find-anime-lists =
    .words = anilist myanimelist mal connexion connecter identifiant compte déconnecter suivi
settings-find-main-list =
    .words = progression cartes principal suivi
settings-find-sync-now =
    .words = actualiser synchro synchroniser progression listes suivi
settings-find-on-hold =
    .words = en attente pause inactif inactive jours
settings-find-language =
    .words = langue traduction locale français anglais
settings-find-mode =
    .words = sombre clair système thème nuit mode
settings-find-colours-from =
    .words = terminal bureau système thème source kitty couleurs
settings-find-dark-theme =
    .words = couleur palette base16 sombre
settings-find-light-theme =
    .words = couleur palette base16 clair
settings-find-accent =
    .words = couleur surbrillance accent
settings-find-text-size =
    .words = police caractères mots plus grand plus petit zoom échelle lire accessibilité taille texte
settings-find-density =
    .words = compact confortable espacement taille densité
settings-find-poster-size =
    .words = cartes petit moyen grand affiches
settings-find-out-now =
    .words = diffusion sortis cartes liste lignes style bannière affiches
settings-find-out-now-cards =
    .words = diffusion sortis grille colonnes lignes disposition
settings-find-reduce-motion =
    .words = animation accessibilité mouvement
settings-find-app-icon =
    .words = logo lanceur dock barre des tâches fenêtre sombre icône
settings-find-skip-intros =
    .words = opening op générique auto passer aniskip chapitres intro
settings-find-skip-outros =
    .words = ending ed générique fin auto passer aniskip outro
settings-find-mpv-settings =
    .words = mpv mpv.conf configuration avancé
settings-find-discord =
    .words = discord rich presence statut profil
settings-find-next-episode =
    .words = suivant bouton titre lecture automatique fin
settings-find-subtitle-languages =
    .words = langue piste sous-titres
settings-find-audio-languages =
    .words = langue piste doublage vf japonais audio
settings-find-start-over =
    .words = réinitialiser défaut style sous-titres
settings-find-size =
    .words = échelle taille sous-titres
settings-find-position =
    .words = hauteur marge position sous-titres
settings-find-styled =
    .words = ass stylés forcer remplacer
settings-find-font =
    .words = police caractères
settings-find-text-colour =
    .words = couleur texte
settings-find-outline =
    .words = contour bordure trait
settings-find-background =
    .words = fond boîte opacité
settings-find-bold =
    .words = gras graisse
settings-find-keys-player =
    .words = clavier raccourcis touches lecteur
settings-find-keys-player-around =
    .words = clavier raccourcis plein écran suivant précédent épisode retour
settings-find-keys-around =
    .words = clavier raccourcis retour avancer clic droit menu plein écran
settings-find-keys-anywhere =
    .words = clavier raccourcis touches réglages quitter
settings-find-torrent-client =
    .words = qbittorrent mot de passe utilisateur adresse webui seedbox client torrent
settings-find-search-helper =
    .words = anirss rss flux jeton commande serveur avancé assistant
settings-find-next-few =
    .words = glissant épisodes avance supprimer vus garder disque espace
settings-find-unsubscribe =
    .words = désabonner flux terminées nettoyer
settings-find-last-search =
    .words = téléchargements recherche mémoriser dernière
settings-find-search-sources =
    .words = sources endpoints flux configuration anirss
settings-find-source-lists =
    .words = dépôt dépôts repo endpoint signé sources jackett prowlarr
settings-find-share-tailscale =
    .words = téléphone mobile android tailnet distant hôte tailscale partager
settings-find-stream =
    .words = diffusion streaming lire caster épisodes téléphone fichiers
settings-find-devices =
    .words = confiance approuvé désactiver bloquer appareil tailscale
settings-find-back-up =
    .words = exporter sauvegarde fichier json
settings-find-restore =
    .words = importer restaurer fichier json
settings-find-image-cache =
    .words = affiches effacer images disque cache
settings-find-files =
    .words = base de données données configuration cache dossier chemin fichiers
settings-find-help-folders =
    .words = aide dossier organisation nommage épisodes saisons films fichiers structure
settings-find-help-recognised =
    .words = aide association mauvaise série introuvable non associée reconnue
settings-find-help-anilist =
    .words = aide anilist myanimelist mal connexion connecter identifiant suivi
settings-find-help-downloads =
    .words = aide téléchargements vide sources rien torrent qbittorrent
settings-find-help-data =
    .words = aide données fichiers sauvegarde base de données vie privée
settings-find-help-else =
    .words = aide configuration premier lancement visite signaler bug problème ticket
settings-find-version =
    .words = build version nouveautés changelog
settings-find-setup =
    .words = premier lancement visite configuration relancer recommencer
settings-find-updates =
    .words = stable nightly canal vérifier mise à jour
settings-find-developer-options =
    .words = debug chronométrage développeur
settings-find-ratings =
    .words = imdb crédit notes

## SettingsDownloadsTab

settings-downloads-title = Téléchargements
settings-downloads-lead = Où AniBeam envoie ce que vous téléchargez, et comment il cherche dans les flux de releases.
settings-downloads-keep-none = Terminer un épisode supprime le précédent.
settings-downloads-keep-line = Épisodes vus conservés avant le dernier fini. Avec { $count }, finir l'épisode { $finishing } supprime l'épisode 1.
settings-downloads-anirss-ready = En marche, version { $version } sur { $url }.
settings-downloads-anirss-starting = Démarrage de l'assistant de recherche.
settings-downloads-anirss-missing = L'assistant de recherche n'est pas installé. Réinstallez AniBeam, ou indiquez votre propre anirss sous Commande.
settings-downloads-anirss-failed = L'assistant de recherche s'est arrêté. Réessayer le relance ; Détails montre ce qu'il a dit.
settings-downloads-anirss-idle = Pas encore démarré. Il démarre à votre première recherche sur la page Téléchargements.
settings-downloads-torrent-trying = Essai de { $client }.
settings-downloads-torrent-answered = { $client } { $version } a répondu.
settings-downloads-torrent-failed = { $client } injoignable. Vérifiez l'adresse et le mot de passe, puis réessayez.
settings-downloads-torrent-untried = Pas encore essayé.
settings-downloads-mode-existing = Un qBittorrent que vous faites tourner, avec son interface Web activée.
settings-downloads-mode-managed = AniBeam fait tourner son propre qBittorrent dans ~/Downloads/Anime. Il continue après la fermeture d'AniBeam, donc les abonnements continuent de télécharger.
settings-downloads-mode-auto = Utilise un qBittorrent qui répond à l'adresse ci-dessous, sinon en lance un à lui.
settings-downloads-password-saved = Un mot de passe est enregistré ici pour ce client.
settings-downloads-password-anirss = Utilise le mot de passe enregistré par l'assistant de recherche ; enregistrez-en un ici pour ne plus en dépendre.
settings-downloads-password-none = Aucun mot de passe enregistré, inutile si le client exempte cette machine.
settings-downloads-mismatch = L'assistant de recherche parle à { $helperUrl } alors qu'AniBeam parle à { $url }. Un assistant que vous faites tourner lit l'adresse dans son propre config.toml, donc changer de client ici ne le déplace pas.
settings-downloads-torrent-title = Client torrent
settings-downloads-torrent-helper = Le qBittorrent qui télécharge. Son mot de passe est rangé dans le trousseau du système.
settings-downloads-which-client = Quel client
settings-downloads-pick-for-me = Auto
settings-downloads-my-own = Le mien
settings-downloads-built-in = Intégré
settings-downloads-saved-clients = Clients enregistrés
settings-downloads-saved-clients-helper = Une seedbox et celui de cette machine, par exemple. La puce du client sur la page Téléchargements passe aussi de l'un à l'autre.
settings-downloads-add-client = Ajouter
settings-downloads-name = Nom
settings-downloads-name-placeholder = Nommé d'après son adresse
settings-downloads-address = Adresse
settings-downloads-address-managed-helper = Seule cette machine peut le joindre, et il n'a pas besoin de mot de passe.
settings-downloads-address-helper = L'adresse de l'interface Web.
settings-downloads-username = Utilisateur
settings-downloads-password = Mot de passe
settings-downloads-secret-saved = Enregistré
settings-downloads-secret-unset = Non défini
settings-downloads-connection = Connexion
settings-downloads-open-config = Ouvrir sa config
settings-downloads-details = Détails
settings-downloads-details-torrent = { $client } a dit ceci quand AniBeam l'a essayé.
settings-downloads-test-trying = Essai
settings-downloads-test-save = Enregistrer et tester
settings-downloads-test = Tester
settings-downloads-searching = Recherche
settings-downloads-search-helper = Assistant de recherche
settings-downloads-searching-helper = AniBeam cherche lui-même dans vos sources et envoie vos choix au client torrent, sans assistant de recherche.
settings-downloads-search-helper-helper = La partie d'AniBeam qui cherche et s'abonne, un programme appelé anirss. AniBeam le lance pour vous ; ne changez ceci que si vous faites tourner le vôtre.
settings-downloads-runs-in = Les recherches passent par
settings-downloads-runs-in-native-helper = Les abonnements restent au client torrent : les épisodes arrivent même AniBeam fermé.
settings-downloads-runs-in-helper = L'assistant de recherche, un programme à part qu'AniBeam lance ou que vous faites tourner.
settings-downloads-engine-native = AniBeam lui-même
settings-downloads-how-runs = Fonctionnement
settings-downloads-how-runs-sidecar = AniBeam le démarre à l'ouverture et l'arrête à la fermeture.
settings-downloads-how-runs-remote = AniBeam utilise celui que vous faites déjà tourner, à l'adresse et avec la clé ci-dessous.
settings-downloads-runs-sidecar = AniBeam le lance
settings-downloads-runs-remote = Je le lance
settings-downloads-command = Commande
settings-downloads-command-helper = Un nom dans le PATH ou un chemin complet. ~/.local/bin est aussi parcouru.
settings-downloads-server-address-helper = La base du serveur, sans /api/v1.
settings-downloads-key = Clé
settings-downloads-key-saved-helper = Une clé est enregistrée. Saisissez-en une nouvelle pour la remplacer, ou videz le champ pour la retirer.
settings-downloads-key-helper = La clé que l'assistant de recherche a affichée au démarrage. Elle laisse entrer AniBeam.
settings-downloads-details-anirss = L'assistant de recherche a dit ceci en s'arrêtant.
settings-downloads-starting = Démarrage
settings-downloads-reconnect = Reconnecter
settings-downloads-save-first = Enregistrez d'abord vos modifications
settings-downloads-roll-title = Garder les suivants prêts
settings-downloads-roll-helper = Une longue série arrive quelques épisodes à la fois ; chaque épisode fini lance le suivant. Le tout n'a pas à tenir sur le disque.
settings-downloads-roll-big = Pour les longues séries
settings-downloads-roll-big-helper = Au-delà de { $count } épisodes, une série arrive d'office par lots. Sa ligne dans Téléchargements peut aussi tout prendre d'un coup.
settings-downloads-roll-ahead = Épisodes d'avance
settings-downloads-roll-ahead-helper = Combien restent téléchargés après celui en cours. 12 correspond environ à une saison.
settings-downloads-unit-episodes = épisodes
settings-downloads-roll-over = Longue au-delà de
settings-downloads-roll-over-helper = Les téléchargements avec plus d'épisodes que ça en prennent quelques-uns à la fois.
settings-downloads-roll-watched = Épisodes vus
settings-downloads-roll-watched-delete-helper = Supprimés une fois que vous êtes plus loin, donc le disque ne garde que ce qui reste à voir.
settings-downloads-roll-watched-keep-helper = Chaque fichier reste sur le disque après visionnage.
settings-downloads-keep-all = Tout garder
settings-downloads-keep-behind = En garder
settings-downloads-subscriptions-title = Abonnements et recherches
settings-downloads-last-search = Garder la dernière recherche
settings-downloads-last-search-helper = Téléchargements rouvre la dernière recherche. Sinon, la page s'ouvre vide.
settings-downloads-prune = Se désabonner des séries terminées
settings-downloads-prune-helper = 16 jours après la diffusion du dernier épisode d'une série, son flux est abandonné. La série et ses fichiers restent.
settings-downloads-sources-title = Sources de recherche
settings-downloads-sources-helper = Où va une recherche, dans l'ordre. Sans résultat, on passe à la suivante.
settings-downloads-sources-empty = Aucune source configurée pour l'instant
settings-downloads-sources-connect = Connectez l'assistant de recherche pour les lister
settings-downloads-sources-empty-helper = Ajoutez une liste ci-dessous pour voir ses sources.
settings-downloads-config-file = Fichier de réglages de l'assistant de recherche
settings-downloads-config-native-helper = Contient les dossiers d'enregistrement, les tags et les règles du meilleur choix, qu'AniBeam suit lui-même quand il lance l'assistant. Ses propres sources ne servent qu'à l'assistant.
settings-downloads-config-helper = Contient les sources, les dossiers d'enregistrement, les tags et les règles du meilleur choix.
settings-downloads-unsaved = Modifications non enregistrées
settings-downloads-unsaved-helper = Enregistrer relance l'assistant de recherche avec les nouveaux réglages.
settings-downloads-discard = Abandonner

## SettingsDeveloperTab

settings-developer-title = Développeur
settings-developer-lead = Pourquoi une page est lente, mesuré. Chaque interrupteur s'applique à la prochaine page ouverte.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } Mo
settings-developer-seconds = { $seconds } s
settings-developer-switches = Interrupteurs
settings-developer-load-images = Charger les images
settings-developer-load-images-helper = Désactivé, aucune affiche, bannière ou portrait n'est lu ni décodé.
settings-developer-overlay = Incrustation de chargement
settings-developer-overlay-helper = Le dernier chargement de page et la mémoire, en bas à droite de la fenêtre.
settings-developer-read-cache = Garder les lectures de bibliothèque
settings-developer-read-cache-helper = La réponse à une lecture complète de la bibliothèque est gardée jusqu'à ce que la bibliothèque change. Désactivé, chaque page redemande au cœur.
settings-developer-poster-pool = Préchauffer les affiches
settings-developer-poster-pool-helper = Décode les 60 affiches que vous verrez le plus probablement et les garde en mémoire.
settings-developer-options = Options développeur
settings-developer-options-helper = Désactive cet onglet, l'incrustation et les mesures.
settings-developer-memory = Mémoire
settings-developer-memory-helper = Ce que le noyau compte pour ce processus. Les images gardent des pixels décodés en mémoire et sur la carte graphique.
settings-developer-memory-unknown = Ce système n'indique pas combien de mémoire l'appli utilise.
settings-developer-in-use = Utilisée
settings-developer-peak = Pic
settings-developer-heap = Tas
settings-developer-files-mapped = Fichiers mappés
settings-developer-threads = Threads
settings-developer-give-back = Rendre la mémoire
settings-developer-give-back-helper = Abandonne ce que le graphe de scène peut reconstruire, ramasse les déchets JavaScript et rend le tas libre au système. Ce qui est libéré, c'est ce que l'appli gardait sans en avoir besoin.
settings-developer-released = { $before } à { $after }
settings-developer-free-now = Libérer
settings-developer-images-total = Images depuis le lancement
settings-developer-images-total-line = { $decodes } décodées, { $pixels } de pixels depuis { $disk } sur le disque, { $ms } ms
settings-developer-loads = Chargements de page
settings-developer-loads-helper = Construite est l'attente après un clic : le thread Qt ne fait rien d'autre tant que la page n'existe pas. Stable est l'arrivée de sa dernière image.
settings-developer-touring = Chronométrage, { $at } sur { $steps }
settings-developer-tour = Chronométrer chaque page
settings-developer-copy-report = Copier le rapport
settings-developer-col-page = Page
settings-developer-col-built = Construite
settings-developer-col-painted = Peinte
settings-developer-col-settled = Stable
settings-developer-col-calls = Appels, données
settings-developer-col-decodes = Décodages, temps, pixels
settings-developer-col-items = Éléments
settings-developer-col-offscreen = Images hors vue
settings-developer-col-memory = Mémoire
settings-developer-load-no-images = { $page }, sans images
settings-developer-offscreen-of = { $offscreen } sur { $total }
settings-developer-loads-empty = Ouvrez une page et elle s'affiche ici.
settings-developer-load-images-off = { $page }, images désactivées
settings-developer-load-frames = { $frames } images peintes pendant le chargement, le plus long écart entre deux : { $gap }.
settings-developer-load-frames-timed-out = { $frames } images peintes pendant le chargement, le plus long écart entre deux : { $gap }. Attente de ses images abandonnée après 15 s.
settings-developer-built = Construite
settings-developer-settled = Stable
settings-developer-core-data = Données du cœur
settings-developer-pixels = Pixels décodés
settings-developer-page-items = Éléments sur la page
settings-developer-cached-images = Images déjà en mémoire
settings-developer-calls = Appels au cœur, les plus lents d'abord
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  en cache
settings-developer-call = { $name }  { $ms } ms  { $size }  cœur { $core }, vers QML { $qml }
settings-developer-decodes = Décodages les plus lents
settings-developer-decodes-line = { $count } décodages à la suite sur le thread des images, { $span } du premier au dernier ; { $requests } demandés, { $cached } déjà décodés, { $blocked } bloqués
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } vers { $w }x{ $h }  { $size }
settings-developer-decodes-none = Rien de décodé : chaque image était déjà en mémoire.
settings-developer-images-were-off = Les images étaient désactivées.
settings-developer-scene = De quoi est faite la page
settings-developer-scene-line = { $items } éléments, { $offscreen } hors de la fenêtre ; { $texts } textes, { $shapes } formes, { $icons } icônes ; { $images } images, dont { $imagesOffscreen } hors de la fenêtre qui occupent { $size }
settings-developer-copy-load = Copier ce chargement
settings-developer-tour-title = Chaque page, images activées et désactivées
settings-developer-tour-helper = La médiane de trois visites de chaque page. Là où les deux colonnes concordent, les images ne sont pas ce qui ralentit la page.
settings-developer-col-built-on = Construite, images
settings-developer-col-built-off = Construite, sans images
settings-developer-col-settled-on = Stable, images
settings-developer-col-settled-off = Stable, sans images
settings-developer-col-decodes-time = Décodages, temps
settings-developer-col-data = Données
settings-developer-tour-peak = Pic de mémoire pendant la tournée { $size }
settings-developer-events = Événements
settings-developer-events-helper = Tout ce que dit le cœur, à tous les niveaux, du plus récent. Chaque événement est copié vers le thread Qt en JSON ; la taille est ce que l'envoi a coûté.
settings-developer-event-rate = { $kind } { $count }/min, { $size }
settings-developer-events-filter = Filtrer par mots, type ou étape
settings-developer-event-plain = { $message }   (brut : { $plain })
settings-developer-copy-events = Copier les événements

## SettingsKeysTab

settings-keys-title = Clavier
settings-keys-lead = Tous les raccourcis d'AniBeam. Appuyez sur ? partout pour voir ceux de l'écran.
settings-keys-player = Dans le lecteur
settings-keys-player-around = Se déplacer dans le lecteur
settings-keys-around = Se déplacer
settings-keys-anywhere = Partout

## KeyHelp

key-help-title = Raccourcis clavier
key-help-group-playback = Lecture
key-help-group-navigation = Navigation
key-help-group-library = Bibliothèque
key-help-key-space = Espace / K
key-help-key-left-right = Gauche / Droite
key-help-key-ctrl-right = Ctrl+Droite
key-help-key-backspace = Retour arrière
key-help-key-up-down = Haut / Bas
key-help-key-hold = Maintenir sur la vidéo
key-help-key-page = Pg. préc. / Pg. suiv.
key-help-key-home-end = Début / Fin
key-help-key-escape = Échap
key-help-key-alt-left = Alt+Gauche
key-help-key-alt-right = Alt+Droite
key-help-key-shift-f10 = Maj+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / ou Ctrl+F
key-help-key-ctrl-space = Ctrl+Espace
key-help-key-shift-space = Maj+Espace
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = Lecture ou pause
key-help-seek-5 = Avancer ou reculer de 5 s
key-help-seek-10 = Avancer ou reculer de 10 s
key-help-skip = Passer l'intro ou l'outro, sinon 90 s
key-help-frame-step = Une image en arrière ou en avant
key-help-speed = Plus lent ou plus rapide, par pas de 0,25
key-help-normal-speed = Vitesse normale
key-help-volume = Volume plus ou moins 5
key-help-mute = Couper le son
key-help-subtitles-toggle = Masquer et réafficher les sous-titres
key-help-subtitle-delay = Sous-titres 100 ms plus tôt ou plus tard
key-help-audio-delay = Audio 100 ms plus tôt ou plus tard
key-help-hold-speed = Vitesse 2× en maintenant
key-help-screenshot = Enregistrer une capture dans Images
key-help-episode = Épisode suivant ou précédent
key-help-chapter = Chapitre précédent ou suivant
key-help-start-end = Le début, ou l'outro
key-help-fullscreen = Plein écran
key-help-escape-player = Fermer ce qui est ouvert, sinon quitter le plein écran, sinon quitter le lecteur
key-help-toggle-list = Afficher ou masquer cette liste
key-help-library-search = Bibliothèque et recherche
key-help-search-page = Chercher dans cette page
key-help-activity = Activité
key-help-settings = Réglages
key-help-quit = Quitter
key-help-forward = Avancer
key-help-escape-app = Fermer ce qui est ouvert, sinon retour
key-help-context-menu = Le menu du clic droit
key-help-pick-card = Choisir une carte, pour agir sur plusieurs
key-help-pick-range = Choisir toutes les cartes jusqu'à celle-ci
key-help-pick-all = Choisir toutes les cartes affichées

## SettingsAboutTab

settings-about-title = À propos
settings-about-lead = Quel AniBeam c'est, et comment il apprend l'arrivée du suivant.
settings-about-update-failed = La mise à jour n'a pas abouti. Réessayez, ou récupérez-la sur la page de la version.
settings-about-update-ready = { $version } est prête à installer.
settings-about-update-command = { $version } est sortie. Mettez à jour avec { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = { $version } est sortie. Cette copie est compilée depuis les sources : faites un pull et recompilez.
settings-about-update-elsewhere = { $version } est sortie, sans téléchargement ici pour cet ordinateur.
settings-about-version = Version
settings-about-build = AniBeam { $build }
settings-about-whats-new = Nouveautés
settings-about-install = Installer { $version }
settings-about-setup = Configuration
settings-about-setup-helper = Le guide de démarrage : votre dossier, votre liste et les téléchargements.
settings-about-run-setup = Relancer la configuration
settings-about-updates = Mises à jour
settings-about-nightly-helper = Une nouvelle build presque chaque jour. Plus récente, et moins testée.
settings-about-stable-helper = Une nouvelle version une fois testée et publiée.
settings-about-stable = Stable
settings-about-nightly = Nightly
settings-about-check-now = Vérifier
settings-about-developer = Options développeur
settings-about-developer-helper = Ajoute une section Développeur : temps des pages, mémoire, journal complet et images désactivables.
settings-about-credits = Crédits
settings-about-ratings = Notes
settings-about-ratings-helper = Information courtesy of IMDb (https://www.imdb.com). Used with permission.
settings-about-series = Séries et épisodes
settings-about-series-helper = AniList et TVmaze. Horaires de saut depuis AniSkip.

## SettingsPlaybackTab

settings-playback-title = Lecture
settings-playback-lead = Ce que le lecteur fait tout seul, pour n'y toucher que si vous le voulez.
settings-playback-skipping = Sauts
settings-playback-skipping-helper = Prend les chapitres du fichier, sinon les sauts partagés en ligne. Annuler dans le lecteur les coupe jusqu'à ce qu'AniBeam ferme.
settings-playback-skip-intros = Passer les intros
settings-playback-skip-outros = Passer les outros
settings-playback-skip-outros-helper = Atteindre l'outro marque aussi l'épisode comme vu.
settings-playback-next-episode = Épisode suivant
settings-playback-name-next = Nommer l'épisode suivant
settings-playback-name-next-helper = Le bouton Suivant en fin d'épisode affiche le titre du suivant. Désactivé, il n'affiche que son numéro.
settings-playback-languages = Langues
settings-playback-languages-helper = Codes de langue, préférée d'abord. La première présente l'emporte. Une piste choisie dans le lecteur vaut pour toute la série.
settings-playback-subtitle-languages = Langues des sous-titres
settings-playback-audio-languages = Langues audio
settings-playback-discord = Afficher ce que vous regardez sur Discord
settings-playback-discord-check-failed = AniBeam n'a pas pu vérifier Discord. Réessayez dans un instant.
settings-playback-discord-helper = La série, l'épisode et l'affiche apparaissent sur votre profil Discord pendant la lecture. Rien n'est envoyé quand c'est désactivé ou que Discord est fermé.
settings-playback-discord-unavailable = Cette version n'a pas d'application Discord.
settings-playback-discord-connected = Connecté à Discord
settings-playback-discord-not-connected = Non connecté. AniBeam se connecte pendant la lecture d'un épisode, Discord ouvert
settings-playback-advanced = Avancé
settings-playback-my-mpv = Utiliser mes réglages mpv
settings-playback-my-mpv-helper = Lit votre mpv.conf. Les options qui ne marchent qu'au démarrage sont ignorées, et les scripts ne se chargent jamais.
settings-playback-no-mpv-conf = Aucun fichier de réglages mpv trouvé. Mettez vos options dans mpv.conf, puis activez ceci.

## SliderRow

slider-row-spoken-unit = { $name }, en { $unit }
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = Accent rouge
swatches-accent-orange = Accent orange
swatches-accent-yellow = Accent jaune
swatches-accent-green = Accent vert
swatches-accent-cyan = Accent cyan
swatches-accent-blue = Accent bleu

## LookPreview

look-preview-dark = Sombre
look-preview-light = Clair

## SettingsAccountsTab

settings-accounts-title = Comptes
settings-accounts-lead = Connectez AniList ou MyAnimeList et chaque épisode terminé y est marqué pour vous. Les compteurs ne font que monter.
settings-accounts-syncing = Synchro de vos listes
settings-accounts-synced = Listes synchronisées
settings-accounts-lists = Vos listes d'animes
settings-accounts-lists-helper = Sans liste, AniBeam retient quand même ce que vous avez vu sur cet ordinateur. Une liste sur AniList ou MyAnimeList l'emporte sur votre téléphone et votre profil.
settings-accounts-progress = Progression
settings-accounts-main-list = Liste principale
settings-accounts-main-list-helper = Les cartes montrent la progression de celle-ci. L'autre reçoit quand même chaque mise à jour.
settings-accounts-local-lists = Listes sur cet ordinateur
settings-accounts-local-lists-helper =
    { $count ->
        [one] Une série a
       *[other] { $count } séries ont
    } un statut ou une note seulement ici. Transférez-les vers { $tracker } pour les voir sur votre téléphone et votre profil.
settings-accounts-review-move = Vérifier et transférer
settings-accounts-nothing-to-move = Plus rien à transférer
settings-accounts-sync-now = Synchroniser
settings-accounts-sync-now-helper = Relit chaque liste connectée. AniBeam le fait aussi au démarrage.
settings-accounts-sync = Synchro
settings-accounts-move-on-hold = Passer en attente
settings-accounts-move-on-hold-helper = Une série En cours à laquelle vous n'avez pas touché depuis un moment passe en attente dans vos listes. La lire la fait revenir. Une série qui attend son prochain épisode ne bouge pas.
settings-accounts-after = Après
settings-accounts-after-line = Jours sans lire d'épisode
settings-accounts-days-unit = jours

## SettingsPhoneTab

settings-phone-title = Appli mobile
settings-phone-lead = AniBeam sur votre téléphone peut lire cette bibliothèque, voir ce qui est en diffusion et suivre vos téléchargements, où que vous soyez.
settings-phone-online = En ligne
settings-phone-offline = Hors ligne
settings-phone-untrusted = Ne reçoit rien de cet ordinateur
settings-phone-sharing = Partage
settings-phone-sharing-helper = Seuls vos appareils Tailscale y accèdent. Un épisode vu sur le téléphone garde ici sa position et sa progression.
settings-phone-share = Partager via Tailscale
settings-phone-stream = Diffuser les épisodes
settings-phone-stream-on = Vos appareils peuvent lire et caster les épisodes de cet ordinateur.
settings-phone-stream-off = Vos appareils voient la bibliothèque et ce qui est en diffusion, mais ne peuvent rien lire d'ici.
settings-phone-host-service = Service hôte
settings-phone-not-running = Arrêté
settings-phone-running = En marche
settings-phone-service-off-helper = Lancez la commande une fois : AniBeam reste joignable dès la connexion, sans fenêtre.
settings-phone-service-on-helper = Vos appareils joignent AniBeam sans fenêtre ouverte. Tant que cette fenêtre est ouverte, c'est elle qui partage à la place du service.
settings-phone-copy-command = Copier la commande
settings-phone-command-copied = Commande copiée
settings-phone-nobody = Ne reçoit rien : Tailscale ne lui associe personne
settings-phone-people = Personnes
settings-phone-own-profile = Profil de cet ordinateur
settings-phone-person-own-devices =
    { $count ->
        [0] Cet ordinateur
        [one] Cet ordinateur et { $count } appareil
       *[other] Cet ordinateur et { $count } appareils
    }
settings-phone-person-devices =
    { $count ->
        [one] { $count } appareil
       *[other] { $count } appareils
    }
settings-phone-devices = Appareils sur votre tailnet
settings-phone-devices-helper = Chaque appareil de votre tailnet peut joindre cet ordinateur tant que le partage est actif. Désactivez-en un et il ne reçoit plus rien d'ici.
settings-phone-addresses = Adresses
settings-phone-addresses-helper = Saisissez l'une d'elles dans l'appli mobile la première fois qu'elle la demande.
settings-phone-copy = Copier
settings-phone-address-copied = Adresse copiée

## SettingsHelpTab

settings-help-title = Aide
settings-help-lead = Réponses courtes aux questions fréquentes, chacune menant à la solution.
settings-help-folders = Comment organiser mes dossiers ?
settings-help-folders-helper = Un dossier par série, épisodes dedans. Chaque fichier porte son numéro, comme les releases : « Frieren - 01.mkv » ou « Frieren S01E01.mkv ». Saison 2 : son dossier, ou « Season 2 » dans la série. Films : dossier Movies, un fichier chacun.
settings-help-for-example = Par exemple
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = Aucun dossier ajouté pour l'instant
settings-help-reads-folders =
    { $count ->
        [one] AniBeam lit un dossier
       *[other] AniBeam lit { $count } dossiers
    }
settings-help-add-folder-helper = Ajoutez le dossier qui contient vos séries, pas chaque série séparément.
settings-help-add-folder = Ajouter un dossier
settings-help-open-folders = Ouvrir vos dossiers
settings-help-match = Pourquoi ma série n'a-t-elle pas été reconnue ?
settings-help-match-helper = AniBeam cherche chaque dossier par son nom, sur AniList puis TVmaze. Mots en trop, faute ou nom étranger peuvent le faire rater : choisissez la série dans Infos de la série. Votre choix n'est jamais remplacé.
settings-help-all-recognised = Chaque série est reconnue
settings-help-unrecognised =
    { $count ->
        [one] Une série n'est pas reconnue
       *[other] { $count } séries ne sont pas reconnues
    }
settings-help-open-show-info = Ouvrir Infos de la série
settings-help-connect = Comment connecter AniList ?
settings-help-connect-helper = Connectez-le et chaque épisode fini est marqué dans votre liste : téléphone et profil suivent. AniList demande de créer une fois une petite appli à vous ; AniBeam vous guide, en une minute environ. MyAnimeList marche pareil.
settings-help-both-connected = Les deux sont connectés
settings-help-anilist-connected = AniList est connecté
settings-help-mal-connected = MyAnimeList est connecté
settings-help-none-connected = Aucun connecté
settings-help-connect-anilist = Connecter AniList
settings-help-connect-mal = Connecter MyAnimeList
settings-help-downloads = Pourquoi Téléchargements est-il vide ?
settings-help-downloads-helper = AniBeam n'a aucune source au départ. Ajoutez une liste de sources dans Réglages, sous Téléchargements, et installez-en les sources voulues. Il faut aussi un client torrent : AniBeam lance le sien, ou utilise votre qBittorrent.
settings-help-sources = Sources et torrents
settings-help-open-downloads = Ouvrir les réglages Téléchargements
settings-help-data = Où sont mes données ?
settings-help-data-helper = Bibliothèque, épisodes vus et réglages restent sur cet ordinateur. Les connexions AniList et MyAnimeList vont dans le trousseau du système, sinon dans un fichier lisible par vous seul. Sauvegardez tout depuis Stockage et sauvegarde.
settings-help-library-history = Bibliothèque et historique
settings-help-show-folder = Afficher le dossier
settings-help-backup = Sauvegarder ou restaurer
settings-help-open-storage = Ouvrir Stockage et sauvegarde
settings-help-more = Autre chose ?
settings-help-more-helper = Relancez la configuration ou dites ce qui cloche. Une capture aide beaucoup.
settings-help-first-setup = La première configuration
settings-help-first-setup-helper = Dossier, liste et téléchargements.
settings-help-run-setup = Relancer la configuration
settings-help-report = Signaler un problème
settings-help-report-helper = Ouvre un rapport de bug sur GitHub dans votre navigateur, avec cette version déjà remplie.

## SettingsSubtitlesTab

settings-subtitles-title = Sous-titres
settings-subtitles-lead = Réglages de départ de chaque épisode. L'image ci-dessous suit vos changements.
settings-subtitles-reset-done = Les sous-titres sont revenus à leur aspect d'origine
settings-subtitles-size-place = Taille et position
settings-subtitles-size = Taille
settings-subtitles-position = Position
settings-subtitles-position-helper = 100 se pose sur le bord inférieur. Des nombres plus bas remontent les lignes dans l'image.
settings-subtitles-style = Style
settings-subtitles-styled = Sous-titres stylés
settings-subtitles-as-made-helper = Les fichiers stylés (ASS) s'affichent exactement comme leurs auteurs les ont réglés.
settings-subtitles-mine-helper = Votre style ci-dessous s'applique à chaque fichier. Panneaux et karaoké peuvent casser.
settings-subtitles-size-only-helper = Un fichier stylé garde son style, à votre taille. Un fichier simple prend le vôtre.
settings-subtitles-as-made = D'origine
settings-subtitles-size-only = Taille seule
settings-subtitles-mine = Le mien
settings-subtitles-font = Police
settings-subtitles-font-invalid = Saisissez un nom de police, comme Noto Sans.
settings-subtitles-bold = Gras
settings-subtitles-text-colour = Couleur du texte
settings-subtitles-colour-invalid = Saisissez une couleur comme #ffcc00.
settings-subtitles-outline = Contour
settings-subtitles-outline-invalid = La largeur est un nombre, 0 ou plus.
settings-subtitles-outline-colour-invalid = Saisissez la couleur comme #000000.
settings-subtitles-outline-helper = Sa largeur, puis sa couleur.
settings-subtitles-shadow = Ombre
settings-subtitles-shadow-invalid = Un nombre, 0 ou plus. 0 signifie sans ombre.
settings-subtitles-shadow-helper = La portée de l'ombre, 0 pour aucune.
settings-subtitles-box = Fond
settings-subtitles-box-helper = Un fond sombre derrière les lignes. 0 l'enlève.
settings-subtitles-start-over = Repartir de zéro
settings-subtitles-start-over-helper = Remet la taille, la police, les couleurs, le contour, le fond et la position comme à l'origine.
settings-subtitles-reset = Réinitialiser

## SettingsDataTab

settings-data-title = Stockage et sauvegarde
settings-data-lead = Changez d'ordinateur, sauvegardez la bibliothèque ou libérez l'espace des images.
settings-data-backed-up = Sauvegardé dans { $path }
settings-data-restore-finished = Restauration terminée
settings-data-images-removed =
    { $count ->
        [one] { $count } image supprimée
       *[other] { $count } images supprimées
    }
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
        [one] { $count } dossier ajouté
       *[other] { $count } dossiers ajoutés
    }
settings-data-summary-series = { $count } séries créées
settings-data-summary-matches =
    { $count ->
        [one] { $count } association appliquée
       *[other] { $count } associations appliquées
    }
settings-data-summary-views =
    { $views ->
        [one] { $views } visionnage
       *[other] { $views } visionnages
    } et { $episodes ->
        [one] { $episodes } épisode terminé fusionnés
       *[other] { $episodes } épisodes terminés fusionnés
    }
settings-data-summary-resume-points =
    { $count ->
        [one] { $count } point de reprise fusionné
       *[other] { $count } points de reprise fusionnés
    }
settings-data-summary-accounts =
    { $count ->
        [one] { $count } compte de liste importé
       *[other] { $count } comptes de liste importés
    }
settings-data-summary-fixes =
    { $count ->
        [one] { $count } fichier reprend une correction que vous avez faite.
       *[other] { $count } fichiers reprennent une correction que vous avez faite.
    }
settings-data-summary-unavailable =
    { $count ->
        [one] { $count } dossier est introuvable sur cet ordinateur.
       *[other] { $count } dossiers sont introuvables sur cet ordinateur.
    }
settings-data-summary-left-out = Exclus : { $fields }.
settings-data-backup = Sauvegarde
settings-data-back-up-library = Sauvegarder la bibliothèque
settings-data-back-up-helper = Un fichier : dossiers, associations, historique.
settings-data-last-written = Dernière écriture dans { $path }
settings-data-open-folder = Ouvrir le dossier
settings-data-back-up = Sauvegarder
settings-data-private = Inclure les données privées
settings-data-private-helper = Ajoute les connexions AniList et MyAnimeList, l'historique et les préférences. Connexions en clair : gardez le fichier pour vous.
settings-data-restore-from = Restaurer une sauvegarde
settings-data-restore-helper = Lit une sauvegarde dans cette bibliothèque. Ce qui est sur le disque l'emporte ; le fichier complète le reste.
settings-data-restore = Restaurer
settings-data-image-cache = Cache d'images
settings-data-image-cache-helper = Affiches, bannières et portraits restent sur le disque pour un usage hors ligne. Une image effacée se retélécharge au besoin.
settings-data-cache-unmeasured = Impossible de mesurer le cache d'images
settings-data-cache-size =
    { $count ->
        [one] { $size } pour { $count } image
       *[other] { $size } pour { $count } images
    }
settings-data-cache-empty = Aucune image en cache
settings-data-cache-unmeasured-helper = Réessayez dans un instant. Les images elles-mêmes vont bien.
settings-data-remove-images =
    { $count ->
        [one] Supprimer { $count } image ?
       *[other] Supprimer { $count } images ?
    }
settings-data-files = Où AniBeam garde ses fichiers
settings-data-files-database = Base de la bibliothèque
settings-data-files-data = Données
settings-data-files-settings = Réglages
settings-data-files-cache = Cache
settings-data-backup-filter = Sauvegarde AniBeam (*.json)
settings-data-all-files-filter = Tous les fichiers (*)
settings-data-backing-up = Sauvegarde
settings-data-restoring = Restauration

## SettingsLibraryTab

settings-library-title = Bibliothèque
settings-library-lead = Les dossiers qu'AniBeam surveille et comment il en lit le contenu.
settings-library-not-scanned = Pas encore analysé
settings-library-scan-nothing-changed = Rien n'a changé
settings-library-scan-added = { $count } ajoutées
settings-library-scan-changed = { $count } modifiées
settings-library-scan-removed = { $count } retirées
settings-library-scan-in-folder = { $folder } : { $what }
settings-library-anilist-about = Anime : séries, films et leurs liens. Ce que vous regardez est marqué dans vos suivis.
settings-library-tvmaze-about = Autres programmes TV : prises de vue réelles, dessins animés, documentaires. Rien n'est marqué dans un suivi.
settings-library-tried-first = Essayé en 1er
settings-library-tried-second = Essayé en 2e
settings-library-tried-third = Essayé en 3e
settings-library-tried-fourth = Essayé en 4e
settings-library-tried-nth = Essayé en { $number }e
settings-library-folders = Dossiers
settings-library-folders-helper = Chaque sous-dossier est une série. Un fichier à la racine d'un dossier Movies est un film.
settings-library-rescan-started = Nouvelle analyse lancée
settings-library-load-failed = Impossible de lire vos dossiers
settings-library-load-failed-helper = AniBeam n'a pas pu lire sa liste de dossiers. Réessayez dans un instant.
settings-library-no-folders = Pas encore de dossier
settings-library-no-folders-helper = Indiquez à AniBeam le dossier où vivent vos animes. Il trouve les séries et les films tout seul.
settings-library-scanned = Analysé { $when }
settings-library-add-a-folder = Ajouter un dossier
settings-library-scan-all = Tout analyser
settings-library-scan-started = Analyse lancée
settings-library-add-folder = Ajouter
settings-library-matching = Association
settings-library-matching-helper = Un nouveau dossier suit cet ordre et garde le premier bon résultat. Un choix manuel n'est jamais remplacé.
settings-library-try-later = Essayer après
settings-library-try-first = Essayer avant
settings-library-check-matches = Vérifier les associations
settings-library-check-matches-helper = Infos de la série liste les infos trouvées par série et corrige les erreurs.
settings-library-open-show-info = Ouvrir Infos de la série
settings-library-hidden-shows = Séries masquées
settings-library-show-hidden = Afficher les séries masquées
settings-library-show-hidden-helper = Réaffiche partout les séries masquées jusqu'à ce qu'AniBeam ferme.
settings-library-hidden-from-airing = Masquées dans En diffusion
settings-library-hidden-from-airing-helper = Les séries marquées sans intérêt. Chacune reste hors d'En diffusion jusqu'à ce que vous la rameniez.
settings-library-hidden-from-airing-empty = Rien pour l'instant. Les séries marquées sans intérêt dans En diffusion s'accumulent ici, pour que chacune puisse revenir.
settings-library-show-again = Réafficher
settings-library-calendar = Calendrier
settings-library-calendar-feed = Horaires de diffusion dans un calendrier
settings-library-calendar-feed-helper = Tient un fichier calendrier des prochains épisodes des séries de vos listes, mis à jour quand le planning change.
settings-library-calendar-file = Fichier calendrier
settings-library-calendar-file-helper = Abonnez-vous à ce fichier dans votre appli de calendrier.
settings-library-copy = Copier
settings-library-path-copied = Chemin copié
settings-library-open-folder = Ouvrir le dossier
settings-library-notifications = Notifications
settings-library-new-episodes = Nouveaux épisodes
settings-library-new-episodes-helper = Une notification de bureau quand un épisode d'une série de votre liste En cours est diffusé. Cliquez dessus pour ouvrir la série.
settings-library-folder-added = Dossier ajouté, analyse en cours

## SettingsAppearanceTab

settings-appearance-language = Langue
settings-appearance-language-system = Comme le système ({ $language })
settings-appearance-text-smaller = Petit
settings-appearance-text-normal = Normal
settings-appearance-text-large = Grand
settings-appearance-text-larger = Plus grand
settings-appearance-text-largest = Maximum
settings-appearance-title = Apparence
settings-appearance-lead = Les changements s'appliquent aussitôt, ici et sur chaque page. Ils sont dans ~/.config/anibeam/theme.toml, que vous pouvez aussi modifier à la main.
settings-appearance-colours = Couleurs
settings-appearance-mode = Mode
settings-appearance-mode-dark = Sombre
settings-appearance-mode-light = Clair
settings-appearance-mode-system = Système
settings-appearance-colours-from = Couleurs depuis
settings-appearance-from-theme-helper = La paire de thèmes ci-dessous, un sombre et un clair.
settings-appearance-from-desktop-helper = Les couleurs de votre terminal, sinon la palette et l'accent du bureau.
settings-appearance-from-desktop = Bureau
settings-appearance-from-theme = Thème
settings-appearance-dark-theme = Thème sombre
settings-appearance-light-theme = Thème clair
settings-appearance-themes-helper = Les fichiers de thème déposés dans ~/.config/anibeam/themes s'affichent dans les deux listes.
settings-appearance-accent = Accent
settings-appearance-accent-helper = Sélection, lecture et nouveautés.
settings-appearance-icon = Icône
settings-appearance-app-icon = Icône de l'appli
settings-appearance-app-icon-helper = Celle de la fenêtre, de la barre des tâches et du lanceur. Sombre inverse le jaune et le noir.
settings-appearance-icon-default = Par défaut
settings-appearance-icon-default-name = Icône par défaut
settings-appearance-icon-dark = Sombre
settings-appearance-icon-dark-name = Icône sombre
settings-appearance-size-motion = Taille et mouvement
settings-appearance-text-size = Taille du texte
settings-appearance-text-size-helper = La taille des mots partout dans AniBeam, en plus de celle de votre système.
settings-appearance-density = Densité
settings-appearance-density-helper = L'espace autour de tout, de serré à aéré.
settings-appearance-density-compact = Compact
settings-appearance-density-normal = Normal
settings-appearance-density-comfortable = Aéré
settings-appearance-poster-size = Taille des affiches
settings-appearance-poster-size-helper = La taille d'une série dans la bibliothèque et l'accueil.
settings-appearance-poster-small = Petite
settings-appearance-poster-medium = Moyenne
settings-appearance-poster-large = Grande
settings-appearance-out-now = Sortis, dans En diffusion
settings-appearance-out-now-helper = Des cartes avec la bannière de chaque série, ou une liste de petites affiches et de titres, sur trois rangées.
settings-appearance-out-now-cards = Cartes
settings-appearance-out-now-list = Liste
settings-appearance-out-now-grid = Cartes des sortis
settings-appearance-out-now-grid-helper = Combien de cartes s'affichent, colonnes par rangées, avant que Tout ouvre le reste.
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = Réduire les animations
settings-appearance-reduce-motion-helper = Coupe l'animation d'ouverture de la bibliothèque et les effets décoratifs.

## LookPane

look-pane-sample-series = Série d'exemple
look-pane-episode-code = Ép. { $number }
look-pane-library = Bibliothèque
look-pane-series-count =
    { $count ->
        [one] { $count } série
       *[other] { $count } séries
    }
look-pane-search = Titre ou dossier
look-pane-search-hint = / ou Ctrl+K
look-pane-all = Tout
look-pane-series = Séries
look-pane-movies = Films
look-pane-plain = Simple
look-pane-selected = Sélectionné
look-pane-errors =
    { $count ->
        [one] { $count } erreur
       *[other] { $count } erreurs
    }
look-pane-type-large = Grand
look-pane-type-normal = Normal
look-pane-type-small = Petit, atténué
look-pane-sample-switch = Interrupteur d'exemple
look-pane-button = Bouton
look-pane-scan = Analyser
look-pane-sample-status =
    { $count ->
        [one] { $count } ép. dans { $title }
       *[other] { $count } ép. dans { $title }
    }

## FeedPage

feed-title = Fil
feed-aired = { $episode } diffusé { $when }
feed-downloaded = Fichier téléchargé { $when }
feed-countdown-days = { $days } j { $hours } h
feed-countdown-hours = { $hours } h { $minutes } min
feed-countdown-minutes = { $minutes } min
feed-section-this-week = Cette semaine
feed-new-episodes =
    { $count ->
        [one] { $count } nouvel épisode
       *[other] { $count } nouveaux épisodes
    }
feed-section-earlier = Plus tôt
feed-latest-out = Dernier sorti { $when }
feed-section-coming-soon = Bientôt
feed-this-week-count = { $count } cette semaine
feed-section-everything-else = Tout le reste
feed-out-this-week = { $count } sortis cette semaine
feed-nothing-scheduled = Rien de prévu pour l'instant.
feed-next = Suivant : { $title } dans { $countdown }
feed-nothing-new = Rien de nouveau cette semaine.
feed-new-this-week =
    { $count ->
        [one] { $count } nouvel épisode cette semaine.
       *[other] { $count } nouveaux épisodes cette semaine.
    }
feed-sort-recent = Sortis récemment
feed-sort-upcoming = Bientôt
feed-failed-title = Le fil n'a pas chargé
feed-empty-title = Rien ici pour l'instant
feed-failed-body = AniBeam n'a pas pu lire le fil. Réessayez dans un instant.
feed-empty-body-folders = Le fil se remplit à mesure que les épisodes sont diffusés et arrivent. En diffusion montre ce qui passe cette saison.
feed-empty-body = Ajoutez le dossier où vivent vos animes et le fil se remplit de ses épisodes.
feed-add-folder = Ajouter un dossier
feed-airing-now = En diffusion

## RecommendedPage

recommended-title = Recommandées pour vous
recommended-ranked-minutes = Classées il y a { $minutes } min
recommended-ranked-hours = Classées il y a { $hours } h
recommended-ranked-days = Classées il y a { $days } j
recommended-scope-name = Recommandées depuis
recommended-scope-all = Tout
recommended-scope-library = Bibliothèque
recommended-scope-list = Liste
recommended-ranking = Classement
recommended-rank-again = Reclasser
recommended-no-scores-title = Notez d'abord quelques séries
recommended-no-list-title = Rien dont apprendre pour l'instant
recommended-never-fitted-title = Classement de vos séries
recommended-nothing-left-title = Plus rien ici
recommended-no-scores-body = Les recommandations apprennent des notes de votre liste. Notez quelques séries sur AniList et elles apparaîtront ici.
recommended-no-list-body = Connectez AniList dans les réglages ou tenez une liste ici : vos notes guident cette page.
recommended-never-fitted-body = Le premier classement lit AniList pendant une minute ou deux. Il tourne en arrière-plan.
recommended-nothing-left-library-body = Tout ce qui convient dans votre bibliothèque est vu. Tout trouve des séries que vous n'avez pas encore.
recommended-nothing-left-body = Tout ce qui convient dans vos listes est vu. Tout trouve des séries absentes de vos listes.

## MoveListsSheet

move-lists-moving =
    { $count ->
        [one] Transfert d'1 série vers { $site }
       *[other] Transfert de { $count } séries vers { $site }
    }
move-lists-score = note { $score }
move-lists-title = Transférer votre liste vers { $site }
move-lists-body =
    { $count ->
        [one] Une série a
       *[other] { $count } séries ont
    } un statut ou une note seulement sur cet ordinateur. Les transférer les met dans votre liste { $site }, où votre téléphone et votre profil les voient. Pour ces séries, ce que { $site } a maintenant est remplacé. Rien n'est envoyé avant que vous appuyiez sur Transférer.
move-lists-head-show = Série
move-lists-head-here = Sur cet ordinateur
move-lists-head-tracker = Sur { $site } maintenant
move-lists-nothing = Rien
move-lists-not-on-list = Pas dans votre liste
move-lists-later-hint = Plus tard : Réglages, Comptes
move-lists-not-now = Plus tard
move-lists-move =
    { $count ->
        [one] Transférer 1 série
       *[other] Transférer { $count } séries
    }

## StatsPage

stats-title = Stats
stats-month-jan = janv.
stats-month-feb = févr.
stats-month-mar = mars
stats-month-apr = avr.
stats-month-may = mai
stats-month-jun = juin
stats-month-jul = juil.
stats-month-aug = août
stats-month-sep = sept.
stats-month-oct = oct.
stats-month-nov = nov.
stats-month-dec = déc.
stats-nothing-yet = Rien pour l'instant
stats-ranked-episodes =
    { $count ->
        [one] { $count } ép.
       *[other] { $count } ép.
    }
stats-year-summary = Ce que vous avez regardé en { $year }
stats-empty-title = Rien de vu pour l'instant
stats-empty-body = Terminez un épisode et il compte ici : heures, genres et saisons regardées.
stats-open-library = Bibliothèque
stats-hours-watched = heures de visionnage
stats-episodes-label =
    { $count ->
        [one] épisode
       *[other] épisodes
    }
stats-series-finished = séries terminées
stats-none = Aucun
stats-busiest-month = mois le plus chargé
stats-by-month = Par mois
stats-month-hours = { $hours } h
stats-top-genres = Genres favoris
stats-top-studios = Studios favoris
stats-seasons-watched = Saisons regardées
stats-recap-title = Mon année en anime
stats-recap-hours = heures
stats-recap-episodes = épisodes
stats-recap-top-genre = genre favori
stats-save-image = Enregistrer en image
stats-save-dialog-title = Enregistrer le bilan
stats-save-filter-png = Image PNG (*.png)
stats-recap-saved = Bilan enregistré
stats-recap-save-failed = Impossible d'enregistrer le bilan

## TrackerRow

tracker-row-connected = Connecté en tant que { $user }, synchronisé { $when }
tracker-row-unknown-user = ?
tracker-row-never = jamais
tracker-row-waiting = En attente de votre accord dans le navigateur
tracker-row-not-connected = Non connecté
tracker-row-open-again = Rouvrir
tracker-row-open-again-tip = Rouvrir la page de connexion
tracker-row-disconnect = Déconnecter
tracker-row-disconnect-question = Déconnecter { $name } ? Votre connexion sera supprimée.
tracker-row-secret-saved = Un secret client est enregistré pour cette appli.
tracker-row-remove-secret = Le retirer
tracker-row-secret-removed = Secret client retiré
tracker-row-add-secret = Ajouter le secret client
tracker-row-secret-where = C'est la longue chaîne sous « Client Secret » sur la page de l'appli, sur myanimelist.net/apiconfig.
tracker-row-secret-placeholder = Secret client
tracker-row-save-connect = Enregistrer et connecter

## RemoveSheet

remove-unsubscribe-delete = Se désabonner et supprimer
remove-unsubscribe-torrents = Se désabonner et retirer les torrents
remove-unsubscribe = Se désabonner
remove-delete-files = Retirer et supprimer les fichiers
remove-torrents =
    { $count ->
        [one] Retirer le torrent
       *[other] Retirer les torrents
    }
remove-also-torrents =
    { $count ->
        [one] Retirer aussi son torrent
       *[other] Retirer aussi ses { $count } torrents
    }
remove-torrents-go-with-files = Supprimer les fichiers emporte les torrents avec eux.
remove-torrents-files-stay = Ils quittent le client ; les fichiers restent.
remove-also-files = Supprimer aussi les fichiers
remove-files-default-where = ce que les torrents ont téléchargé
remove-files-detail-rule = Met { $where } à la corbeille, et retire les torrents.
remove-files-detail = Met { $where } à la corbeille.

## NotesSheet

notes-your-tracker = votre suivi
notes-title = Notes
notes-placeholder = Où vous en êtes, qui l'a recommandée, quelle release prendre
notes-over-limit = { $count } caractères en trop
notes-saved-to = Enregistré sur { $sites }

## RecommendationCard

recommendation-card-available = Disponible
recommendation-card-open-library = Ouvrir { $title } dans votre bibliothèque
recommendation-card-open-anilist = Ouvrir { $title } sur AniList

## SearchField

search-field-placeholder = Titre ou dossier
search-field-hint = / ou Ctrl+K
search-field-clear = Effacer la recherche

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent } %
recommended-hover-hidden-tags =
    { $count ->
        [one] Un tag caché que vous adorez
       *[other] { $count } tags cachés que vous adorez
    }
recommended-hover-because = Parce que vous avez vu

## PersonCard

person-card-unknown = Inconnu
person-card-role-main = Principal
person-card-role-supporting = Secondaire
person-card-role-background = Figurant
person-card-the-character = le personnage
person-card-open = Ouvrir { $name } sur AniList

## StatusDot

status-dot-on-list = Dans votre liste : { $status }

## TrackerNudge

tracker-nudge-title = Connectez votre liste
tracker-nudge-body = AniBeam marque les épisodes vus sur AniList ou MyAnimeList.
tracker-nudge-connect-anilist = Connecter AniList
tracker-nudge-connect-mal = Connecter MyAnimeList
tracker-nudge-not-now = Plus tard

## RecommendedCard

recommended-card-spoken = { $title }, { $percent } pour cent pour vous
recommended-card-forecast = { $percent } %
recommended-card-in-library = Bibliothèque

## InlineConfirm

inline-confirm-keep = Garder

## Tokens

tokens-cube-watched-tracked = Vu, suivi en ligne
    .inline = vu, suivi en ligne
tokens-cube-watched-here = Vu ici, pas encore suivi
    .inline = vu ici, pas encore suivi
tokens-cube-on-disk = Sur le disque
    .inline = sur le disque
tokens-cube-downloading = Téléchargement
    .inline = en téléchargement
tokens-cube-missing = Manquant
    .inline = manquant
tokens-cube-error = Erreur
    .inline = erreur
tokens-cube-not-downloaded = Non téléchargé
    .inline = non téléchargé
tokens-cube-not-aired = Pas encore diffusé
    .inline = pas encore diffusé
tokens-cube-available = Disponible
    .inline = disponible
tokens-cube-picked = Choisi
    .inline = choisi
tokens-cube-already-on-disk = Déjà sur le disque
    .inline = déjà sur le disque
tokens-cube-no-release = Aucune release
    .inline = aucune release

## Card

card-not-interested = Sans intérêt

## PagePlaceholder

page-placeholder-coming-soon = Bientôt

## ErrorLine

error-line-hide-details = Masquer les détails
error-line-details = Détails
error-line-details-tip = Voir exactement ce qui n'a pas marché, et le copier
error-line-copied = Copié.

## Readouts (apps/linux/src/format.rs): times, countdowns and lengths

fmt-just-now = à l'instant
fmt-now = maintenant
fmt-ago = il y a { $span }
fmt-in = dans { $span }
fmt-seconds-short = { $n } s
fmt-minutes-short = { $n } min
fmt-hours-short = { $n } h
fmt-days-short = { $n } j
fmt-months-short = { $n } mois
fmt-years-short = { $n ->
    [one] { $n } an
   *[other] { $n } ans
}
fmt-years-months-short = { $years ->
    [one] { $years } an
   *[other] { $years } ans
} { $months } mois
fmt-countdown-days = { $d } j { $h } h { $m } min
fmt-countdown-hours = { $h } h { $m } min
fmt-countdown-days-seconds = { $d } j { $h } h { $m } min { $s } s
fmt-countdown-hours-seconds = { $h } h { $m } min { $s } s
fmt-countdown-minutes-seconds = { $m } min { $s } s
fmt-runtime-minutes = { $m } min
fmt-runtime-hours = { $h } h
fmt-runtime-hours-minutes = { $h } h { $m } min
fmt-episode = Épisode
fmt-size-b = { $size } o
fmt-size-kb = { $size } Ko
fmt-size-mb = { $size } Mo
fmt-size-gb = { $size } Go
fmt-size-tb = { $size } To
fmt-episode-span = Épisodes { $first } à { $through }

## apps/linux/src/problem.rs: every call error in a person's words, what happened and what to do

problem-report = Réessayez. Si ça continue, signalez-le depuis le journal d'activité.
problem-try-later = Réessayez plus tard.
problem-helper-unreachable = L'assistant de recherche n'a pas répondu.
problem-helper-unauthorized = L'assistant de recherche a refusé la connexion.
problem-helper-not-running = L'assistant de recherche ne tourne pas.
problem-helper-missing = L'assistant de recherche n'est pas installé.
problem-helper-failed = L'assistant de recherche n'a pas pu faire ça.
problem-helper-no-login = L'assistant de recherche ne peut pas se connecter au client torrent.
problem-helper-no-login-next = Enregistrez le mot de passe du client torrent dans Réglages, sous Téléchargements.
problem-torrent-unreachable = Le client torrent n'a pas répondu.
problem-torrent-unauthorized = Le client torrent a refusé la connexion.
problem-torrent-not-running = Le client torrent ne tourne pas.
problem-torrent-missing = Le client torrent n'est pas installé.
problem-torrent-failed = Le client torrent n'a pas pu faire ça.
problem-downloads-unreachable-next = Vérifiez qu'il tourne et que son adresse est correcte dans Réglages, sous Téléchargements.
problem-downloads-unauthorized-next = Vérifiez le nom d'utilisateur et le mot de passe dans Réglages, sous Téléchargements.
problem-downloads-not-running-next = Démarrez-le depuis Réglages, sous Téléchargements.
problem-downloads-missing-next = Réglages, sous Téléchargements, explique comment l'installer.
problem-trash = { -app-name } n'a pas pu mettre les fichiers à la corbeille.
problem-trash-next = Vérifiez que les fichiers vous appartiennent et que le disque n'est pas en lecture seule.
problem-source-list = Cette liste de sources n'a pas marché.
problem-source-list-next = Vérifiez son adresse dans Réglages, sous Téléchargements, Listes de sources.
problem-remote-unreachable = { $name } n'a pas répondu.
problem-remote-unreachable-next = Le site est peut-être en panne. Réessayez plus tard, ou cherchez sur une autre source.
problem-remote-unauthorized = { $name } a refusé la clé fournie.
problem-remote-unauthorized-next = Changez sa clé dans Réglages, sous Téléchargements, Listes de sources.
problem-remote-failed = { $name } n'a pas pu faire ça.
problem-remote-failed-next = Réessayez plus tard, ou cherchez sur une autre source.
problem-provider-sign-in = { $name } a refusé la connexion d'{ -app-name }.
problem-provider-sign-in-next = Reconnectez-le dans Réglages, sous Comptes.
problem-provider-turned-away = { $name } a refusé { -app-name }.
problem-provider-busy = { $name } est occupé en ce moment.
problem-provider-busy-next = Attendez une minute, puis réessayez.
problem-provider-no-answer = { $name } n'a pas répondu.
problem-provider-no-answer-next = Vérifiez votre connexion Internet, puis réessayez.
problem-provider-trouble = { $name } rencontre des difficultés en ce moment.
problem-provider-trouble-next = Réessayez dans quelques minutes.
problem-provider-rejected = { $name } n'a pas accepté ça.
problem-not-found = { -app-name } ne le trouve plus.
problem-not-found-next = Il a peut-être été retiré. Réessayez pour recharger la page.
problem-invalid-next = Vérifiez-le et réessayez.
problem-unavailable = Ce dossier est injoignable pour l'instant.
problem-unavailable-next = Branchez son disque ou vérifiez le dossier dans Réglages, sous Bibliothèque.
problem-not-connected = { $tracker } n'est pas connecté.
problem-not-connected-next = Connectez-le dans Réglages, sous Comptes.
problem-io = { -app-name } n'a pas pu lire ou écrire un fichier.
problem-io-next = Vérifiez qu'il reste de la place sur le disque et que le dossier vous appartient, puis réessayez.
problem-storage = { -app-name } n'a pas pu enregistrer votre bibliothèque.
problem-storage-next = Vérifiez qu'il reste de la place sur le disque, puis réessayez.
problem-keyring = Le gestionnaire de mots de passe du système n'a pas répondu.
problem-keyring-next = Déverrouillez le trousseau de votre bureau, puis réessayez.
problem-unsupported = { -app-name } ne sait pas encore faire ça ici.
problem-version = Cette sauvegarde vient d'un { -app-name } plus récent.
problem-version-next = Mettez { -app-name } à jour dans Réglages, sous À propos, puis importez-la de nouveau.
problem-internal = Un problème est survenu dans { -app-name }.
problem-refused-hidden = Cette série est masquée.
problem-refused-hidden-next = Affichez-la d'abord depuis sa page.
problem-refused-no-match = Cette série n'est pas encore associée.
problem-refused-no-match-next = Associez-la d'abord dans Infos de la série.
problem-refused-not-newer = Votre liste a déjà dépassé cet épisode.
problem-refused-extra = Ce fichier est un bonus, pas un épisode.
problem-refused-unmatched = Ce fichier n'est lié à aucun épisode.
problem-refused-unmatched-next = Corrigez-le depuis la page de la série, sous Corriger.
problem-refused-on-disk = Ce fichier n'existe que sur cet ordinateur.

## apps/linux/src/library_view.rs: the library page's section titles and the one fact under each
## (Dropped, On hold and Watching reuse status-* from core.ftl)

library-view-airing = En diffusion
library-view-no-date = Sans date
library-view-type-tv = TV
library-view-type-movies = Films
library-view-type-extras = OVA, ONA et spéciaux
library-view-type-music = Musique
library-view-type-unknown = Inconnu
library-view-no-genre = Non associées
library-view-state-caught-up = À jour
library-view-state-not-started = Pas commencées
library-view-next-episode = Prochain épisode dans { $span }
library-view-countdown-days = { $d } j { $h } h
library-view-countdown-hours = { $h } h { $m } min
library-view-countdown-minutes = { $m } min
library-view-in-progress = { $count } en cours
library-view-not-started = { $count } pas commencées
library-view-all-watched = Tout vu
library-view-on-disk =
    { $count ->
        [one] { $count } épisode
       *[other] { $count } épisodes
    } sur le disque
library-view-dropped-on-disk =
    { $count ->
        [one] { $count } fichier encore
       *[other] { $count } fichiers encore
    } sur le disque

## apps/linux/src/dbus/notify.rs: the desktop notification for a new episode
## (its Open button reuses action-open from core.ftl)

notify-episode-out = L'épisode { $episode } de { $title } est sorti
notify-click-open = Cliquez pour l'ouvrir dans { -app-name }
notify-click-get = Cliquez pour obtenir les épisodes dans { -app-name }
notify-get-episodes = Obtenir des épisodes

## apps/linux/src/nowplaying.rs: the second line a media widget shows while an episode plays

now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: why the screen stays on, as the desktop's power panel shows it

inhibit-reason = Lecture d'un épisode

## apps/linux/src/bridge/door.rs: why a value a page sent cannot be used (an error's detail line)

door-not-a-choice = { $value } n'est pas un choix valide
door-tokens-op-empty = une opération tokens a besoin de tokens
door-not-a-refine-op = { $value } n'est pas une opération d'affinage
door-limit-too-low = doit valoir au moins 1, reçu { $limit }
door-not-a-torrent-action = { $value } n'est pas une action torrent

## apps/linux/src/bridge/shell.rs: the first-run starter folder and the update notice

shell-starter-folder-failed = Impossible de créer { $folder } : { $error }
shell-update-gone = { -app-name } { $version } n'est plus proposé

## Watch together (#846)

onboarding-step-together = Regarder ensemble
onboarding-title-together = Regarder entre amis
onboarding-body-together = Ouvrez un salon depuis le lecteur et envoyez son code. Chacun lit sa propre copie de l'épisode, et lecture, pause et déplacements restent synchronisés pour tous.
onboarding-ready-together = Regarder ensemble
onboarding-ready-together-as = Dans les salons, vous apparaissez comme { $name }.
onboarding-ready-together-none = Pas encore de nom. Le panneau du salon le demande la première fois.
onboarding-together-you = Vous
onboarding-together-code = Code du salon
onboarding-together-episode = Frieren · Épisode 3
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = Votre nom dans les salons
rail-room = Regarder ensemble (Ctrl+J)
rail-room-in = Dans un salon de visionnage (Ctrl+J)
player-chrome-watch-together = Regarder ensemble
player-chrome-room = Salon { $code } · { $count ->
    [one] { $count } personne
   *[other] { $count } personnes
}
room-name-empty = Saisissez le nom que les autres verront dans le salon.
room-name-bad = Ce nom ne convient pas. Utilisez de 1 à 32 caractères.
room-error-not-found = Aucun salon n'a ce code. Vérifiez-le auprès de qui vous l'a envoyé ; le salon est peut-être fermé.
room-error-full = Ce salon est plein. Un salon accueille 10 personnes.
room-error-unreachable = Impossible de joindre le serveur des salons. Vérifiez votre connexion, puis réessayez.
room-error-refused = Le serveur des salons a refusé. Réessayez dans une minute.
room-error-proto = Ce salon utilise une version plus récente des salons. Mettez à jour { -app-name } pour le rejoindre.
room-error-not-in-room = Vous n'êtes plus dans ce salon. Ouvrez-en ou rejoignez-en un autre.
room-error-bad-code = Un code de salon fait 6 lettres et chiffres, comme K7QM2X.
room-error-bad-relay = Le serveur des salons dans Réglages, Lecture n'est pas une adresse web. Corrigez-le là, puis réessayez.
room-panel-title = Regarder ensemble
room-panel-close = Fermer
room-panel-connected = Connecté
room-panel-reconnecting = Reconnexion
room-panel-name-label = Votre nom dans les salons
room-panel-name-placeholder = Votre nom
room-panel-name-start = Ouvrir le salon
room-panel-name-join = Rejoindre le salon
room-panel-start = Ouvrir un salon
room-panel-or-join = Ou rejoignez-en un avec son code
room-panel-join-label = Code du salon
room-panel-join = Rejoindre
room-panel-you-are = Vous apparaissez comme { $name }
room-panel-change-name = Modifier
room-panel-code-label = Code du salon
room-panel-copy = Copier le lien
room-panel-link-copied = Lien du salon copié
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = Cet épisode n'est pas dans votre bibliothèque, vous ne pouvez donc pas encore suivre.
room-panel-open-episode = Ouvrir l'épisode
room-panel-people = { $count ->
    [one] { $count } personne
   *[other] { $count } personnes
}
room-panel-host = Hôte
room-panel-you = Vous
room-panel-ready = Prêt
room-panel-opening = Ouverture
room-panel-missing = Ne l'a pas
room-panel-offset = Votre décalage
room-panel-offset-earlier = Une demi-seconde plus tôt
room-panel-offset-later = Une demi-seconde plus tard
room-panel-seconds = { $value } s
room-panel-suggested = Les intros suggèrent { $offset }
room-panel-use-suggested = L'utiliser
room-panel-leave = Quitter le salon
player-room-speed = La vitesse suit le salon
settings-find-room-name =
    .words = regarder ensemble salon nom amis
settings-find-room-relay =
    .words = regarder ensemble salon relais serveur
key-help-key-ctrl-j = Ctrl+J
key-help-room = Regarder ensemble
settings-playback-together = Regarder ensemble
settings-playback-room-name = Votre nom dans les salons
settings-playback-room-name-placeholder = Votre nom
settings-playback-room-relay = Serveur des salons
settings-playback-room-relay-helper = Les salons sont créés ici. Ne le changez que pour un serveur que vous gérez vous-même.
