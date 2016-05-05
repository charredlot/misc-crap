#CC=clang-3.5
CC=gcc
CFLAGS=-g -Wall -Werror -O3

TARGET=file_watcher_test
SRC=file_watcher_inotify.c file_watcher_test.c
OBJS=$(patsubst %.c,%.o,$(SRC))

default: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $(TARGET) $(OBJS)

.PHONY: clean
clean:
	$(RM) $(OBJS) $(TARGET)
