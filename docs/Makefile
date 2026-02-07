.PHONY: all
all: build

.PHONY: clean
clean:
	rm -r recordings/out
	mdbook clean

.PHONY: test
test:

build: dependencies build_recordings changelog
	mdbook build

serve: dependencies build_recordings changelog
	mdbook serve

build_recordings:
	cd recordings; compile-recordings -d configs; cd ..

changelog:
	git-cliff -r .. -o changelog.md

dependencies:
	npm install
