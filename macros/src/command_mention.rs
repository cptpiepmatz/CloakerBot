#[macro_export]
macro_rules! command_mention {
    (command) => {{
        use ::twilight_mention::Mention;
        ::std::sync::LazyLock::new(|| {
            ::twilight_mention::fmt::CommandMention::Command { id: *ID, name: NAME.to_string() }
                .mention()
        })
    }};

    (subcommand) => {{
        use ::twilight_mention::Mention;
        ::std::sync::LazyLock::new(|| {
            ::twilight_mention::fmt::CommandMention::SubCommand {
                id: super::super::COMMANDS_REGISTER.config,
                name: super::NAME.to_string(),
                sub_command: NAME.to_string()
            }
            .mention()
        })
    }};
}
