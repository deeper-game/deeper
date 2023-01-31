{ fetchFromGitHub, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "wasm-server-runner";
  version = "0.4.0";

  src = fetchFromGitHub {
    owner = "jakobhellermann";
    repo = pname;
    rev = "v${version}";
    sha256 = "16shzz5fh7lhwz2mk1vrswafdvp63zkj5y9x9ni7dvpz67ndiwdv";
  };

  cargoPatches = [
    ./cargo-lock.patch
  ];

  cargoHash = "sha256-8cB1FBDRRVEgxWzlI5tfgETTErP71yCQL/asH2/fnlE=";

  doCheck = false;
}
