// src/callback.rs
use crate::RequestId;
use std::future::Future;
use std::pin::Pin;

pub(crate) type OnSessionStarted = Box<dyn Fn(RequestId) -> BoxFuture>;
pub(crate) type OnSessionEnded = Box<dyn Fn(RequestId) -> BoxFuture>;
pub(crate) type OnError = Box<dyn Fn(RequestId, crate::Error) -> BoxFuture>;
pub(crate) type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[async_trait::async_trait]
pub trait Callback
{
    type Item;
    fn on_event(&self, item: Self::Item) -> impl Future<Output = ()>;
}