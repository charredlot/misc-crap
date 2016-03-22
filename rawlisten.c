
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <arpa/inet.h>
#include <linux/if_ether.h>
#include <sys/types.h>
#include <sys/socket.h>

int main(int argc, char **argv)
{
    int fd = -1;

    if (argc < 2) {
        printf("usage: %s <device_name>\ne.g. eth0\n", argv[0]);
        return 0;
    }

    fd = socket(PF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
    if (fd < 0) {
        perror("socket err");
        goto done;
    }

    if (setsockopt(fd, SOL_SOCKET, SO_BINDTODEVICE,
                   argv[1], strlen(argv[1])) != 0) {
        perror("setsockopt err");
        goto done;
    }

    while (true) {
        uint8_t buf[2048];
        ssize_t n;

        n = recvfrom(fd, buf, sizeof(buf), 0, NULL, NULL);
        if (n <= 0) {
            perror("recv error");
            break;
        }

        printf("rcvd %zu bytes\n", n);
    }


done:
    if (fd >= 0) {
        close(fd);
    }
    return 0;
}

