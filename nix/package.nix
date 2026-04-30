{
  lib,
  stdenv,
  src,
  version,

  rustPlatform,
  fetchPnpmDeps,

  cargo-tauri,
  pnpmConfigHook,
  nodejs,
  pnpm,
  openssl,
  pkg-config,
  webkitgtk_4_1,
  wrapGAppsHook4,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  inherit src version;
  pname = "blur-autoclicker";

  cargoRoot = "src-tauri";
  buildAndTestSubdir = finalAttrs.cargoRoot;
  cargoHash = "sha256-9KGt/Ea+wr1PBU+fVmq0igiBttUYAT86hJEBKmloC7c=";

  pnpmDeps = fetchPnpmDeps {
    name = "${finalAttrs.pname}-${version}-pnpm-deps";
    inherit (finalAttrs) src pname;
    hash = "sha256-abTJIq6NGlw0GFhsDU6ryQhPSk5NwPzn/G8UZmD6ZNw=";
    fetcherVersion = 2;
  };

  nativeBuildInputs = [
    cargo-tauri.hook
    nodejs
    pnpm
    pnpmConfigHook
    pkg-config
    wrapGAppsHook4
  ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    openssl
    webkitgtk_4_1
  ];

  meta = with lib; {
    description = "An Auto-clicker with a few advanced features and generally better performance than popular alternatives. Now ported to Linux";
    homepage = "https://blur009.vercel.app/projects/blur-autoclicker";
    changelog = "https://github.com/Zynix-Scripts/Blur-AutoClicker-Linux/releases/tag/v${version}";
    platforms = platforms.linux;
    license = licenses.gpl3;
    maintainers = with maintainers; [ ];
    mainProgram = "BlurAutoClicker";
  };
})