build:
	npm install
	bash ./recordings/compile.sh
	git-cliff --workdir .. -o changelog.md
	mdbook build
