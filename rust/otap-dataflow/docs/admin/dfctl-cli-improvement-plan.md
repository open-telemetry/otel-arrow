# dfctl CLI Improvement Plan

This document tracks CLI best-practice improvements that are useful but not
required for the initial `dfctl` operator workflow.

## Recommended Next Items

1. Add `dfctl doctor` and `dfctl capabilities`.
   Validate target reachability, API compatibility, TLS configuration, retained
   log availability, metrics availability, and admin API feature support.

2. Add explicit mutation preflight support.
   Start with `--dry-run` for commands that can validate local input without
   changing engine state. Extend later when the admin API exposes server-side
   validation.

3. Add completion installation helpers.
   Keep `dfctl completions <shell>` as the primitive, then add documented
   install snippets or a `dfctl completions install <shell>` helper.

4. Add consistent large-output shaping.
   Extend list and table commands with common `--filter`, `--sort-by`,
   `--columns`, and `--limit` conventions where the admin API or local snapshot
   data makes this cheap and deterministic.

5. Add configurable redaction controls.
   Introduce `--redact`, `--no-redact`, or a redaction policy for support
   bundles and machine-readable output that may include environment-specific
   names, headers, paths, or configuration values.

6. Add a machine-readable command catalog.
   Provide `dfctl commands --output json` or generated metadata from the clap
   command tree so AI agents can discover commands, arguments, output modes,
   and examples without scraping help text.

## Later Options

- Add default profile discovery with explicit precedence, for example
  `$XDG_CONFIG_HOME/dfctl/config.yaml` and `~/.config/dfctl/config.yaml`.
- Generate man pages or a full command reference from the clap command tree.
- Add more precise HTTP tracing once the SDK exposes safe request diagnostics.
- Review legacy aliases and decide whether they should remain hidden,
  documented, or deprecated.
