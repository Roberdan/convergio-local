# Convergio — English message bundle.
# Fluent syntax: https://projectfluent.org/fluent/guide/

# ---------- generic ----------
ok = OK
not-found = Not found
internal-error = Internal error

# ---------- daemon ----------
daemon-starting = Starting Convergio daemon at { $url }
daemon-listening = Listening on { $bind }
daemon-version = Convergio { $version }

# ---------- CLI: health ----------
health-ok = Daemon is healthy. Version: { $version }
health-unreachable = Could not reach daemon at { $url }: { $reason }

# ---------- CLI: status ----------
status-header = Convergio status
status-active-header = Active plans:
status-active-empty = No active plans.
status-completed-header = Recently completed plans:
status-completed-empty = No completed plans yet.
status-tasks-header = Recently completed tasks:
status-tasks-empty = No completed tasks yet.
status-plan-line = - { $title } [{ $status }] project: { $project } tasks: { $done }/{ $total } done
status-work-line =   does: { $work }
status-next-line =   next: { $tasks }
status-task-line = - { $title } in { $plan } project: { $project }

# ---------- CLI: CRDT ----------
crdt-conflicts-empty = No unresolved CRDT conflicts.
crdt-conflicts-header = Unresolved CRDT conflicts:
crdt-conflict-line = - { $entity }/{ $id } field { $field } type { $type }

# ---------- CLI: workspace ----------
workspace-leases-empty = No active workspace leases.
workspace-leases-header = Active workspace leases:
workspace-lease-line = - { $agent } holds { $kind } { $path } until { $expires }

# ---------- CLI: capabilities ----------
capabilities-empty = No local capabilities registered.
capabilities-header = Local capabilities:
capability-line = - { $name } { $version } [{ $status }]
capability-signature-ok = Capability signature verified for { $name } { $version } with key { $key }
capability-installed = Capability installed: { $name } { $version } [{ $status }]
capability-disabled = Capability disabled: { $name } { $version }

# ---------- CLI: setup / doctor ----------
setup-config-created = Config created: { $path }
setup-config-exists = Config already exists: { $path }
setup-config-backed-up = Existing config backed up: { $path }
setup-complete = Setup complete: { $path }
setup-next-start = Next: start the daemon with `convergio start`
setup-next-doctor = Then: run `cvg doctor`
setup-agent-created = Adapter snippets created for { $host }: { $path }
setup-agent-copy = Copy mcp.json into the agent host MCP configuration and prompt.txt into its instructions.
doctor-header = Convergio doctor for { $url }
doctor-ok = OK { $name }: { $message }
doctor-warn = WARN { $name }: { $message }
doctor-fail = FAIL { $name }: { $message }
doctor-summary-ok = Doctor passed.
doctor-summary-fail = Doctor found failing checks.
mcp-log-missing = No MCP log found yet.
service-installed = Service file written: { $path }
service-started = Service started.
service-stopped = Service stopped.
service-status-loaded = Service is loaded.
service-status-not-loaded = Service is not loaded.
service-uninstalled = Service uninstalled.

# ---------- CLI: plan ----------
plan-created = Plan created: { $id }
plan-not-found = Plan not found: { $id }
plan-list-empty = No plans yet.
plan-list-header = { $count ->
    [one] One plan:
   *[other] { $count } plans:
}

# ---------- gate refusals (human side) ----------
# The `code` field stays English (it's an API contract).
# The `message` is what the human reads.
gate-refused-evidence = Missing evidence: { $kinds }
gate-refused-no-debt = Technical debt found in evidence: { $markers }
gate-refused-no-stub = Scaffolding markers found in evidence: { $markers }
gate-refused-zero-warnings = Build/lint signal is not clean: { $signals }
gate-refused-plan-status = Plan is { $status }; cannot accept new transitions
gate-refused-wave-sequence = { $count ->
    [one] One earlier-wave task is still open
   *[other] { $count } earlier-wave tasks are still open
}

# ---------- audit ----------
audit-clean = Audit chain verified: { $count } events, no tampering detected.
audit-broken = Audit chain broken at sequence { $seq }.

# ---------- CLI: pr stack ----------
pr-stack-empty = No open PRs.
pr-stack-header = { $count ->
    [one] One open PR:
   *[other] { $count } open PRs:
}
pr-stack-no-manifest = no Files-touched manifest
pr-stack-manifest-mismatch = manifest does not match diff
pr-stack-files-summary = { $count ->
    [one] one file
   *[other] { $count } files
}
pr-stack-suggested-order = Suggested merge order:

# ---------- CLI: session resume ----------
session-resume-header = Convergio session resume
session-resume-health-ok = Daemon: ok (version { $version })
session-resume-health-down = Daemon: NOT ok (version { $version })
session-resume-audit-ok = Audit chain: ok ({ $count } events)
session-resume-audit-broken = Audit chain: BROKEN ({ $count } events checked)
session-resume-plan-line = Plan: { $title } [{ $status }] project: { $project } id: { $id }
session-resume-counts-line = Tasks: { $done }/{ $total } done — in_progress: { $in_progress }, submitted: { $submitted }, pending: { $pending }
session-resume-next-empty = Next priority: none (no pending tasks).
session-resume-next-header = Next priority (top pending):
session-resume-next-line =   - w{ $wave }.{ $sequence } { $title } [{ $id }]
session-resume-prs-empty = Open PRs: none.
session-resume-prs-unavailable = Open PRs: gh not available (skipped).
session-resume-prs-header = Open PRs:
session-resume-pr-line =   - #{ $number } { $title } ({ $branch })
session-resume-pr-line-draft =   - #{ $number } [draft] { $title } ({ $branch })
session-resume-pack-line = Context-pack for task { $task_id }: { $nodes } matched nodes, { $files } files, ~{ $est_tokens } tokens
