use std;
use std::collections::Bitv;

#[deriving(Eq, PartialEq, Ord, PartialOrd, Show)]
pub struct EntityID(u32);

#[deriving(Show)]
pub struct ComponentID<ComponentTy: Component>(u32);
impl<ComponentTy: Component> PartialEq for ComponentID<ComponentTy> {
    fn eq(&self, other: &ComponentID<ComponentTy>) -> bool {
        let (&ComponentID(thisid), &ComponentID(otherid)) = (self, other);
        thisid == otherid
    }
}   
impl<ComponentTy: Component> Ord for ComponentID<ComponentTy> {
    fn cmp(&self, other: &ComponentID<ComponentTy>) -> Ordering {
        let (&ComponentID(thisid), &ComponentID(otherid)) = (self, other);
        thisid.cmp(&otherid)
    }
}   
impl<ComponentTy: Component> PartialOrd for ComponentID<ComponentTy> {
    fn partial_cmp(&self, other: &ComponentID<ComponentTy>) -> Option<Ordering> {
        let (&ComponentID(thisid), &ComponentID(otherid)) = (self, other);
        thisid.partial_cmp(&otherid)
    }
}  
impl<ComponentTy: Component> Eq for ComponentID<ComponentTy> {}

pub trait Component {
    /// Returns the ID of this component.
    fn get_id(&self) -> ComponentID<Self>;
    
    /// Returns the ID of the entity this component is attached to.
    fn get_entity_id(&self) -> EntityID;
}

pub struct ComponentStore<ComponentTy: Component> {
    components: Vec<ComponentTy>,
    next_id: ComponentID<ComponentTy>
}

impl<ComponentTy: Component> ComponentStore<ComponentTy> {
    pub fn new() -> ComponentStore<ComponentTy> {
        ComponentStore { components: Vec::new(), next_id: ComponentID(0) }
    }

    /// Entity IDs are used in a sort of backwards garbage collection scheme;
    /// specifically, various component stores may occasionally run through
    /// all the components contained therein, checking the entity ID of each one
    /// against the EntityStore for liveness.
    /// This should be a fairly infrequent operation.
    // TODO: we can probably save reallocating here if we're clever
    // is it worth it?
    pub fn gc(self, is_ent_alive: |EntityID| -> bool) -> ComponentStore<ComponentTy> {
        ComponentStore {
            components: self.components.into_iter()
                .filter(|comp| is_ent_alive(comp.get_entity_id())).collect(),
            next_id: self.next_id
        }
    }
    
    pub fn iter(&self) -> std::slice::Items<ComponentTy> { self.components.iter() }

    fn bump_id(&mut self) {
        let ComponentID(id) = self.next_id;
        // beware of overflow. if we actually fail here,
        // we'll cross that bridge when we come to it.
        self.next_id = ComponentID(id.checked_add(&1).unwrap());
    }

    pub fn add_component(&mut self,
            constructor: |ComponentID<ComponentTy>| -> ComponentTy) -> ComponentID<ComponentTy>
    {
        self.components.push(constructor(self.next_id));
        let id = self.next_id;
        self.bump_id();
        id
    }

    pub fn find(&self, id: &ComponentID<ComponentTy>) -> Option<&ComponentTy> {
        use std::slice::{Found, NotFound};
        let result = self.components[].binary_search(|probe| probe.get_id().cmp(id));
        match result {
            Found(pos) => Some(&self.components[pos]),
            NotFound(_) => None
        }
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
}
#[cfg(test)]
mod test {
    use test::Bencher;
    use super::{ComponentID, EntityID, Component, ComponentStore, EntityStore};
    struct TestStringComponent {
        id: ComponentID<TestStringComponent>,
        entity: EntityID,
        name: String
    }
    struct TrivialComponent {
        id: ComponentID<TrivialComponent>
    }
    impl Component for TrivialComponent {
        fn get_id(&self) -> ComponentID<TrivialComponent> { self.id }
        fn get_entity_id(&self) -> EntityID { unimplemented!() }
    }
    impl Component for TestStringComponent {
        fn get_id(&self) -> ComponentID<TestStringComponent> { self.id }
        fn get_entity_id(&self) -> EntityID { self.entity }
    }

    #[test]
    fn smoke_componentstore() {
        let mut cstore = ComponentStore::new();
        let mut estore = EntityStore::new();
        let ent = estore.create_entity();
        let comp = cstore.add_component(|id| TestStringComponent { id: id, entity: ent, name: "Hello, world!".to_string()});
        cstore = cstore.gc(|ref id| estore.is_alive(id));
        assert_eq!(cstore.find(&comp).unwrap().name.as_slice(), "Hello, world!");
    }

    #[bench]
    fn bench_entcreation_singlecomponent(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        b.iter(|| cstore.add_component(|id| TrivialComponent {id: id}));
        ::test::black_box(cstore);
    }

    #[bench]
    fn bench_componentid_lookup_2048(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let ids = Vec::from_fn(2048, |_| cstore.add_component(|id| TrivialComponent {id: id}));
        for id in ids.iter() {
            b.iter(|| cstore.find(id));
        }
    }

    #[bench]
    fn bench_component_iteration_2048(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let _ = Vec::from_fn(2048, |_| cstore.add_component(|id| TrivialComponent {id: id}));
        b.iter(|| for comp in cstore.iter() { ::test::black_box(comp) })
    }


} 
