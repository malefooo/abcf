#![feature(generic_associated_types)]

use std::marker::PhantomData;

use abcf::{
    abci::{RequestBeginBlock, RequestEndBlock},
    manager::{AContext, TContext},
    module::{
        types::{
            RequestCheckTx, RequestDeliverTx, ResponseCheckTx, ResponseDeliverTx, ResponseEndBlock,
        },
        StorageTransaction,
    },
    Application, Event, Result, Stateful, StatefulBatch, Stateless, StatelessBatch,
};
use bs3::model::{Map, Value};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Module's Event
#[derive(Clone, Debug, Deserialize, Serialize, Event)]
pub struct Event1 {}

pub trait Config: Send + Sync {
    type Ty: Debug + Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static;
}

#[abcf::module(name = "utxo", version = 1, impl_version = "0.1.1", target_height = 0)]
pub struct UTXOModule<C: Config> {
    pub inner: u32,
    marker: PhantomData<C>,
    #[stateful]
    pub sf_value: Value<C::Ty>,
    #[stateful]
    pub sf_value1: Value<C::Ty>,
    #[stateless]
    pub sl_value: Value<C::Ty>,
    #[stateless]
    pub sl_map: Map<i32, C::Ty>,
}

#[abcf::rpcs]
impl<C: Config + Sync + Send> UTXOModule<C> {}

/// Module's block logic.
#[abcf::application]
impl<C: Config + Sync + Send> Application for UTXOModule<C> {
    type Transaction = Vec<u8>;

    async fn check_tx(
        &mut self,
        _context: &mut TContext<StatelessBatch<'_, Self>, StatefulBatch<'_, Self>>,
        _req: &RequestCheckTx<Self::Transaction>,
    ) -> Result<ResponseCheckTx> {
        let e = Event1 {};
        _context.events.emmit(e).unwrap();
        Ok(Default::default())
    }

    async fn begin_block(
        &mut self,
        _context: &mut AContext<Stateless<Self>, Stateful<Self>>,
        _req: &RequestBeginBlock,
    ) {
        // use bs3::ValueStore;
        // let a = _context.stateless.sl_value.get();
    }

    async fn deliver_tx(
        &mut self,
        _context: &mut TContext<StatelessBatch<'_, Self>, StatefulBatch<'_, Self>>,
        _req: &RequestDeliverTx<Self::Transaction>,
    ) -> Result<ResponseDeliverTx> {
        Ok(Default::default())
    }

    async fn end_block(
        &mut self,
        _context: &mut AContext<Stateless<Self>, Stateful<Self>>,
        _req: &RequestEndBlock,
    ) -> ResponseEndBlock {
        Default::default()
    }
}

/// Module's methods.
#[abcf::methods]
impl<C: Config + Sync + Send> UTXOModule<C> {}
