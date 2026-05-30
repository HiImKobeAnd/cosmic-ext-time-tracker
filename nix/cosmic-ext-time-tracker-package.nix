{
  lib,
  version,
  dbus,
  libcosmicAppHook,
  git,
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "cosmic-ext-time-tracker";
  inherit version;

  src = lib.cleanSource ../.;

  buildAndTestSubdir = "cosmic-ext-time-tracker";

  cargoHash = "sha256-W4M1XUNOHbaZiXS5AhqsxzV7+HJTvGO1oriNLCsiMPI=";

  VERGEN_GIT_SHA = "unknown";
  VERGEN_GIT_COMMIT_DATE = "unknown";

  # Buildtime
  nativeBuildInputs = [
    git
    libcosmicAppHook
  ];

  # Runtime
  buildInputs = [
    dbus
  ];

  meta = {
    description = "";
    homepage = "";
    mainProgram = "cosmic-ext-time-tracker";
  };
})
