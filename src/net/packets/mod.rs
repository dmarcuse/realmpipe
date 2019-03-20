//! Types and adapters representing packets sent between the ROTMG client and
//! server

/// Define the structure of a packet
macro_rules! define_packet_structure {
    ($name:ident {
        $(
            $fieldname: ident : $fieldtype:ty
        ),* $(,)?
    }) => {
        #[derive(Debug, PartialEq, Clone)]
        #[allow(missing_docs)]
        pub struct $name {
            $(
                pub $fieldname: $fieldtype
            ),*
        }
    }
}

/// Define an adapter for a packet
macro_rules! define_packet_adapter {
    ($name: ident {
        $(
            $fieldname:ident : $fieldtype:ty
        ),* $(,)?
    }) => {
        #[allow(unused_variables)]
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
}

/// Define a single packet and optionally an adapter for it
///
/// # Examples
///
/// Define a client packet and an adapter
/// ```
/// define_single_packet! {
///     Client MyPacket { num_field: u32, string: RLE<String> }
/// }
/// ```
///
/// Define a server packet but not an adapter (for packets which require
/// special behavior)
///
/// ```
/// define_single_packet! {
///     Server MyServerPacket(ManualAdapter) { num_field: u32, bools: RLE<Vec<bool>> }
/// }
/// ```
macro_rules! define_single_packet {
    ($side:tt $name:ident (ManualAdapter) $fields:tt) => {
        define_packet_structure! { $name $fields }
    };
    ($side:tt $name:ident $fields:tt) => {
        define_single_packet! { $side $name (ManualAdapter) $fields }
        define_packet_adapter! { $name $fields }
    };
}

/// Define which packets belong to the client/server sides
macro_rules! define_side {
    (Client: $( $name:ident ),* $(,)? ) => {
        /// Packets that may be sent by the client
        pub mod client { $( pub use super::$name; )* }
    };
    (Server: $( $name:ident ),* $(,)? ) => {
        /// Packets that may be sent by the server
        pub mod server { $( pub use super::$name; )* }
    };
}

/// One macro to rule them all
macro_rules! define_packets {
    (
        $(
            $side:ident {
                $(
                    $name:ident $( ( $adapterspec:tt ) )? {
                        $(
                            $fieldname:ident: $fieldtype:ty
                        ),* $(,)?
                    }
                ),* $(,)?
            }
        ),* $(,)?
    ) => {
        // first define all the packet types and chosen adapters
        $( // each side...
            $( // each packet...
                define_single_packet! {
                    $side $name $( ( $adapterspec ) )* {
                        $( $fieldname : $fieldtype ),*
                    }
                }
            )*

            // also define modules for the sides
            define_side! { $side : $( $name ),*  }
        )*

        // next, define the all-powerful Packet enum
        /// A packet of any type from either the server or the client
        #[derive(Debug, PartialEq, Clone)]
        #[allow(missing_docs)]
        pub enum Packet {
            $( // each side
                $( // each packet
                    $name($name)
                ),*
            ),*
        }

        // define an enum for internal packet ids...
        /// A representation of packet types used internally
        #[repr(u8)]
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
        #[allow(missing_docs)]
        pub enum InternalPacketId {
            $( // each side
                $( // each packet
                    $name
                ),*
            ),*
        }

        // ...and then a method to get the internal id of an existing packet
        impl Packet {
            /// Get the internal ID for this packet
            pub fn get_internal_id(&self) -> InternalPacketId {
                match self {
                    $(
                        $(
                            Packet::$name(..) => InternalPacketId::$name
                        ),*
                    ),*
                }
            }
        }

        // next, downcast functionality, achieved with a trait...
        pub trait Downcast<T> {
            fn downcast(self) -> Option<T>;
        }

        // ...then impls for each type...
        $(
            $(
                impl Downcast<$name> for Packet {
                    fn downcast(self) -> Option<$name> {
                        match self {
                            Packet::$name(v) => Some(v),
                            _ => None
                        }
                    }
                }

                impl<'a> Downcast<&'a $name> for &'a Packet {
                    fn downcast(self) -> Option<&'a $name> {
                        match self {
                            Packet::$name(v) => Some(v),
                            _ => None
                        }
                    }
                }
            )*
        )*

        // ...and finally methods on `Packet`.
        impl Packet {
            /// Attempt to downcast this packet to a specific type, consuming the packet.
            /// See `downcast_ref` for an example.
            pub fn downcast<T>(self) -> Option<T> where Self: Downcast<T> {
                Downcast::downcast(self)
            }

            /// Attempt to downcast this packet to a specific type by reference.
            ///
            /// # Example
            ///
            /// ```
            /// use realmpipe::net::packets::{Packet, client};
            ///
            /// // create a wrapped packet
            /// let pkt = Packet::CancelTrade(client::CancelTrade {});
            ///
            /// // downcast it to its original type (will return Some(T) on success)
            /// assert_eq!(pkt.downcast_ref::<client::CancelTrade>(), Some(&client::CancelTrade {}));
            ///
            /// // downcast it to a different type (will return None on failure)
            /// assert_eq!(pkt.downcast_ref::<client::AcceptTrade>(), None);
            /// ```
            pub fn downcast_ref<'a, T>(&'a self) -> Option<&'a T> where &'a Self: Downcast<&'a T> {
                Downcast::downcast(self)
            }
        }
    };
}

// re-export the packets and other types (defined below)
pub use self::unified_definitions::client;
pub use self::unified_definitions::server;
pub use self::unified_definitions::InternalPacketId;
pub use self::unified_definitions::Packet;

/// Unified set of all packet definitions
mod unified_definitions {
    use crate::net::adapters::prelude::*;
    use crate::net::data::*;

    define_packets! {
        Client {
            AcceptTrade { my_offer: RLE<Vec<bool>>, your_offer: RLE<Vec<bool>> },
            ActivePetUpdateRequest { command_type: u8, instance_id: u32 },
            AoeAck { time: u32, pos: WorldPosData },
            Buy { object_id: u32, quantity: u32 },
            CancelTrade {},
            ChangeGuildRank { name: RLE<String>, guild_rank: u32 },
            ChangePetSkin { pet_id: u32, skin_type: u32, currency: u32 },
            ChangeTrade { offer: RLE<Vec<bool>> },
            CheckCredits {},
            ChooseName { name: RLE<String> },
            Create { class_type: u16, skin_type: u16 },
            CreateGuild { name: RLE<String> },
            EditAccountList { account_list_id: u32, add: bool, object_id: u32 },
            EnemyHit { time: u32, bullet_id: u8, target_id: u32, kill: bool },
            EnterArena { currency: u32 },
            Escape {},
            GoToQuestRoom {},
            GotoAck { time: u32 },
            GroundDamage { time: u32, pos: WorldPosData },
            GuildInvite { name: RLE<String> },
            GuildRemove { name: RLE<String> },
            Hello {
                build_version: RLE<String>,
                game_id: u32,
                guid: RLE<String>,
                rand1: u32,
                password: RLE<String>,
                rand2: u32,
                secret: RLE<String>,
                key_time: u32,
                key: RLE<Vec<u8>>,
                map_json: RLE<String, u32>,
                entry_tag: RLE<String>,
                game_net: RLE<String>,
                game_net_user_id: RLE<String>,
                play_platform: RLE<String>,
                platform_token: RLE<String>,
                user_token: RLE<String>,
            },
            InvDrop { slot: SlotObjectData },
            InvSwap { time: u32, pos: WorldPosData, slot1: SlotObjectData, slot2: SlotObjectData },
            JoinGuild { guild_name: RLE<String> },
            KeyInfoRequest { item_type: u32 },
            Load { char_id: u32, from_arena: bool },
            Move {
                tick_id: u32,
                time: u32,
                new_pos: WorldPosData,
                records: RLE<Vec<MoveRecord>>
            },
            OtherHit { time: u32, bullet_id: u8, object_id: u32, target_id: u32 },
            PetUpgradeRequest {
                pet_trans_type: u8,
                pid_one: u32,
                pid_two: u32,
                object_id: u32,
                payment_trans_type: u8,
                slots: RLE<Vec<SlotObjectData>>
            },
            PlayerHit { bullet_id: u8, object_id: u32 },
            PlayerShoot {
                time: u32,
                bullet_id: u8,
                container_type: u16,
                starting_pos: WorldPosData,
                angle: f32
            },
            PlayerText { text: RLE<String> },
            QuestRedeem { quest_id: RLE<String>, item: u32, slots: RLE<Vec<SlotObjectData>> },
            Pong { serial: u32, time: u32 },
            RequestTrade { name: RLE<String> },
            ResetDailyQuests {},
            Reskin { skin_id: u32 },
            ReskinPet { instance_id: u32, picked_new_pet_type: u32, item: SlotObjectData },
            SetCondition { effect: u8, duration: f32 },
            ShootAck { time: u32 },
            SquareHit { time: u32, bullet_id: u8, object_id: u32 },
            Teleport { object_id: u32 },
            UseItem { time: u32, slot: SlotObjectData, pos: WorldPosData, use_type: u32 },
            UsePortal { object_id: u32 }
        },
        Server {
            AccountList {
                account_list_id: u32,
                account_ids: RLE<Vec<RLE<String>>>,
                lock_action: u32
            },
            ActivePet { instance_id: u32 },
            AllyShoot { bullet_id: u8, owner_id: u32, container_type: u16, angle: f32 },
            Aoe {
                pos: WorldPosData,
                radius: f32,
                damage: u16,
                effect: u8,
                duration: f32,
                orig_type: u16,
                color: u32,
                armor_pierce: bool
            },
            ArenaDeath { cost: u32 },
            BuyResult { result: u32, result_string: RLE<String> }, // TODO: consts for this?
            ClientStat { name: RLE<String>, value: u32 },
            CreateSuccess { object_id: u32, char_id: u32 },
            Damage {
                target_id: u32,
                effects: RLE<Vec<u8>, u8>,
                damage_amount: u16,
                kill: bool,
                armor_pierce: bool,
                bullet_id: u8,
                object_id: u32
            },
            Death {
                account_id: RLE<String>,
                char_id: u32,
                killed_by: RLE<String>,
                zombie_type: u32,
                zombie_id: u32,
            },
            DeletePetMessage { pet_id: u32 },
            EnemyShoot {
                bullet_id: u8,
                owner_id: u32,
                bullet_type: u8,
                starting_pos: WorldPosData,
                angle: f32,
                damage: u16,
                num_shots: Option<u8>,
                angle_inc: Option<f32>
            },
            EvolvedPetMessage { pet_id: u32, initial_skin: u32, final_skin: u32 },
            Failure { error_id: u32, error_description: RLE<String> }, // TODO: consts?
            File { filename: RLE<String>, file: RLE<String, u32> }, // TODO: investigate this
            GlobalNotification { notification_type: u32, text: RLE<String> },
            Goto { object_id: u32, pos: WorldPosData },
            GuildResult { success: bool, line_builder_json: RLE<String> },
            HatchPetMessage { pet_name: RLE<String>, pet_skin: u32, item_type: u32 },
            InvResult { result: u32 },
            InvitedToGuild { name: RLE<String>, guild_name: RLE<String> },
            ImminentArenaWave { current_runtime: u32 },
            KeyInfoResponse { name: RLE<String>, description: RLE<String>, creator: RLE<String> },
            MapInfo { // TODO: double check this, maybe use manual adapter
                width: u32,
                height: u32,
                name: RLE<String>,
                display_name: RLE<String>,
                fp: u32,
                background: u32,
                difficulty: u32,
                allow_player_teleport: bool,
                show_displays: bool,
                client_xml: RLE<Vec<RLE<String, u32>>>,
                extra_xml: RLE<Vec<RLE<String, u32>>>
            },
            NameResult { success: bool, error_text: RLE<String> },
            NewAbilityMessage { typ: u32 },
            NewTick { tick_id: u32, tick_time: u32, statuses: RLE<Vec<ObjectStatusData>> },
            Notification { object_id: u32, message: RLE<String>, color: u32 },
            PasswordPrompt { clean_password_status: u32 },
            PetYard { typ: u32 },
            Pic(ManualAdapter) { bitmap_data: Vec<u8> },
            Ping { serial: u32 },
            PlaySound { owner_id: u32, sound_id: u8 },
            QuestObjId { object_id: u32 },
            QuestRedeemResponse { ok: bool, message: RLE<String> },
            RealmHeroesResponse { number_of_realm_heroes: u32 },
            Reconnect {
                name: RLE<String>,
                host: RLE<String>,
                stats: RLE<String>,
                port: u32,
                game_id: u32,
                key_time: u32,
                is_from_arena: bool,
                key: RLE<Vec<u8>>
            },
            ReskinUnlock { skin_id: u32, is_pet_skin: u32 },
            ServerPlayerShoot {
                bullet_id: u8,
                owner_id: u32,
                container_type: u32,
                starting_pos: WorldPosData,
                angle: f32,
                damage: u16
            },
            ShowEffect { // TODO: consts?
                effect_type: u8,
                target_object_id: u32,
                pos1: WorldPosData,
                pos2: WorldPosData,
                color: u32,
                duration: f32
            },
            Text {
                name: RLE<String>,
                object_id: u32,
                num_stars: u32,
                bubble_time: u8,
                recipient: RLE<String>,
                text: RLE<String>,
                clean_text: RLE<String>,
                is_supporter: bool
            },
            TradeAccepted { my_offer: RLE<Vec<bool>>, your_offer: RLE<Vec<bool>> },
            TradeChanged { offer: RLE<Vec<bool>> },
            TradeDone { code: u32, description: RLE<String> }, // TODO: consts?
            TradeRequested { name: RLE<String> },
            TradeStart {
                my_items: RLE<Vec<TradeItem>>,
                your_name: RLE<String>,
                your_items: RLE<Vec<TradeItem>>
            },
            Update {
                tiles: RLE<Vec<GroundTileData>>,
                new_objs: RLE<Vec<ObjectData>>,
                drops: RLE<Vec<u32>>
            },
            VerifyEmail {}
        }
    }
}
