
OUTPUT_ARCH(x86_64)
ENTRY(_start)
BASE_ADDRESS = 0x100000000;

SECTIONS {
    . = BASE_ADDRESS;
    skernel = .; /* start of kernel */

    stext = .; /* start of text/code */
    .text : {
        *(.text.entry)
        . = ALIGN(4K);

        *(.text)
        . = ALIGN(4K);
    }
    etext = .; /* end of text/code */

    srodata = .; /* start of read-only data */
    .rodata : { 
        *(.rodata .rodata.*) 
    }
    . = ALIGN(4K);
    erodata = .; /* end of read-only data */


    .data : { *(.data) }
    .bss : { *(.bss) }
}