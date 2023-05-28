use std::path::PathBuf;

use wasmer::{MemoryError, MemoryType, Pages, Tunables};

/// Represents input data for the code.
#[derive(Debug, Clone)]
pub enum InputData {
    /// Stdin will be read from the given file.
    File(PathBuf),
    /// Stdin will be read from the given string.
    String(String),
    /// Stdin will be ignored.
    Ignore,
}

/// Limiting tunables for wasm runtime.
/// This allows to limit the resources used by the code.
pub struct LimitingTunables<T: Tunables> {
    /// Maximum amount of memory that can be used by the code.
    /// It is provided in pages, where each page is 64KiB.
    limit: Pages,
    /// The base implementation.
    base: T,
}

impl<T: Tunables> LimitingTunables<T> {
    /// Creates new limiting tunables.
    pub fn new(limit: Pages, base: T) -> Self {
        Self { limit, base }
    }

    fn adjust_memory(&self, requested: &MemoryType) -> MemoryType {
        let mut adjusted = requested.clone();
        if requested.maximum.is_none() {
            adjusted.maximum = Some(self.limit);
        }
        adjusted
    }

    /// Ensures that the memory limit is not exceeded.
    fn validate_memory(&self, memory: &MemoryType) -> Result<(), MemoryError> {
        if memory.minimum > self.limit {
            return Err(MemoryError::Generic(
                "Minimum memory exceeds the limit".to_string(),
            ));
        }

        if let Some(maximum) = memory.maximum {
            if maximum > self.limit {
                return Err(MemoryError::Generic(
                    "Maximum memory exceeds the limit".to_string(),
                ));
            }
        } else {
            return Err(MemoryError::Generic(
                "Maxiumum memory is not specified".to_string(),
            ));
        }

        Ok(())
    }
}

impl<T: Tunables> Tunables for LimitingTunables<T> {
    fn memory_style(&self, memory: &MemoryType) -> wasmer::vm::MemoryStyle {
        let adjusted = self.adjust_memory(memory);
        self.base.memory_style(&adjusted)
    }

    fn table_style(&self, table: &wasmer::TableType) -> wasmer::vm::TableStyle {
        self.base.table_style(table)
    }

    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &wasmer::vm::MemoryStyle,
    ) -> Result<wasmer::vm::VMMemory, MemoryError> {
        let adjusted = self.adjust_memory(ty);
        self.validate_memory(&adjusted)?;
        self.base.create_host_memory(&adjusted, style)
    }

    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &wasmer::vm::MemoryStyle,
        vm_definition_location: std::ptr::NonNull<wasmer::vm::VMMemoryDefinition>,
    ) -> Result<wasmer::vm::VMMemory, MemoryError> {
        let adjusted = self.adjust_memory(ty);
        self.validate_memory(&adjusted)?;
        self.base
            .create_vm_memory(&adjusted, style, vm_definition_location)
    }

    fn create_host_table(
        &self,
        ty: &wasmer::TableType,
        style: &wasmer::vm::TableStyle,
    ) -> Result<wasmer::vm::VMTable, String> {
        self.base.create_host_table(ty, style)
    }

    unsafe fn create_vm_table(
        &self,
        ty: &wasmer::TableType,
        style: &wasmer::vm::TableStyle,
        vm_definition_location: std::ptr::NonNull<wasmer::vm::VMTableDefinition>,
    ) -> Result<wasmer::vm::VMTable, String> {
        self.base.create_vm_table(ty, style, vm_definition_location)
    }
}
