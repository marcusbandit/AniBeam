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

    /** Runs one command line and answers with what it printed. */
    static native String run(String line);

    public static synchronized void start(Context context) {
        if (receiver != null) return;
        HandlerThread thread = new HandlerThread("anibeam-control");
        thread.start();
        Handler handler = new Handler(thread.getLooper());
        receiver = new Control();
        IntentFilter filter = new IntentFilter(ACTION);
        // From Android 13 a receiver registered at run time says whether other processes may
        // reach it; the shell is another process.
        int flags = Build.VERSION.SDK_INT >= 33 ? Context.RECEIVER_EXPORTED : 0;
        context.getApplicationContext().registerReceiver(receiver, filter, PERMISSION, handler, flags);
    }

    @Override
    public void onReceive(Context context, Intent intent) {
        String line = intent.getStringExtra("cmd");
        String out;
        try {
            out = run(line == null ? "" : line);
        } catch (Throwable t) {
            out = "error: " + t;
        }
        if (out == null) out = "error: no answer";
        boolean failed = out.startsWith("error: ") || out.contains("\nerror: ");
        setResultCode(failed ? 1 : 0);
        setResultData(out);
    }
}
