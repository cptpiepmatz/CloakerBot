use twilight_model::application::command::{Command, CommandType};
use twilight_util::builder::command::{
    BooleanBuilder, CommandBuilder, NumberBuilder, SubCommandBuilder
};

pub const NAME: &str = "config";
const DESCRIPTION: &str = "Configure the CloakerBot.";

pub fn chat_input() -> Command {
    let log_channel = SubCommandBuilder::new("logchannel", "Define your log channel.").build();
    let danger_zones = SubCommandBuilder::new("dangerzones", "Define your danger zones.").build();
    let counter = SubCommandBuilder::new("counter", "Configure countering.")
        .option(BooleanBuilder::new("enable", "Enable/disable countering.").build())
        .build();
    let random = SubCommandBuilder::new("random", "Configure random takedown behavior.")
        .option(BooleanBuilder::new("enable", "Enable/Disable random takedowns.").build())
        .option(
            NumberBuilder::new(
                "min",
                "Minimum time between takedowns. `-1` to set default value."
            )
            .build()
        )
        .option(
            NumberBuilder::new(
                "max",
                "Maximum time between takedowns. `-1` to set default value."
            )
            .build()
        )
        .build();

    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .option(log_channel)
        .option(danger_zones)
        .option(counter)
        .option(random)
        .build()
}
