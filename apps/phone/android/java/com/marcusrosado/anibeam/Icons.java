package com.marcusrosado.anibeam;

import android.app.Activity;
import android.content.ComponentName;
import android.content.pm.PackageManager;

/**
 * The launcher icon. Each icon is an activity-alias in the manifest with its own icon, and
 * exactly one of them is enabled; the launcher shows that one. Default is enabled by the
 * manifest, the rest by a pick here.
 */
public final class Icons {
    private Icons() {}

    /** In the order More lists them: Default, Dark, Line art. */
    private static final String[] ALIASES = {"IconDefault", "IconDark", "IconLine"};

    private static ComponentName alias(Activity activity, int i) {
        return new ComponentName(activity.getPackageName(), "com.marcusrosado.anibeam." + ALIASES[i]);
    }

    private static boolean enabled(PackageManager pm, ComponentName name, boolean byDefault) {
        int state = pm.getComponentEnabledSetting(name);
        if (state == PackageManager.COMPONENT_ENABLED_STATE_DEFAULT) return byDefault;
        return state == PackageManager.COMPONENT_ENABLED_STATE_ENABLED;
    }

    /** The icon the launcher shows now, as its index in ALIASES. */
    public static int current(Activity activity) {
        PackageManager pm = activity.getPackageManager();
        for (int i = 1; i < ALIASES.length; i++) {
            if (enabled(pm, alias(activity, i), false)) return i;
        }
        return 0;
    }

    /**
     * Shows icon `which` and hides the rest. The new one goes on before the old ones go off,
     * so the app always has a launcher entry. DONT_KILL_APP keeps this activity running.
     */
    public static void pick(Activity activity, int which) {
        if (which < 0 || which >= ALIASES.length) return;
        PackageManager pm = activity.getPackageManager();
        pm.setComponentEnabledSetting(alias(activity, which),
                PackageManager.COMPONENT_ENABLED_STATE_ENABLED, PackageManager.DONT_KILL_APP);
        for (int i = 0; i < ALIASES.length; i++) {
            if (i == which) continue;
            pm.setComponentEnabledSetting(alias(activity, i),
                    PackageManager.COMPONENT_ENABLED_STATE_DISABLED, PackageManager.DONT_KILL_APP);
        }
    }

    /**
     * The line art icon's two colours as the launcher will draw them, for More's preview:
     * the ground in the high 32 bits, the ink in the low, both ARGB. On Android 12 and later
     * these are the phone's Material You colours.
     */
    @SuppressWarnings("deprecation")
    public static long lineColours(Activity activity) {
        android.content.res.Resources res = activity.getResources();
        String pkg = activity.getPackageName();
        int ground = res.getColor(res.getIdentifier("line_ground", "color", pkg));
        int ink = res.getColor(res.getIdentifier("line_ink", "color", pkg));
        return ((long) ground << 32) | (ink & 0xffffffffL);
    }
}
