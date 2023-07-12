# SPDX-License-Identifier: GPL-2.0

KDIR ?= /lib/modules/$(shell uname -r)/build
MODULE_NAME = dummy_module

default:
	$(MAKE) -C $(KDIR) M=$$PWD
clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) CC=$(CC) clean

# Extra Makefile command that make it faster to insert, remove and inspect the module
ins:
	sudo insmod $(MODULE_NAME).ko
rm:
	sudo rmmod $(MODULE_NAME)
dmesg:
	sudo dmesg | tail -n 20
info:
	sudo modinfo $(MODULE_NAME).ko