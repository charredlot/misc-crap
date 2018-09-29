from collections import namedtuple
from itertools import cycle


Note = namedtuple('Note', ['midi_pitch', 'name', 'freq'], verbose=False)

IONIAN_MODE = [2, 2, 1, 2, 2, 2, 1]
DORIAN_MODE = [2, 1, 2, 2, 2, 1, 2]
PHRYGIAN_MODE = [1, 2, 2, 2, 1, 2, 2]
LYDIAN_MODE = [2, 2, 2, 1, 2, 2, 1]
MIXOLYDIAN_MODE = [2, 2, 1, 2, 2, 2]
AEOLIAN_MODE = [2, 1, 2, 2, 1, 2, 2]
LOCRIAN_MODE = [1, 2, 2, 1, 2, 2, 2]

_notes = [
    Note(midi_pitch=127, name='G9', freq=12543.85),
    Note(midi_pitch=126, name='F#9', freq=11839.82),
    Note(midi_pitch=125, name='F9', freq=11175.3),
    Note(midi_pitch=124, name='E9', freq=10548.08),
    Note(midi_pitch=123, name='D#9', freq=9956.06),
    Note(midi_pitch=122, name='D9', freq=9397.27),
    Note(midi_pitch=121, name='C#9', freq=8869.84),
    Note(midi_pitch=120, name='C9', freq=8372.02),
    Note(midi_pitch=119, name='B8', freq=7902.13),
    Note(midi_pitch=118, name='A#8', freq=7458.62),
    Note(midi_pitch=117, name='A8', freq=7040.0),
    Note(midi_pitch=116, name='G#8', freq=6644.88),
    Note(midi_pitch=115, name='G8', freq=6271.93),
    Note(midi_pitch=114, name='F#8', freq=5919.91),
    Note(midi_pitch=113, name='F8', freq=5587.65),
    Note(midi_pitch=112, name='E8', freq=5274.04),
    Note(midi_pitch=111, name='D#8', freq=4978.03),
    Note(midi_pitch=110, name='D8', freq=4698.64),
    Note(midi_pitch=109, name='C#8', freq=4434.92),
    Note(midi_pitch=108, name='C8', freq=4186.01),
    Note(midi_pitch=107, name='B7', freq=3951.07),
    Note(midi_pitch=106, name='A#7', freq=3729.31),
    Note(midi_pitch=105, name='A7', freq=3520.0),
    Note(midi_pitch=104, name='G#7', freq=3322.44),
    Note(midi_pitch=103, name='G7', freq=3135.96),
    Note(midi_pitch=102, name='F#7', freq=2959.96),
    Note(midi_pitch=101, name='F7', freq=2793.83),
    Note(midi_pitch=100, name='E7', freq=2637.02),
    Note(midi_pitch=99, name='D#7', freq=2489.02),
    Note(midi_pitch=98, name='D7', freq=2349.32),
    Note(midi_pitch=97, name='C#7', freq=2217.46),
    Note(midi_pitch=96, name='C7', freq=2093.0),
    Note(midi_pitch=95, name='B6', freq=1975.53),
    Note(midi_pitch=94, name='A#6', freq=1864.66),
    Note(midi_pitch=93, name='A6', freq=1760.0),
    Note(midi_pitch=92, name='G#6', freq=1661.22),
    Note(midi_pitch=91, name='G6', freq=1567.98),
    Note(midi_pitch=90, name='F#6', freq=1479.98),
    Note(midi_pitch=89, name='F6', freq=1396.91),
    Note(midi_pitch=88, name='E6', freq=1318.51),
    Note(midi_pitch=87, name='D#6', freq=1244.51),
    Note(midi_pitch=86, name='D6', freq=1174.66),
    Note(midi_pitch=85, name='C#6', freq=1108.73),
    Note(midi_pitch=84, name='C6', freq=1046.5),
    Note(midi_pitch=83, name='B5', freq=987.77),
    Note(midi_pitch=82, name='A#5', freq=932.33),
    Note(midi_pitch=81, name='A5', freq=880.0),
    Note(midi_pitch=80, name='G#5', freq=830.61),
    Note(midi_pitch=79, name='G5', freq=783.99),
    Note(midi_pitch=78, name='F#5', freq=739.99),
    Note(midi_pitch=77, name='F5', freq=698.46),
    Note(midi_pitch=76, name='E5', freq=659.26),
    Note(midi_pitch=75, name='D#5', freq=622.25),
    Note(midi_pitch=74, name='D5', freq=587.33),
    Note(midi_pitch=73, name='C#5', freq=554.37),
    Note(midi_pitch=72, name='C5', freq=523.25),
    Note(midi_pitch=71, name='B4', freq=493.88),
    Note(midi_pitch=70, name='A#4', freq=466.16),
    Note(midi_pitch=69, name='A4', freq=440.0),
    Note(midi_pitch=68, name='G#4', freq=415.3),
    Note(midi_pitch=67, name='G4', freq=392.0),
    Note(midi_pitch=66, name='F#4', freq=369.99),
    Note(midi_pitch=65, name='F4', freq=349.23),
    Note(midi_pitch=64, name='E4', freq=329.63),
    Note(midi_pitch=63, name='D#4', freq=311.13),
    Note(midi_pitch=62, name='D4', freq=293.66),
    Note(midi_pitch=61, name='C#4', freq=277.18),
    Note(midi_pitch=60, name='C4', freq=261.63),
    Note(midi_pitch=59, name='B3', freq=246.94),
    Note(midi_pitch=58, name='A#3', freq=233.08),
    Note(midi_pitch=57, name='A3', freq=220.0),
    Note(midi_pitch=56, name='G#3', freq=207.65),
    Note(midi_pitch=55, name='G3', freq=196.0),
    Note(midi_pitch=54, name='F#3', freq=185.0),
    Note(midi_pitch=53, name='F3', freq=174.61),
    Note(midi_pitch=52, name='E3', freq=164.81),
    Note(midi_pitch=51, name='D#3', freq=155.56),
    Note(midi_pitch=50, name='D3', freq=146.83),
    Note(midi_pitch=49, name='C#3', freq=138.59),
    Note(midi_pitch=48, name='C3', freq=130.81),
    Note(midi_pitch=47, name='B2', freq=123.47),
    Note(midi_pitch=46, name='A#2', freq=116.54),
    Note(midi_pitch=45, name='A2', freq=110.0),
    Note(midi_pitch=44, name='G#2', freq=103.83),
    Note(midi_pitch=43, name='G2', freq=98.0),
    Note(midi_pitch=42, name='F#2', freq=92.5),
    Note(midi_pitch=41, name='F2', freq=87.31),
    Note(midi_pitch=40, name='E2', freq=82.41),
    Note(midi_pitch=39, name='D#2', freq=77.78),
    Note(midi_pitch=38, name='D2', freq=73.42),
    Note(midi_pitch=37, name='C#2', freq=69.3),
    Note(midi_pitch=36, name='C2', freq=65.41),
    Note(midi_pitch=35, name='B1', freq=61.74),
    Note(midi_pitch=34, name='A#1', freq=58.27),
    Note(midi_pitch=33, name='A1', freq=55.0),
    Note(midi_pitch=32, name='G#1', freq=51.91),
    Note(midi_pitch=31, name='G1', freq=49.0),
    Note(midi_pitch=30, name='F#1', freq=46.25),
    Note(midi_pitch=29, name='F1', freq=43.65),
    Note(midi_pitch=28, name='E1', freq=41.2),
    Note(midi_pitch=27, name='D#1', freq=38.89),
    Note(midi_pitch=26, name='D1', freq=36.71),
    Note(midi_pitch=25, name='C#1', freq=34.65),
    Note(midi_pitch=24, name='C1', freq=32.7),
    Note(midi_pitch=23, name='B0', freq=30.87),
    Note(midi_pitch=22, name='A#0', freq=29.14),
    Note(midi_pitch=21, name='A0', freq=27.5),
]

_note_min = None
_note_max = None
_note_by_pitch = None
_note_by_name = None


def note_min():
    global _note_min
    if _note_min is None:
        _note_min = min(_notes, key=lambda n: n.midi_pitch)
    return _note_min


def note_max():
    global _note_max
    if _note_max is None:
        _note_max = max(_notes, key=lambda n: n.midi_pitch)
    return _note_max


def note_from_pitch(pitch):
    global _note_by_pitch
    if _note_by_pitch is None:
        _note_by_pitch = {n.midi_pitch: n for n in _notes}
    return _note_by_pitch.get(pitch)


def note_from_name(name):
    global _note_by_name
    if _note_by_name is None:
        _note_by_name = {n.name: n for n in _notes}
    return _note_by_name.get(name)


def mode_scale(initial_note, mode):
    scale = [initial_note]
    p = initial_note.midi_pitch
    for steps in mode:
        p += steps
        scale.append(note_from_pitch(p))
    return scale


def mode_gen(initial_note, mode):
    p = initial_note.midi_pitch
    last_p = note_max()
    yield initial_note
    for steps in cycle(iter(mode)):
        p += steps
        if p > last_p.midi_pitch:
            break
        yield note_from_pitch(p)


def major_triad(bass_note):
    # major third is 4 half-steps, major fifth is 7
    return (bass_note,
            note_from_pitch(bass_note.midi_pitch + 4),
            note_from_pitch(bass_note.midi_pitch + 7))


def minor_triad(bass_note):
    # minor third is 3 half-steps, major fifth is 7
    return (bass_note,
            note_from_pitch(bass_note.midi_pitch + 3),
            note_from_pitch(bass_note.midi_pitch + 7))
