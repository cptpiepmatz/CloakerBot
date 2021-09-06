import {ApplicationCommandPermissionData, Guild} from "discord.js";
import readDatabase from "../../functions/database/readDatabase";
import DatabaseTables from "../../enums/DatabaseTables";
import RolesDBData from "../../types/database/RolesDBData";

export default async function allowCloakerRole(
  guild: Guild
): Promise<ApplicationCommandPermissionData | null> {
  const rolesDBData =
    await readDatabase(DatabaseTables.ROLES, guild) as RolesDBData;
  if (!rolesDBData) return null;
  const roleId = rolesDBData.takedownRole;
  if (!roleId) return null;
  return {
    id: roleId,
    type: "ROLE",
    permission: true
  }
}
