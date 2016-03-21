package msg

// match tap.h
type PktHdr struct {
    L2ID uint32
    Len  uint16
}

type PktDesc struct {
    L2ID uint32
    Pkt  []byte
}

