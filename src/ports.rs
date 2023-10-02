use core::marker::PhantomData;
use x86_64::instructions::port::Port;

pub struct PortDefinition<T> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T> PortDefinition<T> {
    const fn new(port: u16) -> PortDefinition<T> {
        PortDefinition { port, phantom: PhantomData }
    }

    pub fn get_port(self) -> Port<T> {
        Port::new(self.port)
    }
}

// port must match isa-debug-exit configuration in Cargo.toml
pub const ISA_DEBUG_EXIT: PortDefinition<u32> = PortDefinition::new(0xf4);
