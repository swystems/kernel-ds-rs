obj-m = rustdev.o
KDIR ?= ~/git-repo/linux

all:
		make LLVM=-15 -C $(KDIR) M=$(PWD)
clean:
		make -C $(KDIR) M=$(PWD) clean
