use std::fmt;

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
        F: FnOnce(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self.result = self.result.and_then(|_| {
            if !self.first {
                self.fmt.write_str(" ")?;
            }
            f(self.fmt)
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
        self.raw_item(|f| value.fmt(f))
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
