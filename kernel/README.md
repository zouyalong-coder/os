## 环境
```bash
# 安装模拟器
brew install qemu
# 安装x86汇编编译器
brew install nasm

# 安装 bootimage 工具，此工具负责生成bootloader 并打包成系统镜像。
cargo install bootimage
```

## 命令
```bash
# 在 qemu 中运行
cargo run
# 此命令等价于下面几个命令
cargo bootimage # 此命令使用 cargo build 将内核编译为 elf，再编译 bootloader，并将它们打成系统镜像
qemu-system-x86_64 -drive format=raw,file=target/x86_64-myos/debug/bootimage-kernel.bin
```

## 在真机上运行
```bash
dd if=target/x86_64-myos/debug/bootimage-kernel.bin of=/dev/sdX && sync
```