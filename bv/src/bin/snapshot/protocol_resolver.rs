use eyre::Result;
use std::collections::HashMap;

pub struct ProtocolResolver {
    known_protocols: HashMap<String, String>, // User input -> normalized form
    display_names: HashMap<String, String>,   // Normalized -> user-friendly display
}

impl ProtocolResolver {
    pub fn new() -> Self {
        let mut known_protocols = HashMap::new();
        let mut display_names = HashMap::new();
        
        // Multi-word protocols - handle different input formats
        let arbitrum_variants = vec![
            "arbitrum one",
            "arbitrum-one", 
            "arbitrumone",
            "arbitrum_one",
        ];
        for variant in arbitrum_variants {
            known_protocols.insert(variant.to_string(), "arbitrum-one".to_string());
        }
        display_names.insert("arbitrum-one".to_string(), "arbitrum-one".to_string());
        
        // Polygon variants
        let polygon_variants = vec![
            "polygon",
            "polygon pos",
            "polygon-pos",
            "polygonpos",
        ];
        for variant in polygon_variants {
            known_protocols.insert(variant.to_string(), "polygon".to_string());
        }
        display_names.insert("polygon".to_string(), "polygon".to_string());
        
        // Single word protocols
        let simple_protocols = vec![
            "ethereum",
            "bitcoin", 
            "solana",
            "avalanche",
        ];
        for protocol in simple_protocols {
            known_protocols.insert(protocol.to_string(), protocol.to_string());
            display_names.insert(protocol.to_string(), protocol.to_string());
        }
        
        Self { 
            known_protocols, 
            display_names 
        }
    }
    
    /// Normalize user input to the canonical protocol name used in bucket paths
    pub fn normalize_protocol(&self, input: &str) -> Result<String> {
        let input_clean = input.trim().to_lowercase();
        
        // Direct match first
        if let Some(normalized) = self.known_protocols.get(&input_clean) {
            return Ok(normalized.clone());
        }
        
        // Try with spaces replaced by dashes
        let with_dashes = input_clean.replace(' ', "-");
        if let Some(normalized) = self.known_protocols.get(&with_dashes) {
            return Ok(normalized.clone());
        }
        
        // Try with dashes replaced by spaces  
        let with_spaces = input_clean.replace('-', " ");
        if let Some(normalized) = self.known_protocols.get(&with_spaces) {
            return Ok(normalized.clone());
        }
        
        // Try with underscores replaced by dashes
        let underscores_to_dashes = input_clean.replace('_', "-");
        if let Some(normalized) = self.known_protocols.get(&underscores_to_dashes) {
            return Ok(normalized.clone());
        }
        
        // Try fuzzy matching for small typos
        if let Some(normalized) = self.find_closest_match(&input_clean) {
            return Ok(normalized);
        }
        
        Err(eyre::anyhow!(
            "Unknown protocol: '{}'. Available protocols: {}", 
            input,
            self.available_protocols().join(", ")
        ))
    }
    
    /// Get user-friendly display name for a normalized protocol
    pub fn display_name(&self, normalized: &str) -> String {
        self.display_names
            .get(normalized)
            .cloned()
            .unwrap_or_else(|| normalized.to_string())
    }
    
    /// Get list of available protocols for help messages
    pub fn available_protocols(&self) -> Vec<String> {
        let mut protocols: Vec<String> = self.display_names.values().cloned().collect();
        protocols.sort();
        protocols.dedup();
        protocols
    }
    
    /// Simple fuzzy matching for typos (Levenshtein distance)
    fn find_closest_match(&self, input: &str) -> Option<String> {
        let mut best_match = None;
        let mut best_distance = usize::MAX;
        
        for known_protocol in self.known_protocols.keys() {
            let distance = levenshtein_distance(input, known_protocol);
            
            // Only suggest if distance is small relative to input length
            if distance <= input.len() / 3 && distance < best_distance {
                best_distance = distance;
                best_match = self.known_protocols.get(known_protocol).cloned();
            }
        }
        
        best_match
    }
    
    /// Check if a protocol is known (for validation)
    pub fn is_known_protocol(&self, input: &str) -> bool {
        self.normalize_protocol(input).is_ok()
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();
    
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }
    
    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];
    
    // Initialize first row and column
    for i in 0..=a_len { matrix[i][0] = i; }
    for j in 0..=b_len { matrix[0][j] = j; }
    
    // Fill the matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1       // insertion
                ),
                matrix[i - 1][j - 1] + cost    // substitution
            );
        }
    }
    
    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrum_variants() {
        let resolver = ProtocolResolver::new();
        
        // Test all arbitrum variants normalize to the same thing
        let variants = vec![
            "arbitrum one",
            "arbitrum-one",
            "arbitrumone", 
            "Arbitrum One",
            "ARBITRUM-ONE",
        ];
        
        for variant in variants {
            let normalized = resolver.normalize_protocol(variant).unwrap();
            assert_eq!(normalized, "arbitrum-one", "Failed for variant: {}", variant);
        }
    }
    
    #[test]
    fn test_single_word_protocols() {
        let resolver = ProtocolResolver::new();
        
        assert_eq!(resolver.normalize_protocol("ethereum").unwrap(), "ethereum");
        assert_eq!(resolver.normalize_protocol("Ethereum").unwrap(), "ethereum");
        assert_eq!(resolver.normalize_protocol("ETHEREUM").unwrap(), "ethereum");
    }
    
    #[test]
    fn test_polygon_variants() {
        let resolver = ProtocolResolver::new();
        
        assert_eq!(resolver.normalize_protocol("polygon").unwrap(), "polygon");
        assert_eq!(resolver.normalize_protocol("polygon pos").unwrap(), "polygon");
        assert_eq!(resolver.normalize_protocol("polygon-pos").unwrap(), "polygon");
    }
    
    #[test]
    fn test_unknown_protocol() {
        let resolver = ProtocolResolver::new();
        
        let result = resolver.normalize_protocol("unknown_chain");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown protocol"));
    }
    
    #[test]
    fn test_fuzzy_matching() {
        let resolver = ProtocolResolver::new();
        
        // Small typos should be corrected
        assert_eq!(resolver.normalize_protocol("ethereun").unwrap(), "ethereum");
        assert_eq!(resolver.normalize_protocol("arbitrum on").unwrap(), "arbitrum-one");
    }
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("ethereum", "ethereun"), 1);
        assert_eq!(levenshtein_distance("arbitrum one", "arbitrum on"), 1);
        assert_eq!(levenshtein_distance("test", "test"), 0);
        assert_eq!(levenshtein_distance("", "abc"), 3);
    }
}