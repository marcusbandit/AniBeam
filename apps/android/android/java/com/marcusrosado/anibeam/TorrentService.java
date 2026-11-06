package com.marcusrosado.anibeam;

import android.app.Activity;
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

/**
 * Keeps the process alive while episodes download. It holds no logic: the downloads run in
 * Rust, in the core's torrent engine, and this service only gives the process a foreground
 * notification so Android does not cut its network when the screen goes off. The app sends
 * what to show as the "title", "text" and "progress" extras, and stops the service when
 * nothing is downloading. A finished episode gets a notification of its own, `ready`.
 */
public class TorrentService extends Service {
    private static final String CHANNEL = "downloads";
    /** Finished episodes: a channel of its own, so it can go quiet and the ongoing one stays. */
    private static final String READY = "ready";

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        channels(this);
        String title = intent != null ? intent.getStringExtra("title") : null;
        String text = intent != null ? intent.getStringExtra("text") : null;
        // Tenths of a percent, or -1 while nothing says how far along it is.
        int progress = intent != null ? intent.getIntExtra("progress", -1) : -1;
        Notification n = ongoing(this, title, text, progress);
        if (Build.VERSION.SDK_INT >= 29) {
            startForeground(1, n, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC);
        } else {
            startForeground(1, n);
        }
        return START_NOT_STICKY;
    }

    /**
     * Both channels, made as the service starts, so Episodes ready is in the app's
     * notification settings to turn off before the first episode finishes.
     */
    private static void channels(Context context) {
        NotificationManager nm = context.getSystemService(NotificationManager.class);
        if (nm == null) return;
        nm.createNotificationChannel(
                new NotificationChannel(CHANNEL, "Downloads", NotificationManager.IMPORTANCE_LOW));
        nm.createNotificationChannel(
                new NotificationChannel(READY, "Episodes ready", NotificationManager.IMPORTANCE_DEFAULT));
    }

    private static Notification ongoing(Context context, String title, String text, int progress) {
        Notification.Builder b = new Notification.Builder(context, CHANNEL)
                .setContentTitle(title != null ? title : "Downloading")
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
     * Changes the running service's notification in place: what is downloading and how far
     * along. Posting under the service's own id is allowed from the background, where a
     * second startForegroundService is not.
     */
    public static void update(Context context, String title, String text, int progress) {
        NotificationManager nm = context.getSystemService(NotificationManager.class);
        if (nm == null) return;
        nm.notify(1, ongoing(context, title, text, progress));
    }

    /** Opens AniBeam where it was, the way its launcher icon does. */
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
     * outlives them. It stops here instead; downloads go on the next time the app is open.
     */
    @Override
    public void onTimeout(int startId, int fgsType) {
        stopSelf();
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }
}
