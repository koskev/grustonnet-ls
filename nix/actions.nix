{ inputs, lib, ... }:
let
  actions = {
    checkout = "actions/checkout@93cb6efe18208431cddfb8368fd83d5badbf9bfd"; # v5
    deploy-pages = "actions/deploy-pages@d6db90164ac5ed86f2b6aed7e0febac5b3c0c03e"; # v4
    upload-pages-artifacts = "actions/upload-pages-artifact@7b1f4a764d45c48632c6b24a0339c27f5614fb0b"; # v4
    download-artifact = "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093"; # v4
    upload-artifact = "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02"; # v4
    setup-go = "actions/setup-go@4b73464bb391d4059bd26b0524d20df3927bd417"; # v6
    nothing-but-nix = "wimpysworld/nothing-but-nix@687c797a730352432950c707ab493fcc951818d7";
    cachix-installer = "cachix/install-nix-action@4e002c8ec80594ecd40e759629461e26c8abed15"; # v31
    cachix = "cachix/cachix-action@ad2ddac53f961de1989924296a1f236fcfbaa4fc"; # v15
    wine-test = "Reloaded-Project/devops-rust-test-in-latest-wine@b707ebd77c7b98be7666c672d7c947aefeb1ac45"; # v1
    gh-release = "softprops/action-gh-release@a06a81a03ee405af7f2048a818ed3f03bbf83c7b"; # v2
    docker-login = "docker/login-action@c94ce9fb468520275223c153574b00df6fe4bcc9"; # v3
    git-cliff = "orhun/git-cliff-action@c93ef52f3d0ddcdcc9bd5447d98d458a11cd4f72"; # v4
  };
  packages = {
    go = {
      conform = "github.com/siderolabs/conform/cmd/conform@v0.1.0-alpha.30";
    };
    rust = {
      cargo2junit = "cargo2junit@0.1.15";
      tarpaulin = "cargo-tarpaulin@0.35.1";
      cargo-set-version = "cargo-set-version";
      rust2go-cli = "rust2go-cli";
    };
  };

  mkStep =
    {
      step,
      branches ? null,
    }:
    step // lib.optional branches != null {
      "if" =
        let
          branch_array = map (branch: "github.ref == 'refs/heads/${branch}'") branches;
        in
        builtins.concatStringsSep " || " branch_array;
    };
  steps = {
    checkout-full = {
      name = "checkout full";
      uses = actions.checkout;
      "with".fetch-depth = 0;
    };
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
      uses = actions.docker-login;
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
    steps.checkout-full
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
        jobs =
          let
            TAG = "\${GITHUB_REF_NAME//\\//-}";
          in
          rec {
            build = {
              strategy.matrix.platform = [
                platforms.linux
                platforms.linux_aarch64
              ];
              runs-on = "\${{ matrix.platform.runs-on }}";
              steps = [
                steps.checkout-full
                steps.installNix
                steps.dockerLogin
                {
                  name = "Build and push image";
                  run = ''
                    nix run ".#dockerImageFull.copyTo" -- "docker://''${{ env.IMAGE }}:${TAG}-''${{ matrix.platform.target }}"
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
                      docker manifest create "''${{ env.IMAGE }}:${TAG}" ${images}
                      docker manifest push "''${{ env.IMAGE }}:${TAG}"
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
            run = "go install ${packages.go.conform}";
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
              run = "cargo install ${packages.rust.rust2go-cli}";
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
