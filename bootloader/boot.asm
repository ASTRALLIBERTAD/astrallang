[bits 16]
[org 0x7c00]

start:
    mov ah, 0x0e
    mov al, 'A'
    int 0x10
    jmp $

times 510 - ($ - $$) db 0
dw 0xAA55
