#!/bin/bash

if [[ -z "$@" ]];then
    echo "usage: |install|uninstall|"
    exit -1
fi

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

uninstall() {
    sudo /bin/bash <<EOF
rm -rf /usr/share/razercontrol
rm -f /usr/bin/razer-cli
rm -f /etc/udev/rules.d/99-hidraw-permissions.rules
rm -f /usr/lib/systemd/user/razerdaemon.service
udevadm control --reload-rules
EOF

}
install() {
    echo "Creating directories, copying files, and setting up services..."
    mkdir -p ~/.local/share/razercontrol
    sudo /bin/bash <<EOF
mkdir -p /usr/share/razercontrol
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

    return $?
}

case "$@" in
    "install")
        install
        ;;
    "uninstall")
        uninstall
        ;;
    *)
        echo "unknown arg $@"
        exit -1
        ;;
esac

exit $?
