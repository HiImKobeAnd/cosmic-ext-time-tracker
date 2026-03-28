# ⚠️ Project Status: Experimental

This project is in very early development. The API and architecture are subject to frequent, breaking changes without notice.

# Cosmic Ext Time Tracker

A Cosmic applet which provides a unified controller for external time-tracking services. 
<img width="370" height="281" alt="image-of-cosmic-ext-time-tracker" src="https://github.com/user-attachments/assets/d58a093e-a57b-4484-89f5-27d20c06377c" />

## Installation

A [justfile](./justfile) is included by default for the [casey/just][just] command runner.

- `just` builds the application with the default `just build-release` recipe
- `just run` builds and runs the application
- `just install` installs the project into the system
- `just vendor` creates a vendored tarball
- `just build-vendored` compiles with vendored dependencies from that tarball
- `just check` runs clippy on the project to check for linter warnings
- `just check-json` can be used by IDEs that support LSP

## Developers

This project provides a Nix Flake which is available using `nix develop`.

### Useful links

- libcosmic: https://github.com/pop-os/libcosmic
- fluent: https://projectfluent.org/
- fluent-guide: https://projectfluent.org/fluent/guide/hello.html
- iso-codes: https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes
- just: https://github.com/casey/just
