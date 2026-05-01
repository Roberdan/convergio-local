# Adversarial challenge prompt template (v1)

> Versioned prompt template for the adversarial-review service
> documented in [ADR-0022](../adr/0022-adversarial-review-service.md).

## When to invoke

Required (advisory now, gating in Wave 2) for:

- new ADRs at tier ≥ 2 (touching CONSTITUTION, data schema,
  public API, or cross-crate contracts)
- changes to `docs/vision.md`, `ROADMAP.md`, `CONSTITUTION.md`
- new PRDs
- changes to capability namespacing (ADR-0018)

## How to invoke

### Today (manual fallback, pre-Wave-2)

Spawn an independent agent (Claude general-purpose, OpenAI Codex
CLI in `challenge` mode via gstack `/codex challenge`, or any
sibling-session agent) with the prompt below, providing the file
list under review and the target branch / commit.

### Wave 2 (automated)

```
convergio.act {
  "type": "governance.adversarial_review",
  "params": {
    "files": ["docs/vision.md", "docs/adr/0016-…md", …],
    "template_version": "v1",
    "budget_usd": 0.50,
    "min_findings_per_category": 1
  }
}
```

The capability bundle resolves the runner adapter via ADR-0020
model evaluation, runs the prompt below against the file set,
and attaches structured findings as `doc-review` evidence on the
originating plan.

## Prompt

```
You are an outside-voice reviewer for Convergio strategic
documents. The author has explicitly requested adversarial
challenge, not cheerleading. Find real problems. Find
contradictions. Find politically risky framings. Find promises
that cannot be kept. Find metaphors that break under pressure.

CONTEXT: Convergio is a Rust HTTP daemon that does runtime
enforcement for AI agents (server-side gates, hash-chained audit,
CRDT, Ed25519-signed capability bundles). The author works in a
context where this work overlaps with adjacent corporate
projects; political framing matters.

READ THESE FILES (paths are repository-relative):
{{file_list}}

OUTPUT in {{language|default=italiano}}, max 1500 words,
structure:

### A) Internal contradictions (top 5)
Where doc X says one thing and doc Y says another. Cite
file:line. Which prevails?

### B) Unsustainable promises (top 5)
Claims the codebase cannot back today and that would take months
to make true. ADRs may legitimately describe future work, but if
something is stated as "this is enforced" and is not, that is a
lie that must be removed or reframed.

### C) Political / social / legal risks (top 3)
Phrases that could trigger problems with adjacent maintainers,
employer IP review, OSS community norms, or accidentally imply
endorsements that do not exist. Be specific about who would
react and how.

### D) Metaphors that break (top 3)
The doc set uses an extended urbanism metaphor (Le Corbusier,
Jane Jacobs, Comune italiano, Design Week, Modulor). Where does
the metaphor become marketing speak? Where does a
technical-pragmatic reader think "OK, but what does this
actually mean"?

### E) Roadmap gaps (top 3)
Are the wave dependencies coherent? Are timelines realistic given
single-developer feasibility? Are deliverables defined precisely
enough?

### F) Technical errors (top 5)
Endpoint names, schema, ADR cross-references, command-line
syntax, type signatures that do NOT match the actual codebase.
Verify by `gh search`, `find`, or `grep` in the repository
before listing.

### G) Verdict
"Ship now" — list at most 5 fixes that must happen before
commit, ranked.
"Not now" — list what must change to reach "ship now".

Be brutal where it helps the author. The author wants a real
challenge, not an OK.
```

## Versioning

- v1 — 2026-05-01, initial template, used for Wave 0a review
  (24 findings).
- v2+ — every revision is a small ADR, tracked.

When the template changes, the version string passed to the
`governance.adversarial_review` action changes. Reviews are
comparable across years only within the same `template_version`.

## Output format expectations

- Findings are addressable by `category-letter + integer` (e.g.
  `C1`, `F3`).
- Each finding cites at least one file path and, where relevant,
  line numbers.
- Each finding has an explicit severity-implied stance ("fix
  before merge" or "deferred with note" or "wont-fix with
  rationale").

## Anti-patterns (from Wave 0a dogfood)

- *Theatrical compliance*: running the review and ignoring
  findings ("we ran it, fine") — Wave 2 gating addresses this;
  pre-Wave-2 the PR template requires explicit response to each
  finding.
- *Over-citation*: findings that depend on the reviewer
  hallucinating issue numbers in adjacent repos. Sanitise
  political-risk findings to remove unverified specifics.
- *Conflict between findings and fixes*: when fix #N for finding
  #X creates finding #Y, the next adversarial-review pass should
  catch it. The practice is iterative.
