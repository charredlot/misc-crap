package main

import (
    "crypto/tls"
    "fmt"
    "log"
    "net"
    "net/http"
    "time"

    "golang.org/x/net/http2"

    "http2proxy/server/options"
)

type http2Handler struct {
}

func (h *http2Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
    log.Println("method", r.Method)
    log.Println("path", r.URL)
    log.Println("headers", r.Header)
    log.Printf("%+v\n", r)
    w.Write([]byte("boppity"))
}

func handle(s *http2.Server, sopts *http2.ServeConnOpts, conn net.Conn) {
    defer conn.Close()
    log.Println("conn", conn.RemoteAddr())
    s.ServeConn(conn, sopts)
    log.Println("conn done")
}

func listenLoop(l net.Listener) {
    s := &http2.Server{IdleTimeout: time.Second * 30}
    hh := &http2Handler{}
    sopts := &http2.ServeConnOpts{
        BaseConfig: &http.Server{
            Handler: hh,
        },
        Handler: hh,
    }

    for {
        conn, err := l.Accept()
        if err != nil {
            log.Println("accept err", err)
            return
        }
        go handle(s, sopts, conn)
    }
}

func listenTLS() {
    cert, err := tls.LoadX509KeyPair(options.CertPath, options.KeyPath)
    if err != nil {
        log.Println("tls err", err)
        return
    }

    cfg := &tls.Config{
        Certificates: []tls.Certificate{cert},
    }

    l, err := tls.Listen("tcp", fmt.Sprintf(":%d", options.TLSPort), cfg)
    if err != nil {
        log.Println("listen err", err)
        return
    }
    defer l.Close()

    listenLoop(l)
}

func listen() {
    l, err := net.Listen("tcp", fmt.Sprintf(":%d", options.Port))
    if err != nil {
        log.Println("listen err", err)
        return
    }
    defer l.Close()

    listenLoop(l)
}

func main() {
    err := options.Load()
    if err != nil {
        log.Println(err)
        return
    }

    go listen()
    listenTLS()
}
