package rcv

import (
	"bufio"
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"io"
	"log"
	"os"
	"strings"
	"sync"
	"time"

	"ssldaemon/daemon/options"
	"ssldaemon/protocol"
)

type Logfile struct {
	f        *os.File
	path     string
	wg       sync.WaitGroup
	done     bool
	doneChan chan struct{}
}

func NewLogfile(path string) *Logfile {
	return &Logfile{path: path, doneChan: make(chan struct{})}
}

func (l *Logfile) Sleep() {
	select {
	case <-time.After(time.Second * 5):
	case <-l.doneChan:
	}
}

func (l *Logfile) OpenLoop() {
	log.Println("opening", l.path)
	for !l.done {
		var err error

		l.f, err = os.Open(l.path)
		if err == nil {
			log.Println("opened", l.path)
			break
		}

		log.Println("open", l.path, err)
		l.Sleep()
	}
}

func (l *Logfile) ReadLoop(rcver Receiver) {
	// TODO: want inotify to handle eof, but
	// it's linux-only and go doesn't have a package
	rw := bytes.NewBuffer(make([]byte, 0, 4096))
	written := make(chan struct{})

	l.wg.Add(1)
	go func() {
		defer l.wg.Done()
		for !l.done {
			rcver.ReadFrom(rw)
			select {
			case <-time.After(time.Second * 5):
			case <-written: // basically pthread_cond_wait
			case <-l.doneChan:
			}
		}
	}()

	l.OpenLoop()

	r := bufio.NewReader(l.f)
	for !l.done {
		s, err := r.ReadString('\n')
		if err != nil {
			if options.Debug {
				log.Println("read", l.path, err)
			}
			if err != io.EOF {
				l.OpenLoop()
			}
			l.Sleep()
			continue
		}

		if len(s) == 0 {
			log.Println("empty")
			continue
		}

		tokens := strings.Split(s[:len(s)-1], " ")
		if len(tokens) == 0 {
			log.Println("bad num tokens", s)
			continue
		}

		if tokens[0] == "#" {
			continue
		}

		if len(tokens) < 3 {
			log.Println("bad num tokens", s)
			continue
		}

		switch tokens[0] {
		case "CLIENT_RANDOM":
			var hdr protocol.ClientRandomRecordHdr

			clientRandom, err := hex.DecodeString(tokens[1])
			if err != nil {
				log.Println(err, "\n", s)
				continue
			}

			if len(clientRandom) != len(hdr.ClientRandom) {
				log.Println("bad client random", s)
				continue
			}

			masterSecret, err := hex.DecodeString(tokens[2])
			if err != nil {
				log.Println(err, "\n", s)
				continue
			}

			if (len(masterSecret) == 0) || (len(masterSecret) > 255) {
				log.Println("bad master secret", s)
				continue
			}

			hdr.Hdr.Type = protocol.RecordClientRandom
			hdr.Hdr.MasterSecretLen = uint8(len(masterSecret))
			hdr.Hdr.Len = uint32(binary.Size(&hdr)) +
				uint32(hdr.Hdr.MasterSecretLen)

			for i := range clientRandom {
				hdr.ClientRandom[i] = clientRandom[i]
			}

			binary.Write(rw, binary.BigEndian, &hdr)
			binary.Write(rw, binary.BigEndian, masterSecret)
		case "RSA":
			var hdr protocol.PremasterSecretRecordHdr

			pms, err := hex.DecodeString(tokens[1])
			if err != nil {
				log.Println(err, "\n", s)
				continue
			}

			if (len(pms) == 0) || (len(pms) > 65535) {
				log.Println("bad premaster secret", s)
				continue
			}

			masterSecret, err := hex.DecodeString(tokens[2])
			if err != nil {
				log.Println(err, "\n", s)
				continue
			}

			if (len(masterSecret) == 0) || (len(masterSecret) > 255) {
				log.Println("bad master secret", s)
				continue
			}

			hdr.Hdr.Type = protocol.RecordPremasterSecret
			hdr.Hdr.MasterSecretLen = uint8(len(masterSecret))
			// hdr plus master secret plus pms and 2 bytes for pms len
			hdr.Hdr.Len = uint32(binary.Size(&hdr)) +
				uint32(hdr.Hdr.MasterSecretLen) +
				uint32(len(pms))
			hdr.PremasterSecretLen = uint16(len(pms))

			binary.Write(rw, binary.BigEndian, &hdr)
			binary.Write(rw, binary.BigEndian, pms)
			binary.Write(rw, binary.BigEndian, masterSecret)
		default:
			log.Println("unknown line", s)
			continue
		}

		if options.Debug {
			log.Println("sent line", s)
		}

		// basically a pthread_cond_signal
		var empty struct{}
		select {
		case written <- empty:
		default: // skip if it's full
		}
	}

	rw.Reset()
	l.wg.Wait()
}

func (l *Logfile) Cleanup() {
	if l.f != nil {
		l.f.Close()
		l.f = nil
	}
	l.done = true
	close(l.doneChan)
}
