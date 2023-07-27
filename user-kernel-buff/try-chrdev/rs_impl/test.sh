#!/bin/bash

sudo insmod virt_chrdev.ko
sudo ./a.out /dev/rust_miscdev
sudo rmmod virt_chrdev
