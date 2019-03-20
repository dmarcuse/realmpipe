//! Definitions and adapters for basic ROTMG data types

#![allow(missing_docs)]

use super::stat::StatData;
use crate::net::adapters::prelude::*;

macro_rules! auto_data {
    ($name:ident {
        $(
            $fieldname:ident: $fieldtype:ty
        ),* $(,)?
    }) => {
        #[derive(Debug, PartialEq, Clone)]
        pub struct $name {
            $(
                pub $fieldname: $fieldtype
            ),*
        }

        impl NetworkAdapter for $name {
            fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
                $( let $fieldname = NetworkAdapter::get_be(bytes)?; )*

                Ok(Self { $( $fieldname ),* })
            }

            fn put_be(self, bytes: &mut dyn BufMut) -> Result<()> {
                let Self { $( $fieldname ),* } = self;

                $( $fieldname.put_be(bytes)?; )*

                Ok(())
            }
        }
    };

    ($(
        $name: ident {
            $(
                $fieldname:ident: $fieldtype:ty
            ),* $(,)?
        }
    ),* $(,)?) => {
        $(auto_data! { $name { $($fieldname: $fieldtype),* } })*
    }
}

auto_data! {
    GroundTileData { x: u16, y: u16, tile: u16 },
    MoveRecord { time: u32, x: f32, y: f32 },
    ObjectData { object_type: u16, status: ObjectStatusData },
    ObjectStatusData { object_id: u32, pos: WorldPosData, stats: RLE<u16, Vec<StatData>> },
    SlotObjectData { object_id: u32, slot_id: u8, object_type: u32 },
    TradeItem { item: u32, slot_type: u32, tradeable: bool, included: bool },
    WorldPosData { x: f32, y: f32 }
}
