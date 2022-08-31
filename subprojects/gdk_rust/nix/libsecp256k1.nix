{ pkgs }:

with pkgs; stdenv.mkDerivation {
  name = "libsecp256k1";

  src = builtins.fetchGit {
    url = "https://github.com/ElementsProject/secp256k1-zkp.git";
    rev = "6c0aecf72b1f4290f50302440065392715d6240a";
  };

  buildInputs = [
    autoconf
    automake
    libtool
  ];

  preConfigurePhases = [ "autogenPhase" ];

  autogenPhase = ''
    ./autogen.sh
  '';

  configureFlags = [
    "--enable-static"
    "--disable-shared"
  ];

  installPhase = ''
    mkdir -p $out/lib
    mv --force libsecp256k1.la .libs/libsecp256k1.la
    mv --force libsecp256k1_precomputed.la .libs/libsecp256k1_precomputed.la
    mv .libs/* $out/lib
  '';
}
