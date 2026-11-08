use std::fmt::Display;
use std::sync::LazyLock;

use derive_more::From;
use thiserror::Error;
use twilight_mention::Mention;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::{Interaction, InteractionContextType};
use twilight_model::channel::message::component::{
    MediaGallery, MediaGalleryItem, Section, Separator, TextDisplay, Thumbnail, UnfurledMediaItem
};
use twilight_model::channel::message::{AllowedMentions, Component, MessageFlags};
use twilight_model::guild::Member;
use twilight_model::http::attachment::Attachment;
use twilight_model::id::Id;
use twilight_model::id::marker::{CommandMarker, GuildMarker, UserMarker};
use twilight_util::builder::command::{CommandBuilder, UserBuilder};

use super::COMMANDS_REGISTER;
use crate::assets::graphics::user_rank::BANNER;
use crate::discord::emojis::EMOJIS_REGISTER;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::defer;
use crate::ranks::{self, Badges, CalculateRanksError, Ranks, Stats, Times};
use crate::{APPLICATION_ID, STATE};

pub const NAME: &str = "ranks";
pub const DESCRIPTION: &str = "Show off your takedown stats in a rank profile.";
pub static ID: LazyLock<Id<CommandMarker>> = LazyLock::new(|| COMMANDS_REGISTER.ranks);
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::mention![command];

#[derive(Debug)]
pub struct RanksCommand {
    user_id: Option<Id<UserMarker>>
}

impl TryFrom<&CommandData> for RanksCommand {
    type Error = TryFromInteractionError;

    fn try_from(data: &CommandData) -> Result<Self, Self::Error> {
        let mut user: Option<Id<UserMarker>> = None;
        for option in &data.options {
            let CommandDataOption { name, value } = option;
            match (name.as_str(), value) {
                ("user", CommandOptionValue::User(u)) => user = Some(*u),
                ("user", _) => {
                    return Err(TryFromInteractionError::IncorrectFieldType {
                        command: NAME.into(),
                        field: name.clone(),
                        value: value.clone()
                    });
                }
                (_, _) => {
                    return Err(TryFromInteractionError::UnknownField {
                        command: NAME.into(),
                        field: name.clone()
                    });
                }
            }
        }

        Ok(RanksCommand { user_id: user })
    }
}

pub fn chat_input() -> Command {
    let option = UserBuilder::new("user", "User to show ranks for.").required(false).build();

    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .contexts([InteractionContextType::Guild])
        .option(option)
        .build()
}

#[derive(Debug, Error, From)]
pub enum RanksActionError {
    #[error("not in guild")]
    NotInGuild,

    #[error("author not found")]
    NoAuthor,

    #[error(transparent)]
    CalculateRanks(CalculateRanksError)
}

pub async fn action(
    interaction: &Interaction,
    command: RanksCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(RanksActionError::NotInGuild)?;
    let author_id = interaction.author_id().ok_or(RanksActionError::NoAuthor)?;
    defer::ephemeral(interaction).await?;

    let user_id = command.user_id.unwrap_or(author_id);
    let member = STATE.client.guild_member(guild_id, user_id).await?.model().await?;
    let stats = ranks::calculate_ranks(user_id, guild_id)
        .await
        .map_err(RanksActionError::CalculateRanks)?;
    let immunity = ranks::has_immunity(guild_id, &member.roles).await?;
    let badges = ranks::determine_badges(&stats, &immunity);
    let avatar = avatar_url(guild_id, user_id, &member);
    let components = profile_components(user_id, avatar, stats, badges);

    STATE
        .client
        .interaction(APPLICATION_ID)
        .update_response(interaction.token.as_str())
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .components(Some(&components))
        .attachments(&[Attachment {
            description: Some("CloakerBot rank profile banner".into()),
            file: BANNER.into(),
            filename: "rank_banner.webp".into(),
            id: 0
        }])
        .await?;

    Ok(())
}

fn avatar_url(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>, member: &Member) -> String {
    match (member.avatar, member.user.avatar) {
        (Some(hash), _) => format!(
            "https://cdn.discordapp.com/guilds/{guild_id}/users/{user_id}/avatars/{hash}.png?size=256"
        ),
        (None, Some(hash)) => {
            format!("https://cdn.discordapp.com/avatars/{user_id}/{hash}.png?size=256")
        }
        (None, None) => {
            format!("https://cdn.discordapp.com/embed/avatars/{}.png", (user_id.get() >> 22) % 6)
        }
    }
}

fn profile_components(
    user_id: Id<UserMarker>,
    avatar: String,
    stats: Stats,
    badges: Badges
) -> [Component; 8] {
    [
        banner_component(),
        header_component(user_id, avatar),
        divider(),
        stats_component("📍 This server", stats.here_ranks, stats.here_times),
        divider(),
        stats_component("🌐 Global", stats.global_ranks, stats.global_times),
        divider(),
        text(format!("## 🏅 Server badges\n{}", badge_labels(badges)))
    ]
}

fn banner_component() -> Component {
    Component::MediaGallery(MediaGallery {
        id: None,
        items: vec![MediaGalleryItem {
            media: media("attachment://rank_banner.webp"),
            description: Some("Cloaker in the shadows with glowing green goggles".into()),
            spoiler: None
        }]
    })
}

fn header_component(user_id: Id<UserMarker>, avatar: String) -> Component {
    Component::Section(Section {
        id: None,
        components: vec![
            text("-# CLOAKERBOT · RANK PROFILE"),
            text(format_args!("# {}", user_id.mention())),
        ],
        accessory: Box::new(Component::Thumbnail(Thumbnail {
            id: None,
            media: media(avatar),
            description: Some(Some("Member avatar".into())),
            spoiler: None
        }))
    })
}

fn media(url: impl Into<String>) -> UnfurledMediaItem {
    UnfurledMediaItem {
        url: url.into(),
        proxy_url: None,
        height: None,
        width: None,
        content_type: None
    }
}

fn text(content: impl Display) -> Component {
    Component::TextDisplay(TextDisplay { id: None, content: content.to_string() })
}

fn divider() -> Component {
    Component::Separator(Separator { id: None, divider: Some(true), spacing: None })
}

fn stats_component(title: &str, ranks: Ranks, times: Times) -> Component {
    text(format!(
        "## {title}\n**Victim** · Rank **#{}** · Smacked **{}** times\n**Punisher** · Rank \
         **#{}** · Called **{}** times\n**Counter** · Rank **#{}** · Countered **{}** times",
        ranks.victim_rank,
        times.times_smacked,
        ranks.punisher_rank,
        times.times_called,
        ranks.counter_rank,
        times.times_countered
    ))
}

fn badge_labels(badges: Badges) -> String {
    let labels = [
        (Badges::TOP_STRIKER, EMOJIS_REGISTER.top_striker, "Top striker"),
        (Badges::TOP_VICTIM, EMOJIS_REGISTER.top_victim, "Top victim"),
        (Badges::TOP_COUNTERER, EMOJIS_REGISTER.top_counterer, "Top counterer"),
        (Badges::NEVER_INITIATED, EMOJIS_REGISTER.never_initiated, "Never initiated"),
        (Badges::NEVER_TAKEN_DOWN, EMOJIS_REGISTER.never_taken_down, "Never taken down"),
        (Badges::IMMUNITY, EMOJIS_REGISTER.immunity, "General immunity"),
        (Badges::RANDOM_IMMUNITY, EMOJIS_REGISTER.random_immunity, "Random immunity")
    ];
    let labels: Vec<_> = labels
        .into_iter()
        .filter(|(badge, _, _)| badges.contains(*badge))
        .map(|(_, emoji, label)| format!("- {} {label}", emoji.mention()))
        .collect();
    if labels.is_empty() { "-# No badges yet.".into() } else { labels.join("\n") }
}
