use super::*;

pub struct ASpaceTranscriber<'a, 'c> { pub(super) count: usize, pub(super) wz: &'c mut Vec<u8>, pub(super) pdp: ParDataParser<'a> }
impl <'a, 'c> ASpaceTranscriber<'a, 'c> {
    #[inline(always)] fn write<S : AsRef<[u8]>>(&mut self, s: S) -> impl Iterator<Item=&'static [u8]> {
        gen move {
            let token = self.pdp.tokenizer(s.as_ref());
            self.wz.push(item_byte(Tag::SymbolSize(token.len() as u8)));
            self.wz.extend_from_slice(token);
            yield unsafe { std::mem::transmute(&self.wz[..]) };
            self.wz.truncate(self.wz.len() - (token.len() + 1));
        }
    }
    pub(super) fn destruct(self) -> (usize, &'c mut Vec<u8>, ParDataParser<'a>) {
        (self.count, self.wz, self.pdp)
    }
}
impl <'a, 'c> mork_frontend::json_parser::ATranscriber<&'static [u8]> for ASpaceTranscriber<'a, 'c> {
    #[inline(always)] fn descend_index(&mut self, i: usize, first: bool) -> () {
        if first { self.wz.push(item_byte(Tag::Arity(2))); }
        let token = self.pdp.tokenizer(i.to_string().as_bytes());
        self.wz.push(item_byte(Tag::SymbolSize(token.len() as u8)));
        self.wz.extend_from_slice(token);
    }
    #[inline(always)] fn ascend_index(&mut self, i: usize, last: bool) -> () {
        self.wz.truncate(self.wz.len() - (self.pdp.tokenizer(i.to_string().as_bytes()).len() + 1));
        if last { self.wz.truncate(self.wz.len() - 1); }
    }
    #[inline(always)] fn write_empty_array(&mut self) -> impl Iterator<Item=&'static [u8]> { self.count += 1; self.write("[]") }
    #[inline(always)] fn descend_key(&mut self, k: &str, first: bool) -> () {
        if first { self.wz.push(item_byte(Tag::Arity(2))); }
        let token = self.pdp.tokenizer(k.as_bytes());
        self.wz.push(item_byte(Tag::SymbolSize(token.len() as u8)));
        self.wz.extend_from_slice(token);
    }
    #[inline(always)] fn ascend_key(&mut self, k: &str, last: bool) -> () {
        let token = self.pdp.tokenizer(k.as_bytes());
        self.wz.truncate(self.wz.len() - (token.len() + 1));
        if last { self.wz.truncate(self.wz.len() - 1); }
    }
    #[inline(always)] fn write_empty_object(&mut self) -> impl Iterator<Item=&'static [u8]> { self.count += 1; self.write("{}") }
    #[inline(always)] fn write_string(&mut self, s: &str) -> impl Iterator<Item=&'static [u8]> { self.count += 1; self.write(s) }
    #[inline(always)] fn write_number(&mut self, negative: bool, mantissa: u64, exponent: i16) -> impl Iterator<Item=&'static [u8]> {
        let mut buf = [0u8; 64];
        let mut cur = std::io::Cursor::new(&mut buf[..]);
        if negative { write!(cur, "-").unwrap(); }
        write!(cur, "{}", mantissa).unwrap();
        if exponent != 0 { write!(cur, "e{}", exponent).unwrap(); }
        let len = cur.position() as usize;
        self.count += 1;
        self.write(unsafe { std::mem::transmute::<_, &'static [u8]>(&cur.into_inner()[..len]) })
    }
    #[inline(always)] fn write_true(&mut self) -> impl Iterator<Item=&'static [u8]> { self.count += 1; self.write("true") }
    #[inline(always)] fn write_false(&mut self) -> impl Iterator<Item=&'static [u8]> { self.count += 1; self.write("false") }
    #[inline(always)] fn write_null(&mut self) -> impl Iterator<Item=&'static [u8]> { self.count += 1; self.write("null") }
    #[inline(always)] fn begin(&mut self) -> () {}
    #[inline(always)] fn end(&mut self) -> () {}
}
