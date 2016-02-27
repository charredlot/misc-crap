package options

import (
    "errors"
    "flag"
)

var (
    ListenHost string
    ForwardHost string
    ForwardHandshake bool
    Debug bool
    Diagnostic bool
)

func Load() error {
    flag.StringVar(&ListenHost, "l", "@/tmp/ssl_agent",
        "host or unix socket (if starts with @) to listen on")
    flag.StringVar(&ForwardHost, "f", "localhost:12000",
        "host to forward to")
    flag.BoolVar(&ForwardHandshake, "handshake", false,
        "whether to handshake with the forwarder")

    flag.BoolVar(&Debug, "d", false, "debug mode (verbose)")
    flag.BoolVar(&Diagnostic, "diagnostic", false, "diagnostic mode")

    flag.Parse()

    if ListenHost == "" {
        return errors.New("-l $listen-host is required\n")
    }

    return nil
}
