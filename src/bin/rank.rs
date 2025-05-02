use std::{env, iter};

use hieroglyph::{DrawingContext, Segments};
use image::{Pixel, Rgba, RgbaImage};
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;

const GUILD_ID: Id<GuildMarker> = Id::new(581884294495207444);
const USER_ID: Id<UserMarker> = Id::new(137280289708834816);

struct Ranks {
    victim_rank: u32,
    punisher_rank: u32,
    counter_rank: u32
}

struct Counters {
    times_smacked: u32,
    times_called: u32,
    times_countered: u32
}

struct Stats {
    global_ranks: Ranks,
    here_ranks: Ranks,
    global_counters: Counters,
    here_counters: Counters
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq)]
    struct Badges: u8 {
        const TOP_STRIKER = 1 << 0;
        const TOP_VICTIM = 1 << 1;
        const NEVER_INITIATED = 1 << 2;
        const NEVER_TAKEN_DOWN = 1 << 3;
        const IMMUNITY = 1 << 4;
        const RANDOM_IMMUNITY = 1 << 5;
        const TOP_COUNTERER = 1 << 6;
    }
}

#[tokio::main]
async fn main() {
    let discord_token = env::var("DISCORD_TOKEN").expect("missing env variable DISCORD_TOKEN");
    let client = twilight_http::Client::new(discord_token);
    let member = client.guild_member(GUILD_ID, USER_ID).await.unwrap().model().await.unwrap();

    let member_avatar = member.avatar.unwrap();
    let member_avatar = format!("https://cdn.discordapp.com/guilds/{GUILD_ID}/users/{USER_ID}/avatars/{member_avatar}.png?size=256");
    let member_avatar = reqwest::get(member_avatar).await.unwrap().bytes().await.unwrap();
    let member_avatar = image::load_from_memory(&member_avatar).unwrap();
    let member_avatar = member_avatar.resize(207, 207, image::imageops::FilterType::Lanczos3);

    let user_avatar = member.user.avatar.unwrap();
    let user_avatar =
        format!("https://cdn.discordapp.com/avatars/{USER_ID}/{user_avatar}.png?size=64");
    let user_avatar = reqwest::get(user_avatar).await.unwrap().bytes().await.unwrap();
    let user_avatar = image::load_from_memory(&user_avatar).unwrap();
    let user_avatar = user_avatar.resize(35, 35, image::imageops::FilterType::Lanczos3);

    let nick = member.nick.unwrap();
    let nick = Segments::new(&nick);

    let name = member.user.name;
    let name = Segments::new(&name);

    const TEMPLATE: &[u8] = include_bytes!("../../assets/graphics/user_rank/template.png");
    let template = image::load_from_memory(&TEMPLATE).unwrap();

    let mut canvas = RgbaImage::new(template.width(), template.height());
    image::imageops::overlay(&mut canvas, &member_avatar, 26, 75);
    image::imageops::overlay(&mut canvas, &user_avatar, 42, 366);
    image::imageops::overlay(&mut canvas, &template, 0, 0);

    let mut ctx = DrawingContext::new();
    ctx.rgb(255, 255, 255);

    fn draw_cb(
        canvas: &mut RgbaImage,
        x_offset: i32,
        y_offset: i32
    ) -> impl FnMut((i32, i32), [u8; 4]) + use<'_> {
        move |(x, y), rgba| {
            let color = Rgba(rgba);
            let Ok(x) = ((x + x_offset) as i32).try_into()
            else {
                return;
            };
            let Ok(y) = ((y + y_offset) as i32).try_into()
            else {
                return;
            };
            let Some(pixel) = canvas.get_pixel_checked(x, y)
            else {
                return;
            };
            let mut pixel = pixel.clone();
            pixel.blend(&color);
            canvas.put_pixel(x, y, pixel);
        }
    }

    ctx.font_size(50.0);
    ctx.draw(&nick, draw_cb(&mut canvas, 45, 309));

    ctx.font_size(28.0);
    ctx.draw(&name, draw_cb(&mut canvas, 84, 373));

    ctx.font_size(20.0);
    let stats = Stats {
        global_ranks: Ranks { victim_rank: 42, punisher_rank: 17, counter_rank: 8 },
        here_ranks: Ranks { victim_rank: 5, punisher_rank: 2, counter_rank: 1 },
        global_counters: Counters { times_smacked: 120, times_called: 85, times_countered: 30 },
        here_counters: Counters { times_smacked: 15, times_called: 10, times_countered: 3 }
    };

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

    let counts = [
        (stats.global_counters.times_smacked, (347, 666)),
        (stats.here_counters.times_smacked, (494, 666)),
        (stats.global_counters.times_called, (347, 705)),
        (stats.here_counters.times_called, (494, 705)),
        (stats.global_counters.times_countered, (347, 743)),
        (stats.here_counters.times_countered, (494, 743))
    ];
    for (counts, (x, y)) in counts {
        let counts = counts.to_string();
        let counts = Segments::new(&counts);
        let width = ctx.width(&counts) as i32;
        ctx.draw(&counts, draw_cb(&mut canvas, x - width / 2, y));
    }

    let badges =
        Badges::TOP_STRIKER | Badges::NEVER_TAKEN_DOWN | Badges::IMMUNITY | Badges::TOP_COUNTERER;
    const BADGE_SIZE: u32 = 44;
    const FIRST_BADGE: (i64, i64) = (526, 228);

    // TODO: draw badges
    let blender_x_coords =
        iter::successors(Some(FIRST_BADGE.0 - (BADGE_SIZE / 2) as i64), |prev| {
            Some(prev - BADGE_SIZE as i64)
        });
    const BLENDER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/blender.png");
    let blender = image::load_from_memory(BLENDER).unwrap();
    let blender = blender.resize(BADGE_SIZE, BADGE_SIZE, image::imageops::FilterType::Triangle);
    for x in blender_x_coords.take(badges.bits().count_ones() as usize - 1) {
        image::imageops::overlay(&mut canvas, &blender, x, FIRST_BADGE.1);
    }

    let badge_x_coords =
        iter::successors(Some(FIRST_BADGE.0), |prev| Some(prev - BADGE_SIZE as i64));
    for (badge, x) in badges.iter().zip(badge_x_coords) {
        let badge = match badge {
            Badges::TOP_STRIKER => image::load_from_memory(include_bytes!(
                "../../assets/graphics/user_rank/badges/top_striker.png"
            ))
            .unwrap(),
            Badges::TOP_VICTIM => image::load_from_memory(include_bytes!(
                "../../assets/graphics/user_rank/badges/top_victim.png"
            ))
            .unwrap(),
            Badges::NEVER_INITIATED => image::load_from_memory(include_bytes!(
                "../../assets/graphics/user_rank/badges/never_initiated.png"
            ))
            .unwrap(),
            Badges::NEVER_TAKEN_DOWN => image::load_from_memory(include_bytes!(
                "../../assets/graphics/user_rank/badges/never_taken_down.png"
            ))
            .unwrap(),
            Badges::IMMUNITY => image::load_from_memory(include_bytes!(
                "../../assets/graphics/user_rank/badges/immunity.png"
            ))
            .unwrap(),
            Badges::RANDOM_IMMUNITY => image::load_from_memory(include_bytes!(
                "../../assets/graphics/user_rank/badges/random_immunity.png"
            ))
            .unwrap(),
            Badges::TOP_COUNTERER => image::load_from_memory(include_bytes!(
                "../../assets/graphics/user_rank/badges/top_counterer.png"
            ))
            .unwrap(),
            _ => unreachable!()
        };
        let badge = badge.resize(BADGE_SIZE, BADGE_SIZE, image::imageops::FilterType::Triangle);
        image::imageops::overlay(&mut canvas, &badge, x, FIRST_BADGE.1);
    }

    canvas.save("rank.png").unwrap();
}
