{ pkgs, libsecp256k1 }:

let
  gsed = pkgs.gnused.overrideAttrs (oldAttrs: {
    fixupPhase = ''
      mv $out/bin/sed $out/bin/gsed
    '';
  });
in
with pkgs; stdenv.mkDerivation {
  pname = "libwally-core";
  version = "0.8.5";

  src = builtins.fetchGit {
    url = "https://github.com/ElementsProject/libwally-core.git";
    rev = "e6ab70fba3387d39cf5d871e45a8e6d16d90f593";
    submodules = true;
  };

  buildInputs = [
    autoconf
    automake
    gsed
    libtool
    python39
  ];

  preConfigurePhases = [ "autogenPhase" ];

  autogenPhase = ''
    ./tools/autogen.sh
  '';

  configureFlags = [
    "--enable-static"
    "--disable-shared"
    "--enable-elements"
    "--disable-tests"
  ];

  installPhase = ''
    mkdir -p $out/lib
    mv --force src/libwallycore.la src/.libs/libwallycore.la
    mv src/.libs/* $out/lib
    ln -s ${libsecp256k1}/lib/* $out/lib
  '';
}
