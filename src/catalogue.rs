use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Pollutant {
    pub id: String,
    pub label: String,
    pub name: String,
    pub unit: String,
    pub groups: Vec<String>,
    pub aliases: Vec<String>,
    pub describe: String,
}

#[derive(Clone, Deserialize)]
pub struct Group {
    pub label: String,
    pub describe: String,
    pub members: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub struct Column {
    pub name: String,
    pub maps_to: Option<String>,
    pub samples: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub description: String,
    pub publisher: String,
    pub updated: i32,
    pub levels: HashMap<String, String>,
    pub coverage: HashMap<String, Vec<String>>,
    pub pollutants: Vec<String>,
    pub rows: HashMap<String, HashMap<String, i32>>,
    pub columns: Vec<Column>,
}

#[derive(Clone, Deserialize)]
pub struct Catalogue {
    pub summary: String,
    pub years: Vec<i32>,
    pub dimensions: HashMap<String, Vec<String>>,
    pub default_level: HashMap<String, String>,
    pub groups: HashMap<String, Group>,
    pub pollutants: Vec<Pollutant>,
    pub members: HashMap<String, Vec<String>>,
    pub parents: HashMap<String, HashMap<String, String>>,
    pub aliases: HashMap<String, String>,
    pub level_terms: HashMap<String, Vec<String>>,
    pub sources: Vec<Source>,
}

impl Catalogue {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("catalogue json: {e}"))
    }

    pub fn pollutant(&self, id: &str) -> &Pollutant {
        self.pollutants
            .iter()
            .find(|p| p.id == id)
            .expect("pollutant")
    }

    pub fn source(&self, id: &str) -> &Source {
        self.sources.iter().find(|s| s.id == id).expect("source")
    }

    pub fn dim_levels(&self, dim: &str) -> &[String] {
        &self.dimensions[dim]
    }

    pub fn members_of(&self, dim: &str, level: &str) -> &[String] {
        &self.members[&format!("{dim}.{level}")]
    }

    pub fn parent(&self, dim: &str, level: &str, member: &str) -> Option<&str> {
        self.parents
            .get(&format!("{dim}.{level}"))?
            .get(member)
            .map(String::as_str)
    }

    pub fn roll_up(&self, dim: &str, level: &str, member: &str, target: &str) -> Option<String> {
        let levels = self.dim_levels(dim);
        let i = levels.iter().position(|l| l == level)?;
        let j = levels.iter().position(|l| l == target)?;
        if i > j {
            return None;
        }
        let mut current = Some(member.to_string());
        for k in i..j {
            current = current.and_then(|m| self.parent(dim, &levels[k], &m).map(str::to_string));
        }
        current
    }

    pub fn level_rolls_up(&self, dim: &str, source_level: &str, target_level: &str) -> bool {
        let levels = self.dim_levels(dim);
        let i = levels.iter().position(|l| l == source_level);
        let j = levels.iter().position(|l| l == target_level);
        matches!((i, j), (Some(a), Some(b)) if a <= b)
    }

    pub fn plural(&self, level: &str, n: usize) -> String {
        let word = if n == 1 {
            level.to_string()
        } else if let Some(stem) = level.strip_suffix('y') {
            format!("{stem}ies")
        } else {
            format!("{level}s")
        };
        format!("{n} {word}")
    }

    pub fn order_of(&self, id: &str) -> usize {
        self.pollutants
            .iter()
            .position(|p| p.id == id)
            .unwrap_or(usize::MAX)
    }
}
