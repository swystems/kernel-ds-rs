#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/ioctl.h>

#define PAGE_SIZE (getpagesize())
#define MSG_SIZE 128
#define MSG_NUM 500000

int main(int argc, char *argv[])
{
	for (int i = 0; i < 3; i++)
	{
		putchar('\n');
	}

	int fd = open(argv[1], O_RDWR);
	printf("try to open: %s | fd = %d, page_size = %d\n", argv[1], fd, PAGE_SIZE);

	if (fd == -1)
	{
		perror("");
		return 0;
	}
	char *buf = malloc(MSG_SIZE * MSG_NUM);

	for (int i = 0; i < MSG_NUM; i++)
	{
		char *msg = buf + MSG_SIZE * i;
		ioctl(fd, _IOR('c', 1, char[MSG_SIZE]), msg);
		// printf("[%d]read: %s|\n", i, msg);
	}

	double sum_size = MSG_SIZE * MSG_NUM / 1024 / 1024;
	printf("read all %.1fMB, checking...\n", sum_size);

	for (int i = 0; i < MSG_NUM; i++)
	{
		char *msg = buf + MSG_SIZE * i;
		int x;
		sscanf(msg, "message %d", &x);
		if (x != i)
		{
			printf("--------- ERROR! [%d]------------\n", i);
			for (int k = i; k < i+10; k++)
			{
				printf("%s\n", buf + MSG_SIZE * k);
			}
			printf("%s\n", msg);
			close(fd);
			return 0;
		}
	}

	printf("correct\n");

	close(fd);
	return 0;
}
