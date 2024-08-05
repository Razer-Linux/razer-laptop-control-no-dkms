#!/usr/bin/env bash

detect_init_system() {
    if pidof systemd 1>/dev/null 2>/dev/null; then
        INIT_SYSTEM="systemd"
    elif [ -f "/sbin/rc-update" ]; then
        INIT_SYSTEM="openrc"
    else
        INIT_SYSTEM="other"
    fi
}

install() {
    echo "Building the project..."
    cargo build --release # TODO: The GUI should be optional. At least for now. Before releasing this, it sould be turned into a feature with an explicit cli switch to install it

    if [ $? -ne 0 ]; then
        echo "An error occurred while building the project"
        exit 1
    fi

    # Stop the service if it's running
    echo "Stopping the service..."
    case $INIT_SYSTEM in
    systemd)
        systemctl --user stop razercontrol
        ;;
    openrc)
        sudo rc-service razercontrol stop
        ;;
    esac

    # Install the files
    echo "Installing the files..."
    mkdir -p ~/.local/share/razercontrol
    sudo bash <<EOF
        mkdir -p /usr/share/razercontrol
        cp target/release/razer-cli /usr/bin/
        cp target/release/razer-settings /usr/bin/
        if ls /usr/share/applications/*.desktop 1> /dev/null 2>&1; then
            # We only install the desktop file if there are already desktop
            # files on the system
            cp data/gui/razer-settings.desktop /usr/share/applications/
        fi
        cp target/release/daemon /usr/share/razercontrol/
        cp data/devices/laptops.json /usr/share/razercontrol/
        cp data/udev/99-hidraw-permissions.rules /etc/udev/rules.d/
        udevadm control --reload-rules
EOF

    if [ $? -ne 0 ]; then
        echo "An error occurred while installing the files"
        exit 1
    fi

    # Start the service
    echo "Starting the service..."
    case $INIT_SYSTEM in
    systemd)
        sudo cp data/services/systemd/razercontrol.service /etc/systemd/user/
        systemctl --user enable --now razercontrol
        ;;
    openrc)
        sudo bash <<EOF
            cp data/services/openrc/razercontrol /etc/init.d/
            # HACK: Change the username in the script
            sed -i 's/USERNAME_CHANGEME/$USER/' /etc/init.d/razercontrol

            chmod +x /etc/init.d/razercontrol
            rc-update add razercontrol default
            rc-service razercontrol start
EOF
        ;;
    esac

    echo "Installation complete"
}

uninstall() {
    # Remove the files
    echo "Uninstalling the files..."
    sudo bash <<EOF
        rm -f /usr/bin/razer-cli
        rm -f /usr/bin/razer-settings
        rm -f /usr/share/applications/razer-settings.desktop
        rm -f /usr/share/razercontrol/daemon
        rm -f /usr/share/razercontrol/laptops.json
        rm -f /etc/udev/rules.d/99-hidraw-permissions.rules
        udevadm control --reload-rules
EOF

    if [ $? -ne 0 ]; then
        echo "An error occurred while uninstalling the files"
        exit 1
    fi

    # Stop the service
    echo "Stopping the service..."
    case $INIT_SYSTEM in
    systemd)
        systemctl --user disable --now razercontrol
    sudo bash <<EOF
        rm -f /etc/systemd/user/razercontrol.service
EOF
        ;;
    openrc)
        sudo bash <<EOF
            rc-service razercontrol stop
            rc-update del razercontrol default
            rm -f /etc/init.d/razercontrol
EOF
        ;;
    esac

    echo "Uninstalled"
}

main() {
    if [ "$EUID" -eq 0 ]; then
        echo "Please do not run as root"
        exit 1
    fi

    detect_init_system

    if [ "$INIT_SYSTEM" = "other" ]; then
        echo "Unsupported init system"
        exit 1
    fi

    case $1 in
    install)
        install
        ;;
    uninstall)
        uninstall
        ;;
    *)
        echo "Usage: $0 {install|uninstall}"
        exit 1
        ;;
    esac
}

main $@
