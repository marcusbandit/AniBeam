package com.marcusrosado.anibeam;

import android.app.Activity;
import android.content.Context;
import android.content.pm.ActivityInfo;
import android.media.AudioManager;
import android.provider.Settings;
import android.net.wifi.WifiManager;
import android.os.Build;
import android.view.View;
import android.view.Window;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;

/**
 * What the player asks of the screen, which Android only lets its UI thread change: the
 * picture in landscape with the system bars hidden, the display kept on while it plays, and
 * the brightness a swipe sets. The volume a swipe sets is the media stream's, the same the
 * phone's buttons move.
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

    /**
     * The window's brightness from 0 to 1: its own when a swipe set one, else the phone's.
     * Phones store the setting on scales of their own (255 on most, 2047 or 4095 on some), so
     * it is read against the phone's own top, and kept within 0 to 1 whatever it says.
     */
    public static float brightness(final Activity activity) {
        float own = activity.getWindow().getAttributes().screenBrightness;
        if (own >= 0) return Math.min(1f, own);
        try {
            int value = Settings.System.getInt(activity.getContentResolver(), Settings.System.SCREEN_BRIGHTNESS);
            int top = 255;
            int id = activity.getResources().getIdentifier("config_screenBrightnessSettingMaximum", "integer", "android");
            if (id != 0) top = Math.max(1, activity.getResources().getInteger(id));
            if (value > top) top = value;
            return Math.max(0f, Math.min(1f, value / (float) top));
        } catch (Settings.SettingNotFoundException | android.content.res.Resources.NotFoundException e) {
            return 0.5f;
        }
    }

    /** Sets this window's brightness, 0 to 1; below 0 hands it back to the phone's setting. */
    public static void setBrightness(final Activity activity, final float level) {
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                Window window = activity.getWindow();
                WindowManager.LayoutParams lp = window.getAttributes();
                lp.screenBrightness = level < 0
                        ? WindowManager.LayoutParams.BRIGHTNESS_OVERRIDE_NONE
                        : Math.max(0.01f, Math.min(1f, level));
                window.setAttributes(lp);
            }
        });
    }

    /** The media volume from 0 to 1. */
    public static float volume(final Activity activity) {
        AudioManager audio = (AudioManager) activity.getSystemService(Context.AUDIO_SERVICE);
        if (audio == null) return 0.5f;
        int max = audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC);
        return max > 0 ? audio.getStreamVolume(AudioManager.STREAM_MUSIC) / (float) max : 0.5f;
    }

    /** Sets the media volume, 0 to 1, in the stream's own steps. */
    public static void setVolume(final Activity activity, final float level) {
        AudioManager audio = (AudioManager) activity.getSystemService(Context.AUDIO_SERVICE);
        if (audio == null) return;
        int max = audio.getStreamMaxVolume(AudioManager.STREAM_MUSIC);
        int step = Math.round(Math.max(0f, Math.min(1f, level)) * max);
        audio.setStreamVolume(AudioManager.STREAM_MUSIC, step, 0);
    }
}
