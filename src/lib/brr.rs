use std::collections::VecDeque;

pub struct BrrIterator<T: Iterator<Item = u8>> {
    iter: T,
    buffer: VecDeque<i16>,
}

impl<T> BrrIterator<T> 
    where T: Iterator<Item = u8>
{
    pub fn new(iter: T) -> BrrIterator<T> {
        BrrIterator { iter, buffer: VecDeque::new() }
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
            let scalar = header >> 4;
            for i in 0..8 {
                let byte = match self.iter.next() {
                    Some(b) => b,
                    None => break,
                };

                let top = (byte & 0b11110000) >> 4;
                let bottom = byte & 0b00001111;

                self.buffer.push_front(convert_with_sign(top, scalar));
                self.buffer.push_front(convert_with_sign(bottom, scalar));
            }
        }

        self.buffer.pop_back()
    }
}

fn convert_with_sign(mut nib: u8, scalar: u8) -> i16 {
    // Test if msb of nibble is set.
    let negative = (nib | 0b11110111) == 0xFF;

    if negative {
        // get complement
        nib = !nib;
        // zero out upper four bits
        nib = nib & 0b00001111
    }

    let mut shifted = (nib as i16) << scalar;

    if negative {
        // set the complement back
        shifted = !shifted;
    }

    shifted >> 1
}