build: dependencies build_recordings changelog
	mdbook build

serve: dependencies build_recordings changelog
	mdbook serve

build_recordings:
	cd recordings; compile-recordings -d configs; cd ..

changelog:
	git-cliff --workdir .. -o changelog.md

dependencies:
	npm install
