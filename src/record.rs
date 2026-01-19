use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Transformation options for extracted records.
#[derive(Debug, Clone)]
pub enum DataRecordConversion {
    /// Convert all field names to lowercase.
    LowercaseKeys,
}

/// Represents a single row of extracted data from a TextFSM template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DataRecord {
    /// Map of value names to their extracted values.
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
    /// An optional key used to identify the record, constructed from fields marked as 'Key'.
    #[serde(skip_deserializing)]
    pub record_key: Option<String>,
}

impl DataRecord {
    /// Creates a new, empty `DataRecord`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Overwrites existing fields in this record with fields from another record.
    pub fn overwrite_from(&mut self, from: DataRecord) {
        for (k, v) in from.fields {
            self.fields.insert(k, v);
        }
    }

    /// Compares two sets of records and returns differences.
    /// Returns a tuple of (fields only in result, fields only in other).
    pub fn compare_sets(result: &[Self], other: &[Self]) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
        let mut only_in_result: Vec<Vec<String>> = vec![];
        let mut only_in_other: Vec<Vec<String>> = vec![];

        for (i, irec) in result.iter().enumerate() {
            let mut vo: Vec<String> = vec![];
            for (k, v) in &irec.fields {
                if i < other.len() {
                    let v0 = other[i].get(k);
                    if v0.is_none() || v0.unwrap() != v {
                        vo.push(format!("{}:{:?}", &k, &v));
                    }
                } else {
                    vo.push(format!("{}:{:?}", &k, &v));
                }
            }
            only_in_result.push(vo);
        }

        for (i, irec) in other.iter().enumerate() {
            let mut vo: Vec<String> = vec![];
            for (k, v) in &irec.fields {
                if i < result.len() {
                    let v0 = result[i].get(k);
                    if v0.is_none() || v0.unwrap() != v {
                        vo.push(format!("{}:{:?}", &k, &v));
                    }
                } else {
                    vo.push(format!("{}:{:?}", &k, &v));
                }
            }
            only_in_other.push(vo);
        }
        (only_in_result, only_in_other)
    }

    /// Inserts a single string value into the record.
    /// If the key already exists, it converts the value to a list or appends to it.
    pub fn insert(&mut self, name: String, value: String) {
        use std::collections::hash_map::Entry;
        match self.fields.entry(name) {
            Entry::Occupied(mut entry) => {
                let old_value = entry.get_mut();
                if let Value::Single(old_str) = old_value {
                    let s = std::mem::take(old_str);
                    *old_value = Value::List(vec![s, value]);
                } else if let Value::List(list) = old_value {
                    list.push(value);
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(Value::Single(value));
            }
        }
    }

    /// Appends a `Value` to the record.
    pub fn append_value(&mut self, name: String, value: Value) {
        if let Some(old_value) = self.fields.get_mut(&name) {
            match old_value {
                Value::Single(old_str_ref) => match value {
                    Value::Single(val) => {
                        *old_value = Value::Single(val);
                    }
                    Value::List(lst) => {
                        panic!(
                            "can not append list {:?} to single {:?} in var {}",
                            &lst, &old_str_ref, &name
                        );
                    }
                },
                Value::List(list) => match value {
                    Value::Single(val) => {
                        list.push(val);
                    }
                    Value::List(mut lst) => {
                        list.append(&mut lst);
                    }
                },
            }
        } else {
            self.fields.insert(name, value);
        }
    }

    /// Removes a field from the record.
    pub fn remove(&mut self, key: &str) {
        self.fields.remove(key);
    }

    /// Returns an iterator over the field names.
    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, String, Value> {
        self.fields.keys()
    }

    /// Retrieves a reference to a field's value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }

    /// Returns an iterator over the record's fields.
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Value> {
        self.fields.iter()
    }
}

/// Represents an extracted value, which can be either a single string or a list of strings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    /// A single extracted string.
    Single(String),
    /// A list of extracted strings (used for fields with 'List' option).
    List(Vec<String>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Single(s) => write!(f, "{}", s),
            Value::List(l) => write!(f, "{:?}", l),
        }
    }
}