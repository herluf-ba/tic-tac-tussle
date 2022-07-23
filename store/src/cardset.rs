use std::collections::HashMap;

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CardSpec {
    pub name: String,
    pub group: String,
    pub tier: u32,
    pub cost: HashMap<String, u32>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CardSet {
    pub tiers: Vec<Vec<String>>,
    pub cards: HashMap<String, CardSpec>,
}

impl CardSet {
    pub fn from_file(path: &str) -> Self {
        // Load and deserialize card specification
        let file = std::fs::read_to_string(path).expect("No card specification found");
        let cards: HashMap<String, CardSpec> = serde_json::from_str(&file).unwrap();

        // Get highest tier to init 'tiers' field
        let max = cards
            .iter()
            .max_by_key(|(_, card)| card.tier)
            .and_then(|(_, card)| Some(card.tier))
            .unwrap_or(1);

        // Compute tiers from loaded specification for easy lookups later
        let mut tiers: Vec<Vec<String>> = (0..max).map(|_| Vec::new()).collect();
        for (card_id, card) in cards.iter() {
            tiers[(card.tier - 1) as usize].push(card_id.clone())
        }

        Self { cards, tiers }
    }
    pub fn get_card_from_tier(&self, tier: usize) -> String {
        let mut rng = rand::thread_rng();
        let tier = &self.tiers[tier];
        let random_index = rng.gen_range(0..(tier.len() - 1));
        tier[random_index].clone()
    }
}
