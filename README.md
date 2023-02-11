# os

参考：【【系统内核】Udemy 高分付费课程，从头开始开发一个多线程系统内核，双语字幕】https://www.bilibili.com/video/BV1Sx4y137JF?p=15&vd_source=f696611a731053452d16c5cafd7ee1e5

## 环境准备
```bash
# 安装模拟器
brew install qemu
# 安装x86汇编编译器
brew install nasm

# 安装 bootimage 工具，此工具负责生成bootloader 并打包成系统镜像。
cargo install bootimage
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