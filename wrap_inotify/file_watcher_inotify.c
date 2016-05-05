
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
    char *pathname;
    int fd;
    int watch_fd;
    bool debug;
};

static char *
event_mask_str(uint32_t mask)
{
    static struct {
        uint32_t flag;
        char c;
    } flag_to_char[] = {
        {IN_DELETE_SELF, 'D'},
        {IN_IGNORED, 'I'},
        {IN_MODIFY, 'M'},
        {IN_UNMOUNT, 'U'},
    };
    static char buf[sizeof(flag_to_char) / sizeof(flag_to_char[0]) + 1];
    int i = 0;

    for (i = 0; i < sizeof(flag_to_char) / sizeof(flag_to_char[0]); i++) {
        if ((mask & flag_to_char[i].flag) == 0) {
            buf[i] = '-';
        }
        else {
            buf[i] = flag_to_char[i].c;
        }
    }
    buf[i] = '\0';
    return buf;
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

int
file_watcher_wait_write(struct file_watcher *fw)
{
    ssize_t n;
    int i;
    char buf[sizeof(struct inotify_event) + NAME_MAX + 1];

    if (fw->watch_fd < 0) {
        fw->watch_fd = inotify_add_watch(fw->fd, fw->pathname,
                                         IN_DELETE_SELF | IN_MODIFY);
        if (fw->watch_fd < 0) {
            fprintf(stderr, "inotify_add_watch %s failed: %s\n",
                    fw->pathname, strerror(errno));
            return -1;
        }
    }

    n = read(fw->fd, buf, sizeof(buf));
    if (n < 0) {
        fprintf(stderr, "%s read failure: %s\n", __func__, strerror(errno));
        return -1;
    }

    if (n == 0) {
        fprintf(stderr, "%s read eof\n", __func__);
        return -1;
    }

    for (i = 0; i <= n - sizeof(struct inotify_event);) {
        struct inotify_event *event = (struct inotify_event*)buf;

        if (fw->debug) {
            printf("wd %d mask 0x%08x %s cookie %u %.*s\n",
                   event->wd, event->mask,
                   event_mask_str(event->mask), event->cookie,
                   event->len, event->name);
        }

        if ((event->mask & (IN_IGNORED | IN_DELETE_SELF | IN_UNMOUNT)) != 0) {
            close(fw->watch_fd);
            fw->watch_fd = -1;
            return -1;
        }

        i += sizeof(struct inotify_event) + event->len;
    }

    return 0;
}
