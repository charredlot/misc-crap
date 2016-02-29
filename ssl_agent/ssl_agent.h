#pragma once

#include <stdint.h>

#define SSL_AGENT_VERSION 1

struct sa_handshake_hdr {
    uint8_t version;
    uint16_t hdr_len;
} __attribute__((packed));

enum sa_record_type {
    SSL_AGENT_RECORD_CLIENT_RANDOM_TO_SECRET = 1,
    SSL_AGENT_RECORD_SERVER_RANDOM_TO_SECRET = 2,
    SSL_AGENT_RECORD_CLIENT_SERVER_RANDOMS_TO_SECRET = 3,
    SSL_AGENT_RECORD_PREMASTER_SECRET_TO_SECRET = 4,
};

struct sa_record_hdr {
    uint8_t  type;
    uint32_t len;
    uint8_t  master_secret_len;
} __attribute__((packed));
