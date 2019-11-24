//! A library to help you convert your sync functions into non-blocking thread futures.
//!
//! Futurify uses Futures 0.3 by default but if you want to use 0.1, you can opt-in by using
//! the `futures_01` feature.
//!
//! By using `futurify` you'll be able to run the closure in a new thread and get the returned value in a future.

#[cfg(feature = "futures_01")]
mod futures_01;
#[cfg(feature = "futures_01")]
pub use futures_01::wrap;
#[cfg(feature = "futures_01")]
pub use futures_01::wrap_eager;

#[cfg(feature = "futures_03")]
mod futures_03;
#[cfg(feature = "futures_03")]
pub use futures_03::wrap;
#[cfg(feature = "futures_03")]
pub use futures_03::wrap_eager;

//#[cfg(not(feature = "futures_01"))]
//mod futures_03;
//#[cfg(not(feature = "futures_01"))]
//pub use futures_03::wrap;
//#[cfg(not(feature = "futures_01"))]
//pub use futures_03::wrap_eager;
