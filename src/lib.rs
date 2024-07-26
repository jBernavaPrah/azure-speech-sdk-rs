#![deny(unsafe_code)]

//! Azure Speech SDK - Pure Rust, unofficial and opinionated project.
//!
//! This crate provides a high-level API to interact with Azure Speech Services.
//! It is designed to be simple and easy to use, while providing a lot of flexibility,
//! without any external C dependencies, and it is based on the `tokio` runtime.
//!
//! It's use channels to return the `events`. The `events` are the result of the recognition process.
//!
//! This crate not require any external C dependencies, and it is based on the `tokio` runtime.
//!
//! For more information about Microsoft Speech Service see [here](https://docs.microsoft.com/en-us/azure/cognitive-services/speech-service/speech-sdk?tabs=windows%2Cubuntu%2Cios-xcode%2Cmac-xcode%2Candroid-studio).
//!
//! # Features
//! - [X] Speech to Text
//!     - [X] Continuous Recognition
//!     - [X] Single Shot Recognition (with a manual break in the events loop)
//!     - [X] File Recognition (with hound crate)
//!     - [X] Microphone Recognition (with cpal crate)
//!     - [X] Stream Recognition (with tokio::sync::mpsc)
//!     - [ ] Translation (work in progress) 
//! - [ ] Text to Speech (work in progress)
//!
//!
//! # Example
//! You can find examples in the `examples` directory.
//!

/// Specific events for the speech recognition
//mod recognizer;
mod connector;
pub mod auth;
mod utils;
mod event;
mod config;
mod source;

pub(crate) mod message;
pub mod synthesizer;

use std::result;
use serde::Deserialize;
pub use crate::event::{Event, EventBase, CancelledReason};
pub use crate::source::{Details, Spec, SampleFormat};

/// Result type for the library.
pub type Result<T> = result::Result<T, Error>;


#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// Error struct, used to represent errors in the library.
pub enum Error {
    IOError(String),
    InvalidResponse(String),
    ParseError(String),
    InternalError(String),
    ServerDisconnect(String),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::InternalError(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::InternalError(s)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e.to_string())
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Error::InternalError(e.to_string())
    }

}

pub mod stream_ext {
    use core::fmt;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use pin_project_lite::pin_project;
    use tokio_stream::Stream;

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
                    let ready = ready.
                        and_then(|item| {
                            if (self.as_mut().project().predicate)(&item) {
                                *self.as_mut().project().done = true;
                            }
                            Some(item)
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
    
    
    pub trait StreamExt: Stream {
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
        /// use crate::azure_speech::stream_ext::StreamExt  as _;
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
    }

    impl<St: ?Sized> StreamExt for St where St: Stream {}
    
}

// 
// pub(crate) mod trait_ext {
// 
//     use std::iter::{FusedIterator};
//     use tokio_stream::{StreamExt as StreamExtTokio};
// 
//     pub(crate) trait IteratorExt: Iterator {
//         fn stop_after<P>(self, predicate: P) -> StopAfter<Self, P>
//         where
//             Self: Sized,
//             P: FnMut(&Self::Item) -> bool,
//         {
//             StopAfter::new(self, predicate)
//         }
//     }
//     
//     pub(crate) trait StreamExt: StreamExtTokio {
//         fn stop_after<P>(self, predicate: P) -> StopAfter<Self, P>
//         where
//             Self: Sized,
//             P: FnMut(&Self::Item) -> bool,
//         {
//             StopAfter::new(self, predicate)
//         }
//     }
// 
//     pub(crate) struct StopAfter<I, P> {
//         iter: I,
//         flag: bool,
//         predicate: P,
//     }
// 
//     impl<I, P> StopAfter<I, P> {
//         fn new(iter: I, predicate: P) -> Self {
//             Self {
//                 iter,
//                 predicate,
//                 flag: false,
//             }
//         }
//     }
// 
//     impl<I: Iterator, P> Iterator for StopAfter<I, P>
//     where
//         P: FnMut(&I::Item) -> bool,
//     {
//         type Item = I::Item;
// 
//         #[inline]
//         fn next(&mut self) -> Option<I::Item> {
//             if self.flag {
//                 None
//             } else {
//                 let x = self.iter.next()?;
//                 if (self.predicate)(&x) {
//                     self.flag = true;
//                 }
//                 Some(x)
//             }
//         }
//     }
// 
//     impl<I, P> FusedIterator for StopAfter<I, P>
//     where
//         I: FusedIterator,
//         P: FnMut(&I::Item) -> bool,
//     {
//     }
// 
//     impl<I: Iterator> IteratorExt for I {}
//     impl<I: StreamExt> StreamExt for I {}
//     
// }