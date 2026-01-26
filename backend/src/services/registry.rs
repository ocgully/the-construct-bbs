use super::{Service, example::ExampleService};
use crate::config::Config;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServiceRegistry {
    services: HashMap<String, Arc<dyn Service>>,
}

impl ServiceRegistry {
    pub fn from_config(config: &Config) -> Self {
        let mut services = HashMap::new();

        for service_config in &config.services {
            if !service_config.enabled {
                continue;
            }

            // Service factory - match service name to implementation
            let service: Arc<dyn Service> = match service_config.name.as_str() {
                "example" => Arc::new(ExampleService),
                name => {
                    eprintln!("Warning: Unknown service '{}' in config", name);
                    continue;
                }
            };

            services.insert(service_config.name.clone(), service);
        }

        Self { services }
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn Service>> {
        self.services.get(name)
    }

    pub fn list(&self) -> Vec<(&str, &str)> {
        self.services
            .iter()
            .map(|(_, service)| (service.name(), service.description()))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }
}
