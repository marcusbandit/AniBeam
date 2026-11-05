package com.marcusrosado.anibeam;

import android.app.Activity;
import android.content.Context;
import android.media.AudioAttributes;
import android.media.AudioFocusRequest;
import android.media.AudioManager;

/**
 * The player's audio focus. The player asks for it as an episode starts playing and gives
 * it back when the page closes; Android lowers the volume itself for a duck, and every
 * other change goes to the native side through {@link #changed}, which pauses for a call
 * and plays again when it ends. The native side registers {@code changed} before the
 * first {@link #take}.
 */
public final class Focus {
    private Focus() {}

    private static AudioFocusRequest request;

    /** Called on the UI thread with AudioManager's AUDIOFOCUS_ number. */
    static native void changed(int change);

    /**
     * Asks for the focus a movie wants: 1 when granted, 0 when refused, as during a call.
     */
    public static synchronized int take(final Activity activity) {
        AudioManager audio = (AudioManager) activity.getSystemService(Context.AUDIO_SERVICE);
        if (audio == null) return 1;
        if (request == null) {
            AudioAttributes attributes = new AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_MEDIA)
                    .setContentType(AudioAttributes.CONTENT_TYPE_MOVIE)
                    .build();
            request = new AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
                    .setAudioAttributes(attributes)
                    .setOnAudioFocusChangeListener(new AudioManager.OnAudioFocusChangeListener() {
                        @Override
                        public void onAudioFocusChange(int change) {
                            changed(change);
                        }
                    })
                    .build();
        }
        return audio.requestAudioFocus(request) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED ? 1 : 0;
    }

    /** Gives the focus back, so the music the episode paused can play on. */
    public static synchronized void give(final Activity activity) {
        AudioManager audio = (AudioManager) activity.getSystemService(Context.AUDIO_SERVICE);
        if (audio == null || request == null) return;
        audio.abandonAudioFocusRequest(request);
    }
}
