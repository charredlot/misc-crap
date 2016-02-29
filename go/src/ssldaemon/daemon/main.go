package main

import (
	"log"
	"net"
	"os"
	"strings"
	"sync"

	"ssldaemon/daemon/options"
	"ssldaemon/proxy"
	"ssldaemon/rcv"
	"ssldaemon/util"
)

func isUnixHost(host string) bool {
	return strings.HasPrefix(host, "@")
}

func toUnixPath(host string) string {
	return host[len("@"):]
}

func initListener(host string) net.Listener {
	var l net.Listener
	var err error

	if isUnixHost(host) {
		unixPath := toUnixPath(host)

		os.Remove(unixPath)
		l, err = net.Listen("unix", unixPath)
		if err == nil {
			// so all users can connect
			os.Chmod(unixPath, os.ModePerm)
		}
	} else {
		l, err = net.Listen("tcp", host)
	}

	if err != nil {
		log.Println(err)
		return nil
	} else {
		return l
	}
}

func initReceiver(host string, forwardHandshake bool) rcv.Receiver {
	if options.Diagnostic {
		return &rcv.PrintReceiver{}
	} else {
		if isUnixHost(host) {
			return proxy.NewProxy(toUnixPath(host), "unix", forwardHandshake)
		} else {
			return proxy.NewProxy(host, "tcp", forwardHandshake)
		}
	}
}

func main() {
	var wg sync.WaitGroup

	err := options.Load()
	if err != nil {
		log.Println(err)
		return
	}

	defer wg.Wait()

	l := initListener(options.ListenHost)
	defer l.Close()

	rcver := initReceiver(options.ForwardHost, options.ForwardHandshake)
	defer rcver.Cleanup()

	wg.Add(1)
	go func() {
		defer wg.Done()
		go rcver.Tick()
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		rcv.AcceptConns(l, rcver)
	}()

	util.WaitForSignal(func() {})
}
