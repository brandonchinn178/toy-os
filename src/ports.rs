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

// https://wiki.osdev.org/%228042%22_PS/2_Controller#PS.2F2_Controller_IO_Ports
pub const PS2: PortDefinition<u8> = PortDefinition::new(0x60);
