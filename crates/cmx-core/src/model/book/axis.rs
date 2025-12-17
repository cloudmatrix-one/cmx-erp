
pub trait Axis {
    fn get_name(&self) -> Option<String>;
    fn get_index(&self) -> u32;
    fn get_values(&self) -> Vec<String>;
}