<!-- markdownlint-disable MD013 -->

# Kubernetes Service Account Token Extension

**Status:** Draft

| | |
| --- | --- |
| **URN** | `urn:otel:extension:k8s_service_account_token` |
| **Feature gate** | `k8s-service-account-token-extension` |
| **Capability** | `bearer_token_provider` |
| **Execution model** | Active + Shared |

Reads the pod's Kubernetes service account token from its mounted file and
exposes it to data-path nodes through the `BearerTokenProvider` capability, so
nodes never read the token file or manage rotation themselves. The token is
re-read and re-published as kubelet rotates it.

## Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `token_file_path` | path | `/var/run/secrets/kubernetes.io/serviceaccount/token` | Path to the mounted service account token. Point this at a projected token with a specific audience when using one. |
| `startup_timeout` | duration | `15s` | How long pipeline startup waits for the first token to be published before aborting. |

## Supported modes

The token is always consumed from a **mounted file** — the standard, zero-RBAC
source for a workload's own service account token. Two file layouts are
supported, distinguished automatically by whether the token's JWT carries an
`exp` claim:

- **Projected volume (default).** Short-lived, audience-scoped token that kubelet
  auto-rotates in place (at ~80% of its lifetime). Has `exp`.
- **Legacy / static secret volume.** A non-expiring token mounted from a
  `kubernetes.io/service-account-token` Secret. Has no `exp`.

The `TokenSource` trait keeps refresh mechanics per-mode, so API-based sources
(the `TokenRequest` API, or reading a `Secret` via the Kubernetes API) can be
added later without changing the refresh loop. These are not implemented, as
they require a Kubernetes API client and additional RBAC.

## Refresh logic

kubelet rotates the projected token by **atomically swapping a symlink** in the
mount directory (it does not overwrite the file in place), so refresh is
**event-driven**:

- **Primary — directory watch.** The extension watches the token's mount
  *directory* (not the file, whose resolved inode does not change on a swap) and
  re-reads on any change. This is exactly the "a new token exists" signal, so
  each re-read pulls a genuinely new token rather than polling blindly.
- **Backstop — `exp`-derived timer.** A filesystem watch can miss events
  (e.g. inotify queue overflow), so a timer fires shortly before `exp` purely as
  a safety net; in normal operation the watch fires first and reschedules it.
- **No `exp` (static token).** Nothing rotates, so there is no timer — the watch
  alone catches a rare manual secret rotation.

A cached token is treated as unusable within a short margin of `exp`, forcing an
on-demand re-read, so an expired token is never served even if both the watch
and the backstop are late.
