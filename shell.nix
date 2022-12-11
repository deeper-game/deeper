(import <nixpkgs> {}).callPackage (

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
      pkgs.valgrind
      pkgs.renderdoc
      pkgs.gdb
    ];

    buildInputs = [
      pkgs.alsaLib
      pkgs.udev
      pkgs.libclang.lib
      pkgs.mesa
      pkgs.shaderc
      pkgs.shaderc.lib
      pkgs.xorg.libX11
      pkgs.xorg.libXcursor
      pkgs.xorg.libXrandr
      pkgs.xorg.libXi
      pkgs.libxkbcommon
      pkgs.SDL2
      pkgs.vulkan-loader
      pkgs.vulkan-validation-layers
    ];

    VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d:${pkgs.vulkan-tools-lunarg}/etc/vulkan/explicit_layer.d";
    SHADERC_LIB_DIR = "${pkgs.shaderc.lib}/lib";
    LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
  }

) {}
