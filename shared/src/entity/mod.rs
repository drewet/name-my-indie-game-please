use std;
use std::collections::Bitv;
pub use self::components::PositionComponent;
pub mod components;

#[deriving(Eq, PartialEq, Ord, PartialOrd, Show)]
/// A globally unique entity identifier.
pub struct EntityID(u32);

#[deriving(Show)]
/// A handle to a component.
pub struct ComponentHandle<Payload> {
    id: u16,
    serial: u32
}
impl<Payload> PartialEq for ComponentHandle<Payload> {
    fn eq(&self, other: &ComponentHandle<Payload>) -> bool {
        self.id == other.id && self.serial == other.serial
    }
}

/// A component.
pub struct Component<Payload> {
    entity: EntityID,
    payload: Payload
}

impl<Payload> Component<Payload>  {
    /// Returns the ID of the entity this component is attached to.
    pub fn get_entity_id(&self) -> EntityID { self.entity }
}
impl<Payload> Deref<Payload> for Component<Payload> {
    fn deref(&self) -> &Payload { &self.payload }
}
impl<Payload> DerefMut<Payload> for Component<Payload> {
    fn deref_mut(&mut self) -> &mut Payload { &mut self.payload }
}

/// This has to be public for iterator reasons.
/// Should fix later.
pub struct ComponentBookkeeper<Payload> {
    serial: u32,
    component: Option<Component<Payload>>
}
impl<Payload> ComponentBookkeeper<Payload> {
    pub fn bump_serial(&mut self) {
        self.serial = self.serial.checked_add(&1).expect("Serial no. overflow!");
    }
}

/// Stores components.
pub struct ComponentStore<Payload> {
    // TODO: replace Vecs and stuff w/ fixed-size arrays
    components: Vec<ComponentBookkeeper<Payload>>
}

impl<Payload> ComponentStore<Payload> {
    pub fn new() -> ComponentStore<Payload> {
        ComponentStore { components: Vec::from_fn(2048, |_| ComponentBookkeeper {
            serial: 0,
            component: None
        })} 
    }

    /// Iterate over all components.
    pub fn iter(&self) -> std::iter::FilterMap<&ComponentBookkeeper<Payload>, &Component<Payload>, std::slice::Items<ComponentBookkeeper<Payload>>> {
        self.components.iter().filter_map(|&ComponentBookkeeper{component: ref comp, ..}| comp.as_ref())
    }
    
    /// Add a component (by payload and entity ID.)
    pub fn add_component(&mut self, entity: EntityID, payload: Payload) -> ComponentHandle<Payload> {
        let result = self.components.iter().position(|bookkeeper| bookkeeper.component.is_none());
        let pos = result.expect("Out of room in ComponentStore!");
        
        let comp = self.components.get_mut(pos);

        // Note: we do NOT bump the serial here.
        // It is only done on component destruction
        // This is because it's impossible to get a handle that points to a bookkeeper
        // without a component inside it.

        comp.component = Some(Component {
            entity: entity,
            payload: payload
        });

        let handle = ComponentHandle { id: pos as u16, serial: comp.serial };

        handle
    }

    /// Removes a component by handle.
    /// Returns whether the component was found.
    pub fn remove(&mut self, handle: ComponentHandle<Payload>) -> bool {
        match self.components.get_mut(handle.id as uint) {
            ref mut bookkeeper if bookkeeper.serial == handle.serial => {
                bookkeeper.bump_serial();
                bookkeeper.component = None;
                true
            },
            _ => false
        }
    }

    /// Gets a reference to a component by handle, or None if the component is not found.
    pub fn find(&self, handle: ComponentHandle<Payload>) -> Option<&Component<Payload>> {
        let bookkeeper = &self.components[handle.id as uint];

        if bookkeeper.serial == handle.serial {
            bookkeeper.component.as_ref()
        } else {
            None
        }
    }
    /// Gets a mutable reference to a component by handle, or None if the component is not found.
    pub fn find_mut(&mut self, handle: ComponentHandle<Payload>) -> Option<&mut Component<Payload>> {
        let bookkeeper = self.components.get_mut(handle.id as uint);

        if bookkeeper.serial == handle.serial {
            bookkeeper.component.as_mut()
        } else {
            None
        }
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
        let comp = cstore.add_component(ent, TrivialComponent);
        assert!(cstore.remove(comp));

        assert!(cstore.find(comp).is_none());
        assert!(cstore.iter().next().is_none());
        assert_eq!(cstore.remove(comp), false);

        // Now make sure serials do their job
        // and prevent an old handle from referencing a new component.
        let new_comp = cstore.add_component(ent, TrivialComponent);
        assert_eq!(new_comp.id, comp.id); // just checking
        assert!(cstore.find(comp).is_none());
    }

    #[test]
    // Make sure the underlying Vec does not grow if there is free space
    fn component_leak_check() { 
        let mut cstore = ComponentStore::new();
        let mut estore = EntityStore::new();
        let ent = estore.create_entity();

        let mut comp = cstore.add_component(ent, TrivialComponent);
        let first_len = cstore.components.len();

        for _ in range(0u, 10000) {
            cstore.remove(comp);
            comp = cstore.add_component(ent, TrivialComponent);
        }

        assert_eq!(cstore.components.len(), first_len);
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
    fn bench_componentcreationandremoval(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        b.iter(|| {
            let handle = cstore.add_component(EntityID(0), TrivialComponent);
            ::test::black_box(&mut cstore);
            cstore.remove(handle);
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

    #[bench]
    fn bench_component_iteration_1024_fragmented(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let ids = Vec::from_fn(2048, |_| cstore.add_component(EntityID(0), TrivialComponent));
        for x in ids.iter().enumerate().filter_map(|(pos, &id)| if pos%2 == 0 { Some(id) } else { None }) {
            cstore.remove(x);
        }

        b.iter(|| for comp in cstore.iter() { ::test::black_box(comp) })
    }

    /*fn bench_gc_2048_50percent(b: &mut Bencher) {
      let mut cstore = ComponentStore::new();
      let mut estore = EntityStore::new();
      let ents = Vec::from_fn(2048, |_| estore.create_entity());
      }*/
}
