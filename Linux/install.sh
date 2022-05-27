#!/bin/bash
# apt install remmina gnome-terminal -y

NAME='JumpServerClient'
INSTALLDIR=$HOME/$NAME

if [ -d "$INSTALLDIR" ]; then
    # delete old version
    rm -rf "$INSTALLDIR"
fi

ARCHIVE=`awk '/^__ARCHIVE_BOUNDARY__/ { print NR + 1; exit 0; }' $0`
tail -n +$ARCHIVE $0 > $NAME.tar.gz
tar -zvxf $NAME.tar.gz -C $HOME

chmod 777 $INSTALLDIR/
chmod 777 $INSTALLDIR/JumpServerClient
chmod 777 $INSTALLDIR/client

echo "[Desktop Entry]
Name=jms
Exec=$INSTALLDIR/JumpServerClient %u
Type=Application
Terminal=false
MimeType=x-scheme-handler/jms;" > ~/.local/share/applications/jms.desktop
echo x-scheme-handler/jms=jms.desktop >> ~/.local/share/applications/mimeapps.list
update-desktop-database ~/.local/share/applications

rm $NAME.tar.gz

exit 0
__ARCHIVE_BOUNDARY__
