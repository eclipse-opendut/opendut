# Secrets for test environment

This repository contains secrets for testing purposes. 
These secrets are not supposed to be used in a production environment.
There are two formats defined in the repository that document their location:
* ~/.gitguardian.yml
* .secretscanner-false-positives.json

Alternative strategy to avoid this:
auto-generate secrets during test environment setup.

## GitGuardian

### Getting started with ggshield

* Install ggshield
    ```shell
    sudo apt install -y python3-pip
    pip install ggshield
    export PATH=~/.local/bin/:$PATH
    ```
* Login to https://dashboard.gitguardian.com
* Either use PAT or service account (https://docs.gitguardian.com/api-docs/service-accounts)
* Goto API -> Personal access tokens
    * and create a token
* Use API token to login: `ggshield auth login --method token`

### Scan repository

* See https://docs.gitguardian.com/ggshield-docs/getting-started
* Scan repo
    ```shell
    ggshield secret scan repo ./
    ```
* Ignore secrets found in last run and remove them or document them in `.gitguardian.yml`
    ```shell
    ggshield secret ignore --last-found
    ```

* Review changes in `.gitguardian.yml` and commit

