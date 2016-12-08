package main

import (
    "fmt"
	"log"
	"net"
	"os"
	"sync"

	"http2proxy/client/backend"
	"http2proxy/client/options"
	"http2proxy/util"
)

var (
	finished      = false
	finishedMutex sync.RWMutex
)

func listenProxy(listen string, bend *backend.Backend, done chan bool) {
	l, err := net.Listen("tcp", listen)
	if err != nil {
        log.Println(listen, err)
		return
	}

    go func() {
	    defer l.Close()
        <-done
    }()

	for {
		conn, err := l.Accept()
		if err != nil {
            if !finished {
                log.Println("listen on", listen, "error", err)
            }
			return
		}

		log.Println("connection from", conn.RemoteAddr())
        bend.Proxy(fmt.Sprintf("service from %s", listen), conn, done)
	}
}

func main() {
	options.Load()

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

	bend := backend.NewBackend(options.Connect, done)
	for i := range options.Listen {
		wg.Add(1)
		go func(listen string) {
			defer wg.Done()
			listenProxy(listen, bend, done)
            // this should loop forever without errors
            finish()
		}(options.Listen[i])
	}

	wg.Add(1)
	go func() {
		defer wg.Done()
        bend.Connect()
	}()

	util.WaitForSignal(func(s os.Signal) {
		finish()
	}, finish, done)
	wg.Wait()
}
