let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell rec {
  nativeBuildInputs = with pkgs; [
    rustc
    nodejs_22

    pkg-config
    wrapGAppsHook4
  ];

  buildInputs = with pkgs; [
    openssl
    webkitgtk_4_1
    libayatana-appindicator
  ];

  shellHook = ''
    export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
  '';

  GIO_MODULE_DIR = "${pkgs.glib-networking}/lib/gio/modules/";
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
