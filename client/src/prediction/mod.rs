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
    history: RingBuf<PredictionState>
}

impl Prediction {
    pub fn new(controllable: ControllableComponent) -> Prediction {
        Prediction {
            controllable: controllable,
            history: RingBuf::new()
        }
    }

    pub fn add_server_update(&mut self,
                             tick: u64,
                             entities: ComponentStore<EntityComponent>) {
        while self.history.front().map(|state| state.tick < tick).unwrap_or(false) {
            self.history.pop_front();
        }

        let oldest = self.history.pop_front();

        let oldest = oldest.map(|state| {
            assert_eq!(state.tick, tick);

            PredictionState {
                entities: entities.clone(),
                ..state
            }
        }).unwrap_or(PredictionState {
            tick: tick,
            entities: entities,
            cmd: None
        });

        

        self.history.push_front(oldest);
    }

    pub fn predict_tick(&mut self, cmd: PlayerCommand) -> bool {
        let (mut entities, prev_tick) = {
            let prev_state = match self.history.back() {
                Some(state) => state,
                None => return false
            };
            (prev_state.entities.clone(), prev_state.tick)
        };

        playercmd::run_command(cmd,
                               &mut self.controllable,
                               &mut entities);

        self.history.push(PredictionState {
            tick: prev_tick + 1,
            entities: entities,
            cmd: Some(cmd)
        });

        true
    }

    pub fn len(&self) -> uint {
        self.history.len()
    }

    pub fn get_predicted_state(&mut self) -> Option<ComponentStore<EntityComponent>> {
        let mut controllable = self.controllable;

        let entities = self.history.front().map(|oldest| oldest.entities.clone()).map(|entities| {
            self.history.iter_mut().fold(entities, |mut entities, state| {
                state.cmd.map(|cmd| {
                    playercmd::run_command(cmd,
                                           &mut controllable,
                                           &mut entities)
                });

                entities
            })
        });

        self.controllable = controllable; 

        entities
    }
}

struct PredictionState {
    tick: u64,

    entities: ComponentStore<EntityComponent>,

    /// For the current tick- the effects of this command are included
    /// in this state!
    cmd: Option<PlayerCommand>
}
