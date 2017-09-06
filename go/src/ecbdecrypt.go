package main

import (
	"crypto/aes"
	"crypto/cipher"
	"encoding/base64"
	"fmt"
	"io/ioutil"
	"os"
)

type ecb struct {
	b         cipher.Block
	blockSize int
}

func newECB(b cipher.Block) *ecb {
	return &ecb{
		b:         b,
		blockSize: b.BlockSize(),
	}
}

type ecbEncrypter ecb

func NewECBEncrypter(b cipher.Block) cipher.BlockMode {
	return (*ecbEncrypter)(newECB(b))
}

func (x *ecbEncrypter) BlockSize() int { return x.blockSize }

func (x *ecbEncrypter) CryptBlocks(dst, src []byte) {
	if len(src)%x.blockSize != 0 {
		panic("crypto/cipher: input not full blocks")
	}
	if len(dst) < len(src) {
		panic("crypto/cipher: output smaller than input")
	}
	for len(src) > 0 {
		x.b.Encrypt(dst, src[:x.blockSize])
		src = src[x.blockSize:]
		dst = dst[x.blockSize:]
	}
}

type ecbDecrypter ecb

func NewECBDecrypter(b cipher.Block) cipher.BlockMode {
	return (*ecbDecrypter)(newECB(b))
}

func (x *ecbDecrypter) BlockSize() int { return x.blockSize }

func (x *ecbDecrypter) CryptBlocks(dst, src []byte) {
	if len(src)%x.blockSize != 0 {
		panic("crypto/cipher: input not full blocks")
	}
	if len(dst) < len(src) {
		panic("crypto/cipher: output smaller than input")
	}
	for len(src) > 0 {
		x.b.Decrypt(dst, src[:x.blockSize])
		src = src[x.blockSize:]
		dst = dst[x.blockSize:]
	}
}

func base64FileToSlice(filename string) ([]byte, error) {
	f, err := os.Open(os.Args[1])
	if err != nil {
		return nil, err
	}
	defer f.Close()

	b, err := ioutil.ReadAll(f)
	if err != nil {
		return nil, err
	}

	out := make([]byte, base64.StdEncoding.DecodedLen(len(b)))
	n, err := base64.StdEncoding.Decode(out, b)
	if err != nil {
		return nil, err
	}

	return out[:n], nil
}

func main() {
	// https://gist.github.com/DeanThompson/17056cc40b4899e3e7f4
	block, err := aes.NewCipher([]byte("YELLOW SUBMARINE"))
	if err != nil {
		fmt.Println(err)
		return
	}

	plaintext := "boopboopboopboop"
	buf := []byte(plaintext)
	encrypter := NewECBEncrypter(block)
	encrypter.CryptBlocks(buf, buf)
	fmt.Printf("encrypted %s: % x\n", plaintext, buf)

	out, err := base64FileToSlice(os.Args[1])
	if err != nil {
		fmt.Println(err)
		return
	}

	decrypter := NewECBDecrypter(block)
	decrypter.CryptBlocks(out, out)
	fmt.Println(len(out))
	fmt.Println(out[len(out)-4:])
	fmt.Println(string(out))
}
