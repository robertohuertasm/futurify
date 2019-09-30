//! A library to help you convert your sync functions into non-blocking thread futures.
//!
//! Futurify uses `futures 0.1` for the moment.
//!
//! # Examples
//!
//! A simple `actix-web` server serving async endpoints:
//!
//! ```rust,no_run
//! use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
//! use futures::{Async, Future, Poll};
//! use std::time::Duration;
//!
//! fn index_async() -> impl Future<Item = impl Responder, Error = Error> {
//!    futures::future::ok("Hello world")
//! }
//!
//! // this function will block the executor's thread.
//! fn index_async_blocking() -> impl Future<Item = impl Responder, Error = Error> {
//!    futures::lazy(|| {
//!        std::thread::sleep(Duration::from_secs(5));
//!        futures::future::ok("Hello blocking world")
//!    })
//! }
//!
//! // this function won't block the executor's thread.
//! fn index_async_non_blocking() -> impl Future<Item = impl Responder, Error = Error> {
//!    futurify::wrap(|| {
//!       std::thread::sleep(Duration::from_secs(5));
//!        "Hello blocking world"
//!    })
//! }
//!
//! fn main() -> std::io::Result<()> {
//!    HttpServer::new(|| {
//!        App::new()
//!            .route("/", web::get().to_async(index_async))
//!            .route("/blocking", web::get().to_async(index_async_blocking))
//!            .route("/non-blocking", web::get().to_async(index_async_non_blocking))
//!    })
//!    .workers(1)
//!    .bind("localhost:8080")?
//!    .run()
//! }
//! ```
//! By using `futurify` you'll be able to run the closure in a new thread and get the returned value in a future.

use futures::{Async, Future};
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// Future wrapping a sync function that will be executed
/// in a separate thread.
///
/// It uses `std::thread::spawn` and `mpsc::channel` under the hood.
pub struct Futurified<T: Send + 'static, F: FnOnce() -> T + Send, E: Error> {
    tx: Sender<T>,
    rx: Receiver<T>,
    wrapped: Option<F>,
    is_running: bool,
    error: std::marker::PhantomData<E>,
}
/// Wraps a closure to be executed in a separate thread.
/// It will be executed once the returning Future is polled.
///
/// The Future will return whatever the closure returns.
pub fn wrap<T: Send + 'static, F: FnOnce() -> T + Send + 'static, E: Error>(
    wrapped: F,
) -> Futurified<T, F, E> {
    let (tx, rx) = channel();
    Futurified {
        tx,
        rx,
        wrapped: Some(wrapped),
        is_running: false,
        error: std::marker::PhantomData,
    }
}

/// Similar to `wrap` but this will execute the closure even if the
/// future is never polled.
///
/// See [`wrap`] for more details.
pub fn wrap_eager<T: Send + 'static, F: FnOnce() -> T + Send + 'static, E: Error>(
    wrapped: F,
) -> Futurified<T, F, E> {
    let mut this = wrap(wrapped);
    this.run();
    this
}

impl<T: Send + 'static, F: FnOnce() -> T + Send + 'static, E: Error> Futurified<T, F, E> {
    fn run(&mut self) {
        self.is_running = true;
        let tx = self.tx.clone();
        let sfn = self.wrapped.take().unwrap();
        thread::spawn(move || {
            let result = sfn();
            if let Err(e) = tx.send(result) {
                println!("Error sending result: {}", e)
            }
        });
    }
}

impl<T: Send + 'static, F: FnOnce() -> T + Send + 'static, E: Error> Future
    for Futurified<T, F, E>
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if !self.is_running {
            self.run();
        }
        if let Ok(x) = self.rx.try_recv() {
            Ok(Async::Ready(x))
        } else {
            Ok(Async::NotReady)
        }
    }
}
