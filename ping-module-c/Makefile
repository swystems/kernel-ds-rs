# SPDX-License-Identifier: GPL-2.0

KDIR ?= ../linux-rust/

default:
	$(MAKE) -C $(KDIR) M=$$PWD
clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) CC=$(CC) clean
ins:
	sudo insmod ping-module.ko
rm:
	sudo rmmod ping-module