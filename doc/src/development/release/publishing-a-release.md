# Publishing a release

This is a checklist for the steps to take to create a release for public usage.

* [ ] Increment version number in workspace `Cargo.toml`
* [ ] Run `cargo ci check` to update all `Cargo.lock` files
* [ ] Update top-most changelog heading from "Unreleased" to the new version number
* [ ] Increment the version the CARL container used in CI/CD deployments
* [ ] Create commit and push to `development`
* [ ] Open PR from `development` to `main`
* [ ] Merge PR once its checks have succeeded
* [ ] Tag the commit with the respective version number in the format "v1.2.3" and push the tag


## After the release

* [ ] Increment version number in workspace `Cargo.toml` to development version, e.g. "1.2.3-alpha"
* [ ] Run `cargo ci check` to update all `Cargo.lock` files
* [ ] Add a new heading "Unreleased" to the changelog with contents "tbd."
* [ ] Create commit and push to `development`
