use std;
use std::hash::Hash;
use serialize::{Encodable, Decoder, Encoder};

pub use self::components::{EntityComponent, EntityHandle};
pub mod components;

/// A handle to a component.
pub struct ComponentHandle<Component> {
    id: u16,
    serial: u32
}
#[deriving(Encodable, Decodable, Hash, PartialEq, Eq, Clone)]
pub struct RawComponentHandle {
    id: u16,
    serial: u32
}
impl<Component> ComponentHandle<Component> {
    pub fn cast<NewComponent>(self) -> ComponentHandle<NewComponent> {
        ComponentHandle { id: self.id, serial: self.serial }
    }
    pub fn to_raw(self) -> RawComponentHandle {
        RawComponentHandle { id: self.id, serial: self.serial }
    }
}
impl<Component> PartialEq for ComponentHandle<Component> {
    fn eq(&self, other: &ComponentHandle<Component>) -> bool {
        self.id == other.id && self.serial == other.serial
    }
}
impl<Component> Eq for ComponentHandle<Component> {}
impl<Component> Clone for ComponentHandle<Component> {
    fn clone(&self) -> ComponentHandle<Component> {
        ComponentHandle { id: self.id, serial: self.serial }
    }
}

impl<Component> Hash<std::hash::sip::SipState> for ComponentHandle<Component> {
    fn hash(&self, state: &mut std::hash::sip::SipState) {
        self.id.hash(state);
        self.serial.hash(state);
    }
}

#[deriving(Clone)]
struct ComponentBookkeeper<Component> {
    serial: u32,
    component: Option<Component>
}
impl<Component> ComponentBookkeeper<Component> {
    pub fn bump_serial(&mut self) {
        self.serial = self.serial.checked_add(&1).expect("Serial no. overflow!");
    }
}

pub struct Components<'a, Component: 'a> {
    actual_iterator: std::iter::FilterMap<'a,(uint,&'a ComponentBookkeeper<Component>),(ComponentHandle<Component>, &'a Component),std::iter::Enumerate<std::slice::Items<'a,ComponentBookkeeper<Component>>>>
}
impl<'a, Component> Iterator<(ComponentHandle<Component>, &'a Component)> for Components<'a, Component> {
    fn next(&mut self) -> Option<(ComponentHandle<Component>, &'a Component)> {
        self.actual_iterator.next()
    }
}

pub struct MutComponents<'a, Component: 'a> {
    actual_iterator: std::iter::FilterMap<'a,(uint,&'a mut ComponentBookkeeper<Component>),(ComponentHandle<Component>, &'a mut Component),std::iter::Enumerate<std::slice::MutItems<'a,ComponentBookkeeper<Component>>>>
}
impl<'a, Component> Iterator<(ComponentHandle<Component>, &'a mut Component)> for MutComponents<'a, Component> {
    fn next(&mut self) -> Option<(ComponentHandle<Component>, &'a mut Component)> {
        self.actual_iterator.next()
    }
}

/// Stores components.
pub struct ComponentStore<Component> {
    // TODO: replace Vecs and stuff w/ fixed-size arrays
    components: Vec<ComponentBookkeeper<Component>>
}

impl<Component> ComponentStore<Component> {
    pub fn new() -> ComponentStore<Component> {
        ComponentStore { components: Vec::from_fn(2048, |_| ComponentBookkeeper {
            serial: 0,
            component: None
        })} 
    }

    /// Iterate over all components.
    pub fn iter(&self) -> Components<Component> {
        Components {
            actual_iterator:
                self.components.iter().enumerate()
                .filter_map(|(pos, &ComponentBookkeeper{component: ref comp, serial: serial})|
                            match *comp {
                                Some(ref c) => Some((ComponentHandle { id: pos as u16, serial: serial }, c)),
                                None => None
                            }
                )
        }
    }
    /// Iterate over all components, mutably.
    pub fn iter_mut(&mut self) -> MutComponents<Component> {
        MutComponents {
            actual_iterator:
                self.components.iter_mut().enumerate()
                .filter_map(|(pos, &ComponentBookkeeper{component: ref mut comp, serial: serial})|
                            match *comp {
                                Some(ref mut c) => Some((ComponentHandle { id: pos as u16, serial: serial }, c)),
                                None => None
                            }
                )
        }
    }

    /// Add a component.
    pub fn add(&mut self, component: Component) -> ComponentHandle<Component> {
        let pos = self.find_free_bookkeeper();
        let comp = self.components.get_mut(pos);

        // Note: we do NOT bump the serial here.
        // It is only done on component destruction
        // This is because it's impossible to get a handle that points to a bookkeeper
        // without a component inside it.

        comp.component = Some(component);
        ComponentHandle { id: pos as u16, serial: comp.serial }
    }
    
    /// Adds a component, constructing it by giving it a handle
    /// to itself.
    pub fn add_with_handle(&mut self,
                           constructor: |ComponentHandle<Component>| -> Component) -> ComponentHandle<Component> {
        // FIXME: too much duplication from .add
        let pos = self.find_free_bookkeeper();
        let comp = self.components.get_mut(pos);

        // Note: we do NOT bump the serial here.
        // It is only done on component destruction
        // This is because it's impossible to get a handle that points to a bookkeeper
        // without a component inside it.

        let handle = ComponentHandle { id: pos as u16, serial: comp.serial };
        comp.component = Some(constructor(handle));
        handle
    }


    /// Removes a component by handle.
    /// Returns whether the component was found.
    pub fn remove(&mut self, handle: ComponentHandle<Component>) -> bool {
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
    pub fn find(&self, handle: ComponentHandle<Component>) -> Option<&Component> {
        let bookkeeper = &self.components[handle.id as uint];

        if bookkeeper.serial == handle.serial {
            bookkeeper.component.as_ref()
        } else {
            None
        }
    }
    /// Gets a mutable reference to a component by handle, or None if the component is not found.
    pub fn find_mut(&mut self, handle: ComponentHandle<Component>) -> Option<&mut Component> {
        let bookkeeper = self.components.get_mut(handle.id as uint);

        if bookkeeper.serial == handle.serial {
            bookkeeper.component.as_mut()
        } else {
            None
        }
    }
    fn find_free_bookkeeper(&self) -> uint {
        self.components.iter().position(|bookkeeper| bookkeeper.component.is_none()).expect("Out of room in ComponentStore!")
    }
}

impl<Component: Clone> Clone for ComponentStore<Component> {
    fn clone(&self) -> ComponentStore<Component> {
        ComponentStore {
            components: self.components.clone()
        }
    }

    fn clone_from(&mut self, source: &ComponentStore<Component>) {
        self.components.clone_from(&source.components)
    }
}
#[cfg(test)]
mod test {
    use test::Bencher;
    use super::{ComponentStore};
    struct TestStringComponent {
        name: String
    }
    struct TrivialComponent;

    #[test]
    fn smoke_componentstore() {
        let mut cstore = ComponentStore::new();
        let comp = cstore.add(TestStringComponent { name: "Hello, world!".to_string()});
        assert_eq!(cstore.find(comp).unwrap().name.as_slice(), "Hello, world!");
    }

    #[test]
    fn component_removal() { 
        let mut cstore = ComponentStore::new();
        let comp = cstore.add(TrivialComponent);
        assert!(cstore.remove(comp));

        assert!(cstore.find(comp).is_none());
        assert!(cstore.iter().next().is_none());
        assert_eq!(cstore.remove(comp), false);

        // Now make sure serials do their job
        // and prevent an old handle from referencing a new component.
        let new_comp = cstore.add(TrivialComponent);
        assert_eq!(new_comp.id, comp.id); // just checking
        assert!(cstore.find(comp).is_none());
    }

    #[test]
    // Make sure the underlying Vec does not grow if there is free space
    fn component_leak_check() { 
        let mut cstore = ComponentStore::new();

        let mut comp = cstore.add(TrivialComponent);
        let first_len = cstore.components.len();

        for _ in range(0u, 10000) {
            cstore.remove(comp);
            comp = cstore.add(TrivialComponent);
        }

        assert_eq!(cstore.components.len(), first_len);
    }

    #[bench]
    fn bench_componentcreation_32(b: &mut Bencher) {
        b.iter(|| {
            let mut cstore = ComponentStore::new();
            for _ in range(0u, 32) {
                cstore.add(TrivialComponent);
            }
            ::test::black_box(&mut cstore);
        })
    }

    #[bench]
    fn bench_componentcreationandremoval(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        b.iter(|| {
            let handle = cstore.add(TrivialComponent);
            ::test::black_box(&mut cstore);
            cstore.remove(handle);
        })
    }

    #[bench]
    fn bench_componentid_lookup(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let ids = Vec::from_fn(2048, |_| cstore.add(TrivialComponent));
        b.iter(|| cstore.find(ids[ids.len()/2]));
    }

    #[bench]
    fn bench_component_iteration_2048(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let _ = Vec::from_fn(2048, |_| cstore.add(TrivialComponent));
        b.iter(|| for comp in cstore.iter() { ::test::black_box(comp) })
    }

    #[bench]
    fn bench_component_iteration_1024_fragmented(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let ids = Vec::from_fn(2048, |_| cstore.add(TrivialComponent));
        for x in ids.iter().enumerate().filter_map(|(pos, &id)| if pos%2 == 0 { Some(id) } else { None }) {
            cstore.remove(x);
        }

        b.iter(|| for comp in cstore.iter() { ::test::black_box(comp) })
    }
    
    #[bench]
    fn bench_componentcreation_worstcase(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        for _ in range (0u, 2047) {
           cstore.add(TrivialComponent);
        }

        b.iter(|| {
            let id = cstore.add(TrivialComponent);
            ::test::black_box(id);
            cstore.remove(id);
        });
    }


    /*fn bench_gc_2048_50percent(b: &mut Bencher) {
      let mut cstore = ComponentStore::new();
      let mut estore = EntityStore::new();
      let ents = Vec::from_fn(2048, |_| estore.create_entity());
      }*/
}
