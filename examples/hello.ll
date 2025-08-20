; LLVM IR generated for Android target
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-linux-android"

declare i32 @printf(ptr, ...)
declare ptr @malloc(i64)
declare void @free(ptr)

@.str.i8 = private unnamed_addr constant [6 x i8] c"%hhd\0A\00", align 1
@.str.i16 = private unnamed_addr constant [5 x i8] c"%hd\0A\00", align 1
@.str.i32 = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1
@.str.i64 = private unnamed_addr constant [5 x i8] c"%ld\0A\00", align 1
@.str.f32 = private unnamed_addr constant [4 x i8] c"%f\0A\00", align 1
@.str.f64 = private unnamed_addr constant [5 x i8] c"%lf\0A\00", align 1
@.str.bool_true = private unnamed_addr constant [6 x i8] c"true\0A\00", align 1
@.str.bool_false = private unnamed_addr constant [7 x i8] c"false\0A\00", align 1
@.str.str = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1

define i64 @main() {
entry:
  %0 = add i64 0, 5
  %1 = alloca i64, align 8
  store i64 %0, ptr %1, align 8
  %2 = add i64 0, 10
  %3 = alloca i64, align 8
  store i64 %2, ptr %3, align 8
  %4 = load i64, ptr %3, align 8
  %5 = call i32 (ptr, ...) @printf(ptr @.str.i64, i64 %4)
  %6 = add i64 0, 59
  %7 = alloca i64, align 8
  store i64 %6, ptr %7, align 8
  %8 = add i64 0, 3
  %9 = alloca i64, align 8
  store i64 %8, ptr %9, align 8
  br label %label0
label0:
  %10 = load i64, ptr %1, align 8
  %11 = load i64, ptr %7, align 8
  %12 = icmp slt i64 %10, %11
  %13 = zext i1 %12 to i64
  %14 = icmp ne i64 %13, 0
  br i1 %14, label %label1, label %label2
label1:
  %15 = load i64, ptr %9, align 8
  %16 = call i32 (ptr, ...) @printf(ptr @.str.i64, i64 %15)
  br label %label0
label2:
  %17 = add i64 0, 8
  %18 = alloca i64, align 8
  store i64 %17, ptr %18, align 8
  %19 = add i64 0, 4
  %20 = add i64 0, %19
  %21 = load i64, ptr %18, align 8
  %22 = trunc i64 %21 to i8
  %23 = call i64 @cow(ptr %20, i8 %22)
  ret i64 0
}

define i64 @cow(ptr %0, i8 %1) {
entry:
  %0 = alloca i64, align 8
  %1 = add i64 0, %0
  store i64 %1, ptr %0, align 8
  %2 = alloca i64, align 8
  %3 = sext i8 %1 to i64
  store i64 %3, ptr %2, align 8
  %4 = load i64, ptr %2, align 8
  %5 = add i64 0, 99
  %6 = add i64 %4, %5
  %7 = alloca i64, align 8
  store i64 %6, ptr %7, align 8
  %8 = load i64, ptr %7, align 8
  %9 = call i32 (ptr, ...) @printf(ptr @.str.i64, i64 %8)
  %10 = add i64 0, 1
  %11 = alloca i64, align 8
  store i64 %10, ptr %11, align 8
  %12 = load i64, ptr %11, align 8
  %13 = icmp ne i64 %12, 0
  br i1 %13, label %label3, label %label4
label3:
  %14 = load i64, ptr %0, align 8
  %15 = call i32 (ptr, ...) @printf(ptr @.str.i64, i64 %14)
  br label %label5
label4:
  br label %label5
label5:
  ret i64 0
}

