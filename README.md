# pipepiper

PipePiper (ppr) is a utility written in Rust that is designed to transmit data
using UNIX pipes across the internet, particularly when long distance and high
latency is involved.

It uses the QUIC protocol thanks to the [quinn-rs/quinn](https://github.com/quinn-rs/quinn) crate.

## Getting started
There are no binaries available just yet (sorry), so you will need to compile PipePiper yourself.

```sh
# Clone the repo
git clone https://github.com/Zorlin/pipepiper.git
# Change directory into the Ferrous Pipe repo
cd pipepiper
# Build and install the software
cargo install --bin ppr --path .
```

Or install from [crates.io](https://crates.io)

```sh
cargo install pipepiper
```

This will install a binary named `ppr` to Cargo's local set of installed
binaries. The same binary can be found in the `target/release` folder.

## Usage
Copy `example-config.toml` from PipePiper's repository to `config.toml` and edit the settings as appropriate.

Once installed, start the receiver:

* `ppr recv 0.0.0.0 8000 --config config.toml`

and then run the sender and pipe some simple text in

* `echo "Hello world!" | ppr send 127.0.0.1 8000 --config config.toml`

Use the help flag (`-h or --help`) to see the full set of options.
For config file specification, see the `example-config.toml` file.

This tool is intended to be used to speed up ZFS send/receive over
intercontinental distances, but can be used for anything you can use a UNIX
pipe to achieve!

## Credits and License
Made available under the terms of, at your choice and preference, the MIT or Apache software licenses.

Developed by [@brkp](https://github.com/brkp) and [@Zorlin](https://github.com/Zorlin).

Development funded by [@Zorlin](https://github.com/Zorlin) and [@application-research](https://github.com/application-research).
