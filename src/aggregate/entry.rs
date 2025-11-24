///
pub enum Entry<Occupied, Vacant> {
    ///
    Occupied(Occupied),

    ///
    Vacant(Vacant),
}

///
pub trait VacantEntry {
    ///
    type Key;

    ///
    type Value;

    ///
    fn key(&self) -> &Self::Key;

    ///
    fn insert(self, value: Self::Value);
}

///
pub trait OccupiedEntry {
    ///
    type Key;

    ///
    type Value;

    ///
    fn key(&self) -> &Self::Key;

    ///
    fn value(&self) -> &Self::Value;

    ///
    fn value_mut(&mut self) -> &mut Self::Value;
}
