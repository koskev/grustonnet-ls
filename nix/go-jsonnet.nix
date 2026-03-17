{ self, ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      packages = {
        go-jsonnet = pkgs.stdenv.mkDerivation {
          name = "rust2go-vendor";
          src = "${self}/crates/jsonnet-bridge/go";
          dontUnpack = true;

          nativeBuildInputs = [ pkgs.go ];

          buildPhase = ''
            # Subdir since go refuses to work in the "system tmp" directory, which is /build in this case
            mkdir go && cd go
            cp -r $src/* .
            export GOPATH="$TMPDIR/go-path"
            export GOCACHE="$TMPDIR/go-cache"
            go mod vendor
          '';

          installPhase = ''
            mkdir -p $out
            cp -r vendor $out
          '';

          outputHashMode = "recursive";
          outputHashAlgo = "sha256";
          # Do NOT set to `null` for testing. `go mod vendor` WILL break
          outputHash = "sha256-y+xVII0JTQv9DwYgk5cbPPEsb8OpCYzFfw4PXD6Raoc=";
        };
      };
    };
}
