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
