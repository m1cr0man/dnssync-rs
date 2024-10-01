{ config, lib, pkgs, ... }:
let
  inherit (lib) types mkOption;
  cfg = config.dnssync.backend.machinectl;
  cidrs = builtins.map lib.escapeShellArg cfg.excludedCidrs;
in
{
  options.dnssync.backend.machinectl = {
    enable = lib.mkEnableOption "Systemd Machined source of records";
    domain = mkOption {
      type = types.str;
      description = "The domain suffix for all records";
    };
    excludedCidrs = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "IPV4/IPV6 CIDR blocks to skip creating records for";
    };
    includedCidrs = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "IPV4/IPV6 CIDR blocks to skip creating records for";
    };
  };

  config = lib.mkIf (cfg.enable) {
    dnssync.backends = "machinectl";
    # Not explicitly adding systemd to the path, but if needed use config.systemd.package.
    # It should be present by default.
    # Run when machines.target is reached
    after = [ "machines.target" ];
    wantedBy = [ "machines.target" ];
    systemd.services.dnssync.environment = {
      "DNSSYNC_MACHINECTL_DOMAIN" = cfg.domain;
      "DNSSYNC_MACHINECTL_EXCLUDED_CIDRS" = builtins.concatStringsSep "," cfg.excludedCidrs;
      "DNSSYNC_MACHINECTL_INCLUDED_CIDRS" = builtins.concatStringsSep "," cfg.includedCidrs;
    };
  };
}
