{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    recordings.url = "git+https://codeberg.org/kokev/lsp-recorder.git";
    grustonnet.url = "..";
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      recordings,
      grustonnet,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        baseNeovim = recordings.lib.${system}.baseNeovim.mkNeovim {
          treesitterPlugins = [ "jsonnet" ];
          extraConfig = ''
            vim.lsp.config['grustonnet'] = {
              cmd = { "grustonnet-ls" },
              filetypes = { 'jsonnet', 'libsonnet' },
              root_markers = { 'jsonnetfile.json', '.git' },
            }
            vim.lsp.enable('grustonnet')
          '';
        };

        nativeBuildInputs =
          with pkgs;
          [
            baseNeovim
            grustonnet.defaultPackage.${system}

            gnumake
            mdbook
            git-cliff
            nodejs
          ]
          ++ recordings.lib.${system}.baseNeovim.nativeBuildInputs;
      in
      {
        # For `nix develop`:
        devShell = pkgs.mkShell {
          inherit nativeBuildInputs;
        };
      }
    );
}
