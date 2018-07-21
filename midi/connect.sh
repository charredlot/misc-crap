#!/bin/bash

set -e
KEYBOARD="Keystation 49"
OUTPUT="FLUID Synth"

# format looks like:
# client 24: 'Keystation 49' [type=kernel]
MIDI_IN=$(aconnect -i | grep "client.*$KEYBOARD" | awk '{ print $2 }' | \
          sed 's/://')

MIDI_OUT=$(aconnect -o | grep "client.*$OUTPUT" | awk '{ print $2 }' | \
           sed 's/://')

echo "MIDI in is: $MIDI_IN"
echo "MIDI out is: $MIDI_OUT"

if [ -z $MIDI_IN ] || [ -z $MIDI_OUT ]; then
    echo "Couldn't find matching input $KEYBOARD or output $OUTPUT"
    exit 1
fi

aconnect $MIDI_IN $MIDI_OUT
