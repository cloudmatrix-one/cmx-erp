use crate::model::book::axis::Axis;


pub struct Head {

}

impl Axis for Head {
    fn get_name(&self) -> Option<String> {
        None
    }

    fn get_index(&self) -> u32 {
        0
    }

    fn get_values(&self) -> Vec<String> {
        Vec::new()
    }
}