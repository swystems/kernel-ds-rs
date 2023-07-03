
# Playground for Rust kernel-level distributed services


This repository contains the framework for a very basic cluster based on Vagrant, Ansible. The has 3+ nodes running Ubuntu 18.04

## Pre-requisites

- Install virtualbox `version 6.1` (newer versions might not be compatible with
vagrant): https://www.virtualbox.org/
- Install Vagrant: https://www.vagrantup.com/
- Install Ansible: https://docs.ansible.com

## Getting Started

- Clone this repository
- Run `vagrant up` to fire up the cluster
- Once the cluster is booted, you can run `vagrant ssh [NODENAME]` where `[NODENAME]` can be `node01`, `node02` etc. See Vagrantfile for an updated list of nodes available.
- The nodes are inst

## Useful Info

- The repositroy folder is automatically sync'd on the `/vagrant` folder of the remote nodes. It can be used as:
    - shared folder between the nodes
    - to copy files and folders to the node
- Hosts IP addresses are registered by name, for example running `ping node02` from
`node01` will automatically get node02 address
- Basic packages (GCC, CLang, Rust etc.) are installed at startup, edit `ansible/` playbooks to install/update additional packagesa
- Useful commands
    - `vagrant provision [NODENAME]` -> run the Ansible playbook only
    - `vagrant reload` -> rebuild the VM
    - `vagrant destroy` -> kill all the machines 
- The virtualbox GUI can be used for managing VMs

## Overview, Compilation and LKM intgration guide

Read `overview_Zampiello.pdf`
