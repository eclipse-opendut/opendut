# Test Execution

In a nutshell, test execution in openDuT works by executing containerized (Docker or Podman) test applications on a peer and uploading the results to a WebDAV directory. Test executors can be configured through either CLEO or LEA.

The container image specified by the `image` parameter in the test executor configuration can either be a container image already present on the peer or an image remotely available, e.g., in the Docker Hub.

A containerized test application is expected to move all test results to be uploaded to the `/results/` directory within its container and create an empty file `/results/.results_ready` when all results have been copied there. When this file exists, or when the container exits and no results have been uploaded yet, EDGAR creates a ZIP archive from the contents of the `/results` directory and uploads it to the WebDAV server specified by the `results-url` parameter in the test executor configuration.

In the `testenv` launched by THEO, a WebDAV server is started automatically and can be reached at `http://nginx-webdav/`. In the [Local Test Environment](https://github.com/eclipse-opendut/opendut/tree/development/.ci/deploy/localenv), a WebDAV server is also started automatically and reachable at `http://nginx-webdav.opendut.local`.

Note that the execution of executors is only triggered by deploying the cluster.

## Test Execution using CLEO
In CLEO, test executors can be configured either by passing all configuration parameters as command line arguments or by providing a JSON-formatted configuration file.

    $ opendut-cleo create container-executor --help                                                            
    Create a container executor. Parameters can either be provided as individual arguments or by means of a JSON-formatted configuration file

    Usage: opendut-cleo create container-executor [OPTIONS]

    Options:
        --peer-id <PEER_ID>          ID of the peer to add the container executor to
    -e, --engine <ENGINE>            Engine [possible values: docker, podman]
    -n, --name <NAME>                Container name
    -i, --image <IMAGE>              Container image
    -v, --volumes <VOLUMES>...       Container volumes
        --devices <DEVICES>...       Container devices
        --envs <ENVS>...             Container envs
    -p, --ports <PORTS>...           Container ports
    -c, --command <COMMAND>          Container command
    -a, --args <ARGS>...             Container arguments
    -r, --results-url <RESULTS_URL>  URL to which results will be uploaded
        --config-file <CONFIG_FILE>  Path to the JSON-formatted executor configuration file
    -h, --help                       Print help

Note that the `volumes`, `devices`, and `ports` arguments are currently unused and will not be considered during test execution.

### Configuration File Format
A JSON configuration file for a test executor that is passed to CLEO may look as follows.

```json
{
    "peer-id": "26ada545-e834-4af3-8b66-af860ad19dbe",
    "container": {
        "engine": "docker",
        "name": "nmap-test",
        "image": "nmap-test",
        "results-url": "http://nginx-webdav:80/",
        "envs": {
            "ENVVAR1": "someenvironmentvariable",
            "ENVVAR2": "anotherenvironmentvariable"
        },
        "args": [
            "-A",
            "-T4",
            "127.0.0.1"
        ],
        "ports":  [],
        "volumes": [],
        "command": "",
        "devices": []
    }
}
``` 

## Test Execution Through LEA
In LEA, executors can be configured via the tab `Executor` during peer configuration, using similar parameters as for CLEO.
