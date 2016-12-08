package main

import (
	"crypto/tls"
	"fmt"
	"log"
	"net"
    "sync"

	"http2proxy/server/manager"
	"http2proxy/server/options"
	"http2proxy/util"
)

var (                                                                            
    finished      = false                                                        
    finishedMutex sync.RWMutex                                                   
)                                                                                

func handleClient(conn net.Conn) {
	log.Println("connection from", conn.RemoteAddr())
    conn.Close()
}

func listenLoop(l net.Listener, mgr *manager.ClientMgr) {
    go func() {
        <-done
        l.Close()
    }()

	for {
		conn, err := l.Accept()
		if err != nil {
			log.Println("accept err", err)
			return
		}
		go mgr.HandleNewClientConn(conn)
	}
}

func listenTLS(mgr *manager.ClientMgr) {
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

	listenLoop(l, mgr)
}

func listen(mgr *manager.ClientMgr) {
	l, err := net.Listen("tcp", fmt.Sprintf(":%d", options.Port))
	if err != nil {
		log.Println("listen err", err)
		return
	}

	listenLoop(l, mgr)
}

func main() {
	err := options.Load()
	if err != nil {
		log.Println(err)
		return
	}

	var wg sync.WaitGroup
	done := make(chan bool)

	finish := func() {
		finishedMutex.Lock()
		if !finished {
			close(done)
			finished = true
		}
		finishedMutex.Unlock()
	}
    mgr := manager.NewClientMgr(done)

	wg.Add(1)
	go func() {
		defer wg.Done()
		listen(mgr)
        finish()
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		listenTLS(mgr)
        finish()
	}()

	util.WaitForSignal(func(s os.Signal) { finish() }, finish, done)
	wg.Wait()
}
