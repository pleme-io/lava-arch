# nix/modules/home-manager.nix — auto-generated from lava-arch.caixa.lisp
{ config, lib, pkgs, ... }:
let cfg = config.programs.lava-arch; in {
  options.programs.lava-arch = {
    enable = lib.mkEnableOption "lava-arch";
    package = lib.mkOption { type = lib.types.package; default = pkgs.lava-arch or null; };
  };
  config = lib.mkIf cfg.enable { home.packages = [ cfg.package ]; };
}
