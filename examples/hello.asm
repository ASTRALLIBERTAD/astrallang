section .data
    fmt db '%ld', 10, 0  ; format string for printf

section .text
global _start
extern printf
extern exit

_start:
    push rbp
    mov rbp, rsp
    mov rax, 5
    mov [rbp-8], rax
    mov rax, 10
    mov [rbp-16], rax
    mov rax, [rbp-8]
    push rax
    mov rax, [rbp-16]
    mov rbx, rax
    pop rax
    imul rax, rbx
    mov rsi, rax
    mov rdi, fmt
    mov rax, 0
    call printf
    mov rdi, 0
    call exit
