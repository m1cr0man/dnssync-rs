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
- Architected to be useful as a library as well as a CLI.

## Configuration

### NixOS flake quick start

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

### Nix quick start

If you are just using Nix as a package manager, you can quickly compile and
launch DNSSync using this command:

```bash
nix run github:m1cr0man/dnssync-rs -- --help
```

### Other distributions

DNSSync is configured through environment variables and CLI args. Check out
[the example config](./config.example.env) for a list of available options.

To keep API keys secure, you can specify a path to any `_API_KEY` option by
prefixing it with an `@` symbol. DNSSync will read this file at runtime.

Here's some example invocations:

```bash
# Compile with cargo
cargo build . -F cli
# Test your configuration
$ dnssync --backends headscale,machinectl,jsonfile --frontends cloudflare --test
# Dry run the changes
$ dnssync --backends headscale,machinectl,jsonfile --frontends cloudflare --dry-run
# Do DNS Sync!
$ dnssync --backends headscale,machinectl,jsonfile --frontends cloudflare
```

## Development

This project uses Nix to manage the development environment.
Run `nix develop` for a shell with the Rust toolchain ready to go.

### Adding a frontend or backend

- Create a subdirectory under [src](./src/).
- Create a struct and implement either the Frontend or Backend
 trait from [common](./src/common/models.rs).
- Extend [config.rs](./src/config.rs) to load the configuration for your struct.
- Add a `default.nix` with the Nix options and config for the struct.
- Import the Nix module in the [flake.nix](./flake.nix#92).
