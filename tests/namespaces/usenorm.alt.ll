define void @"a::x" () {
entry:
    ret void
}
define void @"a::y" () {
entry:
    ret void
}
define void @"a::b::z" () {
entry:
    ret void
}
define void @"a::b::l" () {
entry:
    ret void
}
define void @"a::b::ll" () {
entry:
    ret void
}
define void @"main" () {
entry:
    call void @"a::x" ()
    call void @"a::y" ()
    call void @"a::b::z" ()
    call void @"a::b::l" ()
    call void @"a::b::ll" ()
    ret void
}
