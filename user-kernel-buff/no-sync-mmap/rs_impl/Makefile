obj-m = rustdev.o
KDIR ?= ~/git-repo/linux

all:
		gcc userprog.c -static -o userprog
		make LLVM=-15 -C $(KDIR) M=$(PWD) modules
clean:
		make -C $(KDIR) M=$(PWD) clean
		rm userprog