//! A library to help you convert your sync functions into non-blocking thread futures.
//!
//! Futurify uses `futures 0.3` by default.
//!
//! # Examples
//!
//! A simple `actix-web` server serving async endpoints:
//!
//! ```rust,no_run
//! use actix_web_2::{web, App, HttpServer, Responder};
//! use std::time::Duration;
//!
//! async fn index_async() -> impl Responder {
//!    futures03::future::ready("Hello world").await
//! }
//!
//! // this function will block the executor's thread.
//! async fn index_async_blocking() -> impl Responder {
//!    futures03::future::lazy(|_| {
//!        std::thread::sleep(Duration::from_secs(5));
//!        "Hello blocking world"
//!    })
//!    .await
//! }
//!
//! // this function won't block the executor's thread.
//! async fn index_async_non_blocking() -> impl Responder {
//!    futurify::wrap(|| {
//!       std::thread::sleep(Duration::from_secs(5));
//!        "Hello blocking world"
//!    })
//!    .await
//! }
//!
//! fn main() -> std::io::Result<()> {
//!    HttpServer::new(|| {
//!        App::new()
//!            .route("/", web::get().to(index_async))
//!            .route("/blocking", web::get().to(index_async_blocking))
//!            .route("/non-blocking", web::get().to(index_async_non_blocking))
//!    })
//!    .workers(1)
//!    .bind("localhost:8080")?
//!    .run()
//! }
//! ```
//! By using `futurify` you'll be able to run the closure in a new thread and get the returned value in a future.

use futures03::task::Poll;
use futures03::Future;
use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::task::Context;
use std::thread;

/// Future wrapping a sync function that will be executed
/// in a separate thread.
///
/// It uses `std::thread::spawn` and `mpsc::channel` under the hood.
pub struct Futurified<T: Send + 'static, F: FnOnce() -> T + Send + Unpin> {
    tx: Sender<T>,
    rx: Receiver<T>,
    wrapped: Option<F>,
    is_running: bool,
}
/// Wraps a closure to be executed in a separate thread.
/// It will be executed once the returning Future is polled.
///
/// The Future will return whatever the closure returns.
pub fn wrap<T: Send + 'static, F: FnOnce() -> T + Send + 'static + Unpin>(
    wrapped: F,
) -> Futurified<T, F> {
    let (tx, rx) = channel();
    Futurified {
        tx,
        rx,
        wrapped: Some(wrapped),
        is_running: false,
    }
}

/// Similar to `wrap` but this will execute the closure even if the
/// future is never polled.
///
/// See [`wrap`] for more details.
pub fn wrap_eager<T: Send + 'static, F: FnOnce() -> T + Send + 'static + Unpin>(
    wrapped: F,
) -> Futurified<T, F> {
    let mut this = wrap(wrapped);
    this.run();
    this
}

impl<T: Send + 'static, F: FnOnce() -> T + Send + 'static + Unpin> Futurified<T, F> {
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

impl<T: Send + 'static, F: FnOnce() -> T + Send + 'static + Unpin> Future for Futurified<T, F> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let mut_self = self.get_mut();
        if !mut_self.is_running {
            mut_self.run();
        }
        if let Ok(x) = mut_self.rx.try_recv() {
            Poll::Ready(x)
        } else {
            Poll::Pending
        }
    }
}
