package protocol

import (
	"encoding/binary"
	"encoding/hex"
	"io"
	"log"
)

const (
	handshakeVersion = 1
	handshakeHdrLen  = 3
)

type PktHandshake struct {
	Version uint8
	HdrLen  uint16
}

type PktSessionHdr struct {
	ClientRandom    [32]byte
	ServerRandom    [32]byte
	MasterSecretLen uint8
}

type SessionMapping struct {
	ClientRandom [32]byte
	ServerRandom [32]byte
	MasterSecret []byte
}

func NewSessionMapping(clientRandom, serverRandom [32]byte,
	masterSecret []byte) SessionMapping {

	masterSecretCopy := make([]byte, len(masterSecret))
	copy(masterSecretCopy, masterSecret)

	return SessionMapping{ClientRandom: clientRandom,
		ServerRandom: serverRandom,
		MasterSecret: masterSecretCopy}
}

func sendHandshakeBytes(rw io.ReadWriter, id string) error {
	var h PktHandshake

	h.Version = handshakeVersion
	h.HdrLen = handshakeHdrLen

	err := binary.Write(rw, binary.BigEndian, &h)
	if err == nil {
		log.Printf("%s: handshake sent %+v\n", id, h)
	}
	return err
}

func rcvHandshakeBytes(rw io.ReadWriter, id string) error {
	var h PktHandshake

	err := binary.Read(rw, binary.BigEndian, &h)
	if err != nil {
		return err
	}

	/* TODO: sizeof PktHandshake*/
	if h.HdrLen > handshakeHdrLen {
		skip := int(h.HdrLen) - handshakeHdrLen
		b := make([]byte, skip)

		for skip > 0 {
			n, err := rw.Read(b[:skip])
			if err != nil {
				return err
			}
			skip -= n
		}
	}

	log.Printf("%s: handshake rcvd %+v\n", id, h)
	return nil
}

func ClientHandshake(rw io.ReadWriter, id string) error {
	err := sendHandshakeBytes(rw, id)
	if err != nil {
		return err
	}

	err = rcvHandshakeBytes(rw, id)
	if err != nil {
		return err
	}

	log.Printf("%s: client handshake done\n", id)
	return nil
}

func ServerHandshake(rw io.ReadWriter, id string) error {
	err := rcvHandshakeBytes(rw, id)
	if err != nil {
		return err
	}

	err = sendHandshakeBytes(rw, id)
	if err != nil {
		return err
	}

	log.Printf("%s: server handshake done\n", id)
	return nil
}

func PrintSecrets(r io.Reader) {
	var masterSecret [255]byte
	for {
		var hdr PktSessionHdr

		err := binary.Read(r, binary.BigEndian, &hdr)
		if err != nil {
			if err != io.EOF {
				log.Println("read session hdr err:", err)
			}
			return
		}

		log.Println("client random", "\n"+hex.Dump(hdr.ClientRandom[:]))
		log.Println("server random", "\n"+hex.Dump(hdr.ServerRandom[:]))

		_, err = io.ReadFull(r, masterSecret[:hdr.MasterSecretLen])
		if err != nil {
			log.Println("read master secret err:", err)
			return
		}

		log.Println("master secret",
			"\n"+hex.Dump(masterSecret[:hdr.MasterSecretLen]))
	}
}
