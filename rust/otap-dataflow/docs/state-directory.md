# Engine state directory

`engine.state_dir` optionally selects a stable, absolute directory for engine
state. There is no default. Engines without this setting retain their existing
behavior; a future component that requires the directory must reject its absence.
Provisioning is supported only on Linux. Configuring it on another platform
fails startup with an unsupported-platform error.

```yaml
version: otel_dataflow/v1
engine:
  state_dir: /var/lib/otel-arrow
```

The normal engine configuration loader supports `${env:NAME}` substitution
before validation. The resulting value must satisfy the same path requirements.
There is no implicit `OTAP_DF_STATE_DIR` lookup, component-specific lookup, or
working-directory fallback. Direct construction of configuration objects does
not expand placeholders.

## Path and access requirements

Use an explicit absolute path below `/`. Empty paths, `/` itself, relative paths,
NUL bytes, unresolved `${...}` placeholders, `.` or `..` components, repeated
separators, and trailing separators are rejected. Literal `${` in directory
names is also rejected. Paths are not canonicalized or normalized to bypass
component checks. Filesystem identity retains native path bytes; diagnostic
rendering is not used to open objects.

Provisioning starts at `/` and walks one component at a time through open
directory descriptors. Each open requires a directory and refuses symlinks.
Ownership, permissions, and type are checked on the opened object:

- Ancestors must belong to root or the collector's effective UID, with no group
  or other write permission. Writable shared directories such as `/tmp` are
  rejected even when they have the sticky bit. The collector needs read and
  search access through every ancestor.
- The final root must belong to the collector's effective UID and have private
  owner read, write, and search permissions (0700).
- Missing directories are requested with mode 0700. Newly created intermediate
  directories and the final root are checked for the resulting 0700 permissions.
  Provisioning does not change existing permissions, ownership, or the process
  umask. A restrictive umask such as 0100 can produce an unusable 0600 directory
  and fail startup; 0077 leaves a requested 0700 unchanged.

For a new, absent root beneath trusted ancestors, provision it for the intended
service user and group (here both named `otel-arrow`):

```sh
sudo install -d -o otel-arrow -g otel-arrow -m 0700 /var/lib/otel-arrow
```

Use this example for a new root, not to repair existing checkpoint state.
Startup still validates the directory and performs the required syncs.

The access policy covers Linux POSIX ACL semantics: when an extended access ACL
exists, the group mode bits represent its ACL mask. Disallowing group write also
limits effective named-user/group write grants. The root's zero group bits mask
all such grants. There is no ACL marker bit in `st_mode`. Inherited default ACLs
can restrict the requested owner permissions, so the actual result is validated.
This is not a claim of support for arbitrary ACL models. See the Linux
[ACL documentation](https://man7.org/linux/man-pages/man5/acl.5.html).

Handle-relative traversal prevents pathname re-resolution from redirecting later
operations through already opened ancestors. Provisioning rejects observed
entry substitutions. It does not freeze permissions, prevent renames, or defend
against privileged actors or another process using the trusted collector UID.

## Startup, durability, and lifecycle

The controller provisions the directory synchronously before starting pipelines
or controller extensions. Starting at the filesystem root, it synchronizes the
anchor and every parent directory after opening or creating the next component,
including existing entries. It synchronizes the final root before handoff.
Existence alone does not establish durability after interrupted provisioning.
Directory synchronization is required to persist child entries; see
[`fsync(2)`](https://man7.org/linux/man-pages/man2/fsync.2.html).

Use trusted host-local storage that implements Linux POSIX permissions and
required directory synchronization. NFS, CIFS, and FUSE deployments have not
been qualified. All required sync errors, including unsupported operations,
fail startup with operation, path, and OS error context.
Successful system calls still depend on the filesystem, mount, and storage
stack honoring their durability contracts. Fault-injection tests establish
failure ordering, not physical power-loss qualification.

Failures leave partial directories and existing state intact. Correct the
reported problem and retry startup at the same configured location; validation
and all durability barriers repeat. There is no alternate-directory fallback,
automatic checkpoint reset, or deletion of existing state.

The root is immutable for the engine lifetime. Pipeline restarts, resizing, and
live reconciliation share the same capability. Live addition, removal, and
replacement are rejected, including updates delivered through OpAMP. Selecting
a different root requires an engine restart and an explicit operator decision:
it selects different state and does not relocate existing checkpoints.

## Consumer adoption

`ControllerContext::state_directory()` and `PipelineContext::state_directory()`
return an optional, cheaply cloned `StateDirectory`. On Linux its `AsFd`
implementation supplies the validated directory descriptor for relative
operations. The retained path is diagnostic only. Consumers must validate their
relative names and descendant objects and establish their own namespace locking,
recovery, and durability rules. They must not reopen the root by pathname or
reprovision it when a live root disappears. No exclusive engine-root lock is
held; multiple processes with the same service identity may share it.

This facility does not register Filelog or implement checkpoints. Journald still
uses its existing `${engine.state_dir}` text expansion, `OTAP_DF_STATE_DIR` lookup,
and `.otap-state` fallback. Database-scraper checkpoint storage with that legacy
expansion also requires separate adoption work. Adding the engine setting does
not migrate either consumer or relocate its checkpoints.
