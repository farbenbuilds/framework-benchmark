use crate::container::stop_container;
use crate::error::OrchestratorError;
use std::path::Path;

pub(crate) struct ContainerGuard<'a> {
    framework_dir: &'a Path,
    name: String,
    armed: bool,
}

impl<'a> ContainerGuard<'a> {
    pub(crate) fn new(framework_dir: &'a Path, name: &str) -> Self {
        Self {
            framework_dir,
            name: name.into(),
            armed: true,
        }
    }

    pub(crate) fn stop(&mut self) -> Result<(), OrchestratorError> {
        self.armed = false;
        stop_container(self.framework_dir, &self.name)
    }
}

impl Drop for ContainerGuard<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let _ = stop_container(self.framework_dir, &self.name);
    }
}
