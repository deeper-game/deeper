{
  pkgs,
  nixpkgs,
  system,
  craneLib,
  craneLibWasm,
  rustToolchain,
  rustToolchainWasm,
  wasmTarget,
  nix-gitignore,
  std,
}:

let
  # Cleaned source
  src = nix-gitignore.lib.gitignoreSource ./.;

  # some helpful utilities
  isLinux = std.string.hasInfix "linux";
  isDarwin = std.string.hasInfix "darwin";
  isArm64 = std.string.hasInfix "aarch64";

  linker =
    if isDarwin system
    then pkgs.zld
    else if isArm64 system
         then pkgs.lld
         else pkgs.mold;

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
      #pkgs.vulkan-headers
      #pkgs.vulkan-tools
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

  # Helper simply to prevent annoying indentation.
  # Justification for its existence may be dubious.
  withCommonArgs = x: commonArgs // x;

  cargoArtifacts = craneLib.buildDepsOnly (withCommonArgs {});
in
rec {
  inherit rustToolchain;
  inherit rustToolchainWasm;
  inherit commonArgs;
  inherit linker;

  app = craneLib.buildPackage (withCommonArgs ({
    inherit cargoArtifacts;

    postInstall = ''
      # Make sure assets are findable
      cp -r assets/ $out/bin/

      # Needed for graphics
      wrapProgram $out/bin/deeper \
        --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ]}"
    '';
  }) // (if isLinux system && !(isArm64 system) then {
    CARGO_RUSTFLAGS = "-Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold -Zshare-generics=y";
    RUSTFLAGS = "-Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold -Zshare-generics=y";
  } else {}));

  wasm = craneLibWasm.buildPackage (withCommonArgs {
    inherit cargoArtifacts;

    cargoExtraArgs = "--target ${wasmTarget}";

    # Override crane's use of --workspace, which tries to build everything
    cargoCheckCommand = "cargo check --release";
    cargoBuildCommand = "cargo build --release";

    # https://github.com/gfx-rs/wgpu/discussions/1776
    # https://github.com/gfx-rs/wgpu/wiki/Running-on-the-Web-with-WebGPU-and-WebGL
    RUSTFLAGS = "--cfg=web_sys_unstable_apis";

    # Tests currently need to be run via `cargo wasi` which
    # isn't packaged in nixpkgs yet
    doCheck = false;
  });

  wasmRunner = pkgs.writeShellApplication {
    name = "run-deeper-wasm";
    text = ''
      export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN="wasm-server-runner"
      export WASM_SERVER_RUNNER_ADDRESS="127.0.0.1"
      export WASM_SERVER_RUNNER_DIRECTORY="."
      export WASM_SERVER_RUNNER_HTTPS="true"
      export WASM_SERVER_RUNNER_NO_MODULE="false"

      wasm-server-runner ${wasm}/bin/deeper.wasm
    '';
    runtimeInputs = [pkgs.wasm-server-runner];
  };
}
