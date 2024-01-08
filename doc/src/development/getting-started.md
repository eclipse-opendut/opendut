# Getting Started

## Development Setup
Install the Rust toolchain: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

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
Run `cargo ci lea run` in a separate terminal in the root directory of the repository to continuously build the newest changes.

## Rendered Documentation
To view this documentation fully rendered, run:
```sh
cargo ci doc book open
```
