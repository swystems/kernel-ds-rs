Vagrant.configure("2") do |config|
  config.ssh.username = "vagrant"
  config.ssh.password = "vagrant"
  config.vm.box = "bento/ubuntu-22.04"

  config.vm.define "node01" do |node01|
    node01.vm.hostname = "node01"
    node01.vm.network :private_network, ip: "192.168.56.101"
  end

  config.vm.define "node02" do |node02|
    node02.vm.hostname = "node02"
    node02.vm.network :private_network, ip: "192.168.56.102"
  end

  config.vm.define "node03" do |node03|
    node03.vm.hostname = "node03"
    node03.vm.network :private_network, ip: "192.168.56.103"
  end

  config.vm.synced_folder "~/git-repo/linux", "/linux_src"

  config.vm.provider :virtualbox do |vb|
    vb.customize ["modifyvm", :id, "--natdnshostresolver1", "on"]
    vb.memory = 2048
    vb.cpus = 4
  end

#config.vm.provision "shell", path: "install.sh"
#config.vm.provision "shell", reboot: true
#config.vm.provision "shell", inline: "uname -a"
  config.vm.provision "shell", inline: "apt install make && cd /linux_src && make install"
  config.vm.provision "shell", reboot: true

end
