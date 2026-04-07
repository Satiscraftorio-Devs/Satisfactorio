use crate::common::geometry::direction::Direction;

const VISITED_SHIFT: u64 = 63;
const BLOCK_ID_SHIFT: u64 = 31;
const BLOCK_ID_MASK: u64 = 0xFFFF_FFFF;
const FACE_MASK: u64 = 0b111;
const AO_SHIFT: u64 = 23;
const AO_MASK: u64 = 0xFF; // 8 bits

#[derive(Clone, Copy)]
pub struct FaceMask {
    pub data: u64,
}

impl FaceMask {
    #[inline(always)]
    pub const fn empty() -> FaceMask {
        return FaceMask {
            data: 0x8000_0000_0000_0000u64,
        };
    }

    pub fn from(visited: bool, id: u32, face: Direction, ao: u8) -> FaceMask {
        let mut mask = FaceMask::empty();
        mask.set_visited(visited);
        mask.set_block_id(id);
        mask.set_face(face);
        mask.set_ao(ao);
        mask
    }

    pub fn to(&self) -> (bool, u32, Direction, u8) {
        (self.get_visited(), self.get_block_id(), self.get_face(), self.get_ao())
    }

    #[inline(always)]
    pub fn get_visited(self) -> bool {
        (self.data >> VISITED_SHIFT) != 0
    }

    #[inline(always)]
    pub fn set_visited(&mut self, v: bool) {
        self.data ^= (-(v as i64) as u64 ^ self.data) & (1 << VISITED_SHIFT);
    }

    #[inline(always)]
    pub fn get_block_id(self) -> u32 {
        ((self.data >> BLOCK_ID_SHIFT) & BLOCK_ID_MASK) as u32
    }

    #[inline(always)]
    pub fn set_block_id(&mut self, id: u32) {
        self.data = (self.data & !(BLOCK_ID_MASK << BLOCK_ID_SHIFT)) | ((id as u64) << BLOCK_ID_SHIFT);
    }

    #[inline(always)]
    pub fn get_face(self) -> Direction {
        Direction::from_bits_unchecked((self.data & FACE_MASK) as u8)
    }

    #[inline(always)]
    pub fn set_face(&mut self, face: Direction) {
        self.data = (self.data & !FACE_MASK) | (face as u64);
    }

    #[inline(always)]
    pub fn set_ao(&mut self, ao: u8) {
        self.data = (self.data & !(AO_MASK << AO_SHIFT)) | (((ao as u64) & AO_MASK) << AO_SHIFT);
    }

    #[inline(always)]
    pub fn get_ao(self) -> u8 {
        ((self.data >> AO_SHIFT) & AO_MASK) as u8
    }

    #[inline(always)]
    pub fn get_ao_corner(self, corner: u8) -> u8 {
        let ao = self.get_ao();
        (ao >> (corner * 2)) & 0b11
    }

    #[inline(always)]
    pub fn set_ao_corner(&mut self, corner: u8, value: u8) {
        let shift = corner * 2;
        let mut ao = self.get_ao();

        ao &= !(0b11 << shift);
        ao |= (value & 0b11) << shift;

        self.set_ao(ao);
    }

    #[inline(always)]
    pub fn can_merge_with(&self, other: &FaceMask) -> bool {
        self.get_block_id() == other.get_block_id() && self.get_face() == other.get_face()
    }
}
