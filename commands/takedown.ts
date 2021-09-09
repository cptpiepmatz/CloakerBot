import {
  ApplicationCommandData,
  ApplicationCommandPermissionData,
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
import allowGuildOwner from "./permissions/allowGuildOwner";
import allowAdmins from "./permissions/allowAdmins";
import allowBotOwner from "./permissions/allowBotOwner";
import allowRolesWithPermission from "./permissions/allowRolesWithPermission";
import allowCloakerRole from "./permissions/allowCloakerRole";
import ownerId from "../provider/ownerId";
import readDatabase from "../functions/database/readDatabase";
import DatabaseTables from "../enums/DatabaseTables";
import RolesDBData from "../types/database/RolesDBData";

/**
 * Command for taking down users.
 * @param interaction
 */
async function takedownCommand(interaction: GuildCommandInteraction) {
  let commander = interaction.member;
  let dbData = await readDatabase(DatabaseTables.ROLES, interaction.guild);
  let roleId = (dbData as RolesDBData)?.takedownRole ?? "";
  if (!commander) return;
  if (!(
    commander.user.id === ownerId ||
    commander.user.id === interaction.guild.ownerId ||
    (commander.permissions as Permissions).has("ADMINISTRATOR") ||
    (commander.permissions as Permissions).has("MOVE_MEMBERS") ||
    (commander.roles as GuildMemberRoleManager).resolve(roleId)
  )) {
    await interaction.reply({
      content: getRandom(replies.noPerms),
      ephemeral: true
    });
    return;
  }
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
  defaultPermission: true,
  options: [{
    name: "user",
    type: "USER",
    description: "User to take down",
    required: true
  }]
};

export default constructCommand(takedownCommand, commandData);
