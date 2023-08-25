#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/ioctl.h>
#include <stdbool.h>

#define PAGE_SIZE (getpagesize())
#define MSG_SIZE 4096
#define MSG_SLOTS 8
long long MSG_NUM = 0;

void show_info(double bytes, bool sync);

int main(int argc, char *argv[])
{
	if (argc > 2)
	{
		sscanf(argv[2], "%lld", &MSG_NUM);
	}
	else
	{
		MSG_NUM = 4000000;
	}

	printf("page_size = %d, try to open: %s\n", PAGE_SIZE, argv[1]);
	int fd = open(argv[1], O_RDWR);
	printf("fd = %d\n", fd);

	if (fd == -1)
	{
		perror("");
		return 0;
	}

	char *buf = malloc(MSG_SIZE);
	int sync = true;
	for (int i = 0; i < MSG_NUM; i++)
	{
		char *msg = buf;
		ioctl(fd, _IOR('c', 1, char[MSG_SIZE]), msg);
		int x;
		sscanf(msg, "message %d", &x);
		if (x != i)
		{
			sync = false;
			printf("epected: %d, received: %d\n", i, x);
			break;
		}
	}

	double sum_size = (double)MSG_SIZE * MSG_NUM;
	show_info(sum_size, sync);

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