package com.marcusrosado.anibeam;

import android.app.Activity;
import android.content.Context;
import android.content.pm.ActivityInfo;
import android.net.wifi.WifiManager;
import android.os.Build;
import android.view.View;
import android.view.Window;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;

/**
 * What the player asks of the screen, which Android only lets its UI thread change: the
 * picture in landscape with the system bars hidden, and the display kept on while it plays.
 * The native side calls these from its own thread; each posts to the UI thread.
 */
public final class Screen {
    private Screen() {}

    private static WifiManager.MulticastLock multicast;

    /**
     * Holds or lets go of Wi-Fi multicast while the player looks for Chromecasts and TVs:
     * without it many phones drop the mDNS and SSDP answers to save battery.
     */
    public static synchronized void multicast(final Activity activity, final boolean on) {
        if (on) {
            if (multicast == null) {
                WifiManager wifi = (WifiManager) activity.getApplicationContext().getSystemService(Context.WIFI_SERVICE);
                if (wifi == null) return;
                multicast = wifi.createMulticastLock("anibeam-cast");
                multicast.setReferenceCounted(false);
            }
            multicast.acquire();
        } else if (multicast != null && multicast.isHeld()) {
            multicast.release();
        }
    }

    public static void player(final Activity activity, final boolean on) {
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                activity.setRequestedOrientation(on
                        ? ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
                        : ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED);
                Window window = activity.getWindow();
                if (Build.VERSION.SDK_INT >= 28) {
                    WindowManager.LayoutParams lp = window.getAttributes();
                    lp.layoutInDisplayCutoutMode = on
                            ? WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
                            : WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_DEFAULT;
                    window.setAttributes(lp);
                }
                if (Build.VERSION.SDK_INT >= 30) {
                    WindowInsetsController c = window.getInsetsController();
                    if (c == null) return;
                    if (on) {
                        c.setSystemBarsBehavior(WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
                        c.hide(WindowInsets.Type.systemBars());
                    } else {
                        c.show(WindowInsets.Type.systemBars());
                    }
                } else {
                    View decor = window.getDecorView();
                    decor.setSystemUiVisibility(on
                            ? View.SYSTEM_UI_FLAG_FULLSCREEN | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                              | View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                            : 0);
                }
            }
        });
    }

    public static void awake(final Activity activity, final boolean on) {
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                if (on) {
                    activity.getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
                } else {
                    activity.getWindow().clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
                }
            }
        });
    }
}
