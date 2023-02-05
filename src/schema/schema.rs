// RGB Core Library: a reference implementation of RGB smart contract standards.
// Written in 2019-2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// To the extent possible under law, the author(s) have dedicated all copyright
// and related and neighboring rights to this software to the public domain
// worldwide. This software is distributed without any warranty.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::cmp::Ordering;
use std::io;

use amplify::confinement::{MediumVec, TinyOrdMap, TinyOrdSet};
use amplify::flags::FlagVec;
use amplify::{Bytes32, RawArray};
use baid58::ToBaid58;
use commit_verify::{strategies, CommitStrategy, CommitmentId};
use strict_encoding::{
    DecodeError, ReadTuple, StrictDecode, StrictEncode, StrictProduct, StrictTuple, StrictType,
    TypeName, TypedRead, TypedWrite, WriteTuple,
};
use strict_types::SemId;

use super::{
    ExtensionSchema, GenesisSchema, OwnedRightType, PublicRightType, StateSchema, TransitionSchema,
    ValidationScript,
};
use crate::LIB_NAME_RGB;

pub type FieldType = u16;
pub type ExtensionType = u16;
pub type TransitionType = u16;

/// Schema identifier.
///
/// Schema identifier commits to all of the schema data.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[wrapper(Deref, BorrowSlice, FromStr, Hex, Index, RangeOps)]
#[display(Self::to_baid58)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct SchemaId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl CommitStrategy for SchemaId {
    type Strategy = strategies::Strict;
}

impl ToBaid58<32> for SchemaId {
    const HRP: &'static str = "sch";
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_raw_array() }
}

#[derive(Clone, Eq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Schema {
    /// Feature flags control which of the available RGB features are allowed
    /// for smart contracts created under this schema.
    ///
    /// NB: This is not the same as RGB protocol versioning: feature flag set
    /// is specific to a particular RGB protocol version. The only currently
    /// defined RGB version is RGBv1; future versions may change the whole
    /// structure of Schema data, use of feature flags, re-define their meaning
    /// or do other backward-incompatible changes (RGB protocol versions are
    /// not interoperable and backward-incompatible by definitions and the
    /// nature of client-side-validation which does not allow upgrades).
    #[serde(skip)]
    pub rgb_features: SchemaFlags,
    pub subset_of: Option<SchemaId>,

    pub field_types: TinyOrdMap<FieldType, SemId>,
    pub owned_right_types: TinyOrdMap<OwnedRightType, StateSchema>,
    pub public_right_types: TinyOrdSet<PublicRightType>,
    pub genesis: GenesisSchema,
    pub extensions: TinyOrdMap<ExtensionType, ExtensionSchema>,
    pub transitions: TinyOrdMap<TransitionType, TransitionSchema>,

    /// Type system
    pub type_system: MediumVec<u8>, // TODO: TypeSystem,
    /// Validation code.
    pub script: ValidationScript,
}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool { self.schema_id() == other.schema_id() }
}

impl Ord for Schema {
    fn cmp(&self, other: &Self) -> Ordering { self.schema_id().cmp(&other.schema_id()) }
}

impl PartialOrd for Schema {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl CommitStrategy for Schema {
    type Strategy = strategies::Strict;
}

impl CommitmentId for Schema {
    const TAG: [u8; 32] = *b"urn:lnpbp:rgb:schema:v01#202302A";
    type Id = SchemaId;
}

impl Schema {
    #[inline]
    pub fn schema_id(&self) -> SchemaId { self.commitment_id() }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct SchemaFlags(FlagVec);

impl StrictType for SchemaFlags {
    const STRICT_LIB_NAME: &'static str = LIB_NAME_RGB;
    fn strict_name() -> Option<TypeName> { Some(tn!("SchemaFlags")) }
}
impl StrictProduct for SchemaFlags {}
impl StrictTuple for SchemaFlags {
    const FIELD_COUNT: u8 = 1;
}
impl StrictEncode for SchemaFlags {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> io::Result<W> {
        writer.write_tuple::<Self>(|w| Ok(w.write_field(&self.0.shrunk().into_inner())?.complete()))
    }
}
impl StrictDecode for SchemaFlags {
    fn strict_decode(reader: &mut impl TypedRead) -> Result<Self, DecodeError> {
        reader.read_tuple(|r| r.read_field().map(|vec| Self(FlagVec::from_inner(vec))))
    }
}

#[cfg(test)]
mod test {
    use strict_encoding::StrictDumb;

    use super::*;

    #[test]
    fn display() {
        let dumb = SchemaId::strict_dumb();
        assert_eq!(dumb.to_string(), "11111111111111111111111111111111");
        assert_eq!(
            &format!("{dumb::^#}"),
            "sch:11111111111111111111111111111111#dallas-liter-marco"
        );

        let less_dumb = SchemaId::from_raw_array(*b"EV4350-'4vwj'4;v-w94w'e'vFVVDhpq");
        assert_eq!(less_dumb.to_string(), "5ffNUkMTVSnWquPLT6xKb7VmAxUbw8CUNqCkUWsZfkwz");
        assert_eq!(
            &format!("{less_dumb::^#}"),
            "sch:5ffNUkMTVSnWquPLT6xKb7VmAxUbw8CUNqCkUWsZfkwz#hotel-urgent-child"
        );
    }
}
