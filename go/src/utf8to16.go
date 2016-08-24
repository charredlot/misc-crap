package main

import (
    "bytes"
    "fmt"
    "os"
    "unicode/utf16"
)

func main() {
    s := os.Args[1]

    fmt.Printf("Converting %s to UTF16\n", s)

    var b bytes.Buffer
    pairs := utf16.Encode([]rune(s))
    for _, short := range pairs {
        b.WriteString(fmt.Sprintf("\\u%04X", short))
    }

    fmt.Println(b.String())
}
