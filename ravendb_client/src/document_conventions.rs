#[derive(Clone, Debug)]
pub struct DocumentConventions {
    disable_topology_updates: bool,
    send_application_identified: bool,
}

//TODO: Remove this when default can no longer be derived
#[allow(clippy::derivable_impls)]
impl Default for DocumentConventions {
    fn default() -> Self {
        Self {
            disable_topology_updates: bool::default(),
            send_application_identified: bool::default(),
        }
    }
}
// Mutators
impl DocumentConventions {
    pub fn default_for_single_server() -> Self {
        Self {
            send_application_identified: false,
            ..Default::default()
        }
    }
}

// Getters
impl DocumentConventions {
    pub fn topology_updates_disabled(&self) -> bool {
        self.disable_topology_updates
    }
}
