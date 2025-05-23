use crate::memory::Memory;

pub fn simple(memory: &mut Memory) {
    memory.write_byte(0x0100, 0x3E); // LD A, n
    memory.write_byte(0x0101, 0x05); // A = 5
    memory.write_byte(0x0102, 0x06); // LD B, n
    memory.write_byte(0x0103, 0x01); // B = 1
    memory.write_byte(0x0104, 0x90); // SUB A, B
    memory.write_byte(0x0105, 0x28); // JR Z, offset
    memory.write_byte(0x0106, 0x02); // offset +2
    memory.write_byte(0x0107, 0x18); // JR offset
    memory.write_byte(0x0108, 0xFB); // offset -5
    memory.write_byte(0x0109, 0x00); // NOP
}

pub fn ld_commands(memory: &mut Memory) {
    memory.write_byte(0x0100, 0x3E); // LD A, n
    memory.write_byte(0x0101, 0x42); // A = 0x42

    memory.write_byte(0x0102, 0x47); // LD B, A  (0x47 = dest=0, src=7)
    memory.write_byte(0x0103, 0x48); // LD C, B  (0x48 = dest=1, src=0)
    memory.write_byte(0x0104, 0x51); // LD D, C  (0x51 = dest=2, src=1)
    memory.write_byte(0x0105, 0x5A); // LD E, D  (0x5A = dest=3, src=2)
    memory.write_byte(0x0106, 0x63); // LD H, E  (0x63 = dest=4, src=3)
    memory.write_byte(0x0107, 0x6C); // LD L, H  (0x6C = dest=5, src=4)
    memory.write_byte(0x0108, 0x7D); // LD A, L  (0x7D = dest=7, src=5)

    memory.write_byte(0x0109, 0x00); // NOP
    memory.write_byte(0x010A, 0x00); // NOP
}

pub fn alu(memory: &mut Memory) {
    memory.write_byte(0x0100, 0x3E); // LD A, n
    memory.write_byte(0x0101, 0x0F); // A = 0x0F

    memory.write_byte(0x0102, 0x06); // LD B, n
    memory.write_byte(0x0103, 0x01); // B = 0x01

    memory.write_byte(0x0104, 0x80); // ADD A, B → A = 0x0F + 0x01 = 0x10
    memory.write_byte(0x0105, 0x91); // SUB A, C → A = 0x10 - 0x00 = 0x10
    memory.write_byte(0x0106, 0xA0); // AND A, B → A = 0x10 & 0x01 = 0x00
    memory.write_byte(0x0107, 0x3E); // LD A, n
    memory.write_byte(0x0108, 0x0F); // A = 0x0F
    memory.write_byte(0x0109, 0xA8); // XOR A, B → A = 0x0F ^ 0x01 = 0x0E
    memory.write_byte(0x010A, 0xB0); // OR A, B  → A = 0x0E | 0x01 = 0x0F
    memory.write_byte(0x010B, 0xB8); // CP A, B  → Compare 0x0F with 0x01 (A doesn´t change)

    memory.write_byte(0x010C, 0x00); // NOP
}

pub fn function(memory: &mut Memory) {
    memory.write_byte(0x0100, 0x3E); // LD A, 10
    memory.write_byte(0x0101, 0x0A);
    memory.write_byte(0x0102, 0x06); // LD B, 2
    memory.write_byte(0x0103, 0x02);
    memory.write_byte(0x0104, 0xCD); // CALL 0x0200 
    memory.write_byte(0x0105, 0x00);
    memory.write_byte(0x0106, 0x02);
    memory.write_byte(0x0107, 0x3C); // INC A
    memory.write_byte(0x0108, 0x00); // NOP

    // Function
    memory.write_byte(0x0200, 0x3C); // INC A
    memory.write_byte(0x0201, 0x3C); // INC A
    memory.write_byte(0x0202, 0x04); // INC B
    memory.write_byte(0x0203, 0xC9); // RET
}
