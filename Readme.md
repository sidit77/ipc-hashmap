# Interprocess Hashmap

## Building
```shell
cargo build --release
```

## Running

**Server**
```shell
# Start the server with 10 buckets
./target/release/server 10
```

**Client**
```shell
# Start 64 clients in batches
parallel ./target/release/client ::: {0..64}
```
GNU Parallel can be installed using `sudo apt install parallel`.

## Architecture

### Overview
This project contains three crates, two binary crates (`server` and `client`) and one library crate (`shared`),
that is shared between both binaries and mostly contains shared memory ipc code.

* `server/main.rs`: Server entry point and implementation of the threadsafe hashmap
* `client/main.rs`: Client entry point
* `shared/lib.rs`: Message definitions
* `shared/buffer.rs`: Circular buffer implementation
* `shared/connection.rs`: Connections built on top of `shm.rs` and `buffer.rs`
* `shared/shm.rs`: (Hopefully) safe abstractions on top of POSIX shared memory
* `shared/signal.rs`: Signal handling code for `SIGINT` to allow to use to cleanly shutdown the server. Should probably be in `server` instead of `shared` but it's simpler this way.
* `shared/slot.rs`: The slot system used to establish connections.   

### External Dependencies

* `crossbeam-utils` for the `CachePadded` wrapper to avoid false sharing
* `nix` for POSIX bindings
* `fastrand` for random number generation

### Shared Memory Communication

When the server is started it creates a shared memory region called `/distributed-memory-master`
that contains a monotonic counter for clients. When a new client wants to connect to the server it 
increments this counter and starts a new spsc connection under the name `/distributed-memory-connection-{id}` (more on that later).
The server then detects this increment, spawns a new thread and tries to connect to this new connection.

A connection contains two lock-free circulars buffers for bidirectional communication.
On top of that a sits a blocking interface that uses yielding spin locks.

So when a client wants to retrieve a value from the hashmap it simply put the key and a sentinel value on one
of the circular buffers and then waits for the result of the request to arrive on the second buffer.

To manage disconnects connections use an additional flag that signals that the connection should be considered closed.
A connection can be closed by simply dropping one of the ends (typically the client).

### Future Work

Use some kind of OS primitive (`EventFd`?) to avoid spending to much time in spin locks.

