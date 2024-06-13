set -e
cargo build
sudo env RUST_LOG=debug target/debug/time-guardian run
