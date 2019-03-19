#[macro_use]
extern crate typedgl_derive;

pub trait IsVertex {
    fn element_size(&self, index: usize) -> gl::types::GLint;
    fn element_type(&self, index: usize) -> VertexElementType;
    fn element_stride(&self) -> gl::types::GLsizei;
    fn element_pointer(&self, index: usize) -> *const std::os::raw::c_void;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VertexElementType{
    UnsignedByte,
    Byte,
    UnsignedShort,
    Short,
    Float,
}

impl VertexElementType{
    pub fn value(&self) -> gl::types::GLenum {
        use VertexElementType::*;
        match self {
            UnsignedByte => gl::UNSIGNED_BYTE,
            Byte => gl::BYTE,
            UnsignedShort => gl::UNSIGNED_SHORT,
            Short => gl::SHORT,
            Float => gl::FLOAT,
        }
    }
}

#[derive(IsVertex)]
struct Vertex{
    pub position: [f32; 3],
    pub texture: [f32; 2],
    pub normal: [f32; 3],
    test: u32,
}

#[test]
fn test_size(){
    let vertex = Vertex{position: [0.0; 3], texture: [0.0; 2], normal: [0.0; 3]};
    assert_eq!(vertex.element_size(0), 3);
    assert_eq!(vertex.element_size(1), 2);
    assert_eq!(vertex.element_size(2), 3);
}

#[test]
fn test_type(){
    let vertex = Vertex{position: [0.0; 3], texture: [0.0; 2], normal: [0.0; 3]};
    assert_eq!(vertex.element_type(0), VertexElementType::Float);
    assert_eq!(vertex.element_type(1), VertexElementType::Float);
    assert_eq!(vertex.element_type(2), VertexElementType::Float);
}

#[test]
fn test_stride(){
    let vertex = Vertex{position: [0.0; 3], texture: [0.0; 2], normal: [0.0; 3]};
    assert_eq!(vertex.element_stride(), 32);
}

#[test]
fn test_pointer(){
    let vertex = Vertex{position: [0.0; 3], texture: [0.0; 2], normal: [0.0; 3]};
    assert_eq!(vertex.element_pointer(0) as usize, 0);
    assert_eq!(vertex.element_pointer(1) as usize, 12);
    assert_eq!(vertex.element_pointer(2) as usize, 20);
}
