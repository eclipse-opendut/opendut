# Getting Started

## Development Setup
1. Install Rust toolchain: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
2. Install WASM toolchain:
    ```sh
    rustup target add wasm32-unknown-unknown
    ```
3. Install required crates:
    ```sh
    cargo install --force cargo-make trunk mdbook mdbook-mermaid mdbook-svgbob
    ```

## Starting the application
1. Run CARL:
    ```sh
    cargo carl
    ```
2. Open the UI by going to https://localhost:8080/ in your web browser.
3. Run EDGAR:
    ```sh
    cargo run edgar -- service
    ```

## UI Development
Run `cargo lea` in a separate terminal in the root directory of the repository to continuously build the newest changes.
