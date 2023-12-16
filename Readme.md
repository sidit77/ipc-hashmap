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

### External Dependencies

* `crossbeam-utils` for the `CachePadded` wrapper to avoid false sharing
* `nix` for POSIX bindings
* `fastrand` for random number generation

### Shared Memory Communication

When the server is started it creates a shared memory region called `/distributed-memory-master`
that contains a monotonic counter for clients. When a new client wants to connect to the server it 
increments this counter and starts a new spsc connection under the name `/distributed-memory-connection-{id}` (more on that later).
The server then detects this increment, spawn a new thread and ties to connect to this new connection.

A connection contains two lock-free circulars buffers for bidirectional communication.
On top of that a sits a blocking interface that uses yielding spin locks.

So when a client wants to retrieve a value from the hashmap it simply put the key and a sentinel value on one
of the circular buffers and then waits for the result of the request to arrive on the second buffer.

To manage disconnects connections use an additional flag that signals that the connection should be considered closed.
A connection can be closed by simply dropping one of the ends (typically the client).

