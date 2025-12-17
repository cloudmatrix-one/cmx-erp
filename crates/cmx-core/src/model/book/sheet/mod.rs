use area::Area;

pub mod area;

pub struct Sheet {
    pub name: String,
    pub areas: Vec<Area>,
}
