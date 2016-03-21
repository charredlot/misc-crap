package options

import (
    "errors"
    "flag"
    "fmt"
    "strconv"
    "strings"
)

var (
    DoSniff bool
    Interface string

    DoListen bool

    DoVXLAN bool
    VXLANHost string
    VXLANPort uint16

    DoNVGRE bool
    NVGREHost string

    MaxL2Domains uint
    Debug bool
)

func Load() error {
    var vxlan string

    flag.StringVar(&Interface, "i", "", "network device to sniff, e.g. -i eth0")
    flag.BoolVar(&DoListen, "l", false, "listen as endpoint of tunnel")

    flag.StringVar(&vxlan, "vxlan", "",
        "set VXLAN host and port (e.g. -vxlan 127.0.0.1:4789)")
    flag.StringVar(&NVGREHost, "nvgre", "",
        "set NVGRE host (e.g. -nvgre 127.0.0.1)")

    flag.UintVar(&MaxL2Domains, "n", 31,
         "number of unique L2 domains to generate\n" +
         "e.g. for VXLAN, number of unique VNIs\n" +
         "note the hash kinda sucks so maybe use a prime number")
    flag.BoolVar(&Debug, "d", false, "debug mode (verbose)")
    flag.Parse()

    DoSniff = (Interface != "")
    DoNVGRE = (NVGREHost != "")

    if vxlan != "" {
        i := strings.LastIndex(vxlan, ":")
        if (i < 0) || (i == len(vxlan) - 1) {
            return errors.New(
                fmt.Sprintf("%s should be of the format host:port", vxlan))
        }

        u, err := strconv.ParseUint(vxlan[i+1:], 10, 16)
        if err != nil {
            return err
        }

        VXLANHost = vxlan[:i]
        VXLANPort = uint16(u)
        DoVXLAN = true
    }

    if !DoSniff && !DoListen {
        return errors.New("must set sniff interface (-i) or listen (-l)")
    }

    if !DoVXLAN && !DoNVGRE {
        return errors.New("vxlan or nvgre must be set")
    }

    return nil
}
