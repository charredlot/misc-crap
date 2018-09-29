#!/usr/bin/python3

import json
import sys

from midiutil.MidiFile import MIDIFile, ProgramChange

from notes import *


def insert_scale(midi_file, track, mode, start_beat):
    last_beat = None
    volume = 50
    for channel in range(1):
        notes = [
            (n, 1) for n in mode_scale(note_from_name('C4'), mode)
        ]

        beat = start_beat
        for n, duration in notes:
            midi_file.addNote(track, channel, n.midi_pitch, beat, duration,
                              volume)
            beat += duration
        last_beat = beat

    return last_beat


def main():
    num_tracks = 1
    track = 0
    f = MIDIFile(num_tracks)

    for track in range(num_tracks):
        beat = 0  # which beat to start on
        bpm = 120
        f.addTrackName(track, beat, "Track " + str(track))
        f.addTempo(track, beat, bpm)

    """
    sample all modes
    for mode in (IONIAN_MODE,
                 DORIAN_MODE,
                 PHRYGIAN_MODE,
                 LYDIAN_MODE,
                 MIXOLYDIAN_MODE,
                 AEOLIAN_MODE,
                 LOCRIAN_MODE):
        beat = insert_scale(f, track, mode, beat)
    """

    """
    make sure notes for mode make sense
    volume = 50
    channel = 0
    duration = 1
    for n in mode_gen(note_from_name("C4"), DORIAN_MODE):
        print(n)
        f.addNote(track, channel, n.midi_pitch, beat, duration, volume)
        beat += duration
    """

    """
    pitch wheel test
    channel = 0
    volume = 50
    note_duration = 16
    n = note_from_name("C4")
    f.addNote(track, channel, n.midi_pitch, beat, note_duration, volume)
    for wheel in range(0, 8192, 1024):
        print(wheel, beat)
        f.addPitchWheelEvent(track, channel, beat, wheel)
        beat += 1
    for wheel in range(8192, 0, -1024):
        print(wheel, beat)
        f.addPitchWheelEvent(track, channel, beat, wheel)
        beat += 1
    """

    """
    circle of fifths
    channel = 0
    volume = 50
    duration = 1
    first = note_from_name("C2")
    last = note_from_name("C6")

    n = first
    for i in range(64):
        chord = minor_triad(n)
        for note in chord:
            f.addNote(track, channel, note.midi_pitch, beat, duration, volume)
            beat += 1

        print(n)
        p = n.midi_pitch + 7
        if p > last.midi_pitch:
            p = first.midi_pitch + p - last.midi_pitch
        n = note_from_pitch(p)
        beat += 1
    """

    """
    channels
    volume = 50
    duration = 1
    bass_note = note_from_name("C1")
    for track, program in enumerate((0, 32, 40, 42, 68, 71)):
        channel = track
        f.addProgramChange(track, channel, 0, program)

        beat = 0
        for n in mode_gen(note_from_pitch(bass_note.midi_pitch + track * 3),
                          IONIAN_MODE):
            f.addNote(track, channel, n.midi_pitch, beat, duration, volume)
            beat += duration
    """

    """
    drums
    """
    # channel 10 for drums is a default to timidity, but may be different
    # similarly all these note to drum mappings are for freepats.cfg
    volume = 75
    channel = 10
    duration = 1
    sticks = note_from_pitch(31)
    kick = note_from_pitch(35)
    snare = note_from_pitch(38)
    hihat = note_from_pitch(44)
    for n in (snare, snare, kick, hihat, sticks, sticks, snare, sticks, kick):
        f.addNote(track, channel, n.midi_pitch, beat, duration, volume)
        beat += duration


    with open("test.mid", 'wb') as out:
       f.writeFile(out)


if __name__ == "__main__":
    main()
