package main

import (
    "flag"
    "fmt"
    "io"
    "net"
    "os"
    "os/signal"
    "syscall"
)

func waitForSignal() {
    c := make(chan os.Signal, 1)
    signal.Notify(c, os.Interrupt, syscall.SIGTERM, syscall.SIGHUP)

    sig := <-c
    fmt.Println("Got signal:", sig)
}

func main() {
    var err error
    var listenHost, forwardHost string

    flag.StringVar(&listenHost, "listen-host", "", "listen tcp host")
    flag.StringVar(&forwardHost, "forward-host", "", "forward to tcp host")
    flag.Parse()

    if (listenHost == "") || (forwardHost == "") {
        fmt.Println("path host and port are required\n")
        return
    }

    lConn, err := net.Listen("tcp", listenHost)
    if err != nil {
        fmt.Printf("tcp connect error %v\n", err)
        return
    }
    defer lConn.Close()

    done := make(chan struct{})
    defer close(done)

    go acceptConns(lConn, forwardHost, done)

    waitForSignal()
}

func pipeConn(dst net.Conn, src net.Conn) {
    defer src.Close()
    _, err := io.Copy(dst, src)
    if err != nil {
        fmt.Println(err)
    }
}

func acceptConns(l net.Listener, forwardHost string, done chan struct{}) {
    defer l.Close()
    for {
        conn, err := l.Accept()
        if err != nil {
            fmt.Printf("accept err: %v\n", err)
            select {
            case <-done:
                fmt.Printf("done\n")
                break
            default:
                continue
            }
        }

        fConn, err := net.Dial("tcp", forwardHost)
        if err != nil {
            fmt.Printf("forward connect error %v\n", err)
            conn.Close()
            continue
        }

        fmt.Println(conn.RemoteAddr(), "to", fConn.RemoteAddr())
        go pipeConn(conn, fConn)
        go pipeConn(fConn, conn)
    }
}
