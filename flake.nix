{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        deps = with pkgs; [
          rust-bin.stable.latest.default
          bacon
          just
          libcosmicAppHook
          dbus
        ];

        qtEnv =
          with pkgs.qt6;
          env "qt-custom-${qtbase.version}" [
            pkgs.qt6.qtbase
            pkgs.qt6.qtdeclarative
            pkgs.qt6.qtwayland
            pkgs.quickshell
          ];
      in
      {
        devShells.default =
          let
            qmlls-wrapped =
              with pkgs;
              writeShellScriptBin "qmlls" ''
                exec ${pkgs.qt6.qtdeclarative}/bin/qmlls \
                -i "${pkgs.qt6.qtdeclarative}/lib/qt-6/qml" \
                -i "${pkgs.qt6.qtbase}/lib/qt-6/qml" \
                "$@"
              '';
          in
          with pkgs;
          mkShell {
            buildInputs = [
              qtEnv
              pkgs.qt6.qtbase
              deps
              pkgs.noctalia-shell

              pkgs.qt6.wrapQtAppsHook
              pkgs.makeWrapper
            ];
            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath deps}";
            QT_PLUGIN_PATH = "${qtEnv}/lib/qt-6/plugins";
            QML_IMPORT_PATH = "${qtEnv}/lib/qt-6/qml";
            QT_QPA_PLATFORM_PLUGIN_PATH = "${qtEnv}/lib/qt-6/plugins/platforms";
            PKG_CONFIG_PATH = "${qtEnv}/lib/pkgconfig:$PKG_CONFIG_PATH";
          };
      }
    );
}
