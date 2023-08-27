import os
from time import sleep

MAX_TOTAL = 2 ** 20 * 256  # 256 MB
BATCHES = 2
LOG = './log'
MSG_NUM = 300


def insmod(mod, data_offset, msg_slots, msg_size):
    cmd = ("sudo insmod {} buf_offset={} msg_slots={} msg_size={}".format(mod, data_offset, msg_slots, msg_size))
    os.system(cmd)


def rmmod(mod='rustdev.ko'):
    cmd = "sudo rmmod {}".format(mod)
    os.system(cmd)


def userprog(prog, data_offset, msg_slots, msg_size, total_size, log=LOG):
    msg_num = total_size // msg_size
    cmd = ("sudo {} /dev/rustdev {} {} {} {} >> {}".format(prog, data_offset, msg_slots, msg_size, msg_num, log))
    print(cmd)
    os.system(cmd)


def test_in_max_total(head, mod, data_offset, msg_slots, msg_size, prog, log=LOG, max_total=MAX_TOTAL, batches=BATCHES):
    insmod(mod, data_offset, msg_slots, msg_size)
    for i in range(1, batches + 1):
        with open(log, 'a') as f:
            f.write('{} {} {}\n'.format(head, msg_size, msg_slots))
        total_size = max_total // batches * i
        userprog(prog, data_offset, msg_slots, msg_size, total_size, log)
    rmmod(mod)


def test_in_msg_num(head, mod, data_offset, msg_slots, msg_size, prog, log=LOG, msg_num=MSG_NUM):
    insmod(mod, data_offset, msg_slots, msg_size)
    with open(log, 'a') as f:
        f.write('{} {} {}\n'.format(head, msg_size, msg_slots))
    userprog(prog, data_offset, msg_slots, msg_size, msg_size * msg_num)
    rmmod(mod)


if __name__ == '__main__':
    # test_in_max_total("@fsrw_atomic_cstmsg[512B*8]", 'fsrw_atomic_cstmsg.ko', 0, 8, 512, './fsrw_cstmsg')
    # test_in_max_total("@fsrw_atomic_cstmsg[4k*8]", 'fsrw_atomic_cstmsg.ko', 0, 8, 4096, './fsrw_cstmsg')
    # test_in_max_total("@fsrw_atomic_cstmsg[512k*8]", 'fsrw_atomic_cstmsg.ko', 0, 8, 2 ** 10 * 512, './fsrw_cstmsg')
    # test_in_max_total("@fsrw_atomic_cstmsg[4m*8]", 'fsrw_atomic_cstmsg.ko', 0, 8, 2 ** 20 * 4, './fsrw_cstmsg')
    #
    # test_in_max_total("@mmap_atomic_cstmsg[512B*8]", 'mmap_atomic_cstmsg.ko', 4096, 8, 512, './mmap_atomic_cstmsg')
    # test_in_max_total("@mmap_atomic_cstmsg[4k*8]", 'mmap_atomic_cstmsg.ko', 4096, 8, 4096, './mmap_atomic_cstmsg')
    # test_in_max_total("@mmap_atomic_cstmsg[512k*8]", 'mmap_atomic_cstmsg.ko', 4096, 8, 2 ** 10 * 512, './mmap_atomic_cstmsg')
    # test_in_max_total("@mmap_atomic_cstmsg[4m*8]", 'mmap_atomic_cstmsg.ko', 4096, 8, 2 ** 20 * 4, './mmap_atomic_cstmsg')
    
    os.system('rm log')
    test_in_msg_num("@fsrw_atomic_cstmsg[512B*8]", 'fsrw_atomic_cstmsg.ko', 0, 8, 512, './fsrw_cstmsg')
    test_in_msg_num("@fsrw_atomic_cstmsg[4k*8]", 'fsrw_atomic_cstmsg.ko', 0, 8, 4096, './fsrw_cstmsg')
    test_in_msg_num("@fsrw_atomic_cstmsg[512k*8]", 'fsrw_atomic_cstmsg.ko', 0, 8, 2 ** 10 * 512, './fsrw_cstmsg')
    test_in_msg_num("@fsrw_atomic_cstmsg[4m*8]", 'fsrw_atomic_cstmsg.ko', 0, 8, 2 ** 20 * 4, './fsrw_cstmsg')

    test_in_msg_num("@mmap_atomic_cstmsg[512B*8]", 'mmap_atomic_cstmsg.ko', 4096, 8, 512, './mmap_atomic_cstmsg')
    test_in_msg_num("@mmap_atomic_cstmsg[4k*8]", 'mmap_atomic_cstmsg.ko', 4096, 8, 4096, './mmap_atomic_cstmsg')
    test_in_msg_num("@mmap_atomic_cstmsg[512k*8]", 'mmap_atomic_cstmsg.ko', 4096, 8, 2 ** 10 * 512,
                    './mmap_atomic_cstmsg')
    test_in_msg_num("@mmap_atomic_cstmsg[4m*8]", 'mmap_atomic_cstmsg.ko', 4096, 8, 2 ** 20 * 4, './mmap_atomic_cstmsg')
