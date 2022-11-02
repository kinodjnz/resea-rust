pub fn kmain() -> ! {
    super::cycle::init();
    super::main();
    loop {}
}
