mod emulator;

fn main() {
    println!("12 + 33 = {}", emulator::alu::calculate(4, 12, 33, false).0);
}
