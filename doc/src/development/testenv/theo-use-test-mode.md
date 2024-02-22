# Use virtual machine for testing

This mode is used to test a distribution of OpenDuT. 

![OpenDuT-VM](..%2F..%2F..%2F..%2Fresources%2Fdiagrams%2Fopendut-vm-user.drawio.svg)


* Ensure a distribution of openDuT is present
    * By either creating one yourself on your **host**:
      ```shell
      cargo ci distribution
      ```
    * Or in the **opendut-vm**. 
      Within the VM the rust target directory is overridden to `/home/vagrant/rust-target`.
      Therefore, you need the to copy the created distribution to the expected location.
        ```shell
        cargo ci distribution
        mkdir -p /vagrant/target/ci/distribution/x86_64-unknown-linux-gnu/
        cp ~/rust-target/ci/distribution/x86_64-unknown-linux-gnu/* /vagrant/target/ci/distribution/x86_64-unknown-linux-gnu/
        ```
    * Or by copying one to the target directory `target/ci/distribution/x86_64-unknown-linux-gnu/`
      ```sh
      # ensure directory is present
      mkdir -p target/ci/distribution/x86_64-unknown-linux-gnu/
      # copy distribution to target directory
      ```

* Login to the virtual machine from your **host** (assumes you have already set up the virtual machine)
  ```shell
  cargo theo vagrant ssh
  ```

* Start test environment in **opendut-vm**:
  ```shell
  cargo theo testenv start
  ```

* Start a cluster in **opendut-vm**:
  ```shell
  cargo theo testenv edgar start
  ```
  This will start several EDGAR containers and create an OpenDuT cluster. 
