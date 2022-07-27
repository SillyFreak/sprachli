use std::fmt::{self, Write};

pub trait FormatterExt<'a> {
    fn debug_sexpr<'b>(&'b mut self) -> DebugSexpr<'b, 'a>;
}

impl<'a> FormatterExt<'a> for fmt::Formatter<'a> {
    fn debug_sexpr<'b>(&'b mut self) -> DebugSexpr<'b, 'a> {
        DebugSexpr::new(self)
    }
}

pub struct DebugSexpr<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    first: bool,
}

impl<'a, 'b: 'a> DebugSexpr<'a, 'b> {
    fn new(fmt: &'a mut fmt::Formatter<'b>) -> Self {
        let result = fmt.write_str("(");
        Self {
            fmt,
            result,
            first: true,
        }
    }

    pub fn raw_item<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut dyn fmt::Write) -> fmt::Result,
    {
        self.result = self.result.and_then(|_| {
            if self.is_pretty() {
                let mut write = SexprPad::new(self.fmt);
                if !self.first {
                    write.write_str("\n")?;
                }
                f(&mut write)
            } else {
                if !self.first {
                    self.fmt.write_str(" ")?;
                }
                f(self.fmt)
            }
        });

        self.first = false;
        self
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.raw_item(|f| f.write_str(name))
    }

    pub fn names<I>(&mut self, values: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        for value in values.into_iter() {
            self.name(value.as_ref());
        }
        self
    }

    pub fn item(&mut self, value: &dyn fmt::Debug) -> &mut Self {
        let alternate = self.fmt.alternate();
        self.raw_item(|f| {
            if alternate {
                f.write_fmt(format_args!("{value:#?}"))
            } else {
                f.write_fmt(format_args!("{value:?}"))
            }
        })
    }

    pub fn items<I>(&mut self, values: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: fmt::Debug,
    {
        for value in values.into_iter() {
            self.item(&value);
        }
        self
    }

    pub fn finish(&mut self) -> fmt::Result {
        self.result = self.result.and_then(|_| self.fmt.write_str(")"));
        self.result
    }

    fn is_pretty(&self) -> bool {
        self.fmt.alternate()
    }
}

struct SexprPad<'a, 'b> {
    fmt: &'a mut fmt::Formatter<'b>,
    on_newline: bool,
}

impl<'a, 'b> SexprPad<'a, 'b> {
    fn new(fmt: &'a mut fmt::Formatter<'b>) -> Self {
        Self {
            fmt,
            on_newline: false,
        }
    }
}

impl fmt::Write for SexprPad<'_, '_> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            if self.on_newline {
                self.fmt.write_str(" ")?;
            }

            let split = match s.find('\n') {
                Some(pos) => {
                    self.on_newline = true;
                    pos + 1
                }
                None => {
                    self.on_newline = false;
                    s.len()
                }
            };
            self.fmt.write_str(&s[..split])?;
            s = &s[split..];
        }

        Ok(())
    }
}
