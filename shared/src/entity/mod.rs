use std;
use std::collections::Bitv;

#[deriving(Eq, PartialEq, Ord, PartialOrd, Show)]
pub struct EntityID(u32);

#[deriving(Show)]
/// Type parameter used only for correctness.
pub struct ComponentID<PayloadTy>(u32);

pub struct Component<Payload> {
    id: ComponentID<Payload>,
    entity: EntityID,
    payload: Payload
}
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


impl<Payload> PartialEq for Component<Payload> {
    fn eq(&self, other: &Component<Payload>) -> bool {
        self.id == other.id
    }
}   
impl<Payload> Ord for Component<Payload> {
    fn cmp(&self, other: &Component<Payload>) -> Ordering {
        self.id.cmp(&other.id)
    }
}   
impl<Payload> PartialOrd for Component<Payload> {
    fn partial_cmp(&self, other: &Component<Payload>) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}  
impl<Payload> Eq for Component<Payload> {}

impl<Payload> Component<Payload>  {
    /// Returns the ID of this component.
    fn get_id(&self) -> ComponentID<Payload> { self.id }
    
    /// Returns the ID of the entity this component is attached to.
    fn get_entity_id(&self) -> EntityID { self.entity }
}

pub struct ComponentStore<Payload> {
    components: Vec<Component<Payload>>,
    next_id: ComponentID<Payload>
}

impl<Payload> ComponentStore<Payload> {
    pub fn new() -> ComponentStore<Payload> {
        ComponentStore { components: Vec::new(), next_id: ComponentID(0) }
    }

    /// Entity IDs are used in a sort of backwards garbage collection scheme;
    /// specifically, various component stores may occasionally run through
    /// all the components contained therein, checking the entity ID of each one
    /// against the EntityStore for liveness.
    /// This should be a fairly infrequent operation.
    // TODO: we can probably save reallocating here if we're clever
    // is it worth it?
    pub fn gc(self, is_ent_alive: |EntityID| -> bool) -> ComponentStore<Payload> {
        ComponentStore {
            components: self.components.into_iter()
                .filter(|comp| is_ent_alive(comp.get_entity_id())).collect(),
            next_id: self.next_id
        }
    }
    
    pub fn iter(&self) -> std::slice::Items<Component<Payload>> { self.components.iter() }

    fn bump_id(&mut self) {
        let ComponentID(id) = self.next_id;
        // beware of overflow. if we actually fail here,
        // we'll cross that bridge when we come to it.
        self.next_id = ComponentID(id.checked_add(&1).unwrap());
    }

    pub fn add_component(&mut self, entity: EntityID, payload: Payload) -> ComponentID<Payload> {
        let id = self.next_id;
        self.components.push(Component { id: id, entity: entity, payload: payload });
        self.bump_id();
        id
    }

    pub fn find(&self, id: &ComponentID<Payload>) -> Option<&Component<Payload>> {
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
        name: String
    }
    struct TrivialComponent;

    #[test]
    fn smoke_componentstore() {
        let mut cstore = ComponentStore::new();
        let mut estore = EntityStore::new();
        let ent = estore.create_entity();
        let comp = cstore.add_component(EntityID(0), TestStringComponent { name: "Hello, world!".to_string()});
        cstore = cstore.gc(|ref id| estore.is_alive(id));
        assert_eq!(cstore.find(&comp).unwrap().payload.name.as_slice(), "Hello, world!");
    }

    #[bench]
    fn bench_entcreation_singlecomponent(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        b.iter(|| cstore.add_component(EntityID(0), TrivialComponent));
        ::test::black_box(cstore);
    }

    #[bench]
    fn bench_componentid_lookup_2048(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let ids = Vec::from_fn(2048, |_| cstore.add_component(EntityID(0), TrivialComponent));
        for id in ids.iter() {
            b.iter(|| cstore.find(id));
        }
    }

    #[bench]
    fn bench_component_iteration_2048(b: &mut Bencher) {
        let mut cstore = ComponentStore::new();
        let _ = Vec::from_fn(2048, |_| cstore.add_component(EntityID(0), TrivialComponent));
        b.iter(|| for comp in cstore.iter() { ::test::black_box(comp) })
    }


} 
