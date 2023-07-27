#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>

int main(int argc, char *argv[]) {
	int page_size = getpagesize();
	int msg_size = 32;
	int msg_slots = 100;
	int msgs = 10000;

	int fd = open(argv[1], O_RDWR);
	printf("try to open: %s | fd = %d, page_size = %d\n", argv[1], fd, page_size);
	if (fd == -1) {
		perror("");
		return 0;
	}
	char *buf = malloc(page_size);

	char *p = mmap(NULL, getpagesize(), PROT_READ|PROT_WRITE, MAP_SHARED, fd, 0);
	for (int i = 1; i <= msgs; i++) {
		read(fd, buf, msg_size);

		// check if data is wrote to buf 
		// [synchronized]
		printf("[%d]read: %s\n", i, buf);

		// check if we can access data with mapped memory
		// [NOT synchronized]
		p[page_size-1] = 0;
		printf("mmap: ");
		for (int j = 0; j < msg_slots; j++) {
			for (int k = 0; k < msg_size; k++) {
				char ch = p[j*msg_size+k];
				if (ch == '\0') {
					putchar('|');
					break;
				} else {
					putchar(ch);
				}
			}
		}
		putchar('\n');
	}
	munmap(p, getpagesize());

	close(fd);
	printf("close fd\n");
	return 0;
}

