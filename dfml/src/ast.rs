use base::Span;

pub struct ShapeDef<'src> {
    name: &'src str,
    params: Vec<Param<'src>>,
}

struct Param<'src> {
    span: Span,
    name: &'src str,
    ty: Ty<'src>,
}

struct Ty<'src> {
    span: Span,
    name: &'src str,
}
