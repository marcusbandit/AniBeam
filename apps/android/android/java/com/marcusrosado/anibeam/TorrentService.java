package com.marcusrosado.anibeam;

import android.app.Activity;
import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.Service;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.content.pm.ServiceInfo;
import android.os.Build;
import android.os.IBinder;

/**
 * Keeps the process alive while episodes download. It holds no logic: the downloads run in
 * Rust, in the core's torrent engine, and this service only gives the process a foreground
 * notification so Android does not cut its network when the screen goes off. The app sends
 * the text to show as the "text" extra, and stops the service when nothing is downloading.
 */
public class TorrentService extends Service {
    private static final String CHANNEL = "downloads";

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        NotificationManager nm = getSystemService(NotificationManager.class);
        nm.createNotificationChannel(
                new NotificationChannel(CHANNEL, "Downloads", NotificationManager.IMPORTANCE_LOW));
        String text = intent != null ? intent.getStringExtra("text") : null;
        Notification n = new Notification.Builder(this, CHANNEL)
                .setContentTitle("AniBeam")
                .setContentText(text != null ? text : "Downloading")
                .setSmallIcon(android.R.drawable.stat_sys_download)
                .setOngoing(true)
                .setOnlyAlertOnce(true)
                .build();
        if (Build.VERSION.SDK_INT >= 29) {
            startForeground(1, n, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC);
        } else {
            startForeground(1, n);
        }
        return START_NOT_STICKY;
    }

    /**
     * Asks for the notification permission Android 13 and later want before this service
     * may show its notification: true when the ask went out or was not needed. Only an
     * Activity can ask, and from its UI thread; the native side calls this from its own.
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
