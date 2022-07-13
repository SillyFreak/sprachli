use std::fmt;

pub trait FormatterExt<'a> {
    fn debug_prefixed<'b>(&'b mut self) -> DebugPrefixed<'b, 'a>;
}

impl<'a> FormatterExt<'a> for fmt::Formatter<'a> {
    fn debug_prefixed<'b>(&'b mut self) -> DebugPrefixed<'b, 'a> {
        DebugPrefixed::new(self)
    }
}
pub struct DebugPrefixed<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    first: bool,
}

impl<'a, 'b: 'a> DebugPrefixed<'a, 'b> {
    fn new(fmt: &'a mut fmt::Formatter<'b>) -> Self {
        let result = fmt.write_str("(");
        Self {
            fmt,
            result,
            first: true,
        }
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.result = self.result.and_then(|_| {
            if !self.first {
                self.fmt.write_str(" ")?;
            }
            self.fmt.write_str(name)
        });

        self.first = false;
        self
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
        self.result = self.result.and_then(|_| {
            if !self.first {
                self.fmt.write_str(" ")?;
            }
            value.fmt(self.fmt)
        });

        self.first = false;
        self
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
