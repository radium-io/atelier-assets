use atelier_core::{ArtifactId, AssetRef, AssetTypeId, AssetUuid, CompressionType};
use atelier_importer::{ArtifactMetadata, SerdeObj, SerializedAsset};
use uuid::Uuid;

use crate::Result;

pub fn create(
    hash: u64,
    id: AssetUuid,
    build_deps: Vec<AssetRef>,
    load_deps: Vec<AssetRef>,
    value: &dyn SerdeObj,
    compression: CompressionType,
    scratch_buf: &mut Vec<u8>,
) -> Result<SerializedAsset<Vec<u8>>> {
    let size = bincode::serialized_size(value)? as usize;
    scratch_buf.clear();
    scratch_buf.resize(size, 0);
    bincode::serialize_into(scratch_buf.as_mut_slice(), value)?;
    let asset_buf = {
        match compression {
            CompressionType::None => scratch_buf.clone(),
            CompressionType::Lz4 => unimplemented!(),
        }
    };

    Ok(SerializedAsset {
        metadata: ArtifactMetadata {
            id: ArtifactId(hash),
            asset_id: id,
            build_deps,
            load_deps,
            compression,
            uncompressed_size: Some(size as u64),
            compressed_size: Some(asset_buf.len() as u64),
            type_id: AssetTypeId(Uuid::from_bytes(value.uuid())),
        },
        data: asset_buf,
    })
}
