package http2util

import (
    "golang.org/x/net/http2"
)

func SendSettings(framer *http2.Framer) error {
    settings := []http2.Setting{
        http2.Setting{
            ID: http2.SettingEnablePush,
            Val: 1,
        },
        http2.Setting{
            ID: http2.SettingMaxFrameSize,
            Val: 16 * 1024,
        },
        http2.Setting{
            ID: http2.SettingMaxConcurrentStreams,
            Val: 512,
        },
        http2.Setting{
            ID: http2.SettingMaxHeaderListSize,
            Val: 0xffffffff,
        },
    }

    if err := framer.WriteSettings(settings...); err != nil {
        return err
    }

    return nil
}
