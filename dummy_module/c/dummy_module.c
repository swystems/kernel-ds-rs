#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Robert W. Oliver II");
MODULE_DESCRIPTION("A simple example Linux module.");
MODULE_VERSION("0.01");

static int __init
init_fn (void)
{
    pr_info ("Hello, World!\n");
    return 0;
}

static void __exit
exit_fn (void)
{
    pr_info ("Goodbye, World!\n");
}

module_init(init_fn);
module_exit(exit_fn);
