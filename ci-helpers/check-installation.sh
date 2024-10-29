set -o errexit
set -o nounset
set -o pipefail

check_file() {
    if [ ! -e $1 ]; then 
        echo "File \"$1\" does not exist."
        exit 1
    fi
}

echo "Checking the existence of the necessary files..."
# Daemon
check_file "$HOME/.local/share/razercontrol"
check_file "/usr/share/razercontrol/daemon"
check_file "/usr/share/razercontrol/laptops.json"
check_file "/etc/udev/rules.d/99-hidraw-permissions.rules"
# CLI
check_file "/usr/bin/razer-cli"
# GUI
check_file "/usr/bin/razer-settings"
check_file "/usr/share/applications/razer-settings.desktop"
echo "All files are present"

printf "Checking that the service is enabled: "
systemctl --user is-enabled razercontrol.service

echo "Checking files on the path"
printf -- "- " && which razer-cli
printf -- "- " && which razer-settings

echo "Done!"
