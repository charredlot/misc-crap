package util

import (
	"log"
	"os"
	"os/signal"
	"syscall"
)

func WaitForSignal(sigFunc func()) {
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, os.Kill,
		syscall.SIGTERM, syscall.SIGHUP, syscall.SIGUSR1)

mainLoop:
	for {
		select {
		case sig := <-c:
			log.Println("Caught signal", sig)
			if sig == syscall.SIGUSR1 {
				sigFunc()
				continue
			}
			break mainLoop
		}
	}
}
