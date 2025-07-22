; LLVM IR generated from custom language
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

declare i32 @printf(i8*, ...)
declare i8* @malloc(i64)
declare void @free(i8*)

@.str.num = private unnamed_addr constant [5 x i8] c"%ld\0A\00", align 1
@.str.bool_true = private unnamed_addr constant [6 x i8] c"true\0A\00", align 1
@.str.bool_false = private unnamed_addr constant [7 x i8] c"false\0A\00", align 1
@.str.null = private unnamed_addr constant [6 x i8] c"null\0A\00", align 1
@.str.str = private unnamed_addr constant [4 x i8] c"%s\0A\00", align 1

define i64 @main() {
entry:
  %0 = add i64 0, 5
  %1 = alloca i64, align 8
  store i64 %0, i64* %1, align 8
  %2 = add i64 0, 10
  %3 = alloca i64, align 8
  store i64 %2, i64* %3, align 8
  %4 = add i64 0, 0
  %5 = alloca i64, align 8
  store i64 %4, i64* %5, align 8
  %6 = add i64 0, 5
  %7 = add i64 0, 10
  %8 = icmp sgt i64 %6, %7
  %9 = zext i1 %8 to i64
  %10 = icmp ne i64 %9, 0
  br i1 %10, label %label0, label %label1
label0:
  %11 = load i64, i64* %1, align 8
  %12 = load i64, i64* %3, align 8
  %13 = mul i64 %11, %12
  %14 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.num, i64 0, i64 0), i64 %13)
  br label %label2
label1:
  %15 = add i64 0, 59
  %16 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.num, i64 0, i64 0), i64 %15)
  br label %label2
label2:
  br label %label3
label3:
  %17 = add i64 0, 10
  %18 = load i64, i64* %3, align 8
  %19 = icmp eq i64 %17, %18
  %20 = zext i1 %19 to i64
  %21 = icmp ne i64 %20, 0
  br i1 %21, label %label4, label %label5
label4:
  %22 = add i64 0, 3
  %23 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.num, i64 0, i64 0), i64 %22)
  br label %label3
label5:
  ret i64 0
}

