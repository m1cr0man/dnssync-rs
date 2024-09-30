{
  description = "DNSSync - Dynamic DNS for services and networks";

  inputs = {
    nixpkgs.follows = "fenix/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "fenix/nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      # inputs.rust-analyzer-src.follows = "";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, advisory-db, ... }:
    {
      overlays = {
        dnssync-rs-nixpkgs =
          let
            cargoConfig = (builtins.fromTOML (builtins.readFile "${self}/Cargo.toml"));
            pname = cargoConfig.package.name;
          in
          final: prev:
            let
              craneLib = crane.mkLib prev;
            in
            {
              ${pname} = final.rustPlatform.buildRustPackage {
                inherit pname;
                version = cargoConfig.package.version;
                src = craneLib.cleanCargoSource (craneLib.path self.outPath);
                buildFeatures = [ "cli" ];

                cargoLock.lockFile = "${self}/Cargo.lock";

                OPENSSL_NO_VENDOR = "1";
                PKG_CONFIG_PATH = "${prev.openssl.dev}/lib/pkgconfig";
                PKG_CONFIG = "${prev.pkg-config}/bin/pkg-config";

                meta = with prev.lib; {
                  description = "Dynamic DNS for services and networks";
                  homepage = "https://github.com/m1cr0man/dnssync-rs";
                  license = licenses.mit;
                  maintainers = [ maintainers.m1cr0man ];
                };
              };
            };
      };

      nixosModules.dnssync = { config, pkgs, lib, ... }:
        let
          inherit (lib) types mkOption;
          cfg = config.dnssync;
          esa = lib.escapeShellArg;
        in
        {
          options.dnssync = {
            enable = lib.mkEnableOption "dynamic DNS for services and networks";
            backends = mkOption {
              type = types.commas;
              description = "Enabled backend implementations, comma separated";
            };
            frontends = mkOption {
              type = types.commas;
              description = "Enabled frontend implementations, comma separated";
            };
          };

          config = lib.mkIf cfg.enable {
            imports = [
              "${self}/src/cloudflare/default.nix"
              "${self}/src/headscale/default.nix"
              "${self}/src/jsonfile/default.nix"
              "${self}/src/machinectl/default.nix"
            ];

            users.users.dnssync = {
              group = "dnssync";
              home = "/var/empty";
              createHome = false;
              isSystemUser = true;
            };
            users.groups.dnssync = { };

            systemd.services.dnssync = {
              inherit (self) description;
              after = [ "network-online.target" ];
              wantedBy = [ "multi-user.target" ];
              serviceConfig = {
                ExecStart = "${pkgs.dnssync-rs}/bin/dnssync-rs --backends ${esa cfg.backends} --frontends ${esa cfg.frontends}";
                Type = "oneshot";
                RemainAfterExit = "no";
                User = "dnssync";
                Group = "dnssync";
                ProtectSystem = "full";
                PrivateTmp = "yes";
              };
            };
          };
        };
    } //
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        envVars = {
          OPENSSL_NO_VENDOR = "1";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          PKG_CONFIG = "${pkgs.pkg-config}/bin/pkg-config";
        };

        stdenv =
          if pkgs.stdenv.isLinux then
            pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv
          else
            pkgs.stdenv;

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;

        mkToolchain = fenix.packages.${system}.combine;

        toolchain = fenix.packages.${system}.stable;

        buildToolchain = mkToolchain (with toolchain; [
          cargo
          rustc
        ]);

        craneLibBuild = craneLib.overrideToolchain buildToolchain;

        devToolchain = mkToolchain (with toolchain; [
          cargo
          clippy
          rust-src
          rustc
          llvm-tools
          rust-analyzer

          # Always use nightly rustfmt because most of its options are unstable
          fenix.packages.${system}.latest.rustfmt
        ]);

        craneLibDev = craneLib.overrideToolchain devToolchain;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src stdenv;
          strictDeps = true;

          buildInputs = [
            # Add additional build inputs here
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
        } // envVars;

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLibBuild.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        dnssync-rs = craneLibBuild.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          cargoExtraArgs = "-F cli";
        });
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit dnssync-rs;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          dnssync-rs-clippy = craneLibDev.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          dnssync-rs-doc = craneLibDev.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });

          # Check formatting
          dnssync-rs-fmt = craneLibDev.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          # Broken for now
          # dnssync-rs-audit = craneLib.cargoAudit {
          #   inherit src advisory-db;
          # };

          # Audit licenses
          dnssync-rs-deny = craneLibDev.cargoDeny {
            inherit src;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `dnssync-rs` if you do not want
          # the tests to run twice
          dnssync-rs-nextest = craneLibDev.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });

          overlay = (import nixpkgs {
            inherit system;
            overlays = [ self.overlays.dnssync-rs-nixpkgs ];
          }).dnssync-rs;
        };

        packages = {
          inherit dnssync-rs;
          default = dnssync-rs;
          dnssync-rs-lib = craneLibBuild.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
          dnssync-rs-llvm-coverage = craneLibDev.cargoLlvmCov (commonArgs // {
            inherit cargoArtifacts;
          });
          devTools = pkgs.linkFarm "vscode-dev-tools" {
            inherit (pkgs) nixpkgs-fmt rnix-lsp gcc pkg-config;
            openssl = pkgs.openssl.dev;
            rust = devToolchain;
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = dnssync-rs;
        };

        devShells.default = craneLibDev.devShell
          {
            # Inherit inputs from checks.
            checks = self.checks.${system};

            # Additional dev-shell environment variables can be set directly
            # MY_CUSTOM_DEVELOPMENT_VAR = "something else";
            RUST_SRC_PATH = "${devToolchain}/lib/rustlib/src/rust/library";
            MEGHAN = "bab";

            # Extra inputs can be added here; cargo and rustc are provided by default.
            packages = [
              # pkgs.ripgrep
            ];
          } // envVars;
      })
    );
}
