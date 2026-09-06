//! Player: the shell-owned player state QML reads, player.toml behind it, and the pure
//! helpers the player page calls into Rust for.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QJsonArray, QJsonValue, QString, QStringList};

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
}
