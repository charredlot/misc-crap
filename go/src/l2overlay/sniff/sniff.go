package sniff

import (
    "errors"
    "log"

    "github.com/google/gopacket"
    "github.com/google/gopacket/layers"
    "github.com/google/gopacket/pcap"

    "l2overlay/msg"
    "l2overlay/tunnel/options"
)

func openInterface(name string) (*pcap.Handle, *gopacket.PacketSource) {
    handle, err := pcap.OpenLive(name, 1600, true, 0)
    if err != nil {
        log.Println("pcap open err", err)
        return nil, nil
    }

    return handle, gopacket.NewPacketSource(handle, handle.LinkType())
}

func max(l, r int) int {
    if l > r {
        return l
    } else {
        return r
    }
}

func getID(packet gopacket.Packet) uint32 {
    ethLayer := packet.Layer(layers.LayerTypeEthernet)
    if ethLayer == nil {
        return uint32(options.MaxL2Domains)
    }
    eth := ethLayer.(*layers.Ethernet)

    maxLen := max(len(eth.SrcMAC), len(eth.DstMAC))

    var hash uint32 = 0
    for i := 0; i < maxLen; i++ {
        var l, r byte

        if i < len(eth.SrcMAC) {
            l = eth.SrcMAC[i]
        }
        if i < len(eth.DstMAC) {
            r = eth.DstMAC[i]
        }

        // SDBM hash
        // also we want the input byte to be symmetric
        // i.e. if src and dst mac are swapped
        hash = uint32(l ^ r) + (hash << 6) + (hash << 16) - hash
    }

    // +1 since we don't like zeroes
    return (hash % uint32(options.MaxL2Domains)) + 1
}

func getL3Proto(packet gopacket.Packet) (proto layers.IPProtocol, err error) {
    net := packet.NetworkLayer()
    if net == nil {
        err = errors.New("no l3 layer")
        return
    }

    ipv4, ok := net.(*layers.IPv4)
    if ok {
        proto = ipv4.Protocol
        return
    }

    l := packet.Layer(layers.LayerTypeIPv6Fragment)
    if l != nil {
        frag := l.(*layers.IPv6Fragment)
        proto = frag.NextHeader
        return
    }

    // not ipv6 fragmented
    ipv6, ok := net.(*layers.IPv6)
    if ok {
        proto = ipv6.NextHeader
        return
    }

    err = errors.New("no known l3 layer")
    return
}

func isGRE(packet gopacket.Packet) bool {
    // XXX: argh no gre layer for frag case, so just check proto
    if proto, err := getL3Proto(packet); err == nil {
        return proto == layers.IPProtocolGRE
    } else {
        return false
    }
}

func isVXLAN(packet gopacket.Packet) bool {
    if udpLayer := packet.Layer(layers.LayerTypeUDP); udpLayer != nil {
        udp := udpLayer.(*layers.UDP)
        return udp.DstPort == 4789
    }

    // XXX: argh, don't accidentally encap frag vxlan, check udp
    if fragLayer := packet.Layer(gopacket.LayerTypeFragment); fragLayer == nil {
        return false
    }

    if proto, err := getL3Proto(packet); err == nil {
        return proto == layers.IPProtocolUDP
    } else {
        return false
    }
}

func excludeFilter(packet gopacket.Packet) bool {
    if options.DoNVGRE && isGRE(packet) {
        return true
    } else if options.DoVXLAN && isVXLAN(packet) {
        return true
    } else {
        return false
    }
}

func SniffInterface(intf string, outChans []chan msg.PktDesc) {
    handle, src := openInterface(intf)
    if handle == nil {
        return
    }
    defer handle.Close()

    for packet := range src.Packets() {
        if excludeFilter(packet) {
            continue
        }

        desc := msg.PktDesc{L2ID: getID(packet), Pkt: packet.Data()}
        for i := range outChans {
            outChans[i] <-desc
        }
    }
}
