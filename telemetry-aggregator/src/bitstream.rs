pub struct BitWriter {
    buffer: Vec<u64>,
    current: u64,
    bits_in_current: u8,
}

impl BitWriter {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            current: 0,
            bits_in_current: 0,
        }
    }

    pub fn write_bit(&mut self, bit: bool) {
        if bit {
            self.current |= 1u64 << (63 - self.bits_in_current);
        }
        self.bits_in_current += 1;
        if self.bits_in_current == 64 {
            self.buffer.push(self.current);
            self.current = 0;
            self.bits_in_current = 0;
        }
    }

    pub fn write_bits(&mut self, value: u64, num_bits: u8) {
        if num_bits == 0 {
            return;
        }
        let shifted = value << (64 - num_bits);
        let remaining = 64 - self.bits_in_current;

        if num_bits <= remaining {
            self.current |= shifted >> self.bits_in_current;
            self.bits_in_current += num_bits;
            if self.bits_in_current == 64 {
                self.buffer.push(self.current);
                self.current = 0;
                self.bits_in_current = 0;
            }
        } else {
            let first_part = shifted >> self.bits_in_current;
            self.current |= first_part;
            self.buffer.push(self.current);
            let second_bits = num_bits - remaining;
            self.current = shifted << remaining >> (64 - second_bits);
            self.bits_in_current = second_bits;
        }
    }

    pub fn finish(&mut self) -> &[u64] {
        if self.bits_in_current > 0 {
            self.buffer.push(self.current);
            self.current = 0;
            self.bits_in_current = 0;
        }
        &self.buffer
    }

    pub fn finish_bytes(&mut self) -> Vec<u8> {
        let words = self.finish();
        let mut bytes = Vec::with_capacity(words.len() * 8);
        for &w in words {
            bytes.extend_from_slice(&w.to_be_bytes());
        }
        bytes
    }

    pub fn bit_count(&self) -> usize {
        self.buffer.len() * 64 + self.bits_in_current as usize
    }
}

pub struct BitReader<'a> {
    pub words: &'a [u64],
    pub word_idx: usize,
    pub bits_consumed: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(words: &'a [u64]) -> Self {
        Self { words, word_idx: 0, bits_consumed: 0 }
    }

    pub fn new_from_bytes(bytes: &'a [u8]) -> Self {
        let word_count = bytes.len() / 8;
        let words_ptr = bytes.as_ptr() as *const u64;
        let words = unsafe { std::slice::from_raw_parts(words_ptr, word_count) };
        Self::new(words)
    }

    pub fn read_bit(&mut self) -> bool {
        if self.word_idx >= self.words.len() {
            return false;
        }
        let bit = (self.words[self.word_idx] >> (63 - self.bits_consumed)) & 1 == 1;
        self.bits_consumed += 1;
        if self.bits_consumed == 64 {
            self.word_idx += 1;
            self.bits_consumed = 0;
        }
        bit
    }

    pub fn read_bits(&mut self, num_bits: u8) -> u64 {
        if num_bits == 0 || self.word_idx >= self.words.len() {
            return 0;
        }
        if num_bits >= 64 {
            if self.bits_consumed == 0 {
                let val = self.words[self.word_idx];
                self.word_idx += 1;
                return val;
            }
            let remaining = 64 - self.bits_consumed;
            let first = self.words[self.word_idx] & ((1u64 << remaining).wrapping_sub(1));
            self.word_idx += 1;
            self.bits_consumed = 0;
            if self.word_idx >= self.words.len() {
                return first;
            }
            let second = self.words[self.word_idx];
            self.word_idx += 1;
            return (first << (num_bits - remaining)) | second;
        }

        let remaining = 64 - self.bits_consumed;
        if num_bits <= remaining {
            let mask = (1u64 << num_bits).wrapping_sub(1) << (remaining - num_bits);
            let value = (self.words[self.word_idx] & mask) >> (remaining - num_bits);
            self.bits_consumed += num_bits;
            if self.bits_consumed == 64 {
                self.word_idx += 1;
                self.bits_consumed = 0;
            }
            value
        } else {
            let first_part = (self.words[self.word_idx] & ((1u64 << remaining) - 1)) as u64;
            self.word_idx += 1;
            self.bits_consumed = 0;
            let second_bits = num_bits - remaining;

            if self.word_idx >= self.words.len() {
                return first_part << second_bits;
            }

            let second_part = self.words[self.word_idx] >> (64 - second_bits);
            self.bits_consumed = second_bits;
            (first_part << second_bits) | second_part
        }
    }
}
