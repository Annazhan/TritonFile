use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ListOp {
    Append,
    Remove,
    Clear,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OpKind {
    KeyString,
    KeyList(ListOp),
}

// Key-value operations recorded in the backend's log.
#[derive(Serialize, Deserialize, Debug)]
pub struct LogOp {
    pub val: String,
    pub clock: u64,
    pub kind: OpKind,
}

// Union b into a.
pub fn union<T: Ord>(a: &mut Vec<T>, b: &mut Vec<T>) {
    a.append(b);
    a.sort();
    a.dedup();
}

// Subtract b from a.
pub fn subtract<T: Ord>(a: Vec<T>, b: Vec<T>) -> Vec<T> {
    let mut result = vec![];
    for elm in a {
        if !b.contains(&elm) {
            result.push(elm);
        }
    }
    result
}
