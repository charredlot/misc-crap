package options

import (
    "errors"
    "flag"
)

var (
    CertPath string
    KeyPath string
    Port int
    TLSPort int
)

func Load() error {
    flag.StringVar(&CertPath, "cert", "cert.pem", "server certificate")
    flag.StringVar(&KeyPath, "key", "key.pem", "server certificate key")
    flag.IntVar(&Port, "port", 10080, "port for cleartext http2")
    flag.IntVar(&TLSPort, "tls-port", 10443, "tls port for http2")
    flag.Parse()

    if Port < 0 || Port > 65535 {
        return errors.New("bad port value")
    }

    if TLSPort < 0 || TLSPort > 65535 {
        return errors.New("bad TLS port value")
    }

    return nil
}
