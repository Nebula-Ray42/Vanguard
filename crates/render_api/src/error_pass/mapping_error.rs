#[derive(Debug)]
pub enum MappingError {
    MissingTransform,
    MissingPosition,
    MissingRotation,
    MissingScale,
}