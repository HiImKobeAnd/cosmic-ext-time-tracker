{
  lib,
  stdenv,
  rustPlatform,
  just,
  libcosmicAppHook,
  dbus,
  version,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "cosmic-ext-time-tracker";
  inherit version;

  src = lib.cleanSource ../.;

  cargoHash = "sha256-5fTg9ZEhu33QgEsBOlFhNChS95qFDagBpIBjuCN/zIk=";

  VERGEN_GIT_SHA = "unknown";
  VERGEN_GIT_COMMIT_DATE = "unknown";

  nativeBuildInputs = [
    just
    libcosmicAppHook
  ];

  buildInputs = [
    dbus
  ];

  justFlags = [
    "--set"
    "prefix"
    (placeholder "out")
    "--set"
    "bin-src"
    "target/release/cosmic-ext-time-tracker"
  ];

  meta = {
    description = "";
    homepage = "";
    mainProgram = "cosmic-ext-time-tracker";
  };
})
