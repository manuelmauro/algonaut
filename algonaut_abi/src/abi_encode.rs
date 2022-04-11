use crate::{
    abi_error::AbiError,
    abi_type::{AbiType, AbiValue, ADDRESS_BYTE_SIZE, LENGTH_ENCODE_BYTE_SIZE, SINGLE_BYTE_SIZE},
    biguint_ext::BigUintExt,
};
use algonaut_core::Address;
use num_bigint::BigUint;
use std::{collections::HashSet, convert::TryInto};

impl AbiType {
    /// ABI type method to encode abi values into bytes following ABI encoding rules
    pub fn encode(&self, value: AbiValue) -> Result<Vec<u8>, AbiError> {
        match self {
            AbiType::UInt { bit_size } | AbiType::UFixed { bit_size, .. } => {
                self.encode_int(value, *bit_size)
            }
            AbiType::Byte => self.encode_bytes(value),
            AbiType::Bool => self.encode_bool(value),
            AbiType::Address => self.encode_static_array(value, &[]),
            AbiType::StaticArray { .. } => self.encode_static_array(value, &[]),
            AbiType::DynamicArray { .. } => self.encode_dynamic_array(value),
            AbiType::String => self.encode_string(value),
            AbiType::Tuple { .. } => self.encode_tuple(value),
        }
    }

    fn encode_int(&self, value: AbiValue, bit_size: u16) -> Result<Vec<u8>, AbiError> {
        match value {
            AbiValue::Int(u) => self.encode_u64(u, bit_size / 8),
            _ => Err(AbiError::Msg(
                "Incompatible value: {value} for ABI type: {self}".to_owned(),
            )),
        }
    }

    fn encode_bytes(&self, value: AbiValue) -> Result<Vec<u8>, AbiError> {
        match value {
            AbiValue::Byte(b) => Ok(vec![b]),
            _ => Err(incompatible_value_for_type_err(self, &value)),
        }
    }

    fn encode_bool(&self, value: AbiValue) -> Result<Vec<u8>, AbiError> {
        match value {
            AbiValue::Bool(b) => Ok(vec![if b { 0x80 } else { 0x00 }]),
            _ => Err(incompatible_value_for_type_err(self, &value)),
        }
    }

    fn encode_u64(&self, value: BigUint, byte_len: u16) -> Result<Vec<u8>, AbiError> {
        let array = value.to_bytes_be_padded(byte_len as usize)?;
        Ok(array)
    }

    fn encode_static_array(&self, value: AbiValue, tup_len: &[usize]) -> Result<Vec<u8>, AbiError> {
        let casted_type = self.type_cast_to_tuple(tup_len)?;
        casted_type.encode(value)
    }

    fn encode_string(&self, value: AbiValue) -> Result<Vec<u8>, AbiError> {
        match value {
            AbiValue::String(str) => {
                let str_bytes: Vec<u8> = str.bytes().collect();
                if str_bytes.len() >= (1 << 16) {
                    return Err(AbiError::Msg(
                        "string casted to byte exceeds uint16 maximum, error".to_owned(),
                    ));
                }

                let tuple_type = self.type_cast_to_tuple(&[str_bytes.len()])?;
                let values = str_bytes.into_iter().map(AbiValue::Byte).collect();
                let encoded_as_tuple = tuple_type.encode(AbiValue::Array(values))?;

                Self::bytes_with_length(encoded_as_tuple.len(), encoded_as_tuple)
            }
            _ => Err(incompatible_value_for_type_err(self, &value)),
        }
    }

    fn encode_dynamic_array(&self, value: AbiValue) -> Result<Vec<u8>, AbiError> {
        match &value {
            AbiValue::Array(values) => {
                let tuple_type = self.type_cast_to_tuple(&[values.len()])?;
                let encoded_as_tuple = tuple_type.encode(value.clone())?;

                Self::bytes_with_length(values.len(), encoded_as_tuple)
            }
            _ => Err(incompatible_value_for_type_err(self, &value)),
        }
    }

    fn bytes_with_length(len: usize, bytes: Vec<u8>) -> Result<Vec<u8>, AbiError> {
        let mut value = vec![];
        let mut length_encode = BigUint::from(len).to_bytes_be_padded(LENGTH_ENCODE_BYTE_SIZE)?;
        value.append(&mut length_encode);
        for b in bytes {
            value.push(b);
        }
        Ok(value)
    }

    fn encode_tuple(&self, value: AbiValue) -> Result<Vec<u8>, AbiError> {
        let values_array = match value {
            AbiValue::Array(array) => array,
            AbiValue::Address(address) => address.0.iter().map(|b| AbiValue::Byte(*b)).collect(),
            _ => {
                return Err(AbiError::Msg(format!(
                    "Can't encode tuple: value: {value:?} is not an array."
                )))
            }
        };

        let children = self.children();
        let children_len = children.len();

        if values_array.len() != children_len {
            return Err(AbiError::Msg(format!(
                "abi tuple child type size ({}) != abi tuple element value size ({})",
                children_len,
                values_array.len(),
            )));
        }

        let mut heads: Vec<Vec<u8>> = vec![vec![]; values_array.len()];
        let mut tails: Vec<Vec<u8>> = vec![vec![]; values_array.len()];

        let mut dynamic_index: HashSet<usize> = HashSet::new();

        let mut i = 0;
        while i < values_array.len() {
            let curr_type = children[i].clone();
            let curr_value = values_array[i].clone();

            let curr_head;
            let curr_tail;

            if curr_type.is_dynamic() {
                curr_head = vec![0x00, 0x00];
                curr_tail = curr_type.encode(curr_value)?;
                dynamic_index.insert(i);
            } else {
                match curr_type {
                    AbiType::Bool => {
                        let before = find_bool_lr(children, i, -1)?;
                        let mut after = find_bool_lr(children, i, 1)?;
                        if before % 8 != 0 {
                            return Err(AbiError::Msg(
                                "expected before has number of bool mod 8 == 0".to_owned(),
                            ));
                        }
                        after = after.min(7);

                        let mut compressed = 0;
                        for bool_index in 0..=after {
                            let value = &values_array[i + bool_index];
                            match value {
                                AbiValue::Bool(b) => {
                                    if *b {
                                        compressed |= 1u8 << (7 - bool_index);
                                    }
                                }
                                _ => {
                                    return Err(AbiError::Msg(format!(
                                        "value: {value:?} is not a bool"
                                    )));
                                }
                            }
                        }

                        curr_head = vec![compressed];
                        curr_tail = vec![];
                        i += after;
                    }
                    _ => {
                        curr_head = curr_type.encode(curr_value)?;
                        curr_tail = vec![];
                    }
                }
            }

            heads[i] = curr_head;
            tails[i] = curr_tail;

            i += 1;
        }

        let mut head_length = 0;
        for h in &heads {
            head_length += h.len();
        }

        let mut tail_curr_length = 0;
        for i in 0..heads.len() {
            if dynamic_index.contains(&i) {
                let head_value = (head_length + tail_curr_length) as u32;
                if head_value >= (1 << 16) {
                    return Err(AbiError::Msg(
                        "encoding error: byte length >= 2^16".to_owned(),
                    ));
                }
                heads[i] = BigUint::from(head_value).to_bytes_be_padded(LENGTH_ENCODE_BYTE_SIZE)?;
            }
            tail_curr_length += tails[i].len();
        }

        Ok([&heads[..], &tails[..]]
            .concat()
            .into_iter()
            .flatten()
            .collect())
    }

    pub fn decode(&self, encoded: &[u8]) -> Result<AbiValue, AbiError> {
        match self {
            AbiType::UInt { bit_size } | AbiType::UFixed { bit_size, .. } => {
                decode_uint(encoded, *bit_size)
            }
            AbiType::Bool => {
                if encoded.len() != 1 {
                    return Err(AbiError::Msg(
                        "boolean byte should be length 1 byte".to_owned(),
                    ));
                }
                if encoded[0] == 0x00 {
                    Ok(AbiValue::Bool(false))
                } else if encoded[0] == 0x80 {
                    Ok(AbiValue::Bool(true))
                } else {
                    Err(AbiError::Msg(
                        "single boolean encoded byte should be of form 0x80 or 0x00".to_owned(),
                    ))
                }
            }
            AbiType::Byte => {
                if encoded.len() != 1 {
                    return Err(AbiError::Msg(
                        "boolean byte should be length 1 byte".to_owned(),
                    ));
                }
                Ok(AbiValue::Byte(encoded[0]))
            }
            AbiType::StaticArray { .. } => {
                let casted_type = self.type_cast_to_tuple(&[])?;
                casted_type.decode(encoded)
            }
            AbiType::Address { .. } => {
                Ok(AbiValue::Address(Address(encoded.try_into().map_err(
                    |e| AbiError::Msg(format!("Address couldn't be decoded from: {e}")),
                )?)))
            }
            AbiType::DynamicArray { .. } => {
                if encoded.len() < LENGTH_ENCODE_BYTE_SIZE {
                    return Err(AbiError::Msg("dynamic array format corrupted".to_owned()));
                }

                let arr: [u8; 2] = encoded[..LENGTH_ENCODE_BYTE_SIZE]
                    .try_into()
                    .map_err(|e| AbiError::Msg(format!("Couldn't convert slice to array: {e}")))?;
                let dynamic_len = u16::from_be_bytes(arr);
                let casted_type = self.type_cast_to_tuple(&[dynamic_len as usize])?;

                casted_type.decode(&encoded[LENGTH_ENCODE_BYTE_SIZE..])
            }
            AbiType::String => {
                if encoded.len() < LENGTH_ENCODE_BYTE_SIZE {
                    return Err(AbiError::Msg("string format corrupted".to_owned()));
                }

                let arr: [u8; 2] = encoded[..LENGTH_ENCODE_BYTE_SIZE]
                    .try_into()
                    .map_err(|e| AbiError::Msg(format!("Couldn't convert slice to string: {e}")))?;
                let byte_len = u16::from_be_bytes(arr);

                if encoded[LENGTH_ENCODE_BYTE_SIZE..].len() != byte_len as usize {
                    return Err(AbiError::Msg(
                        "string representation in byte: length not matching".to_owned(),
                    ));
                }

                let s = String::from_utf8(encoded[LENGTH_ENCODE_BYTE_SIZE..].to_vec()).map_err(
                    |e| AbiError::Msg(format!("Couldn't create string from slice: {e}")),
                )?;
                Ok(AbiValue::String(s))
            }

            AbiType::Tuple { child_types, .. } => {
                Ok(AbiValue::Array(decode_tuple(encoded, child_types)?))
            }
        }
    }

    /// Cast an array-like ABI type into an ABI tuple type.
    fn type_cast_to_tuple(&self, tup_len: &[usize]) -> Result<AbiType, AbiError> {
        let child_types = match self {
            AbiType::Address => {
                let mut child_types = Vec::with_capacity(ADDRESS_BYTE_SIZE);
                for _ in 0..ADDRESS_BYTE_SIZE {
                    child_types.push(AbiType::byte())
                }
                child_types
            }
            AbiType::StaticArray { len, child_type } => {
                let mut child_types = Vec::with_capacity(*len as usize);
                for _ in 0..*len {
                    child_types.push(child_type.as_ref().clone())
                }
                child_types
            }
            AbiType::DynamicArray { child_type } => {
                if tup_len.len() != 1 {
                    return Err(AbiError::Msg(
                        "dynamic array type conversion to tuple need 1 length argument".to_owned(),
                    ));
                }
                let mut child_types = Vec::with_capacity(tup_len[0]);
                for _ in 0..tup_len[0] {
                    child_types.push(child_type.as_ref().clone())
                }
                child_types
            }
            AbiType::String => {
                if tup_len.len() != 1 {
                    return Err(AbiError::Msg(
                        "string type conversion to tuple need 1 length argument".to_owned(),
                    ));
                }
                let mut child_types = Vec::with_capacity(tup_len[0]);

                for _ in 0..tup_len[0] {
                    child_types.push(AbiType::byte())
                }
                child_types
            }
            _ => {
                return Err(AbiError::Msg(
                    "type cannot support conversion to tuple".to_owned(),
                ))
            }
        };

        AbiType::tuple(child_types)
    }

    /// ByteLen method calculates the byte length of a static ABI type.
    pub fn byte_len(&self) -> Result<usize, AbiError> {
        match self {
            AbiType::Address => Ok(ADDRESS_BYTE_SIZE),
            AbiType::Byte => Ok(SINGLE_BYTE_SIZE),
            AbiType::UInt { bit_size } | AbiType::UFixed { bit_size, .. } => {
                Ok((bit_size / 8) as usize)
            }
            AbiType::Bool => Ok(SINGLE_BYTE_SIZE),
            AbiType::StaticArray { len, child_type } => match child_type.as_ref() {
                AbiType::Bool { .. } => {
                    let byte_len = (*len as usize + 7) / 8;
                    Ok(byte_len)
                }
                _ => {
                    let elem_byte_len = child_type.byte_len()?;
                    Ok(*len as usize * elem_byte_len)
                }
            },
            AbiType::Tuple { child_types, .. } => {
                let mut size = 0;
                let mut i = 0;
                while i < child_types.len() {
                    match child_types[i] {
                        AbiType::Bool { .. } => {
                            // search after bool
                            let after = find_bool_lr(self.children(), i, 1)?;
                            // shift the index
                            i += after;
                            // get number of bool
                            let bool_num = after + 1;
                            size += (bool_num + 7) / 8;
                        }
                        _ => {
                            let child_byte_size = self.children()[i].byte_len()?;
                            size += child_byte_size
                        }
                    }

                    i += 1;
                }
                Ok(size)
            }
            _ => Err(AbiError::Msg(format!(
                "Can't pre-compute byte length: {} is a dynamic type",
                self,
            ))),
        }
    }
}

fn incompatible_value_for_type_err(type_: &AbiType, value: &AbiValue) -> AbiError {
    AbiError::Msg(format!(
        "Incompatible value: {value:?} for ABI type: {type_:?}"
    ))
}

fn decode_uint(encoded: &[u8], bit_size: u16) -> Result<AbiValue, AbiError> {
    let byte_size = bit_size / 8;
    if encoded.len() != byte_size as usize {
        return Err(AbiError::Msg(format!(
            "uint/ufixed decode: expected byte length {}, but got byte length {}",
            byte_size,
            encoded.len()
        )));
    }

    Ok(AbiValue::Int(BigUint::from_bytes_be(encoded)))
}

fn decode_tuple(encoded: &[u8], children: &[AbiType]) -> Result<Vec<AbiValue>, AbiError> {
    let mut dynamic_segments: Vec<usize> = Vec::with_capacity(children.len() + 1);
    let mut value_partition: Vec<Vec<u8>> = Vec::with_capacity(children.len());
    let mut iter_index = 0;

    let mut i = 0;
    while i < children.len() {
        if children[i].is_dynamic() {
            if encoded[iter_index..].len() < LENGTH_ENCODE_BYTE_SIZE {
                return Err(AbiError::Msg(
                    "ill formed tuple dynamic typed value encoding".to_owned(),
                ));
            }
            let arr: [u8; 2] = encoded[iter_index..iter_index + LENGTH_ENCODE_BYTE_SIZE]
                .try_into()
                .map_err(|e| AbiError::Msg(format!("Couldn't convert slice to array: {e}")))?;
            let dynamic_index = u16::from_be_bytes(arr);
            dynamic_segments.push(dynamic_index as usize);
            value_partition.push(vec![]);

            iter_index += LENGTH_ENCODE_BYTE_SIZE;
        } else {
            match children[i] {
                AbiType::Bool => {
                    // search previous bool
                    let before = find_bool_lr(children, i, -1)?;
                    // search after bool
                    let mut after = find_bool_lr(children, i, 1)?;
                    if before % 8 == 0 {
                        if after > 7 {
                            after = 7
                        }
                        // parse bool in a byte to multiple byte strings
                        for bool_index in 0..=after {
                            let bool_mask = 0x80 >> bool_index as u8;
                            if encoded[iter_index] & bool_mask > 0 {
                                value_partition.push(vec![0x80]);
                            } else {
                                value_partition.push(vec![0x00]);
                            }
                        }
                        i += after;
                        iter_index += 1;
                    } else {
                        return Err(AbiError::Msg(
                            "expected before bool number mod 8 == 0".to_owned(),
                        ));
                    }
                }
                _ => {
                    // not bool
                    let curr_len = children[i].byte_len()?;

                    if iter_index + curr_len > encoded.len() {
                        return Err(AbiError::Msg(format!("ill formed tuple static typed element encoding: not enough bytes. child: {:?} --- iter_index: {}, current_len: {}, encoded len: {}", children[i], iter_index, curr_len, &encoded.len())));
                    }

                    value_partition.push(encoded[iter_index..iter_index + curr_len].to_vec());
                    iter_index += curr_len;
                }
            }
        }

        if i != children.len() - 1 && iter_index >= encoded.len() {
            return Err(AbiError::Msg("input byte not enough to decode".to_owned()));
        }

        i += 1;
    }

    if !dynamic_segments.is_empty() {
        dynamic_segments.push(encoded.len());
        iter_index = encoded.len()
    }
    if iter_index < encoded.len() {
        return Err(AbiError::Msg(format!(
            "input byte not fully consumed. children: {:?}",
            children
        )));
    }

    let mut index_temp: i32 = -1;
    for var in &dynamic_segments {
        if *var as i32 >= index_temp {
            index_temp = *var as i32;
        } else {
            return Err(AbiError::Msg(format!(
                "dynamic segment should display a [l, r] scope with l <= r, dynamic segment: {:?}",
                dynamic_segments
            )));
        }
    }

    let mut seg_index = 0;
    for i in 0..children.len() {
        if children[i].is_dynamic() {
            value_partition[i] =
                encoded[dynamic_segments[seg_index]..dynamic_segments[seg_index + 1]].to_vec();
            seg_index += 1;
        }
    }

    let mut values = Vec::with_capacity(children.len());
    for i in 0..children.len() {
        values.push(children[i].decode(&value_partition[i])?);
    }

    Ok(values)
}

pub(crate) fn find_bool_lr(types: &[AbiType], index: usize, delta: i32) -> Result<usize, AbiError> {
    let mut until: usize = 0;
    loop {
        let current_index: usize = (index as i32 + delta * until as i32)
            .try_into()
            .map_err(|e| AbiError::Msg(format!("Couldn't convert i32 to usize: {e}")))?;
        match types[current_index] {
            AbiType::Bool => {
                if current_index != types.len() - 1 && delta > 0 || current_index > 0 && delta < 0 {
                    until += 1;
                } else {
                    break;
                }
            }
            _ => {
                until -= 1;
                break;
            }
        }
    }
    Ok(until)
}
