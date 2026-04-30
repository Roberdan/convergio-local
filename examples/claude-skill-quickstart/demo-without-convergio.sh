#!/usr/bin/env bash
# Demo: the world today — an agent claims "done" and nothing pushes back.
#
# This script does NOT use Convergio. It exists so that the
# `demo-with-convergio.sh` companion has a thing to compare against.
# The whole point of Convergio is that this exact loop becomes
# loud and auditable.

set -euo pipefail

cyan="\033[36m"
red="\033[31m"
green="\033[32m"
reset="\033[0m"

say() { printf "${cyan}%s${reset}\n" "$*"; }
ok()  { printf "${green}%s${reset}\n" "$*"; }
bad() { printf "${red}%s${reset}\n" "$*"; }

say "[1/3] agent decides what to ship"
diff_text="// TODO: wire this later
fn handler() {}"
echo "$diff_text"

say "[2/3] agent claims the work is done"
ok    "    agent: \"shipped handler, all good\""

say "[3/3] human believes the agent"
ok    "    human: \"great, marking task as done\""
echo
bad   "Outcome: a TODO is now in production. No refusal. No audit row."
bad   "If a different reviewer reads the diff later, the only signal is"
bad   "the diff itself — there is no record of who claimed what, when."
echo
echo "Run ./demo-with-convergio.sh next to see the Convergio version."
