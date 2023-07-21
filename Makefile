obj-m = dummy_module.o
KDIR ?= /linux_src
all:
		make -C $(KDIR) M=$(PWD) modules
clean:
		make -C $(KDIR) M=$(PWD) clean
