set -e

cargo build --features deploy
sudo systemctl stop time-guardian.service
echo "Stopped service"
sudo cp target/debug/time-guardian /usr/local/bin/time-guardian
echo "Copied binary"
sudo systemctl start time-guardian.service
echo "Started service"
