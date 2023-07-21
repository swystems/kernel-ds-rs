# SPDX-License-Identifier: GPL-2.0

KDIR ?= /linux_src

default:
	$(MAKE) -C $(KDIR) M=$$PWD
clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) CC=$(CC) clean

