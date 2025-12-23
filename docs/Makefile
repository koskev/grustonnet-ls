build:
	npm install
	tmux start-server
	bash ./recordings/compile.sh
	git-cliff --workdir .. -o changelog.md
	mdbook build
