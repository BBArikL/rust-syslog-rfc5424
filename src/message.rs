//! In-memory representation of a single Syslog message.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::convert::{Into, TryFrom};
use std::ops;
use std::str::FromStr;
use std::string::String;

#[cfg(feature = "serde-serialize")]
use serde::{de::Visitor, Deserialize, Serialize, Serializer};

#[allow(non_camel_case_types)]
pub type time_t = i64;
#[allow(non_camel_case_types)]
pub type pid_t = i32;
#[allow(non_camel_case_types)]
pub type msgid_t = String;

use crate::facility;
use crate::parser;
use crate::severity;

#[derive(Clone, Debug, PartialEq, Eq)]
/// `ProcID`s are usually numeric PIDs; however, on some systems, they may be something else
pub enum ProcId {
    PID(pid_t),
    Name(String),
}

impl PartialOrd for ProcId {
    fn partial_cmp(&self, other: &ProcId) -> Option<Ordering> {
        match (self, other) {
            (&ProcId::PID(ref s_p), &ProcId::PID(ref o_p)) => Some(s_p.cmp(o_p)),
            (&ProcId::Name(ref s_n), &ProcId::Name(ref o_n)) => Some(s_n.cmp(o_n)),
            _ => None,
        }
    }
}

#[cfg(feature = "serde-serialize")]
impl Serialize for ProcId {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match *self {
            ProcId::PID(ref p) => ser.serialize_i32(*p),
            ProcId::Name(ref n) => ser.serialize_str(n),
        }
    }
}

#[cfg(feature = "serde-serialize")]
struct ProcIDVisitor;

#[cfg(feature = "serde-serialize")]
impl<'de> Visitor<'de> for ProcIDVisitor {
    type Value = ProcId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an i32 or a String")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ProcId::Name(v.to_string()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ProcId::Name(v.to_string()))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ProcId::PID(i32::from(v)))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ProcId::PID(i32::from(v)))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ProcId::PID(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let v = i32::try_from(v).map_err(|_| Err(E::custom(format!("i32 out of range: {}", v))));
        if v.is_ok() {
            Ok(ProcId::PID(v.unwrap()))
        } else {
            v.err().unwrap()
        }
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ProcId::PID(i32::from(v)))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ProcId::PID(i32::from(v)))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let v = i32::try_from(v).map_err(|_| Err(E::custom(format!("i32 out of range: {}", v))));
        if v.is_ok() {
            Ok(ProcId::PID(v.unwrap()))
        } else {
            v.err().unwrap()
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let v = i32::try_from(v).map_err(|_| Err(E::custom(format!("i32 out of range: {}", v))));
        if v.is_ok() {
            Ok(ProcId::PID(v.unwrap()))
        } else {
            v.err().unwrap()
        }
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let v = i32::try_from(v).map_err(|_| Err(E::custom(format!("i32 out of range: {}", v))));
        if v.is_ok() {
            Ok(ProcId::PID(v.unwrap()))
        } else {
            v.err().unwrap()
        }
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let v = i32::try_from(v).map_err(|_| Err(E::custom(format!("i32 out of range: {}", v))));
        if v.is_ok() {
            Ok(ProcId::PID(v.unwrap()))
        } else {
            v.err().unwrap()
        }
    }
}

#[cfg(feature = "serde-serialize")]
impl<'de> Deserialize<'de> for ProcId {
    fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        des.deserialize_any(ProcIDVisitor)
    }
}

pub type SDIDType = String;
pub type SDParamIDType = String;
pub type SDParamValueType = String;

pub type StructuredDataElement = BTreeMap<SDParamIDType, SDParamValueType>;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Container for the `StructuredData` component of a syslog message.
///
/// This is a map from `SD_ID` to pairs of `SD_ParamID`, `SD_ParamValue`
///
/// The spec does not forbid repeated keys. However, for convenience, we *do* forbid repeated keys.
/// That is to say, if you have a message like
///
/// [foo bar="baz" bar="bing"]
///
/// There's no way to retrieve the original "baz" mapping.
pub struct StructuredData {
    elements: BTreeMap<SDIDType, StructuredDataElement>,
}

impl ops::Deref for StructuredData {
    type Target = BTreeMap<SDIDType, StructuredDataElement>;
    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

#[cfg(feature = "serde-serialize")]
impl Serialize for StructuredData {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.elements.serialize(ser)
    }
}

#[cfg(feature = "serde-serialize")]
struct BtreeMapVisitor;

#[cfg(feature = "serde-serialize")]
impl<'de> Visitor<'de> for BtreeMapVisitor {
    type Value = BTreeMap<SDIDType, StructuredDataElement>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut btree = BTreeMap::new();

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = map.next_entry()? {
            btree.insert(key, value);
        }

        Ok(btree)
    }
}

#[cfg(feature = "serde-serialize")]
impl<'de> Deserialize<'de> for StructuredData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let elements = deserializer.deserialize_map(BtreeMapVisitor).unwrap();
        Ok(Self { elements })
    }
}

impl StructuredData {
    pub fn new_empty() -> Self {
        StructuredData {
            elements: BTreeMap::new(),
        }
    }

    /// Fetch or insert a new sd_id entry into the StructuredData
    pub fn entry<SI>(&mut self, sd_id: SI) -> &mut BTreeMap<String, String>
    where
        SI: Into<SDIDType>,
    {
        self.elements
            .entry(sd_id.into())
            .or_insert_with(BTreeMap::new)
    }

    /// Insert a new (sd_id, sd_param_id) -> sd_value mapping into the StructuredData
    pub fn insert_tuple<SI, SPI, SPV>(&mut self, sd_id: SI, sd_param_id: SPI, sd_param_value: SPV)
    where
        SI: Into<SDIDType>,
        SPI: Into<SDParamIDType>,
        SPV: Into<SDParamValueType>,
    {
        self.entry(sd_id)
            .insert(sd_param_id.into(), sd_param_value.into());
    }

    /// Lookup by SDID, SDParamID pair
    pub fn find_tuple<'b>(
        &'b self,
        sd_id: &str,
        sd_param_id: &str,
    ) -> Option<&'b SDParamValueType> {
        // TODO: use traits to make these based on the public types instead of &str
        if let Some(sub_map) = self.elements.get(sd_id) {
            if let Some(value) = sub_map.get(sd_param_id) {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Find all param/value mappings for a given SDID
    pub fn find_sdid<'b>(&'b self, sd_id: &str) -> Option<&'b StructuredDataElement> {
        self.elements.get(sd_id)
    }

    /// The number of distinct SD_IDs
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Whether or not this is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
/// A RFC5424-protocol syslog message
pub struct SyslogMessage {
    pub severity: severity::SyslogSeverity,
    pub facility: facility::SyslogFacility,
    pub version: i32,
    pub timestamp: Option<time_t>,
    pub timestamp_nanos: Option<u32>,
    pub hostname: Option<String>,
    pub appname: Option<String>,
    pub procid: Option<ProcId>,
    pub msgid: Option<msgid_t>,
    pub sd: StructuredData,
    pub msg: String,
}

impl FromStr for SyslogMessage {
    type Err = parser::ParseErr;

    /// Parse a string into a `SyslogMessage`
    ///
    /// Just calls `parser::parse_message`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse_message(s)
    }
}

#[cfg(test)]
mod tests {
    use super::StructuredData;
    use super::SyslogMessage;
    #[cfg(feature = "serde-serialize")]
    use crate::facility::SyslogFacility::*;
    #[cfg(feature = "serde-serialize")]
    use crate::severity::SyslogSeverity::*;
    #[cfg(feature = "serde-serialize")]
    use serde_json;

    #[test]
    fn test_structured_data_basic() {
        let mut s = StructuredData::new_empty();
        s.insert_tuple("foo", "bar", "baz");
        let v = s.find_tuple("foo", "bar").expect("should find foo/bar");
        assert_eq!(v, "baz");
        assert!(s.find_tuple("foo", "baz").is_none());
    }

    #[cfg(feature = "serde-serialize")]
    #[test]
    fn test_structured_data_serialization_serde() {
        let mut s = StructuredData::new_empty();
        s.insert_tuple("foo", "bar", "baz");
        s.insert_tuple("foo", "baz", "bar");
        s.insert_tuple("faa", "bar", "baz");
        let encoded = serde_json::to_string(&s).expect("Should encode to JSON");
        assert_eq!(
            encoded,
            r#"{"faa":{"bar":"baz"},"foo":{"bar":"baz","baz":"bar"}}"#
        );
    }

    #[cfg(feature = "serde-serialize")]
    #[test]
    fn test_serialization_serde() {
        let m = SyslogMessage {
            severity: SEV_INFO,
            facility: LOG_KERN,
            version: 1,
            timestamp: None,
            timestamp_nanos: None,
            hostname: None,
            appname: None,
            procid: None,
            msgid: None,
            sd: StructuredData::new_empty(),
            msg: String::from(""),
        };

        let encoded = serde_json::to_string(&m).expect("Should encode to JSON");
        // XXX: we don't have a guaranteed order, I don't think, so this might break with minor
        // version changes. *shrug*
        assert_eq!(encoded,
                   "{\"severity\":\"info\",\"facility\":\"kern\",\"version\":1,\"timestamp\":null,\"timestamp_nanos\":null,\"hostname\":null,\"appname\":null,\"procid\":null,\"msgid\":null,\"sd\":{},\"msg\":\"\"}");
    }

    #[test]
    fn test_deref_structureddata() {
        let mut s = StructuredData::new_empty();
        s.insert_tuple("foo", "bar", "baz");
        s.insert_tuple("foo", "baz", "bar");
        s.insert_tuple("faa", "bar", "baz");
        assert_eq!("baz", s.get("foo").and_then(|foo| foo.get("bar")).unwrap());
        assert_eq!("bar", s.get("foo").and_then(|foo| foo.get("baz")).unwrap());
        assert_eq!("baz", s.get("faa").and_then(|foo| foo.get("bar")).unwrap());
    }

    #[test]
    fn test_fromstr() {
        let msg = "<1>1 1985-04-12T23:20:50.52Z host - - - -"
            .parse::<SyslogMessage>()
            .expect("Should parse empty message");
        assert_eq!(msg.timestamp, Some(482196050));
    }
}
