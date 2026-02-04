{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src =
          let
            fs = pkgs.lib.fileset;
            serverDir = ./server;
          in
          fs.toSource {
            root = serverDir;
            fileset = fs.intersection (fs.gitTracked serverDir) (
              fs.unions [
                (serverDir + "/Cargo.toml")
                (serverDir + "/Cargo.lock")
                (serverDir + "/src")
                (serverDir + "/migrations")
              ]
            );
          };

        # let
        #   sqlxFilter = fpath: _type: builtins.match ".*\.sqlx/.*" fpath != null;
        #   sourceFilter =
        #     fpath: type:
        #     (sqlxFilter fpath type) || (craneLib.filterCargoSources fpath type);
        # in
        # pkgs.lib.cleanSourceWith {
        #   src = ./server;
        #   filter = sourceFilter;
        # };

        commonArgs = {
          inherit src;
          strictDeps = true;
          SQLX_OFFLINE = "true";

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [
            openssl
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        api = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;

            # don't run tests because i didnt write any meaningful ones
            doCheck = false;
          }
        );

        # client = pkgs.stdenv.mkDerivation {
        #   pname = "piss-fan-client";
        #   version = "1.0.0";
        #   src = ./client;
        #
        #   nativeBuildInputs = [ pkgs.bun ];
        #
        #   buildPhase = ''
        #     export HOME=$TMPDIR
        #     bun install --frozen-lockfile
        #     bun run build
        #   '';
        #
        #   installPhase = ''
        #     mkdir -p $out
        #     cp -r dist/* $out/
        #   '';
        # };
      in
      {
        packages = {
          # inherit client;
          inherit api;
          default = api;
        };

        devShells.default = craneLib.devShell {
          buildInputs = with pkgs; [
            pkg-config
            openssl
            jq

            feroxbuster
            seclists

            cargo
            bun
            deno

            twitch-cli
            sqlx-cli

            postgresql
            redis

            grafana-alloy
          ];

          shellHook = ''
            export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig":$PKG_CONFIG_PATH"
            export PGHOST="$PWD/.pg-sock"
            export PGDATA="$PWD/.pg-data"

            if [ ! -f ".pg-data/PG_VERSION" ]; then
              mkdir -p "$PGHOST"
              mkdir -p "$PGDATA"

              initdb -D "$PGDATA"

              echo "unix_socket_directories = '$PWD/.pg-sock'" >> .pg-data/postgresql.conf
              echo "port = 5432" >> .pg-data/postgresql.conf
            fi

            echo "in shell"
          '';

        };
      }
    );
}
