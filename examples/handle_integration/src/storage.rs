use atelier_assets::core::type_uuid::TypeUuid;
use atelier_assets::loader::{
    crossbeam_channel::Sender,
    handle::{AssetHandle, RefOp, TypedAssetStorage},
    storage::{AssetLoadOp, AssetStorage, IndirectionTable, LoadHandle, LoaderInfoProvider},
    AssetTypeId,
};
use std::{any::Any, cell::RefCell, collections::HashMap, error::Error, sync::Arc};
use uuid::Uuid;

pub struct GenericAssetStorage {
    storage: RefCell<HashMap<AssetTypeId, Box<dyn TypedStorage>>>,
    refop_sender: Arc<Sender<RefOp>>,
    indirection_table: IndirectionTable,
}

impl GenericAssetStorage {
    pub fn new(refop_sender: Arc<Sender<RefOp>>, indirection_table: IndirectionTable) -> Self {
        Self {
            storage: RefCell::new(HashMap::new()),
            refop_sender,
            indirection_table,
        }
    }

    pub fn add_storage<T: TypeUuid + for<'a> serde::Deserialize<'a> + 'static>(&self) {
        let mut storages = self.storage.borrow_mut();
        storages.insert(
            AssetTypeId(Uuid::from_bytes(T::UUID)),
            Box::new(Storage::<T>::new(
                self.refop_sender.clone(),
                self.indirection_table.clone(),
            )),
        );
    }
}

struct AssetState<A> {
    version: u32,
    asset: A,
}
pub struct Storage<A: TypeUuid> {
    refop_sender: Arc<Sender<RefOp>>,
    assets: HashMap<LoadHandle, AssetState<A>>,
    uncommitted: HashMap<LoadHandle, AssetState<A>>,
    indirection_table: IndirectionTable,
}
impl<A: TypeUuid> Storage<A> {
    fn new(sender: Arc<Sender<RefOp>>, indirection_table: IndirectionTable) -> Self {
        Self {
            refop_sender: sender,
            assets: HashMap::new(),
            uncommitted: HashMap::new(),
            indirection_table,
        }
    }
    fn get<T: AssetHandle>(&self, handle: &T) -> Option<&A> {
        let handle = if handle.load_handle().is_indirect() {
            self.indirection_table.resolve(handle.load_handle())?
        } else {
            handle.load_handle()
        };
        self.assets.get(&handle).map(|a| &a.asset)
    }
    fn get_version<T: AssetHandle>(&self, handle: &T) -> Option<u32> {
        let handle = if handle.load_handle().is_indirect() {
            self.indirection_table.resolve(handle.load_handle())?
        } else {
            handle.load_handle()
        };
        self.assets.get(&handle).map(|a| a.version)
    }
    fn get_asset_with_version<T: AssetHandle>(&self, handle: &T) -> Option<(&A, u32)> {
        let handle = if handle.load_handle().is_indirect() {
            self.indirection_table.resolve(handle.load_handle())?
        } else {
            handle.load_handle()
        };
        self.assets.get(&handle).map(|a| (&a.asset, a.version))
    }
}
impl<A: TypeUuid + for<'a> serde::Deserialize<'a> + 'static> TypedAssetStorage<A>
    for GenericAssetStorage
{
    fn get<T: AssetHandle>(&self, handle: &T) -> Option<&A> {
        // This transmute can probably be unsound, but I don't have the energy to fix it right now
        unsafe {
            std::mem::transmute(
                self.storage
                    .borrow()
                    .get(&AssetTypeId(Uuid::from_bytes(A::UUID)))
                    .expect("unknown asset type")
                    .as_ref()
                    .any()
                    .downcast_ref::<Storage<A>>()
                    .expect("failed to downcast")
                    .get(handle),
            )
        }
    }
    fn get_version<T: AssetHandle>(&self, handle: &T) -> Option<u32> {
        self.storage
            .borrow()
            .get(&AssetTypeId(Uuid::from_bytes(A::UUID)))
            .expect("unknown asset type")
            .as_ref()
            .any()
            .downcast_ref::<Storage<A>>()
            .expect("failed to downcast")
            .get_version(handle)
    }
    fn get_asset_with_version<T: AssetHandle>(&self, handle: &T) -> Option<(&A, u32)> {
        // This transmute can probably be unsound, but I don't have the energy to fix it right now
        unsafe {
            std::mem::transmute(
                self.storage
                    .borrow()
                    .get(&AssetTypeId(Uuid::from_bytes(A::UUID)))
                    .expect("unknown asset type")
                    .as_ref()
                    .any()
                    .downcast_ref::<Storage<A>>()
                    .expect("failed to downcast")
                    .get_asset_with_version(handle),
            )
        }
    }
}
pub trait TypedStorage: Any {
    fn any(&self) -> &dyn Any;
    fn update_asset(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        data: Vec<u8>,
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send + 'static>>;
    fn commit_asset_version(&mut self, handle: LoadHandle, version: u32);
    fn free(&mut self, handle: LoadHandle, version: u32);
}

impl<A: for<'a> serde::Deserialize<'a> + 'static + TypeUuid> TypedStorage for Storage<A> {
    fn any(&self) -> &dyn Any {
        self
    }
    fn update_asset(
        &mut self,
        loader_info: &dyn LoaderInfoProvider,
        data: Vec<u8>,
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send + 'static>> {
        // To enable automatic serde of Handle, we need to set up a SerdeContext with a RefOp sender
        let asset = futures_executor::block_on(atelier_assets::loader::handle::SerdeContext::with(
            loader_info,
            (*self.refop_sender).clone(),
            async { bincode::deserialize::<A>(&data) },
        ))
        .expect("failed to deserialize asset");
        self.uncommitted
            .insert(load_handle, AssetState { asset, version });
        log::info!("{} bytes loaded for {:?}", data.len(), load_handle);
        // The loading process could be async, in which case you can delay
        // calling `load_op.complete` as it should only be done when the asset is usable.
        load_op.complete();
        Ok(())
    }
    fn commit_asset_version(&mut self, load_handle: LoadHandle, _version: u32) {
        // The commit step is done after an asset load has completed.
        // It exists to avoid frames where an asset that was loaded is unloaded, which
        // could happen when hot reloading. To support this case, you must support having multiple
        // versions of an asset loaded at the same time.
        self.assets.insert(
            load_handle,
            self.uncommitted
                .remove(&load_handle)
                .expect("asset not present when committing"),
        );
        log::info!("Commit {:?}", load_handle);
    }
    fn free(&mut self, load_handle: LoadHandle, version: u32) {
        if let Some(asset) = self.uncommitted.get(&load_handle) {
            if asset.version == version {
                self.uncommitted.remove(&load_handle);
            }
        }
        if let Some(asset) = self.assets.get(&load_handle) {
            if asset.version == version {
                self.assets.remove(&load_handle);
            }
        }
        log::info!("Free {:?}", load_handle);
    }
}

// Untyped implementation of AssetStorage that finds the asset_type's storage and forwards the call
impl AssetStorage for GenericAssetStorage {
    fn update_asset(
        &self,
        loader_info: &dyn LoaderInfoProvider,
        asset_type_id: &AssetTypeId,
        data: Vec<u8>,
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send + 'static>> {
        self.storage
            .borrow_mut()
            .get_mut(asset_type_id)
            .expect("unknown asset type")
            .update_asset(loader_info, data, load_handle, load_op, version)
    }
    fn commit_asset_version(
        &self,
        asset_type: &AssetTypeId,
        load_handle: LoadHandle,
        version: u32,
    ) {
        self.storage
            .borrow_mut()
            .get_mut(asset_type)
            .expect("unknown asset type")
            .commit_asset_version(load_handle, version)
    }
    fn free(&self, asset_type_id: &AssetTypeId, load_handle: LoadHandle, version: u32) {
        self.storage
            .borrow_mut()
            .get_mut(asset_type_id)
            .expect("unknown asset type")
            .free(load_handle, version)
    }
}
