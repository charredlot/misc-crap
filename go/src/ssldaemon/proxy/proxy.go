package proxy

import (
	"io"
	"log"
	"net"
	"time"

	"ssldaemon/protocol"
)

type Proxy struct {
	host  string
	proto string
	conn  net.Conn

	forwardHandshake bool
	connErr          bool
	done             chan bool
}

func (proxy *Proxy) Dial() error {
	if proxy.conn != nil {
		proxy.conn.Close()
		proxy.conn = nil
	}

	// TODO: tls
	conn, err := net.Dial(proxy.proto, proxy.host)
	if err != nil {
		log.Println(err)
		proxy.connErr = true
		return err
	}

	s := conn.LocalAddr().String() + "->" + conn.RemoteAddr().String()
	log.Printf("%s: dial success\n", s)

	if proxy.forwardHandshake {
		err = protocol.ClientHandshake(conn, s)
		if err != nil {
			log.Println("handshake err:", err)
			proxy.connErr = true
			return err
		}
	}

	proxy.conn = conn
	proxy.connErr = false
	return nil
}

func NewProxy(host string, proto string, forwardHandshake bool) *Proxy {
	proxy := &Proxy{host: host,
		proto:            proto,
		forwardHandshake: forwardHandshake,
		done:             make(chan bool)}
	proxy.Dial()
	return proxy
}

func (proxy *Proxy) ReadFrom(reader io.Reader) {
	// proxy.conn might become nil
	conn := proxy.conn
	if conn == nil {
		return
	}

	_, err := io.Copy(conn, reader)
	if (err != nil) && (err != io.EOF) {
		// TODO: restart on error or something
		log.Println(err)
		proxy.connErr = true
	}
}

func (proxy *Proxy) Tick() {
	tick := time.NewTicker(15 * time.Second)
	defer tick.Stop()

topLoop:
	for {
		select {
		case _ = <-tick.C:
			if proxy.connErr {
				log.Println("proxy conn error? redialing")
				proxy.Dial()
			}
		case _, ok := <-proxy.done:
			if !ok {
				break topLoop
			}
		}
	}
}

func (proxy *Proxy) Cleanup() {
	if proxy.conn != nil {
		proxy.conn.Close()
	}
	close(proxy.done)
}
