mod emulator;

fn main() {
    println!("12 + 33 = {}", emulator::alu::calculate(4, 12, 33, false).0);

    let inst = emulator::cpu::Instruction::new(0b1000001100010100010).unwrap();
    print!("Instruction: {:025b}, ", inst.get_instruction());
    println!("MRGAB0-3: {:b}", inst.get_register_address_b());
}
