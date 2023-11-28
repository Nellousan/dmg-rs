pub struct Instruction {
    mnemonic: String,
    length: u32,
    cycles: u32,
    cycles2: Option<u32>,
}
