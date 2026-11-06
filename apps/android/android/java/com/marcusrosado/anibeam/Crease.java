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
    /** The activity the listener is on. A relaunch in the same process moves it. */
    private static Activity on;
    private static Object component;
    private static java.util.function.Consumer<Object> listener;

    /**
     * The crease in the window's own pixels, packed in a long: bit 40 set for a crease that
     * runs left to right, bit 41 set while the phone is half open, bits 20 to 39 where the
     * crease starts along the window's width or height, bits 0 to 19 where it ends. -1 when
     * no fold crosses the window, -2 on a phone without the extensions. The first call from
     * an activity starts listening on it.
     */
    public static long at(final Activity activity) {
        synchronized (Crease.class) {
            if (on != activity) {
                final Activity old = on;
                on = activity;
                last = -1;
                activity.runOnUiThread(new Runnable() {
                    @Override
                    public void run() {
                        listen(old, activity);
                    }
                });
            }
        }
        return last;
    }

    /** The extensions' public types, so a vendor's hidden implementation class is never called directly. */
    private static Method method(String type, Object target, String name, Class<?>... params) throws Exception {
        try {
            return Class.forName(type).getMethod(name, params);
        } catch (ClassNotFoundException e) {
            return target.getClass().getMethod(name, params);
        }
    }

    private static void listen(Activity old, Activity activity) {
        try {
            Class<?> provider = Class.forName("androidx.window.extensions.WindowExtensionsProvider");
            if (component == null) {
                Object extensions = provider.getMethod("getWindowExtensions").invoke(null);
                if (extensions == null) { last = -2; return; }
                component = method("androidx.window.extensions.WindowExtensions", extensions,
                        "getWindowLayoutComponent").invoke(extensions);
                if (component == null) { last = -2; return; }
            }
            String layout = "androidx.window.extensions.layout.WindowLayoutComponent";
            if (listener != null && old != null) {
                // The activity that went away keeps no listener, and holds no reference here.
                method(layout, component, "removeWindowLayoutInfoListener",
                        java.util.function.Consumer.class).invoke(component, listener);
            }
            listener = new java.util.function.Consumer<Object>() {
                @Override
                public void accept(Object info) {
                    last = read(info);
                }
            };
            // Vendor API level 1's listener, which every later level still takes.
            method(layout, component, "addWindowLayoutInfoListener",
                    Activity.class, java.util.function.Consumer.class).invoke(component, activity, listener);
        } catch (Throwable e) {
            // No extensions on this phone: it never says, and the layout keeps its guess.
            last = -2;
        }
    }

    /** The first fold in a WindowLayoutInfo, packed as `at` says. */
    static long read(Object info) {
        try {
            List<?> features = (List<?>) method("androidx.window.extensions.layout.WindowLayoutInfo",
                    info, "getDisplayFeatures").invoke(info);
            if (features == null) return -1;
            String folding = "androidx.window.extensions.layout.FoldingFeature";
            for (Object f : features) {
                if (!f.getClass().getName().endsWith("FoldingFeature")) continue;
                Rect r = (Rect) method(folding, f, "getBounds").invoke(f);
                int state = (Integer) method(folding, f, "getState").invoke(f);
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
