use std::collections::VecDeque;

pub struct BrrIterator<T: Iterator<Item = u8>> {
    iter: T,
    buffer: VecDeque<i16>,
    scalar: u8,
}

impl<T> BrrIterator<T> 
    where T: Iterator<Item = u8>
{
    pub fn new(iter: T) -> BrrIterator<T> {
        BrrIterator { iter, buffer: VecDeque::new(), scalar: 0 }
    }

    pub fn get_scalar(&self) -> u8 {
        self.scalar
    }
}

impl<T> Iterator for BrrIterator<T>
    where 
        T: Iterator<Item = u8> 
{
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            let header = match self.iter.next() {
                Some(b) => b,
                None => return None,
            };

            // 4 MSBs of "header" byte are the scalar to shift the BRR by
            self.scalar = header >> 4;
            for _i in 0..8 {
                let byte = match self.iter.next() {
                    Some(b) => b,
                    None => break,
                };

                let top = (byte & 0b11110000) >> 4;
                let bottom = byte & 0b00001111;

                self.buffer.push_front(convert_with_sign(top, self.scalar));
                self.buffer.push_front(convert_with_sign(bottom, self.scalar));
            }
        }

        self.buffer.pop_back()
    }
}

#[allow(overflowing_literals)]
fn convert_with_sign(nib: u8, scalar: u8) -> i16 {
    let mut signed = nib as i8;

    // if the nibble is negative (msb is set)
    if (signed & 0b00001000) != 0 {
        // make the entire number negative
        signed |= 0b11110000;
    }

    (signed as i16) * 2i16.pow(scalar as u32 - 1)
}