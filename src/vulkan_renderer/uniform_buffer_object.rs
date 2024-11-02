type Mat4 = linear_algebra::Matrix<f32, 4, 4>;

#[repr(C)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}
