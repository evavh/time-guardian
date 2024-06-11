set -e
cargo build
target/debug/time-guardian status $USER
