use crate::{module::StorageTransaction, Storage};

pub trait ModuleStorage {
    type Stateless: Storage + StorageTransaction;

    type Stateful: Storage + StorageTransaction;
}
