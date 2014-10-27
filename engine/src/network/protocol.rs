use std::collections::HashMap;
use component::{RawComponentHandle, ComponentHandle, ComponentStore};
use super::{Change, ComponentUpdate, Destroy};

pub fn apply_update<Component, MarshalledComponent, UpdatesIter: Iterator<ComponentUpdate<MarshalledComponent>>>(
    mut updates: UpdatesIter,
    hdict: &mut HashMap<RawComponentHandle, ComponentHandle<Component>>,
    store: &mut ComponentStore<Component>,
    unmarshaller: |MarshalledComponent, ComponentHandle<Component>| -> Component,
    inserter: |MarshalledComponent, &mut ComponentStore<Component>| -> ComponentHandle<Component>)
{
    for update in updates {
        match update.data {
            Change(comp) => match hdict.find_copy(&update.target) {
                Some(handle) => {
                    *store.find_mut(handle).unwrap() = unmarshaller(comp, handle);
                },
                None => {
                    hdict.insert(update.target, inserter(comp, store));
                }
            },
            Destroy => match hdict.find_copy(&update.target) {
                Some(handle) => {
                    hdict.remove(&update.target);
                    store.remove(handle);
                },
                None => () // weeeird.
            }
        }
    }
}
