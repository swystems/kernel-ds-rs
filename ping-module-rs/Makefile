# SPDX-License-Identifier: GPL-2.0

# KDIR ?= /lib/modules/$(shell uname -r)/build
KDIR = ../linux-rust/

default:
	$(MAKE) -C $(KDIR) M=$$PWD
debug:
	$(MAKE) -C $(KDIR) M=$$PWD EXTRA_CFLAGS="-g -DDEBUG"
clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) CC=$(CC) clean
ins:
	sudo insmod ping_module.ko
rm:
	sudo rmmod ping_module
