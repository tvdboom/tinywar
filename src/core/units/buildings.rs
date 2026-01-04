use strum_macros::EnumIter;

#[derive(EnumIter, Debug)]
pub enum Building {
    Barracks,
    Castle,
}
