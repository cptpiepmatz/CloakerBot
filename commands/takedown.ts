import {
  ApplicationCommandData,
  ContextMenuInteraction,
  Guild,
  GuildMember,
  GuildMemberRoleManager,
  Permissions
} from "discord.js";

import constructCommand from "../functions/util/constructCommand";
import takedown from "../functions/takedown";
import replies from "../provider/replies";
import getRandom from "../functions/util/getRandom";
import GuildCommandInteraction from "../types/GuildCommandInteraction";
import ownerId from "../provider/ownerId";
import readDatabase from "../functions/database/readDatabase";
import DatabaseTables from "../enums/DatabaseTables";
import RolesDBData from "../types/database/RolesDBData";
import Logger from "js-logger";
import memberString from "../functions/util/memberString";
import guildString from "../functions/util/guildString";

/**
 * Command for taking down users.
 * @param interaction
 */
async function takedownCommand(interaction: GuildCommandInteraction) {
  if (!await commandTakedownPermCheck(interaction)) return;
  let member = interaction.options.get("user")?.member;
  if (member instanceof GuildMember) {
    await tryTakedownMember(member, interaction);
  }
  else {
    await interaction.reply({
      content: getRandom(replies.noMember),
      ephemeral: true
    });
  }
}

export async function commandTakedownPermCheck(
  interaction: GuildCommandInteraction | ContextMenuInteraction
): Promise<boolean> {
  let commander = interaction.member;
  let dbData = await readDatabase(DatabaseTables.ROLES, interaction.guild as Guild);
  let roleId = (dbData as RolesDBData)?.takedownRole ?? "";
  if (!commander) return false;
  if (!(
    commander.user.id === ownerId ||
    commander.user.id === (interaction.guild as Guild).ownerId ||
    (commander.permissions as Permissions).has("ADMINISTRATOR") ||
    (commander.permissions as Permissions).has("MOVE_MEMBERS") ||
    (commander.roles as GuildMemberRoleManager).resolve(roleId)
  )) {
    await interaction.reply({
      content: getRandom(replies.noPerms),
      ephemeral: true
    });
    return false;
  }
  return true;
}

export async function tryTakedownMember(
  member: GuildMember,
  interaction: GuildCommandInteraction | ContextMenuInteraction
) {
  let commander = interaction.member;
  if (member.id === client.user!.id) {
    await interaction.reply({
      ephemeral: true,
      content: getRandom(replies.commandTakedownSelf)
    });
    Logger.info(
      memberString(interaction.member as GuildMember) +
      " tried to takedown the bot itself on " +
      guildString(interaction.guild as Guild)
    );
    return;
  }
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

const commandData: ApplicationCommandData = {
  name: "cloaker",
  description: "Take down your friends",
  defaultPermission: true,
  options: [{
    name: "user",
    type: "USER",
    description: "User to take down",
    required: true
  }]
};

export default constructCommand(takedownCommand, commandData);
