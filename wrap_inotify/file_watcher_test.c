
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "file_watcher.h"

static void
test_wait(const char *pathname)
{
    struct file_watcher *fw;

    fw = file_watcher_alloc(pathname, true);
    if (fw == NULL) {
        return;
    }

    while (file_watcher_wait_write(fw) == 0) {
        asm volatile("":::"memory");
    }

    file_watcher_free(fw);
}

int
main(int argc, char **argv)
{
    if (argc <= 1) {
        printf("usage: %s $filepath\n", argv[0]);
        return 0;
    }

    test_wait(argv[1]);
    return 0;
}

