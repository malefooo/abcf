use alloc::boxed::Box;
pub use tm_protos::abci::{RequestBeginBlock, RequestCheckTx, RequestDeliverTx, RequestEndBlock};

mod types;
pub use types::{ResponseCheckTx, ResponseDeliverTx, ResponseEndBlock};

/// This trait define module's main logic.
#[async_trait::async_trait]
pub trait Application: Send{
    /// Define how to check transaction.
    ///
    /// In this function, do some lightweight check for transaction, for example: check signature,
    /// check balance and so on.
    /// This method will be called at external user or another node.
    async fn check_tx(&mut self, _req: &RequestCheckTx) -> ResponseCheckTx {
        ResponseCheckTx::default()
    }

    /// Begin block.
    async fn begin_block(&mut self, _req: &RequestBeginBlock) {}

    /// Execute transaction on state.
    async fn deliver_tx(&mut self, _req: &RequestDeliverTx) -> ResponseDeliverTx {
        ResponseDeliverTx::default()
    }

    /// End Block.
    async fn end_block(&mut self, _req: &RequestEndBlock) -> ResponseEndBlock {
        ResponseEndBlock::default()
    }
}
