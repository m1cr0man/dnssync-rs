{ config, lib, ... }:
let
  inherit (lib) types mkOption;
  cfg = config.dnssync.frontend.cloudflare;
in
{
  options.dnssync.frontend.cloudflare = {
    enable = lib.mkEnableOption "cloudflare server frontend for records";
    domain = mkOption {
      type = types.str;
      description = "The base domain consumed/supported by this frontend";
    };
    keyFile = mkOption {
      type = types.path;
      description = "Path to a file containing the Cloudflare API key. Must be owned by the dnssync user";
    };
    instanceId = mkOption {
      type = types.str;
      description = "A unique identifier for this deployment of dnssync."
        + " Used to avoid collisions with other instances on the same domain";
    };
  };

  config = lib.mkIf (cfg.enable) {
    dnssync.frontends = "cloudflare";
    systemd.services.dnssync.requires = [ "network-online.target" ];
    systemd.services.dnssync.environment = {
      "DNSSYNC_CLOUDFLARE_DOMAIN" = cfg.domain;
      "DNSSYNC_CLOUDFLARE_INSTANCE_ID" = cfg.instanceId;
      "DNSSYNC_CLOUDFLARE_API_KEY" = "@${cfg.instanceId}";
    };
  };
}
