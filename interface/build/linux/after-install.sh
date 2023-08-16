#!/bin/bash
# apt install remmina gnome-terminal deepin-terminal -y
set -e
INSTALLDIR='/opt/JumpServerClient'

# SUID chrome-sandbox for Electron 5+
chmod 4755 '/opt/${sanitizedProductName}/chrome-sandbox' || true

chmod -R 777 '/opt/${sanitizedProductName}/resources/bin'

ARCH=$(uname -m)
if [[ $ARCH == 'aarch64' ]];then
  ARCH='arm64'
else
  ARCH='amd64'
fi

echo "[Desktop Entry]
Name=jms
Exec=$INSTALLDIR/resources/bin/linux-$ARCH/JumpServerClient %u
Type=Application
Terminal=false
MimeType=x-scheme-handler/jms;" > /usr/share/applications/jms.desktop

update-desktop-database /usr/share/applications

xdg-mime default /usr/share/applications/jms.desktop x-scheme-handler/jms

update-mime-database /usr/share/mime

exit 0