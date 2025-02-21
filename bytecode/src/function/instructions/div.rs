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
    LiteralType,
    Program,
    Value,
};
use snarkvm_circuit::{
    count,
    Count,
    DivChecked,
    Field,
    Literal,
    Metrics,
    Parser,
    ParserResult,
    I128,
    I16,
    I32,
    I64,
    I8,
    U128,
    U16,
    U32,
    U64,
    U8,
};
use snarkvm_utilities::{FromBytes, ToBytes};

use core::{fmt, ops::Div as DivCircuit};
use nom::combinator::map;
use std::io::{Read, Result as IoResult, Write};

/// Divides `first` by `second`, storing the outcome in `destination`.
pub struct Div<P: Program> {
    operation: BinaryOperation<P>,
}

impl<P: Program> Div<P> {
    /// Returns the operands of the instruction.
    pub fn operands(&self) -> Vec<Operand<P>> {
        self.operation.operands()
    }

    /// Returns the destination register of the instruction.
    pub fn destination(&self) -> &Register<P> {
        self.operation.destination()
    }
}

impl<P: Program> Opcode for Div<P> {
    /// Returns the opcode as a string.
    #[inline]
    fn opcode() -> &'static str {
        "div"
    }
}

impl<P: Program> Operation<P> for Div<P> {
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
            (Literal::Field(a), Literal::Field(b)) => Literal::Field(a / b),
            (Literal::I8(a), Literal::I8(b)) => Literal::I8(a.div_checked(&b)),
            (Literal::I16(a), Literal::I16(b)) => Literal::I16(a.div_checked(&b)),
            (Literal::I32(a), Literal::I32(b)) => Literal::I32(a.div_checked(&b)),
            (Literal::I64(a), Literal::I64(b)) => Literal::I64(a.div_checked(&b)),
            (Literal::I128(a), Literal::I128(b)) => Literal::I128(a.div_checked(&b)),
            (Literal::U8(a), Literal::U8(b)) => Literal::U8(a.div_checked(&b)),
            (Literal::U16(a), Literal::U16(b)) => Literal::U16(a.div_checked(&b)),
            (Literal::U32(a), Literal::U32(b)) => Literal::U32(a.div_checked(&b)),
            (Literal::U64(a), Literal::U64(b)) => Literal::U64(a.div_checked(&b)),
            (Literal::U128(a), Literal::U128(b)) => Literal::U128(a.div_checked(&b)),
            _ => P::halt(format!("Invalid '{}' instruction", Self::opcode())),
        };

        registers.assign(self.operation.destination(), result);
    }
}

impl<P: Program> Metrics<Self> for Div<P> {
    type Case = (LiteralType<P::Environment>, LiteralType<P::Environment>);

    fn count(case: &Self::Case) -> Count {
        crate::match_count!(match DivCircuit::count(case) {
            (Field, Field) => Field,
            (I8, I8) => I8,
            (I16, I16) => I16,
            (I32, I32) => I32,
            (I64, I64) => I64,
            (I128, I128) => I128,
            (U8, U8) => U8,
            (U16, U16) => U16,
            (U32, U32) => U32,
            (U64, U64) => U64,
            (U128, U128) => U128,
        })
    }
}

impl<P: Program> Parser for Div<P> {
    type Environment = P::Environment;

    /// Parses a string into a 'div' operation.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        // Parse the operation from the string.
        map(BinaryOperation::parse, |operation| Self { operation })(string)
    }
}

impl<P: Program> fmt::Display for Div<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.operation)
    }
}

impl<P: Program> FromBytes for Div<P> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        Ok(Self { operation: BinaryOperation::read_le(&mut reader)? })
    }
}

impl<P: Program> ToBytes for Div<P> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.operation.write_le(&mut writer)
    }
}

#[allow(clippy::from_over_into)]
impl<P: Program> Into<Instruction<P>> for Div<P> {
    /// Converts the operation into an instruction.
    fn into(self) -> Instruction<P> {
        Instruction::Div(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{function::Register, test_instruction_halts, test_modes, Identifier, Process};

    type P = Process;

    const FIELD_MODE_TESTS: [[&str; 3]; 9] = [
        ["public", "public", "private"],
        ["public", "constant", "public"],
        ["public", "private", "private"],
        ["private", "public", "private"],
        ["private", "constant", "private"],
        ["private", "private", "private"],
        ["constant", "public", "private"],
        ["constant", "constant", "constant"],
        ["constant", "private", "private"],
    ];

    test_modes!(field, Div, "2field", "1field", "2field", FIELD_MODE_TESTS);
    test_modes!(i8, Div, "-4i8", "2i8", "-2i8");
    test_modes!(i16, Div, "-4i16", "2i16", "-2i16");
    test_modes!(i32, Div, "-4i32", "2i32", "-2i32");
    test_modes!(i64, Div, "-4i64", "2i64", "-2i64");
    test_modes!(i128, Div, "-4i128", "2i128", "-2i128");
    test_modes!(u8, Div, "4u8", "2u8", "2u8");
    test_modes!(u16, Div, "4u16", "2u16", "2u16");
    test_modes!(u32, Div, "4u32", "2u32", "2u32");
    test_modes!(u64, Div, "4u64", "2u64", "2u64");
    test_modes!(u128, Div, "4u128", "2u128", "2u128");

    test_instruction_halts!(
        i8_underflow_halts,
        Div,
        "Overflow or underflow on division of two integer constants",
        &format!("{}i8", i8::MIN),
        "-1i8.constant"
    );
    test_instruction_halts!(
        i16_underflow_halts,
        Div,
        "Overflow or underflow on division of two integer constants",
        &format!("{}i16", i16::MIN),
        "-1i16.constant"
    );
    test_instruction_halts!(
        i32_underflow_halts,
        Div,
        "Overflow or underflow on division of two integer constants",
        &format!("{}i32", i32::MIN),
        "-1i32.constant"
    );
    test_instruction_halts!(
        i64_underflow_halts,
        Div,
        "Overflow or underflow on division of two integer constants",
        &format!("{}i64", i64::MIN),
        "-1i64.constant"
    );
    test_instruction_halts!(
        i128_underflow_halts,
        Div,
        "Overflow or underflow on division of two integer constants",
        &format!("{}i128", i128::MIN),
        "-1i128.constant"
    );
    test_instruction_halts!(u8_division_by_zero_halts, Div, "Division by zero error", "1u8.constant", "0u8.constant");
    test_instruction_halts!(
        u16_division_by_zero_halts,
        Div,
        "Division by zero error",
        "1u16.constant",
        "0u16.constant"
    );
    test_instruction_halts!(
        u32_division_by_zero_halts,
        Div,
        "Division by zero error",
        "1u32.constant",
        "0u32.constant"
    );
    test_instruction_halts!(
        u64_division_by_zero_halts,
        Div,
        "Division by zero error",
        "1u64.constant",
        "0u64.constant"
    );
    test_instruction_halts!(
        u128_division_by_zero_halts,
        Div,
        "Division by zero error",
        "1u128.constant",
        "0u128.constant"
    );

    test_instruction_halts!(
        address_halts,
        Div,
        "Invalid 'div' instruction",
        "aleo1d5hg2z3ma00382pngntdp68e74zv54jdxy249qhaujhks9c72yrs33ddah.constant",
        "aleo1d5hg2z3ma00382pngntdp68e74zv54jdxy249qhaujhks9c72yrs33ddah.constant"
    );
    test_instruction_halts!(boolean_halts, Div, "Invalid 'div' instruction", "true.constant", "true.constant");
    test_instruction_halts!(string_halts, Div, "Invalid 'div' instruction", "\"hello\".constant", "\"world\".constant");

    #[test]
    #[should_panic(expected = "message is not a literal")]
    fn test_definition_halts() {
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

        Div::from_str("r0 r1 into r2").evaluate(&registers);
    }
}
