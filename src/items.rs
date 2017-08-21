use errors::*;
use {Struct, Enum, NewType, Alias, Id};

use std::collections::BTreeMap;
use std::fmt;

pub struct ItemMap(BTreeMap<Id, Box<Item>>);

impl ItemMap {
    pub fn build(items: Vec<Box<Item>>) -> Result<ItemMap> {
        let mut map = BTreeMap::new();
        for item in items {
            let name = item.name().clone();
            if let Some(item) = map.insert(name, item) {
                bail!("None-unique Id: {}", item.name())
            }
        }
        Ok(ItemMap(map))
    }

    pub fn get(&self, id: &Id) -> Option<&Box<Item>> {
        self.0.get(id)
    }

    fn find_named_types(&self) -> Vec<&Id> {
        self.0.iter().flat_map(|(id, item)| {
            let mut v = item.get_named_types();
            v.push(id);
            v
        }).collect()
    }
}

pub trait Item: fmt::Display {
    fn name(&self) -> &Id;
    fn is_defaultable(&self, &ItemMap) -> bool;
    fn contains_unboxed_id(&self, id: &Id, map: &ItemMap) -> bool;
    fn get_named_types(&self) -> Vec<&Id>;
    fn is_recursive(&self, map: &ItemMap) -> bool {
        self.contains_unboxed_id(self.name(), map)
    }
}

impl Item for Struct {
    fn name(&self) -> &Id {
        &self.name
    }
    fn is_defaultable(&self, map: &ItemMap) -> bool {
        self.fields.iter().all(|field| field.is_defaultable(map))
    }
    fn contains_unboxed_id(&self, id: &Id, map: &ItemMap) -> bool {
        self.fields.iter().any(|field| field.contains_unboxed_id(id, map))
    }
    fn get_named_types(&self) -> Vec<&Id> {
        self.fields
            .iter()
            .filter_map(|field| field.get_named_type())
            .collect()
    }
}

impl Item for Enum {
    fn name(&self) -> &Id {
        &self.name
    }
    fn is_defaultable(&self, map: &ItemMap) -> bool {
        false
    }
    fn contains_unboxed_id(&self, id: &Id, map: &ItemMap) -> bool {
        self.variants.iter().any(|v| v.contains_unboxed_id(id, map))
    }
    fn get_named_types(&self) -> Vec<&Id> {
        self.variants
            .iter()
            .filter_map(|variant| variant.get_named_type())
            .collect()
    }
}
