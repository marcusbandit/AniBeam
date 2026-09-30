//! RecordModel: the one QAbstractListModel every list sits on. Rows are JSON objects keyed
//! by `idKey`; `roles` names the keys a delegate reads (dotted paths reach into nested
//! objects, so a feed card's `series.title` is a role). Every begin/end pair is written
//! out in the same function with no early return between them; cxx-qt ships no guard.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{
    QByteArray, QHash, QHashPair_i32_QByteArray, QJsonArray, QJsonObject, QJsonValue, QList,
    QModelIndex, QString, QStringList, QVariant, QVector,
};

const USER_ROLE: i32 = 256;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        include!("cxx-qt-lib/qjsonobject.h");
        type QJsonObject = cxx_qt_lib::QJsonObject;
        include!("cxx-qt-lib/qjsonarray.h");
        type QJsonArray = cxx_qt_lib::QJsonArray;
    }

    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(QStringList, roles)]
        #[qproperty(QString, id_key)]
        #[qproperty(i32, count)]
        type RecordModel = super::RecordModelRust;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &RecordModel, index: &QModelIndex, role: i32) -> QVariant;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &RecordModel, parent: &QModelIndex) -> i32;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &RecordModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        fn reset(self: Pin<&mut RecordModel>, records: &QJsonArray);
        #[qinvokable]
        fn upsert(self: Pin<&mut RecordModel>, record: &QJsonObject);
        #[qinvokable]
        fn upsert_all(self: Pin<&mut RecordModel>, records: &QJsonArray);
        #[qinvokable]
        fn remove(self: Pin<&mut RecordModel>, id: f64);
        #[qinvokable]
        fn remove_all(self: Pin<&mut RecordModel>, ids: &QJsonArray);
        #[qinvokable]
        fn patch(self: Pin<&mut RecordModel>, id: f64, fields: &QJsonObject);
        #[qinvokable]
        fn at(self: &RecordModel, row: i32) -> QJsonObject;
        #[qinvokable]
        fn index_of(self: &RecordModel, id: f64) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(
            self: Pin<&mut RecordModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[inherit]
        fn index(self: &RecordModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
    }

    extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        unsafe fn begin_insert_rows(
            self: Pin<&mut RecordModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endInsertRows"]
        unsafe fn end_insert_rows(self: Pin<&mut RecordModel>);
        #[inherit]
        #[cxx_name = "beginRemoveRows"]
        unsafe fn begin_remove_rows(
            self: Pin<&mut RecordModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endRemoveRows"]
        unsafe fn end_remove_rows(self: Pin<&mut RecordModel>);
        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut RecordModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut RecordModel>);
    }

    impl cxx_qt::Initialize for RecordModel {}
}

pub struct RecordModelRust {
    roles: QStringList,
    id_key: QString,
    count: i32,
    keys: Vec<String>,
    rows: Vec<(f64, QJsonObject)>,
}

impl Default for RecordModelRust {
    fn default() -> Self {
        RecordModelRust {
            roles: QStringList::default(),
            id_key: QString::from("id"),
            count: 0,
            keys: vec![],
            rows: vec![],
        }
    }
}

/// `a.b.c` into a JSON object.
fn lookup(o: &QJsonObject, path: &str) -> QJsonValue {
    let mut current = QJsonValue::from(o);
    for part in path.split('.') {
        if !current.is_object() {
            return QJsonValue::default();
        }
        current = current.to_object().value(&QString::from(part));
    }
    current
}

fn id_of(o: &QJsonObject, key: &str) -> f64 {
    let v = lookup(o, key);
    if v.is_double() { v.to_double() } else { -1.0 }
}

fn to_variant(v: &QJsonValue) -> QVariant {
    if v.is_bool() {
        QVariant::from(&v.to_bool())
    } else if v.is_double() {
        QVariant::from(&v.to_double())
    } else if v.is_string() {
        QVariant::from(&v.to_string())
    } else if v.is_null() || v.is_undefined() {
        QVariant::default()
    } else {
        QVariant::from(v)
    }
}

impl cxx_qt::Initialize for qobject::RecordModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_roles_changed(|model| model.reload_roles())
            .release();
        self.on_id_key_changed(|model| model.reload_ids()).release();
    }
}

impl qobject::RecordModel {
    /// A role id is a position in `keys`, so a view still holding the old ids would read
    /// the wrong column. Swapping the keys under it is not enough: the whole model resets
    /// and `roleNames` is asked again.
    fn reload_roles(mut self: Pin<&mut Self>) {
        let keys: Vec<String> = QList::<QString>::from(self.as_ref().roles())
            .iter()
            .map(|s| s.to_string())
            .collect();
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().keys = keys;
            self.as_mut().end_reset_model();
        }
    }

    /// The key that identifies a row changed, so every cached id is stale. Recomputed
    /// inside a reset, since `at`, `indexOf` and every later upsert read those ids.
    fn reload_ids(mut self: Pin<&mut Self>) {
        let key = self.as_ref().id_key().to_string();
        let ids: Vec<f64> = self
            .as_ref()
            .rows
            .iter()
            .map(|(_, o)| id_of(o, &key))
            .collect();
        unsafe {
            self.as_mut().begin_reset_model();
            {
                let mut rust = self.as_mut().rust_mut();
                for (row, id) in rust.rows.iter_mut().zip(ids) {
                    row.0 = id;
                }
            }
            self.as_mut().end_reset_model();
        }
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let (Some((_, row)), Some(key)) = (
            self.rows.get(index.row() as usize),
            self.keys.get((role - USER_ROLE) as usize),
        ) else {
            return QVariant::default();
        };
        to_variant(&lookup(row, key))
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut h = QHash::<QHashPair_i32_QByteArray>::default();
        for (i, key) in self.keys.iter().enumerate() {
            h.insert(USER_ROLE + i as i32, QByteArray::from(key.as_str()));
        }
        h
    }

    fn sync_count(mut self: Pin<&mut Self>) {
        let n = self.rows.len() as i32;
        self.as_mut().set_count(n);
    }

    pub fn reset(mut self: Pin<&mut Self>, records: &QJsonArray) {
        let key = self.id_key().to_string();
        let rows: Vec<(f64, QJsonObject)> = records
            .iter()
            .filter(|v| v.is_object())
            .map(|v| {
                let o = v.to_object();
                (id_of(&o, &key), o)
            })
            .collect();
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().rows = rows;
            self.as_mut().end_reset_model();
        }
        self.sync_count();
    }

    pub fn index_of(&self, id: f64) -> i32 {
        self.rows
            .iter()
            .position(|(rid, _)| *rid == id)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }

    pub fn at(&self, row: i32) -> QJsonObject {
        self.rows
            .get(row as usize)
            .map(|(_, o)| o.clone())
            .unwrap_or_default()
    }

    fn touch(mut self: Pin<&mut Self>, row: i32) {
        let index = self.as_ref().index(row, 0, &QModelIndex::default());
        let roles = QVector::<i32>::default();
        self.as_mut().data_changed(&index, &index, &roles);
    }

    pub fn upsert(mut self: Pin<&mut Self>, record: &QJsonObject) {
        let key = self.id_key().to_string();
        let id = id_of(record, &key);
        let row = self.as_ref().index_of(id);
        if row >= 0 {
            self.as_mut().rust_mut().rows[row as usize].1 = record.clone();
            self.touch(row);
        } else {
            let end = self.rows.len() as i32;
            unsafe {
                self.as_mut()
                    .begin_insert_rows(&QModelIndex::default(), end, end);
                self.as_mut().rust_mut().rows.push((id, record.clone()));
                self.as_mut().end_insert_rows();
            }
            self.sync_count();
        }
    }

    pub fn upsert_all(mut self: Pin<&mut Self>, records: &QJsonArray) {
        for v in records.iter() {
            if v.is_object() {
                self.as_mut().upsert(&v.to_object());
            }
        }
    }

    pub fn remove(mut self: Pin<&mut Self>, id: f64) {
        let row = self.as_ref().index_of(id);
        if row < 0 {
            return;
        }
        unsafe {
            self.as_mut()
                .begin_remove_rows(&QModelIndex::default(), row, row);
            self.as_mut().rust_mut().rows.remove(row as usize);
            self.as_mut().end_remove_rows();
        }
        self.sync_count();
    }

    pub fn remove_all(mut self: Pin<&mut Self>, ids: &QJsonArray) {
        for v in ids.iter() {
            if v.is_double() {
                self.as_mut().remove(v.to_double());
            }
        }
    }

    pub fn patch(mut self: Pin<&mut Self>, id: f64, fields: &QJsonObject) {
        let row = self.as_ref().index_of(id);
        if row < 0 {
            return;
        }
        {
            let mut rust = self.as_mut().rust_mut();
            let (_, record) = &mut rust.rows[row as usize];
            for key in fields.keys().iter() {
                record.insert(key, &fields.value(key));
            }
        }
        self.touch(row);
    }
}
