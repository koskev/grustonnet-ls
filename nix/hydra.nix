_: {
  flake =
    { config, lib, ... }:
    {
      hydraJobs = {
        grustonnet = builtins.mapAttrs (
          arch: packages: lib.attrsets.filterAttrs (name: value: name == "grustonnet") packages
        ) config.packages;
      };
    };
}
