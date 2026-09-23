
use core::fmt;

const UART0_BASE: usize = 0x1000_0000;

/* register offsets */
const UART_THR: usize = 0x00; // Transmit Holding Register (write)
const UART_LSR: usize = 0x05; // Line status register (read)
const UART_LSR_THRE: u8 = 0x20; // Transmitter holding the register empty 

pub struct Uart;

impl Uart {
    pub const fn new() -> Self {
        Uart
    }

    /*Wait for the TX FIFO to have space, then send one byte*/ 
    pub fn putc(&self, c: u8) {
        unsafe {
            // Spin until the UART is ready to accept a new character. 
            while (core::ptr::read_volatile((UART0_BASE + UART_LSR) as *const u8) & UART_LSR_THRE) == 0 {} 
            core::ptr::write_volatile((UART0_BASE + UART_THR) as *mut u8, c);
        }
    }

    pub fn write_bytes(&self, data: &[u8]) {
        for &b in data {
            self.putc(b);
        }
    }

}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            self.putc(b);
        }
        Ok(())
    }

}
