# ko translation of en/core.ftl. An ID missing here is shown in English.

## The tailnet host

tailnet-person-no-name = Tailscale에 이름 없음

## Counts

count-episodes =
    { $count ->
       *[other] 에피소드 { $count }개
    }
count-shows =
    { $count ->
       *[other] 작품 { $count }개
    }

## Actions every screen uses

action-cancel = 취소
action-close = 닫기
action-try-again = 다시 시도
action-remove = 제거
action-delete = 삭제
action-open = 열기
action-save = 저장
action-apply = 적용
action-clear = 지우기
action-refresh = 새로고침
action-play = 재생
action-connect = 연결
action-start = 시작
action-back = 뒤로
action-see-all = 모두 보기
action-open-series = 작품 열기
action-open-settings = 설정 열기

## The lists a tracker keeps

status-watching = 시청 중
status-completed = 완료
status-paused = 일시정지
status-on-hold = 보류
status-dropped = 중단
status-planning = 시청 예정
status-rewatching = 재시청 중

## Episodes

episode-number = { $number }화

## AniList's words for a show: its format, its airing status, how two shows relate

anilist-format-tv = TV
anilist-format-tv-short = TV 숏
anilist-format-film = 영화
anilist-format-special = 스페셜
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = 뮤직
anilist-status-finished = 완결
anilist-status-airing = 방영 중
anilist-status-not-yet-aired = 방영 전
anilist-status-cancelled = 취소됨
anilist-status-hiatus = 휴재 중
anilist-relation-adaptation = 각색
anilist-relation-prequel = 전편
anilist-relation-sequel = 속편
anilist-relation-parent = 본편
anilist-relation-side-story = 외전
anilist-relation-character = 캐릭터
anilist-relation-summary = 요약
anilist-relation-alternative = 대체 버전
anilist-relation-spin-off = 스핀오프
anilist-relation-other = 기타
anilist-relation-source = 원작
anilist-relation-compilation = 총집편
anilist-relation-contains = 포함

## The activity log: a problem whose place had no words of its own

event-problem-library = 라이브러리에 문제가 생겼어요. 자세한 내용은 개발자 옵션에 있어요.
event-problem-metadata = 작품 정보에 문제가 생겼어요. 자세한 내용은 개발자 옵션에 있어요.
event-problem-trackers = 목록에 문제가 생겼어요. 자세한 내용은 개발자 옵션에 있어요.
event-problem-franchise = 관련 작품에 문제가 생겼어요. 자세한 내용은 개발자 옵션에 있어요.
event-problem-playback = 재생에 문제가 생겼어요. 자세한 내용은 개발자 옵션에 있어요.
event-problem-store = 데이터를 저장하다가 문제가 생겼어요. 자세한 내용은 개발자 옵션에 있어요.
event-problem-system = Anibeam에 문제가 생겼어요. 자세한 내용은 개발자 옵션에 있어요.
event-problem-downloads = 다운로드에 문제가 생겼어요. 자세한 내용은 개발자 옵션에 있어요.

## Errors (contract::CoreError)

core-error-not-found-source = 폴더 { $id }을(를) 찾지 못했어요.
core-error-not-found-series = 작품 { $id }을(를) 찾지 못했어요.
core-error-not-found-file = 파일 { $id }을(를) 찾지 못했어요.
core-error-not-found-session = 재생 세션 { $id }을(를) 찾지 못했어요.
core-error-not-found-job = 작업 { $id }을(를) 찾지 못했어요.
core-error-invalid = { $field } 값이 올바르지 않아요: { $message }.
core-error-invalid-ended = { $field } 값이 올바르지 않아요: { $message }
core-error-unavailable = 이 폴더를 사용할 수 없어요: { $path }
core-error-not-connected = { $tracker }이(가) 연결되지 않았어요.
core-error-refused-hidden = 할 수 없어요: 숨긴 작품이에요.
core-error-refused-no-match = 할 수 없어요: 아직 매칭되지 않은 작품이에요. 작품 정보에서 매칭하세요.
core-error-refused-not-newer = 할 수 없어요: 목록이 이미 이 에피소드 이후까지 기록돼 있어요.
core-error-refused-extra = 할 수 없어요: 번외 에피소드예요.
core-error-refused-unmatched = 할 수 없어요: 이 파일은 에피소드에 매칭되지 않았어요.
core-error-refused-on-disk = 할 수 없어요: 디스크에만 있는 파일이에요.
# Why a list was not updated, read after "Not tracked:" or a tracker's name, on every shell's player
refusal-hidden = 숨긴 작품이에요
refusal-no-match = 아직 매칭되지 않은 작품이에요. 작품 정보에서 매칭하세요
refusal-not-newer = 목록이 이미 이 에피소드 이후까지 기록돼 있어요
refusal-extra = 번외 에피소드예요
refusal-unmatched = 이 파일은 에피소드에 매칭되지 않았어요
refusal-on-disk = 디스크에만 있는 파일이에요
core-error-provider = { $provider }에서 응답이 없어요: { $message }
core-error-io = 파일을 읽거나 쓰지 못했어요: { $message }
core-error-storage = 라이브러리를 저장하지 못했어요: { $message }
core-error-keyring = 시스템 비밀번호 저장소에 연결하지 못했어요: { $message }
core-error-unsupported = { $what }은(는) 아직 지원하지 않아요.
core-error-version = 더 새로운 Anibeam에서 내보낸 파일이에요(형식 { $found }). 가져오려면 Anibeam을 업데이트하세요.
core-error-internal = Anibeam 내부 오류가 발생했어요: { $message }
core-error-service = { $service }: { $message }

## releases/src/repos/mod.rs

core-repos-paste-address = 소스 목록 주소를 붙여넣으세요
core-repos-must-be-https = 주소는 https를 써야 해요
core-repos-unreachable = 소스 목록에 연결하지 못했어요: { $message }
core-repos-not-found = 그 주소에서 아무것도 찾지 못했어요(비공개 GitHub 저장소도 404를 반환해요)
core-repos-http-status = 소스 목록이 HTTP { $status }(으)로 응답했어요
core-repos-unreadable = { $path }을(를) 읽지 못했어요: { $error }
core-repos-no-such-endpoint = 그 ID로 설치된 소스가 없어요
core-repos-key-changed = 이 소스 목록이 이제 신뢰한 키와 다른 키로 서명돼 있어요. 새 키를 받아들이려면 제거하고 다시 추가하세요.
core-repos-stale = 소스 목록이 더 오래된 인덱스를 보냈어요(일련번호 { $kept } 이후에 { $serial }). 더 새로운 것을 유지해요
core-repos-key-not-shown = 서명 키가 표시된 키와 달라요. 소스 목록을 다시 추가하고 지문을 비교하세요.
core-repos-repo-added = 소스 목록을 추가했어요.
core-repos-repo-removed = 소스 목록을 제거했어요.
core-repos-no-field = { $name }에 { $key } 필드가 없어요
core-repos-needs-field = { $name }에 { $label }이(가) 필요해요
core-repos-host-only = { $name }: { $label }에는 192.0.2.10:9696처럼 호스트나 호스트:포트만 입력해야 해요
core-repos-not-added = 추가되지 않은 소스 목록이에요
core-repos-not-offered = { $repo }에는 { $id }이(가) 없어요
core-repos-already-installed = { $name }은(는) 이미 설치돼 있어요
core-repos-taken-by-you = { $id }(이)라는 소스가 이미 설치돼 있어요(직접 추가함). 먼저 제거하세요
core-repos-taken-by-repo = { $id }(이)라는 소스가 이미 설치돼 있어요(다른 소스 목록). 먼저 제거하세요
core-repos-source-added = 다운로드 소스를 추가했어요.
core-repos-source-updated = 다운로드 소스를 업데이트했어요.
core-repos-source-removed = 다운로드 소스를 제거했어요.
core-repos-source-changed = 다운로드 소스를 편집했어요.
core-repos-give-name = 이름을 입력하세요
core-repos-put-query = 주소에서 검색어가 들어갈 자리에 {"{"}query{"}"}를 넣으세요
core-repos-name-taken = { $id }(이)라는 소스가 이미 설치돼 있어요
core-repos-only-by-hand = 직접 추가한 소스만 편집할 수 있어요
core-repos-keep-key = 주소의 {"{"}{ $key }{"}"}는 그대로 두고 { $label } 변경에서 값을 바꾸거나, 사이트를 제거하고 그 부분 없이 다시 추가하세요
core-repos-note-kind = anirss는 { $kind } 소스를 검색할 수 없어서 데스크톱에서는 건너뛰어요. 휴대전화 앱에서는 검색할 수 있어요
core-repos-note-by-hand = anirss에 직접 설정한 { $id }이(가) 따로 있어요. 데스크톱은 그것을 검색해요
core-repos-note-theirs = 이 anirss는 직접 관리하는 것이라 { -app-name }은(는) 설정을 편집하지 않아요. 데스크톱에서 검색하려면 anirss 설정에 { $id }을(를) 추가하세요
core-repos-note-sandbox = 샌드박스 모드에서는 anirss 설정에 쓰지 않으므로 여기서 데스크톱은 { $id }을(를) 검색하지 않아요
core-repos-note-old = anirss { $version }은(는) { -app-name }의 소스를 지원하지 않아요. 데스크톱에서 검색하려면 { $min } 이상으로 업데이트하세요

## releases/src/repos/index.rs

core-repos-key-not-ed25519 = 인덱스 키가 ed25519 키가 아니에요
core-repos-key-not-base64 = 인덱스 키가 32바이트 base64가 아니에요
core-repos-key-invalid = 인덱스 키가 올바른 ed25519 키가 아니에요
core-repos-too-large = 소스 목록이라고 하기엔 파일이 너무 커요
core-repos-not-a-repo = 올바른 { -app-name } 소스 목록이 아니에요
core-repos-format-version = 이 소스 목록은 형식 버전 { $found }을(를) 쓰지만 { -app-name }은(는) 버전 { $reads }을(를) 읽어요. { -app-name }을(를) 업데이트하세요.
core-repos-payload-not-base64 = 소스 목록 페이로드가 base64가 아니에요
core-repos-signature-malformed = 소스 목록 서명 형식이 잘못됐어요
core-repos-index-unreadable = 소스 목록 인덱스를 읽지 못했어요: { $error }
core-repos-signature-mismatch = 소스 목록 서명이 내용과 일치하지 않아요
core-repos-check-scheme = { $id }: 주소는 https://로 시작해야 해요
core-repos-check-scheme-http = { $id }: 주소는 http:// 또는 https://로 시작해야 해요
core-repos-check-auth-key = { $id }: 잘못된 인증 키 { $key }
core-repos-check-host-torznab = { $id }: torznab 주소만 {"{"}{ $key }{"}"}를 호스트로 쓸 수 있어요
core-repos-check-host-placeholder = { $id }: 호스트에는 자리표시자를 쓸 수 없어요

## playback/src/tracks.rs

core-playback-track-number = 트랙 { $number }
core-playback-track-file = { $label } (파일)
core-playback-lang-region = { $name }({ $region })
core-playback-lang-english = 영어
core-playback-lang-japanese = 일본어
core-playback-lang-spanish = 스페인어
core-playback-lang-portuguese = 포르투갈어
core-playback-lang-french = 프랑스어
core-playback-lang-german = 독일어
core-playback-lang-italian = 이탈리아어
core-playback-lang-russian = 러시아어
core-playback-lang-chinese = 중국어
core-playback-lang-korean = 한국어
core-playback-lang-arabic = 아랍어
core-playback-lang-hindi = 힌디어
core-playback-lang-indonesian = 인도네시아어
core-playback-lang-malay = 말레이어
core-playback-lang-thai = 태국어
core-playback-lang-vietnamese = 베트남어
core-playback-lang-polish = 폴란드어
core-playback-lang-turkish = 튀르키예어
core-playback-lang-dutch = 네덜란드어
core-playback-lang-swedish = 스웨덴어
core-playback-lang-norwegian = 노르웨이어
core-playback-lang-danish = 덴마크어
core-playback-lang-finnish = 핀란드어
core-playback-lang-hebrew = 히브리어
core-playback-lang-ukrainian = 우크라이나어
core-playback-lang-czech = 체코어
core-playback-lang-hungarian = 헝가리어
core-playback-lang-greek = 그리스어
core-playback-lang-romanian = 루마니아어
core-playback-lang-filipino = 필리핀어
core-playback-region-brazil = 브라질
core-playback-region-portugal = 포르투갈
core-playback-region-latin-america = 중남미
core-playback-region-spain = 스페인
core-playback-region-simplified = 간체
core-playback-region-traditional = 번체

## contract/src/enums.rs

core-jobs-scan = 폴더 확인 중
    .stopped = 폴더 확인 중 문제가 생겨 멈췄어요.
core-jobs-auto-match = 작품 매칭 중
    .stopped = 작품 매칭 중 문제가 생겨 멈췄어요.
core-jobs-search = 작품 검색 중
    .stopped = 작품 검색 중 문제가 생겨 멈췄어요.
core-jobs-resolve-link = 링크 읽는 중
    .stopped = 링크를 읽다가 문제가 생겨 멈췄어요.
core-jobs-apply-match = 작품 매칭 중
    .stopped = 작품 매칭 중 문제가 생겨 멈췄어요.
core-jobs-refresh = 작품 업데이트 중
    .stopped = 작품 업데이트 중 문제가 생겨 멈췄어요.
core-jobs-refresh-all = 작품 정보 업데이트 중
    .stopped = 작품 정보 업데이트 중 문제가 생겨 멈췄어요.
core-jobs-refresh-airing = 방영 중인 작품 확인 중
    .stopped = 방영 중인 작품을 확인하다가 문제가 생겨 멈췄어요.
core-jobs-refresh-imdb = 평점 업데이트 중
    .stopped = 평점 업데이트 중 문제가 생겨 멈췄어요.
core-jobs-clear-images = 이미지 지우는 중
    .stopped = 이미지를 지우다가 문제가 생겨 멈췄어요.
core-jobs-fill-images = 이미지 가져오는 중
    .stopped = 이미지를 가져오다가 문제가 생겨 멈췄어요.
core-jobs-connect-tracker = 목록 연결 중
    .stopped = 목록 연결 중 문제가 생겨 멈췄어요.
core-jobs-mark = 에피소드를 시청함으로 표시 중
    .stopped = 에피소드를 시청함으로 표시하다가 문제가 생겨 멈췄어요.
core-jobs-set-progress = 진행 상황 설정 중
    .stopped = 진행 상황을 설정하다가 문제가 생겨 멈췄어요.
core-jobs-score = 점수 저장 중
    .stopped = 점수를 저장하다가 문제가 생겨 멈췄어요.
core-jobs-refresh-progress = 목록 진행 상황 읽는 중
    .stopped = 목록 진행 상황을 읽다가 문제가 생겨 멈췄어요.
core-jobs-refresh-watching = 시청 중 목록 읽는 중
    .stopped = 시청 중 목록을 읽다가 문제가 생겨 멈췄어요.
core-jobs-refresh-airing-now = 이번 시즌 확인 중
    .stopped = 이번 시즌을 확인하다가 문제가 생겨 멈췄어요.
core-jobs-refresh-recommended = 추천 작품 찾는 중
    .stopped = 추천 작품을 찾다가 문제가 생겨 멈췄어요.
core-jobs-read-show-page = 작품 읽는 중
    .stopped = 작품을 읽다가 문제가 생겨 멈췄어요.
core-jobs-on-hold = 손대지 않은 작품을 보류로 옮기는 중
    .stopped = 손대지 않은 작품을 보류로 옮기다가 문제가 생겨 멈췄어요.
core-jobs-set-status = 목록 상태 변경 중
    .stopped = 목록 상태를 바꾸다가 문제가 생겨 멈췄어요.
core-jobs-move-lists = 목록을 트래커로 옮기는 중
    .stopped = 목록을 트래커로 옮기다가 문제가 생겨 멈췄어요.
core-jobs-notes = 메모 저장 중
    .stopped = 메모를 저장하다가 문제가 생겨 멈췄어요.
core-jobs-crawl = 관련 작품 찾는 중
    .stopped = 관련 작품을 찾다가 문제가 생겨 멈췄어요.
core-jobs-skip-windows = 오프닝과 엔딩 찾는 중
    .stopped = 오프닝과 엔딩을 찾다가 문제가 생겨 멈췄어요.
core-jobs-export = 라이브러리를 파일로 저장 중
    .stopped = 라이브러리를 파일로 저장하다가 문제가 생겨 멈췄어요.
core-jobs-import = 파일에서 라이브러리 불러오는 중
    .stopped = 파일에서 라이브러리를 불러오다가 문제가 생겨 멈췄어요.
core-jobs-subscriptions = 구독 확인 중
    .stopped = 구독을 확인하다가 문제가 생겨 멈췄어요.
core-jobs-connect-downloads = 다운로드 앱 연결 중
    .stopped = 다운로드 앱을 연결하다가 문제가 생겨 멈췄어요.
core-jobs-search-releases = 에피소드 찾는 중
    .stopped = 에피소드를 찾다가 문제가 생겨 멈췄어요.
core-jobs-refine-releases = 에피소드 좁히는 중
    .stopped = 에피소드를 좁히다가 문제가 생겨 멈췄어요.
core-jobs-subscribe = 작품 구독 중
    .stopped = 작품을 구독하다가 문제가 생겨 멈췄어요.
core-jobs-download = 다운로드 추가 중
    .stopped = 다운로드를 추가하다가 문제가 생겨 멈췄어요.
core-jobs-remove-subscription = 구독 제거 중
    .stopped = 구독을 제거하다가 문제가 생겨 멈췄어요.
core-jobs-control-torrents = 다운로드 변경 중
    .stopped = 다운로드를 변경하다가 문제가 생겨 멈췄어요.
core-jobs-test-torrent-client = 다운로드 앱 테스트 중
    .stopped = 다운로드 앱을 테스트하다가 문제가 생겨 멈췄어요.
core-jobs-remove-show = 작품 제거 중
    .stopped = 작품을 제거하다가 문제가 생겨 멈췄어요.
core-jobs-repair-subscription = 구독 복구 중
    .stopped = 구독을 복구하다가 문제가 생겨 멈췄어요.
core-jobs-check-endpoints = 다운로드 소스 확인 중
    .stopped = 다운로드 소스를 확인하다가 문제가 생겨 멈췄어요.
core-jobs-start-endpoint = 다운로드 소스 시작 중
    .stopped = 다운로드 소스를 시작하다가 문제가 생겨 멈췄어요.
core-jobs-schedule-agent = 코딩 에이전트에게 방영 일정 요청 중
    .stopped = 코딩 에이전트에게 방영 일정을 요청하다가 문제가 생겨 멈췄어요.

## metadata/src/airing_now.rs

core-airing-now-now = 지금 방영 중
core-airing-now-winter = { $year }년 겨울
core-airing-now-spring = { $year }년 봄
core-airing-now-summer = { $year }년 여름
core-airing-now-fall = { $year }년 가을
core-airing-now-planned = { $site }의 시청 예정 목록에 { $title }을(를) 추가했어요.
core-airing-now-planned-both = { $site }와(과) { $other }의 시청 예정 목록에 { $title }을(를) 추가했어요.
core-airing-now-already-on =
    { $title }은(는) 이미 { $site } 목록에 { $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }(으)로 있어요.
core-airing-now-plan-failed = { $title }을(를) 시청 예정 목록에 추가하지 못했어요. 나중에 다시 시도하세요.

## metadata/src/aired.rs

core-aired-episode-out = { $title } { $number }화가 공개됐어요.

## playback/src/cast/mod.rs

core-cast-connecting = { $device }(으)로 캐스트 중.
core-cast-ended = { $device } 캐스트를 마쳤어요.
core-cast-stopped = { $device } 캐스트를 중지했어요.
core-cast-failed = { $device }(으)로 캐스트하지 못했어요. 같은 네트워크에 있는지 확인하세요.
core-cast-not-built = 이 빌드에는 캐스트 기능이 없어요
core-cast-serve-failed = 에피소드를 전송하지 못했어요: { $error }
core-cast-device-gone = 그 기기가 더 이상 네트워크에 없어요
core-cast-not-a-playlist = 재생목록 주소가 아니에요
core-cast-nothing = 캐스트 중인 항목이 없어요
core-cast-still-starting = 기기가 아직 시작 중이에요
core-cast-no-route = 이 기기에서 TV로 가는 경로가 없어요
core-cast-no-control = TV가 제어 주소를 알려 주지 않았어요
core-cast-not-handed = 상대 컴퓨터가 에피소드를 보내지 못했어요: { $error }
core-cast-empty-playlist = 상대 컴퓨터가 빈 재생목록을 보냈어요
core-cast-refused-both = { $error }. 원본 파일도 거부했어요({ $first })
core-cast-chromecast-stopped = Chromecast가 멈췄어요
core-cast-hevc = { $error }. 이 에피소드는 HEVC라서 최신 Chromecast(Google TV 또는 Ultra)에서만 지원돼요

## playback/src/cast/castv2.rs

core-cast-chromecast-unreachable = Chromecast에 연결하지 못했어요: { $error }
core-cast-chromecast-broke = Chromecast 연결이 끊겼어요: { $error }
core-cast-chromecast-no-player = Chromecast가 미디어 플레이어를 시작하지 않았어요
core-cast-chromecast-refused-player = Chromecast가 미디어 플레이어를 시작하지 못했어요({ $reason })
core-cast-no-reason = 이유 없음
core-cast-chromecast-cannot-play = Chromecast에서 이 파일을 재생할 수 없어요({ $kind })
core-cast-chromecast-error = Chromecast가 이 파일을 재생하다 오류로 멈췄어요

## playback/src/cast/dlna.rs

core-cast-tv-no-answer = TV가 응답하지 않았어요: { $error }
core-cast-tv-refused = TV가 { $action } 명령을 거부했어요
core-cast-tv-refused-code = TV가 { $action } 명령을 거부했어요({ $code })

## playback/src/cast/media.rs

core-cast-no-video = 이 파일에는 영상이 없어요.
core-cast-hi10p = 이 에피소드는 10비트 H.264(Hi10P)라서 TV와 Chromecast에서 디코딩할 수 없어요. 캐스트하려면 트랜스코딩이 필요한데, 아직 지원하지 않아요.
core-cast-video-codec = 이 에피소드의 영상({ $codec })은 캐스트할 수 없어요.
core-cast-audio-codec = 이 에피소드의 오디오({ $codec })는 캐스트할 수 없어요.

## downloads/src/phone/mod.rs

core-phone-download-started =
    { $count ->
       *[other] { $folder } 에피소드 { $count }개 다운로드를 시작했어요.
    }
core-phone-download-partial = { $folder }의 릴리스를 모두 추가하지는 못했어요: { $error }
core-phone-subscribed = { $name } 구독을 시작했어요.
core-phone-rule-stopped = { $name }의 새 에피소드를 확인하지 않아요: 소스가 제거됐거나 키를 다시 입력해야 해요.
core-phone-new-episode = { $name }의 새 에피소드를 다운로드하고 있어요.
core-phone-name-taken = { $name }: 같은 이름의 소스가 이미 있어요
core-phone-nothing-to-download = 다운로드할 항목이 없어요
core-phone-over-cap = { -app-name }의 저장 공간 제한을 넘게 돼요({ $cap } GB 중 { $used } GB 사용 중이거나 대기 중). 에피소드를 삭제하거나 제한을 늘리세요
core-phone-empty-search = 검색어가 비어 있어요
core-phone-no-results = { $query } 검색 결과가 없어요
core-phone-own-engine = 자체 토렌트 엔진

## downloads/src/phone/check.rs

core-phone-check-connected = 연결됨
core-phone-check-refused = 거부됨: { $error }
core-phone-check-dropped = 응답 없음(차단됨)
core-phone-check-finding = { $label }: 주소 확인 중
core-phone-check-failed = 실패: { $error }
core-phone-check-no-answer = 응답 없음
core-phone-check-private = { $label }: 비공개로 확인 중
core-phone-check-skipped-address = 건너뜀, IP 주소
core-phone-check-skipped-home = 건너뜀, 로컬 네트워크 이름
core-phone-check-no-record = 공개 레코드 없음, 네트워크 DNS 사용
core-phone-check-no-resolver = 응답한 DNS 리졸버 없음
core-phone-check-reaching = { $label }: { $ip }에 연결 중
core-phone-check-http = HTTP { $status }, { $bytes }바이트
core-phone-check-setup = 설정

## playback/src/session.rs

core-playback-not-saved = { $folder }의 진행 상황이 저장되지 않았어요. 먼저 작품을 매칭하세요.
core-playback-still-downloading = { $number }화는 아직 다운로드 중이에요, { $done }% 완료

## playback/src/skip.rs

core-playback-bad-duration = 길이는 0보다 큰 초 단위 숫자여야 해요

## core/src/agent.rs

core-agent-no-answer = 에이전트가 답을 돌려주지 않았어요
core-agent-searching = 웹에서 { $query } 검색 중
core-agent-searching-schedule = 웹에서 방영 일정 검색 중
core-agent-reading = { $url } 읽는 중
core-agent-reading-page = 페이지 읽는 중
core-agent-using = { $tool } 사용 중
core-agent-none = 이 컴퓨터의 코딩 에이전트
core-agent-ended = { $name }이(가) 답 없이 끝났어요
core-agent-too-long = { $name }이(가) 너무 오래 걸렸어요

## core/src/core.rs

core-core-guest-trackers = 그 컴퓨터의 트래커 계정은 소유자의 것이라 이 기기에서 업데이트할 수 없어요
core-core-guest-not-interested = 그 컴퓨터의 관심 없음은 소유자의 것이라 이 기기에서 바꿀 수 없어요
core-core-guest-recommended = 그 컴퓨터의 추천은 소유자의 목록에서 학습해요
core-core-nothing-picked = 선택한 항목 없음

## foundation/src/prefs.rs

core-prefs-not-a-provider = { $provider }은(는) 매칭 제공처가 아니에요
core-prefs-provider-count = { $provider }이(가) { $count }번 나와요. 제공처는 한 번씩만 넣을 수 있어요
core-prefs-outside = { $value }은(는) { $low }~{ $high } 사이여야 해요
core-prefs-empty = 비어 있음

## metadata/src/recommended/mod.rs

core-recommended-ranking = 추천 작품 순위 정하는 중

## metadata/src/recommended/sync.rs

core-recommended-finding-people = 취향이 비슷한 사람 찾는 중
core-recommended-reading-lists = 그 사람들의 목록 읽는 중
core-recommended-reading-yours = 내 작품 읽는 중
core-recommended-reading-suggestions = 추천할 작품 읽는 중

## metadata/src/recommended/page.rs

core-recommended-no-anime = { -anilist }에 애니 { $id }이(가) 없어요

## host/src/lib.rs

core-host-off = 꺼짐
core-host-starting = 시작 중
core-host-serving = { $addresses }에서 공유 중
core-host-no-tailscale = Tailscale이 연결되지 않았어요
core-host-cannot-listen = 대기할 수 없어요
core-host-port-taken = 포트 { $port }이(가) 사용 중이에요
core-host-cannot-listen-on = { $address }에서 대기할 수 없어요: { $error }

## host/src/addr.rs

core-host-not-an-address = { $text }은(는) 올바른 주소와 포트가 아니에요

## host/src/keys.rs

core-host-not-a-key = { $key }은(는) 기기 키가 아니에요

## net/src/limiter.rs

core-net-timed-out = { $ms }ms 후 시간 초과
core-net-rate-limited = { $provider } 요청 한도 초과

## net/src/lib.rs

core-net-waiting = { $host }: 최근에 시간이 초과됐어요. { -app-name }이(가) { $seconds }초 기다린 뒤 다시 시도해요

## net/src/anilist.rs

core-net-no-data = 데이터 없음

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version }에는 이 컴퓨터용 다운로드가 없어요
core-update-cannot-make = { $path }을(를) 만들지 못했어요: { $error }
core-update-cannot-mark = 새 AppImage를 실행 가능하게 만들지 못했어요: { $error }
core-update-cannot-replace = { $path }을(를) 교체하지 못했어요: { $error }
core-update-not-here = 이 설치본은 여기서 스스로 업데이트할 수 없어요
core-update-cannot-start = 다운로드를 시작하지 못했어요: { $error }
core-update-did-not-start = 다운로드가 시작되지 않았어요: { $error }
core-update-cannot-write = { $path }에 쓰지 못했어요: { $error }
core-update-stopped = 다운로드가 멈췄어요: { $error }
core-update-cancelled = 취소됨
core-update-ended-early = 다운로드가 일찍 끝났어요({ $size }바이트 중 { $got }바이트)
core-update-checksum = 다운로드가 GitHub의 체크섬과 일치하지 않아요
core-update-cannot-move = 다운로드한 파일을 제자리로 옮기지 못했어요: { $error }
core-update-cannot-open-dmg = DMG를 열지 못했어요: { $error }
core-update-cannot-read-dmg = DMG를 읽지 못했어요: { $error }
core-update-no-app = DMG에 앱이 없어요
core-update-cannot-copy = 새 앱을 이 앱 옆에 복사하지 못했어요: { $error }

## metadata/src/calendar.rs

core-calendar-season-episode = { $title } - 시즌 { $season } { $number }화
core-calendar-episode = { $title } - { $number }화

## foundation/src/paths.rs

core-paths-no-home = 홈 디렉터리가 없어요

## core/src/first_run.rs

core-first-run-not-absolute = 동영상 폴더는 절대 경로여야 해요

## playback/src/presence/mod.rs

core-presence-extra = 번외
core-presence-room = 함께 보는 중
core-presence-in-room = 방에 있음
core-presence-no-room = 이 Discord 초대에는 아직 방이 없습니다. 방을 열어 달라고 하세요.

## trackers/src/accounts.rs

core-trackers-no-client-id = { $tracker } 클라이언트 ID가 설정되지 않았어요.
core-trackers-mal-reconnect = 설정에서 MyAnimeList를 다시 연결하세요.
core-trackers-mal-wants-secret = MyAnimeList에 이 앱의 클라이언트 시크릿이 필요해요. Anibeam 설정의 계정에서 추가하거나, App Type을 "other"로 해서 앱을 등록하세요.
core-trackers-token-reply-unreadable = 토큰 응답을 읽지 못했어요: { $error }
core-trackers-token-reply-no-token = 토큰 응답에 액세스 토큰이 없어요

## trackers/src/lib.rs

core-trackers-no-token = 저장된 액세스 토큰이 없어요. 설정에서 다시 연결하세요

## trackers/src/cache.rs

core-trackers-no-user-id = 계정 응답에 사용자 ID가 없어요

## trackers/src/move_up.rs

core-trackers-move-none-picked = 옮길 작품을 선택하지 않았어요
core-trackers-moved-up =
    { $count ->
       *[other] 작품 { $count }개를 이 컴퓨터에서 { $site }(으)로 옮겼어요.
    }
core-trackers-moved-up-some-refused = { $site }(으)로 작품 { $moved }개를 옮겼어요. 작품 { $failed }개는 옮기지 못했어요.
core-trackers-move-refused = { $site }에서 { $show }을(를) 업데이트하지 못했어요: { $error }

## trackers/src/oauth.rs

core-trackers-oauth-timed-out = 인증 시간이 초과됐어요. 다시 시도하세요.
core-trackers-oauth-state-mismatch = state가 일치하지 않아요. CSRF일 수 있어 중단했어요.
core-trackers-oauth-no-code = 콜백에 코드가 없어요
core-trackers-oauth-no-token = 콜백에 access_token이 없어요
core-trackers-oauth-waiting = 브라우저에서 { $site }에 로그인하기를 기다리는 중이에요.
core-trackers-oauth-connected = { $site }에 { $username } 계정으로 연결했어요.
core-trackers-oauth-no-name = 프로필 응답에 사용자 이름이 없어요
core-trackers-oauth-could-not-start = 로그인을 시작하지 못했어요
core-trackers-oauth-port-failed = 이 컴퓨터의 포트 { $port }을(를) 열지 못했어요: { $error }
core-trackers-oauth-port-busy = 이 컴퓨터의 포트 { $port }을(를) 다른 프로그램이 쓰고 있어요. 잠시 후 다시 시도하거나, 계속 사용 중이면 그 프로그램을 닫으세요.
core-trackers-oauth-provider-error = { $site }에서 오류를 보냈어요: { $error }
core-trackers-page-connected = { $site }이(가) { $app }에 연결됐어요
core-trackers-page-close-tab = 이 탭을 닫아도 돼요.
core-trackers-page-failed-title = { $app }, 로그인 실패
core-trackers-page-failed = 로그인을 마치지 못했어요
core-trackers-page-go-back = { $app }(으)로 돌아가서 다시 시도하세요.
core-trackers-page-finishing-title = { $app }, 마무리 중
core-trackers-page-finishing = 마무리 중
core-trackers-page-finishing-body = { $site }이(가) 응답했고 Anibeam이 로그인을 마무리하고 있어요. 계속하려면 { $app }(으)로 돌아가세요.
core-trackers-page-cancelled-title = { $app }, 로그인 취소됨
core-trackers-page-cancelled = 로그인이 취소됐어요
core-trackers-page-cancelled-body = 이 탭을 닫아도 돼요. 그래도 로그인하려면 { $app }에서 다시 시작하세요.

## trackers/src/on_hold.rs

core-trackers-on-hold-moving = { $tracker }에서 보류로 옮기는 중
core-trackers-on-hold-moved =
    { $count ->
       *[other] { $site }에서 작품 { $count }개를 보류로 옮겼어요.
    }
core-trackers-on-hold-moved-two =
    { $count ->
       *[other] { $site }에서 작품 { $count }개, { $secondSite }에서 { $second }개를 보류로 옮겼어요.
    }
core-trackers-on-hold-failed =
    { $failed ->
       *[other] 작품 { $failed }개를 보류로 옮기지 못했어요.
    }
core-trackers-on-hold-moved-failed =
    { $count ->
       *[other] { $site }에서 작품 { $count }개를 보류로 옮겼어요.
    } { $failed ->
       *[other] 작품 { $failed }개는 보류로 옮기지 못했어요.
    }
core-trackers-on-hold-moved-two-failed =
    { $count ->
       *[other] { $site }에서 작품 { $count }개, { $secondSite }에서 { $second }개를 보류로 옮겼어요.
    } { $failed ->
       *[other] 작품 { $failed }개는 보류로 옮기지 못했어요.
    }
core-trackers-series-fallback = 작품 { $id }
core-trackers-back-on-watching = { $name }이(가) { $site }에서 다시 시청 중으로 돌아왔어요.
core-trackers-back-on-watching-two = { $name }이(가) { $site }와(과) { $secondSite }에서 다시 시청 중으로 돌아왔어요.
core-trackers-back-on-watching-failed = { $name }을(를) 시청 중으로 되돌리지 못했어요. 나중에 다시 시도하세요.

## trackers/src/writes.rs

core-trackers-rate-limited = { $tracker } 요청 한도 초과. 1분 뒤 다시 시도하세요.
core-trackers-auth-expired = { $tracker } 인증이 만료됐어요. 설정에서 다시 연결하세요.
core-trackers-entry-not-found = { $tracker }에서 항목을 찾지 못했어요.
core-trackers-server-error = { $tracker } 서버 오류({ $status }). 나중에 다시 시도하세요.
core-trackers-error-see-log = { $tracker } 오류. 활동 로그를 확인하세요.
core-trackers-episode-not-whole = { $episode }은(는) 0보다 큰 정수여야 해요
core-trackers-score-out-of-range = { $score }은(는) 0~10 사이여야 해요
core-trackers-rewatch-not-settable = 재시청은 직접 설정할 수 없어요
core-trackers-notes-too-long = 메모는 { $max }자를 넘을 수 없어요
core-trackers-nothing-picked = 선택한 항목 없음
core-trackers-notes-not-on-list = 아직 { $tracker } 목록에 없어요. 먼저 목록에 추가하세요.
core-trackers-sites =
    { $sites ->
        [both] { -anilist }와 { -mal }
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
       *[repeating] 재시청 중
    }
core-trackers-marked = { core-trackers-sites }에서 { $show } { $episode }화를 표시했어요.
core-trackers-progress-set = { core-trackers-sites }에서 { $show } 진행 상황을 { $progress }화로 설정했어요.
core-trackers-scored = { core-trackers-sites }에서 { $show }에 { $score }점을 줬어요.
core-trackers-score-cleared = { core-trackers-sites }에서 { $show } 점수를 지웠어요.
core-trackers-status-set = { core-trackers-sites }에서 { $show } 상태를 { core-trackers-status }(으)로 설정했어요.
core-trackers-notes-saved = { $show } 메모를 { core-trackers-sites }에 저장했어요.
core-trackers-notes-cleared = { core-trackers-sites }에서 { $show } 메모를 지웠어요.
core-trackers-update-failed = { $site }을(를) 업데이트하지 못했어요. 나중에 다시 시도하세요.
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
       *[no] 작품 { $done }개
    }
core-trackers-written-done =
    { $action ->
        [clear] { core-trackers-sites }에서 { core-trackers-written-who }의 점수를 지웠어요.
        [status] { core-trackers-sites }에서 { core-trackers-written-who }을(를) { core-trackers-status }(으)로 설정했어요.
       *[score] { core-trackers-sites }에서 { core-trackers-written-who }에 { $score }점을 줬어요.
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] 작품 { $failed }개를 업데이트하지 못했어요: { $reason }.
       *[no] 작품 { $failed }개를 업데이트하지 못했어요.
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] { $failed }개는 업데이트하지 못했어요: { $reason }.
       *[no] { $failed }개는 업데이트하지 못했어요.
    }
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
       *[other] {" "}작품 { $noMatch }개는 트래커 매칭이 없어요.
    }{ $hidden ->
        [0] {""}
       *[other] {" "}작품 { $hidden }개는 숨김 상태라 동기화가 꺼져 있어요.
    }{ $notConnected ->
        [0] {""}
       *[other] {" "}작품 { $notConnected }개는 연결되지 않은 트래커에만 있어요.
    }{ $kept ->
        [0] {""}
       *[other] {" "}작품 { $kept }개는 보낼 트래커가 없어 이 컴퓨터에 보관돼요.
    }

## metadata/src/automatch.rs

core-metadata-matched-to = { $show }을(를) { $title }에 매칭했어요.
core-metadata-auto-matched =
    { $count ->
       *[other] 작품 { $count }개를 매칭했어요.
    }
core-metadata-auto-unmatched =
    { $count ->
       *[other] 작품 { $count }개는 아직 매칭이 필요해요.
    }
core-metadata-auto-matched-some = 작품 { $matched }개를 매칭했어요. { $unmatched }개는 아직 매칭이 필요해요.
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
        [yes] { $hitFolder }을(를) { $hitTitle }에 매칭했어요.
       *[no] 작품 { $matched }개를 매칭했어요.
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] { $missName }의 매칭을 찾지 못했어요.
       *[no] 작품 { $unmatched }개의 매칭을 찾지 못했어요.
    }
core-metadata-find-failed = 작품 { $failed }개를 검색하지 못했어요. 나중에 다시 시도하세요.
core-metadata-find-had = 작품 { $had }개는 이미 매칭돼 있었어요.
core-metadata-find-busy = 작품 { $busy }개는 이미 매칭 중이에요.

## metadata/src/apply.rs

core-metadata-link-unreadable = 그 링크를 읽지 못했어요. AniList, MyAnimeList, TVmaze 또는 IMDb 페이지 주소를 붙여넣으세요.
core-metadata-no-mal-entry = AniList에 그 MyAnimeList ID의 항목이 없어요.
core-metadata-no-imdb-show = TVmaze에 그 IMDb 작품이 없어요.
core-metadata-search-on = { $provider } 검색
core-metadata-query-too-short = 검색하려면 두 글자 이상 입력하세요.
core-metadata-not-a-link = 링크가 아니에요
core-metadata-refresh-unsupported = MAL 전용 또는 TMDB 매칭 새로고침
core-metadata-matched = { $show } 매칭 완료.
core-metadata-updated = { $show } 정보를 업데이트했어요.
core-metadata-update-failed = { $show } 정보를 업데이트하지 못했어요. 나중에 다시 시도하세요.
core-metadata-progress-refresh = 새로고침
core-metadata-progress-backfill = 채우기
core-metadata-updated-many =
    { $count ->
       *[other] 작품 { $count }개의 정보를 업데이트했어요.
    }
core-metadata-update-failed-many =
    { $count ->
       *[other] 작품 { $count }개의 정보를 업데이트하지 못했어요. 나중에 다시 시도하세요.
    }
core-metadata-updated-some =
    { $count ->
       *[other] 작품 { $count }개의 정보를 업데이트했어요. { $failed }개는 업데이트하지 못했어요.
    }

## metadata/src/fetch.rs

core-metadata-no-such-show = TVmaze에 작품 { $id }이(가) 없어요
core-metadata-no-such-media = AniList에 미디어 { $id }이(가) 없어요

## metadata/src/imdb.rs

core-metadata-progress-imdb-links = Wikidata의 IMDb 링크
core-metadata-progress-imdb-ratings = IMDb 평점

## metadata/src/franchise/layout.rs

core-franchise-relation-character = 같은 캐릭터

## net/src/images.rs

core-images-progress-fetching = 이미지 가져오는 중

## host/src/transfer/export.rs

core-transfer-exported =
    { $count ->
       *[other] 작품 { $count }개를 { $folder }에 저장했어요.
    }

## host/src/transfer/format.rs

core-transfer-not-json = JSON이 아니에요: { $error }
core-transfer-wrong-format = { $format } 문서가 아니에요

## host/src/transfer/import.rs

core-transfer-imported = 파일에서 라이브러리를 불러왔어요.

## library/src/delete.rs

core-library-no-such-file = 라이브러리에 파일 { $id }이(가) 없어요
core-library-file-elsewhere-not-removed = 파일 { $id }은(는) 다른 작품의 파일이라 아무것도 제거하지 않았어요
core-library-file-outside = { $path }은(는) 라이브러리 폴더 안의 파일이 아니라서 아무것도 제거하지 않았어요
core-library-episodes-removed =
    { $title } 에피소드 { $count }개를 { $place ->
        [deleted] 삭제했어요
       *[trash] 휴지통으로 옮겼어요
    }.
core-library-finished-removed =
    완결 작품 { $count }개를 { $place ->
        [deleted] 삭제했어요
       *[trash] 휴지통으로 옮겼어요
    }.
core-library-watched-removed =
    작품 { $series }개에서 시청한 에피소드 { $count }개를 { $place ->
        [deleted] 삭제했어요
       *[trash] 휴지통으로 옮겼어요
    }.

## library/src/detail.rs

core-library-fix-one-file =
    여기 있는 파일 하나가 { $read ->
        [episode] { $number }화
       *[label] { $label }
    }(으)로 읽혀요. { $what ->
        [film] 영화
       *[episode] 유일한 에피소드
    }인 것 같아요.
core-library-fix-specials =
    스페셜 { $count }개가 누락된 { $episodes ->
       *[other] 에피소드
    } { $numbers }화인 것 같아요.
core-library-fix-numbered-on = { $from }화는 이전 시즌에서 번호가 이어진 것으로, { $to }화인 것 같아요.
core-library-runs-span = { $first }~{ $last }
core-library-runs-list = { $init }, { $last }

## library/src/fixes.rs

core-library-fix-episode-range = 에피소드 번호는 0~10000 사이여야 해요
core-library-fix-season-range = 시즌 번호는 200 이하여야 해요
core-library-fix-extra-range = 번외 번호는 10000 이하여야 해요
core-library-file-elsewhere-not-fixed = 파일 { $id }은(는) 다른 작품의 파일이라 아무것도 수정하지 않았어요
core-library-fix-undone =
    { $count ->
       *[other] { $title } 파일 { $count }개를 다시 이름으로 읽어요.
    }
core-library-fixed =
    { $count ->
       *[other] { $title } 파일 { $count }개를 읽는 방식을 수정했어요.
    }

## library/src/reads.rs

core-library-hid-one = { $title }을(를) 숨겼어요.
core-library-back-one = { $title }이(가) 다시 보여요.
core-library-hid-many = 작품 { $count }개를 숨겼어요.
core-library-back-many = 작품 { $count }개가 다시 보여요.
core-library-episode-count-set = 이제 { $title }은(는) 에피소드 { $count }개예요.
core-library-episode-count-cleared = { $title }의 에피소드 수를 지웠어요.

## library/src/schedule.rs

core-library-schedule-cleared = { $title }의 방영 일정 메모를 지웠어요.
core-library-schedule-saved = { $title }의 방영 일정 메모 { $count }개를 저장했어요.

## library/src/scan.rs

core-library-source-not-absolute = 소스 경로는 절대 경로여야 해요
core-library-source-exists = 이미 추가된 소스예요
core-library-source-nested = 기존 소스 안에 있는 폴더예요
core-library-source-added = { $folder } 폴더를 라이브러리에 추가했어요.
core-library-source-removed = { $folder } 폴더를 라이브러리에서 제거했어요.
core-library-source-removed-with-shows =
    { $count ->
       *[other] { $folder } 폴더와 그 안의 작품 { $count }개를 라이브러리에서 제거했어요.
    }
core-library-series-forgotten = { $name }을(를) 라이브러리에서 제거했어요.
core-library-source-back = { $folder } 폴더가 다시 연결됐어요.
core-library-source-unavailable = { $folder } 폴더를 사용할 수 없어요. 드라이브가 연결돼 있는지 확인하세요.
core-library-source-unreadable = { $folder } 폴더를 읽지 못했어요. 드라이브가 연결돼 있는지 확인하세요.
core-library-source-looks-empty = { $folder } 폴더가 비어 있는 것 같아서 작품은 그대로 두었어요.
core-library-source-reconcile-failed = { $folder }의 작품을 업데이트하지 못했어요.
core-library-scan-done =
    폴더를 확인했어요: { $added ->
        [0] {""}
       *[other] 새 작품 { $added }개
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {", "}
        }변경된 작품 { $changed }개
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {", "}
        }누락 { $removed }개
    }.

## library/src/report.rs

core-library-agent-no-answer = { $span } 안에 응답 없음
core-library-agent-stopped-code = 코드 { $code }(으)로 멈췄어요
core-library-agent-stopped = 멈췄어요
core-library-agent-answered-nothing = 아무 출력도 돌려주지 않았어요
core-library-span-minutes =
    { $count ->
       *[other] { $count }분
    }
core-library-span-seconds =
    { $count ->
       *[other] { $count }초
    }
core-library-report-no-fix = 이 파일들에는 보고할 수정 사항이 없어요
core-library-agent-no-triage = { $agent }이(가) 분류 결과를 쓰지 않았어요: { $why }
core-transfer-progress-sources = 소스
core-transfer-progress-series = 작품
core-transfer-progress-history = 기록
core-transfer-progress-accounts = 계정
core-transfer-progress-preferences = 환경설정
core-transfer-progress-done = 완료

## downloads/src/anirss.rs

core-downloads-anirss-no-answer = anirss가 응답하지 않았어요: { $message }
core-downloads-anirss-answered = anirss 응답 HTTP { $status }: { $text }
core-downloads-no-free-port = 빈 포트가 없어요: { $error }
core-downloads-anirss-exited = anirss가 종료됐어요({ $status })
core-downloads-anirss-exited-said = anirss가 종료됐어요({ $status }): { $said }
core-downloads-anirss-slow-start = anirss가 { $seconds }초 안에 시작되지 않았어요
core-downloads-anirss-slow-start-said = anirss가 { $seconds }초 안에 시작되지 않았어요: { $said }
core-downloads-anirss-install-mac = anirss { $version } 이상을 설치하세요: { $command }
core-downloads-anirss-install-linux = anirss { $version } 이상을 설치하세요: AUR의 anirss 패키지 또는 { $command }
core-downloads-anirss-too-old = anirss { $version }은(는) 너무 오래됐어요. Anibeam에는 { $minimum } 이상이 필요해요. { $update }.
core-downloads-anirss-lacks-feature = anirss { $version }은(는) 토렌트 클라이언트별 로그인을 따로 보관할 수 없어서, Anibeam이 클라이언트를 전환할 수 없어요. anirss { $minimum } 이상은 가능해요. { $update }.
core-downloads-anirss-update-mac = 다음 명령으로 업데이트하세요: { $command }
core-downloads-anirss-update-linux = anirss 패키지를 업데이트하거나 다음을 실행하세요: { $command }

## downloads/src/contents.rs

core-downloads-torrent-file-no-answer = 토렌트 파일을 다운로드하지 못했어요: { $message }
core-downloads-torrent-file-answered = 토렌트 파일 응답 HTTP { $status }

## downloads/src/feed.rs

core-downloads-feed-answered = 피드 응답 HTTP { $status }
core-downloads-not-a-feed = 피드가 아니에요: { $error }

## downloads/src/jackett.rs

core-downloads-jackett-missing = Jackett가 설치되지 않았어요: { $program }을(를) 찾지 못했어요. 설치하거나 설정에서 명령을 바꾸세요.
core-downloads-endpoint-start-failed = { $name }을(를) 시작하지 못했어요: { $why }
core-downloads-endpoint-start-timeout = { $name }이(가) 시작됐지만 { $seconds }초 안에 응답하지 않았어요.
core-downloads-command-exited = 명령이 { $status }(으)로 종료됐어요

## downloads/src/lib.rs

core-downloads-anirss-stopped = anirss가 멈췄어요
core-downloads-anirss-missing = { $install }. 설정에서 실행 중인 서버를 지정할 수도 있어요.
core-downloads-anirss-command-missing = { $command }을(를) 찾지 못했어요. { $install }. 설정에서 실행 중인 서버를 지정할 수도 있어요.
core-downloads-anirss-start-failed = { $command }을(를) 시작하지 못했어요: { $error }
core-downloads-anirss-too-old-said = { $tooOld } 응답 내용: { $message }
core-downloads-no-torrent-client = { $address }에서 응답한 qBittorrent가 없고, Anibeam에서 쓸 수 있는 내장 클라이언트도 없어요. qBittorrent를 설치하고 Web UI를 켜거나, qbittorrent-nox를 설치하면 Anibeam이 자동으로 실행해요
core-downloads-own-client-gone = Anibeam 내장 qBittorrent가 없어요(qbittorrent-nox가 설치돼 있지 않아요). Anibeam을 다시 설치하거나 설정에서 직접 쓰는 qBittorrent를 지정하세요
core-downloads-no-port = { $address }에 포트가 없어요
core-downloads-own-client-settings = qBittorrent 설정을 쓸 수 없어요: { $error }
core-downloads-own-client-start-failed = { $command }을(를) 시작하지 못했어요: { $error }
core-downloads-own-client-slow-start = Anibeam의 qBittorrent가 시작되지 않았어요
core-downloads-own-client-slow-start-said = Anibeam의 qBittorrent가 시작되지 않았어요: { $said }
core-downloads-not-ready = 다운로드를 사용할 준비가 안 됐어요. 설정의 다운로드를 확인하세요.
core-downloads-ready = 다운로드를 사용할 준비가 됐어요.
core-downloads-helper-sources =
    { $count ->
       *[other] 이제 다운로드 검색을 Anibeam이 직접 해요. 검색 도우미 설정에 있는 소스 { $count }개는 읽지 않아요. 설정의 다운로드에서 추가하거나, 그곳에서 검색 도우미로 되돌리세요.
    }
core-downloads-client-connected = 다운로드 앱이 연결됐어요.
core-downloads-client-not-connected = 다운로드 앱이 연결되지 않았어요. 설정에서 주소를 확인하세요.
core-downloads-not-on-demand = { $name }은(는) 필요할 때 시작하는 소스가 아니에요
core-downloads-source-running = 다운로드 소스가 실행 중이에요.
core-downloads-source-not-running = { $name }이(가) 실행 중이 아니에요. 소스에서 켜고 확인을 눌러 시작하세요.
core-downloads-empty-query = 검색어를 입력하세요
core-downloads-first-endpoint = 첫 번째 소스
core-downloads-search-starting = 검색 시작 중
core-downloads-search-starting-endpoint = { $endpoint } 시작 중
core-downloads-search-searching-endpoint = { $endpoint } 검색 중
core-downloads-search-reading = 결과 읽는 중
core-downloads-subscribed = { $name } 구독을 시작했어요.
core-downloads-client-no-rule = qBittorrent에 { $name } 규칙이 없어요
core-downloads-trying-search = "{ $search }" 시도 중
core-downloads-nothing-selected = 다운로드할 항목을 선택하지 않았어요
core-downloads-added =
    { $count ->
       *[other] 다운로드 { $count }개를 추가했어요.
    }
core-downloads-movie-link = 영화 링크는 magnet이나 http(s) 주소여야 해요
core-downloads-added-movie = 다운로드에 영화를 추가했어요.
core-downloads-library-folder = { $folder }은(는) { $name }의 라이브러리 폴더이고, 이 구독이 받지 않은 에피소드가 있어요. 파일은 두고 구독만 제거하거나, { $name } 페이지에서 작품을 제거하세요
core-downloads-no-torrents-named = 지정한 토렌트가 없어요
core-downloads-paused =
    { $count ->
       *[other] 다운로드 { $count }개를 일시정지했어요.
    }
core-downloads-resumed =
    { $count ->
       *[other] 다운로드 { $count }개를 재개했어요.
    }
core-downloads-deleted-with-files =
    { $count ->
       *[other] 다운로드 { $count }개와 파일을 삭제했어요.
    }
core-downloads-removed-from-list =
    { $count ->
       *[other] 목록에서 다운로드 { $count }개를 제거했어요.
    }
core-downloads-nothing-to-remove = 제거할 항목이 없어요
core-downloads-removed-show = 다운로드에서 { $title }을(를) 제거했어요.
core-downloads-removed-the-show = 다운로드에서 작품을 제거했어요.
core-downloads-deleted-still-listed = 삭제한 파일이 다운로드 앱 목록에 아직 남아 있어요.
core-downloads-feed-url = 피드 URL은 http:// 또는 https://로 시작해야 해요
core-downloads-rule-needs-name = 규칙 이름을 비워 둘 수 없어요
core-downloads-no-rule = { $name } 규칙이 없어요
core-downloads-feed-no-search = 이 피드에는 줄일 검색어가 없어요
core-downloads-no-repair = { $name }은(는) 복구할 필요가 없어요.
core-downloads-repaired = { $name } 구독을 복구했어요.
core-downloads-repair-failed = 더 짧은 검색어로도 결과가 없어요. "{ $search }"도 결과가 없었어요
core-downloads-no-saved-client = 저장된 토렌트 클라이언트 { $id }이(가) 없어요
core-downloads-client-switched = 이제 다운로드에 다른 다운로드 앱을 사용해요.
core-downloads-host-not-running = 호스트에서 실행 중이 아니에요
core-downloads-host-late = 제시간에 응답하지 않았어요

## downloads/src/native.rs

core-downloads-native-empty-query = 검색어가 비어 있어요
core-downloads-native-no-results = { $query } 검색 결과가 없어요
core-downloads-native-feed-taken = { $name } 피드가 이미 다른 주소를 가리키고 있어요
core-downloads-native-no-items = 항목이 없어요
core-downloads-native-bad-link = 링크는 magnet: 또는 http(s) 주소여야 해요. 받은 값: { $link }
core-downloads-native-no-links = 링크가 없어요
core-downloads-native-no-names = 구독 이름이 없어요
core-downloads-native-files-elsewhere = 토렌트 클라이언트가 파일을 다른 기기에 보관해서, 토렌트와 함께만 삭제할 수 있어요
core-downloads-native-unknown-feeds = 알 수 없는 피드: { $names }
core-downloads-native-outside-folders = 다운로드 폴더 밖의 파일은 제거하지 않아요: { $paths }
core-downloads-refuse-home-folder = { $path }은(는) 홈 폴더이거나 홈 폴더를 포함해서 아무것도 제거하지 않았어요
core-downloads-refuse-library-folder = { $path }은(는) 라이브러리 폴더이거나 라이브러리 폴더를 포함해서 아무것도 제거하지 않았어요

## downloads/src/paths.rs

core-downloads-no-default-folder = qBittorrent에 기본 저장 폴더가 설정돼 있지 않아요. qBittorrent에서 설정하세요
# $folder is the folder given
core-downloads-save-folder-not-absolute = { $folder }은(는) 폴더의 전체 경로가 아니에요
# $folder is the folder given
core-downloads-save-folder-unusable = { $folder }은(는) 다운로드에 쓸 수 없어요. 홈 폴더가 아닌 기존 폴더를 고르세요
core-downloads-name-empty = 이름은 비워 둘 수 없어요
core-downloads-name-slash = 이름에 / 또는 \를 쓸 수 없어요
core-downloads-name-not-folder = { $name }은(는) 폴더 이름이 아니에요

## downloads/src/prune.rs

core-downloads-pruned = { $title } 구독을 해지했어요(방영 종료).
core-downloads-prune-failed = { $title } 구독을 해지하지 못했어요. 설정의 다운로드를 확인하세요.

## library/src/trash.rs

core-downloads-refuse-not-absolute = { $path }은(는) 일반 절대 경로가 아니라서 아무것도 제거하지 않았어요
core-downloads-refuse-root = { $path }은(는) 파일 시스템 루트라서 아무것도 제거하지 않았어요
core-downloads-refuse-outside-library = { $path }은(는) 라이브러리 폴더 안에 있지 않아서 아무것도 제거하지 않았어요
core-downloads-refuse-save-folder = { $path }은(는) 다운로드한 파일이 아니라 토렌트 저장 폴더라서 아무것도 제거하지 않았어요

## downloads/src/watch.rs

core-downloads-episode-missed = { $title } { $episode }화가 방영 후 3일이 지나도 나오지 않았어요. 피드에서 계속 확인해요.

## downloads/src/settings.rs

core-downloads-settings-command-empty = anirss 명령은 비워 둘 수 없어요
core-downloads-settings-anirss-address = anirss 주소는 http:// 또는 https://로 시작해야 해요
core-downloads-settings-client-address = { $client } 주소는 http:// 또는 https://로 시작해야 해요
core-downloads-settings-bad-id = { $id }은(는) 클라이언트 ID가 아니에요. 영문자, 숫자, -, _만 쓰세요
core-downloads-settings-same-id = 두 클라이언트의 ID가 { $id }(으)로 같아요
core-downloads-settings-saved-address = { $name }의 주소는 http:// 또는 https://로 시작해야 해요
core-downloads-settings-same-name = 두 클라이언트의 이름이 { $name }(으)로 같아요. 각각 다른 이름을 지정하세요

## downloads/src/qbt.rs

core-downloads-qbt-no-answer = { $address }의 qBittorrent가 응답하지 않았어요: { $message }
core-downloads-qbt-password-rejected = qBittorrent가 비밀번호를 거부했어요. 설정에서 고치세요
core-downloads-qbt-banned = 로그인에 실패해서 qBittorrent가 이 주소를 차단했어요. 기다리거나 qBittorrent를 다시 시작하세요
core-downloads-qbt-login-rejected = qBittorrent가 사용자 이름이나 비밀번호를 거부했어요
core-downloads-qbt-login-failed = { $address }의 qBittorrent 로그인 응답 HTTP { $status }. 다음 요청에서 다시 시도해요
core-downloads-qbt-not-web-ui = { $address }이(가) HTTP { $status }(으)로 응답했지만 qBittorrent Web UI가 아닌 것 같아요. 설정의 주소를 확인하고 qBittorrent에서 Web UI가 켜져 있는지 확인하세요
core-downloads-qbt-not-added = qBittorrent가 토렌트를 추가하지 않았어요: { $said }
core-downloads-qbt-some-not-added = qBittorrent가 토렌트 { $count }개를 추가하지 못했어요: { $said }
core-downloads-qbt-answered = qBittorrent 응답 HTTP { $status }: { $line }

## downloads/src/replace.rs

core-downloads-replace-bad-item = 항목 { $item }은(는) 릴리스 { $count }개 중 하나가 아니에요
core-downloads-not-replaced = { $episode }화를 교체하지 않았어요.
core-downloads-replaced = { $episode }화를 새 다운로드로 교체했어요.
core-downloads-no-video-moved = { $name }에는 동영상이 없어서 Anibeam에서 재생할 수 없어요. 지금은 { $folder }에 있어요.
core-downloads-no-video = { $name }에는 동영상이 없어서 Anibeam에서 재생할 수 없어요.

## downloads/src/rolling.rs

core-downloads-no-roll = 미리 받기 { $id }이(가) 없어요
core-downloads-rolled = { $title }: 이제 { $total }화 중 { $from }~{ $to }화를 보관해요.
core-downloads-roll-ahead = 최소 한 화는 앞서 받아 두세요
core-downloads-roll-nothing = 가져올 항목이 없어요

## releases/src/endpoints.rs

core-releases-json-needs-repo = JSON 검색 소스는 필드를 매핑한 정의가 있는 소스 목록에서만 와야 해요
core-releases-endpoint-no-name = `name`이 없어요
core-releases-endpoint-bad-name = { $name }: 영문자, 숫자, '.', '_', '-'만 쓰세요
core-releases-endpoint-json = { $name }: { $why }
core-releases-endpoint-unknown-kind = { $name }: 알 수 없는 종류 { $kind }(사용 가능: { $valid })
core-releases-endpoint-no-url = { $name }: `url`이 없어요
core-releases-endpoint-not-http = { $name }: 주소는 http:// 또는 https://로 시작해야 해요
core-releases-endpoint-needs-query = { $name }: { $kind } URL에는 {"{"}query{"}"} 자리표시자가 있어야 해요
core-releases-endpoint-no-query = { $name }: { $kind } URL은 검색어를 자동으로 붙이므로 {"{"}query{"}"}를 넣지 않아요

## releases/src/fetch.rs

core-releases-indexer-error = 인덱서가 오류 { $code }(으)로 응답했어요
core-releases-bad-feed = 잘못된 피드: { $error }
core-releases-feed-answered = 피드 응답 HTTP { $status }

## releases/src/json_feed.rs

core-releases-not-feed-json = 피드의 JSON이 아니에요: { $error }

## releases/src/json.rs

core-releases-not-json = JSON이 아니에요: { $error }
core-releases-search-error-said = 검색이 오류를 반환했어요: { $message }
core-releases-search-error = 검색이 오류를 반환했어요
core-releases-no-results-field = 응답의 { $path }에 결과가 없어요
core-releases-results-not-list = 응답의 결과가 목록이 아니에요
core-releases-search-answered = 검색 응답 HTTP { $status }

## releases/src/native.rs

core-releases-no-endpoints = 아직 다운로드 소스가 없어요. 먼저 소스 목록을 추가하거나 소스를 직접 추가하세요
core-releases-no-source-named = { $name } 소스가 없어요
core-releases-empty-query = 검색어를 입력하세요
core-releases-cannot-follow = { $name }은(는) 구독할 수 없어요. 검색은 지원하지만 새 에피소드를 확인할 피드가 없어요
core-releases-endpoint-gone = { $name }을(를) 더 이상 쓸 수 없어요: 제거됐거나, 꺼졌거나, 키가 없어요

## releases/src/quirks.rs

core-releases-quirks-variant = one_variant { $word }은(는) 짧은 한 단어여야 해요
core-releases-quirks-pattern-count = show_feed는 페이지 패턴을 1~{ $most }개 받아요
core-releases-quirks-pattern-long = show_feed 페이지 패턴이 너무 길어요
core-releases-quirks-pattern-bad = show_feed 페이지 패턴: { $error }
core-releases-quirks-pattern-slug = show_feed 페이지 패턴에는 slug라는 그룹이 있어야 해요
core-releases-quirks-url-https = show_feed url은 https여야 해요
core-releases-quirks-url-host = show_feed url은 엔드포인트와 같은 호스트에 있어야 해요
core-releases-quirks-url-slug = show_feed url에는 {"{"}slug{"}"}가 한 번만 있고 다른 자리표시자는 없어야 해요

## releases/src/refinement.rs

core-releases-refine-no-tokens = op 'tokens'에는 비어 있지 않은 `tokens` 목록이 필요해요

## releases/src/grid.rs

core-releases-group-batch = { $group } 일괄
core-releases-episodes-phrase =
    { $count ->
       *[other] { $list }화
    }
core-releases-episode-range = { $from }~{ $to }

## Names that stay the same in every language

-app-name = Anibeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = 이름은 1~{ $max }자여야 하고 제어 문자는 쓸 수 없어요
core-prefs-room-relay = 릴레이 주소는 https:// 또는 http://로 시작해야 해요
core-room-service = 함께 보기
core-room-not-found = 그 코드의 방이 없어요. 닫혔을 수 있어요.
core-room-full = 방이 꽉 찼어요.
core-room-proto = 그 서버는 다른 버전의 함께 보기를 써요. 참여하려면 { -app-name }을(를) 업데이트하세요.
core-room-needs-episode = 그 서버는 방을 만들기 전에 에피소드가 있어야 해요. 플레이어에서 시작하거나 서버를 업데이트하세요.
core-room-unreachable = 방 서버에 연결하지 못했어요: { $message }
core-room-refused = 방 서버가 요청을 거부했어요: { $message }
core-room-not-in = 방에 있지 않아요.
core-room-no-name = 방에서 다른 사람에게 보일 이름을 입력하세요
core-room-bad-code = 방 코드는 영문자나 숫자 6자여야 해요
core-room-created = 함께 보기 방을 만들었어요. 코드: { $code }.
core-room-joined = 함께 보기 방 { $code }에 참여했어요.
core-room-left = 함께 보기 방에서 나왔어요.
core-room-member-joined = { $name }님이 방에 들어왔어요.
core-room-member-left = { $name }님이 방에서 나갔어요.
core-room-closed = 함께 보기 방이 닫혔어요.
core-room-lost = 함께 보기 방과 연결이 끊겼어요. 다시 연결하는 중이에요.
core-room-back = 함께 보기 방에 다시 연결됐어요.
core-room-media-asked = 호스트에게 에피소드를 요청했어요.
core-room-media-cannot = 호스트가 지금은 이 에피소드를 공유할 수 없어요.
core-room-media-sent = { $name }님에게 에피소드 토렌트를 보냈어요.
core-room-media-sent-name = { $name }님에게 에피소드 릴리스 이름을 보냈어요.
core-room-media-nothing = 호스트에게 공유할 에피소드 사본이 없어요.
core-room-media-no-answer = 호스트의 앱이 응답하지 않았어요. 잠시 후 다시 시도하세요.
core-room-media-torrent = 호스트의 토렌트로 에피소드를 내려받고 있어요.
core-room-media-searching = 내 소스에서 호스트의 릴리스를 찾고 있어요.
core-room-media-not-found = 내 소스 중 호스트의 릴리스가 있는 곳이 없어요.
core-room-media-no-sources = 모든 소스가 꺼져 있어요. 검색하려면 하나를 켜세요.
core-room-media-no-start = 다운로드를 시작할 수 없었어요: { $message }
core-room-media-client = 토렌트 클라이언트가 거부했어요
core-room-media-lost = 토렌트 클라이언트가 에피소드 다운로드를 놓쳤어요.
core-room-media-stalled = 다운로드가 10분 동안 멈춰 있어요. 나중에 다시 시도하세요.
core-room-media-not-in-library = 에피소드 다운로드는 끝났지만 라이브러리에 나타나지 않았어요. 다운로드가 라이브러리 폴더에 저장되는지 확인하세요.
core-room-media-here = 에피소드가 이제 라이브러리에 있어요.
core-room-media-rest-missing = 시리즈의 나머지를 가져오지 못했어요. 이 에피소드만 다운로드하고 있어요.
