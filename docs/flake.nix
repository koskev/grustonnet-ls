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
                require("snacks").setup({
                  input = {},
                  notifier = {}
                })
                require("telescope").setup({
                  extensions = {
                    ["ui-select"] = { layout_strategy = "center" },
                  }
                })
                -- Uses telescope for all select dialogs
                require("telescope").load_extension("ui-select")
                require("tiny-inline-diagnostic").setup({
                  options = {
                    multilines = {
                      enabled = true,
                    }
                  },
                  signs = {
                    left = "",
                    right = "",
                    diag = "●",
                    arrow = "    ",
                    up_arrow = "    ",
                    vertical = " │",
                    vertical_end = " ",
                  }
                })


                vim.cmd[[colorscheme tokyonight]]
                vim.lsp.inlay_hint.enable(true, nil)
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
                        columns = { { "label", "label_description", gap = 1 }, { "source_name" } },
                      }
                    }
                  }
                  })
                require('nvim-treesitter.configs').setup({
                  highlight = {
                    enable = true,
                  }


                })
              EOF
            '';

            packages.myPlugins.start = with pkgs.vimPlugins; [
              nvim-treesitter
              (nvim-treesitter.withPlugins (p: [ p.jsonnet ]))
              blink-cmp
              tokyonight-nvim
              snacks-nvim
              telescope-nvim
              telescope-ui-select-nvim
              tiny-inline-diagnostic-nvim
            ];
          };
        };

        nativeBuildInputs = with pkgs; [
          asciinema_3
          python3
          python313Packages.pynvim

          grustonnet.defaultPackage.${pkgs.system}

          neovim
          tmux
          bash
          mdbook
          git-cliff
          nodejs
          coreutils
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
