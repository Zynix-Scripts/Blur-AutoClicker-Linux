{
  lib,
  stdenv,
  src,
  version,

  rustPlatform,
  fetchNpmDeps,

  cargo-tauri,
  npmHooks,
  nodejs,
  openssl,
  pkg-config,
  webkitgtk_4_1,
  wrapGAppsHook4,
  libayatana-appindicator,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  inherit src version;
  pname = "blur-autoclicker";

  cargoRoot = "src-tauri";
  buildAndTestSubdir = finalAttrs.cargoRoot;
  cargoHash = "sha256-lfouJzoAuaZm+N3FWDB28yl+LVlw0wlUQDK59ZhJY8o=";

  npmDeps = fetchNpmDeps {
    name = "${finalAttrs.pname}-${version}-npm-deps";
    inherit (finalAttrs) src;
    hash = "sha256-xSLN9kP4sBXbkqRJZQlm+3kPaQ5O0XQzpqLT2nCcgv4=";
  };

  nativeBuildInputs = [
    cargo-tauri.hook

    nodejs
    npmHooks.npmConfigHook

    pkg-config
    wrapGAppsHook4
  ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    openssl
    webkitgtk_4_1
    libayatana-appindicator
  ];

  meta = with lib; {
    description = "An Auto-clicker with a few advanced features and generally better performance than popular alternatives. Now ported to Linux";
    homepage = "https://autoclicker.blur009.com";
    changelog = "https://github.com/Zynix-Scripts/Blur-AutoClicker-Linux/releases/tag/v${version}";
    platforms = platforms.linux;
    license = licenses.gpl3;
    maintainers = with maintainers; [ ];
    mainProgram = "BlurAutoClicker";
  };
})
