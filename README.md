# fsdr-blocks -- FutureSDR Community Blocks

A community-made library of blocks for FutureSDR. FutureSDR is An experimental asynchronous SDR runtime for heterogeneous architectures.

[![Crates.io][crates-badge]][crates-url]
[![Apache 2.0 licensed][apache-badge]][apache-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/fsdr-blocks.svg
[crates-url]: https://crates.io/crates/fsdr-blocks
[apache-badge]: https://img.shields.io/badge/license-Apache%202-blue
[apache-url]: https://github.com/futuresdr/fsdr-blocks/blob/master/LICENSE
[actions-badge]: https://github.com/futuresdr/fsdr-blocks/workflows/CI/badge.svg
[actions-url]: https://github.com/futuresdr/fsdr-blocks/actions?query=workflow%3ACI+branch%3Amaster

[Website](https://www.futuresdr.org) |
[Guides](https://www.futuresdr.org/tutorial) |
[API Docs](https://docs.rs/futuresdr/latest/fsdr-blocks) |
[Chat](https://discord.com/invite/vCz29eDbGP/)


## Overview

FutureSDR supports *Blocks* with synchronous or asynchronous implementations for
stream-based or message-based data processing. Blocks can be combined to a
*Flowgraph* and launched on a *Runtime* that is driven by a *Scheduler*.

This library acts as a toolbox on top of FutureSDR to easily build your own flowgraph.

It is made by the community for the community.

## Contributing

Contributions are very welcome. Please see the (work-in-progress) [contributing
guide][contr] for more information. If you develop larger features or work on
major changes with the main intention to submit them upstream, it would be
great, if you could announce them in advance.

[contr]: https://github.com/futuresdr/fsdr-blocks/blob/master/CONTRIBUTING.md

## Conduct

The FutureSDR project adheres to the [Rust Code of Conduct][coc]. It describes
the _minimum_ behavior expected from all contributors.

[coc]: https://github.com/rust-lang/rust/blob/master/CODE_OF_CONDUCT.md

## License

This project is licensed under the [Apache 2.0 license](LICENSE) as [FutureSDR](https://github.com/futuresdr/fsdr-blocks/blob/master/LICENSE)

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in FutureSDR community blocks, shall be licensed as Apache 2.0, without any
additional terms or conditions.