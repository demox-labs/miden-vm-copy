use super::{
    parse_checked_param, parse_hex_value, parse_param_with_constant_lookup, Endianness, Felt,
    Instruction::*,
    LocalConstMap,
    Node::{self, Instruction},
    ParsingError, Token, CONSTANT_LABEL_PARSER, HEX_CHUNK_SIZE,
};
use crate::{StarkField, ADVICE_READ_LIMIT, MAX_PUSH_INPUTS};
use alloc::vec::Vec;
use core::ops::RangeBounds;
use vm_core::WORD_SIZE;

// CONSTANTS
// ================================================================================================

/// The maximum parts number allowed for the `push` instruction.
const MAX_PUSH_PARTS: usize = MAX_PUSH_INPUTS + 1;

// INSTRUCTION PARSERS
// ================================================================================================

/// Returns one of the `Push` instruction nodes.
///
/// # Errors
/// Returns an error if the instruction token has invalid values or inappropriate number of
/// values.
pub fn parse_push(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "push");
    match op.num_parts() {
        0 => unreachable!("missing token"),
        1 => Err(ParsingError::missing_param(op, "push.<a?>")),
        2 => {
            let param_str = op.parts()[1];
            match param_str.strip_prefix("0x") {
                // if we have only one hex parameter
                Some(param_str) if param_str.len() <= HEX_CHUNK_SIZE => {
                    let value = parse_hex_value(op, param_str, 1, Endianness::Big)?;
                    build_push_one_instruction(value)
                }
                // if we have many hex parameters without delimiter
                Some(param_str) => parse_long_hex_param(op, param_str),
                // if we have one decimal parameter
                None => {
                    let value = parse_non_hex_param_with_constants_lookup(
                        op,
                        constants,
                        1,
                        0..Felt::MODULUS,
                    )?;
                    build_push_one_instruction(value)
                }
            }
        }
        // if we have many parameters (decimal or hex) separated by delimiters
        3..=MAX_PUSH_PARTS => parse_param_list(op, constants),
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `Locaddr` instruction node.
///
/// # Errors
/// Returns an error if the instruction token contains a wrong number of parameters, or if
/// the provided parameter is not a u16 value.
pub fn parse_locaddr(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "locaddr");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Err(ParsingError::missing_param(op, "locaddr.<index>")),
        2 => {
            let index = parse_param_with_constant_lookup::<u16>(op, 1, constants)?;
            Ok(Instruction(Locaddr(index)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `AdvPush` instruction node.
///
/// # Errors
/// Returns an error if the instruction token does not have exactly one parameter, or if the
/// parameter is smaller than 1 or greater than 16.
pub fn parse_adv_push(op: &Token) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "adv_push");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Err(ParsingError::missing_param(op, "adv_push.<num_vals>")),
        2 => {
            let num_vals = parse_checked_param(op, 1, 1..=ADVICE_READ_LIMIT)?;
            Ok(Instruction(AdvPush(num_vals)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `MemLoad` instruction node if no immediate value is provided, or `MemLoadImm`
/// instruction node otherwise.
///
/// # Errors
/// Returns an error if the instruction token contains a wrong number of parameters, or if
/// the provided parameter is not a u32 value.
pub fn parse_mem_load(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "mem_load");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Ok(Instruction(MemLoad)),
        2 => {
            let address = parse_param_with_constant_lookup::<u32>(op, 1, constants)?;
            Ok(Instruction(MemLoadImm(address)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `LocLoad` instruction node.
///
/// # Errors
/// Returns an error if the instruction token contains a wrong number of parameters, or if
/// the provided parameter is not a u16 value.
pub fn parse_loc_load(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "loc_load");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Err(ParsingError::missing_param(op, "loc_load.<index>")),
        2 => {
            let index = parse_param_with_constant_lookup::<u16>(op, 1, constants)?;
            Ok(Instruction(LocLoad(index)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `MemLoadW` instruction node if no immediate value is provided, or `MemLoadWImm`
/// instruction node otherwise.
///
/// # Errors
/// Returns an error if the instruction token contains a wrong number of parameters, or if
/// the provided parameter is not a u32 value.
pub fn parse_mem_loadw(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "mem_loadw");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Ok(Instruction(MemLoadW)),
        2 => {
            let address = parse_param_with_constant_lookup::<u32>(op, 1, constants)?;
            Ok(Instruction(MemLoadWImm(address)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `LocLoadW` instruction node.
///
/// # Errors
/// Returns an error if the instruction token contains a wrong number of parameters, or if
/// the provided parameter is not a u16 value.
pub fn parse_loc_loadw(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "loc_loadw");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Err(ParsingError::missing_param(op, "loc_loadw.<index>")),
        2 => {
            let index = parse_param_with_constant_lookup::<u16>(op, 1, constants)?;
            Ok(Instruction(LocLoadW(index)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `MemStore` instruction node if no immediate value is provided, or `MemStoreImm`
/// instruction node otherwise.
///
/// # Errors
/// Returns an error if the instruction token contains a wrong number of parameters, or if
/// the provided parameter is not a u32 value.
pub fn parse_mem_store(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "mem_store");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Ok(Instruction(MemStore)),
        2 => {
            let address = parse_param_with_constant_lookup::<u32>(op, 1, constants)?;
            Ok(Instruction(MemStoreImm(address)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `LocStore` instruction node.
///
/// # Errors
/// Returns an error if the instruction token contains a wrong number of parameters, or if
/// the provided parameter is not a u16 value.
pub fn parse_loc_store(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "loc_store");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Err(ParsingError::missing_param(op, "loc_store.<index>")),
        2 => {
            let index = parse_param_with_constant_lookup::<u16>(op, 1, constants)?;
            Ok(Instruction(LocStore(index)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `MemStoreW` instruction node if no immediate value is provided, or `MemStoreWImm`
/// instruction node otherwise.
///
/// # Errors
/// Returns an error if the instruction token contains a wrong number of parameters, or if
/// the provided parameter is not a u32 value.
pub fn parse_mem_storew(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "mem_storew");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Ok(Instruction(MemStoreW)),
        2 => {
            let address = parse_param_with_constant_lookup::<u32>(op, 1, constants)?;
            Ok(Instruction(MemStoreWImm(address)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

/// Returns `LocStoreW` instruction node.
///
/// # Errors
/// Returns an error if the instruction token contains a wrong number of parameters, or if
/// the provided parameter is not a u16 value.
pub fn parse_loc_storew(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    debug_assert_eq!(op.parts()[0], "loc_storew");
    match op.num_parts() {
        0 => unreachable!(),
        1 => Err(ParsingError::missing_param(op, "loc_storew.<index>")),
        2 => {
            let index = parse_param_with_constant_lookup::<u16>(op, 1, constants)?;
            Ok(Instruction(LocStoreW(index)))
        }
        _ => Err(ParsingError::extra_param(op)),
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Parses a list of parameters (each of which could be in decimal or hexadecimal form) and returns
/// an appropriate push instruction node.
fn parse_param_list(op: &Token, constants: &LocalConstMap) -> Result<Node, ParsingError> {
    let values =
        op.parts().iter().enumerate().skip(1).map(|(param_idx, &param_str)| {
            match param_str.strip_prefix("0x") {
                Some(param_str) => parse_hex_value(op, param_str, param_idx, Endianness::Big),
                None => parse_non_hex_param_with_constants_lookup(
                    op,
                    constants,
                    param_idx,
                    0..Felt::MODULUS,
                ),
            }
        });

    build_push_many_instruction(values)
}

/// Parses a non hexadecimal parameter and returns the value. Takes as argument a constant map
/// for constant lookup.
fn parse_non_hex_param_with_constants_lookup<R: RangeBounds<u64>>(
    op: &Token,
    constants: &LocalConstMap,
    param_idx: usize,
    range: R,
) -> Result<u64, ParsingError> {
    let param_str = op.parts()[param_idx];
    // if we have a valid constant label then try and fetch it
    match CONSTANT_LABEL_PARSER.parse_label(param_str) {
        Ok(_) => constants
            .get(param_str)
            .cloned()
            .ok_or_else(|| ParsingError::const_not_found(op)),
        Err(_) => parse_checked_param(op, param_idx, range),
    }
}

/// Parses a 64-character hex string into a word (4 field elements) and returns an appropriate push
/// instruction node.
///
/// # Errors
/// Returns an error if:
/// - The length of hex string is not equal to 64.
/// - If the string does not contain a valid hexadecimal value.
/// - If the parsed value is greater than or equal to the field modulus.
fn parse_long_hex_param(op: &Token, hex_str: &str) -> Result<Node, ParsingError> {
    // handle error cases where the hex string is poorly formed
    if hex_str.len() != HEX_CHUNK_SIZE * WORD_SIZE {
        // hex string doesn't contain a valid number of bytes
        return Err(ParsingError::invalid_param_with_reason(
            op,
            1,
            &format!("long hex string '{hex_str}' must contain exactly 64 characters"),
        ));
    }

    // iterate over the multi-value hex string and parse each 8-byte chunk into a valid u64
    let values = (0..hex_str.len())
        .step_by(HEX_CHUNK_SIZE)
        .map(|i| parse_hex_value(op, &hex_str[i..i + HEX_CHUNK_SIZE], 1, Endianness::Little));

    build_push_many_instruction(values)
}

/// Determines the minimal type appropriate for provided value and returns appropriate instruction
/// for this value
fn build_push_one_instruction(value: u64) -> Result<Node, ParsingError> {
    if let Ok(data) = u8::try_from(value) {
        Ok(Instruction(PushU8(data)))
    } else if let Ok(data) = u16::try_from(value) {
        Ok(Instruction(PushU16(data)))
    } else if let Ok(data) = u32::try_from(value) {
        Ok(Instruction(PushU32(data)))
    } else if value < Felt::MODULUS {
        Ok(Instruction(PushFelt(Felt::new(value))))
    } else {
        unreachable!()
    }
}

/// Determines the minimal type appropriate for provided values iterator and returns appropriate
/// instruction for this values
fn build_push_many_instruction<I>(values_iter: I) -> Result<Node, ParsingError>
where
    I: Iterator<Item = Result<u64, ParsingError>> + Clone + ExactSizeIterator,
{
    assert!(values_iter.len() != 0);
    let max_value = values_iter.clone().try_fold(0, |max, value| Ok(value?.max(max)))?;
    if u8::try_from(max_value).is_ok() {
        let values_u8 = values_iter.map(|v| Ok(v? as u8)).collect::<Result<Vec<u8>, _>>()?;
        Ok(Instruction(PushU8List(values_u8)))
    } else if u16::try_from(max_value).is_ok() {
        let values_u16 = values_iter.map(|v| Ok(v? as u16)).collect::<Result<Vec<u16>, _>>()?;
        Ok(Instruction(PushU16List(values_u16)))
    } else if u32::try_from(max_value).is_ok() {
        let values_u32 = values_iter.map(|v| Ok(v? as u32)).collect::<Result<Vec<u32>, _>>()?;
        Ok(Instruction(PushU32List(values_u32)))
    } else if max_value < Felt::MODULUS {
        let values_len = values_iter.len();
        let values_felt =
            values_iter.map(|imm| Ok(Felt::new(imm?))).collect::<Result<Vec<Felt>, _>>()?;
        if values_len == WORD_SIZE {
            Ok(Instruction(PushWord(
                values_felt.try_into().expect("Invalid constatnts length"),
            )))
        } else {
            Ok(Instruction(PushFeltList(values_felt)))
        }
    } else {
        unreachable!()
    }
}
