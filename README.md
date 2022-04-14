# deadpool-diesel-async
A [deadpool](https://github.com/bikeshedder/deadpool) backend implementation for the upcoming [diesel-async](https://github.com/weiznich/diesel_async) crate. This provides async connection pooling of *async* [diesel](https://github.com/diesel-rs/diesel) connections. It currently only supports the [tokio](https://github.com/tokio-rs/tokio) async runtime as that's the only one supported by diesel-async currently (from what I can tell).

This crate depends on specific Github revisions for both diesel and diesel-async -- not exactly the definition of stable but it works from what I can tell :) The diesel revision depended on has version `2.0.0`, the docs can be found at https://docs.diesel.rs/master/diesel.

The two main structs exported are:
- `Manager`: implements the `deadpool::managed::Manager` trait
- `AsyncDieselConnection` - modeled off of [deadpool-sync](https://docs.rs/deadpool-sync/0.1.0/deadpool_sync)'s [SyncWrapper](https://docs.rs/deadpool-sync/0.1.0/deadpool_sync/struct.SyncWrapper.html), this is the object which will effectively be returned from `deadpool::managed::Pool.get` and can access a mutable reference to an `AsyncPgConnection`/`AsyncMysqlConnection` through a callback passed to its `interact` method
