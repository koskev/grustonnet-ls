{ inputs, ... }:
let
  actions = {
    checkout = "actions/checkout@v5";
    nothing-but-nix = "wimpysworld/nothing-but-nix@687c797a730352432950c707ab493fcc951818d7";
    cachix-installer = "cachix/install-nix-action@v31";
    cachix = "cachix/cachix-action@v15";
    deploy-pages = "actions/deploy-pages@v4";
    upload-pages-artifacts = "actions/upload-pages-artifact@v4";
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
    {
      name = "Install nix";
      uses = actions.cachix-installer;
      "with".github_access_token = "\${{ secrets.GITHUB_TOKEN }}";
    }
  ];
  platforms = {
    linux = {
      os-name = "Linux-x86_64";
      runs-on = "ubuntu-24.04";
      target = "x86_64-unknown-linux-gnu";
    };
    mac = {
      os-name = "macOS-aarch64";
      runs-on = "macos-latest";
      target = "aarch64-apple-darwin";
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
    };
  };
}
