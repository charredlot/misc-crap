#!/usr/bin/python

import ssl
import sys


host = sys.argv[1]
try:
    port = int(sys.argv[2])
except Exception:
    port = 443

pem_cert = ssl.get_server_certificate( (host, port) )
sys.stdout.write(pem_cert)
sys.stdout.flush()
