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
      packages = {
        dockerImage =
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
          in
          # pkgs.dockerTools.buildImage {
          nix2containerPkgs.nix2container.buildImage {
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
