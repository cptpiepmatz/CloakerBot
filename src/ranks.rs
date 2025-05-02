use std::sync::LazyLock;

use hieroglyph::{DrawingContext, Segments};
use image::imageops::FilterType;
use image::{DynamicImage, ImageError, Pixel, Rgba, RgbaImage};
use thiserror::Error;
use twilight_http::error::Error as TwilightHttpError;
use twilight_http::response::DeserializeBodyError;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::util::require;
use crate::STATE;

macro_rules! static_images {
    ($(static $name:ident = $path:literal $(* $resize:ident)? ? $expect:literal;)*) => {
        $(static $name: LazyLock<DynamicImage> = LazyLock::new(|| {
            const BYTES: &[u8] = include_bytes!(concat!("../assets/graphics/user_rank/", $path));
            let img = image::load_from_memory(BYTES).expect($expect);
            $(let img = img.resize($resize, $resize, FilterType::Lanczos3);)?
            img
        });)+
    };
}

static_images! {
    static TEMPLATE = "template.png" ? "valid template png";
    static BLENDER = "badges/blender.png" * BADGE_SIZE ? "valid blender png";

    static TOP_STRIKER = "badges/top_striker.png" * BADGE_SIZE ? "valid badge png";
    static TOP_VICTIM = "badges/top_victim.png" * BADGE_SIZE ? "valid badge png";
    static NEVER_INITIATED = "badges/never_initiated.png" * BADGE_SIZE ? "valid badge png";
    static NEVER_TAKEN_DOWN = "badges/never_taken_down.png" * BADGE_SIZE ? "valid badge png";
    static IMMUNITY = "badges/immunity.png" * BADGE_SIZE ? "valid badge png";
    static RANDOM_IMMUNITY = "badges/random_immunity.png" * BADGE_SIZE ? "valid badge png";
    static TOP_COUNTERER = "badges/top_counterer.png" * BADGE_SIZE ? "valid badge png";
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
pub struct Counters {
    pub times_smacked: u32,
    pub times_called: u32,
    pub times_countered: u32
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Stats {
    pub global_ranks: Ranks,
    pub here_ranks: Ranks,
    pub global_counters: Counters,
    pub here_counters: Counters
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
    Http(#[from] TwilightHttpError),
    DeserializeBody(#[from] DeserializeBodyError),
    Reqwest(#[from] reqwest::Error),
    Image(#[from] ImageError)
}

pub async fn create_ranks_graphic(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>
) -> Result<RgbaImage, CreateRanksGraphicError> {
    let member_data = fetch_member_data(user_id, guild_id).await?;
    let stats = calculate_ranks(user_id, guild_id);
    let badges = determine_badges();

    let mut canvas = RgbaImage::new(TEMPLATE.width(), TEMPLATE.height());
    image::imageops::overlay(&mut canvas, &member_data.big_avatar, 26, 75);
    image::imageops::overlay(&mut canvas, &member_data.small_avatar, 42, 366);
    image::imageops::overlay(&mut canvas, &*TEMPLATE, 0, 0);

    let mut ctx = DRAWING_CONTEXT.lock().await;

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
        (stats.global_counters.times_smacked, (347, 666)),
        (stats.here_counters.times_smacked, (494, 666)),
        (stats.global_counters.times_called, (347, 705)),
        (stats.here_counters.times_called, (494, 705)),
        (stats.global_counters.times_countered, (347, 743)),
        (stats.here_counters.times_countered, (494, 743))
    ];
    for (times, (x, y)) in times {
        let times = times.to_string();
        let times = Segments::new(&times);
        let width = ctx.width(&times) as i32;
        ctx.draw(&times, draw_cb(&mut canvas, x - width / 2, y));
    }

    todo!()
}

struct MemberData {
    big_avatar: DynamicImage,
    small_avatar: DynamicImage,
    nick: String,
    name: String
}

async fn fetch_member_data(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>
) -> Result<MemberData, CreateRanksGraphicError> {
    let member = STATE.client.guild_member(guild_id, user_id).await?.model().await?;

    let http_client = reqwest::Client::new();

    let big_avatar = match (member.avatar, member.user.avatar) {
        (Some(avatar), _) => format!("https://cdn.discordapp.com/guilds/{guild_id}/users/{user_id}/avatars/{avatar}.png?size=256"),
        (None, Some(avatar)) => format!("https://cdn.discordapp.com/avatars/{user_id}/{avatar}.png?size=256"),
        (None, None) => format!("https://cdn.discordapp.com/embed/avatars/{}.png", (user_id.get() >> 22) % 6),
    };
    let big_avatar = http_client.get(big_avatar).send().await?.bytes().await?;
    let big_avatar = image::load_from_memory(&big_avatar)?;

    let small_avatar = match member.user.avatar {
        Some(avatar) => {
            format!("https://cdn.discordapp.com/avatars/{user_id}/{avatar}.png?size=256")
        }
        None => {
            format!("https://cdn.discordapp.com/embed/avatars/{}.png", (user_id.get() >> 22) % 6)
        }
    };
    let small_avatar = http_client.get(small_avatar).send().await?.bytes().await?;
    let small_avatar = image::load_from_memory(&small_avatar)?;

    let nick =
        member.nick.and(member.user.global_name).unwrap_or_else(|| member.user.name.to_string());
    let name = member.user.name;

    Ok(MemberData { big_avatar, small_avatar, nick, name })
}

fn calculate_ranks(user_id: Id<UserMarker>, guild_id: Id<GuildMarker>) -> Stats {
    todo!()
}

fn determine_badges() -> Badges {
    todo!()
}
