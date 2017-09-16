#!/usr/bin/python

from itertools import cycle

import argparse
import re
import subprocess
import sys
import time

CHANNEL_RE=re.compile("Channel\s*(?P<channel>[0-9]*)\s*:\s*(?P<freq>[\.0-9]*)\s*GHz")

def hop(intf, channels, dwell_sec):
    print("Hopping with interval {} sec".format(dwell_sec))
    for channel, freq in channels:
        print("  Channel {} = {} GHz".format(channel, freq))

    for channel, freq in cycle(channels):
        cmd = ["iwconfig", intf, "channel", str(channel)]
        subprocess.check_output(cmd)
        time.sleep(dwell_sec)


def main():
    parser = argparse.ArgumentParser(
                description="Periodically hop channels on wireless interface")
    parser.add_argument("-i", "--intf",
                        dest="intf", required=True,
                        help="Interface name to listen on, e.g. wlan0")
    parser.add_argument("-d", "--dwell",
                        dest="dwell", default=0.25,
                        help=("Time to dwell on each channel,"
                              "in floating point seconds, e.g. 0.5 seconds"))
    parser.add_argument("--channels",
                        dest="channels", nargs="+",
                        help=("List of channels to hop on"
                              " e.g. --channels 2 44 "))
    args = parser.parse_args()

    print("Hopping on interface {}".format(args.intf))

    freqs = subprocess.check_output(["iwlist", args.intf, "frequency"])
    freqs = str(freqs)

    channels = []
    for line in freqs.split('\n'):

        m = CHANNEL_RE.match(line.strip())
        if not m:
            continue

        channels.append((int(m.group("channel"), 10),
                         m.group("freq")))

    if not channels:
        print("Could not find frequencies for {}".format(args.intf))
        return

    if args.channels:
        match = set(int(c) for c in args.channels)
        channels = [(channel, freq) for channel, freq in channels
                    if channel in match]
        if not channels:
            print("None of channels {} found in {}".format(args.channels,
                                                           args.intf))
            return

    hop(args.intf, channels, 0.5)

if __name__ == "__main__":
    main()
