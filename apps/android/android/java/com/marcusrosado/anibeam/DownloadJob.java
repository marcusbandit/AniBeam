package com.marcusrosado.anibeam;

import android.app.job.JobInfo;
import android.app.job.JobParameters;
import android.app.job.JobScheduler;
import android.app.job.JobService;
import android.content.ComponentName;
import android.content.Context;
import android.util.Log;

/**
 * Keeps downloads going with the screen off on Android 14 and later, as a user-initiated
 * data transfer job: the kind Android gives a transfer the person started, with no daily
 * limit. A dataSync foreground service gets six hours a day from Android 15 on and is then
 * stopped. Like TorrentService it holds no logic: the downloads run in Rust, and the job
 * only holds the notification that keeps the process and its network up.
 *
 * Everything comes through TorrentService's static calls, which pick this job or the
 * service by Android version, and every start and stop moves TorrentService's state, which
 * the app reads. Android may still stop the job (its own runtime cap, the battery, the
 * network gone). The job then asks to run again, and the app pauses the downloads until it
 * does or the app is opened.
 */
public class DownloadJob extends JobService {
    private static final String TAG = "Anibeam";
    static final int ID = 1;
    /** The job while it runs, and its parameters: null while it waits or after it ended. */
    private static DownloadJob self;
    private static JobParameters running;

    /**
     * Makes sure the job is running or waiting to, and that a running one shows the newest
     * words. `fresh` replaces a job Android is holding back after a stop, which may wait out
     * a long backoff, with one that runs now; only the open app may schedule one. False when
     * Android refused to schedule it: from the background, for one. The caller tries again
     * on its next look. Called with TorrentService.LOCK held.
     */
    static boolean ensure(Context context, boolean fresh) {
        if (self != null && running != null) {
            self.show();
            return true;
        }
        // Android stopped it a moment ago: the app reads that on its next look, and the
        // stop must not read as waiting.
        if (TorrentService.stoppedNow()) return !fresh;
        JobScheduler js = context.getSystemService(JobScheduler.class);
        if (js == null) return false;
        if (js.getPendingJob(ID) != null && !(fresh && TorrentService.onScreen())) {
            TorrentService.set(TorrentService.WAITING);
            return true;
        }
        JobInfo info = new JobInfo.Builder(ID, new ComponentName(context, DownloadJob.class))
                .setUserInitiated(true)
                .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY)
                .build();
        try {
            if (js.schedule(info) != JobScheduler.RESULT_SUCCESS) return false;
            TorrentService.set(TorrentService.WAITING);
            return true;
        } catch (RuntimeException e) {
            Log.w(TAG, "the download job was not scheduled", e);
            return false;
        }
    }

    /** Ends the job, running or waiting. Called with TorrentService.LOCK held. */
    static void cancel(Context context) {
        if (self != null && running != null) {
            self.jobFinished(running, false);
        }
        self = null;
        running = null;
        JobScheduler js = context.getSystemService(JobScheduler.class);
        if (js != null) js.cancel(ID);
    }

    /** Puts the notification up, or changes it in place. Called with TorrentService.LOCK held. */
    private void show() {
        TorrentService.channels(this);
        setNotification(running, TorrentService.NOTIFICATION,
                TorrentService.ongoing(this, TorrentService.shownTitle, TorrentService.shownText,
                        TorrentService.shownProgress),
                JobService.JOB_END_NOTIFICATION_POLICY_REMOVE);
    }

    @Override
    public boolean onStartJob(JobParameters params) {
        synchronized (TorrentService.LOCK) {
            // A process Android started only for this job runs no downloads: the app's
            // native side never started in it. Nothing to hold up, so the job ends, and the
            // Paused notification stays true until the app is opened.
            if (!TorrentService.live) return false;
            self = this;
            running = params;
            show();
            TorrentService.set(TorrentService.RUNNING);
        }
        TorrentService.cancelPaused(this);
        return true;
    }

    @Override
    public boolean onStopJob(JobParameters params) {
        int reason = params.getStopReason();
        synchronized (TorrentService.LOCK) {
            // The app's own cancel, which already let go of the job and said where the work
            // stands; a run it scheduled since must stay.
            if (reason == JobParameters.STOP_REASON_CANCELLED_BY_APP) return false;
            // There is one job, and Android hands this call parameters of its own, never the
            // object the run started with, so the run is let go of without comparing them.
            self = null;
            running = null;
            Log.i(TAG, "Android stopped the download job, reason " + reason);
            // Stopped from the phone's list of running apps: the person's call, so it stays
            // stopped until they open the app again.
            if (reason == JobParameters.STOP_REASON_USER) {
                TorrentService.set(TorrentService.STOPPED_BY_USER);
                return false;
            }
            // With the app open nothing pauses: the job ends, and the app schedules a fresh
            // one on its next look, which runs at once where a rerun would back off first.
            if (TorrentService.onScreen()) {
                TorrentService.set(TorrentService.IDLE);
                return false;
            }
            // Anything else runs again when Android lets it.
            TorrentService.set(TorrentService.STOPPED);
            return true;
        }
    }
}
