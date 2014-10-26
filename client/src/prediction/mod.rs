use std::collections::RingBuf;
use std::collections::Deque;
use shared::{
    ComponentStore,
    EntityComponent,
    EntityHandle,

    playercmd
};
use shared::network::UpdatePacket;
use shared::playercmd::{ControllableComponent, PlayerCommand};

pub struct Prediction {
    controllable: ControllableComponent,
    history: RingBuf<PlayerCommand>,

    history_start: u64
}

impl Prediction {
    pub fn new(controllable: ControllableComponent) -> Prediction {
        Prediction {
            controllable: controllable,
            history: RingBuf::new(),
            history_start: 0
        }
    }

    pub fn add_command(&mut self, cmd: PlayerCommand) {
        self.history.push(cmd);
    }

    pub fn predict(&mut self, ticks: u64, entities: &mut ComponentStore<EntityComponent>) {
        while self.history.len() as u64 > ticks {
            self.history.pop_front();
        }

        let mut controllable = self.controllable;

        for cmd in self.history.iter_mut() {
            playercmd::run_command(*cmd,
                                   &mut controllable,
                                   entities);
        }

        self.controllable = controllable; 
    }
}
