use crate::manager::RContext;
use crate::{Error, Result};
use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::Debug;
use serde::Serialize;
use serde_json::Value;

/// Response of RPC.
#[derive(Debug)]
pub struct Response<T: Serialize> {
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> Default for Response<T> {
    fn default() -> Self {
        Self {
            code: 0,
            message: String::from("success"),
            data: None,
        }
    }
}

impl<T: Serialize> From<Error> for Response<T> {
    fn from(e: Error) -> Self {
        Self {
            code: e.code(),
            message: e.message(),
            data: None,
        }
    }
}

impl<T: Serialize> Response<T> {
    pub fn new(t: T) -> Self {
        Self {
            code: 0,
            message: String::from("success"),
            data: Some(t),
        }
    }
}

/// Define module's RPC.
#[async_trait::async_trait]
pub trait RPCs<Sl, Sf>: Send + Sync {
    async fn call(
        &mut self,
        ctx: &mut RContext<Sl, Sf>,
        method: &str,
        params: Value,
    ) -> Result<Option<Value>>;
}
