# nix/modules/darwin.nix — auto-generated from lava-arch.caixa.lisp
{ config, lib, pkgs, ... }:
let cfg = config.services.lava-arch; in {
  options.services.lava-arch = {
    enable = lib.mkEnableOption "lava-arch";
    package = lib.mkOption { type = lib.types.package; default = pkgs.lava-arch or null; };
  };
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}
