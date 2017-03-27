#[derive(Clone, Debug)]
pub struct Shape {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
    pub default_value: Option<Literal>,
}

#[derive(Clone, Debug)]
pub enum Ty {
    Bool,
    Float,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

#[derive(Clone, Debug)]
pub enum Literal {
    Bool(bool),
    Float(f64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}
