// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::{
    function::{parsers::*, Instruction, Opcode, Operation, Register, Registers},
    Program,
    Value,
};
use snarkvm_circuit::{Literal, Parser, ParserResult, ShlWrapped as ShlWrappedCircuit};
use snarkvm_utilities::{FromBytes, ToBytes};

use core::fmt;
use nom::combinator::map;
use std::io::{Read, Result as IoResult, Write};

/// Shifts `first` left by `second` bits, wrapping around at the boundary of the type, storing the outcome in `destination`.
pub struct ShlWrapped<P: Program> {
    operation: BinaryOperation<P>,
}

impl<P: Program> ShlWrapped<P> {
    /// Returns the operands of the instruction.
    pub fn operands(&self) -> Vec<Operand<P>> {
        self.operation.operands()
    }

    /// Returns the destination register of the instruction.
    pub fn destination(&self) -> &Register<P> {
        self.operation.destination()
    }
}

impl<P: Program> Opcode for ShlWrapped<P> {
    /// Returns the opcode as a string.
    #[inline]
    fn opcode() -> &'static str {
        "shl.w"
    }
}

impl<P: Program> Operation<P> for ShlWrapped<P> {
    /// Evaluates the operation.
    #[inline]
    fn evaluate(&self, registers: &Registers<P>) {
        // Load the values for the first and second operands.
        let first = match registers.load(self.operation.first()) {
            Value::Literal(literal) => literal,
            Value::Definition(name, ..) => P::halt(format!("{name} is not a literal")),
        };
        let second = match registers.load(self.operation.second()) {
            Value::Literal(literal) => literal,
            Value::Definition(name, ..) => P::halt(format!("{name} is not a literal")),
        };

        // Perform the operation.
        let result = match (first, second) {
            (Literal::I8(a), Literal::U8(b)) => Literal::I8(a.shl_wrapped(&b)),
            (Literal::I8(a), Literal::U16(b)) => Literal::I8(a.shl_wrapped(&b)),
            (Literal::I8(a), Literal::U32(b)) => Literal::I8(a.shl_wrapped(&b)),
            (Literal::I16(a), Literal::U8(b)) => Literal::I16(a.shl_wrapped(&b)),
            (Literal::I16(a), Literal::U16(b)) => Literal::I16(a.shl_wrapped(&b)),
            (Literal::I16(a), Literal::U32(b)) => Literal::I16(a.shl_wrapped(&b)),
            (Literal::I32(a), Literal::U8(b)) => Literal::I32(a.shl_wrapped(&b)),
            (Literal::I32(a), Literal::U16(b)) => Literal::I32(a.shl_wrapped(&b)),
            (Literal::I32(a), Literal::U32(b)) => Literal::I32(a.shl_wrapped(&b)),
            (Literal::I64(a), Literal::U8(b)) => Literal::I64(a.shl_wrapped(&b)),
            (Literal::I64(a), Literal::U16(b)) => Literal::I64(a.shl_wrapped(&b)),
            (Literal::I64(a), Literal::U32(b)) => Literal::I64(a.shl_wrapped(&b)),
            (Literal::I128(a), Literal::U8(b)) => Literal::I128(a.shl_wrapped(&b)),
            (Literal::I128(a), Literal::U16(b)) => Literal::I128(a.shl_wrapped(&b)),
            (Literal::I128(a), Literal::U32(b)) => Literal::I128(a.shl_wrapped(&b)),
            (Literal::U8(a), Literal::U8(b)) => Literal::U8(a.shl_wrapped(&b)),
            (Literal::U8(a), Literal::U16(b)) => Literal::U8(a.shl_wrapped(&b)),
            (Literal::U8(a), Literal::U32(b)) => Literal::U8(a.shl_wrapped(&b)),
            (Literal::U16(a), Literal::U8(b)) => Literal::U16(a.shl_wrapped(&b)),
            (Literal::U16(a), Literal::U16(b)) => Literal::U16(a.shl_wrapped(&b)),
            (Literal::U16(a), Literal::U32(b)) => Literal::U16(a.shl_wrapped(&b)),
            (Literal::U32(a), Literal::U8(b)) => Literal::U32(a.shl_wrapped(&b)),
            (Literal::U32(a), Literal::U16(b)) => Literal::U32(a.shl_wrapped(&b)),
            (Literal::U32(a), Literal::U32(b)) => Literal::U32(a.shl_wrapped(&b)),
            (Literal::U64(a), Literal::U8(b)) => Literal::U64(a.shl_wrapped(&b)),
            (Literal::U64(a), Literal::U16(b)) => Literal::U64(a.shl_wrapped(&b)),
            (Literal::U64(a), Literal::U32(b)) => Literal::U64(a.shl_wrapped(&b)),
            (Literal::U128(a), Literal::U8(b)) => Literal::U128(a.shl_wrapped(&b)),
            (Literal::U128(a), Literal::U16(b)) => Literal::U128(a.shl_wrapped(&b)),
            (Literal::U128(a), Literal::U32(b)) => Literal::U128(a.shl_wrapped(&b)),
            _ => P::halt(format!("Invalid '{}' instruction", Self::opcode())),
        };

        registers.assign(self.operation.destination(), result);
    }
}

impl<P: Program> Parser for ShlWrapped<P> {
    type Environment = P::Environment;

    /// Parses a string into a 'shl.w' operation.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        // Parse the operation from the string.
        let (string, operation) = map(BinaryOperation::parse, |operation| Self { operation })(string)?;
        // Return the operation.
        Ok((string, operation))
    }
}

impl<P: Program> fmt::Display for ShlWrapped<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.operation)
    }
}

impl<P: Program> FromBytes for ShlWrapped<P> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        Ok(Self { operation: BinaryOperation::read_le(&mut reader)? })
    }
}

impl<P: Program> ToBytes for ShlWrapped<P> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.operation.write_le(&mut writer)
    }
}

#[allow(clippy::from_over_into)]
impl<P: Program> Into<Instruction<P>> for ShlWrapped<P> {
    /// Converts the operation into an instruction.
    fn into(self) -> Instruction<P> {
        Instruction::ShlWrapped(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_instruction_halts, test_modes, Identifier, Process, Register};

    type P = Process;

    const SHL_WRAPPED_MODES: [[&str; 3]; 9] = [
        ["public", "public", "private"],
        ["public", "constant", "public"],
        ["public", "private", "private"],
        ["private", "constant", "private"],
        ["private", "public", "private"],
        ["private", "private", "private"],
        ["constant", "private", "private"],
        ["constant", "public", "private"],
        ["constant", "constant", "constant"],
    ];

    #[test]
    fn test_parse() {
        let (_, instruction) = Instruction::<P>::parse("shl.w r0 r1 into r2;").unwrap();
        assert!(matches!(instruction, Instruction::ShlWrapped(_)));
    }

    test_modes!(i8_shl_u8, ShlWrapped, "1i8", "7u8", &format!("{}i8", 1i8 << 7), SHL_WRAPPED_MODES);
    test_modes!(i8_shl_u16, ShlWrapped, "1i8", "7u16", &format!("{}i8", 1i8 << 7), SHL_WRAPPED_MODES);
    test_modes!(i8_shl_u32, ShlWrapped, "1i8", "7u32", &format!("{}i8", 1i8 << 7), SHL_WRAPPED_MODES);
    test_modes!(i8_shl_u8_wraps, ShlWrapped, "1i8", "8u8", "1i8", SHL_WRAPPED_MODES);
    test_modes!(i8_shl_u16_wraps, ShlWrapped, "1i8", "8u16", "1i8", SHL_WRAPPED_MODES);
    test_modes!(i8_shl_u32_wraps, ShlWrapped, "1i8", "8u32", "1i8", SHL_WRAPPED_MODES);

    test_modes!(i16_shl_u8, ShlWrapped, "1i16", "15u8", &format!("{}i16", 1i16 << 15), SHL_WRAPPED_MODES);
    test_modes!(i16_shl_u16, ShlWrapped, "1i16", "15u16", &format!("{}i16", 1i16 << 15), SHL_WRAPPED_MODES);
    test_modes!(i16_shl_u32, ShlWrapped, "1i16", "15u32", &format!("{}i16", 1i16 << 15), SHL_WRAPPED_MODES);
    test_modes!(i16_shl_u8_wraps, ShlWrapped, "1i16", "16u8", "1i16", SHL_WRAPPED_MODES);
    test_modes!(i16_shl_u16_wraps, ShlWrapped, "1i16", "16u16", "1i16", SHL_WRAPPED_MODES);
    test_modes!(i16_shl_u32_wraps, ShlWrapped, "1i16", "16u32", "1i16", SHL_WRAPPED_MODES);

    test_modes!(i32_shl_u8, ShlWrapped, "1i32", "31u8", &format!("{}i32", 1i32 << 31), SHL_WRAPPED_MODES);
    test_modes!(i32_shl_u16, ShlWrapped, "1i32", "31u16", &format!("{}i32", 1i32 << 31), SHL_WRAPPED_MODES);
    test_modes!(i32_shl_u32, ShlWrapped, "1i32", "31u32", &format!("{}i32", 1i32 << 31), SHL_WRAPPED_MODES);
    test_modes!(i32_shl_u8_wraps, ShlWrapped, "1i32", "32u8", "1i32", SHL_WRAPPED_MODES);
    test_modes!(i32_shl_u16_wraps, ShlWrapped, "1i32", "32u16", "1i32", SHL_WRAPPED_MODES);
    test_modes!(i32_shl_u32_wraps, ShlWrapped, "1i32", "32u32", "1i32", SHL_WRAPPED_MODES);

    test_modes!(i64_shl_u8, ShlWrapped, "1i64", "63u8", &format!("{}i64", 1i64 << 63), SHL_WRAPPED_MODES);
    test_modes!(i64_shl_u16, ShlWrapped, "1i64", "63u16", &format!("{}i64", 1i64 << 63), SHL_WRAPPED_MODES);
    test_modes!(i64_shl_u32, ShlWrapped, "1i64", "63u32", &format!("{}i64", 1i64 << 63), SHL_WRAPPED_MODES);
    test_modes!(i64_shl_u8_wraps, ShlWrapped, "1i64", "64u8", "1i64", SHL_WRAPPED_MODES);
    test_modes!(i64_shl_u16_wraps, ShlWrapped, "1i64", "64u16", "1i64", SHL_WRAPPED_MODES);
    test_modes!(i64_shl_u32_wraps, ShlWrapped, "1i64", "64u32", "1i64", SHL_WRAPPED_MODES);

    test_modes!(i128_shl_u8, ShlWrapped, "1i128", "127u8", &format!("{}i128", 1i128 << 127), SHL_WRAPPED_MODES);
    test_modes!(i128_shl_u16, ShlWrapped, "1i128", "127u16", &format!("{}i128", 1i128 << 127), SHL_WRAPPED_MODES);
    test_modes!(i128_shl_u32, ShlWrapped, "1i128", "127u32", &format!("{}i128", 1i128 << 127), SHL_WRAPPED_MODES);
    test_modes!(i128_shl_u8_wraps, ShlWrapped, "1i128", "128u8", "1i128", SHL_WRAPPED_MODES);
    test_modes!(i128_shl_u16_wraps, ShlWrapped, "1i128", "128u16", "1i128", SHL_WRAPPED_MODES);
    test_modes!(i128_shl_u32_wraps, ShlWrapped, "1i128", "128u32", "1i128", SHL_WRAPPED_MODES);

    test_modes!(u8_shl_u8, ShlWrapped, "1u8", "7u8", &format!("{}u8", 1u8 << 7), SHL_WRAPPED_MODES);
    test_modes!(u8_shl_u16, ShlWrapped, "1u8", "7u16", &format!("{}u8", 1u8 << 7), SHL_WRAPPED_MODES);
    test_modes!(u8_shl_u32, ShlWrapped, "1u8", "7u32", &format!("{}u8", 1u8 << 7), SHL_WRAPPED_MODES);
    test_modes!(u8_shl_u8_wraps, ShlWrapped, "1u8", "8u8", "1u8", SHL_WRAPPED_MODES);
    test_modes!(u8_shl_u16_wraps, ShlWrapped, "1u8", "8u16", "1u8", SHL_WRAPPED_MODES);
    test_modes!(u8_shl_u32_wraps, ShlWrapped, "1u8", "8u32", "1u8", SHL_WRAPPED_MODES);

    test_modes!(u16_shl_u8, ShlWrapped, "1u16", "15u8", &format!("{}u16", 1u16 << 15), SHL_WRAPPED_MODES);
    test_modes!(u16_shl_u16, ShlWrapped, "1u16", "15u16", &format!("{}u16", 1u16 << 15), SHL_WRAPPED_MODES);
    test_modes!(u16_shl_u32, ShlWrapped, "1u16", "15u32", &format!("{}u16", 1u16 << 15), SHL_WRAPPED_MODES);
    test_modes!(u16_shl_u8_wraps, ShlWrapped, "1u16", "16u8", "1u16", SHL_WRAPPED_MODES);
    test_modes!(u16_shl_u16_wraps, ShlWrapped, "1u16", "16u16", "1u16", SHL_WRAPPED_MODES);
    test_modes!(u16_shl_u32_wraps, ShlWrapped, "1u16", "16u32", "1u16", SHL_WRAPPED_MODES);

    test_modes!(u32_shl_u8, ShlWrapped, "1u32", "31u8", &format!("{}u32", 1u32 << 31), SHL_WRAPPED_MODES);
    test_modes!(u32_shl_u16, ShlWrapped, "1u32", "31u16", &format!("{}u32", 1u32 << 31), SHL_WRAPPED_MODES);
    test_modes!(u32_shl_u32, ShlWrapped, "1u32", "31u32", &format!("{}u32", 1u32 << 31), SHL_WRAPPED_MODES);
    test_modes!(u32_shl_u8_wraps, ShlWrapped, "1u32", "32u8", "1u32", SHL_WRAPPED_MODES);
    test_modes!(u32_shl_u16_wraps, ShlWrapped, "1u32", "32u16", "1u32", SHL_WRAPPED_MODES);
    test_modes!(u32_shl_u32_wraps, ShlWrapped, "1u32", "32u32", "1u32", SHL_WRAPPED_MODES);

    test_modes!(u64_shl_u8, ShlWrapped, "1u64", "63u8", &format!("{}u64", 1u64 << 63), SHL_WRAPPED_MODES);
    test_modes!(u64_shl_u16, ShlWrapped, "1u64", "63u16", &format!("{}u64", 1u64 << 63), SHL_WRAPPED_MODES);
    test_modes!(u64_shl_u32, ShlWrapped, "1u64", "63u32", &format!("{}u64", 1u64 << 63), SHL_WRAPPED_MODES);
    test_modes!(u64_shl_u8_wraps, ShlWrapped, "1u64", "64u8", "1u64", SHL_WRAPPED_MODES);
    test_modes!(u64_shl_u16_wraps, ShlWrapped, "1u64", "64u16", "1u64", SHL_WRAPPED_MODES);
    test_modes!(u64_shl_u32_wraps, ShlWrapped, "1u64", "64u32", "1u64", SHL_WRAPPED_MODES);

    test_modes!(u128_shl_u8, ShlWrapped, "1u128", "127u8", &format!("{}u128", 1u128 << 127), SHL_WRAPPED_MODES);
    test_modes!(u128_shl_u16, ShlWrapped, "1u128", "127u16", &format!("{}u128", 1u128 << 127), SHL_WRAPPED_MODES);
    test_modes!(u128_shl_u32, ShlWrapped, "1u128", "127u32", &format!("{}u128", 1u128 << 127), SHL_WRAPPED_MODES);
    test_modes!(u128_shl_u8_wraps, ShlWrapped, "1u128", "128u8", "1u128", SHL_WRAPPED_MODES);
    test_modes!(u128_shl_u16_wraps, ShlWrapped, "1u128", "128u16", "1u128", SHL_WRAPPED_MODES);
    test_modes!(u128_shl_u32_wraps, ShlWrapped, "1u128", "128u32", "1u128", SHL_WRAPPED_MODES);

    test_instruction_halts!(
        address_halts,
        ShlWrapped,
        "Invalid 'shl.w' instruction",
        "aleo1d5hg2z3ma00382pngntdp68e74zv54jdxy249qhaujhks9c72yrs33ddah.constant",
        "aleo1d5hg2z3ma00382pngntdp68e74zv54jdxy249qhaujhks9c72yrs33ddah.constant"
    );
    test_instruction_halts!(boolean_halts, ShlWrapped, "Invalid 'shl.w' instruction", "true.constant", "true.constant");
    test_instruction_halts!(group_halts, ShlWrapped, "Invalid 'shl.w' instruction", "2group.constant", "1u8.constant");
    test_instruction_halts!(field_halts, ShlWrapped, "Invalid 'shl.w' instruction", "1field.constant", "1u8.constant");
    test_instruction_halts!(
        scalar_halts,
        ShlWrapped,
        "Invalid 'shl.w' instruction",
        "1scalar.constant",
        "1u8.constant"
    );
    test_instruction_halts!(
        string_halts,
        ShlWrapped,
        "Invalid 'shl.w' instruction",
        "\"hello\".constant",
        "\"world\".constant"
    );

    #[test]
    #[should_panic(expected = "message is not a literal")]
    fn test_composite_halts() {
        let first = Value::<P>::Definition(Identifier::from_str("message"), vec![
            Value::from_str("2group.public"),
            Value::from_str("10field.private"),
        ]);
        let second = first.clone();

        let registers = Registers::<P>::default();
        registers.define(&Register::from_str("r0"));
        registers.define(&Register::from_str("r1"));
        registers.define(&Register::from_str("r2"));
        registers.assign(&Register::from_str("r0"), first);
        registers.assign(&Register::from_str("r1"), second);

        ShlWrapped::from_str("r0 r1 into r2").evaluate(&registers);
    }
}
