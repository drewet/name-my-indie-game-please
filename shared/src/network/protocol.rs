use serialize::{Decoder, Decodable, Encoder, Encodable};
use std;
use std::collections::HashMap;
use std::intrinsics::TypeId;
use std::io::{MemWriter, MemReader};
use component::ComponentHandle;
use anymap::AnyMap;

#[deriving(Show, PartialEq, Eq)]
pub enum NetProtocolError {
    UnknownHandle,
    NotPortable,
    IoFailed(std::io::IoError),
    Other
}


pub type HandleToIdMap = HashMap<ComponentHandle<()>, u64>;
pub type IdToHandleMap = HashMap<u64, (TypeId, ComponentHandle<()>)>;

pub struct NetEncoder {
    buf: MemWriter,
    handlemap: HandleToIdMap
}

impl NetEncoder {
    pub fn emit_handle<Component>(&mut self, v: ComponentHandle<Component>) -> Result<(), NetProtocolError> {
        // darn you borrow checker
        let maybe_id = match self.handlemap.find(&v.cast()) {
            Some(&id) => Ok(id),
            None => Err(UnknownHandle)
        };
        maybe_id.and_then(|id| self.emit_u64(id))
    }

    pub fn new(handlemap: HandleToIdMap) -> NetEncoder {
        NetEncoder {
            buf: MemWriter::new(),
            handlemap: handlemap
        }
    }

    pub fn unwrap(self) -> (Vec<u8>, HandleToIdMap) {
        (self.buf.unwrap(), self.handlemap)
    }
}

impl Encoder<NetProtocolError> for NetEncoder {
    fn emit_nil(&mut self) -> Result<(), NetProtocolError> { Ok(()) }

    fn emit_uint(&mut self, v: uint) -> Result<(), NetProtocolError> {
        Err(NotPortable) // 32- vs. 64-bit
    }
    fn emit_u64(&mut self, v: u64) -> Result<(), NetProtocolError>{
        self.buf.write_le_u64(v).map_err(IoFailed)
    }
    fn emit_u32(&mut self, v: u32) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_u16(&mut self, v: u16) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_u8(&mut self, v: u8) -> Result<(), NetProtocolError> {
        self.buf.write_u8(v).map_err(IoFailed)
    }
    fn emit_int(&mut self, v: int) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_i64(&mut self, v: i64) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_i32(&mut self, v: i32) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_i16(&mut self, v: i16) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_i8(&mut self, v: i8) -> Result<(), NetProtocolError> {
        self.buf.write_i8(v).map_err(IoFailed)
    }
    fn emit_bool(&mut self, v: bool) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_f64(&mut self, v: f64) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_f32(&mut self, v: f32) -> Result<(), NetProtocolError> {
        self.buf.write_le_f32(v).map_err(IoFailed)
    }
    fn emit_char(&mut self, v: char) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_str(&mut self, v: &str) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_enum(&mut self, name: &str, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_enum_variant(&mut self, v_name: &str, v_id: uint, len: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_enum_variant_arg(&mut self, a_idx: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_enum_struct_variant(&mut self, v_name: &str, v_id: uint, len: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_enum_struct_variant_field(&mut self, f_name: &str, f_idx: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_struct(&mut self, name: &str, len: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError> {
        f(self)
    }
    fn emit_struct_field(&mut self, f_name: &str, f_idx: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{
        f(self)
    }
    fn emit_tuple(&mut self, len: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_tuple_arg(&mut self, idx: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_tuple_struct(&mut self, name: &str, len: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_tuple_struct_arg(&mut self, f_idx: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_option(&mut self, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_option_none(&mut self) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_option_some(&mut self, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_seq(&mut self, len: uint, f: |this: &mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_seq_elt(&mut self, idx: uint, f: |this: &mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_map(&mut self, len: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_map_elt_key(&mut self, idx: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
    fn emit_map_elt_val(&mut self, idx: uint, f: |&mut NetEncoder| -> Result<(), NetProtocolError>) -> Result<(), NetProtocolError>{unimplemented!()}
}

pub struct NetDecoder {
    buf: MemReader,
    idmap: IdToHandleMap
}

impl NetDecoder {
    pub fn read_handle<Component: 'static>(&mut self) -> Result<ComponentHandle<Component>, NetProtocolError> {
        self.read_u64().and_then(|id|
            match self.idmap.find(&id) {
                Some(&(typeid, handle)) if typeid == TypeId::of::<Component>() => {
                    Ok(handle.cast())
                },
                _ => Err(UnknownHandle)
            }
        )
    }

    pub fn new(buf: Vec<u8>, idmap: IdToHandleMap) -> NetDecoder {
        NetDecoder {
            buf: MemReader::new(buf),
            idmap: idmap
        }
    }
}
impl Decoder<NetProtocolError> for NetDecoder {
    fn read_nil(&mut self) -> Result<(), NetProtocolError>{unimplemented!()}
    fn read_uint(&mut self) -> Result<uint, NetProtocolError>{unimplemented!()}
    fn read_u64(&mut self) -> Result<u64, NetProtocolError>{
        self.buf.read_le_u64().map_err(IoFailed)
    }
    fn read_u32(&mut self) -> Result<u32, NetProtocolError>{unimplemented!()}
    fn read_u16(&mut self) -> Result<u16, NetProtocolError>{unimplemented!()}
    fn read_u8(&mut self) -> Result<u8, NetProtocolError>{unimplemented!()}
    fn read_int(&mut self) -> Result<int, NetProtocolError>{unimplemented!()}
    fn read_i64(&mut self) -> Result<i64, NetProtocolError>{unimplemented!()}
    fn read_i32(&mut self) -> Result<i32, NetProtocolError>{unimplemented!()}
    fn read_i16(&mut self) -> Result<i16, NetProtocolError>{unimplemented!()}
    fn read_i8(&mut self) -> Result<i8, NetProtocolError>{unimplemented!()}
    fn read_bool(&mut self) -> Result<bool, NetProtocolError>{unimplemented!()}
    fn read_f64(&mut self) -> Result<f64, NetProtocolError>{unimplemented!()}
    fn read_f32(&mut self) -> Result<f32, NetProtocolError>{unimplemented!()}
    fn read_char(&mut self) -> Result<char, NetProtocolError>{unimplemented!()}
    fn read_str(&mut self) -> Result<String, NetProtocolError>{unimplemented!()}
    fn read_enum<T>(&mut self, name: &str, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_enum_variant<T>(&mut self, names: &[&str], f: |&mut NetDecoder, uint| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_enum_variant_arg<T>(&mut self, a_idx: uint, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_enum_struct_variant<T>(&mut self, names: &[&str], f: |&mut NetDecoder, uint| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_enum_struct_variant_field<T>(&mut self, f_name: &str, f_idx: uint, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_struct<T>(&mut self, s_name: &str, len: uint, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{
        f(self)
    }
    fn read_struct_field<T>(&mut self, f_name: &str, f_idx: uint, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError> {
        f(self)
    }
    fn read_tuple<T>(&mut self, f: |&mut NetDecoder, uint| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_tuple_arg<T>(&mut self, a_idx: uint, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_tuple_struct<T>(&mut self, s_name: &str, f: |&mut NetDecoder, uint| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_tuple_struct_arg<T>(&mut self, a_idx: uint, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_option<T>(&mut self, f: |&mut NetDecoder, bool| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_seq<T>(&mut self, f: |&mut NetDecoder, uint| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_seq_elt<T>(&mut self, idx: uint, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_map<T>(&mut self, f: |&mut NetDecoder, uint| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_map_elt_key<T>(&mut self, idx: uint, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn read_map_elt_val<T>(&mut self, idx: uint, f: |&mut NetDecoder| -> Result<T, NetProtocolError>) -> Result<T, NetProtocolError>{unimplemented!()}
    fn error(&mut self, err: &str) -> NetProtocolError{unimplemented!()}
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::{
        NetEncoder,
        NetDecoder,
        UnknownHandle
    };
    use std::intrinsics::TypeId;
    use serialize::{Decodable, Encodable};
    use cgmath;
    use component::{ComponentHandle, ComponentStore, EntityComponent};

    #[test]
    fn smoke_encoding() {
        let mut enc = NetEncoder::new(HashMap::new());
        b'x'.encode(&mut enc).unwrap();
        let (buf, _) = enc.unwrap();
        assert_eq!(buf.as_slice(), b"x");
    }

    #[test]
    fn component_encoding_unknownhandle() {
        let mut enc = NetEncoder::new(HashMap::new());
        
        let mut entities = ComponentStore::new();
        let ent = EntityComponent::new(&mut entities,
                                       cgmath::Point3::new(0., 0., 0.),
                                       cgmath::Quaternion::new(0., 0., 0., 0.,)
                                       );
        assert_eq!(ent.encode(&mut enc), Err(UnknownHandle));
    }
    
    #[test]
    fn component_encoding() {
        let mut handlemap = HashMap::new();
        
        let mut entities = ComponentStore::new();
        let ent = EntityComponent::new(&mut entities,
                                       cgmath::Point3::new(0., 0., 0.),
                                       cgmath::Quaternion::new(0., 0., 0., 0.,)
                                       );
        handlemap.insert(ent.cast(), 0);

        let mut enc = NetEncoder::new(handlemap);
        assert_eq!(ent.encode(&mut enc), Ok(()));
    }
    #[test]
    fn component_decoding() {
        let mut handlemap = HashMap::new();
        let mut idmap = HashMap::new();

        let mut entities = ComponentStore::new();
        let ent = EntityComponent::new(&mut entities,
                                       cgmath::Point3::new(0., 0., 0.),
                                       cgmath::Quaternion::new(0., 0., 0., 0.,)
                                       );
        handlemap.insert(ent.cast(), 0);
        idmap.insert(0, (TypeId::of::<EntityComponent>(), ent.cast()));

        let mut enc = NetEncoder::new(handlemap);
        assert_eq!(ent.encode(&mut enc), Ok(()));
        let (buf, _) = enc.unwrap();

        let mut dec = NetDecoder::new(buf, idmap);
        let decoded: ComponentHandle<EntityComponent> = Decodable::decode(&mut dec).unwrap();
        assert!(decoded == ent);
    }

}

