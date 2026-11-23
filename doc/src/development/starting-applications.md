# Starting Applications

* Run CARL (backend):
    ```sh
    cargo carl
    ```
    You can then open the UI by going to <https://localhost:8080/> in your web browser.

    You can additionally push sample data into the running CARL instance with:
    ```sh
    cargo ci carl push-samples
    ```

* Run CLEO (CLI for managing CARL):
    ```sh
    cargo cleo
    ```

* Run EDGAR (edge software):  
    You cannot currently run EDGAR directly like the other applications,
    as it's necessary to create a peer in CARL and go through `opendut-edgar setup`.  
    To test EDGAR, see [test-environment](testenv/index.md).


## UI Development
Run `cargo lea` to continuously build the newest changes in the LEA codebase.  
Then you can simply refresh your browser to see them.
