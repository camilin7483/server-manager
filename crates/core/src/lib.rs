// sm-core — tipos, traits, errores y eventos fundamentales del Server Manager.
// Este crate no debe depender de ningún otro crate interno.

pub mod error;
pub mod events;
pub mod id;
pub mod profiles;
pub mod protocol;
pub mod traits;
pub mod types;

use std::sync::Arc;

pub type Shared<T> = Arc<parking_lot::RwLock<T>>;

pub fn new_shared<T>(value: T) -> Shared<T> {
    Arc::new(parking_lot::RwLock::new(value))
}

// Allow direct access to common types via sm_core::
// Types defined in `types` module are also accessible via `sm_core::types::*`
// IDs are accessible via `sm_core::id::*`
