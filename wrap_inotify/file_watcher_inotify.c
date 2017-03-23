
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <errno.h>
#include <limits.h>
#include <sys/inotify.h>

#include "file_watcher.h"

struct file_watcher {
    /* will get at least one event */
    uint8_t buf[sizeof(struct inotify_event) + NAME_MAX + 1];
    uint32_t off;
    uint32_t len;

    char *pathname;
    int fd;
    int watch_fd;
    bool debug;
};

struct flag_to_char {
    uint32_t flag;
    char c;
};

#define FATAL_MASK (IN_CLOSE_WRITE | IN_IGNORED | IN_DELETE_SELF | \
                    IN_MOVE_SELF | IN_UNMOUNT)
static const struct flag_to_char flags_to_char[] = {
    {IN_DELETE_SELF, 'D'},
    {IN_IGNORED, 'I'},
    {IN_MODIFY, 'M'},
    {IN_MOVE_SELF, 'V'},
    {IN_UNMOUNT, 'U'},
};
#define EVENT_MASK_LEN (sizeof(flags_to_char) / sizeof(flags_to_char[0]))
#define EVENT_MASK_STR_LEN (EVENT_MASK_LEN + 1)

static void
event_mask_to_str(uint32_t mask, char buf[EVENT_MASK_STR_LEN])
{
    int i = 0;

    for (i = 0; i < EVENT_MASK_LEN; i++) {
        if ((mask & flags_to_char[i].flag) == 0) {
            buf[i] = '-';
        }
        else {
            buf[i] = flags_to_char[i].c;
        }
    }
    buf[i] = '\0';
}

struct file_watcher *
file_watcher_alloc(const char *pathname, bool debug)
{
    struct file_watcher *fw;

    fw = calloc(1, sizeof(struct file_watcher));
    if (fw == NULL) {
        goto err;
    }

    fw->debug = debug;
    fw->fd = -1;
    fw->watch_fd = -1;

    fw->pathname = strdup(pathname);
    if (fw->pathname == NULL) {
        fprintf(stderr, "%s out of memory\n", __func__);
        goto err;
    }

    fw->fd = inotify_init();
    if (fw->fd < 0) {
        fprintf(stderr, "inotify_init failed: %s\n", strerror(errno));
        goto err;
    }

    fw->watch_fd = inotify_add_watch(fw->fd, fw->pathname,
                                     IN_DELETE_SELF | IN_MODIFY | IN_UNMOUNT);
    if (fw->watch_fd < 0) {
        fprintf(stderr, "warning inotify_add_watch %s failed: %s\n",
                pathname, strerror(errno));
        // not fatal
    }

    return fw;

 err:
    file_watcher_free(fw);
    return NULL;
}

void
file_watcher_free(struct file_watcher *fw)
{
    if (fw != NULL) {
        if (fw->fd >= 0) {
            close(fw->fd);
        }
        if (fw->watch_fd >= 0) {
            close(fw->watch_fd);
        }
        if (fw->pathname != NULL) {
            free(fw->pathname);
        }
        free(fw);
    }
}

/* pointer returned is only valid until next call and while fw is valid */
static struct inotify_event *
file_watcher_read_event(struct file_watcher *fw)
{
    struct inotify_event *event;
    uint32_t event_len;

    /* always start by moving leftover data to front of buf */
    if (fw->off > 0) {
        memmove(fw->buf, fw->buf + fw->off, fw->len);
        fw->off = 0;
    }

    while (true) {
        ssize_t n;

        if (fw->len >= sizeof(struct inotify_event)) {
            event = (struct inotify_event *)fw->buf;

            /* TODO: should we care about overflow of event_len? */
            event_len = sizeof(struct inotify_event) + event->len;
            if (event_len <= fw->len) {
                fw->off += event_len;
                fw->len -= event_len;
                return event;
            }
        }

        if (fw->len >= sizeof(fw->buf)) {
            /* should be unreachable... */
            return NULL;
        }

        n = read(fw->fd, fw->buf + fw->len, sizeof(fw->buf) - fw->len);
        if (n < 0) {
            fprintf(stderr, "%s read failure: %s\n", __func__, strerror(errno));
            return NULL;
        }
        else if (n == 0) {
            fprintf(stderr, "%s read eof\n", __func__);
            /* TODO: make sure we can't have leftover events in this case */
            return NULL;
        }

        fw->len += n;
    }

    return NULL;
}

int
file_watcher_wait_write(struct file_watcher *fw)
{
    if (fw->watch_fd < 0) {
        fw->watch_fd = inotify_add_watch(fw->fd, fw->pathname,
                                         IN_DELETE_SELF | IN_MODIFY);
        if (fw->watch_fd < 0) {
            fprintf(stderr, "inotify_add_watch %s failed: %s\n",
                    fw->pathname, strerror(errno));
            return -1;
        }
    }

    while (true) {
        struct inotify_event *event;

        event = file_watcher_read_event(fw);
        if (event == NULL) {
            return -1;
        }

        if (fw->debug) {
            char event_mask_str[EVENT_MASK_STR_LEN];

            event_mask_to_str(event->mask, event_mask_str);
            printf("file_watcher: %s wd %d mask 0x%08x %s cookie %u %.*s\n",
                   fw->pathname, event->wd, event->mask,
                   event_mask_str, event->cookie,
                   event->len, event->name);
        }

        /*
         * TODO: should double-check behavior of IN_CLOSE_WRITE
         * probably need to reopen file for e.g. logrotate case
         * also, not sure what to do for IN_Q_OVERFLOW
         */
        if ((event->mask & FATAL_MASK) != 0) {
            close(fw->watch_fd);
            fw->watch_fd = -1;
            return -1;
        }

        if ((event->mask & IN_MODIFY) != 0) {
            return 0;
        }
    }
}
