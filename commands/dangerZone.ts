import GuildCommandInteraction from "../types/GuildCommandInteraction";
import {ApplicationCommandData, ApplicationCommandPermissionData, Guild, VoiceChannel} from "discord.js";
import allowGuildOwner from "./permissions/allowGuildOwner";
import allowAdmins from "./permissions/allowAdmins";
import allowBotOwner from "./permissions/allowBotOwner";
import constructCommand from "../functions/util/constructCommand";

async function dangerZoneCommand(interaction: GuildCommandInteraction) {
  await interaction.deferReply({ephemeral: true});
  const channel = interaction.options.getChannel("channel");
  if (channel) {
    if (channel.type === "GUILD_VOICE") {
      await dangerZoneManager.putDangerZone(channel as VoiceChannel);
      await interaction.editReply(`Set ${channel} as danger zone.`);
    }
    else {
      await interaction.editReply(`${channel} is no voice channel.`);
    }
    return;
  }
  const currentDangerZone = dangerZoneManager.getDangerZone(interaction.guild.id);
  if (currentDangerZone) {
    const dangerZoneChannel = await interaction.guild.channels.fetch(currentDangerZone);
    if (dangerZoneChannel && !dangerZoneChannel.deleted) {
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
    allowAdmins(guild),
    allowBotOwner(guild)
  ].flat();
}

export default constructCommand(dangerZoneCommand, commandData, getPermissions);
