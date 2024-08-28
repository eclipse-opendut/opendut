# Deployment of CARL in Kubernetes

This is an example on how to use the helm chart for [OpenDuT-CARL](https://github.com/eclipse-opendut/opendut/pkgs/container/opendut-chart-carl).
There are several additional requirements to be met before this actually works.
At the moment it is assumed there is already a deployment existing for:
* NetBird
* KeyCloak

## Usage

The following steps will install CARL into its own namespace `opendut`: 
```shell
kubectl create ns opendut
helm dependency build
helm -n opendut install opendut-carl .
```
