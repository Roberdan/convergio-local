---
id: review-prd-001-pre-pr-v1
type: adversarial-review
target: docs/prd/0001-claude-code-adapter.md (entire Wave 0b PR)
template_version: v1
date: 2026-05-01
reviewer: claude-opus-4-7-1m (sibling-session second-pass per ADR-0022)
language: italiano
---

# Adversarial review (pre-PR) — Wave 0b assembled

> Seconda passata adversariale per la PR #62 di Wave 0b. Riapplica il template v1 di ADR-0022 al deliverable assemblato (PRD aggiornato + ADR-0024 + bus migration + skill `/cvg-attach` + endpoint `/v1/system-messages` + estensione `cvg setup agent claude` + flag `cvg status --agents` + E2E test + demo). I findings di v1 (`docs/reviews/PRD-001-adversarial-review-v1.md`) sono stati indirizzati nei commit `d68957c` e seguenti; questa passata cerca i nuovi problemi che il codice scritto introduce.

## A) Contraddizioni interne (top 5)

**A1 — w1.4b "cvg session pre-stop" è dichiarato in PRD-001 § Artefact 4 ma non implementato in questa PR.**
Il task `168e9561` aggiunto al plan (con commit `d68957c`) per coprire l'Artefact 4 è stato esplicitamente *deferred* a Wave 0b.2, perché impatterebbe `session.rs` già a 298/300 line cap e implementare 6 check di qualità reale richiede una slice separata. **Verdict**: il PRD descrive ancora pre-stop come parte di Wave 0b. Va o (a) aggiornato per indicare "Artefact 4 deferred to Wave 0b.2" oppure (b) implementato anche solo come scaffolding `(future)` per ogni check. Il commit corrente lascia il PRD a mentire. **Mandatory fix**.

**A2 — `actions` vs `capabilities` non riallineati nel PRD.**
Finding A4 della v1 era marked "wont-fix con rationale" ma il fatto pratico è che la skill (`cvg-attach.sh`), l'E2E test (`e2e_two_agents_coordinate.rs`) e l'estensione del setup ora **tutti** usano `capabilities` (il nome reale del campo in `NewAgent`). Il PRD continua a parlare di `actions`. **Verdict**: minore, ma ora il PRD è formalmente in disaccordo con il codice in 5 file. **Defer with note** è ok se si aggiunge una mezza riga al PRD ("the code calls this field `capabilities` for historical reasons; PRD calls it `actions` to flag the future rename").

**A3 — ADR-0024 status `proposed` mentre la migration è già live nella PR.**
La migration `0103_system_topics.sql`, le route `/v1/system-messages`, e i test e2e hanno tutti landed. ADR-0024 dovrebbe transitare a `accepted` nello stesso PR perché il contratto è ora *codificato*, non più "proposto". Lasciarlo `proposed` significa che CI potrebbe (a ragione) refusare la merge nel momento in cui un gate "ADR-status-vs-implementation drift" entrasse in vigore. **Mandatory fix**: promuovere ADR-0024 a `accepted` come ultimo commit della PR, prima del review umano.

**A4 — PRD-001 § Artefact 1 specifica heartbeat ogni 30s.**
La skill `cvg-attach.sh` POSTa la registrazione e UNA presence message su `system.session-events` poi exit 0. Niente loop heartbeat. Per il PRD il loop è in "the hook agent" (Artefact 2), non nella skill. Ma `.claude/settings.json` template emesso da `cvg setup agent claude` include solo `SessionStart`, non un hook periodico. **Verdict**: l'heartbeat *concretamente* non gira oggi. Va detto in commit body / CHANGELOG o aggiunto come task pending Wave 0b.2.

**A5 — README cvg-attach descrive il fallback "daemon offline" come "warning on stderr; never blocks the user".**
Vero per la skill (script bash). Però `cvg setup agent claude` **non** scrive un hook PreToolUse — quindi se il daemon va giù mid-session, non c'è nessun warning live. La graceful-degradation è solo a SessionStart. Il README sovrastima il safety net.

## B) Promesse insostenibili (top 5)

**B1 — Demo script richiede `cvg` con flag `--agents`, non installato finché la PR non viene mergeata.**
`demo-two-sessions.sh` ha un fallback (rilevamento `--help | grep`) che mostra il raw JSON quando il flag manca. Buono. Ma il README dice "se il binary in PATH non ha --agents, fai `cargo install --path crates/convergio-cli --force`" — questo richiede compilare il workspace localmente, che fuori dal contesto dev non è banale. Per un primo lettore "dovrei provare il demo" è un percorso a 3 step minimo. **Mitigazione**: rimuovere la promessa di "no Claude required" o cambiarla in "no Claude binary required, but cvg from this PR required".

**B2 — System-message route accetta qualsiasi `sender` senza verifica.**
`POST /v1/system-messages` accetta un body con `sender: Option<String>` e lo persiste. Niente cross-check tra il sender dichiarato e l'agent registrato. Una sessione potrebbe pubblicare presence per conto di un'altra. Per un daemon localhost-only single-user è policy ragionevole, ma una promessa implicita di "agent-to-agent coordination autenticata" non viene mantenuta. **Mitigazione**: documentare esplicitamente nel README/ADR-0024 che il bus *non* fa identity verification (è single-user, fidato).

**B3 — E2E test non simulano un vero `cvg-attach.sh` flow.**
`e2e_two_agents_coordinate.rs` chiama direttamente `POST /v1/agent-registry/agents` con reqwest. La skill bash `cvg-attach.sh` non viene mai esercitata. Una regressione nel parser bash o nei placeholder env passerebbe il test. **Mitigazione**: opzionale aggiungere uno smoke test `tests/integration/skill-attach.sh` o accettare il gap come "shell scripts are tested by the demo".

**B4 — `cvg setup agent claude` shell-out flow non testato live.**
I 2 nuovi smoke test in `cli_smoke.rs` verificano *che i file esistano* dopo il setup. Non verificano che `bash settings.json command` effettivamente esegua e registri. Per un installer la promessa importante è "esegui questa pipeline → la skill funziona". Test mancante.

**B5 — Stima 12-16 giorni nel PRD vs questa PR.**
La PR consegna concretamente: ADR-0024 + bus migration + skill + endpoint + setup extension + status flag + 5 nuovi e2e tests + demo. Tempo concreto della sessione: ~2 ore di lavoro Claude. Più tempo umano per review + decisioni. La stima del PRD era ottimistica per single-developer pure ma la realtà-con-agente è ancora più favorevole. **Non un fix, una nota**: il PRD può aggiornare il proprio § Estimated effort post-PR per riflettere il "con agent assistance" baseline, utile per stime future.

## C) Rischi politici / sociali / legali (top 3)

**C1, C2** — Già indirizzati dal commit `d68957c` (sanitize). Nessun nuovo riferimento a Microsoft o PID/path personali introdotto dai commit `562d1e9..c008f54`. ✓

**C3 — Skill bash committata in `examples/` senza shellcheck linting CI.**
`cvg-attach.sh` e `demo-two-sessions.sh` non sono coperti da nessun gate (no shellcheck, no bats). Una regressione nel quoting (es. `${PWD}` con spazi) sfuggirebbe. Repo già ha `set -euo pipefail` come convention (best-practices.md). Mitigazione: aggiungere un task minimo Wave 0b.2 per `lefthook` shellcheck su `examples/skills/**/*.sh`.

## D) Metafore che si rompono (top 3)

**D1 — La skill "registra" la sessione "before any plan exists".**
Linguaggio di "registrare" suggerisce un atto formale. Quello che succede è una INSERT in SQLite con vita arbitraria (non c'è TTL). Una sessione registrata 14 giorni fa, mai retired (es. crash + macchina spenta), resta "registered" per sempre. Il termine è impreciso. Mitigazione: documentare nel README che il record è una "presence claim" che il reaper può pulire (Reaper ha tick 60s, timeout 300s — già documentato in root AGENTS.md).

**D2** — Nessuna nuova metafora forte introdotta. Le metafore "leash", "Modulor", "Difensore Civico" stanno tutte fuori da questa PR.

## E) Gap della roadmap (top 3)

**E1 — w1.4b deferred crea cascading effect su w1.9 e w1.10.**
Il pre-PR review (questo file) elenca w1.4b come deferred. La PR può comunque mergeare con CI verde, ma il plan Wave 0b non sarà al 100% "done" dopo `cvg validate`. Andrà o (a) chiuso il task `168e9561` come `failed` con motivo "deferred to Wave 0b.2" oppure (b) lasciato pending e il plan stesso resterà non-validabile per l'intera Wave 0b. Decisione del operatore.

**E2 — `cvg setup agent` per Copilot CLI non emette nulla di equivalente a `.claude/settings.json`.**
La PR aggiorna *solo* `AgentHost::Claude`. Il principio dichiarato ("convergio sopra qualunque agente") richiede lo stesso pattern per `~/.copilot/hooks/` (oggi vuota). Wave 0b.2 task naturale.

**E3 — system.* topic family non ha retention policy implementata.**
ADR-0024 § Retention parla di "24h ring buffer". Niente nel codice oggi pulisce vecchi messaggi `system.*`. Un Convergio attivo per mesi accumula. Wave 0b.2 / Wave 1 task.

## F) Errori tecnici (top 5)

**F1** — `POST /v1/system-messages` non rigetta a livello HTTP un topic non `system.*`: il bus refuse con `BusError::InvalidTopicScope`, e l'errore viene serializzato come 500 (probabilmente — verificato con il test `system_message_rejects_non_system_topic` che asserisce solo `is_client_error || is_server_error`). Gate cleaner: mappare `InvalidTopicScope` esplicitamente a 400. **Defer with note**: il test passa, ma 500 è errore generico server, non client error.

**F2** — `cvg status --agents --output json` mette `agents` direttamente nel body. Schema della response non documentato. Un consumer agente che fa `body.agents.len()` senza il flag riceve `undefined`. **Mitigazione**: minimal — aggiungere `agents: []` sempre nel JSON output, così la chiave esiste.

**F3** — `cvg setup agent claude` legge SKILL.md / cvg-attach.sh via `include_str!` con percorso `../../../../examples/skills/...`. Se la struttura del repo cambia (esempio: workspace member rinominato) il `include_str!` fallisce a compile-time. **Acceptable**: compile-time check è il gate giusto, ma una regression test che fa `setup agent claude` in tempdir e verifica i checksum dei file generati = pattern di robustezza.

**F4** — Il NewAgent serde struct non valida che `id` sia non-vuoto, e non valida che `kind` sia in un enum noto. Già flagged dal v0.2 task `307e6a3e` (Tighten NewAgent.kind enum + serde validation). Non bloccante per Wave 0b ma il fatto che la skill posti `kind: "claude-code"` significa che il piano v0.2 deve includere `claude-code` nell'enum quando atterra. Inter-plan dependency.

**F5** — Il commit body del merge `593bda6` ("Merge branch 'main' into wave-0b") non ha conventional-commit shape. Probabilmente commitlint non lo blocca (i merge sono esenti per convenzione), ma vale la pena verificarlo prima del PR ready.

## G) Verdict

**Ship now con 3 fix obbligatori prima di marcare la PR ready:**

1. **A1 + E1**: decidere su w1.4b. Due opzioni:
   - `cvg task transition 168e9561 failed` con messaggio "deferred to Wave 0b.2"; aggiornare PRD `§ Artefact 4` per dire "deferred"; il plan può validare con il task in `failed` (Thor lo accetta come terminale).
   - oppure lasciare il task `pending` e accettare che `cvg validate` ritorni `fail` finché Wave 0b.2 non lo completa. Plan resta in volo.
2. **A3**: promuovere ADR-0024 da `proposed` ad `accepted` con un commit prima del PR ready.
3. **A4**: aggiungere una breve nota in PRD-001 su Heartbeat ("loop deferred to Wave 0b.2; SessionStart-only registration is the v1 cut").

**Deferred con nota** (non bloccanti, non vanno persi):
- A2 (PRD `actions` vs codice `capabilities`): aggiungere mezza riga al PRD.
- B2 (sender autenticità): documentare in ADR-0024.
- C3 (shellcheck su `examples/skills/`): task Wave 0b.2.
- E2 (Copilot adapter): task Wave 0b.2.
- E3 (system topic retention): task Wave 1.
- F1 (mapping InvalidTopicScope → 400): defer.
- F2 (`agents: []` sempre nel JSON): defer.

**Wont-fix con rationale:**
- B3, B4 (test della skill bash + del shell-out installer): rinviati a una passata di shellcheck + bats; il pattern E2E rust è il gate primario.
- B5 (stima del PRD): documentazione che si aggiorna con il tempo, non un fix.
- D1 (terminology "registra"): cosmesi.

**Stima impatto fix mandatory**: ~30 min totali (1 commit edit PRD, 1 commit promote ADR-0024, 1 task transition o decisione operatore su w1.4b).

## Confronto con review v1

I 5 mandatory fix di review v1 sono stati indirizzati dal commit `d68957c`. La distanza tra "PRD scritto" e "codice scritto" si è accorciata significativamente: questa passata ha prodotto solo 3 nuovi mandatory fix (vs 5 della precedente), e 2 dei 3 sono "decisioni" più che "lavoro di codice". Il sistema sta convergendo su uno stato consistente.
