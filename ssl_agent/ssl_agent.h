#pragma once

#include <stdint.h>

#define SSL_AGENT_VERSION 1

struct pkt_handshake_hdr {
    uint8_t version;
    uint16_t hdr_len;
} __attribute__((packed));

#define TLS_RANDOM_LEN 32
struct pkt_session_hdr {
    uint8_t client_random[TLS_RANDOM_LEN];
    uint8_t server_random[TLS_RANDOM_LEN];
    uint8_t master_secret_len;
} __attribute__((packed));
