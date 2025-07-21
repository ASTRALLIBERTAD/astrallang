.section .text
.global _start
.global _start
_start:
    // Set up stack frame
    stp x29, x30, [sp, #-16]!
    mov x29, sp
    mov x8, #93      // sys_exit
    mov x0, #0       // exit status
    svc #0           // system call
    mov x0, #5
    str x0, [x29, #-8]
    mov x0, #10
    str x0, [x29, #-16]
    ldr x0, [x29, #-8]
    str x0, [sp, #-16]!
    ldr x0, [x29, #-16]
    mov x1, x0
    ldr x0, [sp], #16
    mul x0, x0, x1
    str x0, [sp, #-16]!
    mov x0, #9
    mov x1, x0
    ldr x0, [sp], #16
    add x0, x0, x1
    adrp x1, buf
    add x1, x1, :lo12:buf
    add x1, x1, #19
    mov x2, #0
convert_loop:
    mov x3, #10
    udiv x4, x0, x3
    msub x5, x4, x3, x0
    add x5, x5, #'0'
    strb w5, [x1], #-1
    mov x0, x4
    add x2, x2, #1
    cmp x0, #0
    b.ne convert_loop
    mov x0, #1
    add x1, x1, #1
    mov x2, x2
    mov x8, #64
    svc #0
    mov x8, #93      // sys_exit
    mov x0, #0       // exit status
    svc #0           // system call

.section .data
buf: .space 20
