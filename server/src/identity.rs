use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone)]
pub struct IdentityRegistry {
    identity: FxHashMap<u64, String>,
}

impl IdentityRegistry {
    pub fn new() -> Self {
        Self {
            identity: FxHashMap::default(),
        }
    }

    pub fn register(&mut self, player_id: u64, name: String) {
        self.identity.insert(player_id, name);
    }
    pub fn check(&self, player_id: u64, name: &str) -> bool {
        match self.identity.get(&player_id) {
            None => true,
            Some(registered) => registered == name,
        }
    }
    pub fn entries(&self) -> Vec<(&u64, &String)> {
        self.identity.iter().collect()
    }
    pub fn load(&mut self, entries: Vec<(&u64, &String)>) {
        for (player_id, name) in entries {
            self.identity.insert(*player_id, name.to_string());
        }
    }
}
