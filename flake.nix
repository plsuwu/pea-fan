{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        craneLib = crane.mkLib pkgs;
        cargoSrc =
          let
            fs = pkgs.lib.fileset;
            unfilteredRoot = ./server;
          in
          fs.toSource {
            root = unfilteredRoot;
            fileset = fs.intersection (fs.gitTracked unfilteredRoot) (
              fs.unions [
                (unfilteredRoot + "/Cargo.toml")
                (unfilteredRoot + "/Cargo.lock")
                (unfilteredRoot + "/.sqlx")
                (unfilteredRoot + "/src")
                (unfilteredRoot + "/migrations")
              ]
            );
          };

        commonArgs = {
          src = cargoSrc;
          strictDeps = true;
          SQLX_OFFLINE = true;

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
            nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [
              pkgs.sqlx-cli
            ];

            prebuild = ''
              export SQLX_OFFLINE_DIR="./server/.sqlx" 
            '';

            doCheck = false;

            installPhaseCommand = ''
              mkdir -p $out/bin
              mkdir -p $out/lib

              cp -R ./target/release/piss-fan-server $out/bin/piss-fan-server
              cp -R ./migrations $out/lib/
            '';
          }
        );

        client = pkgs.buildNpmPackage {
          pname = "piss-fan-client";
          version = "1.3.2";
          src = ./client;

          npmDepsHash = "sha256-7T8ZdhbsMX7RTWZvf9kkYDN1DPkRT54R+bbpyDa7GcU=";

          buildPhase = ''
            npm install
            npm run build -- --sourcemap

            rm -rf ./node_modules
            npm install --omit dev

          '';

          installPhase = ''
            mkdir -p $out
            cp -R ./node_modules $out/
            cp -R ./build $out/
          '';
        };

      in
      {
        packages = {
          inherit api client;
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

            chromedriver
            chromium

            geckodriver
            firefox

            # python313
            (python313.withPackages (
              ps: with ps; [
                requests
                beautifulsoup4
                numpy
                selenium
                webdriver-manager
                fonttools
                brotli
              ]
            ))

            twitch-cli
            sqlx-cli
            ttfautohint

            # pyftsubset

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

            export PATH="${pkgs.chromedriver}/bin:$PATH"

            echo "in shell"
          '';

        };
      }
    );
}
