
let 
  nixpkgs = import (import nix/nixpkgs.nix) { };
in nixpkgs.callPackage (
  { stdenv, pkgs, lib }:

  stdenv.mkDerivation rec {
    pname = "deeper";
    version = "0.1.0";

    src = lib.cleanSourceWith {
      filter = p: t: !(t == "directory" && baseNameOf p == "target");
      src = lib.cleanSource ./.;
    };

    nativeBuildInputs = [
      pkgs.pkgconfig
      # pkgs.valgrind
      # pkgs.renderdoc
      pkgs.gdb
    ];

    buildInputs = [
      # pkgs.alsaLib
      pkgs.libclang.lib
      pkgs.libiconv
      # pkgs.libxkbcommon
      pkgs.mesa
      pkgs.shaderc
      pkgs.shaderc.lib
      # pkgs.xorg.libX11
      # pkgs.xorg.libXcursor
      # pkgs.xorg.libXrandr
      # pkgs.xorg.libXi
      pkgs.SDL2
      # pkgs.udev
      pkgs.vulkan-loader
      # pkgs.vulkan-validation-layers
      pkgs.darwin.apple_sdk.frameworks.AppKit
    ];

    # VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d:${pkgs.vulkan-tools-lunarg}/etc/vulkan/explicit_layer.d";

    SHADERC_LIB_DIR = "${pkgs.shaderc.lib}/lib";

    # LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

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
  }

) {}
