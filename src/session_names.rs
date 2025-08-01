// Word-pair generation for memorable session names
// Based on deterministic hashing of session ID

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fmt;

/// Dictionary of themed word lists
pub struct WordLists {
    pub space: Vec<&'static str>,
    pub food: Vec<&'static str>,
    pub animals: Vec<&'static str>,
    pub tech: Vec<&'static str>,
    pub nature: Vec<&'static str>,
    pub mythical: Vec<&'static str>,
    pub colors: Vec<&'static str>,
    pub musical: Vec<&'static str>,
    pub literary: Vec<&'static str>,
    pub historical: Vec<&'static str>,
}

impl WordLists {
    pub fn new() -> Self {
        Self {
            space: vec![
                "nebula", "quasar", "pulsar", "comet", "meteor", "asteroid", "galaxy", "star", "planet", "moon", 
                "orbit", "gravity", "cosmos", "universe", "void", "void", "void", "void", "void", "void"
            ],
            food: vec![
                "sushi", "tacos", "pasta", "pizza", "burger", "sandwich", "salad", "soup", "curry", "ramen", 
                "tiramisu", "cake", "cookie", "donut", "cupcake", "muffin", "pancake", "waffle", "fries", "nachos"
            ],
            animals: vec![
                "eagle", "lion", "tiger", "bear", "wolf", "fox", "deer", "bison", "moose", "elk", 
                "otter", "beaver", "raccoon", "squirrel", "badger", "porcupine", "marmot", "groundhog", "beetle", "cricket"
            ],
            tech: vec![
                "processor", "memory", "storage", "network", "bandwidth", "protocol", "algorithm", "syntax", "debug", "compile", 
                "runtime", "framework", "library", "module", "package", "dependency", "version", "release", "patch", "update"
            ],
            nature: vec![
                "mountain", "valley", "forest", "river", "stream", "lake", "ocean", "beach", "desert", "prairie", 
                "canyon", "cliff", "plateau", "plain", "glacier", "tundra", "swamp", "marsh", "fen", "bog"
            ],
            mythical: vec![
                "dragon", "unicorn", "phoenix", "griffin", "centaur", "minotaur", "leviathan", "kraken", "gorgon", "sphinx", 
                "cyclops", "basilisk", "chimera", "hydra", "medusa", "pegasus", "unicorn", "dragon", "phoenix", "griffin"
            ],
            colors: vec![
                "red", "blue", "green", "yellow", "purple", "orange", "pink", "brown", "black", "white", 
                "gray", "silver", "gold", "crimson", "teal", "lavender", "mauve", "beige", "olive", "navy"
            ],
            musical: vec![
                "melody", "harmony", "rhythm", "beat", "tempo", "note", "chord", "scale", "symphony", "opera", 
                "sonata", "concerto", "suite", "aria", "ballad", "jazz", "rock", "pop", "hiphop", "classical"
            ],
            literary: vec![
                "novel", "poem", "story", "chapter", "verse", "stanza", "dialogue", "monologue", "narrative", "plot", 
                "character", "setting", "theme", "mood", "tone", "genre", "fable", "myth", "legend", "epic"
            ],
            historical: vec![
                "empire", "dynasty", "reign", "revolution", "war", "battle", "conquest", "colonization", "exploration", "discovery", 
                "civilization", "kingdom", "province", "territory", "frontier", "epoch", "era", "age", "period", "century"
            ],
        }
    }

    /// Get the appropriate word list based on project type
    pub fn get_list(&self, project_type: &str) -> &Vec<&'static str> {
        match project_type {
            "space" | "astronomy" | "physics" | "cosmology" | "space" | "rocket" => &self.space,
            "food" | "cooking" | "recipe" | "baking" | "culinary" => &self.food,
            "animals" | "zoo" | "wildlife" | "nature" | "ecology" => &self.animals,
            "tech" | "software" | "programming" | "development" | "engineering" => &self.tech,
            "nature" | "outdoor" | "environment" | "ecology" | "wildlife" => &self.nature,
            "myth" | "legend" | "fantasy" | "mythology" | "fairy" => &self.mythical,
            "color" | "design" | "art" | "painting" | "visual" => &self.colors,
            "music" | "song" | "melody" | "instrument" | "band" => &self.musical,
            "literature" | "writing" | "novel" | "poetry" | "story" => &self.literary,
            "history" | "ancient" | "medieval" | "modern" | "period" => &self.historical,
            _ => &self.space, // Default to space
        }
    }
}

/// Session name generator with deterministic hashing
pub struct SessionNameGenerator {
    word_lists: WordLists,
}

impl SessionNameGenerator {
    pub fn new() -> Self {
        Self {
            word_lists: WordLists::new(),
        }
    }

    /// Generate a memorable word-pair name from a session ID
    /// Uses deterministic hashing to ensure consistent names across runs
    pub fn generate(&self, session_id: &str, project_type: &str) -> String {
        // Create a hash of the session ID
        let mut hasher = DefaultHasher::new();
        session_id.hash(&mut hasher);
        let hash = hasher.finish();

        // Get the appropriate word list based on project type
        let word_list = self.word_lists.get_list(project_type);

        // Use the hash to deterministically select two words from the list
        // Ensure we don't get the same word twice
        let first_idx = (hash as usize) % word_list.len();
        let second_idx = ((hash as usize) + 1) % word_list.len();

        // Make sure we don't pick the same word twice
        let second_idx = if second_idx == first_idx {
            (second_idx + 1) % word_list.len()
        } else {
            second_idx
        };

        format!("{}-{}", word_list[first_idx], word_list[second_idx])
    }
}

/// Implement Debug for better error messages
impl fmt::Debug for SessionNameGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SessionNameGenerator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_lists() {
        let generator = SessionNameGenerator::new();
        let word_lists = &generator.word_lists;
        
        // Test that each word list has at least one word
        assert!(word_lists.space.len() > 0);
        assert!(word_lists.food.len() > 0);
        assert!(word_lists.animals.len() > 0);
        assert!(word_lists.tech.len() > 0);
        assert!(word_lists.nature.len() > 0);
        assert!(word_lists.mythical.len() > 0);
        assert!(word_lists.colors.len() > 0);
        assert!(word_lists.musical.len() > 0);
        assert!(word_lists.literary.len() > 0);
        assert!(word_lists.historical.len() > 0);
    }

    #[test]
    fn test_generate_consistent() {
        let generator = SessionNameGenerator::new();
        let session_id = "test-session-123";
        let project_type = "space";
        
        // Generate the same name twice
        let name1 = generator.generate(session_id, project_type);
        let name2 = generator.generate(session_id, project_type);
        
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_generate_different_projects() {
        let generator = SessionNameGenerator::new();
        let session_id = "test-session-123";
        
        // Generate names for different project types
        let name1 = generator.generate(session_id, "space");
        let name2 = generator.generate(session_id, "food");
        let name3 = generator.generate(session_id, "tech");
        
        // Ensure they're different
        assert_ne!(name1, name2);
        assert_ne!(name1, name3);
        assert_ne!(name2, name3);
    }

    #[test]
    fn test_generate_different_sessions() {
        let generator = SessionNameGenerator::new();
        let project_type = "space";
        
        // Generate names for different session IDs
        let name1 = generator.generate("session-1", project_type);
        let name2 = generator.generate("session-2", project_type);
        
        // They should be different
        assert_ne!(name1, name2);
    }
}
