# Dev Setup

To get a dev environment just install Nix and run `nix develop .`
To automatically enter the dev-shell install `direnv` and run `direnv allow`


## Building containers

To ensure reproducibility there is no `Dockerfile` and all images are built using Nix. 
Run `nix build .#dockerImageFull` do build the full image.


| Target | Description |
|----|----|
| dockerImageFull | Contains all binaries |
| dockerLs | Contains the language server |
| dockerLinter | Contains the linter |
| dockerDebugger | Contains the debugger |


To directly load the image into Podman you can use `nix run .#dockerImageFull.copyToPodman`
