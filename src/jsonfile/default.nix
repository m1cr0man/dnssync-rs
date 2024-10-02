{ config, lib, ... }:
let
  inherit (lib) types mkOption;
  cfg = config.dnssync.backends.jsonfile;
in
{
  options.dnssync.backends.jsonfile = {
    enable = lib.mkEnableOption "JSON file source of records";
    source = mkOption {
      type = types.path;
      description = "A JSON file of DNS records to write to frontends";
    };
  };

  config = lib.mkIf (cfg.enable) {
    dnssync.enabledBackends = "jsonfile";
    systemd.services.dnssync.environment = {
      "DNSSYNC_JSONFILE_SOURCE" = cfg.source;
    };
  };
}
