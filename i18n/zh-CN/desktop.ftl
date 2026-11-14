# zh-CN translation of en/desktop.ftl. An ID missing here is shown in English.

## Nav

nav-home = 首页
nav-library = 媒体库
nav-airing = 正在播出
nav-feed = 动态
nav-watching = 在看
nav-stats = 统计
nav-metadata = 番剧信息
nav-downloads = 下载
nav-settings = 设置
nav-series = 番剧
nav-player = 播放器
nav-show = 番剧
nav-recommended = 推荐

## StatusStrip

status-strip-errors = { $count } 个错误

## ScanSummary

scan-summary-nothing = 还没有可显示的内容
scan-summary-found = 找到 { $count } 部番剧，已识别 { $recognised } 部
scan-summary-found-need = 找到 { $count } 部番剧，已识别 { $recognised } 部，{ $need } 部需要你处理
scan-summary-episodes = 磁盘上有 { $count } 集。
scan-summary-episodes-watched = 磁盘上有 { $count } 集，其中 { $watched } 集已在 { $tracker } 上看过。
scan-summary-title = 文件夹已加入
scan-summary-count-found = 找到
scan-summary-count-recognised = 已识别
scan-summary-count-need = 需要处理
scan-summary-later = 稍后
scan-summary-match = 匹配这 { $count } 部
scan-summary-open-library = 打开媒体库

## UpdateNotice

update-notice-check-failed = Anibeam 无法检查更新。请检查网络连接后重试。
update-notice-up-to-date = Anibeam 已是最新版本
update-notice-downloading = 正在下载 Anibeam { $version }
update-notice-ready = Anibeam { $version } 已就绪
update-notice-failed = 更新失败
update-notice-on-aur = Anibeam { $version } 已上架 AUR
update-notice-out = Anibeam { $version } 已发布
update-notice-out-titled = Anibeam { $version }“{ $title }”已发布
update-notice-progress = { $percent }%  ·  { $got } / { $total } MB
update-notice-restart-hint = 立即重启，或下次打开时生效
update-notice-from-source = 这份副本由源码构建：请拉取并重新构建
update-notice-no-download = 这里没有适用于这台电脑的下载
update-notice-install-hint = 点“更新”会安装并重启 Anibeam
update-notice-retry = 重试
update-notice-update = 更新
update-notice-restart = 重启
update-notice-copied = 已复制
update-notice-copy = 复制
update-notice-release-page = 发布页
update-notice-later = 稍后

## WhatsNew

whats-new-title = 更新内容
whats-new-got-it = 知道了

## ActivityDrawer

activity-drawer-time-format = HH:mm:ss
activity-drawer-stamp-format = M月d日 HH:mm
activity-drawer-stage-library = 媒体库
activity-drawer-stage-metadata = 元数据
activity-drawer-stage-trackers = 追踪
activity-drawer-stage-franchise = 系列
activity-drawer-stage-playback = 播放
activity-drawer-stage-store = 存储
activity-drawer-stage-system = 系统
activity-drawer-stage-downloads = 下载
activity-drawer-level-info = 信息
activity-drawer-level-warn = 警告
activity-drawer-level-error = 错误
activity-drawer-copied-line = 已复制这一行
activity-drawer-title = 活动
activity-drawer-everything = 全部
activity-drawer-problems = 问题
activity-drawer-copied = 已复制
activity-drawer-copy = 复制
activity-drawer-clear-every = 清除所有记录
activity-drawer-copy-tip = 点击复制这一行
activity-drawer-fold = 收起
activity-drawer-empty-technical = 还没有记录。
activity-drawer-empty-problems = 没有问题，一切正常。
activity-drawer-empty = 还没有可报告的内容。Anibeam 为你做的事会显示在这里。

## ActivityColumn

activity-column-episode-run = 第 { $first } 至 { $last } 集
activity-column-episode-list = 第 { $list } 集
activity-column-downloads = { $count } 个下载
activity-column-batch = 合集下载
activity-column-download = 下载
activity-column-name-batch = 《{ $title }》合集
activity-column-name-episode = 《{ $title }》第 { $number } 集
activity-column-title = 活动
activity-column-down = 下载
activity-column-up = 上传
activity-column-not-connected = { $client } 未连接，这里不会有动静。
activity-column-connect-again = 重新连接
activity-column-downloading = 正在下载
activity-column-fewer = 收起
activity-column-more = 还有 { $count } 个
activity-column-nothing-downloading = 没有正在下载的内容。
activity-column-finished-today = 今天完成
activity-column-today = 今天
activity-column-today-empty = 搜索、下载和新订阅会实时显示在这里。
activity-column-stopping = 正在停止
activity-column-starting = 正在开始
activity-column-start-tip = 开始下载
activity-column-stop-tip = 停止下载，已下载的部分会保留
activity-column-cancelling = 正在取消
activity-column-cancel-all-tip = 取消这些下载
activity-column-cancel-tip = 取消此下载
activity-column-size-of = { $got } / { $total }
activity-column-files-missing = 文件缺失
activity-column-time-format = HH:mm
activity-column-left = 剩余 { $time }
activity-column-waiting-peers = 正在等待节点
activity-column-episodes-done = 已完成 { $done } / { $count } 集
activity-column-cube-skipped = 第 { $number } 集：已跳过
activity-column-cube-done = 第 { $number } 集：已完成
activity-column-cube-progress = 第 { $number } 集：{ $percent }%

## Frame

frame-window-title = { $page } - Anibeam
frame-strip-ready = 就绪
frame-strip-progress = { $doing }，{ $done } / { $total }
frame-strip-idle = 没有运行中的任务
frame-strip-time-format = HH:mm
frame-finish-sign-in = 请先完成 { $tracker } 登录
frame-did-not-work = 操作没有成功。
frame-could-not-open = Anibeam 无法打开它。请复制后自行打开。
frame-copy = 复制
frame-copied = 已复制
frame-status-refused = { $tracker } 没有接受
frame-status-refused-why = { $tracker } 没有接受：{ $why }
frame-lists-refused = { $tracker } 没有接受 { $count } 部番剧，请查看活动日志
frame-lists-moved = 已把 { $count } 部番剧移到 { $tracker }
frame-notes-refused = { $tracker } 没有接受你的笔记
frame-notes-refused-why = { $tracker } 没有接受你的笔记：{ $why }
frame-notes-saved = 笔记已保存
frame-connected = 已以 { $user } 连接 { $tracker }
frame-sign-in-stopped = { $tracker } 登录在完成前停止了。
frame-toast-hide-details = 隐藏详情
frame-toast-details = 详情
frame-toast-details-tip = 逐字显示出错内容，并可复制
frame-forward = 前进
frame-copy-magnet = 复制磁力链接
frame-copy-link = 复制种子链接
frame-magnet-copied = 已复制磁力链接
frame-link-copied = 已复制链接
frame-open-release-page = 打开资源页面
frame-open-folder = 打开文件夹
frame-copy-path = 复制路径
frame-path-copied = 已复制路径
frame-mark-unwatched = 从这里起标为未看
frame-mark-watched = 标为已看到这里
frame-copy-title = 复制标题
frame-open-on-anilist = 在 AniList 上打开
frame-list-home-online = 在每个已连接的列表上更改
frame-list-home-unmatched = 保存在这台设备上：没有追踪服务收录这部番剧
frame-list-home-not-carried = 保存在这台设备上：你的追踪服务没有收录这部番剧
frame-list-home-offline = 连接 AniList 或 MyAnimeList 之前，保存在这台设备上
frame-moving-to = 正在移到“{ $list }”
frame-moved-to = 已移到“{ $list }”
frame-plan-added = 《{ $title }》已加入“想看”列表
frame-plan-listed = 《{ $title }》已在你的列表中
frame-delete-watched = { $kind ->
    [film] 删除已看的剧场版
   *[episode] 删除已看的剧集
}
frame-delete-all = { $kind ->
    [film] 删除全部 { $count } 部剧场版
   *[episode] 删除全部 { $count } 集
}
frame-series-gone = 该番剧已不存在
frame-delete-none-watched = 磁盘上没有《{ $title }》已看的剧集
frame-delete-none = 磁盘上没有《{ $title }》的内容
frame-delete-files = { $watched ->
    [yes] { $kind ->
        [film] { $count } 部已看的剧场版
       *[episode] 已看的 { $count } 集
    }
   *[no] { $kind ->
        [film] { $count } 部剧场版
       *[episode] { $count } 集
    }
}
frame-delete-extras = { $count } 个附加内容
frame-delete-and = { $first }和{ $second }
frame-delete-heading = 删除《{ $title }》的{ $what }？
frame-delete-base = 文件及其字幕会移到回收站，可以从那里找回。它们的种子会从种子客户端移除，之后可以重新下载。
frame-delete-confirm = 移到回收站
frame-delete-done = 《{ $title }》的{ $what }已移到回收站
frame-delete-watched-done = { $count } 集，{ $size }，已移到回收站
frame-delete-nothing = 没有可删除的内容
frame-fix-not-drafted = 已修正。无法起草问题报告
frame-fix-agent = 已修正。{ $agent } 正在调查，完成后会打开问题报告
frame-fix-opening = 已修正。正在打开问题报告
frame-fix-not-saved = 修正没有保存：{ $error }
frame-fix-undone = { $count } 个文件已恢复按文件名识别
frame-fix-done = 已修正 { $count } 个文件的识别方式
frame-report-failed = 无法起草问题报告：{ $error }
frame-report-untriaged = 问题报告已在浏览器中打开，未经分诊。{ $error }
frame-report-open = 问题报告已在浏览器中打开，可以直接提交
frame-downloads-not-read = 未读取。

## HomePage

home-title = 首页
home-scan-started = 已开始扫描
home-ago-minutes = { $count } 分钟前
home-ago-hours = { $count } 小时前
home-ago-days = { $count } 天前
home-today = 今天
home-tomorrow = 明天
home-yesterday = 昨天
home-weekday-format = dddd
home-weekday-short-format = ddd
home-date-format = M月d日
home-day-date-format = M月d日 ddd
home-time-format = HH:mm
home-day-time-format = ddd HH:mm
home-next-time-format = dddd HH:mm
home-no-date-yet = 暂无日期
home-unit-days = 天
home-unit-hours = 小时
home-unit-minutes = 分钟
home-week-late = { $count } 集延迟了
home-week-waiting = { $count } 集已播出待看
home-week-coming = 未来七天有 { $count } 集
home-week-next = 下一集：《{ $title }》，{ $time }后
home-countdown-days = { $days }天{ $hours }小时
home-countdown-hours = { $hours }小时{ $minutes }分
home-countdown-minutes = { $minutes }分钟
home-out-not-downloaded = 尚未下载
home-out-downloading = 下载中
home-out-when = { $day } { $time }
home-next-up = 下一集
home-next-when = { $span }，{ $time }
home-next-when-estimated = { $span }，{ $time }，时间为估计
home-next-when-season = 第 { $season } 季，{ $span }，{ $time }
home-next-when-season-estimated = 第 { $season } 季，{ $span }，{ $time }，时间为估计
home-no-date = 无日期
home-tile-watched = 已看
home-tile-downloading = 下载中
home-tile-out = { $ago }播出
home-tile-not-out = 尚未播出
home-tile-estimated = 估计
home-tile-season-span = S{ $season } { $span }
home-tile-to-watch = { $count } 集待看
home-day-nothing = 无
home-now = 现在 { $time }
home-card-aired = { $episode }，{ $ago }播出
home-continue = 继续观看
home-continue-fact = { $count } 部番剧可以接着看
home-on-hold = 搁置
home-on-hold-fact = { $count } 部番剧等你回来
home-hero-open = 打开《{ $title }》
home-hero-pick-up = 从停下处继续
home-hero-up-next = 下一集
home-hero-downloading = 正在下载 { $count } 集
home-hero-min-left = 剩余 { $count } 分钟
home-hero-watched = 已看 { $watched } / { $total }
home-hero-resume = 继续第 { $number } 集
home-hero-play = 播放第 { $number } 集
home-hero-resume-unnumbered = 继续播放
home-hero-play-unnumbered = 播放
home-downloaded = 最近下载
home-downloaded-fact = 两周内有 { $count } 部番剧有新文件
home-week = 本周
home-week-show-empty = 显示空白日
home-week-hide-empty = 隐藏空白日
home-week-airing = 正在播出
home-week-out = 已播出
home-week-coming-up = 即将播出
home-recommended = 为你推荐
home-recommended-fact = 根据你的列表
home-recommended-scope = 推荐范围
home-recommended-all = 全部
home-recommended-in-library = 媒体库内
home-recommended-on-list = 列表内
home-recommended-none-library = 媒体库中已没有可推荐的番剧。“全部”会找出你还没有的番剧。
home-recommended-none-list = 列表中已没有可推荐的番剧。“全部”会找出你还没加入列表的番剧。
home-catch-up = 补番
home-catch-up-fact = { $count } 部番剧有剧集待看
home-fresh = 开新番
home-fresh-fact = { $count } 部番剧还没开始看
home-genres = 类型
home-open-library = 打开媒体库
home-dropped = 抛弃
home-dropped-fact = 磁盘上还有 { $count } 个文件
home-out-now = 已播出
home-out-now-fact = 本周新剧集，来自 AniList
home-popular = 本季热门
home-add-folder-title = 你自己的番剧也会显示在这里
home-add-folder-body = 添加存放动漫的文件夹，首页就会填满你在看的番剧。
home-add-folder-button = 添加文件夹
home-empty-scanned-title = 文件夹里还没找到内容
home-empty-title = 还没有番剧
home-empty-scanned-body = Anibeam 在它监视的文件夹里没有找到番剧或剧场版。请重新扫描，或在设置中检查文件夹。
home-empty-body = 添加存放动漫的文件夹，媒体库会自动填满。
home-scan-all = 全部扫描
home-library-settings = 媒体库设置
home-failed-title = 首页没有加载
home-failed-body = Anibeam 刚才无法读取媒体库。请稍后再试。

## Onboarding

onboarding-language = 语言
onboarding-step-welcome = 欢迎
onboarding-step-library = 媒体库
onboarding-step-lists = 你的列表
onboarding-step-downloads = 下载
onboarding-step-ready = 就绪
onboarding-scan-reading = 正在读取文件夹
onboarding-scan-found = 找到 { $count } 部番剧。随时可以继续，其余内容会自动补全。
onboarding-scan-starter-empty = 新文件夹目前是空的。放进去的番剧会自动出现在这里。
onboarding-scan-no-video = 没有找到视频文件。请确认文件夹是否正确。
onboarding-scan-your-shows = 你的番剧
onboarding-scan-looking-up = 正在查找{ $what }
onboarding-scan-looking-up-tally = 正在查找{ $what }，已找到 { $posters } / { $count }
onboarding-scan-posters = 正在获取海报
onboarding-scan-posters-tally = 正在获取海报，已找到 { $posters } / { $count }
onboarding-scan-seasons = 正在查找相关季度
onboarding-scan-seasons-tally = 正在查找相关季度，已找到 { $posters } / { $count }
onboarding-choose-folder = 选择你的动漫文件夹
onboarding-about = 设置只需一分钟，媒体库会自己完成剩下的部分。
onboarding-step-later = 稍后
onboarding-skip-setup = 跳过设置
onboarding-title-welcome = 欢迎使用 Anibeam
onboarding-title-library = 你的动漫在哪里？
onboarding-title-lists = 连接你的动漫列表
onboarding-title-downloads = 下载如何运作
onboarding-title-ready = 一切就绪
onboarding-title-later = { $count ->
    [1] 已就绪，还有一项稍后处理
    [2] 已就绪，还有两项稍后处理
   *[other] 已就绪，还有三项稍后处理
}
onboarding-body-welcome = Anibeam 把你的动漫集中在一处。它会找到磁盘上的番剧，直接在这里播放，并在你观看时同步更新 AniList 或 MyAnimeList。
onboarding-body-library = 选择存放番剧的文件夹。其中每个文件夹都会成为一部番剧，带有海报、剧集和你的观看进度。Anibeam 只读取文件夹，从不移动或重命名文件。
onboarding-body-lists = 登录一次，你在这里看完的每一集都会自动标记到列表上，不用手动输入。登录在浏览器中进行，Anibeam 永远看不到你的密码。
onboarding-body-downloads = 在“下载”页面追更一部番剧，每集新剧集播出后都会进入你的媒体库。这需要两样东西：可以搜索的地方，以及负责下载的种子客户端。
onboarding-body-ready = 这里的一切之后都能在设置中更改，在设置的“关于”中可以重新运行此设置。
onboarding-skip-step = 暂时跳过
onboarding-get-started = 开始使用
onboarding-open-app = 打开 Anibeam
onboarding-continue = 继续
onboarding-payoff-library = 你的媒体库
onboarding-payoff-library-line = 文件夹里的每部番剧都有海报、剧集和接下来要看的内容。
onboarding-payoff-plays = 直接在这里播放
onboarding-payoff-plays-line = 字幕、跳过片头，还会记住你停在哪里。
onboarding-payoff-list = 列表自动同步
onboarding-payoff-list-line = 看完一集，就会计入 AniList 或 MyAnimeList。
onboarding-payoff-episodes = 新剧集自动到来
onboarding-payoff-episodes-line = 追更一部番剧，每集新剧集都会进入你的媒体库。
onboarding-films-too = 存放电影的文件夹也行：把它命名为 Movies。
onboarding-choose-folder-button = 选择文件夹
onboarding-no-folder = 我还没有文件夹
onboarding-starter-where = 这会在 { $dir } 中创建一个 Anime 文件夹并使用它。
onboarding-add-another = 再添加一个文件夹
onboarding-shelf-more = 更多
onboarding-preview-show = 你的番剧
onboarding-preview-finished = 在这里看完第 3 集
onboarding-preview-on-list = 你的列表：已看 3 集
onboarding-preview-set-by = 片尾一出现，Anibeam 就会更新
onboarding-lists-connected = 已连接。你在这里看的内容现在会计入你的列表。
onboarding-lists-no-account = 还没有账号？两个都免费，有一个就够了。随时可以在设置的“账号”中断开连接。
onboarding-search-title = 可以搜索的地方
onboarding-search-ready = 有 { $count } 个来源可供搜索。
onboarding-search-none = Anibeam 本身不带任何来源。你需要从来源仓库添加：别人分享给你的一个链接，粘贴到设置的“下载”中即可。
onboarding-sources-after = 设置完成后打开
onboarding-sources-add = 设置完成后添加仓库
onboarding-fetch-title = 由谁下载
onboarding-fetch-auto = 交给 Anibeam
onboarding-fetch-recommended = 推荐
onboarding-fetch-auto-line = Anibeam 自带 qBittorrent，并在后台静默运行。无需安装，无需填写。
onboarding-fetch-existing = 我已经在用 qBittorrent
onboarding-fetch-existing-line = Anibeam 会连接你正在运行的 qBittorrent。接下来在设置中填写它的地址和登录信息。
onboarding-ready-library = 媒体库
onboarding-ready-library-none = 还没有文件夹。需要时可在设置的“媒体库”中添加。
onboarding-ready-library-series = { $count } 个文件夹，目前有 { $series } 部番剧
onboarding-ready-library-empty = { $count } 个文件夹，目前为空
onboarding-ready-library-scanning = { $count } 个文件夹，正在扫描
onboarding-ready-lists = 你的列表
onboarding-ready-lists-as = { $tracker }，账号 { $user }
onboarding-ready-lists-none = 尚未连接。需要时可在设置的“账号”中连接。
onboarding-ready-downloads = 下载
onboarding-ready-downloads-line = { $sources ->
    [0] 还没有来源
   *[other] { $sources } 个来源
}，{ $client ->
    [own] 使用你自己的 qBittorrent。
   *[bundled] 使用 Anibeam 自带的 qBittorrent。
}{ $next ->
    [ready] 一切就绪。
    [opens] 接下来会打开设置的“下载”。
   *[both] 两者都在设置的“下载”中。
}
onboarding-ready-open-downloads = 打开设置中的“下载”
onboarding-ready-set-up = 立即设置

## Rail.qml

rail-home = 首页
rail-library = 媒体库
rail-airing = 播出
rail-feed = 动态
rail-stats = 统计
rail-show-info = 番剧信息
rail-downloads = 下载
rail-settings = 设置
rail-downloads-complete = 下载：{ $count } 个订阅已完成
rail-title-language-name = 标题语言：{ $language }。点按切换到下一个
rail-title-language-english = 英语
rail-title-language-tip = 标题语言：罗马音、英语或日语
rail-whats-new-in = { $release } 更新内容
rail-whats-new = 更新内容
rail-settings-note = 设置：{ $note }

## FixSheet.qml

fix-kind-episode = 正片
fix-kind-opening = 片头曲
fix-kind-ending = 片尾曲
fix-kind-preview = 预告
fix-kind-special = 特别篇
fix-kind-other = 其他
fix-reads-season-episode = 第 { $season } 季第 { $number } 集
fix-reads-extra = { $kind } { $index }
fix-apply-failed = 无法应用修正。
fix-title = 修正文件识别
fix-lede = 《{ $title }》。说明每个文件实际是什么，之后每次扫描都会保留。
fix-rescan = 重新扫描文件夹
fix-rescan-started = 正在重新读取文件夹
fix-filled-in = 已填写
fix-fill-in = 填写
fix-column-file = 文件
fix-column-reads-as = 识别为
fix-column-should-be = 应为
fix-fixed = 已修正
fix-number-placeholder = 编号
fix-index-placeholder = 无
fix-keep = 保留修正
fix-leave-as-read = 保持当前识别
fix-read-by-name = 重新按文件名识别
fix-report = 在 GitHub 上提交问题
fix-report-body = 打开一份填好的问题报告供你提交。它是公开的，会列出这些文件名。
fix-report-body-agent = 打开一份填好的问题报告供你提交。它是公开的，会列出这些文件名。{ $agent } 会先调查一下。
fix-close-unapplied = 关闭而不应用 { $count } 项修正？
fix-number-invalid = 请先修正标红的数字
fix-close-anyway = 仍然关闭
fix-keep-editing = 继续编辑
fix-apply-count = 应用 { $count } 项修正

## MatchPanel.qml

match-matched-to = 已匹配为《{ $title }》
match-applied = 已应用匹配
match-found-tvmaze = 已在 TVmaze 上找到
match-found-anilist = 已在 AniList 上找到
match-found-anilist-season = 已在 AniList 上找到，第 { $season } 季
match-found-mal = 已在 MyAnimeList 上找到
match-sites-and = { $first } 和 { $next }
match-matched-on = 已在 { $sites } 上匹配
match-provider-silent = { $provider } 没有响应
match-title-walk = 匹配，{ $walk }
match-title = 匹配
match-skip = 跳过
match-refresh-info = 刷新信息
match-refreshing = 正在刷新《{ $title }》
match-clear-confirm = 确认清除：会移除关联和设置的集数
match-clear = 清除匹配
match-cleared = 已清除匹配
match-search-placeholder = 搜索 { $provider }
match-link-tvmaze = 粘贴 TVmaze 或 IMDb 链接
match-link-anilist = 粘贴 AniList 或 MyAnimeList 链接
match-look-up = 查找
match-searching = 正在搜索 { $provider }
match-type-more = 至少输入两个字才能搜索 { $provider }。
match-no-results = 在 { $provider } 上没有找到标题。

## WatchingPage.qml

watching-episode-code = EP { $number }
watching-countdown-days = { $days }天{ $hours }小时
watching-countdown-hours = { $hours }小时{ $minutes }分
watching-countdown-minutes = { $minutes }分钟
watching-on-disk = 磁盘上
watching-on-anilist = AniList 上
watching-ready-to-start = 可以开始
watching-not-in-library = 不在媒体库中
watching-get-to-start = 获取剧集后开始
watching-anilist-updated = AniList 更新于{ $when }
watching-airing = 播出中
watching-next-in = 下一集：《{ $title }》，{ $wait }后
watching-episodes-waiting = 有剧集待看
watching-series-behind = { $count } 部番剧落后
watching-up-to-date = 已追上
watching-anilist-only = 仅在 AniList 上
watching-planning-summary-off = 你在 AniList 上想看的番剧。
watching-planning-summary = 想看 { $count } 部番剧，磁盘上有 { $here } 部。
watching-summary-off = 你在 AniList 上在看或重看的番剧。
watching-summary-waiting = { $count } 部番剧有剧集待看。
watching-caught-up = 全部追上了。
watching-start-watching = 开始观看
watching-play-next = 播放下一集
watching-open-on-anilist = 在 AniList 上打开
watching-refreshing = 正在刷新
watching-connect-title = 未连接 AniList
watching-connect-body = 连接后，此页面会列出你在 AniList 上在看或重看的番剧：已有的可以直接播放，播出中的会显示倒计时。
watching-connect-waiting = 正在等待浏览器
watching-connect-anilist = 连接 AniList
watching-load-failed-title = “在看”列表没有加载
watching-load-failed-body = 无法连接 AniList。下次刷新时列表会恢复。
watching-planning-failed-title = “想看”列表没有加载
watching-planning-failed-body = 无法读取它的缓存。请重试，或等下次刷新时恢复。
watching-planning-empty-title = “想看”列表是空的
watching-planning-empty-body = 在“正在播出”中点“想看”，或在番剧的列表菜单中选“想看”，就会把番剧放到这里。
watching-airing-now = 正在播出
watching-empty-title = “在看”列表是空的
watching-empty-body = 在 AniList 上把番剧标记为“在看”，它就会出现在这里。
watching-open-anilist = 打开 AniList

## LibraryPage.qml

library-hidden = 已隐藏 { $count } 部番剧
library-hidden-sync-stops = 已隐藏 { $count } 部番剧，它们的追踪同步已停止
library-unhidden = { $count } 部番剧已恢复显示，并重新同步
library-undo = 撤销
library-clearing-score = 正在清除 { $count } 部番剧的评分
library-rating = 正在给 { $count } 部番剧评 { $score } 分
library-moving = 正在把 { $count } 部番剧移到“{ $list }”
library-rescanning = 正在重新扫描 { $count } 部番剧
library-finding-matches = 正在为 { $count } 部番剧查找匹配
library-sort-last-viewed = 最近观看
library-sort-downloaded = 最近下载
library-sort-title = 标题
library-sort-progress = 进度
library-sort-average-score = 平均分
library-sort-my-score = 我的评分
library-dir-a-to-z = A 到 Z
library-dir-z-to-a = Z 到 A
library-dir-oldest = 最早优先
library-dir-newest = 最新优先
library-dir-least-watched = 看得最少优先
library-dir-most-watched = 看得最多优先
library-dir-lowest = 最低优先
library-dir-highest = 最高优先
library-group-none = 不分组
library-group-season = 季度
library-group-type = 类型
library-group-genre = 题材
library-group-watch-state = 观看状态
library-scan-started = 已开始扫描
library-tab-all = 全部
library-tab-series = 番剧
library-tab-movies = 剧场版
library-tab-watched = 已看
library-tab-hidden = 已隐藏
library-count-movies = { $count } 部剧场版
library-count-watched = 已看 { $count }
library-count-hidden = 已隐藏 { $count }
library-count-series = { $count } 部番剧
library-title = 媒体库
library-all-genres = 所有题材
library-search-placeholder = 搜索标题或文件夹
library-sort = 排序
library-group = 分组
library-franchises-together = 同系列放一起
library-watched-shows = 已看的番剧
library-load-failed-title = 媒体库没有加载
library-no-matches = 没有与“{ $query }”匹配的内容
library-none-in-genre =
    { $what ->
        [series] 这里没有{ $genre }番剧
        [films] 这里没有{ $genre }剧场版
        [watched] 这里没有已看的{ $genre }番剧
        [hidden] 这里没有已隐藏的{ $genre }番剧
       *[shows] 这里没有{ $genre }番剧
    }
library-none-here =
    { $what ->
        [series] 这里没有番剧
        [films] 这里没有剧场版
        [watched] 这里没有已看的番剧
        [hidden] 这里没有已隐藏的番剧
       *[shows] 这里没有番剧
    }
library-load-failed-body = Anibeam 刚才无法读取媒体库。请稍后再试。
library-no-matches-body = 请检查拼写，或在“全部”中搜索整个媒体库。
library-none-in-genre-body = 当前视图中没有这个题材。“所有题材”会显示其余内容。
library-none-in-tab-body = 你的媒体库里还没有这类内容。“全部”会显示所有内容。
library-all-filtered-body = 媒体库中的所有内容都被这个视图筛掉了。
library-clear-search = 清除搜索
library-show-all = 显示全部
library-picked = 已选 { $count } 项
library-pick-all = 全选
library-pick-all-tip = 选择所有显示的卡片（Ctrl+A）
library-unhide = 取消隐藏
library-hide = 隐藏
library-rate = 评分
library-rate-tip = 为所有选中项在每个已连接的追踪服务上打同一个分
library-needs-tracker = 需要已连接的追踪服务和已匹配的番剧
library-move-to-list = 移到列表
library-rescan = 重新扫描
library-find-match = 查找匹配
library-find-match-tip = 为每个没有匹配的选中项搜索
library-all-matched = 所有选中项都已匹配
library-empty-scanned-title = 文件夹里还没找到内容
library-empty-title = 媒体库是空的
library-empty-scanned-body = Anibeam 在它监视的文件夹里没有找到番剧或剧场版。请重新扫描，或在设置中检查文件夹。
library-empty-body = 添加存放动漫的文件夹，媒体库会自动填满；也可以导入 Anibeam 导出文件。
library-scan-all = 全部扫描
library-add-folder = 添加文件夹
library-settings = 媒体库设置
library-import = 导入
library-import-title = 导入 Anibeam 导出文件
library-import-filter-export = Anibeam 导出文件 (*.json)
library-import-filter-all = 所有文件 (*)
library-import-started = 已开始导入

## MetadataPage.qml

metadata-title = 番剧信息
metadata-filter-all = 全部 { $count }
metadata-filter-series = 番剧 { $count }
metadata-filter-films = 剧场版 { $count }
metadata-filter-missing = 缺失 { $count }
metadata-scan-started = 已开始扫描
metadata-sort-added = 添加时间
metadata-sort-season = 季度
metadata-sort-a-to-z = A 到 Z
metadata-section-unmatched = 未识别
metadata-section-missing = 找不到文件夹
metadata-section-incomplete = 缺少剧集
metadata-section-empty = 未下载
metadata-section-airing = 播出中
metadata-section-complete = 完整
metadata-no-info = 还没有番剧信息
metadata-state-aired = { $have } / { $aired } 已播
metadata-state-of = { $have } / { $expected }
metadata-state-of-extra = { $have } / { $expected } +{ $extra }
metadata-detail-none-aired = 已播出的 { $aired } 集都不在磁盘上
metadata-detail-aired = 已播出 { $aired } 集，磁盘上有 { $have } 集
metadata-detail-aired-missing = 已播出 { $aired } 集，磁盘上有 { $have } 集，缺少 { $missing } 集
metadata-detail-none = { $expected } 集都不在磁盘上
metadata-detail-missing = 共 { $expected } 集，磁盘上有 { $have } 集，缺少 { $missing } 集
metadata-walk-position = { $at } / { $total }
metadata-walk-done = 这是最后一个
metadata-walk-left = “需要匹配”下还剩 { $count } 个
metadata-countdown-days = { $days }天{ $hours }小时
metadata-countdown-hours = { $hours }小时{ $minutes }分
metadata-countdown-minutes = { $minutes }分钟
metadata-next-in = { $wait }后更新下一集
metadata-files-of = { $have } / { $expected }
metadata-files-on-disk = 磁盘上的文件
metadata-forgot = 已移除《{ $title }》
metadata-refreshed-failed = 已刷新 { $refreshed } 部，{ $failed } 部无法刷新
metadata-refreshed = 已刷新 { $count } 部番剧
metadata-found = 找到 { $count } 部番剧
metadata-found-some-left = 找到 { $count } 部番剧，还有 { $unmatched } 部未识别
metadata-found-nothing = 没有找到新内容，还有 { $unmatched } 部未识别
metadata-all-recognised = 所有番剧都已识别
metadata-count = { $count } 部番剧
metadata-search-placeholder = 筛选标题
metadata-find-rest = 查找其余
metadata-find-rest-started = 正在查找 Anibeam 尚未识别的文件夹
metadata-find-rest-tip = 按设置中的顺序，在各个番剧信息网站上查找 Anibeam 尚未识别的每个文件夹
metadata-refresh-all = 全部刷新
metadata-refresh-all-started = 正在更新所有番剧的信息
metadata-refresh-all-tip = 重新获取每部已识别番剧的信息
metadata-crawling = 正在查找系列关联
metadata-crawl-progress = { $done } / { $total }
metadata-load-failed-title = 番剧信息没有加载
metadata-empty-scanned-title = 文件夹里还没找到内容
metadata-no-matches = 没有与“{ $query }”匹配的内容
metadata-nothing-under =
    { $filter ->
        [Series] “番剧”下没有内容
        [Movies] “剧场版”下没有内容
        [MissingFiles] “缺失”下没有内容
       *[other] “{ $filter }”下没有内容
    }
metadata-load-failed-body = Anibeam 刚才无法读取媒体库。请稍后再试。
metadata-empty-scanned-body = Anibeam 在它监视的文件夹里没有找到番剧。请重新扫描，或在设置中检查文件夹。
metadata-empty-body = 媒体库是空的。添加存放动漫的文件夹，Anibeam 会扫描它。
metadata-no-matches-body = 当前视图中没有标题包含这些词。
metadata-nothing-under-body = 目前没有番剧符合这个筛选条件。“全部”会显示其余内容。
metadata-scan-all = 全部扫描
metadata-add-folder = 添加文件夹
metadata-clear-search = 清除搜索
metadata-show-all = 显示全部
metadata-refreshing = 正在刷新《{ $title }》
metadata-refresh-tip = 重新获取这部番剧的信息
metadata-forget-armed = 确认移除
metadata-forget-confirm-tip = 再点一次，从 Anibeam 移除这部番剧及其观看记录
metadata-forget-tip = 磁盘上已没有这个文件夹：从 Anibeam 移除这部番剧及其记录

## AiringPage.qml

airing-title = 正在播出
airing-sort-popular = 人气
airing-sort-score = 评分
airing-sort-trending = 热度
airing-sort-favourites = 收藏数
airing-sort-soonest = 下一集
airing-sort-aired = 已播集数
airing-sort-episodes = 总集数
airing-sort-start = 开播日期
airing-sort-duration = 单集时长
airing-sort-title = 标题
airing-dir-most-popular = 最热门优先
airing-dir-least-popular = 最冷门优先
airing-dir-lowest = 最低优先
airing-dir-highest = 最高优先
airing-dir-least-trending = 热度最低优先
airing-dir-most-trending = 热度最高优先
airing-dir-fewest = 最少优先
airing-dir-most = 最多优先
airing-dir-soonest = 最近优先
airing-dir-latest = 最晚优先
airing-dir-oldest = 最早优先
airing-dir-newest = 最新优先
airing-dir-shortest = 最短优先
airing-dir-longest = 最长优先
airing-dir-a-to-z = A 到 Z
airing-dir-z-to-a = Z 到 A
airing-season =
    { $season ->
        [Winter] { $year } 年冬季
        [Spring] { $year } 年春季
        [Summer] { $year } 年夏季
        [Fall] { $year } 年秋季
       *[other] { $year } 年{ $season }
    }
airing-ago-minutes = { $minutes } 分钟前
airing-ago-hours = { $hours } 小时前
airing-ago-days = { $days } 天前
airing-episode-out = { $episode }，{ $ago }播出
airing-no-date = 暂无日期
airing-in-library = 已在库中
airing-out-today = 今天播出 { $count } 集
airing-dismissed-one = 《{ $title }》不会再出现在这里
airing-dismissed = { $count } 部番剧不会再出现在这里
airing-planned-already-one = 《{ $title }》已在你的列表中
airing-planned-already-all = 它们都已在你的列表中
airing-planned = { $count } 部番剧已加入“想看”列表
airing-planned-some-already = { $count } 部番剧已加入“想看”列表，{ $already } 部原本就在
airing-play-next = 播放下一集
airing-plan-to-watch = 想看
airing-get-more-episodes = 获取更多剧集
airing-get-episodes = 获取剧集
airing-open-on-anilist = 在 AniList 上打开
airing-not-interested = 不感兴趣
airing-not-interested-count = { $count } 部不感兴趣
airing-not-interested-tip = 你标记为不感兴趣的番剧，在设置中
airing-refreshing = 正在刷新
airing-pick-season = 选择其他季度
airing-filter-all = 全部
airing-filter-lists = 我的列表
airing-filter-library = 我的媒体库
airing-hide-lists = 隐藏我的列表
airing-hide-lists-tip = 排除你 AniList 或 MAL 列表中的番剧
airing-hide-library = 隐藏我的媒体库
airing-hide-library-tip = 排除你已有剧集的番剧
airing-out-now = 已播出
airing-out-last-day = 过去 24 小时播出 { $count } 集
airing-out-fewer = 收起
airing-out-all = 全部 { $count }
airing-sort = 排序
airing-load-failed-title = 无法连接 AniList
airing-load-failed-body = 正在播出的番剧来自 AniList。下次刷新时会恢复。
airing-reading-title = 正在读取播出中的番剧
airing-nothing-now = 目前没有播出中的番剧
airing-nothing-for = { $season }没有播出中的番剧
airing-nothing-read = 还没有读取
airing-reading-body = 所有播出中的番剧，来自 AniList。第一次需要几秒钟。
airing-nothing-body = AniList 还没有列出本季的番剧。刷新会重新读取，也可以选择其他季度。
airing-nothing-read-body = 刷新会从 AniList 读取所有播出中的番剧。
airing-hides-empty-title = 隐藏后这里什么都没有了
airing-lists-empty-title = 这里没有你列表中的番剧
airing-library-empty-title = 这里没有你媒体库中的番剧
airing-hides-empty-body = 这里的每部番剧都在你的列表或媒体库中，而你隐藏了它们。
airing-lists-empty-body = 在卡片上点“想看”，就会加入你的列表。
airing-library-empty-body = 在卡片上点“获取剧集”，就会下载到你的媒体库。
airing-show-everything = 显示全部
airing-picked = 已选 { $count } 项
airing-undo = 撤销

## EpisodeRow.qml

episode-row-date-time-format = M月d日 ddd HH:mm
episode-row-date-upcoming-format = M月d日 ddd
episode-row-date-format = yyyy年M月d日
episode-row-mark-unwatched-tip = { $state }。点击从这里起标为未看
episode-row-mark-watched-tip = { $state }。点击标为已看到这里
episode-row-next-up = 下一集
episode-row-versions = { $count } 个版本
episode-row-versions-tip = 同一集有 { $count } 个资源。点击选择播放哪个
episode-row-downloading = 下载中 { $percent }%
episode-row-get = 获取这一集
episode-row-predicted = 预计
episode-row-extra = 附加内容
episode-row-resume = 从 { $time } 继续

## EpisodeCubes.qml

episode-cubes-one = 一集
episode-cubes-range = { $count } 集，第 { $first } 至 { $last } 集
episode-cubes-tip = 第 { $number } 集：{ $state }

## EpisodesPane.qml

episodes-pane-extras = { $count } 个附加内容
episodes-pane-unmatched = { $count } 个未匹配文件
episodes-pane-on-disk = 磁盘上
episodes-pane-not-on-disk = 不在磁盘上
episodes-pane-season = S{ $season }
episodes-pane-get-tip = 获取第 { $number } 集
episodes-pane-none-on-disk = 磁盘上没有剧集
episodes-pane-change-match = 更改匹配
episodes-pane-refreshing = 正在刷新《{ $title }》

## ShowDetail.qml

show-detail-not-in-library = 这个文件夹还不在媒体库中。扫描找到它后，它的海报和剧集会显示在这里。
show-detail-film-on-disk = 剧场版已在磁盘上。
show-detail-film-not-on-disk = 剧场版还不在磁盘上。
show-detail-complete-line = 已完结，全部 { $total } 集都在这里。不会再有新剧集，订阅已无事可做。
show-detail-finished = 已完结。
show-detail-airing = 播出中。
show-detail-have-of-total = 这里有 { $have } / { $total } 集。
show-detail-have = 这里有 { $count } 集。
show-detail-next-predicted = { $span }预计在 { $wait }后播出。
show-detail-next-airs = { $span }将在 { $wait }后播出。
show-detail-missing = 第 { $list } 集不在这里。
show-detail-missing-more = 第 { $list } 集及另外 { $more } 集不在这里。
show-detail-downloaded = 已下载
show-detail-downloading = 下载中 { $percent }%
show-detail-taken-adding = 规则已选中，正在添加
show-detail-taken = 规则已选中
show-detail-left-out = 规则已排除
show-detail-subscribed-reading = 已订阅。qBittorrent 正在读取订阅源。
show-detail-taken-moving = 规则从订阅源的 { $releases } 个资源中选中了 { $taken } 个；{ $moving } 个正在下方下载。
show-detail-taken-held = 规则从订阅源的 { $releases } 个资源中选中了 { $taken } 个；客户端中有 { $torrents } 个种子。
show-detail-taken-adding-line = 规则从订阅源的 { $releases } 个资源中选中了 { $taken } 个；qBittorrent 正在添加。
show-detail-feed-holds-nothing = 订阅源中还没有内容。符合条件的资源一出现就会下载。
show-detail-nothing-fits = 订阅源中还没有符合规则的内容。符合条件的资源一出现就会下载。
show-detail-complete = 已完成
show-detail-open-in-library = 在媒体库中打开
show-detail-feed-finds-nothing = 此订阅源找不到任何内容
show-detail-feed-empty-query = 它向 { $source } 搜索“{ $query }”，结果为空，所以规则收不到新内容。更短的搜索通常会有结果；去掉的词可以改放在规则里。
show-detail-feed-empty = 它向 { $source } 查询，结果为空，所以规则收不到新内容。更短的搜索通常会有结果；去掉的词可以改放在规则里。
show-detail-feed-empty-earlier = qBittorrent 仍显示此订阅源早先版本找到的内容。
show-detail-repairing = 正在修复
show-detail-repair = 修复
show-detail-unsubscribe = 取消订阅
show-detail-remove-torrents = 移除 { $count } 个种子
show-detail-searches = 搜索
show-detail-quoted = “{ $words }”
show-detail-whole-feed = 整个订阅源
show-detail-from = 来源
show-detail-rule = 规则
show-detail-rule-unfollowable = 无法获取新剧集：它的来源能响应搜索，但没有订阅源
show-detail-rule-active = 生效中
show-detail-keeps = 保留
show-detail-keeps-value = 含“{ $words }”的资源
show-detail-drops = 排除
show-detail-drops-value = 含 { $words } 的资源
show-detail-or = { $first }或{ $second }
show-detail-last-took = 上次选中
show-detail-saves-to = 保存到
show-detail-folder = 文件夹
show-detail-in-the-feed = 订阅源内容
show-detail-feed-reading = 正在读取订阅源。
show-detail-feed-offers = 订阅源当前提供的内容，最新在前。
show-detail-feed-is-empty = 订阅源是空的。
show-detail-read-again = 重新读取
show-detail-seeders = { $count } 个做种
show-detail-download = 下载
show-detail-torrents = 种子
show-detail-torrents-moving = { $count } 个正在下载，进度条会随之填满。
show-detail-torrents-held = 客户端中属于这个文件夹的内容。
show-detail-torrents-waiting = 正在等待 qBittorrent 添加规则选中的内容。
show-detail-torrents-none = 客户端中没有属于这个文件夹的内容。

## SeriesPage.qml

series-trackers-both = { $first } 和 { $second }
series-count-cleared = 已清除集数
series-count-set = 已设为 { $count } 集
series-held-one-undated = 第 { $first } 集尚无日期
series-held-range-undated = 第 { $first } 至 { $last } 集尚无日期
series-held-one-due = 第 { $first } 集预计于 { $when } 播出
series-held-range-due = 第 { $first } 至 { $last } 集预计于 { $when } 播出
series-held-month-format = yyyy年M月
series-fix-schedule = 剧集和日期
series-fix-files = 文件
series-fix-rescan-folder = 重新扫描文件夹
series-fix-match = 匹配
series-find-match = 查找匹配
series-rescan-show = 重新扫描番剧
series-open-in-show-info = 在“番剧信息”中打开
series-delete-done =
    { $kind ->
        [film] { $count } 部剧场版已移到回收站
        [episode] { $count } 集已移到回收站
       *[file] { $count } 个文件已移到回收站
    }
series-delete-picked =
    { $kind ->
        [film] 已选 { $count } 部剧场版。
        [episode] 已选 { $count } 集。
       *[file] 已选 { $count } 个文件。
    }
series-delete-move-picked =
    { $kind ->
        [film] 把 { $count } 部剧场版移到回收站
        [episode] 把 { $count } 集移到回收站
       *[file] 把 { $count } 个文件移到回收站
    }
series-delete-click = 点击要删除的文件。
series-delete-where = 它们会移到回收站，可以从那里找回。观看记录仍会保留。
series-delete-move = 移到回收站
series-pick-none = 全不选
series-pick-all = 全选
series-rescan-started = 已开始重新扫描
series-rated = 已评 { $score } 分
series-rated-cleared = 已清除评分
series-score-failed = 评分失败
series-still-downloading = { $episode }仍在下载
series-format-series = 番剧
series-tracker-refused = { $tracker } 没有接受
series-tracker-refused-why = { $tracker } 没有接受：{ $why }
series-watched-up-to = 已看到{ $episode }
series-unwatched-from = 从{ $episode }起标为未看
series-undo = 撤销
series-play-downloading = 正在下载第 { $number } 集
series-play-downloading-unnumbered = 正在下载
series-play-continue = 继续
series-play-again = 重新观看
series-play-continue-episode = 继续第 { $number } 集
series-play-again-from = 从第 { $number } 集重新观看
series-play-episode = 播放第 { $number } 集
series-page-not-tracked = 未追踪
series-not-tracked-on-disk = 未追踪，磁盘上有 { $count } 集
series-watched = 已看 { $count } 集
series-watched-of = 已看 { $watched } / { $total }
series-watched-of-estimate = 已看 { $watched } / { $total }+
series-facts-episodes-estimate = { $count }+ 集
series-facts-each = 每集 { $runtime }
series-facts-on-disk = 磁盘上 { $count } 集
series-next = 下一集：{ $episode }
series-episodes-downloading = { $count } 集正在下载
series-all-watched = 全部已看
series-no-poster = 无海报
series-next-season-span = S{ $season } { $span }
series-next-predicted = { $span }预计于 { $date } 播出，{ $wait }后
series-next-date-format = M月d日 ddd
series-next-airs = { $span }将在 { $wait }后播出
series-downloading-episodes = 正在下载 { $count } 集
series-count-season-ends = 第 { $season } 季到第 { $number } 集结束
series-count-ends = 到第 { $number } 集结束
series-count-gave-way = 你设置的 { $count } 集已不再适用
series-count-set-by-you = { $line }，由你设置
series-change = 更改
series-get-episodes = 获取剧集
series-your-score = 我的评分 { $score }
series-rate = 评分
series-page-average-score = 平均分 { $score }
series-page-imdb = IMDb { $score }
series-page-imdb-votes = { $votes } 票
series-add-to-list = 加入列表
series-page-rewatched = 重看过 { $count } 次
series-your-notes = 你的笔记
series-add-notes = 添加笔记
series-notes-tip = 列表条目上的笔记，保存在 { $sites }
series-unhide = 取消隐藏
series-hide = 隐藏
series-hidden-toast = 《{ $title }》已在这里隐藏，Anibeam 不再把它同步到 AniList 和 MyAnimeList
series-unhidden-toast = 《{ $title }》已恢复显示，并重新同步
series-hide-tip-tracked = 在这里的所有页面中隐藏，Anibeam 不再把它同步到 AniList 和 MyAnimeList
series-hide-tip = 在所有列表中隐藏
series-fixes = 修正
series-fixes-tip = 修正文件、集数、匹配，或重新扫描
series-quick-fix = 快速修正
series-film = 剧场版
series-episodes = 剧集
series-extras = 片头曲、片尾曲及其他
series-extras-openings = 片头曲
series-extras-endings = 片尾曲
series-extras-previews = 预告和宣传片
series-extras-specials = 特别篇
series-extras-other = 其他
series-extras-group = { $kind }，{ $count }
series-extra-files = 多余文件
series-extra-files-warning = 有 { $count } 个文件超出预期的 { $total } 集：命名错误、重复或特别篇。请重命名或移除。
series-synopsis = 简介
series-less = 收起
series-more = 更多
series-match = 匹配
series-not-matched = 尚未匹配，所以没有海报、简介和追踪。请在 AniList 或 TVmaze 上查找它的番剧信息。
series-naming-title = 能匹配的命名
series-naming-body = 文件夹名是番剧标题，按 AniList 的写法，不是第一季时在后面加上季数。每个文件名包含集数。
series-fix-how-files-read = 修正文件识别
series-characters = 角色
series-related = 相关作品
series-related-filling = 仍在从 AniList 补全。
series-tags = 标签
series-hide-spoilers = 隐藏剧透
series-spoilers = 剧透
series-all-tags = 所有标签
series-tag = { $name }，{ $rank }%
series-recommendations = 推荐
series-relation-owned = 已在库中
series-relation-unreleased = 尚未发布
series-relation-on-anilist = AniList 上有
series-score-saved = 评分已保存
series-score-cleared = 评分已清除

## GetShow.qml

get-show-the-client = 种子客户端
get-show-group-batch = { $group } 的合集
get-show-run = { $first } 至 { $last }
get-show-episodes = 第 { $list } 集
get-show-episodes-inline = 第 { $list } 集
get-show-sending-to = 正在发送到 { $client }
get-show-go = 获取 { $count } 集
get-show-go-replace = 获取 { $count } 集并替换
get-show-go-subscribe = 获取 { $count } 集并订阅
get-show-go-replace-subscribe = 获取 { $count } 集，替换并订阅
get-show-subscribe = 订阅新剧集
get-show-nothing-new = 没有新内容可获取
get-show-pick-episode = 选择一集
get-show-replace-off = 已关闭。{ $episodes }的选择被排除，因为媒体库中已有。开启后可以替换。
get-show-replace-on = 新文件下载完成后，你现有的{ $episodes }文件会移到回收站。
get-show-replace-on-the-way = { $episodes }正在下载：该下载会被取消，已写入的内容会移到回收站。
get-show-kind-on-demand = 搜索需要时启动
get-show-kind-rss-search = RSS 搜索
get-show-kind-rss-feed = RSS 订阅源
get-show-kind-json-search = JSON 搜索
get-show-kind-indexer-search = 索引器搜索
get-show-ready = 就绪
get-show-running = 运行中
get-show-not-running = 未运行
get-show-not-running-start = 未运行。点击上方的来源即可启动。
get-show-switched-off = 已关闭
get-show-click-to-see = 点击查看它有这部番剧的哪些资源
get-show-searching = 搜索中
get-show-searching-source = 正在搜索 { $name }
get-show-searching-elapsed = { $label }，{ $seconds } 秒
get-show-releases = { $count } 个资源
get-show-releases-covered = { $count } 个资源，覆盖 { $cover } / { $total } 集
get-show-on-your-list = 已在列表中
get-show-plan-to-watch = 想看
get-show-on-your-list-tip = 它已在你的列表中
get-show-plan-to-watch-tip = 加入“想看”列表
get-show-open-series-tip = 番剧页面：详情、剧集和播放
get-show-next-in = 第 { $number } 集，{ $wait }后
get-show-airing = 播出中
get-show-finished = 已完结
get-show-total-episodes = 共 { $count } 集
get-show-downloading-episodes = 正在下载 { $count } 集
get-show-downloading-torrents = 正在下载 { $count } 个种子
get-show-in-library = 已在媒体库中
get-show-not-in-library = 不在媒体库中
get-show-downloading = 下载中
get-show-stop-all = 全部停止
get-show-start-all = 全部开始
get-show-stop-all-tip = 停止这部番剧的所有下载，已下载的部分会保留
get-show-start-all-tip = 重新开始这部番剧的所有下载
get-show-cancel-all = 全部取消
get-show-cancel-all-tip = 取消这部番剧的所有下载，并把已写入的内容移到回收站
get-show-sources = 可用来源
get-show-check-again = 重新检查
get-show-check-again-tip = 询问每个按需启动的来源是否在运行，不会启动任何服务。
get-show-search-source = 搜索 { $name }
get-show-release-list = 以列表显示所有资源
get-show-your-library = 你的媒体库
get-show-library-on-disk = 磁盘上 { $have } / { $total }
get-show-library-downloading = { $count } 集下载中
get-show-library-nothing = 还没有
get-show-row-batch = { $group }，合集
get-show-batch-counting = 正在统计集数，一个种子
get-show-batch-unlisted = 未列出剧集，按标题获取整部番剧，一个种子
get-show-batch-episodes = { $count } 集，一个种子
get-show-nothing-new-picked = 没有选择新内容
get-show-nothing-picked = 未选择任何内容
get-show-pick-provider = 选择来源
get-show-plan-line = 从 { $source } 获取 { $groups } 的{ $episodes }。
get-show-and = { $first } 和 { $second }
get-show-library-has = 媒体库中已有{ $episodes }。开启下方的“替换现有剧集”即可替换。
get-show-together = 合计
get-show-torrents-unit = 个种子
get-show-replace = 替换现有剧集
get-show-sequential = 按顺序下载
get-show-sequential-on = 开启：每个新种子从头开始下载分块，前面的剧集先下完，你可以边下边看。总体会稍慢一些。
get-show-sequential-off = 关闭：客户端按节点提供的任意顺序下载分块。
get-show-rolling = 保持后几集就绪
get-show-rolling-on = 开启：现在下载 { $count } 集，每看完一集就开始下载下一集。
get-show-rolling-on-delete = 开启：现在下载 { $count } 集，每看完一集就开始下载下一集。看过的剧集会从磁盘删除。
get-show-rolling-off = 关闭：所有选中的剧集同时下载。
get-show-follow-off = 关闭：新剧集会在这里显示为空缺，不会自动下载。
get-show-follow-no-feed = { $source } 没有可追踪的订阅源，所以新剧集无法从它自动下载。
get-show-follow-pick-group = 请先选择字幕组：一个订阅只追踪一个字幕组和画质。
get-show-follow-batch-only = { $group } 在这里只有合集，没有单集订阅源可订阅。
get-show-follow-on = 开启：在 { $source } 上订阅 { $feed }。每集新剧集出现后就会下载。
get-show-confirm = { $episodes }正在下载。替换会取消该下载，并把已写入的内容移到回收站。
get-show-confirm-go = 取消并替换
get-show-state-error = 错误
get-show-state-files-missing = 文件缺失
get-show-state-stopped = 已停止
get-show-state-waiting-peers = 正在等待节点
get-show-state-metadata = 正在获取元数据
get-show-state-queued = 排队中
get-show-state-checking = 正在校验文件
get-show-state-allocating = 正在分配空间
get-show-state-moving = 正在移动文件
get-show-state-waiting = 等待中
get-show-a-batch = 合集
get-show-no-group = 名称中没有字幕组
get-show-likely = 可能来自 { $source }
get-show-tracker = Tracker { $tracker }
get-show-not-known = 尚不清楚
get-show-bytes-of = { $done } / { $size }
get-show-speed-down = 下载 { $speed }/s
get-show-time-left = 剩余 { $wait }
get-show-stopping = 正在停止
get-show-starting = 正在开始
get-show-stop = 停止
get-show-start-tip = 重新开始下载
get-show-stop-tip = 停止下载，已下载的部分会保留
get-show-cancelling = 正在取消
get-show-cancel-tip = 取消此下载，并把已写入的内容移到回收站
get-show-episode-order = 按集数顺序下载
get-show-episode-order-on = 开启：从每个文件的开头下载分块，前面的剧集先下完。
get-show-cube-skipped = 第 { $number } 集：已跳过
get-show-cube-done = 第 { $number } 集：已完成
get-show-cube-percent = 第 { $number } 集：{ $percent }%
get-show-fact-torrent = 种子
get-show-fact-release = 资源
get-show-fact-from = 来源
get-show-fact-saving-to = 保存到
get-show-span-counting = 统计中
get-show-span-unknown = { $runs }？
get-show-span-episodes = 第 { $runs } 集
get-show-cube-no-release = 第 { $number } 集：{ $group } 没有资源
get-show-seeders = { $count } 个做种
get-show-cube-on-disk = 已在磁盘上，点击替换
get-show-cube-downloading = 正在下载，点击替换

## TorrentRow

torrent-row-state-downloading = 下载中
torrent-row-state-stalled = 停滞
torrent-row-state-metadata = 正在获取元数据
torrent-row-state-queued = 排队中
torrent-row-state-stopped = 已停止
torrent-row-state-done = 已完成
torrent-row-state-seeding = 做种中
torrent-row-state-checking = 校验中
torrent-row-state-moving = 移动中
torrent-row-state-error = 错误
torrent-row-state-files-missing = 文件缺失
torrent-row-state-making-room = 正在腾出空间
torrent-row-state-finding-peers = 正在寻找节点
torrent-row-state-waiting = 等待中
torrent-row-done-of-size = { $done } / { $size }
torrent-row-speed-down = 下载 { $speed }/s
torrent-row-speed-up = 上传 { $speed }/s
torrent-row-time-left = 剩余 { $time }
torrent-row-seeders = { $count } 个做种
torrent-row-rolling = 滚动下载，第 { $next } 至 { $through } 集，共 { $total } 集
torrent-row-stop = 停止

## SourceRow

source-row-unavailable-fix = 接上它所在的磁盘，然后点“重新扫描”。如果文件夹移动了，请移除它并添加新位置。
source-row-counts = { $count } 部番剧
source-row-counts-movies = { $count } 部番剧，{ $movies } 个电影文件夹
source-row-unavailable = 不可用
    .tip = Anibeam 无法访问这个文件夹。如果它在未接入或未挂载的磁盘上，接上后 Anibeam 会重新识别。如果文件夹移动了，请在这里移除它并添加新位置。
source-row-rescan = 重新扫描
source-row-remove-question = 移除 { $name }、其中 { $count } 部番剧及观看记录？

## StartSourceSheet

start-source-title = 为 { $name } 启动 { $service }？
start-source-busy = 正在启动 { $service }。它响应后 { $name } 就会开启，最多可能需要一分钟。
start-source-body = { $service } 是替 Anibeam 搜索 { $name } 的服务。启动后它会在这台电脑的后台运行，Anibeam 关闭后也会继续运行。
start-source-starting = 正在启动

## SearchProgress

search-progress-why-helper = Anibeam 正在启动搜索助手，需要一两秒。
search-progress-why-starting = { $endpoint } 通过一个按需启动的助手运行。第一次搜索要等它启动，最多 { $seconds } 秒；之后的搜索不用再等。
search-progress-why-reading = 正在把资源按字幕组分类，并与媒体库比对。
search-progress-why-ondemand = 每次新搜索时，助手都会重新读取 { $endpoint }，通常需要十到二十秒。
search-progress-why-slow = { $endpoint } 现在响应较慢。
search-progress-why-usual = 通常需要一两秒。
search-progress-why-and-slow = { $why } 这次比平时慢；可以取消后换个来源。
search-progress-searching = 搜索中
search-progress-seconds = { $seconds } 秒
search-progress-try = 试试 { $endpoint }

## ReleaseTable

release-table-col-release = 资源
release-table-col-group = 字幕组
release-table-col-res = 分辨率
release-table-col-rip = 片源
release-table-col-ep = 集
release-table-col-size = 大小
release-table-col-seeders = 做种
release-table-col-leechers = 下载者
release-table-col-downloads = 完成数
release-table-col-released = 发布时间
release-table-tick-all = 勾选所有资源
release-table-untick-all = 取消勾选所有资源
release-table-on-disk = 已在磁盘上
release-table-other-on-disk = 磁盘上有其他资源
release-table-season-batch = 第 { $season } 季合集
release-table-batch = 合集
release-table-via = 来自 { $source }
release-table-season-episode = S{ $season } { $episode }
release-table-date-format = yyyy年M月d日 HH:mm
release-table-download = 下载
release-table-more = 还有 { $count } 个。缩小范围或排序可以先看到其他资源。
release-table-pick = 选择此资源

## DownloadsPage

downloads-title = 下载
downloads-builtin-client = 内置客户端
downloads-start-done = { $name } 正在运行并已开启
downloads-disk-film = 剧场版
downloads-disk-watched = 已看 { $size }
downloads-disk-all-watched = 全部已看
downloads-delete-watched-menu = 所有已看剧集：{ $count } 集，{ $size }
downloads-delete-finished-menu = 所有看完的已完结番剧：{ $count } 部，{ $size }
downloads-titles-and = { $list }和{ $last }
downloads-titles-more = { $list }等 { $count } 部
downloads-delete-finished-none = 没有完整看完的已完结番剧
downloads-delete-watched-none = 磁盘上没有已看的内容
downloads-delete-finished-heading = 删除 { $count } 部已完结番剧，{ $size }？
downloads-delete-finished-base = { $titles }。只包括信息来源标为已完结、且每集都已看过的番剧；仍在播出或结局时间是估计的番剧会保留。它们的所有文件都会移到回收站，附加内容也一样。它们的种子也会从 { $client } 移除，之后可以重新下载。
downloads-delete-watched-heading = 删除 { $count } 集已看剧集，{ $size }？
downloads-delete-watched-base = 所有番剧中你看过的每一集。还没看的剧集和附加内容会保留。文件及其字幕会移到回收站，可以从那里找回。它们的种子也会从 { $client } 移除，之后可以重新下载。
downloads-move-to-trash = 移到回收站
downloads-remove-group-torrents = 移除它的 { $count } 个种子
downloads-run-span = { $from } 至 { $to }
downloads-run-pair = { $first } 和 { $second }
downloads-gap-missing = 缺少第 { $runs } 集
downloads-gap-not-downloaded = 第 { $runs } 集未下载
downloads-gap-then-not-downloaded = { $runs } 未下载
downloads-trying-query = 正在尝试 { $query }
downloads-client-settings = 种子客户端设置
downloads-error-unknown = 出了点问题，请重试。
downloads-search-all-off = 所有来源都已关闭。开启一个才能搜索。
downloads-turn-on = 开启 { $name }
downloads-search-no-sources = 还没有可搜索的来源。请在设置的“下载”中添加来源列表。
downloads-search-none-started = 没有来源能开始搜索，请稍后再试。
downloads-auto-resolution = 已筛选为 { $resolution }，可选的最高清晰度
downloads-search-one-failed = 在 { $name } 上的搜索没有成功。请重试，或换个来源搜索。
downloads-search-all-failed = 所有来源都无法搜索，请稍后再试。
downloads-narrowing-on = 正在 { $name } 上缩小范围
downloads-refine-zero = 这样会什么都不剩，所以列表保持原样。
downloads-refine-no-match = 没有资源匹配这个词。
downloads-refine-empty = 输入一个词来缩小范围。
downloads-refine-no-change = 这样不会有任何变化。
downloads-movie-tick-one = 勾选一个资源，然后点“作为电影下载”。
downloads-group-downloading = 下载中
downloads-group-downloading-line = 正在下载；右侧的“活动”有详情。
downloads-group-seeding = 做种中
downloads-group-seeding-line = 已下载，仍在分享。
downloads-group-done = 已完成
downloads-group-done-line = 已下载；客户端没有在分享。
downloads-group-stopped = 已停止
downloads-group-stopped-line = 在完成前停止了。
downloads-group-broken = 文件缺失
downloads-group-broken-line = 客户端找不到它们的文件：指向的文件夹不存在。移除这些种子可以整理客户端。
downloads-sub-nofeed = 无法追更
downloads-sub-nofeed-line = 它们的来源能响应搜索，但没有订阅源，所以不会有新剧集。请取消订阅，然后在有订阅源的来源上追更。
downloads-sub-complete = 已完成
downloads-sub-complete-line = 每一集都已在媒体库中，也不会再播出：这些订阅已无事可做。
downloads-sub-airing = 播出中
downloads-sub-airing-line = 还有新剧集要播出，最近的在前。
downloads-sub-incomplete = 不完整
downloads-sub-incomplete-line = 已完结但有剧集不在这里，或者无法得知番剧的进度。
downloads-sub-missing = 不在媒体库中
downloads-sub-missing-line = 它们的文件夹中没有媒体库找到的内容：还没有内容到来，或保存到了媒体库不监视的位置。
downloads-torrents = { $count } 个种子
downloads-rule-film-on-disk = 剧场版，已在磁盘上
downloads-rule-all-here = 全部 { $total } 集都在
downloads-rule-next =
    { $predicted ->
        [yes] 第 { $episode } 集预计 { $time }后
       *[no] 第 { $episode } 集 { $time }后
    }
downloads-rule-next-span =
    { $predicted ->
        [yes] 第 { $episode } 至 { $through } 集预计 { $time }后
       *[no] 第 { $episode } 至 { $through } 集 { $time }后
    }
downloads-rule-have-of = 这里有 { $have } / { $total } 集
downloads-rule-have = 这里有 { $count } 集
downloads-tag-not-downloaded = 未下载
downloads-tag-missing = 缺少 { $count } 集
downloads-tag-n-not-downloaded = { $count } 集未下载
downloads-tag-unknown = 未知
downloads-speed-down = 下载 { $speed }/s
downloads-time-left = 剩余 { $time }
downloads-repair-trying = 正在尝试更短的搜索
downloads-sent-to-client = 已把《{ $title }》发送到 { $client }
downloads-files-torrents = 种子下载的内容
downloads-files-torrents-remote = 种子下载的内容，由 { $client } 删除
downloads-files-folder-and = { $folder } 和{ $torrents }
downloads-unsubscribe-heading = 取消订阅《{ $title }》？
downloads-unsubscribe-base = 它的订阅源和下载规则会从 { $client } 移除，不会再有新内容。
downloads-unsubscribe-complete-heading = 取消全部 { $count } 部已完成番剧的订阅？
downloads-unsubscribe-complete-base = 它们的订阅源和下载规则会从 { $client } 移除。它们的每一集都已在媒体库中。
downloads-files-folders-and-torrents = 它们的文件夹和种子下载的内容
downloads-one-at-a-time = 会逐个处理，页面会显示每一步的进度。
downloads-subscription-stays = { $base } 订阅会保留。
downloads-cancel-heading = 取消《{ $title }》？
downloads-cancel-group-base = 它的 { $count } 个下载会停止，并从 { $client } 移除。
downloads-files-so-far-it = 它目前已下载的内容
downloads-files-so-far-they = 它们目前已下载的内容
downloads-cancel-download = 取消下载
downloads-keep-downloading = 继续下载
downloads-remove-heading = 移除《{ $title }》？
downloads-remove-group-base = 它的种子会从 { $client } 移除；除非你勾选下方选项，文件会保留。
downloads-remove-detail-base = 它的 { $count } 个种子会从 { $client } 移除；除非你勾选下方选项，文件会保留。
downloads-remove-broken-heading = 移除全部 { $count } 部文件缺失番剧的种子？
downloads-remove-broken-base = 它们的 { $count } 个种子会从 { $client } 移除。它们指向不存在的文件夹，所以不会动任何文件；订阅会保留。
downloads-remove-torrent-heading = 移除这个种子？
downloads-files-it-downloaded = 它下载的内容
downloads-cancel-torrent-heading = 取消此下载？
downloads-cancel-many-heading = 取消 { $count } 个下载？
downloads-cancel-many-base = 这部番剧的所有下载都会停止，并从 { $client } 移除。
downloads-cancel-all = 全部取消
downloads-summary-connecting = 正在连接。
downloads-summary-client-on = { $client } { $version } 已连接。
downloads-summary-client-off = { $client } 未连接。请在设置的“下载”中检查它的地址和密码。
downloads-summary-native = Anibeam 自己搜索你的来源。
downloads-summary-native-with = Anibeam 自己搜索你的来源。{ $client }
downloads-summary-starting = 正在启动下载。
downloads-summary-not-running = 下载没有运行。
downloads-summary-not-started = 下载还没有启动。
downloads-summary-helper-ready = 搜索助手 { $version } 已就绪。{ $client }
downloads-subscribed-as = 已订阅为“{ $name }”。
downloads-repaired = 已修复“{ $name }”：订阅源找到 { $count } 个资源
downloads-repaired-keeps = 已修复“{ $name }”：订阅源找到 { $count } 个资源，规则保留“{ $term }”
downloads-sent-torrents = 已把 { $count } 个种子发送到 { $client }。
downloads-added-torrents = 已添加 { $count } 个种子。
downloads-trashed-paths = 《{ $title }》的 { $count } 个路径已移到回收站
downloads-show-gone = 这部番剧已不在媒体库中
downloads-activity-hide = 隐藏活动
downloads-activity = 活动
downloads-moving = { $count } 个下载进行中
downloads-back = 返回下载
downloads-chip-search-ready = 搜索就绪
downloads-chip-search-starting = 搜索启动中
downloads-chip-search-off = 搜索已关闭
downloads-chip-answers-at = { $summary } 它在 { $url } 响应。
downloads-chip-helper-missing = 未安装搜索助手。点击打开下载设置，那里说明了如何添加
downloads-chip-helper-failed = 搜索助手已停止。“重新连接”会再次启动它；下载设置中说明了出错原因
downloads-chip-open-settings = 打开下载设置
downloads-chip-client-on = { $client } { $version }
downloads-chip-client-off = { $client } 已关闭
downloads-chip-client-not-connected = { $client } 未连接。请重新连接，或在设置中检查它的地址
downloads-chip-client-remote = 你自己运行的搜索助手会使用它自己设置中的客户端
downloads-chip-client-switch = 点击切换客户端
downloads-connecting = 正在连接
downloads-reconnect = 重新连接
downloads-search-placeholder = 搜索番剧或资源名称
downloads-search-go-tip = 在 { $names } 上搜索资源名称
downloads-search-go-off = 所有来源都已关闭。
downloads-suggest-airing =
    { $predicted ->
        [yes] 播出中，第 { $episode } 集预计 { $time }后
       *[no] 播出中，第 { $episode } 集 { $time }后
    }
downloads-suggest-airing-span =
    { $predicted ->
        [yes] 播出中，第 { $episode } 至 { $through } 集预计 { $time }后
       *[no] 播出中，第 { $episode } 至 { $through } 集 { $time }后
    }
downloads-suggest-finished = 已完结
downloads-suggest-episodes = { $count } 集
downloads-suggest-in-library = 已在媒体库中
downloads-suggest-on-anilist = AniList 上有，不在媒体库中
downloads-plan-to-watch = 想看
downloads-suggest-release-search = 改为在 { $names } 上搜索资源名称“{ $query }”
downloads-suggest-release-search-none = 改为搜索资源名称“{ $query }”，但没有可用来源
downloads-key-return = Enter
downloads-sources = 来源
downloads-source-hint-starting = 启动中
downloads-source-hint-down = 未运行
downloads-source-tip-down = { $name } 需要的辅助服务没有运行。点击启动。
downloads-source-tip-on = 搜索会询问 { $name }。点击关闭。
downloads-source-tip-off = 已关闭。点击重新询问 { $name }。
downloads-source-tip-url = { $tip } { $url }
downloads-check = 检查
    .tip = 询问每个按需启动的来源是否在运行，不会启动任何服务。
downloads-check-busy = 检查中
downloads-back-to-grid = 返回网格
    .tip = 回到《{ $title }》及各来源找到的内容。按 Esc 也可以。
downloads-releases-on = { $name } 上有 { $count } 个资源
downloads-found-on = 找到于
downloads-found-searching = 搜索中
downloads-found-failed = 失败
downloads-found-cancelled = 已取消
downloads-releases = { $count } 个资源
downloads-found-retry-tip =
    { $why }
    点击重试
downloads-facet-groups = 字幕组
downloads-facet-quality = 画质
downloads-facet-words = 关键词
downloads-filters-fewer = 收起筛选
downloads-filters-more = 更多筛选（{ $count }）
downloads-narrow-placeholder = 输入词语缩小范围，-词语 表示排除
downloads-best-match = 最佳匹配
    .tip = 保留看起来最符合的资源：{ $query }
downloads-best-match-none = 这里没有特别突出的资源
downloads-strip-have-of = 磁盘上有 { $have } / { $total } 集
downloads-strip-have = 磁盘上有 { $count } 集
downloads-strip-next =
    { $predicted ->
        [yes] 第 { $episode } 集预计 { $time }后
       *[no] 第 { $episode } 集 { $time }后
    }
downloads-strip-next-span =
    { $predicted ->
        [yes] 第 { $episode } 至 { $through } 集预计 { $time }后
       *[no] 第 { $episode } 至 { $through } 集 { $time }后
    }
downloads-strip-complete = 已完整
downloads-strip-line-more = { $disk }，{ $more }。
downloads-sentence = { $text }。
downloads-strip-none-on-disk = 这些资源都不是你已有的剧集。
downloads-strip-on-disk = 这里有 { $count } 个资源就是你已有的文件，标为“已在磁盘上”
downloads-strip-other-on-disk = 有 { $count } 个资源对应的剧集你已从其他资源获得
downloads-strip-both = { $first }；{ $second }。
downloads-tick-missing = 勾选缺少的 { $count } 集
    .tip = 为媒体库中缺少的每一集选择做种最多的资源
downloads-open-in-library = 在媒体库中打开
downloads-not-this-show = 不是这部番剧
downloads-no-releases = 没有资源
downloads-no-releases-body = { $name } 上没有与“{ $query }”匹配的内容。换个来源，或减少关键词，也许能找到。
downloads-try-source = 试试 { $name }
downloads-ticked = 已勾选 { $count } 个资源
downloads-nothing-ticked = 未勾选任何资源
downloads-ticked-size = 合计 { $size }。
downloads-tick-hint = 在表格中勾选资源，或全部下载。
downloads-tick-all = 全部勾选
downloads-working = 处理中
downloads-download-ticked = 下载 { $count } 个资源
downloads-download-shown = 下载显示的 { $count } 个
downloads-download-all = 全部下载 { $count } 个
downloads-download-movie = 作为电影下载
    .tip = 一个勾选的资源，直接存入电影文件夹
downloads-subscribe = 订阅
downloads-subscribe-body = 与此搜索匹配的新资源一出现就会下载到这里指定的文件夹。
downloads-name-placeholder = 取自第一个资源
downloads-subscribe-search = 订阅此搜索
downloads-subscribe-no-feed = 这里没有可订阅的订阅源
downloads-preview-again = 再次显示规则
downloads-preview-show = 显示它生成的规则
downloads-preview-from = 来自 { $host }
downloads-preview-from-named = 来自 { $host }（{ $name }）
downloads-preview-must = 必须包含 { $terms }
downloads-preview-must-not = 不能包含 { $terms }
downloads-helper-changed = 下载设置指向的助手没有响应
downloads-helper-missing = 这份 Anibeam 不带下载功能
downloads-helper-failed = 下载无法启动
downloads-helper-changed-body = 搜索和下载通过一个小助手程序运行。设置指向的不是 Anibeam 自带的那个，而它没有响应。切换回来，它就会自动启动。
downloads-helper-missing-body = 它在构建时没有包含搜索和下载所需的助手程序。发布页上的安装包带有它。设置的“下载”中说明了如何为这份副本添加。
downloads-helper-failed-body = 负责搜索和下载的助手程序在就绪前停止了。大多数情况下重新启动即可解决。
downloads-helper-use-own = 使用 Anibeam 自带的
downloads-helper-start-again = 重新启动
downloads-helper-show-how = 告诉我怎么做
downloads-helper-what-went-wrong = 出了什么问题
downloads-lens-gaps-count = 媒体库空缺  { $count }
downloads-lens-airing-count = 播出中  { $count }
downloads-lens-following-count = 追更中  { $count }
downloads-lens-downloaded-count = 已下载  { $count }
downloads-lens-gaps = 媒体库空缺
downloads-lens-gaps-line = 已播出但不在磁盘上的剧集。打开一部番剧，看看谁有资源。
downloads-lens-gaps-none = 没有缺失：每部番剧已播出的每一集都在磁盘上。
downloads-lens-airing-line = 媒体库中还有剧集要播出的番剧。打开一部即可追更。
downloads-lens-airing-none = 媒体库中没有播出中的番剧
downloads-lens-airing-none-body = 番剧在媒体库中且还有剧集要播出时，就会显示在这里。“正在播出”会列出所有播出中的番剧。
downloads-airing-now = 正在播出
downloads-lens-following = 追更中
downloads-card-next =
    { $predicted ->
        [yes] 第 { $episode } 集预计 { $time }后
       *[no] 第 { $episode } 集 { $time }后
    }
downloads-card-next-span =
    { $predicted ->
        [yes] 第 { $episode } 至 { $through } 集预计 { $time }后
       *[no] 第 { $episode } 至 { $through } 集 { $time }后
    }
downloads-rules-unread = 无法读取你的订阅。
downloads-lens-following-line = Anibeam 监视的订阅源，按它们填充的番剧显示。
downloads-none-yet = 还没有。
downloads-reading = 读取中。
downloads-rules-error = Anibeam 无法读取你的订阅，请稍后再试。
downloads-removing-to-go = 正在移除，还剩 { $count } 个
downloads-unsubscribe-all = 全部取消订阅 { $count } 个
downloads-unsubscribe = 取消订阅
downloads-no-rules = 没有订阅
downloads-no-rules-body = 在上方搜索并订阅，番剧就会持续更新。
downloads-search = 搜索
downloads-lens-downloaded = 已下载
downloads-dl-line = 磁盘上 { $size }。{ $rest }
downloads-dl-client-off = { $client } 未连接。
downloads-dl-unread = 无法读取下载列表。
downloads-dl-in-client = { $client } 中有 { $count } 个种子，{ $size }。
downloads-usage-watched = 已看
downloads-usage-not-watched = 未看
downloads-deleting = 正在删除
downloads-delete-watched = 删除已看
downloads-remove-all = 全部移除 { $count } 个
downloads-on-disk = 磁盘上
downloads-on-disk-line = 在媒体库中但 { $client } 里没有种子的内容，最大的在前。右键点击即可删除。
downloads-nothing-downloaded = 还没有下载
downloads-nothing-downloaded-body = 你下载或订阅的内容会按番剧显示在这里。
downloads-client-off-title = { $client } 未连接
downloads-client-off-body = 请确认它正在运行，然后重新连接。它的地址和密码在设置的“下载”中。
downloads-connect-again = 重新连接
downloads-view-grid = 网格
downloads-view-list = 列表
downloads-chip-subscribed = 已订阅
downloads-speed-up = 上传 { $speed }/s
downloads-broken-some = { $count } 个文件缺失
downloads-tag-watched = 已看

## PlayerDelays.qml

player-delays-title = 延迟
player-delays-subtitles = 字幕
player-delays-audio = 音频
player-delays-subtitles-earlier = 字幕提前 0.1 秒
player-delays-subtitles-later = 字幕延后 0.1 秒
player-delays-audio-earlier = 音频提前 0.1 秒
player-delays-audio-later = 音频延后 0.1 秒
player-delays-reset = 重置为 0，直到下一个文件
player-delays-none = 无延迟

## TrackPicker.qml

track-picker-off = 关闭

## RollControls.qml

roll-controls-window = 保持 { $ahead } 集就绪：第 { $next } 至 { $through } 集，共 { $total } 集
roll-controls-window-watched = 保持 { $ahead } 集就绪：第 { $next } 至 { $through } 集，共 { $total } 集，已看 { $watched } 集
roll-controls-more = 再获取 { $count } 集
roll-controls-more-tip = 现在就下载后面的剧集。你追上后，Anibeam 会继续保持后几集就绪。
roll-controls-fewer-tip = 少预留一集
roll-controls-ahead = 预留 { $count } 集
roll-controls-further-tip = 多预留一集
roll-controls-all = 全部获取
roll-controls-all-tip = 不再只预留几集，下载剩下的所有剧集
roll-controls-all-question = 下载剩下的全部 { $count } 集？

## PlayerChrome.qml

player-chrome-delays = 字幕和音频延迟
player-chrome-audio = 音轨
player-chrome-subtitles = 字幕
player-chrome-subtitles-off = 关闭字幕
player-chrome-casting-to = 正在投屏到 { $device }
player-chrome-cast = 投屏到电视
player-chrome-mark-watched = 标为已看
player-chrome-shortcuts = 键盘快捷键
player-chrome-fullscreen = 全屏
player-chrome-exit-fullscreen = 退出全屏
player-chrome-stay = 留下
player-chrome-skip-intro = 跳过片头
player-chrome-skip-outro = 跳过片尾
player-chrome-playing-on = 正在 { $device } 上播放
player-chrome-replay = 重播

## PlayerPage.qml

player-title = 播放器
player-next = 下一集
player-extra-no-next = 附加内容没有下一集
player-last-episode = 这是最后一集
player-extra-no-previous = 附加内容没有上一集
player-first-episode = 这是第一集
player-resumed-at = 从 { $time } 继续
player-start-over = 从头开始
player-audio-track = 音频：{ $track }
player-subtitles-track = 字幕：{ $track }
player-subtitles-on = 字幕：开启
player-subtitles-off = 字幕已关闭
player-no-subtitles = 这个文件没有字幕
player-delay-seconds = { $value } 秒
player-audio-delay = 音频延迟 { $value }
player-subtitle-delay = 字幕延迟 { $value }
player-volume = 音量 { $volume }
player-speed = 速度 { $speed }×
player-not-while-casting = 投屏时不可用
player-hold-fast = 按住时 2×
player-no-chapters = 这个文件没有章节
player-last-chapter = 这是最后一章
player-chapter = 第 { $number } 章
player-chapter-titled = 第 { $number } 章：{ $title }
player-screenshot-saved = 已保存到 Pictures/AniBeam
player-skipped-intro = 已跳过片头
player-skipped-outro = 已跳过片尾
player-undo = 撤销
player-refusal-hidden = 这部番剧已隐藏
player-refusal-no-match = 这部番剧尚未匹配，请在“番剧信息”中匹配
player-refusal-not-newer = 你的列表进度已超过这一集
player-refusal-extra = 这是附加内容
player-refusal-unmatched = 这个文件没有匹配到剧集
player-refusal-on-disk = 这个文件只在磁盘上
player-not-tracked-because = 未追踪：{ $reason }。
player-outcome-said = { $tracker }：{ $words }
player-outcome-failed = 无法更新 { $tracker }
player-outcome-ok = { $tracker } 已更新
player-outcome-at = { $tracker } 进度 { $progress }
player-outcome-tracked = 已追踪
player-outcome-not-tracked = 未追踪
player-outcome-could-not-update = 无法更新你的列表
player-outcome-tracked-list = 已追踪：{ $list }
player-outcome-not-tracked-list = 未追踪：{ $list }
player-outcome-failed-list = 无法更新你的列表：{ $list }
player-rated = 已评 { $score } 分
player-score-failed = 无法保存评分
player-frame = 第 { $number } 帧，{ $time }
player-audio-title = 音频
player-subtitles-title = 字幕
player-cast-to = 投屏到
player-cast-looking = 正在查找 Chromecast 和电视
player-cast-unreachable = 无法连接电视
player-cast-tv = 电视
player-cast-stop = 停止投屏到 { $device }
player-cast-connecting = 正在连接
player-cast-loading = 正在加载
player-cast-playing = 正在播放
player-cast-paused = 已暂停
player-cast-finished = 已结束
player-cast-stopped = 已停止
player-rate-prompt = 已追踪最后一集。给这部番剧评个分？
player-rate = 评分
player-not-now = 以后再说

## EndpointReposPanel.qml

endpoint-repos-panel-title = 来源列表
endpoint-repos-panel-helper = 来源列表是一组经过签名的搜索位置，通过地址添加。Anibeam 会检查签名，并在保存前向你展示密钥，然后你可以安装需要的来源。已安装的来源会加入上方的“搜索来源”。
endpoint-repos-panel-could-not-start = Anibeam 无法启动它，请稍后再试。
endpoint-repos-panel-needs-value = { $name } 需要填写值
endpoint-repos-panel-values-of = { $name } 的值
endpoint-repos-panel-secret-note = 保存在这台机器的密钥存储中，不会进入媒体库或导出文件。
endpoint-repos-panel-secret-note-keep = 保存在这台机器的密钥存储中，不会进入媒体库或导出文件。留空的字段会保留已存储的值。
endpoint-repos-panel-asks-again = { $name } 版本 { $version } 再次请求填写
endpoint-repos-panel-update-sends-to = 新版本会把它发送到 { $host }。
endpoint-repos-panel-fields-changed = 它的字段有变化。
endpoint-repos-panel-needs-fields = 需要先填写 { $fields } 才能搜索。
endpoint-repos-panel-no-longer-offered = 它的来源列表已不再提供。
endpoint-repos-panel-update-requests-to = 更新后会向 { $host } 发送请求。
endpoint-repos-panel-unreadable = 无法读取此列表。“刷新列表”会重试。
endpoint-repos-panel-add-list = 添加来源列表
endpoint-repos-panel-add-list-helper = 索引文件的地址，或这台机器上的文件路径。
endpoint-repos-panel-checking = 检查中
endpoint-repos-panel-add = 添加
endpoint-repos-panel-trust-question = 信任 { $name }？
endpoint-repos-panel-fingerprint = 密钥指纹  { $fingerprint }
endpoint-repos-panel-preview =
    { $url }
    { $count } 个来源，已签名。它的来源决定你的搜索发往何处；只信任来自你信任的人的列表。
endpoint-repos-panel-preview-trusted =
    { $url }
    { $count } 个来源，已签名，已信任。它的来源决定你的搜索发往何处；只信任来自你信任的人的列表。
endpoint-repos-panel-trust = 信任
endpoint-repos-panel-saving = 保存中
endpoint-repos-panel-install = 安装
endpoint-repos-panel-update = 更新
endpoint-repos-panel-added-by-you = 由你添加
endpoint-repos-panel-version = v{ $version }
endpoint-repos-panel-update-to = 更新到 v{ $version }
endpoint-repos-panel-add-value = 填写它的值
endpoint-repos-panel-change = 更改 { $fields }
endpoint-repos-panel-asks-for = 需要你的 { $fields }。
endpoint-repos-panel-by-hand = 手动添加
endpoint-repos-panel-by-hand-helper = 适用于没有列表收录的站点：任何在搜索词位置写上 {"{"}query{"}"} 的订阅源都行，包括搜索代理提供的订阅源。地址中的密钥会单独保存，有系统密码存储时就存放在那里。
endpoint-repos-panel-name = 名称
endpoint-repos-panel-refreshing = 正在刷新
endpoint-repos-panel-refresh = 刷新列表

## ScheduleSheet.qml

schedule-episodes-range = 第 { $first } 至 { $last } 集
schedule-day-format = yyyy年M月d日 ddd
schedule-month-format = yyyy年M月
schedule-tile-day-format = M月d日
schedule-tile-month-format = yyyy年M月
schedule-some-time-in = { $year } 年内
schedule-no-date-yet = 暂无日期
schedule-tile-guess = ≈ { $date }
schedule-tile-no-date = 无日期
schedule-tile-last = 最终
schedule-tile-out = 已播
schedule-ends-at = 到第 { $number } 集结束
schedule-dated-by-site = { $count } 集日期来自 { $site }
schedule-dated-by-record = { $count } 集日期来自条目
schedule-set-by-you = { $count } 集由你设置
schedule-guessed-weekly = { $count } 集按每周推算
schedule-nothing-set = 还没有设置
schedule-saved = 已保存
schedule-cleared = 已清除排期备注
schedule-title = 剧集和日期
schedule-season = 第 { $number } 季
schedule-ask-agent = 询问 { $agent }
schedule-done-pasting = 粘贴完成
schedule-paste-list = 粘贴列表
schedule-agent-looking = { $agent } 正在查找日期。点“保存”之前不会保存任何内容。
schedule-agent-stopped = { $agent } 已停止：{ $error }
schedule-stop = 停止
schedule-paste-placeholder = 粘贴放送列表、表格或帖子
schedule-paste-helper = Anibeam 读取到的内容会显示在下方的剧集上。
schedule-agent-hint = 这台电脑上有 Claude Code 时，Anibeam 可以让它帮你查找日期。
schedule-show-more = 显示更多剧集
schedule-pick-hint = 点击一集为它设置日期。按住 Shift 点击可选择一段。
schedule-picked-one = 第 { $number } 集：
schedule-picked = { $count } 集：
schedule-date-placeholder = 1月7日、2027年3月、2027
schedule-set-date = 设置日期
schedule-weekly = 每周
schedule-not-last = 不是最后一集
schedule-last-episode = 最后一集
schedule-clear-date = 清除日期

## ShowPage.qml

show-title = 番剧
show-runtime-each = 每集 { $runtime }
show-add-to-list = 加入列表
show-moving-to = 正在移到“{ $list }”
show-moved-to = 已移到“{ $list }”
show-relation-related = 相关
show-no-poster = 无海报
show-reading-anilist = 正在读取 AniList
show-not-in-library = 不在媒体库中
show-get-episodes = 获取剧集
show-trailer = 预告片
show-your-score = 我的评分 { $score }
show-rate-it = 评分
show-anilist-score = AniList { $score }
show-forecast = 契合度 { $percent }%
show-because-you-watched = 因为你看过
show-hidden-loved = 背后有 { $count } 个你喜爱的隐藏标签
show-you-gave-it = 你给了 { $score } 分
show-story = 剧情
show-genres-tags = 类型和标签
show-studios = 制作公司  { $studios }
show-related = 相关作品
show-recommendations = 推荐
show-from-anilist = 来自 AniList
show-reading-rest = 正在从 AniList 读取其余内容

## ConnectSheet.qml

connect-connected-to = 已连接 { $name }
connect-connect-to = 连接 { $name }
connect-did-not-finish = 登录没有完成
connect-finish-in-browser = 请在浏览器中完成
connect-signed-in-anilist = 已登录为 { $account }。你的列表在“在看”页面中。
connect-signed-in-mal = 已登录为 { $account }。观看时会在那里标记剧集。
connect-setup-lead = { $name } 需要通过你自己的应用让 Anibeam 登录。只需设置一次，约一分钟。
connect-page-open = { $name } 的登录页面已在浏览器中打开。在那里批准 Anibeam，这里就会自动关闭。
connect-page-opening = 正在浏览器中打开 { $name } 的登录页面。
connect-open-watching = 打开“在看”
connect-done = 完成
connect-open-again = 重新打开登录页面
connect-secret-where = 它是 myanimelist.net/apiconfig 应用页面上“Client Secret”下方的长字符串。
connect-client-secret = 客户端密钥
connect-save-try-again = 保存并重试
connect-change-client-id = 更改客户端 ID
connect-step-create-anilist = 在 AniList 上创建客户端
connect-step-create-mal = 在 MyAnimeList 上创建应用
connect-step-create-anilist-body = 打开 AniList 的开发者设置，创建新客户端，并把这个地址原样粘贴到它的 Redirect URL 框中。
connect-step-create-mal-body = 打开 MyAnimeList 的 API 页面，以 App Type“other”创建 ID，并把这个地址原样粘贴到它的 App Redirect URL 框中。
connect-open-developer-page = 打开开发者页面
connect-copy-address = 复制地址
connect-step-paste-id = 粘贴应用的 ID
connect-step-paste-id-anilist = AniList 称它为 Client ID：新客户端页面上的数字。
connect-step-paste-id-mal = MyAnimeList 称它为 Client ID：新应用页面上的长字符串。
connect-client-id = 客户端 ID
connect-client-secret-web = 客户端密钥（仅限 Web 应用）
connect-step-connect-body = 浏览器会打开，{ $name } 会请你登录并批准 Anibeam，Anibeam 会接收回复。

## SettingsPage

settings-title = 设置
settings-search-placeholder = 搜索设置
settings-section-library = 媒体库
settings-section-accounts = 账号
settings-section-appearance = 外观
settings-section-playback = 播放
settings-section-subtitles = 字幕
settings-section-keys = 键盘
settings-section-downloads = 下载
settings-section-phone = 手机应用
settings-section-storage = 存储和备份
settings-section-help = 帮助
settings-section-about = 关于
settings-section-developer = 开发者
settings-status-library-empty = 还没有文件夹
settings-status-library = { $folders } 个文件夹，{ $count } 部番剧
settings-status-library-missing = { $gone } 个文件夹找不到，{ $folders } 个文件夹中有 { $count } 部番剧
settings-status-accounts-anilist = 正在同步到 AniList
settings-status-accounts-mal = 正在同步到 MyAnimeList
settings-status-accounts-both = 正在同步到 AniList 和 MyAnimeList
settings-status-accounts-none = 未连接列表
settings-status-appearance =
    { $mode ->
        [system] 跟随系统
        [light] 浅色
       *[dark] 深色
    }，{ $source ->
        [theme] 主题配色
       *[desktop] 桌面配色
    }
settings-status-playback =
    { $skip ->
        [both] 跳过片头和片尾
        [intro] 跳过片头
        [outro] 跳过片尾
       *[none] 完整播放
    }
settings-status-playback-audio =
    { $skip ->
        [both] 跳过片头和片尾
        [intro] 跳过片头
        [outro] 跳过片尾
       *[none] 完整播放
    }，音频 { $languages }
settings-status-subtitles =
    大小 { $percent }%，{ $override ->
        [AsScripted] 特效字幕保持原样
        [Force] 所有文件使用你的样式
       *[ScaleOnly] 特效字幕保留原有外观
    }
settings-status-keys = Anibeam 响应的所有按键
settings-status-downloads-unread = 尚未读取
settings-status-downloads-answered = qBittorrent 已响应
settings-status-downloads-missing = 未安装搜索助手
settings-status-downloads-unconnected = 尚未连接
settings-status-phone-shared = 已在你的 tailnet 上共享
settings-status-phone-off = 未共享
settings-status-storage = 已缓存 { $size } 图片
settings-status-storage-unread = 图片、导出和导入
settings-status-help = 常见问题解答
settings-status-about-update = 版本 { $version } 已发布
settings-status-developer = 正在为每个页面计时
settings-status-developer-last = { $page } 构建用时 { $ms } ms
settings-results = { $count } 项设置匹配
settings-results-none = 没有匹配项
settings-results-hint = 按 Enter 打开第一项。
settings-results-none-hint = 换个更简单的词试试：主题、字幕、追踪、密码、文件夹。

settings-find-folders =
    .words = 添加 文件夹 扫描 重新扫描 移除 来源 路径 收藏
settings-find-matching =
    .words = 元数据 番剧信息 信息来源 anilist tvmaze 顺序 匹配
settings-find-hidden-shows =
    .words = 隐藏 显示 取消隐藏
settings-find-hidden-airing =
    .words = 不感兴趣 播出 重新显示
settings-find-calendar =
    .words = 日历 ics ical 播出 排期 订阅
settings-find-anime-lists =
    .words = anilist myanimelist mal 登录 连接 账号 断开 追踪
settings-find-main-list =
    .words = 进度 卡片 主要 追踪
settings-find-sync-now =
    .words = 刷新 进度 列表 追踪 同步
settings-find-on-hold =
    .words = 搁置 暂停 闲置 未动 天数
settings-find-language =
    .words = 语言 翻译 区域 英语 中文
settings-find-mode =
    .words = 深色 浅色 系统 主题 夜间
settings-find-colours-from =
    .words = 终端 桌面 系统 主题 来源 kitty 配色
settings-find-dark-theme =
    .words = 配色 方案 颜色 base16
settings-find-light-theme =
    .words = 配色 方案 颜色 base16
settings-find-accent =
    .words = 颜色 强调色 高亮
settings-find-text-size =
    .words = 字体 文字 字号 更大 更小 缩放 阅读 无障碍
settings-find-density =
    .words = 紧凑 舒适 间距 大小 密度
settings-find-poster-size =
    .words = 卡片 小 中 大 海报
settings-find-out-now =
    .words = 播出 已播出 卡片 列表 行 样式 横幅 海报
settings-find-out-now-cards =
    .words = 播出 已播出 网格 列 行 布局
settings-find-reduce-motion =
    .words = 动画 动效 无障碍
settings-find-app-icon =
    .words = 图标 标志 启动器 程序坞 任务栏 窗口 深色
settings-find-skip-intros =
    .words = 片头 op 自动 跳过 aniskip 章节
settings-find-skip-outros =
    .words = 片尾 ed 演职员表 自动 跳过 aniskip
settings-find-mpv-settings =
    .words = mpv mpv.conf 配置 高级
settings-find-discord =
    .words = discord rich presence 状态 个人资料
settings-find-next-episode =
    .words = 下一集 按钮 标题 自动播放 结束
settings-find-subtitle-languages =
    .words = 语言 轨道 字幕
settings-find-audio-languages =
    .words = 语言 音轨 配音 日语
settings-find-start-over =
    .words = 重置 默认 字幕 样式
settings-find-size =
    .words = 大小 缩放 字幕
settings-find-position =
    .words = 位置 高度 边距 字幕
settings-find-styled =
    .words = 特效字幕 ass 覆盖 强制
settings-find-font =
    .words = 字体
settings-find-text-colour =
    .words = 颜色 文字颜色
settings-find-outline =
    .words = 描边 边框 轮廓
settings-find-background =
    .words = 背景框 透明度 背景
settings-find-bold =
    .words = 粗体 字重
settings-find-keys-player =
    .words = 键盘 快捷键 按键 热键
settings-find-keys-player-around =
    .words = 键盘 快捷键 全屏 下一集 上一集 返回
settings-find-keys-around =
    .words = 键盘 快捷键 返回 前进 右键 菜单 全屏
settings-find-keys-anywhere =
    .words = 键盘 快捷键 按键 热键 设置 退出
settings-find-torrent-client =
    .words = qbittorrent 密码 用户名 地址 webui seedbox 种子 客户端
settings-find-search-helper =
    .words = anirss rss 订阅源 令牌 命令 服务器 高级 搜索助手
settings-find-next-few =
    .words = 滚动 预留 剧集 提前 删除 已看 保留 磁盘 空间
settings-find-unsubscribe =
    .words = 清理 订阅源 完结 取消订阅
settings-find-last-search =
    .words = 下载 搜索 记住
settings-find-search-sources =
    .words = 端点 订阅源 配置 anirss 来源
settings-find-source-lists =
    .words = 仓库 端点 签名 jackett prowlarr 来源列表
settings-find-share-tailscale =
    .words = 手机 android tailnet tailscale 远程 主机 共享
settings-find-stream =
    .words = 串流 播放 投屏 剧集 手机 文件
settings-find-devices =
    .words = 信任 关闭 屏蔽 设备 tailscale
settings-find-back-up =
    .words = 导出 备份 文件 json
settings-find-restore =
    .words = 导入 恢复 文件 json
settings-find-image-cache =
    .words = 海报 清除 图片 磁盘 缓存
settings-find-files =
    .words = 数据库 数据 配置 缓存 文件夹 路径
settings-find-help-folders =
    .words = 帮助 文件夹 结构 命名 剧集 季 电影 文件
settings-find-help-recognised =
    .words = 帮助 匹配 错误 番剧 找不到 未匹配 识别
settings-find-help-anilist =
    .words = 帮助 anilist myanimelist mal 登录 连接 追踪
settings-find-help-downloads =
    .words = 帮助 下载 空 来源 没有 种子 qbittorrent
settings-find-help-data =
    .words = 帮助 数据 文件 备份 数据库 隐私
settings-find-help-else =
    .words = 帮助 设置 新手引导 向导 报告 bug 问题 反馈
settings-find-version =
    .words = 版本 build 更新内容 更新日志
settings-find-setup =
    .words = 首次运行 向导 新手引导 重新设置
settings-find-updates =
    .words = 稳定版 nightly 通道 检查 更新
settings-find-developer-options =
    .words = 调试 计时 开发者
settings-find-ratings =
    .words = imdb 评分 致谢

## SettingsDownloadsTab

settings-downloads-title = 下载
settings-downloads-lead = Anibeam 把下载内容发往何处，以及如何搜索资源订阅源。
settings-downloads-keep-none = 看完一集会删除它的前一集。
settings-downloads-keep-line = 在刚看完的那一集之前保留的已看剧集数。设为 { $count } 时，看完第 { $finishing } 集会删除第 1 集。
settings-downloads-anirss-ready = 运行中，版本 { $version }，地址 { $url }。
settings-downloads-anirss-starting = 正在启动搜索助手。
settings-downloads-anirss-missing = 未安装搜索助手。请重新安装 Anibeam，或在“命令”中填写你自己的 anirss。
settings-downloads-anirss-failed = 搜索助手已停止。“重试”会启动它；“详情”中有它的输出。
settings-downloads-anirss-idle = 尚未启动。第一次在“下载”页面搜索时启动。
settings-downloads-torrent-trying = 正在尝试 { $client }。
settings-downloads-torrent-answered = { $client } { $version } 已响应。
settings-downloads-torrent-failed = { $client } 没有连接。请检查上方的地址和密码，然后重新测试。
settings-downloads-torrent-untried = 尚未尝试。
settings-downloads-mode-existing = 你自己运行的 qBittorrent，需开启 Web UI。
settings-downloads-mode-managed = Anibeam 运行自带的 qBittorrent，下载到 ~/Downloads/Anime。Anibeam 退出后它会继续运行，订阅会持续下载。
settings-downloads-mode-auto = 如果下方地址有 qBittorrent 响应就使用它，否则运行自带的。
settings-downloads-password-saved = 这里已为此客户端保存了密码。
settings-downloads-password-anirss = 正在使用搜索助手保存的密码；在这里保存一个就不再依赖它。
settings-downloads-password-none = 没有保存密码。对这台机器免登录的客户端不需要密码。
settings-downloads-mismatch = 搜索助手连接的是 { $helperUrl }，而 Anibeam 连接的是 { $url }。你自己运行的搜索助手从它自己的 config.toml 读取地址，所以在这里切换客户端不会影响它。
settings-downloads-torrent-title = 种子客户端
settings-downloads-torrent-helper = 负责下载的 qBittorrent。它的密码保存在系统密码存储中。
settings-downloads-which-client = 使用哪个客户端
settings-downloads-pick-for-me = 自动选择
settings-downloads-my-own = 我自己的
settings-downloads-built-in = 内置
settings-downloads-saved-clients = 已保存的客户端
settings-downloads-saved-clients-helper = 比如一台 seedbox 和这台机器上的客户端。“下载”页面上的客户端标签也可以在它们之间切换。
settings-downloads-add-client = 添加
settings-downloads-name = 名称
settings-downloads-name-placeholder = 以地址命名
settings-downloads-address = 地址
settings-downloads-address-managed-helper = 只有这台机器能访问它，不需要密码。
settings-downloads-address-helper = Web UI 的地址。
settings-downloads-username = 用户名
settings-downloads-password = 密码
settings-downloads-secret-saved = 已保存
settings-downloads-secret-unset = 未设置
settings-downloads-connection = 连接
settings-downloads-open-config = 打开它的配置
settings-downloads-details = 详情
settings-downloads-details-torrent = Anibeam 尝试连接时 { $client } 返回了这些内容。
settings-downloads-test-trying = 正在尝试
settings-downloads-test-save = 保存并测试
settings-downloads-test = 测试
settings-downloads-searching = 搜索
settings-downloads-search-helper = 搜索助手
settings-downloads-searching-helper = Anibeam 自己搜索你的来源，并把你选中的内容发送到种子客户端。不需要搜索助手。
settings-downloads-search-helper-helper = Anibeam 中负责搜索和订阅的部分，是一个名为 anirss 的程序。Anibeam 会替你运行它；只有你自己运行时才需要更改。
settings-downloads-runs-in = 搜索运行于
settings-downloads-runs-in-native-helper = 订阅仍交给种子客户端，所以 Anibeam 关闭时新剧集也会到来。
settings-downloads-runs-in-helper = 搜索助手，一个由 Anibeam 或你自己运行的独立程序。
settings-downloads-engine-native = Anibeam 本身
settings-downloads-how-runs = 运行方式
settings-downloads-how-runs-sidecar = Anibeam 打开时启动它，退出时停止它。
settings-downloads-how-runs-remote = Anibeam 使用你已在运行的那个，地址和密钥见下方。
settings-downloads-runs-sidecar = 由 Anibeam 运行
settings-downloads-runs-remote = 由我运行
settings-downloads-command = 命令
settings-downloads-command-helper = PATH 中的名称或完整路径。也会搜索 ~/.local/bin。
settings-downloads-server-address-helper = 服务器的基础地址，不含 /api/v1。
settings-downloads-key = 密钥
settings-downloads-key-saved-helper = 已保存密钥。输入新的即可替换，清空即可移除。
settings-downloads-key-helper = 搜索助手启动时显示的密钥，用于让 Anibeam 访问。
settings-downloads-details-anirss = 搜索助手停止时输出了这些内容。
settings-downloads-starting = 正在启动
settings-downloads-reconnect = 重新连接
settings-downloads-save-first = 请先保存更改
settings-downloads-roll-title = 保持后几集就绪
settings-downloads-roll-helper = 长篇番剧每次只下载几集，每看完一集就开始下载下一集。整部番剧不必全部放进磁盘。
settings-downloads-roll-big = 适用于长篇番剧
settings-downloads-roll-big-helper = 超过 { $count } 集的番剧会自动分批下载。它在“下载”页面上的条目仍可一次获取全部。
settings-downloads-roll-ahead = 预留集数
settings-downloads-roll-ahead-helper = 在当前这一集之后保持下载的集数。12 集大约是一季。
settings-downloads-unit-episodes = 集
settings-downloads-roll-over = 长篇指超过
settings-downloads-roll-over-helper = 集数超过这个值的下载会分批进行。
settings-downloads-roll-watched = 已看剧集
settings-downloads-roll-watched-delete-helper = 往后看了之后会删除，磁盘上只保留还没看的内容。
settings-downloads-roll-watched-keep-helper = 看过的每个文件都留在磁盘上。
settings-downloads-keep-all = 全部保留
settings-downloads-keep-behind = 保留之前
settings-downloads-subscriptions-title = 订阅和搜索
settings-downloads-last-search = 保留上次搜索
settings-downloads-last-search-helper = 打开“下载”页面时显示你上次的搜索。关闭时页面为空。
settings-downloads-prune = 取消订阅已完结番剧
settings-downloads-prune-helper = 番剧最后一集播出 16 天后，会移除它的订阅源。番剧和文件会保留。
settings-downloads-sources-title = 搜索来源
settings-downloads-sources-helper = 搜索的去处，按尝试顺序排列。一个找不到就交给下一个。
settings-downloads-sources-empty = 还没有设置来源
settings-downloads-sources-connect = 连接搜索助手后即可列出
settings-downloads-sources-empty-helper = 在下方添加来源列表，它的来源就会显示在这里。
settings-downloads-config-file = 搜索助手的设置文件
settings-downloads-config-native-helper = 包含保存文件夹、标签和最佳匹配规则，Anibeam 运行助手时自己也会遵循。其中的来源只属于助手。
settings-downloads-config-helper = 包含来源、保存文件夹、标签和最佳匹配规则。
settings-downloads-unsaved = 有未保存的更改
settings-downloads-unsaved-helper = 保存后搜索助手会以新设置重启。
settings-downloads-discard = 放弃

## SettingsDeveloperTab

settings-developer-title = 开发者
settings-developer-lead = 测量页面为什么慢。每个开关从你打开的下一个页面开始生效。
settings-developer-ms = { $ms } ms
settings-developer-mb = { $mb } MB
settings-developer-seconds = { $seconds } 秒
settings-developer-switches = 开关
settings-developer-load-images = 加载图片
settings-developer-load-images-helper = 关闭时不读取也不解码任何海报、横幅或头像。
settings-developer-overlay = 页面加载浮层
settings-developer-overlay-helper = 在窗口右下角显示上次页面加载和内存。
settings-developer-read-cache = 保留媒体库读取结果
settings-developer-read-cache-helper = 整个媒体库的读取结果会保留到媒体库变化为止。关闭时每个页面都会重新询问核心。
settings-developer-poster-pool = 在后台预热海报
settings-developer-poster-pool-helper = 解码你最可能看到的 60 张海报并保留在内存中。
settings-developer-options = 开发者选项
settings-developer-options-helper = 关闭此标签页、浮层和测量。
settings-developer-memory = 内存
settings-developer-memory-helper = 内核为此进程统计的内存。图片解码后的像素会占用内存和显存。
settings-developer-memory-unknown = 此系统不提供应用的内存用量。
settings-developer-in-use = 使用中
settings-developer-peak = 峰值
settings-developer-heap = 堆
settings-developer-files-mapped = 映射文件
settings-developer-threads = 线程
settings-developer-give-back = 释放内存
settings-developer-give-back-helper = 丢弃场景图可以重建的内容，回收 JavaScript 垃圾，并把空闲堆交还给系统。释放的就是应用占着却不需要的部分。
settings-developer-released = { $before } 到 { $after }
settings-developer-free-now = 立即释放
settings-developer-images-total = 启动以来的图片
settings-developer-images-total-line = 解码 { $decodes } 次，{ $pixels } 像素，来自磁盘上的 { $disk }，{ $ms } ms
settings-developer-loads = 页面加载
settings-developer-loads-helper = “构建”是点击后的等待时间：页面出现之前 Qt 线程不做其他事。“完成”是最后一张图片到达的时间。
settings-developer-touring = 正在计时，{ $at } / { $steps }
settings-developer-tour = 为每个页面计时
settings-developer-copy-report = 复制报告
settings-developer-col-page = 页面
settings-developer-col-built = 构建
settings-developer-col-painted = 绘制
settings-developer-col-settled = 完成
settings-developer-col-calls = 调用，数据
settings-developer-col-decodes = 解码，用时，像素
settings-developer-col-items = 元素
settings-developer-col-offscreen = 视野外的图片
settings-developer-col-memory = 内存
settings-developer-load-no-images = { $page }，无图片
settings-developer-offscreen-of = { $offscreen } / { $total }
settings-developer-loads-empty = 打开任意页面，它就会显示在这里。
settings-developer-load-images-off = { $page }，图片已关闭
settings-developer-load-frames = 加载期间绘制了 { $frames } 帧，两帧之间最长间隔 { $gap }。
settings-developer-load-frames-timed-out = 加载期间绘制了 { $frames } 帧，两帧之间最长间隔 { $gap }。等待图片 15 秒后放弃。
settings-developer-built = 构建
settings-developer-settled = 完成
settings-developer-core-data = 来自核心的数据
settings-developer-pixels = 已解码像素
settings-developer-page-items = 页面上的元素
settings-developer-cached-images = 已在内存中的图片
settings-developer-calls = 核心调用，最慢在前
settings-developer-call-cached = { $name }  { $ms } ms  { $size }  已缓存
settings-developer-call = { $name }  { $ms } ms  { $size }  核心 { $core }，到 QML { $qml }
settings-developer-decodes = 最慢的解码
settings-developer-decodes-line = 图片线程上依次解码 { $count } 次，从第一次到最后一次共 { $span }；请求 { $requests } 次，{ $cached } 次已解码，{ $blocked } 次被阻塞
settings-developer-decode = { $ms } ms  { $srcW }x{ $srcH } 到 { $w }x{ $h }  { $size }
settings-developer-decodes-none = 没有解码：所有图片都已在内存中。
settings-developer-images-were-off = 图片已关闭。
settings-developer-scene = 页面的构成
settings-developer-scene-line = { $items } 个元素，{ $offscreen } 个在窗口外；{ $texts } 个文本，{ $shapes } 个形状，{ $icons } 个图标；{ $images } 张图片，其中 { $imagesOffscreen } 张在窗口外，占用 { $size }
settings-developer-copy-load = 复制此次加载
settings-developer-tour-title = 每个页面，开启和关闭图片
settings-developer-tour-helper = 每个页面访问三次取中间值。两列数值相同的地方，拖慢页面的不是图片。
settings-developer-col-built-on = 构建，开启图片
settings-developer-col-built-off = 构建，关闭图片
settings-developer-col-settled-on = 完成，开启
settings-developer-col-settled-off = 完成，关闭
settings-developer-col-decodes-time = 解码，用时
settings-developer-col-data = 数据
settings-developer-tour-peak = 计时期间内存峰值 { $size }
settings-developer-events = 事件
settings-developer-events-helper = 核心输出的所有内容，所有级别，最新在前。每个事件都以 JSON 复制到 Qt 线程；大小即发送成本。
settings-developer-event-rate = { $kind } { $count }/分钟，{ $size }
settings-developer-events-filter = 按词语、类型或阶段筛选
settings-developer-event-plain = { $message }   （原文：{ $plain }）
settings-developer-copy-events = 复制事件

## SettingsKeysTab

settings-keys-title = 键盘
settings-keys-lead = Anibeam 响应的所有按键。在任何地方按 ? 都能看到当前位置可用的按键。
settings-keys-player = 播放器中
settings-keys-player-around = 在播放器中导航
settings-keys-around = 导航
settings-keys-anywhere = 任何地方

## KeyHelp

key-help-title = 键盘快捷键
key-help-group-playback = 播放
key-help-group-navigation = 导航
key-help-group-library = 媒体库
key-help-key-space = 空格 / K
key-help-key-left-right = ← / →
key-help-key-ctrl-right = Ctrl+→
key-help-key-backspace = Backspace
key-help-key-up-down = ↑ / ↓
key-help-key-hold = 按住画面
key-help-key-page = Page Up / Down
key-help-key-home-end = Home / End
key-help-key-escape = Esc
key-help-key-alt-left = Alt+←
key-help-key-alt-right = Alt+→
key-help-key-shift-f10 = Shift+F10
key-help-key-ctrl-k = Ctrl+K
key-help-key-search = / 或 Ctrl+F
key-help-key-ctrl-space = Ctrl+空格
key-help-key-shift-space = Shift+空格
key-help-key-ctrl-a = Ctrl+A
key-help-key-ctrl-l = Ctrl+L
key-help-key-ctrl-comma = Ctrl+,
key-help-key-ctrl-q = Ctrl+Q
key-help-play-pause = 播放或暂停
key-help-seek-5 = 快进快退 5 秒
key-help-seek-10 = 快进快退 10 秒
key-help-skip = 跳过片头或片尾，否则跳 90 秒
key-help-frame-step = 后退或前进一帧
key-help-speed = 减速或加速，每次 0.25
key-help-normal-speed = 正常速度
key-help-volume = 音量增减 5
key-help-mute = 静音
key-help-subtitles-toggle = 关闭或重新打开字幕
key-help-subtitle-delay = 字幕延迟提前或延后 100 ms
key-help-audio-delay = 音频延迟提前或延后 100 ms
key-help-hold-speed = 按住时 2× 速度
key-help-screenshot = 截图保存到 Pictures
key-help-episode = 下一集或上一集
key-help-chapter = 上一章或下一章
key-help-start-end = 开头，或片尾
key-help-fullscreen = 全屏
key-help-escape-player = 关闭打开的内容，否则退出全屏，否则离开播放器
key-help-toggle-list = 显示或隐藏此列表
key-help-library-search = 媒体库和搜索
key-help-search-page = 搜索此页面
key-help-activity = 活动
key-help-settings = 设置
key-help-quit = 退出
key-help-forward = 前进
key-help-escape-app = 关闭打开的内容，否则返回
key-help-context-menu = 右键菜单
key-help-pick-card = 选择卡片，以便批量操作
key-help-pick-range = 选择到这张为止的所有卡片
key-help-pick-all = 选择所有显示的卡片

## SettingsAboutTab

settings-about-title = 关于
settings-about-lead = 这是哪个版本的 Anibeam，以及它如何得知新版本。
settings-about-update-failed = 更新没有完成。请重试，或从发布页获取。
settings-about-update-ready = { $version } 已可安装。
settings-about-update-command = { $version } 已发布。更新命令：{ $command }
settings-about-update-progress = { $headline }  ·  { $detail }
settings-about-update-source = { $version } 已发布。这份副本由源码构建：请拉取并重新构建。
settings-about-update-elsewhere = { $version } 已发布，但这里没有适用于这台电脑的下载。
settings-about-version = 版本
settings-about-build = Anibeam { $build }
settings-about-whats-new = 更新内容
settings-about-install = 安装 { $version }
settings-about-setup = 设置向导
settings-about-setup-helper = 新安装时打开的向导：你的文件夹、列表和下载。
settings-about-run-setup = 重新运行设置
settings-about-updates = 更新
settings-about-nightly-helper = 几乎每天都有新版本。更新，但测试较少。
settings-about-stable-helper = 经过测试并发布后才有新版本。
settings-about-stable = 稳定版
settings-about-nightly = Nightly
settings-about-check-now = 立即检查
settings-about-developer = 开发者选项
settings-about-developer-helper = 添加“开发者”分区，可为每个页面计时、显示内存和完整的技术日志，还能关闭图片。
settings-about-credits = 致谢
settings-about-ratings = 评分
settings-about-ratings-helper = Information courtesy of IMDb (https://www.imdb.com). Used with permission.
settings-about-series = 番剧和剧集
settings-about-series-helper = AniList 和 TVmaze。跳过时间来自 AniSkip。

## SettingsPlaybackTab

settings-playback-title = 播放
settings-playback-lead = 播放器自动完成的事，让你只在需要时才去操作。
settings-playback-skipping = 跳过
settings-playback-skipping-helper = 使用文件的章节，否则使用其他观众在网上分享的跳过时间。在播放器中点“撤销”会关闭跳过，直到 Anibeam 关闭。
settings-playback-skip-intros = 跳过片头
settings-playback-skip-outros = 跳过片尾
settings-playback-skip-outros-helper = 播到片尾也会把这一集标为已看。
settings-playback-next-episode = 下一集
settings-playback-name-next = 显示下一集标题
settings-playback-name-next-helper = 一集结束时的“下一集”按钮会显示下一集的标题。关闭时只显示集数。
settings-playback-languages = 语言
settings-playback-languages-helper = 语言代码，最想要的在前。文件中第一个匹配的生效。你在播放器中选的轨道会为该番剧记住。
settings-playback-subtitle-languages = 字幕语言
settings-playback-audio-languages = 音频语言
settings-playback-discord = 在 Discord 上显示你在看什么
settings-playback-discord-check-failed = Anibeam 刚才无法检查 Discord，请稍后再试。
settings-playback-discord-helper = 播放时，番剧、集数和海报会显示在你的 Discord 个人资料上。关闭此项或 Discord 未运行时不会发送任何内容。
settings-playback-discord-unavailable = 此版本没有 Discord 应用。
settings-playback-discord-connected = 已连接 Discord
settings-playback-discord-not-connected = 未连接。播放剧集且 Discord 已打开时，Anibeam 会自动连接
settings-playback-advanced = 高级
settings-playback-my-mpv = 使用我的 mpv 设置
settings-playback-my-mpv-helper = 读取你的 mpv.conf。仅在启动时生效的选项会被跳过，脚本永远不会加载。
settings-playback-no-mpv-conf = 没有找到 mpv 设置文件。把选项写入 mpv.conf 后再开启此项。

## SliderRow

slider-row-spoken-unit = { $name }，单位：{ $unit }
slider-row-readout = { $value } { $unit }

## Swatches

swatches-accent-red = 强调色：红
swatches-accent-orange = 强调色：橙
swatches-accent-yellow = 强调色：黄
swatches-accent-green = 强调色：绿
swatches-accent-cyan = 强调色：青
swatches-accent-blue = 强调色：蓝

## LookPreview

look-preview-dark = 深色
look-preview-light = 浅色

## SettingsAccountsTab

settings-accounts-title = 账号
settings-accounts-lead = 连接 AniList 或 MyAnimeList，你看完的每一集都会自动在那里标记。进度只增不减。
settings-accounts-syncing = 正在同步列表
settings-accounts-synced = 列表已同步
settings-accounts-lists = 你的动漫列表
settings-accounts-lists-helper = 没有列表时，Anibeam 仍会在这台电脑上记住你看过什么。AniList 或 MyAnimeList 上的列表会把它带到你的手机和个人资料上。
settings-accounts-progress = 进度
settings-accounts-main-list = 主列表
settings-accounts-main-list-helper = 卡片显示这个列表的进度。另一个仍会收到每次更新。
settings-accounts-local-lists = 这台电脑上的列表
settings-accounts-local-lists-helper = 有 { $count } 部番剧的状态或评分只在这里。把它们移到 { $tracker }，就能在手机和个人资料上看到。
settings-accounts-review-move = 查看并移动
settings-accounts-nothing-to-move = 没有需要移动的内容
settings-accounts-sync-now = 立即同步
settings-accounts-sync-now-helper = 重新读取每个已连接的列表。Anibeam 启动时也会这样做。
settings-accounts-sync = 同步
settings-accounts-move-on-hold = 移到“搁置”
settings-accounts-move-on-hold-helper = 一段时间没碰的“在看”番剧会在你的列表上移到“搁置”。再播放就会移回来。在等下一集的番剧保持不变。
settings-accounts-after = 之后
settings-accounts-after-line = 没有播放任何一集的天数
settings-accounts-days-unit = 天

## SettingsPhoneTab

settings-phone-title = 手机应用
settings-phone-lead = 手机上的 Anibeam 可以播放这个媒体库、查看正在播出的番剧并跟进你的下载，无论你身在何处。
settings-phone-online = 在线
settings-phone-offline = 离线
settings-phone-untrusted = 从这台电脑获取不到任何内容
settings-phone-sharing = 共享
settings-phone-sharing-helper = 只有你自己在 Tailscale 上的设备能访问。在手机上看的剧集，进度会同步到这里。
settings-phone-share = 通过 Tailscale 共享
settings-phone-stream = 串流剧集
settings-phone-stream-on = 你的设备可以播放和投屏这台电脑上的剧集。
settings-phone-stream-off = 你的设备能看到媒体库和正在播出的番剧，但不能从这里播放。
settings-phone-host-service = 主机服务
settings-phone-not-running = 未运行
settings-phone-running = 运行中
settings-phone-service-off-helper = 运行一次这条命令，之后从登录起你的设备就能访问 Anibeam，不用打开窗口。
settings-phone-service-on-helper = 你的设备不用打开窗口也能访问 Anibeam。这个窗口打开时，由它代替服务共享。
settings-phone-copy-command = 复制命令
settings-phone-command-copied = 已复制命令
settings-phone-nobody = 获取不到任何内容：Tailscale 没有为它指定用户
settings-phone-people = 用户
settings-phone-own-profile = 这台电脑的资料
settings-phone-person-own-devices =
    { $count ->
        [0] 这台电脑
       *[other] 这台电脑和 { $count } 台设备
    }
settings-phone-person-devices = { $count } 台设备
settings-phone-devices = 你 tailnet 上的设备
settings-phone-devices-helper = 开启共享时，你 tailnet 上的每台设备都能访问这台电脑。关闭某台设备后，它从这里获取不到任何内容。
settings-phone-addresses = 地址
settings-phone-addresses-helper = 手机应用第一次询问时，输入其中一个地址。
settings-phone-copy = 复制
settings-phone-address-copied = 已复制地址

## SettingsHelpTab

settings-help-title = 帮助
settings-help-lead = 常见问题的简短解答，每条都能直接跳到解决办法。
settings-help-folders = 文件夹应该怎么组织？
settings-help-folders-helper = 每部番剧一个文件夹，剧集放在里面。每个文件名带上集数，就像资源发布时那样：“Frieren - 01.mkv”或“Frieren S01E01.mkv”。第二季可以单独一个文件夹，也可以是番剧文件夹里的“Season 2”文件夹。剧场版放在名为 Movies 的文件夹里，每部一个文件。
settings-help-for-example = 例如
settings-help-folders-example =
    Anime / Frieren / Frieren - 01.mkv
    Anime / Frieren / Season 2 / Frieren S02E01.mkv
    Anime / Movies / Suzume.mkv
settings-help-no-folder = 还没有添加文件夹
settings-help-reads-folders = Anibeam 读取 { $count } 个文件夹
settings-help-add-folder-helper = 添加存放番剧的总文件夹，而不是逐个添加每部番剧。
settings-help-add-folder = 添加文件夹
settings-help-open-folders = 打开你的文件夹
settings-help-match = 为什么我的番剧没有被识别？
settings-help-match-helper = Anibeam 按文件夹名称查找每部番剧，先查 AniList，再查 TVmaze。多余的词、拼写错误或其他语言的名称都可能导致找不到。可以在“番剧信息”中手动选择正确的番剧；你的选择永远不会被替换。
settings-help-all-recognised = 所有番剧都已识别
settings-help-unrecognised = 有 { $count } 部番剧未被识别
settings-help-open-show-info = 打开“番剧信息”
settings-help-connect = 如何连接 AniList？
settings-help-connect-helper = 连接后，你看完的每一集都会标记到列表上，手机和个人资料也会同步。AniList 需要你先创建一个自己的小应用；Anibeam 会一步步引导你，大约一分钟。MyAnimeList 也一样。
settings-help-both-connected = 两者都已连接
settings-help-anilist-connected = AniList 已连接
settings-help-mal-connected = MyAnimeList 已连接
settings-help-none-connected = 两者都还没有连接
settings-help-connect-anilist = 连接 AniList
settings-help-connect-mal = 连接 MyAnimeList
settings-help-downloads = 为什么“下载”是空的？
settings-help-downloads-helper = Anibeam 本身不带任何搜索位置。请在设置的“下载”中添加来源列表，并从中安装需要的来源。下载还需要种子客户端：Anibeam 可以运行自带的，也可以使用你已有的 qBittorrent。
settings-help-sources = 来源和种子客户端
settings-help-open-downloads = 打开下载设置
settings-help-data = 我的数据在哪里？
settings-help-data-helper = 你的媒体库、观看记录和设置都保存在这台电脑上。AniList 和 MyAnimeList 的登录信息保存在系统密码存储中；没有时则保存在只有你能读取的文件中。可以在“存储和备份”中把所有内容备份到一个文件。
settings-help-library-history = 媒体库和历史
settings-help-show-folder = 显示文件夹
settings-help-backup = 备份或恢复
settings-help-open-storage = 打开“存储和备份”
settings-help-more = 还有其他问题？
settings-help-more-helper = 重新运行首次设置，或告诉我们哪里出了问题。附上截图会很有帮助。
settings-help-first-setup = 首次设置
settings-help-first-setup-helper = 你的文件夹、列表和下载，一步一步来。
settings-help-run-setup = 重新运行设置
settings-help-report = 反馈问题
settings-help-report-helper = 在浏览器中打开 GitHub 上的问题报告，并填好当前版本。

## SettingsSubtitlesTab

settings-subtitles-title = 字幕
settings-subtitles-lead = 每一集的初始字幕设置。更改会实时显示在下方的画面上。
settings-subtitles-reset-done = 字幕已恢复为最初的样子
settings-subtitles-size-place = 大小和位置
settings-subtitles-size = 大小
settings-subtitles-position = 位置
settings-subtitles-position-helper = 100 贴着底边。数值越小，字幕越往上。
settings-subtitles-style = 样式
settings-subtitles-styled = 特效字幕
settings-subtitles-as-made-helper = 特效（ASS）字幕文件完全按制作者的设置显示。
settings-subtitles-mine-helper = 下方你的样式会用于每个文件。标牌和卡拉 OK 字幕可能出错。
settings-subtitles-size-only-helper = 特效字幕文件保留原有外观，只采用你的大小。普通文件采用你的样式。
settings-subtitles-as-made = 原样
settings-subtitles-size-only = 仅大小
settings-subtitles-mine = 我的样式
settings-subtitles-font = 字体
settings-subtitles-font-invalid = 请输入字体名称，例如 Noto Sans。
settings-subtitles-bold = 粗体
settings-subtitles-text-colour = 文字颜色
settings-subtitles-colour-invalid = 请输入颜色，例如 #ffcc00。
settings-subtitles-outline = 描边
settings-subtitles-outline-invalid = 宽度必须是 0 或更大的数字。
settings-subtitles-outline-colour-invalid = 请输入颜色，例如 #000000。
settings-subtitles-outline-helper = 先是宽度，再是颜色。
settings-subtitles-shadow = 阴影
settings-subtitles-shadow-invalid = 0 或更大的数字。0 表示无阴影。
settings-subtitles-shadow-helper = 阴影的距离，0 表示无阴影。
settings-subtitles-box = 背景框
settings-subtitles-box-helper = 字幕后面的深色框。0 表示不显示。
settings-subtitles-start-over = 恢复默认
settings-subtitles-start-over-helper = 把大小、字体、颜色、描边、背景框和位置恢复为初始设置。
settings-subtitles-reset = 重置

## SettingsDataTab

settings-data-title = 存储和备份
settings-data-lead = 把媒体库迁移到另一台电脑、保存一份副本，或释放图片占用的空间。
settings-data-backed-up = 已备份到 { $path }
settings-data-restore-finished = 恢复完成
settings-data-images-removed = 已移除 { $count } 张图片
settings-data-summary = { $parts }。
settings-data-summary-folders = 添加了 { $count } 个文件夹
settings-data-summary-series = 创建了 { $count } 部番剧
settings-data-summary-matches = 应用了 { $count } 个匹配
settings-data-summary-views = 合并了 { $views } 次观看和 { $episodes } 集看完的剧集
settings-data-summary-resume-points = 合并了 { $count } 个续播位置
settings-data-summary-accounts = 导入了 { $count } 个列表账号
settings-data-summary-fixes = 有 { $count } 个文件采用了你做的修正。
settings-data-summary-unavailable = 有 { $count } 个文件夹在这台电脑上找不到。
settings-data-summary-left-out = 未包含：{ $fields }。
settings-data-backup = 备份
settings-data-back-up-library = 备份媒体库
settings-data-back-up-helper = 一个文件，包含你的文件夹、匹配结果和观看记录。
settings-data-last-written = 上次写入 { $path }
settings-data-open-folder = 打开文件夹
settings-data-back-up = 备份
settings-data-private = 包含私人数据
settings-data-private-helper = 加入 AniList 和 MyAnimeList 的登录信息、观看历史和偏好设置。登录信息以明文写入，请妥善保管文件。
settings-data-restore-from = 从备份恢复
settings-data-restore-helper = 把备份读入这个媒体库。以磁盘上的内容为准，文件补全其余部分。
settings-data-restore = 恢复
settings-data-image-cache = 图片缓存
settings-data-image-cache-helper = 海报、横幅和头像保存在磁盘上，让媒体库离线也能用。清除的图片会在页面需要时重新下载。
settings-data-cache-unmeasured = 无法统计图片缓存
settings-data-cache-size = { $count } 张图片，共 { $size }
settings-data-cache-empty = 没有缓存图片
settings-data-cache-unmeasured-helper = 请稍后再试。图片本身没有问题。
settings-data-remove-images = 移除 { $count } 张图片？
settings-data-files = Anibeam 的文件位置
settings-data-files-database = 媒体库数据库
settings-data-files-data = 数据
settings-data-files-settings = 设置
settings-data-files-cache = 缓存
settings-data-backup-filter = Anibeam 备份 (*.json)
settings-data-all-files-filter = 所有文件 (*)
settings-data-backing-up = 正在备份
settings-data-restoring = 正在恢复

## SettingsLibraryTab

settings-library-title = 媒体库
settings-library-lead = Anibeam 监视的文件夹，以及它如何识别其中的内容。
settings-library-not-scanned = 尚未扫描
settings-library-scan-nothing-changed = 没有变化
settings-library-scan-added = 新增 { $count }
settings-library-scan-changed = 变化 { $count }
settings-library-scan-removed = 移除 { $count }
settings-library-scan-in-folder = { $folder }：{ $what }
settings-library-anilist-about = 动画：番剧、剧场版及其关联。你看过的内容会在追踪服务上标记。
settings-library-tvmaze-about = 其他电视节目：真人剧、卡通、纪录片。不会在追踪服务上标记。
settings-library-tried-first = 第一个尝试
settings-library-tried-second = 第二个尝试
settings-library-tried-third = 第三个尝试
settings-library-tried-fourth = 第四个尝试
settings-library-tried-nth = 第 { $number } 个尝试
settings-library-folders = 文件夹
settings-library-folders-helper = 这些文件夹中的每个子文件夹都是一部番剧。Movies 文件夹顶层的文件是剧场版。
settings-library-rescan-started = 已开始重新扫描
settings-library-load-failed = 无法读取你的文件夹
settings-library-load-failed-helper = Anibeam 无法读取它的文件夹列表，请稍后再试。
settings-library-no-folders = 还没有文件夹
settings-library-no-folders-helper = 让 Anibeam 指向存放动漫的文件夹，它会自己找出番剧和剧场版。
settings-library-scanned = 扫描于{ $when }
settings-library-add-a-folder = 添加文件夹
settings-library-scan-all = 全部扫描
settings-library-scan-started = 已开始扫描
settings-library-add-folder = 添加文件夹
settings-library-matching = 匹配
settings-library-matching-helper = 新文件夹按这个顺序查找，并保留第一个合适的匹配。手动设置的匹配永远不会被替换。
settings-library-try-later = 往后
settings-library-try-first = 优先
settings-library-check-matches = 检查匹配
settings-library-check-matches-helper = “番剧信息”页面列出每部番剧及 Anibeam 为它找到的信息，可以修正错误的匹配。
settings-library-open-show-info = 打开“番剧信息”
settings-library-hidden-shows = 隐藏的番剧
settings-library-show-hidden = 显示隐藏的番剧
settings-library-show-hidden-helper = 让你隐藏的番剧重新出现在每个页面上，直到 Anibeam 关闭。
settings-library-hidden-from-airing = 从“正在播出”中隐藏
settings-library-hidden-from-airing-helper = 你标记为不感兴趣的番剧。在你恢复之前，它们不会出现在“正在播出”中。
settings-library-hidden-from-airing-empty = 还没有。你在“正在播出”中标记为不感兴趣的番剧会集中在这里，随时可以恢复。
settings-library-show-again = 重新显示
settings-library-calendar = 日历
settings-library-calendar-feed = 日历中的播出时间
settings-library-calendar-feed-helper = 生成一个日历文件，包含你列表中番剧即将播出的剧集，排期变化时会更新。
settings-library-calendar-file = 日历文件
settings-library-calendar-file-helper = 在你的日历应用中订阅这个文件。
settings-library-copy = 复制
settings-library-path-copied = 已复制路径
settings-library-open-folder = 打开文件夹
settings-library-notifications = 通知
settings-library-new-episodes = 新剧集
settings-library-new-episodes-helper = “在看”列表中的番剧有新一集播出时，弹出桌面通知。点击即可打开番剧。
settings-library-folder-added = 已添加文件夹，正在扫描

## SettingsAppearanceTab

settings-appearance-language = 语言
settings-appearance-language-system = 跟随系统（{ $language }）
settings-appearance-text-smaller = 较小
settings-appearance-text-normal = 标准
settings-appearance-text-large = 大
settings-appearance-text-larger = 更大
settings-appearance-text-largest = 最大
settings-appearance-title = 外观
settings-appearance-lead = 更改会立即在这里和每个页面上生效。设置保存在 ~/.config/anibeam/theme.toml，也可以手动编辑。
settings-appearance-colours = 颜色
settings-appearance-mode = 模式
settings-appearance-mode-dark = 深色
settings-appearance-mode-light = 浅色
settings-appearance-mode-system = 跟随系统
settings-appearance-colours-from = 配色来源
settings-appearance-from-theme-helper = 下方的一对主题，一个用于深色，一个用于浅色。
settings-appearance-from-desktop-helper = 你的终端配色，否则使用桌面的配色方案和强调色。
settings-appearance-from-desktop = 桌面
settings-appearance-from-theme = 主题
settings-appearance-dark-theme = 深色主题
settings-appearance-light-theme = 浅色主题
settings-appearance-themes-helper = 放进 ~/.config/anibeam/themes 的主题文件会出现在两个列表中。
settings-appearance-accent = 强调色
settings-appearance-accent-helper = 用来标记已选中、正在播放或新增的内容。
settings-appearance-icon = 图标
settings-appearance-app-icon = 应用图标
settings-appearance-app-icon-helper = 窗口、任务栏和启动器上的图标。深色版会对调黄色和黑色。
settings-appearance-icon-default = 默认
settings-appearance-icon-default-name = 默认应用图标
settings-appearance-icon-dark = 深色
settings-appearance-icon-dark-name = 深色应用图标
settings-appearance-size-motion = 大小和动效
settings-appearance-text-size = 文字大小
settings-appearance-text-size-helper = Anibeam 中所有文字的大小，叠加在系统自身的字体大小之上。
settings-appearance-density = 密度
settings-appearance-density-helper = 各元素周围的留白，从紧凑到宽松。
settings-appearance-density-compact = 紧凑
settings-appearance-density-normal = 标准
settings-appearance-density-comfortable = 宽松
settings-appearance-poster-size = 海报大小
settings-appearance-poster-size-helper = 番剧在媒体库和首页上显示的大小。
settings-appearance-poster-small = 小
settings-appearance-poster-medium = 中
settings-appearance-poster-large = 大
settings-appearance-out-now = “正在播出”的已播出
settings-appearance-out-now-helper = 带有番剧横幅的卡片，或由小海报和标题组成、三行深的列表。
settings-appearance-out-now-cards = 卡片
settings-appearance-out-now-list = 列表
settings-appearance-out-now-grid = 已播出卡片
settings-appearance-out-now-grid-helper = 点“全部”展开其余内容前显示多少卡片，列数乘行数。
settings-appearance-grid = { $columns } × { $rows }
settings-appearance-reduce-motion = 减少动效
settings-appearance-reduce-motion-helper = 跳过媒体库的开场动画和其他纯装饰性的动效。

## LookPane

look-pane-sample-series = 示例番剧
look-pane-episode-code = EP { $number }
look-pane-library = 媒体库
look-pane-series-count = { $count } 部番剧
look-pane-search = 搜索标题或文件夹
look-pane-search-hint = / 或 Ctrl+K
look-pane-all = 全部
look-pane-series = 番剧
look-pane-movies = 剧场版
look-pane-plain = 普通
look-pane-selected = 已选中
look-pane-errors = { $count } 个错误
look-pane-type-large = 大
look-pane-type-normal = 标准
look-pane-type-small = 小，暗淡
look-pane-sample-switch = 示例开关
look-pane-button = 按钮
look-pane-scan = 扫描
look-pane-sample-status = 《{ $title }》有 { $count } 集新剧集

## FeedPage

feed-title = 动态
feed-aired = { $episode }，{ $when }播出
feed-downloaded = 文件下载于{ $when }
feed-countdown-days = { $days }天{ $hours }小时
feed-countdown-hours = { $hours }小时{ $minutes }分
feed-countdown-minutes = { $minutes }分钟
feed-section-this-week = 本周
feed-new-episodes = { $count } 集新剧集
feed-section-earlier = 更早
feed-latest-out = 最新一集{ $when }播出
feed-section-coming-soon = 即将播出
feed-this-week-count = 本周 { $count } 集
feed-section-everything-else = 其他
feed-out-this-week = 本周播出 { $count } 集
feed-nothing-scheduled = 还没有排期。
feed-next = 下一集：《{ $title }》，{ $countdown }后
feed-nothing-new = 本周没有新内容。
feed-new-this-week = 本周有 { $count } 集新剧集。
feed-sort-recent = 最近发布
feed-sort-upcoming = 即将播出
feed-failed-title = 动态没有加载
feed-empty-title = 这里还没有内容
feed-failed-body = Anibeam 刚才无法读取动态，请稍后再试。
feed-empty-body-folders = 剧集播出和到来时，动态会逐渐填满。“正在播出”会显示本季的番剧。
feed-empty-body = 添加存放动漫的文件夹，动态就会填满它的剧集。
feed-add-folder = 添加文件夹
feed-airing-now = 正在播出

## RecommendedPage

recommended-title = 为你推荐
recommended-ranked-minutes = { $minutes } 分钟前排序
recommended-ranked-hours = { $hours } 小时前排序
recommended-ranked-days = { $days } 天前排序
recommended-scope-name = 推荐范围
recommended-scope-all = 全部
recommended-scope-library = 媒体库内
recommended-scope-list = 列表内
recommended-ranking = 正在排序
recommended-rank-again = 重新排序
recommended-no-scores-title = 先给几部番剧评分
recommended-no-list-title = 还没有可学习的内容
recommended-never-fitted-title = 正在为你的番剧排序
recommended-nothing-left-title = 这里没有更多了
recommended-no-scores-body = 推荐会根据你列表中的评分学习。在 AniList 上给几部番剧评分，推荐就会出现在这里。
recommended-no-list-body = 在设置中连接 AniList，或在这台设备上保留一个列表，你评分的番剧就会训练这个页面。
recommended-never-fitted-body = 第一次排序需要读取 AniList 一两分钟，在后台进行。
recommended-nothing-left-library-body = 媒体库中合适的番剧都已看过。“全部”会找出你还没有的番剧。
recommended-nothing-left-body = 列表中合适的番剧都已看过。“全部”会找出你还没加入列表的番剧。

## MoveListsSheet

move-lists-moving = 正在把 { $count } 部番剧移到 { $site }
move-lists-score = 评分 { $score }
move-lists-title = 把你的列表移到 { $site }
move-lists-body = 有 { $count } 部番剧的状态或评分只在这台电脑上。移动后它们会进入你的 { $site } 列表，手机和个人资料都能看到。对这些番剧，{ $site } 上现有的内容会被替换。点“移动”之前不会发送任何内容。
move-lists-head-show = 番剧
move-lists-head-here = 这台电脑上
move-lists-head-tracker = { $site } 当前
move-lists-nothing = 无
move-lists-not-on-list = 不在你的列表中
move-lists-later-hint = 稍后：设置中的“账号”
move-lists-not-now = 以后再说
move-lists-move = 移动 { $count } 部番剧

## StatsPage

stats-title = 统计
stats-month-jan = 1月
stats-month-feb = 2月
stats-month-mar = 3月
stats-month-apr = 4月
stats-month-may = 5月
stats-month-jun = 6月
stats-month-jul = 7月
stats-month-aug = 8月
stats-month-sep = 9月
stats-month-oct = 10月
stats-month-nov = 11月
stats-month-dec = 12月
stats-nothing-yet = 还没有
stats-ranked-episodes = { $count } 集
stats-year-summary = 你在 { $year } 年看了什么
stats-empty-title = 还没有观看记录
stats-empty-body = 看完一集就会计入这里：时长、类型和你看过的季度。
stats-open-library = 媒体库
stats-hours-watched = 观看小时数
stats-episodes-label = 集
stats-series-finished = 部番剧看完
stats-none = 无
stats-busiest-month = 最忙的月份
stats-by-month = 按月
stats-month-hours = { $hours } 小时
stats-top-genres = 最常看的类型
stats-top-studios = 最常看的制作公司
stats-seasons-watched = 看过的季度
stats-recap-title = 我的年度动漫
stats-recap-hours = 小时
stats-recap-episodes = 集
stats-recap-top-genre = 最爱类型
stats-save-image = 保存为图片
stats-save-dialog-title = 保存年度回顾
stats-save-filter-png = PNG 图片 (*.png)
stats-recap-saved = 年度回顾已保存
stats-recap-save-failed = 无法保存年度回顾

## TrackerRow

tracker-row-connected = 已以 { $user } 连接，{ $when }同步
tracker-row-unknown-user = ?
tracker-row-never = 从未
tracker-row-waiting = 正在等待你在浏览器中批准
tracker-row-not-connected = 未连接
tracker-row-open-again = 重新打开
tracker-row-open-again-tip = 重新打开登录页面
tracker-row-disconnect = 断开连接
tracker-row-disconnect-question = 断开 { $name }？你的登录信息会被移除。
tracker-row-secret-saved = 已为此应用保存客户端密钥。
tracker-row-remove-secret = 移除
tracker-row-secret-removed = 已移除客户端密钥
tracker-row-add-secret = 添加客户端密钥
tracker-row-secret-where = 它是 myanimelist.net/apiconfig 应用页面上“Client Secret”下方的长字符串。
tracker-row-secret-placeholder = 客户端密钥
tracker-row-save-connect = 保存并连接

## RemoveSheet

remove-unsubscribe-delete = 取消订阅并删除
remove-unsubscribe-torrents = 取消订阅并移除种子
remove-unsubscribe = 取消订阅
remove-delete-files = 移除并删除文件
remove-torrents = 移除种子
remove-also-torrents = 同时移除它的 { $count } 个种子
remove-torrents-go-with-files = 删除文件时会一并移除种子。
remove-torrents-files-stay = 它们会从客户端移除，文件会保留。
remove-also-files = 同时删除文件
remove-files-default-where = 种子下载的内容
remove-files-detail-rule = 把{ $where }移到回收站，并移除种子。
remove-files-detail = 把{ $where }移到回收站。

## NotesSheet

notes-your-tracker = 你的追踪服务
notes-title = 笔记
notes-placeholder = 看到哪里、谁推荐的、该下哪个资源
notes-over-limit = 超出限制 { $count } 个字符
notes-saved-to = 已保存到 { $sites }

## RecommendationCard

recommendation-card-available = 可获取
recommendation-card-open-library = 在媒体库中打开《{ $title }》
recommendation-card-open-anilist = 在 AniList 上打开《{ $title }》

## SearchField

search-field-placeholder = 搜索标题或文件夹
search-field-hint = / 或 Ctrl+K
search-field-clear = 清除搜索

## RecommendedHover

recommended-hover-anilist-score = AniList { $score }
recommended-hover-forecast = { $percent }%
recommended-hover-hidden-tags = { $count } 个你喜爱的隐藏标签
recommended-hover-because = 因为你看过

## PersonCard

person-card-unknown = 未知
person-card-role-main = 主角
person-card-role-supporting = 配角
person-card-role-background = 龙套
person-card-the-character = 该角色
person-card-open = 在 AniList 上打开 { $name }

## StatusDot

status-dot-on-list = 在你的列表中：{ $status }

## TrackerNudge

tracker-nudge-title = 连接你的动漫列表
tracker-nudge-body = 你观看时，Anibeam 会在 AniList 或 MyAnimeList 上标记剧集。
tracker-nudge-connect-anilist = 连接 AniList
tracker-nudge-connect-mal = 连接 MyAnimeList
tracker-nudge-not-now = 以后再说

## RecommendedCard

recommended-card-spoken = 《{ $title }》，契合度百分之 { $percent }
recommended-card-forecast = { $percent }%
recommended-card-in-library = 已在库中

## InlineConfirm

inline-confirm-keep = 保留

## Tokens

tokens-cube-watched-tracked = 已看，已在线追踪
    .inline = 已看，已在线追踪
tokens-cube-watched-here = 在这里看过，尚未追踪
    .inline = 在这里看过，尚未追踪
tokens-cube-on-disk = 已在磁盘上
    .inline = 已在磁盘上
tokens-cube-downloading = 下载中
    .inline = 下载中
tokens-cube-missing = 缺失
    .inline = 缺失
tokens-cube-error = 错误
    .inline = 错误
tokens-cube-not-downloaded = 未下载
    .inline = 未下载
tokens-cube-not-aired = 尚未播出
    .inline = 尚未播出
tokens-cube-available = 可获取
    .inline = 可获取
tokens-cube-picked = 已选择获取
    .inline = 已选择获取
tokens-cube-already-on-disk = 已在磁盘上
    .inline = 已在磁盘上
tokens-cube-no-release = 没有资源
    .inline = 没有资源

## Card

card-not-interested = 不感兴趣

## PagePlaceholder

page-placeholder-coming-soon = 即将推出

## ErrorLine

error-line-hide-details = 隐藏详情
error-line-details = 详情
error-line-details-tip = 逐字显示出错内容，并可复制
error-line-copied = 已复制。

## Readouts (apps/linux/src/format.rs): times, countdowns and lengths

fmt-just-now = 刚刚
fmt-now = 现在
fmt-ago = { $span }前
fmt-in = { $span }后
fmt-seconds-short = { $n }秒
fmt-minutes-short = { $n }分钟
fmt-hours-short = { $n }小时
fmt-days-short = { $n }天
fmt-months-short = { $n }个月
fmt-years-short = { $n }年
fmt-years-months-short = { $years }年{ $months }个月
fmt-countdown-days = { $d }天{ $h }小时{ $m }分
fmt-countdown-hours = { $h }小时{ $m }分
fmt-countdown-days-seconds = { $d }天{ $h }小时{ $m }分{ $s }秒
fmt-countdown-hours-seconds = { $h }小时{ $m }分{ $s }秒
fmt-countdown-minutes-seconds = { $m }分{ $s }秒
fmt-runtime-minutes = { $m } 分钟
fmt-runtime-hours = { $h } 小时
fmt-runtime-hours-minutes = { $h } 小时 { $m } 分钟
fmt-episode = 剧集
fmt-size-b = { $size } B
fmt-size-kb = { $size } KB
fmt-size-mb = { $size } MB
fmt-size-gb = { $size } GB
fmt-size-tb = { $size } TB
fmt-episode-span = 第 { $first } 至 { $through } 集

## apps/linux/src/problem.rs: every call error in a person's words, what happened and what to do

problem-report = 请重试。如果一直出现，请从活动日志中反馈。
problem-try-later = 请稍后再试。
problem-helper-unreachable = 搜索助手没有响应。
problem-helper-unauthorized = 搜索助手拒绝了登录。
problem-helper-not-running = 搜索助手没有运行。
problem-helper-missing = 未安装搜索助手。
problem-helper-failed = 搜索助手无法完成该操作。
problem-helper-no-login = 搜索助手无法登录种子客户端。
problem-helper-no-login-next = 请在设置的“下载”中保存种子客户端的密码。
problem-torrent-unreachable = 种子客户端没有响应。
problem-torrent-unauthorized = 种子客户端拒绝了登录。
problem-torrent-not-running = 种子客户端没有运行。
problem-torrent-missing = 未安装种子客户端。
problem-torrent-failed = 种子客户端无法完成该操作。
problem-downloads-unreachable-next = 请确认它正在运行，并在设置的“下载”中检查地址是否正确。
problem-downloads-unauthorized-next = 请在设置的“下载”中检查用户名和密码。
problem-downloads-not-running-next = 请在设置的“下载”中启动它。
problem-downloads-missing-next = 设置的“下载”中说明了如何设置。
problem-trash = { -app-name } 无法把文件移到回收站。
problem-trash-next = 请确认这些文件属于你，且磁盘不是只读的。
problem-source-list = 该来源列表无法使用。
problem-source-list-next = 请在设置的“下载”>“来源列表”中检查它的地址。
problem-remote-unreachable = { $name } 没有响应。
problem-remote-unreachable-next = 该站点可能已宕机。请稍后再试，或换个来源搜索。
problem-remote-unauthorized = { $name } 拒绝了提供的密钥。
problem-remote-unauthorized-next = 请在设置的“下载”>“来源列表”中更改它的密钥。
problem-remote-failed = { $name } 无法完成该操作。
problem-remote-failed-next = 请稍后再试，或换个来源搜索。
problem-provider-sign-in = { $name } 拒绝了 { -app-name } 的登录。
problem-provider-sign-in-next = 请在设置的“账号”中重新连接。
problem-provider-turned-away = { $name } 拒绝了 { -app-name } 的请求。
problem-provider-busy = { $name } 现在很忙。
problem-provider-busy-next = 请等一分钟后重试。
problem-provider-no-answer = { $name } 没有响应。
problem-provider-no-answer-next = 请检查网络连接后重试。
problem-provider-trouble = { $name } 现在出了点问题。
problem-provider-trouble-next = 请过几分钟再试。
problem-provider-rejected = { $name } 没有接受。
problem-not-found = { -app-name } 已找不到它。
problem-not-found-next = 它可能已被移除。请重试以重新加载页面。
problem-invalid-next = 请检查后重试。
problem-unavailable = 现在无法访问该文件夹。
problem-unavailable-next = 请接上它所在的磁盘，或在设置的“媒体库”中检查文件夹。
problem-not-connected = 未连接 { $tracker }。
problem-not-connected-next = 请在设置的“账号”中连接。
problem-io = { -app-name } 无法读写文件。
problem-io-next = 请确认磁盘有空间且文件夹属于你，然后重试。
problem-storage = { -app-name } 无法保存你的媒体库。
problem-storage-next = 请确认磁盘有空间，然后重试。
problem-keyring = 系统密码存储没有响应。
problem-keyring-next = 请解锁桌面的密钥环，然后重试。
problem-unsupported = { -app-name } 在这里暂时无法做到。
problem-version = 这份备份来自更新版本的 { -app-name }。
problem-version-next = 请在设置的“关于”中更新 { -app-name }，然后重新导入。
problem-internal = { -app-name } 内部出了问题。
problem-refused-hidden = 这部番剧已隐藏。
problem-refused-hidden-next = 请先在它的页面上取消隐藏。
problem-refused-no-match = 这部番剧尚未匹配。
problem-refused-no-match-next = 请先在“番剧信息”中匹配。
problem-refused-not-newer = 你的列表进度已超过这一集。
problem-refused-extra = 这个文件是附加内容，不是剧集。
problem-refused-unmatched = 这个文件没有对应到任何剧集。
problem-refused-unmatched-next = 请在番剧页面的“修正”中处理。
problem-refused-on-disk = 这个文件只在这台电脑上。

## apps/linux/src/library_view.rs: the library page's section titles and the one fact under each

library-view-airing = 播出中
library-view-no-date = 无日期
library-view-type-tv = TV
library-view-type-movies = 剧场版
library-view-type-extras = OVA、ONA 和特别篇
library-view-type-music = 音乐
library-view-type-unknown = 未知
library-view-no-genre = 未匹配
library-view-state-caught-up = 已追上
library-view-state-not-started = 未开始
library-view-next-episode = 下一集 { $span }后
library-view-countdown-days = { $d }天{ $h }小时
library-view-countdown-hours = { $h }小时{ $m }分
library-view-countdown-minutes = { $m }分钟
library-view-in-progress = { $count } 部在看
library-view-not-started = { $count } 部未开始
library-view-all-watched = 全部已看
library-view-on-disk = 磁盘上有 { $count } 集
library-view-dropped-on-disk = 磁盘上还有 { $count } 个文件

## apps/linux/src/dbus/notify.rs: the desktop notification for a new episode

notify-episode-out = 《{ $title }》第 { $episode } 集已播出
notify-click-open = 点击在 { -app-name } 中打开
notify-click-get = 点击在 { -app-name } 中获取剧集
notify-get-episodes = 获取剧集

## apps/linux/src/nowplaying.rs: the second line a media widget shows while an episode plays

now-playing-show-episode = { $show }，{ $episode }

## apps/linux/src/dbus/inhibit.rs: why the screen stays on, as the desktop's power panel shows it

inhibit-reason = 正在播放剧集

## apps/linux/src/bridge/door.rs: why a value a page sent cannot be used (an error's detail line)

door-not-a-choice = { $value } 不是有效的选项
door-tokens-op-empty = tokens 操作需要 tokens
door-not-a-refine-op = { $value } 不是有效的筛选操作
door-limit-too-low = 必须至少为 1，收到的是 { $limit }
door-not-a-torrent-action = { $value } 不是有效的种子操作

## apps/linux/src/bridge/shell.rs: the first-run starter folder and the update notice

shell-starter-folder-failed = 无法创建 { $folder }：{ $error }
shell-update-gone = { -app-name } { $version } 已不再提供

## Watch together (#846)

onboarding-step-together = 一起看
onboarding-title-together = 和朋友一起看
onboarding-body-together = 在播放器中创建房间并发送房间代码。每个人播放自己的那份剧集，播放、暂停和跳转对所有人保持同步。
onboarding-ready-together = 一起看
onboarding-ready-together-as = 你在房间中显示为 { $name }。
onboarding-ready-together-none = 还没有名字。第一次使用时房间面板会询问。
onboarding-together-you = 你
onboarding-together-code = 房间代码
onboarding-together-episode = Frieren · 第 3 集
onboarding-together-friend-1 = Sam
onboarding-together-friend-2 = Robin
onboarding-together-name = 你在房间中的名字
rail-room = 一起看（Ctrl+J）
rail-room-in = 正在一起看房间中（Ctrl+J）
player-chrome-watch-together = 一起看
player-chrome-room = 房间 { $code } · { $count ->
   *[other] { $count } 人
}
room-name-empty = 输入房间里其他人会看到的名字。
room-name-bad = 这个名字不可用。请使用 1 到 32 个字符。
room-error-not-found = 没有使用该代码的房间。请向发送者核对，房间可能已关闭。
room-error-full = 该房间已满。每个房间最多 10 人。
room-error-unreachable = 无法连接房间服务器。请检查网络连接后重试。
room-error-refused = 房间服务器拒绝了请求。请过一分钟再试。
room-error-proto = 该房间使用更新版本的房间功能。请更新 { -app-name } 后再加入。
room-error-not-in-room = 你已不在该房间中。请重新创建或加入房间。
room-error-bad-code = 房间代码是 6 位字母和数字，例如 K7QM2X。
room-error-bad-relay = 设置的“播放”中的房间服务器不是网址。请在那里修改后重试。
room-panel-title = 一起看
room-panel-close = 关闭
room-panel-connected = 已连接
room-panel-reconnecting = 正在重新连接
room-panel-name-label = 你在房间中的名字
room-panel-name-placeholder = 你的名字
room-panel-name-start = 创建房间
room-panel-name-join = 加入房间
room-panel-start = 创建房间
room-panel-or-join = 或用房间代码加入
room-panel-join-label = 房间代码
room-panel-join = 加入
room-panel-you-are = 你显示为 { $name }
room-panel-change-name = 更改
room-panel-code-label = 房间代码
room-panel-copy = 复制链接
room-panel-link-copied = 已复制房间链接
room-panel-watching = { $series } · { $episode }
room-panel-missing-here = 你的媒体库中没有这一集，暂时无法一起播放。
room-panel-open-episode = 打开这一集
room-panel-people = { $count ->
   *[other] { $count } 人
}
room-panel-host = 房主
room-panel-you = 你
room-panel-ready = 就绪
room-panel-opening = 正在打开
room-panel-missing = 没有这一集
room-panel-offset = 你的偏移
room-panel-offset-earlier = 提前 0.5 秒
room-panel-offset-later = 延后 0.5 秒
room-panel-seconds = { $value } 秒
room-panel-suggested = 根据片头建议 { $offset }
room-panel-use-suggested = 使用
room-panel-leave = 离开房间
player-room-speed = 速度跟随房间
settings-find-room-name =
    .words = 一起看 房间 名字 朋友
settings-find-room-relay =
    .words = 一起看 房间 中继 服务器
key-help-key-ctrl-j = Ctrl+J
key-help-room = 一起看
settings-playback-together = 一起看
settings-playback-room-name = 你在一起看房间中的名字
settings-playback-room-name-placeholder = 你的名字
settings-playback-room-relay = 房间服务器
settings-playback-room-relay-helper = 房间在这里创建。只有使用你自己运行的服务器时才需要更改。
