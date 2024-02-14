pub struct Diagnostic {
    err: Option<syn::Error>,
}

impl Diagnostic {
    pub fn new() -> Self {
        Diagnostic { err: None }
    }

    pub fn push(&mut self, err: syn::Error) {
        if let Some(prev) = &mut self.err {
            prev.combine(err);
        } else {
            self.err = Some(err);
        }
    }

    pub fn take(&mut self) -> Option<syn::Error> {
        self.err.take()
    }
}
