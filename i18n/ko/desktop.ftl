# ko translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = 홈
nav-library = 라이브러리
nav-airing = 방영 중
nav-feed = 피드
nav-watching = 시청 중
nav-stats = 통계
nav-metadata = 작품 정보
nav-downloads = 다운로드
nav-settings = 설정
nav-series = 작품
nav-player = 플레이어
nav-show = 작품
nav-recommended = 추천

## StatusStrip

status-strip-errors = { $count ->
   *[other] 오류 { $count }개
}

## ScanSummary

scan-summary-nothing = 아직 보여 줄 것이 없어요
scan-summary-found = 작품 { $count }개 발견, { $recognised }개 인식
scan-summary-found-need = 작품 { $count }개 발견, { $recognised }개 인식, { $need }개 확인 필요
scan-summary-episodes = 디스크에 에피소드 { $count }개.
scan-summary-episodes-watched = 디스크에 에피소드 { $count }개, { $tracker }에서 { $watched }개 시청함.
scan-summary-title = 폴더를 불러왔어요
scan-summary-count-found = 발견
scan-summary-count-recognised = 인식
scan-summary-count-need = 확인 필요
scan-summary-later = 나중에
scan-summary-match = { $count }개 매칭
scan-summary-open-library = 라이브러리 열기

## UpdateNotice

update-notice-check-failed = AniBeam이 업데이트를 확인하지 못했어요. 인터넷 연결을 확인하고 다시 시도하세요.
update-notice-up-to-date = AniBeam이 최신 버전이에요
update-notice-downloading = AniBeam { $version } 다운로드 중
update-notice-ready = AniBeam { $version } 준비 완료
update-notice-failed = 업데이트 실패
update-notice-on-aur = AniBeam { $version }이(가) AUR에 올라왔어요
update-notice-out = AniBeam { $version } 출시
update-notice-out-titled = AniBeam { $version } “{ $title }” 출시
update-notice-progress = { $percent }%  ·  { $total } MB 중 { $got } MB
update-notice-restart-hint = 지금 다시 시작하거나 다음 실행 때 적용돼요
update-notice-from-source = 소스로 빌드한 사본이에요: pull 후 다시 빌드하세요
update-notice-no-download = 이 컴퓨터용 다운로드가 없어요
update-notice-install-hint = 업데이트를 누르면 설치하고 AniBeam을 다시 시작해요
update-notice-retry = 다시 시도
update-notice-update = 업데이트
update-notice-restart = 다시 시작
update-notice-copied = 복사됨
update-notice-copy = 복사
update-notice-release-page = 릴리스 페이지
update-notice-later = 나중에

## WhatsNew

whats-new-title = 새로운 기능
whats-new-got-it = 확인

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = M월 d일 HH:mm
activity-drawer-stage-library = 라이브러리
activity-drawer-stage-metadata = 메타데이터
activity-drawer-stage-trackers = 트래커
activity-drawer-stage-franchise = 프랜차이즈
activity-drawer-stage-playback = 재생
activity-drawer-stage-store = 저장소
activity-drawer-stage-system = 시스템
activity-drawer-stage-downloads = 다운로드
activity-drawer-level-info = 정보
activity-drawer-level-warn = 경고
activity-drawer-level-error = 오류
activity-drawer-copied-line = 줄을 복사했어요
activity-drawer-title = 활동
activity-drawer-everything = 전체
activity-drawer-problems = 문제
activity-drawer-copied = 복사됨
activity-drawer-copy = 복사
activity-drawer-clear-every = 모두 지우기
activity-drawer-copy-tip = 클릭해서 줄 복사
activity-drawer-fold = 접기
activity-drawer-empty-technical = 아직 기록된 내용이 없어요.
activity-drawer-empty-problems = 문제 없음. 모두 잘 됐어요.
activity-drawer-empty = 아직 보고할 내용이 없어요. AniBeam이 한 일이 여기에 표시돼요.

## ActivityColumn

activity-column-episode-run = { $first }~{ $last }화
activity-column-episode-list = { $list }화
activity-column-downloads = { $count ->
   *[other] 다운로드 { $count }개
}
activity-column-batch = 일괄 다운로드
activity-column-download = 다운로드
activity-column-name-batch = { $title }, 일괄
activity-column-name-episode = { $title }, { $number }화
activity-column-title = 활동
activity-column-down = 다운
activity-column-up = 업
activity-column-not-connected = { $client }이(가) 연결되지 않아서 아무것도 진행되지 않아요.
activity-column-connect-again = 다시 연결
activity-column-downloading = 다운로드 중
activity-column-fewer = 접기
activity-column-more = { $count }개 더 보기
activity-column-nothing-downloading = 다운로드 중인 항목이 없어요.
activity-column-finished-today = 오늘 완료
activity-column-today = 오늘
activity-column-today-empty = 검색, 다운로드, 새 구독이 생길 때마다 여기에 표시돼요.
activity-column-stopping = 중지 중
activity-column-starting = 시작 중
activity-column-start-tip = 다운로드 시작
activity-column-stop-tip = 다운로드 중지. 받은 부분은 남아요
activity-column-cancelling = 취소 중
activity-column-cancel-all-tip = 이 다운로드들 취소
activity-column-cancel-tip = 이 다운로드 취소
activity-column-size-of = { $total } 중 { $got }
activity-column-files-missing = 파일 없음
activity-column-time-format = HH:mm
activity-column-left = { $time } 남음
activity-column-waiting-peers = 피어 기다리는 중
activity-column-episodes-done = { $count ->
   *[other] 에피소드 { $count }개 중 { $done }개 완료
}
activity-column-cube-skipped = { $number }화: 건너뜀
activity-column-cube-done = { $number }화: 완료
activity-column-cube-progress = { $number }화: { $percent }%

## Frame

frame-window-title = { $page } - AniBeam
frame-strip-ready = 준비됨
frame-strip-progress = { $doing }, { $total }개 중 { $done }개
frame-strip-idle = 실행 중인 작업 없음
frame-strip-time-format = HH:mm
frame-finish-sign-in = 먼저 { $tracker } 로그인을 마치세요
frame-did-not-work = 실패했어요.
frame-could-not-open = AniBeam이 열지 못했어요. 복사해서 직접 여세요.
frame-copy = 복사
frame-copied = 복사됨
frame-status-refused = { $tracker }이(가) 받지 않았어요
frame-status-refused-why = { $tracker }이(가) 받지 않았어요: { $why }
frame-lists-refused = { $tracker }이(가) 작품 { $count }개를 받지 않았어요. 활동 로그를 확인하세요
frame-lists-moved = 작품 { $count }개를 { $tracker }(으)로 옮겼어요
frame-notes-refused = { $tracker }이(가) 메모를 받지 않았어요
frame-notes-refused-why = { $tracker }이(가) 메모를 받지 않았어요: { $why }
frame-notes-saved = 메모 저장됨
frame-connected = { $tracker }에 { $user } 계정으로 연결됨
frame-sign-in-stopped = { $tracker } 로그인이 끝나기 전에 멈췄어요.
frame-toast-hide-details = 세부 정보 숨기기
frame-toast-details = 세부 정보
frame-toast-details-tip = 무엇이 잘못됐는지 그대로 보여 주고 복사해요
frame-forward = 앞으로
frame-copy-magnet = 마그넷 복사
frame-copy-link = 토렌트 링크 복사
frame-magnet-copied = 마그넷 복사됨
frame-link-copied = 링크 복사됨
frame-open-release-page = 릴리스 페이지 열기
frame-open-folder = 폴더 열기
frame-copy-path = 경로 복사
frame-path-copied = 경로 복사됨
frame-mark-unwatched = 여기부터 시청 안 함으로 표시
frame-mark-watched = 여기까지 시청함으로 표시
frame-copy-title = 제목 복사
frame-open-on-anilist = AniList에서 열기
frame-list-home-online = 연결된 각 목록에서 바꿀 수 있어요
frame-list-home-unmatched = 이 기기에 보관돼요: 이 작품을 다루는 트래커가 없어요
frame-list-home-not-carried = 이 기기에 보관돼요: 내 트래커에 이 작품이 없어요
frame-list-home-offline = AniList나 MyAnimeList를 연결할 때까지 이 기기에 보관돼요
frame-moving-to = 이동 중: { $list }
frame-moved-to = 이동 완료: { $list }
frame-plan-added = { $title }을(를) 시청 예정 목록에 추가해요
frame-plan-listed = { $title }은(는) 이미 목록에 있어요
frame-delete-watched = { $kind ->
    [film] 시청한 영화 삭제
   *[episode] 시청한 에피소드 삭제
}
frame-delete-all = { $count ->
   *[other] { $kind ->
        [film] 영화 { $count }개 모두 삭제
       *[episode] 에피소드 { $count }개 모두 삭제
    }
}
frame-series-gone = 그 작품이 없어졌어요
frame-delete-none-watched = 디스크에 { $title }의 시청한 에피소드가 없어요
frame-delete-none = 디스크에 { $title } 파일이 없어요
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] 시청한 영화 { $count }개
       *[episode] 시청한 에피소드 { $count }개
    }
   *[no] { $kind ->
        [film] 영화 { $count }개
       *[episode] 에피소드 { $count }개
    }
}
frame-delete-extras = { $count ->
   *[other] 번외 { $count }개
}
frame-delete-and = { $first }, { $second }
frame-delete-heading = { $title }의 { $what }을(를) 삭제할까요?
frame-delete-base = 파일과 자막은 휴지통으로 가니 거기서 되살릴 수 있어요. 토렌트는 토렌트 클라이언트에서 빠지므로 다시 다운로드할 수 있어요.
frame-delete-confirm = 휴지통으로 이동
frame-delete-done = { $title }의 { $what }을(를) 휴지통으로 옮겼어요
frame-delete-watched-done = { $count ->
   *[other] 에피소드 { $count }개
}, { $size }, 휴지통으로 옮겼어요
frame-delete-nothing = 삭제할 항목이 없어요
frame-fix-not-drafted = 수정했어요. 이슈 초안은 만들지 못했어요
frame-fix-agent = 수정했어요. { $agent }이(가) 살펴보는 중이고, 끝나면 이슈가 열려요
frame-fix-opening = 수정했어요. 이슈를 여는 중
frame-fix-not-saved = 수정 사항이 저장되지 않았어요: { $error }
frame-fix-undone = { $count ->
   *[other] 파일 { $count }개
}를 다시 이름으로 읽어요
frame-fix-done = { $count ->
   *[other] 파일 { $count }개
}를 읽는 방식을 수정했어요
frame-report-failed = 이슈 초안을 만들지 못했어요: { $error }
frame-report-untriaged = 분류 없이 브라우저에서 이슈를 열었어요. { $error }
frame-report-open = 브라우저에서 이슈를 열었어요. 바로 보낼 수 있어요
frame-downloads-not-read = 읽지 않음.

## HomePage

home-title = 홈
home-scan-started = 스캔 시작됨
home-ago-minutes = { $count }분 전
home-ago-hours = { $count }시간 전
home-ago-days = { $count }일 전
home-today = 오늘
home-tomorrow = 내일
home-yesterday = 어제
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = M월 d일
home-day-date-format = M월 d일 (ddd)
home-time-format = HH:mm
home-day-time-format = (ddd) HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = 날짜 미정
home-unit-days = { $count ->
   *[other] 일
}
home-unit-hours = 시간
home-unit-minutes = 분
home-week-late = { $count ->
   *[other] 에피소드 { $count }개 지연
}
home-week-waiting = { $count ->
   *[other] 볼 수 있는 에피소드 { $count }개
}
home-week-coming = { $count ->
   *[other] 7일 안에 에피소드 { $count }개
}
home-week-next = 다음: { $time } 후 { $title }
home-countdown-days = { $days }일 { $hours }시간
home-countdown-hours = { $hours }시간 { $minutes }분
home-countdown-minutes = { $minutes }분
home-out-not-downloaded = 아직 다운로드 안 됨
home-out-downloading = 다운로드 중
home-out-when = { $day } { $time }
home-next-up = 다음 에피소드
home-next-when = { $span }, { $time }
home-next-when-estimated = { $span }, { $time }, 시간 추정
home-next-when-season = 시즌 { $season }, { $span }, { $time }
home-next-when-season-estimated = 시즌 { $season }, { $span }, { $time }, 시간 추정
home-no-date = 날짜 없음
home-tile-watched = 시청함
home-tile-downloading = 다운로드 중
home-tile-out = { $ago } 공개
home-tile-not-out = 아직 공개 안 됨
home-tile-estimated = 추정
home-tile-season-span = S{ $season } { $span }
home-tile-to-watch = 볼 에피소드 { $count }개
home-day-nothing = 없음
home-now = 지금 { $time }
home-card-aired = { $episode } { $ago } 공개
home-continue = 이어 보기
home-continue-fact = 이어 볼 작품 { $count }개
home-on-hold = 보류
home-on-hold-fact = 다시 볼 작품 { $count }개
home-hero-open = { $title } 열기
home-hero-pick-up = 멈춘 곳부터 이어 보기
home-hero-up-next = 다음 에피소드
home-hero-downloading = 에피소드 { $count }개 다운로드 중
home-hero-min-left = { $count }분 남음
home-hero-watched = { $total }화 중 { $watched }화 시청
home-hero-resume = { $number }화 이어 보기
home-hero-play = { $number }화 재생
home-hero-resume-unnumbered = 에피소드 이어 보기
home-hero-play-unnumbered = 에피소드 재생
home-downloaded = 최근 다운로드
home-downloaded-fact = 최근 2주 동안 새 파일이 생긴 작품 { $count }개
home-week = 이번 주
home-week-show-empty = 빈 날 보기
home-week-hide-empty = 빈 날 숨기기
home-week-airing = 방영 중
home-week-out = 공개됨
home-week-coming-up = 방영 예정
home-recommended = 추천 작품
home-recommended-fact = 내 목록 기반
home-recommended-scope = 추천 범위
home-recommended-all = 전체
home-recommended-in-library = 라이브러리
home-recommended-on-list = 내 목록
home-recommended-none-library = 라이브러리에서 더 추천할 작품이 없어요. 전체에서 아직 없는 작품을 찾아보세요.
home-recommended-none-list = 목록에서 더 추천할 작품이 없어요. 전체에서 목록에 없는 작품을 찾아보세요.
home-catch-up = 밀린 에피소드
home-catch-up-fact = 볼 에피소드가 쌓인 작품 { $count }개
home-fresh = 새 작품 시작하기
home-fresh-fact = 시작 안 한 작품 { $count }개
home-genres = 장르
home-open-library = 라이브러리 열기
home-dropped = 중단
home-dropped-fact = { $count ->
   *[other] 디스크에 남은 파일 { $count }개
}
home-out-now = 지금 공개
home-out-now-fact = AniList 기준 이번 주 새 에피소드
home-popular = 이번 시즌 인기작
home-add-folder-title = 내 작품도 여기에 표시돼요
home-add-folder-body = 애니가 있는 폴더를 추가하면 홈에 보고 있는 작품이 채워져요.
home-add-folder-button = 폴더 추가
home-empty-scanned-title = 폴더에서 아직 찾은 게 없어요
home-empty-title = 아직 작품이 없어요
home-empty-scanned-body = AniBeam이 지켜보는 폴더에서 작품이나 영화를 찾지 못했어요. 다시 스캔하거나 설정에서 폴더를 확인하세요.
home-empty-body = 애니가 있는 폴더를 추가하면 라이브러리가 저절로 채워져요.
home-scan-all = 모두 스캔
home-library-settings = 라이브러리 설정
home-failed-title = 홈을 불러오지 못했어요
home-failed-body = 지금은 AniBeam이 라이브러리를 읽지 못했어요. 잠시 후 다시 시도하세요.

## Onboarding

onboarding-language = 언어
onboarding-step-welcome = 시작
onboarding-step-library = 라이브러리
onboarding-step-lists = 내 목록
onboarding-step-downloads = 다운로드
onboarding-step-ready = 준비 완료
onboarding-scan-reading = 폴더 읽는 중
onboarding-scan-found = 작품 { $count }개를 찾았어요. 언제든 계속하세요. 나머지는 알아서 채워져요.
onboarding-scan-starter-empty = 새 폴더는 아직 비어 있어요. 작품을 넣으면 여기에 저절로 나타나요.
onboarding-scan-no-video = 동영상 파일을 찾지 못했어요. 올바른 폴더인지 확인하세요.
onboarding-scan-your-shows = 내 작품
onboarding-scan-looking-up = { $what } 조회 중
onboarding-scan-looking-up-tally = { $what } 조회 중, { $count }개 중 { $posters }개 찾음
onboarding-scan-posters = 포스터 가져오는 중
onboarding-scan-posters-tally = 포스터 가져오는 중, { $count }개 중 { $posters }개 찾음
onboarding-scan-seasons = 관련 시즌 찾는 중
onboarding-scan-seasons-tally = 관련 시즌 찾는 중, { $count }개 중 { $posters }개 찾음
onboarding-choose-folder = 애니 폴더 선택
onboarding-about = 1분이면 설정 끝. 라이브러리는 알아서 완성돼요.
onboarding-step-later = 나중에
onboarding-skip-setup = 설정 건너뛰기
onboarding-title-welcome = AniBeam에 오신 걸 환영해요
onboarding-title-library = 애니는 어디에 있나요?
onboarding-title-lists = 애니 목록 연결
onboarding-title-downloads = 다운로드 방식
onboarding-title-ready = 모두 준비됐어요
onboarding-title-later = { $count ->
    [1] 준비 완료, 나중에 할 일 하나
    [2] 준비 완료, 나중에 할 일 둘
   *[other] 준비 완료, 나중에 할 일 셋
}
onboarding-body-welcome = AniBeam은 애니를 한곳에 모아 줘요. 디스크에 있는 작품을 찾아 바로 여기서 재생하고, 보는 대로 AniList나 MyAnimeList를 업데이트해요.
onboarding-body-library = 작품을 보관하는 폴더를 고르세요. 그 안의 폴더 하나하나가 포스터, 에피소드, 이어 볼 위치를 가진 작품이 돼요. AniBeam은 폴더를 읽기만 하고, 파일을 옮기거나 이름을 바꾸지 않아요.
onboarding-body-lists = 한 번만 로그인하면 여기서 다 본 에피소드가 입력할 필요 없이 목록에 기록돼요. 로그인은 브라우저에서 하므로 AniBeam은 비밀번호를 보지 않아요.
onboarding-body-downloads = 다운로드 페이지에서 작품을 구독하면 새 에피소드가 방영되는 대로 라이브러리에 들어와요. 이를 위해 두 가지가 필요해요: 검색할 곳과 받아 올 토렌트 클라이언트.
onboarding-body-ready = 여기 있는 모든 것은 나중에 설정에서 바꿀 수 있고, 설정, 정보에서 이 설정을 다시 할 수 있어요.
onboarding-skip-step = 지금은 건너뛰기
onboarding-get-started = 시작하기
onboarding-open-app = AniBeam 열기
onboarding-continue = 계속
onboarding-payoff-library = 내 라이브러리
onboarding-payoff-library-line = 폴더 속 모든 작품의 포스터, 에피소드, 다음 볼 화.
onboarding-payoff-plays = 바로 여기서 재생
onboarding-payoff-plays-line = 자막, 오프닝 스킵, 멈춘 위치까지 기억해요.
onboarding-payoff-list = 목록 자동 동기화
onboarding-payoff-list-line = 에피소드를 다 보면 AniList나 MyAnimeList에 기록돼요.
onboarding-payoff-episodes = 에피소드가 알아서
onboarding-payoff-episodes-line = 작품을 구독하면 새 에피소드가 라이브러리에 들어와요.
onboarding-films-too = 영화 폴더도 돼요: 이름을 Movies로 하세요.
onboarding-choose-folder-button = 폴더 선택
onboarding-no-folder = 아직 폴더가 없어요
onboarding-starter-where = { $dir }에 Anime 폴더를 만들어 사용해요.
onboarding-add-another = 다른 폴더 추가
onboarding-shelf-more = 더 보기
onboarding-preview-show = 내 작품
onboarding-preview-finished = 여기서 3화 시청 완료
onboarding-preview-on-list = 내 목록: 3화 시청
onboarding-preview-set-by = 엔딩 크레딧이 나오는 순간 AniBeam이 기록
onboarding-lists-connected = 연결됐어요. 이제 여기서 본 작품이 목록에 기록돼요.
onboarding-lists-no-account = 아직 계정이 없나요? 둘 다 무료이고 하나면 충분해요. 설정, 계정에서 언제든 연결을 끊을 수 있어요.
onboarding-search-title = 검색할 곳
onboarding-search-ready = { $count ->
   *[other] 검색 가능한 소스 { $count }개.
}
onboarding-search-none = AniBeam에는 자체 소스가 없어요. 소스 저장소에서 추가해요: 누군가 공유해 준 링크를 설정, 다운로드에 붙여넣으면 돼요.
onboarding-sources-after = 설정 후에 열려요
onboarding-sources-add = 설정 후 저장소 추가
onboarding-fetch-title = 받아 오는 곳
onboarding-fetch-auto = AniBeam에 맡기기
onboarding-fetch-recommended = 추천
onboarding-fetch-auto-line = AniBeam이 자체 qBittorrent를 백그라운드에서 조용히 실행해요. 설치할 것도, 입력할 것도 없어요.
onboarding-fetch-existing = 이미 qBittorrent를 쓰고 있어요
onboarding-fetch-existing-line = AniBeam이 실행 중인 qBittorrent에 연결해요. 다음으로 설정에서 주소와 로그인 정보를 입력하세요.
onboarding-ready-library = 라이브러리
onboarding-ready-library-none = 아직 폴더가 없어요. 원할 때 설정, 라이브러리에서 추가하세요.
onboarding-ready-library-series = { $count ->
   *[other] 폴더 { $count }개
}, 지금까지 작품 { $series }개
onboarding-ready-library-empty = { $count ->
   *[other] 폴더 { $count }개
}, 아직 비어 있음
onboarding-ready-library-scanning = { $count ->
   *[other] 폴더 { $count }개
}, 스캔 중
onboarding-ready-lists = 내 목록
onboarding-ready-lists-as = { $tracker }, { $user }
onboarding-ready-lists-none = 아직 연결 안 됨. 원할 때 설정, 계정에서 연결하세요.
onboarding-ready-downloads = 다운로드
onboarding-ready-downloads-line = { $sources ->
    [0] 아직 소스 없음
   *[other] 소스 { $sources }개
}, { $client ->
    [own] 내 qBittorrent.
   *[bundled] AniBeam 자체 qBittorrent.
} { $next ->
    [ready] 모두 준비됐어요.
    [opens] 다음으로 설정, 다운로드가 열려요.
   *[both] 둘 다 설정, 다운로드에 있어요.
}
onboarding-ready-open-downloads = 설정, 다운로드 열기
onboarding-ready-set-up = 지금 설정

## Rail.qml

rail-home = 홈
rail-library = 라이브러리
rail-airing = 방영 중
rail-feed = 피드
rail-stats = 통계
rail-show-info = 작품 정보
rail-downloads = 다운로드
rail-settings = 설정
rail-downloads-complete =
    { $count ->
       *[other] 다운로드: 구독 { $count }개 완료
    }
rail-title-language-name = 제목 언어: { $language }. 누르면 다음 언어로
rail-title-language-english = 영어
rail-title-language-tip = 제목 언어: 로마자, 영어, 일본어
rail-whats-new-in = { $release }의 새로운 기능
rail-whats-new = 새로운 기능
rail-settings-note = 설정: { $note }

## FixSheet.qml

fix-kind-episode = 에피소드
fix-kind-opening = 오프닝
fix-kind-ending = 엔딩
fix-kind-preview = 예고
fix-kind-special = 스페셜
fix-kind-other = 기타
fix-reads-season-episode = 시즌 { $season } { $number }화
fix-reads-extra = { $kind } { $index }
fix-apply-failed = 수정 사항을 적용하지 못했어요.
fix-title = 파일 읽는 방식 수정
fix-lede = { $title }. 각 파일이 실제로 무엇인지 지정하세요. 이후 모든 스캔에서 유지돼요.
fix-rescan = 폴더 다시 스캔
fix-rescan-started = 폴더를 다시 읽는 중
fix-filled-in = 채워짐
fix-fill-in = 채우기
fix-column-file = 파일
fix-column-reads-as = 읽힌 값
fix-column-should-be = 올바른 값
fix-fixed = 수정됨
fix-number-placeholder = 번호
fix-index-placeholder = 없음
fix-keep = 수정 유지
fix-leave-as-read = 읽힌 그대로 두기
fix-read-by-name = 이 파일을 다시 이름으로 읽기
fix-report = GitHub에 이슈 등록
fix-report-body = 내용이 채워진 이슈를 열어요. 이슈는 공개되고 이 파일 이름이 들어가요.
fix-report-body-agent = 내용이 채워진 이슈를 열어요. 이슈는 공개되고 이 파일 이름이 들어가요. 먼저 { $agent }이(가) 살펴봐요.
fix-close-unapplied =
    { $count ->
       *[other] 수정 사항 { $count }개를 적용하지 않고 닫을까요?
    }
fix-number-invalid = 빨간색 숫자를 먼저 고쳐야 해요
fix-close-anyway = 그래도 닫기
fix-keep-editing = 계속 편집
fix-apply-count =
    { $count ->
       *[other] 수정 사항 { $count }개 적용
    }

## MatchPanel.qml

match-matched-to = 매칭됨: { $title }
match-applied = 매칭 적용됨
match-found-tvmaze = TVmaze에서 작품을 찾았어요
match-found-anilist = AniList에서 찾았어요
match-found-anilist-season = AniList에서 찾았어요, 시즌 { $season }
match-found-mal = MyAnimeList에서 찾았어요
match-sites-and = { $first }, { $next }
match-matched-on = { $sites }에서 매칭됨
match-provider-silent = { $provider }에서 응답이 없어요
match-title-walk = 매칭, { $walk }
match-title = 매칭
match-skip = 건너뛰기
match-refresh-info = 정보 새로고침
match-refreshing = { $title } 새로고침 중
match-clear-confirm = 해제하면 연결과 설정한 에피소드 수가 사라져요
match-clear = 매칭 해제
match-cleared = 매칭 해제됨
match-search-placeholder = { $provider } 검색
match-link-tvmaze = TVmaze 또는 IMDb 링크 붙여넣기
match-link-anilist = AniList 또는 MyAnimeList 링크 붙여넣기
match-look-up = 조회
match-searching = { $provider } 검색 중
match-type-more = 두 글자 이상 입력하면 { $provider }에서 검색해요.
match-no-results = { $provider }에서 찾은 제목이 없어요.

## WatchingPage.qml

watching-episode-code = { $number }화
watching-countdown-days = { $days }일 { $hours }시간
watching-countdown-hours = { $hours }시간 { $minutes }분
watching-countdown-minutes = { $minutes }분
watching-on-disk = 디스크에 있음
watching-on-anilist = AniList에 있음
watching-ready-to-start = 시작할 준비 완료
watching-not-in-library = 라이브러리에 없음
watching-get-to-start = 에피소드를 받아 시작하세요
watching-anilist-updated = AniList 업데이트: { $when }
watching-airing = 방영 중
watching-next-in = 다음: { $wait } 후 { $title }
watching-episodes-waiting = 볼 에피소드 있음
watching-series-behind =
    { $count ->
       *[other] 밀린 작품 { $count }개
    }
watching-up-to-date = 최신 상태
watching-anilist-only = AniList에만 있음
watching-planning-summary-off = AniList에서 볼 예정인 작품이에요.
watching-planning-summary =
    { $count ->
       *[other] 시청 예정 작품 { $count }개, 디스크에 { $here }개.
    }
watching-summary-off = AniList에서 시청 중이거나 재시청 중인 작품이에요.
watching-summary-waiting =
    { $count ->
       *[other] 볼 에피소드가 있는 작품 { $count }개.
    }
watching-caught-up = 모두 따라잡았어요.
watching-start-watching = 보기 시작
watching-play-next = 다음 화 재생
watching-open-on-anilist = AniList에서 열기
watching-refreshing = 새로고침 중
watching-connect-title = AniList가 연결되지 않았어요
watching-connect-body = 연결하면 AniList에서 시청 중이거나 재시청 중인 작품이 여기에 표시돼요. 가지고 있는 작품에는 재생 버튼이, 방영 중인 작품에는 카운트다운이 붙어요.
watching-connect-waiting = 브라우저를 기다리는 중
watching-connect-anilist = AniList 연결
watching-load-failed-title = 시청 중 목록을 불러오지 못했어요
watching-load-failed-body = AniList에 연결할 수 없었어요. 다음 새로고침 때 목록이 돌아와요.
watching-planning-failed-title = 시청 예정 목록을 불러오지 못했어요
watching-planning-failed-body = 캐시를 읽지 못했어요. 다시 시도하거나 다음 새로고침 때 돌아와요.
watching-planning-empty-title = 시청 예정 목록이 비어 있어요
watching-planning-empty-body = 방영 중에서 시청 예정을 누르거나 작품의 목록 메뉴에서 시청 예정을 고르면 여기에 표시돼요.
watching-airing-now = 방영 중
watching-empty-title = 시청 중 목록이 비어 있어요
watching-empty-body = AniList에서 작품을 시청 중으로 표시하면 여기에 나타나요.
watching-open-anilist = AniList 열기

## LibraryPage.qml

library-hidden =
    { $count ->
       *[other] 작품 { $count }개 숨김
    }
library-hidden-sync-stops =
    { $count ->
       *[other] 작품 { $count }개 숨김, 트래커 동기화도 멈춰요
    }
library-unhidden =
    { $count ->
       *[other] 작품 { $count }개 다시 표시, 동기화도 재개돼요
    }
library-undo = 되돌리기
library-clearing-score =
    { $count ->
       *[other] 작품 { $count }개의 점수 지우는 중
    }
library-rating =
    { $count ->
       *[other] 작품 { $count }개에 { $score }점 주는 중
    }
library-moving =
    { $count ->
       *[other] 작품 { $count }개를 { $list }(으)로 옮기는 중
    }
library-rescanning =
    { $count ->
       *[other] 작품 { $count }개 다시 스캔 중
    }
library-finding-matches =
    { $count ->
       *[other] 작품 { $count }개의 매칭 찾는 중
    }
library-sort-last-viewed = 최근 시청
library-sort-downloaded = 최근 다운로드
library-sort-title = 제목
library-sort-progress = 진행도
library-sort-average-score = 평균 점수
library-sort-my-score = 내 점수
library-dir-a-to-z = 가나다순
library-dir-z-to-a = 가나다 역순
library-dir-oldest = 오래된 순
library-dir-newest = 최신순
library-dir-least-watched = 적게 본 순
library-dir-most-watched = 많이 본 순
library-dir-lowest = 낮은 순
library-dir-highest = 높은 순
library-group-none = 없음
library-group-season = 시즌
library-group-type = 유형
library-group-genre = 장르
library-group-watch-state = 시청 상태
library-scan-started = 스캔 시작됨
library-tab-all = 전체
library-tab-series = 시리즈
library-tab-movies = 영화
library-tab-watched = 시청함
library-tab-hidden = 숨김
library-count-movies =
    { $count ->
       *[other] 영화 { $count }개
    }
library-count-watched = 시청함 { $count }개
library-count-hidden = 숨김 { $count }개
library-count-series =
    { $count ->
       *[other] 시리즈 { $count }개
    }
library-title = 라이브러리
library-all-genres = 모든 장르
library-search-placeholder = 제목 또는 폴더 검색
library-sort = 정렬
library-group = 그룹
library-franchises-together = 프랜차이즈끼리 묶기
library-watched-shows = 시청한 작품
library-load-failed-title = 라이브러리를 불러오지 못했어요
library-no-matches = "{ $query }"와(과) 일치하는 항목 없음
library-none-in-genre =
    { $what ->
        [series] 여기에 { $genre } 시리즈가 없어요
        [films] 여기에 { $genre } 영화가 없어요
        [watched] 여기에 시청한 { $genre } 작품이 없어요
        [hidden] 여기에 숨긴 { $genre } 작품이 없어요
       *[shows] 여기에 { $genre } 작품이 없어요
    }
library-none-here =
    { $what ->
        [series] 여기에 시리즈가 없어요
        [films] 여기에 영화가 없어요
        [watched] 여기에 시청한 작품이 없어요
        [hidden] 여기에 숨긴 작품이 없어요
       *[shows] 여기에 작품이 없어요
    }
library-load-failed-body = 지금은 AniBeam이 라이브러리를 읽지 못했어요. 잠시 후 다시 시도하세요.
library-no-matches-body = 철자를 확인하거나 전체에서 라이브러리 전체를 검색하세요.
library-none-in-genre-body = 이 보기에는 그 장르의 작품이 없어요. 모든 장르에서 나머지를 볼 수 있어요.
library-none-in-tab-body = 라이브러리에 아직 이런 작품이 없어요. 전체에서 모두 볼 수 있어요.
library-all-filtered-body = 라이브러리의 모든 항목이 이 보기에서 걸러졌어요.
library-clear-search = 검색 지우기
library-show-all = 모두 보기
library-picked = { $count }개 선택됨
library-pick-all = 전체
library-pick-all-tip = 보이는 카드 모두 선택(Ctrl+A)
library-unhide = 숨김 해제
library-hide = 숨기기
library-rate = 평가
library-rate-tip = 선택한 모든 작품에 같은 점수를 연결된 모든 트래커에 줘요
library-needs-tracker = 연결된 트래커와 매칭된 작품이 필요해요
library-move-to-list = 목록으로 이동
library-rescan = 다시 스캔
library-find-match = 매칭 찾기
library-find-match-tip = 선택한 작품 중 매칭이 없는 것을 하나씩 검색해요
library-all-matched = 선택한 작품이 모두 매칭돼 있어요
library-empty-scanned-title = 폴더에서 아직 찾은 게 없어요
library-empty-title = 라이브러리가 비어 있어요
library-empty-scanned-body = AniBeam이 지켜보는 폴더에서 작품이나 영화를 찾지 못했어요. 다시 스캔하거나 설정에서 폴더를 확인하세요.
library-empty-body = 애니가 있는 폴더를 추가하면 라이브러리가 저절로 채워져요. AniBeam 내보내기 파일을 가져와도 돼요.
library-scan-all = 모두 스캔
library-add-folder = 폴더 추가
library-settings = 라이브러리 설정
library-import = 가져오기
library-import-title = AniBeam 내보내기 파일 가져오기
library-import-filter-export = AniBeam 내보내기 (*.json)
library-import-filter-all = 모든 파일 (*)
library-import-started = 가져오기 시작됨

## MetadataPage.qml

metadata-title = 작품 정보
metadata-filter-all = 전체 { $count }
metadata-filter-series = 시리즈 { $count }
metadata-filter-films = 영화 { $count }
metadata-filter-missing = 누락 { $count }
metadata-scan-started = 스캔 시작됨
metadata-sort-added = 추가순
metadata-sort-season = 시즌
metadata-sort-a-to-z = 가나다순
metadata-section-unmatched = 인식 안 됨
metadata-section-missing = 폴더 없음
metadata-section-incomplete = 누락된 에피소드
metadata-section-empty = 다운로드 안 됨
metadata-section-airing = 방영 중
metadata-section-complete = 완비
metadata-no-info = 아직 작품 정보 없음
metadata-state-aired = { $aired }화 방영 중 { $have }화
metadata-state-of = { $expected }화 중 { $have }화
metadata-state-of-extra = { $expected }화 중 { $have }화 +{ $extra }
metadata-detail-none-aired = 방영된 { $aired }화 중 디스크에 있는 게 없어요
metadata-detail-aired = 방영된 { $aired }화 중 { $have }화가 디스크에 있음
metadata-detail-aired-missing = 방영된 { $aired }화 중 { $have }화가 디스크에 있음, { $missing }화 누락
metadata-detail-none = { $expected }화 중 디스크에 있는 게 없어요
metadata-detail-missing = { $expected }화 중 { $have }화가 디스크에 있음, { $missing }화 누락
metadata-walk-position = { $total }개 중 { $at }번째
metadata-walk-done = 마지막 작품이었어요
metadata-walk-left =
    { $count ->
       *[other] 매칭 필요에 { $count }개 남음
    }
metadata-countdown-days = { $days }일 { $hours }시간
metadata-countdown-hours = { $hours }시간 { $minutes }분
metadata-countdown-minutes = { $minutes }분
metadata-next-in = 다음: { $wait } 후
metadata-files-of = { $expected }개 중 { $have }개
metadata-files-on-disk = 디스크의 파일
metadata-forgot = { $title }을(를) 제거했어요
metadata-refreshed-failed = { $refreshed }개 새로고침, { $failed }개는 새로고침하지 못했어요
metadata-refreshed =
    { $count ->
       *[other] 작품 { $count }개 새로고침함
    }
metadata-found =
    { $count ->
       *[other] 작품 { $count }개 찾음
    }
metadata-found-some-left =
    { $count ->
       *[other] 작품 { $count }개 찾음, { $unmatched }개는 아직 인식 안 됨
    }
metadata-found-nothing = 새로 찾은 게 없어요, { $unmatched }개는 아직 인식 안 됨
metadata-all-recognised = 모든 작품이 인식됐어요
metadata-count =
    { $count ->
       *[other] 작품 { $count }개
    }
metadata-search-placeholder = 제목 필터
metadata-find-rest = 나머지 찾기
metadata-find-rest-started = AniBeam이 아직 모르는 폴더를 조회하는 중
metadata-find-rest-tip = AniBeam이 아직 모르는 폴더를 모두 설정에 지정된 순서대로 각 작품 정보 사이트에서 조회해요
metadata-refresh-all = 모두 새로고침
metadata-refresh-all-started = 모든 작품의 정보 업데이트 중
metadata-refresh-all-tip = 인식된 모든 작품의 정보를 다시 가져와요
metadata-crawling = 프랜차이즈 연결 찾는 중
metadata-crawl-progress = { $total }개 중 { $done }개
metadata-load-failed-title = 작품 정보를 불러오지 못했어요
metadata-empty-scanned-title = 폴더에서 아직 찾은 게 없어요
metadata-no-matches = "{ $query }"와(과) 일치하는 항목 없음
metadata-nothing-under =
    { $filter ->
        [Series] 시리즈에 해당하는 항목 없음
        [Movies] 영화에 해당하는 항목 없음
        [MissingFiles] 누락에 해당하는 항목 없음
       *[other] { $filter }에 해당하는 항목 없음
    }
metadata-load-failed-body = 지금은 AniBeam이 라이브러리를 읽지 못했어요. 잠시 후 다시 시도하세요.
metadata-empty-scanned-body = AniBeam이 지켜보는 폴더에서 작품을 찾지 못했어요. 다시 스캔하거나 설정에서 폴더를 확인하세요.
metadata-empty-body = 라이브러리가 비어 있어요. 애니가 있는 폴더를 추가하면 AniBeam이 스캔해요.
metadata-no-matches-body = 이 보기에는 그 단어가 들어간 제목이 없어요.
metadata-nothing-under-body = 지금은 이 필터에 맞는 작품이 없어요. 전체에서 나머지를 볼 수 있어요.
metadata-scan-all = 모두 스캔
metadata-add-folder = 폴더 추가
metadata-clear-search = 검색 지우기
metadata-show-all = 모두 보기
metadata-refreshing = { $title } 새로고침 중
metadata-refresh-tip = 이 작품의 정보를 다시 가져오기
metadata-forget-armed = 제거
metadata-forget-confirm-tip = 한 번 더 클릭하면 이 작품과 시청 기록을 AniBeam에서 제거해요
metadata-forget-tip = 폴더가 디스크에 더 이상 없어요: 이 작품과 기록을 AniBeam에서 제거해요

## AiringPage.qml

airing-title = 방영 중
airing-sort-popular = 인기
airing-sort-score = 점수
airing-sort-trending = 트렌드
airing-sort-favourites = 즐겨찾기
airing-sort-soonest = 다음 에피소드
airing-sort-aired = 방영된 화수
airing-sort-episodes = 에피소드 수
airing-sort-start = 시작일
airing-sort-duration = 에피소드 길이
airing-sort-title = 제목
airing-dir-most-popular = 인기 높은 순
airing-dir-least-popular = 인기 낮은 순
airing-dir-lowest = 낮은 순
airing-dir-highest = 높은 순
airing-dir-least-trending = 트렌드 낮은 순
airing-dir-most-trending = 트렌드 높은 순
airing-dir-fewest = 적은 순
airing-dir-most = 많은 순
airing-dir-soonest = 빠른 순
airing-dir-latest = 늦은 순
airing-dir-oldest = 오래된 순
airing-dir-newest = 최신순
airing-dir-shortest = 짧은 순
airing-dir-longest = 긴 순
airing-dir-a-to-z = 가나다순
airing-dir-z-to-a = 가나다 역순
airing-season =
    { $season ->
        [Winter] { $year }년 겨울
        [Spring] { $year }년 봄
        [Summer] { $year }년 여름
        [Fall] { $year }년 가을
       *[other] { $year }년 { $season }
    }
airing-ago-minutes = { $minutes }분 전
airing-ago-hours = { $hours }시간 전
airing-ago-days = { $days }일 전
airing-episode-out = { $episode } { $ago } 공개
airing-no-date = 날짜 미정
airing-in-library = 라이브러리에 있음
airing-out-today =
    { $count ->
       *[other] 오늘 공개된 에피소드 { $count }개
    }
airing-dismissed-one = { $title }은(는) 여기에 다시 표시되지 않아요
airing-dismissed =
    { $count ->
       *[other] 작품 { $count }개는 여기에 다시 표시되지 않아요
    }
airing-planned-already-one = { $title }은(는) 이미 목록에 있어요
airing-planned-already-all = 모두 이미 목록에 있어요
airing-planned =
    { $count ->
       *[other] 작품 { $count }개를 시청 예정 목록에 추가해요
    }
airing-planned-some-already =
    { $count ->
       *[other] 작품 { $count }개를 시청 예정 목록에 추가해요, { $already }개는 이미 있었어요
    }
airing-play-next = 다음 화 재생
airing-plan-to-watch = 시청 예정
airing-get-more-episodes = 에피소드 더 받기
airing-get-episodes = 에피소드 받기
airing-open-on-anilist = AniList에서 열기
airing-not-interested = 관심 없음
airing-not-interested-count = 관심 없음 { $count }개
airing-not-interested-tip = 관심 없음으로 표시한 작품은 설정에 있어요
airing-refreshing = 새로고침 중
airing-pick-season = 다른 시즌 선택
airing-filter-all = 전체
airing-filter-lists = 내 목록
airing-filter-library = 내 라이브러리
airing-hide-lists = 내 목록 숨기기
airing-hide-lists-tip = AniList나 MAL 목록에 있는 작품 빼기
airing-hide-library = 내 라이브러리 숨기기
airing-hide-library-tip = 에피소드를 가진 작품 빼기
airing-out-now = 지금 공개
airing-out-last-day =
    { $count ->
       *[other] 최근 24시간 동안 에피소드 { $count }개
    }
airing-out-fewer = 접기
airing-out-all = 전체 { $count }
airing-sort = 정렬
airing-load-failed-title = AniList에 연결할 수 없어요
airing-load-failed-body = 방영 중인 작품은 AniList에서 가져와요. 다음 새로고침 때 돌아와요.
airing-reading-title = 방영 중인 작품 읽는 중
airing-nothing-now = 지금 방영 중인 작품이 없어요
airing-nothing-for = { $season }에 방영하는 작품이 없어요
airing-nothing-read = 아직 읽은 게 없어요
airing-reading-body = AniList에서 방영 중인 모든 작품을 가져와요. 처음에는 몇 초 걸려요.
airing-nothing-body = AniList에 아직 이번 시즌 작품이 없어요. 새로고침으로 다시 읽거나 다른 시즌을 고르세요.
airing-nothing-read-body = 새로고침하면 AniList에서 방영 중인 모든 작품을 읽어요.
airing-hides-empty-title = 숨김 설정 때문에 남은 게 없어요
airing-lists-empty-title = 여기 있는 작품 중 목록에 있는 게 없어요
airing-library-empty-title = 여기 있는 작품 중 라이브러리에 있는 게 없어요
airing-hides-empty-body = 여기 있는 작품은 모두 목록이나 라이브러리에 있고, 그것들을 숨겼어요.
airing-lists-empty-body = 카드에서 시청 예정을 누르면 목록에 추가돼요.
airing-library-empty-body = 카드에서 에피소드 받기를 누르면 라이브러리로 다운로드해요.
airing-show-everything = 모두 보기
airing-picked = { $count }개 선택됨
airing-undo = 되돌리기

## EpisodeRow.qml

episode-row-date-time-format = M월 d일 (ddd), HH:mm
episode-row-date-upcoming-format = M월 d일 (ddd)
episode-row-date-format = yyyy년 M월 d일
episode-row-mark-unwatched-tip = { $state }. 클릭하면 여기부터 시청 안 함으로 표시
episode-row-mark-watched-tip = { $state }. 클릭하면 여기까지 시청함으로 표시
episode-row-next-up = 다음 순서
episode-row-versions =
    { $count ->
       *[other] 버전 { $count }개
    }
episode-row-versions-tip =
    { $count ->
       *[other] 같은 에피소드의 릴리스 { $count }개. 클릭해서 재생할 것을 고르세요
    }
episode-row-downloading = 다운로드 중 { $percent }%
episode-row-get = 에피소드 받기
episode-row-predicted = 예상
episode-row-extra = 번외
episode-row-resume = { $time }부터 이어 보기

## EpisodeCubes.qml

episode-cubes-one = 에피소드 1개
episode-cubes-range =
    { $count ->
       *[other] 에피소드 { $count }개, { $first }~{ $last }화
    }
episode-cubes-tip = { $number }화: { $state }

## EpisodesPane.qml

episodes-pane-extras =
    { $count ->
       *[other] 번외 { $count }개
    }
episodes-pane-unmatched =
    { $count ->
       *[other] 매칭 안 된 파일 { $count }개
    }
episodes-pane-on-disk = 디스크에 있음
episodes-pane-not-on-disk = 디스크에 없음
episodes-pane-season = S{ $season }
episodes-pane-get-tip = { $number }화 받기
episodes-pane-none-on-disk = 디스크에 에피소드가 없어요
episodes-pane-change-match = 매칭 변경
episodes-pane-refreshing = { $title } 새로고침 중

## ShowDetail.qml

show-detail-not-in-library = 이 폴더는 아직 라이브러리에 없어요. 스캔에서 찾으면 포스터와 에피소드가 여기에 표시돼요.
show-detail-film-on-disk = 영화가 디스크에 있어요.
show-detail-film-not-on-disk = 영화가 아직 디스크에 없어요.
show-detail-complete-line = 방영이 끝났고 { $total }화가 모두 있어요. 더 나올 게 없으니 구독이 할 일도 없어요.
show-detail-finished = 방영 종료.
show-detail-airing = 방영 중.
show-detail-have-of-total = { $total }화 중 { $have }화 있음.
show-detail-have =
    { $count ->
       *[other] 에피소드 { $count }개 있음.
    }
show-detail-next-predicted =
    { $count ->
       *[other] { $span } 예상: { $wait } 후.
    }
show-detail-next-airs =
    { $count ->
       *[other] { $span } 방영: { $wait } 후.
    }
show-detail-missing =
    { $count ->
       *[other] { $list }화가 없어요.
    }
show-detail-missing-more = { $list }화와 { $more }개가 더 없어요.
show-detail-downloaded = 다운로드됨
show-detail-downloading = 다운로드 중 { $percent }%
show-detail-taken-adding = 규칙에 해당, 추가 중
show-detail-taken = 규칙에 해당
show-detail-left-out = 규칙에서 제외
show-detail-subscribed-reading = 구독했어요. qBittorrent가 피드를 읽는 중이에요.
show-detail-taken-moving = 규칙이 피드의 릴리스 { $releases }개 중 { $taken }개를 가져와요. { $moving }개는 아래에서 다운로드 중이에요.
show-detail-taken-held = 규칙이 피드의 릴리스 { $releases }개 중 { $taken }개를 가져와요. 토렌트 { $torrents }개가 클라이언트에 있어요.
show-detail-taken-adding-line = 규칙이 피드의 릴리스 { $releases }개 중 { $taken }개를 가져와요. qBittorrent가 추가하는 중이에요.
show-detail-feed-holds-nothing = 피드가 아직 비어 있어요. 맞는 릴리스가 올라오면 바로 다운로드해요.
show-detail-nothing-fits = 피드에 규칙에 맞는 항목이 아직 없어요. 맞는 릴리스가 올라오면 바로 다운로드해요.
show-detail-complete = 완비
show-detail-open-in-library = 라이브러리에서 열기
show-detail-feed-finds-nothing = 이 피드에서 찾은 게 없어요
show-detail-feed-empty-query = { $source }에 “{ $query }”를 검색하지만 결과가 없어서, 새 항목이 규칙에 들어올 수 없어요. 보통 검색어를 줄이면 결과가 나와요. 뺀 단어는 대신 규칙에 남겨 둘 수 있어요.
show-detail-feed-empty = { $source }에 요청하지만 결과가 없어서, 새 항목이 규칙에 들어올 수 없어요. 보통 검색어를 줄이면 결과가 나와요. 뺀 단어는 대신 규칙에 남겨 둘 수 있어요.
show-detail-feed-empty-earlier = qBittorrent에는 이 피드의 이전 버전이 찾은 항목이 아직 보여요.
show-detail-repairing = 복구 중
show-detail-repair = 복구
show-detail-unsubscribe = 구독 해지
show-detail-remove-torrents =
    { $count ->
       *[other] 토렌트 { $count }개 제거
    }
show-detail-searches = 검색어
show-detail-quoted = “{ $words }”
show-detail-whole-feed = 피드 전체
show-detail-from = 소스
show-detail-rule = 규칙
show-detail-rule-unfollowable = 새 에피소드를 받을 수 없어요: 소스가 검색에는 응답하지만 피드가 없어요
show-detail-rule-active = 활성
show-detail-keeps = 포함
show-detail-keeps-value = “{ $words }”가 들어간 릴리스
show-detail-drops = 제외
show-detail-drops-value = { $words }가 들어간 릴리스
show-detail-or = { $first } 또는 { $second }
show-detail-last-took = 마지막으로 받은 항목
show-detail-saves-to = 저장 위치
show-detail-folder = 폴더
show-detail-in-the-feed = 피드 내용
show-detail-feed-reading = 피드 읽는 중.
show-detail-feed-offers = 지금 피드에 있는 항목, 최신순.
show-detail-feed-is-empty = 피드가 비어 있어요.
show-detail-read-again = 다시 읽기
show-detail-seeders =
    { $count ->
       *[other] 시더 { $count }
    }
show-detail-download = 다운로드
show-detail-torrents = 토렌트
show-detail-torrents-moving =
    { $count ->
       *[other] { $count }개 다운로드 중. 진행에 따라 막대가 채워져요.
    }
show-detail-torrents-held = 이 폴더에 대해 클라이언트가 가진 항목이에요.
show-detail-torrents-waiting = 규칙이 가져온 항목을 qBittorrent가 추가하기를 기다리는 중이에요.
show-detail-torrents-none = 이 폴더에 대해 클라이언트가 가진 항목이 없어요.

## SeriesPage.qml

series-trackers-both = { $first }와(과) { $second }
series-count-cleared = 에피소드 수를 지웠어요
series-count-set = 에피소드 { $count }개로 설정
series-held-one-undated = { $first }화 날짜 미정
series-held-range-undated = { $first }~{ $last }화 날짜 미정
series-held-one-due = { $first }화 { $when } 공개 예정
series-held-range-due = { $first }~{ $last }화 { $when } 공개 예정
series-held-month-format = yyyy년 M월
series-fix-schedule = 에피소드와 날짜
series-fix-files = 파일
series-fix-rescan-folder = 폴더 다시 스캔
series-fix-match = 매칭
series-find-match = 매칭 찾기
series-rescan-show = 작품 다시 스캔
series-open-in-show-info = 작품 정보에서 열기
series-delete-done =
    { $kind ->
        [film] 영화 { $count }개를 휴지통으로 옮겼어요
        [episode] 에피소드 { $count }개를 휴지통으로 옮겼어요
       *[file] 파일 { $count }개를 휴지통으로 옮겼어요
    }
series-delete-picked =
    { $kind ->
        [film] 영화 { $count }개 선택됨.
        [episode] 에피소드 { $count }개 선택됨.
       *[file] 파일 { $count }개 선택됨.
    }
series-delete-move-picked =
    { $kind ->
        [film] 영화 { $count }개를 휴지통으로 이동
        [episode] 에피소드 { $count }개를 휴지통으로 이동
       *[file] 파일 { $count }개를 휴지통으로 이동
    }
series-delete-click = 삭제할 파일을 클릭하세요.
series-delete-where = 휴지통으로 가니 되살릴 수 있어요. 시청 기록은 그대로 남아요.
series-delete-move = 휴지통으로 이동
series-pick-none = 선택 해제
series-pick-all = 모두 선택
series-rescan-started = 다시 스캔 시작됨
series-rated = { $score }점 평가함
series-rated-cleared = 평가를 지웠어요
series-score-failed = 점수 저장 실패
series-still-downloading = { $episode }은(는) 아직 다운로드 중이에요
series-format-series = 시리즈
series-tracker-refused = { $tracker }이(가) 받지 않았어요
series-tracker-refused-why = { $tracker }이(가) 받지 않았어요: { $why }
series-watched-up-to = { $episode }까지 시청함
series-unwatched-from = { $episode }부터 시청 안 함
series-undo = 되돌리기
series-play-downloading = { $number }화 다운로드 중
series-play-downloading-unnumbered = 에피소드 다운로드 중
series-play-continue = 이어 보기
series-play-again = 다시 보기
series-play-continue-episode = { $number }화 이어 보기
series-play-again-from = { $number }화부터 다시 보기
series-play-episode = { $number }화 재생
series-page-not-tracked = 추적 안 함
series-not-tracked-on-disk =
    { $count ->
       *[other] 추적 안 함, 디스크에 에피소드 { $count }개
    }
series-watched = { $count }화 시청
series-watched-of = { $total }화 중 { $watched }화 시청
series-watched-of-estimate = { $total }+화 중 { $watched }화 시청
series-facts-episodes-estimate =
    { $count ->
       *[other] 에피소드 { $count }개 이상
    }
series-facts-each = 화당 { $runtime }
series-facts-on-disk = 디스크에 { $count }개
series-next = 다음: { $episode }
series-episodes-downloading =
    { $count ->
       *[other] 에피소드 { $count }개 다운로드 중
    }
series-all-watched = 모두 시청함
series-no-poster = 포스터 없음
series-next-season-span = S{ $season } { $span }
series-next-predicted = { $span } { $date } 예상, { $wait } 후
series-next-date-format = M월 d일 (ddd)
series-next-airs =
    { $count ->
       *[other] { $span } 방영: { $wait } 후
    }
series-downloading-episodes =
    { $count ->
       *[other] 에피소드 { $count }개 다운로드 중
    }
series-count-season-ends = 시즌 { $season }은(는) { $number }화에서 완결
series-count-ends = { $number }화에서 완결
series-count-gave-way = 에피소드 수를 { $count }개로 설정했지만 더 이상 적용되지 않아요
series-count-set-by-you = { $line }, 직접 설정
series-change = 변경
series-get-episodes = 에피소드 받기
series-your-score = 내 점수 { $score }
series-rate = 평가
series-page-average-score = 평균 점수 { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = 투표 { $votes }건
series-add-to-list = 목록에 추가
series-page-rewatched =
    { $count ->
       *[other] { $count }회 다시 봄
    }
series-your-notes = 내 메모
series-add-notes = 메모 추가
series-notes-tip = 목록 항목에 붙는 메모, { $sites }에 저장돼요
series-unhide = 숨김 해제
series-hide = 숨기기
series-hidden-toast = { $title }을(를) 여기서 숨겼고, AniBeam이 AniList와 MyAnimeList 동기화를 멈춰요
series-unhidden-toast = { $title }이(가) 다시 보이고, 동기화도 재개돼요
series-hide-tip-tracked = 여기 모든 페이지에서 숨기고, AniBeam이 AniList와 MyAnimeList 동기화를 멈춰요
series-hide-tip = 모든 목록에서 숨겨요
series-fixes = 수정
series-fixes-tip = 파일, 에피소드 수, 매칭을 수정하거나 다시 스캔
series-quick-fix = 빠른 수정
series-film = 영화
series-episodes = 에피소드
series-extras = 오프닝, 엔딩 외
series-extras-openings = 오프닝
series-extras-endings = 엔딩
series-extras-previews = 예고편
series-extras-specials = 스페셜
series-extras-other = 기타
series-extras-group = { $kind }, { $count }
series-extra-files = 남는 파일
series-extra-files-warning =
    { $count ->
       *[other] 파일 { $count }개가 예상 에피소드 수 { $total }개를 넘어요: 이름이 잘못됐거나, 중복이거나, 스페셜이에요. 이름을 바꾸거나 제거하세요.
    }
series-synopsis = 줄거리
series-less = 접기
series-more = 더 보기
series-match = 매칭
series-not-matched = 아직 매칭되지 않아서 포스터, 줄거리, 기록이 없어요. AniList나 TVmaze에서 작품 정보를 찾으세요.
series-naming-title = 매칭되는 이름
series-naming-body = 폴더 이름은 AniList 표기대로 작품 제목이고, 첫 시즌이 아니면 뒤에 시즌을 붙여요. 파일마다 에피소드 번호가 들어가요.
series-fix-how-files-read = 파일 읽는 방식 수정
series-characters = 캐릭터
series-related = 관련 작품
series-related-filling = AniList에서 아직 채우는 중이에요.
series-tags = 태그
series-hide-spoilers = 스포일러 숨기기
series-spoilers = 스포일러
series-all-tags = 모든 태그
series-tag = { $name }, { $rank }%
series-recommendations = 추천
series-relation-owned = 내 라이브러리
series-relation-unreleased = 미공개
series-relation-on-anilist = AniList에 있음
series-score-saved = 점수 저장됨
series-score-cleared = 점수를 지웠어요

## GetShow.qml

get-show-the-client = 토렌트 클라이언트
get-show-group-batch = { $group } 일괄
get-show-run = { $first }~{ $last }
get-show-episodes =
    { $count ->
       *[other] { $list }화
    }
get-show-episodes-inline =
    { $count ->
       *[other] { $list }화
    }
get-show-sending-to = { $client }에 전송 중
get-show-go =
    { $count ->
       *[other] 에피소드 { $count }개 받기
    }
get-show-go-replace =
    { $count ->
       *[other] 에피소드 { $count }개 받고 교체
    }
get-show-go-subscribe =
    { $count ->
       *[other] 에피소드 { $count }개 받고 구독
    }
get-show-go-replace-subscribe =
    { $count ->
       *[other] 에피소드 { $count }개 받고 교체, 구독
    }
get-show-subscribe = 새 에피소드 구독
get-show-nothing-new = 새로 받을 항목 없음
get-show-pick-episode = 에피소드 선택
get-show-replace-off = 꺼짐. 라이브러리에 이미 있어서 { $episodes } 선택은 제외했어요. 켜면 교체해요.
get-show-replace-on = 새 파일이 다 받아지면 { $episodes }의 기존 파일은 휴지통으로 가요.
get-show-replace-on-the-way =
    { $count ->
       *[other] { $episodes }을(를) 지금 다운로드 중이에요: 그 다운로드는 취소되고 받은 부분은 휴지통으로 가요.
    }
get-show-kind-on-demand = 검색할 때 시작
get-show-kind-rss-search = RSS 검색
get-show-kind-rss-feed = RSS 피드
get-show-kind-json-search = JSON 검색
get-show-kind-indexer-search = 인덱서 검색
get-show-ready = 준비됨
get-show-running = 실행 중
get-show-not-running = 실행 안 됨
get-show-not-running-start = 실행 중이 아니에요. 위의 소스를 클릭해서 시작하세요.
get-show-switched-off = 꺼짐
get-show-click-to-see = 클릭해서 이 작품의 릴리스 보기
get-show-searching = 검색 중
get-show-searching-source = { $name } 검색 중
get-show-searching-elapsed = { $label }, { $seconds }초
get-show-releases =
    { $count ->
       *[other] 릴리스 { $count }개
    }
get-show-releases-covered =
    { $count ->
       *[other] 릴리스 { $count }개, 에피소드 { $total }개 중 { $cover }개 포함
    }
get-show-on-your-list = 내 목록에 있음
get-show-plan-to-watch = 시청 예정
get-show-on-your-list-tip = 목록에 있는 작품이에요
get-show-plan-to-watch-tip = 시청 예정 목록에 추가
get-show-open-series-tip = 작품 페이지: 상세 정보, 에피소드, 재생
get-show-next-in = { $number }화: { $wait } 후
get-show-airing = 방영 중
get-show-finished = 방영 종료
get-show-total-episodes = 에피소드 { $count }개
get-show-downloading-episodes =
    { $count ->
       *[other] 에피소드 { $count }개 다운로드 중
    }
get-show-downloading-torrents =
    { $count ->
       *[other] 토렌트 { $count }개 다운로드 중
    }
get-show-in-library = 라이브러리에 있음
get-show-not-in-library = 라이브러리에 없음
get-show-downloading = 다운로드 중
get-show-stop-all = 모두 중지
get-show-start-all = 모두 시작
get-show-stop-all-tip = 이 작품의 모든 다운로드 중지. 받은 부분은 남아요
get-show-start-all-tip = 이 작품의 모든 다운로드 다시 시작
get-show-cancel-all = 모두 취소
get-show-cancel-all-tip = 이 작품의 모든 다운로드를 취소하고 받은 부분은 휴지통으로 옮겨요
get-show-sources = 사용 가능한 소스
get-show-check-again = 다시 확인
get-show-check-again-tip = 필요할 때 시작하는 각 제공처가 실행 중인지 물어봐요. 아무것도 시작하지 않아요.
get-show-search-source = { $name } 검색
get-show-release-list = 모든 릴리스를 목록으로
get-show-your-library = 내 라이브러리
get-show-library-on-disk = 디스크에 { $total }개 중 { $have }개
get-show-library-downloading = { $count }개 다운로드 중
get-show-library-nothing = 아직 없음
get-show-row-batch = { $group }, 일괄
get-show-batch-counting = 에피소드 세는 중, 토렌트 1개
get-show-batch-unlisted = 에피소드 목록 없음, 제목 기준 작품 전체, 토렌트 1개
get-show-batch-episodes =
    { $count ->
       *[other] 에피소드 { $count }개, 토렌트 1개
    }
get-show-nothing-new-picked = 새로 선택한 항목 없음
get-show-nothing-picked = 선택한 항목 없음
get-show-pick-provider = 제공처 선택
get-show-plan-line = { $source }의 { $groups }에서 { $episodes }.
get-show-and = { $first }, { $second }
get-show-library-has =
    { $count ->
       *[other] 라이브러리에 { $episodes }이(가) 있어요. 바꾸려면 아래의 기존 에피소드 교체를 켜세요.
    }
get-show-together = 합계
get-show-torrents-unit =
    { $count ->
       *[other] 토렌트
    }
get-show-replace = 기존 에피소드 교체
get-show-sequential = 순서대로 다운로드
get-show-sequential-on = 켜짐: 새 토렌트마다 조각을 처음부터 받아서 앞 에피소드가 먼저 끝나고, 나머지를 받는 동안 볼 수 있어요. 전체적으로는 조금 느려요.
get-show-sequential-off = 꺼짐: 클라이언트가 스웜이 주는 순서대로 조각을 받아요.
get-show-rolling = 다음 몇 화 미리 받기
get-show-rolling-on = 켜짐: 지금 { $count }화를 받고, 한 화를 다 볼 때마다 다음 화를 시작해요.
get-show-rolling-on-delete = 켜짐: 지금 { $count }화를 받고, 한 화를 다 볼 때마다 다음 화를 시작해요. 본 에피소드는 디스크에서 지워져요.
get-show-rolling-off = 꺼짐: 선택한 에피소드를 한꺼번에 다운로드해요.
get-show-follow-off = 꺼짐: 새 에피소드는 여기에 빈칸으로 보이고, 저절로 다운로드되지 않아요.
get-show-follow-no-feed = { $source }에는 구독할 피드가 없어서 새 에피소드가 자동으로 다운로드되지 않아요.
get-show-follow-pick-group = 먼저 그룹을 고르세요: 구독은 그룹 하나와 화질 하나를 따라가요.
get-show-follow-batch-only = 여기서 { $group }은(는) 일괄만 있어서 구독할 개별 에피소드 피드가 없어요.
get-show-follow-on = 켜짐: { $source }의 { $feed } 피드. 새 에피소드가 올라오는 대로 다운로드해요.
get-show-confirm =
    { $count ->
       *[other] { $episodes }을(를) 지금 다운로드 중이에요. 교체하면 그 다운로드가 취소되고 받은 부분은 휴지통으로 가요.
    }
get-show-confirm-go = 취소하고 교체
get-show-state-error = 오류
get-show-state-files-missing = 파일 없음
get-show-state-stopped = 중지됨
get-show-state-waiting-peers = 피어 기다리는 중
get-show-state-metadata = 메타데이터 가져오는 중
get-show-state-queued = 대기열
get-show-state-checking = 파일 확인 중
get-show-state-allocating = 공간 할당 중
get-show-state-moving = 파일 이동 중
get-show-state-waiting = 대기 중
get-show-a-batch = 일괄
get-show-no-group = 이름에 그룹 없음
get-show-likely = { $source }(으)로 추정
get-show-tracker = 트래커 { $tracker }
get-show-not-known = 아직 모름
get-show-bytes-of = { $size } 중 { $done }
get-show-speed-down = 다운 { $speed }/s
get-show-time-left = { $wait } 남음
get-show-stopping = 중지 중
get-show-starting = 시작 중
get-show-stop = 중지
get-show-start-tip = 다운로드 다시 시작
get-show-stop-tip = 다운로드 중지. 받은 부분은 남아요
get-show-cancelling = 취소 중
get-show-cancel-tip = 이 다운로드를 취소하고 받은 부분은 휴지통으로 옮겨요
get-show-episode-order = 에피소드 순서대로 다운로드
get-show-episode-order-on = 켜짐: 각 파일의 처음부터 조각을 받아서 앞 에피소드가 먼저 끝나요.
get-show-cube-skipped = { $number }화: 건너뜀
get-show-cube-done = { $number }화: 완료
get-show-cube-percent = { $number }화: { $percent }%
get-show-fact-torrent = 토렌트
get-show-fact-release = 릴리스
get-show-fact-from = 소스
get-show-fact-saving-to = 저장 위치
get-show-span-counting = 세는 중
get-show-span-unknown = { $runs }?
get-show-span-episodes = { $runs }화
get-show-cube-no-release = { $number }화: { $group } 릴리스 없음
get-show-seeders = 시더 { $count }
get-show-cube-on-disk = 이미 디스크에 있음, 클릭하면 교체
get-show-cube-downloading = 다운로드 중, 클릭하면 교체

## TorrentRow

torrent-row-state-downloading = 다운로드 중
torrent-row-state-stalled = 정체됨
torrent-row-state-metadata = 메타데이터 가져오는 중
torrent-row-state-queued = 대기열
torrent-row-state-stopped = 중지됨
torrent-row-state-done = 완료
torrent-row-state-seeding = 시딩 중
torrent-row-state-checking = 확인 중
torrent-row-state-moving = 이동 중
torrent-row-state-error = 오류
torrent-row-state-files-missing = 파일 없음
torrent-row-state-making-room = 공간 확보 중
torrent-row-state-finding-peers = 피어 찾는 중
torrent-row-state-waiting = 대기 중
torrent-row-done-of-size = { $size } 중 { $done }
torrent-row-speed-down = 다운 { $speed }/s
torrent-row-speed-up = 업 { $speed }/s
torrent-row-time-left = { $time } 남음
torrent-row-seeders =
    { $count ->
       *[other] 시더 { $count }
    }
torrent-row-rolling = 순환, { $total }화 중 { $next }~{ $through }화
torrent-row-stop = 중지

## SourceRow

source-row-unavailable-fix = 드라이브를 연결한 뒤 다시 스캔하세요. 옮겼다면 제거하고 새 위치를 추가하세요.
source-row-counts = 작품 { $count }개
source-row-counts-movies =
    { $movies ->
       *[other] 작품 { $count }개, 영화 폴더 { $movies }개
    }
source-row-unavailable = 사용할 수 없음
    .tip = AniBeam이 이 폴더에 접근할 수 없어요. 연결되지 않았거나 마운트되지 않은 드라이브에 있다면 연결하세요. AniBeam이 다시 찾아요. 옮겼다면 여기서 제거하고 새 위치를 추가하세요.
source-row-rescan = 다시 스캔
source-row-remove-question = { $name }과(와) 작품 { $count }개, 그 기록을 제거할까요?

## StartSourceSheet

start-source-title = { $name }용 { $service }을(를) 시작할까요?
start-source-busy = { $service } 시작 중. 응답하면 { $name }이(가) 켜져요. 최대 1분 걸릴 수 있어요.
start-source-body = { $service }은(는) AniBeam 대신 { $name }을(를) 검색하는 서비스예요. 시작하면 이 컴퓨터의 백그라운드에서 실행되고, AniBeam을 닫아도 계속 실행돼요.
start-source-starting = 시작 중

## SearchProgress

search-progress-why-helper = AniBeam이 검색 도우미를 시작하는 중이에요. 1~2초 걸려요.
search-progress-why-starting = { $endpoint }은(는) 검색할 때 시작하는 도우미를 거쳐요. 첫 검색은 최대 { $seconds }초 기다리고, 그다음부터는 바로 해요.
search-progress-why-reading = 릴리스를 그룹별로 나누고 라이브러리와 비교하는 중이에요.
search-progress-why-ondemand = 새 검색마다 도우미가 { $endpoint }을(를) 새로 읽어요. 보통 10~20초 걸려요.
search-progress-why-slow = 지금 { $endpoint }의 응답이 느려요.
search-progress-why-usual = 보통 1~2초 걸려요.
search-progress-why-and-slow = { $why } 평소보다 오래 걸리고 있어요. 취소하고 다른 소스를 써 보세요.
search-progress-searching = 검색 중
search-progress-seconds = { $seconds }초
search-progress-try = { $endpoint } 시도

## ReleaseTable

release-table-col-release = 릴리스
release-table-col-group = 그룹
release-table-col-res = 해상도
release-table-col-rip = 소스
release-table-col-ep = 화
release-table-col-size = 크기
release-table-col-seeders = 시더
release-table-col-leechers = 리처
release-table-col-downloads = 다운로드
release-table-col-released = 공개일
release-table-tick-all = 모든 릴리스 선택
release-table-untick-all = 모든 릴리스 선택 해제
release-table-on-disk = 디스크에 있음
release-table-other-on-disk = 다른 릴리스 있음
release-table-season-batch = 시즌 { $season } 일괄
release-table-batch = 일괄
release-table-via = { $source }에서
release-table-season-episode = S{ $season } { $episode }
release-table-date-format = yyyy년 M월 d일 HH:mm
release-table-download = 다운로드
release-table-more = 외 { $count }개. 다른 항목을 먼저 보려면 목록을 좁히거나 정렬하세요.
release-table-pick = 이 릴리스 선택

## DownloadsPage

downloads-title = 다운로드
downloads-builtin-client = 내장 클라이언트
downloads-start-done = { $name }이(가) 실행 중이고 켜져 있어요
downloads-disk-film = 영화
downloads-disk-watched = { $size } 시청함
downloads-disk-all-watched = 모두 시청함
downloads-delete-watched-menu =
    { $count ->
       *[other] 시청한 모든 에피소드: { $count }개, { $size }
    }
downloads-delete-finished-menu =
    { $count ->
       *[other] 다 본 모든 완결 작품: { $count }개, { $size }
    }
downloads-titles-and = { $list }, { $last }
downloads-titles-more = { $list } 외 { $count }개
downloads-delete-finished-none = 끝까지 본 완결 작품이 없어요
downloads-delete-watched-none = 디스크에 시청한 항목이 없어요
downloads-delete-finished-heading =
    { $count ->
       *[other] 완결 작품 { $count }개를 삭제할까요? ({ $size })
    }
downloads-delete-finished-base = { $titles }. 제공처에서 완결로 표시했고 모든 에피소드를 시청한 작품만 해당돼요. 아직 방영 중이거나 종영 여부가 추정인 작품은 남아요. 번외를 포함해 모든 파일이 휴지통으로 가요. 토렌트도 { $client }에서 빠지므로 다시 다운로드할 수 있어요.
downloads-delete-watched-heading =
    { $count ->
       *[other] 시청한 에피소드 { $count }개를 삭제할까요? ({ $size })
    }
downloads-delete-watched-base = 모든 작품에서 시청한 모든 에피소드가 대상이에요. 아직 안 본 에피소드와 번외는 남아요. 파일과 자막은 휴지통으로 가니 거기서 되살릴 수 있어요. 토렌트도 { $client }에서 빠지므로 다시 다운로드할 수 있어요.
downloads-move-to-trash = 휴지통으로 이동
downloads-remove-group-torrents =
    { $count ->
       *[other] 토렌트 { $count }개 제거
    }
downloads-run-span = { $from }~{ $to }
downloads-run-pair = { $first }, { $second }
downloads-gap-missing =
    { $count ->
       *[other] { $runs }화 누락
    }
downloads-gap-not-downloaded =
    { $count ->
       *[other] { $runs }화 다운로드 안 됨
    }
downloads-gap-then-not-downloaded = { $runs }화 다운로드 안 됨
downloads-trying-query = { $query } 시도 중
downloads-client-settings = 토렌트 클라이언트 설정
downloads-error-unknown = 문제가 생겼어요. 다시 시도하세요.
downloads-search-all-off = 모든 소스가 꺼져 있어요. 검색하려면 하나를 켜세요.
downloads-turn-on = { $name } 켜기
downloads-search-no-sources = 아직 검색할 소스가 없어요. 설정의 다운로드에서 소스 목록을 추가하세요.
downloads-search-none-started = 검색을 시작할 수 있는 소스가 없어요. 잠시 후 다시 시도하세요.
downloads-auto-resolution = 제공되는 가장 선명한 { $resolution }(으)로 좁혔어요
downloads-search-one-failed = { $name } 검색이 실패했어요. 다시 시도하거나 다른 소스를 검색하세요.
downloads-search-all-failed = 검색할 수 있는 소스가 없었어요. 잠시 후 다시 시도하세요.
downloads-narrowing-on = { $name }에서 좁히는 중
downloads-refine-zero = 남는 게 없어지므로 목록을 그대로 둬요.
downloads-refine-no-match = 그 단어와 일치하는 릴리스가 없어요.
downloads-refine-empty = 좁힐 단어를 입력하세요.
downloads-refine-no-change = 바뀌는 게 없어요.
downloads-movie-tick-one = 릴리스 하나를 선택한 뒤 영화로 다운로드를 누르세요.
downloads-group-downloading = 다운로드 중
downloads-group-downloading-line = 진행 중. 자세한 내용은 오른쪽 활동에 있어요.
downloads-group-seeding = 시딩 중
downloads-group-seeding-line = 다운로드했고 아직 공유 중이에요.
downloads-group-done = 완료
downloads-group-done-line = 다운로드했고 클라이언트가 공유하지 않아요.
downloads-group-stopped = 중지됨
downloads-group-stopped-line = 끝나기 전에 멈췄어요.
downloads-group-broken = 파일 없음
downloads-group-broken-line = 클라이언트가 파일을 찾을 수 없어요: 없는 폴더를 가리키고 있어요. 토렌트를 제거하면 클라이언트가 정리돼요.
downloads-sub-nofeed = 구독 불가
downloads-sub-nofeed-line = 소스가 검색에는 응답하지만 피드가 없어서 새 에피소드가 오지 않아요. 구독을 해지하고 피드가 있는 소스에서 구독하세요.
downloads-sub-complete = 완비
downloads-sub-complete-line = 모든 에피소드가 라이브러리에 있고 더 방영되지 않아요: 이 구독들은 할 일이 없어요.
downloads-sub-airing = 방영 중
downloads-sub-airing-line = 앞으로 나올 새 에피소드, 가까운 순.
downloads-sub-incomplete = 불완전
downloads-sub-incomplete-line = 방영이 끝났지만 없는 에피소드가 있거나, 작품 상태를 알 수 없어요.
downloads-sub-missing = 라이브러리에 없음
downloads-sub-missing-line = 폴더에 라이브러리가 찾은 항목이 없어요: 아직 받은 게 없거나, 라이브러리가 보지 않는 곳에 저장하고 있어요.
downloads-torrents =
    { $count ->
       *[other] 토렌트 { $count }개
    }
downloads-rule-film-on-disk = 영화, 디스크에 있음
downloads-rule-all-here = { $total }화 모두 있음
downloads-rule-next =
    { $predicted ->
        [yes] { $episode }화 { $time } 후 예상
       *[no] { $episode }화 { $time } 후
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] { $episode }~{ $through }화 { $time } 후 예상
       *[no] { $episode }~{ $through }화 { $time } 후
    }
downloads-rule-have-of = { $total }화 중 { $have }화 있음
downloads-rule-have =
    { $count ->
       *[other] 에피소드 { $count }개 있음
    }
downloads-tag-not-downloaded = 다운로드 안 됨
downloads-tag-missing = { $count }개 누락
downloads-tag-n-not-downloaded = { $count }개 다운로드 안 됨
downloads-tag-unknown = 알 수 없음
downloads-speed-down = 다운 { $speed }/s
downloads-time-left = { $time } 남음
downloads-repair-trying = 더 짧은 검색어로 시도 중
downloads-sent-to-client = { $title }을(를) { $client }에 보냈어요
downloads-files-torrents = 토렌트가 받은 파일
downloads-files-torrents-remote = 토렌트가 받은 파일, { $client }이(가) 삭제
downloads-files-folder-and = { $folder }와(과) { $torrents }
downloads-unsubscribe-heading = { $title } 구독을 해지할까요?
downloads-unsubscribe-base = 피드와 다운로드 규칙이 { $client }에서 빠지므로 새 항목이 오지 않아요.
downloads-unsubscribe-complete-heading = 완비된 작품 { $count }개의 구독을 모두 해지할까요?
downloads-unsubscribe-complete-base = 피드와 다운로드 규칙이 { $client }에서 빠져요. 모든 에피소드가 라이브러리에 있어요.
downloads-files-folders-and-torrents = 폴더와 토렌트가 받은 파일
downloads-one-at-a-time = 하나씩 처리되고, 진행되는 대로 페이지에 표시돼요.
downloads-subscription-stays = { $base } 구독은 유지돼요.
downloads-cancel-heading = { $title }을(를) 취소할까요?
downloads-cancel-group-base =
    { $count ->
       *[other] 다운로드 { $count }개가 멈추고 { $client }에서 빠져요.
    }
downloads-files-so-far-it = 지금까지 받은 파일
downloads-files-so-far-they = 지금까지 받은 파일
downloads-cancel-download = 다운로드 취소
downloads-keep-downloading = 계속 다운로드
downloads-remove-heading = { $title }을(를) 제거할까요?
downloads-remove-group-base =
    { $count ->
       *[other] 토렌트 { $count }개가 { $client }에서 빠져요. 아래를 선택하지 않으면 파일은 남아요.
    }
downloads-remove-detail-base =
    { $count ->
       *[other] 토렌트 { $count }개가 { $client }에서 빠져요. 아래를 선택하지 않으면 파일은 남아요.
    }
downloads-remove-broken-heading = 파일이 없는 작품 { $count }개의 토렌트를 모두 제거할까요?
downloads-remove-broken-base = 토렌트 { $count }개가 { $client }에서 빠져요. 없는 폴더를 가리키므로 파일은 건드리지 않고, 구독은 유지돼요.
downloads-remove-torrent-heading = 이 토렌트를 제거할까요?
downloads-files-it-downloaded = 받은 파일
downloads-cancel-torrent-heading = 이 다운로드를 취소할까요?
downloads-cancel-many-heading =
    { $count ->
       *[other] 다운로드 { $count }개를 취소할까요?
    }
downloads-cancel-many-base = 이 작품의 모든 다운로드가 멈추고 { $client }에서 빠져요.
downloads-cancel-all = 모두 취소
downloads-summary-connecting = 연결 중.
downloads-summary-client-on = { $client } { $version } 연결됨.
downloads-summary-client-off = { $client }이(가) 연결되지 않았어요. 설정의 다운로드에서 주소와 비밀번호를 확인하세요.
downloads-summary-native = AniBeam이 소스를 직접 검색해요.
downloads-summary-native-with = AniBeam이 소스를 직접 검색해요. { $client }
downloads-summary-starting = 다운로드 시작 중.
downloads-summary-not-running = 다운로드가 실행 중이 아니에요.
downloads-summary-not-started = 다운로드가 아직 시작되지 않았어요.
downloads-summary-helper-ready = 검색 도우미 { $version } 준비됨. { $client }
downloads-subscribed-as = { $name }(으)로 구독했어요.
downloads-repaired =
    { $count ->
       *[other] { $name } 복구: 피드에서 릴리스 { $count }개를 찾았어요
    }
downloads-repaired-keeps =
    { $count ->
       *[other] { $name } 복구: 피드에서 릴리스 { $count }개를 찾았고 규칙은 “{ $term }”을(를) 유지해요
    }
downloads-sent-torrents =
    { $count ->
       *[other] 토렌트 { $count }개를 { $client }에 보냈어요.
    }
downloads-added-torrents =
    { $count ->
       *[other] 토렌트 { $count }개를 추가했어요.
    }
downloads-trashed-paths =
    { $count ->
       *[other] { $title }의 경로 { $count }개를 휴지통으로 옮겼어요
    }
downloads-show-gone = 그 작품은 더 이상 라이브러리에 없어요
downloads-activity-hide = 활동 숨기기
downloads-activity = 활동
downloads-moving =
    { $count ->
       *[other] 다운로드 { $count }개 진행 중
    }
downloads-back = 다운로드로 돌아가기
downloads-chip-search-ready = 검색 준비됨
downloads-chip-search-starting = 검색 시작 중
downloads-chip-search-off = 검색 꺼짐
downloads-chip-answers-at = { $summary } 주소: { $url }.
downloads-chip-helper-missing = 검색 도우미가 설치되지 않았어요. 클릭하면 추가 방법이 있는 다운로드 설정이 열려요
downloads-chip-helper-failed = 검색 도우미가 멈췄어요. 다시 연결하면 다시 시작해요. 무엇이 잘못됐는지는 다운로드 설정에 있어요
downloads-chip-open-settings = 다운로드 설정 열기
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } 꺼짐
downloads-chip-client-not-connected = { $client }이(가) 연결되지 않았어요. 다시 연결하거나 설정에서 주소를 확인하세요
downloads-chip-client-remote = 직접 실행하는 검색 도우미는 자체 설정에 지정된 클라이언트를 써요
downloads-chip-client-switch = 클릭해서 클라이언트 전환
downloads-connecting = 연결 중
downloads-reconnect = 다시 연결
downloads-search-placeholder = 작품 또는 릴리스 이름 검색
downloads-search-go-tip = { $names }에서 릴리스 이름 검색
downloads-search-go-off = 모든 소스가 꺼져 있어요.
downloads-suggest-airing =
    { $predicted ->
        [yes] 방영 중, { $episode }화 { $time } 후 예상
       *[no] 방영 중, { $episode }화 { $time } 후
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] 방영 중, { $episode }~{ $through }화 { $time } 후 예상
       *[no] 방영 중, { $episode }~{ $through }화 { $time } 후
    }
downloads-suggest-finished = 방영 종료
downloads-suggest-episodes = 에피소드 { $count }개
downloads-suggest-in-library = 내 라이브러리에 있음
downloads-suggest-on-anilist = AniList에 있음, 라이브러리에는 없음
downloads-plan-to-watch = 시청 예정
downloads-suggest-release-search = 대신 { $names }에서 “{ $query }” 릴리스 이름 검색
downloads-key-return = Enter
downloads-sources = 소스
downloads-source-hint-starting = 시작 중
downloads-source-hint-down = 실행 안 됨
downloads-source-tip-down = { $name }에 필요한 도우미 서비스가 실행 중이 아니에요. 클릭해서 시작하세요.
downloads-source-tip-on = { $name }에서 검색해요. 클릭해서 끄기.
downloads-source-tip-off = 꺼짐. 클릭해서 { $name } 다시 사용.
downloads-source-tip-url = { $tip } { $url }
downloads-check = 확인
    .tip = 필요할 때 시작하는 각 소스가 실행 중인지 물어봐요. 아무것도 시작하지 않아요.
downloads-check-busy = 확인 중
downloads-back-to-grid = 격자로 돌아가기
    .tip = { $title }와(과) 각 제공처가 찾은 항목으로 돌아가요. Esc로도 돌아가요.
downloads-releases-on =
    { $count ->
       *[other] { $name }의 릴리스 { $count }개
    }
downloads-found-on = 찾은 곳
downloads-found-searching = 검색 중
downloads-found-failed = 실패
downloads-found-cancelled = 취소됨
downloads-releases =
    { $count ->
       *[other] 릴리스 { $count }개
    }
downloads-found-retry-tip =
    { $why }
    클릭해서 다시 시도
downloads-facet-groups = 릴리스 그룹
downloads-facet-quality = 화질
downloads-facet-words = 단어
downloads-filters-fewer = 필터 접기
downloads-filters-more = 필터 더 보기({ $count })
downloads-narrow-placeholder = 단어로 좁히기, 제외는 -단어
downloads-best-match = 최적
    .tip = 가장 좋아 보이는 릴리스와 비슷한 것만 남겨요: { $query }
downloads-best-match-none = 여기서 눈에 띄는 릴리스가 없어요
downloads-strip-have-of = 디스크에 에피소드 { $total }개 중 { $have }개
downloads-strip-have =
    { $count ->
       *[other] 디스크에 에피소드 { $count }개
    }
downloads-strip-next =
    { $predicted ->
        [yes] { $episode }화 { $time } 후 예상
       *[no] { $episode }화 { $time } 후
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] { $episode }~{ $through }화 { $time } 후 예상
       *[no] { $episode }~{ $through }화 { $time } 후
    }
downloads-strip-complete = 완비
downloads-strip-line-more = { $disk }, { $more }.
downloads-sentence = { $text }.
downloads-strip-none-on-disk = 이미 가진 에피소드의 릴리스는 없어요.
downloads-strip-on-disk =
    { $count ->
       *[other] 릴리스 { $count }개는 가진 파일 그대로라 디스크에 있음으로 표시했어요
    }
downloads-strip-other-on-disk =
    { $count ->
       *[other] 릴리스 { $count }개는 다른 릴리스로 이미 가진 에피소드예요
    }
downloads-strip-both = { $first }. { $second }.
downloads-tick-missing =
    { $count ->
       *[other] 누락된 에피소드 { $count }개 선택
    }
    .tip = 라이브러리에 없는 에피소드마다 시더가 가장 많은 릴리스를 골라요
downloads-open-in-library = 라이브러리에서 열기
downloads-not-this-show = 이 작품이 아님
downloads-no-releases = 릴리스 없음
downloads-no-releases-body = { $name }에 “{ $query }”와(과) 일치하는 항목이 없어요. 다른 소스나 더 적은 단어로 찾아보세요.
downloads-try-source = { $name } 시도
downloads-ticked =
    { $count ->
       *[other] 릴리스 { $count }개 선택됨
    }
downloads-nothing-ticked = 선택한 항목 없음
downloads-ticked-size = 합계 { $size }.
downloads-tick-hint = 표에서 릴리스를 선택하거나 모두 다운로드하세요.
downloads-tick-all = 모두 선택
downloads-working = 처리 중
downloads-download-ticked =
    { $count ->
       *[other] 릴리스 { $count }개 다운로드
    }
downloads-download-shown = 표시된 { $count }개 다운로드
downloads-download-all = { $count }개 모두 다운로드
downloads-download-movie = 영화로 다운로드
    .tip = 선택한 릴리스 하나를 바로 영화 폴더로
downloads-subscribe = 구독
downloads-subscribe-body = 이 검색과 일치하는 새 릴리스가 올라오는 대로 여기 지정한 폴더로 다운로드해요.
downloads-name-placeholder = 첫 릴리스에서 가져옴
downloads-subscribe-search = 이 검색 구독
downloads-subscribe-no-feed = 여기에는 구독할 피드가 없어요
downloads-preview-again = 규칙 다시 보기
downloads-preview-show = 만들어질 규칙 보기
downloads-preview-from = 소스: { $host }
downloads-preview-from-named = 소스: { $host } ({ $name })
downloads-preview-must = 포함해야 함: { $terms }
downloads-preview-must-not = 포함하면 안 됨: { $terms }
downloads-helper-changed = 다운로드가 응답하지 않는 도우미로 설정돼 있어요
downloads-helper-missing = 이 AniBeam에는 다운로드 기능이 없어요
downloads-helper-failed = 다운로드를 시작하지 못했어요
downloads-helper-changed-body = 검색과 다운로드는 작은 도우미 프로그램을 거쳐요. 설정이 AniBeam 자체 도우미가 아닌 다른 도우미를 가리키고 있고, 그 도우미가 응답하지 않아요. 되돌리면 저절로 시작해요.
downloads-helper-missing-body = 검색과 다운로드에 쓰는 도우미 프로그램 없이 빌드됐어요. 릴리스 페이지의 설치 파일에는 들어 있어요. 이 사본에 추가하는 방법은 설정, 다운로드에 있어요.
downloads-helper-failed-body = 검색과 다운로드에 쓰는 도우미 프로그램이 준비되기 전에 멈췄어요. 대부분 다시 시작하면 해결돼요.
downloads-helper-use-own = AniBeam 자체 도우미 사용
downloads-helper-start-again = 다시 시작
downloads-helper-show-how = 방법 보기
downloads-helper-what-went-wrong = 무엇이 잘못됐나요
downloads-lens-gaps-count = 라이브러리 누락  { $count }
downloads-lens-airing-count = 방영 중  { $count }
downloads-lens-following-count = 구독 중  { $count }
downloads-lens-downloaded-count = 받은 항목  { $count }
downloads-lens-gaps = 라이브러리 누락
downloads-lens-gaps-line = 방영했지만 디스크에 없는 에피소드예요. 작품을 열어 누가 갖고 있는지 확인하세요.
downloads-lens-gaps-none = 누락 없음: 모든 작품의 방영된 에피소드가 디스크에 있어요.
downloads-lens-airing-line = 라이브러리에서 앞으로 나올 에피소드가 있는 작품이에요. 하나를 열어 구독하세요.
downloads-lens-airing-none = 라이브러리에 방영 중인 작품이 없어요
downloads-lens-airing-none-body = 라이브러리에 있고 앞으로 나올 에피소드가 있는 작품이 여기에 표시돼요. 방영 중에서 방영하는 모든 작품을 볼 수 있어요.
downloads-airing-now = 방영 중
downloads-lens-following = 구독 중
downloads-card-next =
    { $predicted ->
        [yes] { $episode }화 { $time } 후 예상
       *[no] { $episode }화 { $time } 후
    }
downloads-card-next-span =
    { $predicted ->
        [yes] { $episode }~{ $through }화 { $time } 후 예상
       *[no] { $episode }~{ $through }화 { $time } 후
    }
downloads-rules-unread = 구독을 읽지 못했어요.
downloads-lens-following-line = AniBeam이 지켜보는 피드를 채우는 작품별로 보여 줘요.
downloads-none-yet = 아직 없음.
downloads-reading = 읽는 중.
downloads-rules-error = AniBeam이 구독을 읽지 못했어요. 잠시 후 다시 시도하세요.
downloads-removing-to-go = 제거 중, { $count }개 남음
downloads-unsubscribe-all = { $count }개 모두 구독 해지
downloads-unsubscribe = 구독 해지
downloads-no-rules = 구독 없음
downloads-no-rules-body = 위에서 검색하고 구독하면 작품이 계속 들어와요.
downloads-search = 검색
downloads-lens-downloaded = 받은 항목
downloads-dl-line = 디스크에 { $size }. { $rest }
downloads-dl-client-off = { $client }이(가) 연결되지 않았어요.
downloads-dl-unread = 다운로드 목록을 읽지 못했어요.
downloads-dl-in-client =
    { $count ->
       *[other] { $client }에 토렌트 { $count }개, { $size }.
    }
downloads-usage-watched = 시청함
downloads-usage-not-watched = 미시청
downloads-deleting = 삭제 중
downloads-delete-watched = 시청한 항목 삭제
downloads-remove-all = { $count }개 모두 제거
downloads-on-disk = 디스크
downloads-on-disk-line = 라이브러리에 있지만 { $client }에 토렌트가 없는 항목, 큰 순. 오른쪽 클릭으로 삭제하세요.
downloads-nothing-downloaded = 받은 항목 없음
downloads-nothing-downloaded-body = 다운로드하거나 구독한 항목이 작품별로 여기에 표시돼요.
downloads-client-off-title = { $client }이(가) 연결되지 않았어요
downloads-client-off-body = 실행 중인지 확인한 뒤 다시 연결하세요. 주소와 비밀번호는 설정의 다운로드에 있어요.
downloads-connect-again = 다시 연결
downloads-view-grid = 격자
downloads-view-list = 목록
downloads-chip-subscribed = 구독 중
downloads-speed-up = 업 { $speed }/s
downloads-broken-some = 파일 없음 { $count }개
downloads-tag-watched = 시청함

## PlayerDelays.qml

player-delays-title = 지연
player-delays-subtitles = 자막
player-delays-audio = 오디오
player-delays-subtitles-earlier = 자막 0.1초 빠르게
player-delays-subtitles-later = 자막 0.1초 느리게
player-delays-audio-earlier = 오디오 0.1초 빠르게
player-delays-audio-later = 오디오 0.1초 느리게
player-delays-reset = 다음 파일까지 0으로 초기화
player-delays-none = 지연 없음

## TrackPicker.qml

track-picker-off = 끔

## RollControls.qml

roll-controls-window = { $ahead }화 미리 받기: { $total }화 중 { $next }~{ $through }화
roll-controls-window-watched = { $ahead }화 미리 받기: { $total }화 중 { $next }~{ $through }화, { $watched }화 시청
roll-controls-more = { $count }화 더 받기
roll-controls-more-tip = 다음 에피소드를 지금 다운로드해요. 따라잡으면 AniBeam이 다시 다음 몇 화를 미리 받아 둬요.
roll-controls-fewer-tip = 미리 받을 화수 하나 줄이기
roll-controls-ahead = { $count }화 앞
roll-controls-further-tip = 미리 받을 화수 하나 늘리기
roll-controls-all = 모두 받기
roll-controls-all-tip = 미리 받기를 멈추고 남은 에피소드를 모두 다운로드
roll-controls-all-question = 남은 에피소드 { $count }개를 모두 다운로드할까요?

## PlayerChrome.qml

player-chrome-delays = 자막 및 오디오 지연
player-chrome-audio = 오디오 트랙
player-chrome-subtitles = 자막
player-chrome-subtitles-off = 자막 끔
player-chrome-casting-to = { $device }(으)로 캐스트 중
player-chrome-cast = TV로 캐스트
player-chrome-mark-watched = 시청함으로 표시
player-chrome-shortcuts = 키보드 단축키
player-chrome-fullscreen = 전체 화면
player-chrome-exit-fullscreen = 전체 화면 종료
player-chrome-stay = 유지
player-chrome-skip-intro = 오프닝 스킵
player-chrome-skip-outro = 엔딩 스킵
player-chrome-playing-on = { $device }에서 재생 중
player-chrome-replay = 다시 재생

## PlayerPage.qml

player-title = 플레이어
player-next = 다음
player-extra-no-next = 번외에는 다음 에피소드가 없어요
player-last-episode = 마지막 에피소드예요
player-extra-no-previous = 번외에는 이전 에피소드가 없어요
player-first-episode = 첫 에피소드예요
player-resumed-at = { $time }부터 이어 재생
player-start-over = 처음부터
player-audio-track = 오디오: { $track }
player-subtitles-track = 자막: { $track }
player-subtitles-on = 자막: 켬
player-subtitles-off = 자막 끔
player-no-subtitles = 이 파일에는 자막이 없어요
player-delay-seconds = { $value }초
player-audio-delay = 오디오 지연 { $value }
player-subtitle-delay = 자막 지연 { $value }
player-volume = 볼륨 { $volume }
player-speed = 속도 { $speed }×
player-not-while-casting = 캐스트 중에는 불가
player-hold-fast = 누르는 동안 2배속
player-no-chapters = 이 파일에는 챕터가 없어요
player-last-chapter = 마지막 챕터예요
player-chapter = 챕터 { $number }
player-chapter-titled = 챕터 { $number }: { $title }
player-screenshot-saved = Pictures/AniBeam에 저장했어요
player-skipped-intro = 오프닝 건너뜀
player-skipped-outro = 엔딩 건너뜀
player-undo = 되돌리기
player-refusal-hidden = 숨긴 작품이에요
player-refusal-no-match = 아직 매칭되지 않은 작품이에요. 작품 정보에서 매칭하세요
player-refusal-not-newer = 목록이 이미 이 에피소드 이후까지 기록돼 있어요
player-refusal-extra = 번외 에피소드예요
player-refusal-unmatched = 이 파일은 에피소드에 매칭되지 않았어요
player-refusal-on-disk = 디스크에만 있는 파일이에요
player-not-tracked-because = 기록 안 됨: { $reason }.
player-outcome-said = { $tracker }: { $words }
player-outcome-failed = { $tracker } 업데이트 실패
player-outcome-ok = { $tracker } 정상
player-outcome-at = { $tracker } { $progress }화
player-outcome-tracked = 기록됨
player-outcome-not-tracked = 기록 안 됨
player-outcome-could-not-update = 목록을 업데이트하지 못했어요
player-outcome-tracked-list = 기록됨: { $list }
player-outcome-not-tracked-list = 기록 안 됨: { $list }
player-outcome-failed-list = 목록을 업데이트하지 못했어요: { $list }
player-rated = { $score }점 평가함
player-score-failed = 점수를 저장하지 못했어요
player-frame = { $time }의 프레임 { $number }
player-audio-title = 오디오
player-subtitles-title = 자막
player-cast-to = 캐스트 대상
player-cast-looking = Chromecast와 TV 찾는 중
player-cast-unreachable = TV에 연결하지 못했어요
player-cast-tv = TV
player-cast-stop = { $device } 캐스트 중지
player-cast-connecting = 연결 중
player-cast-loading = 로드 중
player-cast-playing = 재생 중
player-cast-paused = 일시정지
player-cast-finished = 종료
player-cast-stopped = 중지됨
player-rate-prompt = 마지막 에피소드를 기록했어요. 이 작품을 평가할까요?
player-rate = 평가
player-not-now = 나중에

## EndpointReposPanel.qml

endpoint-repos-panel-title = 소스 목록
endpoint-repos-panel-helper = 소스 목록은 주소로 추가하는, 서명된 검색 장소 모음이에요. 저장하기 전에 AniBeam이 서명을 확인하고 키를 보여 주며, 그다음 원하는 소스를 설치하면 돼요. 설치한 소스는 위의 검색 소스에 추가돼요.
endpoint-repos-panel-could-not-start = AniBeam이 시작하지 못했어요. 잠시 후 다시 시도하세요.
endpoint-repos-panel-needs-value = { $name }에 값이 필요해요
endpoint-repos-panel-values-of = { $name }의 값
endpoint-repos-panel-secret-note = 이 기기의 비밀 저장소에 보관되고, 라이브러리나 내보내기에는 들어가지 않아요.
endpoint-repos-panel-secret-note-keep = 이 기기의 비밀 저장소에 보관되고, 라이브러리나 내보내기에는 들어가지 않아요. 비워 둔 필드는 저장된 값을 유지해요.
endpoint-repos-panel-asks-again = { $name } 버전 { $version }이(가) 다시 물어봐요
endpoint-repos-panel-update-sends-to = 새 버전은 이 값을 { $host }에 보내요.
endpoint-repos-panel-fields-changed = 필드가 바뀌었어요.
endpoint-repos-panel-needs-fields = 검색하려면 { $fields }이(가) 필요해요.
endpoint-repos-panel-no-longer-offered = 소스 목록에서 더 이상 제공하지 않아요.
endpoint-repos-panel-update-requests-to = 업데이트하면 { $host }에 요청을 보내요.
endpoint-repos-panel-unreadable = 이 목록을 읽지 못했어요. 목록 새로고침으로 다시 시도해요.
endpoint-repos-panel-add-list = 소스 목록 추가
endpoint-repos-panel-add-list-helper = 인덱스 파일의 주소나 이 기기에 있는 파일 경로.
endpoint-repos-panel-checking = 확인 중
endpoint-repos-panel-add = 추가
endpoint-repos-panel-trust-question = { $name }을(를) 신뢰할까요?
endpoint-repos-panel-fingerprint = 키 지문  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    소스 { $count }개, 서명됨. 검색 요청이 어디로 갈지는 소스가 정해요. 신뢰하는 사람의 목록만 믿으세요.
endpoint-repos-panel-preview-trusted =
    { $url }
    소스 { $count }개, 서명됨, 이미 신뢰함. 검색 요청이 어디로 갈지는 소스가 정해요. 신뢰하는 사람의 목록만 믿으세요.
endpoint-repos-panel-trust = 신뢰
endpoint-repos-panel-saving = 저장 중
endpoint-repos-panel-install = 설치
endpoint-repos-panel-update = 업데이트
endpoint-repos-panel-added-by-you = 직접 추가함
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = v{ $version } 업데이트
endpoint-repos-panel-add-value = 값 추가
endpoint-repos-panel-change = { $fields } 변경
endpoint-repos-panel-asks-for = { $fields }이(가) 필요해요.
endpoint-repos-panel-by-hand = 직접 추가
endpoint-repos-panel-by-hand-helper = 어떤 목록에도 없는 사이트용이에요. 검색어 자리에 {"{"}query{"}"}가 있는 피드면 무엇이든 되고, 검색 프록시의 피드도 돼요. 주소에 든 키는 따로 보관되며, 시스템 비밀번호 저장소가 있으면 거기에 저장돼요.
endpoint-repos-panel-name = 이름
endpoint-repos-panel-refreshing = 새로고침 중
endpoint-repos-panel-refresh = 목록 새로고침

## ScheduleSheet.qml

schedule-episodes-range = { $first }~{ $last }화
schedule-day-format = yyyy년 M월 d일 (ddd)
schedule-month-format = yyyy년 M월
schedule-tile-day-format = M월 d일
schedule-tile-month-format = yyyy년 M월
schedule-some-time-in = { $year }년 중
schedule-no-date-yet = 날짜 미정
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = 날짜 없음
schedule-tile-last = 마지막
schedule-tile-out = 공개
schedule-ends-at = { $number }화에서 완결
schedule-dated-by-site = { $count }화는 { $site } 날짜
schedule-dated-by-record = { $count }화는 기록상 날짜
schedule-set-by-you = { $count }화는 직접 지정
schedule-guessed-weekly = { $count }화는 매주로 추정
schedule-nothing-set = 지정한 것 없음
schedule-saved = 저장됨
schedule-cleared = 방영 일정 메모를 지웠어요
schedule-title = 에피소드와 날짜
schedule-season = 시즌 { $number }
schedule-ask-agent = { $agent }에게 묻기
schedule-done-pasting = 붙여넣기 끝
schedule-paste-list = 목록 붙여넣기
schedule-agent-looking = { $agent }이(가) 날짜를 찾고 있어요. 저장을 누르기 전에는 아무것도 저장되지 않아요.
schedule-agent-stopped = { $agent }이(가) 멈췄어요: { $error }
schedule-stop = 중지
schedule-paste-placeholder = 방영 목록, 표, 게시글 붙여넣기
schedule-paste-helper = AniBeam이 읽은 내용은 아래 에피소드에 표시돼요.
schedule-agent-hint = 이 컴퓨터에 Claude Code가 있으면 AniBeam이 대신 날짜를 찾아 달라고 요청할 수 있어요.
schedule-show-more = 에피소드 더 보기
schedule-pick-hint = 에피소드를 클릭해 날짜를 지정하세요. Shift+클릭으로 범위를 선택해요.
schedule-picked-one = { $number }화:
schedule-picked = { $count ->
   *[other] 에피소드 { $count }개:
}
schedule-date-placeholder = 1월 7일, 2027년 3월, 2027년
schedule-set-date = 날짜 지정
schedule-weekly = 매주
schedule-not-last = 마지막 아님
schedule-last-episode = 마지막 화
schedule-clear-date = 날짜 지우기

## ShowPage.qml

show-title = 작품
show-runtime-each = 화당 { $runtime }
show-add-to-list = 목록에 추가
show-moving-to = 이동 중: { $list }
show-moved-to = 이동 완료: { $list }
show-relation-related = 관련
show-no-poster = 포스터 없음
show-reading-anilist = AniList 읽는 중
show-not-in-library = 라이브러리에 없음
show-get-episodes = 에피소드 받기
show-trailer = 예고편
show-your-score = 내 점수 { $score }
show-rate-it = 평가하기
show-anilist-score = AniList { $score }
show-forecast = 나와 { $percent }% 일치
show-because-you-watched = 이 작품을 봤기 때문에
show-hidden-loved = { $count ->
   *[other] 좋아하는 숨은 태그 { $count }개 덕분이에요
}
show-you-gave-it = 내가 준 점수 { $score }
show-story = 줄거리
show-genres-tags = 장르와 태그
show-studios = 스튜디오  { $studios }
show-related = 관련 작품
show-recommendations = 추천
show-from-anilist = AniList 제공
show-reading-rest = AniList에서 나머지 읽는 중

## ConnectSheet.qml

connect-connected-to = { $name } 연결됨
connect-connect-to = { $name } 연결
connect-did-not-finish = 로그인을 마치지 못했어요
connect-finish-in-browser = 브라우저에서 마무리하세요
connect-signed-in-anilist = { $account } 계정으로 로그인했어요. 목록은 시청 중 페이지에 있어요.
connect-signed-in-mal = { $account } 계정으로 로그인했어요. 보는 대로 에피소드가 거기에 기록돼요.
connect-setup-lead = { $name }은(는) 직접 만든 앱을 통해 AniBeam에 로그인해요. 한 번만, 1분 정도 걸려요.
connect-page-open = 브라우저에 { $name } 로그인 페이지가 열렸어요. 거기서 AniBeam을 승인하면 이 창은 저절로 닫혀요.
connect-page-opening = 브라우저에서 { $name } 로그인 페이지를 여는 중이에요.
connect-open-watching = 시청 중 열기
connect-done = 완료
connect-open-again = 로그인 페이지 다시 열기
connect-secret-where = myanimelist.net/apiconfig의 앱 페이지에서 "Client Secret" 아래에 있는 긴 문자열이에요.
connect-client-secret = 클라이언트 시크릿
connect-save-try-again = 저장하고 다시 시도
connect-change-client-id = 클라이언트 ID 변경
connect-step-create-anilist = AniList에서 클라이언트 만들기
connect-step-create-mal = MyAnimeList에서 앱 만들기
connect-step-create-anilist-body = AniList 개발자 설정을 열고 새 클라이언트를 만든 뒤, Redirect URL 칸에 이 주소를 그대로 붙여넣으세요.
connect-step-create-mal-body = MyAnimeList API 페이지를 열고 App Type을 "other"로 해서 ID를 만든 뒤, App Redirect URL 칸에 이 주소를 그대로 붙여넣으세요.
connect-open-developer-page = 개발자 페이지 열기
connect-copy-address = 주소 복사
connect-step-paste-id = 앱 ID 붙여넣기
connect-step-paste-id-anilist = AniList에서는 Client ID라고 해요: 새 클라이언트 페이지에 있는 숫자예요.
connect-step-paste-id-mal = MyAnimeList에서는 Client ID라고 해요: 새 앱 페이지에 있는 긴 문자열이에요.
connect-client-id = 클라이언트 ID
connect-client-secret-web = 클라이언트 시크릿(웹 앱만)
connect-step-connect-body = 브라우저가 열리고 { $name }이(가) 로그인과 AniBeam 승인을 요청하면, AniBeam이 응답을 받아요.

## SettingsPage

settings-title = 설정
settings-search-placeholder = 설정 검색
settings-section-library = 라이브러리
settings-section-accounts = 계정
settings-section-appearance = 모양
settings-section-playback = 재생
settings-section-subtitles = 자막
settings-section-keys = 키보드
settings-section-downloads = 다운로드
settings-section-phone = 휴대전화 앱
settings-section-storage = 저장 공간 및 백업
settings-section-help = 도움말
settings-section-about = 정보
settings-section-developer = 개발자
settings-status-library-empty = 아직 폴더 없음
settings-status-library = 폴더 { $folders }개에 작품 { $count }개
settings-status-library-missing = 폴더 { $gone }개 없음, 폴더 { $folders }개에 작품 { $count }개
settings-status-accounts-anilist = AniList에 동기화 중
settings-status-accounts-mal = MyAnimeList에 동기화 중
settings-status-accounts-both = AniList와 MyAnimeList에 동기화 중
settings-status-accounts-none = 연결된 목록 없음
settings-status-appearance =
    { $mode ->
        [system] 시스템 설정 따름
        [light] 라이트
       *[dark] 다크
    }, { $source ->
        [theme] 테마 색상
       *[desktop] 데스크톱 색상
    }
settings-status-playback =
    { $skip ->
        [both] 오프닝과 엔딩 스킵
        [intro] 오프닝 스킵
        [outro] 엔딩 스킵
       *[none] 모두 끝까지 재생
    }
settings-status-playback-audio =
    { $skip ->
        [both] 오프닝과 엔딩 스킵
        [intro] 오프닝 스킵
        [outro] 엔딩 스킵
       *[none] 모두 끝까지 재생
    }, 오디오 { $languages }
settings-status-subtitles =
    크기 { $percent }%, { $override ->
        [AsScripted] 스타일 파일은 원본대로
        [Force] 모든 파일에 내 스타일
       *[ScaleOnly] 스타일 파일은 모양 유지
    }
settings-status-keys = AniBeam이 반응하는 모든 키
settings-status-downloads-unread = 아직 읽지 않음
settings-status-downloads-answered = qBittorrent 응답함
settings-status-downloads-missing = 검색 도우미 미설치
settings-status-downloads-unconnected = 아직 연결 안 됨
settings-status-phone-shared = tailnet에 공유 중
settings-status-phone-off = 공유 안 함
settings-status-storage = 캐시된 이미지 { $size }
settings-status-storage-unread = 이미지, 내보내기, 가져오기
settings-status-help = 자주 묻는 질문과 답
settings-status-about-update = 버전 { $version } 출시
settings-status-developer = 모든 페이지 시간 측정 중
settings-status-developer-last = { $page } { $ms } ms 만에 생성
settings-results =
    { $count ->
       *[other] 일치하는 설정 { $count }개
    }
settings-results-none = 일치하는 항목 없음
settings-results-hint = Enter를 누르면 첫 번째 항목이 열려요.
settings-results-none-hint = 더 쉬운 단어로 검색해 보세요: 테마, 자막, 트래커, 비밀번호, 폴더.

settings-find-folders =
    .words = 폴더 추가 스캔 다시 스캔 제거 소스 경로 컬렉션
settings-find-matching =
    .words = 메타데이터 작품 정보 제공처 anilist tvmaze 순서 매칭
settings-find-hidden-shows =
    .words = 숨김 숨긴 다시 보이기
settings-find-hidden-airing =
    .words = 관심 없음 방영 다시 보이기
settings-find-calendar =
    .words = 캘린더 달력 ics ical 방영 일정 구독
settings-find-anime-lists =
    .words = anilist myanimelist mal 로그인 연결 계정 연결 해제 트래커
settings-find-main-list =
    .words = 진행 카드 기본 트래커
settings-find-sync-now =
    .words = 새로고침 진행 목록 트래커 동기화
settings-find-on-hold =
    .words = 보류 일시정지 비활성 방치 일수
settings-find-language =
    .words = 언어 번역 로케일 영어 한국어
settings-find-mode =
    .words = 다크 라이트 시스템 테마 야간 모드
settings-find-colours-from =
    .words = 터미널 데스크톱 시스템 테마 소스 kitty 색상
settings-find-dark-theme =
    .words = 색 구성표 base16 다크 테마
settings-find-light-theme =
    .words = 색 구성표 base16 라이트 테마
settings-find-accent =
    .words = 색상 강조색 하이라이트
settings-find-text-size =
    .words = 글꼴 글자 크게 작게 확대 배율 읽기 접근성
settings-find-density =
    .words = 촘촘 여유 간격 크기 밀도
settings-find-poster-size =
    .words = 카드 작게 중간 크게 포스터 크기
settings-find-out-now =
    .words = 방영 지금 공개 카드 목록 행 스타일 배너 포스터
settings-find-out-now-cards =
    .words = 방영 지금 공개 격자 열 행 레이아웃
settings-find-reduce-motion =
    .words = 애니메이션 움직임 줄이기 접근성
settings-find-app-icon =
    .words = 로고 아이콘 런처 독 작업 표시줄 창 다크
settings-find-skip-intros =
    .words = 오프닝 op 자동 스킵 건너뛰기 aniskip 챕터
settings-find-skip-outros =
    .words = 엔딩 ed 크레딧 자동 스킵 건너뛰기 aniskip
settings-find-mpv-settings =
    .words = mpv.conf 설정 고급 mpv
settings-find-discord =
    .words = discord 디스코드 rich presence 상태 프로필
settings-find-next-episode =
    .words = 다음 버튼 제목 자동 재생 끝
settings-find-subtitle-languages =
    .words = 언어 트랙 자막
settings-find-audio-languages =
    .words = 언어 트랙 더빙 일본어 오디오
settings-find-start-over =
    .words = 초기화 기본값 자막 스타일
settings-find-size =
    .words = 크기 배율 자막
settings-find-position =
    .words = 높이 여백 위치 자막
settings-find-styled =
    .words = ass 덮어쓰기 강제 스타일
settings-find-font =
    .words = 글꼴 폰트 서체
settings-find-text-colour =
    .words = 색상 글자 색
settings-find-outline =
    .words = 윤곽선 테두리 외곽선
settings-find-background =
    .words = 상자 배경 불투명도
settings-find-bold =
    .words = 굵게 두께
settings-find-keys-player =
    .words = 키보드 단축키 키
settings-find-keys-player-around =
    .words = 키보드 단축키 전체 화면 다음 이전 에피소드 뒤로
settings-find-keys-around =
    .words = 키보드 단축키 뒤로 앞으로 오른쪽 클릭 메뉴 전체 화면
settings-find-keys-anywhere =
    .words = 키보드 단축키 키 설정 종료
settings-find-torrent-client =
    .words = qbittorrent 비밀번호 사용자 이름 주소 webui 시드박스 토렌트 클라이언트
settings-find-search-helper =
    .words = anirss rss 피드 토큰 명령 서버 고급 검색 도우미
settings-find-next-few =
    .words = 순환 미리 받기 에피소드 앞 삭제 시청 보관 디스크 공간
settings-find-unsubscribe =
    .words = 정리 피드 완결 구독 해지
settings-find-last-search =
    .words = 다운로드 검색어 기억
settings-find-search-sources =
    .words = 엔드포인트 피드 설정 anirss 검색 소스
settings-find-source-lists =
    .words = 저장소 repo 엔드포인트 서명 jackett prowlarr 소스 목록
settings-find-share-tailscale =
    .words = 휴대전화 android tailnet 원격 호스트 tailscale 공유
settings-find-stream =
    .words = 스트리밍 재생 캐스트 에피소드 휴대전화 파일
settings-find-devices =
    .words = 신뢰 신뢰함 끄기 차단 기기 tailscale
settings-find-back-up =
    .words = 내보내기 백업 파일 json
settings-find-restore =
    .words = 가져오기 복원 파일 json
settings-find-image-cache =
    .words = 포스터 지우기 이미지 디스크 캐시
settings-find-files =
    .words = 데이터베이스 데이터 설정 캐시 폴더 경로 파일
settings-find-help-folders =
    .words = 도움말 폴더 구조 이름 규칙 에피소드 시즌 영화 파일
settings-find-help-recognised =
    .words = 도움말 매칭 잘못된 작품 찾을 수 없음 인식 안 됨
settings-find-help-anilist =
    .words = 도움말 anilist myanimelist mal 로그인 연결 트래커
settings-find-help-downloads =
    .words = 도움말 다운로드 비어 있음 소스 없음 토렌트 qbittorrent
settings-find-help-data =
    .words = 도움말 데이터 파일 백업 데이터베이스 개인정보
settings-find-help-else =
    .words = 도움말 설정 온보딩 안내 신고 버그 문제 이슈
settings-find-version =
    .words = 빌드 새로운 기능 변경 내역 버전
settings-find-setup =
    .words = 첫 실행 안내 온보딩 다시 설정
settings-find-updates =
    .words = 안정 버전 nightly 채널 확인 업데이트
settings-find-developer-options =
    .words = 디버그 시간 측정 개발자
settings-find-ratings =
    .words = imdb 크레딧 평점

## SettingsDownloadsTab

settings-downloads-title = 다운로드
settings-downloads-lead = 받은 항목을 AniBeam이 어디로 보내고, 릴리스 피드를 어떻게 검색하는지 정해요.
settings-downloads-keep-none = 에피소드를 다 보면 바로 전 에피소드가 삭제돼요.
settings-downloads-keep-line = 방금 다 본 에피소드 앞에 남겨 둘 시청한 에피소드 수예요. { $count }(으)로 하면 { $finishing }화를 다 볼 때 1화가 삭제돼요.
settings-downloads-anirss-ready = 실행 중, 버전 { $version }, 주소 { $url }.
settings-downloads-anirss-starting = 검색 도우미 시작 중.
settings-downloads-anirss-missing = 검색 도우미가 설치되지 않았어요. AniBeam을 다시 설치하거나 명령에 직접 쓰는 anirss를 지정하세요.
settings-downloads-anirss-failed = 검색 도우미가 멈췄어요. 다시 시도하면 시작해요. 무슨 말을 남겼는지는 세부 정보에 있어요.
settings-downloads-anirss-idle = 아직 시작되지 않았어요. 다운로드 페이지에서 처음 검색할 때 시작해요.
settings-downloads-torrent-trying = { $client } 시도 중.
settings-downloads-torrent-answered = { $client } { $version } 응답함.
settings-downloads-torrent-failed = { $client }에 연결되지 않았어요. 위의 주소와 비밀번호를 확인하고 다시 테스트하세요.
settings-downloads-torrent-untried = 아직 시도하지 않았어요.
settings-downloads-mode-existing = 직접 실행하는 qBittorrent, Web UI가 켜져 있어야 해요.
settings-downloads-mode-managed = AniBeam이 자체 qBittorrent를 실행해 ~/Downloads/Anime에 저장해요. AniBeam을 종료해도 계속 실행되어 구독한 작품을 계속 받아요.
settings-downloads-mode-auto = 아래 주소에서 응답하는 qBittorrent를 쓰고, 없으면 자체 qBittorrent를 실행해요.
settings-downloads-password-saved = 이 클라이언트의 비밀번호가 여기에 저장돼 있어요.
settings-downloads-password-anirss = 검색 도우미가 저장한 비밀번호를 쓰고 있어요. 여기에 저장하면 더 이상 의존하지 않아요.
settings-downloads-password-none = 저장된 비밀번호가 없어요. 이 기기에 대해 로그인을 생략하는 클라이언트라면 필요 없어요.
settings-downloads-mismatch = 검색 도우미는 { $helperUrl }와(과), AniBeam은 { $url }와(과) 통신해요. 직접 실행하는 검색 도우미는 자체 config.toml에서 주소를 읽으므로, 여기서 클라이언트를 바꿔도 따라오지 않아요.
settings-downloads-torrent-title = 토렌트 클라이언트
settings-downloads-torrent-helper = 다운로드를 맡는 qBittorrent예요. 비밀번호는 시스템 비밀번호 저장소에 보관돼요.
settings-downloads-which-client = 클라이언트
settings-downloads-pick-for-me = 자동 선택
settings-downloads-my-own = 내 클라이언트
settings-downloads-built-in = 내장
settings-downloads-saved-clients = 저장된 클라이언트
settings-downloads-saved-clients-helper = 예를 들어 시드박스와 이 기기의 클라이언트. 다운로드 페이지의 클라이언트 칩으로도 전환할 수 있어요.
settings-downloads-add-client = 추가
settings-downloads-name = 이름
settings-downloads-name-placeholder = 주소를 따라 이름 지정
settings-downloads-address = 주소
settings-downloads-address-managed-helper = 이 기기에서만 접근할 수 있고 비밀번호가 필요 없어요.
settings-downloads-address-helper = Web UI 주소.
settings-downloads-username = 사용자 이름
settings-downloads-password = 비밀번호
settings-downloads-secret-saved = 저장됨
settings-downloads-secret-unset = 설정 안 됨
settings-downloads-connection = 연결
settings-downloads-open-config = 설정 파일 열기
settings-downloads-details = 세부 정보
settings-downloads-details-torrent = AniBeam이 시도했을 때 { $client }의 응답이에요.
settings-downloads-test-trying = 시도 중
settings-downloads-test-save = 저장하고 테스트
settings-downloads-test = 테스트
settings-downloads-searching = 검색
settings-downloads-search-helper = 검색 도우미
settings-downloads-searching-helper = AniBeam이 소스를 직접 검색하고 선택한 항목을 토렌트 클라이언트로 보내요. 검색 도우미는 필요 없어요.
settings-downloads-search-helper-helper = AniBeam에서 검색과 구독을 맡는 부분으로, anirss라는 프로그램이에요. AniBeam이 대신 실행하니, 직접 실행하는 경우에만 바꾸세요.
settings-downloads-runs-in = 검색 실행 위치
settings-downloads-runs-in-native-helper = 구독은 여전히 토렌트 클라이언트로 가므로 AniBeam이 닫혀 있어도 새 에피소드가 들어와요.
settings-downloads-runs-in-helper = AniBeam이 실행하거나 직접 실행하는 별도 프로그램인 검색 도우미.
settings-downloads-engine-native = AniBeam 자체
settings-downloads-how-runs = 실행 방식
settings-downloads-how-runs-sidecar = AniBeam이 열릴 때 시작하고 종료할 때 멈춰요.
settings-downloads-how-runs-remote = 이미 실행 중인 것을 아래 주소와 키로 사용해요.
settings-downloads-runs-sidecar = AniBeam이 실행
settings-downloads-runs-remote = 직접 실행
settings-downloads-command = 명령
settings-downloads-command-helper = PATH의 이름이나 전체 경로. ~/.local/bin도 검색해요.
settings-downloads-server-address-helper = /api/v1을 뺀 서버 기본 주소.
settings-downloads-key = 키
settings-downloads-key-saved-helper = 키가 저장돼 있어요. 새 키를 입력하면 바뀌고, 칸을 비우면 제거돼요.
settings-downloads-key-helper = 검색 도우미가 시작할 때 보여 준 키예요. 이 키로 AniBeam이 접속해요.
settings-downloads-details-anirss = 검색 도우미가 멈출 때 남긴 말이에요.
settings-downloads-starting = 시작 중
settings-downloads-reconnect = 다시 연결
settings-downloads-save-first = 먼저 변경 사항을 저장하세요
settings-downloads-roll-title = 다음 몇 화 미리 받기
settings-downloads-roll-helper = 긴 작품은 몇 화씩 다운로드하고, 한 화를 다 볼 때마다 다음 화를 받아요. 전체가 디스크에 다 들어갈 필요가 없어요.
settings-downloads-roll-big = 긴 작품
settings-downloads-roll-big-helper = { $count }화가 넘는 작품은 저절로 몇 화씩 받아요. 다운로드 페이지에서 그 행으로 한꺼번에 받을 수도 있어요.
settings-downloads-roll-ahead = 미리 받을 화수
settings-downloads-roll-ahead-helper = 지금 보는 화 이후로 받아 둘 화수예요. 12화가 대략 한 시즌이에요.
settings-downloads-unit-episodes = 화
settings-downloads-roll-over = 긴 작품 기준
settings-downloads-roll-over-helper = 이보다 에피소드가 많으면 몇 화씩 받아요.
settings-downloads-roll-watched = 시청한 에피소드
settings-downloads-roll-watched-delete-helper = 더 진행하면 삭제돼서 디스크에는 남은 화만 있어요.
settings-downloads-roll-watched-keep-helper = 보고 나도 모든 파일이 디스크에 남아요.
settings-downloads-keep-all = 모두 보관
settings-downloads-keep-behind = 뒤에 남길 화수
settings-downloads-subscriptions-title = 구독과 검색
settings-downloads-last-search = 마지막 검색 유지
settings-downloads-last-search-helper = 다운로드 페이지가 마지막 검색으로 열려요. 끄면 빈 상태로 열려요.
settings-downloads-prune = 완결 작품 구독 해지
settings-downloads-prune-helper = 마지막 화 방영 후 16일이 지나면 피드를 정리해요. 작품과 파일은 남아요.
settings-downloads-sources-title = 검색 소스
settings-downloads-sources-helper = 검색이 가는 곳, 시도하는 순서대로. 찾은 게 없으면 다음 소스로 넘어가요.
settings-downloads-sources-empty = 아직 설정된 소스가 없어요
settings-downloads-sources-connect = 목록을 보려면 검색 도우미를 연결하세요
settings-downloads-sources-empty-helper = 아래에서 소스 목록을 추가하면 그 소스가 여기에 표시돼요.
settings-downloads-config-file = 검색 도우미 설정 파일
settings-downloads-config-native-helper = 저장 폴더, 태그, 최적 일치 규칙이 들어 있고, AniBeam이 도우미를 실행할 때 이를 따라요. 도우미 자체 소스는 도우미만 써요.
settings-downloads-config-helper = 소스, 저장 폴더, 태그, 최적 일치 규칙이 들어 있어요.
settings-downloads-unsaved = 저장 안 된 변경 사항
settings-downloads-unsaved-helper = 저장하면 새 설정으로 검색 도우미가 다시 시작돼요.
settings-downloads-discard = 취소

## SettingsDeveloperTab

settings-developer-title = 개발자
settings-developer-lead = 페이지가 느린 이유를 측정해요. 각 스위치는 다음에 여는 페이지부터 적용돼요.
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds }초
settings-developer-switches = 스위치
settings-developer-load-images = 이미지 불러오기
settings-developer-load-images-helper = 끄면 포스터, 배너, 인물 사진을 읽거나 디코딩하지 않아요.
settings-developer-overlay = 페이지 로드 오버레이
settings-developer-overlay-helper = 마지막 페이지 로드와 메모리를 창 오른쪽 아래에 표시해요.
settings-developer-read-cache = 라이브러리 읽기 캐시
settings-developer-read-cache-helper = 라이브러리 전체 읽기 결과를 라이브러리가 바뀔 때까지 보관해요. 끄면 페이지마다 코어에 다시 요청해요.
settings-developer-poster-pool = 백그라운드에서 포스터 미리 불러오기
settings-developer-poster-pool-helper = 볼 가능성이 높은 포스터 60개를 디코딩해 메모리에 둬요.
settings-developer-options = 개발자 옵션
settings-developer-options-helper = 이 탭, 오버레이, 측정을 끕니다.
settings-developer-memory = 메모리
settings-developer-memory-helper = 커널이 이 프로세스에 대해 집계한 값이에요. 이미지는 디코딩된 픽셀을 메모리와 그래픽 카드에 보관해요.
settings-developer-memory-unknown = 이 시스템은 앱의 메모리 사용량을 알려 주지 않아요.
settings-developer-in-use = 사용 중
settings-developer-peak = 최대
settings-developer-heap = 힙
settings-developer-files-mapped = 매핑된 파일
settings-developer-threads = 스레드
settings-developer-give-back = 메모리 반환
settings-developer-give-back-helper = 씬 그래프가 다시 만들 수 있는 것을 버리고, JavaScript 가비지를 수집하고, 빈 힙을 시스템에 돌려줘요. 해제된 양은 앱이 필요 없이 쥐고 있던 양이에요.
settings-developer-released = { $before } → { $after }
settings-developer-free-now = 지금 해제
settings-developer-images-total = 시작 이후 이미지
settings-developer-images-total-line = 디코딩 { $decodes }회, 디스크 { $disk }에서 픽셀 { $pixels }, { $ms } ms
settings-developer-loads = 페이지 로드
settings-developer-loads-helper = 생성은 클릭 후 기다리는 시간이에요: 페이지가 생길 때까지 Qt 스레드는 다른 일을 하지 않아요. 안정은 마지막 이미지가 도착한 시점이에요.
settings-developer-touring = 측정 중, { $steps }개 중 { $at }번째
settings-developer-tour = 모든 페이지 측정
settings-developer-copy-report = 보고서 복사
settings-developer-col-page = 페이지
settings-developer-col-built = 생성
settings-developer-col-painted = 그리기
settings-developer-col-settled = 안정
settings-developer-col-calls = 호출, 데이터
settings-developer-col-decodes = 디코딩, 시간, 픽셀
settings-developer-col-items = 항목
settings-developer-col-offscreen = 화면 밖 이미지
settings-developer-col-memory = 메모리
settings-developer-load-no-images = { $page }, 이미지 없음
settings-developer-offscreen-of = { $total }개 중 { $offscreen }개
settings-developer-loads-empty = 아무 페이지나 열면 여기에 표시돼요.
settings-developer-load-images-off = { $page }, 이미지 끔
settings-developer-load-frames = 로드하는 동안 프레임 { $frames }개를 그렸고, 두 프레임 사이 최대 간격은 { $gap }이에요.
settings-developer-load-frames-timed-out = 로드하는 동안 프레임 { $frames }개를 그렸고, 두 프레임 사이 최대 간격은 { $gap }이에요. 15초 후 이미지 기다리기를 포기했어요.
settings-developer-built = 생성
settings-developer-settled = 안정
settings-developer-core-data = 코어에서 받은 데이터
settings-developer-pixels = 디코딩한 픽셀
settings-developer-page-items = 페이지의 항목
settings-developer-cached-images = 이미 메모리에 있는 이미지
settings-developer-calls = 코어 호출, 느린 순
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  캐시됨
settings-developer-call = { $name }  { $ms } ms  { $size }  코어 { $core }, QML까지 { $qml }
settings-developer-decodes = 가장 느린 디코딩
settings-developer-decodes-line = 이미지 스레드에서 연달아 디코딩 { $count }회, 처음부터 끝까지 { $span }. 요청 { $requests }, 이미 디코딩됨 { $cached }, 차단됨 { $blocked }
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } → { $w }x{ $h }  { $size }
settings-developer-decodes-none = 디코딩한 것 없음: 모든 이미지가 이미 메모리에 있었어요.
settings-developer-images-were-off = 이미지가 꺼져 있었어요.
settings-developer-scene = 페이지 구성
settings-developer-scene-line = 항목 { $items }개, 창 밖 { $offscreen }개. 텍스트 { $texts }개, 도형 { $shapes }개, 아이콘 { $icons }개. 이미지 { $images }개, 그중 창 밖 { $imagesOffscreen }개가 { $size } 차지
settings-developer-copy-load = 이 로드 복사
settings-developer-tour-title = 모든 페이지, 이미지 켬과 끔
settings-developer-tour-helper = 각 페이지를 세 번 방문한 값의 중간값이에요. 두 열이 같다면 이미지 때문에 페이지가 느린 게 아니에요.
settings-developer-col-built-on = 생성, 이미지 켬
settings-developer-col-built-off = 생성, 이미지 끔
settings-developer-col-settled-on = 안정, 켬
settings-developer-col-settled-off = 안정, 끔
settings-developer-col-decodes-time = 디코딩, 시간
settings-developer-col-data = 데이터
settings-developer-tour-peak = 측정 중 최대 메모리 { $size }
settings-developer-events = 이벤트
settings-developer-events-helper = 코어가 보내는 모든 것, 모든 수준, 최신순. 각 이벤트는 JSON으로 Qt 스레드에 복사되며, 크기는 전송 비용이에요.
settings-developer-event-rate = { $kind } 분당 { $count }회, { $size }
settings-developer-events-filter = 단어, 종류, 단계로 필터
settings-developer-event-plain = { $message }   (일반: { $plain })
settings-developer-copy-events = 이벤트 복사

## SettingsKeysTab

settings-keys-title = 키보드
settings-keys-lead = AniBeam이 반응하는 모든 키예요. 어디서든 ?를 누르면 그 자리에서 쓸 수 있는 키가 보여요.
settings-keys-player = 플레이어에서
settings-keys-player-around = 플레이어 이동
settings-keys-around = 이동
settings-keys-anywhere = 어디서나

## KeyHelp

key-help-title = 키보드 단축키
key-help-group-playback = 재생
key-help-group-navigation = 이동
key-help-group-library = 라이브러리
key-help-key-space = Space / K
key-help-key-left-right = ← / →
key-help-key-ctrl-right = Ctrl+→
key-help-key-backspace = Backspace
key-help-key-up-down = ↑ / ↓
key-help-key-hold = 영상 길게 누르기
key-help-key-page = Page Up / Down
key-help-key-home-end = Home / End
key-help-key-escape = Esc
key-help-key-alt-left = Alt+←
key-help-key-alt-right = Alt+→
key-help-key-shift-f10 = Shift+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / 또는 Ctrl+F
key-help-key-ctrl-space = Ctrl+Space
key-help-key-shift-space = Shift+Space
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = 재생 또는 일시정지
key-help-seek-5 = 5초 이동
key-help-seek-10 = 10초 이동
key-help-skip = 오프닝이나 엔딩 스킵, 없으면 90초
key-help-frame-step = 한 프레임 뒤로 또는 앞으로
key-help-speed = 0.25씩 느리게 또는 빠르게
key-help-normal-speed = 보통 속도
key-help-volume = 볼륨 5씩 올리기 또는 내리기
key-help-mute = 음소거
key-help-subtitles-toggle = 자막 끄기와 다시 켜기
key-help-subtitle-delay = 자막 지연 100ms 빠르게 또는 느리게
key-help-audio-delay = 오디오 지연 100ms 빠르게 또는 느리게
key-help-hold-speed = 누르는 동안 2배속
key-help-screenshot = Pictures에 스크린샷 저장
key-help-episode = 다음 또는 이전 에피소드
key-help-chapter = 이전 또는 다음 챕터
key-help-start-end = 처음, 또는 엔딩
key-help-fullscreen = 전체 화면
key-help-escape-player = 열린 것 닫기, 없으면 전체 화면 종료, 없으면 플레이어 나가기
key-help-toggle-list = 이 목록 보기 또는 숨기기
key-help-library-search = 라이브러리와 검색
key-help-search-page = 이 페이지 검색
key-help-activity = 활동
key-help-settings = 설정
key-help-quit = 종료
key-help-forward = 앞으로
key-help-escape-app = 열린 것 닫기, 없으면 뒤로
key-help-context-menu = 오른쪽 클릭 메뉴
key-help-pick-card = 카드 선택, 여러 개에 한꺼번에 적용
key-help-pick-range = 여기까지 모든 카드 선택
key-help-pick-all = 보이는 모든 카드 선택

## SettingsAboutTab

settings-about-title = 정보
settings-about-lead = 지금 쓰는 AniBeam 버전과 새 버전 소식을 받는 방식이에요.
settings-about-update-failed = 업데이트를 마치지 못했어요. 다시 시도하거나 릴리스 페이지에서 받으세요.
settings-about-update-ready = { $version } 설치 준비 완료.
settings-about-update-command = { $version } 출시. 다음 명령으로 업데이트하세요: { $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = { $version } 출시. 소스로 빌드한 사본이에요: pull 후 다시 빌드하세요.
settings-about-update-elsewhere = { $version } 출시, 이 컴퓨터용 다운로드는 없어요.
settings-about-version = 버전
settings-about-build = AniBeam { $build }
settings-about-whats-new = 새로운 기능
settings-about-install = { $version } 설치
settings-about-setup = 설정 안내
settings-about-setup-helper = 새로 설치하면 처음 나오는 안내예요: 폴더, 목록, 다운로드.
settings-about-run-setup = 설정 안내 다시 하기
settings-about-updates = 업데이트
settings-about-nightly-helper = 거의 매일 새 빌드. 더 새롭지만 덜 테스트됐어요.
settings-about-stable-helper = 테스트를 거쳐 출시된 새 버전.
settings-about-stable = 안정 버전
settings-about-nightly = Nightly
settings-about-check-now = 지금 확인
settings-about-developer = 개발자 옵션
settings-about-developer-helper = 모든 페이지의 시간을 재고, 메모리와 전체 기술 로그를 보여 주고, 이미지를 끌 수 있는 개발자 섹션을 추가해요.
settings-about-credits = 크레딧
settings-about-ratings = 평점
settings-about-ratings-helper = 정보 제공: IMDb (https://www.imdb.com). 허가를 받아 사용해요.
settings-about-series = 작품과 에피소드
settings-about-series-helper = AniList와 TVmaze. 스킵 구간은 AniSkip 제공.

## SettingsPlaybackTab

settings-playback-title = 재생
settings-playback-lead = 플레이어가 알아서 하는 일이에요. 필요할 때만 손대면 돼요.
settings-playback-skipping = 스킵
settings-playback-skipping-helper = 파일 챕터를, 없으면 시청자들이 공유한 스킵 구간을 써요. 플레이어에서 되돌리면 AniBeam을 닫을 때까지 꺼져요.
settings-playback-skip-intros = 오프닝 스킵
settings-playback-skip-outros = 엔딩 스킵
settings-playback-skip-outros-helper = 엔딩에 도달하면 에피소드를 시청함으로 표시해요.
settings-playback-next-episode = 다음 에피소드
settings-playback-name-next = 다음 에피소드 제목 표시
settings-playback-name-next-helper = 에피소드 끝의 다음 버튼에 다음 화 제목이 보여요. 끄면 번호만 보여요.
settings-playback-languages = 언어
settings-playback-languages-helper = 언어 코드, 원하는 순서대로. 파일에 있는 첫 번째 언어가 선택돼요. 플레이어에서 고른 트랙은 그 작품에 기억돼요.
settings-playback-subtitle-languages = 자막 언어
settings-playback-audio-languages = 오디오 언어
settings-playback-discord = 보는 작품을 Discord에 표시
settings-playback-discord-check-failed = 지금은 AniBeam이 Discord를 확인하지 못했어요. 잠시 후 다시 시도하세요.
settings-playback-discord-helper = 재생하는 동안 작품, 에피소드, 포스터가 Discord 프로필에 보여요. 꺼져 있거나 Discord가 닫혀 있으면 아무것도 보내지 않아요.
settings-playback-discord-unavailable = 이 빌드에는 Discord 애플리케이션이 없어요.
settings-playback-discord-connected = Discord에 연결됨
settings-playback-discord-not-connected = 연결 안 됨. 에피소드를 재생하고 Discord가 열려 있으면 AniBeam이 연결해요
settings-playback-advanced = 고급
settings-playback-my-mpv = 내 mpv 설정 사용
settings-playback-my-mpv-helper = mpv.conf를 읽어요. 시작할 때만 동작하는 옵션은 건너뛰고, 스크립트는 불러오지 않아요.
settings-playback-no-mpv-conf = mpv 설정 파일이 없어요. mpv.conf에 옵션을 넣은 뒤 켜세요.

## SliderRow

slider-row-spoken-unit = { $name }, 단위 { $unit }
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = 강조색 빨강
swatches-accent-orange = 강조색 주황
swatches-accent-yellow = 강조색 노랑
swatches-accent-green = 강조색 초록
swatches-accent-cyan = 강조색 청록
swatches-accent-blue = 강조색 파랑

## LookPreview

look-preview-dark = 다크
look-preview-light = 라이트

## SettingsAccountsTab

settings-accounts-title = 계정
settings-accounts-lead = AniList나 MyAnimeList를 연결하면 다 본 에피소드가 자동으로 기록돼요. 기록은 늘기만 하고 줄지 않아요.
settings-accounts-syncing = 목록 동기화 중
settings-accounts-synced = 목록 동기화됨
settings-accounts-lists = 애니 목록
settings-accounts-lists-helper = 없어도 AniBeam은 이 컴퓨터에서 본 것을 기억해요. AniList나 MyAnimeList 목록이 있으면 휴대전화와 프로필로도 이어져요.
settings-accounts-progress = 진행 상황
settings-accounts-main-list = 기본 목록
settings-accounts-main-list-helper = 카드에는 이 목록의 진행 상황이 보여요. 다른 목록도 모든 업데이트를 받아요.
settings-accounts-local-lists = 이 컴퓨터의 목록
settings-accounts-local-lists-helper = 작품 { $count }개의 상태나 점수가 여기에만 있어요. { $tracker }(으)로 옮기면 휴대전화와 프로필에서도 볼 수 있어요.
settings-accounts-review-move = 확인하고 옮기기
settings-accounts-nothing-to-move = 옮길 항목 없음
settings-accounts-sync-now = 지금 동기화
settings-accounts-sync-now-helper = 연결된 모든 목록을 다시 읽어요. AniBeam이 시작할 때도 해요.
settings-accounts-sync = 동기화
settings-accounts-move-on-hold = 보류로 옮기기
settings-accounts-move-on-hold-helper = 한동안 손대지 않은 시청 중 작품을 목록에서 보류로 옮겨요. 재생하면 다시 돌아와요. 다음 에피소드를 기다리는 작품은 그대로 둬요.
settings-accounts-after = 기간
settings-accounts-after-line = 에피소드를 재생하지 않은 일수
settings-accounts-days-unit = 일

## SettingsPhoneTab

settings-phone-title = 휴대전화 앱
settings-phone-lead = 휴대전화의 AniBeam으로 어디서든 이 라이브러리를 재생하고, 방영 중인 작품을 보고, 다운로드를 확인할 수 있어요.
settings-phone-online = 온라인
settings-phone-offline = 오프라인
settings-phone-untrusted = 이 컴퓨터에서 받는 것 없음
settings-phone-sharing = 공유
settings-phone-sharing-helper = Tailscale에 있는 내 기기만 접근할 수 있어요. 휴대전화에서 본 에피소드의 위치와 진행 상황이 여기에도 유지돼요.
settings-phone-share = Tailscale로 공유
settings-phone-stream = 에피소드 스트리밍
settings-phone-stream-on = 내 기기에서 이 컴퓨터의 에피소드를 재생하고 캐스트할 수 있어요.
settings-phone-stream-off = 내 기기에서 라이브러리와 방영 중인 작품은 볼 수 있지만 여기서 재생할 수는 없어요.
settings-phone-host-service = 호스트 서비스
settings-phone-not-running = 실행 안 됨
settings-phone-running = 실행 중
settings-phone-service-off-helper = 명령을 한 번 실행하면 로그인할 때부터 창을 열지 않아도 내 기기가 AniBeam에 접근해요.
settings-phone-service-on-helper = 창을 열지 않아도 내 기기가 AniBeam에 접근해요. 이 창이 열려 있는 동안은 서비스 대신 창이 공유해요.
settings-phone-copy-command = 명령 복사
settings-phone-command-copied = 명령 복사됨
settings-phone-nobody = 받는 것 없음: Tailscale에 소유자 이름이 없어요
settings-phone-people = 사람
settings-phone-own-profile = 이 컴퓨터의 프로필
settings-phone-person-own-devices =
    { $count ->
        [0] 이 컴퓨터
       *[other] 이 컴퓨터와 기기 { $count }대
    }
settings-phone-person-devices =
    { $count ->
       *[other] 기기 { $count }대
    }
settings-phone-devices = tailnet의 기기
settings-phone-devices-helper = 공유가 켜져 있으면 tailnet의 모든 기기가 이 컴퓨터에 접근할 수 있어요. 기기를 끄면 여기서 아무것도 받지 않아요.
settings-phone-addresses = 주소
settings-phone-addresses-helper = 휴대전화 앱이 처음 물어볼 때 이 중 하나를 입력하세요.
settings-phone-copy = 복사
settings-phone-address-copied = 주소 복사됨

## SettingsHelpTab

settings-help-title = 도움말
settings-help-lead = 자주 묻는 질문에 대한 짧은 답과 바로 해결하는 방법이에요.
settings-help-folders = 폴더는 어떻게 구성해야 하나요?
settings-help-folders-helper = 작품마다 폴더 하나, 그 안에 에피소드. 각 파일 이름에는 릴리스처럼 에피소드 번호를 넣으세요: "Frieren - 01.mkv" 또는 "Frieren S01E01.mkv". 2기는 별도 폴더로 두거나 작품 폴더 안에 "Season 2" 폴더로 둬도 돼요. 영화는 Movies라는 폴더에 파일 하나씩 넣으세요.
settings-help-for-example = 예시
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = 아직 추가한 폴더가 없어요
settings-help-reads-folders =
    { $count ->
       *[other] AniBeam이 폴더 { $count }개를 읽어요
    }
settings-help-add-folder-helper = 작품 하나하나가 아니라 작품들이 들어 있는 폴더를 추가하세요.
settings-help-add-folder = 폴더 추가
settings-help-open-folders = 폴더 열기
settings-help-match = 작품이 왜 인식되지 않았나요?
settings-help-match-helper = AniBeam은 폴더 이름으로 먼저 AniList, 다음으로 TVmaze에서 작품을 찾아요. 단어가 더 붙었거나, 오타가 있거나, 다른 언어로 된 이름이면 놓칠 수 있어요. 작품 정보에서 직접 골라 주세요. 직접 고른 것은 절대 바뀌지 않아요.
settings-help-all-recognised = 모든 작품이 인식됐어요
settings-help-unrecognised =
    { $count ->
       *[other] 작품 { $count }개가 인식되지 않았어요
    }
settings-help-open-show-info = 작품 정보 열기
settings-help-connect = AniList는 어떻게 연결하나요?
settings-help-connect-helper = 연결하면 다 본 에피소드가 목록에 기록돼서 휴대전화와 프로필도 최신 상태를 유지해요. AniList는 처음 한 번 작은 앱을 직접 만들어야 해요. AniBeam이 안내하고 1분 정도 걸려요. MyAnimeList도 같은 방식이에요.
settings-help-both-connected = 둘 다 연결됨
settings-help-anilist-connected = AniList 연결됨
settings-help-mal-connected = MyAnimeList 연결됨
settings-help-none-connected = 아직 둘 다 연결 안 됨
settings-help-connect-anilist = AniList 연결
settings-help-connect-mal = MyAnimeList 연결
settings-help-downloads = 다운로드가 왜 비어 있나요?
settings-help-downloads-helper = AniBeam에는 검색할 곳이 들어 있지 않아요. 설정의 다운로드에서 소스 목록을 추가하고 원하는 소스를 설치하세요. 다운로드하려면 토렌트 클라이언트도 필요해요: AniBeam이 자체 클라이언트를 실행하거나, 이미 쓰는 qBittorrent를 사용할 수 있어요.
settings-help-sources = 소스와 토렌트 클라이언트
settings-help-open-downloads = 다운로드 설정 열기
settings-help-data = 내 데이터는 어디에 있나요?
settings-help-data-helper = 라이브러리, 시청 기록, 설정은 이 컴퓨터에 남아요. AniList와 MyAnimeList 로그인 정보는 시스템 비밀번호 저장소에, 없으면 나만 읽을 수 있는 파일에 저장돼요. 저장 공간 및 백업에서 모두 파일 하나로 백업할 수 있어요.
settings-help-library-history = 라이브러리와 기록
settings-help-show-folder = 폴더 보기
settings-help-backup = 백업 또는 복원
settings-help-open-storage = 저장 공간 및 백업 열기
settings-help-more = 다른 문제가 있나요?
settings-help-more-helper = 처음 설정을 다시 하거나, 무엇이 잘못됐는지 알려 주세요. 스크린샷이 큰 도움이 돼요.
settings-help-first-setup = 처음 설정
settings-help-first-setup-helper = 폴더, 목록, 다운로드를 한 단계씩.
settings-help-run-setup = 설정 다시 하기
settings-help-report = 문제 신고
settings-help-report-helper = 이 버전 정보가 채워진 GitHub 버그 신고를 브라우저에서 열어요.

## SettingsSubtitlesTab

settings-subtitles-title = 자막
settings-subtitles-lead = 모든 에피소드가 이 설정으로 시작해요. 바꾸는 대로 아래 화면에 보여요.
settings-subtitles-reset-done = 자막이 처음 상태로 돌아갔어요
settings-subtitles-size-place = 크기와 위치
settings-subtitles-size = 크기
settings-subtitles-position = 위치
settings-subtitles-position-helper = 100은 아래쪽 가장자리예요. 숫자가 작을수록 자막이 화면 위로 올라가요.
settings-subtitles-style = 스타일
settings-subtitles-styled = 스타일 자막
settings-subtitles-as-made-helper = 스타일(ASS) 파일은 제작자가 정한 모양 그대로 보여요.
settings-subtitles-mine-helper = 아래 내 스타일이 모든 파일에 적용돼요. 사인과 가라오케가 깨질 수 있어요.
settings-subtitles-size-only-helper = 스타일 자막 파일은 모양을 유지하고 내 크기만 따라요. 일반 파일은 내 스타일을 따라요.
settings-subtitles-as-made = 원본
settings-subtitles-size-only = 크기만
settings-subtitles-mine = 내 설정
settings-subtitles-font = 글꼴
settings-subtitles-font-invalid = Noto Sans처럼 글꼴 이름을 입력하세요.
settings-subtitles-bold = 굵게
settings-subtitles-text-colour = 글자 색
settings-subtitles-colour-invalid = #ffcc00 같은 색을 입력하세요.
settings-subtitles-outline = 윤곽선
settings-subtitles-outline-invalid = 너비는 0 이상의 숫자예요.
settings-subtitles-outline-colour-invalid = #000000처럼 색을 입력하세요.
settings-subtitles-outline-helper = 너비, 그다음 색.
settings-subtitles-shadow = 그림자
settings-subtitles-shadow-invalid = 0 이상의 숫자예요. 0은 그림자 없음.
settings-subtitles-shadow-helper = 그림자가 떨어지는 거리, 0이면 없음.
settings-subtitles-box = 배경 상자
settings-subtitles-box-helper = 자막 뒤에 깔리는 어두운 상자예요. 0이면 없애요.
settings-subtitles-start-over = 처음 상태로
settings-subtitles-start-over-helper = 크기, 글꼴, 색상, 윤곽선, 상자, 위치를 기본값으로 되돌려요.
settings-subtitles-reset = 초기화

## SettingsDataTab

settings-data-title = 저장 공간 및 백업
settings-data-lead = 라이브러리를 다른 컴퓨터로 옮기거나, 사본을 안전하게 보관하거나, 이미지가 차지하는 공간을 비워요.
settings-data-backed-up = { $path }에 백업했어요
settings-data-restore-finished = 복원 완료
settings-data-images-removed =
    { $count ->
       *[other] 이미지 { $count }개 삭제됨
    }
settings-data-summary = { $parts }.
settings-data-summary-folders =
    { $count ->
       *[other] 폴더 { $count }개 추가
    }
settings-data-summary-series = 작품 { $count }개 생성
settings-data-summary-matches =
    { $count ->
       *[other] 매칭 { $count }개 적용
    }
settings-data-summary-views = 시청 기록 { $views }개와 완료한 에피소드 { $episodes }개 병합
settings-data-summary-resume-points =
    { $count ->
       *[other] 이어보기 지점 { $count }개 병합
    }
settings-data-summary-accounts =
    { $count ->
       *[other] 목록 계정 { $count }개 가져옴
    }
settings-data-summary-fixes =
    { $count ->
       *[other] 파일 { $count }개에 직접 만든 수정이 적용돼요.
    }
settings-data-summary-unavailable =
    { $count ->
       *[other] 폴더 { $count }개를 이 컴퓨터에서 찾지 못했어요.
    }
settings-data-summary-left-out = 제외됨: { $fields }.
settings-data-backup = 백업
settings-data-back-up-library = 라이브러리 백업
settings-data-back-up-helper = 폴더, 매칭, 시청 기록을 담은 파일 하나예요.
settings-data-last-written = 마지막 저장 위치: { $path }
settings-data-open-folder = 폴더 열기
settings-data-back-up = 백업
settings-data-private = 비공개 데이터 포함
settings-data-private-helper = AniList와 MyAnimeList 로그인 정보, 시청 기록, 환경설정을 추가해요. 로그인 정보는 일반 텍스트로 저장되니 파일을 다른 사람에게 보이지 마세요.
settings-data-restore-from = 백업에서 복원
settings-data-restore-helper = 백업을 이 라이브러리로 읽어 와요. 디스크에 있는 내용이 우선이고, 파일은 나머지를 채워요.
settings-data-restore = 복원
settings-data-image-cache = 이미지 캐시
settings-data-image-cache-helper = 포스터, 배너, 인물 사진을 디스크에 보관해서 오프라인에서도 라이브러리가 동작해요. 지운 이미지는 페이지에서 필요할 때 다시 받아 와요.
settings-data-cache-unmeasured = 이미지 캐시 크기를 측정하지 못했어요
settings-data-cache-size =
    { $count ->
       *[other] 이미지 { $count }개, { $size }
    }
settings-data-cache-empty = 캐시된 이미지 없음
settings-data-cache-unmeasured-helper = 잠시 후 다시 시도하세요. 이미지는 문제없어요.
settings-data-remove-images =
    { $count ->
       *[other] 이미지 { $count }개를 삭제할까요?
    }
settings-data-files = AniBeam 파일 위치
settings-data-files-database = 라이브러리 데이터베이스
settings-data-files-data = 데이터
settings-data-files-settings = 설정
settings-data-files-cache = 캐시
settings-data-backup-filter = AniBeam 백업 (*.json)
settings-data-all-files-filter = 모든 파일 (*)
settings-data-backing-up = 백업 중
settings-data-restoring = 복원 중

## SettingsLibraryTab

settings-library-title = 라이브러리
settings-library-lead = AniBeam이 지켜보는 폴더와, 그 안에 무엇이 있는지 알아내는 방식이에요.
settings-library-not-scanned = 아직 스캔 안 됨
settings-library-scan-nothing-changed = 변경 없음
settings-library-scan-added = { $count }개 추가
settings-library-scan-changed = { $count }개 변경
settings-library-scan-removed = { $count }개 제거
settings-library-scan-in-folder = { $folder }: { $what }
settings-library-anilist-about = 애니: 시리즈, 영화, 그리고 작품 간 관계. 본 작품이 트래커에 기록돼요.
settings-library-tvmaze-about = 그 밖의 TV: 실사, 카툰, 다큐멘터리. 트래커에 기록되지 않아요.
settings-library-tried-first = 1순위
settings-library-tried-second = 2순위
settings-library-tried-third = 3순위
settings-library-tried-fourth = 4순위
settings-library-tried-nth = { $number }순위
settings-library-folders = 폴더
settings-library-folders-helper = 이 폴더 안의 각 폴더가 작품이 돼요. Movies 폴더 바로 아래 파일은 영화예요.
settings-library-rescan-started = 다시 스캔 시작됨
settings-library-load-failed = 폴더를 읽지 못했어요
settings-library-load-failed-helper = AniBeam이 폴더 목록을 읽지 못했어요. 잠시 후 다시 시도하세요.
settings-library-no-folders = 아직 폴더 없음
settings-library-no-folders-helper = 애니가 있는 폴더를 AniBeam에 알려 주세요. 작품과 영화는 알아서 찾아요.
settings-library-scanned = { $when } 스캔함
settings-library-add-a-folder = 폴더 추가
settings-library-scan-all = 모두 스캔
settings-library-scan-started = 스캔 시작됨
settings-library-add-folder = 폴더 추가
settings-library-matching = 매칭
settings-library-matching-helper = 새 폴더는 이 순서로 조회하고 처음 맞는 결과를 써요. 직접 지정한 매칭은 절대 바뀌지 않아요.
settings-library-try-later = 나중에 시도
settings-library-try-first = 먼저 시도
settings-library-check-matches = 매칭 확인
settings-library-check-matches-helper = 작품 정보 페이지에서 AniBeam이 찾은 정보와 함께 모든 작품을 보고, 잘못된 것을 고칠 수 있어요.
settings-library-open-show-info = 작품 정보 열기
settings-library-hidden-shows = 숨긴 작품
settings-library-show-hidden = 숨긴 작품 보기
settings-library-show-hidden-helper = AniBeam을 닫을 때까지 숨긴 작품을 모든 페이지에 다시 보여 줘요.
settings-library-hidden-from-airing = 방영 중에서 숨긴 작품
settings-library-hidden-from-airing-helper = 관심 없음으로 표시한 작품이에요. 되돌리기 전까지 방영 중에 나오지 않아요.
settings-library-hidden-from-airing-empty = 아직 없어요. 방영 중에서 관심 없음으로 표시한 작품이 여기 모이고, 언제든 되돌릴 수 있어요.
settings-library-show-again = 다시 보이기
settings-library-calendar = 캘린더
settings-library-calendar-feed = 캘린더 속 방영 시간
settings-library-calendar-feed-helper = 목록에 있는 작품의 다가오는 에피소드를 캘린더 파일로 만들어 두고, 일정이 바뀌면 업데이트해요.
settings-library-calendar-file = 캘린더 파일
settings-library-calendar-file-helper = 캘린더 앱에서 이 파일을 구독하세요.
settings-library-copy = 복사
settings-library-path-copied = 경로 복사됨
settings-library-open-folder = 폴더 열기
settings-library-notifications = 알림
settings-library-new-episodes = 새 에피소드
settings-library-new-episodes-helper = 시청 중 목록에 있는 작품의 에피소드가 방영되면 데스크톱 알림을 보내요. 클릭하면 작품이 열려요.
settings-library-folder-added = 폴더 추가됨, 스캔 중

## SettingsAppearanceTab

settings-appearance-language = 언어
settings-appearance-language-system = 시스템과 동일({ $language })
settings-appearance-text-smaller = 작게
settings-appearance-text-normal = 보통
settings-appearance-text-large = 크게
settings-appearance-text-larger = 더 크게
settings-appearance-text-largest = 최대
settings-appearance-title = 모양
settings-appearance-lead = 변경 사항이 여기와 모든 페이지에 바로 적용돼요. ~/.config/anibeam/theme.toml에 저장되며 직접 편집할 수도 있어요.
settings-appearance-colours = 색상
settings-appearance-mode = 모드
settings-appearance-mode-dark = 다크
settings-appearance-mode-light = 라이트
settings-appearance-mode-system = 시스템
settings-appearance-colours-from = 색상 출처
settings-appearance-from-theme-helper = 아래 테마 한 쌍, 다크용과 라이트용.
settings-appearance-from-desktop-helper = 터미널 색상, 없으면 데스크톱의 색 구성표와 강조색.
settings-appearance-from-desktop = 데스크톱
settings-appearance-from-theme = 테마
settings-appearance-dark-theme = 다크 테마
settings-appearance-light-theme = 라이트 테마
settings-appearance-themes-helper = ~/.config/anibeam/themes에 넣은 테마 파일이 두 목록에 나타나요.
settings-appearance-accent = 강조색
settings-appearance-accent-helper = 선택됨, 재생 중, 새 항목을 표시해요.
settings-appearance-icon = 아이콘
settings-appearance-app-icon = 앱 아이콘
settings-appearance-app-icon-helper = 창, 작업 표시줄, 런처의 아이콘이에요. 다크는 노랑과 검정을 바꿔요.
settings-appearance-icon-default = 기본
settings-appearance-icon-default-name = 기본 앱 아이콘
settings-appearance-icon-dark = 다크
settings-appearance-icon-dark-name = 다크 앱 아이콘
settings-appearance-size-motion = 크기와 움직임
settings-appearance-text-size = 글자 크기
settings-appearance-text-size-helper = AniBeam 전체의 글자 크기예요. 시스템의 글자 크기 설정에 더해져요.
settings-appearance-density = 밀도
settings-appearance-density-helper = 요소 사이 여백, 촘촘하게부터 여유 있게까지.
settings-appearance-density-compact = 촘촘
settings-appearance-density-normal = 보통
settings-appearance-density-comfortable = 여유
settings-appearance-poster-size = 포스터 크기
settings-appearance-poster-size-helper = 라이브러리와 홈에서 작품이 보이는 크기예요.
settings-appearance-poster-small = 작게
settings-appearance-poster-medium = 중간
settings-appearance-poster-large = 크게
settings-appearance-out-now = 방영 중의 지금 공개
settings-appearance-out-now-helper = 작품마다 배너가 있는 카드, 또는 작은 포스터와 제목 목록을 세 줄로.
settings-appearance-out-now-cards = 카드
settings-appearance-out-now-list = 목록
settings-appearance-out-now-grid = 지금 공개 카드
settings-appearance-out-now-grid-helper = 전체를 열기 전에 보이는 카드 수, 열 × 행.
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = 움직임 줄이기
settings-appearance-reduce-motion-helper = 라이브러리의 시작 애니메이션과 보여 주기용 움직임을 생략해요.

## LookPane

look-pane-sample-series = 샘플 작품
look-pane-episode-code = { $number }화
look-pane-library = 라이브러리
look-pane-series-count =
    { $count ->
       *[other] 작품 { $count }개
    }
look-pane-search = 제목 또는 폴더 검색
look-pane-search-hint = / 또는 Ctrl+K
look-pane-all = 전체
look-pane-series = 시리즈
look-pane-movies = 영화
look-pane-plain = 기본
look-pane-selected = 선택됨
look-pane-errors =
    { $count ->
       *[other] 오류 { $count }개
    }
look-pane-type-large = 크게
look-pane-type-normal = 보통
look-pane-type-small = 작고 흐리게
look-pane-sample-switch = 샘플 스위치
look-pane-button = 버튼
look-pane-scan = 스캔
look-pane-sample-status =
    { $count ->
       *[other] { $title }에 새 에피소드 { $count }개
    }

## FeedPage

feed-title = 피드
feed-aired = { $episode } { $when } 방영
feed-downloaded = { $when } 파일 다운로드
feed-countdown-days = { $days }일 { $hours }시간
feed-countdown-hours = { $hours }시간 { $minutes }분
feed-countdown-minutes = { $minutes }분
feed-section-this-week = 이번 주
feed-new-episodes =
    { $count ->
       *[other] 새 에피소드 { $count }개
    }
feed-section-earlier = 이전
feed-latest-out = 최근 공개 { $when }
feed-section-coming-soon = 공개 예정
feed-this-week-count = 이번 주 { $count }개
feed-section-everything-else = 그 외
feed-out-this-week = 이번 주 공개 { $count }개
feed-nothing-scheduled = 예정된 항목이 아직 없어요.
feed-next = 다음: { $title }, { $countdown } 후
feed-nothing-new = 이번 주에 새 항목이 없어요.
feed-new-this-week =
    { $count ->
       *[other] 이번 주 새 에피소드 { $count }개.
    }
feed-sort-recent = 최근 공개
feed-sort-upcoming = 공개 예정
feed-failed-title = 피드를 불러오지 못했어요
feed-empty-title = 아직 아무것도 없어요
feed-failed-body = 지금은 AniBeam이 피드를 읽지 못했어요. 잠시 후 다시 시도하세요.
feed-empty-body-folders = 에피소드가 방영되고 들어오는 대로 피드가 채워져요. 방영 중에서 이번 시즌 작품을 볼 수 있어요.
feed-empty-body = 애니가 있는 폴더를 추가하면 피드가 에피소드로 채워져요.
feed-add-folder = 폴더 추가
feed-airing-now = 방영 중

## RecommendedPage

recommended-title = 추천 작품
recommended-ranked-minutes = { $minutes }분 전 순위 매김
recommended-ranked-hours = { $hours }시간 전 순위 매김
recommended-ranked-days = { $days }일 전 순위 매김
recommended-scope-name = 추천 범위
recommended-scope-all = 전체
recommended-scope-library = 라이브러리
recommended-scope-list = 내 목록
recommended-ranking = 순위 매기는 중
recommended-rank-again = 다시 순위 매기기
recommended-no-scores-title = 먼저 작품 몇 개를 평가하세요
recommended-no-list-title = 아직 배울 데이터가 없어요
recommended-never-fitted-title = 작품 순위 매기는 중
recommended-nothing-left-title = 남은 게 없어요
recommended-no-scores-body = 추천은 목록의 점수에서 배워요. AniList에서 작품 몇 개에 점수를 주면 여기에 나타나요.
recommended-no-list-body = 설정에서 AniList를 연결하거나 이 기기에 목록을 두면, 평가한 작품으로 이 페이지가 배워요.
recommended-never-fitted-body = 첫 순위 매기기는 AniList를 1~2분 동안 읽어요. 백그라운드에서 실행돼요.
recommended-nothing-left-library-body = 라이브러리에서 맞는 작품은 모두 봤어요. 전체에서 아직 없는 작품을 찾아보세요.
recommended-nothing-left-body = 목록에서 맞는 작품은 모두 봤어요. 전체에서 목록에 없는 작품을 찾아보세요.

## MoveListsSheet

move-lists-moving =
    { $count ->
       *[other] { $site }(으)로 작품 { $count }개 옮기는 중
    }
move-lists-score = 점수 { $score }
move-lists-title = 목록을 { $site }(으)로 옮기기
move-lists-body = 작품 { $count }개의 상태나 점수가 이 컴퓨터에만 있어요. 옮기면 { $site } 목록에 올라가서 휴대전화와 프로필에서도 보여요. 이 작품들은 { $site }에 있던 내용이 바뀌어요. 이동을 누르기 전에는 아무것도 전송되지 않아요.
move-lists-head-show = 작품
move-lists-head-here = 이 컴퓨터
move-lists-head-tracker = 지금 { $site }
move-lists-nothing = 없음
move-lists-not-on-list = 내 목록에 없음
move-lists-later-hint = 나중에: 설정, 계정
move-lists-not-now = 나중에
move-lists-move =
    { $count ->
       *[other] 작품 { $count }개 이동
    }

## StatsPage

stats-title = 통계
stats-month-jan = 1월
stats-month-feb = 2월
stats-month-mar = 3월
stats-month-apr = 4월
stats-month-may = 5월
stats-month-jun = 6월
stats-month-jul = 7월
stats-month-aug = 8월
stats-month-sep = 9월
stats-month-oct = 10월
stats-month-nov = 11월
stats-month-dec = 12월
stats-nothing-yet = 아직 없음
stats-ranked-episodes =
    { $count ->
       *[other] { $count }화
    }
stats-year-summary = { $year }년에 본 작품
stats-empty-title = 아직 본 작품이 없어요
stats-empty-body = 에피소드를 끝까지 보면 여기에 집계돼요: 시청 시간, 장르, 시청한 시즌.
stats-open-library = 라이브러리
stats-hours-watched = 시청 시간
stats-episodes-label =
    { $count ->
       *[other] 에피소드
    }
stats-series-finished = 완주한 작품
stats-none = 없음
stats-busiest-month = 가장 많이 본 달
stats-by-month = 월별
stats-month-hours = { $hours }시간
stats-top-genres = 상위 장르
stats-top-studios = 상위 스튜디오
stats-seasons-watched = 시청한 시즌
stats-recap-title = 나의 애니 한 해
stats-recap-hours = 시간
stats-recap-episodes = 에피소드
stats-recap-top-genre = 최다 장르
stats-save-image = 이미지로 저장
stats-save-dialog-title = 요약 저장
stats-save-filter-png = PNG 이미지 (*.png)
stats-recap-saved = 요약을 저장했어요
stats-recap-save-failed = 요약을 저장하지 못했어요

## TrackerRow

tracker-row-connected = { $user } 계정으로 연결됨, { $when } 동기화
tracker-row-unknown-user = ?
tracker-row-never = 없음
tracker-row-waiting = 브라우저에서 승인하기를 기다리는 중
tracker-row-not-connected = 연결 안 됨
tracker-row-open-again = 다시 열기
tracker-row-open-again-tip = 로그인 페이지 다시 열기
tracker-row-disconnect = 연결 해제
tracker-row-disconnect-question = { $name } 연결을 해제할까요? 로그인 정보가 삭제돼요.
tracker-row-secret-saved = 이 앱의 클라이언트 시크릿이 저장돼 있어요.
tracker-row-remove-secret = 삭제
tracker-row-secret-removed = 클라이언트 시크릿 삭제됨
tracker-row-add-secret = 클라이언트 시크릿 추가
tracker-row-secret-where = myanimelist.net/apiconfig의 앱 페이지에서 "Client Secret" 아래에 있는 긴 문자열이에요.
tracker-row-secret-placeholder = 클라이언트 시크릿
tracker-row-save-connect = 저장하고 연결

## RemoveSheet

remove-unsubscribe-delete = 구독 해지하고 삭제
remove-unsubscribe-torrents = 구독 해지하고 토렌트 제거
remove-unsubscribe = 구독 해지
remove-delete-files = 제거하고 파일 삭제
remove-torrents =
    { $count ->
       *[other] 토렌트 제거
    }
remove-also-torrents =
    { $count ->
       *[other] 토렌트 { $count }개도 제거
    }
remove-torrents-go-with-files = 파일을 삭제하면 토렌트도 함께 제거돼요.
remove-torrents-files-stay = 클라이언트에서 빠지고, 파일은 남아요.
remove-also-files = 파일도 삭제
remove-files-default-where = 토렌트가 받은 파일
remove-files-detail-rule = { $where }을(를) 휴지통으로 옮기고 토렌트를 제거해요.
remove-files-detail = { $where }을(를) 휴지통으로 옮겨요.

## NotesSheet

notes-your-tracker = 내 트래커
notes-title = 메모
notes-placeholder = 어디까지 봤는지, 누가 추천했는지, 어떤 릴리스를 받을지
notes-over-limit = 한도를 { $count }자 초과
notes-saved-to = { $sites }에 저장했어요

## RecommendationCard

recommendation-card-available = 보유
recommendation-card-open-library = 라이브러리에서 { $title } 열기
recommendation-card-open-anilist = AniList에서 { $title } 열기

## SearchField

search-field-placeholder = 제목 또는 폴더 검색
search-field-hint = / 또는 Ctrl+K
search-field-clear = 검색 지우기

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent }%
recommended-hover-hidden-tags =
    { $count ->
       *[other] 좋아하는 숨은 태그 { $count }개
    }
recommended-hover-because = 이 작품을 봤기 때문에

## PersonCard

person-card-unknown = 알 수 없음
person-card-role-main = 주연
person-card-role-supporting = 조연
person-card-role-background = 단역
person-card-the-character = 캐릭터
person-card-open = AniList에서 { $name } 열기

## StatusDot

status-dot-on-list = 내 목록: { $status }

## TrackerNudge

tracker-nudge-title = 애니 목록 연결
tracker-nudge-body = 보는 대로 AniBeam이 AniList나 MyAnimeList에 에피소드를 기록해요.
tracker-nudge-connect-anilist = AniList 연결
tracker-nudge-connect-mal = MyAnimeList 연결
tracker-nudge-not-now = 나중에

## RecommendedCard

recommended-card-spoken = { $title }, 나와 { $percent }% 일치
recommended-card-forecast = { $percent }%
recommended-card-in-library = 라이브러리에 있음

## InlineConfirm

inline-confirm-keep = 유지

## Tokens

tokens-cube-watched-tracked = 시청함, 온라인에 기록됨
    .inline = 시청함, 온라인에 기록됨
tokens-cube-watched-here = 여기서 시청함, 아직 기록 안 됨
    .inline = 여기서 시청함, 아직 기록 안 됨
tokens-cube-on-disk = 디스크에 있음
    .inline = 디스크에 있음
tokens-cube-downloading = 다운로드 중
    .inline = 다운로드 중
tokens-cube-missing = 누락
    .inline = 누락
tokens-cube-error = 오류
    .inline = 오류
tokens-cube-not-downloaded = 다운로드 안 됨
    .inline = 다운로드 안 됨
tokens-cube-not-aired = 아직 방영 안 됨
    .inline = 아직 방영 안 됨
tokens-cube-available = 받을 수 있음
    .inline = 받을 수 있음
tokens-cube-picked = 받을 항목으로 선택됨
    .inline = 받을 항목으로 선택됨
tokens-cube-already-on-disk = 이미 디스크에 있음
    .inline = 이미 디스크에 있음
tokens-cube-no-release = 릴리스 없음
    .inline = 릴리스 없음

## Card

card-not-interested = 관심 없음

## PagePlaceholder

page-placeholder-coming-soon = 준비 중

## ErrorLine

error-line-hide-details = 세부 정보 숨기기
error-line-details = 세부 정보
error-line-details-tip = 무엇이 잘못됐는지 그대로 보여 주고 복사해요
error-line-copied = 복사했어요.

## Readouts (apps/linux/src/format.rs): times, countdowns and lengths

fmt-just-now = 방금
fmt-now = 지금
fmt-ago = { $span } 전
fmt-in = { $span } 후
fmt-seconds-short = { $n }초
fmt-minutes-short = { $n }분
fmt-hours-short = { $n }시간
fmt-days-short = { $n }일
fmt-months-short = { $n }개월
fmt-years-short = { $n }년
fmt-years-months-short = { $years }년 { $months }개월
fmt-countdown-days = { $d }일 { $h }시간 { $m }분
fmt-countdown-hours = { $h }시간 { $m }분
fmt-countdown-days-seconds = { $d }일 { $h }시간 { $m }분 { $s }초
fmt-countdown-hours-seconds = { $h }시간 { $m }분 { $s }초
fmt-countdown-minutes-seconds = { $m }분 { $s }초
fmt-runtime-minutes = { $m }분
fmt-runtime-hours = { $h }시간
fmt-runtime-hours-minutes = { $h }시간 { $m }분
fmt-episode = 에피소드
fmt-size-b = { $size } B
fmt-size-kb = { $size } KB
fmt-size-mb = { $size } MB
fmt-size-gb = { $size } GB
fmt-size-tb = { $size } TB
fmt-episode-span = { $first }~{ $through }화

## apps/linux/src/problem.rs: every call error in a person's words, what happened and what to do

problem-report = 다시 시도하세요. 계속 일어나면 활동 로그에서 신고하세요.
problem-try-later = 나중에 다시 시도하세요.
problem-helper-unreachable = 검색 도우미가 응답하지 않았어요.
problem-helper-unauthorized = 검색 도우미가 로그인을 거부했어요.
problem-helper-not-running = 검색 도우미가 실행 중이 아니에요.
problem-helper-missing = 검색 도우미가 설치되지 않았어요.
problem-helper-failed = 검색 도우미가 그 작업을 하지 못했어요.
problem-helper-no-login = 검색 도우미가 토렌트 클라이언트에 로그인할 수 없어요.
problem-helper-no-login-next = 설정의 다운로드에서 토렌트 클라이언트 비밀번호를 저장하세요.
problem-torrent-unreachable = 토렌트 클라이언트가 응답하지 않았어요.
problem-torrent-unauthorized = 토렌트 클라이언트가 로그인을 거부했어요.
problem-torrent-not-running = 토렌트 클라이언트가 실행 중이 아니에요.
problem-torrent-missing = 토렌트 클라이언트가 설치되지 않았어요.
problem-torrent-failed = 토렌트 클라이언트가 그 작업을 하지 못했어요.
problem-downloads-unreachable-next = 실행 중인지, 설정의 다운로드에 있는 주소가 맞는지 확인하세요.
problem-downloads-unauthorized-next = 설정의 다운로드에서 사용자 이름과 비밀번호를 확인하세요.
problem-downloads-not-running-next = 설정의 다운로드에서 시작하세요.
problem-downloads-missing-next = 설정 방법은 설정의 다운로드에 있어요.
problem-trash = { -app-name }이(가) 파일을 휴지통으로 옮기지 못했어요.
problem-trash-next = 내 파일인지, 디스크가 읽기 전용이 아닌지 확인하세요.
problem-source-list = 그 소스 목록이 동작하지 않았어요.
problem-source-list-next = 설정의 다운로드, 소스 목록에서 주소를 확인하세요.
problem-remote-unreachable = { $name }이(가) 응답하지 않았어요.
problem-remote-unreachable-next = 사이트가 다운됐을 수 있어요. 나중에 다시 시도하거나 다른 소스를 검색하세요.
problem-remote-unauthorized = { $name }이(가) 받은 키를 거부했어요.
problem-remote-unauthorized-next = 설정의 다운로드, 소스 목록에서 키를 바꾸세요.
problem-remote-failed = { $name }이(가) 그 작업을 하지 못했어요.
problem-remote-failed-next = 나중에 다시 시도하거나 다른 소스를 검색하세요.
problem-provider-sign-in = { $name }이(가) { -app-name } 로그인을 거부했어요.
problem-provider-sign-in-next = 설정의 계정에서 다시 연결하세요.
problem-provider-turned-away = { $name }이(가) { -app-name }의 요청을 거부했어요.
problem-provider-busy = 지금 { $name }이(가) 바빠요.
problem-provider-busy-next = 1분 기다린 뒤 다시 시도하세요.
problem-provider-no-answer = { $name }이(가) 응답하지 않았어요.
problem-provider-no-answer-next = 인터넷 연결을 확인하고 다시 시도하세요.
problem-provider-trouble = 지금 { $name }에 문제가 있어요.
problem-provider-trouble-next = 몇 분 뒤 다시 시도하세요.
problem-provider-rejected = { $name }이(가) 받아들이지 않았어요.
problem-not-found = { -app-name }이(가) 더 이상 찾을 수 없어요.
problem-not-found-next = 제거됐을 수 있어요. 다시 시도하면 페이지를 새로 불러와요.
problem-invalid-next = 확인하고 다시 시도하세요.
problem-unavailable = 지금은 그 폴더에 접근할 수 없어요.
problem-unavailable-next = 드라이브를 연결하거나 설정의 라이브러리에서 폴더를 확인하세요.
problem-not-connected = { $tracker }이(가) 연결되지 않았어요.
problem-not-connected-next = 설정의 계정에서 연결하세요.
problem-io = { -app-name }이(가) 파일을 읽거나 쓰지 못했어요.
problem-io-next = 디스크에 공간이 있는지, 내 폴더인지 확인하고 다시 시도하세요.
problem-storage = { -app-name }이(가) 라이브러리를 저장하지 못했어요.
problem-storage-next = 디스크에 공간이 있는지 확인하고 다시 시도하세요.
problem-keyring = 시스템 비밀번호 저장소가 응답하지 않았어요.
problem-keyring-next = 데스크톱 키링의 잠금을 풀고 다시 시도하세요.
problem-unsupported = 여기서는 { -app-name }이(가) 아직 그 작업을 할 수 없어요.
problem-version = 더 새로운 { -app-name }에서 만든 백업이에요.
problem-version-next = 설정의 정보에서 { -app-name }을(를) 업데이트한 뒤 다시 가져오세요.
problem-internal = { -app-name } 내부에서 문제가 생겼어요.
problem-refused-hidden = 숨긴 작품이에요.
problem-refused-hidden-next = 먼저 작품 페이지에서 숨김을 해제하세요.
problem-refused-no-match = 아직 매칭되지 않은 작품이에요.
problem-refused-no-match-next = 먼저 작품 정보에서 매칭하세요.
problem-refused-not-newer = 목록이 이미 이 에피소드 이후까지 기록돼 있어요.
problem-refused-extra = 에피소드가 아니라 번외 파일이에요.
problem-refused-unmatched = 이 파일은 에피소드와 연결되지 않았어요.
problem-refused-unmatched-next = 작품 페이지의 수정에서 고치세요.
problem-refused-on-disk = 이 컴퓨터에만 있는 파일이에요.

## apps/linux/src/library_view.rs: the library page's section titles and the one fact under each

library-view-airing = 방영 중
library-view-no-date = 날짜 없음
library-view-type-tv = TV
library-view-type-movies = 영화
library-view-type-extras = OVA, ONA, 스페셜
library-view-type-music = 뮤직
library-view-type-unknown = 알 수 없음
library-view-no-genre = 매칭 안 됨
library-view-state-caught-up = 모두 따라잡음
library-view-state-not-started = 시작 안 함
library-view-next-episode = 다음 에피소드: { $span } 후
library-view-countdown-days = { $d }일 { $h }시간
library-view-countdown-hours = { $h }시간 { $m }분
library-view-countdown-minutes = { $m }분
library-view-in-progress = 보는 중 { $count }개
library-view-not-started = 시작 안 함 { $count }개
library-view-all-watched = 모두 시청함
library-view-on-disk =
    디스크에 { $count ->
       *[other] 에피소드 { $count }개
    }
library-view-dropped-on-disk =
    디스크에 남은 { $count ->
       *[other] 파일 { $count }개
    }

## apps/linux/src/dbus/notify.rs: the desktop notification for a new episode

notify-episode-out = { $title } { $episode }화가 공개됐어요
notify-click-open = 클릭하면 { -app-name }에서 열려요
notify-click-get = 클릭하면 { -app-name }에서 에피소드를 받아요
notify-get-episodes = 에피소드 받기

## apps/linux/src/nowplaying.rs: the second line a media widget shows while an episode plays

now-playing-show-episode = { $show }, { $episode }

## apps/linux/src/dbus/inhibit.rs: why the screen stays on, as the desktop's power panel shows it

inhibit-reason = 에피소드 재생 중

## apps/linux/src/bridge/door.rs: why a value a page sent cannot be used (an error's detail line)

door-not-a-choice = { $value }은(는) 올바른 선택이 아니에요
door-tokens-op-empty = tokens op에는 tokens가 필요해요
door-not-a-refine-op = { $value }은(는) refine op가 아니에요
door-limit-too-low = 1 이상이어야 해요. 받은 값: { $limit }
door-not-a-torrent-action = { $value }은(는) 토렌트 동작이 아니에요

## apps/linux/src/bridge/shell.rs: the first-run starter folder and the update notice

shell-starter-folder-failed = { $folder }을(를) 만들지 못했어요: { $error }
shell-update-gone = { -app-name } { $version }은(는) 더 이상 제공되지 않아요

## Watch together (#846)

onboarding-step-together = 함께 보기
onboarding-title-together = 친구와 함께 보기
onboarding-body-together = 플레이어에서 방을 만들고 코드를 보내세요. 각자 자기 파일로 에피소드를 재생하고, 재생, 일시정지, 탐색은 모두에게 똑같이 맞춰져요.
onboarding-ready-together = 함께 보기
onboarding-ready-together-as = 방에서 { $name }(으)로 보여요.
onboarding-ready-together-none = 아직 이름이 없어요. 처음 쓸 때 방 패널에서 물어봐요.
onboarding-together-you = 나
onboarding-together-code = 방 코드
onboarding-together-episode = Frieren · 3화
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = 방에서 쓸 이름
rail-room = 함께 보기(Ctrl+J)
rail-room-in = 함께 보기 방에 있음(Ctrl+J)
player-chrome-watch-together = 함께 보기
player-chrome-room = 방 { $code } · { $count ->
   *[other] { $count }명
}
room-name-empty = 방에서 다른 사람에게 보일 이름을 입력하세요.
room-name-bad = 쓸 수 없는 이름이에요. 1~32자로 정하세요.
room-error-not-found = 그 코드의 방이 없어요. 보낸 사람에게 확인하세요. 방이 닫혔을 수 있어요.
room-error-full = 방이 꽉 찼어요. 방에는 10명까지 들어갈 수 있어요.
room-error-unreachable = 방 서버에 연결하지 못했어요. 연결을 확인하고 다시 시도하세요.
room-error-refused = 방 서버가 거부했어요. 잠시 후 다시 시도하세요.
room-error-proto = 그 방은 더 새로운 버전의 방을 써요. 참여하려면 { -app-name }을(를) 업데이트하세요.
room-error-not-in-room = 더 이상 그 방에 있지 않아요. 방을 다시 만들거나 참여하세요.
room-error-bad-code = 방 코드는 K7QM2X처럼 영문자와 숫자 6자예요.
room-error-bad-relay = 설정, 재생의 방 서버가 웹 주소가 아니에요. 거기서 고친 뒤 다시 시도하세요.
room-panel-title = 함께 보기
room-panel-close = 닫기
room-panel-connected = 연결됨
room-panel-reconnecting = 다시 연결 중
room-panel-name-label = 방에서 쓸 이름
room-panel-name-placeholder = 이름
room-panel-name-start = 방 만들기
room-panel-name-join = 방 참여
room-panel-start = 방 만들기
room-panel-or-join = 또는 코드로 참여
room-panel-join-label = 방 코드
room-panel-join = 참여
room-panel-you-are = { $name }(으)로 보여요
room-panel-change-name = 변경
room-panel-code-label = 방 코드
room-panel-copy = 링크 복사
room-panel-link-copied = 방 링크 복사됨
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = 이 에피소드가 라이브러리에 없어서 아직 같이 볼 수 없어요.
room-panel-open-episode = 에피소드 열기
room-panel-people = { $count ->
   *[other] { $count }명
}
room-panel-host = 호스트
room-panel-you = 나
room-panel-ready = 준비됨
room-panel-opening = 여는 중
room-panel-missing = 파일 없음
room-panel-offset = 내 오프셋
room-panel-offset-earlier = 0.5초 빠르게
room-panel-offset-later = 0.5초 느리게
room-panel-seconds = { $value }초
room-panel-suggested = 오프닝 기준 추천: { $offset }
room-panel-use-suggested = 적용
room-panel-leave = 방 나가기
player-room-speed = 속도는 방에 맞춰져요
settings-find-room-name =
    .words = 함께 보기 방 이름 친구
settings-find-room-relay =
    .words = 함께 보기 방 릴레이 서버
key-help-key-ctrl-j = Ctrl+J
key-help-room = 함께 보기
settings-playback-together = 함께 보기
settings-playback-room-name = 함께 보기 방에서 쓸 이름
settings-playback-room-name-placeholder = 이름
settings-playback-room-relay = 방 서버
settings-playback-room-relay-helper = 방은 여기서 만들어져요. 직접 운영하는 서버일 때만 바꾸세요.
