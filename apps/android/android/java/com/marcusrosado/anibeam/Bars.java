package com.marcusrosado.anibeam;

import android.app.Activity;
import android.graphics.Color;
import android.graphics.drawable.ColorDrawable;
import android.os.Build;
import android.view.View;
import android.view.WindowInsetsController;

/**
 * The system bars follow the page: over a light ground their icons turn dark so they stay
 * readable, and the window shows the ground colour before the app draws. More's Appearance
 * calls this each time the theme changes.
 */
public final class Bars {
    private Bars() {}

    /** The page's ground as ARGB. The native side calls this from its own thread. */
    public static void ground(final Activity activity, final int argb) {
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                activity.getWindow().setBackgroundDrawable(new ColorDrawable(argb));
                boolean light = Color.red(argb) * 0.2126 + Color.green(argb) * 0.7152
                        + Color.blue(argb) * 0.0722 > 140;
                View decor = activity.getWindow().getDecorView();
                if (Build.VERSION.SDK_INT >= 30) {
                    WindowInsetsController c = decor.getWindowInsetsController();
                    // APPEARANCE_LIGHT_STATUS_BARS | APPEARANCE_LIGHT_NAVIGATION_BARS.
                    int mask = 8 | 16;
                    if (c != null) c.setSystemBarsAppearance(light ? mask : 0, mask);
                } else {
                    // SYSTEM_UI_FLAG_LIGHT_STATUS_BAR | SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR.
                    int flags = 0x2000 | 0x10;
                    int now = decor.getSystemUiVisibility();
                    decor.setSystemUiVisibility(light ? now | flags : now & ~flags);
                }
            }
        });
    }
}
