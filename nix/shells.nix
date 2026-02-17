_: {
  perSystem =
    {
      pkgs,
      sharedNativeBuildInputs,
      sharedBuildInputs,
      ...
    }:
    {
      devShells = {
        clippy = pkgs.mkShell {
          nativeBuildInputs =
            with pkgs;
            sharedNativeBuildInputs
            ++ [
              cargo
              clippy
              gnumake
            ];
          buildInputs =
            with pkgs;
            sharedBuildInputs
            ++ [
              rustc
            ];
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          LIBCLANG_PATH = with pkgs; "${llvmPackages.libclang.lib}/lib";
        };

        # For `nix develop`:
        default = pkgs.mkShell {
          nativeBuildInputs =
            let
              rust2gocli = pkgs.rustPlatform.buildRustPackage rec {
                name = "rust2go-cli";
                src = pkgs.fetchCrate {
                  pname = name;
                  version = "0.4.2";
                  hash = "sha256-WT09837Y6lwH6usdoOQ7UTm9HcuHKify/jA8v8R4Fek=";
                };
                cargoHash = "sha256-WP7j+JSByT9fyblcL2saDMmOz2dq0jTSJTjxkTJwn5M=";
              };

            in
            with pkgs;
            sharedNativeBuildInputs
            ++ [
              cargo
              gdb
              cargo-tarpaulin
              clippy
              rustfmt

              go-jsonnet
              rust-analyzer
              bacon
              tracy
              reuse

              conform
              prek
              gnumake
              git-cliff

              rust2gocli
            ];
          buildInputs =
            with pkgs;
            sharedBuildInputs
            ++ [
              rustc
            ];
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          LIBCLANG_PATH = with pkgs; "${llvmPackages.libclang.lib}/lib";
          GODEBUG = "invalidptr=0,cgocheck=0";
        };
      };
    };
}
