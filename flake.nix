{
  description =
    "Stability: The world's first tokenless, permissionless, public blockchain.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils-plus = {
      url = "github:gytis-ivaskevicius/flake-utils-plus";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, ... }@inputs:
    let
      FL = inputs.flake-utils.lib;
      FLP = inputs.flake-utils-plus.lib;
      supportedSystems = [ FLP.system.x86_64-linux ];

    in FLP.mkFlake {
      inherit self inputs supportedSystems;

      channelsConfig = { allowUnfree = true; };
      sharedOverlays = [ (import inputs.rust-overlay) ];

      outputsBuilder = channels:
        let
          P = channels.nixpkgs;
          B = builtins;
          L = P.lib;

        in rec {
          packages = { };

          devShell = P.stdenv.mkDerivation {
            name = "stability-dev-shell";
            buildInputs = with P; [
              llvmPackages_11.libclang
              llvmPackages_11.clang
              protobuf
              nixfmt
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            ];

            shellHook = ''
              export __NIX_PS1__="stability";
              source "${P.bash-completion}/etc/profile.d/bash_completion.sh";
              source "${P.git}/share/bash-completion/completions/git";

              export LIBCLANG_PATH="${P.llvmPackages_11.libclang.lib}/lib";

              export BINDGEN_EXTRA_CLANG_ARGS="$(< ${P.stdenv.cc}/nix-support/libc-crt1-cflags) \
                $(< ${P.stdenv.cc}/nix-support/libc-cflags) \
                $(< ${P.stdenv.cc}/nix-support/cc-cflags) \
                $(< ${P.stdenv.cc}/nix-support/libcxx-cxxflags) \
                ${
                  L.optionalString P.stdenv.cc.isClang
                  "-idirafter ${P.stdenv.cc.cc}/lib/clang/${
                    L.getVersion P.stdenv.cc.cc
                  }/include"
                } \
                ${
                  L.optionalString P.stdenv.cc.isGNU
                  "-isystem ${P.stdenv.cc.cc}/include/c++/${
                    L.getVersion P.stdenv.cc.cc
                  } -isystem ${P.stdenv.cc.cc}/include/c++/${
                    L.getVersion P.stdenv.cc.cc
                  }/${P.stdenv.hostPlatform.config} -idirafter ${P.stdenv.cc.cc}/lib/gcc/${P.stdenv.hostPlatform.config}/${
                    L.getVersion P.stdenv.cc.cc
                  }/include"
                }"
            '';
          };
        };
    };
}
