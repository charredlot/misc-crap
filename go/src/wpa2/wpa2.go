package wpa2

import (
	"crypto/hmac"
	"crypto/sha1"

	"golang.org/x/crypto/pbkdf2"
)

func WPAPassphraseToPSK(ssid, passphrase string) []byte {
	return pbkdf2.Key([]byte(passphrase), []byte(ssid), 4096, 32, sha1.New)
}

func wpaPRF(key, label, input []byte, bits int) []byte {
	var prefix []byte

	prefix = append(prefix, label...)
	prefix = append(prefix, 0)
	prefix = append(prefix, input...)
	wpa_hmac := hmac.New(sha1.New, key)

	var r []byte
	b := append(prefix, 0)
	for i := 0; i < (bits+159)/160; i++ {
		b = b[:len(prefix)]
		b = append(b, uint8(i))

		wpa_hmac.Reset()
		wpa_hmac.Write(b)
		r = append(r, wpa_hmac.Sum(nil)...)
	}
	return r[:bits/8]
}

func WPADeriveCCMPPTK(pmk, aa, spa, anonce, snonce []byte) []byte {
	input := minThenMax(aa, spa)
	input = append(input, minThenMax(anonce, snonce)...)

	// CCMP uses PRF-384
	return wpaPRF(pmk, []byte("Pairwise key expansion"), input, 384)
}

func sliceLessThan(l, r []byte) bool {
	if len(l) < len(r) {
		return true
	} else if len(l) > len(r) {
		return false
	} else {
		for i := range l {
			if l[i] < r[i] {
				return true
			} else if l[i] > r[i] {
				return false
			}
		}
	}
	return false
}

func minThenMax(l, r []byte) []byte {
	var b []byte
	if sliceLessThan(l, r) {
		b = append(b, l...)
		b = append(b, r...)
	} else {
		b = append(b, r...)
		b = append(b, l...)
	}

	return b
}
