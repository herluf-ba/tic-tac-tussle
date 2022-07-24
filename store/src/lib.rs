use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Tic,
    Tac,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum State {
    PreGame,
    Playing,
    Ended,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub state: State,
    pub board: [Tile; 9],
    pub active_player_id: u64,
    pub players: HashMap<u64, Player>,
    pub history: Vec<GameEvent>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            state: State::PreGame,
            board: [
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
                Tile::Empty,
            ],
            active_player_id: 0,
            players: HashMap::new(),
            history: Vec::new(),
        }
    }
}

/// The various reasons why game has been ended
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub enum EndGameReason {
    // In tic tac toe it doesn't make sense to keep playing when one of the players disconnect.
    // Note that it might make sense to keep playing in some other game (like Team Fight Tactics for instance).
    PlayerDisconnected { player_id: u64 },
    PlayerWon { winner: u64 },
}

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub enum GameEvent {
    BeginGame {
        goes_first: u64,
    },
    EndGame {
        reason: EndGameReason,
    },
    PlayerJoined {
        player_id: u64,
        name: String,
    },
    PlayerDisconnected {
        player_id: u64,
    },
    PlaceTile {
        player_id: u64,
        tile: Tile,
        at: usize,
    },
}

impl GameEvent {
    pub fn is_valid_on(&self, game_state: &GameState) -> bool {
        use GameEvent::*;

        match self {
            BeginGame { goes_first } => {
                let player_is_unknown = game_state.players.contains_key(goes_first);
                if game_state.state != State::PreGame || player_is_unknown {
                    return false;
                }
            }
            EndGame { reason } => match reason {
                EndGameReason::PlayerWon { winner: _ } => {
                    if game_state.state != State::Playing {
                        return false;
                    }
                }
                _ => {}
            },
            PlayerJoined { player_id, name: _ } => {
                if game_state.players.contains_key(player_id) {
                    return false;
                }
            }
            PlayerDisconnected { player_id } => {
                if !game_state.players.contains_key(player_id) {
                    return false;
                }
            }
            PlaceTile {
                player_id,
                tile,
                at,
            } => {
                if !game_state.players.contains_key(player_id) {
                    return false;
                }
                if game_state.active_player_id == *player_id {
                    return false;
                }
                if *at > 8 {
                    return false;
                }
                if game_state.board[*at] == *tile {
                    return false;
                }
            }
        }

        true
    }
}

impl GameState {
    fn reduce(&mut self, event: &GameEvent) {
        use GameEvent::*;
        match event {
            BeginGame { goes_first } => self.active_player_id = *goes_first,
            EndGame { reason: _ } => self.state = State::Ended,
            PlayerJoined { player_id, name } => {
                self.players.insert(
                    *player_id,
                    Player {
                        name: name.to_string(),
                    },
                );
            }
            PlayerDisconnected { player_id } => {
                self.players.remove(player_id);
            }
            PlaceTile {
                player_id,
                tile,
                at,
            } => {
                self.board[*at] = *tile;
                self.active_player_id = self
                    .players
                    .keys()
                    .find(|id| *id != player_id)
                    .unwrap()
                    .clone();
            }
        }

        self.history.push(event.clone());
    }

    pub fn dispatch(&mut self, event: &GameEvent) {
        self.reduce(event);
    }
}
