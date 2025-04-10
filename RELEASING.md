# Releasing

To make a release of one or more Golang modules in the OTel Arrow repository,
follow these steps.

1. Using Git, checkout the version of the repo that will be released and create
   a new branch for the release, for example,

```shell
git checkout main
git pull upstream main
git checkout -b release_xx_yy_zz
```

1. Update OTel-Collector dependencies.  It will frequently be necessary to
   update all v0.x OTel-Collector dependencies.  The steps are:

   a. For each go.mod in the repository, run a command like

   ```shell
   for x in `grep OLDVERSION go.mod | awk '{print $1}'`; do go get $x@NEWVERSION; done
   ```

   b. Run `make test`; fix any build breakage.  If there have been refactorings
      in the Collector APIs, there are likely to be small changes required.

   c. Edit `Makefile`, replace the string
      `go.opentelemetry.io/collector/cmd/builder@OLDVERSION` with
      `go.opentelemetry.io/collector/cmd/builder@NEWVERSION`.

   d. Run `make genotelarrowcol`

   e. Run `make otelarrowcol`

1. Make sure the CHANGELOG.md file is up to date, add entries describing the
   changes in the new release.  If collector dependencies have changed during
   this release cycle, `make genotelarrowcol` should have been run to
   synchronize dependencies.

1. Using Make, prepare the release means updating Go modules and checking in the
   changes, for example.

   ```shell
   make prepare-release
   ```

1. Push the branch and open a PR to submit these changes to the upstream
   repository's main branch.

1. After merging the PR, pull the upstream commit, for example,

   ```shell
   git checkout main
   git pull upstream main
   ```

1. Push the release, for example,

   ```shell
   make push-release
   ```

1. The tag has now been published, but need to manually create the release in
   the Github UI. Go to
   [releases](https://github.com/open-telemetry/otel-arrow/releases) and click
   `Draft a new release`. Select the tag you pushed in step 6 and link the
   Changelog in the description, following the convention of previous releases.
   TODO: automate this step to trigger whenever a new release tag is pushed.

The release has been published.  Note that these instructions do not cover the
use of multiple module sets, since this repository uses a single module set
named "beta" at this time.

1. Re-enable CI/CD by reverting the change to `.github/workflows/ci.yml`.
