{
  perSystem =
    { pkgs, ... }:
    {
      _module.args =
        let
          rustPlatform = pkgs.makeRustPlatform {
            inherit (pkgs) rustc;
            inherit (pkgs) cargo;
          };
        in
        {
          sharedBuildInputs = with pkgs; [
            go
            clang
            rustc
            cargo
            rustPlatform.bindgenHook
          ];
          sharedNativeBuildInputs = with pkgs; [
            pkg-config
            git
          ];
        };
    };
}
