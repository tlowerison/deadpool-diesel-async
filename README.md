# deadpool-diesel-async

NOTE: Use of this crate should be deprecated in favor of [diesel-async's deadpool integration](https://docs.rs/diesel-async/latest/diesel_async/pooled_connection/deadpool/index.html).

A [deadpool](https://github.com/bikeshedder/deadpool) backend implementation for the upcoming [diesel-async](https://github.com/weiznich/diesel_async) crate. This provides async connection pooling of *async* [diesel](https://github.com/diesel-rs/diesel) connections. It currently only supports the [tokio](https://github.com/tokio-rs/tokio) async runtime as that's the only one supported by diesel-async.

The two main structs exported are:
- `Manager`: implements the `deadpool::managed::Manager` trait
- `AsyncDieselConnection` - modeled off of [deadpool-sync](https://docs.rs/deadpool-sync/0.1.0/deadpool_sync)'s [SyncWrapper](https://docs.rs/deadpool-sync/0.1.0/deadpool_sync/struct.SyncWrapper.html), this is the object which will effectively be returned from `deadpool::managed::Pool.get` and can access a mutable reference to an `AsyncPgConnection`/`AsyncMysqlConnection` through a callback passed to its `interact` method
