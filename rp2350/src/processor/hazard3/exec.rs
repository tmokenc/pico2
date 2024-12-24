use super::*;

use super::OpValue;
use Instruction::*;

pub struct ExecResult {
    cycles: u64,
    exception: Option<Exception>,
}

impl Hazard3 {
    #[inline]
    pub(super) fn exec_instruction(&mut self) -> ExecResult {
        let code = self.fetch_instruction();
        let inst = Instruction::decode(code);

        let mut next_pc = self.get_pc() + if inst & 0b11 == 0b11 { 4 } else { 2 };

        let mut result = ExecResult {
            cycles: 1,
            exception: None,
        };

        match inst {
            Ebreak => {
                result.cycles = 3;
                result.exception = Some(Exception::BreakPoint);
            }
            Ecall => {
                result.cycles = 3;
                result.exception = Some(match self.privilege_mode {
                    PrivilegeMode::Machine => Exception::EcallMMode,
                    PrivilegeMode::User => Exception::EcallUMode,
                })
            }
            Fence | FenceI => {
                // Do nothing as in the section 3.8.1.21. of rp2350 specification
            }
            Mret => {
                todo!()
            }
            Wfi => {
                todo!()
            }
            Lui(op) => {
                self.registers.write(op.rd, op.imm);
            }
            Auipc(op) => {
                self.registers.write(op.rd, self.pc.wrapping_add(op.imm));
            }
            Jal(op) => {
                self.registers.write(op.rd, next_pc);
                self.pc = op.imm;
            }
            Jalr(op) => {
                self.registers.write(op.rd, next_pc);
                self.pc = op.rs1.wrapping_add(op.imm);
            }

            Beq(op) => {
                let (a, b, imm) = op.get(&self.registers);
                if a == b {
                    self.pc = imm;
                }
            }

            Bne(op) => {
                let (a, b, imm) = op.get(&self.registers);
                if a != b {
                    self.pc = imm;
                }
            }

            Blt(op) => {
                let (a, b, imm) = op.get(&self.registers);
                if a.signed() < b.signed() {
                    self.pc = imm;
                }
            }

            Bltu(op) => {
                let (a, b, imm) = op.get(&self.registers);
                if a < b {
                    self.pc = imm;
                }
            }

            Bge(op) => {
                let (a, b, imm) = op.get(&self.registers);
                if a.signed() >= b.signed() {
                    self.pc = imm;
                }
            }

            Bgeu(op) => {
                let (a, b, imm) = op.get(&self.registers);
                if a >= b {
                    self.pc = op.imm;
                }
            }

            Lb(op) => todo!(),
            Lh(op) => todo!(),
            Lw(op) => todo!(),
            Lbu(op) => todo!(),
            Lhu(op) => todo!(),

            Sb(op) => todo!(),
            Sh(op) => todo!(),
            Sw(op) => todo!(),

            Addi(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a.wrapping_add(b));
            }
            Andi(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a & b);
            }
            Ori(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a | b);
            }
            Slti(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a.signed() < b.signed());
            }
            Sltiu(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a < b);
            }
            Slli(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a << b);
            }
            Srai(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a.signed() >> b);
            }

            Srli(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a >> b);
            }

            Xori(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a ^ b);
            }
            Ori(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a | b);
            }

            Add(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a.wrapping_add(b));
            }
            Sub(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a.wrapping_sub(b));
            }
            Sll(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a << (b & 0b1111));
            }
            Slt(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a.signed() < b.signed());
            }
            Sltu(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a < b);
            }
            Xor(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a ^ b);
            },
            Srl(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a >> (b & 0b1111));
            },
            Sra(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a.signed() >> (b & 0b1111).signed());
            }
            Or(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd a | b);
            },
            And(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd a & b);
            }

            Mul(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a.wrapping_mul(b));
            }
            Mulh(op) => {
                let (a, b) = op.get(&self.registers);
                let result = (a.signed() as i64).wrapping_mul(b.signed() as i64);
                self.registers.write(op.rd, (result >> 32) as u32);
            }
            Mulhsu(op) => {
                let (a, b) = op.get(&self.registers);
                let result = (a.signed() as i64).wrapping_mul(b as u64 as i64);
                self.registers.write(op.rd, (result >> 32) as u32);
            }
            Mulhu(op) => {
                let (a, b) = op.get(&self.registers);
                let result = (a as u64).wrapping_mul(b as u64);
                self.registers.write(op.rd, (result >> 32) as u32);
            }
            Div(op) => {
                let (a, b) = op.get(&self.registers);
                let a = a.signed();
                let b = b.signed();
                let result = if b == 0 { 
                    -1 
                } else if (a == 0x80000000) && (b == -1) {
                    0x80000000
                } else {
                    a.wrapping_div(b)
                };
                     
                } else { a.wrapping_div(b) };
                self.registers.write(op.rd, result);
            },
            Divu(op) => {
                let (a, b) = op.get(&self.registers);
                let result = if b == 0 { 
                    0xffffffff
                } else {
                    a.wrapping_div(b)
                };
                self.registers.write(op.rd, result);
            },
            Rem(op) => {
                let (a, b) = op.get(&self.registers);
                let a = a.signed();
                let b = b.signed();
                let result = if b == 0 { a } else { a.wrapping_rem(b) };
                self.registers.write(op.rd, result);
            }
            Remu(op) => {
                let (a, b) = op.get(&self.registers);
                let result = if b == 0 { a } else { a.wrapping_rem(b) };
                self.registers.write(op.rd, result);
            },

            Sh1add(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, (a << 1).wrapping_add(b));
            }
            Sh2add(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, (a << 2).wrapping_add(b));
            }
            Sh3add(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, (a << 3).wrapping_add(b));
            }

            // Atomic extension
            LrW(op) => todo!(),
            ScW(op) => todo!(),
            AmoaddW(op) => todo!(),
            AmoandW(op) => todo!(),
            AmomaxW(op) => todo!(),
            AmomaxuW(op) => todo!(),
            AmominW(op) => todo!(),
            AmominuW(op) => todo!(),
            AmoxorW(op) => todo!(),
            AmoorW(op) => todo!(),
            AmoswapW(op) => todo!(),

            Andn(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a & !b)
            }
            Clz(op) => {
                let a = op.get(&self.registers);
                self.registers.write(op.rd, a.leading_zeros());
            }
            Cpop(op) => {
                let a = op.get(&self.registers);
                self.registers.write(op.rd, a.count_ones());
            }
            Ctz(op) => {
                let a = op.get(&self.registers);
                self.registers.write(op.rd, a.trailing_zeros());
            }
            Max(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, if a.signed() < b.signed() { b } else { a });
            }
            Maxu(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, if a < b { b } else { a });
            }
            Min(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, if a.signed() < b.signed() { a } else { b });
            }
            Minu(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, if a < b { a } else { b });
            }
            OrcB(op) => {
                let a = op.get(&self.registers);
                todo!()
            }
            Orn(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a | !b);
            }
            Rev8(op) => {
                let a = op.get(&self.registers);
                let result = a.swap_bytes();
                self.registers.write(op.rd, result);
            }
            Rol(op) => {
                let (a, shamt) = op.get(&self.registers);
                let result = a << shamt | a >> (32 - shamt);
                self.registers.write(op.rd, result);
            }
            Ror(op) => {
                let (a, shamt) = op.get(&self.registers);
                let result = a >> shamt | a << (32 - shamt);
                self.registers.write(op.rd, result);
            }
            Rori(op) => {
                let (a, _) = op.get(&self.registers);
                let shamt = op.rs2;
                let result = a >> shamt | a << (32 - shamt);
                self.registers.write(op.rd, result);
            }
            SextB(op) => todo!(),
            SextH(op) => todo!(),
            Xnor(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a ^ !b);
            }
            // ZextB(op) => {
            //     let a = op.get(&self.registers);
            //     self.registers.write(op.rd, a & 0xff);
            // }
            ZextH(op) => {
                let a = op.get(&self.registers);
                self.registers.write(op.rd, a & 0xffff);
            }
            _ => unimplemented!(),
        }

        result
    }
}
