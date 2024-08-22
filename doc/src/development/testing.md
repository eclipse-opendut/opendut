# Testing

* Run tests with `cargo ci check`.

## Special test cases

### Database tests
At the moment the database tests run with `testcontainers-rs` crate which only supports the official docker runtime.
If you are using `podman` you may want to skip those tests.
You may do so by setting the following environment variable:
```shell
export SKIP_DATABASE_CONTAINER_TESTS=true
```

### Keycloak integration tests

There are some tests that require the keycloak of the test environment to be running.
The tests assume that you run them in the virtual machine of the test environment.
```shell
cargo theo vagrant ssh
cargo theo testenv start --skip-telemetry
export RUN_KEYCLOAK_INTEGRATION_TESTS=true
cargo ci check

# explicitly run integration test only
cargo test --package opendut-auth confidential::client::auth_tests::test_confidential_client_get_token --all-features --
cargo test --package opendut-auth-tests --all-features -- --nocapture
```
