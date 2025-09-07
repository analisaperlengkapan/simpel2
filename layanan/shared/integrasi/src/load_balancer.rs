pub struct LoadBalancer;

impl LoadBalancer {
    pub fn new() -> Self {
        Self
    }

    pub fn get_next_instance(&self, _service_name: &str) -> Option<String> {
        // TODO: Implement load balancing
        None
    }
}
