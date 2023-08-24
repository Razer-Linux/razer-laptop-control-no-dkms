#!/bin/bash

# Build the project
echo "Building the project..."
cargo build --release

# Check if the build was successful
if [ $? -ne 0 ]; then
    echo "Build failed, exiting."
    exit 1
fi

echo "Stopping razerdaemon service..."
systemctl --user stop razerdaemon.service

echo "Creating directories, copying files, and setting up services..."
mkdir -p ~/.local/share/razercontrol
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

# Check if the previous commands were successful
if [ $? -ne 0 ]; then
    echo "An error occurred while setting up, exiting."
    exit 1
fi

echo "Enabling razerdaemon service..."
systemctl --user enable razerdaemon.service

echo "Starting razerdaemon service..."
systemctl --user start razerdaemon.service

# Check if the service started successfully
if [ $? -ne 0 ]; then
    echo "Failed to start razerdaemon service, exiting."
    exit 1
fi

echo "Install complete!"
