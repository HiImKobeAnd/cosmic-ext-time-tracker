name := 'cosmic-ext-time-tracker'
appid := 'com.github.hiimkobeand.cosmic-ext-time-tracker'

# Default recipe which runs `just build-release`
# default: build-release

# Run the application for testing purposes
# run *args:
# env RUST_BACKTRACE=full cargo run --release {{args}}

reset-shell:
    -pkill quickshell
    cargo build --bin noctalia-shell-time-tracker
    cp ./target/debug/noctalia-shell-time-tracker ./noctalia-shell-time-tracker/
    NOCTALIA_DEBUG=1 noctalia-shell
