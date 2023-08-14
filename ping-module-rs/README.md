# Rust Ping Ping LKM

This proof-of-concept module is a simple kernel module that lets two nodes in a network exchange UDP packets.

One node, the `sender` sends a packet; the other node, when the packet is received, will send back to the sender a packet with the received payload.

## Compilation
The kernel module uses features that are not yet available in the Linux tree or the RustForLinux tree. 
In order to work, the modules have to be compiled against this kernel tree: https://github.com/swystems/linux/tree/rust-next-net.

Running `make` will compile the modules against the running kernel. In order to specify a kernel source in a specific folder, the KDIR argument must be specified:
`make KDIR=<path to kernel tree>`.

## How to use
Once the modules are compiled, obtaining `ping_sender.ko` and `ping_receiver.ko`, they must be installed on the nodes:
- In the root folder of the project, run `vagrant up node01 node02` to power up the two nodes. A provisioning script will be executed,
which will install in the machines all the required tools to use and eventually compile Rust modules. If the provisioning is not wanted, it can
be switched off with the `--no-provision` flag.
- Once the machines are booted, on **node01** insert the receiver module:
  - Navigate to the modules folder: `cd /vagrant/ping-module-rs`
  - Insert the compiled receiver module: `sudo insmod ping_receiver.ko`
- On **node02** insert the sender module:
  - Navigate to the modules folder: `cd /vagrant/ping-module-rs`
  - Insert the compiled sender module: `sudo insmod ping_sender.ko`
- On both machine, in the kernel log (`sudo dmesg`), the module will print information about the exchange of messages.
