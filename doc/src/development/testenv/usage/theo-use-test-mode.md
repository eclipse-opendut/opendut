# Use virtual machine for testing

This mode is used to test a distribution of OpenDuT. 

![OpenDuT-VM](../img/opendut-vm-user.drawio.svg)

* Ensure a distribution of openDuT is present
    * By either creating one yourself on your **host**:
      ```shell
      cargo ci distribution
      ```
    * Or by downloading a release and copying to the target directory `target/ci/distribution/x86_64-unknown-linux-gnu/`

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
