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
        test = pkgs.mkShell {
          nativeBuildInputs =
            with pkgs;
            sharedNativeBuildInputs
            ++ [
              cargo
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
            with pkgs;
            sharedNativeBuildInputs
            ++ [
              cargo
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
