//! Player: the shell-owned player state QML reads, player.toml behind it, and the pure
//! helpers the player page calls into Rust for.

use core::pin::Pin;

use anibeam_core::{SubtitleDefaults, TrackChoice};
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QJsonArray, QJsonObject, QJsonValue, QString, QStringList};
use serde_json::Value;

use crate::player_config::{self, PlayerSettings};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qjsonarray.h");
        type QJsonArray = cxx_qt_lib::QJsonArray;
        include!("cxx-qt-lib/qjsonobject.h");
        type QJsonObject = cxx_qt_lib::QJsonObject;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        // The three settings read only, so the one way to change them is the invokable
        // beside them, which writes player.toml as well. A generated WRITE would have
        // taken the setter's name and let QML change the value without persisting it.
        #[qproperty(f64, volume, READ = volume, NOTIFY = volume_changed)]
        #[qproperty(bool, mute, READ = mute, NOTIFY = mute_changed)]
        #[qproperty(bool, use_my_mpv_conf, READ = use_my_mpv_conf, NOTIFY = use_my_mpv_conf_changed)]
        #[qproperty(QStringList, config_layers, READ = config_layers, NOTIFY = config_layers_changed)]
        #[qproperty(QJsonArray, owned_options, READ = owned_options, CONSTANT)]
        #[qproperty(QJsonArray, preview_options, READ = preview_options, CONSTANT)]
        type Player = super::PlayerRust;

        // Read through these rather than a generated `getX`, so QML sees one name per
        // property and the setters below keep theirs.
        fn volume(self: &Player) -> f64;
        fn mute(self: &Player) -> bool;
        fn use_my_mpv_conf(self: &Player) -> bool;
        fn config_layers(self: &Player) -> QStringList;
        fn owned_options(self: &Player) -> QJsonArray;
        fn preview_options(self: &Player) -> QJsonArray;

        /// The three writes. Each sets the value, reports it and saves player.toml.
        #[qinvokable]
        fn set_volume(self: Pin<&mut Self>, volume: f64);
        #[qinvokable]
        fn set_mute(self: Pin<&mut Self>, mute: bool);
        #[qinvokable]
        fn set_use_my_mpv_conf(self: Pin<&mut Self>, on: bool);

        /// The pure helpers the player page calls into Rust for. None of them touches the
        /// core or this object's state, so they take `&Player` and are safe to call from a
        /// binding. Each is total, and `guard` is the barrier for the day one is not.
        #[qinvokable]
        fn pick_tracks(
            self: &Player,
            track_list: &QJsonArray,
            track_choice: &QJsonObject,
            defaults: &QJsonObject,
        ) -> QJsonObject;
        #[qinvokable]
        fn track_label(self: &Player, track: &QJsonObject) -> QString;
        #[qinvokable]
        fn track_ref(self: &Player, track: &QJsonObject) -> QJsonObject;
        #[qinvokable]
        fn subtitle_options(self: &Player, defaults: &QJsonObject) -> QJsonArray;

        #[qsignal]
        fn volume_changed(self: Pin<&mut Player>);
        #[qsignal]
        fn mute_changed(self: Pin<&mut Player>);
        #[qsignal]
        fn use_my_mpv_conf_changed(self: Pin<&mut Player>);
        #[qsignal]
        fn config_layers_changed(self: Pin<&mut Player>);
    }
}

pub struct PlayerRust {
    volume: f64,
    mute: bool,
    use_my_mpv_conf: bool,
    config_layers: QStringList,
    owned_options: QJsonArray,
    preview_options: QJsonArray,
}

/// `[[name, value], ...]`: an array of pairs rather than an object, because the order the
/// options are set in is the whole point of the list.
fn pairs(list: impl IntoIterator<Item = (String, String)>) -> QJsonArray {
    let mut out = QJsonArray::default();
    for (k, v) in list {
        let mut pair = QJsonArray::default();
        pair.append(&QJsonValue::from(&QString::from(&k)));
        pair.append(&QJsonValue::from(&QString::from(&v)));
        out.append(&QJsonValue::from(&pair));
    }
    out
}

fn layers(use_my_conf: bool) -> QStringList {
    let paths = crate::runtime::paths();
    QStringList::from_iter(
        player_config::config_layers(paths, use_my_conf)
            .iter()
            .map(|p| QString::from(&p.to_string_lossy().into_owned())),
    )
}

impl Default for PlayerRust {
    fn default() -> Self {
        let s = player_config::load(&crate::runtime::paths().player_toml());
        PlayerRust {
            volume: s.volume,
            mute: s.mute,
            use_my_mpv_conf: s.use_my_mpv_conf,
            config_layers: layers(s.use_my_mpv_conf),
            owned_options: pairs(
                player_config::owned_options()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v)),
            ),
            preview_options: pairs(
                player_config::preview_options()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string())),
            ),
        }
    }
}

impl qobject::Player {
    pub fn volume(&self) -> f64 {
        self.volume
    }
    pub fn mute(&self) -> bool {
        self.mute
    }
    pub fn use_my_mpv_conf(&self) -> bool {
        self.use_my_mpv_conf
    }
    pub fn config_layers(&self) -> QStringList {
        self.config_layers.clone()
    }
    pub fn owned_options(&self) -> QJsonArray {
        self.owned_options.clone()
    }
    pub fn preview_options(&self) -> QJsonArray {
        self.preview_options.clone()
    }

    fn persist(&self) {
        let s = PlayerSettings {
            volume: self.volume,
            mute: self.mute,
            use_my_mpv_conf: self.use_my_mpv_conf,
        };
        if let Err(e) = player_config::save(&crate::runtime::paths().player_toml(), &s) {
            eprintln!("anibeam: player.toml: {e}");
        }
    }

    pub fn set_volume(mut self: Pin<&mut Self>, volume: f64) {
        let v = volume.clamp(0.0, 100.0);
        if self.volume == v {
            return;
        }
        self.as_mut().rust_mut().volume = v;
        self.as_mut().volume_changed();
        self.persist();
    }
    pub fn set_mute(mut self: Pin<&mut Self>, mute: bool) {
        if self.mute == mute {
            return;
        }
        self.as_mut().rust_mut().mute = mute;
        self.as_mut().mute_changed();
        self.persist();
    }
    pub fn set_use_my_mpv_conf(mut self: Pin<&mut Self>, on: bool) {
        if self.use_my_mpv_conf == on {
            return;
        }
        self.as_mut().rust_mut().use_my_mpv_conf = on;
        // The layer list follows the toggle at once, so the next session includes the
        // user's file without waiting for a relaunch.
        let l = layers(on);
        self.as_mut().rust_mut().config_layers = l;
        self.as_mut().use_my_mpv_conf_changed();
        self.as_mut().config_layers_changed();
        self.persist();
    }

    /// A panic crossing the FFI aborts the process, so it stops here instead. The four
    /// helpers below are total, which makes this belt and braces rather than a net.
    fn guard<T: Default>(what: &str, f: impl FnOnce() -> T) -> T {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| {
            eprintln!("anibeam: player: {what} panicked");
            T::default()
        })
    }

    /// mpv's `track-list` and the series' stored choice in; the `aid` and `sid` to set out,
    /// `-1` for off, because QML has no null number.
    pub fn pick_tracks(
        &self,
        track_list: &QJsonArray,
        track_choice: &QJsonObject,
        defaults: &QJsonObject,
    ) -> QJsonObject {
        Self::guard("pickTracks", || {
            let list = crate::tracks::parse(&Value::Array(
                track_list
                    .iter()
                    .map(|v| crate::json::from_qjson(&v))
                    .collect(),
            ));
            let choice: TrackChoice =
                serde_json::from_value(crate::json::from_qjson_object(track_choice))
                    .unwrap_or_default();
            let d: SubtitleDefaults =
                serde_json::from_value(crate::json::from_qjson_object(defaults))
                    .unwrap_or_default();
            let p = crate::tracks::pick(&list, &choice, &d);
            crate::json::to_qjson_object(&serde_json::json!({
                "aid": p.aid.unwrap_or(-1),
                "sid": p.sid.unwrap_or(-1),
            }))
        })
    }

    /// One entry of `track-list` as the line a picker draws.
    pub fn track_label(&self, track: &QJsonObject) -> QString {
        Self::guard("trackLabel", || {
            let list =
                crate::tracks::parse(&Value::Array(vec![crate::json::from_qjson_object(track)]));
            QString::from(&list.first().map(crate::tracks::label).unwrap_or_default())
        })
    }

    /// One entry of `track-list` as the `TrackRef` the core stores.
    pub fn track_ref(&self, track: &QJsonObject) -> QJsonObject {
        Self::guard("trackRef", || {
            let list =
                crate::tracks::parse(&Value::Array(vec![crate::json::from_qjson_object(track)]));
            let r = list.first().map(crate::tracks::track_ref);
            crate::json::to_qjson_object(&serde_json::to_value(r).unwrap_or(Value::Null))
        })
    }

    /// The settings' subtitle defaults as the mpv options that carry them.
    pub fn subtitle_options(&self, defaults: &QJsonObject) -> QJsonArray {
        Self::guard("subtitleOptions", || {
            let d: SubtitleDefaults =
                serde_json::from_value(crate::json::from_qjson_object(defaults))
                    .unwrap_or_default();
            pairs(
                player_config::subtitle_options(&d)
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v)),
            )
        })
    }
}
