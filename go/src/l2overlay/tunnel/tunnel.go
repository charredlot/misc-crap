package main

import (
    "log"
    "sync"
    "time"

    "l2overlay/msg"
    "l2overlay/nvgre"
    "l2overlay/sniff"
    "l2overlay/vxlan"
    "l2overlay/tunnel/options"
)

type Tunnel interface {
    ForwardLoop(in chan msg.PktDesc)
    ListenLoop()
}

func startTunnel(t Tunnel, outChans []chan msg.PktDesc,
    wg *sync.WaitGroup) []chan msg.PktDesc {
    if options.DoSniff {
        out := make(chan msg.PktDesc, 32)
        outChans = append(outChans, out)

        wg.Add(1)
        go func() {
            defer wg.Done()
            log.Printf("forwarding %T %+v\n", t, t)
            t.ForwardLoop(out)
        }()
    }

    if options.DoListen {
        wg.Add(1)
        go func() {
            defer wg.Done()
            log.Printf("listening %T %+v\n", t, t)
            t.ListenLoop()
        }()
    }

    return outChans
}

func main() {
    if err := options.Load(); err != nil {
        log.Println(err)
        return
    }

    var wg sync.WaitGroup
    defer wg.Wait()

    var outChans []chan msg.PktDesc
    if options.DoVXLAN {
        tunVXLAN := vxlan.VXLANTunnel{IP: options.VXLANHost,
                                      Port: options.VXLANPort}
        outChans = startTunnel(&tunVXLAN, outChans, &wg)
    }
    if options.DoNVGRE {
        tunNVGRE := nvgre.NVGRETunnel{IP: options.NVGREHost}
        outChans = startTunnel(&tunNVGRE, outChans, &wg)
    }

    defer func(){
        for i := range outChans {
            close(outChans[i])
        }
    }()

    if options.DoSniff {
        sniff.SniffInterface(options.Interface, outChans)
    } else {
        // DoListen
        // just idle
        for {
            time.Sleep(time.Second * 60)
        }
    }
}
