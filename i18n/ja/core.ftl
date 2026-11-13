# ja translation of en/core.ftl. An ID missing here is shown in English.

## The tailnet host

# A person on the tailnet whom Tailscale gives neither a name nor a login.
tailnet-person-no-name = Tailscaleに名前がありません

## Counts

count-episodes = { $count }話
count-shows = { $count }作品

## Actions every screen uses

action-cancel = キャンセル
action-close = 閉じる
action-try-again = 再試行
action-remove = 削除
action-delete = 削除
action-open = 開く
action-save = 保存
action-apply = 適用
action-clear = クリア
action-refresh = 更新
action-play = 再生
action-connect = 接続
action-start = 開始
action-back = 戻る
action-see-all = すべて表示
action-open-series = 作品を開く
action-open-settings = 設定を開く

## The lists a tracker keeps

status-watching = 視聴中
status-completed = 視聴完了
status-paused = 一時停止
status-on-hold = 保留
status-dropped = 中止
status-planning = 視聴予定
status-rewatching = 再視聴中

## Episodes

episode-number = 第{ $number }話

## AniList's words for a show: its format, its airing status, how two shows relate

anilist-format-tv = TV
anilist-format-tv-short = TVショート
anilist-format-film = 映画
anilist-format-special = スペシャル
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = 音楽
anilist-status-finished = 放送終了
anilist-status-airing = 放送中
anilist-status-not-yet-aired = 未放送
anilist-status-cancelled = 放送中止
anilist-status-hiatus = 休止中
anilist-relation-adaptation = 翻案
anilist-relation-prequel = 前作
anilist-relation-sequel = 続編
anilist-relation-parent = 本編
anilist-relation-side-story = 外伝
anilist-relation-character = 共通キャラ
anilist-relation-summary = 総集編
anilist-relation-alternative = 別バージョン
anilist-relation-spin-off = スピンオフ
anilist-relation-other = その他
anilist-relation-source = 原作
anilist-relation-compilation = 編集版
anilist-relation-contains = 収録

## The activity log: a problem whose place had no words of its own

event-problem-library = ライブラリで問題が発生しました。詳細は開発者向けオプションにあります。
event-problem-metadata = 作品情報で問題が発生しました。詳細は開発者向けオプションにあります。
event-problem-trackers = リストで問題が発生しました。詳細は開発者向けオプションにあります。
event-problem-franchise = 関連作品で問題が発生しました。詳細は開発者向けオプションにあります。
event-problem-playback = 再生で問題が発生しました。詳細は開発者向けオプションにあります。
event-problem-store = データの保存で問題が発生しました。詳細は開発者向けオプションにあります。
event-problem-system = AniBeamで問題が発生しました。詳細は開発者向けオプションにあります。
event-problem-downloads = ダウンロードで問題が発生しました。詳細は開発者向けオプションにあります。

## Errors (contract::CoreError)

core-error-not-found-source = フォルダ{ $id }が見つかりません。
core-error-not-found-series = 作品{ $id }が見つかりません。
core-error-not-found-file = ファイル{ $id }が見つかりません。
core-error-not-found-session = 再生セッション{ $id }が見つかりません。
core-error-not-found-job = ジョブ{ $id }が見つかりません。
# $field is the setting's own name; $message is already in the reader's language.
core-error-invalid = { $field }が無効です: { $message }。
# The same, for a $message that already ends in its own full stop, question mark or ellipsis.
core-error-invalid-ended = { $field }が無効です: { $message }
core-error-unavailable = このフォルダは利用できません: { $path }
core-error-not-connected = { $tracker }は連携されていません。
core-error-refused-hidden = 実行できません: この作品は非表示です。
core-error-refused-no-match = 実行できません: この作品はまだ照合されていません。作品情報で照合してください。
core-error-refused-not-newer = 実行できません: リストはすでにこのエピソードより先に進んでいます。
core-error-refused-extra = 実行できません: これは特典映像です。
core-error-refused-unmatched = 実行できません: このファイルはエピソードと照合されていません。
core-error-refused-on-disk = 実行できません: このファイルはディスク上にしかありません。
core-error-provider = { $provider }から応答がありません: { $message }
core-error-io = ファイルを読み書きできませんでした: { $message }
core-error-storage = ライブラリを保存できませんでした: { $message }
core-error-keyring = システムのパスワード保管庫に接続できませんでした: { $message }
core-error-unsupported = { $what }にはまだ対応していません。
core-error-version = このエクスポートは新しいAniBeamのものです（形式 { $found }）。読み込むにはAniBeamを更新してください。
core-error-internal = AniBeamで内部エラーが発生しました: { $message }
core-error-service = { $service }: { $message }

# What the core says to a person outside downloads, releases, trackers, metadata, the
# library, franchise, images and transfer: jobs, playback, casting, the host, the phone,
# endpoint repos, updates and the rest.

## core/src/repos/mod.rs

core-repos-paste-address = リポジトリのアドレスを貼り付けてください
core-repos-must-be-https = アドレスはhttpsにしてください
core-repos-unreachable = リポジトリに接続できませんでした: { $message }
core-repos-not-found = そのアドレスには何もありません（非公開のGitHubリポジトリも404を返します）
core-repos-http-status = リポジトリがHTTP { $status }を返しました
core-repos-unreadable = { $path }を読み込めませんでした: { $error }
core-repos-no-such-endpoint = そのIDのエンドポイントはインストールされていません
core-repos-key-changed = リポジトリが、信頼した鍵とは別の鍵で署名されています。新しい鍵を受け入れるには、削除してからもう一度追加してください。
core-repos-stale = リポジトリが古いインデックスを返しました（シリアル{ $kept }の後に{ $serial }）。新しい方を残します
core-repos-key-not-shown = リポジトリの鍵が、表示された鍵と違います。もう一度追加して、フィンガープリントを比べてください。
core-repos-repo-added = ダウンロードソースの一覧を追加しました。
core-repos-repo-removed = ダウンロードソースの一覧を削除しました。
# $key is the field's key in quotes.
core-repos-no-field = { $name }に項目{ $key }はありません
core-repos-needs-field = { $name }には{ $label }が必要です
core-repos-host-only = { $name }: { $label }はホストかホスト:ポートだけにしてください。例: 192.0.2.10:9696
core-repos-not-added = そのリポジトリは追加されていません
core-repos-not-offered = { $repo }は{ $id }を提供していません
core-repos-already-installed = { $name }はすでにインストールされています
core-repos-taken-by-you = { $id }という名前のエンドポイントがすでにインストールされています（自分で追加したもの）。先に削除してください
core-repos-taken-by-repo = { $id }という名前のエンドポイントがすでにインストールされています（別のリポジトリから）。先に削除してください
core-repos-source-added = ダウンロードソースを追加しました。
core-repos-source-updated = ダウンロードソースを更新しました。
core-repos-source-removed = ダウンロードソースを削除しました。
core-repos-source-changed = ダウンロードソースを変更しました。
core-repos-give-name = 名前を付けてください
core-repos-put-query = 検索語が入る位置のアドレスに{"{"}query{"}"}を入れてください
core-repos-name-taken = { $id }という名前のエンドポイントがすでにインストールされています
core-repos-only-by-hand = 編集できるのは手動で追加したソースだけです
# $key is a placeholder's name; it stays in braces in the address.
core-repos-keep-key = アドレスの{"{"}{ $key }{"}"}は残し、値は「{ $label }を変更」で変えてください。または、サイトを削除してそれなしで追加し直してください
core-repos-note-kind = anirssは{ $kind }形式を検索できないため、デスクトップでは除外します。スマホでは検索します
core-repos-note-by-hand = anirssには手動で書かれた独自の{ $id }があり、デスクトップはそちらを検索します
core-repos-note-theirs = このanirssはご自身で動かしているもので、デスクトップがその設定を変えることはありません。デスクトップで検索するには、そのanirssの設定に{ $id }を追加してください
core-repos-note-sandbox = この実行ではanirssの設定に書き込まないため、デスクトップはここで{ $id }を検索しません
core-repos-note-old = anirss { $version }は{ -app-name }からエンドポイントを受け取れません。デスクトップで検索するには{ $min }以降に更新してください

## core/src/repos/index.rs

core-repos-key-not-ed25519 = インデックスの鍵がed25519鍵ではありません
core-repos-key-not-base64 = インデックスの鍵が32バイトのbase64ではありません
core-repos-key-invalid = インデックスの鍵が有効なed25519鍵ではありません
core-repos-too-large = リポジトリとしてはファイルが大きすぎます
core-repos-not-a-repo = これは{ -app-name }のエンドポイントリポジトリではありません
core-repos-format-version = このリポジトリの形式はバージョン{ $found }です。この{ -app-name }が読めるのはバージョン{ $reads }です。{ -app-name }を更新してください。
core-repos-payload-not-base64 = リポジトリのペイロードがbase64ではありません
core-repos-signature-malformed = リポジトリの署名が不正です
core-repos-index-unreadable = リポジトリのインデックスを読めません: { $error }
core-repos-signature-mismatch = リポジトリの署名が内容と一致しません
# $id is the source's id; the rest of these checks name a source by its id first.
core-repos-check-scheme = { $id }: アドレスはhttps://で始めてください
core-repos-check-scheme-http = { $id }: アドレスはhttp://かhttps://で始めてください
# $key is the field's key in quotes.
core-repos-check-auth-key = { $id }: 認証キー{ $key }が不正です
core-repos-check-host-torznab = { $id }: ホストを{"{"}{ $key }{"}"}と書けるのはtorznabのアドレスだけです
core-repos-check-host-placeholder = { $id }: ホストにプレースホルダーがあります

## core/src/playback/tracks.rs

core-playback-track-number = トラック{ $number }
# A sidecar subtitle file beside the video; $label is the rest of its name.
core-playback-track-file = { $label }（ファイル）
# A language and its region or script: "Portuguese (Brazil)", "Chinese (Simplified)".
core-playback-lang-region = { $name }（{ $region }）
core-playback-lang-english = 英語
core-playback-lang-japanese = 日本語
core-playback-lang-spanish = スペイン語
core-playback-lang-portuguese = ポルトガル語
core-playback-lang-french = フランス語
core-playback-lang-german = ドイツ語
core-playback-lang-italian = イタリア語
core-playback-lang-russian = ロシア語
core-playback-lang-chinese = 中国語
core-playback-lang-korean = 韓国語
core-playback-lang-arabic = アラビア語
core-playback-lang-hindi = ヒンディー語
core-playback-lang-indonesian = インドネシア語
core-playback-lang-malay = マレー語
core-playback-lang-thai = タイ語
core-playback-lang-vietnamese = ベトナム語
core-playback-lang-polish = ポーランド語
core-playback-lang-turkish = トルコ語
core-playback-lang-dutch = オランダ語
core-playback-lang-swedish = スウェーデン語
core-playback-lang-norwegian = ノルウェー語
core-playback-lang-danish = デンマーク語
core-playback-lang-finnish = フィンランド語
core-playback-lang-hebrew = ヘブライ語
core-playback-lang-ukrainian = ウクライナ語
core-playback-lang-czech = チェコ語
core-playback-lang-hungarian = ハンガリー語
core-playback-lang-greek = ギリシャ語
core-playback-lang-romanian = ルーマニア語
core-playback-lang-filipino = フィリピン語
core-playback-region-brazil = ブラジル
core-playback-region-portugal = ポルトガル
core-playback-region-latin-america = 中南米
core-playback-region-spain = スペイン
core-playback-region-simplified = 簡体字
core-playback-region-traditional = 繁体字

## core/src/contract/enums.rs

# What a job is doing, for a status line while it runs. .stopped is the line when it
# ended in a problem.
core-jobs-scan = フォルダを確認中
    .stopped = フォルダの確認が問題で停止しました。
core-jobs-auto-match = 作品を照合中
    .stopped = 作品の照合が問題で停止しました。
core-jobs-search = 作品を検索中
    .stopped = 作品の検索が問題で停止しました。
core-jobs-resolve-link = リンクを読み込み中
    .stopped = リンクの読み込みが問題で停止しました。
core-jobs-apply-match = 作品を照合中
    .stopped = 作品の照合が問題で停止しました。
core-jobs-refresh = 作品を更新中
    .stopped = 作品の更新が問題で停止しました。
core-jobs-refresh-all = 作品情報を更新中
    .stopped = 作品情報の更新が問題で停止しました。
core-jobs-refresh-airing = 放送中の作品を確認中
    .stopped = 放送中の作品の確認が問題で停止しました。
core-jobs-refresh-imdb = 評価を更新中
    .stopped = 評価の更新が問題で停止しました。
core-jobs-clear-images = 画像を消去中
    .stopped = 画像の消去が問題で停止しました。
core-jobs-fill-images = 画像を取得中
    .stopped = 画像の取得が問題で停止しました。
core-jobs-connect-tracker = リストを連携中
    .stopped = リストの連携が問題で停止しました。
core-jobs-mark = エピソードを視聴済みにしています
    .stopped = エピソードを視聴済みにする処理が問題で停止しました。
core-jobs-set-progress = 進捗を設定中
    .stopped = 進捗の設定が問題で停止しました。
core-jobs-score = スコアを保存中
    .stopped = スコアの保存が問題で停止しました。
core-jobs-refresh-progress = リストの進捗を読み込み中
    .stopped = リストの進捗の読み込みが問題で停止しました。
core-jobs-refresh-watching = 視聴中リストを読み込み中
    .stopped = 視聴中リストの読み込みが問題で停止しました。
core-jobs-refresh-airing-now = 今期の作品を確認中
    .stopped = 今期の作品の確認が問題で停止しました。
core-jobs-refresh-recommended = おすすめを探しています
    .stopped = おすすめの検索が問題で停止しました。
core-jobs-read-show-page = 作品を読み込み中
    .stopped = 作品の読み込みが問題で停止しました。
core-jobs-on-hold = 動きのない作品を保留に移動中
    .stopped = 保留への移動が問題で停止しました。
core-jobs-set-status = リストのステータスを変更中
    .stopped = リストのステータスの変更が問題で停止しました。
core-jobs-move-lists = リストをトラッカーに移動中
    .stopped = リストのトラッカーへの移動が問題で停止しました。
core-jobs-notes = メモを保存中
    .stopped = メモの保存が問題で停止しました。
core-jobs-crawl = 関連作品を探しています
    .stopped = 関連作品の検索が問題で停止しました。
core-jobs-skip-windows = OPとEDを探しています
    .stopped = OPとEDの検索が問題で停止しました。
core-jobs-export = ライブラリをファイルに保存中
    .stopped = ライブラリのファイルへの保存が問題で停止しました。
core-jobs-import = ファイルからライブラリを読み込み中
    .stopped = ファイルからのライブラリの読み込みが問題で停止しました。
core-jobs-subscriptions = 購読を確認中
    .stopped = 購読の確認が問題で停止しました。
core-jobs-connect-downloads = ダウンロードアプリに接続中
    .stopped = ダウンロードアプリへの接続が問題で停止しました。
core-jobs-search-releases = エピソードを探しています
    .stopped = エピソードの検索が問題で停止しました。
core-jobs-refine-releases = エピソードを絞り込み中
    .stopped = エピソードの絞り込みが問題で停止しました。
core-jobs-subscribe = 作品を購読中
    .stopped = 作品の購読が問題で停止しました。
core-jobs-download = ダウンロードを追加中
    .stopped = ダウンロードの追加が問題で停止しました。
core-jobs-remove-subscription = 購読を削除中
    .stopped = 購読の削除が問題で停止しました。
core-jobs-control-torrents = ダウンロードを変更中
    .stopped = ダウンロードの変更が問題で停止しました。
core-jobs-test-torrent-client = ダウンロードアプリをテスト中
    .stopped = ダウンロードアプリのテストが問題で停止しました。
core-jobs-remove-show = 作品を削除中
    .stopped = 作品の削除が問題で停止しました。
core-jobs-repair-subscription = 購読を修復中
    .stopped = 購読の修復が問題で停止しました。
core-jobs-check-endpoints = ダウンロードソースを確認中
    .stopped = ダウンロードソースの確認が問題で停止しました。
core-jobs-start-endpoint = ダウンロードソースを起動中
    .stopped = ダウンロードソースの起動が問題で停止しました。
core-jobs-schedule-agent = コーディングエージェントにスケジュールを問い合わせ中
    .stopped = コーディングエージェントへのスケジュールの問い合わせが問題で停止しました。

## core/src/airing_now.rs

core-airing-now-now = 放送中
core-airing-now-winter = { $year }年冬
core-airing-now-spring = { $year }年春
core-airing-now-summer = { $year }年夏
core-airing-now-fall = { $year }年秋
core-airing-now-planned = { $site }の視聴予定リストに{ $title }を追加しました。
core-airing-now-planned-both = { $site }と{ $other }の視聴予定リストに{ $title }を追加しました。
# $status is the list the show is on: watching, repeating, planning, completed, paused or dropped.
core-airing-now-already-on =
    { $title }はすでに{ $site }のリストに「{ $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }」で入っています。
core-airing-now-plan-failed = { $title }を視聴予定リストに追加できませんでした。あとでもう一度お試しください。

## core/src/aired.rs

core-aired-episode-out = { $title }の第{ $number }話が公開されました。

## core/src/cast/mod.rs

core-cast-connecting = { $device }にキャスト中です。
core-cast-ended = { $device }へのキャストが終わりました。
core-cast-stopped = { $device }へのキャストを停止しました。
core-cast-failed = { $device }へのキャストに失敗しました。同じネットワークにあるか確認してください。
core-cast-not-built = このビルドにはキャスト機能がありません
core-cast-serve-failed = エピソードを配信できませんでした: { $error }
core-cast-device-gone = そのデバイスはネットワーク上にもうありません
core-cast-not-a-playlist = プレイリストのアドレスではありません
core-cast-nothing = キャスト中のものはありません
core-cast-still-starting = デバイスはまだ起動中です
core-cast-no-route = このデバイスからテレビへの経路がありません
core-cast-no-control = テレビが操作用アドレスを返しませんでした
core-cast-not-handed = 相手のパソコンがエピソードを渡しませんでした: { $error }
core-cast-empty-playlist = 相手のパソコンが空のプレイリストを送ってきました
# $error is why the TV refused the converted stream, $first why it refused the file.
core-cast-refused-both = { $error }。元のファイルも受け付けませんでした（{ $first }）
core-cast-chromecast-stopped = Chromecastが停止しました
core-cast-hevc = { $error }。このエピソードはHEVCで、デコードできるのは新しいChromecast（Google TV搭載またはUltra）だけです

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = Chromecastに接続できませんでした: { $error }
core-cast-chromecast-broke = Chromecastとの接続が切れました: { $error }
core-cast-chromecast-no-player = Chromecastがメディアプレーヤーを起動しませんでした
# $reason is the Chromecast's own word for it, or core-cast-no-reason.
core-cast-chromecast-refused-player = Chromecastがメディアプレーヤーの起動を拒否しました（{ $reason }）
core-cast-no-reason = 理由なし
# $kind is the Chromecast's own word for the failure.
core-cast-chromecast-cannot-play = Chromecastはこれを再生できませんでした（{ $kind }）
core-cast-chromecast-error = Chromecastがこれの再生中にエラーで停止しました

## core/src/cast/dlna.rs

core-cast-tv-no-answer = テレビが応答しませんでした: { $error }
# $action is the DLNA command; $code the TV's own error.
core-cast-tv-refused = テレビが{ $action }を拒否しました
core-cast-tv-refused-code = テレビが{ $action }を拒否しました（{ $code }）

## core/src/cast/media.rs

core-cast-no-video = このファイルには映像がありません。
core-cast-hi10p = このエピソードは10ビットH.264（Hi10P）で、テレビやChromecastではデコードできません。キャストするには変換が必要ですが、まだ実装されていません。
core-cast-video-codec = このエピソードの映像（{ $codec }）はキャストできません。
core-cast-audio-codec = このエピソードの音声（{ $codec }）はキャストできません。

## core/src/phone/mod.rs

core-phone-download-started = { $folder }のエピソード{ $count }話のダウンロードを開始しました。
core-phone-subscribed = { $name }を購読しました。
core-phone-rule-stopped = { $name }の新エピソードは確認されていません: ソースがなくなったか、鍵の再入力が必要です。
core-phone-new-episode = { $name }の新しいエピソードをダウンロード中です。
# $name is the source's name in quotes.
core-phone-name-taken = { $name }: 同じ名前のソースがすでにあります
core-phone-nothing-to-download = ダウンロードするものがありません
# $used and $cap are in GB, with one decimal.
core-phone-over-cap = { -app-name }のストレージ上限を超えてしまいます（{ $cap } GB中{ $used } GBが使用済みまたは予定）。エピソードを削除するか、上限を上げてください
core-phone-empty-search = 検索語が空です
core-phone-no-results = { $query }の結果はありません
# Ends "... is not supported yet." (core-error-unsupported).
core-phone-own-engine = 独自のトレントエンジン

## core/src/phone/check.rs

# The connection check: each line is a step for one host ($label), and what came of it.
core-phone-check-connected = 接続しました
core-phone-check-refused = 拒否されました: { $error }
core-phone-check-dropped = 応答なし（破棄）
core-phone-check-finding = { $label }: アドレスを検出中
core-phone-check-failed = 失敗しました: { $error }
core-phone-check-no-answer = 応答なし
core-phone-check-private = { $label }: 非公開で検出中
core-phone-check-skipped-address = スキップ、アドレスです
core-phone-check-skipped-home = スキップ、ホームネットワークの名前です
core-phone-check-no-record = 公開アドレスなし、このネットワークのDNSに任せます
core-phone-check-no-resolver = 応答したリゾルバーはありません
core-phone-check-reaching = { $label }: { $ip }に接続中
core-phone-check-http = HTTP { $status }、{ $bytes }バイト
core-phone-check-setup = 準備

## core/src/playback/session.rs

core-playback-not-saved = { $folder }の進捗は保存されませんでした。先に作品を照合してください。
# $done is a whole percentage.
core-playback-still-downloading = 第{ $number }話はまだダウンロード中です（{ $done }%完了）

## core/src/playback/skip.rs

core-playback-bad-duration = 長さは正の秒数で指定してください

## core/src/agent.rs

# A coding agent looking up a show's schedule; these lines say what it is doing.
core-agent-no-answer = エージェントから回答がありませんでした
core-agent-searching = { $query }をウェブで検索中
core-agent-searching-schedule = 放送スケジュールをウェブで検索中
core-agent-reading = { $url }を読み込み中
core-agent-reading-page = ページを読み込み中
# $tool is the agent's own name for a tool.
core-agent-using = { $tool }を使用中
# Ends "... is not supported yet." (core-error-unsupported).
core-agent-none = このパソコンのコーディングエージェント
# $name is the coding agent's name.
core-agent-ended = { $name }は回答せずに終了しました
core-agent-too-long = { $name }の処理に時間がかかりすぎました

## core/src/core.rs

# A guest device on another computer's library asks for something that is the owner's.
core-core-guest-trackers = そのパソコンのトラッカーアカウントは持ち主のものなので、このデバイスからは何も記録しません
core-core-guest-not-interested = そのパソコンの「興味なし」は持ち主のリストなので、このデバイスからは変更しません
core-core-guest-recommended = そのパソコンのおすすめは、持ち主のリストから作られています
core-core-nothing-picked = 何も選ばれていません

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider }は照合元ではありません
core-prefs-provider-count = { $provider }が{ $count }回出てきます。各照合元は1回だけにしてください
# A setting out of its bounds: $value is outside $low to $high.
core-prefs-outside = { $value }は{ $low }〜{ $high }の範囲外です
core-prefs-empty = 空

## core/src/recommended/mod.rs

core-recommended-ranking = おすすめを順位付け中

## core/src/recommended/sync.rs

core-recommended-finding-people = 好みの近い人を探しています
core-recommended-reading-lists = その人たちのリストを読み込み中
core-recommended-reading-yours = あなたの作品を読み込み中
core-recommended-reading-suggestions = おすすめ候補を読み込み中

## core/src/recommended/page.rs

core-recommended-no-anime = { -anilist }にアニメ{ $id }はありません

## core/src/host/mod.rs

# The tailnet host's status line in Settings.
core-host-off = オフ
core-host-starting = 起動中
# $addresses is a list of address:port, joined with commas.
core-host-serving = { $addresses }で公開中
core-host-no-tailscale = Tailscaleが接続されていません
core-host-cannot-listen = 待ち受けできませんでした
core-host-port-taken = ポート{ $port }は使用中です
core-host-cannot-listen-on = { $address }で待ち受けできませんでした: { $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text }はアドレスとポートではありません

## core/src/host/keys.rs

# $key is the text given, in quotes.
core-host-not-a-key = { $key }はデバイスキーではありません

## core/src/net/limiter.rs

core-net-timed-out = { $ms }msでタイムアウトしました
core-net-rate-limited = { $provider }のレート制限中です

## core/src/net/mod.rs

core-net-waiting = { $host }: 先ほどタイムアウトしました。{ -app-name }は{ $seconds }秒待ってから再度問い合わせます

## core/src/net/anilist.rs

core-net-no-data = データなし

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version }には、このパソコン向けのダウンロードがありません
core-update-cannot-make = { $path }を作成できませんでした: { $error }
core-update-cannot-mark = 新しいAppImageを実行可能にできませんでした: { $error }
core-update-cannot-replace = { $path }を置き換えられませんでした: { $error }
core-update-not-here = このコピーはここでは更新されません
core-update-cannot-start = ダウンロードを開始できませんでした: { $error }
core-update-did-not-start = ダウンロードが始まりませんでした: { $error }
core-update-cannot-write = { $path }に書き込めませんでした: { $error }
core-update-stopped = ダウンロードが停止しました: { $error }
core-update-cancelled = キャンセルしました
# $got and $size are in bytes.
core-update-ended-early = ダウンロードが途中で終わりました（{ $size }バイト中{ $got }バイト）
core-update-checksum = ダウンロードがGitHubのチェックサムと一致しません
core-update-cannot-move = ダウンロードしたファイルを配置できませんでした: { $error }
core-update-cannot-open-dmg = dmgを開けませんでした: { $error }
core-update-cannot-read-dmg = dmgを読み込めませんでした: { $error }
core-update-no-app = dmgにアプリが入っていません
core-update-cannot-copy = 新しいアプリをこのアプリの隣にコピーできませんでした: { $error }

## core/src/calendar.rs

# An episode's event in the calendar file.
core-calendar-season-episode = { $title } - シーズン{ $season } 第{ $number }話
core-calendar-episode = { $title } - 第{ $number }話

## core/src/paths.rs

core-paths-no-home = ホームディレクトリがありません

## core/src/first_run.rs

core-first-run-not-absolute = 動画フォルダは絶対パスで指定してください

## core/src/presence/mod.rs

# Discord shows this in place of an episode line while an extra plays.
core-presence-extra = 特典映像

# The core's words for the library group: trackers, metadata, library, franchise,
# images and transfer (#798).

## core/src/trackers/accounts.rs

# $tracker is the tracker's short name: AniList or MAL.
core-trackers-no-client-id = { $tracker }のクライアントIDが設定されていません。
core-trackers-mal-reconnect = 設定でMyAnimeListを再連携してください。
core-trackers-mal-wants-secret = MyAnimeListがこのアプリのクライアントシークレットを求めています。設定の「トラッカー」で追加するか、App Typeを「other」にしてアプリを登録してください。
# $error is the parser's own message.
core-trackers-token-reply-unreadable = トークンの応答を読めません: { $error }
core-trackers-token-reply-no-token = トークンの応答にアクセストークンがありませんでした

## core/src/trackers/mod.rs

core-trackers-no-token = アクセストークンが保存されていません。設定で再連携してください

## core/src/trackers/cache.rs

core-trackers-no-user-id = アカウントにユーザーIDがありません

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = 移動する作品が選ばれていません
# $site is the tracker's full name: AniList or MyAnimeList.
core-trackers-moved-up = このパソコンから{ $site }へ{ $count }作品を移動しました。
core-trackers-moved-up-some-refused = { $site }へ{ $moved }作品を移動しました。{ $site }は{ $failed }作品を受け付けませんでした。
# $show is the show's folder name; $error is the tracker's answer, cleaned up.
core-trackers-move-refused = { $site }は{ $show }を受け付けませんでした: { $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = 認証がタイムアウトしました。もう一度お試しください。
core-trackers-oauth-state-mismatch = stateが一致しません。CSRFの可能性があるため中止しました。
core-trackers-oauth-no-code = コールバックにcodeがありません
core-trackers-oauth-no-token = コールバックにaccess_tokenがありません
core-trackers-oauth-waiting = ブラウザで{ $site }にログインするのを待っています。
# $username is the account's name on the tracker.
core-trackers-oauth-connected = { $username }として{ $site }と連携しました。
core-trackers-oauth-no-name = プロフィールに名前がありませんでした
# Shown before a colon and the reason.
core-trackers-oauth-could-not-start = ログインを開始できませんでした
# $error is the system's own message.
core-trackers-oauth-port-failed = このパソコンのポート{ $port }を開けませんでした: { $error }
core-trackers-oauth-port-busy = このパソコンのポート{ $port }は別のプログラムが使用中です。少し待ってからもう一度試すか、使用中のままならそのプログラムを閉じてください。
# $error is the code the tracker sent back.
core-trackers-oauth-provider-error = { $site }がエラーを返しました: { $error }
# The browser tab the sign-in leaves. $app is AniBeam's name, in the brand colour in the
# page's body; keep it as a variable.
core-trackers-page-connected = { $site }と{ $app }を連携しました
core-trackers-page-close-tab = このタブは閉じてかまいません。
core-trackers-page-failed-title = { $app }、ログイン失敗
core-trackers-page-failed = ログインが完了しませんでした
core-trackers-page-go-back = { $app }に戻って、もう一度お試しください。
core-trackers-page-finishing-title = { $app }、処理中
core-trackers-page-finishing = 処理中です
core-trackers-page-finishing-body = { $site }から応答があり、AniBeamがログインを仕上げています。{ $app }に戻ると、完了したときに表示されます。
core-trackers-page-cancelled-title = { $app }、ログインをキャンセル
core-trackers-page-cancelled = このログインはキャンセルされました
core-trackers-page-cancelled-body = このタブは閉じてかまいません。必要なら{ $app }からやり直してください。

## core/src/trackers/on_hold.rs

# A progress label while the pass runs; $tracker is AniList or MAL.
core-trackers-on-hold-moving = { $tracker }で保留に移動中
core-trackers-on-hold-moved = { $site }で{ $count }作品を保留に移動しました。
# $second is how many moved on the second tracker, $secondSite.
core-trackers-on-hold-moved-two = { $site }で{ $count }作品、{ $secondSite }で{ $second }作品を保留に移動しました。
core-trackers-on-hold-failed = { $failed }作品を保留に移動できませんでした。
core-trackers-on-hold-moved-failed = { $site }で{ $count }作品を保留に移動しました。{ $failed }作品は保留に移動できませんでした。
core-trackers-on-hold-moved-two-failed = { $site }で{ $count }作品、{ $secondSite }で{ $second }作品を保留に移動しました。{ $failed }作品は保留に移動できませんでした。
# A show's name when its row is gone.
core-trackers-series-fallback = 作品{ $id }
# $name is the show's folder name.
core-trackers-back-on-watching = { $site }で{ $name }を視聴中に戻しました。
core-trackers-back-on-watching-two = { $site }と{ $secondSite }で{ $name }を視聴中に戻しました。
core-trackers-back-on-watching-failed = { $name }を視聴中に戻せませんでした。あとでもう一度お試しください。

## core/src/trackers/writes.rs

# A tracker's failure in words a person can act on. $tracker is AniList or MAL.
core-trackers-rate-limited = { $tracker }のレート制限中です。1分後にもう一度お試しください。
core-trackers-auth-expired = { $tracker }の認証が期限切れです。設定で再連携してください。
core-trackers-entry-not-found = { $tracker }にエントリーが見つかりません。
# $status is the HTTP status code.
core-trackers-server-error = { $tracker }のサーバーエラーです（{ $status }）。あとでもう一度お試しください。
core-trackers-error-see-log = { $tracker }でエラーが発生しました。動作履歴を確認してください。
# $episode is the number asked for, as it was typed.
core-trackers-episode-not-whole = { $episode }は1以上の整数の話数ではありません
core-trackers-score-out-of-range = { $score }は0〜10の範囲外です
core-trackers-rewatch-not-settable = 再視聴は手動で設定するステータスではありません
core-trackers-notes-too-long = メモは{ $max }文字までです
core-trackers-nothing-picked = 何も選ばれていません
core-trackers-notes-not-on-list = まだ{ $tracker }のリストにありません。先にリストに追加してください。
# Where a write landed, picked by $sites: anilist, mal or both. Used inside the lines below.
core-trackers-sites =
    { $sites ->
        [both] { -anilist }と{ -mal }
        [anilist] { -anilist }
       *[mal] { -mal }
    }
# A list status as the status pill names it, picked by $status.
core-trackers-status =
    { $status ->
        [watching] { status-watching }
        [planning] { status-planning }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[repeating] 再視聴中
    }
# $show is the show's folder name.
core-trackers-marked = { core-trackers-sites }で{ $show }の第{ $episode }話を記録しました。
core-trackers-progress-set = { core-trackers-sites }で{ $show }の進捗を{ $progress }に設定しました。
core-trackers-scored = { core-trackers-sites }で{ $show }に{ $score }を付けました。
core-trackers-score-cleared = { core-trackers-sites }で{ $show }のスコアを消去しました。
core-trackers-status-set = { core-trackers-sites }で{ $show }を「{ core-trackers-status }」にしました。
core-trackers-notes-saved = { $show }のメモを{ core-trackers-sites }に保存しました。
core-trackers-notes-cleared = { core-trackers-sites }で{ $show }のメモを消去しました。
# $site is AniList or MyAnimeList.
core-trackers-update-failed = { $site }を更新できませんでした。あとでもう一度お試しください。
# One write to many shows: what landed, what failed and what was left out, in one line.
# $done shows were written and $failed were not; $named is yes when the one show
# written goes by its $name. $action is score, clear or status; $hasReason is yes when
# $reason holds the first tracker's answer.
core-trackers-written =
    { $done ->
        [0] { core-trackers-written-failed }
       *[other] { core-trackers-written-done }{ $failed ->
            [0] {""}
           *[other] { core-trackers-written-failed-more }
        }
    }{ core-trackers-left-out }
core-trackers-written-who =
    { $named ->
        [yes] { $name }
       *[no] { $done }作品
    }
core-trackers-written-done =
    { $action ->
        [clear] { core-trackers-sites }で{ core-trackers-written-who }のスコアを消去しました。
        [status] { core-trackers-sites }で{ core-trackers-written-who }を「{ core-trackers-status }」にしました。
       *[score] { core-trackers-sites }で{ core-trackers-written-who }に{ $score }を付けました。
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] { $failed }作品を更新できませんでした: { $reason }。
       *[no] { $failed }作品を更新できませんでした。
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] { $failed }作品は更新できませんでした: { $reason }。
       *[no] { $failed }作品は更新できませんでした。
    }
# The shows a write to many left out, one sentence per reason, each after a space.
# Keep each [0] first: it leaves the sentence out.
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
       *[other] { $noMatch }作品はトラッカーと照合されていません。
    }{ $hidden ->
        [0] {""}
       *[other] { $hidden }作品は非表示のため、同期がオフです。
    }{ $notConnected ->
        [0] {""}
       *[other] { $notConnected }作品は未連携のトラッカーにしかありません。
    }{ $kept ->
        [0] {""}
       *[other] { $kept }作品は送信先のトラッカーがなく、このパソコンに保存されています。
    }

## core/src/metadata/automatch.rs

# $show is the show's folder name; $title is the title AniList or TVmaze gives it.
core-metadata-matched-to = { $show }を{ $title }に照合しました。
core-metadata-auto-matched = { $count }作品を照合しました。
core-metadata-auto-unmatched = { $count }作品はまだ照合が必要です。
core-metadata-auto-matched-some = { $matched }作品を照合しました。{ $unmatched }作品はまだ照合が必要です。
# Find a match's one line: the sentences below that have something to say, in order,
# each after the first set off by a space. $first names the sentence that leads.
core-metadata-find =
    { $matched ->
        [0] {""}
       *[other] { core-metadata-find-matched }
    }{ $unmatched ->
        [0] {""}
       *[other] { core-metadata-find-unmatched }
    }{ $failed ->
        [0] {""}
       *[other] { core-metadata-find-failed }
    }{ $had ->
        [0] {""}
       *[other] { core-metadata-find-had }
    }{ $busy ->
        [0] {""}
       *[other] { core-metadata-find-busy }
    }
# $hitNamed is yes when the one show matched goes by its folder name, $hitFolder, and
# the title it matched, $hitTitle.
core-metadata-find-matched =
    { $hitNamed ->
        [yes] { $hitFolder }を{ $hitTitle }に照合しました。
       *[no] { $matched }作品を照合しました。
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] { $missName }の照合先が見つかりませんでした。
       *[no] { $unmatched }作品の照合先が見つかりませんでした。
    }
core-metadata-find-failed = { $failed }作品を検索できませんでした。あとでもう一度お試しください。
core-metadata-find-had = { $had }作品はすでに照合済みでした。
core-metadata-find-busy = { $busy }作品はすでに照合中です。

## core/src/metadata/apply.rs

core-metadata-link-unreadable = そのリンクを読めませんでした。AniList、MyAnimeList、TVmaze、IMDbのページURLを貼り付けてください。
core-metadata-no-mal-entry = そのMyAnimeList IDに対応するエントリーはAniListにありません。
core-metadata-no-imdb-show = そのIMDbタイトルに対応する作品はTVmazeにありません。
# Shown before "is not supported yet". $provider is the provider's code: anilist, mal, tmdb.
core-metadata-search-on = { $provider }での検索
core-metadata-query-too-short = 検索するには2文字以上入力してください。
core-metadata-not-a-link = リンクではありません
core-metadata-refresh-unsupported = MALのみ、またはTMDBで照合した作品の更新
# $show is the show's folder name.
core-metadata-matched = { $show }を照合しました。
core-metadata-updated = { $show }の情報を更新しました。
core-metadata-update-failed = { $show }の情報を更新できませんでした。あとでもう一度お試しください。
# Progress labels while the whole library is updated, or shows only seen in passing
# are filled in.
core-metadata-progress-refresh = 更新
core-metadata-progress-backfill = 補完
core-metadata-updated-many = { $count }作品の情報を更新しました。
core-metadata-update-failed-many = { $count }作品の情報を更新できませんでした。あとでもう一度お試しください。
core-metadata-updated-some = { $count }作品の情報を更新しました。{ $failed }作品は更新できませんでした。

## core/src/metadata/fetch.rs

# $id is the provider's own number for the show.
core-metadata-no-such-show = TVmazeに作品{ $id }はありません
core-metadata-no-such-media = AniListにメディア{ $id }はありません

## core/src/metadata/imdb.rs

# Progress labels while the ratings update.
core-metadata-progress-imdb-links = WikidataのIMDbリンク
core-metadata-progress-imdb-ratings = IMDbの評価

## core/src/franchise/layout.rs

# The words under a related show's title when the two share characters.
core-franchise-relation-character = 共通キャラクター

## core/src/images/mod.rs

# A progress label while covers and banners download.
core-images-progress-fetching = 画像を取得中

## core/src/transfer/export.rs

# $folder is the name of the folder the file went to.
core-transfer-exported = { $count }作品を{ $folder }に保存しました。

## core/src/transfer/format.rs

# $error is the JSON parser's own message.
core-transfer-not-json = JSONではありません: { $error }
# $format is the file format's name, anibeam-export.
core-transfer-wrong-format = { $format }形式のファイルではありません

## core/src/transfer/import.rs

core-transfer-imported = ファイルからライブラリを読み込みました。

## core/src/library/delete.rs

# $id is the file's number in the library.
core-library-no-such-file = ライブラリにファイル{ $id }はありません
core-library-file-elsewhere-not-removed = ファイル{ $id }は別の作品のものなので、何も削除しませんでした
core-library-file-outside = { $path }はライブラリフォルダ内のファイルではないので、何も削除しませんでした
# $place is deleted where the system has no trash, trash otherwise. $title is the show's
# folder name.
core-library-episodes-removed =
    { $title }のエピソード{ $count }話を{ $place ->
        [deleted] 削除しました
       *[trash] ゴミ箱に移動しました
    }。
core-library-finished-removed =
    視聴完了の作品{ $count }件を{ $place ->
        [deleted] 削除しました
       *[trash] ゴミ箱に移動しました
    }。
# $series is how many shows the episodes came from.
core-library-watched-removed =
    { $series }作品の視聴済みエピソード{ $count }話を{ $place ->
        [deleted] 削除しました
       *[trash] ゴミ箱に移動しました
    }。

## core/src/library/detail.rs

# A fix the series page offers. $read is episode, with its $number, or label, with the
# name the file was read by in $label. $what is film or episode.
core-library-fix-one-file =
    ここにある1つのファイルは{ $read ->
        [episode] 第{ $number }話
       *[label] { $label }
    }として読み取られています。{ $what ->
        [film] 映画本編
       *[episode] 唯一のエピソード
    }のようです。
# $numbers is a run of episode numbers, as core-library-runs-span or -list writes it;
# $episodes is how many there are.
core-library-fix-specials = スペシャル{ $count }件は、欠けている第{ $numbers }話のようです。
core-library-fix-numbered-on = 第{ $from }話は、前のシーズンから続く番号で、この作品の第{ $to }話のようです。
# Episode numbers in a row: "1 to 6".
core-library-runs-span = { $first }〜{ $last }
# Episode numbers apart: "2, 4 and 7". $init is every number but the last, comma-separated.
core-library-runs-list = { $init }、{ $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = 話数は0〜10000の範囲です
core-library-fix-season-range = シーズンは200までです
core-library-fix-extra-range = 特典映像の番号は10000までです
core-library-file-elsewhere-not-fixed = ファイル{ $id }は別の作品のものなので、何も修正しませんでした
# $title is the show's title.
core-library-fix-undone = { $title }のファイル{ $count }個を、名前からの読み取りに戻しました。
core-library-fixed = { $title }のファイル{ $count }個の読み取り方を修正しました。

## core/src/library/reads.rs

# $title is the show's title. The -many lines are for any count but one.
core-library-hid-one = { $title }を非表示にしました。
core-library-back-one = { $title }を再表示しました。
core-library-hid-many = { $count }作品を非表示にしました。
core-library-back-many = { $count }作品を再表示しました。
core-library-episode-count-set = { $title }の話数を{ $count }話にしました。
core-library-episode-count-cleared = { $title }の話数をクリアしました。

## core/src/library/schedule.rs

core-library-schedule-cleared = { $title }の放送予定メモをクリアしました。
core-library-schedule-saved = { $title }の放送予定メモを{ $count }件保存しました。

## core/src/library/scan.rs

core-library-source-not-absolute = フォルダのパスは絶対パスにしてください
core-library-source-exists = すでに追加されています
core-library-source-nested = 既存のフォルダの中にあります
# $folder is the last part of the folder's path.
core-library-source-added = フォルダ{ $folder }をライブラリに追加しました。
core-library-source-removed = フォルダ{ $folder }をライブラリから削除しました。
core-library-source-removed-with-shows = フォルダ{ $folder }とその{ $count }作品をライブラリから削除しました。
# $name is the show's folder name.
core-library-series-forgotten = { $name }をライブラリから削除しました。
core-library-source-back = フォルダ{ $folder }が戻りました。
core-library-source-unavailable = フォルダ{ $folder }を利用できません。ドライブが接続されているか確認してください。
core-library-source-unreadable = フォルダ{ $folder }を読み込めませんでした。ドライブが接続されているか確認してください。
core-library-source-looks-empty = フォルダ{ $folder }が空のようなので、作品はそのままにしました。
core-library-source-reconcile-failed = { $folder }の作品を更新できませんでした。
# What a scan found: the parts below that are not nought, in order, the ones after the
# first set off by a comma. $first names the part that leads.
core-library-scan-done =
    フォルダを確認しました: { $added ->
        [0] {""}
       *[other] 新しい作品{ $added }件
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] 、
        }変更{ $changed }件
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] 、
        }見つからない作品{ $removed }件
    }。

## core/src/library/report.rs

# Why a coding agent wrote no triage for a quick fix report. $span is
# core-library-span-minutes or -seconds.
core-library-agent-no-answer = { $span }以内に回答がありませんでした
core-library-agent-stopped-code = コード{ $code }で停止しました
core-library-agent-stopped = 停止しました
core-library-agent-answered-nothing = 何も回答しませんでした
core-library-span-minutes = { $count }分
core-library-span-seconds = { $count }秒
core-library-report-no-fix = 報告できる修正があるファイルはありません
# $agent is the agent's name, such as Claude Code; $why is one of the reasons above, or
# the agent's own last words.
core-library-agent-no-triage = { $agent }はトリアージを書きませんでした: { $why }
# Progress labels while a file is loaded, one per part of the library.
core-transfer-progress-sources = フォルダ
core-transfer-progress-series = 作品
core-transfer-progress-history = 履歴
core-transfer-progress-accounts = アカウント
core-transfer-progress-preferences = 設定
core-transfer-progress-done = 完了

## core/src/downloads/anirss.rs

# $message is what the connection said
core-downloads-anirss-no-answer = anirssから応答がありません: { $message }
# $status is the HTTP status code, $text the body anirss sent
core-downloads-anirss-answered = anirssが{ $status }を返しました: { $text }
core-downloads-no-free-port = 空いているポートがありません: { $error }
# $status is how the process ended, as the system words it
core-downloads-anirss-exited = anirssが終了しました（{ $status }）
# $said is the last lines anirss printed
core-downloads-anirss-exited-said = anirssが終了しました（{ $status }）: { $said }
core-downloads-anirss-slow-start = anirssが{ $seconds }秒以内に起動しませんでした
core-downloads-anirss-slow-start-said = anirssが{ $seconds }秒以内に起動しませんでした: { $said }
# $command is a command to type, kept as it is
core-downloads-anirss-install-mac = anirss { $version }以降をインストールしてください: { $command }
core-downloads-anirss-install-linux = anirss { $version }以降をインストールしてください: AURのanirssパッケージ、または{ $command }
# $update is core-downloads-anirss-update-mac or -linux
core-downloads-anirss-too-old = anirss { $version }はAniBeamには古すぎます。{ $minimum }以降が必要です。{ $update }。
core-downloads-anirss-lacks-feature = anirss { $version }はトレントクライアントごとのログイン情報を分けて保持できません。AniBeamがクライアントを切り替えるにはこれが必要で、anirss { $minimum }以降なら対応しています。{ $update }。
core-downloads-anirss-update-mac = 次のコマンドで更新してください: { $command }
core-downloads-anirss-update-linux = anirssパッケージを更新するか、次を実行してください: { $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = トレントファイルが届きませんでした: { $message }
core-downloads-torrent-file-answered = トレントファイルの取得で{ $status }が返りました

## core/src/downloads/feed.rs

core-downloads-feed-answered = フィードが{ $status }を返しました
core-downloads-not-a-feed = フィードではありません: { $error }

## core/src/downloads/jackett.rs

# $program is the command that was not found
core-downloads-jackett-missing = Jackettがインストールされていません: { $program }が見つかりませんでした。インストールするか、設定でコマンドを変更してください。
# $name is the endpoint, $why what went wrong
core-downloads-endpoint-start-failed = { $name }を起動できませんでした: { $why }
core-downloads-endpoint-start-timeout = { $name }は起動しましたが、{ $seconds }秒以内に応答しませんでした。
core-downloads-command-exited = コマンドが{ $status }で終了しました

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirssが停止しました
# $install is core-downloads-anirss-install-mac or -linux
core-downloads-anirss-missing = { $install }。または、設定で起動中のサーバーを指定してください。
# $command is the anirss command set in Settings
core-downloads-anirss-command-missing = { $command }が見つかりませんでした。{ $install }。または、設定で起動中のサーバーを指定してください。
core-downloads-anirss-start-failed = { $command }を起動できませんでした: { $error }
# $tooOld is core-downloads-anirss-too-old; $message is what anirss said
core-downloads-anirss-too-old-said = { $tooOld }応答: { $message }
core-downloads-no-torrent-client = { $address }で応答するqBittorrentがなく、AniBeam自身が起動できるものもありません。qBittorrentをインストールしてWeb UIをオンにするか、qbittorrent-noxをインストールしてください。AniBeamが代わりに起動します
core-downloads-own-client-gone = AniBeamのqBittorrentがありません: qbittorrent-noxがインストールされていません。AniBeamを再インストールするか、設定で自分のqBittorrentを指定してください
core-downloads-no-port = { $address }にポートがありません
core-downloads-own-client-settings = qBittorrentの設定を書き込めません: { $error }
core-downloads-own-client-start-failed = { $command }を起動できませんでした: { $error }
core-downloads-own-client-slow-start = AniBeamのqBittorrentが起動しませんでした
# $said is the last lines of its log
core-downloads-own-client-slow-start-said = AniBeamのqBittorrentが起動しませんでした: { $said }
core-downloads-not-ready = ダウンロードの準備ができていません。設定の「ダウンロード」を確認してください。
core-downloads-ready = ダウンロードの準備ができました。
# $count is how many sources anirss's own config names
core-downloads-helper-sources = ダウンロードの検索はAniBeam自身で行うようになりました。検索ヘルパーの設定にある{ $count }件のソースは読み込みません。設定の「ダウンロード」で追加するか、そこで検索ヘルパーを選んでください。
core-downloads-client-connected = ダウンロードアプリに接続しました。
core-downloads-client-not-connected = ダウンロードアプリに接続していません。設定でアドレスを確認してください。
core-downloads-not-on-demand = { $name }はオンデマンドのエンドポイントではありません
core-downloads-source-running = ダウンロードソースが動作中です。
core-downloads-source-not-running = { $name }が動作していません。ソースの一覧でオンにし、確認して起動してください。
core-downloads-empty-query = 検索語を入力してください
# What a search's progress line names when it was given no source
core-downloads-first-endpoint = 最初のエンドポイント
# A search's progress line. The desktop's search card compares the line with these, so
# keep the name in core-downloads-search-starting-endpoint a whole placeable.
core-downloads-search-starting = 検索を開始中
core-downloads-search-starting-endpoint = { $endpoint }を起動中
core-downloads-search-searching-endpoint = { $endpoint }を検索中
core-downloads-search-reading = 結果を読み込み中
# $name is the subscription's name
core-downloads-subscribed = { $name }を購読しました。
core-downloads-client-no-rule = qBittorrentに{ $name }という名前のルールはありません
# A progress line while a subscription's feed is narrowed; $search is the search tried
core-downloads-trying-search = 「{ $search }」を試しています
core-downloads-nothing-selected = ダウンロードするものが選ばれていません
core-downloads-added = ダウンロードに{ $count }件追加しました。
core-downloads-movie-link = 映画のリンクはmagnetかhttp(s)のURLにしてください
core-downloads-added-movie = ダウンロードに映画を追加しました。
core-downloads-unsubscribed = { $name }の購読を解除しました。
# $folder is a path, $name the library show that lives there
core-downloads-library-folder = { $folder }は{ $name }のライブラリフォルダで、この購読がダウンロードしていないエピソードがあります。ファイルを残して購読を削除するか、作品ページから{ $name }を削除してください
core-downloads-no-torrents-named = トレントが指定されていません
core-downloads-paused = ダウンロード{ $count }件を一時停止しました。
core-downloads-resumed = ダウンロード{ $count }件を再開しました。
core-downloads-deleted-with-files = ダウンロード{ $count }件をファイルごと削除しました。
core-downloads-removed-from-list = ダウンロード{ $count }件を一覧から削除しました。
core-downloads-nothing-to-remove = 削除するものはありません
# $title is the show's title
core-downloads-removed-show = { $title }をダウンロードから削除しました。
core-downloads-removed-the-show = 作品をダウンロードから削除しました。
core-downloads-deleted-still-listed = 削除したファイルが、まだダウンロードアプリに表示されています。
core-downloads-feed-url = フィードはhttp(s)のURLにしてください
core-downloads-rule-needs-name = ルールには名前が必要です
core-downloads-no-rule = { $name }という名前のルールはありません
core-downloads-feed-no-search = フィードに短くできる検索語がありません
core-downloads-no-repair = { $name }は修復の必要がありません。
core-downloads-repaired = { $name }の購読を修復しました。
# $search is the shortest search tried
core-downloads-repair-failed = 検索語を短くしても何も見つかりません。「{ $search }」でも結果は空です
# $id is the client's id, in quotes
core-downloads-no-saved-client = 保存されたトレントクライアント{ $id }はありません
core-downloads-client-switched = ダウンロードに別のダウンロードアプリを使うようにしました。
# After the name of anirss or qBittorrent and a colon
core-downloads-host-not-running = ホストで動作していません
core-downloads-host-late = 時間内に応答しませんでした

## core/src/downloads/native.rs

core-downloads-native-empty-query = 検索語が空です
# $query is the search, in quotes
core-downloads-native-no-results = { $query }の結果はありません
# $name is the feed's name, in quotes
core-downloads-native-feed-taken = { $name }という名前のフィードは、すでに別のアドレスを読み込んでいます
core-downloads-native-no-items = 項目が指定されていません
# $link is the link given, in quotes
core-downloads-native-bad-link = リンクはmagnet:かhttp(s)のURLにしてください。指定されたのは{ $link }です
core-downloads-native-no-links = リンクが指定されていません
core-downloads-native-no-names = 購読の名前が指定されていません
core-downloads-native-files-elsewhere = トレントクライアントは別のマシンにファイルを保存しているため、ファイルはトレントと一緒にしか削除できません
# $names is a list of feed names
core-downloads-native-unknown-feeds = 不明なフィード: { $names }
# $paths is a list of folders
core-downloads-native-outside-folders = ダウンロードフォルダ外のファイルは削除しません: { $paths }
# Removals refused; $path is the folder or file
core-downloads-refuse-home-folder = { $path }はホームフォルダか、それを含んでいるので、何も削除しませんでした
core-downloads-refuse-library-folder = { $path }はライブラリフォルダか、それを含んでいるので、何も削除しませんでした

## core/src/downloads/paths.rs

core-downloads-no-default-folder = qBittorrentに既定の保存フォルダがありません。qBittorrentで設定してください
core-downloads-name-empty = 名前を空にはできません
core-downloads-name-slash = 名前に/や\は使えません
# $name is the name given, in quotes
core-downloads-name-not-folder = { $name }はフォルダ名ではありません

## core/src/downloads/prune.rs

# $title is the show's title
core-downloads-pruned = 放送が終了したため、{ $title }の購読を解除しました。
core-downloads-prune-failed = { $title }の購読を解除できませんでした。設定の「ダウンロード」を確認してください。

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path }は通常の絶対パスではないので、何も削除しませんでした
core-downloads-refuse-root = { $path }はファイルシステムのルートなので、何も削除しませんでした
core-downloads-refuse-outside-library = { $path }はライブラリフォルダ内にないので、何も削除しませんでした
core-downloads-refuse-save-folder = { $path }はトレントの保存先フォルダで、ダウンロードしたものではないので、何も削除しませんでした

## core/src/downloads/watch.rs

# $episode is the episode's number, $title the show's
core-downloads-episode-missed = { $title }の第{ $episode }話が、放送から3日たっても見つかりません。フィードで探し続けます。

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = anirssのコマンドを空にはできません
core-downloads-settings-anirss-address = anirssのアドレスはhttp://かhttps://で始めてください
# $client is the torrent client's name, such as qBittorrent
core-downloads-settings-client-address = { $client }のアドレスはhttp://かhttps://で始めてください
# $id is the id given, in quotes
core-downloads-settings-bad-id = { $id }はクライアントIDではありません。英字、数字、-、_を使ってください
core-downloads-settings-same-id = 2つのクライアントのIDが{ $id }で重複しています
# $name is a saved client's name
core-downloads-settings-saved-address = { $name }のアドレスはhttp://かhttps://で始めてください
core-downloads-settings-same-name = 2つのクライアントの名前が{ $name }で重複しています。それぞれ別の名前を付けてください

## core/src/downloads/qbt.rs

# $address is where qBittorrent was asked, $message what the connection said
core-downloads-qbt-no-answer = { $address }のqBittorrentから応答がありません: { $message }
core-downloads-qbt-password-rejected = qBittorrentがパスワードを拒否しました。設定で修正してください
core-downloads-qbt-banned = ログインの失敗が続いたため、qBittorrentがこのアドレスをブロックしました。しばらく待つか、qBittorrentを再起動してください
core-downloads-qbt-login-rejected = qBittorrentがユーザー名またはパスワードを拒否しました
# $status is the HTTP status code
core-downloads-qbt-login-failed = { $address }のqBittorrentへのログインで{ $status }が返りました。次の呼び出しで再試行します
core-downloads-qbt-not-web-ui = { $address }は{ $status }を返しましたが、qBittorrentのWeb UIではありません。設定のアドレスと、qBittorrentでWeb UIがオンになっているか確認してください
# $said is what qBittorrent answered, in quotes
core-downloads-qbt-not-added = qBittorrentはトレントを追加しませんでした: { $said }
core-downloads-qbt-some-not-added = qBittorrentは{ $count }件のトレントを追加できませんでした: { $said }
# $line is the first line of what qBittorrent answered
core-downloads-qbt-answered = qBittorrentが{ $status }を返しました: { $line }

## core/src/downloads/replace.rs

# $item is the number of the release asked for, $count how many there are
core-downloads-replace-bad-item = 項目{ $item }は{ $count }件のリリースのどれでもありません
# $episode is the episode's number
core-downloads-not-replaced = 第{ $episode }話は置き換えられませんでした。
core-downloads-replaced = 第{ $episode }話を新しいダウンロードに置き換えました。

## core/src/downloads/rolling.rs

# $id is the rolling download's number
core-downloads-no-roll = 順次ダウンロード{ $id }はありません
# $title is the show, $from and $to the episodes kept on disk out of $total
core-downloads-rolled = { $title }: 全{ $total }話中、第{ $from }〜{ $to }話を保持しています。
core-downloads-roll-ahead = 少なくとも1話は先まで保持してください
core-downloads-roll-nothing = 取得するものはありません

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = JSON検索のソースは、項目の対応を定義したエンドポイントリポジトリから追加してください
core-releases-endpoint-no-name = `name`がありません
# $name is the endpoint's name, in quotes
core-releases-endpoint-bad-name = { $name }: 英字、数字、「.」「_」「-」だけを使ってください
# $why is core-releases-json-needs-repo
core-releases-endpoint-json = { $name }: { $why }
# $kind is the kind given, in quotes; $valid lists the kinds there are
core-releases-endpoint-unknown-kind = { $name }: 不明な種類{ $kind }（有効な種類: { $valid }）
core-releases-endpoint-no-url = { $name }: `url`がありません
core-releases-endpoint-not-http = { $name }: アドレスはhttp://かhttps://で始めてください
# $kind is the endpoint's kind; {query} is written as it is in the address
core-releases-endpoint-needs-query = { $name }: { $kind }のurlには{"{"}query{"}"}プレースホルダーが必要です
core-releases-endpoint-no-query = { $name }: { $kind }のurlは検索語を自動で付けるため、{"{"}query{"}"}は使いません

## core/src/releases/fetch.rs

# $code is the error code the indexer sent
core-releases-indexer-error = インデクサーがエラー{ $code }を返しました
core-releases-bad-feed = 不正なフィード: { $error }
# $status is the HTTP status code
core-releases-feed-answered = フィードが{ $status }を返しました

## core/src/releases/json_feed.rs

core-releases-not-feed-json = フィードのJSONではありません: { $error }

## core/src/releases/json.rs

core-releases-not-json = JSONではありません: { $error }
# $message is what the search said about itself
core-releases-search-error-said = 検索がエラーを返しました: { $message }
core-releases-search-error = 検索がエラーを返しました
# $path is where the results were looked for, in quotes
core-releases-no-results-field = 応答の{ $path }に結果がありません
core-releases-results-not-list = 応答の結果がリストではありません
core-releases-search-answered = 検索が{ $status }を返しました

## core/src/releases/native.rs

core-releases-no-endpoints = エンドポイントがまだありません: 「その他」の「エンドポイント」で、リポジトリか手動のエンドポイントを追加してください
core-releases-no-source-named = { $name }という名前のソースはありません
core-releases-empty-query = 検索語を入力してください
core-releases-cannot-follow = { $name }はフォローできません: 検索には応答しますが、新エピソードを確認するフィードがありません
core-releases-endpoint-gone = { $name }はもう利用できません: 削除されたか、オフになっているか、鍵がありません

## core/src/releases/quirks.rs

# What is wrong with an endpoint definition's quirks. one_variant, show_feed, slug and
# {slug} are the definition's own words and stay as they are.
# $word is the word given, in quotes
core-releases-quirks-variant = one_variantの{ $word }は短い1語にしてください
core-releases-quirks-pattern-count = show_feedのページパターンは1〜{ $most }個です
core-releases-quirks-pattern-long = show_feedのページパターンが長すぎます
core-releases-quirks-pattern-bad = show_feedのページパターン: { $error }
core-releases-quirks-pattern-slug = show_feedのページパターンにはslugという名前のグループが必要です
core-releases-quirks-url-https = show_feedのurlはhttpsにしてください
core-releases-quirks-url-host = show_feedのurlはエンドポイントと同じホストにしてください
core-releases-quirks-url-slug = show_feedのurlには{"{"}slug{"}"}を1回だけ使い、ほかのプレースホルダーは使えません

## core/src/releases/refinement.rs

# 'tokens' and `tokens` name the call's own field and stay as they are
core-releases-refine-no-tokens = 操作'tokens'には空でない`tokens`リストが必要です

## core/src/releases/grid.rs

# A release group's whole-season torrent; $group is the group's name
core-releases-group-batch = { $group }のバッチ
# $count is how many episodes, $list their numbers and runs ("1 to 4, 7")
core-releases-episodes-phrase = 第{ $list }話
# A run of episode numbers
core-releases-episode-range = { $from }〜{ $to }

## core/src/subscriptions.rs

# Fills core-error-unsupported: "anirss on this platform is not supported yet."
core-subscriptions-platform = このプラットフォームのanirss
# After "anirss: "; $message is why its answer did not parse
core-subscriptions-unreadable = 応答を読み取れませんでした（{ $message }）
core-subscriptions-stopped = 理由を示さずに停止しました
# $said is what anirss printed
core-subscriptions-stopped-said = 停止しました: { $said }

## Names that stay the same in every language

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList
