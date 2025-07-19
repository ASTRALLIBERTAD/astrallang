all: kernel.iso

out.asm: main.rs examples/hello.astral
	cargo run

out.bin: out.asm
	nasm -f bin out.asm -o out.bin

bootloader.bin: bootloader/boot.asm
	nasm -f bin bootloader/boot.asm -o bootloader.bin

kernel.iso: bootloader.bin out.bin
	cat bootloader.bin out.bin > kernel.iso

run:
	qemu-system-x86_64 -drive format=raw,file=kernel.iso
