use crate::piece::Piece;
use enumset::*;

pub type Bag = EnumSet<Piece>;

pub fn update_bag(bag: &mut Bag, piece: Piece) {
    *bag -= piece;

    if bag.is_empty() {
        *bag = Bag::all();
    }
}
