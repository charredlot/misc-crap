package main

import (
//    "crypto/tls"
    "log"
    "net/http"
    "net/http/httputil"

    "golang.org/x/net/http2"
)

var (
    certFile = "/home/charlotte/cavium/local/rsa_signed.cert.pem"
    keyFile = "/home/charlotte/cavium/local/rsa_signed.key.pem"
)

func setupMultiplexer(w http.ResponseWriter, r *http.Request) {
    httputil.DumpRequest(r, false)
    w.Write([]byte("boop"))
}

func http2Server(addr string) *http.Server {
    //tlsCfg := &tls.Config{CipherSuites: []uint16{tls.TLS_RSA_WITH_AES_256_CBC_SHA}}
    s := &http.Server{Addr: addr, TLSConfig: nil}

    err := http2.ConfigureServer(s,
        &http2.Server{PermitProhibitedCipherSuites: true})
    if err != nil {
        log.Println("http2 err", err)
        return nil
    }

    return s
}

func main() {
    tlsServer := http2Server(":10443")
    if tlsServer == nil {
        return
    }

    plainServer := http2Server(":10080")
    if plainServer == nil {
        return
    }

    http.HandleFunc("/", setupMultiplexer)
    go func() {
        err := tlsServer.ListenAndServeTLS(certFile, keyFile)
        if err != nil {
            log.Println("listen err", err)
            return
        }
    }()
    err := plainServer.ListenAndServe()
    if err != nil {
        log.Println("listen err", err)
        return
    }
}
