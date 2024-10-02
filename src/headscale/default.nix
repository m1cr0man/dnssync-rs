{ config, lib, ... }:
let
  inherit (lib) types mkOption;
  cfg = config.dnssync.backends.headscale;
in
{
  options.dnssync.backends.headscale = {
    enable = lib.mkEnableOption "Headscale server source of records";
    domain = mkOption {
      type = types.str;
      description = "The domain suffix for all records";
    };
    keyFile = mkOption {
      type = types.path;
      description = "Path to a file containing the Headscale API key. Must be owned by the dnssync user";
    };
    baseUrl = mkOption {
      type = types.str;
      description = "The base URL of the Headscale server to use";
    };
    addUserSuffix = lib.mkEnableOption "the user suffix in the record name";
  };

  config = lib.mkIf (cfg.enable) {
    dnssync.enabledBackends = "headscale";
    systemd.services.dnssync.environment = {
      "DNSSYNC_HEADSCALE_DOMAIN" = cfg.domain;
      "DNSSYNC_HEADSCALE_API_KEY" = "@${cfg.keyFile}";
      "DNSSYNC_HEADSCALE_BASE_URL" = cfg.baseUrl;
      "DNSSYNC_HEADSCALE_ADD_USER_SUFFIX" = "${builtins.toString cfg.addUserSuffix}";
    };
  };
}
