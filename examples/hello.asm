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
    mov rax, 0
    mov [rbp-24], rax
    mov rax, 5
    push rax
    mov rax, 10
    mov rbx, rax
    pop rax
    cmp rax, rbx
    setg al
    movzx rax, al
    cmp rax, 0
    je L0
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
    jmp L1
L0:
    mov rax, 59
    mov rsi, rax
    mov rdi, fmt
    mov rax, 0
    call printf
L1:
L2:
    mov rax, 10
    push rax
    mov rax, [rbp-16]
    mov rbx, rax
    pop rax
    cmp rax, rbx
    sete al
    movzx rax, al
    cmp rax, 0
    je L3
    mov rsi, rax
    mov rdi, fmt
    mov rax, 0
    call printf
    jmp L2
L3:
    mov rdi, 0
    call exit
