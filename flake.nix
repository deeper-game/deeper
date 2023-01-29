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
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
          devshell.overlay
        ];
      };
      src = nix-gitignore.lib.gitignoreSource ./.;

      isLinux = std.string.hasInfix "linux";
      isDarwin = std.string.hasInfix "darwin";
      isArm64 = std.string.hasInfix "aarch64";

      # Faster linkers
      linker =
        if isDarwin system
        then pkgs.zld
        else if isArm64 system
             then pkgs.lld
             else pkgs.mold;

      rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
      craneLib = crane.lib.${system}.overrideToolchain rustToolchain;

      commonArgs = {
        inherit src;

        buildInputs = [
          pkgs.libclang.lib
          pkgs.libiconv
          pkgs.shaderc
          pkgs.shaderc.lib
          pkgs.SDL2
          pkgs.vulkan-loader
          pkgs.makeWrapper
        ] ++ std.list.optionals (isLinux system) [
          pkgs.alsaLib
          pkgs.xorg.libX11
          pkgs.xorg.libXcursor
          pkgs.xorg.libXrandr
          pkgs.xorg.libXi
          pkgs.libxkbcommon
          pkgs.mesa
          pkgs.udev
          pkgs.vulkan-validation-layers
        ] ++ std.list.optionals (isDarwin system) [
          pkgs.darwin.apple_sdk.frameworks.AppKit
        ];

        nativeBuildInputs = [
          pkgs.pkgconfig
          pkgs.gdb
          linker
        ] ++ std.list.optionals (isLinux system) [
          pkgs.valgrind
          pkgs.renderdoc
        ];
      };

      cargoArtifacts = craneLib.buildDepsOnly (commonArgs
        // {
          pname = "deps";
        });
      deeper = craneLib.buildPackage (commonArgs
        // rec {
          inherit cargoArtifacts;

          postInstall = ''
            # Make sure assets are findable
            cp -r assets/ $out/bin/

            # Needed for graphics
            wrapProgram $out/bin/deeper \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ]}"
          '';
        }
        // (if isLinux system && !(isArm64 system) then {
          CARGO_LINKER = "clang";
          CARGO_RUSTFLAGS = "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
        } else {}));
    in {
      inherit pkgs;
      inherit rustToolchain;

      inherit deeper;
      packages.default = deeper;

      apps.default = flake-utils.lib.mkApp {drv = self.packages.${system}.default;};

      checks = {
        inherit deeper;
      };

      devShells.default = pkgs.devshell.mkShell {
        env = [
          {
            name = "SHADERC_LIBRARY_PATH";
            value = "${pkgs.shaderc.lib}/lib";
          }

          {
            name = "LD_LIBRARY_PATH";
            value =
              if isLinux system
              then "LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath commonArgs.buildInputs}"
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

        commands = [];

        devshell = {
          name = "deeper";
          packages = [
            # LSP's
            pkgs.rust-analyzer
            pkgs.rnix-lsp

            # Tools
            rustToolchain
            pkgs.alejandra
            pkgs.shellcheck
            pkgs.jq
          ];
        };
      };
    });
}
