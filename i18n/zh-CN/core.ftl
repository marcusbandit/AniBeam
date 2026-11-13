# zh-CN translation of en/core.ftl. An ID missing here is shown in English.

## The tailnet host

tailnet-person-no-name = Tailscale 中没有名称

## Counts

count-episodes = { $count } 集
count-shows = { $count } 部番剧

## Actions every screen uses

action-cancel = 取消
action-close = 关闭
action-try-again = 重试
action-remove = 移除
action-delete = 删除
action-open = 打开
action-save = 保存
action-apply = 应用
action-clear = 清除
action-refresh = 刷新
action-play = 播放
action-connect = 连接
action-start = 开始
action-back = 返回
action-see-all = 查看全部
action-open-series = 打开番剧
action-open-settings = 打开设置

## The lists a tracker keeps

status-watching = 在看
status-completed = 看过
status-paused = 搁置
status-on-hold = 搁置
status-dropped = 抛弃
status-planning = 想看
status-rewatching = 重看

## Episodes

episode-number = 第 { $number } 集

## AniList's words for a show: its format, its airing status, how two shows relate

anilist-format-tv = TV
anilist-format-tv-short = TV 短篇
anilist-format-film = 剧场版
anilist-format-special = 特别篇
anilist-format-ova = OVA
anilist-format-ona = ONA
anilist-format-music = 音乐
anilist-status-finished = 已完结
anilist-status-airing = 连载中
anilist-status-not-yet-aired = 尚未播出
anilist-status-cancelled = 已取消
anilist-status-hiatus = 停播中
anilist-relation-adaptation = 改编
anilist-relation-prequel = 前作
anilist-relation-sequel = 续作
anilist-relation-parent = 正篇
anilist-relation-side-story = 外传
anilist-relation-character = 角色
anilist-relation-summary = 总集篇
anilist-relation-alternative = 另一版本
anilist-relation-spin-off = 衍生作
anilist-relation-other = 其他
anilist-relation-source = 原作
anilist-relation-compilation = 合集
anilist-relation-contains = 包含

## The activity log: a problem whose place had no words of its own

event-problem-library = 媒体库出了问题。详情见开发者选项。
event-problem-metadata = 番剧信息出了问题。详情见开发者选项。
event-problem-trackers = 你的列表出了问题。详情见开发者选项。
event-problem-franchise = 相关番剧出了问题。详情见开发者选项。
event-problem-playback = 播放出了问题。详情见开发者选项。
event-problem-store = 保存数据时出了问题。详情见开发者选项。
event-problem-system = AniBeam 出了问题。详情见开发者选项。
event-problem-downloads = 下载出了问题。详情见开发者选项。

## Errors (contract::CoreError)

core-error-not-found-source = 找不到文件夹 { $id }。
core-error-not-found-series = 找不到番剧 { $id }。
core-error-not-found-file = 找不到文件 { $id }。
core-error-not-found-session = 找不到播放会话 { $id }。
core-error-not-found-job = 找不到任务 { $id }。
core-error-invalid = { $field } 无效：{ $message }。
core-error-invalid-ended = { $field } 无效：{ $message }
core-error-unavailable = 此文件夹不可用：{ $path }
core-error-not-connected = 未连接 { $tracker }。
core-error-refused-hidden = 无法执行：这部番剧已隐藏。
core-error-refused-no-match = 无法执行：这部番剧尚未匹配，请在“番剧信息”中匹配。
core-error-refused-not-newer = 无法执行：你的列表进度已超过这一集。
core-error-refused-extra = 无法执行：这是附加内容。
core-error-refused-unmatched = 无法执行：这个文件没有匹配到剧集。
core-error-refused-on-disk = 无法执行：这个文件只在磁盘上。
core-error-provider = { $provider } 没有响应：{ $message }
core-error-io = 无法读写文件：{ $message }
core-error-storage = 无法保存媒体库：{ $message }
core-error-keyring = 无法访问系统密码存储：{ $message }
core-error-unsupported = 暂不支持 { $what }。
core-error-version = 这份导出来自更新版本的 AniBeam（格式 { $found }）。请更新 AniBeam 后再导入。
core-error-internal = AniBeam 遇到内部错误：{ $message }
core-error-service = { $service }：{ $message }

## core/src/repos/mod.rs

core-repos-paste-address = 粘贴仓库地址
core-repos-must-be-https = 地址必须是 https
core-repos-unreachable = 无法连接仓库：{ $message }
core-repos-not-found = 该地址下没有内容（GitHub 私有仓库也会返回 404）
core-repos-http-status = 仓库返回 HTTP { $status }
core-repos-unreadable = 无法读取 { $path }：{ $error }
core-repos-no-such-endpoint = 没有该 ID 的已安装端点
core-repos-key-changed = 仓库现在使用的签名密钥与你信任的不同。移除后重新添加即可接受新密钥。
core-repos-stale = 仓库提供了较旧的索引（序号 { $serial }，已有 { $kept }），保留较新的那份
core-repos-key-not-shown = 仓库的密钥与你看到的不一致。请重新添加并核对指纹。
core-repos-repo-added = 已添加一份下载来源列表。
core-repos-repo-removed = 已移除一份下载来源列表。
core-repos-no-field = { $name } 没有字段 { $key }
core-repos-needs-field = { $name } 需要填写 { $label }
core-repos-host-only = { $name }：{ $label } 只能是主机或主机:端口，如 192.0.2.10:9696
core-repos-not-added = 尚未添加该仓库
core-repos-not-offered = { $repo } 不提供 { $id }
core-repos-already-installed = { $name } 已安装
core-repos-taken-by-you = 已安装名为 { $id } 的端点（由你添加），请先移除
core-repos-taken-by-repo = 已安装名为 { $id } 的端点（来自另一个仓库），请先移除
core-repos-source-added = 已添加下载来源。
core-repos-source-updated = 已更新下载来源。
core-repos-source-removed = 已移除下载来源。
core-repos-source-changed = 已更改下载来源。
core-repos-give-name = 给它起个名字
core-repos-put-query = 在地址中放入搜索词的位置写上 {"{"}query{"}"}
core-repos-name-taken = 已安装名为 { $id } 的端点
core-repos-only-by-hand = 只有手动添加的来源可以编辑
core-repos-keep-key = 在地址中保留 {"{"}{ $key }{"}"}，并通过“更改 { $label }”修改它的值，或者移除该站点后不带它重新添加
core-repos-note-kind = anirss 无法搜索 { $kind } 类型，所以电脑端不包含它；手机会搜索它
core-repos-note-by-hand = anirss 有自己手动写入的 { $id }，电脑端搜索的是那一个
core-repos-note-theirs = 这个 anirss 由你自己运行，电脑端从不更改它的配置：把 { $id } 加入该 anirss 的配置，电脑端才会搜索它
core-repos-note-sandbox = 这次运行从不写入 anirss 的配置，所以电脑端在这里不搜索 { $id }
core-repos-note-old = anirss { $version } 无法接收来自 { -app-name } 的端点；请更新到 { $min } 或更高版本，电脑端才能搜索它

## core/src/repos/index.rs

core-repos-key-not-ed25519 = 索引的密钥不是 ed25519 密钥
core-repos-key-not-base64 = 索引的密钥不是 32 字节的 base64
core-repos-key-invalid = 索引的密钥不是有效的 ed25519 密钥
core-repos-too-large = 文件太大，不可能是仓库
core-repos-not-a-repo = 这不是 { -app-name } 端点仓库
core-repos-format-version = 该仓库的格式版本为 { $found }；此 { -app-name } 只能读取版本 { $reads }。请更新 { -app-name }。
core-repos-payload-not-base64 = 仓库的数据不是 base64
core-repos-signature-malformed = 仓库的签名格式错误
core-repos-index-unreadable = 无法读取仓库的索引：{ $error }
core-repos-signature-mismatch = 仓库的签名与内容不符
core-repos-check-scheme = { $id }：地址必须以 https:// 开头
core-repos-check-scheme-http = { $id }：地址必须以 http:// 或 https:// 开头
core-repos-check-auth-key = { $id }：认证键 { $key } 无效
core-repos-check-host-torznab = { $id }：只有 torznab 地址可以把主机写成 {"{"}{ $key }{"}"}
core-repos-check-host-placeholder = { $id }：主机中有占位符

## core/src/playback/tracks.rs

core-playback-track-number = 轨道 { $number }
core-playback-track-file = { $label }（文件）
core-playback-lang-region = { $name }（{ $region }）
core-playback-lang-english = 英语
core-playback-lang-japanese = 日语
core-playback-lang-spanish = 西班牙语
core-playback-lang-portuguese = 葡萄牙语
core-playback-lang-french = 法语
core-playback-lang-german = 德语
core-playback-lang-italian = 意大利语
core-playback-lang-russian = 俄语
core-playback-lang-chinese = 中文
core-playback-lang-korean = 韩语
core-playback-lang-arabic = 阿拉伯语
core-playback-lang-hindi = 印地语
core-playback-lang-indonesian = 印尼语
core-playback-lang-malay = 马来语
core-playback-lang-thai = 泰语
core-playback-lang-vietnamese = 越南语
core-playback-lang-polish = 波兰语
core-playback-lang-turkish = 土耳其语
core-playback-lang-dutch = 荷兰语
core-playback-lang-swedish = 瑞典语
core-playback-lang-norwegian = 挪威语
core-playback-lang-danish = 丹麦语
core-playback-lang-finnish = 芬兰语
core-playback-lang-hebrew = 希伯来语
core-playback-lang-ukrainian = 乌克兰语
core-playback-lang-czech = 捷克语
core-playback-lang-hungarian = 匈牙利语
core-playback-lang-greek = 希腊语
core-playback-lang-romanian = 罗马尼亚语
core-playback-lang-filipino = 菲律宾语
core-playback-region-brazil = 巴西
core-playback-region-portugal = 葡萄牙
core-playback-region-latin-america = 拉丁美洲
core-playback-region-spain = 西班牙
core-playback-region-simplified = 简体
core-playback-region-traditional = 繁体

## contract/src/enums.rs

core-jobs-scan = 正在检查文件夹
    .stopped = 检查文件夹时出现问题，已停止。
core-jobs-auto-match = 正在匹配番剧
    .stopped = 匹配番剧时出现问题，已停止。
core-jobs-search = 正在搜索番剧
    .stopped = 搜索番剧时出现问题，已停止。
core-jobs-resolve-link = 正在读取链接
    .stopped = 读取链接时出现问题，已停止。
core-jobs-apply-match = 正在匹配一部番剧
    .stopped = 匹配番剧时出现问题，已停止。
core-jobs-refresh = 正在更新番剧
    .stopped = 更新番剧时出现问题，已停止。
core-jobs-refresh-all = 正在更新番剧信息
    .stopped = 更新番剧信息时出现问题，已停止。
core-jobs-refresh-airing = 正在检查播出情况
    .stopped = 检查播出情况时出现问题，已停止。
core-jobs-refresh-imdb = 正在更新评分
    .stopped = 更新评分时出现问题，已停止。
core-jobs-clear-images = 正在清除图片
    .stopped = 清除图片时出现问题，已停止。
core-jobs-fill-images = 正在获取图片
    .stopped = 获取图片时出现问题，已停止。
core-jobs-connect-tracker = 正在连接你的列表
    .stopped = 连接列表时出现问题，已停止。
core-jobs-mark = 正在标记为已看
    .stopped = 标记已看时出现问题，已停止。
core-jobs-set-progress = 正在设置进度
    .stopped = 设置进度时出现问题，已停止。
core-jobs-score = 正在保存评分
    .stopped = 保存评分时出现问题，已停止。
core-jobs-refresh-progress = 正在读取列表进度
    .stopped = 读取列表进度时出现问题，已停止。
core-jobs-refresh-watching = 正在读取“在看”列表
    .stopped = 读取“在看”列表时出现问题，已停止。
core-jobs-refresh-airing-now = 正在检查本季新番
    .stopped = 检查本季新番时出现问题，已停止。
core-jobs-refresh-recommended = 正在为你寻找番剧
    .stopped = 为你寻找番剧时出现问题，已停止。
core-jobs-read-show-page = 正在读取番剧
    .stopped = 读取番剧时出现问题，已停止。
core-jobs-on-hold = 正在把闲置番剧移到“搁置”
    .stopped = 把闲置番剧移到“搁置”时出现问题，已停止。
core-jobs-set-status = 正在更改列表状态
    .stopped = 更改列表状态时出现问题，已停止。
core-jobs-move-lists = 正在把列表移到追踪服务
    .stopped = 把列表移到追踪服务时出现问题，已停止。
core-jobs-notes = 正在保存笔记
    .stopped = 保存笔记时出现问题，已停止。
core-jobs-crawl = 正在查找相关番剧
    .stopped = 查找相关番剧时出现问题，已停止。
core-jobs-skip-windows = 正在查找片头和片尾
    .stopped = 查找片头和片尾时出现问题，已停止。
core-jobs-export = 正在把媒体库保存到文件
    .stopped = 把媒体库保存到文件时出现问题，已停止。
core-jobs-import = 正在从文件载入媒体库
    .stopped = 从文件载入媒体库时出现问题，已停止。
core-jobs-subscriptions = 正在检查订阅
    .stopped = 检查订阅时出现问题，已停止。
core-jobs-connect-downloads = 正在连接下载应用
    .stopped = 连接下载应用时出现问题，已停止。
core-jobs-search-releases = 正在查找剧集
    .stopped = 查找剧集时出现问题，已停止。
core-jobs-refine-releases = 正在筛选剧集
    .stopped = 筛选剧集时出现问题，已停止。
core-jobs-subscribe = 正在订阅番剧
    .stopped = 订阅番剧时出现问题，已停止。
core-jobs-download = 正在添加下载
    .stopped = 添加下载时出现问题，已停止。
core-jobs-remove-subscription = 正在移除订阅
    .stopped = 移除订阅时出现问题，已停止。
core-jobs-control-torrents = 正在更改下载
    .stopped = 更改下载时出现问题，已停止。
core-jobs-test-torrent-client = 正在测试下载应用
    .stopped = 测试下载应用时出现问题，已停止。
core-jobs-remove-show = 正在移除番剧
    .stopped = 移除番剧时出现问题，已停止。
core-jobs-repair-subscription = 正在修复订阅
    .stopped = 修复订阅时出现问题，已停止。
core-jobs-check-endpoints = 正在检查下载来源
    .stopped = 检查下载来源时出现问题，已停止。
core-jobs-start-endpoint = 正在启动下载来源
    .stopped = 启动下载来源时出现问题，已停止。
core-jobs-schedule-agent = 正在向编程助手询问播出时间
    .stopped = 向编程助手询问播出时间时出现问题，已停止。

## core/src/airing_now.rs

core-airing-now-now = 正在播出
core-airing-now-winter = { $year } 年冬季
core-airing-now-spring = { $year } 年春季
core-airing-now-summer = { $year } 年夏季
core-airing-now-fall = { $year } 年秋季
core-airing-now-planned = 已把《{ $title }》加入 { $site } 的“想看”列表。
core-airing-now-planned-both = 已把《{ $title }》加入 { $site } 和 { $other } 的“想看”列表。
core-airing-now-already-on =
    《{ $title }》已在你的 { $site } 列表中，状态为“{ $status ->
        [watching] { status-watching }
        [repeating] { status-rewatching }
        [completed] { status-completed }
        [paused] { status-on-hold }
        [dropped] { status-dropped }
       *[planning] { status-planning }
    }”。
core-airing-now-plan-failed = 无法把《{ $title }》加入“想看”列表，请稍后再试。

## core/src/aired.rs

core-aired-episode-out = 《{ $title }》第 { $number } 集已播出。

## core/src/cast/mod.rs

core-cast-connecting = 正在投屏到 { $device }。
core-cast-ended = 已结束投屏到 { $device }。
core-cast-stopped = 已停止投屏到 { $device }。
core-cast-failed = 投屏到 { $device } 失败。请确认它在同一网络中。
core-cast-not-built = 此版本不包含投屏功能
core-cast-serve-failed = 无法提供剧集：{ $error }
core-cast-device-gone = 该设备已不在网络中
core-cast-not-a-playlist = 不是播放列表地址
core-cast-nothing = 没有正在投屏的内容
core-cast-still-starting = 设备仍在启动
core-cast-no-route = 这台设备无法连到电视
core-cast-no-control = 电视没有提供控制地址
core-cast-not-handed = 另一台电脑没有交出剧集：{ $error }
core-cast-empty-playlist = 另一台电脑发来的播放列表是空的
core-cast-refused-both = { $error }；原文件它也拒绝了（{ $first }）
core-cast-chromecast-stopped = Chromecast 已停止
core-cast-hevc = { $error }。这一集是 HEVC 编码，只有较新的 Chromecast（带 Google TV 或 Ultra）才能解码

## core/src/cast/castv2.rs

core-cast-chromecast-unreachable = 无法连接 Chromecast：{ $error }
core-cast-chromecast-broke = Chromecast 连接中断：{ $error }
core-cast-chromecast-no-player = Chromecast 没有启动媒体播放器
core-cast-chromecast-refused-player = Chromecast 拒绝启动媒体播放器（{ $reason }）
core-cast-no-reason = 未说明原因
core-cast-chromecast-cannot-play = Chromecast 无法播放此内容（{ $kind }）
core-cast-chromecast-error = Chromecast 播放时出错并停止

## core/src/cast/dlna.rs

core-cast-tv-no-answer = 电视没有响应：{ $error }
core-cast-tv-refused = 电视拒绝了 { $action }
core-cast-tv-refused-code = 电视拒绝了 { $action }（{ $code }）

## core/src/cast/media.rs

core-cast-no-video = 这个文件没有视频。
core-cast-hi10p = 这一集是 10 位 H.264（Hi10P），电视和 Chromecast 都无法解码。投屏需要转码，目前尚未实现。
core-cast-video-codec = 这一集的视频（{ $codec }）无法投屏。
core-cast-audio-codec = 这一集的音频（{ $codec }）无法投屏。

## core/src/phone/mod.rs

core-phone-download-started = 已开始下载《{ $folder }》的 { $count } 集。
core-phone-subscribed = 已订阅《{ $name }》。
core-phone-rule-stopped = 《{ $name }》不再检查新剧集：它的来源已消失，或需要重新填写密钥。
core-phone-new-episode = 《{ $name }》的新一集正在下载。
core-phone-name-taken = { $name }：已有其他来源使用这个名字
core-phone-nothing-to-download = 没有可下载的内容
core-phone-over-cap = 这会让 { -app-name } 超出存储上限（已用或即将占用 { $used } GB，上限 { $cap } GB）；请删除剧集或提高上限
core-phone-empty-search = 搜索内容为空
core-phone-no-results = 没有“{ $query }”的结果
core-phone-own-engine = 自带的种子下载引擎

## core/src/phone/check.rs

core-phone-check-connected = 已连接
core-phone-check-refused = 被拒绝：{ $error }
core-phone-check-dropped = 无响应（被丢弃）
core-phone-check-finding = { $label }：正在查找地址
core-phone-check-failed = 失败：{ $error }
core-phone-check-no-answer = 无响应
core-phone-check-private = { $label }：正在私密查找
core-phone-check-skipped-address = 已跳过，这是地址
core-phone-check-skipped-home = 已跳过，这是家庭网络名称
core-phone-check-no-record = 没有公共地址，交由此网络的 DNS 解析
core-phone-check-no-resolver = 没有解析服务器响应
core-phone-check-reaching = { $label }：正在连接 { $ip }
core-phone-check-http = HTTP { $status }，{ $bytes } 字节
core-phone-check-setup = 设置

## core/src/playback/session.rs

core-playback-not-saved = 《{ $folder }》的进度没有保存。请先匹配这部番剧。
core-playback-still-downloading = 第 { $number } 集仍在下载，已完成 { $done }%

## core/src/playback/skip.rs

core-playback-bad-duration = 时长必须是正数秒

## core/src/agent.rs

core-agent-no-answer = 助手没有给出答案
core-agent-searching = 正在网上搜索 { $query }
core-agent-searching-schedule = 正在网上搜索播出时间
core-agent-reading = 正在阅读 { $url }
core-agent-reading-page = 正在阅读网页
core-agent-using = 正在使用 { $tool }
core-agent-none = 这台电脑上的编程助手
core-agent-ended = { $name } 结束了，没有给出答案
core-agent-too-long = { $name } 用时太长

## core/src/core.rs

core-core-guest-trackers = 那台电脑上的追踪账号属于它的主人，所以这台设备不会在那里标记任何内容
core-core-guest-not-interested = 那台电脑上的“不感兴趣”是它主人的列表，所以这台设备不会在那里做任何更改
core-core-guest-recommended = 那台电脑上的推荐是根据它主人的列表生成的
core-core-nothing-picked = 未选择任何内容

## core/src/prefs.rs

core-prefs-not-a-provider = { $provider } 不是匹配服务
core-prefs-provider-count = { $provider } 出现了 { $count } 次；每个服务只能出现一次
core-prefs-outside = { $value } 不在 { $low } 到 { $high } 之间
core-prefs-empty = 空

## core/src/recommended/mod.rs

core-recommended-ranking = 正在为你排列番剧

## core/src/recommended/sync.rs

core-recommended-finding-people = 正在寻找口味相近的人
core-recommended-reading-lists = 正在读取他们的列表
core-recommended-reading-yours = 正在读取你的番剧
core-recommended-reading-suggestions = 正在读取推荐番剧

## core/src/recommended/page.rs

core-recommended-no-anime = { -anilist } 上没有动画 { $id }

## core/src/host/mod.rs

core-host-off = 关闭
core-host-starting = 正在启动
core-host-serving = 正在 { $addresses } 上提供服务
core-host-no-tailscale = 未连接 Tailscale
core-host-cannot-listen = 无法监听
core-host-port-taken = 端口 { $port } 已被占用
core-host-cannot-listen-on = 无法在 { $address } 上监听：{ $error }

## core/src/host/addr.rs

core-host-not-an-address = { $text } 不是地址和端口

## core/src/host/keys.rs

core-host-not-a-key = { $key } 不是设备密钥

## core/src/net/limiter.rs

core-net-timed-out = { $ms }ms 后超时
core-net-rate-limited = { $provider } 限制了请求频率

## core/src/net/mod.rs

core-net-waiting = { $host }：刚才超时了；{ -app-name } 会等待 { $seconds } 秒后再请求

## core/src/net/anilist.rs

core-net-no-data = 没有数据

## core/src/update/install.rs

core-update-no-download = { -app-name } { $version } 没有适用于这台电脑的下载
core-update-cannot-make = 无法创建 { $path }：{ $error }
core-update-cannot-mark = 无法把新的 AppImage 设为可执行：{ $error }
core-update-cannot-replace = 无法替换 { $path }：{ $error }
core-update-not-here = 这份副本不在这里更新
core-update-cannot-start = 无法开始下载：{ $error }
core-update-did-not-start = 下载没有开始：{ $error }
core-update-cannot-write = 无法写入 { $path }：{ $error }
core-update-stopped = 下载已停止：{ $error }
core-update-cancelled = 已取消
core-update-ended-early = 下载提前结束（{ $got } / { $size } 字节）
core-update-checksum = 下载内容与 GitHub 的校验和不符
core-update-cannot-move = 无法把下载内容移到位：{ $error }
core-update-cannot-open-dmg = 无法打开 dmg：{ $error }
core-update-cannot-read-dmg = 无法读取 dmg：{ $error }
core-update-no-app = dmg 中没有应用
core-update-cannot-copy = 无法把新应用复制到当前应用旁边：{ $error }

## core/src/calendar.rs

core-calendar-season-episode = { $title } - 第 { $season } 季第 { $number } 集
core-calendar-episode = { $title } - 第 { $number } 集

## core/src/paths.rs

core-paths-no-home = 没有主目录

## core/src/first_run.rs

core-first-run-not-absolute = 视频文件夹必须是绝对路径

## core/src/presence/mod.rs

core-presence-extra = 附加内容

## core/src/trackers/accounts.rs

core-trackers-no-client-id = 没有为 { $tracker } 设置客户端 ID。
core-trackers-mal-reconnect = 请在设置中重新连接 MyAnimeList。
core-trackers-mal-wants-secret = MyAnimeList 需要此应用的客户端密钥。请在设置的“追踪”中添加，或以 App Type“other”注册应用。
core-trackers-token-reply-unreadable = 无法读取令牌响应：{ $error }
core-trackers-token-reply-no-token = 令牌响应中没有访问令牌

## core/src/trackers/mod.rs

core-trackers-no-token = 没有保存访问令牌，请在设置中重新连接

## core/src/trackers/cache.rs

core-trackers-no-user-id = 账号中没有用户 ID

## core/src/trackers/move_up.rs

core-trackers-move-none-picked = 没有选择要移动的番剧
core-trackers-moved-up = 已把 { $count } 部番剧从这台电脑移到 { $site }。
core-trackers-moved-up-some-refused = 已把 { $moved } 部番剧移到 { $site }；{ $site } 没有接受 { $failed } 部。
core-trackers-move-refused = { $site } 没有接受《{ $show }》：{ $error }

## core/src/trackers/oauth.rs

core-trackers-oauth-timed-out = 授权超时，请重试。
core-trackers-oauth-state-mismatch = state 不匹配，可能是 CSRF 攻击，已中止。
core-trackers-oauth-no-code = 回调中没有 code
core-trackers-oauth-no-token = 回调中没有 access_token
core-trackers-oauth-waiting = 正在等待你在浏览器中登录 { $site }。
core-trackers-oauth-connected = 已以 { $username } 连接 { $site }。
core-trackers-oauth-no-name = 返回的个人资料没有名称
core-trackers-oauth-could-not-start = 无法开始登录
core-trackers-oauth-port-failed = 无法打开这台电脑上的端口 { $port }：{ $error }
core-trackers-oauth-port-busy = 这台电脑上的端口 { $port } 被其他程序占用。请稍等后重试，如果一直被占用，请关闭占用它的程序。
core-trackers-oauth-provider-error = { $site } 返回错误：{ $error }
core-trackers-page-connected = { $site } 已连接到 { $app }
core-trackers-page-close-tab = 你可以关闭此标签页了。
core-trackers-page-failed-title = { $app }，登录失败
core-trackers-page-failed = 登录没有完成
core-trackers-page-go-back = 请回到 { $app } 重试。
core-trackers-page-finishing-title = { $app }，仍在完成中
core-trackers-page-finishing = 仍在完成中
core-trackers-page-finishing-body = { $site } 已响应，AniBeam 正在完成登录。请回到 { $app }，完成后它会告诉你。
core-trackers-page-cancelled-title = { $app }，登录已取消
core-trackers-page-cancelled = 此次登录已取消
core-trackers-page-cancelled-body = 你可以关闭此标签页。如有需要，请从 { $app } 重新开始。

## core/src/trackers/on_hold.rs

core-trackers-on-hold-moving = 正在 { $tracker } 上移到“搁置”
core-trackers-on-hold-moved = 已在 { $site } 上把 { $count } 部番剧移到“搁置”。
core-trackers-on-hold-moved-two = 已把 { $count } 部番剧移到 { $site } 的“搁置”，{ $second } 部移到 { $secondSite } 的“搁置”。
core-trackers-on-hold-failed = 有 { $failed } 部番剧无法移到“搁置”。
core-trackers-on-hold-moved-failed = 已在 { $site } 上把 { $count } 部番剧移到“搁置”。有 { $failed } 部番剧无法移到“搁置”。
core-trackers-on-hold-moved-two-failed = 已把 { $count } 部番剧移到 { $site } 的“搁置”，{ $second } 部移到 { $secondSite } 的“搁置”。有 { $failed } 部番剧无法移到“搁置”。
core-trackers-series-fallback = 番剧 { $id }
core-trackers-back-on-watching = 《{ $name }》已在 { $site } 上回到“在看”。
core-trackers-back-on-watching-two = 《{ $name }》已在 { $site } 和 { $secondSite } 上回到“在看”。
core-trackers-back-on-watching-failed = 《{ $name }》无法回到“在看”，请稍后再试。

## core/src/trackers/writes.rs

core-trackers-rate-limited = { $tracker } 限制了请求频率，请一分钟后重试。
core-trackers-auth-expired = { $tracker } 授权已过期，请在设置中重新连接。
core-trackers-entry-not-found = 找不到 { $tracker } 条目。
core-trackers-server-error = { $tracker } 服务器错误（{ $status }），请稍后再试。
core-trackers-error-see-log = { $tracker } 出错，请查看活动日志。
core-trackers-episode-not-whole = { $episode } 不是大于零的整数集数
core-trackers-score-out-of-range = { $score } 不在 0 到 10 之间
core-trackers-rewatch-not-settable = “重看”不能手动设置
core-trackers-notes-too-long = 笔记最多 { $max } 个字符
core-trackers-nothing-picked = 未选择任何内容
core-trackers-notes-not-on-list = 还不在你的 { $tracker } 列表中，请先把它加入列表。
core-trackers-sites =
    { $sites ->
        [both] { -anilist } 和 { -mal }
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
       *[repeating] 重看
    }
core-trackers-marked = 已在 { core-trackers-sites } 上标记《{ $show }》第 { $episode } 集。
core-trackers-progress-set = 已在 { core-trackers-sites } 上把《{ $show }》的进度设为 { $progress }。
core-trackers-scored = 已在 { core-trackers-sites } 上给《{ $show }》评 { $score } 分。
core-trackers-score-cleared = 已在 { core-trackers-sites } 上清除《{ $show }》的评分。
core-trackers-status-set = 已在 { core-trackers-sites } 上把《{ $show }》设为“{ core-trackers-status }”。
core-trackers-notes-saved = 已把《{ $show }》的笔记保存到 { core-trackers-sites }。
core-trackers-notes-cleared = 已在 { core-trackers-sites } 上清除《{ $show }》的笔记。
core-trackers-update-failed = 无法更新 { $site }，请稍后再试。
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
        [yes] 《{ $name }》
       *[no] { $done } 部番剧
    }
core-trackers-written-done =
    { $action ->
        [clear] 已在 { core-trackers-sites } 上清除 { core-trackers-written-who } 的评分。
        [status] 已在 { core-trackers-sites } 上把 { core-trackers-written-who } 设为“{ core-trackers-status }”。
       *[score] 已在 { core-trackers-sites } 上给 { core-trackers-written-who } 评 { $score } 分。
    }
core-trackers-written-failed =
    { $hasReason ->
        [yes] 有 { $failed } 部番剧无法更新：{ $reason }。
       *[no] 有 { $failed } 部番剧无法更新。
    }
core-trackers-written-failed-more =
    { $hasReason ->
        [yes] 有 { $failed } 部无法更新：{ $reason }。
       *[no] 有 { $failed } 部无法更新。
    }
core-trackers-left-out =
    { $noMatch ->
        [0] {""}
       *[other] 有 { $noMatch } 部番剧没有匹配到追踪条目。
    }{ $hidden ->
        [0] {""}
       *[other] 有 { $hidden } 部番剧已隐藏，同步已关闭。
    }{ $notConnected ->
        [0] {""}
       *[other] 有 { $notConnected } 部番剧只在未连接的追踪服务上。
    }{ $kept ->
        [0] {""}
       *[other] 有 { $kept } 部番剧保存在这台电脑上，没有可发送的追踪服务。
    }

## core/src/metadata/automatch.rs

core-metadata-matched-to = 已把《{ $show }》匹配为《{ $title }》。
core-metadata-auto-matched = 已匹配 { $count } 部番剧。
core-metadata-auto-unmatched = 还有 { $count } 部番剧需要匹配。
core-metadata-auto-matched-some = 已匹配 { $matched } 部番剧；还有 { $unmatched } 部需要匹配。
core-metadata-find =
    { $matched ->
        [0] {""}
       *[other] { core-metadata-find-matched }
    }{ $unmatched ->
        [0] {""}
       *[other] { $first ->
            [unmatched] {""}
           *[other] {""}
        }{ core-metadata-find-unmatched }
    }{ $failed ->
        [0] {""}
       *[other] { $first ->
            [failed] {""}
           *[other] {""}
        }{ core-metadata-find-failed }
    }{ $had ->
        [0] {""}
       *[other] { $first ->
            [had] {""}
           *[other] {""}
        }{ core-metadata-find-had }
    }{ $busy ->
        [0] {""}
       *[other] { $first ->
            [busy] {""}
           *[other] {""}
        }{ core-metadata-find-busy }
    }
core-metadata-find-matched =
    { $hitNamed ->
        [yes] 已把《{ $hitFolder }》匹配为《{ $hitTitle }》。
       *[no] 已匹配 { $matched } 部番剧。
    }
core-metadata-find-unmatched =
    { $missNamed ->
        [yes] 没有找到《{ $missName }》的匹配。
       *[no] 有 { $unmatched } 部番剧没有找到匹配。
    }
core-metadata-find-failed = 有 { $failed } 部番剧无法搜索，请稍后再试。
core-metadata-find-had = 有 { $had } 部番剧已有匹配。
core-metadata-find-busy = 有 { $busy } 部番剧正在匹配中。

## core/src/metadata/apply.rs

core-metadata-link-unreadable = 无法读取该链接。请粘贴 AniList、MyAnimeList、TVmaze 或 IMDb 页面的网址。
core-metadata-no-mal-entry = AniList 上没有与该 MyAnimeList ID 对应的条目。
core-metadata-no-imdb-show = TVmaze 上没有与该 IMDb 作品对应的剧集。
core-metadata-search-on = 在 { $provider } 上搜索
core-metadata-query-too-short = 请至少输入两个字符再搜索。
core-metadata-not-a-link = 不是链接
core-metadata-refresh-unsupported = 刷新仅 MAL 或 TMDB 的匹配
core-metadata-matched = 已匹配《{ $show }》。
core-metadata-updated = 已更新《{ $show }》的信息。
core-metadata-update-failed = 无法更新《{ $show }》的信息，请稍后再试。
core-metadata-progress-refresh = 刷新
core-metadata-progress-backfill = 补全
core-metadata-updated-many = 已更新 { $count } 部番剧的信息。
core-metadata-update-failed-many = 无法更新 { $count } 部番剧的信息，请稍后再试。
core-metadata-updated-some = 已更新 { $count } 部番剧的信息；{ $failed } 部无法更新。

## core/src/metadata/fetch.rs

core-metadata-no-such-show = TVmaze 上没有剧集 { $id }
core-metadata-no-such-media = AniList 上没有作品 { $id }

## core/src/metadata/imdb.rs

core-metadata-progress-imdb-links = Wikidata 的 IMDb 链接
core-metadata-progress-imdb-ratings = IMDb 评分

## core/src/franchise/layout.rs

core-franchise-relation-character = 共同角色

## core/src/images/mod.rs

core-images-progress-fetching = 正在获取图片

## core/src/transfer/export.rs

core-transfer-exported = 已把 { $count } 部番剧保存到 { $folder }。

## core/src/transfer/format.rs

core-transfer-not-json = 不是 JSON：{ $error }
core-transfer-wrong-format = 不是 { $format } 文档

## core/src/transfer/import.rs

core-transfer-imported = 已从文件载入媒体库。

## core/src/library/delete.rs

core-library-no-such-file = 媒体库中没有文件 { $id }
core-library-file-elsewhere-not-removed = 文件 { $id } 属于另一部番剧，未移除任何内容
core-library-file-outside = { $path } 不是媒体库文件夹中的文件，未移除任何内容
core-library-episodes-removed =
    《{ $title }》的 { $count } 集{ $place ->
        [deleted] 已删除
       *[trash] 已移到回收站
    }。
core-library-finished-removed =
    { $count } 部看完的番剧{ $place ->
        [deleted] 已删除
       *[trash] 已移到回收站
    }。
core-library-watched-removed =
    { $series } 部番剧中已看的 { $count } 集{ $place ->
        [deleted] 已删除
       *[trash] 已移到回收站
    }。

## core/src/library/detail.rs

core-library-fix-one-file =
    这里唯一的文件被识别为{ $read ->
        [episode] 第 { $number } 集
       *[label] { $label }
    }。它看起来是{ $what ->
        [film] 剧场版
       *[episode] 唯一的一集
    }。
core-library-fix-specials = { $count } 个特别篇看起来是缺失的第 { $numbers } 集（共 { $episodes } 集）。
core-library-fix-numbered-on = 第 { $from } 集看起来是此条目的第 { $to } 集，沿用了上一季的编号。
core-library-runs-span = { $first } 至 { $last }
core-library-runs-list = { $init } 和 { $last }

## core/src/library/fixes.rs

core-library-fix-episode-range = 集数范围为 0 到 10000
core-library-fix-season-range = 季数最多为 200
core-library-fix-extra-range = 附加内容的序号最多为 10000
core-library-file-elsewhere-not-fixed = 文件 { $id } 属于另一部番剧，未修正任何内容
core-library-fix-undone = 已恢复按文件名识别《{ $title }》的 { $count } 个文件。
core-library-fixed = 已修正《{ $title }》中 { $count } 个文件的识别方式。

## core/src/library/reads.rs

core-library-hid-one = 已隐藏《{ $title }》。
core-library-back-one = 《{ $title }》已恢复显示。
core-library-hid-many = 已隐藏 { $count } 部番剧。
core-library-back-many = { $count } 部番剧已恢复显示。
core-library-episode-count-set = 《{ $title }》现在共 { $count } 集。
core-library-episode-count-cleared = 已清除《{ $title }》的集数。

## core/src/library/schedule.rs

core-library-schedule-cleared = 已清除《{ $title }》的排期备注。
core-library-schedule-saved = 已为《{ $title }》保存 { $count } 条排期备注。

## core/src/library/scan.rs

core-library-source-not-absolute = 来源路径必须是绝对路径
core-library-source-exists = 已是来源
core-library-source-nested = 位于已有来源之内
core-library-source-added = 已把文件夹 { $folder } 加入媒体库。
core-library-source-removed = 已从媒体库移除文件夹 { $folder }。
core-library-source-removed-with-shows = 已从媒体库移除文件夹 { $folder } 及其中的 { $count } 部番剧。
core-library-series-forgotten = 已从媒体库移除《{ $name }》。
core-library-source-back = 文件夹 { $folder } 已恢复。
core-library-source-unavailable = 文件夹 { $folder } 不可用。请检查所在磁盘是否已连接。
core-library-source-unreadable = 无法读取文件夹 { $folder }。请检查所在磁盘是否已连接。
core-library-source-looks-empty = 文件夹 { $folder } 看起来是空的，你的番剧保持不变。
core-library-source-reconcile-failed = 无法更新 { $folder } 中的番剧。
core-library-scan-done =
    已检查文件夹：{ $added ->
        [0] {""}
       *[other] 新增 { $added } 部番剧
    }{ $changed ->
        [0] {""}
       *[other] { $first ->
            [changed] {""}
           *[other] {"，"}
        }{ $changed } 部有变化
    }{ $removed ->
        [0] {""}
       *[other] { $first ->
            [removed] {""}
           *[other] {"，"}
        }{ $removed } 部缺失
    }。

## core/src/library/report.rs

core-library-agent-no-answer = { $span } 内没有响应
core-library-agent-stopped-code = 它以代码 { $code } 停止
core-library-agent-stopped = 它停止了
core-library-agent-answered-nothing = 它没有回答任何内容
core-library-span-minutes = { $count } 分钟
core-library-span-seconds = { $count } 秒
core-library-report-no-fix = 这些文件都没有可报告的修正
core-library-agent-no-triage = { $agent } 没有写分诊：{ $why }
core-transfer-progress-sources = 来源
core-transfer-progress-series = 番剧
core-transfer-progress-history = 历史
core-transfer-progress-accounts = 账号
core-transfer-progress-preferences = 偏好设置
core-transfer-progress-done = 完成

## core/src/downloads/anirss.rs

core-downloads-anirss-no-answer = anirss 没有响应：{ $message }
core-downloads-anirss-answered = anirss 返回 { $status }：{ $text }
core-downloads-no-free-port = 没有空闲端口：{ $error }
core-downloads-anirss-exited = anirss 已退出（{ $status }）
core-downloads-anirss-exited-said = anirss 已退出（{ $status }）：{ $said }
core-downloads-anirss-slow-start = anirss 没有在 { $seconds } 秒内启动
core-downloads-anirss-slow-start-said = anirss 没有在 { $seconds } 秒内启动：{ $said }
core-downloads-anirss-install-mac = 请安装 anirss { $version } 或更高版本：{ $command }
core-downloads-anirss-install-linux = 请安装 anirss { $version } 或更高版本：AUR 上的 anirss 包，或 { $command }
core-downloads-anirss-too-old = anirss { $version } 版本太旧，AniBeam 需要 { $minimum } 或更高版本。{ $update }。
core-downloads-anirss-lacks-feature = anirss { $version } 无法为每个种子客户端分别保存登录信息，而 AniBeam 切换客户端需要这项功能；anirss { $minimum } 及更高版本支持。{ $update }。
core-downloads-anirss-update-mac = 更新命令：{ $command }
core-downloads-anirss-update-linux = 请更新 anirss 包，或运行：{ $command }

## core/src/downloads/contents.rs

core-downloads-torrent-file-no-answer = 种子文件没有传来：{ $message }
core-downloads-torrent-file-answered = 种子文件返回 { $status }

## core/src/downloads/feed.rs

core-downloads-feed-answered = 订阅源返回 { $status }
core-downloads-not-a-feed = 不是订阅源：{ $error }

## core/src/downloads/jackett.rs

core-downloads-jackett-missing = 未安装 Jackett：找不到 { $program }。请安装它，或在设置中更改命令。
core-downloads-endpoint-start-failed = { $name } 无法启动：{ $why }
core-downloads-endpoint-start-timeout = { $name } 已启动，但没有在 { $seconds } 秒内响应。
core-downloads-command-exited = 命令以 { $status } 退出

## core/src/downloads/mod.rs

core-downloads-anirss-stopped = anirss 已停止
core-downloads-anirss-missing = { $install }。或在设置中指向一个正在运行的服务器。
core-downloads-anirss-command-missing = 找不到 { $command }。{ $install }。或在设置中指向一个正在运行的服务器。
core-downloads-anirss-start-failed = { $command } 无法启动：{ $error }
core-downloads-anirss-too-old-said = { $tooOld } 它说：{ $message }
core-downloads-no-torrent-client = { $address } 上没有 qBittorrent 响应，AniBeam 也没有自带的可运行。请安装 qBittorrent 并开启它的 Web UI，或安装 qbittorrent-nox，AniBeam 会替你运行它
core-downloads-own-client-gone = AniBeam 的 qBittorrent 不见了：未安装 qbittorrent-nox。请重新安装 AniBeam，或在设置中指向你自己的 qBittorrent
core-downloads-no-port = { $address } 中没有端口
core-downloads-own-client-settings = 无法写入 qBittorrent 的设置：{ $error }
core-downloads-own-client-start-failed = { $command } 无法启动：{ $error }
core-downloads-own-client-slow-start = AniBeam 的 qBittorrent 没有启动
core-downloads-own-client-slow-start-said = AniBeam 的 qBittorrent 没有启动：{ $said }
core-downloads-not-ready = 下载尚未就绪。请检查设置中的“下载”。
core-downloads-ready = 下载已就绪。
core-downloads-helper-sources = 下载现在由 AniBeam 自己搜索，不会读取搜索助手配置中的 { $count } 个来源。请在设置的“下载”中添加它们，或在那里选择搜索助手。
core-downloads-client-connected = 下载应用已连接。
core-downloads-client-not-connected = 下载应用未连接。请在设置中检查它的地址。
core-downloads-not-on-demand = { $name } 不是按需启动的端点
core-downloads-source-running = 下载来源正在运行。
core-downloads-source-not-running = { $name } 没有运行。请在来源中开启它并确认启动。
core-downloads-empty-query = 请输入要搜索的内容
core-downloads-first-endpoint = 第一个端点
core-downloads-search-starting = 正在开始搜索
core-downloads-search-starting-endpoint = 正在启动 { $endpoint }
core-downloads-search-searching-endpoint = 正在搜索 { $endpoint }
core-downloads-search-reading = 正在读取结果
core-downloads-subscribed = 已订阅《{ $name }》。
core-downloads-client-no-rule = qBittorrent 中没有名为 { $name } 的规则
core-downloads-trying-search = 正在尝试“{ $search }”
core-downloads-nothing-selected = 没有选择要下载的内容
core-downloads-added = 已添加 { $count } 个下载。
core-downloads-movie-link = 电影链接必须是磁力链接或 http(s) 网址
core-downloads-added-movie = 已把一部电影加入下载。
core-downloads-library-folder = { $folder } 是《{ $name }》的媒体库文件夹，其中有不是此订阅下载的剧集：请移除订阅但保留文件，或在《{ $name }》的页面中移除它
core-downloads-no-torrents-named = 没有指定种子
core-downloads-paused = 已暂停 { $count } 个下载。
core-downloads-resumed = 已继续 { $count } 个下载。
core-downloads-deleted-with-files = 已删除 { $count } 个下载及其文件。
core-downloads-removed-from-list = 已从列表移除 { $count } 个下载。
core-downloads-nothing-to-remove = 没有可移除的内容
core-downloads-removed-show = 已从下载中移除《{ $title }》。
core-downloads-removed-the-show = 已从下载中移除这部番剧。
core-downloads-deleted-still-listed = 已删除的文件仍在下载应用中列出。
core-downloads-feed-url = 订阅源必须是 http(s) 网址
core-downloads-rule-needs-name = 规则需要名称
core-downloads-no-rule = 没有名为 { $name } 的规则
core-downloads-feed-no-search = 该订阅源没有可缩短的搜索
core-downloads-no-repair = 《{ $name }》不需要修复。
core-downloads-repaired = 已修复《{ $name }》的订阅。
core-downloads-repair-failed = 更短的搜索也找不到任何内容；连“{ $search }”都没有结果
core-downloads-no-saved-client = 没有已保存的种子客户端 { $id }
core-downloads-client-switched = 下载现在使用另一个下载应用。
core-downloads-host-not-running = 没有在主机上运行
core-downloads-host-late = 没有及时响应

## core/src/downloads/native.rs

core-downloads-native-empty-query = 搜索为空
core-downloads-native-no-results = 没有 { $query } 的结果
core-downloads-native-feed-taken = 名为 { $name } 的订阅源已在读取另一个地址
core-downloads-native-no-items = 没有提供条目
core-downloads-native-bad-link = 链接必须是 magnet: 或 http(s) 网址，收到的是 { $link }
core-downloads-native-no-links = 没有提供链接
core-downloads-native-no-names = 没有提供订阅名称
core-downloads-native-files-elsewhere = 种子客户端把文件保存在另一台机器上，所以只能随种子一起移除
core-downloads-native-unknown-feeds = 未知的订阅源：{ $names }
core-downloads-native-outside-folders = 拒绝移除下载文件夹以外的文件：{ $paths }
core-downloads-refuse-home-folder = { $path } 是主文件夹或包含主文件夹，未移除任何内容
core-downloads-refuse-library-folder = { $path } 是媒体库文件夹或包含媒体库文件夹，未移除任何内容

## core/src/downloads/paths.rs

core-downloads-no-default-folder = qBittorrent 没有默认保存文件夹，请在 qBittorrent 中设置
core-downloads-name-empty = 名称不能为空
core-downloads-name-slash = 名称不能包含 / 或 \
core-downloads-name-not-folder = { $name } 不是文件夹名称

## core/src/downloads/prune.rs

core-downloads-pruned = 《{ $title }》已完结，已取消订阅。
core-downloads-prune-failed = 无法取消订阅《{ $title }》。请检查设置中的“下载”。

## core/src/downloads/trash.rs

core-downloads-refuse-not-absolute = { $path } 不是普通的绝对路径，未移除任何内容
core-downloads-refuse-root = { $path } 是文件系统根目录，未移除任何内容
core-downloads-refuse-outside-library = { $path } 不在媒体库文件夹内，未移除任何内容
core-downloads-refuse-save-folder = { $path } 是种子的保存文件夹，而不是它下载的内容，未移除任何内容

## core/src/downloads/watch.rs

core-downloads-episode-missed = 《{ $title }》第 { $episode } 集播出三天了仍未出现。你的订阅源会继续寻找。

## core/src/downloads/settings.rs

core-downloads-settings-command-empty = anirss 命令不能为空
core-downloads-settings-anirss-address = anirss 地址必须以 http:// 或 https:// 开头
core-downloads-settings-client-address = { $client } 地址必须以 http:// 或 https:// 开头
core-downloads-settings-bad-id = { $id } 不是有效的客户端 ID：请使用字母、数字、- 和 _
core-downloads-settings-same-id = 有两个客户端的 ID 都是 { $id }
core-downloads-settings-saved-address = { $name } 的地址必须以 http:// 或 https:// 开头
core-downloads-settings-same-name = 有两个客户端都叫 { $name }，请给每个起不同的名字

## core/src/downloads/qbt.rs

core-downloads-qbt-no-answer = { $address } 上的 qBittorrent 没有响应：{ $message }
core-downloads-qbt-password-rejected = qBittorrent 拒绝了密码，请在设置中修改
core-downloads-qbt-banned = 登录失败次数过多，qBittorrent 已封禁此地址；请等待，或重启 qBittorrent
core-downloads-qbt-login-rejected = qBittorrent 拒绝了用户名或密码
core-downloads-qbt-login-failed = { $address } 上的 qBittorrent 登录返回 { $status }；下次调用时会重试
core-downloads-qbt-not-web-ui = { $address } 返回了 { $status }，但不是 qBittorrent 的 Web UI。请检查设置中的地址，并确认 qBittorrent 已开启 Web UI
core-downloads-qbt-not-added = qBittorrent 没有添加这些种子：{ $said }
core-downloads-qbt-some-not-added = qBittorrent 有 { $count } 个种子无法添加：{ $said }
core-downloads-qbt-answered = qBittorrent 返回 { $status }：{ $line }

## core/src/downloads/replace.rs

core-downloads-replace-bad-item = 条目 { $item } 不在 { $count } 个资源之中
core-downloads-not-replaced = 第 { $episode } 集没有被替换。
core-downloads-replaced = 已用新下载替换第 { $episode } 集。

## core/src/downloads/rolling.rs

core-downloads-no-roll = 没有滚动下载 { $id }
core-downloads-rolled = 《{ $title }》：现在保留第 { $from } 至 { $to } 集，共 { $total } 集。
core-downloads-roll-ahead = 至少提前保留一集
core-downloads-roll-nothing = 没有可获取的内容

## core/src/releases/endpoints.rs

core-releases-json-needs-repo = JSON 搜索来源必须来自端点仓库，由其定义映射字段
core-releases-endpoint-no-name = 缺少 `name`
core-releases-endpoint-bad-name = { $name }：只能使用字母、数字、'.'、'_'、'-'
core-releases-endpoint-json = { $name }：{ $why }
core-releases-endpoint-unknown-kind = { $name }：未知类型 { $kind }（有效值：{ $valid }）
core-releases-endpoint-no-url = { $name }：缺少 `url`
core-releases-endpoint-not-http = { $name }：地址必须以 http:// 或 https:// 开头
core-releases-endpoint-needs-query = { $name }：{ $kind } 的 url 必须包含 {"{"}query{"}"} 占位符
core-releases-endpoint-no-query = { $name }：{ $kind } 的 url 会自行添加搜索词，所以不需要 {"{"}query{"}"}

## core/src/releases/fetch.rs

core-releases-indexer-error = 索引器返回错误 { $code }
core-releases-bad-feed = 订阅源有误：{ $error }
core-releases-feed-answered = 订阅源返回 { $status }

## core/src/releases/json_feed.rs

core-releases-not-feed-json = 不是订阅源的 JSON：{ $error }

## core/src/releases/json.rs

core-releases-not-json = 不是 JSON：{ $error }
core-releases-search-error-said = 搜索返回错误：{ $message }
core-releases-search-error = 搜索返回错误
core-releases-no-results-field = 响应中 { $path } 处没有结果
core-releases-results-not-list = 响应中的结果不是列表
core-releases-search-answered = 搜索返回 { $status }

## core/src/releases/native.rs

core-releases-no-endpoints = 还没有端点：请在“更多”的“端点”中添加仓库，或手动添加一个
core-releases-no-source-named = 没有名为 { $name } 的来源
core-releases-empty-query = 请输入要搜索的内容
core-releases-cannot-follow = 无法追更 { $name }：它能响应搜索，但没有可检查新剧集的订阅源
core-releases-endpoint-gone = { $name } 已不可用：已被移除、关闭，或缺少密钥

## core/src/releases/quirks.rs

core-releases-quirks-variant = one_variant { $word } 必须是一个简短的词
core-releases-quirks-pattern-count = show_feed 需要 1 到 { $most } 个页面模式
core-releases-quirks-pattern-long = show_feed 页面模式太长
core-releases-quirks-pattern-bad = show_feed 页面模式：{ $error }
core-releases-quirks-pattern-slug = show_feed 页面模式需要一个名为 slug 的分组
core-releases-quirks-url-https = show_feed url 必须是 https
core-releases-quirks-url-host = show_feed url 必须在端点自己的主机上
core-releases-quirks-url-slug = show_feed url 只能包含一次 {"{"}slug{"}"}，且不能有其他占位符

## core/src/releases/refinement.rs

core-releases-refine-no-tokens = 操作 'tokens' 需要非空的 `tokens` 列表

## core/src/releases/grid.rs

core-releases-group-batch = { $group } 的合集
core-releases-episodes-phrase = 第 { $list } 集
core-releases-episode-range = { $from } 至 { $to }

## Names that stay the same in every language

-app-name = AniBeam
-anilist = AniList
-mal = MyAnimeList

## Watch together (#846)

core-prefs-room-name = 名字须为 1 到 { $max } 个字符，且不含控制字符
core-prefs-room-relay = 中继地址必须以 https:// 或 http:// 开头
core-room-service = 一起看
core-room-not-found = 没有使用该代码的房间。房间可能已关闭。
core-room-full = 该房间已满。
core-room-proto = 该中继运行的是另一个版本的房间。请更新 { -app-name } 后再加入。
core-room-unreachable = 无法连接中继：{ $message }
core-room-refused = 中继拒绝了请求：{ $message }
core-room-not-in = 你不在房间中。
core-room-no-name = 请选择房间里其他人看到的名字
core-room-bad-code = 房间代码是 6 位字母和数字
core-room-created = 已创建一起看房间。房间代码是 { $code }。
core-room-joined = 已加入一起看房间 { $code }。
core-room-left = 已离开一起看房间。
core-room-member-joined = { $name } 加入了房间。
core-room-member-left = { $name } 离开了房间。
core-room-closed = 一起看房间已关闭。
core-room-lost = 与一起看房间的连接已断开，正在重试。
core-room-back = 已重新回到一起看房间。
