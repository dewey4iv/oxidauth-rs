#!/bin/bash -

set -o nounset                              # Treat unset variables as an error

sleep 10

echo "starting migrate"
/bin/api migrate >> /proc/1/fd/1
echo "migration finished"

echo "starting setup"
/bin/api setup >> /proc/1/fd/1
echo "setup finished"

tail -f /dev/null
