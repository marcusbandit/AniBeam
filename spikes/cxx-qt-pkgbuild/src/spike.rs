//! The bridge: one Rust QObject the QML engine owns as a singleton.
//! A call is an invokable that returns at once; the job it starts runs on tokio and
//! reports back through a signal, crossing to the Qt thread with CxxQtThread::queue.

use core::pin::Pin;
use cxx_qt::Threading;
use cxx_qt_lib::QString;
use std::time::Duration;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "C++" {
        include!("helpers.h");
        /// QQuickWindow::setGraphicsApi(OpenGL); must run before QGuiApplication exists.
        fn use_opengl_scene_graph();
        /// QGuiApplication::setDesktopFileName so the compositor pairs the window with the .desktop entry.
        fn set_desktop_file_name(name: &QString);
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(i32, counter)]
        #[qproperty(QString, status)]
        type Spike = super::SpikeRust;

        /// Emitted once per job step, from the Qt thread, carrying the id of the tokio
        /// worker thread that produced it.
        #[qsignal]
        fn tick(self: Pin<&mut Self>, n: i32, worker_thread: QString);

        /// Starts a job of `steps` steps and returns at once.
        #[qinvokable]
        fn start_job(self: Pin<&mut Self>, steps: i32);
    }

    impl cxx_qt::Threading for Spike {}
}

#[derive(Default)]
pub struct SpikeRust {
    counter: i32,
    status: QString,
}

impl qobject::Spike {
    pub fn start_job(mut self: Pin<&mut Self>, steps: i32) {
        let qt_thread_id = format!("{:?}", std::thread::current().id());
        self.as_mut()
            .set_status(QString::from(&format!("job of {steps} started on {qt_thread_id}")));
        let qt = self.qt_thread();
        crate::runtime().spawn(async move {
            for n in 1..=steps {
                tokio::time::sleep(Duration::from_millis(300)).await;
                let worker = format!("{:?}", std::thread::current().id());
                // Runs on the Qt thread; the Rust object is only touched there.
                let posted = qt.queue(move |mut spike: Pin<&mut qobject::Spike>| {
                    spike.as_mut().set_counter(n);
                    spike.as_mut().tick(n, QString::from(&worker));
                });
                if posted.is_err() {
                    return; // the QObject is gone, the job stops
                }
            }
            qt.queue(move |mut spike: Pin<&mut qobject::Spike>| {
                spike.as_mut().set_status(QString::from(&format!("job of {steps} finished")));
            })
            .ok();
        });
    }
}
