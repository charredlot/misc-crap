#define _GNU_SOURCE

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>

#include <arpa/inet.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/syslog.h>
#include <sys/un.h>

#include <pthread.h>
#include <dlfcn.h>

#include <openssl/ssl.h>
#include <openssl/ssl3.h>

#include "ssl_agent.h"

#define SSL_AGENT_UNIX_SOCK_PATH "/tmp/ssl_agent"

const char *SSLeay_version(int type)
{
    return "ssl version overridden";
}

enum conn_state {
    CONN_UNINIT = 0,
    CONN_SEND_HANDSHAKE,
    CONN_RECV_HANDSHAKE,
    CONN_SEND_DATA,
};

struct ssl_agent_ctx {
    pthread_t pthread;
    pid_t tid;
    uint64_t export_no_sock;
    uint64_t zero_len_master_secret;
    uint64_t bad_master_secret_len;
    uint64_t zero_len_session_id;
    uint64_t bad_session_id_len;
    uint64_t would_block;
    enum conn_state state;
    int fd;
    bool f_initialized;
};
static __thread struct ssl_agent_ctx ctx;

static bool log_dirty;
static int log_level = LOG_DEBUG;
static const char *log_filename = "/tmp/ssl_agent.log";
static FILE *log_file = NULL;

static FILE *
open_log_file(void)
{
    if (log_file == NULL) {
        FILE *f;

        f = fopen(log_filename, "w");
        if (f == NULL) {
            return NULL;
        }

        if (!__sync_bool_compare_and_swap(&log_file, NULL, f)) {
            fclose(f);
        }
        else {
            log_file = f;
        }
    }

    return log_file;
}

static void
close_log_file(void)
{
    FILE *f = log_file;

    f = __sync_val_compare_and_swap(&log_file, f, NULL);
    if (f == NULL) {
        return;
    }

    fclose(f);
}

/* TODO: use per-thread stuff  */
#define log_printf(level, f_add_thread, args...) \
    do { \
        FILE *lf; \
        if (log_level < level) { \
            break; \
        } \
        lf = open_log_file(); \
        if (lf != NULL) { \
            if (f_add_thread) { \
                fprintf(lf, "[%" PRIu64 "]", (uint64_t)ctx.tid); \
            } \
            int rc = fprintf(lf, args); \
            if (rc < 0) { \
                close_log_file(); \
            } \
            else { \
                log_dirty = true; \
            } \
        } else { \
            printf(args); \
        } \
    } while (0)

#define log_crit(args...) log_printf(LOG_CRIT, true, args)
#define log_error(args...) log_printf(LOG_ERR, true, args)
#define log_warn(args...) log_printf(LOG_WARNING, true, args)
#define log_info(args...) log_printf(LOG_INFO, true, args)
#define log_debug(args...) log_printf(LOG_DEBUG, true, args)
#define log_partial_debug(args...) log_printf(LOG_DEBUG, false, args)

static inline void
log_flush(void)
{
    if ((log_file != NULL) && log_dirty) {
        log_dirty = false;
        fflush(log_file);
    }
}

static inline void
log_debug_hex(uint8_t *buf, size_t len)
{
    size_t i;

    for (i = 0; i < len; i++) {
        log_partial_debug("%02x ", buf[i]);
        if (i % 16 == 16 - 1) {
            log_partial_debug("\n");
        }
    }

    if (i % 16 != 0) {
        log_partial_debug("\n");
    }
}

static void
print_master_secret(SSL *s)
{
    SSL_SESSION *session;
    struct ssl3_state_st *s3;

    if (s == NULL) {
        return;
    }

    session = s->session;
    s3 = s->s3;
    if ((session == NULL) || (s3 == NULL)) {
        return;
    }

    if (session->tlsext_tick != NULL) {
        log_debug_hex(session->tlsext_tick, session->tlsext_ticklen);
    }

    log_debug("Client Random:\n");
    log_debug_hex(s3->client_random, SSL3_RANDOM_SIZE);
    log_debug("Server Random:\n");
    log_debug_hex(s3->server_random, SSL3_RANDOM_SIZE);
    log_debug("Session ID:\n");
    log_debug_hex(session->session_id, session->session_id_length);
    log_debug("Master Secret:\n");
    log_debug_hex(session->master_key, session->master_key_length);
}

static void
ctx_socket_cleanup(struct ssl_agent_ctx *c)
{
    log_debug("cleanup socket fd %d\n", c->fd);
    if (c->fd >= 0) {
        close(c->fd);
    }
    c->fd = -1;
    c->state = CONN_UNINIT;
}

static int
ctx_handshake_send(struct ssl_agent_ctx *c)
{
    struct sa_handshake_hdr handshake;

    handshake.version = SSL_AGENT_VERSION;
    handshake.hdr_len = htons(sizeof(struct sa_handshake_hdr));

    errno = 0;
    if (send(c->fd, &handshake, sizeof(struct sa_handshake_hdr), 0) !=
            sizeof(struct sa_handshake_hdr)) {
        log_debug("daemon handshake send err: %s\n", strerror(errno));
        if ((errno != EAGAIN) && (errno != EWOULDBLOCK)) {
            ctx_socket_cleanup(c);
        }

        return -1;
    }

    log_debug("daemon handshake send: version %u len %u\n",
              handshake.version, handshake.hdr_len);
    c->state = CONN_RECV_HANDSHAKE;
    return 0;
}

static int
ctx_set_nonblocking(struct ssl_agent_ctx *c)
{
    int rc;
    int flags;

    flags = fcntl(c->fd, F_GETFL, 0);
    errno = 0;
    rc = fcntl(c->fd, F_SETFL, flags | O_NONBLOCK);
    if (rc != 0) {
        log_warn("fcntl failed %s\n", strerror(errno));
    }

    return rc;
}

static int
ctx_handshake_recv(struct ssl_agent_ctx *c)
{
    struct sa_handshake_hdr handshake;
    uint16_t len;
    uint8_t *buf = NULL;
    ssize_t n;
    ssize_t read;

    /* TODO: assuming we don't get a partial header for now */
    errno = 0;
    n = recv(c->fd, &handshake, sizeof(struct sa_handshake_hdr), 0);
    if (n != sizeof(struct sa_handshake_hdr)) {
        log_error("%s: recv %s\n", __func__, strerror(errno));
        goto err;
    }

    len = ntohs(handshake.hdr_len);
    log_debug("%s: version %u len %u\n",
              __func__, handshake.version, len);
    if (len == 0) {
        log_error("%s: zero len\n", __func__);
        goto err;
    }

    buf = malloc(len);
    if (buf == NULL) {
        log_error("%s: oom\n", __func__);
        goto err;
    }

    if (len > sizeof(struct sa_handshake_hdr)) {
        len -= sizeof(struct sa_handshake_hdr);

        for (read = 0; read < len;) {
            n = recv(c->fd, buf, len, 0);
            if (n <= 0) {
                log_error("%s: recv %s\n", __func__, strerror(errno));
                goto err;
            }

            read += n;
        }
    }

    c->state = CONN_SEND_DATA;
    ctx_set_nonblocking(c);
    return 0;

 err:
    if (buf != NULL) {
        free(buf);
    }
    ctx_socket_cleanup(c);
    return -1;
}

static int
ctx_handshake(struct ssl_agent_ctx *c)
{
    int rc = -1;

    if (c->state == CONN_SEND_DATA) {
        return 0;
    }

    if (c->state == CONN_SEND_HANDSHAKE) {
        rc = ctx_handshake_send(c);
        if (rc != 0) {
            return rc;
        }
    }

    if (c->state == CONN_RECV_HANDSHAKE) {
        return ctx_handshake_recv(c);
    }

    asm("int3");
    return -1;
}

static int
ctx_socket_connect(struct ssl_agent_ctx *c)
{
    int fd = -1;
    struct sockaddr_un addr;
    int rc;

    if (c->fd >= 0) {
        return ctx_handshake(c);
    }

    fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd == -1) {
        log_warn("socket failed %s\n", strerror(errno));
        goto fail;
    }

    memset(&addr, 0, sizeof(struct sockaddr_un));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, SSL_AGENT_UNIX_SOCK_PATH, sizeof(addr.sun_path));

    rc = connect(fd, (struct sockaddr *)&addr, sizeof(struct sockaddr_un));
    if (rc != 0) {
        log_warn("connect failed %s\n", strerror(errno));
        goto fail;
    }

    c->fd = fd;
    c->state = CONN_SEND_HANDSHAKE;
    return ctx_handshake(c);

 fail:
    if (fd != -1) {
        close(fd);
    }
    c->fd = -1;
    c->export_no_sock++;
    return -1;
}

static void
export_master_secret(SSL *s)
{
    size_t bytes;
    size_t written;
    struct sa_frame {
        struct sa_record_hdr hdr;
        uint8_t client_random[SSL3_RANDOM_SIZE];
        uint8_t server_random[SSL3_RANDOM_SIZE];
        uint8_t master_secret[SSL_MAX_MASTER_KEY_LENGTH];
    } frame;
    SSL_SESSION *session;
    struct ssl3_state_st *s3;

    if (s == NULL) {
        return;
    }

    session = s->session;
    s3 = s->s3;
    if ((session == NULL) || (s3 == NULL)) {
        return;
    }

    if (log_level >= LOG_DEBUG) {
        print_master_secret(s);
    }

    if (ctx_socket_connect(&ctx) != 0) {
        return;
    }

    if (session->master_key_length > SSL_MAX_MASTER_KEY_LENGTH) {
        ctx.bad_master_secret_len++;
        return;
    }
    else if (session->master_key_length == 0) {
        ctx.zero_len_master_secret++;
        return;
    }

    frame.hdr.type = SSL_AGENT_RECORD_CLIENT_SERVER_RANDOMS_TO_SECRET;
    frame.hdr.master_secret_len = (uint8_t)session->master_key_length;
    frame.hdr.len = htonl(sizeof(struct sa_record_hdr) +
                          frame.hdr.master_secret_len +
                          (2 * SSL3_RANDOM_SIZE));

    memcpy(&frame.client_random, s3->client_random, SSL3_RANDOM_SIZE);
    memcpy(&frame.server_random, s3->server_random, SSL3_RANDOM_SIZE);
    memcpy(&frame.master_secret, session->master_key,
           frame.hdr.master_secret_len);

    bytes = ntohl(frame.hdr.len);
    errno = 0;
    written = write(ctx.fd, &frame, bytes);
    if (written != bytes) {
        /* TODO: decide what to do on partial writes */
        if ((errno == EAGAIN) || (errno == EWOULDBLOCK)) {
            ctx.would_block++;
        }
        else {
            log_warn("write master key failed %" PRIu64 " %s\n",
                     (uint64_t)written, strerror(errno));
            ctx_socket_cleanup(&ctx);
        }
    }
}

static void
ctx_init(void)
{
    if (ctx.f_initialized) {
        return;
    }

    ctx.pthread = pthread_self();
    ctx.tid = syscall(SYS_gettid);
    ctx.fd = -1;
    ctx.f_initialized = true;

    log_info("pthread id %lu\n", (uint64_t)ctx.tid);
}

int
handshake_wrap(SSL *s, const char *func_name)
{
    int ret;
    typeof(SSL_do_handshake) *orig;

    ctx_init();

    log_debug("handshake replace %s\n", func_name);

    orig = dlsym(RTLD_NEXT, func_name);
    if (orig == NULL) {
        log_crit("dlsym next failed\n");
        ret = -1;
        goto done;
    }

    ret = orig(s);
    if (ret == 1) {
        /* 1 is success */
        export_master_secret(s);
    }

 done:
    log_flush();
    return ret;
}

int
ssl3_get_client_key_exchange(SSL *s)
{
    return handshake_wrap(s, __func__);
}

int
SSL_connect(SSL *s)
{
    return handshake_wrap(s, __func__);
}

int
SSL_do_handshake(SSL *s)
{
    return handshake_wrap(s, __func__);
}

int
SSL_accept(SSL *s)
{
    return handshake_wrap(s, __func__);
}
