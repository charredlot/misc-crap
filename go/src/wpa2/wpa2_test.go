package wpa2

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"testing"
)

func hexDecode(s string) []byte {
	b, err := hex.DecodeString(s)
	if err != nil {
		fmt.Println("hex decode", s, err)
		panic("hex decode failure")
	}
	return b
}

func testPSK() {
	type pskTest struct {
		ssid       string
		passphrase string
		expected   []byte
	}

	tests := []pskTest{
		pskTest{
			ssid:       "IEEE",
			passphrase: "password",
			expected: hexDecode("f42c6fc52df0ebef9ebb4b90b38a5f90" +
				"2e83fe1b135a70e23aed762e9710a12e"),
		},
		pskTest{
			ssid:       "ThisIsASSID",
			passphrase: "ThisIsAPassword",
			expected: hexDecode("0dc0d6eb90555ed6419756b9a15ec3e3" +
				"209b63df707dd508d14581f8982721af"),
		},
	}
	for _, test := range tests {
		psk := WPAPassphraseToPSK(test.ssid, test.passphrase)
		if !bytes.Equal(psk, test.expected) {
			fmt.Println("Failed PSK test for", test.ssid, test.passphrase)
			fmt.Println("Expected: %v", test.expected)
			fmt.Println("     Got: %v", psk)
		}
	}
	fmt.Println("PASSED: PSK test vectors")
}

func testPRF() {
	type prfTest struct {
		key      []byte
		prefix   []byte
		input    []byte
		bits     int
		expected string
	}
	prfTests := []prfTest{
		prfTest{
			key:    hexDecode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"),
			prefix: []byte("prefix"),
			input:  []byte("Hi There"),
			bits:   192,
			expected: ("bcd4c650b30b9684951829e0d75f9d54" +
				"b862175ed9f00606"),
		},
		prfTest{
			key:    []byte("Jefe"),
			prefix: []byte("prefix-2"),
			input:  []byte("what do ya want for nothing?"),
			bits:   256,
			expected: ("47c4908e30c947521ad20be9053450ec" +
				"bea23d3aa604b77326d8b3825ff7475c"),
		},
	}

	for _, test := range prfTests {
		expected, err := hex.DecodeString(test.expected)
		if err != nil {
			fmt.Println("expected decode", test.expected, err)
			return
		}

		res := wpaPRF(test.key, test.prefix, test.input, test.bits)
		if !bytes.Equal(res, expected) {
			panic(fmt.Sprintf("Input %v\nExpected: %v\nGot: %v\n",
				test.input, expected, res))
		}
	}
	fmt.Println("PASSED: PRF test vectors")
}

func testPTK() {
	type ptkTest struct {
		pmk      []byte
		aa       []byte
		spa      []byte
		anonce   []byte
		snonce   []byte
		expected []byte
	}
	tests := []ptkTest{
		ptkTest{
			pmk: hexDecode("0dc0d6eb90555ed6419756b9a15ec3e3" +
				"209b63df707dd508d14581f8982721af"),
			aa:  []byte{0xa0, 0xa1, 0xa1, 0xa3, 0xa4, 0xa5},
			spa: []byte{0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5},
			anonce: hexDecode("e0e1e2e3e4e5e6e7e8e9f0f1f2f3f4f5" +
				"f6f7f8f9"),
			snonce: hexDecode("c0c1c2c3c4c5c6c7c8c9d0d1d2d3d4d5" +
				"d6d7d8d9"),
			expected: hexDecode("aa7cfc8560251e4bc687e0cb8d298363" +
				"ba53163df32a8638f479abe34bfd2bc8" +
				"8cb778332e94aca6d30b89cbe82a9ca9"),
		},
	}

	for _, test := range tests {
		ptk := WPADeriveCCMPPTK(test.pmk, test.aa, test.spa, test.anonce, test.snonce)
		if !bytes.Equal(ptk, test.expected) {
			fmt.Printf("FAILED: PTK test for PMK %v", test.pmk)
			fmt.Println("Expected:\n%s", hex.Dump(test.expected))
			fmt.Println("Got:\n%s", hex.Dump(ptk))
			panic("ptk test failed")
		}
	}
	fmt.Println("PASSED: CCMP PTK test vectors")
}

func TestVectors(t *testing.T) {
	testPSK()
	testPRF()
	testPTK()
}
