obj-m = hello.o
KDIR ?= ~/git-repo/linux

all:
		make LLVM=1 -C $(KDIR) M=$(PWD)
		ssh vagrant@localhost -p 2222 'sudo rmmod hello; sudo insmod /vagrant/user-kernel-buff/try-proc/hello.ko'
clean:
		make -C $(KDIR) M=$(PWD) clean
