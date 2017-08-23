#!/usr/bin/python

import sys

m = {
        ' ': 22000,  # supposed to be similar to 'e'
        'A': 14810,
        'B': 2715,
        'C': 4943,
        'D': 7874,
        'E': 21912,
        'F': 4200,
        'G': 3693,
        'H': 10795,
        'I': 13318,
        'J': 188,
        'K': 1257,
        'L': 7253,
        'M': 4761,
        'N': 12666,
        'O': 14003,
        'P': 3316,
        'Q': 205,
        'R': 10977,
        'S': 11450,
        'T': 16587,
        'U': 5246,
        'V': 2019,
        'W': 3819,
        'X': 315,
        'Y': 3853,
        'Z': 128,
}

precision = 10000

# normalize before adding lower case (assume no case sensitivity for now)
total = sum(m.values())
print("total counts: {}".format(total))
print("precision: 1 / {}".format(precision))
for i in range(256):
    c = chr(i)
    if c in m:
        freq = int(m[c] * precision / total)
        m[c] = freq
print("rounded total: {} / {}".format(sum(m.values()), precision))

# add lower case frequencies
m.update({letter.lower(): freq for letter, freq in m.items()})


chunk_size = 16
for i in range(256 / chunk_size):
    for j in range(chunk_size):
        c = chr(i * chunk_size + j)
        sys.stdout.write("{}, ".format(m.get(c, 0)))
    sys.stdout.write('\n')
sys.stdout.flush()
