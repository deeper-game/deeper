{
  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-std = {
      url = "github:chessai/nix-std";
    };
  };

  outputs = {
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    devshell,
    nix-gitignore,
    nix-std,
    self,
    ...
  }: let
    supportedSystems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];
    std = nix-std.lib;
  in
    { inherit std; } // flake-utils.lib.eachSystem supportedSystems (system: let

      wasmTarget = "wasm32-unknown-unknown";

      rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
      rustToolchainWasm = rustToolchain.override {
        targets = [ wasmTarget ];
      };

      craneLib = crane.lib.${system}.overrideToolchain rustToolchain;
      # NB: we don't need to overlay our custom toolchain for the *entire*
      # pkgs (which would require rebuilding anything else which uses rust).
      # Instead, we just want to update the scope that crane will use by appending
      # our specific toolchain there.
      craneLibWasm = craneLib.overrideToolchain rustToolchainWasm;

      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
          devshell.overlay
          (self: super: {
            wasm-server-runner = self.callPackage ./nix/wasm-server-runner {};
          })
        ];
      };

      code = pkgs.callPackage ./deeper.nix {
        inherit nixpkgs system craneLib craneLibWasm rustToolchain rustToolchainWasm wasmTarget nix-gitignore std;
      };

      # some helpful utilities
      isLinux = std.string.hasInfix "linux";
      isDarwin = std.string.hasInfix "darwin";
      isArm64 = std.string.hasInfix "aarch64";
    in {
      inherit pkgs;

      packages = {
        app = code.deeper;
        wasm = code.wasm;
        wasmRunner = code.wasmRunner;
        all = pkgs.symlinkJoin {
          name = "all";
          paths = [ code.app code.wasm code.wasmRunner ];
        };
        default = self.packages.${system}.all;
      };

      apps.default = flake-utils.lib.mkApp {drv = self.packages.${system}.default;};

      checks = {
        inherit (code) deeper;
      };

      devShells.default = pkgs.devshell.mkShell {
        env = [
          {
            name = "SHADERC_LIBRARY_PATH";
            value = "${pkgs.shaderc.lib}/lib";
          }

          {
            name = "PKG_CONFIG_PATH";
            value = std.string.concatSep ":" [
              "${pkgs.alsaLib.dev}/lib/pkgconfig"
              "${pkgs.udev.dev}/lib/pkgconfig"
            ];
          }

          {
            name = "LD_LIBRARY_PATH";
            value =
              if isLinux system
              then "LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath self.packages.${system}.default.buildInputs}"
              else "$LD_LIBRARY_PATH";
          }

          {
            # The coreaudio-sys crate is configured to look for things in whatever the
            # output of `xcrun --sdk macosx --show-sdk-path` is. However, this does not
            # always contain the right frameworks, and it uses system versions instead of
            # what we control via Nix. Instead of having to run a lot of extra scripts
            # to set our systems up to build, we can just create a SDK directory with
            # the same layout as the `MacOSX{version}.sdk` that XCode produces.
            #
            # TODO: I'm not 100% confident that this being blank won't cause issues for
            # Nix-on-Linux development. It may be sufficient to use the pkgs.symlinkJoin
            # above regardless of system! That'd set us up for cross-compilation as well.
            name = "COREAUDIO_SDK_PATH";
            value =
              if isDarwin system
              then pkgs.symlinkJoin {
                name = "sdk";
                paths = with pkgs.darwin.apple_sdk.frameworks; [
                  AppKit
                  AudioToolbox
                  AudioUnit
                  CoreAudio
                  CoreFoundation
                  CoreMIDI
                  OpenAL
                ];
                postBuild = ''
                  mkdir -p $out/System
                  mv $out/Library $out/System
                '';
              }
              else "";
          }
        ];

        # Should add some commands for easier wasm generation
        commands = [];

        devshell = {
          name = "deeper";
          packages = code.commonArgs.buildInputs ++ [
            pkgs.pkg-config
            pkgs.clang

            # LSP's
            pkgs.rust-analyzer
            pkgs.rnix-lsp

            # Tools
            code.rustToolchain
            pkgs.alejandra
            pkgs.shellcheck
            pkgs.jq
          ];
        };
      };
    });
}
