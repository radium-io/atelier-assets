use std::{
    ffi::OsStr,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use crate::{AssetTypeId, AssetUuid};

pub fn make_array<A, T>(slice: &[T]) -> A
where
    A: Sized + Default + AsMut<[T]>,
    T: Copy,
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).copy_from_slice(slice);
    a
}

pub fn type_from_slice(slice: &[u8]) -> Option<AssetTypeId> {
    uuid_from_slice(slice).map(|uuid| AssetTypeId(uuid.0))
}

pub fn uuid_from_slice(slice: &[u8]) -> Option<AssetUuid> {
    const BYTES_LEN: usize = 16;

    let len = slice.len();

    if len != BYTES_LEN {
        return None;
    }

    let mut bytes: uuid::Bytes = [0; 16];
    bytes.copy_from_slice(slice);
    Some(AssetUuid(bytes))
}

pub fn to_meta_path(p: &PathBuf) -> PathBuf {
    p.with_file_name(OsStr::new(
        &(p.file_name().unwrap().to_str().unwrap().to_owned() + ".meta"),
    ))
}

pub fn calc_import_artifact_hash<T, V>(id: &AssetUuid, import_hash: u64, dep_list: T) -> u64
where
    V: std::borrow::Borrow<AssetUuid>,
    T: IntoIterator<Item = V>,
{
    let mut hasher = ::std::collections::hash_map::DefaultHasher::new();
    import_hash.hash(&mut hasher);
    (*id).hash(&mut hasher);
    use std::iter::FromIterator;
    let mut deps = Vec::from_iter(dep_list.into_iter());
    deps.sort_by_key(|dep| *dep.borrow());
    deps.dedup_by_key(|dep| *dep.borrow());
    for dep in &deps {
        dep.borrow().hash(&mut hasher);
    }
    hasher.finish()
}
