let 
  rev = "fc07622617a373a742ed96d4dd536849d4bc1ec6";
in builtins.fetchTarball { 
  url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz"; 
  sha256 = "141ni718vq6pnpspwp0m8nsr2fpn98vls5f4m7vmgk55xwfjr4bw"; 
}