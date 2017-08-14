use errors::*;
use {Struct, Enum, NewType, Alias, Id};

use std::collections::BTreeMap;
use std::fmt;

pub(crate) type IdMap = BTreeMap<Id, Box<Item>>;

pub trait Item: fmt::Display {
    fn name(&self) -> &Id;
    fn is_defaultable(&self, &IdMap) -> bool;
    fn contains_unboxed_id(&self, id: &Id, map: &IdMap) -> bool;
    fn get_named_types(&self) -> Vec<&Id>;
    fn is_recursive(&self, map: &IdMap) -> bool {
        self.contains_unboxed_id(self.name(), map)
    }
}

impl Item for Struct {
    fn name(&self) -> &Id {
        &self.name
    }
    fn is_defaultable(&self, map: &IdMap) -> bool {
        self.fields.iter().all(|field| field.is_defaultable(map))
    }
    fn contains_unboxed_id(&self, id: &Id, map: &IdMap) -> bool {
        self.fields.iter().any(|field| field.contains_unboxed_id(id, map))
    }
    fn get_named_types(&self) -> Vec<&Id> {
        self.fields
            .iter()
            .filter_map(|field| field.get_named_type())
            .collect()
    }
}


fn build_id_map(items: Vec<Box<Item>>) -> Result<IdMap> {
    let mut map = BTreeMap::new();
    for item in items {
        let name = item.name().clone();
        if let Some(item) = map.insert(name, item) {
            bail!("None-unique Id: {}", item.name())
        }
    }
    Ok(map)
}

fn find_named_types(map: &IdMap) -> Vec<&Id> {
    map.iter().flat_map(|(id, item)| {
        let mut v = item.get_named_types();
        v.push(id);
        v
    }).collect()
}
