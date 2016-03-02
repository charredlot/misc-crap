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

func launchListener(wg *sync.WaitGroup, l net.Listener,
	connRcv rcv.Receiver) {
	wg.Add(1)
	go func() {
		defer wg.Done()
		connRcv.Tick()
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		rcv.AcceptConns(l, connRcv)
	}()
}

func launchBrowserLog(wg *sync.WaitGroup, logfile *rcv.Logfile,
	logRcv rcv.Receiver) {
	wg.Add(1)
	go func() {
		defer wg.Done()
		logRcv.Tick()
	}()

	wg.Add(1)
	go func() {
		defer wg.Done()
		logfile.ReadLoop(logRcv)
	}()
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
	if l == nil {
		os.Exit(1)
	}
	defer l.Close()

	connRcv := initReceiver(options.ForwardHost, options.ForwardHandshake)
	defer connRcv.Cleanup()

	launchListener(&wg, l, connRcv)
	if options.BrowserLog != "" {
		logfile := rcv.NewLogfile(options.BrowserLog)
		defer logfile.Cleanup()

		logRcv := initReceiver(options.ForwardHost, options.ForwardHandshake)
		defer logRcv.Cleanup()

		launchBrowserLog(&wg, logfile, logRcv)
	}

	util.WaitForSignal(func() {})
}
