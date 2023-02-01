{ fetchFromGitHub, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "wasm-server-runner";
  version = "0.4.0";

  # Fork that solves https://github.com/jakobhellermann/wasm-server-runner/issues/28
  src = fetchFromGitHub {
    owner = "heisencoder";
    repo = pname;
    rev = "676b5eb57947ae432a9a1ea0d2d84a16c6dac909";
    sha256 = "1xxckn4miyx2bkirfqg373k0drhflxp28vpkji8wm15w4x6w1mgl";
  };

  cargoPatches = [
    ./cargo-lock.patch
  ];

  cargoHash = "sha256-RA80pj2iU3pjIh65sw9xCzS/5JwgpbYBMmJ9HoGnkzs=";

  doCheck = false;
}
