set -e
cargo build
sudo target/debug/time-guardian run
