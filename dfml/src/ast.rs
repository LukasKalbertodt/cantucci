use base::Span;

#[derive(Clone, Debug)]
pub struct ShapeDef<'src> {
    pub name: Ident<'src>,
    pub params: Vec<Param<'src>>,
}

#[derive(Clone, Debug)]
pub struct Param<'src> {
    pub span: Span,
    pub name: Ident<'src>,
    pub ty: Ty<'src>,
}

#[derive(Clone, Debug)]
pub struct Ty<'src> {
    pub name: Ident<'src>,
}

#[derive(Clone, Debug)]
pub struct Ident<'src> {
    pub span: Span,
    pub name: &'src str,
}
