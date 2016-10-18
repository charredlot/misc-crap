
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>

struct ring_wrapped {
    void *ptr;
    size_t len;
};

int
ring_wrapped_init(struct ring_wrapped *r, size_t len, const char *shared_path)
{
    void *ptr = NULL;
    void *tmp;
    int fd = -1;

    memset(r, 0, sizeof(struct ring_wrapped));

    if (len % getpagesize() != 0) {
        fprintf(stderr, "bad len %ju, must be a multiple of %u\n",
                len, getpagesize());
        goto err;
    }

    /* make virtual space so we don't stomp on existing mappings */
    ptr = mmap(NULL, len * 2, PROT_READ | PROT_WRITE,
               MAP_ANONYMOUS | MAP_PRIVATE, -1, 0);
    if (ptr == NULL) {
        perror("mmap anon");
        goto err;
    }

    fd = shm_open(shared_path, O_RDWR | O_CREAT | O_TRUNC, 0644);
    if (fd < 0) {
        perror("shm_open");
        goto err;
    }

    if (ftruncate(fd, len) != 0) {
        perror("ftruncate");
        goto err;
    }

    /* map first half */
    tmp = mmap(ptr, len, PROT_READ | PROT_WRITE,
               MAP_FIXED | MAP_SHARED, fd, 0);
    if (tmp != ptr) {
        perror("mmap first half");
        munmap(tmp, len);
        goto err;
    }

    /* map second half */
    tmp = mmap(ptr + len, len, PROT_READ | PROT_WRITE,
               MAP_FIXED | MAP_SHARED, fd, 0);
    if (tmp != ptr + len) {
        perror("mmap second half");
        munmap(tmp, len);
        goto err;
    }

    close(fd);
    r->ptr = ptr;
    r->len = len;
    return 0;

 err:
    if (ptr != NULL) {
        munmap(ptr, len);
    }
    if (fd >= 0) {
        close(fd);
    }
    return -1;
}

void
ring_wrapped_uninit(struct ring_wrapped *r)
{
    if (r->ptr == NULL) {
        return;
    }

    munmap(r->ptr + r->len, r->len);
    munmap(r->ptr, r->len);
    r->ptr = NULL;
    r->len = 0;
}

int
ring_wrapped_test(struct ring_wrapped *r)
{
    uint8_t *buf;
    int i;

    buf = r->ptr;
    for (i = 0; i < r->len; i++) {
        buf[i] = i & 0xff;
        if (buf[i] != buf[r->len + i]) {
            printf("expected %u got %u at %u", buf[i], buf[r->len + i], i);
            return -1;
        }
    }

    return 0;
}

int
main(int argc, char **argv)
{
    struct ring_wrapped r = {0};
    const char *path = "/boop";
    size_t len = getpagesize() * 2;
    int rc = -1;

    if (ring_wrapped_init(&r, len, path) != 0) {
        printf("qq\n");
        goto done;
    }

    if (ring_wrapped_test(&r) != 0) {
        goto done;
    }

    rc = 0;

 done:
    ring_wrapped_uninit(&r);
    return rc;
}
