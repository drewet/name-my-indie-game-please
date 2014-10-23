use std::collections::{Deque, HashMap, RingBuf};
use component::{RawComponentHandle, ComponentHandle, ComponentStore};
use super::{ComponentUpdate, Change, Destroy};

pub struct DeltaEncoder<Component, MarshalledComponent> {
    states: RingBuf<HashMap<RawComponentHandle, MarshalledComponent>>,
    max_states: uint
}

impl<Component, MarshalledComponent: PartialEq + Clone> DeltaEncoder<Component, MarshalledComponent> {
    pub fn new(max_states: uint) -> DeltaEncoder<Component, MarshalledComponent> {
        DeltaEncoder {
            states: RingBuf::with_capacity(max_states),
            max_states: max_states
        }
    }

    pub fn add_state(&mut self, components: &ComponentStore<Component>,
                    marshaller: |&Component| -> MarshalledComponent ) {
        let mut state = HashMap::new(); // FIXME: with_capacity

        for (handle, component) in components.iter() {
            let marshalled = marshaller(component);
            state.insert(handle.to_raw(), marshalled);
        }

        self.states.push_front(state);

        while self.states.len() > self.max_states {
            self.states.pop();
        }
    }

    fn create_full_update(&self) -> Vec<ComponentUpdate<MarshalledComponent>> {
        println!("Creating full update...");
        let mut updates = Vec::new();
        for (handle, comp) in self.states[0].iter() {
            updates.push(ComponentUpdate {
                target: *handle,
                data: Change(comp.clone())
            });
        }
        updates
    }

    /// Length = number of ticks to cover
    /// e.g. length of 1 is delta between current and previous state
    pub fn create_delta(&self, length: u64) -> Vec<ComponentUpdate<MarshalledComponent>> {
        assert!(length > 0);

        if length >= self.states.len() as u64 {
            return self.create_full_update();
        };

        // FIXME: should this be a hashmap? seems expensive. lots of alloc
        // in this function.
        let mut updates = HashMap::new();

        // borrowck hates iterators
        // remember indices go newest to oldest,
        // so we reverse here.
        for state_idx in range(0u, length as uint - 1).rev() {
            let ref curr_state = self.states[state_idx];
            let ref prev_state = self.states[state_idx + 1];

            for (handle, comp) in curr_state.iter() {
                let has_changed = match prev_state.find(handle) {
                    Some(prev_comp) => prev_comp != comp,
                    None => true
                };
                if has_changed {
                    updates.insert(handle, Change(comp.clone()));
                };
            }
            // removals aren't covered in the previous loop,
            // so we have to go through here. this sucks.
            for (handle, comp) in prev_state.iter().filter(|&(handle, _)| curr_state.find(handle).is_none()) {
                updates.insert(handle, Destroy);
            }
        }

        updates.into_iter().map(|(&k, v)| ComponentUpdate { target: k, data: v }).collect()
    }
}
