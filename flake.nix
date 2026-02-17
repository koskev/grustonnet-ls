{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix2container.url = "github:nlewo/nix2container";
    recordings.url = "git+https://codeberg.org/kokev/lsp-recorder.git";
  };

  outputs =
    inputs@{
      flake-parts,
      ...
    }:
    # https://flake.parts/module-arguments.html
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        ./nix/common.nix
        ./nix/go-jsonnet.nix
        ./nix/grustonnet.nix
        ./nix/shells.nix
        ./nix/container.nix
        ./nix/docs.nix
      ];
      flake = {
        # Put your original flake attributes here.
      };
      systems = [
        # systems for which you want to build the `perSystem` attributes
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
    };

}
