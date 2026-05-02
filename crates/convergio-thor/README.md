# convergio-thor

Layer 4 reference validator.

`Thor::validate` reads a local plan and returns `Pass` only when every
task is `submitted` or `done` and every required evidence kind is
present. On `Pass`, Thor promotes submitted tasks to `done` through the
ADR-0011 validator-only path. Domain validators can replace it while
keeping the same local runtime.

## Smart pipeline command

When `CONVERGIO_THOR_PIPELINE_CMD` is set, Thor runs that command
before promoting `submitted` tasks to `done`. This variable is
trusted-local configuration only: the value is executed through
`sh -c` with the daemon user's privileges and must never be copied from
plans, evidence, agent output, HTTP requests, or other untrusted input.

The command has a bounded default timeout of 600 seconds. Failed
pipeline output is bounded to the last 4096 bytes of combined stdout
and stderr; Thor includes a truncation marker when earlier output was
cut.
