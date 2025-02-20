# Zenoh Blocks Example

## Overview
This example demonstrates how to use the `PubSink` and `SubSource` blocks. The `PubSink` declares a Zenoh publisher that publishes samples to the given key expression. The `SubSource` declares a Zenoh subscriber that subscribes to samples from the given key expression.

## Build
To build the example follow these steps:

- Open a terminal
- Run `cargo build`

## Run
To run the example follow these steps:

- Open a terminal
- Run `cargo run --bin zenoh-receiver`
- Open another terminal
- Run `cargo run --bin zenoh-sender`
- Terminate both programs after an arbitrary amount of time
- The output file will be located at `/tmp/zenoh-log.bin`

## References
- [FutureSDR](https://www.futuresdr.org/)
- [Zenoh](https://zenoh.io/)