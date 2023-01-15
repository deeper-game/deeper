
let
  nixpkgs = import (import nix/nixpkgs.nix) { };
in nixpkgs.callPackage (
  { clangStdenv, pkgs, lib }:

  clangStdenv.mkDerivation ((rec {
    pname = "deeper";
    version = "0.1.0";

    src = lib.cleanSourceWith {
      filter = p: t: !(t == "directory" && baseNameOf p == "target");
      src = lib.cleanSource ./.;
    };

    nativeBuildInputs = [
      pkgs.pkgconfig
      pkgs.gdb
    ] ++ lib.optionals clangStdenv.isLinux [
      pkgs.valgrind
      pkgs.renderdoc
      pkgs.mold
    ];

    buildInputs = [
      pkgs.libclang.lib
      pkgs.libiconv
      pkgs.shaderc
      pkgs.shaderc.lib
      pkgs.SDL2
      pkgs.vulkan-loader
    ] ++ lib.optionals clangStdenv.isLinux [
      pkgs.alsaLib
      pkgs.xorg.libX11
      pkgs.xorg.libXcursor
      pkgs.xorg.libXrandr
      pkgs.xorg.libXi
      pkgs.libxkbcommon
      pkgs.mesa
      pkgs.udev
      pkgs.vulkan-validation-layers
    ] ++ lib.optionals clangStdenv.isDarwin [
      pkgs.darwin.apple_sdk.frameworks.AppKit
    ];

    SHADERC_LIB_DIR = "${pkgs.shaderc.lib}/lib";

    # VK_LAYER_PATH = if stdenv.isLinux
    #   then "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d:${pkgs.vulkan-tools-lunarg}/etc/vulkan/explicit_layer.d"
    #   else "$VK_LAYER_PATH";

    LD_LIBRARY_PATH =
      if clangStdenv.isLinux
      then lib.makeLibraryPath buildInputs
      else "$LD_LIBRARY_PATH";

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
    COREAUDIO_SDK_PATH =
      if pkgs.stdenv.isDarwin
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
            mkdir $out/System
            mv $out/Library $out/System
          '';
        }
    else "";
  }) // (if clangStdenv.isLinux then {
    CARGO_LINKER = "clang";
    CARGO_RUSTFLAGS = "-C link-arg=-fuse-ld=${pkgs.mold}/bin/mold";
  } else {}))
) {}
