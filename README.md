# os

## 环境准备
```bash
# 安装模拟器
brew install qemu
# 安装x86汇编编译器
brew install nasm
```

## 编译
```bash
nasm -f bin boot.asm -o boot.bin
ndisasm boot.bin
qemu-system-x86_64 -hda boot.bin # load to hard disk 0
```

## 其它命令
```bash
# 写入 U 盘

# 查看设备列表，确定 U 盘挂载路径（比如 /dev/sdb）
sudo fdisk -l 
# 写入镜像
sudo dd if=./boot.bin of=/dev/sdb
```