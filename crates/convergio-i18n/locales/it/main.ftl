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
