#!/bin/bash

MAX_WAIT_SECONDS=500
SCRIPT_DIR=$(dirname "$(realpath "$0")")
pushd "$SCRIPT_DIR" || exit 1

FILTER=${1:-*}
SESSIONS=()

function cleanup () {
	for session in "${SESSIONS[@]}"; do
		tmux kill-session -t "$session" > /dev/null 2>&1 || true
	done
}

for filename in ./configs/${FILTER}.py; do
	filename=$(basename "${filename}")
	module_name=configs.${filename%.*}
	SESSION_NAME="compile-${filename%.*}-$(head /dev/urandom | tr -dc 'a-z0-9' | head -c 8)"
	tmux new-session -d -s "${SESSION_NAME}"
	tmux resize-window -t "${SESSION_NAME}" -x 70 -y 20
	tmux send-keys -t "${SESSION_NAME}" "python -m ${module_name}; tmux wait-for -S ${SESSION_NAME}-done ; exit" Enter
	SESSIONS+=("$SESSION_NAME")
done

trap cleanup EXIT

for session in "${SESSIONS[@]}"; do
	echo "Waiting for ${session}..."
	timeout $MAX_WAIT_SECONDS tmux wait-for "${session}-done"
	RETVAL=$?
	if [ $RETVAL != 0 ]; then
		echo "Timeout while waiting for $session"
                # No Error since the Gitlab CI is weird and might randomly not receive the tmux wait signal
        else
	    echo "Compiled ${session}"
        fi
done

popd || exit 1
