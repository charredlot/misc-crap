package util

import (
	"os"
	"os/signal"
	"syscall"
)

func DefaultSignalsChan() chan os.Signal {
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, os.Interrupt, syscall.SIGTERM, syscall.SIGHUP)
    return sigs
}

func WaitForSignals() os.Signal {
    sigs := DefaultSignalsChan()
    return <-sigs
}
