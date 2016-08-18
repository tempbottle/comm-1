CC=gcc
CFLAGS=-I../include/ -L../target/debug/ -lcomm
all: c_example

c_example: c_example.c ../include/comm.h ../target/debug/libcomm.a
	$(CC) $(CFLAGS) -o c_example c_example.c

clean:
	rm -f c_example

