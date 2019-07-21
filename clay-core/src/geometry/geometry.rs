use crate::Pack;


/// Shape of an object.
/// It defines the search of the point where ray intersects this shape.
pub trait Geometry: Pack {
    /// Associated OpenCL code that contains necessary function definition.
    fn ocl_hit_code() -> String;
    /// Name of the function from the code that is used to find an intersection.
    fn ocl_hit_fn() -> String;
}
