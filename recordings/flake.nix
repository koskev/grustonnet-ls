{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    grustonnet.url = "..";
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      grustonnet,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        neovim = pkgs.neovim.override {
          configure = {
            customRC = ''
              lua <<EOF
                vim.cmd[[colorscheme tokyonight]]
                vim.lsp.config['grustonnet'] = {
                  cmd = { "grustonnet-ls" },
                  filetypes = { 'jsonnet', 'libsonnet' },
                  root_markers = { 'jsonnetfile.json', '.git' },
                }
                vim.lsp.enable('grustonnet')
                require('blink.cmp').setup({
                  completion = {
                    ghost_text = { enabled = true },
                    documentation = { auto_show = true },
                    menu = {
                      draw = {
                        columns = { { "label", "label_description", gap = 1 }, { "kind_icon", "kind", gap = 1 }, { "source_name" } },
                      }
                    }
                  }
                  })
              EOF
            '';

            packages.myPlugins.start = with pkgs.vimPlugins; [
              (nvim-treesitter.withPlugins (parsers: builtins.attrValues { inherit (parsers) jsonnet; }))
              blink-cmp
              tokyonight-nvim
            ];
          };
        };

        nativeBuildInputs = with pkgs; [
          asciinema_3
          python3
          python313Packages.pynvim

          grustonnet.defaultPackage.${pkgs.system}

          neovim
        ];

      in
      {
        # For `nix build` & `nix run`:
        defaultPackage = pkgs.stdenv.mkDerivation {
          name = "recordings";
          src = ./.;
          inherit nativeBuildInputs;
          installPhase = ''
            mkdir -p $out
            # TODO: This hangs endlessly
            python build.py --output $out
          '';
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          inherit nativeBuildInputs;
        };
      }
    );
}
