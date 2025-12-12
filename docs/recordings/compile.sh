#!/bin/bash

SCRIPT_DIR=$(dirname "$(realpath "$0")")
pushd "$SCRIPT_DIR" || exit 1

FILTER=${1:-*}
SESSIONS=()

for filename in ./configs/${FILTER}.py; do
	SESSION_NAME="compile-$(head /dev/urandom | tr -dc 'a-z0-9' | head -c 8)"
	filename=$(basename "${filename}")
	module_name=configs.${filename%.*}
	tmux new-session -d -s "${SESSION_NAME}"
	tmux resize-window -t "${SESSION_NAME}" -x 70 -y 20
	tmux send-keys -t "${SESSION_NAME}" "nix develop . --command python -m ${module_name}; tmux wait-for -S ${SESSION_NAME}-done ; exit" Enter
	SESSIONS+=("$SESSION_NAME")
done

for session in "${SESSIONS[@]}"; do
	tmux wait-for "${session}-done"
done

popd || exit 1
