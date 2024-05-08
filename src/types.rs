use serde::{Deserialize, Deserializer, de::{Error, Unexpected}};
use serde_json::Value;
use bevy::prelude::*;

include!(concat!(env!("OUT_DIR"), "/enums.rs"));

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BigInt {
    sign: i8,
    int_lo: u32,
    int_hi: u32,
    #[serde(rename = "len")]
    _len: u8
}

impl TryFrom<BigInt> for u64 {
    type Error = String;

    // Can't overflow since hi is u32. If it is too big it will overflow when that is parsed
    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        let high = (value.int_hi as u64) << 32;
        if value.sign != 1 { return Err("BigInt was negative".to_string()) }
        Ok(high + (value.int_lo as u64))
    }
}

impl TryFrom<BigInt> for i64 {
    type Error = String;

    fn try_from(value: BigInt) -> Result<Self, Self::Error> {
        if value.int_hi >> 31 != 0 { return Err("BigInt overflowed".to_string()) }
        let high = (value.int_hi as i64) << 32;
        Ok((high + (value.int_lo as i64)) * (value.sign as i64))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Int(i64);

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

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let value = Value::deserialize(deserializer)?;
        Ok(Self(match value {
            Value::Number(_) => u64::deserialize(value).map_err(Error::custom)?,
            Value::Object(_) => BigInt::deserialize(value).map_err(Error::custom)?.try_into().map_err(Error::custom)?,
            _ => Err(Error::invalid_type(unexpected(&value), &"BigInt or u64"))?,
        }))
    }
}

impl<'de> Deserialize<'de> for Int {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let value = Value::deserialize(deserializer)?;
        Ok(Self(match value {
            Value::Number(_) => i64::deserialize(value).map_err(Error::custom)?,
            Value::Object(_) => BigInt::deserialize(value).map_err(Error::custom)?.try_into().map_err(Error::custom)?,
            _ => Err(Error::invalid_type(unexpected(&value), &"BigInt or u64"))?,
        }))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct List(Vec<FieldValue>);

fn get_key<'a, E: Error>(value: &'a Value, key: &str) -> Result<&'a Value, E> {
    value.get(key).ok_or(Error::invalid_type(unexpected(value), &"map with key"))
}

fn value_to_u64<E: Error>(value: &Value, key: &str) -> Result<u64, E> {
    let inner = get_key(value, key)?;
    inner.as_u64().ok_or(Error::invalid_type(unexpected(&inner), &"u64"))
}

fn value_to_array<'a, E: Error>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, E> {
    let inner = get_key(value, key)?;
    inner.as_array().ok_or(Error::invalid_type(unexpected(&inner), &"array"))
}

impl<'de> Deserialize<'de> for List {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let json: Value = Value::deserialize(deserializer)?;
        let value_type = value_to_u64(&json, &"type")?;
        let list = value_to_array(&json, &"list")?;
        Ok(List(
            list.iter()
                .map(|val| deserialize_with_type(val, value_type))
                .collect::<Result<Vec<_>, _>>()
                .map_err(Error::custom)?))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LookupList(Vec<(FieldValue, FieldValue)>);

impl<'de> Deserialize<'de> for LookupList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let json: Value = Value::deserialize(deserializer)?;
        let index_type = value_to_u64(&json, &"indexType")?;
        let value_type = value_to_u64(&json, &"type")?;
        let list = value_to_array(&json, &"list")?;
        let mut map = vec![];
        for pair in list {
            let key = get_key(pair, &"key")?;
            let parsed_key = deserialize_with_type(key, index_type).map_err(Error::custom)?;
            let val = get_key(pair, &"val")?;
            let parsed_val = deserialize_with_type(val, value_type).map_err(Error::custom)?;
            map.push((parsed_key, parsed_val));
        }
        Ok(LookupList(map))
    }
}

fn deserialize_with_type<'de, D>(deserializer: D, value_type: u64) -> Result<FieldValue, D::Error>
where
    D: Deserializer<'de>
{
        Ok(match value_type {
            1 => FieldValue::Id(Id::deserialize(deserializer)?),
            2 => FieldValue::Int(Int::deserialize(deserializer)?),
            3 => FieldValue::Boolean(bool::deserialize(deserializer)?),
            4 => FieldValue::Float(f32::deserialize(deserializer)?),
            5 => FieldValue::Enum(Id::deserialize(deserializer)?),
            6 => FieldValue::String(String::deserialize(deserializer)?),
            7 => FieldValue::List(List::deserialize(deserializer)?),
            8 => FieldValue::LookupList(LookupList::deserialize(deserializer)?),
            _ => FieldValue::Other(Value::deserialize(deserializer)?),
            // _ => unimplemented!("Need to implement more types"),
        })
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Id(Id), // type: 1, etc.
    Int(Int),
    Boolean(bool),
    Float(f32),
    Enum(Id),
    String(String),
    List(List),
    LookupList(LookupList),
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

impl<'de> Deserialize<'de> for FieldValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let json: Value = Value::deserialize(deserializer)?;
        let value_type = json.get("type").expect("type").as_u64().expect("is num");
        let value_str = json.get("value").expect("value");
        deserialize_with_type(value_str, value_type).map_err(Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Error;

    // Add test for negative id, over- and underflow id/int

    #[test]
    fn deserialize_id() -> Result<(), Error> {
        let data = r#"{"id":"123","type":1,"value":0}"#;
        let expected = FieldValue::Id(Id(0));
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":1,"value":{"sign":1,"intLo":0,"intHi":0,"len":1}}"#;
        let expected = FieldValue::Id(Id(0));
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":1,"value":1000}"#;
        let expected = FieldValue::Id(Id(1000));
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":1,"value":{"sign":1,"intLo":30,"intHi":0,"len":1}}"#;
        let expected = FieldValue::Id(Id(30));
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"4611686061870631196","type":1,"value":{"sign":1,"intLo":2807709627,"intHi":3758157504,"len":9}}"#;
        let expected = FieldValue::Id(Id(16141163575704698811));
        assert_eq!(expected, serde_json::from_str(data)?);

        // Negative
        let data = r#"{"id":"123","type":1,"value":-1000}"#;
        assert!(serde_json::from_str::<FieldValue>(data).is_err());

        let data = r#"{"id":"123","type":1,"value":{"sign":-1,"intLo":30,"intHi":0,"len":1}}"#;
        assert!(serde_json::from_str::<FieldValue>(data).is_err());

        Ok(())
    }

    #[test]
    fn deserialize_int() -> Result<(), Error> {
        let data = r#"{"id":"123","type":2,"value":0}"#;
        let expected = FieldValue::Int(Int(0));
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":2,"value":{"sign":1,"intLo":0,"intHi":0,"len":1}}"#;
        let expected = FieldValue::Int(Int(0));
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":2,"value":1000}"#;
        let expected = FieldValue::Int(Int(1000));
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":2,"value":{"sign":1,"intLo":30,"intHi":0,"len":1}}"#;
        let expected = FieldValue::Int(Int(30));
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":2,"value":-1000}"#;
        let expected = FieldValue::Int(Int(-1000));
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":2,"value":{"sign":-1,"intLo":30,"intHi":0,"len":1}}"#;
        let expected = FieldValue::Int(Int(-30));
        assert_eq!(expected, serde_json::from_str(data)?);

        // Overflow
        let data = r#"{"id":"4611686061870631196","type":2,"value":{"sign":1,"intLo":2807709627,"intHi":3758157504,"len":9}}"#;
        assert!(serde_json::from_str::<FieldValue>(data).is_err());

        Ok(())
    }

    #[test]
    fn deserialize_bool() -> Result<(), Error> {
        let data = r#"{"id":"123","type":3,"value":true}"#;
        let expected = FieldValue::Boolean(true);
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":3,"value":false}"#;
        let expected = FieldValue::Boolean(false);
        assert_eq!(expected, serde_json::from_str(data)?);

        Ok(())
    }

    #[test]
    fn deserialize_float() -> Result<(), Error> {
        let data = r#"{"id":"123","type":4,"value":-1}"#;
        let expected = FieldValue::Float(-1f32);
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":4,"value":0}"#;
        let expected = FieldValue::Float(0f32);
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"123","type":4,"value":270}"#;
        let expected = FieldValue::Float(270f32);
        assert_eq!(expected, serde_json::from_str(data)?);

        let data = r#"{"id":"4611686019453829664","type":4,"value":0.4000000059604645}"#;
        let expected = FieldValue::Float(0.4f32);
        assert_eq!(expected, serde_json::from_str(data)?);

        Ok(())
    }

    // It's the same as id atm
    // #[test]
    // fn deserialize_enum() -> Result<(), Error> { Ok(()) }

    #[test]
    fn deserialize_string() -> Result<(), Error> {
        let data = r#"{"id":"4611686019453829629","type":6,"value": "viciousslash"}"#;
        let expected = FieldValue::String("viciousslash".to_string());
        assert_eq!(expected, serde_json::from_str(data)?);

        Ok(())
    }

    #[test]
    fn deserialize_list() -> Result<(), Error> {
        let data = r#"{
          "id": "4611686061870631192",
          "type": 7,
          "value": {
            "type": 1,
            "list": [
              {
                "sign": 1,
                "intLo": 2385807014,
                "intHi": 3758149514,
                "len": 9
              },
              {
                "sign": 1,
                "intLo": 2385806453,
                "intHi": 3758149258,
                "len": 9
              },
              {
                "sign": 1,
                "intLo": 2385808156,
                "intHi": 3758150026,
                "len": 9
              }
            ]
          }
        }"#;
        let expected = FieldValue::List(List(vec![
            FieldValue::Id(Id(16141129258494101158)),
            FieldValue::Id(Id(16141128158982472821)),
            FieldValue::Id(Id(16141131457517357852))
        ]));
        assert_eq!(expected, serde_json::from_str(data)?);

        Ok(())
    }

    #[test]
    fn deserialize_lookuplist() -> Result<(), Error> {
        let data = r#"{
            "id": "4611686310422994002",
            "type": 8,
            "value": {
                "indexType": 1,
                "type": 2,
                "list": [
                {
                    "key": {
                        "sign": 1,
                        "intLo": 1195329202,
                        "intHi": 3758117139,
                        "len": 9
                    },
                    "val": {
                        "sign": -1,
                        "intLo": 2873223640,
                        "intHi": 695039895,
                        "len": 9
                    }
                }
                ]
            }
        }"#;
        let expected = FieldValue::LookupList(LookupList(vec![(FieldValue::Id(Id(16140990207737415346)), FieldValue::Int(Int(-2985173621313497560)))]));
        assert_eq!(expected, serde_json::from_str(data)?);

        Ok(())
    }

    // Unimplemented
    // #[test]
    // fn deserialize_classview() -> Result<(), Error> { Ok(()) }
}
