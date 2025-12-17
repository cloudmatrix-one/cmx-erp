use sheet::Sheet;

pub mod sheet;
pub mod axis;


pub struct Book {
    pub sheets: Vec<Sheet>,
}

pub struct WorkSpace {
    pub books: Vec<Book>,
}
