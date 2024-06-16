set -e
cargo build
sudo env RUST_LOG=trace target/debug/time-guardian run
