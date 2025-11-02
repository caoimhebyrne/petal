define i32 @main() {
entry:
  %my_value = alloca i32, align 4
  store i32 512, ptr %my_value, align 4
  %my_value1 = load i32, ptr %my_value, align 4
  ret i32 %my_value1
}
