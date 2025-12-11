#!/bin/bash

SCRIPT_DIR=$(dirname "$(realpath $0)")
pushd $SCRIPT_DIR

FILTER=${1:-*}

for filename in ./configs/${FILTER}.py; do
	filename=$(basename $filename)
	module_name=configs.${filename%.*}
	tmux new-session \; resize-window -x 70 -y 20 \; attach \; send-keys "nix develop . --command python -m ${module_name}; exit" Enter
done

popd
