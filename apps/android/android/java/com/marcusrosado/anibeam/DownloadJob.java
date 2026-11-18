package com.marcusrosado.anibeam;

import android.app.job.JobInfo;
import android.app.job.JobParameters;
import android.app.job.JobScheduler;
import android.app.job.JobService;
import android.content.ComponentName;
import android.content.Context;
import android.os.Handler;
import android.os.Looper;
import android.os.PersistableBundle;
import android.os.SystemClock;
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
    /** The extra that numbers each run the app schedules. */
    private static final String RUN = "run";
    private static final int NONE = 0;
    /** The job while it runs, and its parameters: null while it waits or after it ended. */
    private static DownloadJob self;
    private static JobParameters running;
    /**
     * The run the app wants, by its number, or NONE once the app ended the work. A run
     * Android starts after the app ended it, as a cancel and a start cross, ends at once.
     */
    private static int wanted = NONE;
    /** The last run's number; from the clock, so a run left by an earlier process differs. */
    private static int last = (int) SystemClock.elapsedRealtime();
    private static final Handler MAIN = new Handler(Looper.getMainLooper());

    private static int runOf(PersistableBundle extras) {
        return extras == null ? NONE : extras.getInt(RUN, NONE);
    }

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
        JobInfo pending = js.getPendingJob(ID);
        if (pending != null) {
            // Android's, or one an earlier process scheduled: it is the run wanted.
            wanted = runOf(pending.getExtras());
            if (!(fresh && TorrentService.onScreen())) {
                TorrentService.set(TorrentService.WAITING);
                return true;
            }
        }
        return schedule(context, js);
    }

    /**
     * Schedules a run that starts now, in place of one Android holds back. False when
     * Android refused. Called with TorrentService.LOCK held.
     */
    private static boolean schedule(Context context, JobScheduler js) {
        int run = ++last;
        if (run == NONE) run = ++last;
        PersistableBundle extras = new PersistableBundle();
        extras.putInt(RUN, run);
        JobInfo info = new JobInfo.Builder(ID, new ComponentName(context, DownloadJob.class))
                .setUserInitiated(true)
                .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY)
                .setExtras(extras)
                .build();
        try {
            if (js.schedule(info) != JobScheduler.RESULT_SUCCESS) {
                TorrentService.refused("the download job was not scheduled", null);
                return false;
            }
        } catch (RuntimeException e) {
            TorrentService.refused("the download job was not scheduled", e);
            return false;
        }
        TorrentService.started();
        wanted = run;
        TorrentService.set(TorrentService.WAITING);
        return true;
    }

    /** Ends the job, running or waiting. Called with TorrentService.LOCK held. */
    static void cancel(Context context) {
        if (self != null && running != null) {
            self.jobFinished(running, false);
        }
        self = null;
        running = null;
        wanted = NONE;
        JobScheduler js = context.getSystemService(JobScheduler.class);
        if (js != null) js.cancel(ID);
    }

    /** Puts the notification up, or changes it in place. Called with TorrentService.LOCK held. */
    private void show() {
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
            // The app ended the work while Android was starting this run.
            if (runOf(params.getExtras()) != wanted) return false;
            self = this;
            running = params;
            // Once a run, not on every change of the words: words() renames them in place.
            TorrentService.channels(this);
            show();
            TorrentService.set(TorrentService.RUNNING);
        }
        TorrentService.cancelPaused(this);
        return true;
    }

    @Override
    public boolean onStopJob(JobParameters params) {
        int reason = params.getStopReason();
        final int run = runOf(params.getExtras());
        synchronized (TorrentService.LOCK) {
            // Android hands this call parameters of its own, never the object the run started
            // with, so the run is told by its number.
            boolean current = running != null && runOf(running.getExtras()) == run;
            if (!current) return false;
            self = null;
            running = null;
            // The app's own cancel already said where the work stands; one that left this
            // run recorded (a schedule that replaced it) ends it here.
            if (reason == JobParameters.STOP_REASON_CANCELLED_BY_APP) {
                if (TorrentService.runningNow()) TorrentService.set(TorrentService.IDLE);
                return false;
            }
            Log.i(TAG, "Android stopped the download job, reason " + reason);
            // Stopped from the phone's list of running apps: the person's call, so it stays
            // stopped until they open the app again.
            if (reason == JobParameters.STOP_REASON_USER) {
                TorrentService.set(TorrentService.STOPPED_BY_USER);
                return false;
            }
            // With the app open nothing pauses: a fresh run replaces the rerun as soon as
            // this stop is through, which starts at once where a rerun backs off first.
            if (TorrentService.onScreen()) {
                TorrentService.set(TorrentService.WAITING);
                final Context context = getApplicationContext();
                MAIN.post(new Runnable() {
                    @Override
                    public void run() {
                        replace(context, run);
                    }
                });
                return true;
            }
            // Anything else runs again when Android lets it.
            TorrentService.set(TorrentService.STOPPED);
            return true;
        }
    }

    /**
     * After Android stopped `run` with the app open: schedules a run that starts now, if the
     * app may still. If it has gone from the screen, or Android refused, the rerun Android
     * holds is what is left, and the downloads pause until it runs or the app is opened.
     */
    private static void replace(Context context, int run) {
        synchronized (TorrentService.LOCK) {
            // Moved on since: the app ended the work, or scheduled or started a run itself.
            if (wanted != run || self != null || !TorrentService.waitingNow()) return;
            JobScheduler js = context.getSystemService(JobScheduler.class);
            if (js != null && TorrentService.onScreen() && schedule(context, js)) return;
            TorrentService.set(TorrentService.STOPPED);
        }
    }
}
