package com.marcusrosado.anibeam;

import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * Where the backup goes and where a restore comes from: Android's own file picker, on its
 * own screen over the app. The app's activity is a stock NativeActivity that cannot receive
 * a result, so this one asks for it and copies the file between the picked place and a file
 * the app owns. It then leaves one word in a result file, "ok", "cancel" or "error: ...",
 * which the app looks for. Extras: mode (0 save, 1 open), path (the app's file), name
 * (the file name offered when saving), result (the result file).
 */
public class PickActivity extends Activity {
    private static final int CODE = 41;

    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);
        // Turned or recreated: the first launch is still waiting on its picker.
        if (state != null) return;
        Intent in = getIntent();
        boolean save = in.getIntExtra("mode", 0) == 0;
        Intent ask = new Intent(save ? Intent.ACTION_CREATE_DOCUMENT : Intent.ACTION_OPEN_DOCUMENT);
        ask.addCategory(Intent.CATEGORY_OPENABLE);
        ask.setType(save ? "application/json" : "*/*");
        if (save) ask.putExtra(Intent.EXTRA_TITLE, in.getStringExtra("name"));
        try {
            startActivityForResult(ask, CODE);
        } catch (RuntimeException e) {
            done("error: this phone has no file picker");
        }
    }

    @Override
    protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request != CODE) return;
        Uri uri = data == null ? null : data.getData();
        if (result != RESULT_OK || uri == null) {
            done("cancel");
            return;
        }
        boolean save = getIntent().getIntExtra("mode", 0) == 0;
        File file = new File(getIntent().getStringExtra("path"));
        try {
            if (save) {
                try (InputStream from = new FileInputStream(file);
                        OutputStream to = getContentResolver().openOutputStream(uri, "wt")) {
                    copy(from, to);
                }
            } else {
                try (InputStream from = getContentResolver().openInputStream(uri);
                        OutputStream to = new FileOutputStream(file)) {
                    copy(from, to);
                }
            }
            done("ok");
        } catch (IOException | RuntimeException e) {
            done("error: " + e.getMessage());
        }
    }

    private static void copy(InputStream from, OutputStream to) throws IOException {
        if (from == null || to == null) throw new IOException("the file could not be opened");
        byte[] buffer = new byte[65536];
        int n;
        while ((n = from.read(buffer)) > 0) to.write(buffer, 0, n);
        to.flush();
    }

    private void done(String word) {
        String path = getIntent().getStringExtra("result");
        if (path != null) {
            try (OutputStream out = new FileOutputStream(path)) {
                out.write(word.getBytes("UTF-8"));
            } catch (IOException e) {
                // Nothing to tell; the app gives up waiting.
            }
        }
        finish();
    }
}
