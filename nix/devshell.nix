{
  pkgs,
  cosmic-ext-time-tracker,
}:
pkgs.mkShell (finalAttrs: {
  inputsFrom = [ cosmic-ext-time-tracker ];

  nativeBuildInputs = with pkgs; [
    bacon
    just
    cargo
    rustc
    rustPlatform.cargoSetupHook
  ];

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (
    (cosmic-ext-time-tracker.buildInputs or [ ]) ++ (cosmic-ext-time-tracker.nativeBuildInputs or [ ])

  )}";
})
