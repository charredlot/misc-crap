package main

import (
	"flag"
	"io/ioutil"
	"log"
	"net"
	"net/http"
	"time"
)

var (
	requests chan []byte
)

func forwarder(unixPath string, requests chan []byte) {
	socketFails := 0
	for {
		forward, err := net.Dial("unix", unixPath)
		if err != nil {
			socketFails += 1
			if socketFails > 6 {
				socketFails = 0
				log.Println("unix oops", err)
			}
			time.Sleep(time.Second * 5)
			continue
		}

		for req := range requests {
			var len32 uint32 = uint32(len(req))
			var lenBuf [4]byte

			// XXX: probably faster than binary.Write?
			lenBuf[0] = byte((len32 >> 24) & 0xff)
			lenBuf[1] = byte((len32 >> 16) & 0xff)
			lenBuf[2] = byte((len32 >> 8) & 0xff)
			lenBuf[3] = byte((len32 >> 0) & 0xff)

            _, err = forward.Write(lenBuf[:])
			if err != nil {
				log.Println("unix write error", err)
				break
			}
			_, err = forward.Write(req)
			if err != nil {
				log.Println("unix write error", err)
				break
			}
		}
	}
}

func handle(w http.ResponseWriter, req *http.Request) {
	// TODO: check content-type
	defer req.Body.Close()

	b, err := ioutil.ReadAll(req.Body)
	if err != nil {
		// though this probably means we can't read from the client anyways
		w.WriteHeader(500)
		w.Write([]byte(`{"error":"read error"}`))
		return
	}

	requests <- b
}

func main() {
	var unixPath string
	var httpListen string

	flag.StringVar(&httpListen, "l", ":9000", "http listen host:port")
	flag.StringVar(&unixPath, "f", "", "unix domain socket path")
	flag.Parse()

	if unixPath == "" {
		log.Println("unix path required")
		return
	}

	// scale to num cpus or something
	requests := make(chan []byte, 32)

	go forwarder(unixPath, requests)

	http.HandleFunc("/", handle)
	log.Println(http.ListenAndServe(httpListen, nil))
}
