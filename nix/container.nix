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
          binaries =
            pkgs.runCommand "bins"
              {
                nativeBuildInputs = [ pkgs.removeReferencesTo ];
              }
              ''
                          mkdir -p $out/bin
                          cp ${self'.packages.grustonnet}/bin/grustonnet-* $out/bin/
                # Remove all go references to avoid bloating the image
                          remove-references-to -t ${pkgs.go} $out/bin/*
              '';
          createContainer =
            name:
            nix2containerPkgs.nix2container.buildImage {
              name = "grustonnet-${name}";
              tag = "latest";
              config = {
                Cmd = [ "${binaries}/bin/grustonnet-${name}" ];
              };
            };
        in
        {
          dockerLinter = createContainer "lint";
          dockerDebugger = createContainer "debugger";
          dockerLs = createContainer "ls";
          dockerImageFull = nix2containerPkgs.nix2container.buildImage {
            name = "grustonnet";
            tag = "latest";

            copyToRoot = pkgs.buildEnv {
              name = "root";
              paths = [ binaries ];
              pathsToLink = [ "/bin" ];
            };

            config = {
              Cmd = [ "${binaries}/bin/grustonnet-lint" ];
              Env = [
                "PATH=${binaries}/bin"
              ];
            };
          };

        };
    };
}
