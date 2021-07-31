import {ApplicationCommandData, CommandInteraction, Snowflake} from "discord.js";

import writeDatabase from "../functions/database/writeDatabase";
import DatabaseTables from "../enums/DatabaseTables";
import constructCommand from "../functions/util/constructCommand";
import readDatabase from "../functions/database/readDatabase";
import RolesDBData from "../types/database/RolesDBData";

const collectionName = DatabaseTables.ROLES;

/** Command for setting a role that is allowed to do takedowns. */
async function setRoleCommand(interaction: CommandInteraction): Promise<void> {
  if (!interaction.guild) return;
  await interaction.defer();

  const role = interaction.options.getRole("role");
  if (role) {
    await writeDatabase(collectionName, interaction.guild, {
      takedownRole: role.id
    });
    await interaction.editReply(
      `*You set the role ${role} to takedown people.*\n` +
      "Well, you somehow did it."
    );
    return;
  }

  const dbData =
    await readDatabase(collectionName, interaction.guild) as RolesDBData;
  let resolvedRole = dbData
    ? await interaction.guild.roles.resolve(dbData.takedownRole as Snowflake)
    : null;
  if (!resolvedRole) {
    await interaction.editReply(
      "*Currently no role is set as takedown role.*"
    );
    return;
  }
  await interaction.editReply(
    `*Currently the role ${resolvedRole} is allowed to execute takedowns.*`
  );
}

const commandData: ApplicationCommandData = {
  name: "takedownrole",
  description: "Shows the currently used role as takedown role",
  options: [
    {
      name: "role",
      type: "ROLE",
      description: "Sets a new role that is allowed to execute takedowns"
    }
  ]
}

export default constructCommand(setRoleCommand, commandData);
