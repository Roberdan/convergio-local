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

# ---------- CLI: setup / doctor ----------
setup-config-created = Config created: { $path }
setup-config-exists = Config already exists: { $path }
setup-config-backed-up = Existing config backed up: { $path }
setup-complete = Setup complete: { $path }
setup-next-start = Next: start the daemon with `convergio start`
setup-next-doctor = Then: run `cvg doctor`
doctor-header = Convergio doctor for { $url }
doctor-ok = OK { $name }: { $message }
doctor-warn = WARN { $name }: { $message }
doctor-fail = FAIL { $name }: { $message }
doctor-summary-ok = Doctor passed.
doctor-summary-fail = Doctor found failing checks.

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
