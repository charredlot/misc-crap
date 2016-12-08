package main

import (
    "crypto/tls"
    "flag"
    "log"
    "net"
    "net/http"
    "net/url"
    "io/ioutil"

    "golang.org/x/net/http2"
)

var (
    useTLS = true
)

func connect(c *http.Client, scheme, host string) error {
    req := &http.Request{
        Method: http.MethodConnect,
        URL: &url.URL{
            Scheme: scheme,
            Host: host,
            Path: "/",
        },
        Host: host,
    }

    rsp, err := c.Do(req)
    if err != nil {
        return err
    }
    defer rsp.Body.Close()

    log.Printf("rsp %+v\n", rsp)
    body, err := ioutil.ReadAll(rsp.Body)
    if err != nil {
        return err
    }
    log.Println(string(body))

    return nil
}

func setupClient() *http.Client {
    var c *http.Client

    if useTLS {
        c = &http.Client{
            Transport: &http2.Transport{
                TLSClientConfig: &tls.Config{
                    InsecureSkipVerify: true,
                },
            },
        }
    } else {
        dial := func(network, addr string, cfg *tls.Config) (net.Conn, error) {
            log.Println(network, addr)
            return net.Dial("tcp", addr)
        }
        c = &http.Client{
            Transport:&http2.Transport{
                DialTLS: dial,
                AllowHTTP: true,
            },
        }
    }

    return c
}

func main() {
    flag.BoolVar(&useTLS, "tls", false, "whether to use tls")
    flag.Parse()

    var scheme string
    var addr string

    if useTLS {
        scheme = "https"
        addr = "localhost:10443"
    } else {
        scheme = "http"
        addr = "localhost:10080"
    }

    c := setupClient()

    err := connect(c, scheme, addr)
    if err != nil {
        log.Println("connect err", err)
        return
    }
}
