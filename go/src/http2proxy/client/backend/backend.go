package backend

import (
	"bytes"
	"errors"
	"fmt"
	"log"
	"net"
	"net/http"
	"sync"
	//    "sync/atomic"
	"time"

	"golang.org/x/net/http2"
	"golang.org/x/net/http2/hpack"

	"http2proxy/client/options"
	"http2proxy/http2util"
)

const (
    // spec says client-initiated streams need to have an odd number
    minStreamID = 1001
    // 31-bit
    maxStreamID = 0x7fffffff
)

type serviceStream struct {
	client net.Conn
    ID uint32
    service string
    handshakeDone bool
}

type Backend struct {
	connect string
	done    chan bool

	connLock sync.RWMutex
	conn     net.Conn
	framer   *http2.Framer
	redial   bool
	wg       sync.WaitGroup

	streamLock sync.RWMutex
	streamID    uint32
	newStreams  chan serviceStream
    streams map[uint32]serviceStream

    decoder *hpack.Decoder
    decoderLock sync.RWMutex
}

func (stream *serviceStream) String() string {
    return fmt.Sprintf("Stream %s for %s", stream.ID, stream.service)
}

func requestClientStreamBlockFragment(service string) []byte {
	var buf bytes.Buffer
	var enc *hpack.Encoder

	enc = hpack.NewEncoder(&buf)
	enc.WriteField(hpack.HeaderField{Name: ":authority", Value: service})
	enc.WriteField(hpack.HeaderField{Name: ":method",
		Value: http.MethodConnect})
	enc.WriteField(hpack.HeaderField{Name: "user-agent",
		Value: "beepboop"})
	return buf.Bytes()
}

func (backend *Backend) requestClientStream(stream serviceStream) {
    // TODO: we should probably enforce http2's max streams here too

    backend.streamLock.Lock()
    // spec says client-initiated streams need to have an odd number
    backend.streamID += 2
    if backend.streamID > maxStreamID {
        // technically this is not okay, streamIDs need to always increase
        backend.streamID = minStreamID
    }
    stream.ID = backend.streamID
    backend.streams[stream.ID] = stream
    backend.streamLock.Unlock()

    log.Println("assigned", stream)
    // TODO: we should enforce max frame size here
    bf := requestClientStreamBlockFragment(stream.service)
    err := backend.framer.WriteHeaders(http2.HeadersFrameParam{
            StreamID: stream.ID,
            BlockFragment: bf,
            EndStream: false,
            EndHeaders: true,
        })
    if err != nil {
        // should we try to queue on failure?
        log.Println("could not request new stream for", stream, err)
        return
    }
}

func (backend *Backend) writeLoop() error {
    // NB: http2.Framer can't do concurrent writes
    // so all writes have to be done in one goroutine
    for {
        select {
        case stream := <-backend.newStreams:
            backend.requestClientStream(stream)
        case <-backend.done:
            return nil
        }
    }

    return nil
}

func (backend *Backend) handleSettings(hdr http2.FrameHeader,
	frame http2.Frame) error {
	settings, ok := frame.(*http2.SettingsFrame)
	if !ok {
		return errors.New(fmt.Sprintf("bad settings frame %T %+v",
			frame, frame))
	}

	if settings.IsAck() {
		log.Println("SETTINGS ACK received on stream", hdr.StreamID)
	} else {
		log.Println("SETTINGS received on stream", hdr.StreamID)
		settings.ForeachSetting(func(s http2.Setting) error {
			log.Printf("%+v\n", s)
			return nil
		})
		if err := backend.framer.WriteSettingsAck(); err != nil {
			return err
		}
	}

	return nil
}


func (backend *Backend) handleHeaders(hdr http2.FrameHeader,
	frame http2.Frame) error {
    headers, ok := frame.(*http2.HeadersFrame)
    if !ok {
		return errors.New(fmt.Sprintf("bad headers frame %T %+v",
			frame, frame))
    }

    stream, ok := backend.streams[hdr.StreamID]
    if !ok {
        return errors.New(fmt.Sprintf("unknown stream ID %d", hdr.StreamID))
    }
    log.Println(stream)


    /* XXX: handle continuation */

    bf := headers.HeaderBlockFragment()

    backend.decoderLock.Lock()
    if backend.decoder == nil {
        backend.decoder = hpack.NewDecoder(4096, nil)
    }
    fields, err := backend.decoder.DecodeFull(bf)
    backend.decoderLock.Unlock()

    if err != nil {
        return err
    }

    if options.Debug {
        log.Println("Got Headers:")
        for i := range fields {
            log.Println(" ", fields[i])
        }
    }

    return nil
}

func (backend *Backend) readLoop() error {
	for {
		frame, err := backend.framer.ReadFrame()
		if err != nil {
			return err
		}

		if options.Debug {
			log.Printf("received frame %+v\n", frame)
		}

		hdr := frame.Header()
		switch hdr.Type {
		case http2.FrameSettings:
			err = backend.handleSettings(hdr, frame)
		case http2.FrameHeaders:
            err = backend.handleHeaders(hdr, frame)
		case http2.FrameData:
		case http2.FrameContinuation:
		case http2.FrameGoAway:
		case http2.FramePushPromise:
		case http2.FramePing:
		case http2.FrameWindowUpdate:
		default:
			log.Println("ignoring frame")
		}

        if err != nil {
            log.Println("bad frame", err)
        }
	}

	// unreachable
	return nil
}

func (backend *Backend) closeConn() {
	backend.connLock.Lock()
	if !backend.redial {
        if backend.conn != nil {
            backend.conn.Close()
        }
		backend.redial = true
	}
	backend.connLock.Unlock()
}

func (backend *Backend) reset() {
    backend.decoder = nil
    backend.streamID = minStreamID
    backend.streams = make(map[uint32]serviceStream)
}

func (backend *Backend) dial() error {
	if (backend.conn != nil) && !backend.redial {
		return nil
	}

	backend.wg.Wait()

	conn, err := net.Dial("tcp", backend.connect)
	if err != nil {
		return err
	}

	framer := http2.NewFramer(conn, conn)
	if framer == nil {
		return errors.New("framer error")
	}

    backend.reset()
	backend.conn = conn
	backend.framer = framer
	backend.redial = false
	log.Println("connected to", backend.connect)

	_, err = backend.conn.Write([]byte(http2.ClientPreface))
	if err != nil {
		return err
	}

	if err := http2util.SendSettings(backend.framer); err != nil {
		return err
	}

	backend.wg.Add(1)
	go func() {
		defer backend.wg.Done()
		if err := backend.writeLoop(); err != nil {
			log.Println("write err", err)
		}
	    backend.closeConn()
	}()

	backend.wg.Add(1)
	go func() {
		defer backend.wg.Done()
		if err := backend.readLoop(); err != nil {
			log.Println("read err", err)
		}
	    backend.closeConn()
	}()
	return nil
}

func (backend *Backend) Proxy(service string, conn net.Conn, done chan bool) {
    select {
    // FIXME: need to be able to close newStreams safely
    case backend.newStreams <-serviceStream{client: conn, service: service}:
    case <-done:
    }
}

func (backend *Backend) Connect() {
	t := time.NewTicker(time.Second * 15)
	defer t.Stop()

	if err := backend.dial(); err != nil {
		log.Println("backend error", err)
	}

	for {
		select {
		case <-t.C:
			if err := backend.dial(); err != nil {
				log.Println("backend dial error", err)
				continue
			}
		case <-backend.done:
			backend.closeConn()
			return
		}
	}
}

func NewBackend(connect string, done chan bool) *Backend {
	// NB: streamID 0 is always used for the init, add some padding in case
	backend := &Backend{
        connect: connect,
        done: done,
        newStreams: make(chan serviceStream),
    }
    backend.reset()
    return backend
}
