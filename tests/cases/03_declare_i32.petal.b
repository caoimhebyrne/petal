define void @main() {
entry:
  %my_value = alloca i32, align 4
  store i32 512, ptr %my_value, align 4
  ret void
}
