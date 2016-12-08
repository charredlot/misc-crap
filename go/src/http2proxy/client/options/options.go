package options

import (
    "encoding/json"
    "io/ioutil"
    "log"
    "flag"
)

var (
    CertPath string
    KeyPath string
    Connect string
    Listen []string
    CfgPath string
    UseTLS bool
    Debug bool
)

type config struct {
    Server string  `json:"server"`
    Listen[]string `json:"listen"`
    UseTLS bool `json:"tls"`
}

func Load() error {
    var listen string
    flag.StringVar(&CertPath, "cert", "cert.pem", "server certificate")
    flag.StringVar(&KeyPath, "key", "key.pem", "server certificate key")
    flag.StringVar(&Connect, "connect", "localhost:10080",
        "e.g. localhost:10080")
    flag.StringVar(&listen, "listen", "", "e.g. 'localhost:20000' or ':22'")
    flag.StringVar(&CfgPath, "cfg", "", "configuration file")
    flag.BoolVar(&UseTLS, "tls", false, "use tls?")
    flag.BoolVar(&Debug, "debug", false, "print debug")
    flag.Parse()

    if CfgPath != "" {
        b, err := ioutil.ReadFile(CfgPath)
        if err != nil {
            log.Fatalln("bad cfg path", CfgPath, err)
        }

        var cfg config
        if err := json.Unmarshal(b, &cfg); err != nil {
            log.Fatalln("bad cfg", CfgPath, err)
        }
        log.Println(cfg)
    }

    if listen != "" {
        Listen = append(Listen, ":20000")
    }

    log.Println("Connecting to backend", Connect)
    log.Println("Listening on", Listen)

    return nil
}
