let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    rustc
    nodejs_22
    # pnpm

    pkg-config
    wrapGAppsHook4
  ];

  buildInputs = with pkgs; [
    openssl
    webkitgtk_4_1
  ];

  shellHook = ''
    export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
  '';

  GIO_MODULE_DIR = "${pkgs.glib-networking}/lib/gio/modules/";
}
