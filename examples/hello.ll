; Optimized LLVM IR for Astral language
target triple = "aarch64-linux-android"

declare i32 @printf(i8*, ...)
declare void @exit(i32)

@fmt = private constant [5 x i8] c"%ld\0A\00"

define i32 @main() {
entry:
    %0 = alloca i64
    store i64 5, i64* %0
    %1 = alloca i64
    store i64 10, i64* %1
    %2 = alloca i64
    store i64 0, i64* %2
    %3 = getelementptr [5 x i8], [5 x i8]* @fmt, i32 0, i32 0
    call i32 (i8*, ...) @printf(i8* %3, i64 59)
    br label %label0
label0:
    %4 = icmp ne i64 1, 0
    br i1 %4, label %label1, label %label2
label1:
    %5 = add i64 0, 0  ; unimplemented expression
    %6 = getelementptr [5 x i8], [5 x i8]* @fmt, i32 0, i32 0
    call i32 (i8*, ...) @printf(i8* %6, i64 %5)
    br label %label0
label2:
    ret i32 0
}
