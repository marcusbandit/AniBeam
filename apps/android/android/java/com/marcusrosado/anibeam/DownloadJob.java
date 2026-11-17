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
 * service by Android version. Android may still stop the job (its own runtime cap, the
 * battery, the network gone). The job then says so through `TorrentService.stopped`, asks
 * to run again, and the app pauses the downloads until it does or the app is opened.
 */
public class DownloadJob extends JobService {
    private static final String TAG = "Anibeam";
    static final int ID = 1;
    private static final Object LOCK = new Object();
    /** The job while it runs, and its parameters: null while it waits or after it ended. */
    private static DownloadJob self;
    private static JobParameters running;

    /**
     * Makes sure the job is running or waiting to, and that a running one shows the newest
     * words. False when Android refused to schedule it: from the background, for one. The
     * caller tries again on its next look.
     */
    static boolean ensure(Context context) {
        synchronized (LOCK) {
            if (self != null && running != null) {
                self.show();
                return true;
            }
            JobScheduler js = context.getSystemService(JobScheduler.class);
            if (js == null) return false;
            if (js.getPendingJob(ID) != null) return true;
            JobInfo info = new JobInfo.Builder(ID, new ComponentName(context, DownloadJob.class))
                    .setUserInitiated(true)
                    .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY)
                    .build();
            try {
                return js.schedule(info) == JobScheduler.RESULT_SUCCESS;
            } catch (RuntimeException e) {
                Log.w(TAG, "the download job was not scheduled", e);
                return false;
            }
        }
    }

    /** Ends the job, running or waiting: nothing downloads any more. */
    static void cancel(Context context) {
        synchronized (LOCK) {
            if (self != null && running != null) {
                self.jobFinished(running, false);
            }
            self = null;
            running = null;
            JobScheduler js = context.getSystemService(JobScheduler.class);
            if (js != null) js.cancel(ID);
        }
    }

    /** Puts the notification up, or changes it in place. Called with LOCK held. */
    private void show() {
        TorrentService.channels(this);
        setNotification(running, TorrentService.NOTIFICATION,
                TorrentService.ongoing(this, TorrentService.shownTitle, TorrentService.shownText,
                        TorrentService.shownProgress),
                JobService.JOB_END_NOTIFICATION_POLICY_REMOVE);
    }

    @Override
    public boolean onStartJob(JobParameters params) {
        synchronized (LOCK) {
            // A process Android started only for this job runs no downloads: the app's
            // native side never started in it. Nothing to hold up, so the job ends.
            if (!TorrentService.live) return false;
            self = this;
            running = params;
            show();
        }
        TorrentService.goOn(this);
        return true;
    }

    @Override
    public boolean onStopJob(JobParameters params) {
        synchronized (LOCK) {
            if (running == params) {
                self = null;
                running = null;
            }
        }
        int reason = params.getStopReason();
        // The app's own cancel: nothing downloads any more.
        if (reason == JobParameters.STOP_REASON_CANCELLED_BY_APP) return false;
        Log.i(TAG, "Android stopped the download job, reason " + reason);
        TorrentService.stopped = true;
        // Stopped from the phone's list of running apps: the person's call, so it stays
        // stopped until they open the app. Anything else, it runs again when Android lets it.
        return reason != JobParameters.STOP_REASON_USER;
    }
}
