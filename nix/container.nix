_: {
  perSystem =
    {
      pkgs,
      inputs',
      self',
      ...
    }:
    let
      nix2containerPkgs = inputs'.nix2container.packages;
    in
    {
      packages =
        let
          extraPackages = with pkgs; [
            # Useful for modifying a config or quality output
            jq

            coreutils
            bash
          ];
          mkBinaries =
            name:
            pkgs.runCommand "bins"
              {
                nativeBuildInputs = [ pkgs.removeReferencesTo ];
              }
              ''
                mkdir -p $out/bin
                cp ${self'.packages.grustonnet}/bin/grustonnet-${name} $out/bin/
                # Remove all go references to avoid bloating the image
                remove-references-to -t ${pkgs.go} $out/bin/*
              '';
          createContainer =
            name:
            let
              binaries = mkBinaries name;
            in
            nix2containerPkgs.nix2container.buildImage {
              name = "grustonnet-${name}";
              tag = "latest";
              copyToRoot = pkgs.buildEnv {
                name = "root";
                paths = extraPackages;
                pathsToLink = [ "/bin" ];
              };
              config = {
                Cmd = [ "${binaries}/bin/grustonnet-${name}" ];
                Env = [
                  "PATH=${binaries}/bin:/bin"
                ];
              };
            };
        in
        {
          dockerLinter = createContainer "lint";
          dockerDebugger = createContainer "debugger";
          dockerLs = createContainer "ls";
          dockerImageFull =
            let
              binaries = mkBinaries "*";
            in
            nix2containerPkgs.nix2container.buildImage {
              name = "grustonnet";
              tag = "latest";

              copyToRoot = pkgs.buildEnv {
                name = "root";
                paths = extraPackages;
                pathsToLink = [ "/bin" ];
              };

              config = {
                Cmd = [ "${binaries}/bin/grustonnet-lint" ];
                Env = [
                  "PATH=${binaries}/bin:/bin"
                ];
              };
            };
        };
    };
}
