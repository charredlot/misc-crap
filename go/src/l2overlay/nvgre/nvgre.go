package nvgre

import (
    "bytes"
    "encoding/binary"
    "log"
    "net"

    "github.com/google/gopacket/layers"

    "l2overlay/msg"
    "l2overlay/tunnel/options"
)

const (
    FlagKeyPresent uint16 = (1 << 13)
)

type NVGREHdr struct {
    Flags uint16
    Proto uint16
    VSID  uint32 // most sig 24 bits
}

type NVGRETunnel struct {
    IP string
}

func (t *NVGRETunnel) ListenLoop() {
    addr, _ := net.ResolveIPAddr("ip", t.IP)
    conn, err := net.ListenIP("ip:gre", addr)
    if err != nil {
        log.Println("nvgre listen", err)
        return
    }
    defer conn.Close()

    for {
        buf := make([]byte, 2048)
        n, remote, err := conn.ReadFrom(buf)
        if err != nil {
            log.Println("nvgre rcv err", err)
            // break? jumbo?
            continue
        }

        if options.Debug {
            log.Println("nvgre rcvd", n, "bytes from", remote)
        }
    }
}

func (t *NVGRETunnel) ForwardLoop(in chan msg.PktDesc) {
    conn, err := net.Dial("ip:gre", t.IP)
    if err != nil {
        log.Println("dial err", err)
        return
    }
    defer conn.Close()

    limit := 4096
    b := bytes.NewBuffer(make([]byte, 0, limit))
    for desc := range(in) {
        b.Reset()

        var hdr NVGREHdr
        if binary.Size(&hdr) + len(desc.Pkt) > limit {
            log.Println("nvgre pkt too big", len(desc.Pkt))
            continue
        }

        hdr.Flags = FlagKeyPresent
        hdr.Proto = uint16(layers.EthernetTypeTransparentEthernetBridging)
        hdr.VSID = desc.L2ID << 8 // VSID is top 24 bits
        _ = binary.Write(b, binary.BigEndian, &hdr)
        _, _ = b.Write(desc.Pkt)

        _, err = conn.Write(b.Bytes())
        if err != nil {
            if options.Debug {
                log.Println("nvgre write err:", err)
            }
            break
        }
    }
}
