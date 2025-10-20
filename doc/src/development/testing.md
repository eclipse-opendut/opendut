# Testing

* Run tests with `cargo ci check`.

## Integration tests

There are special test cases in the code base that try to interact with a service run in the test environment.
These tests only run when the respective service is started.

Those tests are flagged as such using the crate [test-with](https://crates.io/crates/test-with) and assume the presence or absence of an environment variable:
* The following test is only run if the environment variable `OPENDUT_RUN_<service-name-in-upper-case>_INTEGRATION_TESTS` is present:
    ```rust
    #[test_with::env(OPENDUT_RUN_KEYCLOAK_INTEGRATION_TESTS)]
    #[test]
    fn test_communication_with_service_works() {
        assert!(true);
    }
    ```

See [Run OpenDuT integration tests](testenv/usage/integration-tests.md) for details.

## Running tests with logging

By default, log output is suppressed when running tests. 
To see log output when running tests, mark your test with the `#[test_log::test]` attribute instead of the standard `#[test]` attribute, for example:
    ```rust
    #[test_log::test]
    fn it_still_works() {
      // ...
    }
    ```
And set the environment variable `RUST_LOG` to the desired log level before running the tests.
To see all log messages at the `debug` level and above, you can then run:
    ```shell
    RUST_LOG=debug cargo ci check
    ```
