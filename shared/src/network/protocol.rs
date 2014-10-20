use serialize::{Decoder, Decodable, Encoder, Encodable};
use std::collections::HashMap;
use std::intrinsics::TypeId;
use component::ComponentHandle;
use anymap::AnyMap;

pub enum NetEncodeError {
    UnknownComponentType,
    UnknownHandle,
    Other
}

pub struct NetEncoder<'a, ReaderType: Reader + 'a> {
    reader: ReaderType,
    handlemapmap: AnyMap
}

type HandleMap<Component> = HashMap<ComponentHandle<Component>, u64>;

impl<'a, ReaderType: Reader + 'a> NetEncoder<'a, ReaderType> {
    pub fn emit_handle<Component>(&mut self, v: ComponentHandle<Component>) -> Result<(), NetEncodeError> {
        // darn you borrow checker
        let maybe_id = match self.handlemapmap.find::<HandleMap<Component>>() {
            Some(handlemap) => {
                match handlemap.find(&v) {
                    Some(&id) => Ok(id),
                    None => Err(UnknownHandle) // TODO: we should probably make a new ID here
                }
            }, 
            None => Err(UnknownComponentType)
        };
        maybe_id.and_then(|id| self.emit_u64(id))
    }
}

impl<'a, ReaderType: Reader+'a> Encoder<NetEncodeError> for NetEncoder<'a, ReaderType> {
    fn emit_nil(&mut self) -> Result<(), NetEncodeError> { unimplemented!()}
    fn emit_uint(&mut self, v: uint) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_u64(&mut self, v: u64) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_u32(&mut self, v: u32) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_u16(&mut self, v: u16) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_u8(&mut self, v: u8) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_int(&mut self, v: int) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_i64(&mut self, v: i64) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_i32(&mut self, v: i32) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_i16(&mut self, v: i16) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_i8(&mut self, v: i8) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_bool(&mut self, v: bool) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_f64(&mut self, v: f64) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_f32(&mut self, v: f32) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_char(&mut self, v: char) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_str(&mut self, v: &str) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_enum(&mut self, name: &str, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_enum_variant(&mut self, v_name: &str, v_id: uint, len: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_enum_variant_arg(&mut self, a_idx: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_enum_struct_variant(&mut self, v_name: &str, v_id: uint, len: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_enum_struct_variant_field(&mut self, f_name: &str, f_idx: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_struct(&mut self, name: &str, len: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{
        fail!("{}", name); unimplemented!()}
    fn emit_struct_field(&mut self, f_name: &str, f_idx: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_tuple(&mut self, len: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_tuple_arg(&mut self, idx: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_tuple_struct(&mut self, name: &str, len: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_tuple_struct_arg(&mut self, f_idx: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_option(&mut self, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_option_none(&mut self) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_option_some(&mut self, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_seq(&mut self, len: uint, f: |this: &mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_seq_elt(&mut self, idx: uint, f: |this: &mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_map(&mut self, len: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_map_elt_key(&mut self, idx: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
    fn emit_map_elt_val(&mut self, idx: uint, f: |&mut NetEncoder<'a, ReaderType>| -> Result<(), NetEncodeError>) -> Result<(), NetEncodeError>{unimplemented!()}
}

#[cfg(test)]
mod test {
}

