# Futurify

Convert your sync functions into non-thread blocking futures.

# When to use it

This is just an example to illustrate when this library could be useful.

Imagine you're creating an [actix-web](https://github.com/actix/actix-web) API and you have and endpoint that has to execute a very long synchronous task.

Normally, most of the modern crates will provide some `async` variants but still there are some of them that only provide a `sync` implementation. 

This could be problematic as this endpoint could block the main thread and avoid other endpoints from being executed.

Take a look at this code:

```rust
use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
use futures::{Async, Future, Poll};

fn index_async() -> impl Future<Item = impl Responder, Error = Error> {
    futures::future::ok("Hello world")
}

fn index_async_blocking() -> impl Future<Item = impl Responder, Error = Error> {
    futures::lazy(|| {
        std::thread::sleep(Duration::from_secs(5));
        futures::future::ok("Hello blocking world")
    })
}

fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to_async(index_async))
            .route("/blocking", web::get().to_async(index_async_blocking))
    })
    .workers(1)
    .bind("localhost:8080")?
    .run()
}
```

If you try to get `localhost:8080/blocking` and then get `localhost:8080`, you will be able to see that you won't get any response until the `blocking` endpoint has returned.

With just a slight change we could get rid of this issue:

```rust
fn index_async_non_blocking() -> impl Future<Item = impl Responder, Error = Error> {
    futurify::wrap(|| {
        std::thread::sleep(Duration::from_secs(5));
        "Hello blocking world"
    })
}

fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to_async(index_async))
            .route("/blocking", web::get().to_async(index_async_blocking))
            .route("/non-blocking", web::get().to_async(index_async_non_blocking))
    })
    .workers(1)
    .bind("localhost:8080")?
    .run()
}
````

Just wrap a closure with `futurify::wrap` and you'll get a `futures 0.1` future ready to be polled.

