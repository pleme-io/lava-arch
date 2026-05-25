# nix/modules/nixos.nix — auto-generated from lava-arch.caixa.lisp
# description: "Composition layer for the lava suite. deflava-architecture form + Rust builders. Architectures consume + return typed ResourceRef chains; downstream architectures slot into upstream outputs at composition time (not apply time). Pangea Architecture/ResourceBuilder analog."
{ config, lib, pkgs, ... }:
let
  cfg = config.services.lava-arch;
in {
  options.services.lava-arch = {
    enable = lib.mkEnableOption "lava-arch";
    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.lava-arch or null;
    };
  };
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}
