#include <stdio.h>

int always_true = 1;

void
recurse(void)
{
    if (always_true) {
        recurse();
    }
}

static void
hello_world(void)
{
    printf("hello, world!\n");
}

int
main(int argc, char **argv)
{
    hello_world();
}
