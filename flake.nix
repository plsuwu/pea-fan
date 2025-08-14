{
  description = "flake";

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
        pgPort = 5432;
      in
      {
        devShells.default = pkgs.mkShell {

          buildInputs = with pkgs; [
            openssl
            pkg-config
          
            # ngrok requires 'NIXPKGS_ALLOW_UNFREE=1' in environment  
            # (see `.envrc`)
            ngrok

            rustc
            cargo
            twitch-cli

            bun
            nodejs

            typescript
            redis
            postgresql

            git
            jq
            ripgrep
          ];

          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

            function pgd() {
              if ! pg_ctl status >/dev/null 2>&1; then
                pg_ctl start -l $PGHOST/postgres.log && sleep 2

                if pg_ctl status >/dev/null 2>&1; then
                  echo "postgres daemon running - sock path: $PGHOST"
                fi

                else echo "postgres daemon running - sock path: $PGHOST"
              fi
            }

            mkdir -p .pg-data
            mkdir -p .pg-sock

            if [ ! -f ".pg-data/PG_VERSION" ]; then
              initdb -D .pg-data

              echo "unix_socket_directories = '$PWD/.pg-sock'" >> .pg-data/postgresql.conf
              echo "port = 5432" >> .pg-data/postgresql.conf
            fi

              export PGHOST="$PWD/.pg-sock"
              export PGDATA="$PWD/.pg-data"

              if ! pg_ctl status >/dev/null 2>&1; then
                echo "postgres daemon isn't running yet (call pdg to start)"
              fi
          '';
        };
      }
    );
}
