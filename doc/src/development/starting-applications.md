# Starting Applications

* Run CARL (backend):
    ```sh
    cargo ci carl run
    ```
You can then open the UI by going to https://localhost:8080/ in your web browser.

* Run CLEO (CLI for managing CARL):
    ```sh
    cargo ci cleo run
    ```

* Run EDGAR (edge software):
    ```sh
    cargo ci edgar run -- service
    ```
  Mind that this is in a somewhat broken state and may be removed in the future,  
  as it's normally necessary to add the peer in CARL and then go through `edgar setup`.  
  For a more realistic environment, see [test-environment](testenv/_chapter).


## UI Development
Run `cargo ci lea run` to continuously build the newest changes in the LEA codebase.  
Then you can simply refresh your browser to see them.
