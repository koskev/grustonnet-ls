_: {
  perSystem = { pkgs, inputs', ... }: {
    apps.gomod = {
      type = "app";
      program = pkgs.writeShellApplication {
        name = "gomod2nix";
        runtimeInputs = [
          inputs'.gomod2nix.packages.default
          pkgs.git
        ];

        text = ''
          gomod2nix generate --dir "$(git rev-parse --show-toplevel)/crates/jsonnet-bridge/go"
        '';
      };
    };
  };
}
