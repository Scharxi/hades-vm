use vm::vm::VirtualMachine;

fn main() {
    let mut vm = VirtualMachine::new();
    vm.load_program(&[0x02, 0x01, 0x02, 0x03, 0x01, 0x00, 0x00, 0x00]);
    vm.run_until_completion();
}
