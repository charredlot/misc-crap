#!/bin/bash -x

# For wifi without network manager

# wifi intf name
INTF="wlp4s0"

# wpa_passphrase $SSID $PASSPHRASE > wifi.conf
# configuration file:
CONF="/path/to/wifi.conf"

sudo wpa_supplicant -B -i${INTF} -c${CONF}
sudo dhclient ${INTF}
