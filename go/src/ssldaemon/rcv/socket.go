package rcv

import (
	"io"
	"log"
	"net"

	"ssldaemon/protocol"
)

type Receiver interface {
	ReadFrom(reader io.Reader)
	Tick()
	Cleanup()
}

type PrintReceiver struct{}

func handleConn(conn net.Conn, receiver Receiver) {
	defer conn.Close()

	s := conn.LocalAddr().String() + "->" + conn.RemoteAddr().String()
	err := protocol.ServerHandshake(conn, s)
	if err != nil {
		log.Println("handshake err:", err)
		return
	}

	receiver.ReadFrom(conn)
}

func AcceptConns(l net.Listener, receiver Receiver) {
	for {
		conn, err := l.Accept()
		if err != nil {
			log.Printf("accept err: %v\n", err)
			break
		}

		log.Println("accepted from ", conn.RemoteAddr())
		go handleConn(conn, receiver)
	}
}

func (pr *PrintReceiver) ReadFrom(reader io.Reader) {
	protocol.PrintSecrets(reader)
}

func (pr *PrintReceiver) Tick() {}

func (pr *PrintReceiver) Cleanup() {}
