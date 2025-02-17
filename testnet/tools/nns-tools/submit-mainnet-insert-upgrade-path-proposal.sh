#!/bin/bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
source "$SCRIPT_DIR/functions.sh"

help() {
    print_green "
Usage: $0 <PROPOSAL_FILE> <NEURON_ID>
    PROPOSAL_FILE: File with proposal created by ./prepare-publish-sns-wasm-proposal-text.sh (or formatted in that way)
    NEURON_ID: Your mainnet neuron ID, associated with your HSM

  This script will create a proposal on mainnet from a given proposal text.
  It outputs the values for confirmation, and ensures that the target version matches the target hash.
  "
    exit 1
}

if [ $# -ne 2 ]; then
    help
fi

PROPOSAL_FILE=$1
NEURON_ID=$2

value_from_proposal_text() {
    local FILE=$1
    local FIELD=$2
    cat $FILE | grep "### $FIELD" | sed "s/.*$FIELD[[:space:]]*//"
}

check_or_set_dfx_hsm_pin() {
    VALUE=${DFX_HSM_PIN:-}
    if [ -z "$VALUE" ]; then
        echo -n "Enter your HSM_PIN":
        read -s DFX_HSM_PIN
        export DFX_HSM_PIN
        echo
    fi
}

extract_versions_to_publish() {
    PROPOSAL_FILE=$1

    cat $PROPOSAL_FILE | sed -n '/^{$/,/^}$/p' | jq -c
}

submit_insert_upgrade_path_proposal_mainnet() {
    ensure_variable_set IC_ADMIN

    PROPOSAL_FILE=$1
    NEURON_ID=$2

    TARGET_SNS_GOVERNANCE_CANISTER=$(value_from_proposal_text $PROPOSAL_FILE "Target SNS Governance Canister:")

    if grep -q -i TODO "$PROPOSAL_FILE"; then
        echo "Cannot submit proposal with 'TODO' items"
        exit 1
    fi

    echo
    print_green "Proposal Text To Submit"
    cat "$PROPOSAL_FILE"
    print_green "End Proposal Text"
    echo
    print_green "Summary of action:
  You are proposing to modify the upgrade paths in SNS-W.
  This will affect $([ "$TARGET_SNS_GOVERNANCE_CANISTER" == "All" ] \
        && echo "All SNSes." \
        || echo "The SNS with Governance Canister ID: $TARGET_SNS_GOVERNANCE_CANISTER.")
  If any of the versions in the proposal are not on the blessed upgrade path, the proposal will fail.
    "

    check_or_set_dfx_hsm_pin

    cmd=($IC_ADMIN --use-hsm --slot=0
        --key-id=01 --pin="$DFX_HSM_PIN"
        --nns-url "https://nns.ic0.app"
        propose-to-insert-sns-wasm-upgrade-path-entries
        --sns-governance-canister-id=$TARGET_SNS_GOVERNANCE_CANISTER
        --summary-file=$PROPOSAL_FILE
        --proposer=$NEURON_ID
    )
    for V in $(extract_versions_to_publish $PROPOSAL_FILE); do
        cmd+=("$V")
    done

    echo "Going to run command: "
    echo ${cmd[@]} | sed 's/pin=[0-9]*/pin=\*\*\*\*\*\*/' | fold -w 100 -s | sed -e "s|^|     |g"

    echo "Type 'yes' to confirm, anything else, or Ctrl+C to cancel"
    read CONFIRM

    if [ "$CONFIRM" != "yes" ]; then
        echo "Aborting proposal execution..."
        exit 1
    fi

    "${cmd[@]}"
}

submit_insert_upgrade_path_proposal_mainnet "$PROPOSAL_FILE" "$NEURON_ID"
