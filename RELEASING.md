To make a release of one or more Golang modules in the OTel Arrow
repository, follow these steps.

1. Using Git, checkout the version of the repo that will be released
   and create a new branch for the release, for example,

```
git checkout main
git pull upstream main
git checkout -b release_xx_yy_zz
```

2. Make sure the CHANGELOG.md file is up to date, add entries 
   describing the changes in the new release.

3. Using Make, prepare the release means updating Go modules and
   checking in the changes, for example.  Edit 

```
make prepare-release
```

4. Push the branch and open a PR to submit these changes to the
   upstream repository's main branch.

5. After merging the PR, pull the upstream commit, for example,

```
git checkout main
git pull upstream main
```

6. Push the release, for example,

```
make push-release
```

The release has been published.  Note that these instructions do not
cover the use of multiple module sets, since this repository uses a
single module set named "beta" at this time.
