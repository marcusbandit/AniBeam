## Names that stay names

tracker-anilist = AniList
tracker-mal = MyAnimeList

## Words many pages share. Use these before adding a page's own copy.

common-cancel = Annuler
common-back = Retour
common-save = Enregistrer
common-done = Terminé
common-close = Fermer
common-delete = Supprimer
common-retry = Réessayer
common-copy = Copier
common-clear = Effacer
common-on = Activé
common-off = Désactivé
common-play = Lire
common-hide = Masquer
common-remove = Retirer
common-add = Ajouter
common-test = Tester
common-reading = Lecture...
common-working = En cours...
common-checking = Vérification...
common-all = Tout
common-none = Aucun
common-episodes = { $count ->
    [one] { $count } épisode
   *[other] { $count } épisodes
}
common-episode-n = Épisode { $n }
common-join-comma = { $a }, { $b }

## Language (#793)

lang-follow-phone = Comme le téléphone
more-language = Langue
more-language-phone = Comme le téléphone : { $lang }
lang-name-en = Anglais
lang-name-da = Danois
lang-name-de = Allemand
lang-name-es = Espagnol
lang-name-fr = Français
lang-name-pt-BR = Portugais (Brésil)
lang-name-it = Italien
lang-name-nl = Néerlandais
lang-name-pl = Polonais
lang-name-ru = Russe
lang-name-ja = Japonais
lang-name-ko = Coréen
lang-name-zh-CN = Chinois (simplifié)

## Onboarding

onb-welcome-title = AniBeam
onb-welcome-note = Vos animes sur ce téléphone. Trouvez une série, voyez qui l'a, téléchargez-la et gardez vos listes à jour.
onb-language-title = Langue
onb-language-note = AniBeam parle la langue de votre téléphone, sauf si vous en choisissez une autre. Vous pourrez la changer plus tard dans Plus.
onb-lists-title = Vos listes
onb-lists-note = Connectez-vous pour que ce que vous regardez ici compte sur AniList ou MyAnimeList. Vous pourrez le faire plus tard dans Plus.
onb-wifi-title = Wi-Fi seul
onb-wifi-note = Les épisodes sont lourds. AniBeam télécharge en Wi-Fi, sauf avis contraire.
onb-storage-title = Stockage
onb-storage-note = Une limite à ce qu'AniBeam peut télécharger. Rien n'est réservé : l'espace reste libre jusqu'à ce que les épisodes le remplissent.
onb-notify-title = Notifications
onb-notify-note = Pendant le téléchargement, une notification montre l'avancement, et une autre indique quand un épisode est prêt à regarder. Les téléchargements se font dans tous les cas.
onb-sites-title = Sites
onb-sites-note = AniBeam n'a aucun site où chercher au départ. Un dépôt est une liste signée de sites de recherche. Collez l'adresse d'un dépôt, vérifiez sa clé, puis installez les sites voulus. Vous pourrez le faire plus tard dans Plus.
onb-wifi-only = Wi-Fi seul
onb-wifi-on = Activé. Les données mobiles demandent d'abord.
onb-wifi-off = Désactivé. Les données mobiles servent aussi.
onb-storage-caption = 18 Go font environ 12 épisodes. Allez jusqu'à tout le stockage du téléphone.
onb-use-cap = Fixer la limite
onb-cap-on = Activée. À { $cap }, AniBeam ne télécharge plus rien tant que vous ne supprimez pas d'épisodes.
onb-cap-off = Désactivée. AniBeam télécharge jusqu'à remplir le téléphone.
onb-start = Commencer
onb-continue = Continuer
onb-back = Retour

## Appearance

look-colours = Couleurs
look-mode = Mode
look-dark = Sombre
look-light = Clair
look-system = Système
look-follows-phone-dark = Suit le réglage du téléphone : sombre.
look-follows-phone-light = Suit le réglage du téléphone : clair.
look-changes-at-once = Les changements s'appliquent aussitôt, ici et sur chaque page.
look-dark-theme = Thème sombre
look-light-theme = Thème clair
look-accent = Accent
look-accent-note = Marque ce qui est sélectionné, en lecture ou nouveau.
look-size = Taille
look-text-size = Taille du texte
look-size-smaller = Petit
look-size-normal = Normal
look-size-large = Grand
look-size-larger = Plus grand
look-size-largest = Maximum
look-text-size-note = La taille des mots partout dans AniBeam, en plus de celle de votre téléphone.

## The window: tabs and the sheets over every page

app-title = AniBeam
phone-nav-home = Accueil
nav-calendar = Calendrier
phone-nav-library = Biblio.
phone-nav-downloads = Téléch.
nav-more = Plus
app-mobile-title = Télécharger en données mobiles ?
app-mobile-note = Un épisode pèse souvent 1 à 2 Go. Avec un forfait limité, une soirée peut suffire à épuiser un mois de données.
app-mobile-keep = Rester en Wi-Fi
app-mobile-use = Utiliser aussi le mobile
app-versions-note = Le même épisode en { $count } releases. Choisissez celle à lire.
app-versions-plays-now = { $release }, en lecture
app-menu-get = Obtenir des épisodes
app-menu-delete = Supprimer des épisodes

## More: the index and each section's line of how it stands

more-settings = Réglages
more-search = Chercher un réglage
more-search-none = Aucun résultat
more-search-hits = { $count ->
    [one] 1 réglage
   *[other] { $count } réglages
}
more-search-tap = Touchez pour y aller
more-search-hint = Essayez un mot plus simple : stockage, Wi-Fi, suivi, site, ordinateur.
more-accounts = Comptes
more-streaming = Diffusion
more-subtitles = Sous-titres
more-connection = Connexion
more-computer = Votre ordinateur
more-icon = Icône
more-stats = Stats
more-data = Données
more-appearance = Apparence
more-activity = Activité
more-about = À propos
more-accounts-none = Aucun suivi connecté
more-accounts-syncing = Synchro avec { $tracker }
more-accounts-syncing-both = Synchro avec AniList et MAL
more-dl-status-wifi = Wi-Fi seul, sans limite
more-dl-status-wifi-cap = Wi-Fi uniquement, limité à { $cap }
more-dl-status-mobile = Wi-Fi et données mobiles, sans limite
more-dl-status-mobile-cap = Wi-Fi et données mobiles, limité à { $cap }
more-stream-status-auto = Auto, 720p en mobile ou via relais
more-stream-status-original = Le fichier d'origine, toujours
more-stream-status-converted = Converti en { $quality }
more-sites-status-none = Aucun dépôt
more-sites-status-on = { $sites ->
    [one] 1 site actif
   *[other] { $sites } sites actifs
}
more-sites-status-on-from = { $sites ->
    [one] 1 site actif
   *[other] { $sites } sites actifs
}, { $repos ->
    [one] d'un dépôt
   *[other] de { $repos } dépôts
}
more-conn-status-unchecked = Pas encore vérifiée
more-conn-status-ok = Toutes les étapes ont réussi
more-conn-status-failed = { $total ->
    *[other] { $failed } étapes sur { $total } en échec
}
more-computer-status-none = Pas encore connecté
more-icon-default = Par défaut
more-icon-dark = Sombre
more-icon-line = Trait fin
more-subs-status = Taille, position, police, couleurs et fond
more-data-status = Sauvegarde, restauration, cache d'images
more-activity-status = Ce qu'AniBeam a fait, du plus récent
more-look-status-phone = Suit le téléphone
more-stats-status = { $count ->
    [one] { $hours } heure vue en { $year }
   *[other] { $hours } heures vues en { $year }
}
more-stats-status-empty = Une année de visionnage

## More: the sections' own settings

more-accounts-note = Ce que vous regardez ici met vos listes à jour, et leur progression s'affiche sur vos séries.
more-storage = Stockage
more-storage-capped = Limité à { $cap }
more-storage-no-cap = Illimité
more-storage-used = pour les épisodes
more-storage-cap-caption = limite
more-storage-free = libres
more-set-storage-cap = Limite de stockage
more-storage-cap-on = À { $cap }, plus rien ne se télécharge tant que vous ne supprimez pas d'épisodes.
more-storage-cap-off = Désactivée : AniBeam télécharge jusqu'à remplir le téléphone.
more-network = Réseau
more-network-wifi = Wi-Fi seul
more-network-mobile = Wi-Fi et données mobiles
more-wifi-on = Activé. Les téléchargements attendent le Wi-Fi ; le mobile demande d'abord.
more-set-stream-quality = Qualité de diffusion
more-stream-question = Comment un épisode de votre ordinateur se lit ici.
more-stream-auto-note = Auto lit le fichier tel quel en Wi-Fi, et converti en 720p en données mobiles ou quand Tailscale joint votre ordinateur par un relais.
more-stream-original-note = Le fichier tel quel, toujours. Les gros fichiers demandent une connexion rapide.
more-stream-converted-note = Converti sur votre ordinateur en { $quality }. Consomme moins de données ; l'avance rapide prend un instant.
more-set-feeds = Vos propres flux
more-feeds-note = Interrogés avant les sites
more-set-check = Vérifier la connexion
more-check-note = Teste chaque étape entre ce téléphone et les sites de recherche : les trouver, les joindre, et une connexion sécurisée.
more-check-copy = Copier les résultats
more-set-computer = AniBeam sur votre ordinateur
more-computer-note = Sa bibliothèque s'affiche dans Bibliothèque, et ses épisodes se lisent ici. Un nom comme mon-ordinateur ou une adresse comme 100.64.0.1.
more-computer-placeholder = mon-ordinateur:8790
more-subs-note = Ce par quoi commence chaque épisode de ce téléphone. Un épisode diffusé depuis votre ordinateur commence avec les réglages de cet ordinateur.
more-icon-note = L'icône sur votre écran d'accueil et dans la liste des applis.
more-icon-line-note = Trait fin reprend les couleurs de votre fond d'écran, comme vos autres icônes Material You.
more-icon-moved-note = Certains lanceurs retirent l'icône de l'écran d'accueil quand elle change. Elle reste dans la liste des applis ; faites-la glisser de là.
more-set-whats-new = Nouveautés
more-whats-new-note = Ce qui a changé à chaque version, du plus récent.
more-set-onboarding = Reconfigurer
more-onboarding-note = Refaire les questions du premier lancement.
more-set-version = Version

## Status lines Rust writes on the pages

host-synced = Synchronisé { $when }.
host-last-try = Dernier essai { $when } : { $error }
host-asking = Demande...
host-reached = { $host } joint, AniBeam { $version }.
host-saved = Enregistré. Synchronisation...
acct-signed-in-as = Connecté : { $name }
acct-signed-in = Connecté
acct-not-in-build = Indisponible dans cette version
acct-tap-to-sign-in = Non connecté. Touchez pour vous connecter.
acct-sign-in-failed = Connexion échouée : { $why }. Touchez pour réessayer.
acct-sign-in-not-started = La connexion n'a pas démarré : { $error }. Touchez pour réessayer.
acct-still-starting = AniBeam démarre encore. Réessayez dans un instant.
home-recently-watched = Vu récemment
home-recently-downloaded = Téléchargé récemment
home-note-empty = Sur ce téléphone
home-note = { $count ->
    [one] Sur ce téléphone, 1 série
   *[other] Sur ce téléphone, { $count } séries
}
lib-this-phone = Ce téléphone
lib-note-phone = { $count ->
    [one] Ce téléphone, 1 série
   *[other] Ce téléphone, { $count } séries
}
lib-note-remote = { $device }, synchronisé { $when }
lib-empty-search = Aucun résultat pour « { $query } ».
lib-empty-phone = Rien sur ce téléphone pour l'instant.
lib-empty-remote = Cet appareil n'a rien dans sa bibliothèque.
cal-view-airing = En diffusion
cal-note-none = Rien n'est diffusé cette semaine
cal-note = { $count ->
    [one] 1 épisode dans les 7 prochains jours
   *[other] { $count } épisodes dans les 7 prochains jours
}
airing-asking = Demande à AniList
airing-asking-newest = Interrogation d'AniList pour les nouveautés
phone-airing-filter-all = Tout { $count }
airing-filter-listed = Mes listes { $count }
airing-filter-owned = Ici { $count }
airing-this-season = Cette saison
airing-next-season = Suivante
airing-other-season = Autre
airing-hidden = { $count ->
    *[other] { $count } masquées, sans intérêt
}
airing-plan-needs-tracker = Pour ajouter à voir, il faut AniList ou MAL. Connectez-vous dans Plus, Comptes.
airing-plan-failed = L'ajout à voir a échoué : { $error }
list-watching = En cours
list-planning = À voir
list-completed = Terminé
list-paused = En pause
list-dropped = Abandonné
list-repeating = Revisionnage
more-check-all-ok = Toutes les étapes ont réussi.
more-check-failed = { $failed } étapes sur { $total } en échec. Une capture de cette liste indique où.
more-check-row = { $step }  { $ms } ms
more-check-copied = Copié. Collez-le dans un message.
more-version = AniBeam pour Android { $version } (build { $build })
dl-note-waiting = En attente du Wi-Fi. Les téléchargements reprennent seuls dès que le téléphone est en Wi-Fi.
dl-note-nothing = Rien en cours.
dl-state-done = Terminé
dl-state-waiting = En attente du Wi-Fi
dl-state-paused = En pause
dl-state-peers = Recherche de pairs
dl-state-checking = Vérification
dl-state-error = Arrêté par une erreur
dl-state-downloading = Téléchargement
dl-amount = { $got } sur { $size }
dl-speed = { $rate }/s
dl-left = { $span } restantes
dl-episodes-range = Épisodes { $from } à { $to }
dl-batch = Un lot
disk-storage-line = { $used } utilisés
disk-storage-line-free = { $used } utilisés, { $free } libres
disk-free = Ce téléphone a { $free } libres.
disk-free-below-cap = Ce téléphone a { $free } libres, moins que la limite : le téléphone sera plein avant qu'AniBeam s'arrête.

## More's search: the words a person might type for each setting

more-set-subs-size = Taille des sous-titres
more-set-subs-style = Style des sous-titres
more-words-anilist = anilist suivi connexion connecter compte liste progression
more-words-mal = mal myanimelist suivi connexion connecter compte liste progression
more-words-storage-cap = espace disque limite go plein stockage supprimer taille
more-words-wifi = wifi wi-fi mobile données cellulaire réseau
more-words-notifications = notification notifier alerte progression autorisation autoriser
more-words-stream-quality = auto original 1080p 720p 480p convertir données ordinateur lecture diffusion
more-words-check-repos = actualiser mise à jour dépôt repo endpoint
more-words-add-repo = dépôt repo endpoint confiance empreinte adresse url
more-words-add-site = personnalisé endpoint flux rss torznab jackett prowlarr adresse
more-words-feeds = flux feed rss source retirer
more-words-check = réseau dns test https sécurisé joindre internet
more-words-computer = tailscale tailnet hôte adresse bureau bibliothèque diffusion test enregistrer appareil
more-words-whats-new = nouveautés changelog notes changements mise à jour
more-words-onboarding = premier lancement recommencer configuration
more-words-version = build version à propos
more-words-stats = heures épisodes genres studios saisons année vu bilan mois historique
more-words-backup = sauvegarde exporter fichier enregistrer copier déplacer ordinateur json privé connexions
more-words-restore = restaurer importer fichier charger bibliothèque fusionner
more-words-cache = images affiches bannières effacer stockage espace cache
more-words-activity = journal événements erreurs problèmes copier effacer historique
more-words-colours = apparence thème sombre clair mode système accent couleur
more-words-text-size = apparence police plus grand plus petit taille texte lire
more-words-icon = icône lanceur écran d'accueil logo sombre trait material you
more-words-subs-size = sous-titres taille échelle position remonter bas
more-words-subs-style = sous-titres police couleur contour ombre fond gras ass stylisé réinitialiser
more-words-language = langue traduction anglais français

## Home

phone-home-title = Accueil
home-next-to-air = Prochaine sortie
home-after-that = Ensuite
home-coming-up = À venir
home-nothing-week = Rien la semaine prochaine.
phone-home-empty-title = Rien sur ce téléphone
phone-home-empty-body = Les séries que vous téléchargez ici, et ce qui sort pour elles, s'affichent dans Accueil. Trouvez-en une dans Téléchargements.

## Library

lib-title = Bibliothèque
lib-refresh = Actualiser
lib-search-phone = Chercher ici
lib-search-device = Chercher sur { $device }
lib-find-in-downloads = Chercher « { $query } » dans Téléchargements
lib-empty-body = Les épisodes téléchargés sur ce téléphone arrivent ici. Trouvez une série dans Téléchargements.
lib-pick-show = Choisissez une série pour l'ouvrir ici.
lib-shows-caption = { $count ->
    [one] série
   *[other] séries
}
lib-on-phone = sur mobile
lib-free = libres
lib-hold-poster = Maintenez une affiche pour obtenir d'autres épisodes ou les supprimer.
phone-nothing-on-phone = rien sur mobile
phone-on-phone = { $count } sur mobile

## Calendar

cal-title = Calendrier
cal-week = Semaine
cal-days = Jours
cal-weekday-short = %a
cal-weekday = %A
cal-day-full = %A %-d %B
cal-today = Aujourd'hui
cal-tomorrow = Demain
cal-yesterday = Hier
cal-nothing = Rien
cal-next = Suivant
cal-get = Obtenir
cal-day-note = { $date }, { $count }
cal-next-airing = Prochaine diffusion : { $title }
cal-nothing-aired-yesterday = Rien n'a été diffusé hier.
cal-nothing-aired-that-day = Rien n'a été diffusé ce jour-là.
cal-nothing-today = Rien n'est diffusé aujourd'hui.
cal-nothing-tomorrow = Rien n'est diffusé demain.
cal-nothing-that-day = Rien n'est diffusé ce jour-là.
cal-empty-title = Aucune série sur ce téléphone
cal-empty-body = Les épisodes des séries gardées ici s'affichent le jour de leur sortie, deux jours en arrière et une semaine en avant.
phone-episodes-range = Épisodes { $from } à { $to }
phone-episode-day = { $episode }, { $day }
phone-on-the-phone = Sur le téléphone
phone-out-ago = Sorti { $ago }

## Airing now

phone-airing-out-now = Sorti
airing-last-two-days = 2 derniers jours
airing-season-note = Les séries de la saison viennent d'AniList.
airing-none-lists-title = Rien en cours de diffusion dans vos listes
airing-none-lists-body = À voir sur une série l'ajoute à votre liste.
airing-none-phone-title = Rien en cours de diffusion sur ce téléphone
airing-none-phone-body = Obtenir des épisodes d'une série la télécharge ici.
airing-marked = { $title } est marquée sans intérêt.
phone-airing-undo = Annuler
airing-plan = À voir
airing-get-more = Plus d'épisodes
airing-get = Obtenir des épisodes
phone-airing-not-interested = Sans intérêt

## Seasons

season-winter = Hiver
season-spring = Printemps
season-summer = Été
season-fall = Automne
season-named = { $season } { $year }
season-pick = Choisir une saison

## A show's page

series-back-home = Accueil
series-back-library = Bibliothèque
series-back-downloads = Téléchargements
series-back-calendar = Calendrier
phone-series-get-episodes = Obtenir des épisodes
phone-series-episodes = Épisodes
series-delete-episodes = Supprimer des épisodes
series-delete-tap = Touchez les épisodes à supprimer.
phone-series-delete-picked = { $count ->
    [one] { $count } épisode choisi.
   *[other] { $count } épisodes choisis.
}
series-delete-note = Leurs fichiers sont supprimés de ce téléphone. Ce que vous avez vu reste suivi.
series-delete-button = { $count ->
    [one] Supprimer { $count } épisode
   *[other] Supprimer { $count } épisodes
}
series-no-episodes = Aucun épisode sur ce téléphone, et aucun prévu.
series-versions = { $count ->
    [one] { $count } version
   *[other] { $count } versions
}
series-note-next-up = À suivre
series-note-missing = Manquant
series-note-not-here = Pas sur ce téléphone
series-note-not-downloaded = Non téléchargé
series-note-downloading = Téléchargement { $percent } %
series-note-resume-at = Reprendre à { $time }
phone-episodes-estimate = { $count ->
    [one] { $count }? épisode
   *[other] { $count }? épisodes
}
phone-next-airs = Épisode { $n } dans { $time }
phone-watched = Vu
phone-not-watched = Pas encore vu
phone-progress = { $watched } sur { $total } vus, { $count } sur ce téléphone
phone-on-this-phone = { $count } sur ce téléphone
phone-play-film = Lire le film
phone-resume-film = Reprendre le film
phone-play-episode = Lire l'épisode { $n }
phone-resume-episode = Reprendre l'épisode { $n }
phone-about-date = vers le { $date }

## A show's list status, score, notes and fixes

show-status-watching = En cours
show-status-planning = À voir
show-status-completed = Terminé
show-status-paused = En pause
show-status-dropped = Abandonné
show-status-repeating = Revisionnage
show-status-now = { $label }, actuel
phone-show-add-to-list = Ajouter à une liste
phone-show-your-score = Votre note { $score }
show-rate = Noter
show-fixes = Corriger
show-your-notes = Vos notes
show-add-notes = Annoter
show-home-each = Modifiez-le sur chaque liste connectée.
show-home-no-tracker = Gardé sur ce téléphone : aucun suivi ne propose cette série.
show-home-tracker-lacks = Gardé sur ce téléphone : votre suivi n'a pas cette série.
show-home-until-connected = Gardé sur ce téléphone jusqu'à la connexion d'AniList ou MyAnimeList.
show-sites-both = AniList et MyAnimeList
show-notes-title = Notes
show-notes-placeholder = Où vous en êtes, qui l'a recommandée, quelle release prendre
show-notes-over = { $count ->
    [one] { $count } caractère en trop
   *[other] { $count } caractères en trop
}
show-notes-saved-to = Enregistré sur { $sites }.
show-notes-saved-to-tracker = Enregistré sur votre suivi.
show-notes-saving = Enregistrement de vos notes.
show-notes-clearing = Effacement de vos notes.
show-score-saving = Enregistrement de votre note.
show-score-saved = Note enregistrée.
show-score-clearing = Effacement de votre note.
show-score-cleared = Note effacée.
phone-show-moving-to = Déplacement vers { $label }.
phone-show-moved-to = Déplacé vers { $label }.
show-rescan-started = Nouvelle analyse lancée.
show-still-starting = AniBeam démarre encore. Réessayez dans un instant.
show-fix-match = Associer
show-fix-find-match = Trouver la série
show-fix-rescan = Réanalyser

## A show's characters, related shows and recommendations

show-characters = Personnages
phone-show-related = Liées
show-related-all = Tout voir ({ $count })
show-related-filling = AniList complète encore la liste.
phone-show-recommendations = Recommandations
show-card-owned = Possédé
show-card-unreleased = Pas encore sortie
show-card-on-anilist = Sur AniList
show-rec-available = Disponible
show-relation-prequel = Préquelle
show-relation-sequel = Suite
show-relation-side-story = Annexe
show-relation-parent = Œuvre mère
show-relation-spin-off = Spin-off
show-relation-adaptation = Adaptation
show-relation-alternative = Alternative
show-relation-summary = Résumé
show-relation-character = Personnage
show-relation-source = Source
show-relation-compilation = Compilation
show-relation-contains = Contient
show-relation-other = Autre
show-role-main = Principal
show-role-supporting = Secondaire
show-role-background = Figurant
show-format-tv = TV
show-format-tv-short = TV court
show-format-ova = OVA
show-format-ona = ONA
show-format-special = Spécial
show-format-film = Film
show-entry-get = Obtenir des épisodes
show-entry-open-anilist = Ouvrir sur AniList
show-entry-line = { $line }, { $status }

## Fix a wrong match

phone-match-title = Association
match-refresh = Actualiser
phone-match-clear = Dissocier
match-clear-it = Dissocier
match-clear-warning = Dissocier supprime le lien de suivi et un nombre d'épisodes fixé à la main. Touchez encore pour dissocier.
match-tvmaze = TVmaze
match-search-anilist = Chercher AniList
match-search-tvmaze = Chercher TVmaze
match-paste-anilist = Collez un lien AniList ou MyAnimeList
match-paste-tvmaze = Collez un lien TVmaze ou IMDb
phone-match-look-up = Chercher
match-apply = Appliquer
match-look-up-all = Chercher tous les dossiers qu'AniBeam ne connaît pas encore
match-format-music = Musique
match-join-and = { $first } et { $second }
phone-match-matched-on = Associé sur { $sites }
phone-match-found-tvmaze = Série trouvée sur TVmaze
phone-match-found-anilist = Trouvée sur AniList
phone-match-found-anilist-season = Trouvée sur AniList, saison { $season }
phone-match-found-mal = Trouvée sur MyAnimeList
phone-match-searching = Recherche sur { $name }
phone-match-type-more = Saisissez au moins deux lettres pour chercher sur { $name }.
match-none = Aucun titre trouvé sur { $name }.
match-looking-up = Recherche en cours.
phone-match-matched-to = Associé à { $title }.
phone-match-applied = Association appliquée.
match-applying = Application de l'association.
phone-match-refreshing = Actualisation de { $title }.
match-clearing = Effacement de l'association.
phone-match-cleared = Association effacée.
match-auto-looking = Recherche des dossiers qu'AniBeam ne connaît pas encore.

## Batches (the line under a release in a search)

inspect-reading = Lecture de sa liste de fichiers
inspect-holds = { $count ->
    [one] { $count } épisode : { $runs }
   *[other] { $count } épisodes : { $runs }
}

## What's new

phone-notes-title = Nouveautés
notes-got-it = Compris

## Move the list to a tracker

move-lists-label = Listes sur ce téléphone
move-title = Déplacer votre liste vers { $site }
move-here = Sur ce téléphone : { $list }
move-there = Sur { $site } : { $list }
move-later-hint = Plus tard : Plus, Comptes
move-not-now = Plus tard
move-nothing = Rien
move-not-on-list = Pas dans votre liste
move-nothing-left = Plus rien à déplacer
move-side-status-score = { $status }, note { $score }
move-side-score = note { $score }
move-lead = { $count ->
    [one] Une série a un statut ou une note uniquement sur ce téléphone. La déplacer la met sur votre liste { $site }, que voient votre ordinateur et votre profil. Pour ces séries, ce que { $site } contient est remplacé. Rien n'est envoyé avant d'appuyer sur Déplacer.
   *[other] { $count } séries ont un statut ou une note uniquement sur ce téléphone. Les déplacer les met sur votre liste { $site }, que voient votre ordinateur et votre profil. Pour ces séries, ce que { $site } contient est remplacé. Rien n'est envoyé avant d'appuyer sur Déplacer.
}
move-label = { $count ->
    [one] Déplacer { $count } série
   *[other] Déplacer { $count } séries
}
move-waiting = { $count ->
    [one] Une série a un statut ou une note uniquement ici. Déplacez-la vers { $site } pour la voir sur votre ordinateur et votre profil.
   *[other] { $count } séries ont un statut ou une note uniquement ici. Déplacez-les vers { $site } pour les voir sur votre ordinateur et votre profil.
}
move-moving = { $count ->
    [one] Déplacement de { $count } série vers { $site }
   *[other] Déplacement de { $count } séries vers { $site }
}
move-moved = { $count ->
    [one] { $count } série déplacée vers { $site }
   *[other] { $count } séries déplacées vers { $site }
}
move-failed = { $count ->
    [one] { $site } n'a pas pris { $count } série ; voir le journal d'activité
   *[other] { $site } n'a pas pris { $count } séries ; voir le journal d'activité
}

## Downloads: find (search, shows, sources, releases)

find-join-comma = { $a }, { $b }
find-join-and = { $a } et { $b }
find-join-sentences = { $a } { $b }

find-downloads = Téléchargements

find-lens-list = Ma liste
find-lens-gaps = Manques
find-lens-airing = Diffusion
find-lens-following = Suivies
find-lens-downloaded = Téléchargées
find-lens-with-count = { $lens } { $count }
find-what-to-get = À obtenir
find-lens-list-line = En cours et À voir dans votre liste. Ouvrez une série pour obtenir ses épisodes.
find-lens-gaps-line = Épisodes diffusés absents de ce téléphone. Ouvrez une série pour voir qui les a.
find-lens-airing-line = Épisodes à venir, les plus proches d'abord.
find-lens-following-line = Les abonnements : les nouveaux épisodes se téléchargent seuls. Ouvrez-en un pour l'arrêter.
find-lens-downloaded-line = Ce que ce téléphone a téléchargé, série par série.

find-lens-gaps-empty = Aucun manque : chaque série commencée a ici ses épisodes diffusés.
find-lens-airing-empty = Rien sur ce téléphone n'est en diffusion.
find-lens-following-empty = Aucun abonnement. Ouvrez une série et activez Suivre.
find-lens-downloaded-empty = Rien de téléchargé pour l'instant.
find-list-empty = Rien dans votre liste pour l'instant. Cherchez une série ci-dessus et ajoutez-la.
find-list-reading = Lecture de votre liste...
find-list-signed-out = Connectez-vous à AniList dans Plus, Comptes, et votre liste s'affichera ici.
find-list-failed = Impossible de lire votre liste : { $error }

find-aired = diffusés
find-ledger-no-feed = aucun nouvel épisode : sa source n'a pas de flux
find-ledger-downloads = { $count ->
    [one] { $count } téléchargement
   *[other] { $count } téléchargements
}
find-ledger-downloaded = téléchargés
find-ledger-planning = À voir
find-ledger-watching-of = En cours, { $progress } sur { $total }
find-ledger-watching-seen = En cours, { $progress } vus

find-gap-missing = { $count } manquants
find-gap-not-downloaded = { $count } non téléchargés

find-list-status-planning = Dans votre liste : À voir
find-list-status-watching = Dans votre liste : En cours
find-list-status-already = Déjà dans votre liste
find-going-planning = { $title } passe en À voir.
find-already-listed = { $title } est déjà dans votre liste.
find-plan-needs-tracker = Pour ajouter à votre liste, il faut AniList ou MAL. Connectez-vous dans Plus, Comptes.
find-plan-failed = Pas ajouté à votre liste : { $error }
find-show-gone = Cette série n'est plus dans la bibliothèque de ce téléphone.

## Shows as you type

find-shows = Séries
find-suggesting = Interrogation d'AniList...
find-suggest-tap = Touchez-en une pour voir qui l'a. Entrée cherche des noms de releases.
find-suggest-none = Aucune série de ce nom. Entrée cherche plutôt des noms de releases.
find-suggest-hint = Les séries s'affichent pendant la saisie. Entrée cherche des noms de releases.
find-suggest-failed = Impossible d'interroger AniList : { $error }
find-library-line = En bibliothèque
find-library-line-none = En bibliothèque, aucun des { $total } sur ce téléphone
find-library-line-some = En bibliothèque, { $here } sur { $total } sur ce téléphone

## The show view

find-plan-to-watch = À voir
find-on-phone-count = { $count } sur ce téléphone
find-airing = En diffusion
find-dropped = { $count ->
    [one] { $count } release qui ne nomme pas cette série est écartée.
   *[other] { $count } releases qui ne nomment pas cette série sont écartées.
}

find-following = Suivie
find-stop-following-ask = Ne plus suivre { $name } ?
find-unfollow-explain = Les nouveaux épisodes ne se téléchargent plus. Ce qui est dans son dossier peut rester, ou partir avec lui : tous les téléchargements qu'il contient, épisodes pris à la main compris.
find-stop-following = Ne plus suivre
find-stop-keep = Arrêter, garder les épisodes
find-stop-delete = Arrêter et supprimer les téléchargements du dossier
find-rule-taken = { $downloads ->
    [one] { $downloads } téléchargement
   *[other] { $downloads } téléchargements
}, { $files ->
    [one] { $files } fichier
   *[other] { $files } fichiers
} dans son dossier
find-rule-paused = Il est en pause, donc aucun nouvel épisode n'arrive. { $taken }.
find-rule-no-feed = Sa source n'a pas de flux, donc aucun nouvel épisode n'arrive. { $taken }.
find-rule-active = Les nouveaux épisodes se téléchargent seuls. { $taken }.
find-unfollowed-deleted = { $name } n'est plus suivie, et les téléchargements de son dossier sont supprimés.
find-unfollowed-kept = { $name } n'est plus suivie. Ses épisodes restent sur ce téléphone.
find-stop-failed = L'arrêt n'a pas abouti : { $error }

find-where = Où l'obtenir
find-where-pick = Choisissez une source pour voir ce qu'elle a.
find-where-nothing = Rien ne se télécharge avant votre accord.
find-source-idle = Interroger
find-source-trying = Essai de { $name }
find-source-searching = Recherche
find-source-silent = Ne répond pas
find-no-sites-title = Aucun site de recherche
find-no-sites-body = Ajoutez un dépôt de sites de recherche, ou un flux à vous, dans Plus.
find-open-more = Ouvrir Plus
find-pick-source-title = Choisir une source
find-pick-source-body = Ses releases s'affichent ici en grille, un cube par épisode.
find-releases = { $count ->
    [one] { $count } release
   *[other] { $count } releases
}
find-unreachable = Impossible de joindre { $host }. Certains réseaux et pays bloquent les sites de releases ; AniBeam contourne déjà les blocages de noms, donc ici le site lui-même est bloqué, ou il est en panne. Un VPN le contourne, ou essayez une autre source.
find-the-source = La source

find-has = Ce qu'elle a
find-has-note = Touchez une ligne pour tout prendre, ou des épisodes seuls.
find-episodes-tag = Épisodes
find-batch = Lot { $label }
find-row-meta = { common-episodes }  { $seeds } seeds  { $size }
find-on-this-phone = Sur ce téléphone
find-on-computer = Sur { $name }

find-plan-no-releases = Cette source n'a aucune release de cette série.
find-plan-nothing-picked = Rien de choisi : touchez une ligne ou ses cubes.
find-plan-line = { $episodes } de { $groups }, { $size }
find-plan-skipped = { $count ->
    [one] { $count } épisode que vous avez déjà est écarté. Activez Remplacer pour le reprendre.
   *[other] Les épisodes que vous avez déjà sont écartés. Activez Remplacer pour les reprendre.
}
find-plan-follow-note = Les nouveaux épisodes de { $query } se téléchargeront seuls.
find-plan-on-desk = Ils se téléchargent sur { $name }, dans sa bibliothèque.
find-follow-new = Suivre les nouveautés
find-follow-new-note = Ils se téléchargent seuls à leur sortie.
find-replace = Remplacer l'existant
find-replace-note = Retélécharge les épisodes choisis par-dessus ceux de ce téléphone.
find-go = { $mode ->
    [replace] Obtenir { common-episodes } et remplacer
    [subscribe] Obtenir { common-episodes } et s'abonner
    [both] Obtenir { common-episodes }, remplacer et s'abonner
   *[plain] Obtenir { common-episodes }
}
find-go-pick = Choisissez des épisodes
find-every-release = Toutes les releases de cette source

## Download to

find-download-to = Destination
find-this-phone = Ce téléphone
find-your-computer = Votre ordinateur
find-desk-asking = Demande à { $name }...
find-desk-silent = { $name } ne répond pas. Choisissez Ce téléphone, ou réessayez quand il est allumé.
find-sending-to = Envoi à { $name }...

## The release list

find-best-fit = Conseillé
find-on-disk = Sur disque
find-get = Obtenir
find-get-all = Tout obtenir ({ $count })
find-subscribe = S'abonner
find-back-step = Reculer
find-narrow-placeholder = Affiner, ou -mot pour exclure
find-best-fit-searches = Conseillé : recherche de { $query }
find-fact-batch = lot
find-fact-seeds = { $count } seeds
find-fact-via = via { $source }
find-searching-for = Recherche de { $text }...
find-search-line = { $count ->
    [0] Rien sur { $source } pour cela
    [one] { $count } release sur { $source }
   *[other] { $count } releases sur { $source }
}{ $fell ->
    [1] , la première source n'ayant rien donné
   *[other] { "" }
}{ $narrowed ->
    [1] , affiné sur { $token }
   *[other] { "" }
}.
find-refined = { find-releases }, { $dir ->
    [down] { $delta } de moins
   *[up] { $delta } de plus
}.
find-refine-nothing-left = Il ne reste rien.
find-refine-no-match = Aucune release n'a cela.
find-refine-empty = Saisissez d'abord quelque chose.
find-refine-no-change = Cela ne change rien.

## Getting and subscribing

find-getting = Obtention de { $what }.
find-subscribed = Abonné : les nouveaux épisodes se téléchargent seuls.
find-subscribed-desk = Abonné sur { $name } : les nouveaux épisodes s'y téléchargent seuls.
find-subscribed-as = Abonné sous { $name } : les nouveaux épisodes se téléchargent seuls.
find-downloading = { $count ->
    [one] Téléchargement de { $count } release.
   *[other] Téléchargement de { $count } releases.
}
find-not-added = Pas ajouté : { $error }
find-not-subscribed = Pas abonné : { $error }
find-desk-refused = { $name } ne l'a pas pris : { $error }
find-desk-not-subscribed = Pas abonné sur { $name } : { $error }

## Player

player-subtitles = Sous-titres
player-audio = Audio
phone-player-speed = Vitesse
player-connecting = Connexion
player-hold-speed = Vitesse 2×
player-replay = Revoir
player-unlock = Déverrouiller
player-next-stay = Rester
player-next-go = Suivant
player-jump-back = −10 s
player-jump-forward = +10 s
player-speed-now = Vitesse { $speed }
player-skip-intro = Passer l'intro
player-skip-outro = Passer l'outro
phone-player-skipped-intro = Intro passée
phone-player-skipped-outro = Outro passé
phone-player-undo = Annuler
phone-player-start-over = Au début
phone-player-resumed-at = Repris à { $time }
player-picture-failed = L'image n'a pas pu démarrer. Le son continue.
player-file-failed = Ce fichier n'a pas pu être lu : { $why }.
player-streaming-from = Diffusion depuis { $host } en { $height }p
player-mpv-no-start = mpv n'a pas démarré
player-mpv-no-start-why = mpv n'a pas démarré : { $why }
player-android-only = le lecteur ne fonctionne que sur Android

## Player: delays

player-delay-title = Décalage
player-delay-note = Chaque appui le déplace de 0,1 s. Jusqu'au prochain fichier.
phone-player-delay-seconds = { $value } s
player-sub-delay-now = Décalage des sous-titres { $value }
player-audio-delay-now = Décalage audio { $value }

## Player: cast

player-cast-playing-on = Lecture sur { $device }
phone-player-cast-stop = Arrêter la diffusion vers { $device }
player-cast-searching = Recherche de Chromecast et de TV
phone-player-cast-to = Diffuser vers
phone-player-cast-tv = TV
phone-player-cast-loading = Chargement
phone-player-cast-playing = Lecture
phone-player-cast-paused = En pause
phone-player-cast-finished = Terminé
phone-player-cast-stopped = Arrêté

## Player: the tracker's word

phone-player-rated = Noté
phone-player-score-failed = Impossible d'enregistrer la note
player-marked-watched = Marqué comme vu
player-tracked = Suivi
player-not-tracked = Non suivi
player-track-update-failed = Impossible de mettre à jour votre liste
player-not-tracked-why = Non suivi : { $reason }.
player-track-ok-at = { $tracker } à { $progress }
player-track-ok = { $tracker } ok
player-track-note = { $tracker } : { $note }
player-track-failed = Impossible de mettre à jour { $tracker }
player-track-line = { $head } : { $parts }
phone-player-refusal-hidden = la série est masquée
phone-player-refusal-no-match = la série n'a pas d'association de suivi
phone-player-refusal-not-newer = le suivi est déjà au-delà de cet épisode
phone-player-refusal-extra = c'est un bonus
phone-player-refusal-unmatched = le fichier n'est pas associé à un épisode
phone-player-refusal-on-disk = le fichier n'existe que sur le disque

## Subtitles

subs-line-default = Taille 1×, les fichiers simples prennent votre style
subs-sample = Où l'avez-vous caché ?
subs-size-place-title = Taille et position
subs-size = Taille
subs-position = Position
subs-position-helper = 100 colle au bord inférieur. Les valeurs plus basses remontent les lignes dans l'image.
subs-style-title = Style
subs-styled = Sous-titres stylisés
subs-styled-as-made = Les fichiers stylisés (ASS) s'affichent exactement comme leurs auteurs les ont réglés.
subs-styled-mine = Votre style ci-dessous s'applique à chaque fichier. Les panneaux et le karaoké peuvent mal s'afficher.
subs-styled-size-only = Les fichiers stylisés gardent leur look et prennent votre taille. Les fichiers simples prennent votre style.
subs-mode-as-made = D'origine
subs-mode-size-only = Taille seule
subs-mode-mine = Le mien
subs-font = Police
subs-font-bad = Saisissez un nom de police, comme Noto Sans.
subs-bold = Gras
subs-colour = Couleur
subs-colour-bad = Saisissez une couleur comme #ffcc00.
subs-outline = Contour
subs-outline-bad-width = L'épaisseur est un nombre, 0 ou plus.
subs-outline-bad-colour = Saisissez la couleur comme #000000.
subs-outline-helper = Son épaisseur, puis sa couleur.
subs-shadow = Ombre
subs-shadow-bad = Un nombre, 0 ou plus. 0 veut dire sans ombre.
subs-shadow-helper = La distance de l'ombre, 0 pour aucune.
subs-box = Boîte de fond
subs-box-helper = Une boîte sombre derrière les lignes. 0 la supprime.
subs-reset-title = Recommencer
subs-reset-helper = Remet la taille, la police, les couleurs, le contour, la boîte et la position comme à l'origine.
subs-reset = Réinitialiser

## Streaming

stream-quality-auto = Auto
stream-quality-original = Original
stream-app-starting = L'appli démarre encore
stream-off-on-host = La diffusion est désactivée sur { $host }. Activez Diffuser les épisodes là-bas, dans Réglages, Appli mobile.
stream-cannot-stream = { $host } exécute AniBeam { $version }, qui ne sait pas encore diffuser. Mettez-le à jour là-bas.
stream-no-session = L'autre appareil n'a donné aucune session
stream-answer-unreadable = La réponse n'a pas pu être lue : { $why }
stream-answered-status = { $name } a répondu { $status }

## Stats

phone-stats-nothing-yet = Rien encore
phone-stats-empty-title = Rien de vu pour l'instant
phone-stats-empty-body = Terminez un épisode et il compte ici : heures, genres et saisons vues.
stats-hours-caption = heures vues
stats-finished-caption = séries terminées
stats-busiest-caption = mois record
stats-episodes-caption = { $count ->
    [one] épisode
   *[other] épisodes
}
phone-stats-by-month = Par mois
phone-stats-top-genres = Genres favoris
phone-stats-top-studios = Studios favoris
stats-seasons = Saisons vues
stats-watched-most = Le plus vu
stats-year-line = Ce que vous avez vu en { $year }
stats-eps = { $count ->
    [one] { $count } ép.
   *[other] { $count } ép.
}
stats-month-1 = janv.
stats-month-2 = févr.
stats-month-3 = mars
stats-month-4 = avr.
stats-month-5 = mai
stats-month-6 = juin
stats-month-7 = juil.
stats-month-8 = août
stats-month-9 = sept.
stats-month-10 = oct.
stats-month-11 = nov.
stats-month-12 = déc.
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

backup-title = Sauvegarde
backup-export = Sauvegarder
backup-export-helper = Un fichier avec vos dossiers, associations et ce que vous avez vu.
backup-private = Inclure les données privées
backup-private-helper = Ajoute les connexions AniList et MyAnimeList, l'historique et les préférences. Les connexions sont écrites en clair : gardez le fichier pour vous.
backup-restore = Restaurer une sauvegarde
backup-restore-helper = Lit une sauvegarde dans cette bibliothèque. Ce qui est sur le disque l'emporte ; le fichier complète le reste.
backup-done = Sauvegardé. Le fichier est là où vous l'avez enregistré.
backup-not-saved = Non enregistré. Rien n'a été écrit.
backup-starting = AniBeam démarre encore. Réessayez dans un instant.
backup-images-title = Cache d'images
backup-images-body = Affiches, bannières et portraits restent sur le téléphone pour que la bibliothèque marche hors ligne. Les images effacées se retéléchargent selon les pages.
backup-keep = Garder
backup-removed = { $count ->
    [one] { $count } image supprimée
   *[other] { $count } images supprimées
}
backup-images-none = Aucune image en cache
backup-images-unmeasured = Impossible de mesurer le cache d'images
backup-images-retry-note = Réessayez dans un instant. Les images elles-mêmes vont bien.
backup-images-line = { $size } pour { $count ->
    [one] { $count } image
   *[other] { $count } images
}
backup-images-question = { $count ->
    [one] Supprimer { $count } image ?
   *[other] Supprimer { $count } images ?
}

backup-sum-list = { $items }.
backup-sum-sep = { ", " }
backup-sum-folders = { $count ->
    [one] { $count } dossier ajouté
   *[other] { $count } dossiers ajoutés
}
backup-sum-series = { $count ->
    [one] { $count } série créée
   *[other] { $count } séries créées
}
backup-sum-matches = { $count ->
    [one] { $count } association appliquée
   *[other] { $count } associations appliquées
}
backup-sum-views = { $views ->
    [one] { $views } visionnage
   *[other] { $views } visionnages
} et { $episodes ->
    [one] { $episodes } épisode terminé
   *[other] { $episodes } épisodes terminés
} fusionnés
backup-sum-resume = { $count ->
    [one] { $count } point de reprise fusionné
   *[other] { $count } points de reprise fusionnés
}
backup-sum-accounts = { $count ->
    [one] { $count } compte de liste importé
   *[other] { $count } comptes de liste importés
}
backup-sum-fixes = { $count ->
    [one] { $count } fichier prend une correction que vous avez faite.
   *[other] { $count } fichiers prennent une correction que vous avez faite.
}
backup-sum-missing = { $count ->
    [one] { $count } dossier est introuvable sur ce téléphone.
   *[other] { $count } dossiers sont introuvables sur ce téléphone.
}
backup-sum-left-out = Ignoré : { $fields }.

## Activity

activity-everything = Tout
activity-problems = Problèmes
activity-copied = Copié
activity-clear-every-line = Tout effacer
activity-fold = Replier
activity-no-problems = Aucun problème. Tout a fonctionné.
activity-nothing-yet = Rien à signaler pour l'instant. Ce qu'AniBeam fait pour vous s'affiche ici.
activity-time-today = %H:%M:%S
activity-time-other = %-d %b %H:%M

## Dates and numbers (companion format.rs)

fmt-clock = %H:%M
fmt-day-month = %-d %b
fmt-weekday-date-time = %a %-d %b %H:%M
fmt-weekday-date-year-time = %a %-d %b %Y %H:%M
fmt-stamp = %-d %b %H:%M
fmt-decimal-point = ,

fmt-today-at = Aujourd'hui { $time }
fmt-tomorrow-at = Demain { $time }
fmt-yesterday-at = Hier { $time }

fmt-span-days = { $d } j { $h } h
fmt-span-hours = { $h } h { $m } min
fmt-span-minutes = { $m } min { $s } s
fmt-span-seconds = { $s } s
phone-fmt-in = dans { $span }
fmt-full-days = { $d } j { $h } h { $m } min { $s } s
fmt-full-hours = { $h } h { $m } min { $s } s
fmt-full-minutes = { $m } min { $s } s

fmt-ago-now = à l'instant
fmt-ago-minutes = il y a { $count } min
fmt-ago-hours = il y a { $count } h
fmt-ago-days = il y a { $count } j

fmt-bytes-b = { $value } o
fmt-bytes-kb = { $value } Ko
fmt-bytes-mb = { $value } Mo
fmt-bytes-gb = { $value } Go
fmt-bytes-tb = { $value } To

## Talking to the computer (companion client.rs, sync.rs, address.rs)

client-not-found = { $host } n'est pas un nom que le téléphone trouve. Tailscale est-il connecté ?
client-timeout = { $host } ne répond pas. Est-il allumé, et Tailscale connecté des deux côtés ?
client-connect-failed = Impossible de joindre { $host }. Est-il allumé, et Tailscale connecté des deux côtés ?
client-refused = { $host } est allumé, mais AniBeam y est fermé ou ne partage pas. Ouvrez AniBeam dessus et activez Partager via Tailscale dans Réglages, Appli mobile.
client-no-route = Aucune route vers { $host }. Tailscale est-il connecté ?
client-invalid-input = Impossible de se connecter à { $host }
client-unreachable = Impossible de joindre { $host } : { $why }
client-cannot-find = Le téléphone ne trouve pas { $host }. Tailscale est-il connecté ?
client-broke-off = La réponse de { $host } s'est interrompue : { $why }
client-answered = { $host } a répondu { $status } pour { $path }
client-not-anibeam-status = Quelque chose a répondu, mais pas AniBeam ({ $host } a répondu { $status } pour { $path })
client-not-anibeam-hello = Quelque chose a répondu, mais pas AniBeam (sa réponse n'est pas le hello d'AniBeam)
client-not-anibeam-name = Quelque chose a répondu, mais pas AniBeam (il se nomme { $name })
client-too-new = L'hôte parle l'API { $host_api } ; cette appli lit { $app_api }. Mettez l'appli à jour.
client-snapshot-unreadable = L'instantané n'a pas pu être lu : { $why }
client-probe-unreadable = Les détails du fichier n'ont pas pu être lus : { $why }
client-peers-unreadable = La liste des AniBeam n'a pas pu être lue : { $why }
sync-save-failed = Impossible d'enregistrer l'instantané : { $why }
address-https = L'AniBeam de votre ordinateur répond en http simple via Tailscale ; retirez le https://.
address-empty = Saisissez le nom ou l'adresse de l'hôte, par exemple my-computer:8790.
address-ipv6-close = Une adresse IPv6 doit se fermer par ].
address-ipv6-port = Mettez le port après le ], comme [fd7a::1]:8790.
address-bad-host = « { $text } » n'est pas un nom d'hôte.
address-bad-port = « { $port } » n'est pas un port.

## Series facts (companion library.rs)

clib-status-airing = En diffusion
clib-status-not-yet = À venir
clib-format-tv-short = TV court
clib-format-movie = Film
clib-format-special = Spécial
clib-format-music = Musique

## Search sites and repos

repos-by-hand-title = À la main
repos-by-hand-note = Pour un site qu'aucun dépôt ne propose. Tout flux avec { "{query}" } à l'endroit de la recherche, dont un flux Jackett ou Prowlarr. Une clé ou un jeton dans l'adresse est déplacé dans le coffre de ce téléphone.
repos-name-placeholder = Nom
repos-url-placeholder = https://...{ "{query}" }...
repos-index-placeholder = https://.../index.json
repos-add-site = Ajouter
repos-add-title = Ajouter un dépôt
repos-add-note = Collez l'adresse de l'index d'un dépôt. AniBeam vérifie sa signature et vous montre sa clé avant d'enregistrer quoi que ce soit.
repos-add-repo = Ajouter
repos-trust-title = Approuver ce dépôt ?
repos-fingerprint = Empreinte de la clé
repos-trust-warning = Ses sites décident où vont vos recherches. Ne faites confiance qu'à un dépôt de quelqu'un de confiance.
repos-trust = Approuver
repos-open-link = Ouvrir { $url }
repos-saving = Enregistrement...
repos-install = Installer
repos-update = Mettre à jour
repos-edit-note = Une clé ou un jeton saisi dans l'adresse est déplacé dans le coffre de ce téléphone.
repos-edit-note-keys = Laissez chaque { "{key}" } dans l'adresse tel quel ; sa valeur reste sur ce téléphone.
repos-change-its = Changer { $keys }
repos-check-repos = Chercher de nouveaux sites
repos-check-sites = Vérifier les sites
repos-add-site-label = Ajouter un site à la main
repos-needs-before-search = Demande { $labels } avant de chercher
repos-no-longer-offered = N'est plus proposé par son dépôt
repos-update-host = La mise à jour envoie des requêtes à { $host }
repos-tap-change-custom = Touchez pour changer son nom ou son adresse
repos-tap-change-keys = Touchez pour changer { $labels }
repos-line-custom = { $host }  { $kind }  ajouté par vous
repos-update-to = Passer à v{ $version }
repos-add-its-value = Ajouter sa valeur
repos-asks-for = Demande { $labels }
repos-preview-line = { $count ->
    [one] { $count } site, signé
   *[other] { $count } sites, signés
}
repos-preview-trusted = { $count ->
    [one] { $count } site, signé, déjà de confiance
   *[other] { $count } sites, signés, déjà de confiance
}
repos-ask-kept-empty = Gardé sur ce téléphone uniquement. Un champ laissé vide garde ce qui est enregistré.
repos-ask-kept-never = Gardé sur ce téléphone uniquement, jamais dans la bibliothèque ni dans un export.
repos-ask-new-host = La nouvelle version envoie ceci à { $host }, donc elle le redemande.
repos-ask-new = La nouvelle version redemande ces valeurs.
repos-type-value = Saisissez d'abord une valeur.
repos-health-answered = A répondu en { $ms } ms
repos-health-failed = N'a pas répondu. { $step } : { $detail }

## Downloads

dl-title = Téléchargements
dl-wifi-only = Wi-Fi seul
dl-wifi-and-mobile = Wi-Fi et données mobiles
dl-joined = { $first }, { $second }
dl-batch-files = { $count ->
    [one] Lot, { $count } fichier
   *[other] Lot, { $count } fichiers
}
dl-batch-tag = Lot
dl-downloading-count = { $count ->
    [one] { $count } en cours
   *[other] { $count } en cours
}
dl-free-on-phone = libres
dl-search-placeholder = Une série, ou un nom de release
dl-sources = Sources
dl-downloading = En cours
dl-queue-note = { $count ->
    [one] { $count } épisode
   *[other] { $count } téléchargements
}
dl-finished = Terminés
dl-finished-note = { $count ->
    [one] { $count } téléchargement, dans votre bibliothèque
   *[other] { $count } téléchargements, dans votre bibliothèque
}
dl-get = Obtenir
dl-pick = Choisir

## Storage and hidden shows

disk-on-phone = Sur ce téléphone
disk-watched = Vus
disk-not-watched = Non vus
disk-deleting = Suppression
disk-delete-watched = Supprimer les vus
disk-hidden-shows = Séries masquées
disk-hidden-helper = Hors d'Accueil et de la bibliothèque jusqu'à leur retour.
disk-unhide = Afficher
disk-delete-note = Les fichiers et sous-titres sont supprimés de ce téléphone définitivement.
disk-every-watched = Tous les épisodes vus
disk-every-finished = Toutes les séries terminées que vous avez vues
disk-episodes-size = { $count ->
    [one] { $count } épisode, { $size }
   *[other] { $count } épisodes, { $size }
}
disk-shows-size = { $count ->
    [one] { $count } série, { $size }
   *[other] { $count } séries, { $size }
}
disk-list-two = { $a } et { $b }
disk-list-three = { $a }, { $b } et { $c }
disk-list-more = { $a }, { $b }, { $c } et { $count } autres
disk-delete-finished-title = Supprimer { $count ->
    [one] { $count } série terminée
   *[other] { $count } séries terminées
}, { $size } ?
disk-delete-watched-title = Supprimer { $count ->
    [one] { $count } épisode vu
   *[other] { $count } épisodes vus
}, { $size } ?
disk-delete-finished-body = { $names }. Seulement les séries que le fournisseur dit terminées, tous les épisodes vus ; une série encore en diffusion, ou dont la fin est une estimation, reste. Chaque fichier est supprimé de ce téléphone, bonus compris. Leurs téléchargements partent aussi, donc ils pourront être retéléchargés.
disk-delete-watched-body = Chaque épisode que vous avez vu, dans chaque série. Les épisodes pas encore vus et les bonus restent. Les fichiers et leurs sous-titres sont supprimés de ce téléphone définitivement.
disk-no-finished = Aucune série terminée n'est vue en entier
disk-nothing-watched = Rien de vu n'est sur ce téléphone
disk-deleted = { $count ->
    [one] { $count } épisode supprimé
   *[other] { $count } épisodes supprimés
}, { $size }
disk-delete-failed = Rien n'a été supprimé : { $error }

## Download notifications

notify-card-head = AniBeam · maintenant
notify-sample-called = Frieren épisode 7
notify-sample-called-ready = Frieren épisode 6
notify-sample-facts = 420 Mo sur 1,4 Go, 3,2 Mo/s, 5 min restantes
notify-downloading = Téléchargement de { $called }
notify-progress = { $percent } %, { $facts }
notify-progress-of = { $percent } %, { $done } sur { $size }
notify-starting = Démarrage
notify-many = { $count ->
    [one] { $count } téléchargement
   *[other] { $count } téléchargements
}
notify-called-episode = { $show } épisode { $n }
notify-called-episodes = { $show } épisodes { $from } à { $to }
notify-ready = { $called } est prêt
notify-ready-many = { $called } sont prêts
notify-ready-batch = Un lot de { $show } est prêt
notify-ready-text = Téléchargé sur ce téléphone. Touchez pour ouvrir AniBeam.
notify-channel-downloads = Téléchargements
notify-channel-ready = Épisodes prêts
notify-fallback-title = Téléchargement
notify-row-on = Notifications
notify-row-allow = Autoriser les notifications
notify-row-open-settings = Ouvrir les réglages de notification
notify-helper-on = Activées. Les téléchargements s'affichent dans vos notifications pendant leur exécution.
notify-helper-asks-next = Android demande ensuite.
notify-helper-asks-next-later = Android demande ensuite. Vous pourrez changer d'avis plus tard dans Plus.
notify-helper-off = Désactivées. Activez les notifications d'AniBeam là-bas.
notify-question-title = Voir les téléchargements dans vos notifications ?
notify-question-body = Pendant le téléchargement, une notification montre l'avancement, et une autre indique quand un épisode est prêt. Les épisodes se téléchargent dans tous les cas ; cela décide seulement si vous le voyez.
notify-allow = Autoriser
notify-not-now = Plus tard
notify-pointer = Les téléchargements ne s'affichent pas dans vos notifications.
notify-turn-on = Activer

## Feed

phone-feed-title = Fil
feed-recently-released = Sorti récemment
feed-coming-soon = Bientôt
feed-this-week = Cette semaine
feed-earlier = Plus tôt
feed-everything-else = Tout le reste
phone-feed-countdown-days = { $d } j { $h } h
phone-feed-countdown-hours = { $h } h { $m } min
phone-feed-countdown-minutes = { $m } min
phone-feed-next = Suivant : { $title } dans { $countdown }
phone-feed-nothing-scheduled = Rien de prévu pour l'instant.
phone-feed-nothing-new = Rien de nouveau cette semaine.
phone-feed-new-episodes = { $count ->
    [one] { $count } nouvel épisode cette semaine.
   *[other] { $count } nouveaux épisodes cette semaine.
}
phone-feed-aired = diffusé { $ago }
phone-feed-downloaded = téléchargé { $ago }

## Files and the file picker

files-already-picking = Un fichier est déjà en cours de sélection.
files-took-too-long = Le sélecteur de fichiers a mis trop de temps.
files-did-not-open = Le sélecteur de fichiers ne s'est pas ouvert : { $error }
files-none-here = Il n'y a pas de sélecteur de fichiers ici.

## The computer's downloads

desk-sent-adding = { $what } envoyé à { $name }. Il les ajoute encore.
desk-sent = Obtention de { $what } sur { $name }.

## Your computer's library

remote-watched = Vu
remote-movie = Film
remote-not-watched-yet = Pas encore vu
remote-watched-of = { $done } sur { $total } vus
remote-watched-count = { $count } vus
remote-on-disk = { $count } sur disque
remote-airs-in-season = S{ $season } épisode { $n } dans { $countdown }
remote-airs-in = Épisode { $n } dans { $countdown }
remote-play-episode = Lire l'épisode { $n }
remote-watch-again = Revoir l'épisode { $n }
remote-update-host = Mettez AniBeam à jour sur cet ordinateur pour lire ceci ici
remote-aired = diffusé
remote-missing = Manquant
remote-on-computer = Sur votre ordinateur
remote-out = Sorti { $ago }
remote-season-episode = Saison { $season }, épisode { $n }

## Episodes and dates: a season's episode count and air dates, set by hand

more-words-ratings = imdb crédits notes évaluations
show-format-series = Série
dates-title = Épisodes et dates
dates-season-line = { $title } · Saison { $season }
dates-paste-open = Coller une liste
dates-paste-done = Terminé
dates-paste-hint = Collez une liste de sorties, un tableau ou un post
dates-paste-note = Ce qu'AniBeam lit s'affiche sur les épisodes ci-dessous.
dates-tap-hint = Touchez un épisode pour le dater.
dates-run = Plage
dates-field-hint = 7 janv., mars 2027, 2027
dates-set = Dater
dates-no-date = Pas de date
dates-weekly = Hebdo
dates-last = Dernier
dates-not-last = Pas le dernier
dates-clear = Effacer
dates-save = Enregistrer
dates-saving = Enregistrement
dates-cancel = Annuler
dates-change = Changer
dates-picked-one = Épisode { $episode } :
dates-picked-many =
    { $count ->
        [one] { $count } épisode :
       *[other] { $count } épisodes :
    }
dates-saved = Enregistré.
dates-cleared = Notes de calendrier effacées.
dates-starting = AniBeam démarre encore. Réessayez dans un instant.
dates-tile-out = Sorti
dates-tile-last = Dernier
dates-tile-none = Sans date
dates-tile-guess = ≈ { $date }
dates-summary-none = Rien de défini
dates-summary-ends = Se termine à l'épisode { $episode }
dates-summary-record =
    { $count ->
        [one] { $count } daté par { $site }
       *[other] { $count } datés par { $site }
    }
dates-summary-record-unnamed =
    { $count ->
        [one] { $count } daté par la fiche
       *[other] { $count } datés par la fiche
    }
dates-summary-set =
    { $count ->
        [one] { $count } défini par vous
       *[other] { $count } définis par vous
    }
dates-summary-guessed =
    { $count ->
        [one] { $count } estimé chaque semaine
       *[other] { $count } estimés chaque semaine
    }
dates-told-one = Épisode { $episode }
dates-told-many = Épisodes { $first } à { $last }
dates-told-undated = { $episodes } sans date
dates-told-due = { $episodes } dans { $when }
dates-told-ends = Se termine à l'épisode { $episode }
dates-told-season-ends = La saison { $season } se termine à l'épisode { $episode }
dates-told-by-you = { $told }, défini par vous
dates-told-gave-way =
    { $count ->
        [one] Vous avez défini { $count } épisode ; cela ne s'applique plus
       *[other] Vous avez défini { $count } épisodes ; cela ne s'applique plus
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
series-runtime-each = { $runtime } chacun
series-status-airing = En diffusion
series-status-finished = Terminée
series-status-upcoming = À venir
series-status-cancelled = Annulée
series-status-hiatus = En pause
series-average-score = Note moyenne { $score }
series-imdb = IMDb { $score }
series-imdb-votes = IMDb { $score } sur { $votes } votes
series-rewatched =
    { $count ->
        [one] Revue { $count } fois
       *[other] Revue { $count } fois
    }
series-not-tracked = Non suivie
series-not-tracked-on-phone =
    { $count ->
        [one] Non suivie, { $count } épisode sur ce téléphone
       *[other] Non suivie, { $count } épisodes sur ce téléphone
    }

## More, About

about-ratings = Notes
about-ratings-credit = Informations fournies par IMDb (https://www.imdb.com). Utilisées avec autorisation.

## Marking episodes on the series page

mark-menu-watched = Marquer vu jusqu'ici
mark-menu-unwatched = Marquer non vu depuis ici
mark-menu-cancel = Annuler
mark-watched-up-to = Vu jusqu'à l'épisode { $episode }
mark-unwatched-from = Non vu à partir de l'épisode { $episode }
mark-not-connected = { $reason } Connectez-vous dans Plus, Comptes.

## The player

player-speed-not-while-casting = Pas en diffusion

## Downloads

downloads-other-release-on-disk = Autre release sur disque
