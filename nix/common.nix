{
  perSystem =
    { pkgs, ... }:
    {
      _module.args = {
        sharedBuildInputs = with pkgs; [
          go
          clang
        ];
        sharedNativeBuildInputs = with pkgs; [
          pkg-config
        ];
      };
    };
}
