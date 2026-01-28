{
  description = "piss.fan flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        pyPg = pkgs.python313.withPackages (
          ps: with ps; [
            psycopg
          ]
        );
      in
      {
        devShells.default = pkgs.mkShell {
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
            pyPg
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

