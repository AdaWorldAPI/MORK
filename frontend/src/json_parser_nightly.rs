use crate::json_parser::*;

impl<'a> Parser<'a> {
    pub fn parse_stream<S, T : ATranscriber<S>>(&mut self, t: &mut T) -> impl std::ops::Coroutine<(), Yield=S, Return=Result<()>> {
        #[coroutine] move || {
        let mut stack = Vec::with_capacity(3);
        let mut ch = expect_byte_ignore_whitespace!(self);
        t.begin();

        'parsing: loop {
            match ch {
                b'[' => {
                    ch = expect_byte_ignore_whitespace!(self);

                    if ch != b']' {
                        if stack.len() == DEPTH_LIMIT {
                            return Err(Error::ExceededDepthLimit);
                        }
                        t.descend_index(0, true);
                        stack.push(StackBlock::Index(0));
                        continue 'parsing;
                    }

                    for k in t.write_empty_array() { yield k }
                },
                b'{' => {
                    ch = expect_byte_ignore_whitespace!(self);

                    if ch != b'}' {
                        if stack.len() == DEPTH_LIMIT {
                            return Err(Error::ExceededDepthLimit);
                        }

                        if ch != b'"' {
                            return self.unexpected_character()
                        }

                        let k = expect_string!(self);
                        t.descend_key(k, true);

                        expect!(self, b':');

                        stack.push(StackBlock::Key(k));

                        ch = expect_byte_ignore_whitespace!(self);

                        continue 'parsing;
                    }

                    for s in t.write_empty_object() { yield s };
                },
                b'"' => {
                    let s = expect_string!(self);
                    for si in t.write_string(s) { yield si }
                },
                b'0' => {
                    let mut mantissa = 0; let mut exponent = 0;
                    if !self.is_eof() {
                        let ch = self.read_byte();
                        allow_number_extensions!(self, mantissa, exponent, ch);
                    }
                    for si in t.write_number(false, mantissa, exponent) { yield si }
                },
                b'1' ..= b'9' => {
                    let mut _mantissa = 0; let mut exponent = 0;
                    expect_number!(self, _mantissa, exponent, ch);
                    for si in t.write_number(false, _mantissa, exponent) { yield si }
                },
                b'-' => {
                    let ch = expect_byte!(self);
                    match ch {
                        b'0' => {
                            let mut mantissa = 0; let mut exponent = 0;
                            if !self.is_eof() {
                                let ch = self.read_byte();
                                allow_number_extensions!(self, mantissa, exponent, ch);
                            }
                            for si in t.write_number(true, mantissa, exponent) { yield si }
                        },
                        b'1' ..= b'9' => {
                            let mut _mantissa = 0; let mut exponent = 0;
                            expect_number!(self, _mantissa, exponent, ch);
                            for si in t.write_number(true, _mantissa, exponent) { yield si }
                        },
                        _    => return self.unexpected_character()
                    };
                }
                b't' => {
                    expect_sequence!(self, b'r', b'u', b'e');
                    for si in t.write_true() { yield si }
                },
                b'f' => {
                    expect_sequence!(self, b'a', b'l', b's', b'e');
                    for si in t.write_false() { yield si }
                },
                b'n' => {
                    expect_sequence!(self, b'u', b'l', b'l');
                    for si in t.write_null() { yield si }
                },
                _    => return self.unexpected_character()
            };

            'popping: loop {
                match stack.last_mut() {
                    None => {
                        expect_eof!(self);
                        t.end();
                        return Ok(());
                    },

                    Some(&mut StackBlock::Index(ref mut cnt)) => {
                        ch = expect_byte_ignore_whitespace!(self);

                        match ch {
                            b',' => {
                                ch = expect_byte_ignore_whitespace!(self);
                                t.ascend_index(*cnt, false);
                                *cnt += 1;
                                t.descend_index(*cnt, false);
                                continue 'parsing;
                            },
                            b']' => { t.ascend_index(*cnt, true); },
                            _    => return self.unexpected_character()
                        }
                    },

                    Some(&mut StackBlock::Key(ref mut key)) => {
                        ch = expect_byte_ignore_whitespace!(self);

                        match ch {
                            b',' => {
                                t.ascend_key(key, false);
                                expect!(self, b'"');
                                let k = expect_string!(self);
                                t.descend_key(k, false);
                                *key = k;
                                expect!(self, b':');

                                ch = expect_byte_ignore_whitespace!(self);

                                continue 'parsing;
                            },
                            b'}' => { t.ascend_key(key, true); },
                            _    => return self.unexpected_character()
                        }
                    }
                }

                match stack.pop() {
                    Some(_) => {},
                    None => break 'popping
                }
            }
        }
    }
    }
}
