#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/slab.h>
#include <linux/types.h>
#include <linux/mm.h>
#include <linux/io.h>
#include <linux/kthread.h>
#include <asm/uaccess.h>
#include <asm/errno.h>
#include <linux/delay.h>
#include <linux/mutex.h>
#include <asm/ptrace.h>
#include <asm/barrier.h>

#define DEVICE_NAME "rustdev"
#define BUFSIZ_PG 10
#define BUFSIZ_BYTE (PAGE_SIZE * BUFSIZ_PG)
#define MSG_SIZE 4096
#define MSG_SLOTS 8

DEFINE_MUTEX(mutex_mem);

static struct cdev rustdev;
static dev_t dev; // major & minor number
static char *buf = NULL;

static struct task_struct *kth = NULL;

static void alloc_mmap_pages(int npages);
static void free_mmap_pages(int npages);

static int kthreadfn(void *);
static int device_open(struct inode *, struct file *);
static int device_release(struct inode *, struct file *);
static int device_mmap(struct file *, struct vm_area_struct *);

static struct file_operations fops = {
	.owner = THIS_MODULE,
	.open = device_open,
	.release = device_release,
	.mmap = device_mmap,
};

int __init rustdev_init(void)
{
	int ret;
	int major = register_chrdev(0, DEVICE_NAME, &fops);
	if (major < 0)
	{
		printk(KERN_ALERT "Registering char device failed with %d\n", major);
		return major;
	}
	dev = MKDEV(major, 1);
	cdev_init(&rustdev, &fops);
	rustdev.owner = THIS_MODULE;
	ret = cdev_add(&rustdev, dev, 1);
	if (ret < 0)
	{
		printk(KERN_ALERT "adding chrdev failed with %d\n", ret);
		return 0;
	}

	printk(KERN_INFO "char dev mounted - %d\n", major);

	return 0;
}

void __exit rustdev_exit(void)
{
	cdev_del(&rustdev);
	unregister_chrdev(MAJOR(dev), DEVICE_NAME);
	printk("rustdev unregistered\n");
}

static void alloc_mmap_pages(int npages)
{
	int i;
	buf = kmalloc(PAGE_SIZE * npages, GFP_KERNEL);
	printk("buffer allocated\n");

	for (i = 0; i < npages; i++)
	{
		SetPageReserved(virt_to_page(buf + i * PAGE_SIZE));
	}
}

static void free_mmap_pages(int npages)
{
	int i;

	for (i = 0; i < npages; i++)
	{
		ClearPageReserved(virt_to_page(buf + i * PAGE_SIZE));
	}

	kfree(buf);
}

// producer
static int kthreadfn(void *data)
{
	int cnt = 0;
	char msg[30];
	int len;
	unsigned char head;
	unsigned char tail;
	mutex_lock(&mutex_mem);
	printk("head=%u - tail=%u\n", buf[0], buf[1]);
	while (true)
	{
		// ssleep(2);
		sprintf(msg, "message %d", cnt++);
		len = strlen(msg) + 1;
		// printk("produce msg: %s|[%u:%u]\n", msg, buf[0], buf[1]);

		while (true)
		{
			// ssleep(2);
			if (kthread_should_stop())
			{
				printk("producer stopped!\n");
				mutex_unlock(&mutex_mem);
				return 0;
			}
			head = buf[0];
			tail = buf[1];
			if ((head + 1) % MSG_SLOTS != tail)
			{
				char *slot = buf + 2 + MSG_SIZE * head;
				for (int i = 0; i < len; i++)
				{
					slot[i] = msg[i];
				}

				// make sure inc buf[0] after writing msg
				// smp_mb();
				asm volatile("mfence" ::: "memory");
				buf[0] = (head + 1) % MSG_SLOTS;

				// this msg has been written
				// break inner loop and produce new msg
				break;
			}
		}
	}
}

static int device_open(struct inode *inode, struct file *file)
{
	printk("openning...");
	alloc_mmap_pages(BUFSIZ_PG);
	strcpy(buf, "");
	kth = kthread_run(kthreadfn, NULL, "ticker");
	printk("openned! PG_SIZE = %d\n", PAGE_SIZE);
	return 0;
}

static int device_release(struct inode *inode, struct file *file)
{
	printk("releasing...\n");
	kthread_stop(kth);
	printk("free mmap pages...\n");
	mutex_lock(&mutex_mem);
	free_mmap_pages(BUFSIZ_PG);
	mutex_unlock(&mutex_mem);
	return 0;
}

static int device_mmap(struct file *filp, struct vm_area_struct *vma)
{
	if (remap_pfn_range(vma, vma->vm_start,
						virt_to_phys(buf) >> PAGE_SHIFT,
						vma->vm_end - vma->vm_start,
						vma->vm_page_prot))
	{
		printk("ERROR: cannot mmap\n");
		return -EAGAIN;
	}
	buf[0] = buf[1] = 0;
	return 0;
}

MODULE_LICENSE("GPL");
module_init(rustdev_init);
module_exit(rustdev_exit);
