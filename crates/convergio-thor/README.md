# convergio-thor

Layer 4 reference validator.

`Thor::validate` reads a local plan and returns `Pass` only when every
task is `done` and every required evidence kind is present. Domain
validators can replace it while keeping the same local runtime.
