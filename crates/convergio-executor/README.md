# convergio-executor

Layer 4 reference dispatcher.

`Executor::tick` finds pending tasks whose wave is ready, spawns a local
worker through `convergio-lifecycle`, and moves each task to
`in_progress` with the spawned process id as `agent_id`.
