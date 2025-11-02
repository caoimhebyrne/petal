define void @main(i32 %argc, i32 %argv) {
entry:
  %argc1 = alloca i32, align 4
  store i32 %argc, ptr %argc1, align 4
  %argv2 = alloca i32, align 4
  store i32 %argv, ptr %argv2, align 4
  ret void
}
