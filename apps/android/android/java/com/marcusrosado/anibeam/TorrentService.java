package com.marcusrosado.anibeam;

import android.app.Activity;
import android.app.ActivityManager;
import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.content.pm.ServiceInfo;
import android.os.Build;
import android.os.IBinder;
import android.util.Log;

/**
 * Keeps the process alive while episodes download. It holds no logic: the downloads run in
 * Rust, in the core's torrent engine, and this only gives the process a notification Android
 * counts as foreground work, so it does not cut the network when the screen goes off. The
 * app calls `start` with what to show, `update` as it changes, and `stop` when nothing is
 * downloading, and `state` tells it where the work stands. A finished episode gets a
 * notification of its own, `ready`.
 *
 * On Android 14 and later the work is a user-initiated data transfer job (DownloadJob),
 * which has no daily limit; before that, this class is a dataSync foreground service, which
 * has none there either. When Android stops the work all the same, `state` says so: the
 * app pauses the downloads and posts `paused`, and they go on when the work runs again or
 * the app is opened.
 *
 * The words Java shows itself, the two channel names and the title it falls back to, come
 * from Rust in the app's language: as arguments to `start`, and through `words` when the
 * language changes. The English below is only for a call that carries none.
 */
public class TorrentService extends Service {
    private static final String TAG = "Anibeam";
    private static final String CHANNEL = "downloads";
    /** Finished episodes: a channel of its own, so it can go quiet and the ongoing one stays. */
    private static final String READY = "ready";
    /** The ongoing notification's id; a finished episode's is 2 or more (notify.rs). */
    static final int NOTIFICATION = 1;
    /** "Paused": above every id a finished episode can get. */
    private static final int PAUSED = Integer.MAX_VALUE;
    private static String downloadsName = "Downloads";
    private static String readyName = "Episodes ready";
    private static String fallbackTitle = "Downloading";
    /** What the ongoing notification says, kept so a job that starts later shows it. */
    static volatile String shownTitle;
    static volatile String shownText;
    /** Tenths of a percent, or -1 while nothing says how far along it is. */
    static volatile int shownProgress = -1;
    /** The app's native side called `start` in this process, so downloads run in it. */
    static volatile boolean live;

    /**
     * What the download work is doing, as `state` tells Rust. Java sees every start, stop
     * and cancel, so Rust reads this instead of guessing from what the downloads look like.
     */
    static final int IDLE = 0;
    /** The job runs, or the service is up. */
    static final int RUNNING = 1;
    /** The job is scheduled and Android has not run it yet. */
    static final int WAITING = 2;
    /** Android stopped the work, or the app rested it; it goes on when the app is opened. */
    static final int STOPPED = 3;
    /** Stopped from the phone's list of running apps: stopped until the app is opened again. */
    static final int STOPPED_BY_USER = 4;
    static final Object LOCK = new Object();
    private static int work = IDLE;
    /** Goes up on every change of `work`, so Rust sees a stop and a rerun between two looks. */
    private static int generation;
    /** The app was seen away from the screen since the person stopped the work. */
    private static boolean away;

    /** Android or the person stopped the work. Called with LOCK held. */
    static boolean stoppedNow() {
        return work == STOPPED || work == STOPPED_BY_USER;
    }

    /** The job runs or the service is up. Called with LOCK held. */
    static boolean runningNow() {
        return work == RUNNING;
    }

    /** The job is scheduled and has not run yet. Called with LOCK held. */
    static boolean waitingNow() {
        return work == WAITING;
    }

    /** Android refused to start the work since it last started, which is logged once. */
    private static boolean refusalSaid;

    /** Android refused to start the work: said once for a run of refusals. LOCK is held. */
    static void refused(String what, RuntimeException e) {
        if (refusalSaid) return;
        refusalSaid = true;
        Log.w(TAG, what, e);
    }

    /** The work started: the next refusal is said again. LOCK is held. */
    static void started() {
        refusalSaid = false;
    }

    /** Moves the work to `state`. Called with LOCK held. */
    static void set(int state) {
        if (state == work) return;
        work = state;
        generation++;
        away = false;
    }

    /** Android 14 and later run downloads as a user-initiated data transfer job. */
    private static boolean jobs() {
        return Build.VERSION.SDK_INT >= 34;
    }

    private static void keep(String title, String text, int progress) {
        shownTitle = title;
        shownText = text;
        shownProgress = progress;
    }

    /**
     * Starts the work that keeps downloads going, showing `title`, `text` and `progress`.
     * False when Android refused, as it does for an app in the background; the app tries
     * again on its next look.
     */
    public static boolean start(Context context, String title, String text, int progress,
            String channelDownloads, String channelReady, String fallback) {
        live = true;
        takeWords(channelDownloads, channelReady, fallback);
        keep(title, text, progress);
        // Downloads run in this process now, so a Paused left by one that died goes.
        cancelPaused(context);
        synchronized (LOCK) {
            if (jobs()) return DownloadJob.ensure(context, true);
            Intent intent = new Intent().setClassName(context, TorrentService.class.getName());
            try {
                context.startForegroundService(intent);
                started();
                set(RUNNING);
                return true;
            } catch (RuntimeException e) {
                refused("the download service did not start", e);
                return false;
            }
        }
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        channels(this);
        Notification n = ongoing(this, shownTitle, shownText, shownProgress);
        if (Build.VERSION.SDK_INT >= 29) {
            startForeground(NOTIFICATION, n, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC);
        } else {
            startForeground(NOTIFICATION, n);
        }
        return START_NOT_STICKY;
    }

    /** Keeps the words that arrived; one that is missing or empty leaves the one held. */
    private static void takeWords(String downloads, String ready, String fallback) {
        if (downloads != null && !downloads.isEmpty()) downloadsName = downloads;
        if (ready != null && !ready.isEmpty()) readyName = ready;
        if (fallback != null && !fallback.isEmpty()) fallbackTitle = fallback;
    }

    /**
     * The app's words in its language, from Rust: before an episode is ready, and when the
     * language changes. A channel that already exists is renamed in place, so the phone's
     * notification settings follow the language; none is made that was not there.
     */
    public static void words(Context context, String downloads, String ready, String fallback) {
        takeWords(downloads, ready, fallback);
        NotificationManager nm = context.getSystemService(NotificationManager.class);
        if (nm == null) return;
        if (nm.getNotificationChannel(CHANNEL) != null || nm.getNotificationChannel(READY) != null) {
            channels(context);
        }
    }

    /**
     * Both channels, made as the service starts, so Episodes ready is in the app's
     * notification settings to turn off before the first episode finishes.
     */
    static void channels(Context context) {
        NotificationManager nm = context.getSystemService(NotificationManager.class);
        if (nm == null) return;
        nm.createNotificationChannel(
                new NotificationChannel(CHANNEL, downloadsName, NotificationManager.IMPORTANCE_LOW));
        nm.createNotificationChannel(
                new NotificationChannel(READY, readyName, NotificationManager.IMPORTANCE_DEFAULT));
    }

    static Notification ongoing(Context context, String title, String text, int progress) {
        Notification.Builder b = new Notification.Builder(context, CHANNEL)
                .setContentTitle(title != null ? title : fallbackTitle)
                .setSmallIcon(android.R.drawable.stat_sys_download)
                .setOngoing(true)
                .setOnlyAlertOnce(true)
                .setShowWhen(false)
                .setContentIntent(openApp(context))
                .setProgress(1000, Math.max(progress, 0), progress < 0);
        if (text != null && !text.isEmpty()) b.setContentText(text);
        return b.build();
    }

    /**
     * Changes the notification in place: what is downloading and how far along. Posting
     * under the service's own id is allowed from the background, where a second
     * startForegroundService is not. On Android 14 and later the job takes the words, and is
     * scheduled again if Android ended it while the app was open; false when that was refused.
     */
    public static boolean update(Context context, String title, String text, int progress) {
        keep(title, text, progress);
        if (jobs()) {
            synchronized (LOCK) {
                return DownloadJob.ensure(context, false);
            }
        }
        NotificationManager nm = context.getSystemService(NotificationManager.class);
        if (nm == null) return true;
        nm.notify(NOTIFICATION, ongoing(context, title, text, progress));
        return true;
    }

    /** Ends the work: nothing is downloading. */
    public static void stop(Context context) {
        keep(null, null, -1);
        synchronized (LOCK) {
            end(context);
            set(IDLE);
        }
        cancelPaused(context);
    }

    /**
     * Ends the work the way Android stopping it does: nothing moved for a long while with the
     * app closed, so the downloads pause until it is opened instead of keeping the phone
     * awake for a torrent nobody seeds.
     */
    public static void rest(Context context) {
        synchronized (LOCK) {
            end(context);
            set(STOPPED);
        }
    }

    /** The person asked a paused download to go on: a stop ends here. */
    public static void goOn(Context context) {
        synchronized (LOCK) {
            if (stoppedNow()) {
                if (jobs()) DownloadJob.cancel(context);
                set(IDLE);
            }
        }
        cancelPaused(context);
    }

    /** Cancels the job or stops the service, whichever this Android runs. LOCK is held. */
    private static void end(Context context) {
        if (jobs()) {
            DownloadJob.cancel(context);
        } else {
            context.stopService(new Intent().setClassName(context, TorrentService.class.getName()));
        }
    }

    /**
     * The work's state for Rust: the generation in the bits from 4 up, 8 while the app is on
     * the screen, and IDLE to STOPPED_BY_USER below. Opening the app is when a stop ends:
     * a stop by Android at once, a stop by the person once the app was away and came back.
     * The job Android holds back after a stop then goes (it backs off, and after a few
     * timeouts a day it waits for hours), and the app starts a fresh one.
     */
    public static int state(Context context) {
        boolean shown = onScreen();
        boolean lifted = false;
        int out;
        synchronized (LOCK) {
            if (work == STOPPED && shown) {
                if (jobs()) DownloadJob.cancel(context);
                set(IDLE);
                lifted = true;
            } else if (work == STOPPED_BY_USER) {
                if (!shown) {
                    away = true;
                } else if (away) {
                    if (jobs()) DownloadJob.cancel(context);
                    set(IDLE);
                    lifted = true;
                }
            }
            out = (generation << 4) | (shown ? 8 : 0) | work;
        }
        if (lifted) cancelPaused(context);
        return out;
    }

    /** The app is in front: on the screen, not only kept up by the job or the service. */
    static boolean onScreen() {
        ActivityManager.RunningAppProcessInfo info = new ActivityManager.RunningAppProcessInfo();
        ActivityManager.getMyMemoryState(info);
        return info.importance <= ActivityManager.RunningAppProcessInfo.IMPORTANCE_FOREGROUND;
    }

    /** "Paused" comes down: the downloads go on. */
    static void cancelPaused(Context context) {
        NotificationManager nm = context.getSystemService(NotificationManager.class);
        if (nm != null) nm.cancel(PAUSED);
    }

    /** Says the downloads are paused, `title` being what they are; a tap opens the app. */
    public static void paused(Context context, String title, String text) {
        NotificationManager nm = context.getSystemService(NotificationManager.class);
        if (nm == null) return;
        channels(context);
        Notification n = new Notification.Builder(context, CHANNEL)
                .setContentTitle(title)
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_media_pause)
                .setShowWhen(false)
                .setContentIntent(openApp(context))
                .build();
        nm.notify(PAUSED, n);
    }

    /** Opens Anibeam where it was, the way its launcher icon does. */
    private static PendingIntent openApp(Context context) {
        Intent open = context.getPackageManager().getLaunchIntentForPackage(context.getPackageName());
        if (open == null) open = new Intent();
        open.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED);
        // PendingIntent.FLAG_IMMUTABLE is 1 << 26, spelled out for the --release 8 build.
        return PendingIntent.getActivity(context, 0, open, PendingIntent.FLAG_UPDATE_CURRENT | (1 << 26));
    }

    /**
     * Says an episode finished downloading, under `id` so the next one does not replace it.
     * Tapping it opens the app. Off when the person turned the channel or notifications off.
     */
    public static void ready(Context context, int id, String title, String text) {
        NotificationManager nm = context.getSystemService(NotificationManager.class);
        if (nm == null) return;
        channels(context);
        Notification n = new Notification.Builder(context, READY)
                .setContentTitle(title)
                .setContentText(text)
                .setSmallIcon(android.R.drawable.stat_sys_download_done)
                .setAutoCancel(true)
                .setContentIntent(openApp(context))
                .build();
        nm.notify(id, n);
    }

    /**
     * Asks for the notification permission Android 13 and later want before this service
     * may show its notification: true when the ask went out or was not needed. Only an
     * Activity can ask, and from its UI thread; the native side calls this from its own.
     * Android stops showing its dialog after two denials, and the app cannot tell that apart
     * from never asked (an earlier build asked without saving that it had). So when no dialog
     * took the window's focus a moment after the ask, the settings open instead.
     */
    public static boolean askPermission(final Activity activity) {
        if (Build.VERSION.SDK_INT < 33) return true;
        // Manifest.permission.POST_NOTIFICATIONS, spelled out for the --release 8 build.
        final String permission = "android.permission.POST_NOTIFICATIONS";
        if (activity.checkSelfPermission(permission) == PackageManager.PERMISSION_GRANTED) return true;
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                activity.requestPermissions(new String[] {permission}, 1);
                activity.getWindow().getDecorView().postDelayed(new Runnable() {
                    @Override
                    public void run() {
                        if (activity.hasWindowFocus()
                                && activity.checkSelfPermission(permission) != PackageManager.PERMISSION_GRANTED) {
                            openSettings(activity);
                        }
                    }
                }, 800);
            }
        });
        return true;
    }

    /**
     * Opens this app's notification settings, the one way back once Android stops asking:
     * after a second Don't allow, requestPermissions shows nothing. Falls back to the app's
     * info page on a phone without the notification page.
     */
    public static boolean openSettings(final Activity activity) {
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                // Settings.ACTION_APP_NOTIFICATION_SETTINGS and EXTRA_APP_PACKAGE, spelled
                // out for the --release 8 build.
                Intent notifications = new Intent("android.settings.APP_NOTIFICATION_SETTINGS")
                        .putExtra("android.provider.extra.APP_PACKAGE", activity.getPackageName());
                try {
                    activity.startActivity(notifications);
                } catch (RuntimeException e) {
                    activity.startActivity(new Intent("android.settings.APPLICATION_DETAILS_SETTINGS",
                            android.net.Uri.fromParts("package", activity.getPackageName(), null)));
                }
            }
        });
        return true;
    }

    /**
     * Android 15 gives a dataSync service six hours a day and stops the app if the service
     * outlives them. The service only runs before Android 14, so this should never come;
     * if it does, it stops in time and the app pauses the downloads until it is opened.
     */
    @Override
    public void onTimeout(int startId, int fgsType) {
        synchronized (LOCK) {
            set(STOPPED);
        }
        stopSelf();
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }
}
