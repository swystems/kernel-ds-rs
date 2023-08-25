#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/ioctl.h>
#include <stdbool.h>
#include <stdatomic.h>

#define PAGE_SIZE (getpagesize())
#define MSG_SIZE 4096
#define MSG_SLOTS 8
#define BUF_OFFSET 4096
#define MMAP_SIZE (MSG_SIZE * MSG_SLOTS + BUF_OFFSET)

long long MSG_NUM = 0;

void show_info(double, bool);

int main(int argc, char *argv[])
{
	if (argc > 2)
	{
		sscanf(argv[2], "%lld", &MSG_NUM);
	}
	else
	{
		MSG_NUM = 4000;
	}

	printf("page_size = %d, try to read %lld messages from  %s\n", PAGE_SIZE, MSG_NUM, argv[1]);
	int fd = open(argv[1], O_RDWR);
	printf("fd = %d\n", fd);

	if (fd == -1)
	{
		perror("");
		close(fd);
		return 0;
	}

	char *mm = mmap(NULL, MMAP_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
	if (mm == MAP_FAILED)
	{
		perror("");
		close(fd);
		return 0;
	}
	
	char *buf = mm + BUF_OFFSET;
	unsigned long long *headp = (unsigned long long *)mm;
	unsigned long long *tailp = headp + 1;

	bool sync = true;
	for (int i = 0; i < MSG_NUM; i++)
	{
		unsigned long long head = *headp;
		unsigned long long tail = *tailp;
		while (head == tail)
		{
			unsigned long long head = *headp;
			// head = atomic_load((atomic_ullong *)headp);
		}

		char *msg = buf + MSG_SIZE * tail;
		int x;
		sscanf(msg, "message %d", &x);

		*tailp = (tail + 1) % MSG_SLOTS;
		// atomic_store((atomic_ullong *)tailp, (tail + 1) % MSG_SLOTS);

		if (x != i)
		{
			sync = false;
			printf("expected: %d, received: %d\n", i, x);
		}
	}
	double sum_size = MSG_SIZE * (double)MSG_NUM;
	show_info(sum_size, sync);

	munmap(mm, MMAP_SIZE);
	close(fd);
	return 0;
}

void show_info(double bytes, bool sync)
{
	double siz_mb = bytes / 1024 / 1024;
	if (siz_mb > 100)
	{
		printf("read all %.2f GB, sync = %s\n", siz_mb / 1024, sync ? "true" : "false");
	}
	else
	{
		printf("read all %.2f MB, sync = %s\n", siz_mb, sync ? "true" : "false");
	}
	printf("%lld messages transfered\n", MSG_NUM);
}