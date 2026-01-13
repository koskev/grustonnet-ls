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
        jsonnet-tools-nvim = pkgs.vimUtils.buildVimPlugin {
          name = "jsonnet-tools.nvim";
          src = pkgs.fetchFromGitHub {
            owner = "koskev";
            repo = "jsonnet-tools.nvim";
            rev = "0d4fd6e4e5f843f43eb7ad4d84910bea9b32b1c6";
            sha256 = "Xsw+5T4LzK5MdnHd+Kva2bI3KQm3fLlQVOKuFniitgU=";
          };
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
            require('jsonnet-tools').setup({language_server_name="grustonnet"})
          '';
          extraPlugins = with pkgs.vimPlugins; [
            jsonnet-tools-nvim
            nvim-dap
          ];
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
