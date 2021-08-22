import {ApplicationCommandData, ApplicationCommandPermissionData, Guild, GuildMember} from "discord.js";

import constructCommand from "../functions/util/constructCommand";
import takedown from "../functions/takedown";
import replies from "../provider/replies";
import getRandom from "../functions/util/getRandom";
import GuildCommandInteraction from "../types/GuildCommandInteraction";
import allowGuildOwner from "./permissions/allowGuildOwner";
import allowAdmins from "./permissions/allowAdmins";
import allowBotOwner from "./permissions/allowBotOwner";

/**
 * Command for taking down users.
 * @param interaction
 */
async function takedownCommand(interaction: GuildCommandInteraction) {
  let commander = interaction.member;
  let member = interaction.options.get("user")?.member;
  if (member instanceof GuildMember) {
    await interaction.deferReply({ephemeral: true});
    let result = await takedown(member, commander as GuildMember | null);
    if (result.success) {
      await interaction.editReply(getRandom(replies.takedownSuccessful));
    }
    else {
      let response = result.message ?? getRandom(replies.failedTakedown);
      await interaction.editReply(response);
    }
  }
  else {
    await interaction.reply({
      content: getRandom(replies.noMember),
      ephemeral: true
    });
  }
}

const commandData: ApplicationCommandData = {
  name: "cloaker",
  description: "Take down your friends",
  defaultPermission: false,
  options: [{
    name: "user",
    type: "USER",
    description: "User to take down",
    required: true
  }]
};

async function getPermissions(
  guild: Guild
): Promise<ApplicationCommandPermissionData[]> {
  return [
    allowGuildOwner(guild),
    allowAdmins(guild),
    allowBotOwner(guild)
  ].flat();
}

export default constructCommand(takedownCommand, commandData);
