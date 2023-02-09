all:
	nasm -fbin -o target/boot.bin ./src/asm/boot.asm
	dd if=./message.txt >> ./target/boot.bin
	dd if=/dev/zero bs=512 count=1 >> ./target/boot.bin