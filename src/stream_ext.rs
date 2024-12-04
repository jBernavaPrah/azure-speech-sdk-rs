use crate::callback::Callback;
use core::fmt;
use core::pin::Pin;
use core::task::{Context, Poll};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::pin;
use tokio_stream::{Stream, StreamExt as _};

pin_project! {
/// Stream for the [`stop_after`](stop_after) method.
#[must_use = "streams do nothing unless polled"]
    pub struct StopAfter<St, F> {
        #[pin]
        stream: St,
        predicate: F,
        done: bool,
    }
}

impl<St, F> StopAfter<St, F> {
    pub(super) fn new(stream: St, predicate: F) -> Self {
        Self {
            stream,
            predicate,
            done: false,
        }
    }
}

impl<St, F> fmt::Debug for StopAfter<St, F>
where
    St: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StopAfter")
            .field("stream", &self.stream)
            .field("done", &self.done)
            .finish()
    }
}

impl<St, F> Stream for StopAfter<St, F>
where
    St: Stream,
    F: FnMut(&St::Item) -> bool,
{
    type Item = St::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !*self.as_mut().project().done {
            self.as_mut().project().stream.poll_next(cx).map(|ready| {
                let ready = ready.map(|item| {
                    if (self.as_mut().project().predicate)(&item) {
                        *self.as_mut().project().done = true;
                    }
                    item
                });
                ready
            })
        } else {
            Poll::Ready(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }

        let (_, upper) = self.stream.size_hint();

        (0, upper)
    }
}

/// An extension trait for `Stream` that provides a variety of convenient combinator functions.
pub trait StreamExt: Stream
where
    Self: 'static,
{
    /// Takes elements from this stream until the provided predicate resolves to `true`.
    ///
    /// This function operates similarly to `Iterator::take_while`, extracting elements from the
    /// stream until the predicate `f` evaluates to `false`. Unlike `Iterator::take_while`, this function
    /// also returns the last evaluated element for which the predicate was `true`, marking the stream as done afterwards.
    /// Once an element causes the predicate to return false, the stream will consistently return that it is finished.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use tokio_stream::{self as stream, StreamExt as _};
    /// use azure_speech::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     
    /// let mut stream = stream::iter(1..=5).stop_after(|&x| x >= 3);
    ///
    ///     assert_eq!(Some(1), stream.next().await);
    ///     assert_eq!(Some(2), stream.next().await);
    ///     assert_eq!(Some(3), stream.next().await);
    ///     // Since 4 > 3, the stream is now considered done
    ///     assert_eq!(None, stream.next().await);
    /// }
    /// ```
    ///
    /// This function is particularly useful when you need to process elements of a stream up to a certain point,
    /// and then stop processing, including the element that caused the stop condition.
    fn stop_after<F>(self, f: F) -> StopAfter<Self, F>
    where
        F: FnMut(&Self::Item) -> bool,
        Self: Sized,
    {
        StopAfter::new(self, f)
    }

    /// Calls the provided callback for each item in the stream.
    fn use_callbacks<C>(self, callback: C) -> impl Future<Output = ()>
    where
        Self: Sized + Send + Sync,
        C: Callback<Item = Self::Item> + 'static,
    {
        async move {
            let mut _self = pin!(self);
            while let Some(event) = _self.next().await {
                callback.on_event(event).await;
            }
        }
    }
}

impl<St: ?Sized + 'static> StreamExt for St where St: Stream {}
