#!/usr/bin/python

import base64
import binascii
import os

fmt = """
    Base64Test {{
        bytes: &{},
        encoded: "{}",
    }},
""".strip()
fmt = "    " + fmt

for i in range(1, 8):
    buf = os.urandom(i)
    s = binascii.hexlify(buf)
    encoded = base64.b64encode(buf)
    print(fmt.format([ord(b) for b in buf], encoded))
