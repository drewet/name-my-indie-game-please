use std;
use std::collections::Bitv;

#[deriving(Eq, PartialEq, Ord, PartialOrd, Show)]
pub struct EntityID(u32);

#[deriving(Show)]
/// Type parameter used only for correctness.
pub struct ComponentHandle<Payload> {
    id: u16,
    serial: u16
}
impl<Payload> PartialEq for ComponentHandle<Payload> {
    fn eq(&self, other: &ComponentHandle<Payload>) -> bool {
        self.id == other.id && self.serial == other.serial
    }
}
pub struct Component<Payload> {
    handle: ComponentHandle<Payload>,
    entity: EntityID,
    payload: Payload
}
/*
impl<Payload> PartialEq for ComponentID<Payload> {
    fn eq(&self, other: &ComponentID<Payload>) -> bool {
        let (&ComponentID(thisid), &ComponentID(otherid)) = (self, other);
        thisid == otherid
    }
}   
impl<Payload> Ord for ComponentID<Payload> {
    fn cmp(&self, other: &ComponentID<Payload>) -> Ordering {
        let (&ComponentID(thisid), &ComponentID(otherid)) = (self, other);
        thisid.cmp(&otherid)
    }
}   
impl<Payload> PartialOrd for ComponentID<Payload> {
    fn partial_cmp(&self, other: &ComponentID<Payload>) -> Option<Ordering> {
        let (&ComponentID(thisid), &ComponentID(otherid)) = (self, other);
        thisid.partial_cmp(&otherid)
    }
}
impl<Payload> Eq for ComponentID<Payload> {}
*/

impl<Payload> PartialEq for Component<Payload> {
    fn eq(&self, other: &Component<Payload>) -> bool {
        self.handle == other.handle
    }
}   
impl<Payload> Eq for Component<Payload> {}

impl<Payload> Component<Payload>  {
    fn get_handle(&self) -> ComponentHandle<Payload> { self.handle }
    /// Returns the ID of the entity this component is attached to.
    fn get_entity_id(&self) -> EntityID { self.entity }
}

pub struct ComponentStore<Payload> {
    // TODO: replace Vecs and stuff w/ fixed-size arrays
    components: Vec<Option<Component<Payload>>>,
}

impl<Payload> ComponentStore<Payload> {
    pub fn new() -> ComponentStore<Payload> {
        ComponentStore { components: Vec::from_fn(2048, |_| None) }
    }

    pub fn iter(&self) -> std::iter::FilterMap<&Option<Component<Payload>>, &Component<Payload>, std::slice::Items<Option<Component<Payload>>>> { self.components.iter().filter_map(|comp| comp.as_ref())
    }

    pub fn add_component(&mut self, entity: EntityID, payload: Payload) -> ComponentHandle<Payload> {
        let result = self.components.iter().position(|component| component.is_none());
        match result {
            Some(pos) => {
                let old_serial = match self.components[pos] {
                    Some(ref comp) => comp.handle.serial,
                    None => 0
                };

                let handle = ComponentHandle { id: pos as u16, serial: old_serial.checked_add(&1).unwrap() };
                *self.components.get_mut(pos) = Some(Component {
                    handle: handle,
                    entity: entity,
                    payload: payload
                });
                handle
            }
            None => {
                let handle = ComponentHandle {
                    id: self.components.len() as u16,
                    serial: 0
                };
                self.components.push(Some(Component {
                    handle: handle,
                    entity: entity,
                    payload: payload
                }));
                handle
            }
        }
    }

    pub fn find(&self, handle: ComponentHandle<Payload>) -> Option<&Component<Payload>> {
        self.components[handle.id as uint].iter()
                .filter(|comp| comp.handle.serial == handle.serial).next()
     }
}

pub struct EntityStore {
    ents: Bitv // true = alive
}
impl EntityStore {
    pub fn new() -> EntityStore { EntityStore { ents: Bitv::new() } }
    pub fn create_entity(&mut self) -> EntityID {
        let id = self.ents.len();
        self.ents.push(true);
        assert!(id < std::u32::MAX as uint);
        EntityID(id as u32)
    }
    pub fn is_alive(&self, &EntityID(id): &EntityID) -> bool {
        self.ents.get(id as uint)
    }
    pub fn kill(&mut self, &EntityID(id): &EntityID) {
        self.ents.set(id as uint, false);
    }
}
#[cfg(test)]
mod test {
    use test::Bencher;
    use super::{ComponentHandle, EntityID, Component, ComponentStore, EntityStore};
    struct TestStringComponent {
        name: String
    }
    struct TrivialComponent;

    #[test]
    fn smoke_componentstore() {
        let mut cstore = ComponentStore::new();
        let mut estore = EntityStore::new();
        let ent = estore.create_entity();
        let comp = cstore.add_component(ent, TestStringComponent { name: "Hello, world!".to_string()});
        assert_eq!(cstore.find(comp).unwrap().payload.name.as_slice(), "Hello, world!");
    }

    #[bench]
    fn bench_entcreation_singlecomponent(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        b.iter(|| cstore.add_component(EntityID(0), TrivialComponent));
    }

    #[bench]
    fn bench_componentid_lookup_2048(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let ids = Vec::from_fn(2048, |_| cstore.add_component(EntityID(0), TrivialComponent));
        b.iter(|| cstore.find(ids[ids.len()/2]));
    }

    #[bench]
    fn bench_component_iteration_2048(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let _ = Vec::from_fn(2048, |_| cstore.add_component(EntityID(0), TrivialComponent));
        b.iter(|| for comp in cstore.iter() { ::test::black_box(comp) })
    }

    /*fn bench_gc_2048_50percent(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let mut estore = EntityStore::new();
        let ents = Vec::from_fn(2048, |_| estore.create_entity());
    }*/
}
