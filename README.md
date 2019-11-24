# Futurify

[![ActionsStatus](https://github.com/robertohuertasm/futurify/workflows/Build/badge.svg)](https://github.com/robertohuertasm/futurify/actions) [![Crates.io](https://img.shields.io/crates/v/futurify.svg)](https://crates.io/crates/futurify)

Convert your sync functions into non-blocking thread futures.

Support for both `Futures 0.1` and `Futures 0.3`. By default, `Futures 0.3` will be used.

## When to use it

This is just an example to illustrate when this library could be useful.

Imagine you're creating an [actix-web](https://github.com/actix/actix-web) API and you have and endpoint that has to execute a very long synchronous task.

Normally, most of the modern crates will provide some `async` variants but still there are some of them that only provide a `sync` implementation. 

This could be problematic as this endpoint could block the main thread and avoid other endpoints from being executed.

Take a look at this code:

```rust
use actix_web::{web, App, HttpServer, Responder};
use std::time::Duration;

async fn index_async() -> impl Responder {
    futures::future::ready("Hello world").await
}

async fn index_async_blocking() -> impl Responder {
    futures::future::lazy(|_| {
        std::thread::sleep(Duration::from_secs(5));
        "Hello blocking world"
    })
    .await
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
async fn index_async_non_blocking() -> impl Responder {
    futurify::wrap(|| {
        std::thread::sleep(Duration::from_secs(5));
        "Hello non blocking world"
    })
}

fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index_async))
            .route("/blocking", web::get().to(index_async_blocking))
            .route("/non-blocking", web::get().to(index_async_non_blocking))
    })
    .workers(1)
    .bind("localhost:8080")?
    .run()
}
````

Just wrap a closure with `futurify::wrap` and you'll get a `futures 0.3` future ready to be polled.

## Futures 0.1 support

If you need support for `Futures 0.1` just import the library like this in your `Cargo.toml`:

```toml
futurify = { version = "0.3", features = "futures_01", default-features = false }
```

That should do the trick!
