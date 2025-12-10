
for filename in ./configs/*.py; do
	filename=$(basename $filename)
	module_name=configs.${filename%.*}
	tmux new-session \; resize-window -x 70 -y 20 \; attach \; send-keys "nix develop . --command python -m ${module_name}; exit" Enter
done
