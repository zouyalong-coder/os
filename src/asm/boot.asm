; use USB as the boot device, and use FAT32 as the filesystem
ORG 0x0; 
BITS 16 ; 16-bit code, tell the assembler to compile this file with 16-bit instructions

section .text
; BPB: BIOS Parameter Block. It is a structure that contains information about the filesystem. It is used by the BIOS to access the filesystem. The BIOS will read the BPB from the boot sector, and use the information in the BPB to access the filesystem.
; .global entry
entry:
    jmp short start
    nop ; nop is a no operation instruction, it does nothing. It is used to pad the code so that the start of the code is aligned to a 16-byte boundary. This is required by the BIOS.
    times 33 db 0 ; used as BPB, ref to https://wiki.osdev.org/FAT#BPB. Total size of BPB is 36 bytes, with jmp short xxx and nop heading. So we need 33 bytes to fill the gap.

start:
    jmp 0x7c0:step2 ; jump to the start of the boot program



step2:
    cli ; disable interrupts
    mov ax, 0x7c0 ; ax is the segment register, 0x7c00 is the segment address of the boot sector
    mov ds, ax ; ds is the data segment register, it is used to access data in memory. Set it to the segment address of the boot sector
    mov es, ax ; es is the extra segment register, it is used to access data in memory. Set it to the segment address of the boot sector
    mov ax, 0
    mov ss, ax ; ss is the stack segment register, it is used to access the stack. Set it to the segment address of the boot sector
    mov sp, 0x7c00 ; sp is the stack pointer, it is used to access the stack. Set it to the start of the boot sector
    sti ; enable interrupts

    ; read content from disk sector
    mov ah, 2; read a sector
    mov al, 1; read 1 sector
    mov ch, 0; read from cylinder 0
    mov cl, 2; read from sector 2
    mov dh, 0; read from head 0
    mov bx, buffer
    int 0x13; call the BIOS interrupt to read the sector, the sector is read into the buffer
    jc error; if the carry flag(CF) is set, jump to error
    
    mov si, buffer
    call print_str
    jmp $

error:
    mov si, error_message
    call print_str

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

error_message: db 'Fail to load sector', 0

    times 510-($-$$) db 0 ; fill the rest of the sector with 0s
    dw 0xaa55 ; the magic number that tells the BIOS that this is a boot sector. This is little endian, so the bytes are reversed.

buffer: