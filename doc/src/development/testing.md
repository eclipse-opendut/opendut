# Testing

* Run tests with `cargo ci check`.

## Special test cases

There are two categories of special test cases in the code base:
1. tests that start a docker container (using `testcontainers-rs` crate)
2. tests that try to interact with a service run in the test environment virtual machine. These tests only run when the tests and the respective service is started in the virtual machine.

Those tests are flagged as such using the crate [test-with](https://crates.io/crates/test-with) and assume the presence or absence of an environment variable:
* The following test is only run if the environment variable `INTEGRATION_TEST_OF_SERVICE` is present:
```rust
#[test_with::env(INTEGRATION_TEST_OF_SERVICE)]
#[test]
fn test_communication_with_service_works() {
    assert!(true);
}
```
* The following test is only run if the environment variable `INTEGRATION_TEST_OF_SERVICE` is absent:
```rust
#[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
#[test]
fn test_communication_with_database() {
    assert!(true);
}
```
This is used to disable the database tests. See below. 

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
