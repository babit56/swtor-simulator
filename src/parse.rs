use std::{path::Path, io::BufReader, fs::File};

use bevy::utils::HashMap;
use serde::{de::{Error, Unexpected}, Deserialize, Deserializer};
use serde_json::Value;
// use assoc::AssocExt;

#[derive(Deserialize, Debug)]
pub struct Node {
    pub id: String,
    pub fqn: String,
    pub path: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub id: String,
    // field_type: usize,
    pub value: FieldValue,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CodeClass(pub Vec<Field>);

#[derive(Deserialize, Debug)]
pub struct NodeObjPair {
    pub node: Node,
    pub obj: CodeClass,
}

fn deserialize_with_type<'de, D>(deserializer: D, value_type: u64) -> Result<FieldValue, D::Error>
where
    D: Deserializer<'de>
{
        Ok(match value_type {
            1 => FieldValue::Id(Int::<u64>::deserialize(deserializer)?.0),
            2 => FieldValue::Int(Int::<i64>::deserialize(deserializer)?.0),
            3 => FieldValue::Boolean(bool::deserialize(deserializer)?),
            4 => FieldValue::Float(f32::deserialize(deserializer)?),
            5 => FieldValue::Enum(Int::<u64>::deserialize(deserializer)?.0),
            6 => FieldValue::String(String::deserialize(deserializer)?),
            7 => { // List
                let json: Value = Value::deserialize(deserializer)?;
                let value_type = json.get("type").expect("type").as_u64().expect("is num");
                let list = json.get("list").expect("list").as_array().expect("is list");
                FieldValue::List(
                    list.iter()
                        .map(|val| deserialize_with_type(val, value_type))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(Error::custom)?)
            },
            8 => { // Lookuplist
                let json: Value = Value::deserialize(deserializer)?;
                let index_type = json.get("indexType").expect("indexType").as_u64().expect("is num");
                let value_type = json.get("type").expect("type").as_u64().expect("is num");
                let list = json.get("list").expect("list").as_array().expect("is list");
                let mut map = vec![];
                for pair in list {
                    let key = pair.get("key").expect("key");
                    let parsed_key = deserialize_with_type(key, index_type).map_err(Error::custom)?;
                    let val = pair.get("val").expect("val");
                    let parsed_val = deserialize_with_type(val, value_type).map_err(Error::custom)?;
                    map.push((parsed_key, parsed_val));
                }
                FieldValue::LookupList(map)
                    // list.iter()
                    //     .map(|pair| {
                    //         match (parsed_key, parsed_val) {
                    //             (Ok(k), Ok(v)) => (k, v),
                    //             _ => todo!(),
                    //         }
                    //     })
                    //     .collect()
            },
            _ => FieldValue::Other(Value::deserialize(deserializer)?),
            // _ => unimplemented!("Need to implement more types"),
        })
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json: Value = Value::deserialize(deserializer)?;
        let id = json.get("id").expect("id").as_str().expect("is str").to_string();
        let value_type = json.get("type").expect("type").as_u64().expect("is num");
        let value_str = json.get("value").expect("value");
        let value = deserialize_with_type(value_str, value_type).map_err(Error::custom)?;
        Ok(Self {
            id,
            value,
        })
    }
}

// TODO: Make into SwtorType or smth
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Id(u64), // type: 1, etc.
    Int(i64),
    Boolean(bool),
    Float(f32),
    Enum(u64),
    String(String),
    List(Vec<FieldValue>),
    LookupList(Vec<(FieldValue, FieldValue)>),
    Other(Value),
    // case 9:
    //   return 'ClassView';
    // case 14:
    //   return 'ScriptRef';
    // case 15:
    //   return 'NodeRef';
    // case 18:
    //   return 'Vector3';
    // case 20:
    //   return 'TimeInterval';
    // case 21:
    //   return 'Date';
    // default:
    //   return 'Unknown (' + type + ')'
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BigInt {
    sign: i8,
    int_lo: u32,
    int_hi: u32,
    #[serde(rename = "len")]
    _len: u8
}

impl From<BigInt> for u64 {
    fn from(value: BigInt) -> Self {
        let (high, overflowed) = (value.int_hi as u64).overflowing_shl(32);
        assert!(value.sign == 1);
        assert!(!overflowed);
        high + (value.int_lo as u64)
    }
}

impl From<BigInt> for i64 {
    fn from(value: BigInt) -> Self {
        let (high, overflowed) = (value.int_hi as i64).overflowing_shl(32);
        assert!(!overflowed);
        (high + (value.int_lo as i64)) * (value.sign as i64)
    }
}

#[derive(Debug, Clone, Copy)]
struct Int<T: IntTrait>(T);

trait IntTrait: From<BigInt> {}

impl IntTrait for u64 {}
impl IntTrait for i64 {}

fn unexpected(value: &Value) -> Unexpected {
    match value {
        Value::Null => Unexpected::Unit,
        Value::Bool(b) => Unexpected::Bool(*b),
        Value::Number(n) => {
            if n.is_f64() {
                Unexpected::Float(n.as_f64().unwrap())
            } else if n.is_i64() {
                Unexpected::Signed(n.as_i64().unwrap())
            } else {
                Unexpected::Unsigned(n.as_u64().unwrap())
            }
        },
        Value::String(s) => Unexpected::Str(s),
        Value::Array(_v) => Unexpected::Seq,
        Value::Object(_m) => Unexpected::Map,
    }
}

impl<'de, T> Deserialize<'de> for Int<T>
where
    T: IntTrait + Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let value = Value::deserialize(deserializer)?;
        Ok(Int(match value {
            Value::Number(_) => T::deserialize(value).map_err(Error::custom)?,
            Value::Object(_) => BigInt::deserialize(value).map_err(Error::custom)?.into(),
            _ => Err(Error::invalid_type(unexpected(&value), &"BigInt or u64"))?,
        }))
    }
}

