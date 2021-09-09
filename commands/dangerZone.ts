import GuildCommandInteraction from "../types/GuildCommandInteraction";
import {ApplicationCommandData, ApplicationCommandPermissionData, Guild, VoiceChannel} from "discord.js";
import allowGuildOwner from "./permissions/allowGuildOwner";
import allowAdmins from "./permissions/allowAdmins";
import allowBotOwner from "./permissions/allowBotOwner";
import constructCommand from "../functions/util/constructCommand";
import Logger from "js-logger";

async function dangerZoneCommand(interaction: GuildCommandInteraction) {
  await interaction.deferReply({ephemeral: true});
  Logger.trace("deferred reply");

  const channel = interaction.options.getChannel("channel");
  if (channel) {
    if (channel.type === "GUILD_VOICE") {
      Logger.debug(`selected ${channel.name} as danger zone on ${channel.guild.name}`);
      await dangerZoneManager.putDangerZone(channel as VoiceChannel);
      await interaction.editReply(`Set ${channel} as danger zone.`);
    }
    else {
      Logger.debug(`selected channel ${channel.name} is no voice channel on ${interaction.guild.name}`);
      await interaction.editReply(`${channel} is no voice channel.`);
    }
    return;
  }
  const currentDangerZone = dangerZoneManager.getDangerZone(interaction.guild.id);
  if (currentDangerZone) {
    const dangerZoneChannel = await interaction.guild.channels.fetch(currentDangerZone);
    if (dangerZoneChannel && !dangerZoneChannel.deleted) {
      Logger.debug(`replied with current danger zone ${dangerZoneChannel.name} on ${interaction.guild.name}`);
      await interaction.editReply(`Currently ${dangerZoneChannel} is set as danger zone.`);
      return;
    }
  }
  await interaction.editReply("Currently no danger zone is set.");
}

const commandData: ApplicationCommandData = {
  name: "dangerZone".toLowerCase(),
  description: "Defines a channel as a danger zone. " +
    "User joining this channel will automatically be taken down.",
  defaultPermission: false,
  options: [
    {
      name: "channel",
      type: "CHANNEL",
      description: "Channel to set as danger zone."
    }
  ]
}

async function getPermissions(
  guild: Guild
): Promise<ApplicationCommandPermissionData[]> {
  return [
    allowGuildOwner(guild),
    allowBotOwner(guild),
    allowAdmins(guild)
  ].flat();
}

export default constructCommand(dangerZoneCommand, commandData, getPermissions);
