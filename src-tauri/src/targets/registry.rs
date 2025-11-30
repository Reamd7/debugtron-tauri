use super::local::LocalTargetAdapter;
use super::types::TargetAdapter;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TargetRegistry {
    targets: HashMap<String, Arc<dyn TargetAdapter>>,
}

impl TargetRegistry {
    pub fn new() -> Self {
        Self {
            targets: HashMap::new(),
        }
    }

    pub fn register(&mut self, target: Arc<dyn TargetAdapter>) {
        self.targets.insert(target.get_id(), target);
    }

    pub fn unregister(&mut self, target_id: &str) {
        self.targets.remove(target_id);
    }

    pub fn get_all(&self) -> Vec<Arc<dyn TargetAdapter>> {
        self.targets.values().cloned().collect()
    }

    pub fn get_by_id(&self, target_id: &str) -> Option<Arc<dyn TargetAdapter>> {
        self.targets.get(target_id).cloned()
    }

    pub async fn initialize_local_targets(&mut self) -> Result<()> {
        let local_adapter = LocalTargetAdapter::new();
        self.register(Arc::new(local_adapter));
        Ok(())
    }
}
