use std::collections::RingBuf;
use std::collections::Deque;
use shared::{
    ComponentStore,
    EntityComponent,

    playercmd
};
use shared::network::channel::{overflow_aware_compare, SequenceNr};
use shared::playercmd::{ControllableComponent, PlayerCommand};
use cgmath::ApproxEq;

pub struct Prediction {
    controllable: ControllableComponent,
    history: RingBuf<(SequenceNr, PlayerCommand)>,

    predicted: Option<ComponentStore<EntityComponent>>
}

impl Prediction {
    pub fn new(controllable: ControllableComponent) -> Prediction {
        Prediction {
            controllable: controllable,
            history: RingBuf::new(),

            predicted: None
        }
    }

    pub fn update(&mut self, acked_sequence: SequenceNr, new_entities: &ComponentStore<EntityComponent>) {
        self.predicted = Some(match self.predicted.take() {
            Some(mut entities) => {
                let oldpos = entities.find(self.controllable.entity).unwrap().pos;

                entities.clone_from(new_entities);

                self.remove_old_history(acked_sequence);
                for &(_, cmd) in self.history.iter() {
                    playercmd::run_command(cmd, &mut self.controllable, &mut entities);
                }

                let newpos = entities.find(self.controllable.entity).unwrap().pos;
                if !newpos.approx_eq(&oldpos) {
                    println!("Prediction error: {} update vs. {} pred", newpos, oldpos)
                };

                entities
            },
            None => new_entities.clone()
        });
    }

    fn remove_old_history(&mut self, latest_ack: SequenceNr) {
        while self.history.front().map(|&(seq, _)| overflow_aware_compare(seq, latest_ack) != ::std::cmp::Greater).unwrap_or(false) {
            self.history.pop_front();
        }
    }

    pub fn predict(&mut self, cmd: PlayerCommand, sequence: SequenceNr) {
        // borrow checker hack
        let mut controllable = self.controllable;

        self.predicted.as_mut().map(|ents| {
            playercmd::run_command(cmd, &mut controllable, ents)
        });

        self.controllable = controllable;

        self.history.push((sequence, cmd));
    }

    pub fn get_entities(&self) -> Option<&ComponentStore<EntityComponent>> {
        self.predicted.as_ref()
    }

}
