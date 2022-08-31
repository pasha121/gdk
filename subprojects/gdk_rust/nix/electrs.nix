{ pkgs }:

with pkgs; let
  esploraCommit = "703c6a20d52b61a234a18812503bd754d448992a";

  archiveName = isLiquid:
    let
      inherit (pkgs.stdenv.hostPlatform) isDarwin;
      suffix = if isLiquid then "_liquid" else "";
      platform = if isDarwin then "macos" else "linux";
    in
    "electrs_${platform}_esplora_${esploraCommit}${suffix}.zip";

  getUrl = isLiquid:
    "https://github.com/RCasatta/electrsd/releases/download/electrs_releases/${archiveName isLiquid}";

  mkElectrsDerivation = { isLiquid, sha256 }: stdenv.mkDerivation {
    name = "electrs_${if isLiquid then "liquid" else "bitcoin"}";

    src = pkgs.fetchzip {
      url = getUrl isLiquid;
      curlOptsList = [ "-L" ];
      inherit sha256;
    };

    phases = [ "installPhase" ];

    installPhase = ''
      mkdir -p $out/bin
      cp $src/electrs $out/bin
      chmod +x $out/bin/electrs
    '';
  };
in
{
  bitcoin = mkElectrsDerivation {
    isLiquid = false;
    sha256 = "sha256-RzaWWzIV4Ebt/5nII8a+uuaN63QaPEIw2onAuwuFX48=";
  };

  liquid = mkElectrsDerivation {
    isLiquid = true;
    sha256 = "sha256-hD+zd/3u0fxBxmB7Nza+jBe9MsAiABnjDr1imS5B7RU=";
  };
}
