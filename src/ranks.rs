use std::iter;
use std::ops::Deref;
use std::sync::LazyLock;

use bytes::{BufMut, Bytes, BytesMut};
use hieroglyph::{DrawingContext, Segments};
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageError, Pixel, Rgba, RgbaImage};
use thiserror::Error;
use twilight_http::error::Error as TwilightHttpError;
use twilight_http::response::DeserializeBodyError;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};

use crate::STATE;
use crate::assets::graphics::user_rank::{self, badges};
use crate::panic::ExpectReport;
use crate::util::iterator_ext::IteratorExt;
use crate::util::require;

macro_rules! lazy_images {
    ($(static $name:ident = ($path:path) $(.resize($resize:ident))? .expect($expect:literal);)*) => {
        $(static $name: LazyLock<DynamicImage> = LazyLock::new(|| {
            let bytes: &[u8] = $path;
            let img = image::load_from_memory(bytes).expect_report($expect);
            $(let img = img.resize($resize, $resize, FilterType::Lanczos3);)?
            img
        });)*
    }
}

lazy_images! {
    static TEMPLATE = (user_rank::TEMPLATE).expect("valid template png");
    static BLENDER = (badges::BLENDER).resize(BADGE_SIZE).expect("valid blender png");

    static TOP_STRIKER = (badges::TOP_STRIKER).resize(BADGE_SIZE).expect("valid badge png");
    static TOP_VICTIM = (badges::TOP_VICTIM).resize(BADGE_SIZE).expect("valid badge png");
    static NEVER_INITIATED = (badges::NEVER_INITIATED).resize(BADGE_SIZE).expect("valid badge png");
    static NEVER_TAKEN_DOWN = (badges::NEVER_TAKEN_DOWN).resize(BADGE_SIZE).expect("valid badge png");
    static IMMUNITY = (badges::IMMUNITY).resize(BADGE_SIZE).expect("valid badge png");
    static RANDOM_IMMUNITY = (badges::RANDOM_IMMUNITY).resize(BADGE_SIZE).expect("valid badge png");
    static TOP_COUNTERER = (badges::TOP_COUNTERER).resize(BADGE_SIZE).expect("valid badge png");
}

static DRAWING_CONTEXT: LazyLock<tokio::sync::Mutex<DrawingContext>> = LazyLock::new(|| {
    let mut ctx = DrawingContext::new();
    ctx.rgb(255, 255, 255);
    tokio::sync::Mutex::new(ctx)
});

const BADGE_SIZE: u32 = 44;
const FIRST_BADGE: (i64, i64) = (526, 228);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Ranks {
    pub victim_rank: u32,
    pub punisher_rank: u32,
    pub counter_rank: u32
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Times {
    pub times_smacked: u32,
    pub times_called: u32,
    pub times_countered: u32
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Stats {
    pub global_ranks: Ranks,
    pub here_ranks: Ranks,
    pub global_times: Times,
    pub here_times: Times
}

bitflags::bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Badges: u8 {
        const TOP_STRIKER      = 1 << 0;
        const TOP_VICTIM       = 1 << 1;
        const NEVER_INITIATED  = 1 << 2;
        const NEVER_TAKEN_DOWN = 1 << 3;
        const IMMUNITY         = 1 << 4;
        const RANDOM_IMMUNITY  = 1 << 5;
        const TOP_COUNTERER    = 1 << 6;
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum CreateRanksGraphicError {
    Database(#[from] sqlx::Error),
    Http(#[from] TwilightHttpError),
    DeserializeBody(#[from] DeserializeBodyError),
    Reqwest(#[from] reqwest::Error),
    Image(#[from] ImageError),
    Join(#[from] tokio::task::JoinError)
}

#[tracing::instrument(skip_all, fields(%user_id, %guild_id))]
pub async fn create_ranks_graphic(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>
) -> Result<Bytes, CreateRanksGraphicError> {
    let member_data = fetch_member_data(user_id, guild_id).await?;
    let stats = calculate_ranks(user_id, guild_id).await?;
    let immunity = has_immunity(guild_id, &member_data.roles).await?;
    let badges = determine_badges(&stats, &immunity);

    let span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        span.in_scope(|| render_ranks_graphic(member_data, stats, badges))
    })
    .await?
}

fn render_ranks_graphic(
    member_data: MemberData,
    stats: Stats,
    badges: Badges
) -> Result<Bytes, CreateRanksGraphicError> {
    let mut canvas = RgbaImage::new(TEMPLATE.width(), TEMPLATE.height());
    image::imageops::overlay(&mut canvas, &member_data.big_avatar, 26, 75);
    image::imageops::overlay(&mut canvas, &member_data.small_avatar, 42, 366);
    image::imageops::overlay(&mut canvas, &*TEMPLATE, 0, 0);

    let blender_x_coords =
        iter::successors(Some(FIRST_BADGE.0 - (BADGE_SIZE / 2) as i64), |prev| {
            Some(prev - BADGE_SIZE as i64)
        });
    for x in blender_x_coords.take((badges.bits().count_ones() as usize).saturating_sub(1)) {
        image::imageops::overlay(&mut canvas, BLENDER.deref(), x, FIRST_BADGE.1);
    }

    let badge_x_coords =
        iter::successors(Some(FIRST_BADGE.0), |prev| Some(prev - BADGE_SIZE as i64));
    for (badge, x) in badges.iter().zip(badge_x_coords) {
        let badge = match badge {
            Badges::TOP_STRIKER => TOP_STRIKER.deref(),
            Badges::TOP_VICTIM => TOP_VICTIM.deref(),
            Badges::NEVER_INITIATED => NEVER_INITIATED.deref(),
            Badges::NEVER_TAKEN_DOWN => NEVER_TAKEN_DOWN.deref(),
            Badges::IMMUNITY => IMMUNITY.deref(),
            Badges::RANDOM_IMMUNITY => RANDOM_IMMUNITY.deref(),
            Badges::TOP_COUNTERER => TOP_COUNTERER.deref(),
            _ => unreachable!()
        };
        image::imageops::overlay(&mut canvas, badge, x, FIRST_BADGE.1);
    }

    let mut ctx = DRAWING_CONTEXT.blocking_lock();

    fn draw_cb(
        canvas: &mut RgbaImage,
        x_offset: i32,
        y_offset: i32
    ) -> impl FnMut((i32, i32), [u8; 4]) + use<'_> {
        move |(x, y), rgba| {
            let color = Rgba(rgba);
            let x = require!(Ok(((x + x_offset) as i32).try_into()));
            let y = require!(Ok(((y + y_offset) as i32).try_into()));
            let pixel = require!(Some(canvas.get_pixel_checked(x, y)));
            let mut pixel = pixel.clone();
            pixel.blend(&color);
            canvas.put_pixel(x, y, pixel);
        }
    }

    ctx.font_size(50.0);
    let nick = Segments::new(&member_data.nick);
    ctx.draw(&nick, draw_cb(&mut canvas, 45, 309));

    ctx.font_size(28.0);
    let name = Segments::new(&member_data.name);
    ctx.draw(&name, draw_cb(&mut canvas, 84, 373));

    ctx.font_size(20.0);
    let ranks = [
        (stats.global_ranks.victim_rank, (347, 525)),
        (stats.here_ranks.victim_rank, (494, 525)),
        (stats.global_ranks.punisher_rank, (347, 564)),
        (stats.here_ranks.punisher_rank, (494, 564)),
        (stats.global_ranks.counter_rank, (347, 602)),
        (stats.here_ranks.counter_rank, (494, 602))
    ];
    for (rank, (x, y)) in ranks {
        let rank = format!("#{rank}");
        let rank = Segments::new(&rank);
        let width = ctx.width(&rank) as i32;
        ctx.draw(&rank, draw_cb(&mut canvas, x - width / 2, y));
    }

    let times = [
        (stats.global_times.times_smacked, (347, 666)),
        (stats.here_times.times_smacked, (494, 666)),
        (stats.global_times.times_called, (347, 705)),
        (stats.here_times.times_called, (494, 705)),
        (stats.global_times.times_countered, (347, 743)),
        (stats.here_times.times_countered, (494, 743))
    ];
    for (times, (x, y)) in times {
        let times = times.to_string();
        let times = Segments::new(&times);
        let width = ctx.width(&times) as i32;
        ctx.draw(&times, draw_cb(&mut canvas, x - width / 2, y));
    }

    drop(ctx);

    let bytes = BytesMut::new();
    let mut writer = bytes.writer();
    let encoder = PngEncoder::new(&mut writer);
    encoder.write_image(&canvas, canvas.width(), canvas.height(), ExtendedColorType::Rgba8)?;

    Ok(Bytes::from(writer.into_inner()))
}

struct MemberData {
    big_avatar: DynamicImage,
    small_avatar: DynamicImage,
    nick: String,
    name: String,
    roles: Vec<Id<RoleMarker>>
}

async fn fetch_member_data(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>
) -> Result<MemberData, CreateRanksGraphicError> {
    let member = STATE.client.guild_member(guild_id, user_id).await?.model().await?;

    let http_client = reqwest::Client::new();

    let big_avatar = match (member.avatar, member.user.avatar) {
        (Some(avatar), _) => format!(
            "https://cdn.discordapp.com/guilds/{guild_id}/users/{user_id}/avatars/{avatar}.png?size=256"
        ),
        (None, Some(avatar)) => {
            format!("https://cdn.discordapp.com/avatars/{user_id}/{avatar}.png?size=256")
        }
        (None, None) => {
            format!("https://cdn.discordapp.com/embed/avatars/{}.png", (user_id.get() >> 22) % 6)
        }
    };
    let big_avatar = http_client.get(big_avatar).send().await?.bytes().await?;

    let small_avatar = match member.user.avatar {
        Some(avatar) => {
            format!("https://cdn.discordapp.com/avatars/{user_id}/{avatar}.png?size=256")
        }
        None => {
            format!("https://cdn.discordapp.com/embed/avatars/{}.png", (user_id.get() >> 22) % 6)
        }
    };
    let small_avatar = http_client.get(small_avatar).send().await?.bytes().await?;
    let span = tracing::Span::current();
    let (big_avatar, small_avatar) = tokio::task::spawn_blocking(move || {
        span.in_scope(|| {
            let big_avatar = image::load_from_memory(&big_avatar)?;
            let big_avatar = big_avatar.resize(207, 207, FilterType::Lanczos3);
            let small_avatar = image::load_from_memory(&small_avatar)?;
            let small_avatar = small_avatar.resize(35, 35, FilterType::Lanczos3);
            Ok::<_, ImageError>((big_avatar, small_avatar))
        })
    })
    .await??;

    let nick =
        member.nick.or(member.user.global_name).unwrap_or_else(|| member.user.name.to_string());
    let name = member.user.name;

    let roles = member.roles.clone();

    Ok(MemberData { big_avatar, small_avatar, nick, name, roles })
}

async fn calculate_ranks(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>
) -> Result<Stats, CreateRanksGraphicError> {
    let stats = STATE.database.stats().await?;

    // cpu-heavy rank calculations on a separate thread
    // keep the tokio worker free for other tasks
    let span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        span.in_scope(|| {
            let is_user = |(_, (uid, _)): &(u32, (Id<UserMarker>, u32))| *uid == user_id;
            type RankEntry = (u32, (Id<UserMarker>, u32));
            let match_rank = |rank: (Option<RankEntry>, Option<RankEntry>)| match rank {
                (None, None) => (1, 0),
                (None, Some((last_rank, _))) => (last_rank + 1, 0),
                (Some((rank, (_, count))), _) => (rank, count)
            };

            let global = {
                let takedowns = &stats.takedowns;

                let mut victim_ranks = takedowns.victim_ranks();
                let victim_rank = victim_ranks.find_and_last(is_user);
                let (victim_rank, times_smacked) = match_rank(victim_rank);

                let mut punisher_ranks = takedowns.punisher_ranks();
                let punisher_rank = punisher_ranks.find_and_last(is_user);
                let (punisher_rank, times_called) = match_rank(punisher_rank);

                let mut counter_ranks = takedowns.counter_ranks();
                let counter_rank = counter_ranks.find_and_last(is_user);
                let (counter_rank, times_countered) = match_rank(counter_rank);

                let ranks = Ranks { victim_rank, punisher_rank, counter_rank };
                let times = Times { times_smacked, times_called, times_countered };
                (ranks, times)
            };

            let here = match stats.takedowns.get(&guild_id) {
                None => {
                    let ranks = Ranks { victim_rank: 1, punisher_rank: 1, counter_rank: 1 };
                    let times = Times { times_smacked: 0, times_called: 0, times_countered: 0 };
                    (ranks, times)
                }
                Some(takedowns) => {
                    let mut victim_ranks = takedowns.victim_ranks();
                    let victim_rank = victim_ranks.find_and_last(is_user);
                    let (victim_rank, times_smacked) = match_rank(victim_rank);

                    let mut punisher_ranks = takedowns.punisher_ranks();
                    let punisher_rank = punisher_ranks.find_and_last(is_user);
                    let (punisher_rank, times_called) = match_rank(punisher_rank);

                    let mut counter_ranks = takedowns.counter_ranks();
                    let counter_rank = counter_ranks.find_and_last(is_user);
                    let (counter_rank, times_countered) = match_rank(counter_rank);

                    let ranks = Ranks { victim_rank, punisher_rank, counter_rank };
                    let times = Times { times_smacked, times_called, times_countered };
                    (ranks, times)
                }
            };

            Stats {
                global_ranks: global.0,
                here_ranks: here.0,
                global_times: global.1,
                here_times: here.1
            }
        })
    })
    .await
    .map_err(Into::into)
}

struct Immunity {
    general: bool,
    random: bool
}

async fn has_immunity(
    guild_id: Id<GuildMarker>,
    user_roles: &[Id<RoleMarker>]
) -> Result<Immunity, sqlx::Error> {
    let guild_config = STATE.database.guild_config(guild_id).await?;

    let contains_any = |immunity_roles: &[Id<RoleMarker>]| {
        user_roles.into_iter().any(|user_role| immunity_roles.contains(user_role))
    };

    let immunity = &guild_config.immunity;
    let general_immunity = contains_any(&immunity.general);
    let general_silent_immunity = contains_any(&immunity.general_silent);
    let random_immunity = contains_any(&immunity.random);
    let random_silent_immunity = contains_any(&immunity.random_silent);

    Ok(Immunity {
        general: general_immunity || general_silent_immunity,
        random: random_immunity || random_silent_immunity
    })
}

fn determine_badges(stats: &Stats, immunity: &Immunity) -> Badges {
    let mut badges = Badges::empty();

    if stats.here_ranks.punisher_rank == 1 && stats.here_times.times_called > 0 {
        badges.insert(Badges::TOP_STRIKER);
    }

    if stats.here_ranks.victim_rank == 1 && stats.here_times.times_smacked > 0 {
        badges.insert(Badges::TOP_VICTIM);
    }

    if stats.here_times.times_called == 0 {
        badges.insert(Badges::NEVER_INITIATED);
    }

    if stats.here_times.times_smacked == 0 {
        badges.insert(Badges::NEVER_TAKEN_DOWN);
    }

    if immunity.general {
        badges.insert(Badges::IMMUNITY);
    }

    if immunity.random {
        badges.insert(Badges::RANDOM_IMMUNITY);
    }

    if stats.here_ranks.counter_rank == 1 && stats.here_times.times_countered > 0 {
        badges.insert(Badges::TOP_COUNTERER);
    }

    badges
}
