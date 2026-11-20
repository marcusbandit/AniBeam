package com.marcusrosado.anibeam;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.os.Build;
import android.os.Handler;
import android.os.HandlerThread;

/**
 * Commands from adb, so an agent can drive the app by name (#1207):
 * {@code adb shell am broadcast -a com.marcusrosado.anibeam.CONTROL -p com.marcusrosado.anibeam
 * --es cmd 'go downloads'}. The answer comes back as the broadcast's result data, which
 * {@code am broadcast} prints, and the result code is 1 when the command failed.
 *
 * <p>Only the sideload build registers it, at run time from the native side, which also
 * gives {@link #run} its body. A sender must hold {@code android.permission.DUMP}, which
 * only the shell user (adb) and the system have: its protection level is
 * signature|privileged|development, so no installed app can be granted it without adb. The
 * receiver runs on a thread of its own, so a command waiting on the UI loop never blocks
 * Android's main thread.
 */
public final class Control extends BroadcastReceiver {
    static final String ACTION = "com.marcusrosado.anibeam.CONTROL";
    static final String PERMISSION = "android.permission.DUMP";

    private static Control receiver;

    /**
     * The most a broadcast may arrive late, by the phone's clock, and still run: past it,
     * ctl.sh (which waits 60 s) could have given up and a retry would run the command twice.
     */
    static final long LATE_MS = 10_000;

    /** Runs one command line, given how late it arrived: the result code, 0 or 1, then what it printed. */
    static native String run(String line, long lateMs);

    /** Registers the receiver once; true when it is registered. */
    public static synchronized boolean start(Context context) {
        if (receiver != null) return true;
        HandlerThread thread = new HandlerThread("anibeam-control");
        thread.start();
        Handler handler = new Handler(thread.getLooper());
        Control r = new Control();
        IntentFilter filter = new IntentFilter(ACTION);
        // From Android 13 a receiver registered at run time says whether other processes may
        // reach it; the shell is another process.
        int flags = Build.VERSION.SDK_INT >= 33 ? Context.RECEIVER_EXPORTED : 0;
        try {
            context.getApplicationContext().registerReceiver(r, filter, PERMISSION, handler, flags);
        } catch (RuntimeException e) {
            // Not registered, so a later start (the activity made again) tries once more.
            thread.quitSafely();
            android.util.Log.w("AnibeamCore", "control: " + e);
            return false;
        }
        receiver = r;
        return true;
    }

    @Override
    public void onReceive(Context context, Intent intent) {
        String line = intent.getStringExtra("cmd");
        // ctl.sh stamps each broadcast with the phone's own clock as it sends it.
        long sent = intent.getLongExtra("sent", 0);
        long late = sent > 0 ? Math.max(0, System.currentTimeMillis() - sent * 1000) : 0;
        String out;
        if (late > LATE_MS) {
            out = "1error: this command reached the app " + (late / 1000)
                    + " s after it was sent, held back by Android while the app was in the"
                    + " background; it was not run, so sending it again runs it once";
        } else {
            try {
                out = run(line == null ? "" : line, late);
            } catch (Throwable t) {
                out = "1error: " + t;
            }
        }
        if (out == null || out.isEmpty()) out = "1error: no answer";
        setResultCode(out.charAt(0) == '0' ? 0 : 1);
        setResultData(out.substring(1));
    }
}
