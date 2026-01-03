build:
	npm install
	pushd recordings && compile-recordings -d configs && popd
	git-cliff --workdir .. -o changelog.md
	mdbook build
