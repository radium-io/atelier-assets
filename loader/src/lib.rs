#![warn(clippy::all, rust_2018_idioms, rust_2018_compatibility)]

/// *feature:* `handle`. Handles provide automatic reference counting of assets, similar to [Rc](`std::rc::Rc`).
#[cfg(feature = "handle")]
pub mod handle;
/// [`LoaderIO`](crate::io::LoaderIO) provides data requested by [`Loader`](crate::loader::Loader).
pub mod io;
/// [`Loader`] loads assets into engine-implemented [`AssetStorage`](crate::storage::AssetStorage)s.
pub mod loader;
/// *feature:* `rpc_io`. `RpcIO` is an implementation of [`LoaderIO`](crate::io::LoaderIO) which communicates with `atelier_daemon`
/// to load and hot reload assets. Intended for development workflows.
#[cfg(feature = "rpc_io")]
pub mod rpc_io;
/// [`AssetStorage`](crate::storage::AssetStorage) is implemented by engines to store loaded asset data.
pub mod storage;

#[cfg(feature = "asset_uuid_macro")]
pub use atelier_core::asset_uuid;
pub use atelier_core::{AssetRef, AssetTypeId, AssetUuid};
pub use crossbeam_channel;
pub use loader::Loader;
#[cfg(feature = "rpc_io")]
pub use rpc_io::RpcIO;
pub use storage::LoadHandle;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + 'static>>;

#[cfg(feature = "handle")]
#[macro_export]
macro_rules! if_handle_enabled {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

#[cfg(not(feature = "handle"))]
#[macro_export]
#[doc(hidden)]
macro_rules! if_handle_enabled {
    ($($tt:tt)*) => {};
}
