; x86 instructions can be refer to https://c9x.me/x86/html/file_module_x86_id_160.html
ORG 0x7c00 ; 0x7c00 is the default start address for the boot sector by BIOS
BITS 16 ; 16-bit code, tell the assembler to compile this file with 16-bit instructions

jmp 0x7c0:start ; jump to the start of the boot program

start:
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