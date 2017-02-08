#CC=clang-3.5
CC=gcc
CFLAGS=-g -Wall -Werror -O3 -I/usr/include/freebsd
LDFLAGS=-lelf-freebsd

default: elftest.c
	$(CC) $(CFLAGS) -o elftest $? $(LDFLAGS)

.PHONY: clean
clean:
	$(RM) -f ssl_agent.so ssl_agent.o
