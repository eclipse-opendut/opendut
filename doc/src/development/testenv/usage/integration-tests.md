# Run OpenDuT integration tests

There are some tests that depend on third-party software.
These tests require the test environment to be running and reachable from the machine where the tests are executed.

* Start the test environment:
    ```shell
    cargo theo vagrant ssh
    cargo theo testenv start --skip-telemetry
    ```

* Run the tests that depend on the test environment:
    ```shell
    export OPENDUT_RUN_KEYCLOAK_INTEGRATION_TESTS=true
    export OPENDUT_RUN_NETBIRD_INTEGRATION_TESTS=true
    cargo ci check
    
    # or explicitly run specific tests only
    cargo test --package opendut-auth-tests client::test_confidential_client_get_token --all-features --
    cargo test --package opendut-auth-tests --all-features -- --nocapture
    cargo test --package opendut-vpn-netbird client::integration_tests --all-features -- --nocapture
    ```
