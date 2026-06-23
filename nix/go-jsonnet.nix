{ self, ... }:
{
  perSystem =
    { pkgs, inputs', ... }:
    let
      rust2gocli = pkgs.rustPlatform.buildRustPackage rec {
        name = "rust2go-cli";
        src = pkgs.fetchCrate {
          pname = name;
          version = "0.4.2";
          hash = "sha256-WT09837Y6lwH6usdoOQ7UTm9HcuHKify/jA8v8R4Fek=";
          # TODO: Workaround until this is in unstable. I don't want to use master
          registryDl = "https://static.crates.io/crates";
        };
        cargoHash = "sha256-WP7j+JSByT9fyblcL2saDMmOz2dq0jTSJTjxkTJwn5M=";
      };
    in
    {
      packages = {
        go-jsonnet = inputs'.gomod2nix.legacyPackages.buildGoApplication rec {
          name = "rust2go-vendor";
          pname = "rust2go-vendor";
          src = "${self}/crates/jsonnet-bridge";
          pwd = "${src}/go";
          modules = ../crates/jsonnet-bridge/go/gomod2nix.toml;
          dontUnpack = true;

          #nativeBuildInputs = [ pkgs.go ];

          buildPhase = ''
            # We need to compile go in a subdir since go refuses to work in the "system tmp" directory, which is /build in this case
            cp -r $src/* .
            chmod 755 -R .
            # We need to run this manually since the go mod vendor runs before the rust code. Without this we are missing entries in the go.sum
            ${rust2gocli}/bin/rust2go-cli --src src/go.rs --dst go/gen.go
            cd go
            export GOPATH="$TMPDIR/go-path"
            export GOCACHE="$TMPDIR/go-cache"
            unset GOPROXY
            go mod vendor
          '';

          installPhase = ''
            mkdir -p $out
            cp -r vendor $out
          '';
        };
      };
    };
}
