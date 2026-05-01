---
id: review-prd-001-v1
type: adversarial-review
target: docs/prd/0001-claude-code-adapter.md
template_version: v1
date: 2026-05-01
reviewer: claude-opus-4-7-1m (sibling-session, manual-fallback per ADR-0022)
language: italiano
---

# Adversarial review — PRD-001 (Claude Code adapter)

> Review prodotta secondo il template v1 di `docs/templates/adversarial-challenge.md`, con verifica diretta nel codebase sul branch `worktree-wave-0b-claude-code-adapter` (HEAD `593bda6`, post-merge di `origin/main`). L'autore ha richiesto challenge adversariale, non rinforzo.

## A) Contraddizioni interne (top 5)

**A1 — ADR-0023 dichiarato "non ancora scritto" ma esiste già.**
PRD-001 § Bus topology recita: *"that schema change is itself a small ADR (proposed but not yet written, will be ADR-0023 if accepted as part of this PRD)"* (`docs/prd/0001-claude-code-adapter.md:147-148`). Però `docs/adr/0023-system-session-events-topic.md` esiste già (285 righe, status `proposed`). **Prevale la realtà**: rimuovere il "not yet written" o riformulare come "drafted alongside this PRD".

**A2 — Naming installer divergente.**
PRD-001 § Risks dichiara: *"ship a one-line installer (`cvg setup claude-code`)"* (`docs/prd/0001-claude-code-adapter.md:303-304`). Il binario reale espone `cvg setup agent claude` (`crates/convergio-cli/src/commands/setup.rs`, host enum value `Self::Claude => "claude"`). Sono due API name diverse. **Prevale il codice già in `main`** — il PRD va riallineato.

**A3 — Heartbeat "background loop" in un hook one-shot.**
§ Artefact 2 dice: *"Heartbeat: a background loop in the hook agent runs `POST /v1/agents/:id/heartbeat` every 30 seconds"* (riga 118-119). I Claude Code hook sono comandi one-shot eseguiti dall'harness, non processi long-running. Il PRD non spiega chi tiene vivo il loop quando l'hook ritorna. Possibili interpretazioni: (a) launchd plist, (b) un daemon helper separato, (c) reaper-driven (il daemon stesso). Va resa esplicita.

**A4 — "actions" vs schema reale di NewAgent.**
§ Artefact 1 specifica un payload con `kind`, `name`, `host`, `actions`, `metadata` (riga 73-86). Il task pending del plan v0.2 — *"Tighten NewAgent.kind enum + serde validation (was: undocumented, accepts garbage)"* (task `307e6a3e`) — implica che il struct `NewAgent` esiste ma con campi non ancora ufficializzati. Il PRD assume un contratto che il codice attuale **non valida**: una sessione potrebbe POSTare un `actions: ["delete-prod", "rm-rf"]` e il daemon accetterebbe garbage. La review precedente ha già nominato questo gap. PRD va aggiornato per dichiarare l'ordine: prima lo strict-enum, poi l'adapter.

**A5 — Drift root `AGENTS.md` vs `/v1/agent-registry/*`.**
Il root `AGENTS.md` § "MCP tools available" elenca solo `/v1/agents/spawn` e `/v1/agents/:id/heartbeat`. Il codice (`crates/convergio-server/src/routes/agent_registry.rs`) ha 4 endpoint distinti su `/v1/agent-registry/agents*`. Il root file non li menziona. Non è colpa del PRD ma diventa un'incoerenza visibile non appena un terzo agente legge i due documenti insieme.

## B) Promesse insostenibili (top 5)

**B1 — "Same skill pattern, separate PRDs, Wave 2".**
§ "What this PRD does *not* deliver" promette adapter Cursor / Codex / Copilot in Wave 2 con la *stessa* skill pattern (`docs/prd/0001-claude-code-adapter.md:285-287`). Wave 2 nella ROADMAP è 4 settimane. 4 vendor × installer + skill template + per-vendor hook semantics = ottimismo. La promessa serve se motivata da urbanism (estensibilità a costo marginale), ma va declinata con un margine concreto: "Wave 2 ships *one* additional vendor as proof; gli altri restano roadmap fino a feedback dogfood".

**B2 — Hook latency "single-digit ms locally".**
§ Risks dichiara *"response is 200 in single digit ms locally"* (riga 298). Nessun benchmark allegato. Una HTTP call locale via curl ha tipicamente RTT 5-15 ms su macOS, non "single digit". Aggiungere telemetria fin dal day-1 (richiamato in Estimated effort, "2 days — telemetry") ma senza un baseline misurato la mitigation è ottimistica.

**B3 — `cvg pr sync <plan_id>` come azione suggerita.**
§ Artefact 4 check 1 raccomanda: *"Suggested action: `cvg pr sync <plan_id>` (T2.04 integration)"* (riga 173). Il binario `cvg pr` espone solo `stack`. Non c'è `sync`. T2.04 non è verificabile come task referenziabile. Promettere un'azione che il PRD non implementa e che non ha una sede (PRD/ADR) dove vivere è un debito di documentazione.

**B4 — `cvg bus ack <message_id>` come azione suggerita.**
Stesso pattern: § Artefact 4 check 2 (riga 174). Non esiste `cvg bus` come subcommand top-level (`cvg --help` non lo lista). Il bus oggi è raggiungibile solo via HTTP `POST /v1/messages/:id/ack`. PRD-001 deve o ammetterlo (e suggerire la curl call) o committarsi a far esistere `cvg bus ack` come parte degli artefatti.

**B5 — Stima 12-16 giorni per single-developer.**
È plausibile in pieno focus. Però PRD-001 stesso ammette che il developer è anche autore di VISION/ADR/Wave 0a, e che lo stesso developer mantiene contesto cross-corporate (vedi C). 12-16 giorni → 3 settimane calendariali sembra ottimistico se conta context switching. Aggiungere range alto a 4-5 settimane onestizza.

## C) Rischi politici / sociali / legali (top 3)

**C1 — Riferimento esplicito a "Microsoft alignment story (ADR-0017)".**
§ "Why now" enumera tra le motivazioni: *"Microsoft alignment story (ADR-0017) needs a working demo"* (riga 55-58) e cita "ISE Engineering Fundamentals". Il PRD diventa un documento committato in repo pubblico. Datori di lavoro corporate hanno IP review process che possono interpretare `Microsoft alignment` come endorsement non autorizzato anche se il riferimento è soft. Mitigazione: spostare la motivazione in un commento interno o riformularla in termini astratti ("alignment with industry-standard engineering principles").

**C2 — "The operator just lived the failure".**
§ "Why now" cita PIDs reali (`5424`, `77685`) e path utente (`/Users/Roberdan/GitHub/convergioV3`) per descrivere l'evidence di stamattina (riga 32-38). È buona pratica per evidence ma rivela informazioni di setup personale in un documento public-facing. Mitigazione: parafrasare *"two concurrent Claude Code sessions in the same repo"* senza i path personali.

**C3 — `kind: "claude-code"` hardcoded come stringa.**
§ Artefact 1 fa POST con `kind: "claude-code"`. È una stringa identificativa di un prodotto commerciale terzo (Anthropic). Convergio è progetto OSS sotto Convergio Community License v1.3. Hardcodare il vendor name in un payload ufficiale è scelta che (a) lega il contratto a un prodotto, (b) crea precedente per `claude-desktop`, `claude-api` e simili senza criterio. Mitigazione: enum lato server + ADR che giustifica il vocabolario (l'enum mancante è già flagged da v0.2 task `307e6a3e`).

## D) Metafore che si rompono (top 3)

**D1 — "Vigile urbano does not sign the certificate of habitability".**
§ Artefact 4 (riga 152) usa la metafora del vigile per giustificare `cvg session pre-stop`. La metafora è elegante in italiano ma travolge un meccanismo che è semplicemente *un check di consistency a session-end*. Il vigile italiano non rifiuta il certificato in via discrezionale — segue checklist normate. La metafora suggerisce discrezionalità che il check NON ha. Riformulare: *"end-of-day audit"* o *"cleanup gate"* è meno romantico ma onesto.

**D2 — "Long-tail thesis (ADR-0016)" come motore della Wave 0.**
§ Problem cita ADR-0016 long-tail come razionale (*"a shovel that does not coordinate parallel diggers is a single-user tool"*, riga 41). Per un lettore tecnico-pragmatico (CI bot, future maintainer in 6 mesi) questa è marketing speak. La motivazione concreta — "due sessioni vedono i propri commit ma non i progressi reciproci" — è già nel PRD e basta. La citazione long-tail è "perché vendere", non "perché costruire".

**D3 — "Convergio is the leash for any AI agent".**
Il root `AGENTS.md` apre con questa frase. È un'immagine forte (e antica nel codebase, non introdotta da PRD-001). Però "leash" suggerisce vincolo unilaterale: il padrone tira l'agente. La realtà del Convergio è cooperativa (gates server-side, ma il client è cooperativo, non forzato — l'utente sa benissimo che un client non-instrumented bypassa tutto). PRD-001 dovrebbe almeno menzionare che il vincolo è cooperativo, non forzato. Altrimenti un primo lettore aspetta enforcement che non c'è.

## E) Gap della roadmap (top 3)

**E1 — `cvg setup agent claude` esiste già, PRD lo ignora.**
Commit `85332ea` (29 aprile, co-authored Copilot) ha aggiunto lo subcommand. Il PRD parla come se `cvg setup claude-code` fosse da costruire da zero. **Effetto**: chi esegue il task w1.5 potrebbe scriverne uno nuovo invece di estendere quello esistente. PRD va aggiornato per riconoscere lo scheletro e definire l'estensione (generare `.claude/settings.json` template oltre a `mcp.json`).

**E2 — Wave 0b plan duplica il task w1.6 con drift su PR #58.**
Plan v0.1.x ha task `9ce7a17c` (`cvg status v2: human-friendly progress dashboard`) che è stato chiuso da PR #58. Wave 0b ha task w1.6 (`cvg status --agents flag + EN/IT i18n`) che dovrebbe estendere il `status_render` introdotto da #58. Il PRD non menziona il piggyback. Senza nota esplicita, l'esecutore di w1.6 rifarà la struttura.

**E3 — Heartbeat 30s vs Watcher tick 30s vs Reaper tick 60s.**
PRD prescrive heartbeat ogni 30s (riga 118). Watcher loop nel daemon gira ogni 30s (`CONVERGIO_WATCHER_TICK_SECS` default), Reaper ogni 60s con timeout 300s. Se hooks falliscono più volte e il daemon perde 2 heartbeat consecutivi, watcher potrebbe flippare prima che reaper rilasci leases. Effetto: lease lock-out spurio. PRD deve dichiarare la finestra `(heartbeat_interval, reaper_timeout, watcher_threshold)` e perché è stabile.

## F) Errori tecnici (top 5)

**F1** — Endpoint `/v1/agent-registry/agents` correttamente referenziato (PRD riga 71-72) — verificato in `crates/convergio-server/src/routes/agent_registry.rs:13`. ✅ OK.
**F2** — `cvg setup claude-code` non esiste; il comando reale è `cvg setup agent claude`. Vedi A2.
**F3** — `cvg pr sync <plan_id>` non esiste; `cvg pr` ha solo `stack`. Vedi B3.
**F4** — `cvg bus ack <message_id>` non esiste; non c'è subcommand `cvg bus`. Vedi B4.
**F5** — `cvg session pre-stop` non esiste; `cvg session` ha solo `resume`. Va creato come parte di Artefact 4 (PRD lo dice implicitamente, ma il task w1.x corrispondente non è chiaramente nel plan Wave 0b: nei 10 task non c'è uno chiamato "implement cvg session pre-stop"). **Plan-PRD drift**.

## G) Verdict

**Ship now con 5 fix obbligatori prima del primo commit di codice nuovo:**

1. **A1**: rimuovere "ADR-0023 not yet written" dal PRD (è già scritto).
2. **A2/F2**: riallineare il PRD a `cvg setup agent claude` (esistente) e dichiarare che w1.5 *estende*, non *crea*.
3. **B3, B4, F3, F4**: per ogni `cvg <verb>` citato nel PRD ma assente, scegliere fra (a) implementarlo come parte di Wave 0b (e aggiungere il task corrispondente al plan), (b) sostituirlo con la curl/HTTP call, (c) marcarlo `(future)`.
4. **F5/Plan drift**: aggiungere al plan Wave 0b un task esplicito per `cvg session pre-stop` o documentare che è già coperto da uno dei 10 esistenti (e quale).
5. **C1, C2**: sanitizzare i riferimenti `Microsoft alignment` e i PID/path personali nel PRD pubblico.

**Deferred con nota** (non bloccanti per Wave 0b ma da riprendere):
- A3 (heartbeat loop): documentare il meccanismo in un mini-ADR o nel PRD aggiornato. Non bloccare il primo skill ship.
- D1, D2, D3 (metafore): cosmesi, riprendere a Wave 1 se la dogfood lo richiede.
- E3 (heartbeat/reaper window): aggiungere una sezione "timing" al PRD se Artefact 4 lo richiede.

**Wont-fix con rationale**:
- A4 (NewAgent.kind enum): è un task v0.2 indipendente, non causa di Wave 0b. Coordinare nel plan ma non bloccare qui.
- A5 (drift root AGENTS.md): è bug del root file, da fixare separatamente con un PR docs.

**Stima impatto fix**: 1 giornata uomo aggiuntiva, principalmente edit di prosa nel PRD e 2-3 task aggiunti al plan. Niente cambia nei 12-16 giorni di engineering core.
