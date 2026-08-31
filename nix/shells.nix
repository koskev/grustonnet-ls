_: {
  perSystem =
    {
      pkgs,
      sharedNativeBuildInputs,
      sharedBuildInputs,
      ...
    }:
    let
      rustPlatform = pkgs.makeRustPlatform {
        inherit (pkgs) rustc;
        inherit (pkgs) cargo;
      };

    in
    {
      devShells = {
        test = pkgs.mkShell {
          nativeBuildInputs = sharedNativeBuildInputs;
          buildInputs = sharedBuildInputs;
        };
        clippy = pkgs.mkShell {
          nativeBuildInputs =
            with pkgs;
            sharedNativeBuildInputs
            ++ [
              cargo
              rustc
              rustPlatform.bindgenHook
              clippy
              gnumake
            ];
          buildInputs = sharedBuildInputs;
        };

        # For `nix develop`:
        default = pkgs.mkShell {
          nativeBuildInputs =
            with pkgs;
            sharedNativeBuildInputs
            ++ [
              gdb
              cargo-tarpaulin
              clippy
              rustfmt
              cargo2junit

              go-jsonnet
              rust-analyzer
              bacon
              tracy
              reuse

              conform
              prek
              gnumake
              git-cliff
            ];
          buildInputs = sharedBuildInputs;
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          GODEBUG = "invalidptr=0,cgocheck=0";
        };
      };
    };
}
