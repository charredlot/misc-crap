package protocol

import (
	"bufio"
	"encoding/binary"
	"encoding/hex"
    "errors"
    "fmt"
	"io"
	"log"
)

const (
	HandshakeVersion = 1

	RecordClientRandom        = 1
	RecordServerRandom        = 2
	RecordClientServerRandoms = 3
	RecordPremasterSecret     = 4
)

type HandshakeHdr struct {
	Version uint8
	HdrLen  uint16
}

type RecordHdr struct {
	Type            uint8
	Len             uint32
	MasterSecretLen uint8
}

type ClientRandomRecordHdr struct {
	Hdr          RecordHdr
	ClientRandom [32]byte
}

type PremasterSecretRecordHdr struct {
	Hdr                RecordHdr
	PremasterSecretLen uint16
}

func sendHandshakeBytes(rw io.ReadWriter, id string) error {
	var h HandshakeHdr

	h.Version = HandshakeVersion
	h.HdrLen = uint16(binary.Size(&h))

	err := binary.Write(rw, binary.BigEndian, &h)
	if err == nil {
		log.Printf("%s: handshake sent %+v\n", id, h)
	}
	return err
}

func rcvHandshakeBytes(rw io.ReadWriter, id string) error {
	var h HandshakeHdr

	err := binary.Read(rw, binary.BigEndian, &h)
	if err != nil {
		return err
	}

	/* TODO: sizeof HandshakeHdr */
	if h.HdrLen > uint16(binary.Size(&h)) {
		skip := int(h.HdrLen) - binary.Size(&h)
        if skip < 0 {
            return errors.New(fmt.Sprintf("bad hdr len %+v", h))
        }

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

func PrintSecrets(reader io.Reader) {
	var masterSecret [255]byte
	var clientRandom [32]byte
	var serverRandom [32]byte

	pms := make([]byte, 65535)
	r := bufio.NewReader(reader)
	for {
		var hdr RecordHdr

		err := binary.Read(r, binary.BigEndian, &hdr)
		if err != nil {
			if err != io.EOF {
				log.Println("read session hdr err:", err)
			}
			return
		}

		log.Printf("%+v\n", hdr)

		switch hdr.Type {
		case RecordClientRandom, RecordClientServerRandoms:
			_, err = io.ReadFull(r, clientRandom[:])
			if err != nil {
				log.Println("read client err:", err)
				return
			}

			log.Println("client random", "\n"+hex.Dump(clientRandom[:]))
			fallthrough
		case RecordServerRandom:
			if hdr.Type == RecordClientRandom {
				// can't use fallthrough in an if ):
				break
			}

			_, err = io.ReadFull(r, serverRandom[:])
			if err != nil {
				log.Println("read server random err:", err)
				return
			}

			log.Println("server random", "\n"+hex.Dump(serverRandom[:]))
		case RecordPremasterSecret:
			var pmsLen uint16
			err := binary.Read(r, binary.BigEndian, &pmsLen)
			if (err != nil) || (pmsLen == 0) {
				log.Println("read premaster secret len err:", err)
				return
			}

			_, err = io.ReadFull(r, pms[:pmsLen])
			if err != nil {
				log.Println("read premaster secret err:", err)
				return
			}

			log.Println("premaster secret", "\n"+hex.Dump(pms[:pmsLen]))
		default:
			return
		}

		if hdr.MasterSecretLen == 0 {
			log.Println("zero master secret len")
			return
		}

		_, err = io.ReadFull(r, masterSecret[:hdr.MasterSecretLen])
		if err != nil {
			log.Println("read master secret err:", err)
			return
		}

		log.Println("master secret",
			"\n"+hex.Dump(masterSecret[:hdr.MasterSecretLen]))
	}
}
