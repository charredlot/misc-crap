package options

import (
    "errors"
    "flag"
)

var (
    UnixSocketPath string
    RemoteHost string
    Debug bool
    Diagnostic bool
)

func Load() error {
    flag.StringVar(&UnixSocketPath, "unix-path", "/tmp/ssl_hijack",
        "listen unix socket path")
    flag.StringVar(&RemoteHost, "forward", "localhost:12000",
        "host:port to forward to")
    flag.BoolVar(&Debug, "d", false, "debug mode (verbose)")
    flag.BoolVar(&Diagnostic, "diagnostic", false,
        "diagnostic mode: just print")
    flag.Parse()

    if UnixSocketPath == "" {
        return errors.New("unix-path is required\n")
    }

    return nil
}
