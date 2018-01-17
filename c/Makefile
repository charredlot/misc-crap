ifeq ($(CC), cc)
CC=gcc
endif

CFLAGS=-g -fno-omit-frame-pointer -Wall -Werror

ring_wrapped_test: ring_wrapped.o
	$(CC) $(CFLAGS) -o ring_wrapped_test $^ -lrt

.PHONY:
clean:
	rm -f ring_wrapped_test ring_wrapped.o
