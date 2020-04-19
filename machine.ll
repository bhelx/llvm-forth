%Stack = type {
    i64,          ; index 0 = sp
    [100 x i64]  ; index 1 = data
}

@dataStack = global %Stack undef
@retnStack = global %Stack undef
@memory = global [1000 x i64] undef

define void @push(%Stack* %stack, i64 %val) {
  %spp = getelementptr %Stack, %Stack* %stack, i32 0, i32 0
  %sp = load i64, i64* %spp
  %data = getelementptr %Stack, %Stack* %stack, i32 0, i32 1
  %addr = getelementptr [100 x i64], [100 x i64]* %data, i64 0, i64 %sp
  store i64 %val, i64* %addr
  %newsp = add i64 %sp, 1
  store i64 %newsp, i64* %spp

  ret void
}

define i64 @peek(%Stack* %stack) {
  %spp = getelementptr %Stack, %Stack* %stack, i32 0, i32 0
  %sp = load i64, i64* %spp
  %topsp = sub i64 %sp, 1
  %data = getelementptr %Stack, %Stack* %stack, i32 0, i32 1
  %addr = getelementptr [100 x i64], [100 x i64]* %data, i64 0, i64 %topsp
  %val = load i64, i64* %addr

  ret i64 %val
}

define i64 @pop(%Stack* %stack) {
  %val = call i64 @peek(%Stack* %stack)

  %spp = getelementptr %Stack, %Stack* %stack, i32 0, i32 0
  %sp = load i64, i64* %spp
  %newsp = sub i64 %sp, 1
  store i64 %newsp, i64* %spp

  ret i64 %val
}

define void @plus() {
  %1 = call i64 @pop(%Stack* @dataStack)
  %2 = call i64 @pop(%Stack* @dataStack)
  %sum = add i64 %1, %2
  call void(%Stack*, i64) @push(%Stack* @dataStack, i64 %sum)
  ret void
}

define void @nand() {
  %1 = call i64 @pop(%Stack* @dataStack)
  %2 = call i64 @pop(%Stack* @dataStack)
  %and = and i64 %1, %2
  %neg = xor i64 %and, -1
  call void(%Stack*, i64) @push(%Stack* @dataStack, i64 %neg)
  ret void
}

define void @fetch() {
  %addr = call i64 @pop(%Stack* @dataStack)
  %ptr = getelementptr [1000 x i64], [1000 x i64]* @memory, i32 0, i64 %addr
  %val = load i64, i64* %ptr
  call void(%Stack*, i64) @push(%Stack* @dataStack, i64 %val)
  ret void
}

define void @store() {
  %addr = call i64 @pop(%Stack* @dataStack)
  %val = call i64 @pop(%Stack* @dataStack)
  %ptr = getelementptr [1000 x i64], [1000 x i64]* @memory, i32 0, i64 %addr
  store i64 %val, i64* %ptr
  ret void
}

define void @rPush() {
  %1 = call i64 @pop(%Stack* @dataStack)
  call void(%Stack*, i64) @push(%Stack* @retnStack, i64 %1)
  ret void
}

define void @rPop() {
  %1 = call i64 @pop(%Stack* @retnStack)
  call void(%Stack*, i64) @push(%Stack* @dataStack, i64 %1)
  ret void
}

define void @initStack(%Stack* %stack) {
  %sp = getelementptr %Stack, %Stack* %stack, i32 0, i32 0
  store i64 0, i64* %sp
  ret void
}

define void @initGlobals() {
  call void(%Stack*) @initStack(%Stack* @dataStack)
  call void(%Stack*) @initStack(%Stack* @retnStack)
  ret void
}

; define i64 @main() #0 {
;   call void() @initGlobals()

;   call void(%Stack*, i64) @push(%Stack* @dataStack, i64 11)
;   call void(%Stack*, i64) @push(%Stack* @dataStack, i64 12)
;   call void(%Stack*, i64) @push(%Stack* @dataStack, i64 13)

;   %val1 = call i64 @pop(%Stack* @dataStack)

;   %val2 = call i64 @peek(%Stack* @dataStack)

;   ret i64 %val2
; }

