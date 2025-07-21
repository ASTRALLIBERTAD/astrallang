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
    %2 = getelementptr [5 x i8], [5 x i8]* @fmt, i32 0, i32 0
    call i32 (i8*, ...) @printf(i8* %2, i64 50)
    ret i32 0
}
