use std::{collections::HashMap, hash::Hash};

/// Group a sequence of objects by a key, returning a `HashMap<KeyType, Vec<ObjType>>`.
pub fn group_by<Container, T, Key, KeyFn>(objects: Container, key_fn: KeyFn) -> HashMap<Key, Vec<T>>
where
    Container: IntoIterator<Item = T>,
    Key: Hash + Eq,
    KeyFn: Fn(&T) -> Key,
{
    let mut result = HashMap::new();

    for obj in objects {
        result
            .entry(key_fn(&obj))
            .or_insert_with(Vec::new)
            .push(obj)
    }

    result
}
