#![forbid(unsafe_block)]
#![feature(default_type_params)]
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

extern crate anymap;
extern crate cgmath;
extern crate serialize;
extern crate test;

pub use component::{
    ComponentStore, ComponentHandle,
    EntityComponent, EntityHandle
};

pub mod component;
pub mod network;
pub mod physics;
pub mod playercmd;

/// Length of one simulation tick, in seconds.
pub static TICK_LENGTH: f32 = 1.0 / 128.0;
