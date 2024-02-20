# Getting Started

## Development Setup
Install the Rust toolchain: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

You may need additional dependencies. On Ubuntu/Debian, these can be installed with:
```sh
sudo apt install build-essential pkg-config libssl-dev
```

To see if your development setup is generally working, you can run `cargo ci check` in the project directory.  
Mind that this runs the unit tests and additional code checks and may occasionally show warnings/errors related to those, rather than pure build errors.


## Tips & Tricks

* `cargo ci` contains many utilities for development in general.

* To view this documentation fully rendered, run `cargo ci doc book open`.

* To have your code validated more extensively, e.g. before publishing your changes, run `cargo ci check`.
