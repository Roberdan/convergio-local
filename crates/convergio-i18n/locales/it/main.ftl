# Convergio — bundle messaggi in italiano.
# Sintassi Fluent: https://projectfluent.org/fluent/guide/

# ---------- generico ----------
ok = OK
not-found = Non trovato
internal-error = Errore interno

# ---------- daemon ----------
daemon-starting = Avvio del daemon Convergio su { $url }
daemon-listening = In ascolto su { $bind }
daemon-version = Convergio { $version }

# ---------- CLI: health ----------
health-ok = Il daemon è attivo. Versione: { $version }
health-unreachable = Impossibile raggiungere il daemon su { $url }: { $reason }

# ---------- CLI: status ----------
status-header = Stato Convergio
status-active-header = Piani attivi:
status-active-empty = Nessun piano attivo.
status-completed-header = Piani completati di recente:
status-completed-empty = Nessun piano completato.
status-tasks-header = Task completati di recente:
status-tasks-empty = Nessun task completato.
status-plan-line = - { $title } [{ $status }] progetto: { $project } task: { $done }/{ $total } completati
status-progress-line =   avanzamento: { $bar } { $done }/{ $total }
status-breakdown-line =   task: { $done } completati · { $submitted } inviati · { $in_progress } in corso · { $pending } in attesa · { $failed } falliti ({ $total } totali)
status-work-line =   fa: { $work }
status-next-line =   prossimi: { $tasks }
status-wave-line =     wave { $wave }: { $done } completati, { $submitted } inviati, { $in_progress } in corso, { $pending } in attesa, { $failed } falliti
status-mine-header = Filtro: solo task dell'agente { $agent }
status-task-line = - { $title } in { $plan } progetto: { $project }

# ---------- CLI: CRDT ----------
crdt-conflicts-empty = Nessun conflitto CRDT aperto.
crdt-conflicts-header = Conflitti CRDT aperti:
crdt-conflict-line = - { $entity }/{ $id } campo { $field } tipo { $type }

# ---------- CLI: workspace ----------
workspace-leases-empty = Nessun lease workspace attivo.
workspace-leases-header = Lease workspace attivi:
workspace-lease-line = - { $agent } mantiene { $kind } { $path } fino a { $expires }

# ---------- CLI: capabilities ----------
capabilities-empty = Nessuna capability locale registrata.
capabilities-header = Capability locali:
capability-line = - { $name } { $version } [{ $status }]
capability-signature-ok = Firma capability verificata per { $name } { $version } con chiave { $key }
capability-installed = Capability installata: { $name } { $version } [{ $status }]
capability-disabled = Capability disabilitata: { $name } { $version }

# ---------- CLI: setup / doctor ----------
setup-config-created = Configurazione creata: { $path }
setup-config-exists = Configurazione già presente: { $path }
setup-config-backed-up = Configurazione esistente salvata: { $path }
setup-complete = Setup completato: { $path }
setup-next-start = Prossimo passo: avvia il daemon con `convergio start`
setup-next-doctor = Poi: esegui `cvg doctor`
setup-agent-created = Snippet adapter creati per { $host }: { $path }
setup-agent-copy = Copia mcp.json nella configurazione MCP dell'agent host e prompt.txt nelle sue istruzioni.
doctor-header = Diagnostica Convergio per { $url }
doctor-ok = OK { $name }: { $message }
doctor-warn = ATTENZIONE { $name }: { $message }
doctor-fail = ERRORE { $name }: { $message }
doctor-summary-ok = Diagnostica completata con successo.
doctor-summary-fail = La diagnostica ha trovato controlli falliti.
mcp-log-missing = Nessun log MCP trovato.
service-installed = File servizio scritto: { $path }
service-started = Servizio avviato.
service-stopped = Servizio fermato.
service-status-loaded = Servizio caricato.
service-status-not-loaded = Servizio non caricato.
service-uninstalled = Servizio disinstallato.

# ---------- CLI: plan ----------
plan-created = Piano creato: { $id }
plan-not-found = Piano non trovato: { $id }
plan-list-empty = Nessun piano presente.
plan-list-header = { $count ->
    [one] Un piano:
   *[other] { $count } piani:
}

# ---------- rifiuti dei gate (lato umano) ----------
# Il campo `code` resta in inglese (è contratto API).
# Il `message` è ciò che l'umano legge.
gate-refused-evidence = Evidenze mancanti: { $kinds }
gate-refused-no-debt = Debito tecnico trovato nelle evidenze: { $markers }
gate-refused-no-stub = Marker di scaffolding trovati nelle evidenze: { $markers }
gate-refused-zero-warnings = Il segnale di build/lint non è pulito: { $signals }
gate-refused-plan-status = Il piano è { $status }; nuove transizioni non accettate
gate-refused-wave-sequence = { $count ->
    [one] Un task della wave precedente è ancora aperto
   *[other] { $count } task delle wave precedenti sono ancora aperti
}

# ---------- audit ----------
audit-clean = Catena audit verificata: { $count } eventi, nessuna manomissione rilevata.
audit-broken = Catena audit rotta alla sequenza { $seq }.

# ---------- CLI: pr stack ----------
pr-stack-empty = Nessuna PR aperta.
pr-stack-header = { $count ->
    [one] Una PR aperta:
   *[other] { $count } PR aperte:
}
pr-stack-no-manifest = manifest Files-touched assente
pr-stack-manifest-mismatch = il manifest non corrisponde al diff
pr-stack-files-summary = { $count ->
    [one] un file
   *[other] { $count } file
}
pr-stack-suggested-order = Ordine di merge suggerito:

# ---------- CLI: session resume ----------
session-resume-header = Riavvio sessione Convergio
session-resume-health-ok = Daemon: ok (versione { $version })
session-resume-health-down = Daemon: NON attivo (versione { $version })
session-resume-audit-ok = Catena audit: ok ({ $count } eventi)
session-resume-audit-broken = Catena audit: ROTTA ({ $count } eventi verificati)
session-resume-plan-line = Piano: { $title } [{ $status }] progetto: { $project } id: { $id }
session-resume-counts-line = Task: { $done }/{ $total } completati — in corso: { $in_progress }, in revisione: { $submitted }, da fare: { $pending }
session-resume-next-empty = Prossima priorità: nessuna (nessun task aperto).
session-resume-next-header = Prossima priorità (primi task aperti):
session-resume-next-line =   - w{ $wave }.{ $sequence } { $title } [{ $id }]
session-resume-prs-empty = PR aperte: nessuna.
session-resume-prs-unavailable = PR aperte: gh non disponibile (saltato).
session-resume-prs-header = PR aperte:
session-resume-pr-line =   - #{ $number } { $title } ({ $branch })
session-resume-pr-line-draft =   - #{ $number } [bozza] { $title } ({ $branch })
session-resume-pack-line = Context-pack del task { $task_id }: { $nodes } nodi, { $files } file, ~{ $est_tokens } token
