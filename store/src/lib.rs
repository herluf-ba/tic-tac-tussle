use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Struct for storing player related data.
/// In tic-tac-toe the only thing we need is the name and the piece the player will be placing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub piece: Tile,
}

/// Possible GameStates for a tile in the board
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Tic,
    Tac,
}

/// The different states a game can be in. (not to be confused with the entire "GameState")
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    PreGame,
    InGame,
    Ended,
}

// This just makes it easier to dissern between a player id and any ol' u64
type PlayerId = u64;

/// A GameState object that is able to keep track of a game of TicTacTussle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub stage: Stage,
    pub board: [Tile; 9],
    pub active_player_id: PlayerId,
    pub players: HashMap<PlayerId, Player>,
    pub history: Vec<GameEvent>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            stage: Stage::PreGame,
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

/// The various reasons why a game could end
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Deserialize)]
pub enum EndGameReason {
    // In tic tac toe it doesn't make sense to keep playing when one of the players disconnect.
    // Note that it might make sense to keep playing in some other game (like Team Fight Tactics for instance).
    PlayerLeft { player_id: PlayerId },
    PlayerWon { winner: PlayerId },
}

/// An event that progresses the GameGameState forward
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub enum GameEvent {
    BeginGame { goes_first: PlayerId },
    EndGame { reason: EndGameReason },
    PlayerJoined { player_id: PlayerId, name: String },
    PlayerDisconnected { player_id: PlayerId },
    PlaceTile { player_id: PlayerId, at: usize },
}

impl GameState {
    /// Determines whether an event is valid considering the current GameState
    pub fn validate(&self, event: &GameEvent) -> bool {
        use GameEvent::*;
        match event {
            BeginGame { goes_first } => {
                let player_is_unknown = self.players.contains_key(goes_first);
                if self.stage != Stage::PreGame || player_is_unknown {
                    return false;
                }
            }
            EndGame { reason } => match reason {
                EndGameReason::PlayerWon { winner: _ } => {
                    if self.stage != Stage::InGame {
                        return false;
                    }
                }
                _ => {}
            },
            PlayerJoined { player_id, name: _ } => {
                if self.players.contains_key(player_id) {
                    return false;
                }
            }
            PlayerDisconnected { player_id } => {
                if !self.players.contains_key(player_id) {
                    return false;
                }
            }
            PlaceTile { player_id, at } => {
                if !self.players.contains_key(player_id) {
                    return false;
                }

                if self.active_player_id != *player_id {
                    return false;
                }

                if *at > 8 {
                    return false;
                }
                if self.board[*at] != Tile::Empty {
                    return false;
                }
            }
        }

        true
    }

    /// Consumes an event, modifying the GameState and adding the event to its history
    /// NOTE: consume assumes the event to have already been validated and will accept *any* event passed to it
    pub fn consume(&mut self, valid_event: &GameEvent) {
        use GameEvent::*;
        match valid_event {
            BeginGame { goes_first } => {
                self.active_player_id = *goes_first;
                self.stage = Stage::InGame;
            }
            EndGame { reason: _ } => self.stage = Stage::Ended,
            PlayerJoined { player_id, name } => {
                self.players.insert(
                    *player_id,
                    Player {
                        name: name.to_string(),
                        // First player to join gets tac, second gets tic
                        piece: if self.players.len() > 0 {
                            Tile::Tac
                        } else {
                            Tile::Tic
                        },
                    },
                );
            }
            PlayerDisconnected { player_id } => {
                self.players.remove(player_id);
            }
            PlaceTile { player_id, at } => {
                let piece = self.get_player_tile(player_id).unwrap();
                self.board[*at] = piece;
                self.active_player_id = self
                    .players
                    .keys()
                    .find(|id| *id != player_id)
                    .unwrap()
                    .clone();
            }
        }

        self.history.push(valid_event.clone());
    }

    /// Gets a players tile, if the player is known to game state
    pub fn get_player_tile(&self, player_id: &PlayerId) -> Option<Tile> {
        if let Some(player) = self.players.get(player_id) {
            return Some(player.piece);
        }

        None
    }

    /// Determines if someone has won the game
    pub fn determine_winner(&self) -> Option<PlayerId> {
        // All the combinations of 3 tiles that wins the game
        let row1: [usize; 3] = [0, 1, 2];
        let row2: [usize; 3] = [3, 4, 5];
        let row3: [usize; 3] = [6, 7, 8];
        let col1: [usize; 3] = [0, 3, 6];
        let col2: [usize; 3] = [1, 4, 7];
        let col3: [usize; 3] = [2, 5, 8];
        let diag1: [usize; 3] = [0, 4, 8];
        let diag2: [usize; 3] = [2, 4, 6];

        for arr in [row1, row2, row3, col1, col2, col3, diag1, diag2] {
            // Read tiles from board
            let tiles: [Tile; 3] = [self.board[arr[0]], self.board[arr[1]], self.board[arr[2]]];
            // Determine if tiles are all equal
            let all_are_the_same = tiles
                .get(0)
                .map(|first| tiles.iter().all(|x| x == first))
                .unwrap_or(true);

            if all_are_the_same {
                // Determine which of the players won
                if let Some((winner, _)) = self
                    .players
                    .iter()
                    .find(|(_, player)| player.piece == self.board[arr[0]])
                {
                    return Some(*winner);
                }
            }
        }

        None
    }
}
