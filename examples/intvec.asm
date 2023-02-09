; use USB as the boot device, and use FAT32 as the filesystem
ORG 0x0; 
BITS 16 ; 16-bit code, tell the assembler to compile this file with 16-bit instructions

; BPB: BIOS Parameter Block. It is a structure that contains information about the filesystem. It is used by the BIOS to access the filesystem. The BIOS will read the BPB from the boot sector, and use the information in the BPB to access the filesystem.
_bpb:
    jmp short start
    nop ; nop is a no operation instruction, it does nothing. It is used to pad the code so that the start of the code is aligned to a 16-byte boundary. This is required by the BIOS.
    times 33 db 0 ; used as BPB, ref to https://wiki.osdev.org/FAT#BPB. Total size of BPB is 36 bytes, with jmp short xxx and nop heading. So we need 33 bytes to fill the gap.

start:
    jmp 0x7c0:step2 ; jump to the start of the boot program

; for zero-devision exception 
handle_zero:
    mov ah, 0eh ; 0eh is the code for printing a character
    mov al, 'A'
    mov bx, 0x00
    int 0x10
    iret ; return from the interrupt

handle_one:
    mov ah, 0eh ; 0eh is the code for printing a character
    mov al, 'V'
    mov bx, 0x00
    int 0x10
    iret ; return from the interrupt 

step2:
    cli ; disable interrupts
    mov ax, 0x7c0 ; ax is the segment register, 0x7c00 is the segment address of the boot sector
    mov ds, ax ; ds is the data segment register, it is used to access data in memory. Set it to the segment address of the boot sector
    mov es, ax ; es is the extra segment register, it is used to access data in memory. Set it to the segment address of the boot sector
    mov ax, 0
    mov ss, ax ; ss is the stack segment register, it is used to access the stack. Set it to the segment address of the boot sector
    mov sp, 0x7c00 ; sp is the stack pointer, it is used to access the stack. Set it to the start of the boot sector
    sti ; enable interrupts

    ; init interrupt vector table. In real mode, IDT is loacated at 0x0000:0x0000, it's a vector table with 256 entries, each entry is 4 bytes.
    ; Entry is a 4-byte value, the lower 2-byte bits is the offset of the interrupt handler, the higher 2-byte is the segment address of the interrupt handler.
    mov word[ss:0x00], handle_zero ; irt[0x00] = handle_zero, 0x7c0:0x00
    mov word[ss:0x02], 0x7c0

    mov word[ss:0x04], handle_one ; irt[0x01] = handle_one, 0x7c0:0x04
    mov word[ss:0x06], 0x7c0

    int 0 ; trigger a zero-devision exception
    mov ax, 0
    div ax ; trigger a zero-devision exception
    int 1 ; trigger 1st-interrupt

    mov si, message ; si is the address of the string to print
    call print_str ; call the print_char subroutine
    jmp $ ; jump to the current address, loop forever

; print a string to the screen. The string is at si
print_str:
    mov bx, 0 ; bx is the index of the character in the string
.loop:
    lodsb ; load the byte at si into al, and increment si(if DF is 0, increment si, otherwise decrement si)
    cmp al, 0 ; compare al to 0
    je .done ; if al is 0, jump to done
    call print_char ; if al is not 0, print the character
    jmp .loop
.done:
    ret


; print a character to the screen. The character is in al, the color is in bl
print_char:
    mov ah, 0x0e ; 0x0e is the code for printing a character
    @ mov bl, 0x0f ; 0x0f is the color code for white on black
    int 0x10 ; 0x10 is the interrupt code for printing a character
    ret ; return from the subroutine

message: db "Hello, World!", 0 ; the message to print, 0 is the null terminator

    times 510-($-$$) db 0 ; fill the rest of the sector with 0s
    dw 0xaa55 ; the magic number that tells the BIOS that this is a boot sector. This is little endian, so the bytes are reversed.