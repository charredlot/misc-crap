#CC=clang-3.5
CC=gcc
CFLAGS=-g -Wall -Werror -O3

default: ssl_agent.c ssl_agent.h
	$(CC) $(CFLAGS) -c -fPIC -o ssl_agent.o ssl_agent.c
	$(CC) $(CFLAGS) -shared -o ssl_agent.so ssl_agent.o

.PHONY: clean
clean:
	$(RM) -f ssl_agent.so ssl_agent.o
