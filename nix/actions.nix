{ inputs, ... }:
let
  actions = {
    checkout = "actions/checkout@v5";
    nothing-but-nix = "wimpysworld/nothing-but-nix@687c797a730352432950c707ab493fcc951818d7";
    cachix-installer = "cachix/install-nix-action@v31";
    cachix = "cachix/cachix-action@v15";
    deploy-pages = "actions/deploy-pages@v4";
    upload-pages-artifacts = "actions/upload-pages-artifact@v4";
    setup-go = "actions/setup-go@v6";
    wine-test = "Reloaded-Project/devops-rust-test-in-latest-wine@v1";
  };

  steps = {
    checkout = {
      name = "checkout";
      uses = actions.checkout;
    };
    installNix = {
      name = "Install nix";
      uses = actions.cachix-installer;
      "with".github_access_token = "\${{ secrets.GITHUB_TOKEN }}";
    };
    dockerLogin = {
      name = "Login to GHCR";
      uses = "docker/login-action@v3";
      "with" = {
        registry = "ghcr.io";
        username = "\${{ github.repository_owner }}";
        password = "\${{ secrets.GITHUB_TOKEN }}";
      };
    };
    setupGo = {
      uses = actions.setup-go;
      "with".go-version = "1.25";
    };
  };
  commonSteps = [
    {
      uses = actions.checkout;
      "with".fetch-depth = 0;
    }
    {
      name = "Most important Action!";
      uses = actions.nothing-but-nix;
      "with".hatchet-protocol = "rampage";
    }
    steps.installNix
  ];
  platforms = {
    linux = {
      os-name = "Linux-x86_64";
      runs-on = "ubuntu-24.04";
      target = "x86_64-unknown-linux-gnu";
    };
    linux_aarch64 = {
      os-name = "Linux-aarch64";
      runs-on = "ubuntu-24.04-arm";
      target = "aarch64-unknown-linux-gnu";
    };
    mac = {
      os-name = "macOS-aarch64";
      runs-on = "macos-latest";
      target = "aarch64-apple-darwin";
    };
    windows-cross = {
      os-name = "Windows-x86_64";
      runs-on = "ubuntu-24.04";
      target = "x86_64-pc-windows-gnu";
    };
  };
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
      ".github/workflows/docker-publish.yaml" = {
        name = "Publish docker image";
        on.push.tags = [ "*" ];
        env = {
          IMAGE = "ghcr.io/\${{ github.repository }}";
        };
        jobs = rec {
          build = {
            strategy.matrix.platform = [
              platforms.linux
              platforms.linux_aarch64
            ];
            runs-on = "\${{ matrix.platform.runs-on }}";
            steps = [
              {
                uses = actions.checkout;
                "with".fetch-depth = 0;
              }
              steps.installNix
              steps.dockerLogin
              {
                name = "Build and push image";
                run = ''
                  TAG="''${GITHUB_REF_NAME//\//-}"
                  nix run ".#dockerImageFull.copyTo" -- "docker://''${{ env.IMAGE }}:''$TAG-''${{ matrix.platform.target }}"
                '';
              }
            ];
          };
          manifest = {
            needs = [ "build" ];
            steps = [
              steps.dockerLogin
              {
                name = "Make manifest";
                run =
                  let
                    images = builtins.concatStringsSep " " (
                      map (platform: "\${{ env.IMAGE }}:\$TAG-${platform.target}") build.strategy.matrix.platform
                    );
                  in
                  ''
                    TAG="''${GITHUB_REF_NAME//\//-}"
                    docker manifest create "''${{ env.IMAGE }}:''${TAG}" ${images}
                    docker manifest push "''${{ env.IMAGE }}:''${TAG}"
                  '';
              }
            ];
          };
        };

      };
      ".github/workflows/mr.yaml" = {
        on = {
          pull_request = { };
        };
        jobs.conform.steps = [
          {
            uses = actions.checkout;
            "with" = {
              fetch-depth = 0;
              ref = "\${{ github.event.pull_request.head.sha }}";
            };
          }
          steps.setupGo
          {
            name = "Install conform";
            run = "go install github.com/siderolabs/conform/cmd/conform@v0.1.0-alpha.30";
          }
          {
            name = "Run conform";
            run = "conform enforce --base-branch remotes/origin/main";
          }
        ];
      };
      ".github/workflows/linting.yaml" = {
        on = {
          push = { };
          pull_request = { };
        };
        jobs.clippy.steps = commonSteps ++ [
          {
            run = "nix develop .#clippy --command make clippy";
          }
        ];
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
      ".github/workflows/cachix.yaml" = {
        name = "Build Nix Configurations";
        on = {
          push.branches = [ "main" ];
        };
        jobs = {
          build = {
            strategy.matrix.platform = [
              platforms.linux
              platforms.mac
            ];
            runs-on = "\${{ matrix.platform.runs-on }}";
            steps = commonSteps ++ [
              {
                uses = actions.cachix;
                "with" = {
                  name = "koskev";
                  authToken = "\${{ secrets.CACHIX_AUTH_TOKEN }}";
                  signingKey = "\${{ secrets.CACHIX_SIGNING_KEY }}";
                  skipPush = true;
                };
              }
              {
                run = "nix build .";
              }
              {
                name = "Push to cachix";
                run = "nix path-info . | cachix push koskev";
              }
            ];
          };
        };
      };
      ".github/workflows/test.yaml" = {
        on = {
          push = { };
          pull_request = { };
        };
        env = {
          CARGO_TERM_COLOR = "always";
        };
        jobs = {
          generated-files.steps = [
            {
              uses = actions.checkout;
            }
            {
              run = "cargo install rust2go-cli";
            }
            {
              run = "make rust2go";
            }
            {
              run = ''
                if [ -n "$(git status --porcelain)" ]; then
                  echo "rust2go-cli found a diff. Make sure to run 'make rust2go' if you change any of the bridge code"
                  git -c color.ui=always diff
                  exit 1
                fi
              '';
            }
          ];
          nix-test = {
            strategy.matrix.platform = [
              platforms.linux
              platforms.linux_aarch64
              platforms.mac
            ];
            runs-on = "\${{ matrix.platform.runs-on }}";
            steps = [
              steps.checkout
              steps.installNix
              {
                name = "Run tests";
                run = "nix build .#grustonnet-test";
              }
            ];
          };
          windows-test = {
            inherit (platforms.windows-cross) runs-on;
            steps = [
              {
                uses = actions.checkout;
              }
              steps.setupGo
              {
                name = "Install Rust test dependencies";
                run = "cargo install cargo2junit@0.1.15 cargo-tarpaulin@0.35.1 --locked";
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
      };
    };
  };
}
