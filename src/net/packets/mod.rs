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
///     Client MyPacket { num_field: u32, string: RLEString<u16> }
/// }
/// ```
///
/// Define a server packet but not an adapter (for packets which require
/// special behavior)
///
/// ```
/// define_single_packet! {
///     Server MyServerPacket(ManualAdapter) { num_field: u32, bools: RLE<u16, Vec<bool>> }
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
            AcceptTrade { my_offer: RLE<u16, Vec<bool>>, your_offer: RLE<u16, Vec<bool>> },
            ActivePetUpdateRequest { command_type: u8, instance_id: u32 },
            AoeAck { time: u32, pos: WorldPosData },
            Buy { object_id: u32, quantity: u32 },
            CancelTrade {},
            ChangeGuildRank { name: RLEString, guild_rank: u32 },
            ChangePetSkin { pet_id: u32, skin_type: u32, currency: u32 },
            ChangeTrade { offer: RLE<u16, Vec<bool>> },
            CheckCredits {},
            ChooseName { name: RLEString },
            Create { class_type: u16, skin_type: u16 },
            CreateGuild { name: RLEString },
            EditAccountList { account_list_id: u32, add: bool, object_id: u32 },
            EnemyHit { time: u32, bullet_id: u8, target_id: u32, kill: bool },
            EnterArena { currency: u32 },
            Escape {},
            GoToQuestRoom {},
            GotoAck { time: u32 },
            GroundDamage { time: u32, pos: WorldPosData },
            GuildInvite { name: RLEString },
            GuildRemove { name: RLEString },
            Hello {
                build_version: RLEString,
                game_id: u32,
                guid: RLEString,
                rand1: u32,
                password: RLEString,
                rand2: u32,
                secret: RLEString,
                key_time: u32,
                key: RLE<u16, Vec<u8>>,
                map_json: RLEString<u32>,
                entry_tag: RLEString,
                game_net: RLEString,
                game_net_user_id: RLEString,
                play_platform: RLEString,
                platform_token: RLEString,
                user_token: RLEString,
            },
            InvDrop { slot: SlotObjectData },
            InvSwap { time: u32, pos: WorldPosData, slot1: SlotObjectData, slot2: SlotObjectData },
            JoinGuild { guild_name: RLEString },
            KeyInfoRequest { item_type: u32 },
            Load { char_id: u32, from_arena: bool },
            Move {
                tick_id: u32,
                time: u32,
                new_pos: WorldPosData,
                records: RLE<u16, Vec<MoveRecord>>
            },
            OtherHit { time: u32, bullet_id: u8, object_id: u32, target_id: u32 },
            PetUpgradeRequest {
                pet_trans_type: u8,
                pid_one: u32,
                pid_two: u32,
                object_id: u32,
                payment_trans_type: u8,
                slots: RLE<u16, Vec<SlotObjectData>>
            },
            PlayerHit { bullet_id: u8, object_id: u32 },
            PlayerShoot {
                time: u32,
                bullet_id: u8,
                container_type: u16,
                starting_pos: WorldPosData,
                angle: f32
            },
            PlayerText { text: RLEString },
            QuestRedeem { quest_id: RLEString, item: u32, slots: RLE<u16, Vec<SlotObjectData>> },
            Pong { serial: u32, time: u32 },
            RequestTrade { name: RLEString },
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
                account_ids: RLE<u16,
                Vec<RLEString>>,
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
            BuyResult { result: u32, result_string: RLEString }, // TODO: consts for this?
            ClientStat { name: RLEString, value: u32 },
            CreateSuccess { object_id: u32, char_id: u32 },
            Damage {
                target_id: u32,
                effects: RLE<u8, Vec<u8>>,
                damage_amount: u16,
                kill: bool,
                armor_pierce: bool,
                bullet_id: u8,
                object_id: u32
            },
            Death {
                account_id: RLEString,
                char_id: u32,
                killed_by: RLEString,
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
            Failure { error_id: u32, error_description: RLEString }, // TODO: consts?
            File { filename: RLEString, file: RLEString<u32> }, // TODO: investigate this
            GlobalNotification { notification_type: u32, text: RLEString },
            Goto { object_id: u32, pos: WorldPosData },
            GuildResult { success: bool, line_builder_json: RLEString },
            HatchPetMessage { pet_name: RLEString, pet_skin: u32, item_type: u32 },
            InvResult { result: u32 },
            InvitedToGuild { name: RLEString, guild_name: RLEString },
            ImminentArenaWave { current_runtime: u32 },
            KeyInfoResponse { name: RLEString, description: RLEString, creator: RLEString },
            MapInfo { // TODO: double check this, maybe use manual adapter
                width: u32,
                height: u32,
                name: RLEString,
                display_name: RLEString,
                fp: u32,
                background: u32,
                difficulty: u32,
                allow_player_teleport: bool,
                show_displays: bool,
                client_xml: RLE<u16, Vec<RLEString<u32>>>,
                extra_xml: RLE<u16, Vec<RLEString<u32>>>
            },
            NameResult { success: bool, error_text: RLEString },
            NewAbilityMessage { typ: u32 },
            NewTick { tick_id: u32, tick_time: u32, statuses: RLE<u16, Vec<ObjectStatusData>> },
            Notification { object_id: u32, message: RLEString, color: u32 },
            PasswordPrompt { clean_password_status: u32 },
            PetYard { typ: u32 },
            Pic(ManualAdapter) { bitmap_data: Vec<u8> },
            Ping { serial: u32 },
            PlaySound { owner_id: u32, sound_id: u8 },
            QuestObjId { object_id: u32 },
            QuestRedeemResponse { ok: bool, message: RLEString },
            RealmHeroesResponse { number_of_realm_heroes: u32 },
            Reconnect {
                name: RLEString,
                host: RLEString,
                stats: RLEString,
                port: u32,
                game_id: u32,
                key_time: u32,
                is_from_arena: bool,
                key: RLE<u16, Vec<u8>>
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
                name: RLEString,
                object_id: u32,
                num_stars: u32,
                bubble_time: u8,
                recipient: RLEString,
                text: RLEString,
                clean_text: RLEString,
                is_supporter: bool
            },
            TradeAccepted { my_offer: RLE<u16, Vec<bool>>, your_offer: RLE<u16, Vec<bool>> },
            TradeChanged { offer: RLE<u16, Vec<bool>> },
            TradeDone { code: u32, description: RLEString }, // TODO: consts?
            TradeRequested { name: RLEString },
            TradeStart {
                my_items: RLE<u16, Vec<TradeItem>>,
                your_name: RLEString,
                your_items: RLE<u16, Vec<TradeItem>>
            },
            Update {
                tiles: RLE<u16, Vec<GroundTileData>>,
                new_objs: RLE<u16, Vec<ObjectData>>,
                drops: RLE<u16, Vec<u32>>
            },
            VerifyEmail {}
        }
    }
}
