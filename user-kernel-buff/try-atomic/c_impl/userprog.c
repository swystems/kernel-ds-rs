#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/ioctl.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <immintrin.h>

#define PAGE_SIZE (getpagesize())
#define MSG_SIZE 4096
// #define MSG_NUM 150
#define MSG_SLOTS 8
#define MMAP_SIZE (MSG_SIZE * MSG_SLOTS + PAGE_SIZE)

int main(int argc, char *argv[])
{
	for (int i = 0; i < 3; i++)
	{
		putchar('\n');
	}

	int MSG_NUM = 0;
	sscanf(argv[2], "%d", &MSG_NUM);

	printf("page_size = %d, try to open: %s\n", PAGE_SIZE, argv[1]);
	int fd = open(argv[1], O_RDWR);
	printf("fd = %d\n", fd);

	if (fd == -1)
	{
		perror("");
		return 0;
	}
	char *buf = mmap(NULL, MMAP_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);

	printf("head = %u\ntail = %u\n", buf[0], buf[1]);

	bool sync = true;
	for (int i = 0; i < MSG_NUM; i++)
	{
		// usleep(100);
		char head = buf[0];
		char tail = buf[1];
		while (head == tail)
		{
			head = buf[0];
		}
		char *msg = buf + 2 + MSG_SIZE * tail;
		msg[MSG_SIZE - 1] = '\0';
		int x;
		sscanf(msg, "message %d", &x);
		if (x != i)
		{
			// printf("expected: %d; received: %d[%u:%u]\n", i, x, buf[0], buf[1]);
			sync = false;
		}
		// printf("--------\n");
		// for (int i = 0; i < MSG_SLOTS; i++)
		// {
		// 	char *slot = buf + 2 + MSG_SIZE * i;
		// 	slot[MSG_SIZE - 1] = '\0';
		// 	printf("|%s|\n", slot);
		// }
		// printf("[%u:%u]\n\n", buf[0], buf[1]);

		// printf("[%d]read: %s|\n", tail, msg);
		asm volatile("mfence" ::: "memory");
		// atomic_thread_fence(memory_order_seq_cst);
		buf[1] = (tail + 1) % MSG_SLOTS;
	}

	double sum_size = MSG_SIZE * MSG_NUM / 1024 / 1024;
	printf("read all %.1fMB, sync = %s\n", sum_size, sync ? "true" : "false");

	close(fd);
	return 0;
}
