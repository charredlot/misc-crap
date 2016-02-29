package main

import (
	"log"
	"net"
	"os"
	"sync"

	"ssldaemon/agent/options"
	"ssldaemon/proxy"
	"ssldaemon/rcv"
	"ssldaemon/util"
)

func main() {
	var wg sync.WaitGroup

	err := options.Load()
	if err != nil {
		log.Println(err)
		return
	}

	os.Remove(options.UnixSocketPath)
	l, err := net.Listen("unix", options.UnixSocketPath)
	if err != nil {
		log.Println("unix listen error %v\n", err)
		return
	}
	// so all users can connect
	os.Chmod(options.UnixSocketPath, os.ModePerm)

	defer wg.Wait()
	defer l.Close()

	var rcver rcv.Receiver
	if options.Diagnostic {
		rcver = &rcv.PrintReceiver{}
	} else {
		rcver = proxy.NewProxy(options.RemoteHost, "tcp", false)
	}

	defer rcver.Cleanup()

	wg.Add(1)
	go func() {
		defer wg.Done()
		rcv.AcceptConns(l, rcver)
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		rcver.Tick()
	}()
	util.WaitForSignal(func() {})
}
