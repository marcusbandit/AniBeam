package com.marcusrosado.anibeam;

import android.app.Activity;
import android.graphics.Rect;

import java.lang.reflect.Method;
import java.util.List;

/**
 * Where the crease runs, from the platform's own FoldingFeature rather than a guess from the
 * rotation. The feature comes from the window extensions library the phone ships
 * (androidx.window.extensions, declared as an optional uses-library in the manifest), read
 * by reflection, since the app is built without Gradle and without androidx. A phone without
 * the library answers -2, one that reports no fold -1, and the layout keeps its guess.
 */
public final class Crease {
    private Crease() {}

    /** The last answer, packed as `at` says; -1 while nothing reported a fold. */
    private static volatile long last = -1;
    private static boolean listening;

    /**
     * The crease in the window's own pixels, packed in a long: bit 40 set for a crease that
     * runs left to right, bit 41 set while the phone is half open, bits 20 to 39 where the
     * crease starts along the window's width or height, bits 0 to 19 where it ends. -1 when
     * no fold crosses the window, -2 on a phone without the extensions. The first call
     * starts listening.
     */
    public static long at(final Activity activity) {
        synchronized (Crease.class) {
            if (!listening) {
                listening = true;
                activity.runOnUiThread(new Runnable() {
                    @Override
                    public void run() {
                        listen(activity);
                    }
                });
            }
        }
        return last;
    }

    private static void listen(Activity activity) {
        try {
            Class<?> provider = Class.forName("androidx.window.extensions.WindowExtensionsProvider");
            Object extensions = provider.getMethod("getWindowExtensions").invoke(null);
            if (extensions == null) { last = -2; return; }
            Object layout = extensions.getClass().getMethod("getWindowLayoutComponent").invoke(extensions);
            if (layout == null) { last = -2; return; }
            // Vendor API level 1's listener, which every later level still takes.
            Method add = layout.getClass().getMethod("addWindowLayoutInfoListener",
                    Activity.class, java.util.function.Consumer.class);
            add.invoke(layout, activity, new java.util.function.Consumer<Object>() {
                @Override
                public void accept(Object info) {
                    last = read(info);
                }
            });
        } catch (Throwable e) {
            // No extensions on this phone: it never says, and the layout keeps its guess.
            last = -2;
        }
    }

    /** The first fold in a WindowLayoutInfo, packed as `at` says. */
    static long read(Object info) {
        try {
            List<?> features = (List<?>) info.getClass().getMethod("getDisplayFeatures").invoke(info);
            if (features == null) return -1;
            for (Object f : features) {
                if (!f.getClass().getName().endsWith("FoldingFeature")) continue;
                Rect r = (Rect) f.getClass().getMethod("getBounds").invoke(f);
                int state = (Integer) f.getClass().getMethod("getState").invoke(f);
                // A fold with no width is a line; it runs the way it is longer.
                boolean across = r.width() > r.height();
                long start = across ? r.top : r.left;
                long end = across ? r.bottom : r.right;
                long packed = (start & 0xFFFFF) << 20 | (end & 0xFFFFF);
                if (across) packed |= 1L << 40;
                // FoldingFeature.STATE_HALF_OPENED is 2.
                if (state == 2) packed |= 1L << 41;
                return packed;
            }
        } catch (Throwable e) {
            return -1;
        }
        return -1;
    }
}
