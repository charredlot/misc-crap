CC=gcc
CFLAGS=-g -O2 -fno-omit-frame-pointer -Wall -Werror

ring_wrapped_test: ring_wrapped.o
	$(CC) $(CFLAGS) -o ring_wrapped_test $^ -lrt

.PHONY:
clean:
	rm -f ring_wrapped_test ring_wrapped.o
