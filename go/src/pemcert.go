package main

import (
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
)

func main() {
	var filename string
	flag.StringVar(&filename, "cert", "", "Cert in PEM format")
	flag.Parse()

	f, err := os.Open(filename)
	if err != nil {
		fmt.Println("error opening", filename, err)
		return
	}

	b, err := ioutil.ReadAll(f)
	if err != nil {
		fmt.Println("error reading", filename, err)
		return
	}

	rest := b
	for {
		var p *pem.Block

        fmt.Printf("%p\n", rest)
		p, rest = pem.Decode(rest)
        fmt.Printf("%p\n", rest)
		if p == nil {
			break
		}

		fmt.Println("PEM Type:", p.Type)
		for header, val := range p.Headers {
			fmt.Println(" Header", header+":", val)
		}

		cert, err := x509.ParseCertificate(p.Bytes)
		if err != nil {
			fmt.Println("error parsing certificate", err)
			continue
		}

        pretty, err := json.MarshalIndent(cert.PublicKey, "", "  ")
        if err != nil {
			fmt.Println("error json unmarshaling PublicKey", err)
			continue
        }

		fmt.Printf("Public Key: %T %+v\n", cert.PublicKey, string(pretty))
	}
}
