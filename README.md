# DNSSync - Dynamic DNS for services and networks

DNSSync is a tool which compiles DNS records from 1 or more backends,
and writes them to 1 or more frontends. A quick summary of features/use cases:

- Reading records from:
    - Systemd's `machinectl`
    - Headscale
    - A JSON file
- Writing records to:
    - Cloudflare
- Fully authoritative (create/update/delete) backend -> frontend one way sync.
- Support on frontend for both managed and unmanaged record mixing.
- Supports multiple running instances of DNSSync.
- Supports nested and overlapping domains and subdomains.

## Configuration

### NixOS quick start

DNSSync was built to be used in NixOS. You can use the module exported from
this repo's flake in your own configuration quite easily. The below snippet
is a stripped down version of [a basic NixOS flake](https://gist.github.com/m1cr0man/8cae16037d6e779befa898bfefd36627),
showing the important pieces.

```nix
{
  inputs = {
    # Extend the inputs
    dnssync.url = "github:m1cr0man/dnssync-rs";
  };

  outputs = { dnssync, ... }@inputs {
    nixosConfigurations = {
      myhost = {
        modules = [
          # Add DNSSync to the module list
          dnssync.nixosModules.dnssync-with-overlay
          # Now configure DNSSync
          ({ config, ... }: {
            services.dnssync = {
              enable = true;
              backends = {
                headscale = {
                  enable = true;
                  # Domain must match or be a subdomain of some frontend
                  domain = "ts.example.com";
                  addUserSuffix = true;
                  baseUrl = "https://headscale.example.com";
                  keyFile = "/var/run/secrets/my_headscale_key";
                };
                # You can enable > 1 backend per instance.
              };
              frontends = {
                cloudflare = {
                  enable = true;
                  domain = "example.com";
                  instanceId = config.networking.hostName;
                  # Requires Zone.DNS (DNS:Edit) permission on the domain
                  keyFile = "/var/run/secrets/my_cloudflare_key";
                };
                # You can enable > 1 frontend per instance.
              };
            };
          })
        ];
      };
    };
  };
}
```

### Other distributions

DNSSync is configured through environment variables and CLI args. Check out
[the example config](./config.example.env) for a list of available options.

To keep API keys secure, you can specify a path to any `_API_KEY` option by
prefixing it with an `@` symbol. DNSSync will read this file at runtime.

Here's some example invocations:

```bash
# Test your configuration
$ dnssync --backends headscale,machinectl,jsonfile --frontends cloudflare --test
# Dry run the changes
$ dnssync --backends headscale,machinectl,jsonfile --frontends cloudflare --dry-run
# Do DNS Sync!
$ dnssync --backends headscale,machinectl,jsonfile --frontends cloudflare
```
