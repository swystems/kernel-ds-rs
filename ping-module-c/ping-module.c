/*
 * Distributed system network communication LKM.
 *
 * Sample ping-pong application.
 */

#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/sched.h>
#include <linux/kthread.h>
#include <linux/delay.h>
#include <linux/moduleparam.h>
#include <linux/net.h>
#include <linux/netfilter.h>
#include <linux/inet.h>
#include <linux/tcp.h>

MODULE_DESCRIPTION("Simple kernel UDP socket");
MODULE_AUTHOR("Michele Dalle Rive");
MODULE_LICENSE("GPL");

static int PORT = 8000;
module_param(PORT,
int, 0);

static char *ADDRESS = "127.0.0.1";
module_param(ADDRESS, charp,
0000);

#define LISTEN_BACKLOG 128
#define MAX_BUFFER_SIZE 1 * 1024 // 1 KB, compiler limit for the whole frame size is 2048 B

#define STOP_MESSAGE "exit"
#define STOP_MESSAGE_LEN 4
#define PING_MESSAGE "Ping!"
#define PING_MESSAGE_LEN 5
#define PONG_MESSAGE "Pong!\n\0"
#define PONG_MESSAGE_LEN 7

#define _try(fn, errmsg, ...) \
    do {                     \
        int err = fn;        \
        if (err < 0) {       \
            _release();      \
            pr_crit (errmsg, ##__VA_ARGS__); \
            return err;      \
        }                    \
    }while(0)

static struct socket *sock;

static void
_sock_release (struct socket **s, bool kernel)
{
    if (*s == NULL) return;

    if (kernel)
        kernel_sock_shutdown (*s, SHUT_RDWR);
    else
        (*s)->ops->shutdown (*s, 0);

    sock_release (*s);
    *s = NULL;
}

static void
_release (void)
{
    _sock_release (&sock, true);
}
//
//int create_client_sock (void)
//{
//    int err = sock_create_lite (PF_INET, SOCK_STREAM, IPPROTO_TCP, &client_sock);
//    if (err < 0) return err;
//    client_sock->ops = sock->ops;
//    return 0;
//}

int __init
init_fn (void)
{
    __be32 BIND_ADDR = in_aton (ADDRESS);
    pr_crit ("Init binding!\n");

    struct sockaddr_in addr = {
        .sin_family = PF_INET,
        .sin_port = htons (PORT),
        .sin_addr = {BIND_ADDR} // htonl is automatically executed by in_aton
    };

    _try(sock_create_kern (&init_net, PF_INET, SOCK_DGRAM, IPPROTO_UDP, &sock),
         "Could not initialize the socket!");

    _try(sock->ops->bind (sock, (struct sockaddr *) &addr, sizeof (addr)),
         "Could not bind the socket to %pI4:%d", &BIND_ADDR, PORT);

    pr_crit ("Successful bind the socket to %pI4:%d", &BIND_ADDR, PORT);

    //_try(sock->ops->listen (sock, LISTEN_BACKLOG), "Could not listen");

    while (1)
        {
            char buffer[MAX_BUFFER_SIZE + 1];
            struct kvec vec = {buffer, MAX_BUFFER_SIZE};

            struct sockaddr_storage address;
            struct msghdr message = {
                .msg_name = (struct sockaddr *) &address
            };

            pr_info ("\n\nWaiting for some data...");

            int received = kernel_recvmsg (sock, &message, &vec, 1, MAX_BUFFER_SIZE, 0);
            if (received < 0)
                {
                    pr_crit ("Could not receive data, aborting.");
                    _release ();
                    return received;
                }

            buffer[received] = '\0';

            struct sockaddr_in *client = (struct sockaddr_in *) &address;
            pr_info ("Sender: %pI4:%d", &client->sin_addr.s_addr, client->sin_port);
            pr_info ("Received %d bytes", received);
            pr_info ("Received data: %s", buffer);

            if (strncmp (buffer, STOP_MESSAGE, STOP_MESSAGE_LEN) == 0)
                {
                    break;
                }
            else if (strncmp (buffer, PING_MESSAGE, PING_MESSAGE_LEN) == 0)
                {
                    pr_info ("Received ping message, responding with a pong!");
                    strncpy (buffer, PONG_MESSAGE, PONG_MESSAGE_LEN);
                    //struct kvec v = {buffer, MAX_BUFFER_SIZE};
                    int err = kernel_sendmsg (sock, &message, &vec, 1, PONG_MESSAGE_LEN);
                    if (err < 0)
                        {
                            pr_crit ("Could not send Pong! back :(");
                        }
                }
        }

    return 0;
}

void __exit
exit_fn (void)
{
    pr_crit ("Removed socket!");
    _release ();
    pr_crit ("All the resources were releases.");
}

module_init(init_fn);
module_exit(exit_fn);
