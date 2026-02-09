{
  description = "GitHub App for fast-forward merging pull requests";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nix2container.url = "github:nlewo/nix2container";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    flake-utils,
    nix2container,
    nixpkgs,
    self,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
        nix2containerPkgs = nix2container.packages.${system};
        pkgs = import nixpkgs {inherit system;};
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            actionlint
            cargo
            clippy
            crate2nix
            gh
            gosmee
            rust-analyzer
            rustc
            rustfmt
            tombi
          ];
        };

        formatter = pkgs.alejandra;

        packages = {
          containerImage = let
            package = self.packages.${system}.default;
            year = builtins.substring 0 4 self.lastModifiedDate;
            month = builtins.substring 4 2 self.lastModifiedDate;
            day = builtins.substring 6 2 self.lastModifiedDate;
            hour = builtins.substring 8 2 self.lastModifiedDate;
            minute = builtins.substring 10 2 self.lastModifiedDate;
            second = builtins.substring 12 2 self.lastModifiedDate;
            created = "${year}-${month}-${day}T${hour}:${minute}:${second}Z";
          in
            nix2containerPkgs.nix2container.buildImage {
              name = "pureff";
              config = {
                Entrypoint = [(pkgs.lib.getExe package)];
                Env = [
                  "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
                ];
                ExposedPorts = {
                  "8000/tcp" = {};
                };
                Labels = {
                  "org.opencontainers.image.created" = created;
                  "org.opencontainers.image.description" = cargoToml.package.description;
                  "org.opencontainers.image.documentation" = cargoToml.package.documentation;
                  "org.opencontainers.image.revision" = self.rev or "";
                  "org.opencontainers.image.source" = cargoToml.package.repository;
                  "org.opencontainers.image.title" = "PureFF";
                  "org.opencontainers.image.url" = cargoToml.package.homepage;
                  "org.opencontainers.image.version" = package.version;
                };
              };
              inherit created;
              tag = self.rev or null;
            };

          default = let
            cargoNix = import ./Cargo.nix {
              inherit nixpkgs;
              inherit pkgs;
            };
          in
            cargoNix.rootCrate.build;
        };
      }
    );
}
