//! This crate contains code that is shared between the client and server.
//! It's probably well-named.
//!
//! This split is for two reasons:
//! 1. The client has code that dedicated servers don't care about (rendering,
//! sound, etc.).
//! 2. Compile times will probably get really hairy. Doing it like this
//! lets us approximate incremental compilation.
//!
//! So what's in here?
//! Simulation code, because client needs to predict locally
//! Map-related code, obviously
//! AI may possibly be here, or end up in server
//! Networking, because client and server need to talk to each other.
//! and other things?

extern crate cgmath;
extern crate test;

pub use entity::{
    Component, ComponentStore, ComponentHandle, EntityID, EntityStore,

    PositionComponent
};

pub mod entity;
    
#[test]
fn it_works() {
}
