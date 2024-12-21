use super::*;

use super::OpValue;
use Instruction::*;

pub struct ExecResult {
    cycles: u64,
    exception: Option<Exception>,
}

impl Hazard3 {
    #[inline]
    pub(super) fn exec_instruction(&mut self, inst: Instruction) -> ExecResult {
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
            Fence => {
                // Do nothing as in the section 3.8.1.21. of rp2350 specification
            }
            Lui(op) => {
                self.registers.write(op.rd, op.imm);
            }
            Auipc(op) => {
                self.registers.write(op.rd, self.pc.wrapping_add(op.imm));
            }
            Jal(op) => {
                self.registers.write(op.rd, self.pc);
                self.pc = op.imm;
            }
            Jalr(op) => {
                self.registers.write(op.rd, self.pc);
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
            // Slli(op) => {
            //     self.registers[op.rd] = self.registers[op.rs1] << op.imm;
            // }
            Slti(op) => {
                todo!()
            }
            Sltiu(op) => {
                let (a, b) = op.get(&self.registers);
                self.registers.write(op.rd, a < b);
            }
            // Srai(op) => {
            //     self.registers[op.rd] = self.registers[op.rs1].signed() >> op.imm.signed();
            // }
            Xori(op) => todo!(),
            Ori(op) => todo!(),
            Andi(op) => todo!(),

            _ => unimplemented!(),
        }

        result
    }
}
