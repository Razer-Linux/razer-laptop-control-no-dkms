#!/bin/bash

cargo build --release

systemctl --user stop razerdaemon.service
sudo /bin/bash << EOF
mkdir -p /usr/share/razercontrol
systemctl stop razerdaemon.service
cp target/release/razer-cli /usr/bin/
cp target/release/daemon /usr/share/razercontrol/
cp data/devices/laptops.json /usr/share/razercontrol/
cp data/udev/99-hidraw-permissions.rules /etc/udev/rules.d/
cp razerdaemon.service /usr/lib/systemd/user/
udevadm control --reload-rules
EOF
systemctl --user enable razerdaemon.service
systemctl --user start razerdaemon.service
echo "Install complete!"
