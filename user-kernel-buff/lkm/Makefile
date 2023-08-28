obj-m += fsrw_atomic_cstmsg.o
obj-m += mmap_atomic_cstmsg.o
KDIR ?= ~/git-repo/linux

all:
		make LLVM=-15 -C $(KDIR) M=$(PWD) modules
clean:
		make -C $(KDIR) M=$(PWD) clean