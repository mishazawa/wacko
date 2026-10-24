use core::cell::RefCell;
use embassy_crypto::{Aes128, driver::Rng};
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};

struct State {
    cipher: Aes128,
    counter: u128,
}

static STATE: Mutex<CriticalSectionRawMutex, RefCell<Option<State>>> =
    Mutex::new(RefCell::new(None));

/// seed: 16 bytes of key + 16 bytes of starting counter, from the hardware RNG.
pub fn seed_crypto_rng(seed: [u8; 32]) {
    let key: [u8; 16] = seed[..16].try_into().unwrap();
    let counter = u128::from_le_bytes(seed[16..].try_into().unwrap());
    STATE.lock(|s| {
        *s.borrow_mut() = Some(State {
            cipher: Aes128::new(&key),
            counter,
        });
    });
}

pub struct SwRng;

impl Rng for SwRng {
    fn fill_bytes(buf: &mut [u8]) {
        STATE.lock(|s| {
            let mut s = s.borrow_mut();
            let st = s.as_mut().expect("crypto rng not seeded");

            // Output: AES(counter) blocks.
            for chunk in buf.chunks_mut(16) {
                let mut block = st.counter.to_le_bytes();
                st.counter = st.counter.wrapping_add(1);
                st.cipher.encrypt_block(&mut block);
                chunk.copy_from_slice(&block[..chunk.len()]);
            }

            // Rekey after every request, so earlier output can't be
            // recomputed from the current state.
            let mut next = [0u8; 16];
            let mut b = st.counter.to_le_bytes();
            st.counter = st.counter.wrapping_add(1);
            st.cipher.encrypt_block(&mut b);
            next.copy_from_slice(&b);
            st.cipher = Aes128::new(&next);
        });
    }
}

embassy_crypto::rng_impl!(SwRng);
