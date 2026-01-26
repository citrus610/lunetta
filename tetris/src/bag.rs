use crate::piece::Piece;
use enumset::*;

pub type Bag = EnumSet<Piece>;

pub fn update_bag(bag: &mut Bag, piece: Piece) -> bool {
    if !bag.remove(piece) {
        return false;
    }
    if bag.is_empty() {
        *bag = Bag::all();
    }
    true
}
