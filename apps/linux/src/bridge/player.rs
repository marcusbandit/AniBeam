//! Player: the shell-owned player state QML reads, player.toml behind it, and the pure
//! helpers the player page calls into Rust for.

use core::pin::Pin;

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
        #[qproperty(f64, volume)]
        #[qproperty(bool, mute)]
        #[qproperty(bool, use_my_mpv_conf)]
        #[qproperty(QStringList, config_layers)]
        #[qproperty(QJsonArray, owned_options)]
        #[qproperty(QJsonArray, preview_options)]
        type Player = super::PlayerRust;

        /// The three writes that reach player.toml. The property setters cxx-qt generates,
        /// `setVolume`, `setMute` and `setUseMyMpvConf`, change the value and nothing else;
        /// anything that should survive the next launch goes through these.
        #[qinvokable]
        fn save_volume(self: Pin<&mut Self>, volume: f64);
        #[qinvokable]
        fn save_mute(self: Pin<&mut Self>, mute: bool);
        #[qinvokable]
        fn save_use_my_mpv_conf(self: Pin<&mut Self>, on: bool);
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
    fn persist(&self) {
        let s = PlayerSettings {
            volume: *self.volume(),
            mute: *self.mute(),
            use_my_mpv_conf: *self.use_my_mpv_conf(),
        };
        if let Err(e) = player_config::save(&crate::runtime::paths().player_toml(), &s) {
            eprintln!("anibeam: player.toml: {e}");
        }
    }
    pub fn save_volume(mut self: Pin<&mut Self>, volume: f64) {
        self.as_mut().set_volume(volume.clamp(0.0, 100.0));
        self.persist();
    }
    pub fn save_mute(mut self: Pin<&mut Self>, mute: bool) {
        self.as_mut().set_mute(mute);
        self.persist();
    }
    pub fn save_use_my_mpv_conf(mut self: Pin<&mut Self>, on: bool) {
        self.as_mut().set_use_my_mpv_conf(on);
        // The layer list follows the toggle at once, so the next session includes the
        // user's file without waiting for a relaunch.
        let l = layers(on);
        self.as_mut().set_config_layers(l);
        self.persist();
    }
}
