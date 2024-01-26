# Getting Started

## Development Setup
Install the Rust toolchain: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)  
For compiling Rust, you additionally need a C linker. On Linux, you can install the `gcc` package from your distribution.

## Starting the Applications
* Run CARL (backend):
    ```sh
    cargo ci carl run
    ```
You can then open the UI by going to https://localhost:8080/ in your web browser.

* Run EDGAR (edge software):
    ```sh
    cargo ci edgar run -- service
    ```

* Run CLEO (CLI for managing CARL):
    ```sh
    cargo ci cleo run
    ```

## UI Development
Run `cargo ci lea run` to continuously build the newest changes.  
During your first run, you may need to install additional system packages. On Linux, this may include the `pkg-config` package, as well as the package for OpenSSL development symbols.

## Tips & Tricks

* `cargo ci` contains many utilities for development in general.

* To view this documentation fully rendered, run `cargo ci doc book open`.

* To have your code validated more extensively, e.g. before publishing your changes, run `cargo ci check`.
