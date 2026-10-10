<!-- markdownlint-disable MD013 -->

# Azure Key Vault Secrets SDK provenance

This is the MIT-licensed `azure_security_keyvault_secrets` **1.1.0-beta.1**
source from the official [Azure SDK for Rust revision
e1f93542648359690c640935f271f49a66dccc25][revision].
Microsoft copyright headers and the upstream `LICENSE.txt` are unchanged.
The crate is outside the OTAP Cargo workspace and is not an OpenTelemetry
component. It is consumed only through the optional Key Vault feature.

## Why this source is pinned

The released Secrets SDK requires stable Azure Core, which cannot share the
existing preview credential types. Downgrading the existing identity/core
dependencies would remove Azure Arc support and standard-runtime cancellation
fixes. This source revision matches the published Core `1.2.0-beta.1`,
Identity `1.1.0-beta.1`, and TypeSpec/client-core preview family already used
by the project.

The official Secrets manifest implicitly enables Core defaults even when
the caller disables Secrets defaults. The only normal-dependency correction is:

```diff
-azure_core = { path = "../../core/azure_core", version = "1.2.0-beta.1" }
+azure_core = { version = "1.2.0-beta.1", default-features = false }
```

Removing the SDK-local path selects the original published Core rather than
a second source of its public types. Disabling defaults preserves the caller's
explicit transport and crypto-provider selection. The SDK's own `default`
feature remains unchanged.

## Packaging and source identity

All 14 Rust source files are byte-identical to the revision above. The upstream
`src/authorizer.rs` symlink is materialized from
`sdk/keyvault/azure_security_keyvault_keys/src/authorizer.rs` at the same revision,
so Windows and Unix builds use the same official authorization implementation.
The required upstream README is retained because `src/lib.rs` includes it.

`Cargo.toml` makes the original workspace package metadata and normal/build
dependency requirements/features explicit, marks the local package non-publishable,
and gives it an isolated workspace. Upstream developer tests, examples, benchmarks,
deployment resources, workspace lints, and their private development dependencies
are not packaged. No runtime, authentication, generated client, or model source
has been edited. Embedded upstream unit tests remain part of the unchanged source;
they are not members of the OTAP test suite.

`UPSTREAM_BLOBS.txt` records Git blob IDs for every retained upstream source,
README, and license. Verify each entry using:

```bash
git hash-object --no-filters <path>
```

These IDs hash exact bytes; Git checkout line-ending conversion must not change
the retained files. The source tree is approximately 3,000 lines, including
embedded upstream tests; those are distinct from first-party extension tests.

## Updating or removing the pin

Prefer removing this directory when an official released Secrets SDK is compatible
with the project's Azure family and does not implicitly select crypto/runtime
defaults. Change the workspace dependency to that release and regenerate the
lockfile; do not downgrade existing identity behavior to accommodate it.

If an interim update is necessary, choose an immutable official SDK revision,
copy its source and required README/license, materialize any source symlinks from
that revision, flatten metadata without inheriting OpenTelemetry's authors/license,
and review any changed runtime/authentication code. Update the blob list and this
provenance document. Do not edit generated or authentication source locally.

Before adoption, compare the actual normal/feature graph with the previous Azure
configuration, retain every `crypto-*` backend, and verify Arc identity,
cancellation, native credential type interoperability, challenge validation,
redaction, startup acquisition, and no secret re-fetch through offline tests.
Run the targeted extension/identity suites, aggregate tests, strict Clippy,
root feature builds, and the required full `cargo xtask check`.

[revision]: https://github.com/Azure/azure-sdk-for-rust/tree/e1f93542648359690c640935f271f49a66dccc25/sdk/keyvault/azure_security_keyvault_secrets
