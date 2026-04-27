# Convergio v3 — Durability Layer Specs

Generato da /office-hours il 2026-04-26 (sessione 2/2)
Branch: main
Status: DRAFT — supersedes `2026-04-26-roberto-main-design-positioning.md` (positioning section). Le verifiche tecniche del doc precedente restano valide.
Mode: Builder + reframe strutturale

---

## Vision (in una frase)

> **Convergio è il "Postgres degli agent workflow": un layer di durabilità, comunicazione e lifecycle che qualunque framework agent (LangGraph, CrewAI, Claude Code skill, custom) può adottare per ottenere stato persistente, gate verificabili a macchina, audit trail immutabile, comunicazione inter-agent, e supervisione di agenti long-running. Funziona personal (SQLite locale) o team (PostgreSQL ostato). Non sostituisce il framework agent — gli dà la spina dorsale che gli manca.**

Non è "un altro orchestrator". È **il pezzo sotto** gli orchestrator esistenti.

---

## Principi non negoziabili

1. **Same binary, two modes.** Personal e team sono configurazione, non fork. Un singolo `convergio` binary che con `CONVERGIO_DB=sqlite://...` fa modalità personal, con `CONVERGIO_DB=postgres://...` fa modalità team. Punto.
2. **Cooperate, don't compete.** I framework agent (LangGraph, CrewAI, Claude Code skills) non sono concorrenti — sono client. La nostra value prop è "rendiamoli auditable e durable", non "sostituiamoli".
3. **Reference implementation = parte del prodotto.** Layer 1+2+3 da soli non vendono. Layer 4 (planner/Thor/executor di riferimento che USA layer 1+2+3) è quello che onboard l'utente in 5 minuti e prova che funziona.
4. **Anti-feature creep.** Tutto quello che non è in questi 4 layer aspetta. Mesh, skills-on-demand, modello organizzativo, billing, knowledge: rimandato.
5. **Ogni feature deve essere vendibile in un tweet.** Se la spiegazione richiede un diagramma, la feature non è pronta o non è la feature giusta.

---

## I 4 layer (in ordine di dipendenza)

### Layer 1 — Durability Core
Stato persistente + gate enforcement + audit log + heartbeat.

**Cosa è:**
- Tabella `plans` (id, org_id, status, created_at, ...)
- Tabella `tasks` (id, plan_id, wave, sequence, status, agent_id, evidence_required, ...)
- Tabella `evidence` (id, task_id, type, payload, exit_code, created_at)
- Tabella `agents` (id, kind, last_heartbeat, status)
- Tabella `audit_log` (id, entity_type, entity_id, transition_from, transition_to, agent_id, timestamp, hash_chain) — **append-only, hash-chained per non-ripudiabilità**
- Gate pipeline server-enforced: identity → plan_status → evidence → test → pr_commit → wave_sequence → validator
- Reaper loop: task in `in_progress` senza heartbeat per >N minuti → marcato `stale` → re-spawn dispatch

**Cosa NON è:**
- Non è un workflow engine (Temporal lo è — non vogliamo competere lì)
- Non è un message broker (Kafka lo è — vedi Layer 2 per il scope ridotto)
- Non sa cosa "deve fare" un agente — sa solo verificare che ha *finito*

**API minimale:**
```
POST   /v1/plans                    create plan
POST   /v1/plans/:id/tasks          add task with required evidence schema
POST   /v1/tasks/:id/transition     attempt state transition (gates enforce)
POST   /v1/tasks/:id/evidence       attach evidence
POST   /v1/tasks/:id/heartbeat      keep-alive
GET    /v1/plans/:id/audit          full hash-chained audit trail
GET    /v1/audit/verify             verify chain integrity (independent process)
```

**Modi:**
- **Personal**: `convergio start` → SQLite in `~/.convergio/state.db`, no auth, localhost-only, single-org implicit.
- **Team**: `convergio start --postgres $DATABASE_URL` → multi-org via `org_id`, HMAC auth headers per request, deployable in Docker / fly.io / qualunque container host.

**SLA che vendiamo:**
- "Lo stato sopravvive a crash, restart, OOM, context compaction, riavvio del modello, swap di vendor."
- "Audit trail hash-chained verificabile esternamente — FDA/HIPAA-grade."
- "Nessun task viene marcato done senza evidence che il daemon ha verificato."

---

### Layer 2 — Agent Communication Bus
Pub/sub minimale tra agenti dello stesso plan.

**Cosa è:**
- Tabella `agent_messages` (id, plan_id, from_agent, to_agent_or_topic, payload, created_at, consumed_at)
- Topic-based: agente pubblica su un topic, altri agenti dello stesso plan sottoscrivono
- Direct messaging: agente A manda a agente B con ack
- Persistente per default (sopravvive al crash di chi consuma)
- Scope: **solo dentro un plan**, non system-wide message broker

**Cosa NON è:**
- Non è Kafka (no partitioning, no millisecond throughput)
- Non è A2A protocol di Google (non parliamo cross-org/cross-vendor)
- Non è MCP (MCP è agent↔tool; questo è agent↔agent)
- Non sa parsare contenuto — passa bytes/JSON, è il client che interpreta

**API minimale:**
```
POST   /v1/plans/:id/messages       publish
GET    /v1/plans/:id/messages       poll/subscribe (long-poll o SSE)
POST   /v1/messages/:id/ack         consume confirmation
```

**Use case che giustifica l'esistenza:**
- Wave 2 task 3 (Thor) deve aspettare l'output di Wave 2 task 1 (writer) e task 2 (reviewer). Senza bus: polling sul DB → race conditions, inefficienza. Con bus: notifica push, consume ordinato.
- Long-running agente A scopre un'informazione che cambia il piano — pubblica `plan_invalidation` topic, l'executor la riceve e ri-pianifica.

**Quello che si vende:**
- "I tuoi agenti smettono di pollare. Pubblicano risultati, consumano notifiche, restano sincronizzati."

---

### Layer 3 — Agent Lifecycle
Spawn + supervise + reap di processi agente long-running.

**Cosa è:**
- Tabella `agent_processes` (id, kind, command, env, pid, started_at, status, exit_code)
- API per spawn: lancia un processo (Claude Code session, Python script, qualunque eseguibile) come task worker
- Supervisione: process monitor che intercetta exit, calcola se restart, aggiorna stato in DB
- Heartbeat client lib: `curl POST /v1/agents/:id/heartbeat` ogni 60s (lato agente)
- Reaper: task in_progress + heartbeat scaduto → release task back to queue, mark agent unhealthy

**Cosa NON è:**
- Non è systemd/launchd (non gestiamo servizi di sistema)
- Non è Kubernetes pod manager (no resource limits, no scheduling, no networking)
- Non è una sandbox (l'agente gira col privilege di chi lo lancia — l'isolamento è worktree/container del client)

**API minimale:**
```
POST   /v1/agents/spawn             { command, env, plan_id, task_id }
GET    /v1/agents/:id               status
POST   /v1/agents/:id/heartbeat     keep-alive
POST   /v1/agents/:id/terminate     graceful shutdown
```

**Use case che giustifica l'esistenza:**
- Plan da 6 ore con 12 task. Il primo agente parte, il context si esaurisce dopo 2 ore, lui muore. Senza Layer 3: workflow ripianta, bisogna ricominciare. Con Layer 3: il reaper se ne accorge in 60s, spawna un nuovo agente, gli passa il task in mano, l'audit log mostra "agent A morto, agent B subentrato a timestamp X".

**Quello che si vende:**
- "Il tuo workflow sopravvive alla morte dell'agente. Sempre."

---

### Layer 4 — Reference Implementation
Planner + Thor + Executor + worktree workflow di riferimento, che USANO layer 1+2+3 e fungono da quickstart per chiunque adotti.

**Perché esiste:**
Senza questo, la durability layer è "Postgres con API custom — costruiti sopra il vostro client". Nessuno fa il lavoro di costruire il client. Layer 4 è il client che dimostra come si fa, e che è abbastanza buono da usare in produzione *as-is* dal solo dev / piccolo team.

**Componenti:**
- **Solver/Planner** (`cvg solve`): prende una mission in linguaggio naturale, produce un plan strutturato in DB (waves, task, evidence required). Adversarial challenge gate.
- **Thor (Validator)**: agente separato che legge plan + task completati, valida verdetto pre-close. Vive come processo Layer 3.
- **Executor**: loop che pesca task ready, spawna agenti via Layer 3, dispatcha task. Cron unico (allinea col fatto che codice ha 1 cron, non 3 — fixiamo la doc).
- **Git worktree workflow**: ogni plan può lavorare in worktree isolato; layer 4 lo gestisce nativamente. Hook G1/G2 restano.
- **CLI `cvg`**: 5-10 comandi essenziali (plan create, plan tree, task complete, doctor run, agent spawn). NIENTE 130 MCP tool — quello viene dopo.

**API esterne (per chi adotta solo il backend, non layer 4):**
- Layer 4 è 100% costruito su HTTP API di Layer 1+2+3. Tu puoi cancellare layer 4 e scriverti il tuo planner sopra, e funziona uguale.

**Quello che si vende:**
- "Installalo e in 60 secondi hai un planner con audit trail, validator indipendente, e crash-resilience. Se non ti basta, scrivici il tuo sopra."

---

## Cosa cuttiamo (e perché)

| Crate / area attuale | Stato | Motivo |
|---|---|---|
| **Mesh multi-nodo** (convergio-mesh) | DEFER | Non lo usi tu, peer offline. Se serve, lo riprendiamo dopo che il primo customer lo chiede. |
| **Knowledge / catalog / org model** | CUT | Modello organizzativo è prematuro. Plan + task + evidence sono il modello primario. |
| **Billing** | CUT | OSS-puro per ora. Se diventa SaaS, billing è banale (Stripe). |
| **Kernel / MLX integration** | DEFER | Cool ma non differenzia il prodotto. Layer 1+2+3 è agnostico al modello — ogni client porta il suo. |
| **Night agents** | DEFER | Pattern interessante ma è già ottenibile con Layer 3 + cron esterno. Non MVP. |
| **130+ MCP tools** | RIDUCI a ~15 | Il 90% dei 130 tool sono surface area senza adopter. Tieni solo quelli che chiamano direttamente Layer 1+2+3+4 (plan, task, evidence, audit, agent spawn, message bus). Il resto si scopre dopo, se serve. |
| **Doctor 127 checks** | TIENI ma slim | Mantieni come integration test, ma il numero è marketing trivia. Concentralo su chaos + e2e core. |
| **Skills on demand** | DEFER | Aspetta finché un utente non lo chiede esplicitamente. Premature optimization. |

**Da 38 crate (workspace + git deps) a ~10-12 nell'MVP.** Numero approssimativo:

KEEP (core):
1. `convergio-db` — abstraction layer SQLite/Postgres (sqlx-based)
2. `convergio-orchestrator` (rinominare → `convergio-durability`) — gate pipeline + state machine
3. `convergio-bus` — Layer 2, agent message bus
4. `convergio-lifecycle` — Layer 3, agent spawn/supervise
5. `convergio-server` — HTTP routing shell
6. `convergio-cli` — `cvg` command
7. `convergio-planner` — Layer 4, solve/plan/adversarial
8. `convergio-thor` — Layer 4, validator agent
9. `convergio-executor` — Layer 4, task dispatch loop
10. `convergio-worktree` — git worktree integration

DEFER (rimettere quando serve):
- `convergio-mesh`, `convergio-doctor`, `convergio-mcp-server` (slim version possibile in MVP)

CUT (rimuovere dal workspace):
- knowledge, catalog, billing, kernel, night-agents, e tutto quello che non è in KEEP

---

## Modalità di deployment

### Personal mode

```bash
cargo install convergio
convergio start
# → SQLite in ~/.convergio/state.db
# → Listen on localhost:8420
# → No auth (localhost bypass)
```

Pensato per:
- Solo dev che vuole audit trail dei suoi agent run
- Esplorazione / valutazione prima di team adoption
- Fightthestroke.org tipo use case (small team, single host)

Vincoli accettabili:
- Single-org implicito (tutto in org `default`)
- No multi-host
- Rate limits liberi
- File-based config + env vars

### Team mode

```bash
docker run -d \
  -e CONVERGIO_DB=postgres://user:pass@host/db \
  -e CONVERGIO_HMAC_KEY=$(openssl rand -hex 32) \
  -p 8420:8420 \
  convergio:latest
```

Pensato per:
- Team 5-50 persone
- Healthcare / regulated AI shop
- Tenant isolation via `org_id`

Differenze rispetto a personal:
- HMAC auth obbligatoria su ogni request
- Multi-org (tabelle scope-by-org_id)
- Postgres = durability + replicabilità + backup standard
- Audit log hash-chain verificabile da processo terzo (es: cron che fa `GET /v1/audit/verify` ogni ora)

**Stesso codice, stesso binary.** Modalità è `match config.db { Sqlite(_) => personal_path, Postgres(_) => team_path }` in 3 punti.

---

## MVP scope (8 settimane)

Settimana 1-2: **Cleanup + Layer 1 hardening**
- Rinomina/riorganizza crate per matchare le 4 layer (no nuovo codice, solo allineamento)
- Verifica modalità SQLite/Postgres sullo stesso binary (probabilmente è già sqlx-pronto, conferma)
- Fix doc lie sui "3 background loop"
- Hash-chain dell'audit log se non c'è già (probabile non c'è)
- Endpoint `/v1/audit/verify` che ricalcola il chain

Settimana 3-4: **Layer 2 sliming + Layer 3 verify**
- Bus message: verificare cosa esiste già in IPC (`ipc_messages`, `ipc_shared_context` viste nel doctor cleanup), trim a topic + direct + ack
- Lifecycle: confermare che spawn + heartbeat + reap funzionano end-to-end (un E2E test apposta)

Settimana 5-6: **Layer 4 minimal viable**
- `cvg solve "<mission>"` produce plan in DB
- `cvg start <plan_id>` lancia executor che spawna agenti
- Thor validator gira come processo separato e valida pre-close
- Quickstart end-to-end: `convergio start && cvg solve "build me a todo CLI" && cvg start <id>` → 5 min, output reale

Settimana 7: **README + landing + demo**
- 7-word value prop testata su 5 amici
- Demo video 60s: gate rejection + heartbeat reaper + audit chain verify
- Replace README. Niente architettura. Niente 38 crate. Quickstart prima.
- Pagina `convergio.dev` (o subdomain) — landing single-page

Settimana 8: **Outreach**
- Messaggio ad Antonio Gatti (è già nel design doc precedente — questa settimana, non la 8a)
- HN show launch
- 5-10 outreach diretti a healthcare/compliance circles
- 3 PR diretti su issue di repo che potrebbero usarlo (Claude Code skills che vogliono audit trail, etc.)

**Success criteria invariati dal doc precedente:** 3 adopter esterni, 10 conversazioni buyer healthcare, 0 nuove feature non richieste.

---

## Tech choices (decisi, non da rivisitare ogni settimana)

- **Lingua**: Rust (mantenuta — riscrittura sarebbe insanity).
- **DB abstraction**: `sqlx` con feature flag SQLite + Postgres (probabilmente già in uso, conferma).
- **HTTP**: `axum 0.7` (mantenuta).
- **Async runtime**: `tokio` (mantenuta).
- **Auth team mode**: HMAC headers (es: `X-Convergio-Signature: hex(hmac(key, body))`). No JWT, no OAuth, no cose complicate. Aggiungere OIDC/SAML solo se enterprise customer lo chiede esplicitamente.
- **CLI**: `clap` derive (probabilmente già).
- **Logging**: `tracing` + json output per team mode.
- **Distribution**: `cargo install convergio` per personal, `docker pull convergio/convergio:latest` per team. Niente .deb, .rpm, brew, snap finché non c'è demand.

**No-go:**
- Niente nuovi linguaggi (no TypeScript dashboard nell'MVP — JSON CLI è abbastanza).
- Niente WebSocket (long-polling + SSE bastano per Layer 2).
- Niente UI grafica nell'MVP. Layer 4 è CLI + JSON. UI viene se qualcuno lo chiede.
- Niente schema migration framework custom — `sqlx::migrate!` + file SQL.

---

## Anti-goals (cose che NON faremo, scriverle qui evita drift)

1. **Non costruiamo un nuovo agent framework.** Se qualcuno chiede "ma come si scrive un agente in Convergio?" → "non si scrive in Convergio. Si scrive in Claude Code, Python, dove vuoi. Convergio gli dà la durability."
2. **Non competiamo con LangGraph/CrewAI/Swarm/Mastra/AutoGen.** Loro sono i nostri client.
3. **Non costruiamo orchestration multi-host nell'MVP.** Mesh è defer, non scope creep.
4. **Non costruiamo un sistema di permission/RBAC complesso.** `org_id` + HMAC è enough per MVP. RBAC quando ci sarà un customer enterprise.
5. **Non aggiungiamo "AI features" gimmick.** Niente "natural-language plan editing", niente "AI suggests next task". Layer 4 fa il minimo — il resto è il client.
6. **Non promettiamo SLA cloud nell'MVP.** Personal e self-hosted team. Cloud SaaS è fase 2 e non è OSS pure.
7. **Non costruiamo agent marketplace, skill marketplace, registry.** Mai detto, mai cominciato.

---

## Cosa cambia nel positioning

**Prima**: "Agent orchestration platform with daemon-enforced gates."
**Dopo**: **"The durability layer for agent workflows. Crash-resilient state, hash-chained audit, agent message bus, long-running supervision. Personal mode in 30 seconds (SQLite), team mode in 5 minutes (Postgres). Drop-in for any agent framework."**

Tagline candidate (da testare):
- *"Postgres for agent workflows."* — più chiara
- *"The audit trail your agents don't have."* — più verticale
- *"Your agents stop lying."* — più viscerale
- *"State, audit, and supervision for agent workflows that actually run for hours."* — più descrittiva

Da provare su 5 dev tecnici e 2 healthcare compliance — vince quello che tutti capiscono senza spiegazione.

---

## Concorrenza (fact-check, riformulato)

| Concorrente | Cosa fa | Cosa NON fa (e noi sì) |
|---|---|---|
| **Temporal** | Durable workflow execution | Non agent-aware, no gate per evidence, no agent message bus dedicato |
| **LangGraph checkpointer** | State persistence per chain | In-process, no audit, no multi-vendor, no liveness reaper |
| **Anthropic MCP** | Agent ↔ tool protocol | Non è state durability, non è audit, non è agent ↔ agent |
| **Google A2A** | Agent ↔ agent protocol cross-vendor | Non è state, non è audit, non è gate enforcement |
| **AutoGen group chat** | Multi-agent in-process | In-process, no durability, no audit |
| **Letta (MemGPT)** | Long-term agent memory | È memoria, non workflow state. Diverso problema. |
| **CrewAI** | Multi-agent orchestration framework | Framework, non backend. Nostro target customer. |

**Posizione vuota dove sediamo noi**: durability + audit + bus + lifecycle, vendor-agnostic, OSS, personal/team modes. Nessuno la occupa con questa precisa shape.

---

## Open questions (devono essere risposte prima di iniziare le 8 settimane)

1. **rusqlite in `daemon/Cargo.toml:11-12`** — production path o test fixture? Rilevante perché:
   - Se test-only: SQLite gira solo in personal mode via `sqlx-sqlite`, OK.
   - Se production path inappropriato: refactor per usare solo `sqlx`.
2. **Audit log hash-chain — già implementato o no?** Se no, Settimana 1 deve includerlo.
3. **`convergio-orchestrator` v0.1.15** è un repo separato. Per ridurre attrito, va re-incorporato nel monorepo o tenuto separato? Suggerimento: re-incorporare nei 10-12 crate KEEP, salvo che ci sia ragione strategica per crates.io standalone.
4. **License** — non ho verificato. Per OSS adoption healthcare/team mode serve Apache 2.0 o MIT. Non AGPL.
5. **Layer 2 — esiste già parzialmente?** Doctor menziona `ipc_messages` e `ipc_shared_context`. Mappa cosa c'è già su quello che serve.
6. **Worktree workflow** — già funzionante (G1 hook OK). Verificare che si integri pulito con Layer 3 spawn.

---

## Migration path da Convergio attuale → v3

**Non è una riscrittura.** È una pulizia + riposizionamento.

1. **Settimana 0** (questa settimana): freeze su nuove feature in tutte le crate non-KEEP. Issues nuove → "v3 backlog, deferred".
2. **Settimana 1**: workspace `Cargo.toml` riorganizzato. Crate non-KEEP rimosse dal workspace ma NON cancellate (resta su crates.io per chi le usa, ma non vengono più aggiornate).
3. **Settimana 2**: doc fix (3-loop lie, count, README), audit hash-chain se manca.
4. **Settimana 3-6**: hardening + reference implementation. Niente regression: ogni feature KEEP deve passare i suoi test E2E esistenti.
5. **Settimana 7-8**: nuovo positioning, nuovo README, nuovo landing, outreach.

Il codice esistente di Convergio attuale **resta nei suoi repo crates.io** — non lo butti via. Solo non è più centro della narrativa.

---

## What I noticed about how you think (round 2)

- **Hai mosso l'idea senza affezionarti**. "Togliamo tutto quello che non serve, ripartiamo dalla base, facciamola funzionare bene bene". Quello è founder mode. Aver speso 12 mesi su 38 crate e poter dire "via il 70%" è raro. Tieni questo.
- **Hai costruito la tesi corretta da solo**, dopo che ti ho messo davanti la domanda del Markdown. Hai fatto i 4 livelli (durability + bus + lifecycle + reference) senza che io te li suggerissi, e hai messo la condizione corretta ("se planner/Thor/executor non sfruttano queste cose, non funziona"). Questo è esattamente il livello di astrazione giusto.
- **Hai detto "forse skills on demand più avanti, non sono sicuro"**. Sentirsi liberi di dire "non lo so" su una decisione di scope è la prova che non sei in pitch mode. Resta in non-pitch mode finché parli con i prospect — è raro e prezioso.

---

## Decisione richiesta

1. **Approvi i 4 layer** così come scritti, o vuoi spostare/togliere qualcosa?
2. **Lista crate KEEP/DEFER/CUT** — ti torna o ci sono crate specifiche che vuoi muovere di categoria?
3. **Settimana 0 = questa settimana**. Vuoi che generi una checklist concreta delle prime 5 azioni operative (non concettuali) da fare entro venerdì 2026-05-01?

Niente codice questa settimana se non per le 5 azioni che decidiamo insieme.

---

*Output sessione 2 di /office-hours del 2026-04-26 — durability layer reframe.*
