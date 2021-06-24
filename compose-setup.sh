#!/bin/bash -
set -o nounset                              # Treat unset variables as an error

LOCK_FILE=/etc/oxidauth/setup.lock

if [ -f "$LOCK_FILE" ]; then
    echo "Lock file already exists"
    tail -f /dev/null
else
    echo "Creating lockfile"
    touch $LOCK_FILE

    echo "No lock file found -- running startup"
    /bin/setup.sh
fi
