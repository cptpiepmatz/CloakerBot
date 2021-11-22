import {ApplicationCommandPermissionData, Guild} from "discord.js";
import allowRolesWithPermission from "./allowRolesWithPermission";

export default function allowAdmins(
  guild: Guild
): ApplicationCommandPermissionData[] {
  return allowRolesWithPermission(guild, "ADMINISTRATOR");
}
