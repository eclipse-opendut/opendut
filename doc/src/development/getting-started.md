# Getting Started

## Development Setup
Install the Rust toolchain: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

## Starting the Applications
* Run CARL (backend):
    ```sh
    cargo carl
    ```
You can then open the UI by going to https://localhost:8080/ in your web browser.

* Run EDGAR (edge software):
    ```sh
    cargo edgar service
    ```

* Run CLEO (CLI for managing CARL):
    ```sh
    cargo cleo
    ```

## UI Development
Run `cargo lea` in a separate terminal in the root directory of the repository to continuously build the newest changes.

## Rendered Documentation
To view this documentation fully rendered, run:
```sh
cargo ci doc book open
```
