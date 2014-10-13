use std;
pub use self::components::PositionComponent;
pub mod components;

#[deriving(Show)]
/// A handle to a component.
pub struct ComponentHandle<Component> {
    id: u16,
    serial: u32
}
impl<Component> PartialEq for ComponentHandle<Component> {
    fn eq(&self, other: &ComponentHandle<Component>) -> bool {
        self.id == other.id && self.serial == other.serial
    }
}

/// This has to be public for iterator reasons.
/// Should fix later.
pub struct ComponentBookkeeper<Component> {
    serial: u32,
    component: Option<Component>
}
impl<Component> ComponentBookkeeper<Component> {
    pub fn bump_serial(&mut self) {
        self.serial = self.serial.checked_add(&1).expect("Serial no. overflow!");
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
    pub fn iter(&self) -> std::iter::FilterMap<&ComponentBookkeeper<Component>, &Component, std::slice::Items<ComponentBookkeeper<Component>>> {
        self.components.iter().filter_map(|&ComponentBookkeeper{component: ref comp, ..}| comp.as_ref())
    }
    /// Iterate over all components, mutably.
    pub fn iter_mut(&mut self) -> std::iter::FilterMap<&mut ComponentBookkeeper<Component>, &mut Component, std::slice::MutItems<ComponentBookkeeper<Component>>> {
        self.components.iter_mut().filter_map(|&ComponentBookkeeper{component: ref mut comp, ..}| comp.as_mut())
    }

    /// Add a component
    pub fn add(&mut self, component: Component) -> ComponentHandle<Component> {
        let result = self.components.iter().position(|bookkeeper| bookkeeper.component.is_none());
        let pos = result.expect("Out of room in ComponentStore!");
        
        let comp = self.components.get_mut(pos);

        // Note: we do NOT bump the serial here.
        // It is only done on component destruction
        // This is because it's impossible to get a handle that points to a bookkeeper
        // without a component inside it.

        comp.component = Some(component);

        ComponentHandle { id: pos as u16, serial: comp.serial }
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

    /*fn bench_gc_2048_50percent(b: &mut Bencher) {
      let mut cstore = ComponentStore::new();
      let mut estore = EntityStore::new();
      let ents = Vec::from_fn(2048, |_| estore.create_entity());
      }*/
}
