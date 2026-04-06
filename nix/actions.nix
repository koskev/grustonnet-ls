{ inputs, ... }:

let
  inherit (inputs.nix-actions.lib) actions;
  inherit (inputs.nix-actions.lib) steps;
  inherit (inputs.nix-actions.lib) packages;
  inherit (inputs.nix-actions.lib) commonSteps;
  inherit (inputs.nix-actions.lib) platforms;
in
{
  imports = [ inputs.actions-nix.flakeModules.default ];
  flake.actions-nix = {
    pre-commit.enable = true;
    defaultValues = {
      jobs = {
        runs-on = "ubuntu-latest";
      };
    };
    workflows = {
      ".github/workflows/docker-publish.yaml" = inputs.nix-actions.lib.mkDocker { };
      ".github/workflows/mr.yaml" = inputs.nix-actions.lib.mkConform { };
      ".github/workflows/linting.yaml" = inputs.nix-actions.lib.mkClippy {
        targetName = ".#clippy";
      };
      ".github/workflows/docs.yaml" = {
        on = {
          push.branches = [ "main" ];
        };
        jobs = {
          docs.steps = commonSteps ++ [
            {
              run = "cd docs && nix develop ..#docs --command make build && cd ..";
            }
            {
              name = "Upload artifacts for pages";
              uses = actions.upload-pages-artifacts;
              "with".path = "docs/book";
            }
          ];
          pages = {
            permissions = {
              id-token = "write";
              pages = "write";
            };
            environment = {
              name = "github-pages";
              url = "\${{steps.deployment.outputs.page_url}}";
            };
            needs = [ "docs" ];
            steps = [
              {
                name = "Deploy to GitHub Pages";
                id = "deployment";
                uses = actions.deploy-pages;
              }
            ];
          };
        };
      };
      ".github/workflows/test.yaml" = inputs.nix-actions.lib.mkBuild {
        targetName = ".#grustonnet-test";
        extraJobs = {
          windows-test = {
            inherit (platforms.windows-cross) runs-on;
            steps = [
              {
                uses = actions.checkout;
              }
              steps.setupGo
              {
                name = "Install Rust test dependencies";
                run = "cargo install ${packages.rust.cargo2junit} ${packages.rust.tarpaulin} --locked";
              }
              {
                name = "Test Windows x86_64 GNU";
                uses = actions.wine-test;
                env = {
                  RUSTFLAGS = "";
                };
                "with" = {
                  rust-project-path = ".";
                  inherit (platforms.windows-cross) target;
                };
              }
            ];
          };
        };
        extraBuildSteps = inputs.nix-actions.lib.mkCachixSteps { };

      };
      ".github/workflows/release.yaml" = {
        on = {
          push.tags = [ "v*" ];
          workflow_dispatch = { };
        };
        jobs = {
          changelog.steps = [
            steps.checkout-full
            {
              name = "Generate a changelog";
              uses = actions.git-cliff;
              "with" = {
                config = "cliff.toml";
                args = "--verbose --current";
              };
              env = {
                OUTPUT = "CHANGELOG.md";
              };
            }
            {
              name = "Upload changelog";
              uses = actions.upload-artifact;
              "with" = {
                name = "changelog";
                path = "CHANGELOG.md";
                retention-days = 1;
              };
            }
          ];
          release = {
            strategy.matrix.platform = [
              platforms.linux
              platforms.linux_aarch64
              platforms.mac
              platforms.windows-cross
            ];
            runs-on = "\${{ matrix.platform.runs-on }}";
            needs = [ "changelog" ];
            steps = [
              steps.checkout
              {
                name = "Generate Version";
                run = ''
                  GITHUB_TAG_NAME=''${{ github.ref_name }}
                  TAG_NAME=''${GITHUB_TAG_NAME:-v0.0.0}
                  TAG_VERSION=''${TAG_NAME: 1}
                  echo "TAG_VERSION=$TAG_VERSION" >> $GITHUB_ENV
                '';
              }
              {
                name = "Generate output name";
                run = ''
                  RELEASE_TAR=$(echo "grustonnet_''${{ matrix.platform.os-name }}.tar.gz" | tr '[:upper:]' '[:lower:]' | tr '-' '_')
                  echo ''${RELEASE_TAR}
                  echo "RELEASE_TAR=''${RELEASE_TAR}" >> $GITHUB_ENV
                '';
              }
              {
                name = "Set Cargo Version";
                run = ''
                  # TODO: Suffix handling (e.g. -RC1)
                  cargo install ${packages.rust.cargo-set-version}
                  echo "Setting program version to ''${TAG_VERSION}"
                  cargo set-version ''${TAG_VERSION}
                '';
              }
              {
                name = "Get changelog";
                uses = actions.download-artifact;
                "with" = {
                  name = "changelog";
                  path = "changelog";
                };
              }
              steps.setupGo
              {
                name = "Install cross compile tools";
                "if" =
                  "\${{ contains(matrix.platform.target, 'windows') && contains(matrix.platform.runs-on, 'ubuntu') }}";
                run = "sudo apt update && sudo apt install gcc-mingw-w64 && rustup target add x86_64-pc-windows-gnu";
              }
              {
                name = "Build binary";
                run = "make build-\${{ matrix.platform.target }}";
                env = {
                  RUST_BACKTRACE = 1;
                  GODEBUG = "invalidptr=0,cgocheck=0";
                };
              }
              {
                name = "Compress binary";
                run = builtins.readFile ./scripts/compress-release.sh;
              }
              {
                name = "Publish artifacts and release";
                uses = actions.gh-release;
                "with" = {
                  files = "\${{ env.RELEASE_TAR }}";
                  body_path = "changelog/CHANGELOG.md";
                };
              }
            ];
          };
        };
      };
    };
  };
}
