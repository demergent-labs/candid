//! Serialize a Rust data structure to Candid binary format

use super::error::{Error, Result};
use super::types;
#[cfg(feature = "value")]
use super::types::value::IDLValue;
use super::types::{internal::Opcode, Field, Type, TypeEnv, TypeInner};
use byteorder::{LittleEndian, WriteBytesExt};
use leb128::write::{signed as sleb128_encode, unsigned as leb128_encode};
use std::collections::{BTreeMap, TryReserveError};
use std::io;
use std::vec::Vec;

/// Use this struct to serialize a sequence of Rust values (heterogeneous) to IDL binary message.
#[derive(Default)]
pub struct IDLBuilder {
    type_ser: TypeSerialize,
    value_ser: ValueSerializer,
}

impl IDLBuilder {
    pub fn new() -> Self {
        // We cannot share the memo table across different Builder. Because the same Rust
        // type can map to a different but equivalent candid type for different builder,
        // due to memo match happening in different time/order.
        types::internal::env_clear();
        IDLBuilder {
            type_ser: TypeSerialize::new(),
            value_ser: ValueSerializer::new(),
        }
    }
    pub fn arg<'a, T: types::CandidType>(&'a mut self, value: &T) -> Result<&'a mut Self> {
        self.type_ser.push_type(&T::ty())?;
        value.idl_serialize(&mut self.value_ser)?;
        Ok(self)
    }
    #[cfg_attr(docsrs, doc(cfg(feature = "value")))]
    #[cfg(feature = "value")]
    pub fn value_arg<'a>(&'a mut self, value: &IDLValue) -> Result<&'a mut Self> {
        use super::CandidType;
        self.type_ser.push_type(&value.value_ty())?;
        value.idl_serialize(&mut self.value_ser)?;
        Ok(self)
    }
    #[cfg_attr(docsrs, doc(cfg(feature = "value")))]
    #[cfg(feature = "value")]
    /// Annotate IDLValue with (TypeEnv, Type). Note that the TypeEnv will be added to the serializer state.
    /// If the Type can already be resolved by previous TypeEnvs, you don't need to pass TypeEnv again.
    pub fn value_arg_with_type<'a>(
        &'a mut self,
        value: &IDLValue,
        env: &TypeEnv,
        t: &Type,
    ) -> Result<&'a mut Self> {
        use super::CandidType;
        let env = self.type_ser.env.merge(env)?;
        let v = value.annotate_type(true, env, t)?;
        self.type_ser.push_type(t)?;
        v.idl_serialize(&mut self.value_ser)?;
        Ok(self)
    }
    pub fn serialize<W: io::Write>(&mut self, mut writer: W) -> Result<()> {
        writer.write_all(b"DIDL")?;
        self.type_ser.serialize()?;
        writer.write_all(self.type_ser.get_result())?;
        writer.write_all(self.value_ser.get_result())?;
        Ok(())
    }
    pub fn serialize_to_vec(&mut self) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        self.serialize(&mut vec)?;
        Ok(vec)
    }
    /// If serializing a large amount of data, you can try to reserve the capacity of the
    /// value serializer ahead of time to avoid reallocation.
    pub fn try_reserve_value_serializer_capacity(
        &mut self,
        additional: usize,
    ) -> std::result::Result<(), TryReserveError> {
        self.value_ser.try_reserve(additional)
    }
}

/// A structure for serializing Rust values to IDL.
#[derive(Default)]
pub struct ValueSerializer {
    value: Vec<u8>,
}

impl ValueSerializer {
    /// Creates a new IDL serializer.
    #[inline]
    pub fn new() -> Self {
        ValueSerializer { value: Vec::new() }
    }
    pub fn get_result(&self) -> &[u8] {
        &self.value
    }
    #[doc(hidden)]
    pub fn write_leb128(&mut self, value: u64) -> Result<()> {
        leb128_encode(&mut self.value, value)?;
        Ok(())
    }
    #[doc(hidden)]
    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        use std::io::Write;
        self.value.write_all(bytes)?;
        Ok(())
    }
    #[doc(hidden)]
    pub fn try_reserve(&mut self, additional: usize) -> std::result::Result<(), TryReserveError> {
        self.value.try_reserve(additional)
    }
}

macro_rules! serialize_num {
    ($name:ident, $ty:ty, $($method:tt)*) => {
        paste::item! {
            fn [<serialize_ $name>](self, v: $ty) -> Result<()> {
                self.value.$($method)*(v)?;
                Ok(())
            }
        }
    };
}

impl<'a> types::Serializer for &'a mut ValueSerializer {
    type Error = Error;
    type Compound = Compound<'a>;
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.write(&[v as u8])?;
        Ok(())
    }
    #[cfg(feature = "bignum")]
    fn serialize_int(self, v: &crate::Int) -> Result<()> {
        v.encode(&mut self.value)
    }
    #[cfg(feature = "bignum")]
    fn serialize_nat(self, v: &crate::Nat) -> Result<()> {
        v.encode(&mut self.value)
    }
    fn serialize_i128(self, v: i128) -> Result<()> {
        crate::types::leb128::encode_int(&mut self.value, v)
    }
    fn serialize_u128(self, v: u128) -> Result<()> {
        crate::types::leb128::encode_nat(&mut self.value, v)
    }
    serialize_num!(nat8, u8, write_u8);
    serialize_num!(nat16, u16, write_u16::<LittleEndian>);
    serialize_num!(nat32, u32, write_u32::<LittleEndian>);
    serialize_num!(nat64, u64, write_u64::<LittleEndian>);

    serialize_num!(int8, i8, write_i8);
    serialize_num!(int16, i16, write_i16::<LittleEndian>);
    serialize_num!(int32, i32, write_i32::<LittleEndian>);
    serialize_num!(int64, i64, write_i64::<LittleEndian>);

    serialize_num!(float32, f32, write_f32::<LittleEndian>);
    serialize_num!(float64, f64, write_f64::<LittleEndian>);

    fn serialize_text(self, v: &str) -> Result<()> {
        let buf = v.as_bytes();
        self.write_leb128(buf.len() as u64)?;
        self.value.extend_from_slice(buf);
        Ok(())
    }
    fn serialize_null(self, _v: ()) -> Result<()> {
        Ok(())
    }
    fn serialize_empty(self) -> Result<()> {
        Err(Error::msg("cannot encode empty type"))
    }
    fn serialize_principal(self, blob: &[u8]) -> Result<()> {
        self.write(&[1])?;
        self.write_leb128(blob.len() as u64)?;
        self.write(blob)?;
        Ok(())
    }
    fn serialize_function(self, blob: &[u8], meth: &str) -> Result<()> {
        self.write(&[1])?;
        self.serialize_principal(blob)?;
        self.serialize_text(meth)
    }
    fn serialize_option<T>(self, v: Option<&T>) -> Result<()>
    where
        T: super::CandidType + ?Sized,
    {
        match v {
            None => {
                self.write_leb128(0)?;
                Ok(())
            }
            Some(v) => {
                self.write_leb128(1)?;
                v.idl_serialize(self)
            }
        }
    }
    fn serialize_variant(self, index: u64) -> Result<Self::Compound> {
        self.write_leb128(index)?;
        Ok(Self::Compound { ser: self })
    }
    fn serialize_struct(self) -> Result<Self::Compound> {
        Ok(Self::Compound { ser: self })
    }
    fn serialize_vec(self, len: usize) -> Result<Self::Compound> {
        self.write_leb128(len as u64)?;
        Ok(Self::Compound { ser: self })
    }
    fn serialize_blob(self, blob: &[u8]) -> Result<()> {
        self.write_leb128(blob.len() as u64)?;
        self.write(blob)?;
        Ok(())
    }
}

pub struct Compound<'a> {
    ser: &'a mut ValueSerializer,
}
impl types::Compound for Compound<'_> {
    type Error = Error;
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: types::CandidType + ?Sized,
    {
        value.idl_serialize(&mut *self.ser)?;
        Ok(())
    }
    fn serialize_blob(&mut self, blob: &[u8]) -> Result<()> {
        use crate::types::Serializer;
        self.ser.serialize_blob(blob)
    }
}

/// A structure for serializing Rust values to IDL types.
#[derive(Default)]
pub struct TypeSerialize {
    type_table: Vec<Vec<u8>>,
    type_map: BTreeMap<Type, i32>,
    env: TypeEnv,
    args: Vec<Type>,
    result: Vec<u8>,
}

impl TypeSerialize {
    #[inline]
    pub fn new() -> Self {
        TypeSerialize {
            type_table: Vec::new(),
            type_map: BTreeMap::new(),
            env: TypeEnv::new(),
            args: Vec::new(),
            result: Vec::new(),
        }
    }
    pub fn get_result(&self) -> &[u8] {
        &self.result
    }
    #[inline]
    fn build_type(&mut self, t: &Type) -> Result<()> {
        if self.type_map.contains_key(t) {
            return Ok(());
        }
        let actual_type = if let TypeInner::Var(id) = t.as_ref() {
            self.env.rec_find_type(id)?
        } else {
            t
        }
        .clone();
        if types::internal::is_primitive(&actual_type) {
            return Ok(());
        }
        // This is a hack to remove (some) equivalent mu types
        // from the type table.
        // Someone should implement Pottier's O(nlogn) algorithm
        // http://gallium.inria.fr/~fpottier/publis/gauthier-fpottier-icfp04.pdf
        // Disable this "optimization", as unroll is expensive and has to be called on every recursion.
        // let unrolled = types::internal::unroll(t);
        // if let Some(idx) = self.type_map.get(&unrolled) {
        //    let idx = *idx;
        //    self.type_map.insert(t.clone(), idx);
        //    return Ok(());
        // }

        let idx = self.type_table.len();
        self.type_map.insert(t.clone(), idx as i32);
        self.type_table.push(Vec::new());
        let mut buf = Vec::new();
        match actual_type.as_ref() {
            TypeInner::Opt(ref ty) => {
                self.build_type(ty)?;
                sleb128_encode(&mut buf, Opcode::Opt as i64)?;
                self.encode(&mut buf, ty)?;
            }
            TypeInner::Vec(ref ty) => {
                self.build_type(ty)?;
                sleb128_encode(&mut buf, Opcode::Vec as i64)?;
                self.encode(&mut buf, ty)?;
            }
            TypeInner::Record(fs) => {
                for Field { ty, .. } in fs {
                    self.build_type(ty)?;
                }

                sleb128_encode(&mut buf, Opcode::Record as i64)?;
                leb128_encode(&mut buf, fs.len() as u64)?;
                for Field { id, ty } in fs {
                    leb128_encode(&mut buf, u64::from(id.get_id()))?;
                    self.encode(&mut buf, ty)?;
                }
            }
            TypeInner::Variant(fs) => {
                for Field { ty, .. } in fs {
                    self.build_type(ty)?;
                }

                sleb128_encode(&mut buf, Opcode::Variant as i64)?;
                leb128_encode(&mut buf, fs.len() as u64)?;
                for Field { id, ty } in fs {
                    leb128_encode(&mut buf, u64::from(id.get_id()))?;
                    self.encode(&mut buf, ty)?;
                }
            }
            TypeInner::Service(ref ms) => {
                for (_, ty) in ms {
                    self.build_type(ty)?;
                }
                sleb128_encode(&mut buf, Opcode::Service as i64)?;
                leb128_encode(&mut buf, ms.len() as u64)?;
                for (id, ty) in ms {
                    let name = id.as_bytes();
                    leb128_encode(&mut buf, name.len() as u64)?;
                    buf.extend_from_slice(name);
                    self.encode(&mut buf, ty)?;
                }
            }
            TypeInner::Func(ref func) => {
                for ty in func.args.iter().chain(func.rets.iter()) {
                    self.build_type(ty)?;
                }
                for ty in &func.rets {
                    self.build_type(ty)?;
                }
                sleb128_encode(&mut buf, Opcode::Func as i64)?;
                leb128_encode(&mut buf, func.args.len() as u64)?;
                for ty in &func.args {
                    self.encode(&mut buf, ty)?;
                }
                leb128_encode(&mut buf, func.rets.len() as u64)?;
                for ty in &func.rets {
                    self.encode(&mut buf, ty)?;
                }
                leb128_encode(&mut buf, func.modes.len() as u64)?;
                for m in &func.modes {
                    use crate::types::FuncMode;
                    let m = match m {
                        FuncMode::Query => 1,
                        FuncMode::Oneway => 2,
                        FuncMode::CompositeQuery => 3,
                    };
                    sleb128_encode(&mut buf, m)?;
                }
            }
            _ => unreachable!(),
        };
        self.type_table[idx] = buf;
        Ok(())
    }
    #[doc(hidden)]
    pub fn push_type(&mut self, t: &Type) -> Result<()> {
        self.args.push(t.clone());
        self.build_type(t)
    }
    fn encode(&self, buf: &mut Vec<u8>, t: &Type) -> Result<()> {
        if let TypeInner::Var(id) = t.as_ref() {
            let actual_type = self.env.rec_find_type(id)?;
            if types::internal::is_primitive(actual_type) {
                return self.encode(buf, actual_type);
            }
        }
        match t.as_ref() {
            TypeInner::Null => sleb128_encode(buf, Opcode::Null as i64),
            TypeInner::Bool => sleb128_encode(buf, Opcode::Bool as i64),
            TypeInner::Nat => sleb128_encode(buf, Opcode::Nat as i64),
            TypeInner::Int => sleb128_encode(buf, Opcode::Int as i64),
            TypeInner::Nat8 => sleb128_encode(buf, Opcode::Nat8 as i64),
            TypeInner::Nat16 => sleb128_encode(buf, Opcode::Nat16 as i64),
            TypeInner::Nat32 => sleb128_encode(buf, Opcode::Nat32 as i64),
            TypeInner::Nat64 => sleb128_encode(buf, Opcode::Nat64 as i64),
            TypeInner::Int8 => sleb128_encode(buf, Opcode::Int8 as i64),
            TypeInner::Int16 => sleb128_encode(buf, Opcode::Int16 as i64),
            TypeInner::Int32 => sleb128_encode(buf, Opcode::Int32 as i64),
            TypeInner::Int64 => sleb128_encode(buf, Opcode::Int64 as i64),
            TypeInner::Float32 => sleb128_encode(buf, Opcode::Float32 as i64),
            TypeInner::Float64 => sleb128_encode(buf, Opcode::Float64 as i64),
            TypeInner::Text => sleb128_encode(buf, Opcode::Text as i64),
            TypeInner::Reserved => sleb128_encode(buf, Opcode::Reserved as i64),
            TypeInner::Empty => sleb128_encode(buf, Opcode::Empty as i64),
            TypeInner::Principal => sleb128_encode(buf, Opcode::Principal as i64),
            TypeInner::Knot(ref id) => {
                let ty = types::internal::find_type(id)
                    .ok_or_else(|| Error::msg("knot TypeId not found"))?;
                let idx = self
                    .type_map
                    .get(&ty)
                    .ok_or_else(|| Error::msg(format!("knot type {ty} not found")))?;
                sleb128_encode(buf, i64::from(*idx))
            }
            TypeInner::Var(_) => {
                let idx = self
                    .type_map
                    .get(t)
                    .ok_or_else(|| Error::msg(format!("var type {t} not found")))?;
                sleb128_encode(buf, i64::from(*idx))
            }
            TypeInner::Future => unreachable!(),
            _ => {
                let idx = self
                    .type_map
                    .get(t)
                    .ok_or_else(|| Error::msg(format!("type {t} not found")))?;
                sleb128_encode(buf, i64::from(*idx))
            }
        }?;
        Ok(())
    }
    #[doc(hidden)]
    pub fn serialize(&mut self) -> Result<()> {
        leb128_encode(&mut self.result, self.type_table.len() as u64)?;
        self.result.append(&mut self.type_table.concat());

        leb128_encode(&mut self.result, self.args.len() as u64)?;
        let mut ty_encode = Vec::new();
        for t in &self.args {
            self.encode(&mut ty_encode, t)?;
        }
        self.result.append(&mut ty_encode);
        Ok(())
    }
}
