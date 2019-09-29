use futures::{Async, Future};
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// Future wrapping a sync function that will be executed
/// in a separate thread.
///
/// It uses `std::thread::spawn` and `mpsc::channel` under the hood.
pub struct Futurified<T: Send + 'static, E: Error> {
    tx: Sender<T>,
    rx: Receiver<T>,
    wrapped: fn() -> T,
    is_running: bool,
    error: std::marker::PhantomData<E>,
}
/// Wraps a closure to be executed in a separate thread.
/// It will be executed once the returning Future is polled.
///
/// The Future will return whatever the closure returns.
pub fn wrap<T: Send + 'static, E: Error>(wrapped: fn() -> T) -> Futurified<T, E> {
    let (tx, rx) = channel();
    Futurified {
        tx,
        rx,
        wrapped,
        is_running: false,
        error: std::marker::PhantomData,
    }
}

/// Similar to `wrap` but this will execute the closure even if the
/// future is never polled.
///
/// See [`wrap`] for more details.
pub fn wrap_eager<T: Send + 'static, E: Error>(wrapped: fn() -> T) -> Futurified<T, E> {
    let mut this = wrap(wrapped);
    this.run();
    this
}

impl<T: Send + 'static, E: Error> Futurified<T, E> {
    fn run(&mut self) {
        self.is_running = true;
        let tx = self.tx.clone();
        let sfn = self.wrapped;
        thread::spawn(move || {
            let result = sfn();
            if let Err(e) = tx.send(result) {
                println!("Error sending result: {}", e)
            }
        });
    }
}

impl<T: Send + 'static, E: Error> Future for Futurified<T, E> {
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
