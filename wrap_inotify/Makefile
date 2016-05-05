#CC=clang-3.5
CC=gcc
CFLAGS=-g -Wall -Werror -O3

TARGETLIB=libfilewatcher.so

TARGETTEST=file_watcher_test
LDFLAGSTEST=-L$(shell pwd) -lfilewatcher -Xlinker -rpath $(shell pwd)

SRC=file_watcher_inotify.c
OBJS=$(patsubst %.c,%.o,$(SRC))

default: $(TARGETLIB) $(TARGETTEST)

$(OBJS): $(SRC)
	$(CC) $(CFLAGS) -fPIC -c -o $@ $^

$(TARGETLIB): $(OBJS)
	$(CC) $(CFLAGS) -shared -o $@ $^

$(TARGETTEST): file_watcher_test.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGSTEST)

.PHONY: clean
clean:
	$(RM) $(OBJS) $(TARGETTEST) $(TARGETLIB)
