package vxlan

import (
    "bytes"
    "encoding/binary"
    "fmt"
    "log"
    "net"

    "l2overlay/msg"
    "l2overlay/tunnel/options"
)

const FlagVNI = (1 << 3)

type VXLANHdr struct {
    Flags uint8
    Rsvd  [3]uint8
    VNI   uint32
}

type VXLANTunnel struct {
    IP string
    Port uint16
}

func (t *VXLANTunnel) ListenLoop() {
    addr := net.UDPAddr{IP: net.ParseIP(t.IP), Port: int(t.Port)}
    conn, err := net.ListenUDP("udp", &addr)
    if err != nil {
        log.Println("listen", err)
        return
    }
    defer conn.Close()

    for {
        buf := make([]byte, 2048)
        n, remote, err := conn.ReadFromUDP(buf)
        if err != nil {
            log.Println("rcv err", err)
            // break? jumbo?
            continue
        }

        if options.Debug {
            log.Println("vxlan rcvd", n, "bytes from", remote)
        }
    }
}

func (t *VXLANTunnel) ForwardLoop(in chan msg.PktDesc) {
    conn, err := net.Dial("udp", fmt.Sprintf("%s:%d", t.IP, t.Port))
    if err != nil {
        log.Println("dial err", err)
        return
    }
    defer conn.Close()
    udpConn := conn.(*net.UDPConn)

    limit := 4096
    b := bytes.NewBuffer(make([]byte, 0, limit))
    for desc := range(in) {
        b.Reset()

        var hdr VXLANHdr
        if binary.Size(&hdr) + len(desc.Pkt) > limit {
            log.Println("vxlan pkt too big %d", len(desc.Pkt))
            continue
        }

        hdr.Flags = FlagVNI
        hdr.VNI = desc.L2ID << 8 // VNI is top 24 bits
        _ = binary.Write(b, binary.BigEndian, &hdr)
        _, _ = b.Write(desc.Pkt)

        _, _, err = udpConn.WriteMsgUDP(b.Bytes(), nil, nil)
        if err != nil {
            // just ignore port unreachables and the like
            if options.Debug {
                log.Println("vxlan write err:", err)
            }
            continue
        }
    }
}
