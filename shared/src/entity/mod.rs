use std;
use std::collections::Bitv;

#[deriving(Eq, PartialEq, Ord, PartialOrd, Show)]
/// A globally unique entity identifier.
pub struct EntityID(u32);

#[deriving(Show)]
/// A handle to a component.
pub struct ComponentHandle<Payload> {
    id: u16,
    serial: u16
}
impl<Payload> PartialEq for ComponentHandle<Payload> {
    fn eq(&self, other: &ComponentHandle<Payload>) -> bool {
        self.id == other.id && self.serial == other.serial
    }
}

/// A component.
pub struct Component<Payload> {
    handle: ComponentHandle<Payload>,
    entity: EntityID,
    payload: Payload
}

impl<Payload> PartialEq for Component<Payload> {
    fn eq(&self, other: &Component<Payload>) -> bool {
        self.handle == other.handle
    }
}   
impl<Payload> Eq for Component<Payload> {}

impl<Payload> Component<Payload>  {
    pub fn get_handle(&self) -> ComponentHandle<Payload> { self.handle }
    /// Returns the ID of the entity this component is attached to.
    pub fn get_entity_id(&self) -> EntityID { self.entity }
}
impl<Payload> Deref<Payload> for Component<Payload> {
    fn deref(&self) -> &Payload { &self.payload }
}
impl<Payload> DerefMut<Payload> for Component<Payload> {
    fn deref_mut(&mut self) -> &mut Payload { &mut self.payload }
}
/// Stores components.
pub struct ComponentStore<Payload> {
    // TODO: replace Vecs and stuff w/ fixed-size arrays
    components: Vec<Option<Component<Payload>>>,
}

impl<Payload> ComponentStore<Payload> {
    pub fn new() -> ComponentStore<Payload> {
        ComponentStore { components: Vec::from_fn(2048, |_| None) }
    }

    /// Iterate over all components.
    pub fn iter(&self) -> std::iter::FilterMap<&Option<Component<Payload>>, &Component<Payload>, std::slice::Items<Option<Component<Payload>>>> { self.components.iter().filter_map(|comp| comp.as_ref())
    }
    
    /// Add a component (by payload and entity ID.)
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

    /// Removes a component by handle.
    /// Returns whether the component was found.
    pub fn remove(&mut self, handle: ComponentHandle<Payload>) -> bool {
        let found = match self.components[handle.id as uint] {
            Some(ref comp) if comp.handle.serial == handle.serial => { true },
            _ => false
        };

        if found {
            *self.components.get_mut(handle.id as uint) = None;
        }
        found
    }

    /// Gets a reference to a component by handle, or None if the component is not found.
    pub fn find(&self, handle: ComponentHandle<Payload>) -> Option<&Component<Payload>> {
        self.components[handle.id as uint].iter()
            .filter(|comp| comp.handle.serial == handle.serial).next()
    }
    /// Gets a mutable reference to a component by handle, or None if the component is not found.
    pub fn find_mut(&mut self, handle: ComponentHandle<Payload>) -> Option<&mut Component<Payload>> {
        self.components.get_mut(handle.id as uint).iter_mut()
            .filter(|comp| comp.handle.serial == handle.serial).next()
    }
}

/// Manages entity IDs.
/// Does not store any data besides basic bookkeeping.
pub struct EntityStore {
    ents: Bitv // true = alive
}
impl EntityStore {
    pub fn new() -> EntityStore { EntityStore { ents: Bitv::new() } }

    /// Allocates a new entity ID.
    pub fn create_entity(&mut self) -> EntityID {
        let id = self.ents.len();
        self.ents.push(true);
        assert!(id < std::u32::MAX as uint);
        EntityID(id as u32)
    }

    /// Returns whether an entity ID refers to a living entity.
    pub fn is_alive(&self, EntityID(id): EntityID) -> bool {
        self.ents.get(id as uint)
    }
    
    /// This is entirely useless at the moment.
    pub fn kill(&mut self, EntityID(id): EntityID) {
        self.ents.set(id as uint, false);
    }
}
#[cfg(test)]
mod test {
    use test::Bencher;
    use super::{EntityID, ComponentStore, EntityStore};
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

    #[test]
    fn component_removal() { 
        let mut cstore = ComponentStore::new();
        let mut estore = EntityStore::new();
        let ent = estore.create_entity();
        let comp = cstore.add_component(ent, TestStringComponent { name: "Hello, world!".to_string()});
        assert!(cstore.remove(comp));
        
        assert!(cstore.find(comp).is_none());
    }

    #[bench]
    fn bench_componentcreation_32(b: &mut Bencher) {
        b.iter(|| {
            let mut cstore = ComponentStore::new();
            for _ in range(0u, 32) {
                cstore.add_component(EntityID(0), TrivialComponent);
            }
        })
    }

    #[bench]
    fn bench_componentid_lookup(b: &mut Bencher) {
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
