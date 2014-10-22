use serialize::{Decoder, Decodable, Encoder, Encodable};
use std::collections::HashMap;
use component::{RawComponentHandle, ComponentHandle, ComponentStore};

#[deriving(Encodable, Decodable)]
pub struct ComponentUpdate<MarshalledComponent> {
    target: RawComponentHandle,
    data: MarshalledComponent
}

pub fn encode_full_update<E, S: Encoder<E>, Component, MarshalledComponent: Encodable<S, E>>(encoder: &mut S,
                                                                        store: &ComponentStore<Component>,
                                                                        marshaller: |&Component| -> MarshalledComponent)
        -> Result<(), E> {
    let mut updates = Vec::new();
    for (handle, component) in store.iter() {
        updates.push(
            ComponentUpdate {
                target: handle.to_raw(),
                data: marshaller(component)
            }
        );
    }
    updates.encode(encoder)
}
pub fn apply_full_update<E, D: Decoder<E>, Component, MarshalledComponent: Decodable<D, E> + Copy>(decoder: &mut D,
                                                                              hdict: &mut HashMap<RawComponentHandle, ComponentHandle<Component>>,
                                                                              store: &mut ComponentStore<Component>,
                                                                              unmarshaller: |MarshalledComponent, ComponentHandle<Component>| -> Component,
                                                                              inserter: |MarshalledComponent, &mut ComponentStore<Component>| -> ComponentHandle<Component>)
{
    let updates: Result<Vec<ComponentUpdate<MarshalledComponent>>, E> = Decodable::decode(decoder);
    updates.map(|updates| for update in updates.into_iter() {
        match hdict.find_copy(&update.target) {
            Some(handle) => {
                *store.find_mut(handle).unwrap() = unmarshaller(update.data, handle);
            },
            None => {
                hdict.insert(update.target, inserter(update.data, store));
            }
        }
    });
}
