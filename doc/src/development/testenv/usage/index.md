## Start testing

Once you have set up and started the test environment, you may start testing the services.

### User interface

The **OpenDuT Browser** is a web browser running in a docker container.
It is based on KasmVNC base image which allows containerized desktop applications from a web browser.
A port forwarding is in place to access the browser from your host.
It has all the necessary certificates pre-installed and is running in headless mode.
You may use this **OpenDuT Browser** to access the services.

* Open following address in your browser: http://localhost:3000
* Passwords of users in test environment are generated.
  You can find them in the file `.ci/deploy/localenv/data/secrets/.env`.
* Services with user interface:
    * https://carl.opendut.local
    * https://netbird.opendut.local
    * https://auth.opendut.local
    * https://monitoring.opendut.local
